mod utils;

use async_trait::async_trait;
use gcloud_sdk::google::{api::{metric_descriptor, metric_descriptor::MetricKind, LabelDescriptor, MetricDescriptor}, monitoring::v3::{metric_service_client::MetricServiceClient, CreateTimeSeriesRequest, TimeSeries}};
use gcp_auth::TokenProvider;
use opentelemetry::{global, metrics::{MetricsError, Result}};
// use opentelemetry_proto::tonic::{collector::metrics::v1::ExportMetricsServiceRequest, metrics::v1::metric::Data as TonicMetricData};
use opentelemetry_sdk::metrics::{
    data::{Metric as OpentelemetrySdkMetric, ResourceMetrics},
    exporter::PushMetricsExporter,
    reader::{AggregationSelector, DefaultAggregationSelector, TemporalitySelector},
    InstrumentKind,
};

use opentelemetry_sdk::metrics::data::{
    self, Aggregation, Exemplar as SdkExemplar, ExponentialHistogram as SdkExponentialHistogram, Gauge as SdkGauge, Histogram as SdkHistogram, Metric as SdkMetric, ScopeMetrics as SdkScopeMetrics, Sum as SdkSum, Temporality
};
use opentelemetry_sdk::Resource as SdkResource;

use tonic::{service::interceptor::InterceptedService, transport::{Channel, ClientTlsConfig}};

use utils::{get_data_points_attributes_keys, normalize_label_key};

use core::time;
use std::{collections::HashSet, fmt::{Debug, Formatter}, sync::Arc};

use crate::{gcloud_sdk, gcp_authorizer::Authorizer};

const UNIQUE_IDENTIFIER_KEY: &str = "opentelemetry_id";
pub struct GCPMetricsExporter<'a, A: Authorizer> {
    prefix: String,
    unique_identifier: bool,
    authorizer: A,
    scopes: &'a [&'a str],
}

impl<'a, A: Authorizer> GCPMetricsExporter<'a, A> {
    pub fn new(authorizer: A) -> Self {
        let scopes = vec!["https://www.googleapis.com/auth/cloud-platform".to_string()];
        Self { 
            prefix: "workload.googleapis.com".to_string(), 
            unique_identifier: false, 
            authorizer, 
            scopes: &["https://www.googleapis.com/auth/cloud-platform"],
        }
    }
}

impl <'a, A: Authorizer> TemporalitySelector for GCPMetricsExporter<'a, A> {
    // This is matching OTLP exporters delta.
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        match kind {
            InstrumentKind::Counter
            | InstrumentKind::ObservableCounter
            | InstrumentKind::ObservableGauge
            | InstrumentKind::Histogram
            | InstrumentKind::Gauge => Temporality::Delta,
            InstrumentKind::UpDownCounter | InstrumentKind::ObservableUpDownCounter => {
                Temporality::Cumulative
            }
        }
    }
}

impl <'a, A: Authorizer> AggregationSelector for GCPMetricsExporter<'a, A> {
    // TODO: this should ideally be done at SDK level by default
    // without exporters having to do it.
    fn aggregation(&self, kind: InstrumentKind) -> opentelemetry_sdk::metrics::Aggregation {
        DefaultAggregationSelector::new().aggregation(kind)
    }
}

impl <'a, A: Authorizer> Debug for GCPMetricsExporter<'a, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Google monitoring metrics exporter")
    }
}


impl <'a, A: Authorizer> GCPMetricsExporter<'a, A> {

    /// We can map Metric to MetricDescriptor using Metric.name or
    /// MetricDescriptor.type. We create the MetricDescriptor if it doesn't
    /// exist already and cache it. Note that recreating MetricDescriptors is
    /// a no-op if it already exists.
    /// 
    /// :param record:
    /// :return:
    async fn get_metric_descriptor(&self, metric: &OpentelemetrySdkMetric) -> Option<MetricDescriptor> {
        let descriptor_type = format!("{}/{}",self.prefix, metric.name);
        // if descriptor_type in self._metric_descriptors:
        //     return self._metric_descriptors[descriptor_type]
        let mut descriptor = MetricDescriptor {
            r#type: descriptor_type,
            display_name: metric.name.to_string(),
            description: metric.description.to_string(),
            unit: metric.unit.to_string(),
            ..Default::default()
        };
        let seen_keys: HashSet<String> = get_data_points_attributes_keys(metric.data.as_any());
        // let metric_data: Option<TonicMetricData> = metric.data.as_any().try_into().ok();
        for key in &seen_keys {
            descriptor.labels.push(LabelDescriptor {
                key: normalize_label_key(key),
                ..Default::default()
            });
        }

        if self.unique_identifier {
            descriptor.labels.push(LabelDescriptor {
                key: UNIQUE_IDENTIFIER_KEY.to_string(),
                ..Default::default()
            });
        }
        let data = metric.data.as_any();
        if let Some(v) = data.downcast_ref::<SdkHistogram<i64>>() {
            descriptor.metric_kind = MetricKind::Cumulative.into();
            descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
        } else if let Some(v) = data.downcast_ref::<SdkHistogram<u64>>() {
            descriptor.metric_kind = MetricKind::Cumulative.into();
            descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
        } else if let Some(v) = data.downcast_ref::<SdkHistogram<f64>>() {
            descriptor.metric_kind = MetricKind::Cumulative.into();
            descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
        } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<i64>>() {
            descriptor.metric_kind = MetricKind::Cumulative.into();
            descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
        } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<u64>>() {
            descriptor.metric_kind = MetricKind::Cumulative.into();
            descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
        } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<f64>>() {
            descriptor.metric_kind = MetricKind::Cumulative.into();
            descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
        } else if let Some(v) = data.downcast_ref::<SdkSum<u64>>() {
            descriptor.metric_kind = if v.is_monotonic {
                MetricKind::Cumulative.into()
            } else {
                MetricKind::Gauge.into()
            };
            descriptor.value_type = metric_descriptor::ValueType::Int64.into();
        } else if let Some(v) = data.downcast_ref::<SdkSum<i64>>() {
            descriptor.metric_kind = if v.is_monotonic {
                MetricKind::Cumulative.into()
            } else {
                MetricKind::Gauge.into()
            };
            descriptor.value_type = metric_descriptor::ValueType::Int64.into();
        } else if let Some(v) = data.downcast_ref::<SdkSum<f64>>() {
            descriptor.metric_kind = if v.is_monotonic {
                MetricKind::Cumulative.into()
            } else {
                MetricKind::Gauge.into()
            };
            descriptor.value_type = metric_descriptor::ValueType::Double.into();
        } else if let Some(v) = data.downcast_ref::<SdkGauge<u64>>() {
            descriptor.metric_kind = MetricKind::Gauge.into();
            descriptor.value_type = metric_descriptor::ValueType::Int64.into();
        } else if let Some(v) = data.downcast_ref::<SdkGauge<i64>>() {
            descriptor.metric_kind = MetricKind::Gauge.into();
            descriptor.value_type = metric_descriptor::ValueType::Int64.into();
        } else if let Some(v) = data.downcast_ref::<SdkGauge<f64>>() {
            descriptor.metric_kind = MetricKind::Gauge.into();
            descriptor.value_type = metric_descriptor::ValueType::Double.into();
        } else {
            global::handle_error(MetricsError::Other("Unsupported metric data type, ignoring it".into()));
            // warning!("Unsupported metric data type, ignoring it");
            return None;
        }

            // let _ = gcp_monitoring_client
            // .get()
            // .create_metric_descriptor(tonic::Request::new(gcloud_sdk::google::monitoring::v3::CreateMetricDescriptorRequest {
            //     name: format!("projects/{}", project_id),
            //     metric_descriptor: metrics.get_metric_descriptor(),
            // })).await.unwrap();




        //     first_point = data.data_points[0] if len(data.data_points) else None
        //     if isinstance(first_point, NumberDataPoint):
        //         descriptor.value_type = (
        //             MetricDescriptor.ValueType.INT64
        //             if isinstance(first_point.value, int)
        //             else MetricDescriptor.ValueType.DOUBLE
        //         )
        //     elif isinstance(first_point, HistogramDataPoint):
        //         descriptor.value_type = MetricDescriptor.ValueType.DISTRIBUTION
        //     elif first_point is None:
        //         pass
        //     else:
        //         # Exhaustive check
        //         _ = first_point
        //         logger.warning(
        //             "Unsupported metric value type %s, ignoring it",
        //             type(first_point).__name__,
        //         )

        //     try:
        //         response_descriptor = self.client.create_metric_descriptor(
        //             CreateMetricDescriptorRequest(
        //                 name=self.project_name, metric_descriptor=descriptor
        //             )
        //         )
        //     # pylint: disable=broad-except
        //     except Exception as ex:
        //         logger.error(
        //             "Failed to create metric descriptor %s",
        //             descriptor,
        //             exc_info=ex,
        //         )
        //         return None
        //     self._metric_descriptors[descriptor_type] = response_descriptor
        //     return descriptor
        unimplemented!()
    }
}



#[async_trait]
impl <A: Authorizer> PushMetricsExporter for GCPMetricsExporter<'static, A> {
    async fn export(&self, metrics: &mut ResourceMetrics) -> Result<()> {
        
        println!("export: {:#?}", metrics);
        // let proto_message: ExportMetricsServiceRequest = (&*metrics).into();
        // println!("export: {}", serde_json::to_string_pretty(&proto_message).unwrap());


        use std::io::Write;
        let mut file = std::fs::File::create("metrics.txt").unwrap();
        file.write_all(format!("{:#?}", metrics).as_bytes()).unwrap();



        let provider: Arc<dyn TokenProvider> = gcp_auth::provider().await.unwrap();
        // let gcp_monitoring_client = GoogleApi::from_function(
        //     MetricServiceClient::new,
        //     "https://monitoring.googleapis.com",
        //     // cloud resource prefix: used only for some of the APIs (such as Firestore)
        //     None,
        // ).await.unwrap();
        let channel = Channel::from_static("https://monitoring.googleapis.com")
        .tls_config(ClientTlsConfig::new()).unwrap()
        .connect().await.unwrap();
        
        let mut msc = MetricServiceClient::new(channel);

        let mut req = tonic::Request::new(gcloud_sdk::google::monitoring::v3::GetMetricDescriptorRequest {
            name: "projects/".to_string(),
            // metric_descriptor: metrics.get_metric_descriptor(),
        });
        self.authorizer.authorize(&mut req, &self.scopes).await.unwrap();
        msc.get_metric_descriptor(req).await.unwrap();

        let project_id = provider.project_id().await.unwrap().to_string();
        

        // let mut timeSeries: Vec<TimeSeries> = Vec::new();

        // let _ = gcp_monitoring_client
        //     .get()
        //     .get_metric_descriptor(tonic::Request::new(gcloud_sdk::google::monitoring::v3::GetMetricDescriptorRequest {
        //         name: format!("projects/{}", project_id),
        //         metric_descriptor: metrics.get_metric_descriptor(),
        //     })).await.unwrap();
        
        // let _ = gcp_monitoring_client
        //     .get()
        //     .create_metric_descriptor(tonic::Request::new(gcloud_sdk::google::monitoring::v3::CreateMetricDescriptorRequest {
        //         name: format!("projects/{}", project_id),
        //         metric_descriptor: metrics.get_metric_descriptor(),
        //     })).await.unwrap();

        // for scope_metrics in &metrics.scope_metrics {
        //     for metric in &scope_metrics.metrics {
        //         let time_series = transform_metric(metric, &project_id);
        //         timeSeries.push(time_series);
        //     }
        // }

        // let _ = gcp_monitoring_client
        //     .get()          
        //     .create_time_series(tonic::Request::new(CreateTimeSeriesRequest {
        //         name: format!("projects/{}", project_id),
        //         time_series: transform(metrics, &project_id),
        //     })).await.unwrap();
        // 

        
        Ok(())
    }

    async fn force_flush(&self) -> Result<()> {
        Ok(()) // In this implementation, flush does nothing
    }

    fn shutdown(&self) -> Result<()> {
        // TracepointState automatically unregisters when dropped
        // https://github.com/microsoft/LinuxTracepoints-Rust/blob/main/eventheader/src/native.rs#L618
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_normalize_label_key() {
        let channel = Channel::from_static("https://monitoring.googleapis.com")
        .tls_config(ClientTlsConfig::new()).unwrap()
        .connect().await.unwrap();
    }
}
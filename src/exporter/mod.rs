mod data_point_to_time_series;
mod histogram_data_point_to_time_series;
mod to_f64;
mod utils;
use crate::{
    gcloud_sdk,
    gcp_authorizer::{Authorizer, FakeAuthorizer, GoogleEnvironment},
};
use async_trait::async_trait;
use gcloud_sdk::google::{
    api::{metric_descriptor, metric_descriptor::MetricKind, LabelDescriptor, MetricDescriptor},
    monitoring::v3::{
        metric_service_client::MetricServiceClient, CreateTimeSeriesRequest, TimeSeries,
    },
};
use itertools::Itertools;
use opentelemetry_resourcedetector_gcp_rust::mapping::get_monitored_resource;

use opentelemetry_sdk::{
    error::OTelSdkError,
    metrics::{
        data::{
            ExponentialHistogram as SdkExponentialHistogram, Gauge as SdkGauge,
            Histogram as SdkHistogram, Metric as OpentelemetrySdkMetric, ResourceMetrics,
            Sum as SdkSum,
        },
        exporter::PushMetricExporter as PushMetricsExporter,
        MetricError as MetricsError, Temporality,
    },
};

use rand::Rng;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    sync::Arc,
    time::{Duration, SystemTime},
};
#[cfg(feature = "tokio")]
use tokio::{sync::RwLock, time::sleep};
use tonic::{metadata::MetadataValue, transport::Channel};
use utils::{get_data_points_attributes_keys, normalize_label_key};

pub(crate) const UNIQUE_IDENTIFIER_KEY: &str = "opentelemetry_id";

pub type AuthorizerType = Arc<dyn Authorizer + Send + Sync>;

/// Implementation of Metrics Exporter to Google Cloud Monitoring.
pub struct GCPMetricsExporter {
    prefix: String,
    project_id: Option<String>,
    add_unique_identifier: bool,
    unique_identifier: String,
    authorizer: AuthorizerType,
    is_test_env: bool,
    metric_descriptors: Arc<RwLock<HashMap<String, MetricDescriptor>>>,
    custom_monitored_resource_data: Option<MonitoredResourceDataConfig>,
}

/// Configuration for the GCP metrics exporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GCPMetricsExporterConfig {
    /// prefix: the prefix of the metric. It is "workload.googleapis.com" by
    ///     default if not specified.
    pub prefix: String,
    /// project id of your Google Cloud project. It is get from GcpAuthorizer by default.
    pub project_id: Option<String>,
    /// add_unique_identifier: Add an identifier to each exporter metric. This
    ///     must be used when there exist two (or more) exporters that may
    ///     export to the same metric name within WRITE_INTERVAL seconds of
    ///     each other.
    pub add_unique_identifier: bool,
    /// custom_monitored_resource_data: Custom monitored resource data to be
    pub custom_monitored_resource_data: Option<MonitoredResourceDataConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Custom monitored resource data
/// need to resolve error 'INVALID_ARGUMENT: One or more TimeSeries could not be written'
/// if we use it we ignore our gcp resource detector and use this data for creating monitored resource
/// https://cloud.google.com/monitoring/api/resources#tag_global
pub struct MonitoredResourceDataConfig {
    pub r#type: String,
    pub labels: HashMap<String, String>,
}

impl Default for GCPMetricsExporterConfig {
    fn default() -> Self {
        Self {
            prefix: "workload.googleapis.com".to_string(),
            project_id: None,
            add_unique_identifier: false,
            custom_monitored_resource_data: None,
        }
    }
}

impl GCPMetricsExporter {
    pub fn new(authorizer: AuthorizerType, config: GCPMetricsExporterConfig) -> Self {
        let my_rundom = format!("{:08x}", rand::rng().random_range(0..u32::MAX));
        Self {
            prefix: config.prefix,
            add_unique_identifier: config.add_unique_identifier,
            project_id: config.project_id,
            unique_identifier: my_rundom,
            authorizer,
            is_test_env: cfg!(test),
            metric_descriptors: Arc::new(RwLock::new(HashMap::new())),
            custom_monitored_resource_data: config.custom_monitored_resource_data,
        }
    }

    pub async fn make_chanel(&self) -> Result<Channel, crate::error::Error> {
        if self.is_test_env {
            Channel::from_static("http://localhost:50051")
                .connect_timeout(Duration::from_secs(30))
                .tcp_keepalive(Some(Duration::from_secs(60)))
                .keep_alive_timeout(Duration::from_secs(60))
                .http2_keep_alive_interval(Duration::from_secs(60))
                .connect()
                .await
                .map_err(|e| crate::error::ErrorKind::Other(e.to_string()).into())
        } else {
            GoogleEnvironment::init_google_services_channel("https://monitoring.googleapis.com")
                .await
        }
    }
}

#[cfg(feature = "gcp_auth")]
impl GCPMetricsExporter {
    pub async fn new_gcp_auth(
        config: GCPMetricsExporterConfig,
    ) -> Result<GCPMetricsExporter, gcp_auth::Error> {
        let auth = crate::gcp_auth_authorizer::GcpAuth::new().await?;
        Ok(GCPMetricsExporter::new(Arc::new(auth), config))
    }
}

impl GCPMetricsExporter {
    pub fn fake_new() -> GCPMetricsExporter {
        GCPMetricsExporter::new(
            Arc::new(FakeAuthorizer::new()),
            GCPMetricsExporterConfig::default(),
        )
    }
}

impl Debug for GCPMetricsExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Google monitoring metrics exporter")
    }
}

impl GCPMetricsExporter {
    /// We can map Metric to MetricDescriptor using Metric.name or
    /// MetricDescriptor.type. We create the MetricDescriptor if it doesn't
    /// exist already and cache it. Note that recreating MetricDescriptors is
    /// a no-op if it already exists.
    ///
    /// :param record:
    /// :return:
    async fn get_metric_descriptor(
        &self,
        metric: &OpentelemetrySdkMetric,
    ) -> Option<MetricDescriptor> {
        let descriptor_type = format!("{}/{}", self.prefix, metric.name);
        let cached_metric_descriptor = {
            let metric_descriptors = self.metric_descriptors.read().await;
            metric_descriptors.get(&descriptor_type).cloned()
        };
        if let Some(cached_metric_descriptor) = cached_metric_descriptor {
            return Some(cached_metric_descriptor);
        }

        let unit = metric.unit.to_string();
        let mut descriptor = MetricDescriptor {
            r#type: descriptor_type.clone(),
            display_name: metric.name.to_string(),
            description: metric.description.to_string(),
            unit: unit,
            ..Default::default()
        };
        let seen_keys: HashSet<String> = get_data_points_attributes_keys(metric.data.as_any());

        for key in &seen_keys {
            descriptor.labels.push(LabelDescriptor {
                key: normalize_label_key(key),
                ..Default::default()
            });
        }

        // todo add unique identifier
        if self.add_unique_identifier {
            descriptor.labels.push(LabelDescriptor {
                key: UNIQUE_IDENTIFIER_KEY.to_string(),
                ..Default::default()
            });
        }
        {
            let data = metric.data.as_any();
            if let Some(_) = data.downcast_ref::<SdkHistogram<i64>>() {
                descriptor.metric_kind = MetricKind::Cumulative.into();
                descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
            } else if let Some(_) = data.downcast_ref::<SdkHistogram<u64>>() {
                descriptor.metric_kind = MetricKind::Cumulative.into();
                descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
            } else if let Some(_) = data.downcast_ref::<SdkHistogram<f64>>() {
                descriptor.metric_kind = MetricKind::Cumulative.into();
                descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
            } else if let Some(_) = data.downcast_ref::<SdkExponentialHistogram<i64>>() {
                descriptor.metric_kind = MetricKind::Cumulative.into();
                descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
            } else if let Some(_) = data.downcast_ref::<SdkExponentialHistogram<u64>>() {
                descriptor.metric_kind = MetricKind::Cumulative.into();
                descriptor.value_type = metric_descriptor::ValueType::Distribution.into();
            } else if let Some(_) = data.downcast_ref::<SdkExponentialHistogram<f64>>() {
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
            } else if let Some(_) = data.downcast_ref::<SdkGauge<u64>>() {
                descriptor.metric_kind = MetricKind::Gauge.into();
                descriptor.value_type = metric_descriptor::ValueType::Int64.into();
            } else if let Some(_) = data.downcast_ref::<SdkGauge<i64>>() {
                descriptor.metric_kind = MetricKind::Gauge.into();
                descriptor.value_type = metric_descriptor::ValueType::Int64.into();
            } else if let Some(_) = data.downcast_ref::<SdkGauge<f64>>() {
                descriptor.metric_kind = MetricKind::Gauge.into();
                descriptor.value_type = metric_descriptor::ValueType::Double.into();
            } else {
                utils::log_warning(MetricsError::Other(format!(
                "GCPMetricsExporter: Unsupported metric data type, ignoring it for metric with name '{}'", metric.name),
            ));
                // warning!("Unsupported metric data type, ignoring it");
                return None;
            }
        }

        let project_id = self
            .project_id
            .clone()
            .unwrap_or(self.authorizer.project_id().to_string());
        let channel = match self.make_chanel().await {
            Ok(channel) => channel,
            Err(err) => {
                utils::log_warning(MetricsError::Other(format!("GCPMetricsExporter: Cant init google services grpc transport channel [Make issue with this case in github repo]: {:?}", err)));
                return None;
            }
        };
        let mut msc = MetricServiceClient::new(channel);
        let mut iteration = 0;
        loop {
            iteration += 1;
            if iteration > 101 {
                utils::log_warning(MetricsError::Other(
                    "GCPMetricsExporter: Cant create_metric_descriptor".into(),
                ));
                return None;
            }
            let mut req = tonic::Request::new(
                gcloud_sdk::google::monitoring::v3::CreateMetricDescriptorRequest {
                    name: format!("projects/{}", project_id),
                    metric_descriptor: Some(descriptor.clone()),
                },
            );
            match self.authorizer.token().await {
                Ok(token) => {
                    req.metadata_mut().insert(
                        "authorization",
                        MetadataValue::try_from(format!("Bearer {}", token.as_str())).unwrap(),
                    );
                }
                Err(err) => {
                    utils::log_warning(MetricsError::Other(format!(
                        "GCPMetricsExporter: cant authorize: {:?}",
                        err
                    )));
                    return None;
                }
            }

            match msc.create_metric_descriptor(req).await {
                Ok(_resp) => break,
                Err(err) => {
                    // logger.error(
                    //     "Failed to create metric descriptor %s",
                    //     descriptor,
                    //     exc_info=ex,
                    // )
                    utils::log_warning(MetricsError::Other(format!(
                        "GCPMetricsExporter: Retry send create_metric_descriptor: {:?}",
                        err
                    )));
                    match err.code() {
                        tonic::Code::Unavailable
                        | tonic::Code::DataLoss
                        | tonic::Code::DeadlineExceeded
                        | tonic::Code::Aborted
                        | tonic::Code::Internal
                        | tonic::Code::FailedPrecondition => {
                            sleep(Duration::from_millis(200)).await;
                            continue;
                        }
                        tonic::Code::AlreadyExists => {
                            break;
                        }
                        _ => {
                            return None;
                        }
                    }
                }
            }
        }

        {
            let mut metric_descriptors = self.metric_descriptors.write().await;
            metric_descriptors.insert(descriptor_type, descriptor.clone());
        }
        Some(descriptor)
    }

    async fn exec_export(&self, metrics: &mut ResourceMetrics) -> Result<(), OTelSdkError> {
        // // println!("export: {:#?}", metrics);
        // let proto_message: ExportMetricsServiceRequest = (&*metrics).into();
        // // println!("export: {}", serde_json::to_string_pretty(&proto_message).unwrap());

        // use std::io::Write;
        // let mut file = std::fs::File::create("metrics.txt").unwrap();
        // file.write_all(format!("{:#?}", metrics).as_bytes()).unwrap();
        let monitored_resource_data = match self.custom_monitored_resource_data.clone() {
            Some(custom_monitored_resource_data) => {
                Some(gcloud_sdk::google::api::MonitoredResource {
                    r#type: custom_monitored_resource_data.r#type,
                    labels: custom_monitored_resource_data.labels,
                })
            }
            None => get_monitored_resource(metrics.resource.clone()).map(|v| {
                gcloud_sdk::google::api::MonitoredResource {
                    r#type: v.r#type,
                    labels: v.labels,
                }
            }),
        };

        let mut all_series = Vec::<TimeSeries>::new();
        for scope_metric in &metrics.scope_metrics {
            for metric in &scope_metric.metrics {
                let descriptor: MetricDescriptor =
                    if let Some(descriptor) = self.get_metric_descriptor(metric).await {
                        descriptor
                    } else {
                        continue;
                    };
                let data = metric.data.as_any();
                if let Some(v) = data.downcast_ref::<SdkHistogram<i64>>() {
                    for data_point in &v.data_points {
                        all_series.push(histogram_data_point_to_time_series::convert(
                            data_point,
                            &v.start_time,
                            &v.time,
                            &descriptor,
                            &monitored_resource_data,
                            self.add_unique_identifier,
                            &self.unique_identifier,
                        ));
                    }
                } else if let Some(v) = data.downcast_ref::<SdkHistogram<u64>>() {
                    for data_point in &v.data_points {
                        all_series.push(histogram_data_point_to_time_series::convert(
                            data_point,
                            &v.start_time,
                            &v.time,
                            &descriptor,
                            &monitored_resource_data,
                            self.add_unique_identifier,
                            &self.unique_identifier,
                        ));
                    }
                } else if let Some(v) = data.downcast_ref::<SdkHistogram<f64>>() {
                    for data_point in &v.data_points {
                        all_series.push(histogram_data_point_to_time_series::convert(
                            data_point,
                            &v.start_time,
                            &v.time,
                            &descriptor,
                            &monitored_resource_data,
                            self.add_unique_identifier,
                            &self.unique_identifier,
                        ));
                    }
                // } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<i64>>() {
                //     for data_point in &v.data_points {
                //         all_series.push(histogram_data_point_to_time_series::convert_exponential(data_point, &descriptor, &monitored_resource_data));
                //     }
                // } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<u64>>() {
                //     for data_point in &v.data_points {
                //         all_series.push(histogram_data_point_to_time_series::convert_exponential(data_point, &descriptor, &monitored_resource_data));
                //     }
                // } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<f64>>() {
                //     for data_point in &v.data_points {
                //         all_series.push(histogram_data_point_to_time_series::convert_exponential(data_point, &descriptor, &monitored_resource_data));
                //     }
                } else if let Some(v) = data.downcast_ref::<SdkSum<u64>>() {
                    for data_point in &v.data_points {
                        all_series.push(data_point_to_time_series::sum_convert_i64(
                            data_point,
                            &v.start_time,
                            &v.time,
                            &descriptor,
                            &monitored_resource_data,
                            self.add_unique_identifier,
                            self.unique_identifier.clone(),
                        ));
                    }
                } else if let Some(v) = data.downcast_ref::<SdkSum<i64>>() {
                    for data_point in &v.data_points {
                        all_series.push(data_point_to_time_series::sum_convert_i64(
                            data_point,
                            &v.start_time,
                            &v.time,
                            &descriptor,
                            &monitored_resource_data,
                            self.add_unique_identifier,
                            self.unique_identifier.clone(),
                        ));
                    }
                } else if let Some(v) = data.downcast_ref::<SdkSum<f64>>() {
                    for data_point in &v.data_points {
                        all_series.push(data_point_to_time_series::sum_convert_f64(
                            data_point,
                            &v.start_time,
                            &v.time,
                            &descriptor,
                            &monitored_resource_data,
                            self.add_unique_identifier,
                            self.unique_identifier.clone(),
                        ));
                    }
                } else if let Some(v) = data.downcast_ref::<SdkGauge<u64>>() {
                    for data_point in &v.data_points {
                        all_series.push(data_point_to_time_series::gauge_convert_i64(
                            data_point,
                            &v.start_time,
                            &v.time,
                            &descriptor,
                            &monitored_resource_data,
                            self.add_unique_identifier,
                            self.unique_identifier.clone(),
                        ));
                    }
                } else if let Some(v) = data.downcast_ref::<SdkGauge<i64>>() {
                    for data_point in &v.data_points {
                        all_series.push(data_point_to_time_series::gauge_convert_i64(
                            data_point,
                            &v.start_time,
                            &v.time,
                            &descriptor,
                            &monitored_resource_data,
                            self.add_unique_identifier,
                            self.unique_identifier.clone(),
                        ));
                    }
                } else if let Some(v) = data.downcast_ref::<SdkGauge<f64>>() {
                    for data_point in &v.data_points {
                        all_series.push(data_point_to_time_series::gauge_convert_f64(
                            data_point,
                            &v.start_time,
                            &v.time,
                            &descriptor,
                            &monitored_resource_data,
                            self.add_unique_identifier,
                            self.unique_identifier.clone(),
                        ));
                    }
                } else {
                    utils::log_warning(MetricsError::Other(format!(
                        "GCPMetricsExporter: Unsupported metric data type, ignoring it for metric with name '{}'", metric.name),
                    ));
                };
            }
        }
        // println!("all_series len: {}", all_series.len());
        let chunked_all_series: Vec<Vec<TimeSeries>> = all_series
            .into_iter()
            .chunks(200)
            .into_iter()
            .map(|chunk| chunk.collect())
            .collect();
        // todo add more usefull error handling and retry
        let project_id = self
            .project_id
            .clone()
            .unwrap_or(self.authorizer.project_id().to_string());
        for chunk in chunked_all_series {
            let mut iteration = 0;
            loop {
                iteration += 1;
                if iteration > 101 {
                    return Err(OTelSdkError::InternalFailure(
                        "GCPMetricsExporter: Cant send time series".into(),
                    ));
                }
                // todo optimize clones
                let create_time_series_request = CreateTimeSeriesRequest {
                    name: format!("projects/{}", project_id),
                    time_series: chunk.clone(),
                };
                // println!("chunk: {:?}", create_time_series_request);
                let mut req = tonic::Request::new(create_time_series_request.clone());
                match self.authorizer.token().await {
                    Ok(token) => {
                        req.metadata_mut().insert(
                            "authorization",
                            MetadataValue::try_from(format!("Bearer {}", token.as_str())).unwrap(),
                        );
                    }
                    Err(err) => {
                        return Err(OTelSdkError::InternalFailure(format!(
                            "GCPMetricsExporter: cant authorize: {:?}",
                            err
                        )));
                    }
                }
                let channel = match self.make_chanel().await {
                    Ok(channel) => channel,
                    Err(err) => {
                        return Err(OTelSdkError::InternalFailure(format!("GCPMetricsExporter: Cant init google services grpc transport channel [Make issue with this case in github repo]: {:?}", err)));
                    }
                };
                let mut msc = MetricServiceClient::new(channel);
                if let Err(err) = msc.create_time_series(req).await {
                    utils::log_warning(MetricsError::Other(format!(
                        "GCPMetricsExporter: Cant send time series: {:?}",
                        err
                    )));
                    match err.code() {
                        tonic::Code::Unavailable
                        | tonic::Code::DataLoss
                        | tonic::Code::DeadlineExceeded
                        | tonic::Code::Aborted
                        | tonic::Code::Internal
                        | tonic::Code::FailedPrecondition => {
                            sleep(Duration::from_millis(200)).await;
                            continue;
                        }
                        _ => {
                            utils::log_warning(MetricsError::Other(format!(
                                "GCPMetricsExporter: Cant send time series: Request: {:?}",
                                create_time_series_request
                            )));
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl PushMetricsExporter for GCPMetricsExporter {
    async fn export(&self, metrics: &mut ResourceMetrics) -> Result<(), OTelSdkError> {
        let sys_time = SystemTime::now();
        let resp = self.exec_export(metrics).await;
        let new_sys_time = SystemTime::now();
        let _difference = new_sys_time
            .duration_since(sys_time)
            .expect("Clock may have gone backwards")
            .as_millis();
        // info!("export time: {}", difference);
        resp
    }

    async fn force_flush(&self) -> Result<(), OTelSdkError> {
        Ok(()) // In this implementation, flush does nothing
    }

    fn shutdown(&self) -> Result<(), OTelSdkError> {
        // TracepointState automatically unregisters when dropped
        // https://github.com/microsoft/LinuxTracepoints-Rust/blob/main/eventheader/src/native.rs#L618
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        Temporality::default()
    }
}

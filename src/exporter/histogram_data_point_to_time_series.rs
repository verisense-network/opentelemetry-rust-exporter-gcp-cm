use gcloud_sdk::google::{api::{metric_descriptor, metric_descriptor::MetricKind, LabelDescriptor, MetricDescriptor}, monitoring::v3::{metric_service_client::MetricServiceClient, CreateTimeSeriesRequest, TimeSeries}};
use gcp_auth::TokenProvider;
use opentelemetry::{global, metrics::{MetricsError, Result as MetricsResult}};
use opentelemetry_proto::tonic::{collector::metrics::v1::ExportMetricsServiceRequest, metrics::v1::metric::Data as TonicMetricData};
use opentelemetry_resourcedetector_gcp_rust::mapping::get_monitored_resource;
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


use core::time;
use std::{collections::HashSet, fmt::{Debug, Formatter}, sync::Arc, time::Duration};
use rand::Rng;
use std::time::SystemTime;

use super::{utils::{get_data_points_attributes_keys, normalize_label_key}, UNIQUE_IDENTIFIER_KEY};
use crate::{gcloud_sdk, gcp_authorizer::{Authorizer, FakeAuthorizer, GoogleEnvironment}};
use crate::exporter::to_f64::ToF64;


pub fn convert<T: ToF64 + Copy>(data_point: &data::HistogramDataPoint<T>, descriptor: &MetricDescriptor,monitored_resource_data: &Option<gcloud_sdk::google::api::MonitoredResource>, add_unique_identifier: bool, unique_identifier: String) -> TimeSeries {
    let data_point_start_time = data_point.start_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    let data_point_time = data_point.time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    let point = gcloud_sdk::google::monitoring::v3::Point {
        interval: Some(gcloud_sdk::google::monitoring::v3::TimeInterval {
            start_time: Some(gcloud_sdk::prost_types::Timestamp {
                seconds: (data_point_start_time / 1_000_000_000) as i64,
                nanos: (data_point_start_time % 1_000_000_000) as i32,
            }),
            end_time: Some(gcloud_sdk::prost_types::Timestamp {
                seconds: (data_point_time / 1_000_000_000) as i64,
                nanos: (data_point_time % 1_000_000_000) as i32,
            }),
        }),
        value: Some(gcloud_sdk::google::monitoring::v3::TypedValue {
            value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::DistributionValue(gcloud_sdk::google::api::Distribution {
                count: data_point.count as i64,
                mean: {
                    if data_point.count == 0 {
                        0.0
                    } else {
                        data_point.sum.to_f64() / data_point.count as f64
                    }
                },
                sum_of_squared_deviation: 0.0,
                bucket_options: Some(gcloud_sdk::google::api::distribution::BucketOptions {
                    options: Some(gcloud_sdk::google::api::distribution::bucket_options::Options::ExplicitBuckets(gcloud_sdk::google::api::distribution::bucket_options::Explicit {
                        bounds: data_point.bounds.clone(),
                    })),
                }),
                range: None,
                bucket_counts: data_point.bucket_counts.iter().map(|v| *v as i64).collect(),
                exemplars: Default::default(),
            })),
        }),
    };

    let mut labels = data_point.attributes.iter().map(|kv| (normalize_label_key(&kv.key.to_string()), kv.value.to_string())).collect::<std::collections::HashMap<String, String>>();
    if add_unique_identifier {
        labels.insert(UNIQUE_IDENTIFIER_KEY.to_string(), unique_identifier.clone());
    } 

    let time_series = TimeSeries {
        resource: monitored_resource_data.clone(),
        metadata: None,
        metric_kind: descriptor.metric_kind,
        value_type: descriptor.value_type,
        metric: Some(gcloud_sdk::google::api::Metric {
            r#type: descriptor.r#type.clone(),
            labels: labels,
        }),
        points: vec![point],
        unit: descriptor.unit.clone(),
    };
    time_series
}

#[cfg(feature = "")]
pub fn convert_exponential<T: ToF64 + Copy>(data_point: &data::ExponentialHistogramDataPoint<T>, descriptor: &MetricDescriptor,monitored_resource_data: &Option<gcloud_sdk::google::api::MonitoredResource>) -> TimeSeries {
    let data_point_start_time = data_point.start_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    let data_point_time = data_point.time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
    let point = gcloud_sdk::google::monitoring::v3::Point {
        interval: Some(gcloud_sdk::google::monitoring::v3::TimeInterval {
            start_time: Some(gcloud_sdk::prost_types::Timestamp {
                seconds: (data_point_start_time / 1_000_000_000) as i64,
                nanos: (data_point_start_time % 1_000_000_000) as i32,
            }),
            end_time: Some(gcloud_sdk::prost_types::Timestamp {
                seconds: (data_point_time / 1_000_000_000) as i64,
                nanos: (data_point_time % 1_000_000_000) as i32,
            }),
        }),
        value: Some(gcloud_sdk::google::monitoring::v3::TypedValue {
            value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::DistributionValue(gcloud_sdk::google::api::Distribution {
                count: data_point.count as i64,
                mean: {
                    if data_point.count == 0 {
                        0.0
                    } else {
                        data_point.sum.to_f64() / data_point.count as f64
                    }
                },
                sum_of_squared_deviation: 0.0,
                bucket_options: Some(gcloud_sdk::google::api::distribution::BucketOptions {
                    options: Some(gcloud_sdk::google::api::distribution::bucket_options::Options::ExponentialBuckets(gcloud_sdk::google::api::distribution::bucket_options::Exponential {
                        bounds: data_point.bounds.clone(),
                    })),
                }),
                range: None,
                bucket_counts: data_point.bucket_counts.iter().map(|v| *v as i64).collect(),
                exemplars: Default::default(),
            })),
        }),
    };

    let labels = data_point.attributes.iter().map(|kv| (normalize_label_key(&kv.key.to_string()), kv.value.to_string())).collect::<std::collections::HashMap<String, String>>();
    // if self.add_unique_identifier {
    //     labels.insert(UNIQUE_IDENTIFIER_KEY.to_string(), self.unique_identifier.clone());
    // }  

    let time_series = TimeSeries {
        resource: monitored_resource_data.clone(),
        metadata: None,
        metric_kind: descriptor.metric_kind,
        value_type: descriptor.value_type,
        metric: Some(gcloud_sdk::google::api::Metric {
            r#type: descriptor.r#type.clone(),
            labels: labels,
        }),
        points: vec![point],
        unit: descriptor.unit.clone(),
    };
    time_series
}
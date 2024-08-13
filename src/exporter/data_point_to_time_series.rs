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

use super::{to_f64::{ToF64, ToI64}, utils::{get_data_points_attributes_keys, normalize_label_key}, UNIQUE_IDENTIFIER_KEY};
use crate::{gcloud_sdk, gcp_authorizer::{Authorizer, FakeAuthorizer, GoogleEnvironment}};


pub fn convert_f64<T: ToF64 + Copy>(data_point: &data::DataPoint<T>, descriptor: &MetricDescriptor, monitored_resource_data: &Option<gcloud_sdk::google::api::MonitoredResource>, add_unique_identifier: bool, unique_identifier: String) -> TimeSeries {

    
    let point = gcloud_sdk::google::monitoring::v3::Point {
        interval: Some(gcloud_sdk::google::monitoring::v3::TimeInterval {
            start_time: if (descriptor.metric_kind == MetricKind::Cumulative as i32) || (descriptor.metric_kind == MetricKind::Delta as i32) {
                data_point.start_time.map(|v| {
                    let data_point_start_time = v.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
                    gcloud_sdk::prost_types::Timestamp {
                        seconds: (data_point_start_time / 1_000_000_000) as i64,
                        nanos: (data_point_start_time % 1_000_000_000) as i32,
                    }
                })
            } else {
                None
            },
            end_time: data_point.time.map(|v| {
                let data_point_time = v.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
                gcloud_sdk::prost_types::Timestamp {
                    seconds: (data_point_time / 1_000_000_000) as i64,
                    nanos: (data_point_time % 1_000_000_000) as i32,
                }
            }),
        }),
        value: Some(gcloud_sdk::google::monitoring::v3::TypedValue {
            value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::DoubleValue(data_point.value.to_f64())),
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

pub fn convert_i64<T: ToI64 + Copy>(data_point: &data::DataPoint<T>, descriptor: &MetricDescriptor,monitored_resource_data: &Option<gcloud_sdk::google::api::MonitoredResource>, add_unique_identifier: bool, unique_identifier: String) -> TimeSeries {
    let point = gcloud_sdk::google::monitoring::v3::Point {
        interval: Some(gcloud_sdk::google::monitoring::v3::TimeInterval {
            start_time: if (descriptor.metric_kind == MetricKind::Cumulative as i32) || (descriptor.metric_kind == MetricKind::Delta as i32) {
                data_point.start_time.map(|v| {
                    let data_point_start_time = v.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
                    gcloud_sdk::prost_types::Timestamp {
                        seconds: (data_point_start_time / 1_000_000_000) as i64,
                        nanos: (data_point_start_time % 1_000_000_000) as i32,
                    }
                })
            } else {
                None
            },
            end_time: data_point.time.map(|v| {
                let data_point_time = v.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
                gcloud_sdk::prost_types::Timestamp {
                    seconds: (data_point_time / 1_000_000_000) as i64,
                    nanos: (data_point_time % 1_000_000_000) as i32,
                }
            }),
        }),
        value: Some(gcloud_sdk::google::monitoring::v3::TypedValue {
            value: Some(gcloud_sdk::google::monitoring::v3::typed_value::Value::Int64Value(data_point.value.to_i64())),
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
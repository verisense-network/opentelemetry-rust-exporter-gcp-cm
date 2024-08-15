crate::import_opentelemetry!();
use super::{
    to_f64::{ToF64, ToI64},
    utils::normalize_label_key,
    UNIQUE_IDENTIFIER_KEY,
};
use crate::gcloud_sdk;
use gcloud_sdk::google::{
    api::{metric_descriptor::MetricKind, MetricDescriptor},
    monitoring::v3::TimeSeries,
};
use opentelemetry_sdk::metrics::data;
use std::time::SystemTime;

pub fn convert_f64<T: ToF64 + Copy>(
    data_point: &data::DataPoint<T>,
    descriptor: &MetricDescriptor,
    monitored_resource_data: &Option<gcloud_sdk::google::api::MonitoredResource>,
    add_unique_identifier: bool,
    unique_identifier: String,
) -> TimeSeries {
    let point = gcloud_sdk::google::monitoring::v3::Point {
        interval: Some(gcloud_sdk::google::monitoring::v3::TimeInterval {
            start_time: if (descriptor.metric_kind == MetricKind::Cumulative as i32)
                || (descriptor.metric_kind == MetricKind::Delta as i32)
            {
                data_point.start_time.map(|v| {
                    let data_point_start_time =
                        v.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
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
            value: Some(
                gcloud_sdk::google::monitoring::v3::typed_value::Value::DoubleValue(
                    data_point.value.to_f64(),
                ),
            ),
        }),
    };

    let mut labels = data_point
        .attributes
        .iter()
        .map(|kv| {
            (
                normalize_label_key(&kv.key.to_string()),
                kv.value.to_string(),
            )
        })
        .collect::<std::collections::HashMap<String, String>>();
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

pub fn convert_i64<T: ToI64 + Copy>(
    data_point: &data::DataPoint<T>,
    descriptor: &MetricDescriptor,
    monitored_resource_data: &Option<gcloud_sdk::google::api::MonitoredResource>,
    add_unique_identifier: bool,
    unique_identifier: String,
) -> TimeSeries {
    let point = gcloud_sdk::google::monitoring::v3::Point {
        interval: Some(gcloud_sdk::google::monitoring::v3::TimeInterval {
            start_time: if (descriptor.metric_kind == MetricKind::Cumulative as i32)
                || (descriptor.metric_kind == MetricKind::Delta as i32)
            {
                data_point.start_time.map(|v| {
                    let data_point_start_time =
                        v.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
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
            value: Some(
                gcloud_sdk::google::monitoring::v3::typed_value::Value::Int64Value(
                    data_point.value.to_i64(),
                ),
            ),
        }),
    };

    let mut labels = data_point
        .attributes
        .iter()
        .map(|kv| {
            (
                normalize_label_key(&kv.key.to_string()),
                kv.value.to_string(),
            )
        })
        .collect::<std::collections::HashMap<String, String>>();
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

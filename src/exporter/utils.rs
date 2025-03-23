crate::import_opentelemetry!();
#[cfg(any(
    // feature = "opentelemetry_0_21",
    // feature = "opentelemetry_0_22",
    // feature = "opentelemetry_0_23",
    feature = "opentelemetry_0_24",
    feature = "opentelemetry_0_25",
    feature = "opentelemetry_0_26",
    feature = "opentelemetry_0_27",
))]
use opentelemetry::KeyValue;
#[cfg(any(
    feature = "opentelemetry_0_21",
    feature = "opentelemetry_0_22",
    feature = "opentelemetry_0_23",
    feature = "opentelemetry_0_24",
    feature = "opentelemetry_0_25",
    feature = "opentelemetry_0_26",
))]
use opentelemetry::{global, metrics::MetricsError};
#[cfg(any(feature = "opentelemetry_0_27",))]
use opentelemetry_sdk::metrics::MetricError as MetricsError;

use opentelemetry_sdk::metrics::data::{
    ExponentialHistogram as SdkExponentialHistogram, Gauge as SdkGauge, Histogram as SdkHistogram,
    Sum as SdkSum,
};
use std::any::Any;
use std::collections::HashSet;

pub(crate) fn log_warning(err: MetricsError) {
    #[cfg(any(
        feature = "opentelemetry_0_21",
        feature = "opentelemetry_0_22",
        feature = "opentelemetry_0_23",
        feature = "opentelemetry_0_24",
        feature = "opentelemetry_0_25",
        feature = "opentelemetry_0_26",
    ))]
    global::handle_error(err);
    #[cfg(any(feature = "opentelemetry_0_27",))]
    tracing::warn!("{}", err);
}

pub(crate) fn get_data_points_attributes_keys(data: &dyn Any) -> HashSet<String> {
    let attributes_keys = if let Some(v) = data.downcast_ref::<SdkHistogram<i64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkHistogram<u64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkHistogram<f64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<i64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<u64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<f64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkSum<u64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkSum<i64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkSum<f64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkGauge<u64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkGauge<i64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else if let Some(v) = data.downcast_ref::<SdkGauge<f64>>() {
        v.data_points
            .iter()
            .map(|point| point.attributes.iter().map(kv_map_k))
            .flatten()
            .collect()
    } else {
        log_warning(MetricsError::Other(
            "Unsupported metric data type, ignoring it".into(),
        ));
        vec![]
    };
    HashSet::from_iter(attributes_keys.into_iter())
}

use unicode_segmentation::UnicodeSegmentation;

///Makes the key into a valid GCM label key
///
///    See reference impl
///
///    https://github.com/GoogleCloudPlatform/opentelemetry-operations-go/blob/e955c204f4f2bfdc92ff0ad52786232b975efcc2/exporter/metric/metric.go#L595-L604
///
pub(crate) fn normalize_label_key(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    let s = sanitize_string(s);
    if s.chars().next().map_or(false, |c| c.is_digit(10)) {
        return format!("key_{}", s);
    }
    s
}

// Converts anything that is not a letter or digit to an underscore
fn sanitize_string(s: &str) -> String {
    s.graphemes(true)
        .map(|g| {
            if g.chars().all(|c| c.is_alphanumeric()) {
                g.to_string()
            } else {
                "_".to_string()
            }
        })
        .collect::<String>()
}

#[cfg(any(
    // feature = "opentelemetry_0_21",
    // feature = "opentelemetry_0_22",
    // feature = "opentelemetry_0_23",
    feature = "opentelemetry_0_24",
    feature = "opentelemetry_0_25",
    feature = "opentelemetry_0_26",
    feature = "opentelemetry_0_27",
))]
pub(crate) fn kv_map_normalize_k_v(kv: &KeyValue) -> (String, String) {
    (
        normalize_label_key(&kv.key.to_string()),
        kv.value.to_string(),
    )
}

#[cfg(any(
    // feature = "opentelemetry_0_21",
    // feature = "opentelemetry_0_22",
    // feature = "opentelemetry_0_23",
    feature = "opentelemetry_0_24",
    feature = "opentelemetry_0_25",
    feature = "opentelemetry_0_26",
    feature = "opentelemetry_0_27",
))]
pub(crate) fn kv_map_k(kv: &KeyValue) -> String {
    kv.key.to_string()
}
#[cfg(any(
    feature = "opentelemetry_0_21",
    feature = "opentelemetry_0_22",
    feature = "opentelemetry_0_23",
))]
pub(crate) fn kv_map_normalize_k_v(
    kv: (&opentelemetry::Key, &opentelemetry::Value),
) -> (String, String) {
    (normalize_label_key(&kv.0.to_string()), kv.1.to_string())
}
#[cfg(any(
    feature = "opentelemetry_0_21",
    feature = "opentelemetry_0_22",
    feature = "opentelemetry_0_23",
))]
pub(crate) fn kv_map_k(kv: (&opentelemetry::Key, &opentelemetry::Value)) -> String {
    kv.0.to_string()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_label_key() {
        assert_eq!(normalize_label_key("valid_key_1"), "valid_key_1");
        assert_eq!(normalize_label_key("hellø"), "hellø");
        assert_eq!(normalize_label_key("123"), "key_123");
        assert_eq!(normalize_label_key("key!321"), "key_321");
        assert_eq!(
            normalize_label_key("hyphens-dots.slashes/"),
            "hyphens_dots_slashes_"
        );
        assert_eq!(
            normalize_label_key("non_letters_:£¢$∞"),
            "non_letters______"
        );
    }
}

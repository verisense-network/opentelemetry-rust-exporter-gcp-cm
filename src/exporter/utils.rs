use std::any::Any;
use std::collections::HashSet;

use opentelemetry::{global, metrics::MetricsError, Key, Value};
use opentelemetry_sdk::metrics::data::{
    self, Aggregation, Exemplar as SdkExemplar, ExponentialHistogram as SdkExponentialHistogram, Gauge as SdkGauge, Histogram as SdkHistogram, Metric as SdkMetric, ScopeMetrics as SdkScopeMetrics, Sum as SdkSum, Temporality
};
use opentelemetry_sdk::Resource as SdkResource;

pub fn get_data_points_attributes_keys(data: &dyn Any) -> HashSet<String> { 
    let attributes_keys = if let Some(v) = data.downcast_ref::<SdkHistogram<i64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkHistogram<u64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkHistogram<f64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<i64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<u64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkExponentialHistogram<f64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkSum<u64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkSum<i64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkSum<f64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkGauge<u64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkGauge<i64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else if let Some(v) = data.downcast_ref::<SdkGauge<f64>>() {
       v.data_points.iter().map(|point| point.attributes.iter().map(|x| x.key.to_string())).flatten().collect()
    } else {
        global::handle_error(MetricsError::Other("unknown aggregator".into()));
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
pub fn normalize_label_key(s: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_label_key() {
        assert_eq!(normalize_label_key("valid_key_1"), "valid_key_1");
        assert_eq!(normalize_label_key("hellø"), "hellø");
        assert_eq!(normalize_label_key("123"), "key_123");
        assert_eq!(normalize_label_key("key!321"), "key_321");
        assert_eq!(normalize_label_key("hyphens-dots.slashes/"), "hyphens_dots_slashes_");
        assert_eq!(normalize_label_key("non_letters_:£¢$∞"), "non_letters______");
    }
}
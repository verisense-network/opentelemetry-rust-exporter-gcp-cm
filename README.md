# OpenTelemetry Google Cloud Monitoring Exporter

This library provides support for exporting metrics to Google Cloud Monitoring.

For resource detection see [opentelemetry-resourcedetector-gcp-rust](https://github.com/Sergo007/opentelemetry-resourcedetector-gcp-rust).


implementation [python version](https://github.com/GoogleCloudPlatform/opentelemetry-operations-python/tree/main/opentelemetry-exporter-gcp-monitoring)

# Installation
`cargo add opentelemetry-gcloud-monitoring-exporter` - exporter
`cargo add opentelemetry_resourcedetector_gcp_rust` - gcp resource detection 

# Usage

```rust
use opentelemetry::{metrics::MeterProvider as _, KeyValue};
use opentelemetry_gcloud_monitoring_exporter::{
    GCPMetricsExporter, GCPMetricsExporterConfig, MonitoredResourceDataConfig,
};
use opentelemetry_resourcedetector_gcp_rust::GoogleCloudResourceDetector;
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    runtime, Resource,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
fn to_labels(kv: serde_json::Value) -> HashMap<String, String> {
    kv.as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.to_string(), v.as_str().unwrap().to_string()))
        .collect()
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = GCPMetricsExporterConfig::default();
    cfg.prefix = "custom.googleapis.com/test_service".to_string();
    let exporter = GCPMetricsExporter::new_gcp_auth(cfg).await?;
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    let gcp_detector = GoogleCloudResourceDetector::new().await;
    let res = Resource::default().merge(&rname);
    SdkMeterProvider::builder()
        .with_resource(res)
        .with_reader(reader)
        .build();

    let meter = meter_provider.meter("user-event-test");

    let counter = meter
        .f64_counter("counter_f64_test")
        .with_description("test_decription")
        .with_unit("test_unit")
        .init();

    loop {
        // Record measurements using the Counter instrument.
        counter.add(
            1.0,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        );
         tokio::time::sleep(Duration::from_millis(300)).await;
    }
}
```

Customize metric location in google monitoring
```rust
    let mut cfg = GCPMetricsExporterConfig::default();
    cfg.prefix = "custom.googleapis.com/test_service".to_string();

    // customize metric location in google monitoring
    cfg.custom_monitored_resource_data = Some(
        // https://cloud.google.com/monitoring/api/resources#tag_global
        MonitoredResourceDataConfig {
            r#type: "global".to_string(),
            labels: to_labels(json!({
                "project_id": "my-project",
            })),
        },
    );
    let exporter = GCPMetricsExporter::new_gcp_auth(cfg).await?;
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    SdkMeterProvider::builder()
        .with_reader(reader)
        .build();
```

# OpenTelemetry Google Cloud Monitoring Exporter

This library provides support for exporting metrics to Google Cloud Monitoring.

For resource detection see [opentelemetry-resourcedetector-gcp-rust](https://github.com/Sergo007/opentelemetry-resourcedetector-gcp-rust).

# Installation
`cargo add opentelemetry_gcloud_monitoring_exporter` - exporter

`cargo add opentelemetry_resourcedetector_gcp_rust` - gcp resource detection 

or add to cargo.toml

```
[dependencies]
opentelemetry_gcloud_monitoring_exporter = { varsion = "0.11.0", features = [
    "tokio",
    "opentelemetry_0_24",
    "gcp_auth",
] }
tokio = { version = "1.0", features = ["full"] }
opentelemetry = { version = "0.24", features = ["metrics"] }
opentelemetry_sdk = { version = "0.24", features = ["metrics", "rt-tokio"] }
opentelemetry_resourcedetector_gcp_rust = "0.11.0"
```

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

Customize metric resource in google monitoring
```rust
    let mut cfg = GCPMetricsExporterConfig::default();
    cfg.prefix = "custom.googleapis.com/test_service".to_string();

    // customize metric resource in google monitoring
    cfg.custom_monitored_resource_data = Some(
        // https://cloud.google.com/monitoring/api/resources#tag_global
        MonitoredResourceDataConfig {
            r#type: "global".to_string(),
            labels: HashMap::from([
                ("project_id".to_string(), "my-project".to_string()),
            ]),
        },
    );
    let exporter = GCPMetricsExporter::new_gcp_auth(cfg).await?;
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    SdkMeterProvider::builder()
        .with_reader(reader)
        .build();
```

## References

[Cloud Monitoring](https://cloud.google.com/monitoring)

[OpenTelemetry Project](https://opentelemetry.io/)


## Test cases from this repo
[opentelemetry-exporter-gcp-monitoring python version](https://github.com/GoogleCloudPlatform/opentelemetry-operations-python/tree/main/opentelemetry-exporter-gcp-monitoring)
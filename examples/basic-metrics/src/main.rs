use opentelemetry::{metrics::MeterProvider as _, KeyValue};
use opentelemetry_gcloud_monitoring_exporter::{GCPMetricsExporter, GCPMetricsExporterConfig};
use opentelemetry_resourcedetector_gcp_rust::GoogleCloudResourceDetector;
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    Resource,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = GCPMetricsExporterConfig::default();
    cfg.prefix = "custom.googleapis.com/test_service".to_string();
    let exporter = GCPMetricsExporter::new_gcp_auth(cfg).await?;
    let reader = PeriodicReader::builder(exporter).build();
    let gcp_detector = Box::new(GoogleCloudResourceDetector::new().await);
    // if we deploy to cloud run or vm instance in gcp we should specify namespace
    // if we don't have namespace we can specify it how 'default'
    let res = Resource::builder_empty()
        .with_attributes(vec![KeyValue::new("service.namespace", "default")])
        .with_detector(gcp_detector)
        .build();
    let meter_provider = SdkMeterProvider::builder()
        .with_resource(res)
        .with_reader(reader)
        .build();

    let meter = meter_provider.meter("user-event-test");

    let counter = meter
        .f64_counter("counter_f64_test")
        .with_description("test_decription")
        .with_unit("test_unit")
        .build();

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

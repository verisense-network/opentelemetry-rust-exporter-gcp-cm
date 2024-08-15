#![allow(dead_code, unused_imports, unused_variables)]
#[macro_use]
pub mod error;
mod exporter;
pub mod gcp_authorizer;
pub use exporter::GCPMetricsExporter;
pub use exporter::GCPMetricsExporterConfig;
pub use exporter::MonitoredResourceDataConfig;
mod gcloud_sdk;
mod opentelemetry;
#[cfg(test)]
mod tests;

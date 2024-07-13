use async_trait::async_trait;
use opentelemetry::metrics::{MetricsError, Result};
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_sdk::metrics::{
    data::{ResourceMetrics, Temporality},
    exporter::PushMetricsExporter,
    reader::{AggregationSelector, DefaultAggregationSelector, TemporalitySelector},
    Aggregation, InstrumentKind,
};

use std::fmt::{Debug, Formatter};


pub struct GCPMetricsExporter {

}

impl GCPMetricsExporter {
    pub fn new() -> Self {
        Self {  }
    }
}

impl Default for GCPMetricsExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalitySelector for GCPMetricsExporter {
    // This is matching OTLP exporters delta.
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        match kind {
            InstrumentKind::Counter
            | InstrumentKind::ObservableCounter
            | InstrumentKind::ObservableGauge
            | InstrumentKind::Histogram
            | InstrumentKind::Gauge => Temporality::Delta,
            InstrumentKind::UpDownCounter | InstrumentKind::ObservableUpDownCounter => {
                Temporality::Cumulative
            }
        }
    }
}

impl AggregationSelector for GCPMetricsExporter {
    // TODO: this should ideally be done at SDK level by default
    // without exporters having to do it.
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        DefaultAggregationSelector::new().aggregation(kind)
    }
}

impl Debug for GCPMetricsExporter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Google monitoring metrics exporter")
    }
}

#[async_trait]
impl PushMetricsExporter for GCPMetricsExporter {
    async fn export(&self, metrics: &mut ResourceMetrics) -> Result<()> {
        println!("export:");
        let proto_message: ExportMetricsServiceRequest = (&*metrics).into();
        println!("export: {}", serde_json::to_string_pretty(&proto_message).unwrap());
        // let mut byte_array = Vec::new();
        // let _encode_result = proto_message
        //     .encode(&mut byte_array)
        //     .map_err(|err| MetricsError::Other(err.to_string()))?;
        // let _result = tracepoint::write(&self.trace_point, byte_array.as_slice());
        // if self.trace_point.enabled() {
        //     let proto_message: ExportMetricsServiceRequest = (&*metrics).into();
        //     println!("export: {:?}", proto_message);
        //     let mut byte_array = Vec::new();
        //     let _encode_result = proto_message
        //         .encode(&mut byte_array)
        //         .map_err(|err| MetricsError::Other(err.to_string()))?;
        //     let _result = tracepoint::write(&self.trace_point, byte_array.as_slice());
        // }
        Ok(())
    }

    async fn force_flush(&self) -> Result<()> {
        Ok(()) // In this implementation, flush does nothing
    }

    fn shutdown(&self) -> Result<()> {
        // TracepointState automatically unregisters when dropped
        // https://github.com/microsoft/LinuxTracepoints-Rust/blob/main/eventheader/src/native.rs#L618
        Ok(())
    }
}

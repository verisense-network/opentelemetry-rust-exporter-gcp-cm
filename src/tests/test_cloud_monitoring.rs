#[cfg(test)]
mod tests {
    use crate::gcloud_sdk::{self, google::monitoring::v3::metric_service_client::MetricServiceClient};
    use metric_service_server::MetricServiceServer;
    use tonic::transport::Channel;
    use tonic::transport::Server;
    use crate::gcloud_sdk::google::api::MetricDescriptor;
    use crate::gcloud_sdk::google::monitoring::v3::*;
    use crate::tests::test_utils::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use std::collections::HashMap;
    use opentelemetry_sdk::Resource;
    use opentelemetry::KeyValue;
    use opentelemetry::metrics::MeterProvider;
    use prost::Message;
    use pretty_assertions_sorted::{assert_eq, assert_eq_sorted};

    #[tokio::test(flavor ="multi_thread", worker_threads = 1)]
    // #[tokio::test]
    async fn test_histogram_default_buckets() {
        let calls = get_gcm_calls().await;
        let metrics_provider = init_metrics(Resource::new(vec![KeyValue::new(
            "service.name",
            "metric-demo",
        )]));
        let meter = metrics_provider.meter("test_cloud_monitoring");
        let histogram = meter
            .f64_histogram("myhistogram")
            .with_description("foo")
            .init();
        for i in 0..10_000 {
            histogram.record(
                i as f64,
                &[
                    KeyValue::new("string", "string"),
                    KeyValue::new("int", 123),
                    KeyValue::new("float", 123.4),
                ],
            );
        }
        metrics_provider.force_flush().unwrap();
        let res = calls.read().await;
        let create_metric_descriptor = res.get("CreateMetricDescriptor").unwrap().iter().map(|v|{
            let msg = CreateMetricDescriptorRequest::decode(v.message.as_slice()).unwrap();
            msg
        }).collect::<Vec<CreateMetricDescriptorRequest>>();
        // create_metric_descriptor.iter().for_each(|v| {
        //     println!("create_metric_descriptor -->");
        //     println!("{:#?}", v);
        // });
        let create_metric_descriptor = create_metric_descriptor.get(0).unwrap().clone();
        
        let expected_create_metric_descriptor = CreateMetricDescriptorRequest {
            name: "projects/fake_project_id".to_string(),
            metric_descriptor: Some(
                MetricDescriptor {
                    name: "".to_string(),
                    r#type: "workload.googleapis.com/myhistogram".to_string(),
                    labels: vec![
                        gcloud_sdk::google::api::LabelDescriptor {
                            key: "int".to_string(),
                            value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                            description: "".to_string(),
                        },
                        gcloud_sdk::google::api::LabelDescriptor {
                            key: "string".to_string(),
                            value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                            description: "".to_string(),
                        },
                        gcloud_sdk::google::api::LabelDescriptor {
                            key: "float".to_string(),
                            value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                            description: "".to_string(),
                        },
                    ],
                    metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                    value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Distribution.into(),
                    unit: "".to_string(),
                    description: "foo".to_string(),
                    display_name: "myhistogram1".to_string(),
                    metadata: None,
                    launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                    monitored_resource_types: Vec::new(),
                },
            ),
        };

        assert_eq_sorted!(create_metric_descriptor, expected_create_metric_descriptor);
    }
}
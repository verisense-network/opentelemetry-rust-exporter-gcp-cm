#[cfg(test)]
mod tests {
    use crate::gcloud_sdk::{self, google::monitoring::v3::metric_service_client::MetricServiceClient};
    use metric_service_server::MetricServiceServer;
    use tonic::transport::Channel;
    use tonic::transport::Server;
    use crate::gcloud_sdk::google::api::MetricDescriptor;
    use crate::gcloud_sdk::google::monitoring::v3::*;
    use crate::tests::test_utils::*;
    use std::collections::hash_map;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use std::collections::HashMap;
    use opentelemetry_sdk::Resource;
    use opentelemetry::KeyValue;
    use opentelemetry::metrics::MeterProvider;
    use prost::Message;
    use pretty_assertions_sorted::{assert_eq, assert_eq_sorted, assert_eq_all_sorted};

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
            .with_unit("myunit")
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
                            key: "string".to_string(),
                            value_type: gcloud_sdk::google::api::label_descriptor::ValueType::String.into(),
                            description: "".to_string(),
                        },
                        gcloud_sdk::google::api::LabelDescriptor {
                            key: "int".to_string(),
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
                    unit: "myunit".to_string(),
                    description: "foo".to_string(),
                    display_name: "myhistogram".to_string(),
                    metadata: None,
                    launch_stage: gcloud_sdk::google::api::LaunchStage::Unspecified.into(),
                    monitored_resource_types: Vec::new(),
                },
            ),
        };
        assert_eq_all_sorted!(create_metric_descriptor, expected_create_metric_descriptor);

        let create_time_series = res.get("CreateTimeSeries").unwrap().iter().map(|v|{
            let msg = CreateTimeSeriesRequest::decode(v.message.as_slice()).unwrap();
            msg
        }).collect::<Vec<CreateTimeSeriesRequest>>();
        // create_time_series.iter().for_each(|v| {
        //     println!("create_time_series -->");
        //     println!("{:#?}", v);
        // });
        let mut create_time_series = create_time_series.get(0).unwrap().clone();
        //WARNING! need to ignore interval becouse its ignored in python tests
        // todo! need to ignore interval for now in tests
        create_time_series.time_series[0].points[0].interval = None;
        let expected_create_time_series = CreateTimeSeriesRequest {
            name: "projects/fake_project_id".to_string(),
            time_series: vec![
                TimeSeries {
                    metric: Some(
                        gcloud_sdk::google::api::Metric {
                            r#type: "workload.googleapis.com/myhistogram".to_string(),
                            labels: HashMap::from([
                                ("float".to_string(), "123.4".to_string()),
                                ("string".to_string(), "string".to_string()),
                                ("int".to_string(), "123".to_string()),
                            ]),
                        },
                    ),
                    resource: Some(
                        gcloud_sdk::google::api::MonitoredResource {
                            r#type: "generic_node".to_string(),
                            labels: HashMap::from([
                                ("location".to_string(), "global".to_string()),
                                ("namespace".to_string(), "".to_string()),
                                ("node_id".to_string(), "".to_string()),
                            ]),
                        },
                    ),
                    metadata: None,
                    metric_kind: gcloud_sdk::google::api::metric_descriptor::MetricKind::Cumulative.into(),
                    value_type: gcloud_sdk::google::api::metric_descriptor::ValueType::Distribution.into(),
                    points: vec![
                        Point {
                            interval: None,
                            //TODO need to ignore interval for now
                            // interval: Some(
                            //     TimeInterval {
                            //         end_time: Some(
                            //             gcloud_sdk::prost_types::Timestamp {
                            //                 seconds: 1723249032,
                            //                 nanos: 972447000,
                            //             },
                            //         ),
                            //         start_time: Some(
                            //             gcloud_sdk::prost_types::Timestamp {
                            //                 seconds: 1723249032,
                            //                 nanos: 929246000,
                            //             },
                            //         ),
                            //     },
                            // ),
                            value: Some(
                                TypedValue {
                                    value: Some(
                                        gcloud_sdk::google::monitoring::v3::typed_value::Value::DistributionValue(
                                            gcloud_sdk::google::api::Distribution {
                                                count: 10000,
                                                mean: 4999.5,
                                                sum_of_squared_deviation: 0.0,
                                                range: None,
                                                bucket_options: Some(
                                                    gcloud_sdk::google::api::distribution::BucketOptions {
                                                        options: Some(
                                                            gcloud_sdk::google::api::distribution::bucket_options::Options::ExplicitBuckets(
                                                                gcloud_sdk::google::api::distribution::bucket_options::Explicit {
                                                                    bounds: [
                                                                        0.0,
                                                                        5.0,
                                                                        10.0,
                                                                        25.0,
                                                                        50.0,
                                                                        75.0,
                                                                        100.0,
                                                                        250.0,
                                                                        500.0,
                                                                        750.0,
                                                                        1000.0,
                                                                        2500.0,
                                                                        5000.0,
                                                                        7500.0,
                                                                        10000.0,
                                                                    ].to_vec(),
                                                                },
                                                            ),
                                                        ),
                                                    },
                                                ),
                                                bucket_counts: [
                                                    1,
                                                    5,
                                                    5,
                                                    15,
                                                    25,
                                                    25,
                                                    25,
                                                    150,
                                                    250,
                                                    250,
                                                    250,
                                                    1500,
                                                    2500,
                                                    2500,
                                                    2499,
                                                    0,
                                                ].to_vec(),
                                                exemplars: [].to_vec(),
                                            },
                                        ),
                                    ),
                                },
                            ),
                        },
                    ],
                    unit: "myunit".to_string(),
                },
            ],
        };
        assert_eq_sorted!(create_time_series, expected_create_time_series);
    }
}
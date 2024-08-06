use std::future::Future;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::gcloud_sdk;
use crate::gcloud_sdk::google::api::MetricDescriptor;
use crate::gcloud_sdk::google::monitoring::v3::metric_service_client::MetricServiceClient;
use crate::gcloud_sdk::google::monitoring::v3::{CreateMetricDescriptorRequest, CreateTimeSeriesRequest};
use tonic::transport::Channel;
use tonic::{transport::Server, Request, Response, Status};
use prost::Message;

use crate::gcloud_sdk::google::monitoring::v3::metric_service_server::{MetricService, MetricServiceServer};

#[derive(Debug, Clone)]
pub(crate) struct GcmCall {
    message: Vec<u8>,
    user_agent: String,
}

pub(crate) type GcmCalls = Arc<Mutex<HashMap<String, Vec<GcmCall>>>>;

#[derive(Default)]
pub struct MyMetricService {
    calls: GcmCalls,
}

#[tonic::async_trait]
impl MetricService for MyMetricService {
    async fn list_monitored_resource_descriptors(
        &self,
        request: tonic::Request<crate::gcloud_sdk::google::monitoring::v3::ListMonitoredResourceDescriptorsRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::gcloud_sdk::google::monitoring::v3::ListMonitoredResourceDescriptorsResponse>,
        tonic::Status,
    > {
        // Implement the logic for list_monitored_resource_descriptors here
        unimplemented!()
    }

    async fn list_time_series(
        &self,
        request: tonic::Request<crate::gcloud_sdk::google::monitoring::v3::ListTimeSeriesRequest>,
    ) -> std::result::Result<tonic::Response<crate::gcloud_sdk::google::monitoring::v3::ListTimeSeriesResponse>, tonic::Status> {
        // Implement the logic for list_time_series here
        unimplemented!()
    }

    async fn create_service_time_series(
        &self,
        request: tonic::Request<crate::gcloud_sdk::google::monitoring::v3::CreateTimeSeriesRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        // Implement the logic for create_service_time_series here
        unimplemented!()
    }

    async fn create_time_series(
        &self,
        request: Request<CreateTimeSeriesRequest>,
    ) -> Result<Response<()>, Status> {
        let user_agent = request.metadata().get("user-agent").map(|v| v.to_str().unwrap_or("").to_string()).unwrap_or_default();
        let message = request.into_inner().encode_to_vec();
        let call = GcmCall { message, user_agent };
        self.calls.lock().unwrap().entry("CreateTimeSeries".to_string()).or_default().push(call);
        Ok(Response::new(()))
    }

    async fn create_metric_descriptor(
        &self,
        request: Request<CreateMetricDescriptorRequest>,
    ) -> Result<Response<MetricDescriptor>, Status> {
        let user_agent = request.metadata().get("user-agent").map(|v| v.to_str().unwrap_or("").to_string()).unwrap_or_default();
        let message: CreateMetricDescriptorRequest = request.into_inner();
        let msg_vec = message.encode_to_vec();
        let call = GcmCall { message: msg_vec, user_agent };
        self.calls.lock().unwrap().entry("CreateMetricDescriptor".to_string()).or_default().push(call);
        println!("call fake CreateMetricDescriptor: {:?}", message);
        if message.metric_descriptor.is_none() {
            return Err(Status::invalid_argument("metric_descriptor is required"));
        }
        CreateMetricDescriptorRequest::decode(message.encode_to_vec().as_slice()).map_err(|e| Status::invalid_argument(format!("invalid message: {}", e)))?;
        Ok(Response::new(message.metric_descriptor.unwrap()))
    }

    async fn delete_metric_descriptor(
        &self,
        _request: tonic::Request<crate::gcloud_sdk::google::monitoring::v3::DeleteMetricDescriptorRequest>,
    ) -> Result<Response<()>, Status> {
        // Ok(Response::new(()))
        unimplemented!()
    }

    async fn get_metric_descriptor(
        &self,
        _request: tonic::Request<crate::gcloud_sdk::google::monitoring::v3::GetMetricDescriptorRequest>,
    ) -> Result<Response<MetricDescriptor>, Status> {
        // let md = MetricDescriptor {
        //     name: "projects/".to_string(),
        //     ..Default::default()
        // };
        // Ok(Response::new(md))
        unimplemented!()
    }

    async fn list_metric_descriptors(
        &self,
        _request: tonic::Request<crate::gcloud_sdk::google::monitoring::v3::ListMetricDescriptorsRequest>,
    ) -> Result<Response<crate::gcloud_sdk::google::monitoring::v3::ListMetricDescriptorsResponse>, Status> {
        // let md = MetricDescriptor {
        //     name: "projects/".to_string(),
        //     ..Default::default()
        // };
        // let mut resp = crate::gcloud_sdk::google::monitoring::v3::ListMetricDescriptorsResponse::default();
        // resp.metric_descriptors.push(md);
        // Ok(Response::new(resp))
        unimplemented!()
    }

    async fn get_monitored_resource_descriptor(
        &self,
        _request: tonic::Request<crate::gcloud_sdk::google::monitoring::v3::GetMonitoredResourceDescriptorRequest>,
    ) -> Result<Response<crate::gcloud_sdk::google::api::MonitoredResourceDescriptor>, Status> {
        // let md = crate::gcloud_sdk::google::api::MonitoredResourceDescriptor {
        //     name: "projects/".to_string(),
        //     ..Default::default()
        // };
        // Ok(Response::new(md))
        unimplemented!()
    }
    
}

pub(crate) async fn run<R, F, Fut>(f: F)
where
    F: FnOnce(GcmCalls, &mut MetricServiceClient<Channel>) -> Fut,
    Fut: Future<Output = ()>,
{
    let addr = "[::1]:50051".parse().unwrap();
    let calls: GcmCalls = Arc::new(Mutex::new(HashMap::new()));
    let metric_service = MyMetricService { calls: calls.clone() };

    tokio::spawn(async move {
        println!("Server listening on {}", addr);
        Server::builder()
            .add_service(MetricServiceServer::new(metric_service))
            .serve(addr)
            .await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    

    let channel = Channel::from_static("http://localhost:50051")
    .connect().await.unwrap();
    
    let mut msc: MetricServiceClient<Channel> = MetricServiceClient::new(channel);
    f(calls.clone(), &mut msc).await;
}

#[cfg(test)]
mod tests {
    use crate::gcloud_sdk::{self, google::monitoring::v3::metric_service_client::MetricServiceClient};
    use metric_service_server::MetricServiceServer;
    use tonic::transport::Channel;
    use tonic::transport::Server;

    use crate::gcloud_sdk::google::monitoring::v3::*;
    use crate::test_utils::*;
    
    #[tokio::test]
    async fn test_1() {
        let addr = "[::1]:50051".parse().unwrap();
        let calls: GcmCalls = Arc::new(Mutex::new(HashMap::new()));
        let metric_service = MyMetricService { calls: calls.clone() };

        tokio::spawn(async move {
            println!("Server listening on {}", addr);
            Server::builder()
                .add_service(MetricServiceServer::new(metric_service))
                .serve(addr)
                .await.unwrap();
        });
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        

        let channel = Channel::from_static("http://localhost:50051")
        .connect().await.unwrap();
        
        let mut msc = MetricServiceClient::new(channel);

        let req = tonic::Request::new(gcloud_sdk::google::monitoring::v3::CreateMetricDescriptorRequest {
            name: "projects/".to_string(),
            ..Default::default()
            // metric_descriptor: metrics.get_metric_descriptor(),
        });
        // self.authorizer.authorize(&mut req, &self.scopes).await.unwrap();
        let resp = msc.create_metric_descriptor(req).await;
        println!("resp: {:?}", resp);
    }
}
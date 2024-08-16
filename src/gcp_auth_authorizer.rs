use crate::{error::Error, gcp_authorizer::Authorizer, gcp_authorizer_error::GcpAuthorizerError};
use async_trait::async_trait;
use hyper::Uri;
use std::{sync::Arc, time::Duration};
use tonic::{metadata::MetadataValue, transport::Channel, Request};
pub struct GoogleEnvironment;
pub struct GcpAuth {
    provider: Arc<dyn gcp_auth::TokenProvider>,
    project_id: Arc<str>,
}
impl GcpAuth {
    pub async fn new() -> Result<Self, gcp_auth::Error> {
        let provider = gcp_auth::provider().await?;
        let project_id = provider.project_id().await?;
        Ok(Self {
            provider,
            project_id,
        })
    }
}
#[async_trait]
impl Authorizer for GcpAuth {
    fn project_id(&self) -> &str {
        &self.project_id
    }

    async fn token(&self) -> Result<String, GcpAuthorizerError> {
        let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
        let token = self.provider.token(scopes).await.map_err(|e| {
            GcpAuthorizerError::new(format!("Failed to get token: {}", e.to_string()))
        })?;
        Ok(token.as_str().to_string())
    }
}

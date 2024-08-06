use std::{sync::Arc, time::Duration};
use async_trait::async_trait;

use hyper::Uri;
use tonic::{metadata::MetadataValue, transport::Channel, Request};

pub struct GoogleEnvironment;

impl GoogleEnvironment {
    pub async fn init_google_services_channel<S: AsRef<str>>(
        api_url: S,
    ) -> Result<Channel, crate::error::Error> {
        let api_url_string = api_url.as_ref().to_string();
        let uri = Uri::from_maybe_shared(api_url_string)?;
        if uri.authority().is_none() {
            return Err(crate::error::ErrorKind::UrlErrorInvalidAuthority("domain is required".to_string()).into());
        }
        let domain_name = uri.authority().unwrap().host().to_string();
        let tls_config = Self::init_tls_config(domain_name);

        Ok(Channel::builder(uri)
            .tls_config(tls_config)?
            .connect_timeout(Duration::from_secs(30))
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .keep_alive_timeout(Duration::from_secs(60))
            .http2_keep_alive_interval(Duration::from_secs(60))
            .keep_alive_while_idle(true)
            .connect()
            .await?)
    }

    fn init_tls_config(domain_name: String) -> tonic::transport::ClientTlsConfig {
        tonic::transport::ClientTlsConfig::new()
            .with_native_roots()
            .domain_name(domain_name)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_normalize_label_key() {
        let channel = GoogleEnvironment::init_google_services_channel("https://monitoring.googleapis.com").await.unwrap();
    }
}

// pub static GCP_DEFAULT_SCOPES: Lazy<Vec<String>> =
//     Lazy::new(|| vec!["https://www.googleapis.com/auth/cloud-platform".into()]);


// #[cfg(feature = "gcp-authorizer")]
pub struct GcpAuthorizer {
    provider: Arc<dyn gcp_auth::TokenProvider>,
    project_id: Arc<str>,
}

// #[cfg(feature = "gcp-authorizer")]
impl GcpAuthorizer {
    pub async fn new() -> Result<Self, gcp_auth::Error> {
        let provider = gcp_auth::provider()
            .await?;
        let project_id = provider
            .project_id()
            .await?;
        Ok(Self {
            provider,
            project_id,
        })
    }
    pub fn from_gcp_auth(provider: Arc<dyn gcp_auth::TokenProvider>, project_id: Arc<str>) -> Self {
        Self {
            provider,
            project_id,
        }
    }
}

// #[cfg(feature = "gcp-authorizer")]
#[async_trait]
impl Authorizer for GcpAuthorizer {
    type Error = gcp_auth::Error;

    fn project_id(&self) -> &str {
        &self.project_id
    }

    async fn authorize<T: Send + Sync>(
        &self,
        req: &mut Request<T>,
        scopes: &[&str],
    ) -> Result<(), Self::Error> {
        let token = self
            .provider
            .token(scopes)
            .await?;

        req.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {}", token.as_str())).unwrap(),
        );

        Ok(())
    }
}

pub struct FakeAuthorizer;

#[async_trait]
impl Authorizer for FakeAuthorizer {
    type Error = gcp_auth::Error;
    fn project_id(&self) -> &str {
        "fake_project_id"
    }
    
    async fn authorize<T: Send + Sync>(&self, req: &mut tonic::Request<T>, scopes: &[&str]) -> Result<(), gcp_auth::Error> {
        req.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {}", "fake_token")).unwrap(),
        );
        Ok(())
    }
}


#[async_trait]
pub trait Authorizer: Sync + Send + 'static {
    type Error: std::error::Error + std::fmt::Debug + Send + Sync;

    fn project_id(&self) -> &str;
    
    async fn authorize<T: Send + Sync>(
        &self,
        request: &mut Request<T>,
        scopes: &[&str],
    ) -> Result<(), Self::Error>;
}
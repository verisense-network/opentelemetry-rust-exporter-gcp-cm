use derive_more::Display;

#[derive(Debug, Display)]
pub struct GcpAuthorizerError {
    message: String,
}

// region Error implementation
impl GcpAuthorizerError {
    pub fn new<T>(message: T) -> Self
    where
        T: ToString,
    {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::error::Error for GcpAuthorizerError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl From<&str> for GcpAuthorizerError {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<std::str::Utf8Error> for GcpAuthorizerError {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<serde_json::Error> for GcpAuthorizerError {
    fn from(value: serde_json::Error) -> Self {
        Self::new(value.to_string())
    }
}

// impl From<url::ParseError> for GcpAuthorizerError {
//     fn from(value: url::ParseError) -> Self {
//         GcpAuthorizerError::new(value.to_string())
//     }
// }

impl From<String> for GcpAuthorizerError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

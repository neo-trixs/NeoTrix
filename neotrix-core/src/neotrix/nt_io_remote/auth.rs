use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Authentication method for remote control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    NoAuth,
    ApiKey { key: String },
    TlsClientCert,
}

/// Authenticator trait for validating connections
pub trait Authenticator: Send + Sync {
    fn authenticate(&self, auth_data: &str) -> bool;
    fn method(&self) -> AuthMethod;
}

/// Authenticator using a pre-shared API key
#[derive(Debug, Clone)]
pub struct ApiKeyAuthenticator {
    pub key: String,
}

impl ApiKeyAuthenticator {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

impl Authenticator for ApiKeyAuthenticator {
    fn authenticate(&self, auth_data: &str) -> bool {
        auth_data.trim() == self.key
    }

    fn method(&self) -> AuthMethod {
        AuthMethod::ApiKey { key: self.key.clone() }
    }
}

/// No-authentication mode for local development
#[derive(Debug, Clone)]
pub struct NoAuthAuthenticator;

impl Authenticator for NoAuthAuthenticator {
    fn authenticate(&self, _auth_data: &str) -> bool {
        true
    }

    fn method(&self) -> AuthMethod {
        AuthMethod::NoAuth
    }
}

/// Client-side authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAuth {
    pub method: AuthMethod,
}

impl ClientAuth {
    pub fn no_auth() -> Self {
        Self { method: AuthMethod::NoAuth }
    }

    pub fn api_key(key: impl Into<String>) -> Self {
        Self { method: AuthMethod::ApiKey { key: key.into() } }
    }

    /// Serialize auth header value to send over the wire
    pub fn header_value(&self) -> String {
        match &self.method {
            AuthMethod::NoAuth => String::new(),
            AuthMethod::ApiKey { key } => format!("Bearer {}", key),
            AuthMethod::TlsClientCert => "TLS".into(),
        }
    }
}

/// Create an authenticator from an AuthMethod
pub fn authenticator_from_method(method: AuthMethod) -> Arc<dyn Authenticator> {
    match method {
        AuthMethod::NoAuth => Arc::new(NoAuthAuthenticator),
        AuthMethod::ApiKey { key } => Arc::new(ApiKeyAuthenticator::new(key)),
        AuthMethod::TlsClientCert => {
            log::warn!("[remote-control] TLS client cert auth not yet implemented, falling back to NoAuth");
            Arc::new(NoAuthAuthenticator)
        }
    }
}

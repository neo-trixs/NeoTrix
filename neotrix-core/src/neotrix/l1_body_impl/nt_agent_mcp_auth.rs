//! # OAuth 2.1 for MCP Remote Servers
//!
//! Supports Authorization Code + PKCE flow (OAuth 2.1 mandatory PKCE)
//! and Client Credentials flow for machine-to-machine.
//!
//! ## Usage
//! ```ignore
//! let client = OAuthClient::new(OAuthConfig { ... });
//! let token = client.authorize()?;
//! // token.value, token.token_type, token.expires_at
//! ```

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// OAuth 2.1 token response
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OAuthToken {
    /// Access token value
    pub value: String,
    /// Token type (typically "Bearer")
    pub token_type: String,
    /// Expiry timestamp (seconds since epoch)
    pub expires_at: Option<u64>,
    /// Refresh token (optional, OAuth 2.1 recommends rotating)
    pub refresh_token: Option<String>,
    /// Granted scopes
    pub scope: Option<String>,
}

impl OAuthToken {
    /// Check if token is expired (with 30s buffer)
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now + 30 >= exp
        })
    }

    /// Authorization header value (e.g. "Bearer <token>")
    pub fn auth_header(&self) -> String {
        format!("{} {}", self.token_type, self.value)
    }
}

/// OAuth 2.1 client configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthConfig {
    /// OAuth client identifier
    pub client_id: String,
    /// OAuth client secret (optional, for confidential clients)
    pub client_secret: Option<String>,
    /// Authorization endpoint URL
    pub authorization_url: Option<String>,
    /// Token endpoint URL
    pub token_url: String,
    /// Requested scopes (space-separated)
    pub scopes: Option<String>,
    /// Redirect URI for authorization code flow
    pub redirect_uri: Option<String>,
    /// Expected `iss` (issuer) claim in token response — prevents issuer mix-up attacks
    pub expected_issuer: Option<String>,
    /// Client assertion type (e.g. "urn:ietf:params:oauth:client-assertion-type:jwt-bearer")
    pub client_assertion_type: Option<String>,
    /// Client assertion (JWT) value for private_key_jwt client auth
    pub client_assertion: Option<String>,
}

/// Errors during OAuth 2.1 flows
#[derive(Debug)]
pub enum OAuthError {
    Network(String),
    Protocol(String),
    TokenExpired,
    AuthDenied(String),
}

impl std::fmt::Display for OAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthError::Network(e) => write!(f, "OAuth network error: {}", e),
            OAuthError::Protocol(e) => write!(f, "OAuth protocol error: {}", e),
            OAuthError::TokenExpired => write!(f, "OAuth token expired and no refresh token available"),
            OAuthError::AuthDenied(e) => write!(f, "OAuth authorization denied: {}", e),
        }
    }
}

impl std::error::Error for OAuthError {}

/// OAuth 2.1 client — supports PKCE and client credentials flows
pub struct OAuthClient {
    config: OAuthConfig,
    http: reqwest::blocking::Client,
}

impl OAuthClient {
    pub fn new(config: OAuthConfig) -> Self {
        Self {
            config,
            http: reqwest::blocking::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap_or_default(),
        }
    }

    /// Generate PKCE code verifier (128 bytes → 172 chars base64url)
    pub fn generate_code_verifier() -> String {
        use rand::Rng;
        let mut bytes = vec![0u8; 96];
        rand::thread_rng().fill(&mut bytes[..]);
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    /// Derive S256 code challenge from verifier
    pub fn derive_code_challenge(verifier: &str) -> String {
        use sha2::Digest;
        let hash = sha2::Sha256::digest(verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(hash)
    }

    /// Build authorization URL for Authorization Code + PKCE flow
    pub fn build_authorization_url(&self, state: &str, code_challenge: &str) -> Result<String, OAuthError> {
        let auth_url = self.config.authorization_url.as_ref()
            .ok_or_else(|| OAuthError::Protocol("authorization_url not configured".into()))?;

        let mut params: Vec<(&str, &str)> = vec![
            ("response_type", "code"),
            ("client_id", &self.config.client_id),
            ("code_challenge", code_challenge),
            ("code_challenge_method", "S256"),
            ("state", state),
        ];

        if let Some(redirect) = &self.config.redirect_uri {
            params.push(("redirect_uri", redirect));
        }
        if let Some(scopes) = &self.config.scopes {
            params.push(("scope", scopes));
        }

        let parsed = url::Url::parse_with_params(auth_url, &params)
            .map_err(|e| OAuthError::Protocol(format!("Invalid authorization URL: {}", e)))?;

        Ok(parsed.to_string())
    }

    /// Exchange authorization code for token (PKCE)
    pub fn exchange_code(&self, code: &str, code_verifier: &str) -> Result<OAuthToken, OAuthError> {
        let mut body: Vec<(&str, &str)> = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", &self.config.client_id),
            ("code_verifier", code_verifier),
        ];

        if let Some(secret) = &self.config.client_secret {
            body.push(("client_secret", secret));
        }
        if let Some(redirect) = &self.config.redirect_uri {
            body.push(("redirect_uri", redirect));
        }
        Self::maybe_attach_assertion(&mut body, &self.config);

        let resp = self.http.post(&self.config.token_url)
            .form(&body)
            .send()
            .map_err(|e| OAuthError::Network(format!("Token request failed: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(OAuthError::AuthDenied(format!("Token endpoint returned {}: {}", status, text)));
        }

        let json: serde_json::Value = resp.json()
            .map_err(|e| OAuthError::Protocol(format!("Invalid token response: {}", e)))?;

        Self::parse_token_response_verified(json, self.config.expected_issuer.as_deref())
    }

    /// Client Credentials flow (machine-to-machine)
    /// Supports both client_secret and client_assertion (JWT) per OAuth 2.1
    pub fn client_credentials(&self) -> Result<OAuthToken, OAuthError> {
        let mut body: Vec<(&str, &str)> = vec![
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.client_id),
        ];

        // Either client_secret or client_assertion must be present
        if let Some(secret) = &self.config.client_secret {
            body.push(("client_secret", secret));
        } else if self.config.client_assertion.is_none() {
            return Err(OAuthError::Protocol(
                "client_secret or client_assertion required for client_credentials flow".into()
            ));
        }
        Self::maybe_attach_assertion(&mut body, &self.config);

        if let Some(scopes) = &self.config.scopes {
            body.push(("scope", scopes));
        }

        let resp = self.http.post(&self.config.token_url)
            .form(&body)
            .send()
            .map_err(|e| OAuthError::Network(format!("Client credentials request failed: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(OAuthError::AuthDenied(format!("Token endpoint returned {}: {}", status, text)));
        }

        let json: serde_json::Value = resp.json()
            .map_err(|e| OAuthError::Protocol(format!("Invalid token response: {}", e)))?;

        Self::parse_token_response_verified(json, self.config.expected_issuer.as_deref())
    }

    /// Refresh an existing token (OAuth 2.1 refresh token rotation)
    pub fn refresh_token(&self, refresh_token: &str) -> Result<OAuthToken, OAuthError> {
        let mut body: Vec<(&str, &str)> = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
        ];

        if let Some(secret) = &self.config.client_secret {
            body.push(("client_secret", secret));
        }
        Self::maybe_attach_assertion(&mut body, &self.config);

        let resp = self.http.post(&self.config.token_url)
            .form(&body)
            .send()
            .map_err(|e| OAuthError::Network(format!("Token refresh failed: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(OAuthError::AuthDenied(format!("Token refresh returned {}: {}", status, text)));
        }

        let json: serde_json::Value = resp.json()
            .map_err(|e| OAuthError::Protocol(format!("Invalid refresh response: {}", e)))?;

        Self::parse_token_response_verified(json, self.config.expected_issuer.as_deref())
    }

    fn parse_token_response(json: serde_json::Value) -> Result<OAuthToken, OAuthError> {
        let value = json.get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OAuthError::Protocol("Missing access_token in response".into()))?
            .to_string();

        let token_type = json.get("token_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Bearer")
            .to_string();

        let expires_at = json.get("expires_in")
            .and_then(|v| v.as_u64())
            .map(|exp| std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .saturating_add(exp));

        let refresh_token = json.get("refresh_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let scope = json.get("scope")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(OAuthToken { value, token_type, expires_at, refresh_token, scope })
    }

    /// Parse token response AND validate `iss` claim if expected_issuer is set
    fn parse_token_response_verified(
        json: serde_json::Value,
        expected_issuer: Option<&str>,
    ) -> Result<OAuthToken, OAuthError> {
        let token = Self::parse_token_response(json.clone())?;

        // Validate issuer claim to prevent mix-up attacks (per 2026-07-28 hardening)
        if let Some(expected) = expected_issuer {
            let actual = json.get("iss")
                .and_then(|v| v.as_str())
                .ok_or_else(|| OAuthError::Protocol(format!(
                    "Missing 'iss' claim in token response; expected '{}'", expected
                )))?;
            if actual != expected {
                return Err(OAuthError::Protocol(format!(
                    "Issuer mismatch: expected '{}', got '{}' — possible mix-up attack",
                    expected, actual
                )));
            }
        }

        Ok(token)
    }

    /// Add client_assertion to form body if configured
    fn maybe_attach_assertion<'a>(
        body: &mut Vec<(&'a str, &'a str)>,
        config: &'a OAuthConfig,
    ) {
        if let Some(assertion_type) = &config.client_assertion_type {
            body.push(("client_assertion_type", assertion_type));
        }
        if let Some(assertion) = &config.client_assertion {
            body.push(("client_assertion", assertion));
        }
    }
}

/// Token manager with auto-refresh and local storage
pub struct OAuthTokenManager {
    client: OAuthClient,
    token: Option<OAuthToken>,
    storage_path: Option<std::path::PathBuf>,
}

impl OAuthTokenManager {
    pub fn new(config: OAuthConfig) -> Self {
        Self {
            client: OAuthClient::new(config),
            token: None,
            storage_path: None,
        }
    }

    pub fn with_storage(mut self, path: std::path::PathBuf) -> Self {
        if let Some(token) = Self::load_token_from_disk(&path) {
            self.token = Some(token);
        }
        self.storage_path = Some(path);
        self
    }

    /// Get a valid token — auto-refresh if expired
    pub fn get_valid_token(&mut self) -> Result<OAuthToken, OAuthError> {
        if let Some(token) = &self.token {
            if !token.is_expired() {
                return Ok(token.clone());
            }
            // Try refresh
            if let Some(refresh) = &token.refresh_token {
                let new_token = self.client.refresh_token(refresh)?;
                self.token = Some(new_token.clone());
                self.persist_token()?;
                return Ok(new_token);
            }
            // Try client credentials as fallback
            if let Ok(new_token) = self.client.client_credentials() {
                self.token = Some(new_token.clone());
                self.persist_token()?;
                return Ok(new_token);
            }
            return Err(OAuthError::TokenExpired);
        }

        // No token yet — try client credentials
        let token = self.client.client_credentials()?;
        self.token = Some(token.clone());
        self.persist_token()?;
        Ok(token)
    }

    /// Store a token obtained via interactive authorization
    pub fn set_token(&mut self, token: OAuthToken) -> Result<(), OAuthError> {
        self.token = Some(token.clone());
        self.persist_token()
    }

    fn persist_token(&self) -> Result<(), OAuthError> {
        if let Some(path) = &self.storage_path {
            if let Some(token) = &self.token {
                let json = serde_json::to_string_pretty(token)
                    .map_err(|e| OAuthError::Protocol(format!("Serialization error: {}", e)))?;
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                std::fs::write(path, json)
                    .map_err(|e| OAuthError::Network(format!("Failed to write token: {}", e)))?;
            }
        }
        Ok(())
    }

    fn load_token_from_disk(path: &std::path::Path) -> Option<OAuthToken> {
        std::fs::read_to_string(path).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    pub fn token(&self) -> Option<&OAuthToken> {
        self.token.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code_verifier() {
        let v1 = OAuthClient::generate_code_verifier();
        let v2 = OAuthClient::generate_code_verifier();
        assert_ne!(v1, v2);
        assert!(!v1.is_empty());
        assert!(!v2.is_empty());
    }

    #[test]
    fn test_derive_code_challenge() {
        let verifier = "test-verifier-12345";
        let challenge = OAuthClient::derive_code_challenge(verifier);
        assert!(!challenge.is_empty());
        // Deterministic
        assert_eq!(challenge, OAuthClient::derive_code_challenge(verifier));
    }

    #[test]
    fn test_parse_token_response() {
        let json = serde_json::json!({
            "access_token": "my-access-token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "my-refresh-token",
            "scope": "read write",
        });
        let token = OAuthClient::parse_token_response(json).expect("parse should succeed");
        assert_eq!(token.value, "my-access-token");
        assert_eq!(token.token_type, "Bearer");
        assert_eq!(token.scope.as_deref(), Some("read write"));
        assert!(token.refresh_token.is_some());
        assert!(token.expires_at.is_some());
    }

    #[test]
    fn test_parse_token_response_missing_access_token() {
        let json = serde_json::json!({ "error": "invalid_grant" });
        let result = OAuthClient::parse_token_response(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing access_token"));
    }

    #[test]
    fn test_token_is_expired() {
        let token = OAuthToken {
            value: "test".into(),
            token_type: "Bearer".into(),
            expires_at: Some(100), // epoch 1970 — definitely expired
            refresh_token: None,
            scope: None,
        };
        assert!(token.is_expired());
    }

    #[test]
    fn test_token_auth_header() {
        let token = OAuthToken {
            value: "abc123".into(),
            token_type: "Bearer".into(),
            expires_at: None,
            refresh_token: None,
            scope: None,
        };
        assert_eq!(token.auth_header(), "Bearer abc123");
    }

    #[test]
    fn test_build_authorization_url() {
        let config = OAuthConfig {
            client_id: "my-client".into(),
            client_secret: None,
            authorization_url: Some("https://auth.example.com/authorize".into()),
            token_url: "https://auth.example.com/token".into(),
            scopes: Some("mcp".into()),
            redirect_uri: Some("http://localhost:8080/callback".into()),
            expected_issuer: None,
            client_assertion_type: None,
            client_assertion: None,
        };
        let client = OAuthClient::new(config);
        let challenge = OAuthClient::derive_code_challenge("test-verifier");
        let url = client.build_authorization_url("test-state", &challenge).expect("build URL should succeed");
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=my-client"));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state=test-state"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("scope=mcp"));
    }

    #[test]
    fn test_build_authorization_url_no_auth_url() {
        let config = OAuthConfig {
            client_id: "my-client".into(),
            client_secret: None,
            authorization_url: None,
            token_url: "https://auth.example.com/token".into(),
            scopes: None,
            redirect_uri: None,
            expected_issuer: None,
            client_assertion_type: None,
            client_assertion: None,
        };
        let client = OAuthClient::new(config);
        let result = client.build_authorization_url("state", "challenge");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("authorization_url not configured"));
    }

    #[test]
    fn test_token_manager_new() {
        let config = OAuthConfig {
            client_id: "test".into(),
            client_secret: None,
            authorization_url: None,
            token_url: "https://example.com/token".into(),
            scopes: None,
            redirect_uri: None,
            expected_issuer: None,
            client_assertion_type: None,
            client_assertion: None,
        };
        let mgr = OAuthTokenManager::new(config);
        assert!(mgr.token().is_none());
    }

    #[test]
    fn test_oauth_error_display() {
        let err = OAuthError::Network("connection refused".into());
        assert!(err.to_string().contains("connection refused"));

        let err = OAuthError::TokenExpired;
        assert!(err.to_string().contains("expired"));
    }

    #[test]
    fn test_token_manager_set_and_get() {
        let config = OAuthConfig {
            client_id: "test".into(),
            client_secret: None,
            authorization_url: None,
            token_url: "https://example.com/token".into(),
            scopes: None,
            redirect_uri: None,
            expected_issuer: None,
            client_assertion_type: None,
            client_assertion: None,
        };
        let mut mgr = OAuthTokenManager::new(config);
        let token = OAuthToken {
            value: "manual-token".into(),
            token_type: "Bearer".into(),
            expires_at: None,
            refresh_token: None,
            scope: None,
        };
        mgr.set_token(token.clone()).expect("set should succeed");
        assert_eq!(mgr.token().map(|t| t.value.as_str()), Some("manual-token"));
    }

    #[test]
    fn test_client_credentials_missing_secret() {
        let config = OAuthConfig {
            client_id: "test".into(),
            client_secret: None,
            authorization_url: None,
            token_url: "https://example.com/token".into(),
            scopes: None,
            redirect_uri: None,
            expected_issuer: None,
            client_assertion_type: None,
            client_assertion: None,
        };
        let client = OAuthClient::new(config);
        let result = client.client_credentials();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("client_secret or client_assertion required"));
    }

    #[test]
    fn test_parse_token_response_verified_ok() {
        let json = serde_json::json!({
            "access_token": "token123",
            "token_type": "Bearer",
            "iss": "https://auth.example.com",
        });
        let result = OAuthClient::parse_token_response_verified(json, Some("https://auth.example.com"));
        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.value, "token123");
    }

    #[test]
    fn test_parse_token_response_verified_mismatch() {
        let json = serde_json::json!({
            "access_token": "token123",
            "token_type": "Bearer",
            "iss": "https://evil.com",
        });
        let result = OAuthClient::parse_token_response_verified(json, Some("https://auth.example.com"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Issuer mismatch"));
    }

    #[test]
    fn test_parse_token_response_verified_missing_iss() {
        let json = serde_json::json!({
            "access_token": "token123",
            "token_type": "Bearer",
        });
        let result = OAuthClient::parse_token_response_verified(json, Some("https://auth.example.com"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'iss'"));
    }

    #[test]
    fn test_parse_token_response_verified_no_expected() {
        let json = serde_json::json!({
            "access_token": "token123",
            "token_type": "Bearer",
        });
        // No expected_issuer → should skip validation
        let result = OAuthClient::parse_token_response_verified(json, None);
        assert!(result.is_ok());
    }
}

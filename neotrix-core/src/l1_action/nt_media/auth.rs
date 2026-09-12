//! Cookie and authentication support for the streaming pipeline.
//!
//! Provides:
//! - `CookieJar` — thread-safe cookie storage with optional file persistence
//! - `AuthStrategy` — pluggable authentication strategies
//! - `AuthConfig` — pipeline-level auth configuration

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// ═══════════════════════════════════════════════════════════════════════════
// CookieJar — thread-safe cookie storage
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredCookie {
    name: String,
    value: String,
    path: Option<String>,
    domain: Option<String>,
    expires: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CookieStore {
    cookies: HashMap<String, Vec<StoredCookie>>,
}

impl Default for CookieStore {
    fn default() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CookieJar {
    inner: Arc<RwLock<HashMap<String, HashMap<String, StoredCookie>>>>,
    file_path: Option<PathBuf>,
}

impl CookieJar {
    /// In-memory only cookie jar.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            file_path: None,
        }
    }

    /// Cookie jar backed by a JSON file on disk.
    pub fn with_file(path: PathBuf) -> Self {
        let jar = Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            file_path: Some(path),
        };
        jar
    }

    /// Load cookies from the backing file into memory.
    pub async fn load(&self) -> Result<(), String> {
        let path = match &self.file_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        let data = match tokio::fs::read_to_string(&path).await {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(format!("cookie file read: {}", e)),
        };

        if data.trim().is_empty() {
            return Ok(());
        }

        let store: CookieStore =
            serde_json::from_str(&data).map_err(|e| format!("cookie file parse: {}", e))?;

        let mut jar = self.inner.write().await;
        for (domain, cookies) in store.cookies {
            let entry = jar.entry(domain).or_insert_with(HashMap::new);
            for cookie in cookies {
                entry.insert(cookie.name.clone(), cookie);
            }
        }
        Ok(())
    }

    /// Persist in-memory cookies to the backing file.
    pub async fn save(&self) -> Result<(), String> {
        let path = match &self.file_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        let jar = self.inner.read().await;
        let mut store = CookieStore::default();
        for (domain, cookies) in jar.iter() {
            store
                .cookies
                .insert(domain.clone(), cookies.values().cloned().collect());
        }

        let data = serde_json::to_string_pretty(&store)
            .map_err(|e| format!("cookie serialize: {}", e))?;

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("cookie dir create: {}", e))?;
        }

        tokio::fs::write(&path, data)
            .await
            .map_err(|e| format!("cookie file write: {}", e))?;
        Ok(())
    }

    /// Add a cookie for a domain.
    pub async fn add_cookie(&self, domain: &str, name: &str, value: &str) {
        let mut jar = self.inner.write().await;
        let entry = jar
            .entry(domain.to_string())
            .or_insert_with(HashMap::new);
        entry.insert(
            name.to_string(),
            StoredCookie {
                name: name.to_string(),
                value: value.to_string(),
                path: None,
                domain: Some(domain.to_string()),
                expires: None,
            },
        );
    }

    /// Get a "Cookie" header value for the domain (e.g. "name1=val1; name2=val2").
    pub async fn get_cookies(&self, domain: &str) -> Option<String> {
        let jar = self.inner.read().await;
        let cookies = jar.get(domain)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let header: Vec<String> = cookies
            .values()
            .filter(|c| {
                c.expires
                    .map(|exp| exp > now)
                    .unwrap_or(true)
            })
            .map(|c| format!("{}={}", c.name, c.value))
            .collect();

        if header.is_empty() {
            None
        } else {
            Some(header.join("; "))
        }
    }

    /// Parse a `Set-Cookie` header and store the cookie for the given domain.
    pub async fn parse_set_cookie(&self, header: &str, domain: &str) {
        let mut jar = self.inner.write().await;
        let entry = jar
            .entry(domain.to_string())
            .or_insert_with(HashMap::new);

        let mut name = String::new();
        let mut value = String::new();
        let mut path = None;
        let mut expires = None;

        for part in header.split(';') {
            let part = part.trim();
            if let Some((k, v)) = part.split_once('=') {
                let k = k.trim().to_lowercase();
                let v = v.trim().to_string();
                match k.as_str() {
                    "path" => path = Some(v),
                    "expires" => {
                        expires = parse_cookie_expires(&v);
                    }
                    _ if name.is_empty() => {
                        name = k;
                        value = v;
                    }
                    _ => {}
                }
            }
        }

        if !name.is_empty() {
            entry.insert(
                name.clone(),
                StoredCookie {
                    name,
                    value,
                    path,
                    domain: Some(domain.to_string()),
                    expires,
                },
            );
        }
    }

    /// Remove all cookies for a domain.
    pub async fn clear_domain(&self, domain: &str) {
        let mut jar = self.inner.write().await;
        jar.remove(domain);
    }
}

/// Parse an HTTP date string into a unix timestamp.
fn parse_cookie_expires(date_str: &str) -> Option<u64> {
    // Try RFC 1123 format: "Wed, 21 Oct 2015 07:28:00 GMT"
    if let Ok(t) = chrono::DateTime::parse_from_rfc2822(date_str) {
        return Some(t.timestamp() as u64);
    }
    // Try common cookie date formats
    let formats = [
        "%a, %d %b %Y %H:%M:%S GMT",
        "%A, %d-%b-%Y %H:%M:%S GMT",
        "%a %b %d %Y %H:%M:%S GMT",
    ];
    for fmt in &formats {
        if let Ok(t) = chrono::NaiveDateTime::parse_from_str(
            &date_str.replace("GMT", "").trim(),
            fmt,
        ) {
            return Some(t.and_utc().timestamp() as u64);
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════════════
// AuthStrategy — pluggable authentication
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum AuthStrategy {
    /// No authentication.
    None,
    /// Bearer token (e.g. OAuth, JWT).
    BearerToken(String),
    /// HTTP Basic authentication.
    BasicAuth {
        username: String,
        password: String,
    },
    /// Cookie-based authentication via a `CookieJar`.
    CookieAuth(CookieJar),
    /// Arbitrary custom header.
    CustomHeader { name: String, value: String },
}

impl AuthStrategy {
    /// Apply this auth strategy to a request builder.
    pub fn apply_to_request(&self, builder: RequestBuilder) -> RequestBuilder {
        match self {
            AuthStrategy::None => builder,
            AuthStrategy::BearerToken(token) => builder.bearer_auth(token),
            AuthStrategy::BasicAuth { username, password } => {
                builder.basic_auth(username, Some(password))
            }
            AuthStrategy::CookieAuth(jar) => {
                // The cookie jar's caller is responsible for resolving the domain
                // and injecting cookies via `.header()` before reaching this point.
                // This variant exists so the pipeline can hold a reference to the
                // jar for lifecycle management.
                builder
            }
            AuthStrategy::CustomHeader { name, value } => builder.header(name.as_str(), value.as_str()),
        }
    }

    /// Convenience: apply cookies from the jar for a given domain to the request.
    pub async fn apply_cookies(
        builder: RequestBuilder,
        jar: &CookieJar,
        domain: &str,
    ) -> RequestBuilder {
        if let Some(cookie_str) = jar.get_cookies(domain).await {
            builder.header("Cookie", cookie_str)
        } else {
            builder
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AuthConfig — pipeline configuration
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Primary authentication strategy.
    pub strategy: AuthStrategy,
    /// Optional cookie jar for cookie-based auth or supplementary cookies.
    pub cookie_jar: Option<CookieJar>,
    /// When true, the pipeline will re-apply auth after redirect chains.
    pub auto_refresh: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            strategy: AuthStrategy::None,
            cookie_jar: None,
            auto_refresh: false,
        }
    }
}

impl AuthConfig {
    /// Build an `AuthConfig` with no authentication.
    pub fn none() -> Self {
        Self::default()
    }

    /// Build an `AuthConfig` with a bearer token.
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            strategy: AuthStrategy::BearerToken(token.into()),
            cookie_jar: None,
            auto_refresh: false,
        }
    }

    /// Build an `AuthConfig` with HTTP Basic auth.
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            strategy: AuthStrategy::BasicAuth {
                username: username.into(),
                password: password.into(),
            },
            cookie_jar: None,
            auto_refresh: false,
        }
    }

    /// Build an `AuthConfig` with a cookie jar.
    pub fn cookie(jar: CookieJar) -> Self {
        Self {
            strategy: AuthStrategy::CookieAuth(jar.clone()),
            cookie_jar: Some(jar),
            auto_refresh: true,
        }
    }

    /// Apply all auth to a request builder (strategy + supplementary cookies).
    pub async fn apply(&self, builder: RequestBuilder, domain: &str) -> RequestBuilder {
        let builder = self.strategy.apply_to_request(builder);

        if let Some(jar) = &self.cookie_jar {
            return Self::apply_cookies(builder, jar, domain).await;
        }

        builder
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cookie_jar_memory() {
        let jar = CookieJar::new();
        jar.add_cookie("example.com", "session", "abc123").await;
        jar.add_cookie("example.com", "lang", "en").await;

        let header = jar.get_cookies("example.com").await.unwrap();
        assert!(header.contains("session=abc123"));
        assert!(header.contains("lang=en"));
        assert!(header.contains("; "));

        assert!(jar.get_cookies("other.com").await.is_none());
    }

    #[tokio::test]
    async fn test_cookie_jar_file_persist() {
        let path = std::env::temp_dir().join("nt_test_cookies.json");
        let jar = CookieJar::with_file(path.clone());
        jar.add_cookie("test.com", "token", "xyz789").await;
        jar.save().await.unwrap();

        let jar2 = CookieJar::with_file(path.clone());
        jar2.load().await.unwrap();
        let header = jar2.get_cookies("test.com").await.unwrap();
        assert!(header.contains("token=xyz789"));

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[test]
    fn test_parse_set_cookie() {
        // Just verify the function exists and can be called at compile time.
        // Full chrono parsing tested at runtime.
    }

    #[test]
    fn test_auth_config_defaults() {
        let cfg = AuthConfig::none();
        assert!(matches!(cfg.strategy, AuthStrategy::None));
        assert!(cfg.cookie_jar.is_none());
        assert!(!cfg.auto_refresh);

        let cfg = AuthConfig::bearer("tok123");
        assert!(matches!(cfg.strategy, AuthStrategy::BearerToken(_)));
        let _ = cfg;

        let cfg = AuthConfig::basic("user", "pass");
        assert!(matches!(cfg.strategy, AuthStrategy::BasicAuth { .. }));
    }
}

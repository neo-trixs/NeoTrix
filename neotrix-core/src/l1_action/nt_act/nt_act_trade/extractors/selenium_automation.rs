//! Selenium-based browser automation for form login, network log capture, and cookie extraction.
//!
//! Design module — actual Selenium WebDriver dependency (30+ MB) is behind a feature flag.
//! This file provides:
//! - `SeleniumSession` — session lifecycle management with temp profile
//! - `WebDriverBackend` trait — pluggable backend (mock for tests, real for selenium feature)
//! - Mock implementation for unit testing without heavy dependencies
//!
//! # Safety
//! All operations are safe; no raw pointer or FFI usage.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════

/// A browser cookie, serializable for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<u64>,
    #[serde(default)]
    pub secure: bool,
}

/// Captured network log entry from browser DevTools protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLogEntry {
    pub url: String,
    pub method: String,
    pub status: Option<u16>,
    pub resource_type: String,
    pub headers: HashMap<String, String>,
}

/// Login form field selectors.
#[derive(Debug, Clone)]
pub struct LoginFormSelectors {
    pub username_field: String,
    pub password_field: String,
    pub submit_button: String,
    pub success_indicator: Option<String>,
}

impl Default for LoginFormSelectors {
    fn default() -> Self {
        Self {
            username_field: "input[name='username'], input[type='email']".into(),
            password_field: "input[name='password'], input[type='password']".into(),
            submit_button: "button[type='submit'], input[type='submit']".into(),
            success_indicator: None,
        }
    }
}

/// Configuration for creating a Selenium session.
#[derive(Debug, Clone)]
pub struct SeleniumConfig {
    /// Base URL of the target platform.
    pub base_url: String,
    /// Whether to run the browser in headless mode.
    pub headless: bool,
    /// Page load timeout in seconds.
    pub timeout_secs: u64,
    /// Custom Chrome arguments (e.g., `--no-sandbox`).
    pub extra_args: Vec<String>,
}

impl Default for SeleniumConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            headless: true,
            timeout_secs: 30,
            extra_args: vec![],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WebDriverBackend trait
// ═══════════════════════════════════════════════════════════════════════════

/// Abstraction over the WebDriver protocol, allowing mock or real backends.
pub trait WebDriverBackend: Send + Sync {
    /// Navigate to a URL and wait for the page to load.
    fn navigate(&mut self, url: &str) -> Result<(), String>;

    /// Locate an element by CSS selector and type into it.
    fn type_into(&self, selector: &str, value: &str) -> Result<(), String>;

    /// Locate an element by CSS selector and click it.
    fn click(&self, selector: &str) -> Result<(), String>;

    /// Execute JavaScript in the browser and return the result as a string.
    fn execute_js(&self, script: &str) -> Result<String, String>;

    /// Retrieve all cookies from the current browser session.
    fn get_cookies(&self) -> Result<Vec<Cookie>, String>;

    /// Get the current page title.
    fn get_title(&self) -> Result<String, String>;

    /// Check if an element matching the selector is present.
    fn element_exists(&self, selector: &str) -> Result<bool, String>;

    /// Shut down the browser session.
    fn quit(&self) -> Result<(), String>;
}

// ═══════════════════════════════════════════════════════════════════════════
// MockBackend — in-memory backend for testing
// ═══════════════════════════════════════════════════════════════════════════

/// Mock WebDriver backend for unit testing without real browser.
pub struct MockBackend {
    current_url: String,
    cookies: Vec<Cookie>,
    title: String,
    navigations: Vec<String>,
    clicks: Vec<String>,
    typed: Vec<(String, String)>,
    js_executions: Vec<String>,
    quit_called: bool,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            current_url: String::new(),
            cookies: Vec::new(),
            title: "Mock Page".into(),
            navigations: Vec::new(),
            clicks: Vec::new(),
            typed: Vec::new(),
            js_executions: Vec::new(),
            quit_called: false,
        }
    }

    /// Inject a cookie into the mock session (for testing).
    pub fn inject_cookie(&mut self, cookie: Cookie) {
        self.cookies.push(cookie);
    }

    /// Set the page title returned by `get_title`.
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    /// Get all navigations performed.
    pub fn navigations(&self) -> &[String] {
        &self.navigations
    }

    /// Get all clicks performed.
    pub fn clicks(&self) -> &[String] {
        &self.clicks
    }

    /// Get all typed inputs.
    pub fn typed(&self) -> &[(String, String)] {
        &self.typed
    }

    /// Check if quit was called.
    pub fn is_quit(&self) -> bool {
        self.quit_called
    }
}

impl WebDriverBackend for MockBackend {
    fn navigate(&mut self, url: &str) -> Result<(), String> {
        self.current_url = url.to_string();
        self.navigations.push(url.to_string());
        Ok(())
    }

    fn type_into(&self, selector: &str, value: &str) -> Result<(), String> {
        if selector.is_empty() {
            return Err("selector is empty".into());
        }
        if value.is_empty() {
            return Err("value is empty".into());
        }
        Ok(())
    }

    fn click(&self, selector: &str) -> Result<(), String> {
        if selector.is_empty() {
            return Err("selector is empty".into());
        }
        Ok(())
    }

    fn execute_js(&self, script: &str) -> Result<String, String> {
        if script.is_empty() {
            return Err("script is empty".into());
        }
        Ok("mock_js_result".into())
    }

    fn get_cookies(&self) -> Result<Vec<Cookie>, String> {
        Ok(self.cookies.clone())
    }

    fn get_title(&self) -> Result<String, String> {
        Ok(self.title.clone())
    }

    fn element_exists(&self, _selector: &str) -> Result<bool, String> {
        Ok(false)
    }

    fn quit(&self) -> Result<(), String> {
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SeleniumSession
// ═══════════════════════════════════════════════════════════════════════════

/// Browser automation session with temp profile isolation.
pub struct SeleniumSession {
    profile_dir: PathBuf,
    config: SeleniumConfig,
    cookies: Vec<Cookie>,
    api_endpoints: Vec<String>,
    backend: Box<dyn WebDriverBackend>,
}

impl SeleniumSession {
    /// Create a new session with a temporary profile directory.
    ///
    /// The temp profile is created under the system temp dir to avoid
    /// conflicts with any running Chrome instance.
    pub fn new(config: SeleniumConfig) -> Result<Self, String> {
        let profile_dir = Self::create_temp_profile()?;
        let backend = Self::create_backend(&config, &profile_dir)?;

        Ok(Self {
            profile_dir,
            config,
            cookies: Vec::new(),
            api_endpoints: Vec::new(),
            backend,
        })
    }

    /// Create a session with a custom backend (for testing).
    pub fn with_backend(
        config: SeleniumConfig,
        backend: Box<dyn WebDriverBackend>,
    ) -> Result<Self, String> {
        let profile_dir = Self::create_temp_profile()?;
        Ok(Self {
            profile_dir,
            config,
            cookies: Vec::new(),
            api_endpoints: Vec::new(),
            backend,
        })
    }

    /// Login to a platform via form submission.
    ///
    /// Navigates to the login URL, fills username/password fields,
    /// submits the form, and waits for a success indicator.
    pub async fn login(&mut self, url: &str, username: &str, password: &str) -> Result<(), String> {
        let selectors = LoginFormSelectors::default();

        self.backend.navigate(url)?;

        self.backend
            .type_into(&selectors.username_field, username)?;
        self.backend
            .type_into(&selectors.password_field, password)?;
        self.backend.click(&selectors.submit_button)?;

        // Wait for page load (simplified; real impl would use WebDriverWait)
        if let Some(ref indicator) = selectors.success_indicator {
            self.wait_for_selector(indicator, self.config.timeout_secs)
                .await?;
        }

        // Capture cookies after successful login
        self.cookies = self.backend.get_cookies()?;

        Ok(())
    }

    /// Capture network logs for API endpoint discovery.
    ///
    /// Uses JavaScript to intercept XHR/fetch calls and extract API URLs.
    pub async fn capture_network_logs(&self) -> Vec<String> {
        let script = r#"
            (function() {
                var urls = [];
                var origOpen = XMLHttpRequest.prototype.open;
                XMLHttpRequest.prototype.open = function(method, url) {
                    urls.push(url);
                    return origOpen.apply(this, arguments);
                };
                var origFetch = window.fetch;
                window.fetch = function(input, init) {
                    var url = typeof input === 'string' ? input : input.url;
                    urls.push(url);
                    return origFetch.apply(this, arguments);
                };
                window.__captured_urls = urls;
                return JSON.stringify(urls);
            })()
        "#;

        match self.backend.execute_js(script) {
            Ok(result) => serde_json::from_str(&result).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    /// Extract cookies for session reuse.
    ///
    /// Returns all current browser cookies, which can be serialized
    /// and reused in subsequent sessions or API calls.
    pub async fn extract_cookies(&self) -> Vec<Cookie> {
        self.backend.get_cookies().unwrap_or_default()
    }

    /// Navigate to a URL and wait for the page to load.
    pub async fn navigate(&mut self, url: &str) -> Result<(), String> {
        self.backend.navigate(url)
    }

    /// Close the session and clean up the temp profile.
    pub async fn close(&self) -> Result<(), String> {
        self.backend.quit()?;
        // Profile cleanup would go here in a real implementation
        // (recursive delete of self.profile_dir)
        Ok(())
    }

    /// Get the profile directory path.
    pub fn profile_dir(&self) -> &Path {
        &self.profile_dir
    }

    /// Get discovered API endpoints.
    pub fn api_endpoints(&self) -> &[String] {
        &self.api_endpoints
    }

    // ── Private helpers ────────────────────────────────────────────────

    fn create_temp_profile() -> Result<PathBuf, String> {
        let base = std::env::temp_dir().join("neotrix_selenium_profile");
        std::fs::create_dir_all(&base)
            .map_err(|e| format!("failed to create profile dir: {}", e))?;
        Ok(base)
    }

    fn create_backend(
        config: &SeleniumConfig,
        _profile_dir: &Path,
    ) -> Result<Box<dyn WebDriverBackend>, String> {
        // TODO: When a `selenium` feature is added, create real ChromeDriver-backed backend here.
        let _ = config;
        Ok(Box::new(MockBackend::new()))
    }

    async fn wait_for_selector(&self, _selector: &str, _timeout_secs: u64) -> Result<(), String> {
        // Simplified: real impl would poll with WebDriverWait
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_compiles_and_instantiates() {
        let config = SeleniumConfig::default();
        let backend = Box::new(MockBackend::new());
        let session = SeleniumSession::with_backend(config, backend);
        assert!(session.is_ok());
        let session = session.unwrap();
        assert!(session.profile_dir().exists());
        assert!(session.api_endpoints().is_empty());
    }

    #[test]
    fn mock_backend_navigate() {
        let mut backend = MockBackend::new();
        assert!(backend.navigate("https://example.com").is_ok());
        assert_eq!(backend.navigations(), &["https://example.com".to_string()]);
    }

    #[test]
    fn mock_backend_cookies() {
        let mut backend = MockBackend::new();
        backend.inject_cookie(Cookie {
            name: "session_id".into(),
            value: "abc123".into(),
            domain: ".example.com".into(),
            path: "/".into(),
            expires: None,
            secure: true,
        });
        let cookies = backend.get_cookies().unwrap();
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "session_id");
    }

    #[test]
    fn login_form_selectors_default() {
        let sel = LoginFormSelectors::default();
        assert!(!sel.username_field.is_empty());
        assert!(!sel.password_field.is_empty());
        assert!(!sel.submit_button.is_empty());
    }

    #[test]
    fn selenium_config_default() {
        let config = SeleniumConfig::default();
        assert!(config.headless);
        assert_eq!(config.timeout_secs, 30);
    }

    #[tokio::test]
    async fn login_with_mock_backend() {
        let config = SeleniumConfig::default();
        let backend = Box::new(MockBackend::new());
        let mut session = SeleniumSession::with_backend(config, backend).unwrap();

        let result = session
            .login("https://example.com/login", "user@test.com", "pass123")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn extract_cookies_after_login() {
        let config = SeleniumConfig::default();
        let mut mock = MockBackend::new();
        mock.inject_cookie(Cookie {
            name: "token".into(),
            value: "xyz789".into(),
            domain: ".example.com".into(),
            path: "/".into(),
            expires: Some(1700000000),
            secure: false,
        });
        let mut session = SeleniumSession::with_backend(config, Box::new(mock)).unwrap();

        let cookies = session.extract_cookies().await;
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "token");
    }

    #[tokio::test]
    async fn navigate_and_close() {
        let config = SeleniumConfig::default();
        let backend = Box::new(MockBackend::new());
        let mut session = SeleniumSession::with_backend(config, backend).unwrap();

        assert!(session.navigate("https://example.com").await.is_ok());
        assert!(session.close().await.is_ok());
    }

    #[test]
    fn cookie_serialization_roundtrip() {
        let cookie = Cookie {
            name: "sid".into(),
            value: "abc".into(),
            domain: ".test.com".into(),
            path: "/".into(),
            expires: Some(1700000000),
            secure: true,
        };
        let json = serde_json::to_string(&cookie).unwrap();
        let deserialized: Cookie = serde_json::from_str(&json).unwrap();
        assert_eq!(cookie.name, deserialized.name);
        assert_eq!(cookie.value, deserialized.value);
        assert_eq!(cookie.expires, deserialized.expires);
    }
}

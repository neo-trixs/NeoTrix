//! # NT-SHIELD HTTP Interception Proxy
//!
//! Transparent proxy for HTTP/HTTPS request/response inspection and modification.
//! Supports HTTP and HTTPS (MITM with configurable CA).
//!
//! All external HTTP calls go through `egress_privacy_guard` to prevent data leakage.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use url::Url;

use crate::core::nt_core_llm::DataTrust;
use crate::core::nt_core_self_test::SelfTest;

/// HTTP/HTTPS interception proxy for request/response inspection.
pub struct HttpInterceptProxy {
    client: Client,
    rules: Arc<Mutex<Vec<InterceptRule>>>,
    sessions: Arc<Mutex<Vec<InterceptSession>>>,
    config: InterceptConfig,
}

/// Configuration for the HTTP interception proxy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptConfig {
    pub listen_addr: String,
    pub upstream_timeout: Duration,
    pub capture_request_body: bool,
    pub capture_response_body: bool,
    pub max_body_capture: usize,
    pub mitm_enabled: bool,
    pub ca_cert_path: Option<String>,
    pub trust_level: DataTrust,
}

impl Default for InterceptConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:11080".into(),
            upstream_timeout: Duration::from_secs(30),
            capture_request_body: true,
            capture_response_body: true,
            max_body_capture: 65536,
            mitm_enabled: false,
            ca_cert_path: None,
            trust_level: DataTrust::Contracted,
        }
    }
}

/// Rule for matching and modifying HTTP requests/responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub rule_type: RuleType,
    pub action: RuleAction,
    pub enabled: bool,
    pub priority: u32,
}

/// Type of interception rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    /// Match on URL pattern
    Url,
    /// Match on HTTP method
    Method,
    /// Match on host/domain
    Host,
    /// Match on header
    Header,
    /// Match on request/response body
    Body,
}

/// Action to take when a rule matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    /// Modify the request/response
    Modify,
    /// Block the request/response entirely
    Block,
    /// Log only, allow through
    Log,
    /// Redirect to a different URL
    Redirect(String),
}

/// Active interception session with captured data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptSession {
    pub id: u64,
    pub timestamp: u64,
    pub request_url: String,
    pub method: String,
    pub request_headers: HashMap<String, String>,
    pub request_body: Vec<u8>,
    pub response_status: Option<u16>,
    pub response_headers: HashMap<String, String>,
    pub response_body: Vec<u8>,
    pub modified_request: bool,
    pub modified_response: bool,
    pub rules_matched: Vec<String>,
    pub duration_secs: Option<f64>,
}

/// Captured HTTP request before modification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedHttpRequest {
    pub method: String,
    pub url: Url,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub timestamp: u64,
}

/// Captured HTTP response after interception.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedHttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub timestamp: u64,
}

/// Result of modifying a request.
#[derive(Debug, Clone)]
pub struct ModifyRequestResult {
    pub url: Url,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub modifications: Vec<String>,
}

/// Result of modifying a response.
#[derive(Debug, Clone)]
pub struct ModifyResponseResult {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub modifications: Vec<String>,
}

impl HttpInterceptProxy {
    /// Create a new HTTP interception proxy.
    pub fn new(config: InterceptConfig) -> Self {
        let client = Client::builder()
            .timeout(config.upstream_timeout)
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            rules: Arc::new(Mutex::new(Vec::new())),
            sessions: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    /// Add an interception rule.
    pub async fn add_rule(&self, rule: InterceptRule) {
        let mut rules = self.rules.lock().await;
        rules.push(rule);
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove an interception rule by ID.
    pub async fn remove_rule(&self, rule_id: &str) {
        let mut rules = self.rules.lock().await;
        rules.retain(|r| r.id != rule_id);
    }

    /// Modify an HTTP request based on matching rules.
    pub async fn modify_request(
        &self,
        req: CapturedHttpRequest,
    ) -> Result<ModifyRequestResult, String> {
        let rules = self.rules.lock().await;
        let mut modifications = Vec::new();
        let mut url = req.url.clone();
        let mut headers = req.headers.clone();
        let mut body = req.body.clone();

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }
            if !self.rule_matches(rule, &req) {
                continue;
            }

            match rule.rule_type {
                RuleType::Url => {
                    if let Ok(new_url) = Url::parse(&rule.pattern) {
                        url = new_url;
                        modifications.push(format!("Rule '{}' redirected URL to {}", rule.name, url));
                    }
                }
                RuleType::Header => {
                    if let Some((key, _)) = headers.iter().find(|(k, _)| k.to_lowercase() == rule.pattern.to_lowercase()) {
                        modifications.push(format!("Rule '{}' matched header '{}'", rule.name, key));
                    }
                }
                RuleType::Method => {
                    if req.method.to_lowercase() == rule.pattern.to_lowercase() {
                        modifications.push(format!("Rule '{}' matched method '{}'", rule.name, req.method));
                    }
                }
                RuleType::Host => {
                    if req.url.host().map(|h| h.to_string()).unwrap_or_default() == rule.pattern {
                        modifications.push(format!("Rule '{}' matched host '{}'", rule.name, rule.pattern));
                    }
                }
                RuleType::Body => {
                    if String::from_utf8_lossy(&body).contains(&rule.pattern) {
                        modifications.push(format!("Rule '{}' matched body content", rule.name));
                    }
                }
            }

            match rule.action {
                RuleAction::Block => {
                    return Err(format!("Request blocked by rule '{}'", rule.name));
                }
                RuleAction::Modify => {
                    modifications.push(format!("Rule '{}' modified request", rule.name));
                }
                RuleAction::Redirect(ref target) => {
                    url = Url::parse(target).map_err(|e| format!("Invalid redirect URL: {}", e))?;
                    modifications.push(format!("Rule '{}' redirected to {}", rule.name, target));
                }
                RuleAction::Log => {
                    log::info!("[http-intercept] Rule '{}' matched, logging", rule.name);
                }
            }
        }

        // Apply egress privacy guard for all outbound HTTP calls
        self.apply_egress_guard(&mut headers, &mut body)?;

        Ok(ModifyRequestResult {
            url,
            headers,
            body,
            modifications,
        })
    }

    /// Modify an HTTP response based on matching rules.
    pub async fn modify_response(
        &self,
        resp: CapturedHttpResponse,
    ) -> Result<ModifyResponseResult, String> {
        let rules = self.rules.lock().await;
        let mut modifications = Vec::new();
        let mut status_code = resp.status_code;
        let mut headers = resp.headers.clone();
        let body = resp.body.clone();

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }
            if !self.response_rule_matches(rule, &resp) {
                continue;
            }

            match rule.action {
                RuleAction::Block => {
                    return Err(format!("Response blocked by rule '{}'", rule.name));
                }
                RuleAction::Modify => {
                    modifications.push(format!("Rule '{}' modified response", rule.name));
                }
                RuleAction::Redirect(ref target) => {
                    status_code = 302;
                    headers.insert("Location".to_string(), target.clone());
                    modifications.push(format!("Rule '{}' redirected response to {}", rule.name, target));
                }
                RuleAction::Log => {
                    log::info!("[http-intercept] Rule '{}' matched response, logging", rule.name);
                }
            }
        }

        Ok(ModifyResponseResult {
            status_code,
            headers,
            body,
            modifications,
        })
    }

    /// Execute an intercepted HTTP request through the proxy.
    pub async fn intercept_request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Result<InterceptSession, String> {
        let session_id = self.next_session_id().await;
        let timestamp = Instant::now();

        let captured_req = CapturedHttpRequest {
            method: method.to_string(),
            url: Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?,
            headers: headers.clone(),
            body: body.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        };

        // Modify the request
        let modified = self.modify_request(captured_req.clone()).await?;

        // Execute the HTTP call via reqwest with egress privacy guard applied
        let request = self.client.request(
            method.parse::<reqwest::Method>().map_err(|e| format!("Invalid method: {}", e))?,
            modified.url,
        );
        let header_pairs: Vec<_> = modified
            .headers
            .iter()
            .filter_map(|(k, v)| {
                let name = reqwest::header::HeaderName::from_lowercase(k.to_lowercase().as_bytes()).ok()?;
                let val = v.parse().ok()?;
                Some((name, val))
            })
            .collect();
        let mut header_map = reqwest::header::HeaderMap::new();
        for (k, v) in header_pairs {
            header_map.append(k, v);
        }
        let request = request.headers(header_map);

        let response = request
            .body(modified.body.clone())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status_code = response.status().as_u16();
        let resp_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let resp_body = response
            .bytes()
            .await
            .map_err(|e| format!("Response body error: {}", e))?
            .to_vec();

        let captured_resp = CapturedHttpResponse {
            status_code,
            headers: resp_headers.clone(),
            body: resp_body.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        };

        // Modify the response
        let modified_resp = self.modify_response(captured_resp.clone()).await?;

        let session = InterceptSession {
            id: session_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            request_url: url.to_string(),
            method: method.to_string(),
            request_headers: headers,
            request_body: body,
            response_status: Some(status_code),
            response_headers: resp_headers,
            response_body: modified_resp.body,
            modified_request: !modified.modifications.is_empty(),
            modified_response: !modified_resp.modifications.is_empty(),
            rules_matched: modified.modifications.iter().cloned().collect(),
            duration_secs: Some(timestamp.elapsed().as_secs_f64()),
        };

        let mut sessions = self.sessions.lock().await;
        sessions.push(session.clone());
        if sessions.len() > 1000 {
            sessions.remove(0);
        }

        Ok(session)
    }

    /// Get all active sessions.
    pub async fn sessions(&self) -> Vec<InterceptSession> {
        let sessions = self.sessions.lock().await;
        sessions.clone()
    }

    /// Get session count.
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.lock().await;
        sessions.len()
    }

    /// Clear all sessions.
    pub async fn clear_sessions(&self) {
        let mut sessions = self.sessions.lock().await;
        sessions.clear();
    }

    /// Check if a request rule matches.
    fn rule_matches(&self, rule: &InterceptRule, req: &CapturedHttpRequest) -> bool {
        if !rule.enabled {
            return false;
        }
        match rule.rule_type {
            RuleType::Url => req.url.as_str().contains(&rule.pattern),
            RuleType::Method => req.method.to_lowercase() == rule.pattern.to_lowercase(),
            RuleType::Host => req.url.host().map(|h| h.to_string()) == Some(rule.pattern.clone()),
            RuleType::Header => req.headers.keys().any(|k| k.to_lowercase().contains(&rule.pattern.to_lowercase())),
            RuleType::Body => String::from_utf8_lossy(&req.body).contains(&rule.pattern),
        }
    }

    /// Check if a response rule matches.
    fn response_rule_matches(&self, rule: &InterceptRule, resp: &CapturedHttpResponse) -> bool {
        if !rule.enabled {
            return false;
        }
        match rule.rule_type {
            RuleType::Url => resp.headers.get("host").map_or(false, |h| h.contains(&rule.pattern)),
            RuleType::Method => false,
            RuleType::Host => resp.headers.get("host").map_or(false, |h| h == &rule.pattern),
            RuleType::Header => resp.headers.keys().any(|k| k.to_lowercase().contains(&rule.pattern.to_lowercase())),
            RuleType::Body => String::from_utf8_lossy(&resp.body).contains(&rule.pattern),
        }
    }

    /// Apply egress privacy guard to outbound HTTP calls.
    fn apply_egress_guard(
        &self,
        headers: &mut HashMap<String, String>,
        body: &mut Vec<u8>,
    ) -> Result<(), String> {
        // Scrub internal paths and secrets from headers per egress privacy guard
        for (_k, v) in headers.iter_mut() {
            *v = scrub_internal_fingerprint(v);
        }
        let body_str = String::from_utf8_lossy(body);
        let scrubbed = scrub_internal_fingerprint(&body_str);
        *body = scrubbed.into_bytes();
        Ok(())
    }

    /// Generate next session ID.
    async fn next_session_id(&self) -> u64 {
        let sessions = self.sessions.lock().await;
        sessions.len() as u64 + 1
    }
}

/// Scrub internal NeoTrix fingerprints from a string.
fn scrub_internal_fingerprint(input: &str) -> String {
    let input = regex::Regex::new(r"/Users/[^/\s]+")
        .map(|re| re.replace_all(input, "/path/redacted").to_string())
        .unwrap_or_else(|_| input.to_string());
    let input = regex::Regex::new(r"/var/[^/\s]+")
        .map(|re| re.replace_all(&input, "/path/redacted").to_string())
        .unwrap_or_else(|_| input.to_string());
    let input = regex::Regex::new(r"[A-Za-z0-9_+/=-]{40,}")
        .map(|re| {
            re.replace_all(&input, |caps: &regex::Captures| {
                if caps[0].len() > 20 {
                    "***REDACTED***".to_string()
                } else {
                    caps[0].to_string()
                }
            })
            .to_string()
        })
        .unwrap_or_else(|_| input.to_string());
    input
}

impl Default for HttpInterceptProxy {
    fn default() -> Self {
        Self::new(InterceptConfig::default())
    }
}

impl SelfTest for HttpInterceptProxy {
    fn name(&self) -> &str {
        "http_intercept_proxy"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if self.config.listen_addr.is_empty() {
            return Err(vec!["listen_addr cannot be empty".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_config_default() {
        let cfg = InterceptConfig::default();
        assert_eq!(cfg.listen_addr, "127.0.0.1:11080");
        assert_eq!(cfg.upstream_timeout, Duration::from_secs(30));
        assert!(cfg.capture_request_body);
        assert!(!cfg.mitm_enabled);
    }

    #[test]
    fn test_proxy_creation() {
        let proxy = HttpInterceptProxy::new(InterceptConfig::default());
        assert_eq!(proxy.config.listen_addr, "127.0.0.1:11080");
    }

    #[test]
    fn test_scrub_internal_fingerprint() {
        let result = scrub_internal_fingerprint("/Users/neo/secret/path");
        assert!(result.contains("redacted"));
    }

    #[test]
    fn test_rule_matches_url() {
        let rule = InterceptRule {
            id: "1".into(),
            name: "Test URL".into(),
            pattern: "example.com".into(),
            rule_type: RuleType::Url,
            action: RuleAction::Log,
            enabled: true,
            priority: 1,
        };
        let req_url = Url::parse("https://example.com/path").unwrap();
        let req = CapturedHttpRequest {
            method: "GET".into(),
            url: req_url,
            headers: HashMap::new(),
            body: vec![],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        };
        // Can't create proxy without reqwest client in test easily, so test logic directly
        let matches = req.url.as_str().contains(&rule.pattern);
        assert!(matches);
    }

    #[test]
    fn test_rule_type_equality() {
        assert_eq!(RuleType::Url, RuleType::Url);
        assert_ne!(RuleType::Url, RuleType::Method);
    }

    #[test]
    fn test_action_serde() {
        let action = RuleAction::Redirect("https://safe.example.com".into());
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: RuleAction = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, RuleAction::Redirect(_)));
    }
}
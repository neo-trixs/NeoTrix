//! HTTP Client with retry, circuit breaker, rate limiting

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use reqwest::{Client, ClientBuilder, RequestBuilder, Method, header::HeaderMap};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::{DeliveryOutcome, HttpClientConfig, HttpRequest, HttpResponse};

/// HTTP Client with retry, circuit breaker, rate limiting
pub struct HttpClient {
    client: Client,
    config: crate::nt_act::HttpClientConfig,
    rate_limiter: Arc<Mutex<HashMap<String, Vec<std::time::Instant>>>>,
    circuit_breaker: Arc<Mutex<HashMap<String, CircuitBreakerState>>>,
}

#[derive(Debug, Clone, Default)]
struct CircuitBreakerState {
    failures: u32,
    last_failure: Option<std::time::Instant>,
    state: CircuitState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// A transport failure annotated with its delivery outcome.
struct RequestFailure {
    message: String,
    outcome: DeliveryOutcome,
}

impl RequestFailure {
    fn pre_send(message: String) -> Self {
        Self {
            message,
            outcome: DeliveryOutcome::Failed,
        }
    }
}

/// Which phase a transport error occurred in (extracted from reqwest for pure testing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransportCause {
    /// Deadline hit; request may or may not have been sent.
    pub timeout: bool,
    /// Connection/DNS/TLS failure; nothing was sent.
    pub connect: bool,
    /// Response-phase failure (body read/decode); peer already processed the request.
    pub response_phase: bool,
}

fn transport_cause(err: &reqwest::Error) -> TransportCause {
    TransportCause {
        timeout: err.is_timeout(),
        connect: err.is_connect(),
        response_phase: err.is_body() || err.is_decode(),
    }
}

/// Pure classifier: map a transport cause to its delivery outcome.
///
/// Conservative per dsh-im: timeouts map to `Unknown` even when they might be
/// connect timeouts — preventing duplicate sends outranks recovering a retry.
pub(crate) fn classify_failure(cause: TransportCause) -> DeliveryOutcome {
    if cause.response_phase || cause.timeout {
        DeliveryOutcome::Unknown
    } else {
        DeliveryOutcome::Failed
    }
}

/// Pure classifier: map an HTTP status to its delivery outcome.
/// 5xx = peer may have processed side effects (`Unknown`); 4xx = definitive rejection.
pub(crate) fn classify_status(status: u16) -> DeliveryOutcome {
    if status >= 500 {
        DeliveryOutcome::Unknown
    } else if status >= 400 {
        DeliveryOutcome::Failed
    } else {
        DeliveryOutcome::Delivered
    }
}

impl HttpClient {
    pub fn new(config: crate::nt_act::HttpClientConfig) -> Self {
        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(10));

        if let Some(proxy) = &config.proxy {
            if let Ok(proxy) = reqwest::Proxy::all(proxy) {
                builder = builder.proxy(proxy);
            }
        }

        if !config.tls_verify {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build().expect("Failed to create HTTP client");

        Self {
            client,
            config,
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
            circuit_breaker: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Execute HTTP request with retry, circuit breaker, rate limiting
    pub async fn request(&self, request: HttpRequest) -> Result<HttpResponse, String> {
        self.request_with_outcome(request).await.0
    }

    /// Execute HTTP request and report the delivery outcome alongside the result.
    ///
    /// dsh-im three-state semantics: failures classified `Unknown` (timeout after
    /// possible send, response-phase errors, 5xx) abort the retry loop immediately —
    /// the peer may have processed a non-idempotent request, so auto-retry risks
    /// duplicate execution. Only `Failed` outcomes (request never left / definitive
    /// rejection) consume retries.
    pub async fn request_with_outcome(
        &self,
        request: HttpRequest,
    ) -> (Result<HttpResponse, String>, DeliveryOutcome) {
        let url = self.build_url(&request);
        self.check_rate_limit(&url).await;
        if let Err(e) = self.check_circuit_breaker(&url).await {
            return (Err(e), DeliveryOutcome::Failed);
        }

        let mut last_failure: Option<RequestFailure> = None;
        for attempt in 0..=self.config.max_retries {
            match self.execute_request(&request).await {
                Ok(resp) => {
                    let outcome = classify_status(resp.status);
                    self.record_success(&url).await;
                    return (Ok(resp), outcome);
                }
                Err(fail) => {
                    // Unknown still counts as host-trouble evidence for the breaker.
                    self.record_failure(&url).await;
                    if fail.outcome == DeliveryOutcome::Unknown {
                        // At-most-once: never spend retries on an unverifiable delivery.
                        return (Err(fail.message), DeliveryOutcome::Unknown);
                    }
                    let retryable = attempt < self.config.max_retries;
                    last_failure = Some(fail);
                    if retryable {
                        tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms))
                            .await;
                    }
                }
            }
        }

        match last_failure {
            Some(f) => (Err(f.message), f.outcome),
            None => (
                Err("Request loop exited without attempts".to_string()),
                DeliveryOutcome::Failed,
            ),
        }
    }

    async fn execute_request(&self, request: &HttpRequest) -> Result<HttpResponse, RequestFailure> {
        let url = self.build_url(request);
        let method = Method::from_bytes(request.method.as_bytes())
            .map_err(|e| RequestFailure::pre_send(format!("Invalid method: {}", e)))?;

        let mut req = self.client.request(method, &url);

        // Add headers
        let mut headers = HeaderMap::new();
        for (k, v) in &request.headers {
            headers.insert(
                k.parse()
                    .map_err(|e| RequestFailure::pre_send(format!("Invalid header key: {}", e)))?,
                v.parse().map_err(|e| {
                    RequestFailure::pre_send(format!("Invalid header value: {}", e))
                })?,
            );
        }
        // Add default headers
        for (k, v) in &self.config.headers {
            headers.insert(k.parse().unwrap(), v.parse().unwrap());
        }
        req = req.headers(headers);

        // Add body
        if let Some(body) = &request.body {
            req = req.body(body.clone());
        }

        // Add query params
        if let Some(params) = &request.query {
            req = req.query(params);
        }

        let resp = req.send().await.map_err(|e| RequestFailure {
            message: format!("Request failed: {}", e),
            outcome: classify_failure(transport_cause(&e)),
        })?;

        let status = resp.status().as_u16();
        let headers = resp.headers().clone();
        let body = resp.bytes().await.map_err(|e| RequestFailure {
            message: format!("Failed to read body: {}", e),
            outcome: DeliveryOutcome::Unknown,
        })?;

        Ok(HttpResponse {
            status,
            headers: headers.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect(),
            body: body.to_vec(),
        })
    }

    fn build_url(&self, request: &HttpRequest) -> String {
        let base = self.config.base_url.as_deref().unwrap_or("");
        if request.url.starts_with("http") {
            request.url.clone()
        } else {
            format!("{}{}", base.trim_end_matches('/'), request.url)
        }
    }

    async fn check_rate_limit(&self, url: &str) {
        let host = url.split('/').nth(2).unwrap_or("default");
        let mut limiter = self.rate_limiter.lock().await;
        let now = std::time::Instant::now();
        let requests = limiter.entry(host.to_string()).or_insert_with(Vec::new);
        
        // Clean old entries (older than 1 second)
        requests.retain(|&t| now.duration_since(t) < Duration::from_secs(1));
        
        // Simple rate limit: 100 requests per second per host
        if requests.len() >= 100 {
            let oldest = requests[0];
            let wait = Duration::from_secs(1).saturating_sub(now.duration_since(oldest));
            tokio::time::sleep(wait).await;
        }
        requests.push(now);
    }

    async fn check_circuit_breaker(&self, url: &str) -> Result<(), String> {
        let host = url.split('/').nth(2).unwrap_or("default");
        let mut cb = self.circuit_breaker.lock().await;
        let state = cb.entry(host.to_string()).or_default();
        
        match state.state {
            CircuitState::Open => {
                if let Some(last) = state.last_failure {
                    if last.elapsed() > Duration::from_secs(60) {
                        state.state = CircuitState::HalfOpen;
                    } else {
                        return Err("Circuit breaker open".to_string());
                    }
                } else {
                    return Err("Circuit breaker open".to_string());
                }
            }
            CircuitState::HalfOpen => {}
            CircuitState::Closed => {}
        }
        Ok(())
    }

    async fn record_success(&self, url: &str) {
        let host = url.split('/').nth(2).unwrap_or("default");
        let mut cb = self.circuit_breaker.lock().await;
        if let Some(state) = cb.get_mut(host) {
            state.failures = 0;
            state.state = CircuitState::Closed;
        }
    }

    async fn record_failure(&self, url: &str) {
        let host = url.split('/').nth(2).unwrap_or("default");
        let mut cb = self.circuit_breaker.lock().await;
        let state = cb.entry(host.to_string()).or_default();
        state.failures += 1;
        state.last_failure = Some(std::time::Instant::now());
        if state.failures >= 5 {
            state.state = CircuitState::Open;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = crate::nt_act::HttpClientConfig::default();
        let _client = HttpClient::new(config);
    }

    #[test]
    fn classify_failure_connect_is_retryable() {
        let outcome = classify_failure(TransportCause {
            timeout: false,
            connect: true,
            response_phase: false,
        });
        assert_eq!(outcome, DeliveryOutcome::Failed);
    }

    #[test]
    fn classify_failure_timeout_is_unknown() {
        let outcome = classify_failure(TransportCause {
            timeout: true,
            connect: false,
            response_phase: false,
        });
        assert_eq!(outcome, DeliveryOutcome::Unknown);
    }

    #[test]
    fn classify_failure_response_phase_is_unknown_even_if_connect_flag_set() {
        let outcome = classify_failure(TransportCause {
            timeout: false,
            connect: true,
            response_phase: true,
        });
        assert_eq!(outcome, DeliveryOutcome::Unknown);
    }

    #[test]
    fn classify_status_matrix() {
        assert_eq!(classify_status(200), DeliveryOutcome::Delivered);
        assert_eq!(classify_status(302), DeliveryOutcome::Delivered);
        assert_eq!(classify_status(403), DeliveryOutcome::Failed);
        assert_eq!(classify_status(413), DeliveryOutcome::Failed);
        assert_eq!(classify_status(429), DeliveryOutcome::Failed);
        assert_eq!(classify_status(502), DeliveryOutcome::Unknown);
    }

    #[tokio::test]
    async fn connect_refused_classifies_failed_and_retries_then_fails() {
        // Port 1 on loopback: connection refused, nothing ever sent -> Failed.
        let config = crate::nt_act::HttpClientConfig {
            base_url: Some("http://127.0.0.1:1".to_string()),
            max_retries: 1,
            retry_delay_ms: 1,
            timeout_secs: 2,
            ..Default::default()
        };
        let client = HttpClient::new(config);
        let request = HttpRequest {
            method: "POST".to_string(),
            url: "/non-idempotent".to_string(),
            headers: HashMap::new(),
            body: Some("{}" .to_string()),
            query: None,
        };
        let (result, outcome) = client.request_with_outcome(request).await;
        assert!(result.is_err());
        assert_eq!(outcome, DeliveryOutcome::Failed);
    }

    #[test]
    fn delivery_outcome_str_roundtrip_labels() {
        assert_eq!(DeliveryOutcome::Delivered.as_str(), "delivered");
        assert_eq!(DeliveryOutcome::Unknown.as_str(), "unknown");
        assert_eq!(DeliveryOutcome::Failed.as_str(), "failed");
    }
}

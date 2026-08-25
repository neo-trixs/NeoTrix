//! HTTP Client with retry, circuit breaker, rate limiting

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use reqwest::{Client, ClientBuilder, RequestBuilder, Method, header::HeaderMap};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::nt_act::{HttpClientConfig, HttpRequest, HttpResponse};

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
        let url = self.build_url(&request);
        self.check_rate_limit(&url).await;
        self.check_circuit_breaker(&url).await?;

        let mut last_error = String::new();
        for attempt in 0..=self.config.max_retries {
            let response = self.execute_request(&request).await;
            
            match response {
                Ok(resp) => {
                    self.record_success(&url).await;
                    return Ok(resp);
                }
                Err(e) => {
                    last_error = e;
                    self.record_failure(&url).await;
                    
                    if attempt < self.config.max_retries {
                        tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                    }
                }
            }
        }
        
        Err(last_error)
    }

    async fn execute_request(&self, request: &HttpRequest) -> Result<HttpResponse, String> {
        let url = self.build_url(request);
        let method = Method::from_bytes(request.method.as_bytes())
            .map_err(|e| format!("Invalid method: {}", e))?;

        let mut req = self.client.request(method, &url);

        // Add headers
        let mut headers = HeaderMap::new();
        for (k, v) in &request.headers {
            headers.insert(k.parse().map_err(|e| format!("Invalid header key: {}", e))?,
                          v.parse().map_err(|e| format!("Invalid header value: {}", e))?);
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

        let resp = req.send().await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = resp.status().as_u16();
        let headers = resp.headers().clone();
        let body = resp.bytes().await
            .map_err(|e| format!("Failed to read body: {}", e))?;

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

    #[tokio::test]
    async fn test_client_creation() {
        let config = crate::nt_act::HttpClientConfig::default();
        let client = HttpClient::new(config);
        assert!(true);
    }
}

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use axum::extract::connect_info::ConnectInfo;
use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;

#[derive(Debug, Clone)]
pub struct A2AAuthConfig {
    pub api_key: Option<String>,
    pub rate_limit_per_min: u32,
    pub max_request_size: usize,
    pub max_concurrent_tasks_per_session: u32,
}

impl Default for A2AAuthConfig {
    fn default() -> Self {
        let api_key = std::env::var("NEOTRIX_A2A_API_KEY").ok().or_else(|| {
            let tmp = format!("nt-{:032x}", rand::random::<u128>());
            log::warn!("[a2a] NEOTRIX_A2A_API_KEY not set; generated temporary key. Set env var for persistent auth.");
            Some(tmp)
        });
        Self {
            api_key,
            rate_limit_per_min: 60,
            max_request_size: 1_048_576,
            max_concurrent_tasks_per_session: 10,
        }
    }
}

impl A2AAuthConfig {
    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    requests: HashMap<SocketAddr, Vec<Instant>>,
    limit_per_min: u32,
}

impl RateLimiter {
    pub fn new(limit_per_min: u32) -> Self {
        Self {
            requests: HashMap::new(),
            limit_per_min,
        }
    }

    pub fn check_and_record(&mut self, addr: SocketAddr) -> bool {
        let now = Instant::now();
        let entries = self.requests.entry(addr).or_default();
        entries.retain(|t| now.duration_since(*t).as_secs() < 60);
        if entries.len() >= self.limit_per_min as usize {
            return false;
        }
        entries.push(now);
        true
    }
}

pub struct A2AMiddlewareState {
    pub config: A2AAuthConfig,
    pub rate_limiter: Mutex<RateLimiter>,
}

pub async fn auth_middleware(
    State(state): State<Arc<A2AMiddlewareState>>,
    req: Request,
    next: Next,
) -> Response {
    // ── API key authentication ──────────────────────────────────────────
    if let Some(ref api_key) = state.config.api_key {
        let auth = req
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok());
        let valid = auth.is_some_and(|v| v == format!("Bearer {api_key}"));
        if !valid {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "missing or invalid API key"})),
            )
                .into_response();
        }
    }

    // ── Request body size limit ─────────────────────────────────────────
    if let Some(cl) = req
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
    {
        if cl > state.config.max_request_size {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(serde_json::json!({"error": "request body too large"})),
            )
                .into_response();
        }
    }

    // ── Rate limiting (IP-based sliding window) ─────────────────────────
    let addr = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0)
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.split(',').next())
                .and_then(|v| v.trim().parse::<SocketAddr>().ok())
        });

    if let Some(addr) = addr {
        if !state.rate_limiter.lock().await.check_and_record(addr) {
            log::warn!("[a2a] rate limit exceeded for {addr}");
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({"error": "rate limit exceeded"})),
            )
                .into_response();
        }
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_default() {
        let cfg = A2AAuthConfig::default();
        assert!(cfg.api_key.is_none());
        assert_eq!(cfg.rate_limit_per_min, 60);
        assert_eq!(cfg.max_request_size, 1_048_576);
        assert_eq!(cfg.max_concurrent_tasks_per_session, 10);
    }

    #[test]
    fn test_auth_config_with_api_key() {
        let cfg = A2AAuthConfig::default().with_api_key("sk-test123");
        assert_eq!(cfg.api_key, Some("sk-test123".into()));
    }

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let mut rl = RateLimiter::new(5);
        let addr = "127.0.0.1:12345".parse().unwrap();
        for _ in 0..5 {
            assert!(rl.check_and_record(addr));
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let mut rl = RateLimiter::new(3);
        let addr = "127.0.0.1:12345".parse().unwrap();
        assert!(rl.check_and_record(addr));
        assert!(rl.check_and_record(addr));
        assert!(rl.check_and_record(addr));
        assert!(!rl.check_and_record(addr));
    }

    #[test]
    fn test_rate_limiter_independent_per_ip() {
        let mut rl = RateLimiter::new(2);
        let a = "10.0.0.1:1111".parse().unwrap();
        let b = "10.0.0.2:2222".parse().unwrap();
        assert!(rl.check_and_record(a));
        assert!(rl.check_and_record(a));
        assert!(!rl.check_and_record(a));
        assert!(rl.check_and_record(b));
        assert!(rl.check_and_record(b));
    }

    #[test]
    fn test_rate_limiter_window_expires() {
        let mut rl = RateLimiter::new(1);
        let addr = "127.0.0.1:9999".parse().unwrap();
        assert!(rl.check_and_record(addr));
        let now = Instant::now();
        let old = now.checked_sub(Duration::from_secs(61)).unwrap();
        rl.requests.get_mut(&addr).unwrap().clear();
        rl.requests.get_mut(&addr).unwrap().push(old);
        assert!(rl.check_and_record(addr));
    }

    use std::time::Duration;
}

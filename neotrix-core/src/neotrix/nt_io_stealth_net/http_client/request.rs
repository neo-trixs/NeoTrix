use std::collections::HashMap;
use std::time::{Duration, Instant};

use log;
use url::Url;

use super::super::geo_proxy::is_timeout_error;
use super::super::ip_privacy::IpPrivacyManager;
use super::super::is_tracker_blocked;
use super::super::rotation_coordinator::RotationDomain;
use super::super::rules::OutboundAction;
use super::config::TOR_SOCKS_ADDR;

use super::config::{gaussian_delay_ms, ProxyConfig};
use super::StealthHttpClient;

#[derive(Debug, Clone)]
pub struct Response {
    pub url: Url,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(|s| s.as_str())
    }

    pub fn is_html(&self) -> bool {
        self.header("content-type")
            .map(|ct| ct.contains("text/html"))
            .unwrap_or(false)
    }
}

impl StealthHttpClient {
    async fn send_request_with_method(
        &self,
        client: &reqwest::Client,
        parsed: &Url,
        method: reqwest::Method,
        body: Option<Vec<u8>>,
    ) -> Result<Response, String> {
        let mut req = client.request(method.clone(), parsed.as_str());

        if let Some(body_bytes) = body {
            req = req.body(body_bytes);
        }
        if method != reqwest::Method::GET {
            req = req.header("content-type", "application/json");
        }

        if let Some(ref mgr) = *self.ip_privacy.read().await {
            let cfg = mgr.current().await;
            let headers = IpPrivacyManager::to_headers(&cfg);
            for (k, v) in headers {
                req = req.header(k, v);
            }
        }

        if let Some(ref router) = *self.lan_router.read().await {
            let wifi = router.current_wifi_info().await;
            for (k, v) in wifi.to_headers() {
                req = req.header(k, v);
            }
        }

        let extra = self.extra_headers.read().await;
        for (k, v) in extra.iter() {
            req = req.header(k, v);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let status = resp.status().as_u16();
        let headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                let val = v
                    .to_str()
                    .inspect_err(|e| log::warn!("[request] non-utf8 header {}: {}", k, e))
                    .ok()?;
                Some((k.to_string(), val.to_string()))
            })
            .collect();
        let body = resp
            .bytes()
            .await
            .map_err(|e| format!("Read body failed: {}", e))?
            .to_vec();

        Ok(Response {
            url: parsed.clone(),
            status,
            headers,
            body,
        })
    }

    async fn send_request(
        &self,
        client: &reqwest::Client,
        parsed: &Url,
    ) -> Result<Response, String> {
        self.send_request_with_method(client, parsed, reqwest::Method::GET, None)
            .await
    }

        if let Some(ref mgr) = *self.ip_privacy.read().await {
            let cfg = mgr.current().await;
            let headers = IpPrivacyManager::to_headers(&cfg);
            for (k, v) in headers {
                req = req.header(k, v);
            }
        }

        if let Some(ref router) = *self.lan_router.read().await {
            let wifi = router.current_wifi_info().await;
            for (k, v) in wifi.to_headers() {
                req = req.header(k, v);
            }
        }

        let extra = self.extra_headers.read().await;
        for (k, v) in extra.iter() {
            req = req.header(k, v);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let status = resp.status().as_u16();
        let headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                let val = v
                    .to_str()
                    .inspect_err(|e| log::warn!("[request] non-utf8 header {}: {}", k, e))
                    .ok()?;
                Some((k.to_string(), val.to_string()))
            })
            .collect();
        let body = resp
            .bytes()
            .await
            .map_err(|e| format!("Read body failed: {}", e))?
            .to_vec();

        Ok(Response {
            url: parsed.clone(),
            status,
            headers,
            body,
        })
    }

    async fn resolve_outbound_action(&self, url: &Url) -> Option<OutboundAction> {
        let engine = self.rule_engine.read().await;
        engine.as_ref().map(|e| e.evaluate(url).clone())
    }

    pub fn compute_reward(status: u16, latency_ms: f64, success: bool) -> f64 {
        if !success {
            return 0.05;
        }
        let base: f64 = match status {
            200 | 201 | 204 => 0.8,
            301 | 302 | 307 | 308 => 0.6,
            400 | 405 => 0.3,
            401 | 403 => 0.15,
            429 => 0.1,
            500..=599 => 0.3,
            _ => 0.4,
        };
        let latency_bonus: f64 = if latency_ms < 800.0 {
            0.15
        } else if latency_ms < 2000.0 {
            0.05
        } else if latency_ms > 8000.0 {
            -0.15
        } else {
            0.0
        };
        (base + latency_bonus).clamp(0.0, 1.0)
    }

    pub fn compute_reward_with_body(status: u16, latency_ms: f64, body: &[u8]) -> f64 {
        let base = Self::compute_reward(status, latency_ms, true);
        if status == 200 {
            let body_lower = String::from_utf8_lossy(body).to_lowercase();
            let captcha_signals = [
                "/recaptcha/",
                "/challenge-platform",
                "verify you're human",
                "cf-ray",
                "cf-nt_world_browse-verification",
                "turnstile",
                "x-served-by: cloudflare",
                "waf-blocked",
                "why do i have to complete a captcha",
                "_cf_chl_opt",
            ];
            let detected = captcha_signals.iter().any(|s| body_lower.contains(s));
            if detected {
                return 0.05;
            }
        }
        base
    }

    async fn report_reward_for_host(&self, reward: f64, host: Option<&str>) {
        let arm = self.current_combo_arm.read().await.clone();
        let bandit = self.get_bandit(host).await;
        bandit.update(arm, reward);
        if reward < 0.2 {
            self.record_detection().await;
        } else if reward > 0.6 {
            self.clear_detection_streak().await;
        }
    }

    pub async fn report_fetch_result(&self, success: bool) {
        self.report_fetch_result_for_host(success, None).await;
    }

    pub async fn report_fetch_result_for_host(&self, success: bool, host: Option<&str>) {
        let reward = if success { 0.9 } else { 0.1 };
        self.report_reward_for_host(reward, host).await;
    }

    pub(super) async fn record_detection(&self) -> bool {
        let streak = self
            .detection_streak
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        if streak >= 5 {
            log::warn!("[detection] streak={}, triggering seal rotation", streak);
            self.detection_streak
                .store(0, std::sync::atomic::Ordering::Relaxed);
            self.force_clear_fingerprint().await;
            if let Some(ref coord) = *self.coordinator.read().await {
                let config = super::super::config::load();
                let fast_mean = config.rotation.min_interval_secs * 1000.0;
                let fast_std = fast_mean * 0.3;
                coord
                    .set_domain_params(RotationDomain::TlsFingerprint, fast_mean, fast_std)
                    .await;
                coord
                    .set_domain_params(RotationDomain::HttpHeaders, fast_mean, fast_std)
                    .await;
            }
            return true;
        }
        false
    }

    pub(super) async fn clear_detection_streak(&self) {
        self.detection_streak
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn fetch(&self, url: &str) -> Result<Response, String> {
        self.fetch_with_method(url, reqwest::Method::GET, None).await
    }

    pub async fn post(&self, url: &str, body: Vec<u8>, content_type: &str) -> Result<Response, String> {
        self.fetch_with_method(url, reqwest::Method::POST, Some((body, content_type.to_string()))).await
    }

    async fn fetch_with_method(
        &self,
        url: &str,
        method: reqwest::Method,
        body: Option<(Vec<u8>, String)>,
    ) -> Result<Response, String> {
        self.auto_rotate_fingerprint().await;

        {
            let delay_ms = gaussian_delay_ms(275.0, 75.0, 50, 500);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        if let Some(ref coord) = *self.coordinator.read().await {
            if coord.should_rotate(RotationDomain::TimingPattern).await {
                coord.mark_rotated(RotationDomain::TimingPattern).await;
            }
        }

        let parsed = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
        let body_bytes = body.as_ref().map(|(b, _)| b.clone());

        if self.check_tracker {
            if let Some(host) = parsed.host_str() {
                if is_tracker_blocked(host) {
                    return Ok(Response {
                        url: parsed,
                        status: 0,
                        headers: HashMap::new(),
                        body: Vec::new(),
                    });
                }
            }
        }

        let action = self.resolve_outbound_action(&parsed).await;
        let host = parsed.host_str();

        macro_rules! send_req {
            ($client:expr) => {
                self.send_request_with_method($client, &parsed, method.clone(), body_bytes.clone())
                    .await
            };
        }

        match action {
            Some(OutboundAction::Block) => {
                return Ok(Response {
                    url: parsed,
                    status: 0,
                    headers: HashMap::new(),
                    body: Vec::new(),
                });
            }
            Some(OutboundAction::Direct) => {
                let client = self.get_or_create_client("").await?;
                let start = Instant::now();
                match send_req!(&client) {
                    Ok(resp) => {
                        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                        let reward = Self::compute_reward(resp.status, elapsed, true);
                        self.report_reward_for_host(reward, host).await;
                        return Ok(resp);
                    }
                    Err(e) if is_timeout_error(&e) => {
                        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                        let reward = Self::compute_reward(0, elapsed, false);
                        self.report_reward_for_host(reward, host).await;
                        return self.fetch_via_proxy(&parsed).await;
                    }
                    Err(e) => {
                        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                        let reward = Self::compute_reward(0, elapsed, false);
                        self.report_reward_for_host(reward, host).await;
                        return Err(e);
                    }
                }
            }
            Some(OutboundAction::Proxy(proxy_url)) => {
                let client = self.get_or_create_client(&proxy_url).await?;
                let start = Instant::now();
                let result = send_req!(&client);
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                let reward = match &result {
                    Ok(r) => Self::compute_reward(r.status, elapsed, true),
                    Err(_) => Self::compute_reward(0, elapsed, false),
                };
                self.report_reward_for_host(reward, host).await;
                return result;
            }
            Some(OutboundAction::Tor) => {
                let client = self
                    .get_or_create_client(&format!("socks5://{}", TOR_SOCKS_ADDR))
                    .await?;
                let start = Instant::now();
                let result = send_req!(&client);
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                let reward = match &result {
                    Ok(r) => Self::compute_reward(r.status, elapsed, true),
                    Err(_) => Self::compute_reward(0, elapsed, false),
                };
                self.report_reward_for_host(reward, host).await;
                return result;
            }
            None => {}
        }

        if let Some(host) = parsed.host_str() {
            let map = self.isolation_map.read().await;
            if let Some(chain) = map.get(host) {
                let exit_url = chain.current_exit_url().await.unwrap_or_default();
                if !exit_url.is_empty() {
                    let client = self.get_or_create_client(&exit_url).await?;
                    let start = Instant::now();
                    let result = send_req!(&client);
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    let reward = match &result {
                        Ok(r) => {
                            if let Some(label) = chain.current_exit_label().await {
                                chain.update_latency(&label, elapsed).await;
                            }
                            Self::compute_reward(r.status, elapsed, true)
                        }
                        Err(_) => Self::compute_reward(0, elapsed, false),
                    };
                    self.report_reward_for_host(reward, Some(host)).await;
                    return result;
                }
            }
        }

        let proxy_config = self.proxy_config.read().await.clone();
        let (chain, exit_url) = match &proxy_config {
            ProxyConfig::DynamicChain(chain) => {
                let url = chain.current_exit_url().await.unwrap_or_default();
                (Some(chain.clone()), url)
            }
            ProxyConfig::Static(url) => (None, url.clone()),
            ProxyConfig::Tor => (None, format!("socks5://{}", TOR_SOCKS_ADDR)),
            ProxyConfig::None => (None, String::new()),
        };
        let client = self.get_or_create_client(&exit_url).await?;

        let start = Instant::now();
        let result = send_req!(&client);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        let reward = match &result {
            Ok(r) => {
                if let Some(ref chain) = chain {
                    if let Some(label) = chain.current_exit_label().await {
                        chain.update_latency(&label, elapsed).await;
                    }
                }
                Self::compute_reward(r.status, elapsed, true)
            }
            Err(_) => Self::compute_reward(0, elapsed, false),
        };
        self.report_reward_for_host(reward, host).await;
        result
    }

    async fn fetch_via_proxy(&self, parsed: &Url) -> Result<Response, String> {
        log::warn!(
            "[geo] {} timeout, retrying via Tor proxy",
            parsed.host_str().unwrap_or("?")
        );
        if let Some(host) = parsed.host_str() {
            if let Ok(is_cn) = super::super::geo_proxy::domain_resolves_to_china(host).await {
                if is_cn {
                    log::warn!(
                        "[geo] domain {} resolves to China IPs, keeping direct",
                        host
                    );
                    let client = self.get_or_create_client("").await?;
                    let start = Instant::now();
                    let result = self.send_request(&client, parsed).await;
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    let reward = match &result {
                        Ok(r) => Self::compute_reward(r.status, elapsed, true),
                        Err(_) => Self::compute_reward(0, elapsed, false),
                    };
                    self.report_reward_for_host(reward, parsed.host_str()).await;
                    return result;
                }
            }
        }
        let client = self
            .get_or_create_client(&format!("socks5://{}", TOR_SOCKS_ADDR))
            .await?;
        let start = Instant::now();
        let result = self.send_request(&client, parsed).await;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        let reward = match &result {
            Ok(r) => Self::compute_reward(r.status, elapsed, true),
            Err(_) => Self::compute_reward(0, elapsed, false),
        };
        self.report_reward_for_host(reward, parsed.host_str()).await;
        {
            let mut budget = self.entropy_budget.write().await;
            budget.record_request();
            if budget.is_exhausted() {
                log::warn!(
                    "[entropy] budget exhausted ({:.1} bits), forcing full rotation",
                    budget.consumed
                );
                self.force_clear_fingerprint().await;
                budget.reset();
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_text_utf8() {
        let resp = Response {
            url: Url::parse("https://example.com").expect("hardcoded test URL should parse"),
            status: 200,
            headers: HashMap::new(),
            body: b"hello world".to_vec(),
        };
        assert_eq!(
            resp.text().expect("text() on UTF-8 body should succeed"),
            "hello world"
        );
    }

    #[test]
    fn test_response_text_invalid_utf8() {
        let resp = Response {
            url: Url::parse("https://example.com").expect("hardcoded test URL should parse"),
            status: 200,
            headers: HashMap::new(),
            body: vec![0xff, 0xfe, 0x00, 0x01],
        };
        assert!(resp.text().is_err());
    }

    #[test]
    fn test_response_header_exists() {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "text/html".into());
        let resp = Response {
            url: Url::parse("https://example.com").expect("hardcoded test URL should parse"),
            status: 200,
            headers,
            body: Vec::new(),
        };
        assert_eq!(resp.header("content-type"), Some("text/html"));
    }

    #[test]
    fn test_response_header_missing() {
        let resp = Response {
            url: Url::parse("https://example.com").expect("hardcoded test URL should parse"),
            status: 200,
            headers: HashMap::new(),
            body: Vec::new(),
        };
        assert_eq!(resp.header("content-type"), None);
    }

    #[test]
    fn test_response_is_html() {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "text/html; charset=utf-8".into());
        let resp = Response {
            url: Url::parse("https://example.com").expect("hardcoded test URL should parse"),
            status: 200,
            headers,
            body: Vec::new(),
        };
        assert!(resp.is_html());
    }

    #[test]
    fn test_response_not_html() {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "application/json".into());
        let resp = Response {
            url: Url::parse("https://example.com").expect("hardcoded test URL should parse"),
            status: 200,
            headers,
            body: Vec::new(),
        };
        assert!(!resp.is_html());
    }

    #[test]
    fn test_compute_reward_success_200() {
        let reward = StealthHttpClient::compute_reward(200, 500.0, true);
        assert!((reward - 0.95).abs() < 0.01); // 0.8 base + 0.15 latency bonus
    }

    #[test]
    fn test_compute_reward_success_204() {
        let reward = StealthHttpClient::compute_reward(204, 100.0, true);
        assert!((reward - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_redirect() {
        let reward = StealthHttpClient::compute_reward(302, 300.0, true);
        assert!((reward - 0.75).abs() < 0.01); // 0.6 base + 0.15 latency bonus
    }

    #[test]
    fn test_compute_reward_client_error() {
        let reward = StealthHttpClient::compute_reward(400, 500.0, true);
        assert!((reward - 0.45).abs() < 0.01); // 0.3 base + 0.15 latency
    }

    #[test]
    fn test_compute_reward_unauthorized() {
        let reward = StealthHttpClient::compute_reward(401, 500.0, true);
        assert!((reward - 0.30).abs() < 0.01); // 0.15 base + 0.15 latency
    }

    #[test]
    fn test_compute_reward_rate_limited() {
        let reward = StealthHttpClient::compute_reward(429, 500.0, true);
        assert!((reward - 0.25).abs() < 0.01); // 0.1 base + 0.15 latency
    }

    #[test]
    fn test_compute_reward_server_error() {
        let reward = StealthHttpClient::compute_reward(500, 500.0, true);
        assert!((reward - 0.45).abs() < 0.01); // 0.3 base + 0.15 latency
    }

    #[test]
    fn test_compute_reward_failure() {
        let reward = StealthHttpClient::compute_reward(0, 9999.0, false);
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_latency_penalty() {
        let slow = StealthHttpClient::compute_reward(200, 9000.0, true);
        let fast = StealthHttpClient::compute_reward(200, 100.0, true);
        assert!(slow < fast); // slow should be penalized
    }

    #[test]
    fn test_compute_reward_unknown_status() {
        let reward = StealthHttpClient::compute_reward(999, 500.0, true);
        assert!((reward - 0.55).abs() < 0.01); // 0.4 base + 0.15 latency
    }

    #[test]
    fn test_compute_reward_with_body_no_captcha() {
        let reward = StealthHttpClient::compute_reward_with_body(200, 500.0, b"normal content");
        assert!((reward - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_with_body_captcha_detected() {
        let reward =
            StealthHttpClient::compute_reward_with_body(200, 500.0, b"verify you're human");
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_with_body_cloudflare() {
        let reward = StealthHttpClient::compute_reward_with_body(200, 500.0, b"cf-ray: abc123");
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_with_body_recaptcha() {
        let reward = StealthHttpClient::compute_reward_with_body(200, 500.0, b"/recaptcha/api.js");
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_with_body_turnstile() {
        let reward = StealthHttpClient::compute_reward_with_body(200, 500.0, b"turnstile");
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_with_body_challenge_platform() {
        let reward =
            StealthHttpClient::compute_reward_with_body(200, 500.0, b"/challenge-platform");
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_with_body_non_200() {
        let reward = StealthHttpClient::compute_reward_with_body(404, 500.0, b"anything");
        assert!((reward - 0.55).abs() < 0.01); // 0.4 base + 0.15 latency, no captcha check for non-200
    }

    #[test]
    fn test_compute_reward_with_body_waf_blocked() {
        let reward = StealthHttpClient::compute_reward_with_body(200, 500.0, b"waf-blocked");
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_with_body_cf_nt_world_browse_verification() {
        let reward = StealthHttpClient::compute_reward_with_body(
            200,
            500.0,
            b"cf-nt_world_browse-verification",
        );
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_response_is_html_no_content_type() {
        let resp = Response {
            url: Url::parse("https://example.com").expect("hardcoded test URL should parse"),
            status: 200,
            headers: HashMap::new(),
            body: Vec::new(),
        };
        assert!(!resp.is_html());
    }

    #[test]
    fn test_response_empty_body_text() {
        let resp = Response {
            url: Url::parse("https://example.com").expect("hardcoded test URL should parse"),
            status: 200,
            headers: HashMap::new(),
            body: Vec::new(),
        };
        assert_eq!(
            resp.text()
                .expect("text() on empty body should return empty string"),
            ""
        );
    }

    #[test]
    fn test_compute_reward_with_body_x_served_by() {
        let reward =
            StealthHttpClient::compute_reward_with_body(200, 500.0, b"x-served-by: cloudflare");
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_with_body_cf_chl_opt() {
        let reward = StealthHttpClient::compute_reward_with_body(200, 500.0, b"_cf_chl_opt");
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_with_body_captcha_why() {
        let reward = StealthHttpClient::compute_reward_with_body(
            200,
            500.0,
            b"why do i have to complete a captcha",
        );
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_clamp_low_extreme() {
        let reward = StealthHttpClient::compute_reward(0, 999999.0, false);
        assert!(reward >= 0.0);
        assert!((reward - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_compute_reward_status_0_success() {
        let reward = StealthHttpClient::compute_reward(0, 500.0, true);
        assert!((reward - 0.55).abs() < 0.01); // _ => 0.4 + 0.15 latency
    }
}

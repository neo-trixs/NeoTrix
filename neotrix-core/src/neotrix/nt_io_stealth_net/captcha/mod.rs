use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::RwLock;

// ─── Part 1: CAPTCHA Type Detection ───────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaType {
    CloudflareTurnstile,
    RecaptchaV2,
    RecaptchaV3,
    HCaptcha,
    Custom(&'static str),
}

#[derive(Debug, Clone)]
pub struct CaptchaDetection {
    pub detected: bool,
    pub captcha_type: Option<CaptchaType>,
    pub confidence: f64,
    pub selector: Option<String>,
    pub frame_src: Option<String>,
    pub site_key: Option<String>,
}

pub struct CaptchaDetector;

impl CaptchaDetector {
    pub fn detect(html: &str) -> CaptchaDetection {
        let mut confidence = 0.0_f64;
        let mut captcha_type = None;
        let mut selector = None;
        let mut frame_src = None;
        let mut site_key = None;

        // Cloudflare Turnstile
        if html.contains("turnstile") || html.contains("cf-turnstile") {
            confidence += 0.35;
            captcha_type = Some(CaptchaType::CloudflareTurnstile);
            selector = Some(".cf-turnstile".into());
            if html.contains("challenges.cloudflare.com") {
                confidence += 0.25;
            }
            if let Some(key) = Self::extract_site_key(html, "data-sitekey") {
                site_key = Some(key);
                confidence += 0.20;
            }
            if html.contains("cf-turnstile") {
                confidence += 0.20;
            }
        }

        // reCAPTCHA V2
        if html.contains("recaptcha/api.js") && !html.contains("recaptcha/api.js?render=") {
            confidence += 0.35;
            if captcha_type.is_none() {
                captcha_type = Some(CaptchaType::RecaptchaV2);
            }
            if html.contains("g-recaptcha") {
                confidence += 0.25;
                selector = Some(".g-recaptcha".into());
            }
            if let Some(key) = Self::extract_site_key(html, "data-sitekey") {
                site_key = Some(key);
                confidence += 0.20;
            }
            if html.contains("recaptcha/api.js") {
                confidence += 0.20;
            }
        }

        // reCAPTCHA V3
        if html.contains("recaptcha/api.js?render=") {
            confidence += 0.35;
            if captcha_type.is_none()
                || matches!(captcha_type, Some(CaptchaType::CloudflareTurnstile))
            {
                captcha_type = Some(CaptchaType::RecaptchaV3);
            }
            if let Some(key) = Self::extract_site_key(html, "data-sitekey") {
                site_key = Some(key);
                confidence += 0.20;
            }
            if html.contains("grecaptcha.execute") {
                confidence += 0.25;
            }
            if html.contains("recaptcha/api.js?render=") {
                confidence += 0.20;
            }
        }

        // hCaptcha
        if html.contains("hcaptcha.com/1/api.js") {
            confidence += 0.35;
            if captcha_type.is_none()
                || matches!(captcha_type, Some(CaptchaType::CloudflareTurnstile))
            {
                captcha_type = Some(CaptchaType::HCaptcha);
            }
            if html.contains("h-captcha") {
                confidence += 0.25;
                selector = Some(".h-captcha".into());
            }
            if let Some(key) = Self::extract_site_key(html, "data-sitekey") {
                site_key = Some(key);
                confidence += 0.20;
            }
            if html.contains("hcaptcha.com/1/api.js") {
                confidence += 0.20;
            }
        }

        let detected = confidence > 0.5;

        CaptchaDetection {
            detected,
            captcha_type,
            confidence: confidence.clamp(0.0, 1.0),
            selector,
            frame_src,
            site_key,
        }
    }

    fn extract_site_key(html: &str, attr_with_key: &str) -> Option<String> {
        let patterns = &["data-sitekey", "data-site_key", "sitekey"];
        for pat in patterns {
            let mut search_start = 0;
            while let Some(pos) = html[search_start..].find(pat) {
                let abs_pos = search_start + pos;
                let after = &html[abs_pos + pat.len()..];
                // skip past '=' and optional quotes
                let remainder = after.trim_start();
                if let Some(eq_pos) = remainder.find('=') {
                    let after_eq = remainder[eq_pos + 1..].trim_start();
                    let end = after_eq
                        .find(|c: char| c == '"' || c == '\'' || c == ' ' || c == '>')
                        .unwrap_or(after_eq.len());
                    let value = after_eq[..end].trim_matches(|c: char| c == '"' || c == '\'');
                    if !value.is_empty() && value.len() < 256 {
                        return Some(value.to_string());
                    }
                }
                search_start = abs_pos + pat.len() + 1;
            }
        }
        None
    }

    pub fn detect_all(html: &str) -> Vec<CaptchaDetection> {
        let mut results = Vec::new();

        // Cloudflare Turnstile
        if html.contains("turnstile")
            || html.contains("cf-turnstile")
            || html.contains("challenges.cloudflare.com")
        {
            let mut confidence = 0.0;
            if html.contains("turnstile") {
                confidence += 0.3;
            }
            if html.contains("cf-turnstile") {
                confidence += 0.3;
            }
            if html.contains("challenges.cloudflare.com") {
                confidence += 0.25;
            }
            let site_key = Self::extract_site_key(html, "data-sitekey");
            if site_key.is_some() {
                confidence += 0.15;
            }
            results.push(CaptchaDetection {
                detected: confidence > 0.5,
                captcha_type: Some(CaptchaType::CloudflareTurnstile),
                confidence: confidence.clamp(0.0, 1.0),
                selector: Some(".cf-turnstile".into()),
                frame_src: None,
                site_key,
            });
        }

        // reCAPTCHA V2
        if html.contains("recaptcha/api.js") && !html.contains("recaptcha/api.js?render=") {
            let mut confidence = 0.0;
            if html.contains("recaptcha/api.js") {
                confidence += 0.3;
            }
            if html.contains("g-recaptcha") {
                confidence += 0.3;
            }
            let site_key = Self::extract_site_key(html, "data-sitekey");
            if site_key.is_some() {
                confidence += 0.25;
            }
            if html.contains("data-theme") {
                confidence += 0.15;
            }
            results.push(CaptchaDetection {
                detected: confidence > 0.5,
                captcha_type: Some(CaptchaType::RecaptchaV2),
                confidence: confidence.clamp(0.0, 1.0),
                selector: Some(".g-recaptcha".into()),
                frame_src: None,
                site_key,
            });
        }

        // reCAPTCHA V3
        if html.contains("recaptcha/api.js?render=") {
            let mut confidence = 0.0;
            if html.contains("recaptcha/api.js?render=") {
                confidence += 0.3;
            }
            if html.contains("grecaptcha.execute") {
                confidence += 0.3;
            }
            let site_key = Self::extract_site_key(html, "data-sitekey");
            if site_key.is_some() {
                confidence += 0.25;
            }
            if html.contains("data-action") {
                confidence += 0.15;
            }
            results.push(CaptchaDetection {
                detected: confidence > 0.5,
                captcha_type: Some(CaptchaType::RecaptchaV3),
                confidence: confidence.clamp(0.0, 1.0),
                selector: None,
                frame_src: None,
                site_key,
            });
        }

        // hCaptcha
        if html.contains("hcaptcha.com/1/api.js") {
            let mut confidence = 0.0;
            if html.contains("hcaptcha.com/1/api.js") {
                confidence += 0.3;
            }
            if html.contains("h-captcha") {
                confidence += 0.3;
            }
            let site_key = Self::extract_site_key(html, "data-sitekey");
            if site_key.is_some() {
                confidence += 0.25;
            }
            if html.contains("data-sentry") {
                confidence += 0.15;
            }
            results.push(CaptchaDetection {
                detected: confidence > 0.5,
                captcha_type: Some(CaptchaType::HCaptcha),
                confidence: confidence.clamp(0.0, 1.0),
                selector: Some(".h-captcha".into()),
                frame_src: None,
                site_key,
            });
        }

        results
    }
}

// ─── Part 2: CAPTCHA Solvers ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub token: String,
    pub solver: &'static str,
    pub solve_time_ms: u64,
    pub success: bool,
}

#[async_trait]
pub trait CaptchaSolver: Send + Sync {
    async fn solve(
        &self,
        page_html: &str,
        captcha_type: CaptchaType,
    ) -> Result<SolveResult, String>;
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
}

pub struct DummySolver;

#[async_trait]
impl CaptchaSolver for DummySolver {
    async fn solve(
        &self,
        _page_html: &str,
        captcha_type: CaptchaType,
    ) -> Result<SolveResult, String> {
        let type_str = match captcha_type {
            CaptchaType::CloudflareTurnstile => "turnstile",
            CaptchaType::RecaptchaV2 => "recaptcha_v2",
            CaptchaType::RecaptchaV3 => "recaptcha_v3",
            CaptchaType::HCaptcha => "hcaptcha",
            CaptchaType::Custom(s) => s,
        };
        Ok(SolveResult {
            token: format!("dummy_token_{}", type_str),
            solver: "dummy",
            solve_time_ms: 0,
            success: true,
        })
    }

    fn name(&self) -> &'static str {
        "dummy"
    }

    fn is_available(&self) -> bool {
        true
    }
}

pub struct TwoCaptchaSolver {
    api_key: String,
    client: reqwest::Client,
}

impl TwoCaptchaSolver {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn submit_captcha(
        &self,
        site_key: &str,
        page_url: &str,
        captcha_type: CaptchaType,
    ) -> Result<String, String> {
        let method = match captcha_type {
            CaptchaType::RecaptchaV2 | CaptchaType::RecaptchaV3 => "userrecaptcha",
            CaptchaType::HCaptcha => "hcaptcha",
            CaptchaType::CloudflareTurnstile => "turnstile",
            CaptchaType::Custom(_) => "userrecaptcha",
        };

        let params = [
            ("key", self.api_key.as_str()),
            ("method", method),
            ("googlekey", site_key),
            ("pageurl", page_url),
            ("json", "1"),
        ];

        let resp = self
            .client
            .post("https://2captcha.com/in.php")
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("2captcha submit error: {}", e))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("2captcha parse error: {}", e))?;

        if body.get("status").and_then(|v| v.as_i64()) == Some(1) {
            body.get("request")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| "2captcha: missing request_id".to_string())
        } else {
            let err = body
                .get("error_description")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown 2captcha error");
            Err(format!("2captcha: {}", err))
        }
    }

    async fn poll_result(&self, request_id: &str) -> Result<String, String> {
        let params = [
            ("key", self.api_key.as_str()),
            ("action", "get"),
            ("id", request_id),
            ("json", "1"),
        ];

        let max_attempts = 60;
        for _ in 0..max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let resp = self
                .client
                .post("https://2captcha.com/res.php")
                .form(&params)
                .send()
                .await
                .map_err(|e| format!("2captcha poll error: {}", e))?;

            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("2captcha poll parse error: {}", e))?;

            if body.get("status").and_then(|v| v.as_i64()) == Some(1) {
                return body
                    .get("request")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| "2captcha: missing token".to_string());
            }

            let request = body.get("request").and_then(|v| v.as_str()).unwrap_or("");
            if request == "CAPCHA_NOT_READY" {
                continue;
            }

            return Err(format!("2captcha: {}", request));
        }

        Err("2captcha: timeout after 5 minutes".to_string())
    }
}

#[async_trait]
impl CaptchaSolver for TwoCaptchaSolver {
    async fn solve(
        &self,
        page_html: &str,
        captcha_type: CaptchaType,
    ) -> Result<SolveResult, String> {
        let start = Instant::now();
        let detection = CaptchaDetector::detect(page_html);
        let site_key = detection
            .site_key
            .ok_or_else(|| "2captcha: no site key found".to_string())?;
        let page_url = "https://unknown"; // caller should pass real URL
        let request_id = self
            .submit_captcha(&site_key, page_url, captcha_type)
            .await?;
        let token = self.poll_result(&request_id).await?;
        let elapsed = start.elapsed().as_millis() as u64;

        Ok(SolveResult {
            token,
            solver: "2captcha",
            solve_time_ms: elapsed,
            success: true,
        })
    }

    fn name(&self) -> &'static str {
        "2captcha"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

pub struct CapsolverSolver {
    api_key: String,
    client: reqwest::Client,
}

impl CapsolverSolver {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn create_task(
        &self,
        site_key: &str,
        page_url: &str,
        captcha_type: CaptchaType,
    ) -> Result<String, String> {
        let task_type = match captcha_type {
            CaptchaType::RecaptchaV2 => "ReCaptchaV2TaskProxyless",
            CaptchaType::RecaptchaV3 => "ReCaptchaV3TaskProxyless",
            CaptchaType::HCaptcha => "HCaptchaTaskProxyless",
            CaptchaType::CloudflareTurnstile => "AntiTurnstileTaskProxyless",
            CaptchaType::Custom(_) => "ReCaptchaV2TaskProxyless",
        };

        let body = serde_json::json!({
            "clientKey": self.api_key,
            "task": {
                "type": task_type,
                "websiteURL": page_url,
                "websiteKey": site_key,
            }
        });

        let resp = self
            .client
            .post("https://api.capsolver.com/createTask")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("capsolver create error: {}", e))?;

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("capsolver parse error: {}", e))?;

        if let Some(task_id) = result.get("taskId").and_then(|v| v.as_str()) {
            Ok(task_id.to_string())
        } else {
            let err = result
                .get("errorDescription")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown capsolver error");
            Err(format!("capsolver: {}", err))
        }
    }

    async fn poll_task(&self, task_id: &str) -> Result<String, String> {
        let body = serde_json::json!({
            "clientKey": self.api_key,
            "taskId": task_id,
        });

        let max_attempts = 60;
        for _ in 0..max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            let resp = self
                .client
                .post("https://api.capsolver.com/getTaskResult")
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("capsolver poll error: {}", e))?;

            let result: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("capsolver poll parse error: {}", e))?;

            let status = result.get("status").and_then(|v| v.as_str()).unwrap_or("");
            if status == "ready" {
                return result
                    .get("solution")
                    .and_then(|s| {
                        s.get("token")
                            .or_else(|| s.get("gRecaptchaResponse"))
                            .or_else(|| s.get("cfTurnstileResponse"))
                    })
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| "capsolver: missing token in solution".to_string());
            }
            if status == "processing" {
                continue;
            }
            return Err(format!("capsolver: unexpected status '{}'", status));
        }

        Err("capsolver: timeout after 3 minutes".to_string())
    }
}

#[async_trait]
impl CaptchaSolver for CapsolverSolver {
    async fn solve(
        &self,
        page_html: &str,
        captcha_type: CaptchaType,
    ) -> Result<SolveResult, String> {
        let start = Instant::now();
        let detection = CaptchaDetector::detect(page_html);
        let site_key = detection
            .site_key
            .ok_or_else(|| "capsolver: no site key found".to_string())?;
        let page_url = "https://unknown";
        let task_id = self.create_task(&site_key, page_url, captcha_type).await?;
        let token = self.poll_task(&task_id).await?;
        let elapsed = start.elapsed().as_millis() as u64;

        Ok(SolveResult {
            token,
            solver: "capsolver",
            solve_time_ms: elapsed,
            success: true,
        })
    }

    fn name(&self) -> &'static str {
        "capsolver"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

// ─── Part 3: CaptchaSolutionManager ──────────────────────────────────────

pub struct CaptchaSolutionManager {
    solver: Box<dyn CaptchaSolver>,
}

impl CaptchaSolutionManager {
    pub fn new(solver: Box<dyn CaptchaSolver>) -> Self {
        Self { solver }
    }

    pub async fn detect_and_solve(&self, html: &str, _page_url: &str) -> CaptchaOutcome {
        let detection = CaptchaDetector::detect(html);
        if !detection.detected {
            return CaptchaOutcome {
                detection,
                result: None,
                solved: false,
            };
        }

        let captcha_type = match detection.captcha_type {
            Some(ct) => ct,
            None => {
                return CaptchaOutcome {
                    detection,
                    result: None,
                    solved: false,
                }
            }
        };

        match self.solver.solve(html, captcha_type).await {
            Ok(result) => CaptchaOutcome {
                detection,
                result: Some(result),
                solved: true,
            },
            Err(_) => CaptchaOutcome {
                detection,
                result: None,
                solved: false,
            },
        }
    }

    pub fn solver(&self) -> &dyn CaptchaSolver {
        self.solver.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct CaptchaOutcome {
    pub detection: CaptchaDetection,
    pub result: Option<SolveResult>,
    pub solved: bool,
}

// ─── Part 4: BrowserSessionPool (G322.1) ─────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Starting,
    Ready,
    Busy,
    Crashed,
    Closed,
}

#[derive(Debug, Clone)]
pub struct BrowserSession {
    pub id: u64,
    pub fingerprint_variant: String,
    pub proxy_url: Option<String>,
    pub cdp_ws_url: Option<String>,
    pub status: SessionStatus,
    pub created_at: Instant,
    pub cookies_populated: bool,
    pub login_state: &'static str,
}

impl BrowserSession {
    pub fn new(id: u64, fingerprint_variant: &str, proxy_url: Option<&str>) -> Self {
        Self {
            id,
            fingerprint_variant: fingerprint_variant.to_string(),
            proxy_url: proxy_url.map(|s| s.to_string()),
            cdp_ws_url: None,
            status: SessionStatus::Starting,
            created_at: Instant::now(),
            cookies_populated: false,
            login_state: "none",
        }
    }
}

pub struct BrowserSessionPool {
    sessions: RwLock<Vec<BrowserSession>>,
    next_id: AtomicU64,
    target_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPoolHealth {
    pub total: usize,
    pub ready: usize,
    pub busy: usize,
    pub crashed: usize,
    pub starting: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPoolStats {
    pub total_created: u64,
    pub total_closed: u64,
    pub current_size: usize,
    pub target_size: usize,
}

impl BrowserSessionPool {
    pub fn new(target_size: usize) -> Self {
        Self {
            sessions: RwLock::new(Vec::with_capacity(target_size)),
            next_id: AtomicU64::new(1),
            target_size,
        }
    }

    pub async fn create_session(
        &self,
        fingerprint: &str,
        proxy: Option<&str>,
    ) -> Result<u64, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let session = BrowserSession::new(id, fingerprint, proxy);
        let mut guard = self.sessions.write().await;
        guard.push(session);
        Ok(id)
    }

    pub async fn close_session(&self, id: u64) -> Result<(), String> {
        let mut guard = self.sessions.write().await;
        if let Some(pos) = guard.iter().position(|s| s.id == id) {
            guard.remove(pos);
            Ok(())
        } else {
            Err(format!("session {} not found", id))
        }
    }

    pub async fn get_ready_session(
        &self,
        preferred_fingerprint: Option<&str>,
    ) -> Option<BrowserSession> {
        let mut guard = self.sessions.write().await;
        // First pass: match preferred fingerprint
        if let Some(fp) = preferred_fingerprint {
            if let Some(pos) = guard
                .iter()
                .position(|s| s.status == SessionStatus::Ready && s.fingerprint_variant == fp)
            {
                let mut session = guard.remove(pos);
                session.status = SessionStatus::Busy;
                guard.push(session.clone());
                return Some(session);
            }
        }
        // Second pass: any ready session
        if let Some(pos) = guard.iter().position(|s| s.status == SessionStatus::Ready) {
            let mut session = guard.remove(pos);
            session.status = SessionStatus::Busy;
            guard.push(session.clone());
            return Some(session);
        }
        None
    }

    pub async fn mark_busy(&self, id: u64) {
        let mut guard = self.sessions.write().await;
        if let Some(session) = guard.iter_mut().find(|s| s.id == id) {
            session.status = SessionStatus::Busy;
        }
    }

    pub async fn mark_ready(&self, id: u64) {
        let mut guard = self.sessions.write().await;
        if let Some(session) = guard.iter_mut().find(|s| s.id == id) {
            session.status = SessionStatus::Ready;
        }
    }

    pub async fn mark_crashed(&self, id: u64) {
        let mut guard = self.sessions.write().await;
        if let Some(session) = guard.iter_mut().find(|s| s.id == id) {
            session.status = SessionStatus::Crashed;
        }
    }

    pub async fn clean_crashed(&self) -> usize {
        let mut guard = self.sessions.write().await;
        let before = guard.len();
        guard.retain(|s| s.status != SessionStatus::Crashed && s.status != SessionStatus::Closed);
        let removed = before - guard.len();
        // recreate up to target_size
        while guard.len() < self.target_size {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            guard.push(BrowserSession::new(id, "default", None));
        }
        removed
    }

    pub async fn health(&self) -> SessionPoolHealth {
        let guard = self.sessions.read().await;
        let mut health = SessionPoolHealth {
            total: guard.len(),
            ready: 0,
            busy: 0,
            crashed: 0,
            starting: 0,
        };
        for s in guard.iter() {
            match s.status {
                SessionStatus::Ready => health.ready += 1,
                SessionStatus::Busy => health.busy += 1,
                SessionStatus::Crashed => health.crashed += 1,
                SessionStatus::Starting => health.starting += 1,
                SessionStatus::Closed => {}
            }
        }
        health
    }

    pub fn stats(&self) -> SessionPoolStats {
        SessionPoolStats {
            total_created: self.next_id.load(Ordering::Relaxed) - 1,
            total_closed: 0,
            current_size: self.target_size,
            target_size: self.target_size,
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Part 1: Detector Tests ─────────────────────────────────────────

    const TURNSTILE_HTML: &str = r#"
<html><head><script src="https://challenges.cloudflare.com/turnstile/v0/api.js"></script></head>
<body><div class="cf-turnstile" data-sitekey="0x4AAAAAAABC123"></div></body>
</html>"#;

    const RECAPTCHA_V2_HTML: &str = r#"
<html><head><script src="https://www.google.com/recaptcha/api.js"></script></head>
<body><div class="g-recaptcha" data-sitekey="6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI"></div></body>
</html>"#;

    const RECAPTCHA_V3_HTML: &str = r#"
<html><head><script src="https://www.google.com/recaptcha/api.js?render=6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI"></script></head>
<body><script>grecaptcha.execute('6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI');</script></body>
</html>"#;

    const HCAPTCHA_HTML: &str = r#"
<html><head><script src="https://hcaptcha.com/1/api.js"></script></head>
<body><div class="h-captcha" data-sitekey="10000000-ffff-ffff-ffff-000000000000"></div></body>
</html>"#;

    const PLAIN_HTML: &str = r#"<html><body><p>No CAPTCHA here</p></body></html>"#;

    const MULTI_CAPTCHA_HTML: &str = r#"
<html>
<head>
<script src="https://challenges.cloudflare.com/turnstile/v0/api.js"></script>
<script src="https://www.google.com/recaptcha/api.js"></script>
</head>
<body>
<div class="cf-turnstile" data-sitekey="0x4AAAAAAABC123"></div>
<div class="g-recaptcha" data-sitekey="6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI"></div>
</body>
</html>"#;

    #[test]
    fn test_captcha_detector_turnstile() {
        let result = CaptchaDetector::detect(TURNSTILE_HTML);
        assert!(result.detected);
        assert_eq!(result.captcha_type, Some(CaptchaType::CloudflareTurnstile));
        assert!(result.confidence > 0.7);
        assert_eq!(result.site_key.as_deref(), Some("0x4AAAAAAABC123"));
        assert_eq!(result.selector.as_deref(), Some(".cf-turnstile"));
    }

    #[test]
    fn test_captcha_detector_recaptcha_v2() {
        let result = CaptchaDetector::detect(RECAPTCHA_V2_HTML);
        assert!(result.detected);
        assert_eq!(result.captcha_type, Some(CaptchaType::RecaptchaV2));
        assert!(result.confidence > 0.7);
        assert_eq!(
            result.site_key.as_deref(),
            Some("6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI")
        );
        assert_eq!(result.selector.as_deref(), Some(".g-recaptcha"));
    }

    #[test]
    fn test_captcha_detector_recaptcha_v3() {
        let result = CaptchaDetector::detect(RECAPTCHA_V3_HTML);
        assert!(result.detected);
        assert_eq!(result.captcha_type, Some(CaptchaType::RecaptchaV3));
        assert!(result.confidence > 0.7);
        assert_eq!(
            result.site_key.as_deref(),
            Some("6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI")
        );
    }

    #[test]
    fn test_captcha_detector_hcaptcha() {
        let result = CaptchaDetector::detect(HCAPTCHA_HTML);
        assert!(result.detected);
        assert_eq!(result.captcha_type, Some(CaptchaType::HCaptcha));
        assert!(result.confidence > 0.7);
        assert_eq!(
            result.site_key.as_deref(),
            Some("10000000-ffff-ffff-ffff-000000000000")
        );
        assert_eq!(result.selector.as_deref(), Some(".h-captcha"));
    }

    #[test]
    fn test_captcha_detector_no_captcha() {
        let result = CaptchaDetector::detect(PLAIN_HTML);
        assert!(!result.detected);
        assert!(result.confidence < 0.5);
    }

    #[test]
    fn test_captcha_detector_extract_site_key() {
        let html = r#"<div data-sitekey="abc123"></div>"#;
        assert_eq!(
            CaptchaDetector::extract_site_key(html, "data-sitekey"),
            Some("abc123".into())
        );

        let html = r#"<div data-site_key='xyz789'></div>"#;
        assert_eq!(
            CaptchaDetector::extract_site_key(html, "data-site_key"),
            Some("xyz789".into())
        );

        let html = r#"<p>no key here</p>"#;
        assert_eq!(
            CaptchaDetector::extract_site_key(html, "data-sitekey"),
            None
        );
    }

    #[test]
    fn test_captcha_detector_detect_all() {
        let results = CaptchaDetector::detect_all(MULTI_CAPTCHA_HTML);
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .any(|r| r.captcha_type == Some(CaptchaType::CloudflareTurnstile)));
        assert!(results
            .iter()
            .any(|r| r.captcha_type == Some(CaptchaType::RecaptchaV2)));
    }

    #[test]
    fn test_captcha_detector_confidence_scoring() {
        // Partial match: only turnstile div without script
        let partial = r#"<div class="cf-turnstile"></div>"#;
        let result = CaptchaDetector::detect(partial);
        assert!(result.detected);
        assert!(result.confidence < 0.8);
        assert!(result.confidence > 0.3);

        // Full match
        let full = CaptchaDetector::detect(TURNSTILE_HTML);
        assert!(full.confidence > 0.8);
    }

    // ── Part 2: Solver Tests ────────────────────────────────────────────

    #[tokio::test]
    async fn test_dummy_solver_solves_immediately() {
        let solver = DummySolver;
        let result = solver
            .solve(TURNSTILE_HTML, CaptchaType::CloudflareTurnstile)
            .await;
        assert!(result.is_ok());
        let solve = result.unwrap();
        assert!(solve.success);
        assert_eq!(solve.solver, "dummy");
        assert_eq!(solve.solve_time_ms, 0);
        assert!(solve.token.starts_with("dummy_token_"));
    }

    #[tokio::test]
    async fn test_dummy_solver_all_types() {
        let solver = DummySolver;
        for ct in &[
            CaptchaType::CloudflareTurnstile,
            CaptchaType::RecaptchaV2,
            CaptchaType::RecaptchaV3,
            CaptchaType::HCaptcha,
            CaptchaType::Custom("test"),
        ] {
            let result = solver.solve("", *ct).await.unwrap();
            assert!(result.success);
            assert!(!result.token.is_empty());
        }
    }

    #[tokio::test]
    async fn test_dummy_solver_name_and_available() {
        let solver = DummySolver;
        assert_eq!(solver.name(), "dummy");
        assert!(solver.is_available());
    }

    // ── Part 3: CaptchaSolutionManager Tests ────────────────────────────

    #[tokio::test]
    async fn test_captcha_solution_manager_detect_unsolved() {
        let manager = CaptchaSolutionManager::new(Box::new(DummySolver));
        let outcome = manager
            .detect_and_solve(PLAIN_HTML, "https://example.com")
            .await;
        assert!(!outcome.solved);
        assert!(!outcome.detection.detected);
        assert!(outcome.result.is_none());
    }

    #[tokio::test]
    async fn test_captcha_solution_manager_detect_and_solve() {
        let manager = CaptchaSolutionManager::new(Box::new(DummySolver));
        let outcome = manager
            .detect_and_solve(TURNSTILE_HTML, "https://example.com")
            .await;
        assert!(outcome.detection.detected);
        assert_eq!(
            outcome.detection.captcha_type,
            Some(CaptchaType::CloudflareTurnstile)
        );
        assert!(outcome.result.is_some());
        let result = outcome.result.unwrap();
        assert!(result.success);
        assert_eq!(result.solver, "dummy");
    }

    // ── Part 4: BrowserSessionPool Tests ────────────────────────────────

    #[tokio::test]
    async fn test_browser_session_pool_create() {
        let pool = BrowserSessionPool::new(5);
        let id = pool
            .create_session("chrome_120", Some("http://proxy:8080"))
            .await
            .unwrap();
        assert_eq!(id, 1);
        let stats = pool.stats();
        assert_eq!(stats.total_created, 1);
    }

    #[tokio::test]
    async fn test_browser_session_pool_create_and_close() {
        let pool = BrowserSessionPool::new(5);
        let id = pool.create_session("firefox_110", None).await.unwrap();
        let health = pool.health().await;
        assert_eq!(health.total, 1);
        assert_eq!(health.starting, 1);

        pool.mark_ready(id).await;
        let health = pool.health().await;
        assert_eq!(health.ready, 1);

        pool.close_session(id).await.unwrap();
        let health = pool.health().await;
        assert_eq!(health.total, 0);
    }

    #[tokio::test]
    async fn test_browser_session_pool_health() {
        let pool = BrowserSessionPool::new(10);
        let id1 = pool.create_session("a", None).await.unwrap();
        let id2 = pool.create_session("b", None).await.unwrap();
        let id3 = pool.create_session("c", None).await.unwrap();

        pool.mark_ready(id1).await;
        pool.mark_busy(id2).await;
        // id3 stays Starting

        let health = pool.health().await;
        assert_eq!(health.total, 3);
        assert_eq!(health.ready, 1);
        assert_eq!(health.busy, 1);
        assert_eq!(health.starting, 1);
        assert_eq!(health.crashed, 0);
    }

    #[tokio::test]
    async fn test_browser_session_pool_mark_crashed() {
        let pool = BrowserSessionPool::new(5);
        let id = pool.create_session("chrome_120", None).await.unwrap();
        pool.mark_ready(id).await;

        pool.mark_crashed(id).await;
        let health = pool.health().await;
        assert_eq!(health.crashed, 1);
        assert_eq!(health.ready, 0);
    }

    #[tokio::test]
    async fn test_browser_session_pool_clean_crashed() {
        let pool = BrowserSessionPool::new(3);
        let id1 = pool.create_session("a", None).await.unwrap();
        let id2 = pool.create_session("b", None).await.unwrap();
        let id3 = pool.create_session("c", None).await.unwrap();

        pool.mark_ready(id1).await;
        pool.mark_crashed(id2).await;
        pool.mark_crashed(id3).await;

        let removed = pool.clean_crashed().await;
        assert_eq!(removed, 2);

        let health = pool.health().await;
        assert_eq!(health.total, 3); // recreated to target_size
        assert_eq!(health.ready, 1);
    }

    #[tokio::test]
    async fn test_browser_session_pool_get_ready_session() {
        let pool = BrowserSessionPool::new(5);
        let id = pool.create_session("chrome_120", None).await.unwrap();
        pool.mark_ready(id).await;

        let session = pool.get_ready_session(Some("chrome_120")).await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().id, id);

        // should be busy now
        let health = pool.health().await;
        assert_eq!(health.busy, 1);
    }

    #[tokio::test]
    async fn test_browser_session_pool_get_ready_session_preferred() {
        let pool = BrowserSessionPool::new(5);
        let id_chrome = pool.create_session("chrome_120", None).await.unwrap();
        let id_firefox = pool.create_session("firefox_110", None).await.unwrap();
        pool.mark_ready(id_chrome).await;
        pool.mark_ready(id_firefox).await;

        // Preferred fingerprint should match
        let session = pool.get_ready_session(Some("firefox_110")).await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().fingerprint_variant, "firefox_110");

        // Non-matching preferred -> falls back to any
        let session = pool.get_ready_session(Some("nonexistent")).await;
        assert!(session.is_some());
    }

    #[tokio::test]
    async fn test_captcha_outcome_unsolved() {
        let outcome = CaptchaOutcome {
            detection: CaptchaDetection {
                detected: true,
                captcha_type: Some(CaptchaType::RecaptchaV2),
                confidence: 0.9,
                selector: Some(".g-recaptcha".into()),
                frame_src: None,
                site_key: Some("test_key".into()),
            },
            result: None,
            solved: false,
        };
        assert!(!outcome.solved);
        assert!(outcome.detection.detected);
        assert!(outcome.result.is_none());
    }
}

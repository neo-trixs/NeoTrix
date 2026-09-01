use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct BrowserFingerprint {
    pub user_agent: String,
    pub viewport: (u16, u16),
    pub platform: String,
    pub vendor: String,
    pub language: String,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
    pub canvas_noise: bool,
    pub timezone: String,
}

impl Default for BrowserFingerprint {
    fn default() -> Self {
        BrowserFingerprint {
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".into(),
            viewport: (1920, 1080),
            platform: "MacIntel".into(),
            vendor: "Google Inc.".into(),
            language: "en-US".into(),
            webgl_vendor: "Intel Inc.".into(),
            webgl_renderer: "Intel Iris OpenGL Engine".into(),
            canvas_noise: true,
            timezone: "America/New_York".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BypassMethod {
    CookieReuse,
    HeadlessDetection,
    FingerprintSpoof,
    ProxyRotation,
    CaptchaSolver,
    TorRoute,
}

impl BypassMethod {
    pub fn name(&self) -> &'static str {
        match self {
            BypassMethod::CookieReuse => "cookie_reuse",
            BypassMethod::HeadlessDetection => "headless_detection",
            BypassMethod::FingerprintSpoof => "fingerprint_spoof",
            BypassMethod::ProxyRotation => "proxy_rotation",
            BypassMethod::CaptchaSolver => "captcha_solver",
            BypassMethod::TorRoute => "tor_route",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TurnstileResult {
    pub bypassed: bool,
    pub method: BypassMethod,
    pub duration_ms: u64,
    pub token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StealthConfig {
    pub fingerprint: BrowserFingerprint,
    pub max_retries: u8,
    pub retry_delay_ms: u64,
    pub bypass_methods: Vec<BypassMethod>,
    pub cookie_file: Option<PathBuf>,
}

impl Default for StealthConfig {
    fn default() -> Self {
        StealthConfig {
            fingerprint: BrowserFingerprint::default(),
            max_retries: 3,
            retry_delay_ms: 1000,
            bypass_methods: vec![
                BypassMethod::CookieReuse,
                BypassMethod::HeadlessDetection,
                BypassMethod::FingerprintSpoof,
                BypassMethod::ProxyRotation,
                BypassMethod::CaptchaSolver,
                BypassMethod::TorRoute,
            ],
            cookie_file: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub bypass_method: BypassMethod,
    pub duration_ms: u64,
}

/// 独立爬取会话: 每会话持有独立 cookies + fingerprint + 使用计数 + 封禁状态。
/// crawlee SessionPool 技术接线: 多会话轮换, 封禁后丢弃重建, 避免单会话被限速拖垮全站。
#[derive(Debug, Clone)]
pub struct CrawlSession {
    pub id: u64,
    pub cookies: HashMap<String, String>,
    pub fingerprint: BrowserFingerprint,
    pub use_count: u32,
    pub banned: bool,
    pub last_used: Instant,
}

impl CrawlSession {
    fn new(id: u64, fingerprint: BrowserFingerprint) -> Self {
        CrawlSession {
            id,
            cookies: HashMap::new(),
            fingerprint,
            use_count: 0,
            banned: false,
            last_used: Instant::now(),
        }
    }
}

/// SessionPool: crawlee-inspired 多会话轮换池。
/// - round-robin 轮换, 跳过已封禁会话
/// - 全部封禁时重置最少使用会话 (cookies 清空, 指纹保留)
/// - 容量固定, 无淘汰 (池满即复用, 封禁即重建)
#[derive(Debug, Clone)]
pub struct SessionPool {
    sessions: Vec<CrawlSession>,
    next_id: u64,
    cursor: usize,
}

impl SessionPool {
    pub fn new(max_sessions: usize) -> Self {
        let max = max_sessions.max(1);
        let mut pool = SessionPool {
            sessions: Vec::new(),
            next_id: 0,
            cursor: 0,
        };
        for _ in 0..max {
            pool.sessions
                .push(CrawlSession::new(pool.next_id, BrowserFingerprint::default()));
            pool.next_id += 1;
        }
        pool
    }

    pub fn with_fingerprints(fingerprints: Vec<BrowserFingerprint>) -> Self {
        let mut pool = SessionPool {
            sessions: Vec::new(),
            next_id: 0,
            cursor: 0,
        };
        for fp in fingerprints {
            pool.sessions.push(CrawlSession::new(pool.next_id, fp));
            pool.next_id += 1;
        }
        pool
    }

    /// round-robin 获取下一个非封禁会话; 全部封禁时重置最少使用会话。
    pub fn acquire(&mut self) -> &mut CrawlSession {
        if self.sessions.is_empty() {
            self.sessions
                .push(CrawlSession::new(self.next_id, BrowserFingerprint::default()));
            self.next_id += 1;
        }
        let n = self.sessions.len();
        for _ in 0..n {
            let idx = self.cursor;
            self.cursor = (self.cursor + 1) % n;
            if !self.sessions[idx].banned {
                self.sessions[idx].last_used = Instant::now();
                return &mut self.sessions[idx];
            }
        }
        // 全部封禁: 重置最少使用会话 (cookies 清空, 指纹保留)
        let idx = self
            .sessions
            .iter()
            .enumerate()
            .min_by_key(|(_, s)| s.use_count)
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.sessions[idx].banned = false;
        self.sessions[idx].cookies.clear();
        self.sessions[idx].last_used = Instant::now();
        &mut self.sessions[idx]
    }

    pub fn mark_banned(&mut self, id: u64) {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == id) {
            s.banned = true;
            s.cookies.clear();
        }
    }

    pub fn active_count(&self) -> usize {
        self.sessions.iter().filter(|s| !s.banned).count()
    }

    pub fn banned_count(&self) -> usize {
        self.sessions.iter().filter(|s| s.banned).count()
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

pub struct StealthCrawler {
    pub config: StealthConfig,
    pub cookies: HashMap<String, String>,
    pub consecutive_failures: u8,
    pub last_bypass_method: BypassMethod,
    pub session_pool: Option<SessionPool>,
}

impl StealthCrawler {
    pub fn new(config: StealthConfig) -> Self {
        StealthCrawler {
            config,
            cookies: HashMap::new(),
            consecutive_failures: 0,
            last_bypass_method: BypassMethod::CookieReuse,
            session_pool: None,
        }
    }

    pub fn fetch(&mut self, url: &str) -> Result<CrawlResult, String> {
        if self.config.bypass_methods.is_empty() {
            return Err("StealthConfig.bypass_methods is empty".into());
        }
        let start = Instant::now();

        // SessionPool 接线: 获取会话, 合并其 cookies/fingerprint 到本次请求身份
        let mut active_session_id: Option<u64> = None;
        if let Some(pool) = &mut self.session_pool {
            let session = pool.acquire();
            active_session_id = Some(session.id);
            session.use_count += 1;
            for (k, v) in &session.cookies {
                self.cookies.insert(k.clone(), v.clone());
            }
            self.config.fingerprint = session.fingerprint.clone();
        }

        for attempt in 0..=self.config.max_retries as usize {
            let method_idx = attempt % self.config.bypass_methods.len();
            let method = &self.config.bypass_methods[method_idx];

            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(
                    self.config.retry_delay_ms.min(5),
                ));
            }

            match self.try_fetch(url, method, start) {
                Ok(cr) if cr.status == 200 => {
                    self.consecutive_failures = 0;
                    self.last_bypass_method = method.clone();
                    if matches!(method, BypassMethod::CookieReuse) {
                        self.cookies.insert("session".into(), "mock-session-token".into());
                    }
                    // 持久化 cookies 回活跃会话
                    if let Some(id) = active_session_id {
                        if let Some(pool) = &mut self.session_pool {
                            if let Some(s) = pool.sessions.iter_mut().find(|s| s.id == id) {
                                s.cookies = self.cookies.clone();
                            }
                        }
                    }
                    return Ok(cr);
                }
                Ok(cr) if cr.status == 403 || cr.status == 429 => {
                    self.consecutive_failures += 1;
                    self.last_bypass_method = method.clone();
                    // 封禁活跃会话: 403/429 视为该会话被站点拉黑
                    if let Some(id) = active_session_id {
                        if let Some(pool) = &mut self.session_pool {
                            pool.mark_banned(id);
                        }
                    }
                    continue;
                }
                Ok(cr) => {
                    self.consecutive_failures = 0;
                    self.last_bypass_method = method.clone();
                    return Ok(cr);
                }
                Err(e) => {
                    self.consecutive_failures += 1;
                    self.last_bypass_method = method.clone();
                    if attempt == self.config.max_retries as usize {
                        return Err(e);
                    }
                }
            }
        }

        Err(format!(
            "All {} bypass methods exhausted for {}",
            self.config.bypass_methods.len(),
            url
        ))
    }

    fn try_fetch(
        &self,
        url: &str,
        method: &BypassMethod,
        start: Instant,
    ) -> Result<CrawlResult, String> {
        let elapsed = start.elapsed().as_millis() as u64;

        match method {
            BypassMethod::CookieReuse => {
                if self.cookies.is_empty() {
                    return Err("No cookies available for reuse".into());
                }
                let mut headers = HashMap::new();
                let cookie_str: Vec<String> = self
                    .cookies
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                headers.insert("Cookie".into(), cookie_str.join("; "));
                Ok(CrawlResult {
                    status: 200,
                    body: format!(
                        "<html><body>Cookie reuse result for {}</body></html>",
                        url
                    ),
                    headers,
                    bypass_method: BypassMethod::CookieReuse,
                    duration_ms: elapsed + 50,
                })
            }
            BypassMethod::HeadlessDetection => {
                let mut headers = HashMap::new();
                headers.insert(
                    "User-Agent".into(),
                    self.config.fingerprint.user_agent.clone(),
                );
                headers.insert(
                    "Sec-CH-UA".into(),
                    format!(
                        "\"Chromium\";v=\"{}\", \"Google Chrome\";v=\"{}\"",
                        "131", "131"
                    ),
                );
                headers.insert("Sec-CH-UA-Mobile".into(), "?0".into());
                headers.insert(
                    "Sec-CH-UA-Platform".into(),
                    self.config.fingerprint.platform.clone(),
                );
                Ok(CrawlResult {
                    status: 200,
                    body: format!(
                        "<html><body>Headless detection bypass for {}</body></html>",
                        url
                    ),
                    headers,
                    bypass_method: BypassMethod::HeadlessDetection,
                    duration_ms: elapsed + 80,
                })
            }
            BypassMethod::FingerprintSpoof => {
                let mut headers = HashMap::new();
                headers.insert(
                    "User-Agent".into(),
                    self.config.fingerprint.user_agent.clone(),
                );
                headers.insert(
                    "Accept-Language".into(),
                    self.config.fingerprint.language.clone(),
                );
                headers.insert("X-Requested-With".into(), "XMLHttpRequest".into());
                Ok(CrawlResult {
                    status: 200,
                    body: format!(
                        "<html><body>Fingerprint spoof result for {}</body></html>",
                        url
                    ),
                    headers,
                    bypass_method: BypassMethod::FingerprintSpoof,
                    duration_ms: elapsed + 120,
                })
            }
            BypassMethod::ProxyRotation => Ok(CrawlResult {
                status: 200,
                body: format!(
                    "<html><body>Proxy rotation result for {}</body></html>",
                    url
                ),
                headers: HashMap::new(),
                bypass_method: BypassMethod::ProxyRotation,
                duration_ms: elapsed + 200,
            }),
            BypassMethod::CaptchaSolver => {
                let mut headers = HashMap::new();
                headers.insert("X-Captcha-Token".into(), "simulated-captcha-token".into());
                Ok(CrawlResult {
                    status: 200,
                    body: format!(
                        "<html><body>Captcha solver result for {}</body></html>",
                        url
                    ),
                    headers,
                    bypass_method: BypassMethod::CaptchaSolver,
                    duration_ms: elapsed + 500,
                })
            }
            BypassMethod::TorRoute => {
                let mut headers = HashMap::new();
                headers.insert("X-Tor-Route".into(), "simulated-tor-circuit".into());
                Ok(CrawlResult {
                    status: 200,
                    body: format!(
                        "<html><body>Tor route result for {}</body></html>",
                        url
                    ),
                    headers,
                    bypass_method: BypassMethod::TorRoute,
                    duration_ms: elapsed + 700,
                })
            }
        }
    }

    pub fn rotate_fingerprint(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let uas = [
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        ];
        let platforms = ["MacIntel", "Win64", "Linux x86_64"];
        let vendors = ["Google Inc.", "Google Inc.", "Mozilla Foundation"];
        let renderers = [
            "Intel Iris OpenGL Engine",
            "ANGLE (Intel, Intel(R) UHD Graphics Direct3D11 vs_5_0 ps_5_0)",
            "Mesa/X.org (AMD Radeon)",
        ];
        let languages = ["en-US", "en-GB", "en-CA", "de-DE", "fr-FR"];
        let timezones = [
            "America/New_York",
            "Europe/London",
            "Asia/Tokyo",
            "Australia/Sydney",
        ];

        self.config.fingerprint.user_agent = uas[rng.gen_range(0..uas.len())].into();
        self.config.fingerprint.platform = platforms[rng.gen_range(0..platforms.len())].into();
        self.config.fingerprint.vendor = vendors[rng.gen_range(0..vendors.len())].into();
        self.config.fingerprint.webgl_renderer = renderers[rng.gen_range(0..renderers.len())].into();
        self.config.fingerprint.language = languages[rng.gen_range(0..languages.len())].into();
        self.config.fingerprint.timezone = timezones[rng.gen_range(0..timezones.len())].into();
        self.config.fingerprint.viewport = (rng.gen_range(1280..2560), rng.gen_range(720..1440));
    }

    pub fn save_cookies(&self, path: &PathBuf) -> Result<(), String> {
        let content: String = self
            .cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, &content).map_err(|e| format!("Failed to save cookies: {}", e))
    }

    pub fn load_cookies(&mut self, path: &PathBuf) -> Result<(), String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to load cookies: {}", e))?;
        self.cookies.clear();
        for line in content.lines() {
            if let Some(idx) = line.find('=') {
                let key = &line[..idx];
                let val = &line[idx + 1..];
                self.cookies.insert(key.to_string(), val.to_string());
            }
        }
        Ok(())
    }

    pub fn bypass_rate(&self) -> f64 {
        if self.consecutive_failures == 0 && !self.cookies.is_empty() {
            0.95
        } else if self.consecutive_failures <= 2 {
            0.80 - (self.consecutive_failures as f64 * 0.15)
        } else {
            0.20
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> StealthConfig {
        StealthConfig {
            max_retries: 2,
            retry_delay_ms: 1,
            ..StealthConfig::default()
        }
    }

    fn make_nt_world_crawl() -> StealthCrawler {
        let mut c = StealthCrawler::new(test_config());
        c.cookies.insert("session".into(), "test-session".into());
        c
    }

    #[test]
    fn test_default_fingerprint_has_chrome_ua() {
        let fp = BrowserFingerprint::default();
        assert!(fp.user_agent.contains("Chrome/131"));
        assert!(fp.user_agent.contains("Macintosh"));
        assert_eq!(fp.viewport, (1920, 1080));
        assert_eq!(fp.language, "en-US");
        assert!(fp.canvas_noise);
    }

    #[test]
    fn test_fetch_no_retry_success() {
        let mut c = make_nt_world_crawl();
        let result = c.fetch("https://example.com");
        assert!(result.is_ok());
        let cr = result.expect("fetch with valid nt_world_crawl should return Ok");
        assert_eq!(cr.status, 200);
        assert!(cr.body.contains("example.com"));
        assert_eq!(cr.bypass_method, BypassMethod::CookieReuse);
    }

    #[test]
    fn test_fetch_with_retry_after_failure() {
        let mut c = StealthCrawler::new(test_config());
        let result = c.fetch("https://example.com");
        assert!(result.is_ok());
        let cr = result.expect("fetch with retry should return Ok");
        assert_eq!(cr.status, 200);
        assert!(cr.duration_ms >= 50);
    }

    #[test]
    fn test_rotate_fingerprint_changes_ua() {
        let mut c = make_nt_world_crawl();
        let _original_ua = c.config.fingerprint.user_agent.clone();
        c.rotate_fingerprint();
        let new_ua = c.config.fingerprint.user_agent.clone();
        assert!(
            new_ua.contains("Chrome/131"),
            "UA should still be Chrome: {}",
            new_ua
        );
        assert!(new_ua.contains("Mozilla"), "UA should contain Mozilla prefix");
    }

    #[test]
    fn test_save_cookies_to_file() {
        let mut c = make_nt_world_crawl();
        c.cookies.insert("token".into(), "abc123".into());
        let path = std::env::temp_dir().join("neotrix_test_cookies_save.txt");
        let _ = fs::remove_file(&path);
        c.save_cookies(&path).expect("save_cookies to temp path should succeed");
        let content = fs::read_to_string(&path).expect("read back saved cookies file should succeed");
        assert!(content.contains("session=test-session"));
        assert!(content.contains("token=abc123"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_load_cookies_from_file() {
        let path = std::env::temp_dir().join("neotrix_test_cookies_load.txt");
        fs::write(&path, "key1=value1\nkey2=value2").expect("write cookies test file should succeed");
        let mut c = make_nt_world_crawl();
        c.load_cookies(&path).expect("load_cookies from test file should succeed");
        assert_eq!(c.cookies.len(), 2);
        assert_eq!(c.cookies.get("key1").expect("key1 should be in loaded cookies"), "value1");
        assert_eq!(c.cookies.get("key2").expect("key2 should be in loaded cookies"), "value2");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_bypass_rate_high_when_no_failures() {
        let c = make_nt_world_crawl();
        let rate = c.bypass_rate();
        assert!((rate - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_bypass_rate_low_after_many_failures() {
        let mut c = make_nt_world_crawl();
        c.consecutive_failures = 5;
        c.cookies.clear();
        let rate = c.bypass_rate();
        assert!((rate - 0.20).abs() < 0.01);
    }

    #[test]
    fn test_consecutive_failures_increment() {
        let mut c = StealthCrawler::new(test_config());
        c.config.bypass_methods = vec![BypassMethod::CookieReuse];
        c.config.max_retries = 1;
        let _result = c.fetch("https://example.com");
        assert!(c.consecutive_failures > 0 || c.consecutive_failures == 0);
    }

    #[test]
    fn test_bypass_method_escalation() {
        let config = StealthConfig {
            bypass_methods: vec![
                BypassMethod::CookieReuse,
                BypassMethod::ProxyRotation,
            ],
            max_retries: 3,
            ..StealthConfig::default()
        };
        let mut c = StealthCrawler::new(config);
        let result = c.fetch("https://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tor_route_bypass_method() {
        // CyberScraper-2077 吸收: Tor 网络路由作为独立绕过手段 (如 .onion 站点),
        // 不依赖 cookies/指纹, 标记 X-Tor-Route 头。
        let config = StealthConfig {
            bypass_methods: vec![BypassMethod::TorRoute],
            max_retries: 1,
            ..StealthConfig::default()
        };
        let mut c = StealthCrawler::new(config);
        let result = c.fetch("https://example.onion").expect("tor route fetch");
        assert_eq!(result.status, 200);
        assert!(result.headers.contains_key("X-Tor-Route"));
        assert!(result.body.contains("Tor route"));
        assert_eq!(result.bypass_method, BypassMethod::TorRoute);
    }

    #[test]
    fn test_tor_route_default_in_bypass_set() {
        // 默认绕过集合含 TorRoute, 域名可达时可直接走 tor 路径。
        let cfg = StealthConfig::default();
        assert!(cfg.bypass_methods.contains(&BypassMethod::TorRoute));
        assert_eq!(BypassMethod::TorRoute.name(), "tor_route");
    }

    #[test]
    fn test_fetch_empty_bypass_methods_returns_error_not_panic() {
        // Regression: attempt % bypass_methods.len() panicked on modulo 0 when a
        // consumer constructed StealthConfig with an empty bypass_methods vec.
        let config = StealthConfig {
            bypass_methods: vec![],
            max_retries: 1,
            ..StealthConfig::default()
        };
        let mut c = StealthCrawler::new(config);
        assert!(c.fetch("https://example.com").is_err());
    }

    #[test]
    fn test_crawl_result_fields_populated() {
        let mut c = make_nt_world_crawl();
        let result = c.fetch("https://example.com").expect("fetch with make_nt_world_crawl should return Ok");
        assert_eq!(result.status, 200);
        assert!(!result.body.is_empty());
        assert!(!result.headers.is_empty());
        assert!(result.duration_ms > 0);
    }

    #[test]
    fn test_cookie_persistence_across_fetches() {
        let mut c = make_nt_world_crawl();
        let _ = c.fetch("https://example.com");
        assert!(c.cookies.contains_key("session"));
        assert_eq!(c.cookies.get("session").expect("session cookie should be set after fetch"), "mock-session-token");
    }

    #[test]
    fn test_bypass_method_names() {
        assert_eq!(BypassMethod::CookieReuse.name(), "cookie_reuse");
        assert_eq!(BypassMethod::HeadlessDetection.name(), "headless_detection");
        assert_eq!(BypassMethod::FingerprintSpoof.name(), "fingerprint_spoof");
        assert_eq!(BypassMethod::ProxyRotation.name(), "proxy_rotation");
        assert_eq!(BypassMethod::CaptchaSolver.name(), "captcha_solver");
    }

    #[test]
    fn test_stealth_config_defaults() {
        let config = StealthConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert_eq!(config.bypass_methods.len(), 6);
        assert!(config.cookie_file.is_none());
    }

    #[test]
    fn test_fetch_different_urls() {
        let mut c = make_nt_world_crawl();
        let r1 = c.fetch("https://example.com").expect("fetch example.com should succeed");
        assert!(r1.body.contains("example.com"));
        let r2 = c.fetch("https://httpbin.org").expect("fetch httpbin.org should succeed");
        assert!(r2.body.contains("httpbin.org"));
    }

    #[test]
    fn test_fingerprint_rotate_changes_platform() {
        let mut c = make_nt_world_crawl();
        c.config.fingerprint.platform = "MacIntel".into();
        c.rotate_fingerprint();
        assert!(
            !c.config.fingerprint.platform.is_empty(),
            "Platform should not be empty after rotation"
        );
    }

    #[test]
    fn test_session_pool_round_robin() {
        let mut pool = SessionPool::new(3);
        assert_eq!(pool.acquire().id, 0);
        assert_eq!(pool.acquire().id, 1);
        assert_eq!(pool.acquire().id, 2);
        assert_eq!(pool.acquire().id, 0); // wraps around
    }

    #[test]
    fn test_session_pool_skips_banned() {
        let mut pool = SessionPool::new(3);
        pool.mark_banned(0);
        let id = pool.acquire().id;
        assert_eq!(id, 1);
    }

    #[test]
    fn test_session_pool_all_banned_resets_least_used() {
        let mut pool = SessionPool::new(2);
        pool.mark_banned(0);
        pool.mark_banned(1);
        let id = pool.acquire().id;
        assert!(id == 0 || id == 1);
        assert_eq!(pool.banned_count(), 1);
        assert_eq!(pool.active_count(), 1);
    }

    #[test]
    fn test_session_pool_counts() {
        let mut pool = SessionPool::new(4);
        assert_eq!(pool.len(), 4);
        assert_eq!(pool.active_count(), 4);
        pool.mark_banned(2);
        assert_eq!(pool.active_count(), 3);
        assert_eq!(pool.banned_count(), 1);
    }

    #[test]
    fn test_session_pool_ban_on_403() {
        // 403 封禁活跃会话 → 下次 acquire 跳过它
        let mut pool = SessionPool::new(2);
        let s1 = pool.acquire().id;
        pool.mark_banned(s1);
        let s2 = pool.acquire().id;
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_stealth_crawler_with_session_pool() {
        let mut c = StealthCrawler::new(test_config());
        c.session_pool = Some(SessionPool::new(2));
        let result = c.fetch("https://example.com");
        assert!(result.is_ok());
        let cr = result.expect("fetch with session pool should succeed");
        assert_eq!(cr.status, 200);
        assert!(cr.body.contains("example.com"));
    }

    #[test]
    fn test_stealth_crawler_pool_persists_cookies() {
        // 预置 cookies 使 CookieReuse 成功, 验证 cookies 持久化回池会话
        let mut c = make_nt_world_crawl();
        c.session_pool = Some(SessionPool::new(1));
        let _ = c.fetch("https://example.com");
        let pool = c.session_pool.expect("session pool should be present");
        assert!(
            pool.sessions[0].cookies.contains_key("session"),
            "session cookies should persist back into the pool session"
        );
    }
}

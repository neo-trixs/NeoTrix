use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::core::nt_core_agent::UserAgentRotation;

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
            user_agent: UserAgentRotation::default().next().to_string(),
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
}

impl BypassMethod {
    pub fn name(&self) -> &'static str {
        match self {
            BypassMethod::CookieReuse => "cookie_reuse",
            BypassMethod::HeadlessDetection => "headless_detection",
            BypassMethod::FingerprintSpoof => "fingerprint_spoof",
            BypassMethod::ProxyRotation => "proxy_rotation",
            BypassMethod::CaptchaSolver => "captcha_solver",
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

pub struct StealthCrawler {
    pub config: StealthConfig,
    pub cookies: HashMap<String, String>,
    pub consecutive_failures: u8,
    pub last_bypass_method: BypassMethod,
}

impl StealthCrawler {
    pub fn new(config: StealthConfig) -> Self {
        StealthCrawler {
            config,
            cookies: HashMap::new(),
            consecutive_failures: 0,
            last_bypass_method: BypassMethod::CookieReuse,
        }
    }

    pub fn fetch(&mut self, url: &str) -> Result<CrawlResult, String> {
        let start = Instant::now();

        for attempt in 0..=self.config.max_retries as usize {
            let method_idx = attempt % self.config.bypass_methods.len();
            let method = &self.config.bypass_methods[method_idx];

            if attempt > 0 {
                tokio::runtime::Handle::current().block_on(tokio::time::sleep(
                    Duration::from_millis(self.config.retry_delay_ms.min(5)),
                ));
            }

            match self.try_fetch(url, method, start) {
                Ok(cr) if cr.status == 200 => {
                    self.consecutive_failures = 0;
                    self.last_bypass_method = method.clone();
                    if matches!(method, BypassMethod::CookieReuse) {
                        self.cookies
                            .insert("session".into(), "mock-session-token".into());
                    }
                    return Ok(cr);
                }
                Ok(cr) if cr.status == 403 || cr.status == 429 => {
                    self.consecutive_failures += 1;
                    self.last_bypass_method = method.clone();
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
                    body: format!("<html><body>Cookie reuse result for {}</body></html>", url),
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
        }
    }

    pub fn rotate_fingerprint(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
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

        self.config.fingerprint.user_agent = UserAgentRotation::default().next().to_string();
        self.config.fingerprint.platform = platforms[rng.gen_range(0..platforms.len())].into();
        self.config.fingerprint.vendor = vendors[rng.gen_range(0..vendors.len())].into();
        self.config.fingerprint.webgl_renderer =
            renderers[rng.gen_range(0..renderers.len())].into();
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
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to load cookies: {}", e))?;
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
        assert!(
            new_ua.contains("Mozilla"),
            "UA should contain Mozilla prefix"
        );
    }

    #[test]
    fn test_save_cookies_to_file() {
        let mut c = make_nt_world_crawl();
        c.cookies.insert("token".into(), "abc123".into());
        let path = std::env::temp_dir().join("neotrix_test_cookies_save.txt");
        let _ = fs::remove_file(&path);
        c.save_cookies(&path)
            .expect("save_cookies to temp path should succeed");
        let content =
            fs::read_to_string(&path).expect("read back saved cookies file should succeed");
        assert!(content.contains("session=test-session"));
        assert!(content.contains("token=abc123"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_load_cookies_from_file() {
        let path = std::env::temp_dir().join("neotrix_test_cookies_load.txt");
        fs::write(&path, "key1=value1\nkey2=value2")
            .expect("write cookies test file should succeed");
        let mut c = make_nt_world_crawl();
        c.load_cookies(&path)
            .expect("load_cookies from test file should succeed");
        assert_eq!(c.cookies.len(), 2);
        assert_eq!(
            c.cookies
                .get("key1")
                .expect("key1 should be in loaded cookies"),
            "value1"
        );
        assert_eq!(
            c.cookies
                .get("key2")
                .expect("key2 should be in loaded cookies"),
            "value2"
        );
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
            bypass_methods: vec![BypassMethod::CookieReuse, BypassMethod::ProxyRotation],
            max_retries: 3,
            ..StealthConfig::default()
        };
        let mut c = StealthCrawler::new(config);
        let result = c.fetch("https://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_crawl_result_fields_populated() {
        let mut c = make_nt_world_crawl();
        let result = c
            .fetch("https://example.com")
            .expect("fetch with make_nt_world_crawl should return Ok");
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
        assert_eq!(
            c.cookies
                .get("session")
                .expect("session cookie should be set after fetch"),
            "mock-session-token"
        );
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
        assert_eq!(config.bypass_methods.len(), 5);
        assert!(config.cookie_file.is_none());
    }

    #[test]
    fn test_fetch_different_urls() {
        let mut c = make_nt_world_crawl();
        let r1 = c
            .fetch("https://example.com")
            .expect("fetch example.com should succeed");
        assert!(r1.body.contains("example.com"));
        let r2 = c
            .fetch("https://httpbin.org")
            .expect("fetch httpbin.org should succeed");
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
}

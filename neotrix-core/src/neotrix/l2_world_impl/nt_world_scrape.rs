use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperConfig {
    pub proxy: Option<String>,
    pub headless: bool,
    pub block_images: bool,
    pub user_agent: Option<String>,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub profile_name: Option<String>,
    pub use_tiny_profile: bool,
    /// 隐身抓取参数集 (G22, CyberScraper-2077 吸收): 请求间隔 / referer /
    /// header 池轮换 / 代理轮换 — 降低指纹一致性被反爬捕获的风险。
    #[serde(default)]
    pub stealth_params: StealthParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthParams {
    /// 相邻请求最小间隔 (ms) — 避免无人类节奏的固定频率。
    pub min_interval_ms: u64,
    /// 间隔抖动幅度 (±ms) — 请求节奏随机化。
    pub interval_jitter_ms: u64,
    /// referer 池 (轮换使用)。
    pub referers: Vec<String>,
    /// header 变体池 (轮换使用, 每次请求选一个)。
    pub header_variants: Vec<HashMap<String, String>>,
    /// 代理轮换 (每 N 请求换代理)。
    pub rotate_proxy_every: u32,
    /// 启用隐身参数 (关掉则退化为普通请求节奏)。
    pub enabled: bool,
}

impl Default for StealthParams {
    fn default() -> Self {
        Self {
            min_interval_ms: 1200,
            interval_jitter_ms: 600,
            referers: vec![
                "https://www.google.com/".into(),
                "https://www.bing.com/".into(),
                "https://duckduckgo.com/".into(),
            ],
            header_variants: vec![
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            rotate_proxy_every: 10,
            enabled: true,
        }
    }
}

impl StealthParams {
    /// 按请求序号取本轮 referer (轮换)。
    pub fn referer_for(&self, request_seq: u32) -> Option<String> {
        if self.referers.is_empty() {
            return None;
        }
        Some(self.referers[(request_seq as usize) % self.referers.len()].clone())
    }

    /// 按请求序号取本轮 header 变体 (轮换; 全部为空则返回 None 表示无额外 header)。
    pub fn headers_for(&self, request_seq: u32) -> Option<HashMap<String, String>> {
        if self.header_variants.is_empty() || self.header_variants.iter().all(|v| v.is_empty()) {
            return None;
        }
        Some(self.header_variants[(request_seq as usize) % self.header_variants.len()].clone())
    }

    /// 当前请求应等待的间隔 (含抖动)。
    pub fn interval_for(&self) -> std::time::Duration {
        if !self.enabled {
            return std::time::Duration::ZERO;
        }
        if self.interval_jitter_ms == 0 {
            return std::time::Duration::from_millis(self.min_interval_ms);
        }
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let jitter = (nanos % (self.interval_jitter_ms as u128 + 1)) as u64;
        std::time::Duration::from_millis(self.min_interval_ms + jitter)
    }

    /// 该请求序号是否应轮换代理 (命中 rotate 边界)。
    pub fn should_rotate_proxy(&self, request_seq: u32) -> bool {
        self.enabled && self.rotate_proxy_every > 0 && request_seq > 0 && request_seq % self.rotate_proxy_every == 0
    }
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            headless: true,
            block_images: true,
            user_agent: None,
            timeout_secs: 30,
            max_retries: 3,
            profile_name: None,
            use_tiny_profile: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeResult {
    pub url: String,
    pub status_code: u16,
    pub html: Option<String>,
    pub text: Option<String>,
    pub headers: HashMap<String, String>,
    pub error: Option<String>,
}

pub struct BrowserScraper {
    config: ScraperConfig,
}

impl BrowserScraper {
    pub fn new(config: ScraperConfig) -> Self {
        Self { config }
    }

    fn do_fetch(&self, url: &str, referer: Option<&str>) -> ScrapeResult {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Ok(h) = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
            .parse::<reqwest::header::HeaderValue>() {
            headers.insert(reqwest::header::ACCEPT, h);
        }
        if let Ok(h) = "en-US,en;q=0.9".parse::<reqwest::header::HeaderValue>() {
            headers.insert(reqwest::header::ACCEPT_LANGUAGE, h);
        }
        if let Some(r) = referer {
            if let Ok(header_val) = r.parse::<reqwest::header::HeaderValue>() {
                headers.insert(reqwest::header::REFERER, header_val);
            }
        }
        let ua = self
            .config
            .user_agent
            .clone()
            .unwrap_or_else(|| AntiDetect::new_with_defaults().random_ua().to_string());
        if let Ok(ua_header) = ua.parse::<reqwest::header::HeaderValue>() {
            headers.insert(reqwest::header::USER_AGENT, ua_header);
        }

        let mut builder = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .default_headers(headers)
            .danger_accept_invalid_certs(true);

        if let Some(ref proxy_url) = self.config.proxy {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                builder = builder.proxy(proxy);
            }
        }

        let client = match builder.build() {
            Ok(c) => c,
            Err(e) => {
                return ScrapeResult {
                    url: url.to_string(),
                    status_code: 0,
                    html: None,
                    text: None,
                    headers: HashMap::new(),
                    error: Some(format!("failed to build client: {e}")),
                }
            }
        };

        match client.get(url).send() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let resp_headers: HashMap<String, String> = resp
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                let text = resp.text().ok();
                let html = text.clone();
                ScrapeResult {
                    url: url.to_string(),
                    status_code: status,
                    html,
                    text,
                    headers: resp_headers,
                    error: None,
                }
            }
            Err(e) => ScrapeResult {
                url: url.to_string(),
                status_code: 0,
                html: None,
                text: None,
                headers: HashMap::new(),
                error: Some(e.to_string()),
            },
        }
    }

    pub fn human_get(&self, url: &str) -> ScrapeResult {
        self.do_fetch(url, Some("https://www.google.com/"))
    }

    pub fn cf_get(&self, url: &str) -> ScrapeResult {
        self.do_fetch(url, None)
    }
}

pub struct RequestScraper {
    config: ScraperConfig,
}

impl RequestScraper {
    pub fn new(config: ScraperConfig) -> Self {
        Self { config }
    }

    fn build_client(&self) -> reqwest::blocking::Client {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Ok(h) = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
            .parse::<reqwest::header::HeaderValue>() {
            headers.insert(reqwest::header::ACCEPT, h);
        }
        if let Ok(h) = "en-US,en;q=0.9".parse::<reqwest::header::HeaderValue>() {
            headers.insert(reqwest::header::ACCEPT_LANGUAGE, h);
        }
        let ua = self
            .config
            .user_agent
            .clone()
            .unwrap_or_else(|| AntiDetect::new_with_defaults().random_ua().to_string());
        if let Ok(h) = ua.parse::<reqwest::header::HeaderValue>() {
            headers.insert(reqwest::header::USER_AGENT, h);
        }

        let mut builder = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .default_headers(headers)
            .danger_accept_invalid_certs(true);

        if let Some(ref proxy_url) = self.config.proxy {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                builder = builder.proxy(proxy);
            }
        }

        match builder.build() {
            Ok(client) => client,
            Err(e) => {
                log::warn!("[scrape] client build failed: {}. Using default.", e);
                reqwest::blocking::Client::new()
            }
        }
    }

    fn fetch(&self, url: &str, referer: Option<&str>) -> ScrapeResult {
        let client = self.build_client();
        let mut req = client.get(url);
        if let Some(r) = referer {
            req = req.header(reqwest::header::REFERER, r);
        }
        match req.send() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let headers: HashMap<String, String> = resp
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                let text = match resp.text() {
                    Ok(t) => Some(t),
                    Err(e) => {
                        log::warn!("[nt_world_scrape] read body: {}", e);
                        None
                    }
                };
                let html = text.clone();
                ScrapeResult {
                    url: url.to_string(),
                    status_code: status,
                    html,
                    text,
                    headers,
                    error: None,
                }
            }
            Err(e) => ScrapeResult {
                url: url.to_string(),
                status_code: 0,
                html: None,
                text: None,
                headers: HashMap::new(),
                error: Some(e.to_string()),
            },
        }
    }

    pub fn get(&self, url: &str) -> ScrapeResult {
        self.fetch(url, None)
    }

    /// SessionPool 身份注入抓取: 用指定 UA + Cookie 请求 (crawlee SessionPool 技术接线)。
    /// 允许 caller 轮换会话身份, 单会话被站点封禁时不影响其它会话。
    pub fn get_with_identity(&self, url: &str, user_agent: &str, cookie_str: &str) -> ScrapeResult {
        let client = self.build_client_with_identity(user_agent, cookie_str);
        match client.get(url).send() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let headers: HashMap<String, String> = resp
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                let text = match resp.text() {
                    Ok(t) => Some(t),
                    Err(e) => {
                        log::warn!("[nt_world_scrape] read body: {}", e);
                        None
                    }
                };
                let html = text.clone();
                ScrapeResult {
                    url: url.to_string(),
                    status_code: status,
                    html,
                    text,
                    headers,
                    error: None,
                }
            }
            Err(e) => ScrapeResult {
                url: url.to_string(),
                status_code: 0,
                html: None,
                text: None,
                headers: HashMap::new(),
                error: Some(e.to_string()),
            },
        }
    }

    fn build_client_with_identity(&self, user_agent: &str, cookie_str: &str) -> reqwest::blocking::Client {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Ok(h) = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
            .parse::<reqwest::header::HeaderValue>() {
            headers.insert(reqwest::header::ACCEPT, h);
        }
        if let Ok(h) = "en-US,en;q=0.9".parse::<reqwest::header::HeaderValue>() {
            headers.insert(reqwest::header::ACCEPT_LANGUAGE, h);
        }
        if let Ok(h) = user_agent.parse::<reqwest::header::HeaderValue>() {
            headers.insert(reqwest::header::USER_AGENT, h);
        }
        if !cookie_str.is_empty() {
            if let Ok(h) = cookie_str.parse::<reqwest::header::HeaderValue>() {
                headers.insert(reqwest::header::COOKIE, h);
            }
        }

        let mut builder = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .default_headers(headers)
            .danger_accept_invalid_certs(true);

        if let Some(ref proxy_url) = self.config.proxy {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                builder = builder.proxy(proxy);
            }
        }

        match builder.build() {
            Ok(client) => client,
            Err(e) => {
                log::warn!("[scrape] client build failed: {}. Using default.", e);
                reqwest::blocking::Client::new()
            }
        }
    }

    pub fn google_get(&self, url: &str) -> ScrapeResult {
        self.fetch(url, Some("https://www.google.com/"))
    }
}

pub struct AntiDetect {
    pub user_agents: Vec<&'static str>,
}

impl AntiDetect {
    pub fn random_ua(&self) -> &str {
        let n = self.user_agents.len();
        if n == 0 {
            return "Mozilla/5.0 (compatible; NeoTrix/1.0)";
        }
        let i = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            % n as u128) as usize;
        self.user_agents[i]
    }

    pub fn tiny_profile_name(base: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        base.hash(&mut hasher);
        let short = hasher.finish();
        format!("tiny_{:x}", short)
    }

    pub fn new_with_defaults() -> Self {
        Self {
            user_agents: vec![
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:127.0) Gecko/20100101 Firefox/127.0",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nt_world_scrape_config_default() {
        let cfg = ScraperConfig::default();
        assert!(cfg.headless);
        assert!(cfg.block_images);
        assert_eq!(cfg.timeout_secs, 30);
        assert_eq!(cfg.max_retries, 3);
        assert!(cfg.use_tiny_profile);
        assert!(cfg.proxy.is_none());
        assert!(cfg.user_agent.is_none());
        assert!(cfg.profile_name.is_none());
    }

    #[test]
    fn test_anti_detect_random_ua() {
        #[allow(deprecated)]
        let ad = AntiDetect::new_with_defaults();
        let ua = ad.random_ua();
        assert!(ua.starts_with("Mozilla/5.0"));
        assert!(ad.user_agents.contains(&ua));
    }

    #[test]
    fn test_anti_detect_tiny_profile() {
        let name = AntiDetect::tiny_profile_name("test-profile");
        assert!(name.starts_with("tiny_"));
        assert_eq!(name.len(), 5 + 16);
        let name2 = AntiDetect::tiny_profile_name("test-profile");
        assert_eq!(name, name2);
        let name3 = AntiDetect::tiny_profile_name("other-profile");
        assert_ne!(name, name3);
    }

    #[test]
    fn test_nt_world_browse_nt_world_scrape_new() {
        let cfg = ScraperConfig::default();
        let bs = BrowserScraper::new(cfg);
        let result = bs.human_get("https://example.com");
        assert_eq!(result.url, "https://example.com");
        // Online: status 200, no error. Offline: status 0, error set.
        assert!(
            result.status_code == 200 || result.error.is_some(),
            "Expected either online success (200) or offline error, got status={}, error={:?}",
            result.status_code, result.error
        );
    }

    #[test]
    fn test_request_nt_world_scrape_new() {
        let cfg = ScraperConfig::default();
        let rs = RequestScraper::new(cfg);
        let result = rs.get("https://example.com");
        assert_eq!(result.url, "https://example.com");
    }

    #[test]
    fn test_anti_detect_default_has_five_uas() {
        #[allow(deprecated)]
        let ad = AntiDetect::new_with_defaults();
        assert_eq!(ad.user_agents.len(), 5);
    }

    #[test]
    fn test_nt_world_scrape_config_custom() {
        let cfg = ScraperConfig {
            headless: false,
            timeout_secs: 60,
            max_retries: 5,
            proxy: Some("http://localhost:8080".into()),
            ..Default::default()
        };
        assert!(!cfg.headless);
        assert_eq!(cfg.timeout_secs, 60);
        assert_eq!(cfg.max_retries, 5);
        assert_eq!(cfg.proxy.as_deref(), Some("http://localhost:8080"));
    }
}

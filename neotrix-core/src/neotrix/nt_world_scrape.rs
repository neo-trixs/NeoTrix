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

    pub fn human_get(&self, url: &str) -> ScrapeResult {
        let _ = &self.config;
        ScrapeResult {
            url: url.to_string(),
            status_code: 200,
            html: None,
            text: None,
            headers: HashMap::from([("referer".into(), "https://www.google.com/".into())]),
            error: Some("BrowserScraper: not yet connected to nt_world_browse_auto".into()),
        }
    }

    pub fn cf_get(&self, url: &str) -> ScrapeResult {
        let _ = &self.config;
        ScrapeResult {
            url: url.to_string(),
            status_code: 200,
            html: None,
            text: None,
            headers: HashMap::new(),
            error: Some("BrowserScraper: not yet connected to nt_world_browse_auto".into()),
        }
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
        headers.insert(
            reqwest::header::ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".parse().expect("valid accept header"),
        );
        headers.insert(
            reqwest::header::ACCEPT_LANGUAGE,
            "en-US,en;q=0.9".parse().expect("valid accept-language header"),
        );
        let ua = self
            .config
            .user_agent
            .clone()
            .unwrap_or_else(|| AntiDetect::new_with_defaults().random_ua().to_string());
        headers.insert(reqwest::header::USER_AGENT, ua.parse().expect("invalid user agent header"));

        let mut builder = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .default_headers(headers)
            .danger_accept_invalid_certs(true);

        if let Some(ref proxy_url) = self.config.proxy {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                builder = builder.proxy(proxy);
            }
        }

        builder.build().expect("failed to build reqwest client")
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

    pub fn google_get(&self, url: &str) -> ScrapeResult {
        self.fetch(url, Some("https://www.google.com/"))
    }
}

pub struct AntiDetect {
    pub user_agents: Vec<&'static str>,
}

impl AntiDetect {
    pub fn random_ua(&self) -> &str {
        let i = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            % self.user_agents.len() as u128) as usize;
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
        assert_eq!(result.status_code, 200);
        assert!(result.error.is_some());
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

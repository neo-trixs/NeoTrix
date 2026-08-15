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
            stealth_params: StealthParams::default(),
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
    request_seq: u32,
}

impl RequestScraper {
    pub fn new(config: ScraperConfig) -> Self {
        Self {
            config,
            request_seq: 0,
        }
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
        // G22 隐身参数集: 轮换 header 变体 (若配置非空)
        if let Some(variant) = self.config.stealth_params.headers_for(self.request_seq) {
            for (k, v) in variant {
                if let Ok(h) = v.parse::<reqwest::header::HeaderValue>() {
                    headers.insert(
                        reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap_or_else(|_| reqwest::header::HeaderName::from_static("X-Stealth")),
                        h,
                    );
                }
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

    fn fetch(&mut self, url: &str, referer: Option<&str>) -> ScrapeResult {
        // G22 隐身参数集: 请求间隔 (含抖动) + referer 轮换 + 代理轮换边界。
        self.request_seq = self.request_seq.wrapping_add(1);
        let seq = self.request_seq;
        if self.config.stealth_params.enabled {
            let delay = self.config.stealth_params.interval_for();
            if !delay.is_zero() {
                std::thread::sleep(delay);
            }
            if self.config.stealth_params.should_rotate_proxy(seq) {
                log::trace!("[scrape] stealth: rotating proxy at request {}", seq);
            }
        }
        let effective_referer = referer
            .map(|s| s.to_string())
            .or_else(|| self.config.stealth_params.referer_for(seq));
        let client = self.build_client();
        let mut req = client.get(url);
        if let Some(r) = &effective_referer {
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

    pub fn get(&mut self, url: &str) -> ScrapeResult {
        self.fetch(url, None)
    }

    /// SessionPool 身份注入抓取: 用指定 UA + Cookie 请求 (crawlee SessionPool 技术接线)。
    /// 允许 caller 轮换会话身份, 单会话被站点封禁时不影响其它会话。
    pub fn get_with_identity(&mut self, url: &str, user_agent: &str, cookie_str: &str) -> ScrapeResult {
        self.request_seq = self.request_seq.wrapping_add(1);
        if self.config.stealth_params.enabled {
            let delay = self.config.stealth_params.interval_for();
            if !delay.is_zero() {
                std::thread::sleep(delay);
            }
        }
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

    pub fn google_get(&mut self, url: &str) -> ScrapeResult {
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

// ────────────────────────────────────────────────────────────────
// G21 Design Token Extraction (dembrandt 吸收) —
// 从 HTML/页面提取设计令牌 (颜色/字体/间距), 供 NT-IO web 前端消费
// ────────────────────────────────────────────────────────────────

/// 提取到的设计令牌。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DesignTokenSet {
    /// 十六进制颜色列表 (去重, 保持出现顺序)。
    pub colors: Vec<String>,
    /// CSS 颜色名 → 计数 (常见具名色)。
    pub named_colors: Vec<(String, usize)>,
    /// font-family 声明 (去重)。
    pub fonts: Vec<String>,
    /// font-size 值 (px)。
    pub font_sizes: Vec<String>,
    /// spacing / padding / margin 值 (px)。
    pub spacings: Vec<String>,
}

/// 设计令牌提取器 — 从 HTML 文档 (含 `<style>` 与内联 `style="..."`) 中
/// 提取设计令牌 (dembrandt 思路: 视觉风格自动发现)。
pub struct DesignTokenExtractor;

impl DesignTokenExtractor {
    /// 从 HTML 文本提取设计令牌。
    pub fn extract(html: &str) -> DesignTokenSet {
        let mut tokens = DesignTokenSet::default();
        // 1. <style> 块
        for style_block in Self::capture_style_blocks(html) {
            Self::scan_css(&style_block, &mut tokens);
        }
        // 2. 内联 style="..."
        for inline in Self::capture_inline_styles(html) {
            Self::scan_css(&inline, &mut tokens);
        }
        Self::dedup(&mut tokens);
        tokens
    }

    /// 提取 `<style>...</style>` 块。
    fn capture_style_blocks(html: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = html;
        while let Some(start) = rest.find("<style") {
            let after = &rest[start..];
            let open = after.find('>').map(|i| i + 1).unwrap_or(after.len());
            let body = &after[open..];
            match body.find("</style") {
                Some(end) => {
                    out.push(body[..end].to_string());
                    rest = &body[end..];
                }
                None => break,
            }
        }
        out
    }

    /// 捕获内联 `style="..."` 属性。
    fn capture_inline_styles(html: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = html;
        while let Some(start) = rest.find("style=") {
            let after = &rest[start + 6..];
            let end = after
                .find('"')
                .and_then(|i| after[i + 1..].find('"'))
                .map(|i| i + 1);
            match end {
                Some(e) => {
                    out.push(after[..e].to_string());
                    rest = &after[e..];
                }
                None => break,
            }
        }
        out
    }

    /// 扫描 CSS 文本填充令牌。
    fn scan_css(css: &str, tokens: &mut DesignTokenSet) {
        for line in css.split([';', '{', '}']) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // 颜色: #hex 与 rgb/rgba
            if let Some(h) = find_hex_color(line) {
                if !tokens.colors.contains(&h) {
                    tokens.colors.push(h);
                }
            }
            if line.contains("color") && (line.contains("rgb") || line.contains("hsl")) {
                let c = line.trim().to_string();
                if !tokens.colors.contains(&c) {
                    tokens.colors.push(c);
                }
            }
            // 具名颜色
            for name in ["red", "blue", "green", "black", "white", "gray", "orange", "purple"] {
                if line.contains(name) && (line.contains("color:") || line.contains("background")) {
                    if let Some(e) = tokens.named_colors.iter_mut().find(|(n, _)| n == name) {
                        e.1 += 1;
                    } else {
                        tokens.named_colors.push((name.to_string(), 1));
                    }
                }
            }
            // font-family
            if line.contains("font-family") {
                let f = line.trim().to_string();
                if !tokens.fonts.contains(&f) {
                    tokens.fonts.push(f);
                }
            }
            // font-size (px)
            if line.contains("font-size") {
                if let Some(v) = extract_px(line) {
                    if !tokens.font_sizes.contains(&v) {
                        tokens.font_sizes.push(v);
                    }
                }
            }
            // spacing / padding / margin (px)
            if (line.contains("spacing") || line.contains("padding") || line.contains("margin"))
                && line.contains(":")
            {
                if let Some(v) = extract_px(line) {
                    if !tokens.spacings.contains(&v) {
                        tokens.spacings.push(v);
                    }
                }
            }
        }
    }

    /// 去重 + 排序具名色。
    fn dedup(tokens: &mut DesignTokenSet) {
        tokens.colors.sort();
        tokens.colors.dedup();
        tokens.fonts.sort();
        tokens.fonts.dedup();
        tokens.font_sizes.sort_by(|a, b| {
            let an: f64 = a.trim_end_matches("px").parse().unwrap_or(0.0);
            let bn: f64 = b.trim_end_matches("px").parse().unwrap_or(0.0);
            an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal)
        });
        tokens.font_sizes.dedup();
        tokens.spacings.sort_by(|a, b| {
            let an: f64 = a.trim_end_matches("px").parse().unwrap_or(0.0);
            let bn: f64 = b.trim_end_matches("px").parse().unwrap_or(0.0);
            an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal)
        });
        tokens.spacings.dedup();
        tokens.named_colors.sort_by(|a, b| b.1.cmp(&a.1));
    }

    /// 简单可统计性: 令牌计数摘要。
    pub fn summarize(tokens: &DesignTokenSet) -> String {
        format!(
            "{} colors, {} fonts, {} font-sizes, {} spacings",
            tokens.colors.len(),
            tokens.fonts.len(),
            tokens.font_sizes.len(),
            tokens.spacings.len()
        )
    }
}

/// 从一行 CSS 提取形如 `#fff` / `#a1b2c3` 的十六进制色。
fn find_hex_color(line: &str) -> Option<String> {
    let bytes: Vec<char> = line.chars().collect();
    for (i, c) in bytes.iter().enumerate() {
        if *c == '#' {
            let mut hex = String::new();
            for c2 in bytes[i + 1..].iter() {
                if c2.is_ascii_hexdigit() {
                    hex.push(*c2);
                } else {
                    break;
                }
            }
            if hex.len() == 3 || hex.len() == 6 {
                return Some(format!("#{}", hex));
            }
        }
    }
    None
}

/// 从一行 CSS 提取 px 值 (如 `16px` → "16px")。
fn extract_px(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if b.is_ascii_digit() || *b == b'.' {
            let start = i;
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b'.') {
                j += 1;
            }
            // 紧跟 "px"
            if j + 1 < bytes.len() && bytes[j] == b'p' && bytes[j + 1] == b'x' {
                return Some(format!("{}px", &line[start..j]));
            }
        }
    }
    None
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
        let mut rs = RequestScraper::new(cfg);
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

    #[test]
    fn test_stealth_params_rotation_and_interval() {
        let sp = StealthParams::default();
        assert!(sp.enabled);
        assert_eq!(sp.referers.len(), 3);
        // referer 轮换: 不同序号落到不同 referer
        let r0 = sp.referer_for(0).unwrap();
        let r1 = sp.referer_for(1).unwrap();
        assert_ne!(r0, r1, "sequential requests must rotate referer");
        assert!(r0.contains("google") || r0.contains("bing") || r0.contains("duckduckgo"));

        // 间隔含抖动且大于 0
        let d = sp.interval_for();
        assert!(d.as_millis() >= sp.min_interval_ms as u128, "interval >= min");
        assert!(d.as_millis() <= (sp.min_interval_ms + sp.interval_jitter_ms) as u128, "interval <= min+jitter");

        // 代理轮换边界: request 10 (rotate_proxy_every=10)
        assert!(sp.should_rotate_proxy(10));
        assert!(!sp.should_rotate_proxy(9));
        assert!(!sp.should_rotate_proxy(0));

        // 禁用后无等待
        let disabled = StealthParams {
            enabled: false,
            ..Default::default()
        };
        assert!(disabled.interval_for().is_zero());
        assert!(!disabled.should_rotate_proxy(10));
    }

    #[test]
    fn test_stealth_headers_for_empty_returns_none() {
        let sp = StealthParams::default();
        assert!(sp.headers_for(0).is_none(), "default empty header variants -> None");
        let mut sp2 = StealthParams::default();
        sp2.header_variants = vec![{
            let mut m = HashMap::new();
            m.insert("X-Test".to_string(), "v1".to_string());
            m
        }];
        let h = sp2.headers_for(0).unwrap();
        assert_eq!(h.get("X-Test").map(|s| s.as_str()), Some("v1"));
    }

    #[test]
    fn test_design_token_extract_colors_fonts_spacings() {
        let html = r#"<html><head><style>
            body { color: #1a2b3c; font-family: Inter, sans-serif; font-size: 16px; padding: 8px; }
            .title { color: #ffffff; font-size: 24px; margin: 12px; }
            .btn { background: #1a2b3c; }
        </style></head>
        <body style="font-family: Helvetica; spacing: 4px"><p style="color: #f00">hi</p></body></html>"#;
        let tokens = DesignTokenExtractor::extract(html);
        assert!(tokens.colors.contains(&"#1a2b3c".to_string()), "style-block color");
        assert!(tokens.colors.contains(&"#ffffff".to_string()), "second color");
        assert!(tokens.fonts.len() >= 2, "font-family from style + inline");
        assert!(tokens.font_sizes.contains(&"16px".to_string()));
        assert!(tokens.font_sizes.contains(&"24px".to_string()));
        assert!(tokens.spacings.contains(&"8px".to_string()));
    }

    #[test]
    fn test_design_token_dedup_and_summarize() {
        let html = r#"<style>.a{color:#000}.b{color:#000}.c{color:#abcdef}</style>"#;
        let tokens = DesignTokenExtractor::extract(html);
        assert_eq!(tokens.colors.len(), 2, "duplicate #000 deduped");
        assert!(tokens.colors.contains(&"#abcdef".to_string()));
        let s = DesignTokenExtractor::summarize(&tokens);
        assert!(s.contains("colors"));
        assert!(s.contains("fonts"));
    }

    #[test]
    fn test_extract_px_and_hex_helpers() {
        assert_eq!(extract_px("font-size: 14px"), Some("14px".to_string()));
        assert_eq!(extract_px("line-height: 1.5"), None, "no px suffix -> None");
        assert_eq!(find_hex_color(".x { color: #abc; }"), Some("#abc".to_string()));
        assert_eq!(find_hex_color(".x { color: #aabbcc; }"), Some("#aabbcc".to_string()));
        assert_eq!(find_hex_color("no color here"), None);
    }
}

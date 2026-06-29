//! # WebContentExtractor — Accessibility-Tree-Equivalent Web Perception
//!
//! 2026 范式: Accessibility Tree > Screenshots > Raw DOM
//! 使用 HTTP + 智能结构化提取替代 PixelRAG 的截图→像素嵌入路径。
//! 无截图、无视觉模型、无 Playwright 运行时依赖。
//!
//! 三层提取:
//!   1. HTTP + 结构化解析 (primary, ~200-400 tokens, <100ms)
//!   2. CDP StealthBrowser (JS-rendered fallback, feature-gated)
//!   3. Simulated (安全降级)

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::neotrix::nt_io_http_factory::{
    BlockingHttpClientAdapter, HttpClientBackend, HttpClientConfig, TlsFingerprint,
};

// ── Extracted Content Types ──

#[derive(Debug, Clone)]
pub struct WebPageContent {
    pub url: String,
    pub title: String,
    pub description: String,
    pub language: String,
    pub text_content: String,
    pub headings: Vec<(u32, String)>,
    pub links: Vec<(String, String)>,
    pub content_type: ContentCategory,
    pub extraction_time_ms: u64,
    pub extraction_method: ExtractionMethod,
    pub source_reliability: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentCategory {
    Article,
    Documentation,
    Product,
    Forum,
    Social,
    SearchResults,
    Media,
    Portal,
    Api,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtractionMethod {
    HttpDirect,
    StealthBrowser,
    Simulated,
}

// ── Extraction Config ──

#[derive(Debug, Clone)]
pub struct WebContentConfig {
    pub request_timeout_ms: u64,
    pub max_text_length: usize,
    pub max_headings: usize,
    pub max_links: usize,
    pub enable_simulated_fallback: bool,
    pub enable_stealth_browser: bool,
    pub enable_stealth_http: bool,
    pub stealth_proxy_url: Option<String>,
    pub cache_ttl_secs: u64,
}

impl Default for WebContentConfig {
    fn default() -> Self {
        Self {
            request_timeout_ms: 15000,
            max_text_length: 100_000,
            max_headings: 50,
            max_links: 200,
            enable_simulated_fallback: true,
            enable_stealth_browser: cfg!(feature = "stealth-browser"),
            enable_stealth_http: false,
            stealth_proxy_url: None,
            cache_ttl_secs: 300,
        }
    }
}

// ── Content Cache ──

#[derive(Debug, Clone)]
struct CacheEntry {
    content: WebPageContent,
    fetched_at: Instant,
}

#[derive(Debug, Clone)]
pub struct WebContentExtractor {
    config: WebContentConfig,
    http_client: Option<BlockingHttpClientAdapter>,
    cache: HashMap<String, CacheEntry>,
    stats: ExtractorStats,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractorStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub http_success: u64,
    pub http_failures: u64,
    pub simulated_fallbacks: u64,
    pub avg_extraction_time_ms: f64,
}

impl WebContentExtractor {
    pub fn new(config: WebContentConfig) -> Self {
        let backend = if config.enable_stealth_http {
            HttpClientBackend::Stealth
        } else {
            HttpClientBackend::Simple
        };
        let http_config = HttpClientConfig {
            backend,
            tls_fingerprint: TlsFingerprint::Chrome116,
            proxy_url: config.stealth_proxy_url.clone(),
            timeout_secs: (config.request_timeout_ms / 1000).max(1),
            max_retries: 3,
            extra_headers: vec![],
        };
        let http_client = match std::panic::catch_unwind(|| {
            crate::neotrix::nt_io_http_factory::create_blocking_http_client(http_config)
        }) {
            Ok(client) => Some(client),
            Err(_) => {
                log::warn!("[WebContentExtractor] failed to create blocking HTTP client (no tokio runtime?)");
                None
            }
        };
        Self {
            config,
            http_client,
            cache: HashMap::new(),
            stats: ExtractorStats::default(),
        }
    }

    /// Extract structured content from a URL.
    /// Returns None when all extraction methods fail.
    pub fn extract(&mut self, url: &str) -> Option<WebPageContent> {
        self.stats.total_requests += 1;

        // 1. Check cache
        if let Some(entry) = self.cache.get(url) {
            if entry.fetched_at.elapsed() < Duration::from_secs(self.config.cache_ttl_secs) {
                self.stats.cache_hits += 1;
                return Some(entry.content.clone());
            }
        }

        // 2. Try HTTP direct (primary path, like accessibility tree)
        let result = self.extract_http(url);

        // 3. Fallback to StealthBrowser (JS-rendered pages)
        let result = result.or_else(|| {
            if self.config.enable_stealth_browser {
                self.extract_stealth(url)
            } else {
                None
            }
        });

        // 4. Simulated fallback (safety net)
        let result = result.or_else(|| {
            if self.config.enable_simulated_fallback {
                self.stats.simulated_fallbacks += 1;
                Some(self.simulate_extraction(url))
            } else {
                None
            }
        });

        // Cache if successful
        if let Some(ref content) = result {
            self.cache.insert(
                url.to_string(),
                CacheEntry {
                    content: content.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }

        result
    }

    // ── HTTP Direct Extraction (Primary) ──

    fn extract_http(&mut self, url: &str) -> Option<WebPageContent> {
        let t0 = Instant::now();
        let client = self.http_client.as_ref()?;

        let result = client.get_blocking(url).ok()?;
        if result.status_code < 200 || result.status_code >= 300 {
            self.stats.http_failures += 1;
            return None;
        }

        let html = String::from_utf8(result.body).ok()?;
        self.stats.http_success += 1;

        let content = self.parse_html(url, &html, ExtractionMethod::HttpDirect);
        let elapsed = t0.elapsed().as_millis() as u64;
        let mut result = content;
        result.extraction_time_ms = elapsed;

        // Update rolling average
        let n = self.stats.http_success as f64;
        self.stats.avg_extraction_time_ms =
            (self.stats.avg_extraction_time_ms * (n - 1.0) + elapsed as f64) / n;

        Some(result)
    }

    // ── HTML Parser (Readability-style extraction) ──

    fn parse_html(&self, url: &str, html: &str, method: ExtractionMethod) -> WebPageContent {
        let lower = html.to_lowercase();

        // Title extraction
        let title = self
            .extract_tag(&lower, "<title", "</title>")
            .or_else(|| self.extract_tag(&lower, "<h1", "</h1>"))
            .unwrap_or_default()
            .trim()
            .to_string();

        // Description / meta
        let description = self
            .extract_meta(&lower, "description")
            .or_else(|| self.extract_meta(&lower, "og:description"))
            .unwrap_or_default();

        // Language
        let language = self
            .extract_meta(&lower, "language")
            .or_else(|| {
                let s = self.extract_attr(&lower, "html", "lang");
                s.map(|s| s.split('-').next().unwrap_or(&s).to_string())
            })
            .unwrap_or_else(|| "en".into());

        // Headings
        let headings: Vec<(u32, String)> = ["h1", "h2", "h3"]
            .iter()
            .flat_map(|tag| {
                let level = tag[1..].parse::<u32>().unwrap_or(1);
                self.extract_all_tags(&lower, tag)
                    .into_iter()
                    .map(move |t| (level, t.trim().to_string()))
            })
            .take(self.config.max_headings)
            .collect();

        // Links
        let links: Vec<(String, String)> = self
            .extract_all_links(html)
            .into_iter()
            .take(self.config.max_links)
            .collect();

        // Main text content (article / body)
        let text_content = self
            .extract_article_text(&lower)
            .or_else(|| self.extract_tag(&lower, "<body", "</body>"))
            .map(|s| {
                let cleaned = self.strip_html_tags(&s);
                let max = self.config.max_text_length;
                if cleaned.len() > max {
                    cleaned[..max].to_string()
                } else {
                    cleaned
                }
            })
            .unwrap_or_default();

        // Content category
        let content_type = self.classify_content(url, &lower);

        WebPageContent {
            url: url.to_string(),
            title,
            description,
            language,
            text_content,
            headings,
            links,
            content_type,
            extraction_time_ms: 0,
            source_reliability: match &method {
                ExtractionMethod::HttpDirect => 0.85,
                ExtractionMethod::StealthBrowser => 0.90,
                ExtractionMethod::Simulated => 0.20,
            },
            extraction_method: method,
        }
    }

    // ── StealthBrowser Fallback ──

    fn extract_stealth(&mut self, url: &str) -> Option<WebPageContent> {
        // Feature-gated: requires `stealth-browser` feature
        #[cfg(feature = "stealth-browser")]
        {
            use crate::neotrix::nt_world_crawl::stealth_browser::StealthBrowser;
            let t0 = Instant::now();
            let mut browser = StealthBrowser::launch().ok()?;
            let html = browser.fetch(url).ok()?;
            let content = self.parse_html(url, &html, ExtractionMethod::StealthBrowser);
            let elapsed = t0.elapsed().as_millis() as u64;
            let mut result = content;
            result.extraction_time_ms = elapsed;
            return Some(result);
        }
        #[cfg(not(feature = "stealth-browser"))]
        {
            let _ = url;
            None
        }
    }

    // ── Simulated Fallback ──

    fn simulate_extraction(&self, url: &str) -> WebPageContent {
        WebPageContent {
            url: url.to_string(),
            title: format!("Page at {}", url),
            description: String::new(),
            language: "en".into(),
            text_content: format!("[Simulated content for {}]", url),
            headings: vec![(1, format!("Page: {}", url))],
            links: vec![],
            content_type: ContentCategory::Other,
            extraction_time_ms: 0,
            extraction_method: ExtractionMethod::Simulated,
            source_reliability: 0.20,
        }
    }

    // ── Content Classification ──

    fn classify_content(&self, url: &str, lower: &str) -> ContentCategory {
        if url.contains("arxiv.org")
            || url.contains("wikipedia.org")
            || url.contains("medium.com")
            || lower.contains("article")
        {
            ContentCategory::Article
        } else if url.contains("docs.")
            || url.contains("documentation")
            || lower.contains("documentation")
        {
            ContentCategory::Documentation
        } else if url.contains("product")
            || lower.contains("buy")
            || lower.contains("cart")
            || lower.contains("price")
        {
            ContentCategory::Product
        } else if lower.contains("forum")
            || lower.contains("thread")
            || lower.contains("reddit.com")
        {
            ContentCategory::Forum
        } else if url.contains("twitter.com")
            || url.contains("x.com")
            || url.contains("instagram")
            || url.contains("tiktok")
        {
            ContentCategory::Social
        } else if lower.contains("search") || url.contains("search") {
            ContentCategory::SearchResults
        } else if lower.contains("video") || lower.contains("media") {
            ContentCategory::Media
        } else if url.contains("api.") || lower.contains("api") {
            ContentCategory::Api
        } else {
            ContentCategory::Other
        }
    }

    // ── HTML Helper Methods ──

    fn extract_tag(&self, lower: &str, open_tag: &str, close_tag: &str) -> Option<String> {
        let start = lower.find(open_tag)?;
        let content_start = lower[start..].find('>')? + start + 1;
        // Handle self-closing tags
        if content_start >= lower.len() {
            return None;
        }
        let end = lower[content_start..].find(close_tag)?;
        let content = &lower[content_start..content_start + end];
        if content.is_empty() {
            None
        } else {
            Some(content.to_string())
        }
    }

    fn extract_all_tags(&self, lower: &str, tag: &str) -> Vec<String> {
        let open = format!("<{}", tag);
        let close = format!("</{}>", tag);
        let mut results = Vec::new();
        let mut search_start = 0;
        while let Some(start) = lower[search_start..].find(&open) {
            let abs_start = search_start + start;
            if let Some(content_start) = lower[abs_start..].find('>') {
                let abs_content = abs_start + content_start + 1;
                if let Some(end) = lower[abs_content..].find(&close) {
                    let content = &lower[abs_content..abs_content + end];
                    results.push(content.trim().to_string());
                    search_start = abs_content + end + close.len();
                    continue;
                }
            }
            search_start = abs_start + open.len();
        }
        results
    }

    fn extract_meta(&self, lower: &str, name: &str) -> Option<String> {
        // <meta name="description" content="...">
        let pattern = format!("name=\"{}\"", name.to_lowercase());
        let start = lower.find(&pattern)?;
        let after = &lower[start + pattern.len()..];
        let content_start = after.find("content=\"")? + 9;
        let content_end = after[content_start..].find('"')?;
        let content = &after[content_start..content_start + content_end];
        if content.is_empty() {
            None
        } else {
            Some(content.to_string())
        }
    }

    fn extract_attr(&self, html: &str, tag: &str, attr: &str) -> Option<String> {
        let open = format!("<{} ", tag);
        let start = html.find(&open)?;
        let after_tag = &html[start + open.len() - 1..];
        let attr_pattern = format!("{}=\"", attr);
        let attr_start = after_tag.find(&attr_pattern)? + attr_pattern.len();
        let attr_end = after_tag[attr_start..].find('"')?;
        let value = &after_tag[attr_start..attr_start + attr_end];
        Some(value.to_string())
    }

    fn strip_html_tags(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut in_tag = false;
        let mut in_entity = false;
        let mut entity_buf = String::new();
        for c in text.chars() {
            match c {
                '<' => in_tag = true,
                '>' if in_tag => in_tag = false,
                _ if !in_tag => {
                    if c == '&' {
                        in_entity = true;
                        entity_buf.clear();
                    } else if in_entity {
                        if c == ';' {
                            let decoded = match entity_buf.as_str() {
                                "amp" => "&",
                                "lt" => "<",
                                "gt" => ">",
                                "quot" => "\"",
                                "apos" => "'",
                                "nbsp" => " ",
                                _ => "",
                            };
                            result.push_str(decoded);
                            in_entity = false;
                        } else {
                            entity_buf.push(c);
                        }
                    } else {
                        result.push(c);
                    }
                }
                _ => {}
            }
        }
        // Collapse whitespace
        let mut cleaned = String::with_capacity(result.len());
        let mut prev_space = false;
        for c in result.chars() {
            if c.is_whitespace() {
                if !prev_space {
                    cleaned.push(' ');
                    prev_space = true;
                }
            } else {
                cleaned.push(c);
                prev_space = false;
            }
        }
        cleaned.trim().to_string()
    }

    fn extract_all_links(&self, html: &str) -> Vec<(String, String)> {
        let mut links = Vec::new();
        let mut search_start = 0;
        while let Some(start) = html[search_start..].find("<a ") {
            let abs_start = search_start + start;
            let end = html[abs_start..].find("</a>").map(|e| abs_start + e + 4);
            let section = if let Some(e) = end {
                &html[abs_start..e]
            } else {
                &html[abs_start..]
            };
            // Extract href
            let href = self.extract_attr(section, "a", "href").unwrap_or_default();
            // Extract text
            let text = self
                .extract_tag(&section.to_lowercase(), ">", "</a>")
                .map(|s| self.strip_html_tags(&s))
                .unwrap_or_default();
            if !href.is_empty() {
                links.push((href, text));
            }
            search_start = end.unwrap_or(abs_start + 4);
        }
        links
    }

    fn extract_article_text(&self, lower: &str) -> Option<String> {
        // Try common article containers
        for tag in &[
            "<article",
            "<main",
            "<div class=\"content\"",
            "<div id=\"content\"",
        ] {
            if let Some(content) = self
                .extract_tag(lower, tag, "</div>")
                .or_else(|| self.extract_tag(lower, tag, "</main>"))
                .or_else(|| self.extract_tag(lower, tag, "</article>"))
            {
                if content.len() > 200 {
                    return Some(content);
                }
            }
        }
        None
    }

    // ── Public API ──

    pub fn stats(&self) -> &ExtractorStats {
        &self.stats
    }

    pub fn config(&self) -> &WebContentConfig {
        &self.config
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Convert extracted web content to a perception-friendly summary string
    pub fn to_perception_text(content: &WebPageContent, max_tokens: usize) -> String {
        let mut parts = Vec::new();
        parts.push(format!("Title: {}", content.title));
        if !content.description.is_empty() {
            parts.push(format!("Description: {}", content.description));
        }
        parts.push(format!("Type: {:?}", content.content_type));
        parts.push(format!("Language: {}", content.language));
        parts.push(format!("Source: {:?}", content.extraction_method));

        // Headings as outline
        if !content.headings.is_empty() {
            let mut outline = String::from("Outline:");
            for (level, text) in &content.headings {
                let indent = "  ".repeat(*level as usize - 1);
                outline.push_str(&format!("\n{}- {}", indent, text));
            }
            parts.push(outline);
        }

        // Main text (truncated)
        let max_chars = max_tokens * 4; // rough estimate: 4 chars per token
        let text = if content.text_content.len() > max_chars {
            format!("{}...", &content.text_content[..max_chars])
        } else {
            content.text_content.clone()
        };
        parts.push(format!("Content:\n{}", text));

        // Links summary
        if !content.links.is_empty() {
            let n = content.links.len().min(10);
            let mut links_text = format!("Top {} links:", n);
            for (href, text) in content.links.iter().take(n) {
                links_text.push_str(&format!("\n- {} ({})", text, href));
            }
            parts.push(links_text);
        }

        parts.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_extractor() {
        let ext = WebContentExtractor::new(WebContentConfig::default());
        assert_eq!(ext.stats.total_requests, 0);
    }

    #[test]
    fn test_parse_html_extracts_title() {
        let ext = WebContentExtractor::new(WebContentConfig::default());
        let html = "<html><head><title>Test Page</title></head><body><p>Hello</p></body></html>";
        let content = ext.parse_html("https://example.com", html, ExtractionMethod::HttpDirect);
        assert_eq!(content.title, "test page");
    }

    #[test]
    fn test_parse_html_extracts_headings() {
        let ext = WebContentExtractor::new(WebContentConfig::default());
        let html = "<html><body><h1>Main</h1><h2>Sub</h2><p>text</p></body></html>";
        let content = ext.parse_html("https://example.com", html, ExtractionMethod::HttpDirect);
        assert!(content.headings.iter().any(|(l, t)| *l == 1 && t == "main"));
        assert!(content.headings.iter().any(|(l, t)| *l == 2 && t == "sub"));
    }

    #[test]
    fn test_parse_html_extracts_links() {
        let ext = WebContentExtractor::new(WebContentConfig::default());
        let html = "<html><body><a href=\"https://example.com/page\">Link text</a></body></html>";
        let content = ext.parse_html("https://example.com", html, ExtractionMethod::HttpDirect);
        assert_eq!(content.links.len(), 1);
        assert_eq!(content.links[0].0, "https://example.com/page");
    }

    #[test]
    fn test_simulate_extraction() {
        let ext = WebContentExtractor::new(WebContentConfig::default());
        let content = ext.simulate_extraction("https://example.com");
        assert_eq!(content.url, "https://example.com");
        assert_eq!(content.source_reliability, 0.20);
    }

    #[test]
    fn test_strip_html_tags() {
        let ext = WebContentExtractor::new(WebContentConfig::default());
        let result = ext.strip_html_tags("<p>Hello <b>world</b> &amp; foo</p>");
        assert_eq!(result, "Hello world & foo");
    }

    #[test]
    fn test_classify_article() {
        let ext = WebContentExtractor::new(WebContentConfig::default());
        assert_eq!(
            ext.classify_content("https://arxiv.org/abs/2501.00001", ""),
            ContentCategory::Article
        );
    }

    #[test]
    fn test_classify_product() {
        let ext = WebContentExtractor::new(WebContentConfig::default());
        assert_eq!(
            ext.classify_content("https://store.example.com/product/123", "buy now price $10"),
            ContentCategory::Product
        );
    }

    #[test]
    fn test_to_perception_text() {
        let content = WebPageContent {
            url: "https://example.com".into(),
            title: "Test".into(),
            description: "A test page".into(),
            language: "en".into(),
            text_content: "Hello world this is test content for the accessibility tree extraction module. It contains useful information about web perception.".into(),
            headings: vec![(1, "Main Heading".into())],
            links: vec![("https://example.com/page".into(), "Link".into())],
            content_type: ContentCategory::Article,
            extraction_time_ms: 42,
            extraction_method: ExtractionMethod::HttpDirect,
            source_reliability: 0.85,
        };
        let text = WebContentExtractor::to_perception_text(&content, 100);
        assert!(text.contains("Title: Test"));
        assert!(text.contains("Link"));
    }

    #[test]
    fn test_cache_hit() {
        let mut ext = WebContentExtractor::new(WebContentConfig::default());
        // Simulate by directly inserting into cache
        let content = ext.simulate_extraction("https://example.com");
        ext.cache.insert(
            "https://example.com".into(),
            CacheEntry {
                content: content.clone(),
                fetched_at: Instant::now(),
            },
        );
        let cached = ext.extract("https://example.com");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().url, "https://example.com");
        assert_eq!(ext.stats.cache_hits, 1);
    }
}

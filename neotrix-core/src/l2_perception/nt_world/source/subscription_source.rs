//! SubscriptionSource — RSS/Atom feed + API subscription data
//!
//! Accesses data via subscription URLs (RSS feeds, API endpoints, webhook subscriptions).
//! Integrates with crawl infrastructure for blocked/restricted network access via
//! proxy, Tor, and Camofox stealth capabilities.

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;

use quick_xml::events::Event as XmlEvent;
use quick_xml::Reader;

use crate::l2_perception::nt_world::crawl;
use super::unified::{CrawlResult, CrawlSource, DataSource, SourceDomain};

// ============================================================================
// StealthLevel — network access obfuscation tiers
// ============================================================================

/// Network access stealth level for blocked/restricted feeds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StealthLevel {
    /// Direct connection — no obfuscation
    None,
    /// HTTP/SOCKS proxy routing
    Proxy,
    /// Tor network (SOCKS5 on 127.0.0.1:9050)
    Tor,
    /// Camofox anti-detection browser
    Camofox,
    /// Full stealth — fingerprint spoofing + proxy + randomized identity
    Stealth,
}

impl Default for StealthLevel {
    fn default() -> Self {
        Self::None
    }
}

impl StealthLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Proxy => "proxy",
            Self::Tor => "tor",
            Self::Camofox => "camofox",
            Self::Stealth => "stealth",
        }
    }
}

// ============================================================================
// SubscriptionItem — parsed feed entry
// ============================================================================

/// A single item from an RSS/Atom feed or API subscription
#[derive(Debug, Clone)]
pub struct SubscriptionItem {
    pub id: String,
    pub title: String,
    pub url: String,
    pub content: String,
    pub published: Option<chrono::DateTime<chrono::Utc>>,
    pub source: String,
    pub tags: Vec<String>,
}

// ============================================================================
// SubscriptionSource — RSS/Atom feed + API subscription data
// ============================================================================

/// Data source for RSS/Atom feeds and API subscription endpoints
pub struct SubscriptionSource {
    id: String,
    name: String,
    feed_url: String,
    proxy: Option<String>,
    stealth_level: StealthLevel,
    tor_socks_port: u16,
    camofox_port: u16,
}

impl SubscriptionSource {
    pub fn new(id: &str, name: &str, feed_url: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            feed_url: feed_url.to_string(),
            proxy: None,
            stealth_level: StealthLevel::None,
            tor_socks_port: 9050,
            camofox_port: 9377,
        }
    }

    pub fn with_proxy(mut self, proxy: &str) -> Self {
        self.proxy = Some(proxy.to_string());
        self
    }

    pub fn with_stealth(mut self, level: StealthLevel) -> Self {
        self.stealth_level = level;
        self
    }

    pub fn with_tor_port(mut self, port: u16) -> Self {
        self.tor_socks_port = port;
        self
    }

    pub fn with_camofox_port(mut self, port: u16) -> Self {
        self.camofox_port = port;
        self
    }

    // ------------------------------------------------------------------
    // Core fetch methods
    // ------------------------------------------------------------------

    /// Fetch the configured feed URL and parse into SubscriptionItems
    pub async fn fetch_feed(&self) -> Result<Vec<SubscriptionItem>, String> {
        let xml = self.fetch_url(&self.feed_url).await?;
        parse_feed(&xml, &self.id)
    }

    /// Fetch a URL using the configured stealth level
    pub async fn fetch_url(&self, url: &str) -> Result<String, String> {
        match self.stealth_level {
            StealthLevel::None => self.fetch_direct(url).await,
            StealthLevel::Proxy => self.fetch_via_proxy(url).await,
            StealthLevel::Tor => self.fetch_via_tor(url).await,
            StealthLevel::Camofox => self.fetch_via_camofox(url).await,
            StealthLevel::Stealth => self.fetch_via_stealth(url).await,
        }
    }

    /// Direct HTTP fetch via reqwest
    async fn fetch_direct(&self, url: &str) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(crawl::stealth::Fingerprint::random().user_agent)
            .build()
            .map_err(|e| format!("client build: {e}"))?;

        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }

        resp.text()
            .await
            .map_err(|e| format!("body read: {e}"))
    }

    /// Fetch via configured HTTP/SOCKS proxy
    async fn fetch_via_proxy(&self, url: &str) -> Result<String, String> {
        let proxy_url = self.proxy.as_deref().ok_or("no proxy configured")?;
        let proxy = reqwest::Proxy::all(proxy_url)
            .map_err(|e| format!("proxy parse: {e}"))?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .proxy(proxy)
            .user_agent(crawl::stealth::Fingerprint::random().user_agent)
            .build()
            .map_err(|e| format!("client build: {e}"))?;

        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }

        resp.text()
            .await
            .map_err(|e| format!("body read: {e}"))
    }

    /// Fetch via Tor network (SOCKS5 proxy on 127.0.0.1:<tor_socks_port>)
    pub async fn fetch_via_tor(&self, url: &str) -> Result<String, String> {
        let tor_addr = format!("socks5h://127.0.0.1:{}", self.tor_socks_port);
        let proxy = reqwest::Proxy::all(&tor_addr)
            .map_err(|e| format!("tor proxy parse: {e}"))?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .proxy(proxy)
            .user_agent(crawl::stealth::Fingerprint::random().user_agent)
            .build()
            .map_err(|e| format!("tor client build: {e}"))?;

        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("tor request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("tor HTTP {}", resp.status()));
        }

        resp.text()
            .await
            .map_err(|e| format!("tor body read: {e}"))
    }

    /// Fetch via Camofox anti-detection browser
    pub async fn fetch_via_camofox(&self, url: &str) -> Result<String, String> {
        use std::process::Command;

        let output = Command::new("camofox")
            .args([
                "fetch", url,
                "--port", &self.camofox_port.to_string(),
                "--format", "text",
            ])
            .output()
            .map_err(|e| format!("camofox spawn: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("camofox failed: {stderr}"));
        }

        let body = String::from_utf8_lossy(&output.stdout).to_string();
        if body.is_empty() {
            return Err("camofox: empty response".into());
        }
        Ok(body)
    }

    /// Full stealth fetch — fingerprint-spoofed session + randomized identity
    async fn fetch_via_stealth(&self, url: &str) -> Result<String, String> {
        let fp = crawl::stealth::Fingerprint::random();

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_str(&fp.user_agent)
                .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("Mozilla/5.0")),
        );
        headers.insert(
            reqwest::header::ACCEPT_LANGUAGE,
            reqwest::header::HeaderValue::from_str(&fp.language)
                .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("en-US,en;q=0.9")),
        );
        headers.insert(
            reqwest::header::DNT,
            reqwest::header::HeaderValue::from_static("1"),
        );

        let mut builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers(headers);

        if let Some(ref proxy_url) = self.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| format!("stealth proxy parse: {e}"))?;
            builder = builder.proxy(proxy);
        }

        let client = builder
            .build()
            .map_err(|e| format!("stealth client build: {e}"))?;

        let resp = client
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .map_err(|e| format!("stealth request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("stealth HTTP {}", resp.status()));
        }

        resp.text()
            .await
            .map_err(|e| format!("stealth body read: {e}"))
    }
}

// ============================================================================
// DataSource + CrawlSource trait implementation
// ============================================================================

impl DataSource for SubscriptionSource {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn domains(&self) -> Vec<SourceDomain> {
        vec![SourceDomain::Tech, SourceDomain::Intel]
    }

    fn requires_key(&self) -> bool {
        false
    }
}

impl CrawlSource for SubscriptionSource {
    fn crawl(
        &self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<CrawlResult, String>> + Send>> {
        let url = url.to_string();
        let this = self.clone_self();
        Box::pin(async move {
            let body = this.fetch_url(&url).await?;
            Ok(CrawlResult {
                url,
                title: None,
                content: body,
                links: Vec::new(),
                metadata: HashMap::new(),
            })
        })
    }

    fn crawl_batch(
        &self,
        urls: &[String],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<CrawlResult>, String>> + Send>> {
        let urls = urls.to_vec();
        let this = self.clone_self();
        Box::pin(async move {
            let mut results = Vec::with_capacity(urls.len());
            for url in &urls {
                match this.fetch_url(url).await {
                    Ok(body) => results.push(CrawlResult {
                        url: url.clone(),
                        title: None,
                        content: body,
                        links: Vec::new(),
                        metadata: HashMap::new(),
                    }),
                    Err(e) => log::warn!("[subscription] batch crawl failed for {url}: {e}"),
                }
            }
            Ok(results)
        })
    }

    fn supported_patterns(&self) -> Vec<&str> {
        vec![
            "rss", "atom", "xml",
            "api", "json", "feed",
            "webhook", "subscribe",
        ]
    }
}

impl SubscriptionSource {
    /// Clone self into an owned value for async move blocks
    fn clone_self(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            feed_url: self.feed_url.clone(),
            proxy: self.proxy.clone(),
            stealth_level: self.stealth_level,
            tor_socks_port: self.tor_socks_port,
            camofox_port: self.camofox_port,
        }
    }
}

// ============================================================================
// RSS/Atom feed parser
// ============================================================================

/// Parse RSS 2.0 or Atom feed XML into SubscriptionItems
fn parse_feed(xml: &str, source_id: &str) -> Result<Vec<SubscriptionItem>, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut items = Vec::new();
    let mut buf = Vec::new();
    let mut in_item = false;
    let mut current_tag = String::new();
    let mut title = String::new();
    let mut link = String::new();
    let mut description = String::new();
    let mut pub_date = String::new();
    let mut content = String::new();
    let mut tags = Vec::new();
    let mut is_atom = false;
    let mut atom_entry_depth = 0u32;

    // Detect Atom vs RSS
    if xml.contains("<feed") && (xml.contains("xmlns=\"http://www.w3.org/2005/Atom\"") || xml.contains("xmlns:atom")) {
        is_atom = true;
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Start(ref e)) | Ok(XmlEvent::Empty(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if is_atom {
                    match tag_name.as_str() {
                        "entry" => {
                            in_item = true;
                            atom_entry_depth += 1;
                            title.clear();
                            link.clear();
                            description.clear();
                            content.clear();
                            pub_date.clear();
                            tags.clear();
                        }
                        "id" if in_item => current_tag = "id".into(),
                        "title" if in_item => current_tag = "title".into(),
                        "link" if in_item => {
                            // Atom link is in attributes
                            for attr in e.attributes().filter_map(|a| a.ok()) {
                                if attr.key.as_ref() == b"href" {
                                    link = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                        }
                        "summary" if in_item => current_tag = "summary".into(),
                        "content" if in_item => current_tag = "content".into(),
                        "published" | "updated" if in_item => current_tag = "pubdate".into(),
                        "category" if in_item => {
                            for attr in e.attributes().filter_map(|a| a.ok()) {
                                if attr.key.as_ref() == b"term" {
                                    tags.push(String::from_utf8_lossy(&attr.value).to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    // RSS 2.0
                    match tag_name.as_str() {
                        "item" => {
                            in_item = true;
                            title.clear();
                            link.clear();
                            description.clear();
                            content.clear();
                            pub_date.clear();
                            tags = Vec::new();
                        }
                        "title" if in_item => current_tag = "title".into(),
                        "link" if in_item => current_tag = "link".into(),
                        "description" if in_item => current_tag = "description".into(),
                        "content:encoded" if in_item => current_tag = "content".into(),
                        "pubDate" if in_item => current_tag = "pubdate".into(),
                        "category" if in_item => current_tag = "category".into(),
                        _ => {}
                    }
                }
            }
            Ok(XmlEvent::Text(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).into_owned();
                if in_item {
                    match current_tag.as_str() {
                        "title" => title.push_str(&text),
                        "link" => link.push_str(&text),
                        "description" => description.push_str(&text),
                        "content" => content.push_str(&text),
                        "pubdate" => pub_date.push_str(&text),
                        "category" => tags.push(text),
                        "id" if is_atom => {
                            if link.is_empty() {
                                link = text;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(XmlEvent::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if is_atom && tag_name == "entry" {
                    atom_entry_depth = atom_entry_depth.saturating_sub(1);
                    if atom_entry_depth == 0 {
                        in_item = false;
                    }
                } else if !is_atom && tag_name == "item" {
                    in_item = false;
                }

                if !in_item && (!title.is_empty() || !link.is_empty()) {
                    let item_content = if content.is_empty() {
                        description.clone()
                    } else {
                        content.clone()
                    };
                    let published = parse_rfc2822_date(&pub_date)
                        .or_else(|| parse_iso8601_date(&pub_date));

                    items.push(SubscriptionItem {
                        id: format!("{}:{}", source_id, items.len()),
                        title: title.trim().to_string(),
                        url: link.trim().to_string(),
                        content: item_content,
                        published,
                        source: source_id.to_string(),
                        tags: std::mem::take(&mut tags),
                    });
                }
                current_tag.clear();
            }
            Ok(XmlEvent::Eof) => break,
            Err(e) => return Err(format!("xml parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    // Handle case where feed has items but no closing tag match
    if !in_item && !title.is_empty() && !link.is_empty() {
        let item_content = if content.is_empty() { description } else { content };
        items.push(SubscriptionItem {
            id: format!("{}:{}", source_id, items.len()),
            title: title.trim().to_string(),
            url: link.trim().to_string(),
            content: item_content,
            published: parse_rfc2822_date(&pub_date).or_else(|| parse_iso8601_date(&pub_date)),
            source: source_id.to_string(),
            tags,
        });
    }

    Ok(items)
}

/// Parse RFC 2822 date (used in RSS pubDate)
fn parse_rfc2822_date(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    chrono::DateTime::parse_from_rfc2822(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

/// Parse ISO 8601 / RFC 3339 date (used in Atom)
fn parse_iso8601_date(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_stealth_level_default() {
        assert_eq!(StealthLevel::default(), StealthLevel::None);
    }

    #[test]
    fn test_stealth_level_as_str() {
        assert_eq!(StealthLevel::None.as_str(), "none");
        assert_eq!(StealthLevel::Proxy.as_str(), "proxy");
        assert_eq!(StealthLevel::Tor.as_str(), "tor");
        assert_eq!(StealthLevel::Camofox.as_str(), "camofox");
        assert_eq!(StealthLevel::Stealth.as_str(), "stealth");
    }

    #[test]
    fn test_subscription_source_builder() {
        let src = SubscriptionSource::new("rss1", "Hacker News", "https://news.ycombinator.com/rss")
            .with_proxy("http://proxy:8080")
            .with_stealth(StealthLevel::Tor)
            .with_tor_port(9150)
            .with_camofox_port(8080);

        assert_eq!(src.id, "rss1");
        assert_eq!(src.name, "Hacker News");
        assert_eq!(src.feed_url, "https://news.ycombinator.com/rss");
        assert_eq!(src.proxy.as_deref(), Some("http://proxy:8080"));
        assert_eq!(src.stealth_level, StealthLevel::Tor);
        assert_eq!(src.tor_socks_port, 9150);
        assert_eq!(src.camofox_port, 8080);
    }

    #[test]
    fn test_subscription_source_data_source_trait() {
        let src = SubscriptionSource::new("rss1", "Test", "https://example.com/feed");
        assert_eq!(src.id(), "rss1");
        assert_eq!(src.name(), "Test");
        assert!(!src.requires_key());
        let domains = src.domains();
        assert!(domains.contains(&SourceDomain::Tech));
        assert!(domains.contains(&SourceDomain::Intel));
    }

    #[test]
    fn test_subscription_source_supported_patterns() {
        let src = SubscriptionSource::new("rss1", "Test", "https://example.com/feed");
        let patterns = src.supported_patterns();
        assert!(patterns.contains(&"rss"));
        assert!(patterns.contains(&"atom"));
        assert!(patterns.contains(&"feed"));
    }

    #[test]
    fn test_parse_rss_feed() {
        let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel>
    <title>Test Feed</title>
    <link>https://example.com</link>
    <description>A test feed</description>
    <item>
        <title>First Article</title>
        <link>https://example.com/1</link>
        <description>Description of first article</description>
        <category>tech</category>
        <category>news</category>
        <pubDate>Mon, 01 Jan 2024 12:00:00 +0000</pubDate>
    </item>
    <item>
        <title>Second Article</title>
        <link>https://example.com/2</link>
        <description>Description of second article</description>
        <category>science</category>
        <pubDate>Tue, 02 Jan 2024 14:30:00 +0000</pubDate>
    </item>
</channel>
</rss>"#;

        let items = parse_feed(rss, "test_rss").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "First Article");
        assert_eq!(items[0].url, "https://example.com/1");
        assert_eq!(items[0].tags, vec!["tech", "news"]);
        assert!(items[0].published.is_some());
        assert_eq!(items[1].title, "Second Article");
        assert_eq!(items[1].source, "test_rss");
    }

    #[test]
    fn test_parse_atom_feed() {
        let atom = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <title>Atom Feed</title>
    <link href="https://example.com"/>
    <entry>
        <title>Atom Entry 1</title>
        <link href="https://example.com/atom/1"/>
        <id>urn:uuid:1</id>
        <summary>Summary of entry 1</summary>
        <published>2024-01-01T12:00:00Z</published>
        <category term="ai"/>
        <category term="ml"/>
    </entry>
    <entry>
        <title>Atom Entry 2</title>
        <link href="https://example.com/atom/2"/>
        <id>urn:uuid:2</id>
        <content>Full content of entry 2</content>
        <updated>2024-01-02T14:30:00Z</updated>
        <category term="rust"/>
    </entry>
</feed>"#;

        let items = parse_feed(atom, "test_atom").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Atom Entry 1");
        assert_eq!(items[0].url, "https://example.com/atom/1");
        assert_eq!(items[0].tags, vec!["ai", "ml"]);
        assert_eq!(items[1].content, "Full content of entry 2");
        assert_eq!(items[1].tags, vec!["rust"]);
    }

    #[test]
    fn test_parse_empty_feed() {
        let items = parse_feed("not xml", "empty").unwrap_or_default();
        // Graceful parse — no items from invalid XML
        assert!(items.is_empty() || items.len() == 0);
    }

    #[test]
    fn test_parse_rfc2822_date() {
        let dt = parse_rfc2822_date("Mon, 01 Jan 2024 12:00:00 +0000");
        assert!(dt.is_some());
        assert_eq!(dt.unwrap().year(), 2024);
    }

    #[test]
    fn test_parse_iso8601_date() {
        let dt = parse_iso8601_date("2024-01-01T12:00:00Z");
        assert!(dt.is_some());
        assert_eq!(dt.unwrap().year(), 2024);
    }

    #[test]
    fn test_parse_date_empty() {
        assert!(parse_rfc2822_date("").is_none());
        assert!(parse_iso8601_date("").is_none());
        assert!(parse_rfc2822_date("  ").is_none());
    }

    #[test]
    fn test_subscription_item_fields() {
        let item = SubscriptionItem {
            id: "rss1:0".into(),
            title: "Test".into(),
            url: "https://example.com/1".into(),
            content: "body".into(),
            published: None,
            source: "rss1".into(),
            tags: vec!["a".into()],
        };
        assert_eq!(item.id, "rss1:0");
        assert!(item.published.is_none());
    }

    #[test]
    fn test_clone_self() {
        let src = SubscriptionSource::new("rss1", "Test", "https://example.com/feed")
            .with_proxy("http://p:80")
            .with_stealth(StealthLevel::Camofox);
        let cloned = src.clone_self();
        assert_eq!(cloned.id, src.id);
        assert_eq!(cloned.proxy, src.proxy);
        assert_eq!(cloned.stealth_level, StealthLevel::Camofox);
    }
}

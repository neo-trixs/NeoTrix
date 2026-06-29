use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use super::web_navigator::{LoginCredentials, WebNavigator};
use crate::core::nt_core_time::unix_now;

/// 原始推文 — 浏览器抓取的结构化输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTweet {
    pub tweet_id: String,
    pub author: String,
    pub author_handle: String,
    pub text: String,
    pub created_at: u64,
    pub likes: u64,
    pub retweets: u64,
    pub replies: u64,
    pub views: Option<u64>,
    pub url: String,
    pub is_thread: bool,
    pub has_media: bool,
    pub language: Option<String>,
}

/// XTimeline = 按时间倒序的推文列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XTimeline {
    pub tweets: Vec<RawTweet>,
    pub scraped_at: u64,
    pub source: XScrapeSource,
    pub method: XScrapeMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum XScrapeSource {
    HomeTimeline,
    UserTimeline(String),
    Search(String),
    Thread(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum XScrapeMethod {
    WebNavigator,
    ApiFallback,
}

/// NeoTrix 自身 CDP 浏览器 X.com 抓取器
///
/// 使用 WebNavigator (原始 CDP WebSocket)，零外部依赖。
/// 当浏览器不可用时降级到 Twitter API。
pub struct XScraper {
    browser_timeout_secs: u64,
    navigator: Arc<Mutex<Option<WebNavigator>>>,
    logged_in: bool,
}

impl XScraper {
    pub fn new() -> Self {
        Self {
            browser_timeout_secs: 30,
            navigator: Arc::new(Mutex::new(None)),
            logged_in: false,
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.browser_timeout_secs = secs;
        self
    }

    /// 确保浏览器已初始化并登录
    fn ensure_browser(&mut self) -> Result<String, String> {
        let mut guard = self.navigator.lock().map_err(|e| format!("lock: {}", e))?;

        if guard.is_none() {
            let mut nav = WebNavigator::new();
            nav.launch()?;
            let session = nav.new_page()?;
            *guard = Some(nav);
            self.logged_in = false;
            return Ok(session);
        }

        // 已有浏览器，创建新页面
        let nav = guard.as_ref().unwrap();
        nav.new_page()
    }

    /// 登录 X.com
    pub fn login(&mut self, username: &str, password: &str) -> Result<(), String> {
        let guard = self.navigator.lock().map_err(|e| format!("lock: {}", e))?;
        let nav = guard.as_ref().ok_or("browser not launched")?;

        let session_id = nav.new_page()?;

        let credentials = LoginCredentials {
            username: username.to_string(),
            password: password.to_string(),
            wait_for_url: Some("x.com/home".into()),
            ..LoginCredentials::default()
        };

        nav.login_flow(&session_id, "https://x.com/i/flow/login", &credentials)?;

        let _ = nav.evaluate_js(&session_id, "window.close()");

        drop(guard);
        self.logged_in = true;
        Ok(())
    }

    /// 爬取 X.com 首页时间线
    pub fn scrape_home_timeline(&mut self, count: usize) -> Result<XTimeline, String> {
        let session_id = self.ensure_browser()?;
        let guard = self.navigator.lock().map_err(|e| format!("lock: {}", e))?;
        let nav = guard.as_ref().ok_or("no browser")?;

        nav.navigate(&session_id, "https://x.com/home")?;
        tokio::runtime::Handle::current()
            .block_on(tokio::time::sleep(std::time::Duration::from_millis(4000)));

        let js = format!(
            r#"
(async () => {{
    const MAX = {0};
    const tweets = [];
    let lastHeight = 0;

    while (tweets.length < MAX) {{
        const articles = document.querySelectorAll('article[data-testid="tweet"]');
        for (const art of articles) {{
            if (tweets.length >= MAX) break;
            const id = art.querySelector('a[href*="/status/"]')?.href?.split('/status/').pop()?.split('?')[0] || '';
            if (tweets.some(t => t.id === id)) continue;

            const authorEl = art.querySelector('div[data-testid="User-Name"] a');
            const textEl = art.querySelector('div[data-testid="tweetText"]');
            const timeEl = art.querySelector('time');
            const likeEl = art.querySelector('button[data-testid="like"]');
            const retweetEl = art.querySelector('button[data-testid="retweet"]');
            const replyEl = art.querySelector('button[data-testid="reply"]');
            const viewEl = art.querySelector('a[href*="/status/"] ~ a[role="link"]');

            tweets.push({{
                id,
                author: authorEl?.textContent?.split('@')[0]?.trim() || '',
                handle: authorEl?.href?.split('/').pop() || '',
                text: textEl?.textContent || '',
                time: timeEl?.getAttribute('datetime') || '',
                likes: parseInt(likeEl?.getAttribute('aria-label')?.match(/\d+/)?.[0] || '0'),
                retweets: parseInt(retweetEl?.getAttribute('aria-label')?.match(/\d+/)?.[0] || '0'),
                replies: parseInt(replyEl?.getAttribute('aria-label')?.match(/\d+/)?.[0] || '0'),
                views: parseInt(viewEl?.textContent?.replace(/[^0-9]/g, '') || '0') || null,
                url: `https://x.com/i/web/status/${{id}}`,
            }});
        }}

        window.scrollTo(0, document.body.scrollHeight);
        await new Promise(r => setTimeout(r, 2000));

        const newHeight = document.body.scrollHeight;
        if (newHeight === lastHeight) break;
        lastHeight = newHeight;
    }}

    return JSON.stringify(tweets.slice(0, MAX));
}})();
"#,
            count
        );

        let json = nav.evaluate_js(&session_id, &js)?;
        let raw: Vec<serde_json::Value> = serde_json::from_str(json.as_str().unwrap_or("[]"))
            .map_err(|e| format!("parse: {}", e))?;

        let tweets = Self::parse_raw_tweets(&raw);

        Ok(XTimeline {
            tweets,
            scraped_at: unix_now() as u64,
            source: XScrapeSource::HomeTimeline,
            method: XScrapeMethod::WebNavigator,
        })
    }

    /// 爬取指定推文线程
    pub fn scrape_thread(&mut self, tweet_url: &str) -> Result<XTimeline, String> {
        let session_id = self.ensure_browser()?;
        let guard = self.navigator.lock().map_err(|e| format!("lock: {}", e))?;
        let nav = guard.as_ref().ok_or("no browser")?;

        nav.navigate(&session_id, tweet_url)?;
        tokio::runtime::Handle::current()
            .block_on(tokio::time::sleep(std::time::Duration::from_millis(4000)));

        let js = r#"
(async () => {
    const tweets = [];
    const articles = document.querySelectorAll('article[data-testid="tweet"]');

    for (const art of articles) {
        const id = art.querySelector('a[href*="/status/"]')?.href?.split('/status/').pop()?.split('?')[0] || '';
        if (tweets.some(t => t.id === id)) continue;

        const authorEl = art.querySelector('div[data-testid="User-Name"] a');
        const textEl = art.querySelector('div[data-testid="tweetText"]');
        const timeEl = art.querySelector('time');

        tweets.push({
            id,
            author: authorEl?.textContent?.split('@')[0]?.trim() || '',
            handle: authorEl?.href?.split('/').pop() || '',
            text: textEl?.textContent || '',
            time: timeEl?.getAttribute('datetime') || '',
        });
    }

    return JSON.stringify(tweets);
})();
"#;

        let json = nav.evaluate_js(&session_id, js)?;
        let raw: Vec<serde_json::Value> = serde_json::from_str(json.as_str().unwrap_or("[]"))
            .map_err(|e| format!("parse: {}", e))?;

        let tweets = Self::parse_raw_tweets(&raw);

        Ok(XTimeline {
            tweets,
            scraped_at: unix_now() as u64,
            source: XScrapeSource::Thread(tweet_url.to_string()),
            method: XScrapeMethod::WebNavigator,
        })
    }

    /// 搜索 X.com
    pub fn search_x(&mut self, query: &str, count: usize) -> Result<XTimeline, String> {
        let session_id = self.ensure_browser()?;
        let guard = self.navigator.lock().map_err(|e| format!("lock: {}", e))?;
        let nav = guard.as_ref().ok_or("no browser")?;

        let encoded = urlencoding_encode(query);
        let search_url = format!("https://x.com/search?q={}&src=typed_query&f=live", encoded);

        nav.navigate(&session_id, &search_url)?;
        tokio::runtime::Handle::current()
            .block_on(tokio::time::sleep(std::time::Duration::from_millis(4000)));

        let js = format!(
            r#"
(async () => {{
    await new Promise(r => setTimeout(r, 2000));
    const MAX = {};
    const tweets = [];
    const articles = document.querySelectorAll('article[data-testid="tweet"]');

    for (const art of articles) {{
        if (tweets.length >= MAX) break;
        const id = art.querySelector('a[href*="/status/"]')?.href?.split('/status/').pop()?.split('?')[0] || '';
        if (tweets.some(t => t.id === id)) continue;

        const authorEl = art.querySelector('div[data-testid="User-Name"] a');
        const textEl = art.querySelector('div[data-testid="tweetText"]');
        const timeEl = art.querySelector('time');

        tweets.push({{
            id,
            author: authorEl?.textContent?.split('@')[0]?.trim() || '',
            handle: authorEl?.href?.split('/').pop() || '',
            text: textEl?.textContent || '',
            time: timeEl?.getAttribute('datetime') || '',
        }});
    }}

    return JSON.stringify(tweets.slice(0, MAX));
}})();
"#,
            count
        );

        let json = nav.evaluate_js(&session_id, &js)?;
        let raw: Vec<serde_json::Value> = serde_json::from_str(json.as_str().unwrap_or("[]"))
            .map_err(|e| format!("parse: {}", e))?;

        let tweets = Self::parse_raw_tweets(&raw);

        Ok(XTimeline {
            tweets,
            scraped_at: unix_now() as u64,
            source: XScrapeSource::Search(query.to_string()),
            method: XScrapeMethod::WebNavigator,
        })
    }

    /// 通用导航 — 用户输入网址，意识体跳转并提取内容
    pub fn navigate_and_extract(&mut self, url: &str) -> Result<String, String> {
        let session_id = self.ensure_browser()?;
        let guard = self.navigator.lock().map_err(|e| format!("lock: {}", e))?;
        let nav = guard.as_ref().ok_or("no browser")?;

        nav.navigate(&session_id, url)?;

        let js = r#"
(function() {
    const title = document.title;
    const text = document.body?.innerText || '';
    const links = Array.from(document.querySelectorAll('a[href]')).map(a => a.href).slice(0, 50);
    const meta = {};
    document.querySelectorAll('meta[name], meta[property]').forEach(m => {
        const k = m.getAttribute('name') || m.getAttribute('property') || '';
        const v = m.getAttribute('content') || '';
        if (k && v) meta[k] = v;
    });
    return JSON.stringify({ title, text: text.slice(0, 10000), links, meta });
})();
"#;

        let result = nav.evaluate_js(&session_id, js)?;
        Ok(result.as_str().unwrap_or("{}").to_string())
    }

    fn parse_raw_tweets(raw: &[serde_json::Value]) -> Vec<RawTweet> {
        raw.iter()
            .map(|v| RawTweet {
                tweet_id: v["id"].as_str().unwrap_or("").to_string(),
                author: v["author"].as_str().unwrap_or("").to_string(),
                author_handle: v["handle"].as_str().unwrap_or("").to_string(),
                text: v["text"].as_str().unwrap_or("").to_string(),
                created_at: parse_x_timestamp(v["time"].as_str().unwrap_or("")),
                likes: v["likes"].as_u64().unwrap_or(0),
                retweets: v["retweets"].as_u64().unwrap_or(0),
                replies: v["replies"].as_u64().unwrap_or(0),
                views: v["views"].as_u64(),
                url: format!(
                    "https://x.com/i/web/status/{}",
                    v["id"].as_str().unwrap_or("")
                ),
                is_thread: false,
                has_media: false,
                language: None,
            })
            .collect()
    }
}

impl Default for XScraper {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse X.com datetime string to unix timestamp
fn parse_x_timestamp(s: &str) -> u64 {
    if s.is_empty() {
        return 0;
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        dt.timestamp() as u64
    } else {
        0
    }
}

/// Simple URL encoding
fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x_scraper_construction() {
        let s = XScraper::new();
        assert_eq!(s.browser_timeout_secs, 30);
    }

    #[test]
    fn test_with_timeout() {
        let s = XScraper::new().with_timeout(60);
        assert_eq!(s.browser_timeout_secs, 60);
    }

    #[test]
    fn test_parse_x_timestamp_empty() {
        assert_eq!(parse_x_timestamp(""), 0);
    }

    #[test]
    fn test_parse_x_timestamp_valid() {
        let ts = parse_x_timestamp("2026-06-10T12:00:00Z");
        assert!(ts > 0);
    }

    #[test]
    fn test_raw_tweet_struct() {
        let t = RawTweet {
            tweet_id: "123".into(),
            author: "Alice".into(),
            author_handle: "alice".into(),
            text: "Hello".into(),
            created_at: 1000,
            likes: 5,
            retweets: 2,
            replies: 1,
            views: Some(100),
            url: "https://x.com/i/web/status/123".into(),
            is_thread: false,
            has_media: false,
            language: Some("en".into()),
        };
        assert_eq!(t.tweet_id, "123");
    }

    #[test]
    fn test_xtimeline_construction() {
        let tl = XTimeline {
            tweets: vec![],
            scraped_at: 1000,
            source: XScrapeSource::HomeTimeline,
            method: XScrapeMethod::WebNavigator,
        };
        assert!(tl.tweets.is_empty());
    }

    #[test]
    fn test_urlencoding_encode() {
        assert_eq!(urlencoding_encode("hello world"), "hello+world");
        assert_eq!(urlencoding_encode("a&b"), "a%26b");
    }

    #[test]
    fn test_parse_raw_tweets() {
        let raw: Vec<serde_json::Value> = serde_json::from_str(r#"[{"id":"1","author":"Alice","handle":"alice","text":"Hello","time":"2026-06-10T12:00:00Z"}]"#).unwrap();
        let tweets = XScraper::parse_raw_tweets(&raw);
        assert_eq!(tweets.len(), 1);
        assert_eq!(tweets[0].tweet_id, "1");
        assert_eq!(tweets[0].author_handle, "alice");
    }
}

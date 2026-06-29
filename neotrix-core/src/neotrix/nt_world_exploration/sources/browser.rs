use crate::neotrix::nt_world_exploration::content::{
    Engagement, ExplorationSourceType, SourceContent,
};
use crate::neotrix::nt_world_exploration::source_trait::ExplorationSource;

/// 浏览器探索源 — 通过 CDP 自动化驱动
///
/// 当前支持: X.com 时间线爬取
/// 可扩展: Reddit, YT, TikTok (通过 WebNavigator)
pub struct BrowserSource {
    pub url: String,
    pub connected: bool,
    tweet_count: usize,
    pub pending: Vec<SourceContent>,
}

impl BrowserSource {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            connected: false,
            tweet_count: 0,
            pending: Vec::new(),
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        self.connected = true;
        Ok(())
    }

    pub fn inject_content(&mut self, items: Vec<SourceContent>) {
        self.pending.extend(items);
    }

    /// JS 提取推文 — 从页面提取标准化源内容
    pub fn extract_tweets_from_page(js_result: &str) -> Vec<SourceContent> {
        let raw: Vec<serde_json::Value> = serde_json::from_str(js_result).unwrap_or_default();

        raw.iter()
            .map(|v| {
                let id = v["id"].as_str().unwrap_or("").to_string();
                let text = v["text"].as_str().unwrap_or("").to_string();
                let author = v["author"].as_str().unwrap_or("").to_string();
                let handle = v["handle"].as_str().unwrap_or("").to_string();

                SourceContent::new(id, text, ExplorationSourceType::BrowserSocial)
                    .with_author(handle)
                    .with_title(format!("@{}", author))
                    .with_url(format!(
                        "https://x.com/i/web/status/{}",
                        v["id"].as_str().unwrap_or("")
                    ))
                    .with_engagement(Engagement {
                        likes: v["likes"].as_u64().unwrap_or(0),
                        shares: v["retweets"].as_u64().unwrap_or(0),
                        replies: v["replies"].as_u64().unwrap_or(0),
                        views: v["views"].as_u64(),
                    })
            })
            .collect()
    }
}

impl ExplorationSource for BrowserSource {
    fn name(&self) -> &'static str {
        "browser"
    }

    fn confidence(&self) -> f64 {
        0.7
    }

    fn explore(&mut self) -> Result<Vec<SourceContent>, String> {
        if !self.connected {
            return Err("Browser not connected. Call connect() first.".into());
        }
        let results = std::mem::take(&mut self.pending);
        self.tweet_count = self.pending.len();
        Ok(results)
    }

    fn is_ready(&self) -> bool {
        self.connected
    }

    fn pending_count(&self) -> usize {
        self.tweet_count + self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_content_stores_items() {
        let mut src = BrowserSource::new("https://x.com");
        let items = vec![
            SourceContent::new("1", "tweet text", ExplorationSourceType::BrowserSocial),
            SourceContent::new("2", "another tweet", ExplorationSourceType::BrowserSocial),
        ];
        src.inject_content(items);
        assert_eq!(src.pending.len(), 2);
    }

    #[test]
    fn test_explore_drains_pending() {
        let mut src = BrowserSource::new("https://x.com");
        src.connect().unwrap();
        src.inject_content(vec![SourceContent::new(
            "1",
            "tweet",
            ExplorationSourceType::BrowserSocial,
        )]);
        let results = src.explore().unwrap();
        assert_eq!(results.len(), 1);
        assert!(src.pending.is_empty());
    }

    #[test]
    fn test_explore_fails_if_not_connected() {
        let mut src = BrowserSource::new("https://x.com");
        let result = src.explore();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not connected"));
    }

    #[test]
    fn test_extract_tweets_from_page_valid_json() {
        let json = r#"[
            {"id":"123","text":"Hello world","author":"Alice","handle":"@alice","likes":5,"retweets":2,"replies":1,"views":100},
            {"id":"456","text":"Another tweet","author":"Bob","handle":"@bob","likes":10,"retweets":3,"replies":0,"views":200}
        ]"#;
        let results = BrowserSource::extract_tweets_from_page(json);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "123");
        assert_eq!(results[1].author.as_deref(), Some("@bob"));
    }

    #[test]
    fn test_extract_tweets_from_page_invalid_json() {
        let results = BrowserSource::extract_tweets_from_page("not json");
        assert!(results.is_empty());
    }
}

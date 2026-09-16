use super::super::{
    ExtractorItem, ExtractorResult, FeedItem, EngagementMetrics,
    SessionEntry, SocialAccessError, SocialAccessResult,
};
use super::super::traits::{SocialPlatformAdapter, FeedResult, TrendingTopic, AuthFlow};

pub struct TwitterExtractor;

impl TwitterExtractor {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_user_tweets(
        &self,
        _session: &SessionEntry,
        username: &str,
        limit: usize,
    ) -> SocialAccessResult<ExtractorResult> {
        let url = format!(
            "https://api.fxtwitter.com/search?q={}&f=jpeg",
            urlencoding::encode(&format!("from:{}", username))
        );
        let resp = reqwest::Client::new()
            .get(&url)
            .header("User-Agent", "NeoTrix/1.0")
            .send().await
.map_err(|e| SocialAccessError::Network(e.to_string()))?;
        let json: serde_json::Value = resp
            .json().await
.map_err(|e| SocialAccessError::Parse(e.to_string()))?;
        let tweets = json["tweets"]
            .as_array()
            .ok_or_else(|| SocialAccessError::Parse("No tweets array".into()))?;
        let items: Vec<ExtractorItem> = tweets
            .iter()
            .take(limit)
            .filter_map(|t| {
                let id = t["id"].as_str()?.to_string();
                let text = t["text"].as_str().unwrap_or("").to_string();
                let author = t["author"]["name"].as_str()?.to_string();
                let photo = t["media"]["photos"][0]["url"].as_str().map(String::from);
                Some(ExtractorItem {
                    id,
                    title: text.chars().take(100).collect(),
                    author,
                    url: None,
                    thumbnail: photo,
                })
            })
            .collect();
        let total = items.len();
        Ok(ExtractorResult {
            items,
            total,
            source: "twitter".into(),
        })
    }

    pub async fn search(
        &self,
        _session: &SessionEntry,
        query: &str,
        _page: u32,
    ) -> SocialAccessResult<ExtractorResult> {
        let q = query.to_string();
        let url = format!(
            "https://api.fxtwitter.com/search?q={}&f=jpeg",
            urlencoding::encode(&q)
        );
        let resp = reqwest::Client::new()
            .get(&url)
            .header("User-Agent", "NeoTrix/1.0")
            .send().await
.map_err(|e| SocialAccessError::Network(e.to_string()))?;
        let json: serde_json::Value = resp
            .json().await
.map_err(|e| SocialAccessError::Parse(e.to_string()))?;
        let tweets = json["tweets"]
            .as_array()
            .ok_or_else(|| SocialAccessError::Parse("No results".into()))?;
        let items: Vec<ExtractorItem> = tweets
            .iter()
            .filter_map(|t| {
                let id = t["id"].as_str()?.to_string();
                let text = t["text"].as_str().unwrap_or("").to_string();
                let author = t["author"]["name"].as_str()?.to_string();
                let photo = t["media"]["photos"][0]["url"].as_str().map(String::from);
                Some(ExtractorItem {
                    id,
                    title: text.chars().take(100).collect(),
                    author,
                    url: None,
                    thumbnail: photo,
                })
            })
            .collect();
        let total = tweets.len();
        Ok(ExtractorResult {
            items,
            total,
            source: "twitter".into(),
        })
    }
}

impl SocialPlatformAdapter for TwitterExtractor {
    fn id(&self) -> String { "twitter".into() }
    fn name(&self) -> &'static str { "Twitter" }
    fn auth_flow(&self) -> AuthFlow { AuthFlow::OAuth2PKCE }
    fn api_base_url(&self) -> &'static str { "https://twitter.com" }
    fn channel_name(&self) -> Option<&'static str> { Some("twitter") }

    fn get_recommended(&self, _session: &SessionEntry, _limit: usize) -> Result<FeedResult, SocialAccessError> {
        Ok(FeedResult { items: vec![], total: 0 })
    }
    fn get_following(&self, _session: &SessionEntry, _limit: usize) -> Result<FeedResult, SocialAccessError> {
        Ok(FeedResult { items: vec![], total: 0 })
    }
    fn get_trending(&self) -> Result<Vec<TrendingTopic>, SocialAccessError> {
        Ok(vec![])
    }

    fn search(&self, _session: &SessionEntry, query: &str, limit: usize) -> Result<FeedResult, SocialAccessError> {
        // Use the async search internally but return sync result
        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            self.search(_session, query, 0).await
        });

        match result {
            Ok(extractor_result) => {
                let items: Vec<FeedItem> = extractor_result.items.into_iter().map(|item| {
                    FeedItem {
                        id: item.id,
                        content: item.title,
                        author: item.author,
                        metrics: EngagementMetrics::default(),
                        score: 0.0,
                        actions: std::collections::HashMap::new(),
                    }
                }).take(limit).collect();
                let total = items.len();
                Ok(FeedResult { items, total })
            }
            Err(e) => Err(e),
        }
    }
}

//! Jina Reader Extractor — zero-API-fee web content extraction
//!
//! Uses Jina Reader (https://r.jina.ai/) for free web scraping.
//! No API key required for basic usage.

use super::super::traits::*;
use super::super::SocialAccessError;

pub struct JinaReaderExtractor;

impl JinaReaderExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Read a URL via Jina Reader
    pub async fn read_url(&self, url: &str) -> Result<ExtractorResult, SocialAccessError> {
        let jina_url = format!("https://r.jina.ai/{}", url);

        let resp = reqwest::Client::new()
            .get(&jina_url)
            .header("User-Agent", "NeoTrix/1.0")
            .header("Accept", "text/plain")
            .send()
            .await
            .map_err(|e| SocialAccessError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(SocialAccessError::Network(format!(
                "Jina Reader returned {}",
                resp.status()
            )));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| SocialAccessError::Network(e.to_string()))?;

        // Extract title from first line or URL
        let title = text
            .lines()
            .next()
            .unwrap_or(url)
            .trim()
            .to_string();

        let item = ExtractorItem {
            id: url.to_string(),
            title,
            author: url.to_string(),
            url: Some(url.to_string()),
            thumbnail: None,
        };

        Ok(ExtractorResult {
            items: vec![item],
            total: 1,
            source: "jina-reader".into(),
        })
    }

    /// Search the web via Jina Reader
    pub async fn search(&self, query: &str, limit: usize) -> Result<ExtractorResult, SocialAccessError> {
        let search_url = format!("https://s.jina.ai/{}", urlencoding::encode(query));

        let resp = reqwest::Client::new()
            .get(&search_url)
            .header("User-Agent", "NeoTrix/1.0")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| SocialAccessError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(SocialAccessError::Network(format!(
                "Jina Search returned {}",
                resp.status()
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SocialAccessError::Parse(e.to_string()))?;

        let results = json["results"]
            .as_array()
            .ok_or_else(|| SocialAccessError::Parse("No results array".into()))?;

        let items: Vec<ExtractorItem> = results
            .iter()
            .take(limit)
            .filter_map(|r| {
                let url = r["url"].as_str()?.to_string();
                let title = r["title"].as_str().unwrap_or("Untitled").to_string();
                let snippet = r["snippet"].as_str().unwrap_or("").to_string();

                Some(ExtractorItem {
                    id: url.clone(),
                    title: if snippet.is_empty() { title } else { format!("{} — {}", title, snippet) },
                    author: r["site"].as_str().unwrap_or("unknown").to_string(),
                    url: Some(url),
                    thumbnail: None,
                })
            })
            .collect();

        let total = items.len();
        Ok(ExtractorResult {
            items,
            total,
            source: "jina-search".into(),
        })
    }
}

impl SocialPlatformAdapter for JinaReaderExtractor {
    fn id(&self) -> PlatformId { "web".into() }
    fn name(&self) -> &'static str { "Web (Jina Reader)" }
    fn auth_flow(&self) -> AuthFlow { AuthFlow::OAuth2PKCE }
    fn api_base_url(&self) -> &'static str { "https://r.jina.ai" }
    fn channel_name(&self) -> Option<&'static str> { Some("web") }

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
        let rt = tokio::runtime::Handle::current();
        let query = query.to_string();
        let result = rt.block_on(async {
            self.search(&query, limit).await
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
                }).collect();
                let total = items.len();
                Ok(FeedResult { items, total })
            }
            Err(e) => Err(e),
        }
    }

    fn get_from_url(&self, _session: &SessionEntry, url: &str) -> Result<ExtractorResult, SocialAccessError> {
        let rt = tokio::runtime::Handle::current();
        let url = url.to_string();
        rt.block_on(async {
            self.read_url(&url).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_creation() {
        let _ext = JinaReaderExtractor::new();
    }
}

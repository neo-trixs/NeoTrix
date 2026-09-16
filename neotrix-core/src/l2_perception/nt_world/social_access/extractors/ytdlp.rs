//! YTDLP Extractor — YouTube 视频/音频提取
//!
//! 通过 yt-dlp CLI 获取 YouTube 视频元数据。

use crate::l2_perception::nt_world::social_access::traits::*;
use crate::l2_perception::nt_world::social_access::SocialAccessError;

pub struct YtdlpExtractor;

impl YtdlpExtractor {
    pub fn new() -> Self { Self }

    /// 通过 yt-dlp 获取视频信息
    pub async fn get_video_info(
        &self,
        _session: &SessionEntry,
        url: &str,
    ) -> Result<ExtractorResult, SocialAccessError> {
        let output = tokio::process::Command::new("yt-dlp")
            .args(["--dump-json", "--no-playlist", url])
            .output().await
            
            .map_err(|e| SocialAccessError::Network(e.to_string()))?;

        if !output.status.success() {
            return Err(SocialAccessError::Network(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| SocialAccessError::Parse(e.to_string()))?;
        let title = json["title"].as_str().unwrap_or("").to_string();
        let id = json["id"].as_str().unwrap_or("").to_string();
        let author = json["uploader"].as_str().unwrap_or("").to_string();

        let item = ExtractorItem {
            id,
            title,
            author,
            url: json["url"].as_str().map(|s| s.to_string()),
            thumbnail: json["thumbnail"].as_str().map(|s| s.to_string()),
        };

        Ok(ExtractorResult {
            items: vec![item],
            total: 1,
            source: "youtube".into(),
        })
    }
}

impl SocialPlatformAdapter for YtdlpExtractor {
    fn id(&self) -> PlatformId { "youtube".into() }
    fn name(&self) -> &'static str { "YouTube" }
    fn auth_flow(&self) -> AuthFlow { AuthFlow::OAuth2PKCE }
    fn api_base_url(&self) -> &'static str { "https://www.youtube.com" }
    fn channel_name(&self) -> Option<&'static str> { Some("youtube") }

    fn get_recommended(
        &self,
        _session: &SessionEntry,
        _limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        let items = Vec::new();
        Ok(FeedResult { items, total: 0 })
    }
    fn get_following(
        &self,
        _session: &SessionEntry,
        _limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        let items = Vec::new();
        Ok(FeedResult { items, total: 0 })
    }
    fn get_trending(&self) -> Result<Vec<TrendingTopic>, SocialAccessError> {
        Ok(vec![
            TrendingTopic { name: "Trending on YouTube".to_string(), volume: 0 },
        ])
    }

    fn search(&self, _session: &SessionEntry, query: &str, _limit: usize) -> Result<FeedResult, SocialAccessError> {
        // Search YouTube via yt-dlp
        let rt = tokio::runtime::Handle::current();
        let query = query.to_string();
        let result = rt.block_on(async {
            let output = tokio::process::Command::new("yt-dlp")
                .args(["--dump-json", "--flat-playlist", &format!("ytsearch:{}", query)])
                .output()
                .await
                .map_err(|e| SocialAccessError::Network(e.to_string()))?;

            if !output.status.success() {
                return Err(SocialAccessError::Network(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ));
            }

            let mut items = Vec::new();
            for line in output.stdout.split(|&b| b == b'\n') {
                if line.is_empty() { continue; }
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(line) {
                    items.push(FeedItem {
                        id: json["id"].as_str().unwrap_or("").to_string(),
                        content: json["title"].as_str().unwrap_or("").to_string(),
                        author: json["uploader"].as_str().unwrap_or("").to_string(),
                        metrics: EngagementMetrics::default(),
                        score: 0.0,
                        actions: std::collections::HashMap::new(),
                    });
                }
            }

            Ok(FeedResult { total: items.len(), items })
        });

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_creation() {
        let _ext = YtdlpExtractor::new();
    }
}

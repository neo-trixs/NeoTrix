use crate::neotrix::nt_act_social::auth::SocialAuth;
use crate::neotrix::nt_act_social::connector::{
    Platform, PlatformError, SearchResult, SocialMediaPlatform, UserInfo, VideoInfo,
};
use crate::neotrix::nt_act_social::rate_limit::RateLimiter;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
struct YouTubeSearchResponse {
    items: Option<Vec<YouTubeSearchItem>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    #[serde(rename = "pageInfo")]
    page_info: Option<YouTubePageInfo>,
}

#[derive(Deserialize)]
struct YouTubePageInfo {
    #[serde(rename = "totalResults")]
    total_results: Option<u64>,
}

#[derive(Deserialize)]
struct YouTubeSearchItem {
    id: YouTubeResourceId,
    snippet: Option<YouTubeSnippet>,
}

#[derive(Deserialize)]
struct YouTubeResourceId {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Deserialize)]
struct YouTubeVideoResponse {
    items: Option<Vec<YouTubeVideoItem>>,
}

#[derive(Deserialize)]
struct YouTubeVideoItem {
    id: String,
    snippet: Option<YouTubeSnippet>,
    statistics: Option<YouTubeStatistics>,
    #[serde(rename = "contentDetails")]
    content_details: Option<YouTubeContentDetails>,
}

#[derive(Deserialize)]
struct YouTubeSnippet {
    title: String,
    description: String,
    thumbnails: Option<YouTubeThumbnails>,
    #[serde(rename = "channelTitle")]
    channel_title: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
}

#[derive(Deserialize)]
struct YouTubeThumbnails {
    high: Option<YouTubeThumbnail>,
    medium: Option<YouTubeThumbnail>,
    #[serde(rename = "default")]
    default: Option<YouTubeThumbnail>,
}

#[derive(Deserialize)]
struct YouTubeThumbnail {
    url: String,
}

#[derive(Deserialize)]
struct YouTubeStatistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
    #[serde(rename = "likeCount")]
    like_count: Option<String>,
    #[serde(rename = "commentCount")]
    comment_count: Option<String>,
}

#[derive(Deserialize)]
struct YouTubeContentDetails {
    duration: Option<String>,
}

pub struct YouTubeConnector {
    api_key: String,
    client: reqwest::Client,
    auth: SocialAuth,
    rate_limiter: RateLimiter,
}

impl YouTubeConnector {
    pub fn new(api_key: String) -> Self {
        YouTubeConnector {
            api_key,
            client: reqwest::Client::new(),
            auth: SocialAuth::new(),
            rate_limiter: RateLimiter::new(),
        }
    }

    fn thumbnail_url(thumbnails: &Option<YouTubeThumbnails>) -> Option<String> {
        thumbnails
            .as_ref()
            .and_then(|t| t.high.as_ref().or(t.medium.as_ref()).or(t.default.as_ref()))
            .map(|t| t.url.clone())
    }

    fn parse_published_at(s: &Option<String>) -> Option<DateTime<Utc>> {
        s.as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
    }

    fn parse_duration_iso8601(duration: &Option<String>) -> Option<u64> {
        let s = duration.as_deref()?;
        if !s.starts_with('P') {
            return None;
        }
        let mut total = 0u64;
        let mut accum = 0u64;
        let mut in_time = false;
        let chars: Vec<char> = s.chars().collect();
        let mut i = 1;
        while i < chars.len() {
            match chars[i] {
                'T' => {
                    in_time = true;
                    i += 1;
                }
                'D' => {
                    total += accum * 86400;
                    accum = 0;
                    i += 1;
                }
                'H' => {
                    total += accum * 3600;
                    accum = 0;
                    i += 1;
                }
                'M' => {
                    if in_time {
                        total += accum * 60;
                    } else {
                        total += accum * 86400 * 30;
                    }
                    accum = 0;
                    i += 1;
                }
                'S' => {
                    total += accum;
                    accum = 0;
                    i += 1;
                }
                c if c.is_ascii_digit() => {
                    accum = accum * 10 + (c as u64 - '0' as u64);
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
        Some(total)
    }

    fn snippet_to_video_info(snippet: &YouTubeSnippet, id: String) -> VideoInfo {
        VideoInfo {
            url: format!("https://www.youtube.com/watch?v={}", id),
            title: snippet.title.clone(),
            description: snippet.description.clone(),
            thumbnail_url: Self::thumbnail_url(&snippet.thumbnails),
            published_at: Self::parse_published_at(&snippet.published_at),
            author: snippet.channel_title.clone(),
            view_count: None,
            like_count: None,
            comment_count: None,
            duration_secs: None,
            platform: Platform::YouTube,
            id,
        }
    }

    async fn send_request(
        &self,
        url: &str,
        params: Vec<(&str, String)>,
    ) -> Result<reqwest::Response, PlatformError> {
        let wait = self.rate_limiter.wait_if_needed("youtube");
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }

        let mut req = self.client.get(url);
        for (k, v) in &params {
            req = req.query(&[(k, v.as_str())]);
        }
        req = req.query(&[("key", &self.api_key)]);

        let resp = req.send().await.map_err(|e| PlatformError::Network(e.to_string()))?;

        self.rate_limiter.record_call("youtube");

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            if status.as_u16() == 429 || status.as_u16() == 403 {
                return Err(PlatformError::RateLimited { retry_after: None });
            }
            return Err(PlatformError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        Ok(resp)
    }
}

#[async_trait]
impl SocialMediaPlatform for YouTubeConnector {
    fn platform(&self) -> Platform {
        Platform::YouTube
    }

    fn is_authenticated(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn login(&mut self) -> Result<(), PlatformError> {
        if self.api_key.is_empty() {
            return Err(PlatformError::AuthFailed("API key is empty".into()));
        }
        if self.auth.is_token_valid("youtube") {
            return Ok(());
        }
        let test_url = format!(
            "https://www.googleapis.com/youtube/v3/videos?part=snippet&id=dQw4w9WgXcQ&key={}",
            self.api_key
        );
        let resp = self
            .client
            .get(&test_url)
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlatformError::ApiError {
                status,
                message: body,
            });
        }
        Ok(())
    }

    async fn search(&self, query: &str, max_results: u32) -> Result<SearchResult, PlatformError> {
        let max = max_results.min(50);
        let resp = self
            .send_request(
                "https://www.googleapis.com/youtube/v3/search",
                vec![
                    ("part", "snippet".into()),
                    ("q", query.to_string()),
                    ("maxResults", max.to_string()),
                    ("type", "video".into()),
                ],
            )
            .await?;

        let body: YouTubeSearchResponse = resp
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        let videos = body
            .items
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                let video_id = item.id.video_id?;
                let snippet = item.snippet?;
                Some(Self::snippet_to_video_info(&snippet, video_id))
            })
            .collect();

        Ok(SearchResult {
            videos,
            next_page_token: body.next_page_token,
            total_estimated: body.page_info.and_then(|p| p.total_results),
        })
    }

    async fn get_video(&self, video_id: &str) -> Result<VideoInfo, PlatformError> {
        let resp = self
            .send_request(
                "https://www.googleapis.com/youtube/v3/videos",
                vec![
                    ("part", "snippet,statistics,contentDetails".into()),
                    ("id", video_id.to_string()),
                ],
            )
            .await?;

        let body: YouTubeVideoResponse = resp
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        let item = body
            .items
            .and_then(|mut items| {
                if items.is_empty() {
                    None
                } else {
                    Some(items.remove(0))
                }
            })
            .ok_or_else(|| {
                PlatformError::ApiError {
                    status: 404,
                    message: format!("video {} not found", video_id),
                }
            })?;

        let mut info = if let Some(ref snippet) = item.snippet {
            Self::snippet_to_video_info(snippet, item.id.clone())
        } else {
            let vid_id = item.id;
            VideoInfo {
                id: vid_id.clone(),
                url: format!("https://www.youtube.com/watch?v={}", vid_id),
                title: String::new(),
                description: String::new(),
                platform: Platform::YouTube,
                thumbnail_url: None,
                duration_secs: None,
                view_count: None,
                like_count: None,
                comment_count: None,
                author: None,
                published_at: None,
            }
        };

        if let Some(ref stats) = item.statistics {
            info.view_count = stats.view_count.as_ref().and_then(|v| v.parse().ok());
            info.like_count = stats.like_count.as_ref().and_then(|v| v.parse().ok());
            info.comment_count = stats.comment_count.as_ref().and_then(|v| v.parse().ok());
        }

        if let Some(ref cd) = item.content_details {
            info.duration_secs = Self::parse_duration_iso8601(&cd.duration);
        }

        Ok(info)
    }

    async fn trending(&self, max_results: u32) -> Result<Vec<VideoInfo>, PlatformError> {
        let max = max_results.min(50);
        let resp = self
            .send_request(
                "https://www.googleapis.com/youtube/v3/videos",
                vec![
                    ("part", "snippet,statistics".into()),
                    ("chart", "mostPopular".into()),
                    ("regionCode", "US".into()),
                    ("maxResults", max.to_string()),
                ],
            )
            .await?;

        let body: YouTubeVideoResponse = resp
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        let videos = body
            .items
            .unwrap_or_default()
            .into_iter()
            .map(|item| {
                let mut info = if let Some(ref snippet) = item.snippet {
                    Self::snippet_to_video_info(snippet, item.id.clone())
                } else {
                    VideoInfo {
                        id: item.id.clone(),
                        url: format!("https://www.youtube.com/watch?v={}", item.id),
                        title: String::new(),
                        description: String::new(),
                        platform: Platform::YouTube,
                        thumbnail_url: None,
                        duration_secs: None,
                        view_count: None,
                        like_count: None,
                        comment_count: None,
                        author: None,
                        published_at: None,
                    }
                };
                if let Some(ref stats) = item.statistics {
                    info.view_count = stats.view_count.as_ref().and_then(|v| v.parse().ok());
                    info.like_count = stats.like_count.as_ref().and_then(|v| v.parse().ok());
                    info.comment_count = stats.comment_count.as_ref().and_then(|v| v.parse().ok());
                }
                info
            })
            .collect();

        Ok(videos)
    }

    async fn get_me(&self) -> Result<UserInfo, PlatformError> {
        Err(PlatformError::NotAuthenticated)
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}

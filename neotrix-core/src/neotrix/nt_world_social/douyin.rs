use async_trait::async_trait;
use chrono::DateTime;
use serde::Deserialize;

use crate::core::nt_core_agent::UserAgentRotation;
use crate::neotrix::nt_world_social::auth::SocialAuth;
use crate::neotrix::nt_world_social::connector::{
    Platform, PlatformError, SocialMediaPlatform, SocialSearchResult, UserInfo, VideoInfo,
};
use crate::neotrix::nt_world_social::rate_limit::RateLimiter;
const WEB_BASE: &str = "https://www.douyin.com";
const OPEN_BASE: &str = "https://open.douyin.com";

#[derive(Deserialize)]
struct DouyinAweme {
    aweme_id: String,
    desc: String,
    #[serde(default)]
    video: Option<DouyinVideoInfo>,
    #[serde(default)]
    statistics: Option<DouyinStatistics>,
    #[serde(default)]
    author: Option<DouyinAuthor>,
    create_time: i64,
}

#[derive(Deserialize)]
struct DouyinVideoInfo {
    #[serde(default)]
    cover: Option<DouyinCover>,
    duration: i64,
}

#[derive(Deserialize)]
struct DouyinCover {
    #[serde(default)]
    url_list: Vec<String>,
}

#[derive(Deserialize)]
struct DouyinStatistics {
    view_count: i64,
    digg_count: i64,
    comment_count: i64,
}

#[derive(Deserialize)]
struct DouyinAuthor {
    nickname: String,
}

#[derive(Deserialize)]
struct DouyinSearchData {
    #[serde(default)]
    aweme_list: Vec<DouyinAweme>,
}

#[derive(Deserialize)]
struct DouyinSearchResponse {
    status_code: i64,
    #[serde(default)]
    data: Option<DouyinSearchData>,
}

#[derive(Deserialize)]
struct DouyinDetailResponse {
    status_code: i64,
    #[serde(default)]
    aweme_detail: Option<DouyinAweme>,
}

#[derive(Deserialize)]
struct DouyinRecommendResponse {
    status_code: i64,
    #[serde(default)]
    aweme_list: Vec<DouyinAweme>,
}

pub struct DouyinConnector {
    client: reqwest::Client,
    auth: SocialAuth,
    rate_limiter: RateLimiter,
    open_api_key: Option<String>,
    web_cookie: Option<String>,
}

impl Default for DouyinConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl DouyinConnector {
    pub fn new() -> Self {
        DouyinConnector {
            client: reqwest::Client::new(),
            auth: SocialAuth::new(),
            rate_limiter: RateLimiter::new(),
            open_api_key: None,
            web_cookie: None,
        }
    }

    fn req(&self, url: String) -> reqwest::RequestBuilder {
        let r = self
            .client
            .get(&url)
            .header("Referer", "https://www.douyin.com")
            .header("User-Agent", UserAgentRotation::default().next());
        if let Some(c) = &self.web_cookie {
            r.header("Cookie", format!("sessionid={}", c))
        } else {
            r
        }
    }

    async fn fetch(&self, builder: reqwest::RequestBuilder) -> Result<Vec<u8>, PlatformError> {
        let wait = self.rate_limiter.wait_if_needed("douyin");
        if wait > std::time::Duration::ZERO {
            tokio::time::sleep(wait).await;
        }
        self.rate_limiter.record_call("douyin");
        let resp = builder
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;
        let status = resp.status();
        let body = resp
            .bytes()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(PlatformError::ApiError {
                status: status.as_u16(),
                message: String::from_utf8_lossy(&body).to_string(),
            });
        }
        Ok(body.to_vec())
    }

    fn aweme_to_video(aweme: DouyinAweme) -> VideoInfo {
        let thumbnail_url = aweme
            .video
            .as_ref()
            .and_then(|v| v.cover.as_ref())
            .and_then(|c| c.url_list.first().cloned());
        let duration_secs = aweme.video.as_ref().map(|v| (v.duration / 1000) as u64);
        let view_count = aweme.statistics.as_ref().map(|s| s.view_count as u64);
        let like_count = aweme.statistics.as_ref().map(|s| s.digg_count as u64);
        let comment_count = aweme.statistics.as_ref().map(|s| s.comment_count as u64);
        let author = aweme.author.as_ref().map(|a| a.nickname.clone());
        VideoInfo {
            id: aweme.aweme_id.clone(),
            title: aweme.desc.clone(),
            description: aweme.desc,
            platform: Platform::Douyin,
            url: format!("https://www.douyin.com/video/{}", aweme.aweme_id),
            thumbnail_url,
            duration_secs,
            view_count,
            like_count,
            comment_count,
            author,
            published_at: DateTime::from_timestamp(aweme.create_time, 0),
        }
    }
}

#[async_trait]
impl SocialMediaPlatform for DouyinConnector {
    fn platform(&self) -> Platform {
        Platform::Douyin
    }

    fn is_authenticated(&self) -> bool {
        self.web_cookie.is_some() || self.open_api_key.is_some()
    }

    async fn login(&mut self) -> Result<(), PlatformError> {
        if let Some(t) = self.auth.get_token("douyin") {
            self.web_cookie = Some(t.access_token);
            self.open_api_key = t.scope;
            return Ok(());
        }
        if let Some(t) = self.auth.get_token("douyin_open") {
            self.open_api_key = Some(t.access_token);
            return Ok(());
        }
        Err(PlatformError::NotAuthenticated)
    }

    async fn search(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<SocialSearchResult, PlatformError> {
        if self.web_cookie.is_some() {
            let count = max_results.to_string();
            let builder = self
                .req(format!("{}/aweme/v1/web/search/item/", WEB_BASE))
                .query(&[("keyword", query), ("count", count.as_str()), ("type", "1")]);
            let body = self.fetch(builder).await?;
            let resp: DouyinSearchResponse = serde_json::from_slice(&body)
                .map_err(|e| PlatformError::ParseError(e.to_string()))?;
            if resp.status_code != 0 {
                return Err(PlatformError::ApiError {
                    status: 200,
                    message: format!("douyin search error code: {}", resp.status_code),
                });
            }
            let items = resp.data.map(|d| d.aweme_list).unwrap_or_default();
            let videos: Vec<VideoInfo> = items
                .into_iter()
                .map(DouyinConnector::aweme_to_video)
                .collect();
            return Ok(SocialSearchResult {
                videos,
                next_page_token: None,
                total_estimated: None,
            });
        }

        if let Some(api_key) = &self.open_api_key {
            let builder = self
                .req(format!("{}/api/douyin/v1/search/", OPEN_BASE))
                .query(&[("keyword", query), ("count", &max_results.to_string())])
                .header("access-token", api_key);
            let body = self.fetch(builder).await?;
            let resp: DouyinSearchResponse = serde_json::from_slice(&body)
                .map_err(|e| PlatformError::ParseError(e.to_string()))?;
            if resp.status_code != 0 {
                return Err(PlatformError::ApiError {
                    status: 200,
                    message: format!("douyin open search error code: {}", resp.status_code),
                });
            }
            let items = resp.data.map(|d| d.aweme_list).unwrap_or_default();
            let videos: Vec<VideoInfo> = items
                .into_iter()
                .map(DouyinConnector::aweme_to_video)
                .collect();
            return Ok(SocialSearchResult {
                videos,
                next_page_token: None,
                total_estimated: None,
            });
        }

        Err(PlatformError::NotAuthenticated)
    }

    async fn get_video(&self, video_id: &str) -> Result<VideoInfo, PlatformError> {
        if let Some(api_key) = &self.open_api_key {
            let builder = self
                .req(format!("{}/api/douyin/v1/video/video_data/", OPEN_BASE))
                .query(&[("aweme_id", video_id)])
                .header("access-token", api_key);
            let body = self.fetch(builder).await?;
            let resp: DouyinDetailResponse = serde_json::from_slice(&body)
                .map_err(|e| PlatformError::ParseError(e.to_string()))?;
            if resp.status_code != 0 {
                return Err(PlatformError::ApiError {
                    status: 200,
                    message: format!("douyin open detail error code: {}", resp.status_code),
                });
            }
            let aweme = resp
                .aweme_detail
                .ok_or_else(|| PlatformError::ParseError("empty aweme_detail".to_string()))?;
            return Ok(DouyinConnector::aweme_to_video(aweme));
        }

        if self.web_cookie.is_none() {
            return Err(PlatformError::NotAuthenticated);
        }

        let builder = self
            .req(format!("{}/aweme/v1/web/aweme/detail/", WEB_BASE))
            .query(&[("aweme_id", video_id)]);
        let body = self.fetch(builder).await?;
        let resp: DouyinDetailResponse =
            serde_json::from_slice(&body).map_err(|e| PlatformError::ParseError(e.to_string()))?;
        if resp.status_code != 0 {
            return Err(PlatformError::ApiError {
                status: 200,
                message: format!("douyin detail error code: {}", resp.status_code),
            });
        }
        let aweme = resp
            .aweme_detail
            .ok_or_else(|| PlatformError::ParseError("empty aweme_detail".to_string()))?;
        Ok(DouyinConnector::aweme_to_video(aweme))
    }

    async fn trending(&self, max_results: u32) -> Result<Vec<VideoInfo>, PlatformError> {
        if self.web_cookie.is_none() {
            return Err(PlatformError::NotAuthenticated);
        }
        let count = max_results.to_string();
        let builder = self
            .req(format!("{}/aweme/v1/web/recommend/", WEB_BASE))
            .query(&[("count", count.as_str())]);
        let body = self.fetch(builder).await?;
        let resp: DouyinRecommendResponse =
            serde_json::from_slice(&body).map_err(|e| PlatformError::ParseError(e.to_string()))?;
        if resp.status_code != 0 {
            return Err(PlatformError::ApiError {
                status: 200,
                message: format!("douyin recommend error code: {}", resp.status_code),
            });
        }
        let videos: Vec<VideoInfo> = resp
            .aweme_list
            .into_iter()
            .map(DouyinConnector::aweme_to_video)
            .collect();
        Ok(videos)
    }

    async fn get_me(&self) -> Result<UserInfo, PlatformError> {
        Err(PlatformError::NotAuthenticated)
    }

    async fn upload_video(
        &self,
        _title: &str,
        _description: &str,
        _file_path: &str,
    ) -> Result<VideoInfo, PlatformError> {
        Err(PlatformError::UploadFailed(
            "douyin upload not supported".to_string(),
        ))
    }
}

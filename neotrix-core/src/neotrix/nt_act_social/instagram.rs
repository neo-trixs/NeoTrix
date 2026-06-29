use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

use crate::neotrix::nt_act_social::auth::SocialAuth;
use crate::neotrix::nt_act_social::connector::{
    Platform, PlatformError, SocialSearchResult, SocialMediaPlatform, UserInfo, VideoInfo,
};
use crate::neotrix::nt_act_social::rate_limit::RateLimiter;

const BASE_URL: &str = "https://graph.facebook.com/v22.0";

#[derive(Deserialize)]
struct MePictureData {
    url: Option<String>,
}

#[derive(Deserialize)]
struct MePicture {
    data: Option<MePictureData>,
}

#[derive(Deserialize)]
struct MeResponse {
    id: String,
    username: Option<String>,
    name: Option<String>,
    picture: Option<MePicture>,
}

#[derive(Deserialize)]
struct IgMediaItem {
    id: String,
    media_url: Option<String>,
    thumbnail_url: Option<String>,
    caption: Option<String>,
    timestamp: Option<String>,
    like_count: Option<i64>,
    comments_count: Option<i64>,
}

#[derive(Deserialize)]
struct IgMediaResponse {
    data: Vec<IgMediaItem>,
    paging: Option<PagingInfo>,
}

#[derive(Deserialize)]
struct PagingInfo {
    cursors: Option<CursorInfo>,
}

#[derive(Deserialize)]
struct CursorInfo {
    after: Option<String>,
}

#[derive(Deserialize)]
struct IgHashtagSearchResponse {
    data: Vec<IgHashtagItem>,
}

#[derive(Deserialize)]
struct IgHashtagItem {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct HashtagMediaResponse {
    data: Vec<IgMediaItem>,
}

pub struct InstagramConnector {
    client: Client,
    auth: SocialAuth,
    rate_limiter: RateLimiter,
    user_id: Option<String>,
}

impl Default for InstagramConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl InstagramConnector {
    pub fn new() -> Self {
        InstagramConnector {
            client: Client::new(),
            auth: SocialAuth::new(),
            rate_limiter: RateLimiter::new(),
            user_id: None,
        }
    }

    fn get_token(&self) -> Result<String, PlatformError> {
        self.auth
            .get_token("instagram")
            .map(|t| t.access_token)
            .ok_or(PlatformError::NotAuthenticated)
    }

    fn get_uid(&self) -> Result<&str, PlatformError> {
        self.user_id
            .as_deref()
            .ok_or(PlatformError::NotAuthenticated)
    }

    async fn api_get<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
        params: &[(&str, &str)],
    ) -> Result<T, PlatformError> {
        let wait = self.rate_limiter.wait_if_needed("instagram");
        if wait > Duration::ZERO {
            sleep(wait).await;
        }

        let resp = self
            .client
            .get(url)
            .query(&[("access_token", token)])
            .query(params)
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        self.rate_limiter.record_call("instagram");

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(PlatformError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }

        resp.json::<T>()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))
    }

    fn parse_timestamp(s: Option<String>) -> Option<DateTime<Utc>> {
        s.and_then(|t| {
            DateTime::parse_from_rfc3339(&t)
                .or_else(|_| DateTime::parse_from_str(&t, "%Y-%m-%dT%H:%M:%S%z"))
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
    }

    fn into_video_info(item: IgMediaItem) -> VideoInfo {
        VideoInfo {
            id: item.id,
            title: item.caption.clone().unwrap_or_default(),
            description: item.caption.unwrap_or_default(),
            platform: Platform::Instagram,
            url: item.media_url.unwrap_or_default(),
            thumbnail_url: item.thumbnail_url,
            duration_secs: None,
            view_count: None,
            like_count: item.like_count.map(|v| v as u64),
            comment_count: item.comments_count.map(|v| v as u64),
            author: None,
            published_at: Self::parse_timestamp(item.timestamp),
        }
    }
}

#[async_trait]
impl SocialMediaPlatform for InstagramConnector {
    fn platform(&self) -> Platform {
        Platform::Instagram
    }

    fn is_authenticated(&self) -> bool {
        self.auth.is_token_valid("instagram") && self.user_id.is_some()
    }

    async fn login(&mut self) -> Result<(), PlatformError> {
        let token = self.get_token()?;
        let url = format!("{}/me", BASE_URL);
        let params = [("fields", "id")];
        let me: MeResponse = self.api_get(&url, &token, &params).await?;
        self.user_id = Some(me.id);
        Ok(())
    }

    async fn search(&self, query: &str, max_results: u32) -> Result<SocialSearchResult, PlatformError> {
        let token = self.get_token()?;
        let uid = self.get_uid()?;
        let limit_str = max_results.to_string();

        let search_url = format!("{}/ig_hashtag_search", BASE_URL);
        let search_params = [("user_id", uid), ("q", query)];
        let hashtag_resp: IgHashtagSearchResponse =
            self.api_get(&search_url, &token, &search_params).await?;

        if let Some(hashtag) = hashtag_resp.data.into_iter().next() {
            let media_url = format!("{}/{}", BASE_URL, hashtag.id);
            let media_params = [
                ("user_id", uid),
                (
                    "fields",
                    "id,media_type,media_url,thumbnail_url,caption,timestamp,like_count,comments_count",
                ),
                ("limit", &limit_str),
            ];
            let media_resp: HashtagMediaResponse =
                self.api_get(&media_url, &token, &media_params).await?;
            let videos: Vec<VideoInfo> = media_resp
                .data
                .into_iter()
                .map(Self::into_video_info)
                .collect();
            let total = videos.len() as u64;
            return Ok(SocialSearchResult {
                videos,
                next_page_token: None,
                total_estimated: Some(total),
            });
        }

        let fallback_url = format!("{}/{}/media", BASE_URL, uid);
        let fallback_params = [
            (
                "fields",
                "id,media_type,media_url,thumbnail_url,caption,timestamp,like_count,comments_count",
            ),
            ("limit", &limit_str),
        ];
        let fallback_resp: IgMediaResponse =
            self.api_get(&fallback_url, &token, &fallback_params).await?;
        let videos: Vec<VideoInfo> = fallback_resp
            .data
            .into_iter()
            .map(Self::into_video_info)
            .collect();

        let total = videos.len() as u64;
        Ok(SocialSearchResult {
            videos,
            next_page_token: fallback_resp
                .paging
                .and_then(|p| p.cursors)
                .and_then(|c| c.after),
            total_estimated: Some(total),
        })
    }

    async fn get_video(&self, video_id: &str) -> Result<VideoInfo, PlatformError> {
        let token = self.get_token()?;
        let url = format!("{}/{}", BASE_URL, video_id);
        let params = [(
            "fields",
            "id,media_type,media_url,thumbnail_url,caption,timestamp,like_count,comments_count",
        )];
        let item: IgMediaItem = self.api_get(&url, &token, &params).await?;
        Ok(Self::into_video_info(item))
    }

    async fn trending(&self, max_results: u32) -> Result<Vec<VideoInfo>, PlatformError> {
        let token = self.get_token()?;
        let uid = self.get_uid()?;
        let limit_str = max_results.to_string();
        let url = format!("{}/{}/media", BASE_URL, uid);
        let params = [
            (
                "fields",
                "id,media_type,media_url,thumbnail_url,caption,timestamp,like_count,comments_count",
            ),
            ("limit", &limit_str),
        ];
        let resp: IgMediaResponse = self.api_get(&url, &token, &params).await?;
        let videos: Vec<VideoInfo> = resp
            .data
            .into_iter()
            .map(Self::into_video_info)
            .collect();
        Ok(videos)
    }

    async fn get_me(&self) -> Result<UserInfo, PlatformError> {
        let token = self.get_token()?;
        let url = format!("{}/me", BASE_URL);
        let params = [("fields", "id,username,name,picture")];
        let me: MeResponse = self.api_get(&url, &token, &params).await?;
        Ok(UserInfo {
            platform_user_id: me.id,
            username: me.username.unwrap_or_default(),
            display_name: me.name,
            avatar_url: me.picture.and_then(|p| p.data).and_then(|d| d.url),
            follower_count: None,
            platform: Platform::Instagram,
        })
    }

    async fn upload_video(
        &self,
        _title: &str,
        _description: &str,
        _file_path: &str,
    ) -> Result<VideoInfo, PlatformError> {
        Err(PlatformError::UploadFailed(
            "Instagram upload requires a Business account with Media Container API".into(),
        ))
    }
}


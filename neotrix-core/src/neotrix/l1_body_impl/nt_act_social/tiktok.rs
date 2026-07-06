use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::neotrix::nt_act_social::connector::{
    Platform, PlatformError, SearchResult, SocialMediaPlatform, UserInfo, VideoInfo,
};
use crate::neotrix::nt_act_social::auth::SocialAuth;
use crate::neotrix::nt_act_social::rate_limit::RateLimiter;

static BASE_URL: &str = "https://open.tiktokapis.com";

#[derive(Deserialize)]
struct TikTokErrorBody {
    code: String,
    message: String,
}

#[derive(Deserialize)]
struct TikTokListData {
    videos: Vec<TikTokVideoItem>,
    cursor: Option<String>,
}

#[derive(Deserialize)]
struct TikTokListResponse {
    data: TikTokListData,
    error: TikTokErrorBody,
}

#[derive(Deserialize)]
struct TikTokQueryData {
    videos: Vec<TikTokVideoItem>,
}

#[derive(Deserialize)]
struct TikTokQueryResponse {
    data: TikTokQueryData,
    error: TikTokErrorBody,
}

#[derive(Deserialize)]
struct TikTokUserData {
    user: TikTokUserItem,
}

#[derive(Deserialize)]
struct TikTokUserResponse {
    data: TikTokUserData,
    error: TikTokErrorBody,
}

#[derive(Deserialize)]
struct TikTokUserItem {
    open_id: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
    follower_count: Option<u64>,
}

#[derive(Deserialize)]
struct TikTokVideoItem {
    id: String,
    title: Option<String>,
    cover_image_url: Option<String>,
    duration: Option<String>,
    view_count: Option<u64>,
    like_count: Option<u64>,
    comment_count: Option<u64>,
    author: Option<TikTokAuthor>,
    create_time: Option<String>,
}

#[derive(Deserialize)]
struct TikTokAuthor {
    display_name: Option<String>,
}

pub struct TikTokConnector {
    client: Client,
    auth: SocialAuth,
    rate_limiter: RateLimiter,
    platform: Platform,
}

impl Default for TikTokConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl TikTokConnector {
    pub fn new() -> Self {
        TikTokConnector {
            client: Client::new(),
            auth: SocialAuth::new(),
            rate_limiter: RateLimiter::new(),
            platform: Platform::TikTok,
        }
    }

    fn get_access_token(&self) -> Result<String, PlatformError> {
        self.auth
            .get_token("tiktok")
            .map(|t| t.access_token)
            .ok_or(PlatformError::NotAuthenticated)
    }
}

#[async_trait]
impl SocialMediaPlatform for TikTokConnector {
    fn platform(&self) -> Platform {
        self.platform
    }

    fn is_authenticated(&self) -> bool {
        self.auth.is_token_valid("tiktok")
    }

    async fn login(&mut self) -> Result<(), PlatformError> {
        if self.auth.is_token_valid("tiktok") {
            return Ok(());
        }
        Err(PlatformError::AuthFailed("no valid tiktok token found".into()))
    }

    async fn search(&self, query: &str, max_results: u32) -> Result<SearchResult, PlatformError> {
        let token = self.get_access_token()?;

        let wait = self.rate_limiter.wait_if_needed("tiktok");
        if wait > std::time::Duration::ZERO {
            tokio::time::sleep(wait).await;
        }

        let body = serde_json::json!({
            "max_count": max_results.min(100),
            "query": { "query": query }
        });

        let resp = self
            .client
            .post(format!("{}/v2/video/list/", BASE_URL))
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        self.rate_limiter.record_call("tiktok");

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(PlatformError::ApiError {
                status: status.as_u16(),
                message: err_text,
            });
        }

        let list_resp: TikTokListResponse = resp
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        if list_resp.error.code != "ok" {
            return Err(PlatformError::ApiError {
                status: 200,
                message: format!("{}: {}", list_resp.error.code, list_resp.error.message),
            });
        }

        let videos: Vec<VideoInfo> = list_resp
            .data
            .videos
            .into_iter()
            .map(|v| {
                let published_at = v.create_time.and_then(|t| {
                    DateTime::parse_from_rfc3339(&t).ok().map(|dt| dt.with_timezone(&Utc))
                });
                let vid_id = v.id;
                VideoInfo {
                    id: vid_id.clone(),
                    title: v.title.unwrap_or_default(),
                    description: String::new(),
                    platform: Platform::TikTok,
                    url: format!("https://www.tiktok.com/@tiktok/video/{}", vid_id),
                    thumbnail_url: v.cover_image_url,
                    duration_secs: v.duration.and_then(|d| d.parse().ok()),
                    view_count: v.view_count,
                    like_count: v.like_count,
                    comment_count: v.comment_count,
                    author: v.author.and_then(|a| a.display_name),
                    published_at,
                }
            })
            .collect();

        Ok(SearchResult {
            videos,
            next_page_token: list_resp.data.cursor,
            total_estimated: None,
        })
    }

    async fn get_video(&self, video_id: &str) -> Result<VideoInfo, PlatformError> {
        let token = self.get_access_token()?;

        let wait = self.rate_limiter.wait_if_needed("tiktok");
        if wait > std::time::Duration::ZERO {
            tokio::time::sleep(wait).await;
        }

        let body = serde_json::json!({
            "filters": {
                "video_ids": [video_id]
            }
        });

        let resp = self
            .client
            .post(format!("{}/v2/video/query/", BASE_URL))
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        self.rate_limiter.record_call("tiktok");

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(PlatformError::ApiError {
                status: status.as_u16(),
                message: err_text,
            });
        }

        let query_resp: TikTokQueryResponse = resp
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        if query_resp.error.code != "ok" {
            return Err(PlatformError::ApiError {
                status: 200,
                message: format!("{}: {}", query_resp.error.code, query_resp.error.message),
            });
        }

        let item = query_resp
            .data
            .videos
            .into_iter()
            .next()
            .ok_or_else(|| PlatformError::ApiError {
                status: 404,
                message: format!("video {} not found", video_id),
            })?;

        let published_at = item.create_time.and_then(|t| {
            DateTime::parse_from_rfc3339(&t).ok().map(|dt| dt.with_timezone(&Utc))
        });

        Ok(VideoInfo {
            id: item.id,
            title: item.title.unwrap_or_default(),
            description: String::new(),
            platform: Platform::TikTok,
            url: format!("https://www.tiktok.com/@tiktok/video/{}", video_id),
            thumbnail_url: item.cover_image_url,
            duration_secs: item.duration.and_then(|d| d.parse().ok()),
            view_count: item.view_count,
            like_count: item.like_count,
            comment_count: item.comment_count,
            author: item.author.and_then(|a| a.display_name),
            published_at,
        })
    }

    async fn trending(&self, _max_results: u32) -> Result<Vec<VideoInfo>, PlatformError> {
        Err(PlatformError::UploadFailed("trending not available on TikTok API".into()))
    }

    async fn get_me(&self) -> Result<UserInfo, PlatformError> {
        let token = self.get_access_token()?;

        let wait = self.rate_limiter.wait_if_needed("tiktok");
        if wait > std::time::Duration::ZERO {
            tokio::time::sleep(wait).await;
        }

        let url = format!(
            "{}/v2/user/info/?fields=open_id,union_id,avatar_url,avatar_url_100,display_name,bio_description,follower_count,following_count,likes_count,video_count,is_verified",
            BASE_URL
        );

        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        self.rate_limiter.record_call("tiktok");

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(PlatformError::ApiError {
                status: status.as_u16(),
                message: err_text,
            });
        }

        let user_resp: TikTokUserResponse = resp
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        if user_resp.error.code != "ok" {
            return Err(PlatformError::ApiError {
                status: 200,
                message: format!("{}: {}", user_resp.error.code, user_resp.error.message),
            });
        }

        Ok(UserInfo {
            platform_user_id: user_resp.data.user.open_id,
            username: user_resp.data.user.display_name.clone().unwrap_or_default(),
            display_name: user_resp.data.user.display_name,
            avatar_url: user_resp.data.user.avatar_url,
            follower_count: user_resp.data.user.follower_count,
            platform: Platform::TikTok,
        })
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}

use base64::Engine;
use chrono::{DateTime, Utc};
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

use crate::neotrix::nt_act_social::auth::{PlatformTokens, SocialAuth};
use crate::neotrix::nt_act_social::connector::{
    Platform, PlatformError, SearchResult, SocialMediaPlatform, UserInfo, VideoInfo,
};
use crate::neotrix::nt_act_social::rate_limit::RateLimiter;

const TOKEN_URL: &str = "https://www.reddit.com/api/v1/access_token";
const OAUTH_BASE: &str = "https://oauth.reddit.com";

pub struct RedditConnector {
    client: reqwest::Client,
    auth: SocialAuth,
    rate_limiter: RateLimiter,
    #[allow(dead_code)]
    user_agent: String,
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    scope: Option<String>,
}

#[derive(Deserialize)]
struct Listing {
    data: ListingData,
}

#[derive(Deserialize)]
struct ListingData {
    children: Vec<ListingChild>,
    after: Option<String>,
    dist: Option<u64>,
}

#[derive(Deserialize)]
struct ListingChild {
    data: PostData,
}

#[derive(Deserialize)]
struct PostData {
    id: String,
    title: String,
    selftext: Option<String>,
    url: String,
    thumbnail: Option<String>,
    #[allow(dead_code)]
    is_video: Option<bool>,
    secure_media: Option<SecureMedia>,
    #[allow(dead_code)]
    score: Option<i64>,
    num_comments: Option<u64>,
    author: Option<String>,
    created_utc: Option<f64>,
    ups: Option<i64>,
}

#[derive(Deserialize)]
struct SecureMedia {
    reddit_video: Option<RedditVideo>,
}

#[derive(Deserialize)]
struct RedditVideo {
    fallback_url: Option<String>,
    duration: Option<u64>,
}

#[derive(Deserialize)]
struct MeResponse {
    name: String,
    id: String,
    subreddit: Option<MeSubreddit>,
    icon_img: Option<String>,
    total_karma: Option<i64>,
}

#[derive(Deserialize)]
struct MeSubreddit {
    display_name_prefixed: Option<String>,
}

impl RedditConnector {
    pub fn new(client_id: &str, client_secret: &str, user_agent: &str) -> Self {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    USER_AGENT,
                    reqwest::header::HeaderValue::from_str(user_agent).expect("result"),
                );
                h
            })
            .build()
            .expect("result");

        RedditConnector {
            client,
            auth: SocialAuth::new(),
            rate_limiter: RateLimiter::new(),
            user_agent: user_agent.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl SocialMediaPlatform for RedditConnector {
    fn platform(&self) -> Platform {
        Platform::Reddit
    }

    fn is_authenticated(&self) -> bool {
        self.auth.is_token_valid("reddit")
    }

    async fn login(&mut self) -> Result<(), PlatformError> {
        let creds = format!("{}:{}", self.client_id, self.client_secret);
        let encoded = base64::engine::general_purpose::STANDARD.encode(creds.as_bytes());

        let params = [("grant_type", "client_credentials")];
        let response = self
            .client
            .post(TOKEN_URL)
            .header(AUTHORIZATION, format!("Basic {}", encoded))
            .form(&params)
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(PlatformError::AuthFailed(format!("{}: {}", status, body)));
        }

        let token_resp: TokenResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        let expires_at = Utc::now().timestamp() + token_resp.expires_in as i64;

        self.auth.set_token(
            "reddit",
            PlatformTokens {
                access_token: token_resp.access_token,
                refresh_token: None,
                expires_at: Some(expires_at),
                scope: token_resp.scope,
            },
        );

        Ok(())
    }

    async fn search(&self, query: &str, max_results: u32) -> Result<SearchResult, PlatformError> {
        let token = self
            .auth
            .get_token("reddit")
            .ok_or(PlatformError::NotAuthenticated)?;

        self.rate_limiter.wait_if_needed("reddit");
        self.rate_limiter.record_call("reddit");

        let url = reqwest::Url::parse_with_params(
            &format!("{}/r/all/search", OAUTH_BASE),
            &[
                ("q", query),
                ("limit", &max_results.to_string()),
                ("restrict_sr", "off"),
                ("sort", "relevance"),
                ("type", "link"),
            ],
        )
        .map_err(|e| PlatformError::Network(e.to_string()))?;

        let response = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(PlatformError::ApiError {
                status,
                message: body,
            });
        }

        let listing: Listing = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        let videos: Vec<VideoInfo> = listing
            .data
            .children
            .into_iter()
            .map(|child| post_to_video_info(child.data))
            .collect();

        Ok(SearchResult {
            videos,
            next_page_token: listing.data.after,
            total_estimated: listing.data.dist,
        })
    }

    async fn get_video(&self, video_id: &str) -> Result<VideoInfo, PlatformError> {
        let token = self
            .auth
            .get_token("reddit")
            .ok_or(PlatformError::NotAuthenticated)?;

        self.rate_limiter.wait_if_needed("reddit");
        self.rate_limiter.record_call("reddit");

        let full_id = if video_id.starts_with("t3_") {
            video_id.to_string()
        } else {
            format!("t3_{}", video_id)
        };

        let url = format!("{}/api/info?id={}", OAUTH_BASE, full_id);

        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(PlatformError::ApiError {
                status,
                message: body,
            });
        }

        let listing: Listing = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        let post = listing
            .data
            .children
            .into_iter()
            .next()
            .ok_or_else(|| PlatformError::ParseError("no post found".into()))?;

        Ok(post_to_video_info(post.data))
    }

    async fn trending(&self, max_results: u32) -> Result<Vec<VideoInfo>, PlatformError> {
        let token = self
            .auth
            .get_token("reddit")
            .ok_or(PlatformError::NotAuthenticated)?;

        self.rate_limiter.wait_if_needed("reddit");
        self.rate_limiter.record_call("reddit");

        let url = format!(
            "{}/r/videos/hot?limit={}&raw_json=1",
            OAUTH_BASE, max_results,
        );

        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(PlatformError::ApiError {
                status,
                message: body,
            });
        }

        let listing: Listing = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        let videos: Vec<VideoInfo> = listing
            .data
            .children
            .into_iter()
            .map(|child| post_to_video_info(child.data))
            .collect();

        Ok(videos)
    }

    async fn get_me(&self) -> Result<UserInfo, PlatformError> {
        let token = self
            .auth
            .get_token("reddit")
            .ok_or(PlatformError::NotAuthenticated)?;

        self.rate_limiter.wait_if_needed("reddit");
        self.rate_limiter.record_call("reddit");

        let url = format!("{}/api/v1/me", OAUTH_BASE);

        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token.access_token))
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(PlatformError::ApiError {
                status,
                message: body,
            });
        }

        let me: MeResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;

        Ok(UserInfo {
            platform_user_id: me.id,
            username: me.name,
            display_name: me.subreddit.and_then(|s| s.display_name_prefixed),
            avatar_url: me.icon_img,
            follower_count: me.total_karma.map(|k| k as u64),
            platform: Platform::Reddit,
        })
    }
}

fn post_to_video_info(data: PostData) -> VideoInfo {
    let video_url = data
        .secure_media
        .as_ref()
        .and_then(|m| m.reddit_video.as_ref())
        .and_then(|v| v.fallback_url.clone())
        .unwrap_or_else(|| data.url.clone());

    let thumbnail_url = data.thumbnail.filter(|t| t != "self" && t != "default");

    let duration_secs = data
        .secure_media
        .as_ref()
        .and_then(|m| m.reddit_video.as_ref())
        .and_then(|v| v.duration);

    let published_at = data.created_utc.map(|ts| {
        DateTime::from_timestamp(ts as i64, 0).unwrap_or_default()
    });

    VideoInfo {
        id: data.id,
        title: data.title,
        description: data.selftext.unwrap_or_default(),
        platform: Platform::Reddit,
        url: video_url,
        thumbnail_url,
        duration_secs,
        view_count: None,
        like_count: data.ups.map(|u| u as u64),
        comment_count: data.num_comments,
        author: data.author,
        published_at,
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}

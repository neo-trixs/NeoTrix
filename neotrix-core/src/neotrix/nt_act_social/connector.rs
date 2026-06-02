use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    YouTube,
    TikTok,
    Instagram,
    Twitter,
    Reddit,
    Bilibili,
    Douyin,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::YouTube => write!(f, "youtube"),
            Platform::TikTok => write!(f, "tiktok"),
            Platform::Instagram => write!(f, "instagram"),
            Platform::Twitter => write!(f, "twitter"),
            Platform::Reddit => write!(f, "reddit"),
            Platform::Bilibili => write!(f, "bilibili"),
            Platform::Douyin => write!(f, "douyin"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub platform: Platform,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub duration_secs: Option<u64>,
    pub view_count: Option<u64>,
    pub like_count: Option<u64>,
    pub comment_count: Option<u64>,
    pub author: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub platform_user_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub follower_count: Option<u64>,
    pub platform: Platform,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub videos: Vec<VideoInfo>,
    pub next_page_token: Option<String>,
    pub total_estimated: Option<u64>,
}

#[derive(Debug)]
pub enum PlatformError {
    NotAuthenticated,
    AuthFailed(String),
    ApiError { status: u16, message: String },
    RateLimited { retry_after: Option<u64> },
    Network(String),
    ParseError(String),
    UploadFailed(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::NotAuthenticated => write!(f, "not authenticated"),
            PlatformError::AuthFailed(msg) => write!(f, "auth failed: {}", msg),
            PlatformError::ApiError { status, message } => {
                write!(f, "API error {}: {}", status, message)
            }
            PlatformError::RateLimited { retry_after } => {
                if let Some(secs) = retry_after {
                    write!(f, "rate limited, retry after {}s", secs)
                } else {
                    write!(f, "rate limited")
                }
            }
            PlatformError::Network(msg) => write!(f, "network error: {}", msg),
            PlatformError::ParseError(msg) => write!(f, "parse error: {}", msg),
            PlatformError::UploadFailed(msg) => write!(f, "upload failed: {}", msg),
        }
    }
}

impl std::error::Error for PlatformError {}

#[async_trait::async_trait]
pub trait SocialMediaPlatform: Send + Sync {
    fn platform(&self) -> Platform;
    fn is_authenticated(&self) -> bool;
    async fn login(&mut self) -> Result<(), PlatformError>;
    async fn search(&self, query: &str, max_results: u32) -> Result<SearchResult, PlatformError>;
    async fn get_video(&self, video_id: &str) -> Result<VideoInfo, PlatformError>;
    async fn trending(&self, max_results: u32) -> Result<Vec<VideoInfo>, PlatformError>;
    async fn get_me(&self) -> Result<UserInfo, PlatformError>;

    async fn upload_video(
        &self,
        _title: &str,
        _description: &str,
        _file_path: &str,
    ) -> Result<VideoInfo, PlatformError> {
        Err(PlatformError::UploadFailed(format!(
            "upload not supported on {}",
            self.platform()
        )))
    }
}

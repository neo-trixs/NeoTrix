use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

pub type PlatformId = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocialPlatform {
    Twitter,
    Reddit,
    Instagram,
    TikTok,
    Youtube,
    Linkedin,
    Other(String),
}

impl SocialPlatform {
    pub fn all() -> &'static [SocialPlatform] {
        &[SocialPlatform::Twitter, SocialPlatform::Reddit, SocialPlatform::Instagram, SocialPlatform::TikTok, SocialPlatform::Youtube, SocialPlatform::Linkedin]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Twitter => "twitter",
            Self::Reddit => "reddit",
            Self::Instagram => "instagram",
            Self::TikTok => "tiktok",
            Self::Youtube => "youtube",
            Self::Linkedin => "linkedin",
            Self::Other(_) => "other",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "twitter" => Self::Twitter,
            "reddit" => Self::Reddit,
            "instagram" => Self::Instagram,
            "tiktok" => Self::TikTok,
            "youtube" => Self::Youtube,
            "linkedin" => Self::Linkedin,
            other => Self::Other(other.to_string()),
        }
    }
}

impl From<&str> for SocialPlatform {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "twitter" => SocialPlatform::Twitter,
            "reddit" => SocialPlatform::Reddit,
            "instagram" => SocialPlatform::Instagram,
            "tiktok" => SocialPlatform::TikTok,
            "youtube" => SocialPlatform::Youtube,
            "linkedin" => SocialPlatform::Linkedin,
            other => SocialPlatform::Other(other.to_string()),
        }
    }
}

impl From<String> for SocialPlatform {
    fn from(s: String) -> Self {
        SocialPlatform::from(s.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub oauth_state: Option<String>,
}

impl Default for Credentials {
    fn default() -> Self {
        Self { access_token: None, refresh_token: None, expires_at: None, oauth_state: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthState { Guest, Authenticated, Expired, Error }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub platform: SocialPlatform,
    pub credentials: Credentials,
    pub state: AuthState,
}

impl SessionEntry {
    pub fn guest(platform: SocialPlatform) -> Self {
        Self { platform, credentials: Credentials::default(), state: AuthState::Guest }
    }
}

pub enum AuthFlow { OAuth2PKCE, OAuth2ClientCredentials, OAuth1a }

pub trait SocialPlatformAdapter: Send + Sync {
    fn id(&self) -> PlatformId;
    fn name(&self) -> &'static str;
    fn auth_flow(&self) -> AuthFlow;
    fn api_base_url(&self) -> &'static str;

    /// Channel name for multi-backend routing (e.g., "twitter", "youtube")
    /// Returns None if this adapter doesn't use channels
    fn channel_name(&self) -> Option<&'static str> {
        None
    }

    fn get_recommended(&self, session: &SessionEntry, limit: usize) -> Result<FeedResult, SocialAccessError>;
    fn get_following(&self, session: &SessionEntry, limit: usize) -> Result<FeedResult, SocialAccessError>;
    fn get_trending(&self) -> Result<Vec<TrendingTopic>, SocialAccessError>;

    /// Search for content on this platform
    fn search(&self, _session: &SessionEntry, _query: &str, _limit: usize) -> Result<FeedResult, SocialAccessError> {
        Ok(FeedResult { items: vec![], total: 0 })
    }

    /// Get content from a URL
    fn get_from_url(&self, _session: &SessionEntry, _url: &str) -> Result<ExtractorResult, SocialAccessError> {
        Err(SocialAccessError::Platform("get_from_url not implemented".into()))
    }
}

pub trait SocialAuth: Send + Sync {
    fn login(&self, creds: Credentials) -> Result<SessionEntry, SocialAccessError>;
    fn refresh(&self, session: &mut SessionEntry) -> Result<(), SocialAccessError>;
    fn logout(&self, session: &SessionEntry) -> Result<(), SocialAccessError>;
}

pub trait SocialPost: Send + Sync {
    fn create_post(&self, session: &SessionEntry, content: PostContent) -> Result<PostResult, SocialAccessError>;
    fn delete_post(&self, session: &SessionEntry, post_id: &str) -> Result<(), SocialAccessError>;
}

#[derive(Debug, Clone)]
pub struct TrendingTopic { pub name: String, pub volume: u64 }

#[derive(Debug, Clone)]
pub struct PostContent { pub text: String, pub media_urls: Vec<String> }
#[derive(Debug, Clone)]
pub struct PostResult { pub id: String, pub url: String }
#[derive(Debug, Clone)]
pub struct FeedResult { pub items: Vec<FeedItem>, pub total: usize }
#[derive(Debug, Clone)]
pub struct FeedItem {
    pub id: String,
    pub content: String,
    pub author: String,
    pub metrics: EngagementMetrics,
    pub score: f64,
    pub actions: HashMap<String, f64>,
}

impl Default for FeedItem {
    fn default() -> Self {
        Self { id: String::new(), content: String::new(), author: String::new(), metrics: EngagementMetrics::default(), score: 0.0, actions: HashMap::new() }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngagementMetrics {
    pub likes: u64, pub replies: u64, pub views: Option<u64>, pub shares: u64,
}

pub type SocialAccessResult<T> = Result<T, SocialAccessError>;

#[derive(Debug)]
pub enum SocialAccessError {
    Network(String),
    Parse(String),
    AuthFailed { platform: SocialPlatform, reason: String },
    RateLimit(String),
    Platform(String),
}

impl std::fmt::Display for SocialAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "network error: {}", e),
            Self::Parse(e) => write!(f, "parse error: {}", e),
            Self::AuthFailed { platform, reason } => write!(f, "auth failed for {:?}: {}", platform, reason),
            Self::RateLimit(e) => write!(f, "rate limit: {}", e),
            Self::Platform(e) => write!(f, "platform error: {}", e),
        }
    }
}
impl std::error::Error for SocialAccessError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedType { Latest, Trending, Following }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub username: String, pub display_name: String, pub user_id: String,
    pub avatar_url: Option<String>, pub followers: Option<u64>, pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub url: String, pub media_type: String, pub width: Option<u32>, pub height: Option<u32>, pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPost {
    pub id: String, pub platform: SocialPlatform, pub author: Author, pub content: String,
    pub title: Option<String>, pub media: Vec<MediaItem>, pub metrics: EngagementMetrics,
    pub created_at: SystemTime, pub url: String, pub parent_id: Option<String>, pub platform_meta: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorResult { pub items: Vec<ExtractorItem>, pub total: usize, pub source: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorItem { pub id: String, pub title: String, pub author: String, pub url: Option<String>, pub thumbnail: Option<String> }

pub struct HttpPool { client: reqwest::Client }

impl HttpPool {
    pub fn standard() -> Self {
        Self { client: reqwest::Client::builder().user_agent("NeoTrix/1.0").build().expect("failed to build HTTP client") }
    }
    pub fn get(&self, url: &str) -> reqwest::RequestBuilder { self.client.get(url) }
    pub fn post(&self, url: &str) -> reqwest::RequestBuilder { self.client.post(url) }
}

pub fn build_headers(session: &SessionEntry) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("NeoTrix/1.0"));
    headers.insert(reqwest::header::ACCEPT, reqwest::header::HeaderValue::from_static("application/json"));
    if let Some(ref token) = session.credentials.access_token {
        if let Ok(val) = reqwest::header::HeaderValue::from_str(token) {
            headers.insert(reqwest::header::AUTHORIZATION, val);
        }
    }
    headers
}

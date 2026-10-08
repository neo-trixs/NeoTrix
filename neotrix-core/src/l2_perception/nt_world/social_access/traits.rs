use serde::{Deserialize, Serialize};
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

    pub fn as_str(&self) -> &str {
        match self {
            Self::Twitter => "twitter",
            Self::Reddit => "reddit",
            Self::Instagram => "instagram",
            Self::TikTok => "tiktok",
            Self::Youtube => "youtube",
            Self::Linkedin => "linkedin",
            // 2026-10-03 修复：原为 `Self::Other(_) => "other"`，
            // 返回类型是 `&'static str`，故**丢弃了内部的名字**。
            //
            // ⛔ 后果实测：`SocialPlatform::Other("web".into()).as_str()`
            //    == "other" ⇒ 所有非枚举平台都无法按自身 id 被寻址。
            //    表现：`social catalog` 里 `web` 的 ADAPTER 显示 `none`，
            //    尽管 `web_jina` extractor 就在清单里。
            //
            //    改签名 `&'static str` → `&str` 以便返回内部 String。
            Self::Other(ref name) => name.as_str(),
        }
    }
    /// 2026-10-03：改为**大小写不敏感**，与 [`From<&str>`] 对齐。
    ///
    /// ⛔ 原实现直接 `match s`，而 `From<&str>` 走 `s.to_lowercase()`
    ///    ⇒ 同一份逻辑有两套大小写语义：`SocialPlatform::from("X")`
    ///    得到 `Twitter`，而 `SocialPlatform::from_str("X")` 得到
    ///    `Other("X")`。两者本该等价（`From<String>` 正是委托给
    ///    `from_str` 的），不一致会让「按用户输入查平台」在
    ///    CLI 参数路径上失效。
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
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
#[derive(Default)]
pub struct Credentials {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub oauth_state: Option<String>,
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
    /// ⛔ **2026-10-03 语义更正**：原字段是 `actions: HashMap<String, f64>`，
    ///    被排序引擎当作**原始计数**使用（`weight * count`）。
    ///
    ///    ⛔ 那是 x-algorithm `param.rs:285-292` 逐字点名为**错误**的读法：
    ///    > the weights do not multiply raw engagement counts. One common
    ///    > misinterpretation is … "one report cancels 468 likes" — this is
    ///    > incorrect because the weights apply to the **predicted probabilities**
    ///    > rather than raw counts.
    ///
    ///    且该 map 的 11 个键**没有任何生产者**（全部 adapter 写空 map），
    ///    所以旧字段既是错的、又恒为空。
    ///
    /// ⇒ 改为 [`PredictedActions`](super::feed::PredictedActions)，
    ///   用类型约束保证传入的是 `0.0..=1.0` 的概率而非计数。
    pub predicted: super::feed::PredictedActions,
    /// 对应上游 `candidate.bidirectional_boost_eligible()`：
    /// 互相关注时 reply 权重获得条件提升（`+15.0`，只作用于 reply 这一个 head）。
    pub bidirectional_eligible: bool,
}

impl Default for FeedItem {
    fn default() -> Self {
        Self {
            id: String::new(),
            content: String::new(),
            author: String::new(),
            metrics: EngagementMetrics::default(),
            score: 0.0,
            predicted: super::feed::PredictedActions::new(),
            bidirectional_eligible: false,
        }
    }
}

impl FeedItem {
    /// 构造一条无预测的帖子（测试与占位用）。
    pub fn with_id(id: &str, content: &str, author: &str) -> Self {
        Self {
            id: id.to_string(),
            content: content.to_string(),
            author: author.to_string(),
            ..Default::default()
        }
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
    /// ⛔ 原实现在此 `.expect("failed to build HTTP client")` —— 生产路径 panic。
    ///    `reqwest::Client::builder().build()` 在 TLS 后端初始化失败时会 Err，
    ///    虽罕见但并非不可能（缺 CA、代理配置非法）。
    ///
    ///    改为 `try_standard()` 返回 `Result`，让失败以值传递；
    ///    保留 `standard()` 供测试与既有调用方使用，但其 panic 语义在此标注清楚。
    pub fn try_standard() -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .user_agent("NeoTrix/1.0")
            .build()
            .map_err(|e| format!("failed to build HTTP client: {}", e))?;
        Ok(Self { client })
    }

    // ⛔ 2026-10-07 **删除** `standard()`（原为 `panic!` 契约的便捷包装）。
    //
    // ⭐ 完整取证链（**第二次尝试删除，第一次证据不足**）：
    //  1. 生产调用方共 **2 个**：`reddit.rs:21` 与 `instagram.rs:20`
    //     ⇒ 两者都已迁到 `try_standard()?`（二者本就返回 `Result`）。
    //  2. 复核用 `--count` 看**全部**命中：`HttpPool::standard` 仅剩 2 处，
    //     且**全是注释**（迁移说明）；`Self::standard` 0 处 ⇒ 真零调用方。
    //  3. `check-unwrap`：删前 NEW=3，删后 NEW=**2** ⇒ 门确认 panic 已消失。
    //
    // ⇒ 零调用方的 panic 函数，其唯一作用是让门永远红，
    //   并给未来留「一调用就重新引入 panic」的坑 ⇒ 删除 + 留痕。
    //
    // ⚠️ 第一次删我犯的错（留作教训）：只 grep 了 `reddit.rs` 就断言
    //   「零调用方」，漏了 `instagram.rs` ⇒ 编译失败后回滚。
    // ⇒ **「grep 只看第一个命中就下结论」= 未取证。**
    //
    // ⇒ 新增调用方请直接用 [`Self::try_standard`]。


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


#[cfg(test)]
mod other_variant_tests {
    use super::*;

    /// 回归：原 `as_str()` 对 `Other(_)` 返回字面量 `"other"`
    /// 并丢弃内部名字（签名是 `&'static str` 逼出了这个 bug）。
    /// 后果：所有非枚举平台无法按自身 id 寻址 ——
    /// 实测 `social catalog` 把 `web` 的 ADAPTER 报成 none。
    #[test]
    fn other_variant_returns_its_own_name_not_the_literal_other() {
        assert_eq!(SocialPlatform::Other("web".into()).as_str(), "web");
        assert_eq!(SocialPlatform::Other("mastodon".into()).as_str(), "mastodon");
        // ⛔ 绝不能是 "other"
        assert_ne!(SocialPlatform::Other("web".into()).as_str(), "other");
    }

    #[test]
    fn from_str_roundtrips_for_other_variants() {
        for id in ["web", "mastodon", "github", "zhihu"] {
            let p = SocialPlatform::from_str(id);
            assert_eq!(p.as_str(), id, "`{}` must round-trip through from_str", id);
        }
    }

    #[test]
    fn from_str_maps_known_ids_to_variants() {
        // 既有 6 个枚举映射不得被本次改动破坏
        assert_eq!(SocialPlatform::from_str("twitter"), SocialPlatform::Twitter);
        assert_eq!(SocialPlatform::from_str("youtube"), SocialPlatform::Youtube);
        assert_eq!(SocialPlatform::from_str("linkedin"), SocialPlatform::Linkedin);
    }

    #[test]
    fn from_str_and_from_impl_agree_on_case() {
        // 三个入口必须大小写语义一致（修复前 from_str 与 From 不一致）
        // ⚠️ 注意 "X" 不是枚举名 —— 小写化后是 "x"，落 Other。
        //    这正是目录主键（"x"）与枚举名（"twitter"）不同的事实，
        //    别名解析由 PlatformCatalog::resolve 负责，不该由 from_str 兜。
        assert_eq!(SocialPlatform::from_str("Twitter"), SocialPlatform::Twitter);
        assert_eq!(SocialPlatform::from("Twitter"), SocialPlatform::Twitter);
        assert_eq!(SocialPlatform::from("Twitter".to_string()), SocialPlatform::Twitter);
        // 大写的 X 仍落 Other("x")，与 as_str 往返一致
        assert_eq!(SocialPlatform::from_str("X"), SocialPlatform::Other("x".into()));
        assert_eq!(SocialPlatform::from_str("X").as_str(), "x");
        // Other 也统一小写化，否则 `Other("GitHub")` 与
        //    `Other("github")` 会是两个不同的 key
        assert_eq!(
            SocialPlatform::from_str("GitHub"),
            SocialPlatform::Other("github".into())
        );
        assert_eq!(
            SocialPlatform::from_str("GitHub").as_str(),
            "github"
        );
    }
}

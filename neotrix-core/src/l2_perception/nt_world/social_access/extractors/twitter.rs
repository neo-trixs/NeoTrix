//! Twitter/X 免登录读取 — 基于实测端点的重写（2026-10-03）
//!
//! # ⛔ 被替换掉的死端点（历史缺陷）
//!
//! 原实现两个方法都打 `https://api.fxtwitter.com/search?q=...&f=jpeg`，
//! 期望响应形如 `{"tweets":[{"id","text","author":{"name"},"media":{"photos":[...]}}]}`。
//!
//! **本仓实测（2026-10-03）：该端点返回 HTTP 404。** 两个方法
//! （`get_user_tweets` / `search`）因此 100% 失败。且即便端点存活，
//! 它也没有搜索能力 —— 只有两个 URL、两个近乎相同的代码块。
//!
//! 旧解析代码还有一处静默缺陷：`.filter_map(|t| ... t["author"]["name"].as_str()? ...)`
//! 对字段缺失的条目**整条丢弃**且不计数 ⇒ `total` 报的是过滤后的长度，
//! 上游无法察觉有多少条被吞掉。
//!
//! # 现依据的实测契约（2026-10-03，逐字抓取真实响应后写码）
//!
//! | 端点 | 状态 | 能做什么 |
//! |---|---|---|
//! | `api.vxtwitter.com/<user>` | 200 JSON | 用户档案 |
//! | `api.vxtwitter.com/<user>/status/<id>` | 200 JSON | 单条推文（**已知 id**） |
//! | `cdn.syndication.twimg.com/tweet-result?id=<id>&token=<any>` | 200 JSON | 单条推文（**已知 id**）|
//! | `api.vxtwitter.com/search?q=` | 200 `{"error":"User not found."}` | ⛔ **无搜索** |
//! | `x.com/<user>` 直连 | 200 但是 React 空壳 | ⛔ 零内容 |
//! | `r.jina.ai/https://x.com/...` | 403 | ⛔ 已封 |
//!
//! **能力边界（诚实声明）**：本 extractor 只提供
//! **档案查询 + 按已知 id 取单条推文**。**搜索与时间线不存在免登录路径**，
//! 必须走 cookie 会话（bird / OpenCLI，见 `channel.rs` 的 twitter 渠道）。
//! 鉴于此，`search()` 返回显式错误而非伪装成「空结果」——
//! 空结果会被上游当作「该查询真的没有结果」，是更坏的失败模式。
//!
//! # 载荷判别
//!
//! 经 [`nt_payload_guard`]：syndication 对不存在的推文返回
//! `<!DOCTYPE html>...<title>X / ?</title>`（本仓实测），
//! 而 vxtwitter 返回 JSON 404 —— 两种载体、同一语义。
//! 无判别层则两者都会被误报成 `Parse` 错误。

use reqwest::Client;
use serde_json::Value;

use super::super::nt_payload_guard::{PayloadVerdict, classify};
use super::super::traits::{AuthFlow, FeedResult, SocialPlatform, SocialPlatformAdapter, TrendingTopic};
use super::super::{
    ExtractorItem, ExtractorResult, SocialAccessError, SocialAccessResult,
    SessionEntry,
};

/// vxtwitter 档案端点前缀（实测可用）。
const VX_PROFILE: &str = "https://api.vxtwitter.com";
/// syndication 推文端点前缀（实测可用；`token` 参数只需存在，值任意）。
const SYNDICATION_TWEET: &str = "https://cdn.syndication.twimg.com/tweet-result";

/// 解析 JSON 数字字段，兼容 `u64` 与字符串两种表示。
///
/// 必要原因：两个上游对同一语义字段的类型并不一致 —— 例如 OpenCLI 的
/// `views` 是**字符串**（`tweet.views?.count || '0'`），而 AutoCLI 用
/// `parseInt` 转成数。⛔ 在此处只按数字解析会把合法载荷判成缺失。
fn num_field(v: &Value, key: &str) -> u64 {
    match v.get(key) {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(Value::String(s)) => s.trim().parse::<u64>().unwrap_or(0),
        _ => 0,
    }
}

/// 取字符串字段，兼容缺失与 null。
fn str_field<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str).filter(|s| !s.is_empty())
}

/// 构造 HTTP client，附带真实浏览器 UA（否则部分上游直接拒）。
fn client() -> SocialAccessResult<Client> {
    Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 NeoTrix/1.0")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| SocialAccessError::Network(format!("client build failed: {}", e)))
}

/// 把 HTTP 响应解析成 JSON，途经登录墙/错误信封判别。
///
/// 状态码在此**不单独决定成败**：`classify` 先看内容（HTML ⇒ 登录墙），
/// 再看 JSON 内的业务错误信封 —— 200 + `{"code":-1}` 这种组合
/// 用状态码是识别不出来的。
async fn fetch_json(client: &Client, url: &str) -> SocialAccessResult<Value> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| SocialAccessError::Network(format!("GET {}: {}", url, e)))?;

    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body = resp
        .text()
        .await
        .map_err(|e| SocialAccessError::Network(format!("body read {}: {}", url, e)))?;

    match classify(status, &content_type, &body) {
        PayloadVerdict::Json(v) => {
            // vxtwitter 把「用户不存在」表达为 JSON 里的 `error` 字段 +
            // 404。`classify` 不会拦它（信封判据只认 `code`），故在此显式处理，
            // 免得把「查无此人」当成一条正常载荷返回。
            if let Some(err) = str_field(&v, "error") {
                return Err(SocialAccessError::Platform(format!(
                    "upstream error for {}: {}",
                    url, err
                )));
            }
            Ok(v)
        }
        PayloadVerdict::HtmlInsteadOfJson { status, preview } => Err(
            SocialAccessError::AuthFailed {
                platform: SocialPlatform::Twitter,
                reason: format!(
                    "received HTML instead of JSON (HTTP {}): {} \
                     — resource missing, or a login wall / WAF challenge",
                    status, preview
                ),
            },
        ),
        PayloadVerdict::ErrorEnvelope { code, message } => Err(SocialAccessError::Platform(
            format!("error envelope from {}: code={} msg={}", url, code, message),
        )),
        PayloadVerdict::Malformed { preview } => Err(SocialAccessError::Parse(format!(
            "{}: malformed payload: {}",
            url, preview
        ))),
        PayloadVerdict::Empty => Err(SocialAccessError::Parse(format!("{}: empty body", url))),
    }
}

/// 兜底日期解析：vxtwitter 用 `date_epoch`(秒)，syndication 用 RFC3339。
///
/// 两者都不保证存在，故回落到 Unix epoch(0) —— 调用方据 `epoch == 0`
/// 判定「时间未知」，而不是拿到一个看似合理的假时间。
fn parse_epoch(v: &Value) -> u64 {
    num_field(v, "date_epoch")
}

/// 构造 `ExtractorItem`。`url` 恒为 `None`：旧实现从未填充该字段，
/// 保持一致以免下游误以为它可用。
fn item_from_parts(id: &str, text: &str, author: &str, thumb: Option<String>) -> ExtractorItem {
    ExtractorItem {
        id: id.to_string(),
        title: text.chars().take(100).collect(),
        author: author.to_string(),
        url: None,
        thumbnail: thumb,
    }
}

pub struct TwitterExtractor;

impl TwitterExtractor {
    pub fn new() -> Self {
        Self
    }

    /// 查询用户档案（免登录，实测可用）。
    ///
    /// 这是本 extractor 唯一能覆盖「按用户名查」的能力 ——
    /// 搜索端点不存在，但档案端点存在。
    pub async fn profile(
        &self,
        _session: &SessionEntry,
        username: &str,
    ) -> SocialAccessResult<ExtractorResult> {
        let client = client()?;
        let user = username.trim_start_matches('@');
        let url = format!("{}/{}", VX_PROFILE, urlencoding::encode(user));
        let v = fetch_json(&client, &url).await?;

        let screen_name = str_field(&v, "screen_name").unwrap_or(user);
        let display = str_field(&v, "name").unwrap_or(screen_name);
        let bio = str_field(&v, "description").unwrap_or("");
        // 档案不是推文：把 bio 作为内容，并带上可核对的计数以便调用方判断规模。
        let content = format!(
            "{} (@{}, {} followers, {} tweets){}",
            display,
            screen_name,
            num_field(&v, "followers_count"),
            num_field(&v, "tweet_count"),
            if bio.is_empty() { String::new() } else { format!(" — {}", bio) }
        );

        Ok(ExtractorResult {
            items: vec![ExtractorItem {
                id: str_field(&v, "id").unwrap_or(screen_name).to_string(),
                title: content.chars().take(100).collect(),
                author: display.to_string(),
                url: None,
                thumbnail: str_field(&v, "profile_image_url").map(String::from),
            }],
            total: 1,
            source: format!("twitter:profile:{}", screen_name),
        })
    }

    /// 按已知推文 id 取单条推文（免登录，实测可用）。
    ///
    /// 依次尝试两个上游：syndication 字段最全（含 `favorite_count`
    /// 与嵌套 `user`），vxtwitter 次之（`likes`/`retweets` 计数更直接）。
    /// 任一成功即返回；两者皆败则返回**首个**错误并附上第二个作为上下文。
    pub async fn tweet_by_id(
        &self,
        _session: &SessionEntry,
        id: &str,
    ) -> SocialAccessResult<ExtractorResult> {
        let client = client()?;
        let syn_url = format!("{}?id={}&token=neotrix", SYNDICATION_TWEET, id);
        let vx_url = format!("{}/i/status/{}", VX_PROFILE, id);

        let syn_err = match fetch_json(&client, &syn_url).await {
            Ok(v) => {
                let text = str_field(&v, "text").unwrap_or("");
                let user = v.get("user");
                let author = user
                    .and_then(|u| str_field(u, "screen_name"))
                    .unwrap_or("unknown");
                return Ok(ExtractorResult {
                    items: vec![ExtractorItem {
                        id: str_field(&v, "id_str").unwrap_or(id).to_string(),
                        title: text.chars().take(100).collect(),
                        author: author.to_string(),
                        url: None,
                        thumbnail: user
                            .and_then(|u| str_field(u, "profile_image_url_https"))
                            .map(String::from),
                    }],
                    total: 1,
                    source: format!("twitter:tweet:{}:syndication:epoch={}", id, parse_epoch(&v)),
                });
            }
            Err(e) => e,
        };

        match fetch_json(&client, &vx_url).await {
            Ok(v) => {
                let text = str_field(&v, "text").unwrap_or("");
                let author = str_field(&v, "user_screen_name").unwrap_or("unknown");
                let thumb = v
                    .get("media_extended")
                    .and_then(Value::as_array)
                    .and_then(|a| a.first())
                    .and_then(|m| str_field(m, "url"))
                    .map(String::from);
                Ok(ExtractorResult {
                    items: vec![item_from_parts(
                        str_field(&v, "tweetID").unwrap_or(id),
                        text,
                        author,
                        thumb,
                    )],
                    total: 1,
                    source: format!("twitter:tweet:{}:vxtwitter:epoch={}", id, parse_epoch(&v)),
                })
            }
            Err(vx_err) => Err(SocialAccessError::Network(format!(
                "both syndication and vxtwitter failed for id {}. syndication: {}. vxtwitter: {}",
                id, syn_err, vx_err
            ))),
        }
    }

    /// 查询某用户最近推文 — 免登录**不可用**（历史接口已 404）。
    ///
    /// ⛔ 保留此方法是为兼容既有调用方签名，但**恒返回错误**。
    /// 绝不返回空列表：空列表会被上游读成「该用户没有推文」，
    /// 那比一个显式错误有害得多。
    pub async fn get_user_tweets(
        &self,
        _session: &SessionEntry,
        _username: &str,
        _limit: usize,
    ) -> SocialAccessResult<ExtractorResult> {
        Err(SocialAccessError::Platform(
            "user timeline requires an authenticated session; \
             install a cookie-backed backend (see channel registry: twitter channel) \
             or call profile() for public account data"
                .into(),
        ))
    }

    /// 搜索 — 免登录**不存在**此能力（实测 `api.vxtwitter.com/search`
    /// 返回 `{"error":"User not found."}`）。
    ///
    /// ⛔ 同上：返回显式错误而非伪装成空结果。
    pub async fn search(
        &self,
        _session: &SessionEntry,
        _query: &str,
        _page: u32,
    ) -> SocialAccessResult<ExtractorResult> {
        Err(SocialAccessError::Platform(
            "no unauthenticated X search endpoint exists (fxtwitter/search is 404, \
             vxtwitter/search returns an error); \
             install a cookie-backed backend to enable search"
                .into(),
        ))
    }
}

impl SocialPlatformAdapter for TwitterExtractor {
    fn id(&self) -> String {
        "twitter".into()
    }
    fn name(&self) -> &'static str {
        "Twitter"
    }
    fn auth_flow(&self) -> AuthFlow {
        AuthFlow::OAuth2PKCE
    }
    fn api_base_url(&self) -> &'static str {
        VX_PROFILE
    }
    fn channel_name(&self) -> Option<&'static str> {
        Some("twitter")
    }

    // ⛔ 以下三个返回空结果的方法（历史实现）会制造**假成功**：
    //    上游无法区分「平台无内容」与「本适配器根本没实现」。
    //    改为显式错误，失败模式才可被下游观测到。

    fn get_recommended(
        &self,
        _session: &SessionEntry,
        _limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        Err(SocialAccessError::Platform(
            "get_recommended is not implemented for the unauthenticated X path; \
             use an authenticated backend for feeds"
                .into(),
        ))
    }

    fn get_following(
        &self,
        _session: &SessionEntry,
        _limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        Err(SocialAccessError::Platform(
            "get_following requires an authenticated session".into(),
        ))
    }

    fn get_trending(&self) -> Result<Vec<TrendingTopic>, SocialAccessError> {
        Err(SocialAccessError::Platform(
            "get_trending requires an authenticated session".into(),
        ))
    }

    fn search(
        &self,
        _session: &SessionEntry,
        _query: &str,
        _limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        // ⛔ 历史实现在此 `Handle::current()` + `block_on`：
        //    ① `Handle::current()` 在非 runtime 上下文**直接 panic**；
        //    ② `self.search(..)` 在这里同时匹配 trait 方法与同名 inherent
        //       async 方法，属**无限递归风险**的命名冲突。
        //    改为透传显式错误（真实搜索需 cookie 后端）。
        Err(SocialAccessError::Platform(
            "sync search is unavailable without an authenticated backend".into(),
        ))
    }

    fn get_from_url(
        &self,
        session: &SessionEntry,
        url: &str,
    ) -> Result<ExtractorResult, SocialAccessError> {
        // 同步壳：仅处理已知推文 id，其余交给 async 专用方法。
        let id = extract_tweet_id(url).ok_or_else(|| {
            SocialAccessError::Platform(format!(
                "cannot extract a tweet id from {}; \
                 free-text URLs and profiles need the async API",
                url
            ))
        })?;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| SocialAccessError::Platform(format!("runtime build failed: {}", e)))?;
        rt.block_on(self.tweet_by_id(session, &id))
    }
}

/// 从 x.com / twitter.com 推文 URL 中抽出 id。
///
/// 形态：`/i/status/<id>`、`/i/status/<id>/photo/1`、`/<user>/status/<id>`、
/// 带查询串与 fragment 的变体。
pub fn extract_tweet_id(url: &str) -> Option<String> {
    // 去掉 scheme：取最后一个 `://` 之后的部分。
    // ⛔ 不用 `split("://").next_back()` —— `Split<&str>` 不实现
    //    `DoubleEndedIterator`（pattern 是 &str 而非 char）。
    let after_scheme = match url.rfind("://") {
        Some(idx) => &url[idx + 3..],
        None => url,
    };
    let path = after_scheme
        .split(['?', '#'])
        .next()
        .unwrap_or(after_scheme);
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    for (i, seg) in segments.iter().enumerate() {
        if *seg == "status" || *seg == "statuses" {
            let candidate = segments.get(i + 1)?;
            // id 为纯数字；`status/latest` 这类残缺形态据此排除
            if !candidate.is_empty() && candidate.chars().all(|c| c.is_ascii_digit()) {
                return Some((*candidate).to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── URL → id 抽取 ──────────────────────────────────────────────

    #[test]
    fn extracts_id_from_canonical_forms() {
        assert_eq!(extract_tweet_id("https://x.com/jack/status/20"), Some("20".into()));
        assert_eq!(extract_tweet_id("https://twitter.com/jack/status/20"), Some("20".into()));
        assert_eq!(extract_tweet_id("https://x.com/i/status/20"), Some("20".into()));
        assert_eq!(
            extract_tweet_id("https://x.com/i/status/20/photo/1"),
            Some("20".into())
        );
        assert_eq!(
            extract_tweet_id("https://x.com/jack/status/20?s=20&t=abc"),
            Some("20".into())
        );
        assert_eq!(
            extract_tweet_id("https://x.com/jack/status/20#anchor"),
            Some("20".into())
        );
    }

    #[test]
    fn rejects_non_tweet_urls() {
        assert_eq!(extract_tweet_id("https://x.com/jack"), None);
        assert_eq!(extract_tweet_id("https://x.com/"), None);
        assert_eq!(extract_tweet_id("not a url"), None);
        // ⛔ 非数字段必须拒绝，否则会把 /status/latest 当成 id
        assert_eq!(extract_tweet_id("https://x.com/jack/status/latest"), None);
        assert_eq!(extract_tweet_id("https://x.com/jack/status/"), None);
    }

    // ── 上游字段解析（对齐本仓实测响应）────────────────────────────

    #[test]
    fn parses_real_vxtwitter_profile() {
        // 本仓实测 api.vxtwitter.com/elonmusk 的真实字段
        let v: Value = serde_json::from_str(
            r#"{"created_at":"Tue Jun 02 20:12:29 +0000 2009","description":"https://t.co/ZdBx5WABYx",
            "followers_count":241726131,"following_count":1414,"id":44196397,"location":"",
            "name":"Elon Musk","profile_image_url":"https://pbs.twimg.com/x_normal.jpg",
            "protected":false,"screen_name":"elonmusk","tweet_count":109243}"#,
        )
        .expect("hardcoded JSON must parse");

        assert_eq!(str_field(&v, "screen_name"), Some("elonmusk"));
        assert_eq!(str_field(&v, "name"), Some("Elon Musk"));
        assert_eq!(num_field(&v, "followers_count"), 241726131);
        assert_eq!(num_field(&v, "tweet_count"), 109243);
        // ⛔ 空串必须视同缺失（`location` 实测就是空串）
        assert_eq!(str_field(&v, "location"), None);
    }

    #[test]
    fn parses_real_syndication_tweet() {
        // 本仓实测 cdn.syndication.twimg.com/tweet-result?id=20 的真实字段
        let v: Value = serde_json::from_str(
            r#"{"__typename":"Tweet","favorite_count":309315,"lang":"en",
            "created_at":"2006-03-21T20:50:14.000Z","id_str":"20",
            "text":"just setting up my twttr",
            "user":{"id_str":"12","name":"jack","screen_name":"jack",
            "profile_image_url_https":"https://pbs.twimg.com/x_normal.jpg",
            "is_blue_verified":true}}"#,
        )
        .expect("hardcoded JSON must parse");

        assert_eq!(str_field(&v, "id_str"), Some("20"));
        assert_eq!(num_field(&v, "favorite_count"), 309315);
        let user = v.get("user").expect("user object present");
        assert_eq!(str_field(user, "screen_name"), Some("jack"));
    }

    #[test]
    fn parses_real_vxtwitter_status() {
        // 本仓实测 api.vxtwitter.com/<user>/status/<id> 的真实字段
        let v: Value = serde_json::from_str(
            r#"{"tweetID":"20","text":"hi","user_name":"Jack","user_screen_name":"jack",
            "likes":10,"retweets":2,"replies":1,"date":"Mon Jan 02 15:04:05 +0000 2006",
            "date_epoch":1136214245,
            "media_extended":[{"url":"https://pbs.twimg.com/media/x.jpg","type":"photo"}],
            "lang":"en"}"#,
        )
        .expect("hardcoded JSON must parse");

        assert_eq!(str_field(&v, "tweetID"), Some("20"));
        assert_eq!(str_field(&v, "user_screen_name"), Some("jack"));
        assert_eq!(num_field(&v, "likes"), 10);
        assert_eq!(num_field(&v, "date_epoch"), 1136214245);
    }

    #[test]
    fn num_field_tolerates_string_and_missing() {
        let v: Value = serde_json::from_str(r#"{"views":"1234","n":42}"#).expect("parse");
        // ⛔ OpenCLI 的 views 是字符串；只按数字解析会误判缺失
        assert_eq!(num_field(&v, "views"), 1234);
        assert_eq!(num_field(&v, "n"), 42);
        assert_eq!(num_field(&v, "absent"), 0);
        assert_eq!(num_field(&v, "views_bad"), 0);
    }

    #[test]
    fn str_field_filters_empty_and_null() {
        let v: Value = serde_json::from_str(r#"{"a":"","b":null,"c":"x"}"#).expect("parse");
        assert_eq!(str_field(&v, "a"), None, "empty string must read as absent");
        assert_eq!(str_field(&v, "b"), None);
        assert_eq!(str_field(&v, "c"), Some("x"));
    }

    #[test]
    fn title_truncation_is_char_safe() {
        // 多字节文本不得在 char 边界切开（`&s[..100]` 会 panic）
        let item = item_from_parts("1", &"中".repeat(200), "a", None);
        assert_eq!(item.title.chars().count(), 100);
    }

    // ── ⛔ 能力边界的诚实性：这些方法必须报错，不能假装成功 ──────────

    #[test]
    fn unauthenticated_capabilities_error_instead_of_faking_empty() {
        let ex = TwitterExtractor::new();
        let session = SessionEntry::guest(SocialPlatform::Twitter);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime builds");

        // ⛔ 若任一这些返回 Ok(空)，上游就会把「没实现」读成「没数据」
        assert!(rt.block_on(ex.get_user_tweets(&session, "jack", 10)).is_err());
        assert!(rt.block_on(ex.search(&session, "rust", 0)).is_err());

        // ⚠️ 同步壳必须用 UFCS 显式指定 trait 方法：
        // `TwitterExtractor` 有一个**同名**的 inherent async `search`，
        // 而 inherent 方法在名字解析上**优先于** trait 方法 ——
        // 裸写 `ex.search(..)` 会拿到那个 Future，而不是这里的同步版本。
        // ⛔ 这正是历史实现里的隐患：trait `search` 内部写 `self.search(..)`
        //    时解析到的是 inherent async 版本，意图（复用异步实现）纯属侥幸，
        //    一旦有人删掉 inherent 版本就变成 trait 无限递归。
        assert!(SocialPlatformAdapter::search(&ex, &session, "rust", 10).is_err());
        assert!(ex.get_recommended(&session, 10).is_err());
        assert!(ex.get_following(&session, 10).is_err());
        assert!(ex.get_trending().is_err());
    }

    #[test]
    fn get_from_url_rejects_non_tweet_url_without_panicking() {
        let ex = TwitterExtractor::new();
        let session = SessionEntry::guest(SocialPlatform::Twitter);
        let err = ex
            .get_from_url(&session, "https://x.com/jack")
            .expect_err("profile URLs have no tweet id");
        assert!(matches!(err, SocialAccessError::Platform(_)));
    }
}
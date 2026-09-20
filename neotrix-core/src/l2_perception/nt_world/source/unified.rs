//! 统一数据源架构 — 6 层能力模型
//!
//! ## 设计原则
//!
//! 1. **能力清晰**：每个 trait 只定义该类源的核心能力
//! 2. **按需实现**：源只需实现自己需要的 trait
//! 3. **统一管理**：通过注册表按域路由
//! 4. **向后兼容**：不修改现有源代码，通过桥接层适配
//!
//! ## 架构图
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                    UnifiedEngine (统一查询入口)                      │
//! │     search() / search_media() / search_intel() / search_llm()     │
//! └───────────────────────────────┬─────────────────────────────────────┘
//!                                 │
//!       ┌─────────────────────────┼─────────────────────────┐
//!       │                         │                         │
//!   ┌───▼───────────┐      ┌─────▼──────┐      ┌──────────▼──────────┐
//!   │ MediaSource   │      │ IntelSource│      │ OsintSource         │
//!   │ (29 个)       │      │ (12 个)    │      │ (30+ 个)            │
//!   │ search()      │      │ fetch()    │      │ investigate()       │
//!   │ play_url()    │      │ stream()   │      │ sweep()             │
//!   │ lyric()       │      │ health()   │      │ harvest()           │
//!   └───────────────┘      └────────────┘      └─────────────────────┘
//!       │                         │                         │
//!   audio/video/text          gdelt/edgar/usgs         dns/shodan/censys
//!   (29 个源)                (12 个源)                (30+ 个源)
//!
//!       ┌─────────────────────────┼─────────────────────────┐
//!       │                         │                         │
//!   ┌───▼───────────┐      ┌─────▼──────┐      ┌──────────▼──────────┐
//!   │ LlmProvider   │      │CrawlSource │      │ LocalSource         │
//!   │ (30+ 个)      │      │ (爬虫能力) │      │ (本地文件)          │
//!   │ complete()    │      │ fetch()    │      │ scan()              │
//!   │ stream()      │      │ crawl()    │      │ search()            │
//!   │ health()      │      │ discover() │      │ index()             │
//!   └───────────────┘      └────────────┘      └─────────────────────┘
//!       │                         │                         │
//!   openai/anthropic/ollama    spider/camofox           本地文件索引
//!   groq/cloudflare (30+)     humanize/stealth         (mp3/flac/...)
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// ============================================================================
// 域标签 — 声明该数据源服务哪些领域
// ============================================================================

/// 数据源服务的领域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceDomain {
    /// 媒体：音乐/视频/播客/歌词
    Media,
    /// 情报：财经/地缘/威胁/OSINT
    Intel,
    /// 学术：论文/文献/数据集
    Academic,
    /// 技术：GitHub/新闻/RSS/开发者
    Tech,
    /// 本地：本地文件索引
    Local,
    /// OSINT：开源情报/网络安全
    Osint,
    /// LLM：大语言模型推理
    Llm,
    /// Crawl：网页爬取/内容提取
    Crawl,
}

impl SourceDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Media => "media",
            Self::Intel => "intel",
            Self::Academic => "academic",
            Self::Tech => "tech",
            Self::Local => "local",
            Self::Osint => "osint",
            Self::Llm => "llm",
            Self::Crawl => "crawl",
        }
    }

    pub fn all() -> &'static [SourceDomain] {
        &[
            Self::Media,
            Self::Intel,
            Self::Academic,
            Self::Tech,
            Self::Local,
            Self::Osint,
            Self::Llm,
            Self::Crawl,
        ]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "media" => Some(Self::Media),
            "intel" => Some(Self::Intel),
            "academic" => Some(Self::Academic),
            "tech" => Some(Self::Tech),
            "local" => Some(Self::Local),
            "osint" => Some(Self::Osint),
            "llm" => Some(Self::Llm),
            "crawl" => Some(Self::Crawl),
            _ => None,
        }
    }
}

impl std::fmt::Display for SourceDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// 基础 trait — 所有数据源都实现
// ============================================================================

/// 数据源基础 trait — 定义标识和域能力
pub trait DataSource: Send + Sync + 'static {
    /// 唯一标识 (如 "netease", "gdelt", "openai")
    fn id(&self) -> &str;

    /// 显示名称 (如 "网易云音乐", "GDELT DOC 2.0")
    fn name(&self) -> &str;

    /// 服务的域列表
    fn domains(&self) -> Vec<SourceDomain>;

    /// 健康检查
    fn health_check(
        &self,
    ) -> Pin<Box<dyn Future<Output = bool> + Send>> {
        Box::pin(async { true })
    }

    /// 是否需要 API key
    fn requires_key(&self) -> bool {
        false
    }
}

// ============================================================================
// 媒体源 trait — 搜索、播放、歌词
// ============================================================================

/// 媒体搜索结果
#[derive(Debug, Clone)]
pub struct MediaSearchResult {
    pub items: Vec<MediaResult>,
    pub total: usize,
    pub page: u32,
}

/// 单个媒体结果
#[derive(Debug, Clone)]
pub struct MediaResult {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: Option<u64>,
    pub cover_url: Option<String>,
    pub qualities: Vec<String>,
}

/// 播放源
#[derive(Debug, Clone)]
pub struct PlaySource {
    pub url: String,
    pub quality: String,
    pub format: String,
    pub bitrate: u32,
    pub size: u64,
}

/// 歌词行
#[derive(Debug, Clone)]
pub struct LyricLine {
    pub timestamp_ms: Option<u64>,
    pub text: String,
}

/// 歌词
#[derive(Debug, Clone)]
pub struct Lyric {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub lines: Vec<LyricLine>,
}

/// 媒体源 trait — 搜索、播放、歌词
pub trait MediaSource: DataSource {
    /// 搜索
    fn search(
        &self,
        query: &str,
        page: u32,
    ) -> Pin<Box<dyn Future<Output = Result<MediaSearchResult, String>> + Send>>;

    /// 获取播放链接
    fn play_url(
        &self,
        item_id: &str,
        quality: &str,
    ) -> Pin<Box<dyn Future<Output = Result<PlaySource, String>> + Send>> {
        let _ = (item_id, quality);
        Box::pin(async { Err("Not implemented".into()) })
    }

    /// 获取歌词
    fn lyric(
        &self,
        item_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Lyric, String>> + Send>> {
        let _ = item_id;
        Box::pin(async { Err("Not implemented".into()) })
    }
}

// ============================================================================
// 情报源 trait — 数据获取
// ============================================================================

/// 情报数据项
#[derive(Debug, Clone)]
pub struct IntelItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
    pub timestamp: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// 情报查询结果
#[derive(Debug, Clone)]
pub struct IntelResult {
    pub items: Vec<IntelItem>,
    pub total: usize,
    pub source_id: String,
}

/// 情报源 trait — 数据获取
pub trait IntelSource: DataSource {
    /// 获取数据
    fn fetch(
        &self,
        query: &str,
    ) -> Pin<Box<dyn Future<Output = Result<IntelResult, String>> + Send>>;
}

// ============================================================================
// OSINT 源 trait — 开源情报调查
// ============================================================================

/// OSINT 调查结果
#[derive(Debug, Clone)]
pub struct OsintResult {
    pub source_id: String,
    pub target: String,
    pub findings: Vec<OsintFinding>,
    pub confidence: f64,
}

/// OSINT 发现
#[derive(Debug, Clone)]
pub struct OsintFinding {
    pub category: String,
    pub key: String,
    pub value: String,
    pub confidence: f64,
    pub source: String,
}

/// OSINT 源 trait — 开源情报调查
pub trait OsintSource: DataSource {
    /// 调查目标
    fn investigate(
        &self,
        target: &str,
    ) -> Pin<Box<dyn Future<Output = Result<OsintResult, String>> + Send>>;

    /// 支持的目标类型
    fn supported_targets(&self) -> Vec<&str>;
}

// ============================================================================
// LLM 提供者 trait — 推理服务
// ============================================================================

/// LLM 请求
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

/// LLM 消息
#[derive(Debug, Clone)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

/// LLM 响应
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: LlmUsage,
    pub finish_reason: Option<String>,
}

/// LLM 用量
#[derive(Debug, Clone, Default)]
pub struct LlmUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// LLM 提供者 trait — 推理服务
pub trait LlmProvider: DataSource {
    /// 完成推理
    fn complete(
        &self,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse, String>> + Send>>;

    /// 是否免费
    fn is_free(&self) -> bool {
        false
    }

    /// 是否需要 API key
    fn requires_api_key(&self) -> bool {
        true
    }
}

// ============================================================================
// 爬虫源 trait — 网页爬取
// ============================================================================

/// 爬取结果
#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub links: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// 爬虫源 trait — 网页爬取
pub trait CrawlSource: DataSource {
    /// 爬取 URL
    fn crawl(
        &self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<CrawlResult, String>> + Send>>;

    /// 批量爬取
    fn crawl_batch(
        &self,
        urls: &[String],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<CrawlResult>, String>> + Send>>;

    /// 支持的 URL 模式
    fn supported_patterns(&self) -> Vec<&str>;
}

// ============================================================================
// 本地源 trait — 文件管理
// ============================================================================

/// 本地文件条目
#[derive(Debug, Clone)]
pub struct LocalEntry {
    pub path: String,
    pub file_name: String,
    pub file_size: u64,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_secs: Option<u64>,
    pub format: String,
    pub cover_path: Option<String>,
    pub lyric_path: Option<String>,
}

/// 本地源 trait — 文件管理
pub trait LocalSource: DataSource {
    /// 扫描目录
    fn scan(
        &self,
        dir: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<LocalEntry>, String>> + Send>>;

    /// 搜索本地文件
    fn search(
        &self,
        query: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<LocalEntry>, String>> + Send>>;
}

// ============================================================================
// 统一数据源包装 — 类型擦除，用于注册表
// ============================================================================

/// 统一数据源包装
pub enum AnyDataSource {
    Media(Arc<dyn MediaSource>),
    Intel(Arc<dyn IntelSource>),
    Osint(Arc<dyn OsintSource>),
    Llm(Arc<dyn LlmProvider>),
    Crawl(Arc<dyn CrawlSource>),
    Local(Arc<dyn LocalSource>),
}

impl AnyDataSource {
    pub fn id(&self) -> &str {
        match self {
            Self::Media(s) => s.id(),
            Self::Intel(s) => s.id(),
            Self::Osint(s) => s.id(),
            Self::Llm(s) => s.id(),
            Self::Crawl(s) => s.id(),
            Self::Local(s) => s.id(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Media(s) => s.name(),
            Self::Intel(s) => s.name(),
            Self::Osint(s) => s.name(),
            Self::Llm(s) => s.name(),
            Self::Crawl(s) => s.name(),
            Self::Local(s) => s.name(),
        }
    }

    pub fn domains(&self) -> Vec<SourceDomain> {
        match self {
            Self::Media(s) => s.domains(),
            Self::Intel(s) => s.domains(),
            Self::Osint(s) => s.domains(),
            Self::Llm(s) => s.domains(),
            Self::Crawl(s) => s.domains(),
            Self::Local(s) => s.domains(),
        }
    }
}

// ============================================================================
// 注册表 — 统一管理所有源
// ============================================================================

/// 数据源注册表
pub struct DataSourceRegistry {
    sources: Vec<AnyDataSource>,
}

impl DataSourceRegistry {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// 注册媒体源
    pub fn register_media(&mut self, source: Arc<dyn MediaSource>) {
        self.sources.push(AnyDataSource::Media(source));
    }

    /// 注册情报源
    pub fn register_intel(&mut self, source: Arc<dyn IntelSource>) {
        self.sources.push(AnyDataSource::Intel(source));
    }

    /// 注册 OSINT 源
    pub fn register_osint(&mut self, source: Arc<dyn OsintSource>) {
        self.sources.push(AnyDataSource::Osint(source));
    }

    /// 注册 LLM 提供者
    pub fn register_llm(&mut self, source: Arc<dyn LlmProvider>) {
        self.sources.push(AnyDataSource::Llm(source));
    }

    /// 注册爬虫源
    pub fn register_crawl(&mut self, source: Arc<dyn CrawlSource>) {
        self.sources.push(AnyDataSource::Crawl(source));
    }

    /// 注册本地源
    pub fn register_local(&mut self, source: Arc<dyn LocalSource>) {
        self.sources.push(AnyDataSource::Local(source));
    }

    /// 按域获取媒体源
    pub fn media_sources(&self, domain: &SourceDomain) -> Vec<&Arc<dyn MediaSource>> {
        self.sources
            .iter()
            .filter_map(|s| match s {
                AnyDataSource::Media(m) if m.domains().contains(domain) => Some(m),
                _ => None,
            })
            .collect()
    }

    /// 按域获取情报源
    pub fn intel_sources(&self, domain: &SourceDomain) -> Vec<&Arc<dyn IntelSource>> {
        self.sources
            .iter()
            .filter_map(|s| match s {
                AnyDataSource::Intel(i) if i.domains().contains(domain) => Some(i),
                _ => None,
            })
            .collect()
    }

    /// 按域获取 OSINT 源
    pub fn osint_sources(&self, domain: &SourceDomain) -> Vec<&Arc<dyn OsintSource>> {
        self.sources
            .iter()
            .filter_map(|s| match s {
                AnyDataSource::Osint(o) if o.domains().contains(domain) => Some(o),
                _ => None,
            })
            .collect()
    }

    /// 按域获取 LLM 提供者
    pub fn llm_providers(&self, domain: &SourceDomain) -> Vec<&Arc<dyn LlmProvider>> {
        self.sources
            .iter()
            .filter_map(|s| match s {
                AnyDataSource::Llm(l) if l.domains().contains(domain) => Some(l),
                _ => None,
            })
            .collect()
    }

    /// 按域获取爬虫源
    pub fn crawl_sources(&self, domain: &SourceDomain) -> Vec<&Arc<dyn CrawlSource>> {
        self.sources
            .iter()
            .filter_map(|s| match s {
                AnyDataSource::Crawl(c) if c.domains().contains(domain) => Some(c),
                _ => None,
            })
            .collect()
    }

    /// 获取所有源
    pub fn all(&self) -> &[AnyDataSource] {
        &self.sources
    }

    /// 按 ID 查找
    pub fn find(&self, id: &str) -> Option<&AnyDataSource> {
        self.sources.iter().find(|s| s.id() == id)
    }

    /// 按域过滤所有源
    pub fn by_domain(&self, domain: &SourceDomain) -> Vec<&AnyDataSource> {
        self.sources
            .iter()
            .filter(|s| s.domains().contains(domain))
            .collect()
    }

    /// 按多个域过滤
    pub fn by_domains(&self, domains: &[SourceDomain]) -> Vec<&AnyDataSource> {
        self.sources
            .iter()
            .filter(|s| {
                let src_domains = s.domains();
                domains.iter().any(|d| src_domains.contains(d))
            })
            .collect()
    }
}

impl Default for DataSourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockMediaSource {
        id: String,
    }

    impl DataSource for MockMediaSource {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            &self.id
        }
        fn domains(&self) -> Vec<SourceDomain> {
            vec![SourceDomain::Media]
        }
    }

    impl MediaSource for MockMediaSource {
        fn search(
            &self,
            _query: &str,
            _page: u32,
        ) -> Pin<Box<dyn Future<Output = Result<MediaSearchResult, String>> + Send>> {
            Box::pin(async {
                Ok(MediaSearchResult {
                    items: vec![],
                    total: 0,
                    page: 1,
                })
            })
        }
    }

    struct MockIntelSource {
        id: String,
    }

    impl DataSource for MockIntelSource {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            &self.id
        }
        fn domains(&self) -> Vec<SourceDomain> {
            vec![SourceDomain::Intel]
        }
    }

    impl IntelSource for MockIntelSource {
        fn fetch(
            &self,
            _query: &str,
        ) -> Pin<Box<dyn Future<Output = Result<IntelResult, String>> + Send>> {
            Box::pin(async {
                Ok(IntelResult {
                    items: vec![],
                    total: 0,
                    source_id: self.id.clone(),
                })
            })
        }
    }

    #[tokio::test]
    async fn test_registry_media_sources() {
        let mut registry = DataSourceRegistry::new();
        registry.register_media(Arc::new(MockMediaSource { id: "netease".into() }));
        registry.register_media(Arc::new(MockMediaSource { id: "kuwo".into() }));
        registry.register_intel(Arc::new(MockIntelSource { id: "gdelt".into() }));

        let media = registry.media_sources(&SourceDomain::Media);
        assert_eq!(media.len(), 2);

        let intel = registry.intel_sources(&SourceDomain::Intel);
        assert_eq!(intel.len(), 1);
    }

    #[tokio::test]
    async fn test_registry_find() {
        let mut registry = DataSourceRegistry::new();
        registry.register_media(Arc::new(MockMediaSource { id: "netease".into() }));

        assert!(registry.find("netease").is_some());
        assert!(registry.find("unknown").is_none());
    }

    #[tokio::test]
    async fn test_registry_by_domain() {
        let mut registry = DataSourceRegistry::new();
        registry.register_media(Arc::new(MockMediaSource { id: "netease".into() }));
        registry.register_intel(Arc::new(MockIntelSource { id: "gdelt".into() }));

        let all = registry.by_domain(&SourceDomain::Media);
        assert_eq!(all.len(), 1);
    }
}

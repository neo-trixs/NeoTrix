use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 统一来源类型 — 所有外部探索模态共用
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExplorationSourceType {
    BrowserSocial,
    BrowserWeb,
    ApiGithub,
    ApiOpenLibrary,
    ApiWikipedia,
    WebSearch,
    FileSystem,
    Sensor,
    CrawlQueue,
    InternalKnowledge,
    System,
    /// 论文数据库 (arXiv API / Semantic Scholar / HF datasets)
    PaperDatabase,
    /// PDF文档 (本地文件提取)
    PdfDocument,
}

impl ExplorationSourceType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BrowserSocial => "browser_social",
            Self::BrowserWeb => "browser_web",
            Self::ApiGithub => "api_github",
            Self::ApiOpenLibrary => "api_openlibrary",
            Self::ApiWikipedia => "api_wikipedia",
            Self::WebSearch => "web_search",
            Self::FileSystem => "file_system",
            Self::Sensor => "sensor",
            Self::CrawlQueue => "crawl_queue",
            Self::InternalKnowledge => "internal_kb",
            Self::System => "system",
            Self::PaperDatabase => "paper_database",
            Self::PdfDocument => "pdf_document",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Self::BrowserSocial => 0.7,
            Self::BrowserWeb => 0.6,
            Self::ApiGithub => 0.9,
            Self::ApiOpenLibrary => 0.8,
            Self::ApiWikipedia => 0.85,
            Self::WebSearch => 0.75,
            Self::FileSystem => 0.5,
            Self::Sensor => 0.3,
            Self::CrawlQueue => 0.65,
            Self::InternalKnowledge => 0.4,
            Self::System => 0.35,
            Self::PaperDatabase => 0.88,
            Self::PdfDocument => 0.75,
        }
    }
}

/// 统一内容块 — 任何外部探索的标准化输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceContent {
    pub id: String,
    pub text: String,
    pub title: String,
    pub source_type: ExplorationSourceType,
    pub url: Option<String>,
    pub author: Option<String>,
    pub timestamp: u64,
    pub engagement: Engagement,
    pub metadata: HashMap<String, String>,
}

impl SourceContent {
    pub fn new(
        id: impl Into<String>,
        text: impl Into<String>,
        source_type: ExplorationSourceType,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            title: String::new(),
            source_type,
            url: None,
            author: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            engagement: Engagement::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn with_engagement(mut self, engagement: Engagement) -> Self {
        self.engagement = engagement;
        self
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// 互动信号 (社交权重)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Engagement {
    pub likes: u64,
    pub shares: u64,
    pub replies: u64,
    pub views: Option<u64>,
}

impl Default for Engagement {
    fn default() -> Self {
        Self {
            likes: 0,
            shares: 0,
            replies: 0,
            views: None,
        }
    }
}

/// 负熵评分结果 — 与来源类型无关
#[derive(Debug, Clone)]
pub struct NegentropyScore {
    pub content: SourceContent,
    pub information_gain: f64,
    pub novelty: f64,
    pub relevance: f64,
    pub signal_purity: f64,
    pub negentropy: f64,
}

impl NegentropyScore {
    pub fn is_worth_absorbing(&self) -> bool {
        self.negentropy > 0.25
    }

    pub fn to_ingestion_text(&self) -> String {
        let src = self.content.source_type.name();
        let tag = self.content.author.as_deref().unwrap_or("unknown");
        format!(
            "[{}/{}] {} (N={:.3})",
            src,
            tag,
            self.content.text.chars().take(500).collect::<String>(),
            self.negentropy,
        )
    }
}

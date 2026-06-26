//! 统一爬虫引擎
//!
//! 取代分散的 nt_world_scrape.rs / tor_crawler.rs / web_miner.rs / KnowledgeChain / ExplorationPipeline
//! 统一为一个管道: SeedManager → DualQueueFrontier → FetcherPool → ContentClassifier → KnowledgeMapper → Absorption
//!
//! 核心特征:
//! - Mercator-style 双队列 Frontier (优先级+按域限速)
//! - WebOrganizer-inspired 二维分类 (Topic + Format)
//! - 自愈循环: 每 N 次迭代分析错误 → 元认知 → 调整策略
//! - 12h 周期调度 (通过 BackgroundLoop ticker)

pub mod adaptive;
pub mod classifier;
pub mod config;
pub mod data_connector;
pub mod enrichment;
pub mod fetcher;
pub mod frontier;
pub mod github_trending;
pub mod mapper;
pub mod research_scanner;
pub mod spider;
pub mod stealth;
#[cfg(feature = "stealth-browser")]
pub mod stealth_browser;
pub mod unified;

pub use classifier::{ClassifiedContent, ClassifierSummary, ContentClassifier};
pub use config::{
    default_seed_urls, CrawlFormat, CrawlStrategy, CrawlTopic, CrawlerConfig, SeedEntry,
};
pub use fetcher::{FetchError, FetchResult, FetcherPool, FetcherProtocol, FetcherSummary};
pub use frontier::{extract_domain, extract_links, DualQueueFrontier, FrontierStats, UrlEntry};
pub use mapper::{KnowledgeMapper, MappedKnowledge, MapperSummary};
pub use unified::{CrawlerSummary, CycleResult, HealAction, UnifiedCrawler};

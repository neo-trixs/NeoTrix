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

pub mod config;
pub mod frontier;
pub mod fetcher;
pub mod classifier;
pub mod mapper;
pub mod adaptive;
pub mod spider;
pub mod stealth;
pub mod unified;
pub mod data_connector;
pub mod enrichment;
pub mod research_scanner;

pub use config::{CrawlerConfig, CrawlStrategy, CrawlTopic, CrawlFormat, SeedEntry, default_seed_urls};
pub use frontier::{DualQueueFrontier, UrlEntry, FrontierStats, extract_domain, extract_links};
pub use fetcher::{FetcherPool, FetchResult, FetchError, FetcherProtocol, FetcherSummary};
pub use classifier::{ContentClassifier, ClassifiedContent, ClassifierSummary};
pub use mapper::{KnowledgeMapper, MappedKnowledge, MapperSummary};
pub use unified::{UnifiedCrawler, CrawlerSummary, CycleResult, HealAction};

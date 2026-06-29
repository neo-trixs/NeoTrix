//! Unified exploration pipeline — external knowledge absorption
//!
//! Sub-modules:
//! - types:     Core type definitions (enums, result structs, stats)
//! - discovery: Auto-discovery from web content + goal generation
//! - processing: Domain processing + round execution

pub mod discovery;
pub mod processing;
pub mod types;

#[cfg(test)]
pub mod tests;

use chrono::Utc;
use lru::LruCache;
use std::collections::{HashMap, HashSet, VecDeque};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::Duration;

use super::knowledge_engine::KnowledgeEngine;
use super::knowledge_miner::KnowledgeMiner;
use super::web_miner::WebKnowledgeMiner;

pub use types::*;

pub use super::exploration_seeds::seed_urls_by_domain;

/// 统一探索管道 — 单入口处理所有外部知识吸收, 自动重试+发现+目标构建
pub struct ExplorationPipeline {
    pub work_dir: PathBuf,
    pub web_miner: WebKnowledgeMiner,
    pub knowledge_miner: KnowledgeMiner,
    pub knowledge_engine: KnowledgeEngine,
    pub seed_queue: VecDeque<(ExploreDomain, Vec<String>)>,
    pub processed: HashSet<String>,
    pub failed: HashMap<String, u32>,
    pub max_retries: u32,
    pub round_count: u64,
    pub domain_interval: HashMap<ExploreDomain, u64>,
    pub last_crawl: HashMap<ExploreDomain, i64>,
    /// 自动发现的 URL 缓存（来自已抓取内容的 Wikipedia 链接）
    pub auto_discovered: HashSet<String>,
    /// LRU 结果缓存：URL → CachedExploreResult
    pub explore_cache: LruCache<String, CachedExploreResult>,
    /// 缓存命中/未命中计数
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl ExplorationPipeline {
    pub fn new(work_dir: PathBuf) -> Self {
        let web_miner = WebKnowledgeMiner::new(work_dir.clone());
        let knowledge_miner = KnowledgeMiner::new(work_dir.clone());
        let ke_path = work_dir.join("knowledge_engine.json");
        let knowledge_engine = KnowledgeEngine::load_from(&ke_path);
        let mut domain_interval = HashMap::new();
        domain_interval.insert(ExploreDomain::Parapsychology, 3600);
        domain_interval.insert(ExploreDomain::Theology, 3600);
        domain_interval.insert(ExploreDomain::EsotericStudies, 3600);
        domain_interval.insert(ExploreDomain::Wiki, 3600);
        domain_interval.insert(ExploreDomain::Papers, 7200);
        domain_interval.insert(ExploreDomain::GitHub, 21600);
        domain_interval.insert(ExploreDomain::General, 36000);
        domain_interval.insert(ExploreDomain::Consciousness, 3600);
        domain_interval.insert(ExploreDomain::RustML, 7200);
        domain_interval.insert(ExploreDomain::Security, 7200);
        domain_interval.insert(ExploreDomain::MathPhysics, 3600);

        Self {
            work_dir,
            web_miner,
            knowledge_miner,
            knowledge_engine,
            seed_queue: VecDeque::new(),
            processed: HashSet::new(),
            failed: HashMap::new(),
            max_retries: 3,
            round_count: 0,
            domain_interval,
            last_crawl: HashMap::new(),
            auto_discovered: HashSet::new(),
            explore_cache: LruCache::new(NonZeroUsize::new(1000).expect("1000 > 0: NonZeroUsize conversion failed, expected positive value")),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// 根据 URL 返回缓存 TTL：Wikipedia=1h, GitHub=6h, 其余=1h
    pub fn cache_ttl_for_url(&self, url: &str) -> Duration {
        let lower = url.to_lowercase();
        if lower.contains("github.com") {
            Duration::from_secs(21600)
        } else if lower.contains("wikipedia.org") || lower.contains("wikidata.org") {
            Duration::from_secs(3600)
        } else {
            Duration::from_secs(3600)
        }
    }

    /// 统一入口：接受任意 URL/来源，分类并入队
    pub fn ingest(&mut self, url: &str, domain: Option<ExploreDomain>) {
        let effective_domain = domain.unwrap_or_else(|| {
            let src = UnifiedKnowledgeSourceType::detect(url);
            match src {
                UnifiedKnowledgeSourceType::GitHub => ExploreDomain::GitHub,
                UnifiedKnowledgeSourceType::ArXiv => ExploreDomain::Papers,
                UnifiedKnowledgeSourceType::Wikipedia => ExploreDomain::Wiki,
                _ => ExploreDomain::General,
            }
        });
        if !self.processed.contains(url) && !self.auto_discovered.contains(url) {
            self.seed_queue
                .push_back((effective_domain, vec![url.to_string()]));
        }
    }

    fn should_crawl(&self, domain: ExploreDomain) -> bool {
        let now = Utc::now().timestamp();
        let last = self.last_crawl.get(&domain).copied().unwrap_or(0);
        let interval = self.domain_interval.get(&domain).copied().unwrap_or(3600);
        (now - last) >= interval as i64
    }

    /// 入队一个域的种子 URL
    pub fn enqueue_domain(&mut self, domain: ExploreDomain) -> usize {
        let urls = seed_urls_by_domain(domain);
        let fresh: Vec<String> = urls
            .into_iter()
            .filter(|u| !self.processed.contains(u) && !self.auto_discovered.contains(u))
            .collect();
        let count = fresh.len();
        if count > 0 {
            self.seed_queue.push_back((domain, fresh));
        }
        count
    }

    fn has_pending_for(&self, domain: ExploreDomain) -> bool {
        self.seed_queue.iter().any(|(d, _)| *d == domain)
    }

    /// 外部 URL 注入：从 SelfEvolver/用户输入等接收新 URL
    pub fn ingest_url(&mut self, url: &str) {
        self.ingest(url, None);
    }

    /// LRU 缓存统计
    pub fn cache_stats(&self) -> CacheStats {
        let total = self.cache_hits + self.cache_misses;
        let hit_rate = if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        };
        CacheStats {
            size: self.explore_cache.len(),
            capacity: self.explore_cache.cap().get(),
            hits: self.cache_hits,
            misses: self.cache_misses,
            hit_rate,
        }
    }

    /// 统计
    pub fn stats(&self) -> PipelineStats {
        PipelineStats {
            rounds: self.round_count,
            web_mined: self.web_miner.mined_history.len(),
            gh_mined: self.knowledge_miner.mined_sources.len(),
            ke_entries: self.knowledge_engine.stats().total_entries,
            queued: self.seed_queue.len(),
            processed: self.processed.len(),
            failed: self.failed.len(),
            auto_discovered: self.auto_discovered.len(),
        }
    }
}

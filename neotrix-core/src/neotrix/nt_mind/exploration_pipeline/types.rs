use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::super::web_miner::WebMinedKnowledge;

/// 统一来源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnifiedKnowledgeSourceType {
    Wikipedia,
    ArXiv,
    GitHub,
    GenericWeb,
    KnowledgeBase,
    SeedDomain,
}

impl UnifiedKnowledgeSourceType {
    pub fn detect(url: &str) -> Self {
        let lower = url.to_lowercase();
        if lower.contains("wikipedia.org") || lower.contains("wikidata.org") {
            UnifiedKnowledgeSourceType::Wikipedia
        } else if lower.contains("arxiv.org") || lower.contains("semanticscholar.org") {
            UnifiedKnowledgeSourceType::ArXiv
        } else if lower.contains("github.com") {
            UnifiedKnowledgeSourceType::GitHub
        } else {
            UnifiedKnowledgeSourceType::GenericWeb
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExploreDomain {
    Parapsychology,
    Theology,
    EsotericStudies,
    Wiki,
    Papers,
    GitHub,
    General,
    Consciousness,
    RustML,
    Security,
    MathPhysics,
}

impl ExploreDomain {
    pub fn name(&self) -> &'static str {
        match self {
            ExploreDomain::Parapsychology => "parapsychology",
            ExploreDomain::Theology => "theology",
            ExploreDomain::EsotericStudies => "esoteric-studies",
            ExploreDomain::Wiki => "wiki",
            ExploreDomain::Papers => "papers",
            ExploreDomain::GitHub => "github",
            ExploreDomain::General => "general",
            ExploreDomain::Consciousness => "consciousness",
            ExploreDomain::RustML => "rust-ml",
            ExploreDomain::Security => "nt_shield",
            ExploreDomain::MathPhysics => "math-physics",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExploreRoundResult {
    pub domains_processed: Vec<ExploreDomain>,
    pub total_mined: usize,
    pub total_absorbed: usize,
    pub total_reward: f64,
    pub ke_entries_added: usize,
    pub new_discoveries: usize,
    pub goals_generated: usize,
    pub details: Vec<String>,
}

/// 缓存条目：包含挖掘结果和获取时间戳
#[derive(Debug, Clone)]
pub struct CachedExploreResult {
    pub result: WebMinedKnowledge,
    pub fetched_at: Instant,
}

/// 缓存统计
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub rounds: u64,
    pub web_mined: usize,
    pub gh_mined: usize,
    pub ke_entries: usize,
    pub queued: usize,
    pub processed: usize,
    pub failed: usize,
    pub auto_discovered: usize,
}

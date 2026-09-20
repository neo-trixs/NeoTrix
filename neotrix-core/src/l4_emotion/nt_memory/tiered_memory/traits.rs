#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Memory tier classification — maps to the mem0-inspired 3-tier model.
///
/// - **Tier1Core**: Always in LLM context (2-5K chars). User identity, active goals,
///   current project state. Write-mostly, read-every-turn.
/// - **Tier2Archival**: Vector-backed storage. Retrieved on-demand via embedding
///   similarity. Long-term knowledge, documents, past analysis.
/// - **Tier3Recall**: Chronological conversation history. Keyword-indexed,
///   time-ordered. Retrieved via recency or keyword match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryTier {
    Tier1Core,
    Tier2Archival,
    Tier3Recall,
}

impl MemoryTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tier1Core => "core",
            Self::Tier2Archival => "archival",
            Self::Tier3Recall => "recall",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "core" => Self::Tier1Core,
            "archival" => Self::Tier2Archival,
            "recall" => Self::Tier3Recall,
            _ => Self::Tier3Recall,
        }
    }
}

/// A memory item stored in any tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    pub tier: MemoryTier,
    pub importance: f64,
    pub confidence: f64,
    pub timestamp: i64,
    pub access_count: u64,
    pub tags: Vec<String>,
    pub source: String,
    /// Optional embedding vector (populated for Tier2Archival).
    #[serde(default)]
    pub embedding: Vec<f64>,
}

impl MemoryItem {
    pub fn new(
        id: String,
        content: String,
        tier: MemoryTier,
        importance: f64,
        source: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        Self {
            id,
            content,
            tier,
            importance: importance.clamp(0.0, 1.0),
            confidence: 0.8,
            timestamp: now,
            access_count: 0,
            tags: Vec::new(),
            source,
            embedding: Vec::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_embedding(mut self, embedding: Vec<f64>) -> Self {
        self.embedding = embedding;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn record_access(&mut self) {
        self.access_count += 1;
    }

    /// Content length in characters — used for Tier1 budget enforcement.
    pub fn char_len(&self) -> usize {
        self.content.len()
    }
}

/// Query for memory retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub text: String,
    pub embedding: Vec<f64>,
    pub tier_filter: Vec<MemoryTier>,
    pub max_results: usize,
    pub min_importance: f64,
    /// If true, only return results from Tier1 + Tier2 (skip recall unless
    /// core+archival are insufficient).
    pub prefer_concise: bool,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            embedding: Vec::new(),
            tier_filter: vec![MemoryTier::Tier1Core, MemoryTier::Tier2Archival],
            max_results: 10,
            min_importance: 0.0,
            prefer_concise: true,
        }
    }
}

/// A single retrieval result with source tier and relevance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResult {
    pub item: MemoryItem,
    pub relevance: f64,
    pub source_tier: MemoryTier,
}

/// Retrieval statistics for observability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetrievalStats {
    pub tier1_hits: usize,
    pub tier2_hits: usize,
    pub tier3_hits: usize,
    pub total_candidates: usize,
    pub escalated_to_tier3: bool,
}

/// Trait unifying access across all 3 tiers — the single entry point for
/// memory operations (R-P116 3-Tier memory).
#[async_trait::async_trait]
pub trait MemoryHub: Send + Sync {
    /// Write an item to the appropriate tier (router decides which tier).
    async fn write(&self, item: MemoryItem) -> Result<(), String>;

    /// Write directly to a specific tier (bypass router).
    async fn write_to_tier(&self, item: MemoryItem, tier: MemoryTier) -> Result<(), String>;

    /// Retrieve items matching a query. The hub cascades through tiers:
    /// Tier1 → Tier2 → Tier3 (on-demand escalation).
    async fn retrieve(&self, query: &MemoryQuery) -> Result<Vec<MemoryResult>, String>;

    /// Retrieve only from a specific tier.
    async fn retrieve_from_tier(
        &self,
        query: &MemoryQuery,
        tier: MemoryTier,
    ) -> Result<Vec<MemoryResult>, String>;

    /// Update an item (replaces content, resets timestamp).
    async fn update(&self, id: &str, content: &str, importance: f64) -> Result<(), String>;

    /// Delete an item from all tiers.
    async fn delete(&self, id: &str) -> Result<(), String>;

    /// Enforce capacity budget — evicts lowest-importance items from Tier1.
    async fn enforce_budget(&self) -> Result<usize, String>;

    /// Snapshot current Tier1 content as a formatted string for LLM context injection.
    async fn snapshot_for_context(&self) -> Result<String, String>;

    /// Aggregate stats across all tiers.
    fn stats(&self) -> HubStats;
}

/// Hub-level statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HubStats {
    pub tier1_count: usize,
    pub tier1_chars: usize,
    pub tier1_budget: usize,
    pub tier2_count: usize,
    pub tier3_count: usize,
    pub total_writes: u64,
    pub total_retrievals: u64,
    pub budget_evictions: u64,
}

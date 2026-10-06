#![deny(clippy::unwrap_used)]


pub mod process_skill_memory;
pub mod bloom_filter;
pub mod bm25;
pub mod cognitive_graph;
pub mod file_centric_state;
pub mod knowledge_storage;
pub mod memory_orchestrator;
pub mod memory_palace;
pub mod nt_absorb_mapper;
pub mod nt_discovery_github_topics;
pub mod nt_discovery_orchestrator;
pub mod nt_discovery_sources;
pub mod nt_field_ledger;
pub mod nt_http;
pub mod nt_memory_adaptive_rag;
pub mod nt_memory_agent_driven;
pub mod nt_memory_agent_session;
pub mod nt_memory_api;
pub mod nt_memory_blocks;
pub mod nt_memory_brain;
pub mod nt_memory_coeffect;
pub mod nt_memory_commit_tracker;
pub mod nt_memory_commitment;
pub mod nt_memory_community;
pub mod nt_memory_confidence;
pub mod nt_memory_cortex_sync;
pub mod nt_memory_crawl;
pub mod nt_memory_curation;
pub mod nt_memory_decompose;
pub mod nt_memory_distill;
pub mod nt_memory_diversity;
pub mod nt_memory_dual_brain;
// 检索准入闸（2026-10-03，吸收 waku-agent 的 retrieval_gate 设计）：
// 「**这条消息需要记忆吗**」在**碰存储之前**回答 ⇒ 治「过度检索偏置答案」。
pub mod nt_retrieval_gate;
pub mod nt_memory_e8_agent;
pub mod nt_memory_embed;
pub mod nt_memory_feedback;
pub mod nt_memory_galaxy_hygiene;
pub mod nt_memory_geo;
pub mod nt_memory_graph;
pub mod nt_memory_graph_cache;
pub mod nt_memory_graphrag;
pub mod nt_memory_gwt_router;
pub mod nt_memory_gwtq;
pub mod nt_memory_hierarchical;
pub mod nt_memory_integration;
pub mod nt_memory_knowledge_assets;
pub mod nt_memory_lifecycle;
pub mod nt_memory_pack;
pub mod nt_memory_pack_chunked;
pub mod nt_memory_pipeline;
pub mod nt_memory_proficiency;
pub mod nt_memory_provenance;
pub mod nt_memory_resource_ingest;
pub mod nt_memory_schema;
pub mod nt_memory_search;
pub mod nt_memory_seed;
pub mod nt_memory_shanhai;
pub mod nt_memory_skill_cost;
pub mod nt_memory_snapshot;
pub mod nt_memory_store;
pub mod nt_memory_svaf_gate;
pub mod nt_memory_tech_reserve;
pub mod nt_memory_types;
pub mod nt_memory_unify;
pub mod nt_memory_visibility;
pub mod nt_memory_vsa_expand;
pub mod nt_memory_weave;
pub mod nt_memory_wiki;
pub mod nt_memory_write_guard;
pub mod nt_memory_write_guard_fallback;
pub mod nt_normalizer;
pub mod privacy;
pub mod retrieval_fusion;
pub mod shared_utils;
pub mod skill_glows;
pub mod spill_storage;
pub mod user_memory;
pub mod vector_adapter;
pub mod vector_index;

pub use nt_discovery_github_topics::{DiscoveryPipelineConfig, GithubDiscoveryStats};
pub use nt_discovery_orchestrator::{DiscoveryCycleConfig, DiscoveryCycleReport};
pub use nt_memory_adaptive_rag::RelevanceGrade;
pub use nt_memory_agent_driven::{
    AgentMemory, AgentMemoryEntry, MemoryConfig, MemoryStats, MemoryTier,
};
pub use nt_memory_agent_session::{AgentSession, AgentSessionEntry, AgentSessionManager};
pub use nt_memory_commitment::EmbeddingCommitmentStore;
pub use nt_memory_community::{
    CommunityAwareSearch, CommunityDetector, CommunityQueryMode, CommunityResult,
};
pub use nt_memory_confidence::{
    search_with_confidence, ConfidenceStore, ConfidenceWeights, DecayConfig, RetrievalStrategy,
    UncertainResult,
};
pub use nt_memory_decompose::{decompose_query, merge_results, Decomposition};
pub use nt_memory_e8_agent::{E8AgentConfig, E8AgentLoop, E8AgentResult, E8Phase};
pub use nt_memory_embed::EmbeddingConfig;
pub use nt_memory_feedback::{FeedbackSignal, FeedbackStore, StrategyStats};
pub use nt_memory_gwt_router::{
    extract_features, GwtRouter, GwtRouterConfig, QueryFeatures, QueryIntent, RetrievalChannel,
};
pub use nt_memory_proficiency::{
    MemoryAction, MemoryActionRecord, MemoryProficiency, MemoryProficiencyReport,
};
pub use nt_memory_store::*;
pub use nt_memory_svaf_gate::{SvafDecision, SvafEvaluation, SvafGate};
pub use nt_memory_types::*;
pub use nt_memory_vsa_expand::VsaAssociativeExpander;
pub use privacy::{PrivacyConfig, PrivacyEnforcer, PrivacyMode};
pub use user_memory::UserMemory;
pub use vector_adapter::KbVectorAdapter;
// pub use nt_memory_primitives::MemoryPrimitives; // DEAD: zero external references
pub use knowledge_storage::{migrate_from_json, KnowledgeStorage};
pub use nt_absorb_mapper::{
    apply_mappings, map_all_nodes, map_batch_nodes, map_node, map_nodes, map_source_core,
    CapabilityMapping, MappingReport,
};
pub use nt_memory_dual_brain::{
    DualBrainWorkingMemory, ExperienceAnchor, DEFAULT_WORKING_CAPACITY,
};
pub use nt_memory_graphrag::{
    Community, EntityGraph, EntityNode, GlobalSummary, GraphQueryMode, GraphRagConfig,
    GraphRagStore, HybridResult, RelationEdge, SubgraphResult,
};
pub use nt_memory_snapshot::{
    diff_snapshots, snapshot_from_file, snapshot_kb, snapshot_to_file, DiffEdge, DiffNode, KbDiff,
    KbSnapshot, SNAPSHOT_FORMAT, SNAPSHOT_VERSION,
};
pub use nt_memory_tech_reserve::{
    extract_tech_domains, ArchitectureGap, TechProfile, TechReserveDimension, TechReserveEntry,
    TechReserveQuery, TechReserveStore,
};
pub use nt_memory_wiki::{WikiEdge, WikiGraph, WikiNode, WikiSearchResult, WikiSyncReport};
pub use nt_memory_write_guard::{
    kb_write_guard, record_write_evidence, WriteGuardVerdict, WRITE_GUARD_NS,
};
pub use nt_memory_write_guard_fallback::{
    check_verbatim_dup, deterministic_fallback, has_hold_conflict,
    normalize_content as normalize_for_dedup, verbatim_content_fingerprint, ContentFingerprint,
    DeterministicVerdict, VerbatimDupResult,
};
pub use nt_normalizer::{
    compute_quality_score, content_fingerprint, detect_language, extract_key_sections,
    normalize_lang, normalize_text, strip_markdown, validate_node_type, validate_relation_type,
};
// pub use nt_memory_zim_absorber::{ZimAbsorbConfig, ZimAbsorbStats}; // DEAD wave2
pub use nt_memory_search::{build_materialized_neighbors, MaterializedNeighborCache};

use rusqlite::Connection;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::sync::RwLock;

use lru::LruCache;

use bm25::Bm25Index;
use nt_memory_adaptive_rag::AdaptiveRetrieval;
use nt_memory_crawl::CrawlCycleReport;

use crate::l4_emotion::nt_memory::nt_memory_historian::{TemporalFact, TemporalFactLedger};

pub mod kb_core;
pub mod kb_locks;
pub mod kb_trust;
pub mod kb_vector;
pub mod kb_nodes;
pub mod kb_write;
pub mod kb_catalog;
pub mod kb_search;
pub mod kb_agent;
pub mod kb_govern;
pub mod kb_graphrag;
pub mod kb_graph;
pub mod kb_ingest;
pub mod kb_kv;
pub mod kb_galaxy;
pub mod kb_session;
pub mod kb_assets;
pub mod kb_evolution;

pub struct KnowledgeBase {
    pub(crate) conn: Mutex<Connection>,
    pub db_path: PathBuf,
    pub(crate) db_file: Option<File>,
    /// 跟踪文件锁状态，避免 macOS flock() 不可重入死锁
    pub(crate) file_lock_held: AtomicBool,
    pub bm25: RwLock<Option<Bm25Index>>,
    pub bm25_dirty: RwLock<bool>,
    pub embedding_config: RwLock<Option<EmbeddingConfig>>,
    pub graph_cache: RwLock<nt_memory_graph_cache::GraphCache>,
    pub fused_cache: Mutex<LruCache<String, Vec<SearchResult>>>,
    pub adaptive: AdaptiveRetrieval,
    pub commitment_store: RwLock<EmbeddingCommitmentStore>,
    pub confidence_store: RwLock<ConfidenceStore>,
    pub community_search: RwLock<CommunityAwareSearch>,
    pub privacy: RwLock<PrivacyEnforcer>,
    pub vector_adapter: RwLock<Option<KbVectorAdapter>>,
    pub agent_memory: RwLock<AgentMemory>,
    pub agent_session: RwLock<bool>,
    pub svaf_gate: RwLock<SvafGate>,
    pub proficiency: RwLock<MemoryProficiency>,
    pub graphrag_store: RwLock<Option<GraphRagStore>>,
    pub tech_reserve: RwLock<TechReserveStore>,
    pub skills_library: RwLock<nt_memory_knowledge_assets::SkillsLibrary>,
    pub feedback_store: RwLock<FeedbackStore>,
    pub gwt_router: RwLock<GwtRouter>,
    pub vsa_expander: RwLock<VsaAssociativeExpander>,
    /// 检索自进化 (SimpleMem EvolveMem absorb, G4): 每次检索记录质量,
    /// 周期性 Diagnose→Propose→Guard 提交召回调参, 影响后续扩召深度。
    pub retrieval_evolver: RwLock<nt_memory_search::RetrievalEvolver>,
    /// 时序事实账本 (TemporalFactLedger 接线, R-P79): 节点写入/更正自动记
    /// temporal_facts, 知识变更获得 append-only + supersede + point-in-time 语义。
    pub temporal_ledger: Mutex<TemporalFactLedger>,
    /// Unified memory lifecycle orchestrator — coordinates ForgettingCurve,
    /// FreshnessLedger, and ConfidenceStore decay into a single interface.
    pub lifecycle: RwLock<nt_memory_lifecycle::MemoryLifecycle>,
    /// 吸收文本毒化扫描器 (L3 self_poison trait 抽象, 消除 L1→L3 直接依赖)。
    pub absorb_scanner:
        RwLock<Option<Box<dyn crate::l0_substrate::nt_core_traits::AbsorbTextScanner>>>,
    /// 可验证回放收据发射器 (L3 AgentReceipt trait 抽象, 消除 L1→L3 直接依赖)。
    pub receipt_emitter:
        RwLock<Option<Box<dyn crate::l0_substrate::nt_core_traits::ReceiptEmitter>>>,
    // 2026-10-04 **接线**：检索准入门（`nt_retrieval_gate`）。
    //
    // 为什么现在才接（这是一条「建成未用」的清账）：
    // 该门自 `8aac2fc6` 落地起 **零生产消费者**
    // （`decide_or_admit` / `RetrievalQuestion` / `FailOpenGate`
    //  全仓外部引用均为 0；`impl RetrievalGate` 只有测试里的
    //  `Broken` / `Skipper`）⇒ **检索能力已上线（kb_search
    // 的 `search_local` 是唯一真入口），门却没装**。
    //
    // **默认 `NoGate` ⇒ 零行为变化**，这是刻意的：
    // 门自己的文档写明「可插拔，**默认不改变任何行为**」，
    // 而我方**没有实测阈值**（`LESSONS-20260929-checked-is-not-verified.md`
    // 警告过「塞一个未测的启发式」）。⇒ **先接线、后测量**。
    //
    // `Arc` 而非 `RwLock`：闸是**只读判定**，换闸不需要
    // 与检索并发协调。
    pub(crate) retrieval_gate: std::sync::Arc<dyn nt_retrieval_gate::RetrievalGate>,
}

impl std::fmt::Debug for KnowledgeBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KnowledgeBase")
            .field("db_path", &self.db_path)
            .field("bm25_dirty", &self.bm25_dirty)
            .field("embedding_config", &self.embedding_config)
            .field("graph_cache", &self.graph_cache)
            .field("skills_library", &self.skills_library)
            .finish()
    }
}


impl Drop for KnowledgeBase {
    fn drop(&mut self) {
        if let Some(ref f) = self.db_file {
            let _ = f.unlock();
            log::info!("[KB] released file lock on knowledge.db");
        }
    }
}

/// 实现 core::nt_core_kb_primitives::KvStore — 让 L6 元认知层通过 trait 访问 KV 存储,
/// 而非直接依赖 `KnowledgeBase` 具体类型。
impl crate::l0_substrate::nt_core_kb_primitives::KvStore for KnowledgeBase {
    fn kv_set(&self, namespace: &str, key: &str, value: &str) -> Result<(), String> {
        KnowledgeBase::kv_set(self, namespace, key, value)
    }

    fn kv_get(&self, namespace: &str, key: &str) -> Result<Option<String>, String> {
        KnowledgeBase::kv_get(self, namespace, key)
    }
}

/// 打通 core/nt_core_traits::MemoryProvider 死抽象 — KnowledgeBase 是记忆存储/检索的
/// 事实提供者。此前 trait 定义但从未实现，任何 `dyn MemoryProvider` 都无法接线。
impl crate::l0_substrate::nt_core_traits::MemoryProvider for KnowledgeBase {
    fn store(&mut self, key: &str, value: &str) -> Result<String, String> {
        let node_id = self.insert_or_get_node(
            key,
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Insight,
            Some(value),
            None,
            Some("memory_provider"),
        )?;
        Ok(node_id)
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<(String, String)>, String> {
        let results = self.search(query, limit)?;
        Ok(results
            .into_iter()
            .map(|r| {
                let content = r.node.summary.clone().unwrap_or_default();
                (r.node.title.clone(), content)
            })
            .collect())
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        let node = crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_store::find_node_by_title_and_type(
            &conn,
            key,
            &crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::NodeType::Insight,
            false,
        ).map_err(|e| format!("find: {}", e))?;
        drop(conn);
        if let Some(n) = node {
            let deleted = self
                .delete_node(&n.id)
                .map_err(|e| format!("delete: {}", e))?;
            if deleted {
                // Invalidate fused search cache so stale results don't resurface
                if let Ok(mut cache) = self.fused_cache.lock() {
                    cache.clear();
                }
            }
            Ok(())
        } else {
            Err(format!("node not found: {}", key))
        }
    }
}

// ── EvolutionPatternType helper ──

impl EvolutionPatternType {
    fn from_str(s: &str) -> Self {
        match s {
            "RecurringError" => EvolutionPatternType::RecurringError,
            "CommunicationOptimization" => EvolutionPatternType::CommunicationOptimization,
            "ProblemDecomposition" => EvolutionPatternType::ProblemDecomposition,
            "VerificationImprovement" => EvolutionPatternType::VerificationImprovement,
            "ToolUsagePattern" => EvolutionPatternType::ToolUsagePattern,
            "StrategyDiscovery" => EvolutionPatternType::StrategyDiscovery,
            "PrincipleUpdate" => EvolutionPatternType::PrincipleUpdate,
            _ => EvolutionPatternType::RecurringError,
        }
    }
}

/// KnowledgeSink trait 实现 — 打通 L2 感知层对 L1 知识层的写入接口
impl crate::l0_substrate::nt_core_traits::KnowledgeSink for KnowledgeBase {
    fn sink_node(
        &self,
        title: &str,
        node_type: neotrix_types::knowledge_access::NodeType,
        summary: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> Result<String, String> {
        self.insert_or_get_node(title, node_type, summary, url, domain)
    }

    fn sink_edge(
        &self,
        source_id: &str,
        target_id: &str,
        relation_type: neotrix_types::knowledge_access::RelationType,
        weight: f64,
        description: Option<&str>,
    ) -> Result<(), String> {
        self.upsert_edge(source_id, target_id, relation_type, weight, description)
    }

    fn sink_edge_with_metadata(
        &self,
        source_id: &str,
        target_id: &str,
        relation_type: neotrix_types::knowledge_access::RelationType,
        weight: f64,
        description: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), String> {
        self.upsert_edge_with_metadata(
            source_id,
            target_id,
            relation_type,
            weight,
            description,
            metadata,
        )
    }

    fn edge_exists(
        &self,
        source_id: &str,
        target_id: &str,
        relation_type: neotrix_types::knowledge_access::RelationType,
    ) -> Result<bool, String> {
        KnowledgeBase::edge_exists(self, source_id, target_id, relation_type)
    }

    fn update_node_metadata(
        &self,
        node_id: &str,
        metadata: &serde_json::Value,
    ) -> Result<(), String> {
        self.update_node_metadata(node_id, metadata)
    }
}

impl crate::l4_emotion::nt_feel_facade::AntiDistilStore
    for KnowledgeBase
{
    fn store_trace_data(&self, data: &serde_json::Value) -> Result<(), String> {
        KnowledgeBase::store_trace_data(self, data)
    }

    fn get_trace_data(&self, limit: usize) -> Result<Vec<serde_json::Value>, String> {
        KnowledgeBase::get_trace_data(self, limit)
    }
}

impl neotrix_types::knowledge_access::KnowledgeAccess for KnowledgeBase {
    fn get_node(
        &self,
        id: &str,
    ) -> Result<Option<neotrix_types::knowledge_access::KnowledgeNode>, String> {
        KnowledgeBase::get_node(self, id)
    }

    fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<neotrix_types::knowledge_access::KnowledgeNode>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_search::search_fts(&conn, query, limit)
            .map(|results| results.into_iter().map(|sr| sr.node).collect())
            .map_err(|e| format!("search: {}", e))
    }

    fn nodes_by_type(
        &self,
        node_type: neotrix_types::knowledge_access::NodeType,
        limit: usize,
    ) -> Result<Vec<neotrix_types::knowledge_access::KnowledgeNode>, String> {
        KnowledgeBase::search_by_type(self, &node_type, limit)
    }

    fn edges_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<neotrix_types::knowledge_access::KnowledgeEdge>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock: {}", e))?;
        nt_memory_store::get_edges_for_node(&conn, node_id)
            .map_err(|e| format!("edges_for_node: {}", e))
    }

    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, String> {
        let config = self
            .embedding_config
            .read()
            .map_err(|e| format!("embedding_config read: {}", e))?
            .clone();
        let config = config.ok_or_else(|| "No embedding configuration available".to_string())?;
        nt_memory_embed::embed_text(&config, text)
    }

    fn embedding_dim(&self) -> usize {
        self.embedding_config
            .read()
            .ok()
            .and_then(|c| c.as_ref().map(|c| c.dimension))
            .unwrap_or(384)
    }
}

#[cfg(test)]
mod tests;
// 2026-08-15 sweep absorption (P6/P15/P16/P17): 记忆层四能力注入
pub mod nt_memory_sweep_20260815;
pub use nt_memory_sweep_20260815::*;

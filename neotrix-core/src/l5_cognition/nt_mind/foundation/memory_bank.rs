use serde::{Deserialize, Serialize};
use crate::core::nt_core_self_test;

/// Memory tier — 4 levels from ephemeral to permanent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryTier {
    /// Working memory — immediate task context, high volatility
    Working,
    /// Episodic memory — session-scoped experiences
    Episodic,
    /// Procedural memory — reusable skill patterns (E8 sequences)
    Procedural,
    /// Semantic memory — consolidated facts, persisted long-term
    Semantic,
}

impl MemoryTier {
    pub fn ttl_secs(&self) -> u64 {
        match self {
            MemoryTier::Working => 300,
            MemoryTier::Episodic => 86400,
            MemoryTier::Procedural => 604800,
            MemoryTier::Semantic => 0,
        }
    }

    /// ai-memory 分层衰减半衰期 (吸收): 每层 salience 按自己的时间尺度指数衰减 —
    /// 工作记忆分钟级挥发, 情景记忆天级, 程序记忆周级, 语义记忆永久。对应
    /// ai-memory M8 的 salience = salience · exp(-λΔt), 各层 λ 不同。
    pub fn half_life_secs(&self) -> u64 {
        match self {
            MemoryTier::Working => 600,
            MemoryTier::Episodic => 86_400,
            MemoryTier::Procedural => 604_800,
            MemoryTier::Semantic => 0,
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            MemoryTier::Working => 0,
            MemoryTier::Episodic => 1,
            MemoryTier::Procedural => 2,
            MemoryTier::Semantic => 3,
        }
    }
}

/// Dual-track memory entry — OMP-compatible
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualTrackEntry {
    pub id: String,
    pub tier: MemoryTier,
    pub agent_id: String,
    pub session_id: String,

    /// Symbolic track — deterministic (AST hashes, signatures, imports)
    pub symbolic: _SymbolicTrack,

    /// Semantic track — probabilistic (embeddings, intent, constraints)
    pub semantic: _SemanticTrack,

    pub created_at: i64,
    pub accessed_at: i64,
    pub access_count: u64,
    pub staleness_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _SymbolicTrack {
    pub ast_hash: String,
    pub file_hash: Option<String>,
    pub signatures: Vec<String>,
    pub dependencies: Vec<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _SemanticTrack {
    pub intent: String,
    pub constraints: Vec<String>,
    pub preferences: Vec<String>,
    pub embedding: Option<Vec<f32>>,
    pub tags: Vec<String>,
}

/// Unified memory store trait
pub trait MemoryStore: Send + Sync {
    fn store(&mut self, entry: DualTrackEntry) -> Result<String, String>;
    fn recall(&self, agent_id: &str, query: &str, tier: Option<MemoryTier>, limit: usize) -> Result<Vec<DualTrackEntry>, String>;
    fn recall_by_session(&self, session_id: &str, query: &str, limit: usize) -> Result<Vec<DualTrackEntry>, String>;
    fn recall_similar(&self, agent_id: &str, embedding: &[f32], tier: Option<MemoryTier>, limit: usize) -> Result<Vec<(DualTrackEntry, f64)>, String>;
    fn forget(&mut self, id: &str) -> Result<(), String>;
    fn consolidate(&mut self, max_age_secs: u64) -> Result<usize, String>;
    fn reconcile(&mut self, entry: &mut DualTrackEntry) -> Result<(), String>;
}

/// OMP (Open Memory Protocol) v1.0 — Five-verb memory surface
/// https://github.com/SMJAI/open-memory-protocol
///
/// | Verb | NeoTrix Mapping | Description |
/// |------|----------------|-------------|
/// | recall | MemoryOrchestrator::recall | Query existing memories |
/// | remember | MemoryStore::store | Create new memory entry |
/// | link | MemoryOrchestrator::promote | Connect / tier-promote memories |
/// | observe | MemoryOrchestrator::drain_expired | Scan for stale / decayed entries |
/// | validate | _OMPReconciler::reconcile | Verify consistency across dual tracks |
pub struct _OMPReconciler;

impl _OMPReconciler {
    /// OMP `reconcile` — merge symbolic + semantic tracks into a unified entry.
    pub fn reconcile(symbolic: &_SymbolicTrack, semantic: &_SemanticTrack) -> DualTrackEntry {
        let ast_hash = if symbolic.ast_hash.is_empty() {
            md5_hash(&semantic.intent)
        } else {
            symbolic.ast_hash.clone()
        };
        DualTrackEntry {
            id: format!("mem_{}", uuid::Uuid::new_v4()),
            tier: MemoryTier::Episodic,
            agent_id: String::new(),
            session_id: String::new(),
            symbolic: symbolic.clone(),
            semantic: semantic.clone(),
            created_at: chrono::Utc::now().timestamp(),
            accessed_at: chrono::Utc::now().timestamp(),
            access_count: 1,
            staleness_hash: Some(ast_hash),
        }
    }

    /// OMP `observe` — check staleness by comparing current hash vs stored.
    pub fn is_stale(entry: &DualTrackEntry, current_hash: &str) -> bool {
        match &entry.staleness_hash {
            Some(h) => h != current_hash,
            None => false,
        }
    }

    /// OMP `validate` — verify dual-track consistency.
    /// Returns true if the symbolic hash matches the semantic intent hash.
    pub fn validate(entry: &DualTrackEntry) -> bool {
        let expected_hash = if entry.symbolic.ast_hash.is_empty() {
            md5_hash(&entry.semantic.intent)
        } else {
            entry.symbolic.ast_hash.clone()
        };
        match &entry.staleness_hash {
            Some(h) => h == &expected_hash,
            None => true,
        }
    }
}

/// Tiered memory orchestrator
#[derive(Default)]
pub struct MemoryOrchestrator {
    working: Vec<DualTrackEntry>,
    episodic: Vec<DualTrackEntry>,
    procedural: Vec<DualTrackEntry>,
    semantic: Vec<DualTrackEntry>,
}

impl MemoryOrchestrator {
    pub fn new() -> Self {
        Self {
            working: Vec::new(),
            episodic: Vec::new(),
            procedural: Vec::new(),
            semantic: Vec::new(),
        }
    }

    pub fn store(&mut self, entry: DualTrackEntry) -> Result<String, String> {
        let id = entry.id.clone();
        let tier = entry.tier;
        match tier {
            MemoryTier::Working => self.working.push(entry),
            MemoryTier::Episodic => self.episodic.push(entry),
            MemoryTier::Procedural => self.procedural.push(entry),
            MemoryTier::Semantic => self.semantic.push(entry),
        }
        Ok(id)
    }

    pub fn recall(&self, agent_id: &str, query: &str, tier: Option<MemoryTier>, limit: usize) -> Vec<DualTrackEntry> {
        let q = query.to_lowercase();
        let mut results: Vec<DualTrackEntry> = Vec::new();

        let pools: &[&[DualTrackEntry]] = match tier {
            Some(MemoryTier::Working) => &[&self.working],
            Some(MemoryTier::Episodic) => &[&self.episodic],
            Some(MemoryTier::Procedural) => &[&self.procedural],
            Some(MemoryTier::Semantic) => &[&self.semantic],
            None => &[&self.working, &self.episodic, &self.procedural, &self.semantic],
        };

        for pool in pools {
            for entry in *pool {
                if entry.agent_id != agent_id { continue; }
                if entry.semantic.intent.to_lowercase().contains(&q)
                    || entry.symbolic.signatures.iter().any(|s| s.to_lowercase().contains(&q))
                    || entry.semantic.tags.iter().any(|t| t.to_lowercase().contains(&q))
                {
                    results.push(entry.clone());
                    if results.len() >= limit { return results; }
                }
            }
        }
        results
    }

    pub fn size(&self) -> usize {
        self.working.len() + self.episodic.len() + self.procedural.len() + self.semantic.len()
    }

    pub fn tier_size(&self, tier: MemoryTier) -> usize {
        match tier {
            MemoryTier::Working => self.working.len(),
            MemoryTier::Episodic => self.episodic.len(),
            MemoryTier::Procedural => self.procedural.len(),
            MemoryTier::Semantic => self.semantic.len(),
        }
    }

    /// Persist a dual-track entry to AgentSessionManager via KnowledgeBase.
    /// Returns the KB entry ID on success.
    pub fn persist_entry(kb: &crate::neotrix::nt_memory_kb::KnowledgeBase, entry: &DualTrackEntry) -> Result<String, String> {
        let content = serde_json::to_string(entry).map_err(|e| format!("serialize: {}", e))?;
        let tier_str = match entry.tier {
            MemoryTier::Working => "working",
            MemoryTier::Episodic => "episodic",
            MemoryTier::Procedural => "procedural",
            MemoryTier::Semantic => "semantic",
        };
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("dual_track_id".into(), entry.id.clone());
        metadata.insert("ast_hash".into(), entry.symbolic.ast_hash.clone());
        metadata.insert("signatures".into(), entry.symbolic.signatures.join(","));
        metadata.insert("intent".into(), entry.semantic.intent.clone());
        metadata.insert("tags".into(), entry.semantic.tags.join(","));
        kb.agent_memory_store(&entry.agent_id, &entry.session_id, &content, tier_str, metadata, entry.semantic.embedding.as_deref())
    }

    /// ai-memory M8 分层 salience 衰减 — 半衰期模型而非硬 TTL 截断。
    /// 每层按 half_life_secs 指数衰减 salience, 低于 `floor` 的条目视为遗忘并移除。
    /// 返回被遗忘的条目 (供上层决定是落盘还是丢弃)。
    pub fn decay_salience(&mut self, floor: f64, now: i64) -> Vec<DualTrackEntry> {
        let mut forgotten = Vec::new();
        for tier in [MemoryTier::Working, MemoryTier::Episodic, MemoryTier::Procedural] {
            let hl = tier.half_life_secs() as f64;
            let pool = match tier {
                MemoryTier::Working => &mut self.working,
                MemoryTier::Episodic => &mut self.episodic,
                MemoryTier::Procedural => &mut self.procedural,
                MemoryTier::Semantic => continue,
            };
            let mut kept = Vec::new();
            for e in pool.drain(..) {
                let age = (now - e.accessed_at.max(e.created_at)).max(0) as f64;
                let decay = (-age * std::f64::consts::LN_2 / hl).exp();
                let salience = e.access_count as f64 * decay;
                if salience < floor {
                    forgotten.push(e);
                } else {
                    kept.push(e);
                }
            }
            *pool = kept;
        }
        forgotten
    }

    /// Drain expired entries from this tier. Removes entries older than `max_age_secs`.
    pub fn drain_expired(&mut self, tier: MemoryTier, max_age_secs: u64) -> Vec<DualTrackEntry> {
        let now = chrono::Utc::now().timestamp();
        let pool = match tier {
            MemoryTier::Working => &mut self.working,
            MemoryTier::Episodic => &mut self.episodic,
            MemoryTier::Procedural => &mut self.procedural,
            MemoryTier::Semantic => return Vec::new(),
        };
        let cutoff = now - max_age_secs as i64;
        let mut kept = Vec::new();
        let mut expired = Vec::new();
        for e in pool.drain(..) {
            if e.created_at < cutoff && tier != MemoryTier::Semantic {
                expired.push(e);
            } else {
                kept.push(e);
            }
        }
        *pool = kept;
        expired
    }

    /// Promote entries up the tier ladder: Working → Episodic → Procedural → Semantic.
    /// Each promoted entry's `consolidate()` callback decides whether to promote.
    pub fn promote<F>(&mut self, tier: MemoryTier, mut predicate: F) -> Vec<DualTrackEntry>
    where
        F: FnMut(&DualTrackEntry) -> bool,
    {
        let (src, dst) = match tier {
            MemoryTier::Working => (&mut self.working, MemoryTier::Episodic),
            MemoryTier::Episodic => (&mut self.episodic, MemoryTier::Procedural),
            MemoryTier::Procedural => (&mut self.procedural, MemoryTier::Semantic),
            MemoryTier::Semantic => return Vec::new(),
        };
        let mut promoted = Vec::new();
        let mut kept = Vec::new();
        for mut e in src.drain(..) {
            if predicate(&e) {
                e.tier = dst;
                e.accessed_at = chrono::Utc::now().timestamp();
                promoted.push(e);
            } else {
                kept.push(e);
            }
        }
        *src = kept;
        promoted
    }
}

fn md5_hash(s: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tier_ordering() {
        assert!(MemoryTier::Semantic.priority() > MemoryTier::Working.priority());
        assert!(MemoryTier::Procedural.priority() > MemoryTier::Episodic.priority());
    }

    #[test]
    fn test_memory_tier_ttl() {
        assert_eq!(MemoryTier::Working.ttl_secs(), 300);
        assert_eq!(MemoryTier::Semantic.ttl_secs(), 0);
    }

    #[test]
    fn test_memory_tier_half_life_ordering() {
        assert!(MemoryTier::Working.half_life_secs() < MemoryTier::Episodic.half_life_secs());
        assert!(MemoryTier::Episodic.half_life_secs() < MemoryTier::Procedural.half_life_secs());
        assert_eq!(MemoryTier::Semantic.half_life_secs(), 0);
    }

    #[test]
    fn test_decay_salience_forgets_old_single_access() {
        let mut orch = MemoryOrchestrator::new();
        orch.store(make_entry("old", MemoryTier::Working, "a", "stale")).unwrap();
        // accessed_at=0, now 远超 Working 半衰期 (600s) → 单次访问 salience 衰减到 floor 下
        let forgotten = orch.decay_salience(0.5, 3_600);
        assert_eq!(forgotten.len(), 1);
        assert_eq!(forgotten[0].id, "old");
        assert_eq!(orch.tier_size(MemoryTier::Working), 0);
    }

    #[test]
    fn test_decay_salience_keeps_frequent_access() {
        let mut orch = MemoryOrchestrator::new();
        orch.store(make_entry("hot", MemoryTier::Working, "a", "frequent")).unwrap();
        if let Some(e) = orch.working.iter_mut().find(|e| e.id == "hot") {
            e.access_count = 10; // 高频访问 → 高 base salience
            e.accessed_at = 3_600; // 最近访问 → age≈0, 无衰减
        }
        let forgotten = orch.decay_salience(0.5, 3_600);
        assert!(forgotten.is_empty(), "高频访问不应被遗忘");
        assert_eq!(orch.tier_size(MemoryTier::Working), 1);
    }

    #[test]
    fn test_decay_salience_skips_semantic() {
        let mut orch = MemoryOrchestrator::new();
        orch.store(make_entry("fact", MemoryTier::Semantic, "a", "permanent")).unwrap();
        let forgotten = orch.decay_salience(0.5, 1_000_000);
        assert!(forgotten.is_empty());
        assert_eq!(orch.tier_size(MemoryTier::Semantic), 1);
    }

    #[test]
    fn test_omp_reconciler_merges_tracks() {
        let symbolic = _SymbolicTrack {
            ast_hash: "abc123".into(),
            file_hash: None,
            signatures: vec!["fn foo()".into()],
            dependencies: vec!["bar".into()],
            imports: vec!["std::collections".into()],
            exports: vec![],
        };
        let semantic = _SemanticTrack {
            intent: "parse user input".into(),
            constraints: vec!["utf-8".into()],
            preferences: vec!["async".into()],
            embedding: None,
            tags: vec!["parsing".into(), "input".into()],
        };
        let entry = _OMPReconciler::reconcile(&symbolic, &semantic);
        assert!(entry.id.starts_with("mem_"));
        assert_eq!(entry.symbolic.ast_hash, "abc123");
        assert_eq!(entry.semantic.intent, "parse user input");
        assert_eq!(entry.tier, MemoryTier::Episodic);
    }

    #[test]
    fn test_omp_staleness_detection() {
        let symbolic = _SymbolicTrack {
            ast_hash: "abc".into(),
            file_hash: None,
            signatures: vec![],
            dependencies: vec![],
            imports: vec![],
            exports: vec![],
        };
        let semantic = _SemanticTrack {
            intent: String::new(),
            constraints: vec![],
            preferences: vec![],
            embedding: None,
            tags: vec![],
        };
        let entry = _OMPReconciler::reconcile(&symbolic, &semantic);
        assert!(_OMPReconciler::is_stale(&entry, "def"));
        assert!(!_OMPReconciler::is_stale(&entry, "abc"));
    }

    #[test]
    fn test_orchestrator_tiered_storage() {
        let mut orch = MemoryOrchestrator::new();
        for (i, tier) in [MemoryTier::Working, MemoryTier::Episodic, MemoryTier::Procedural, MemoryTier::Semantic].iter().enumerate() {
            let entry = DualTrackEntry {
                id: format!("entry_{}", i),
                tier: *tier,
                agent_id: "agent_1".into(),
                session_id: "session_1".into(),
                symbolic: _SymbolicTrack {
                    ast_hash: format!("hash_{}", i),
                    file_hash: None,
                    signatures: vec![],
                    dependencies: vec![],
                    imports: vec![],
                    exports: vec![],
                },
                semantic: _SemanticTrack {
                    intent: format!("task_{}", i),
                    constraints: vec![],
                    preferences: vec![],
                    embedding: None,
                    tags: vec![],
                },
                created_at: 0,
                accessed_at: 0,
                access_count: 1,
                staleness_hash: None,
            };
            orch.store(entry).unwrap();
        }
        assert_eq!(orch.size(), 4);
        assert_eq!(orch.tier_size(MemoryTier::Working), 1);
        assert_eq!(orch.tier_size(MemoryTier::Semantic), 1);
    }

    #[test]
    fn test_omp_validate_consistent() {
        let symbolic = _SymbolicTrack {
            ast_hash: "abc".into(), file_hash: None,
            signatures: vec![], dependencies: vec![], imports: vec![], exports: vec![],
        };
        let semantic = _SemanticTrack {
            intent: "test".into(), constraints: vec![], preferences: vec![],
            embedding: None, tags: vec![],
        };
        let entry = _OMPReconciler::reconcile(&symbolic, &semantic);
        assert!(_OMPReconciler::validate(&entry));
    }

    #[test]
    fn test_omp_validate_inconsistent() {
        let symbolic = _SymbolicTrack {
            ast_hash: "abc".into(), file_hash: None,
            signatures: vec![], dependencies: vec![], imports: vec![], exports: vec![],
        };
        let semantic = _SemanticTrack {
            intent: "test".into(), constraints: vec![], preferences: vec![],
            embedding: None, tags: vec![],
        };
        let entry = _OMPReconciler::reconcile(&symbolic, &semantic);
        // Manually corrupt staleness_hash to simulate inconsistency
        let mut corrupt = entry.clone();
        corrupt.staleness_hash = Some("xyz".into());
        assert!(!_OMPReconciler::validate(&corrupt));
    }

    fn make_entry(id: &str, tier: MemoryTier, agent: &str, intent: &str) -> DualTrackEntry {
        DualTrackEntry {
            id: id.into(),
            tier,
            agent_id: agent.into(),
            session_id: "s1".into(),
            symbolic: _SymbolicTrack {
                ast_hash: format!("h_{}", id),
                file_hash: None,
                signatures: vec![],
                dependencies: vec![],
                imports: vec![],
                exports: vec![],
            },
            semantic: _SemanticTrack {
                intent: intent.into(),
                constraints: vec![],
                preferences: vec![],
                embedding: None,
                tags: vec![],
            },
            created_at: 0,
            accessed_at: 0,
            access_count: 1,
            staleness_hash: None,
        }
    }

    #[test]
    fn test_drain_expired_removes_old_entries() {
        let mut orch = MemoryOrchestrator::new();
        orch.store(make_entry("e1", MemoryTier::Working, "a", "task_old")).unwrap();
        orch.store(make_entry("e2", MemoryTier::Working, "a", "task_fresh")).unwrap();
        // Set second entry to recent time
        if let Some(e) = orch.working.iter_mut().find(|e| e.id == "e2") {
            e.created_at = chrono::Utc::now().timestamp();
        }
        let expired = orch.drain_expired(MemoryTier::Working, 100);
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].id, "e1");
        assert_eq!(orch.tier_size(MemoryTier::Working), 1);
    }

    #[test]
    fn test_promote_moves_entries_up_ladder() {
        let mut orch = MemoryOrchestrator::new();
        orch.store(make_entry("w1", MemoryTier::Working, "a", "task")).unwrap();
        orch.store(make_entry("w2", MemoryTier::Working, "a", "task")).unwrap();
        let promoted = orch.promote(MemoryTier::Working, |e| e.id == "w2");
        assert_eq!(promoted.len(), 1);
        assert_eq!(promoted[0].tier, MemoryTier::Episodic);
        assert_eq!(orch.tier_size(MemoryTier::Working), 1);
    }

    #[test]
    fn test_drain_semantic_never_expires() {
        let mut orch = MemoryOrchestrator::new();
        orch.store(make_entry("s1", MemoryTier::Semantic, "a", "fact")).unwrap();
        let expired = orch.drain_expired(MemoryTier::Semantic, 0);
        assert!(expired.is_empty());
        assert_eq!(orch.tier_size(MemoryTier::Semantic), 1);
    }

    #[test]
    fn test_orchestrator_recall_by_agent() {
        let mut orch = MemoryOrchestrator::new();
        for i in 0..3 {
            let entry = DualTrackEntry {
                id: format!("e{}", i),
                tier: MemoryTier::Working,
                agent_id: "agent_x".into(),
                session_id: "s1".into(),
                symbolic: _SymbolicTrack {
                    ast_hash: format!("h{}", i),
                    file_hash: None,
                    signatures: vec!["fn handle_".to_string()],
                    dependencies: vec![],
                    imports: vec![],
                    exports: vec![],
                },
                semantic: _SemanticTrack {
                    intent: format!("handle request {}", i),
                    constraints: vec![],
                    preferences: vec![],
                    embedding: None,
                    tags: vec!["handler".into()],
                },
                created_at: 0,
                accessed_at: 0,
                access_count: 1,
                staleness_hash: None,
            };
            orch.store(entry).unwrap();
        }
        let results = orch.recall("agent_x", "handle", None, 10);
        assert_eq!(results.len(), 3);
        let empty = orch.recall("agent_y", "handle", None, 10);
        assert!(empty.is_empty());
    }
}

// ============================================================================
// Memory Admission Control — A-MAC inspired gate
// ============================================================================

/// Admission score for a memory candidate before writing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _AdmissionScore {
    pub utility: f64,
    pub confidence: f64,
    pub novelty: f64,
    pub recency: f64,
    pub total: f64,
    pub admitted: bool,
}

/// Adaptive Memory Admission Control gate (inspired by A-MAC, Zhang et al. 2025).
///
/// Scores each memory candidate on 5 dimensions before accepting it into the store:
///   1. **Utility** — LLM-call estimated value
///   2. **Confidence** — ROUGE-L grounding score
///   3. **Novelty** — 1 − max cosine similarity to existing memories
///   4. **Recency** — temporal freshness of the observation
///   5. **Type Prior** — prior probability of this memory type being useful
///
/// Memories below the admission threshold are rejected to prevent memory pollution.
pub struct MemoryAdmissionGate {
    threshold: f64,
    max_store_size: usize,
}

impl MemoryAdmissionGate {
    pub fn new(threshold: f64, max_store_size: usize) -> Self {
        Self { threshold, max_store_size }
    }

    /// Evaluate a memory candidate. Returns true if the memory should be admitted.
    pub fn evaluate(&self, utility: f64, confidence: f64, novelty: f64, recency: f64, type_prior: f64) -> _AdmissionScore {
        let total = (utility + confidence + novelty + recency + type_prior) / 5.0;
        let admitted = total >= self.threshold && self.current_size() < self.max_store_size;
        _AdmissionScore {
            utility, confidence, novelty, recency, total, admitted,
        }
    }

    /// Check if a memory should be admitted (convenience method).
    pub fn admit(&self, utility: f64, confidence: f64, novelty: f64, recency: f64, type_prior: f64) -> bool {
        self.evaluate(utility, confidence, novelty, recency, type_prior).admitted
    }

    fn current_size(&self) -> usize {
        0
    }
}

// SelfTest trait implementation for NeoTrix SelfTest registry
impl nt_core_self_test::SelfTest for MemoryAdmissionGate {
    fn name(&self) -> &str {
        "memory_admission_gate"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        // Verify gate can be created and evaluate function works
        let score = self.evaluate(0.8, 0.7, 0.6, 0.9, 0.5);
        if score.admitted {
            Ok(())
        } else {
            Err(vec!["test failed: expected admission".to_string()])
        }
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;

    #[test]
    fn test_admission_gate_admits_high_quality() {
        let gate = MemoryAdmissionGate::new(0.5, 100);
        assert!(gate.admit(0.8, 0.7, 0.6, 0.9, 0.5));
    }

    #[test]
    fn test_admission_gate_rejects_low_quality() {
        let gate = MemoryAdmissionGate::new(0.5, 100);
        assert!(!gate.admit(0.1, 0.1, 0.1, 0.1, 0.1));
    }

    #[test]
    fn test_admission_score() {
        let gate = MemoryAdmissionGate::new(0.5, 100);
        let score = gate.evaluate(0.8, 0.7, 0.6, 0.9, 0.5);
        assert!(score.admitted);
        assert!((score.total - 0.7).abs() < 0.01);
    }
}

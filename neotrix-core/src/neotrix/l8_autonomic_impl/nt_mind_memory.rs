use serde::{Deserialize, Serialize};

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
    pub symbolic: SymbolicTrack,

    /// Semantic track — probabilistic (embeddings, intent, constraints)
    pub semantic: SemanticTrack,

    pub created_at: i64,
    pub accessed_at: i64,
    pub access_count: u64,
    pub staleness_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicTrack {
    pub ast_hash: String,
    pub file_hash: Option<String>,
    pub signatures: Vec<String>,
    pub dependencies: Vec<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTrack {
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
/// | validate | OMPReconciler::reconcile | Verify consistency across dual tracks |
pub struct OMPReconciler;

impl OMPReconciler {
    /// OMP `reconcile` — merge symbolic + semantic tracks into a unified entry.
    pub fn reconcile(symbolic: &SymbolicTrack, semantic: &SemanticTrack) -> DualTrackEntry {
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
    fn test_omp_reconciler_merges_tracks() {
        let symbolic = SymbolicTrack {
            ast_hash: "abc123".into(),
            file_hash: None,
            signatures: vec!["fn foo()".into()],
            dependencies: vec!["bar".into()],
            imports: vec!["std::collections".into()],
            exports: vec![],
        };
        let semantic = SemanticTrack {
            intent: "parse user input".into(),
            constraints: vec!["utf-8".into()],
            preferences: vec!["async".into()],
            embedding: None,
            tags: vec!["parsing".into(), "input".into()],
        };
        let entry = OMPReconciler::reconcile(&symbolic, &semantic);
        assert!(entry.id.starts_with("mem_"));
        assert_eq!(entry.symbolic.ast_hash, "abc123");
        assert_eq!(entry.semantic.intent, "parse user input");
        assert_eq!(entry.tier, MemoryTier::Episodic);
    }

    #[test]
    fn test_omp_staleness_detection() {
        let symbolic = SymbolicTrack {
            ast_hash: "abc".into(),
            file_hash: None,
            signatures: vec![],
            dependencies: vec![],
            imports: vec![],
            exports: vec![],
        };
        let semantic = SemanticTrack {
            intent: String::new(),
            constraints: vec![],
            preferences: vec![],
            embedding: None,
            tags: vec![],
        };
        let entry = OMPReconciler::reconcile(&symbolic, &semantic);
        assert!(OMPReconciler::is_stale(&entry, "def"));
        assert!(!OMPReconciler::is_stale(&entry, "abc"));
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
                symbolic: SymbolicTrack {
                    ast_hash: format!("hash_{}", i),
                    file_hash: None,
                    signatures: vec![],
                    dependencies: vec![],
                    imports: vec![],
                    exports: vec![],
                },
                semantic: SemanticTrack {
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
        let symbolic = SymbolicTrack {
            ast_hash: "abc".into(), file_hash: None,
            signatures: vec![], dependencies: vec![], imports: vec![], exports: vec![],
        };
        let semantic = SemanticTrack {
            intent: "test".into(), constraints: vec![], preferences: vec![],
            embedding: None, tags: vec![],
        };
        let entry = OMPReconciler::reconcile(&symbolic, &semantic);
        assert!(OMPReconciler::validate(&entry));
    }

    #[test]
    fn test_omp_validate_inconsistent() {
        let symbolic = SymbolicTrack {
            ast_hash: "abc".into(), file_hash: None,
            signatures: vec![], dependencies: vec![], imports: vec![], exports: vec![],
        };
        let semantic = SemanticTrack {
            intent: "test".into(), constraints: vec![], preferences: vec![],
            embedding: None, tags: vec![],
        };
        let entry = OMPReconciler::reconcile(&symbolic, &semantic);
        // Manually corrupt staleness_hash to simulate inconsistency
        let mut corrupt = entry.clone();
        corrupt.staleness_hash = Some("xyz".into());
        assert!(!OMPReconciler::validate(&corrupt));
    }

    fn make_entry(id: &str, tier: MemoryTier, agent: &str, intent: &str) -> DualTrackEntry {
        DualTrackEntry {
            id: id.into(),
            tier,
            agent_id: agent.into(),
            session_id: "s1".into(),
            symbolic: SymbolicTrack {
                ast_hash: format!("h_{}", id),
                file_hash: None,
                signatures: vec![],
                dependencies: vec![],
                imports: vec![],
                exports: vec![],
            },
            semantic: SemanticTrack {
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
                symbolic: SymbolicTrack {
                    ast_hash: format!("h{}", i),
                    file_hash: None,
                    signatures: vec!["fn handle_".to_string()],
                    dependencies: vec![],
                    imports: vec![],
                    exports: vec![],
                },
                semantic: SemanticTrack {
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

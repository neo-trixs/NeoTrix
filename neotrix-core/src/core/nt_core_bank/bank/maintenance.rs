use std::sync::atomic::Ordering;

use chrono::Utc;

use crate::core::knowledge::TaskType;
use crate::core::nt_core_bank::{MemoryIterationResult, MemoryTier, ReasoningBank, ReasoningMemory};

impl ReasoningBank {
    pub fn iterate_memories(&mut self, similarity_threshold: f64, min_reward: f64) -> MemoryIterationResult {
        let before = self.stats();
        let merged = self.consolidate_similar(similarity_threshold);
        let pruned = self.prune_low_value(min_reward);
        let replayed = self.replay_high_value();
        let promoted = self.promote_tiers();
        let expired = self.evict_expired();
        self.bm25_dirty.store(true, Ordering::SeqCst);
        MemoryIterationResult {
            before,
            after: self.stats(),
            merged_count: merged,
            pruned_count: pruned,
            replayed_count: replayed,
            promoted_count: promoted,
            expired_count: expired,
        }
    }

    pub(crate) fn promote_tiers(&mut self) -> usize {
        let now = Utc::now().timestamp();
        let mut promoted = 0;
        for m in &mut self.memories {
            let age_hours = (now - m.timestamp) as f64 / 3600.0;
            let should_promote = match m.tier {
                MemoryTier::Working => age_hours > 1.0 || m.lifecycle.access_count >= 3,
                MemoryTier::Episodic => age_hours > 24.0 || m.lifecycle.access_count >= 10,
                MemoryTier::Semantic => age_hours > 168.0 || m.lifecycle.access_count >= 30,
                MemoryTier::Procedural => false,
            };
            if should_promote {
                if let Some(new_tier) = m.tier.promote() {
                    m.tier = new_tier;
                    promoted += 1;
                }
            }
        }
        promoted
    }

    pub fn prune_expired(&mut self, now: i64) -> usize {
        let before = self.memories.len();
        self.memories.retain(|m| {
            if let Some(ttl) = m.lifecycle.ttl_seconds { now - m.lifecycle.created_at <= ttl } else { true }
        });
        before - self.memories.len()
    }

    fn evict_expired(&mut self) -> usize {
        let expired = self.prune_expired(Utc::now().timestamp());
        let before2 = self.memories.len();
        let now = Utc::now().timestamp();
        let one_week: i64 = 604800;
        self.memories.retain(|m| {
            let age = now - m.timestamp;
            if age > one_week && m.lifecycle.importance < 0.3 && m.lifecycle.access_count < 2 && m.reward < 0.4 { return false; }
            if age > one_week * 4 && m.lifecycle.importance < 0.5 && m.reward < 0.3 { return false; }
            true
        });
        expired + (before2 - self.memories.len())
    }

    pub fn consolidate_similar(&mut self, threshold: f64) -> usize {
        let mut merged = 0;
        let mut i = 0;
        while i < self.memories.len() {
            let mut j = i + 1;
            while j < self.memories.len() {
                let same_type = self.memories[i].task_type == self.memories[j].task_type;
                let reward_sim = 1.0 - (self.memories[i].reward - self.memories[j].reward).abs();
                if same_type && reward_sim > threshold {
                    let reward = (self.memories[i].reward + self.memories[j].reward) / 2.0;
                    let desc = format!("{}; {}", self.memories[i].task_description, self.memories[j].task_description);
                    let mut edits = self.memories[i].micro_edits.clone();
                    edits.extend(self.memories[j].micro_edits.clone());
                    let mut memory = ReasoningMemory::new(&desc, self.memories[i].task_type, &edits, reward);
                    if let Some(ref emb) = self.memories[i].embedding { memory.embedding = Some(emb.clone()); }
                    self.memories[i] = memory;
                    self.memories.remove(j);
                    merged += 1;
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
        merged
    }

    pub fn prune_low_value(&mut self, min_reward: f64) -> usize {
        let before = self.memories.len();
        self.memories.retain(|m| m.reward >= min_reward);
        before - self.memories.len()
    }

    pub fn replay_high_value(&mut self) -> usize {
        let threshold = 0.8;
        let mut replayed = 0;
        let high_value: Vec<ReasoningMemory> = self.memories.iter()
            .filter(|m| m.reward > threshold).cloned().collect();
        for mut mem in high_value {
            mem.timestamp = Utc::now().timestamp();
            if self.memories.len() < self.max_memories {
                self.memories.push_back(mem);
                replayed += 1;
            }
        }
        replayed
    }

    pub fn initialize_with_design_knowledge(&mut self) {
        if !self.memories.is_empty() { return; }
        let design_knowledge = vec![
            ("OpenCodeX 18-parameter layout system: messageSeparation, messagePaddingTop/Bottom, containerPaddingTop/Bottom, containerGap, toolMarginTop, agentInfoMarginTop, containerPaddingLeft/Right, messagePaddingLeft, textIndent, toolIndent, showHeader, showFooter, forceSidebarHidden, showInputAgentInfo, showInputBorder, inputAgentInfoPaddingTop, inputBoxPaddingTop/Bottom", TaskType::UIDesign, 0.95),
            ("macOS design: translucent background (rgba), backdrop-filter: blur(20px), border-radius: 10px, subtle borders (rgba(0,0,0,0.08)), smooth transitions (0.2s cubic-bezier), font: Inter/system-ui", TaskType::UIDesign, 0.92),
            ("Three-column layout: left session list (240px), center chat (flex), right file manager (280px). Gap: 8px, padding: 8px. Each panel: rounded corners, subtle shadow, backdrop blur", TaskType::UIDesign, 0.90),
            ("OpenCodeX design philosophy: JSON-configurable layout parameters, 18 spatial parameters (spacing, visibility, behavior). Users can switch between default/dense layouts via /layout command. Custom layouts in ~/.config/opencode/layout/", TaskType::UIDesign, 0.88),
            ("Session management: unique IDs, timestamp tracking, active state, localStorage persistence, create/switch/delete operations. Display: session name, relative time (2 min ago), active highlight", TaskType::UIDesign, 0.85),
            ("File tree interaction: folder toggle, nested children (16px indent), click to expand/collapse, active state highlighting, file icons, hover effects", TaskType::UIDesign, 0.83),
            ("Chat UI: user/assistant/system message types, fade-in animation, thinking state (pulse/spin animation), status dot indicator, message counting, SEAL badge display", TaskType::UIDesign, 0.87),
        ];
        for (desc, task_type, reward) in design_knowledge {
            self.store(ReasoningMemory::new(desc, task_type, &[], reward));
        }
    }

    pub fn initialize_with_coding_knowledge(&mut self) {
        let code_knowledge = vec![
            ("Rust error handling best practices: use Result<T, E> instead of unwrap(), prefer ? operator for propagation, use thiserror for library errors, use anyhow for application errors", TaskType::CodeReview, 0.92),
            ("Memory safety in Rust: ownership rules, borrowing, lifetime annotations, Rc<RefCell<T>> for shared mutability, Arc<Mutex<T>> for thread-safe shared state", TaskType::CodeReview, 0.90),
            ("Concurrent programming patterns: use tokio for async runtime, async/await for I/O bound tasks, tokio::spawn for CPU-bound tasks, channels for message passing, Mutex/RwLock for shared state", TaskType::CodeGeneration, 0.88),
            ("Security audit checklist: check for command injection, path traversal, SQL injection, XSS, hardcoded secrets, unsafe deserialization, DoS vectors. Use cargo audit regularly", TaskType::Security, 0.95),
            ("API design principles: consistent naming, RESTful URLs, proper HTTP methods, status codes, error body format, pagination, rate limiting headers, API versioning", TaskType::CodeGeneration, 0.87),
            ("Testing strategy: unit tests for pure functions, integration tests for API, property-based testing for edge cases, snapshot testing for UI/output, benchmark tests for performance, doc tests for examples", TaskType::CodeReview, 0.85),
            ("Performance optimization: profile before optimizing, focus on O(n) improvements, use BTreeMap/HashMap wisely, avoid unnecessary allocations, use iterators instead of loops when clearer, batch DB queries", TaskType::CodeGeneration, 0.84),
            ("Database optimization: index columns used in WHERE/JOIN/ORDER BY, use EXPLAIN ANALYZE to check query plans, avoid N+1 queries, connection pooling, migration versioning, transactions for atomic operations, prepared statements for repeated queries", TaskType::CodeGeneration, 0.86),
            ("React component patterns: composition over inheritance, custom hooks for shared logic, useMemo/useCallback for performance, context + useReducer for complex state, error boundaries for crash recovery, lazy loading + code splitting for large apps, TypeScript for type safety", TaskType::UIDesign, 0.89),
            ("CI/CD pipeline: lint, type check, unit tests, integration tests, build, artifact publishing, semantic versioning, changelog generation", TaskType::Planning, 0.82),
        ];
        for (desc, task_type, reward) in code_knowledge {
            self.store(ReasoningMemory::new(desc, task_type, &[], reward));
        }
    }

    pub fn initialize_with_everos_knowledge(&mut self) {
        let everos_knowledge = vec![
//            ("EverOS hypergraph memory architecture: three-layer hierarchy — topic layer (broad themes), event layer (specific episodes), fact layer (atomic details). Hyperedges capture high-order associations between concepts. Retrieval follows coarse-to-fine: topic → event → fact. Each hyperedge connects multiple nodes forming higher-order relationships beyond pairwise associations.", TaskType::Research, 0.93),
            ("Biological memory imprinting pattern: memories are consolidated based on surprise/salience, not recency. Important memories have higher consolidation priority. The system maintains a working-to-long-term memory consolidation pipeline with importance threshold gating. This prevents catastrophic forgetting by prioritizing structurally significant experiences.", TaskType::Learning, 0.91),
            ("Self-evolution evaluation via EvoAgentBench methodology: longitudinal growth curves (measure capability trajectory over successive iterations), transfer efficiency (quantify how well knowledge acquired for one task type transfers to new domains), error avoidance (track reduction of repeated mistakes via contrastive reflection), and skill-hit quality (precision of capability vector targeting measured by cosine similarity to ideal profile).", TaskType::Reflection, 0.88),
            ("Memory extraction from unstructured conversation: detect entity relationships, temporal context, user preferences, task outcomes, and action items. Store as typed memory records with embedding for semantic retrieval. Uses hybrid search (BM25 + vector embedding) with RRF fusion for cross-session recall. Importance-weighted ranking ensures salient memories surface first.", TaskType::CodeAnalysis, 0.85),
            ("Multi-session context continuity via persistent memory bank: each session appends new memories; old memories decay via time-weighted importance but remain retrievable. Cross-session recall uses RRF fusion of BM25 + vector embedding search. Memory lifecycle management includes TTL-based expiration, importance-based consolidation, and periodic pruning of low-value traces.", TaskType::Planning, 0.87),
        ];
        for (desc, task_type, reward) in everos_knowledge {
            self.store(ReasoningMemory::new(desc, task_type, &[], reward));
        }
    }
}

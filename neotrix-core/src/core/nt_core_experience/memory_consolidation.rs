//! # 4-Tier Memory Consolidation Pipeline
//!
//! Inspired by AgentMemory's Working→Episodic→Semantic→Procedural model,
//! CoALA cognitive architecture (Princeton), and Active Dreaming Memory (ADM).
//!
//! Working Memory: raw observations, bounded capacity, LRU eviction, SHA-256 dedup.
//! Episodic Memory: compressed session summaries, TTL-based decay → semantic extraction.
//! Semantic Memory: extracted facts, confidence scoring via Ebbinghaus decay.
//! Procedural Memory: repeated successful patterns → skill extraction.
//!
//! Ebbinghaus forgetting curve: R = e^(-t/S), applied per cycle via decay_all().
//! Every 3rd tick runs full pipeline consolidation.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

// ── Tier enum ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MemoryTier {
    Working,
    Episodic,
    Semantic,
    Procedural,
}

// ── Working Memory ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemory {
    pub items: VecDeque<WorkingItem>,
    pub capacity: usize,
    pub dedup_window_secs: u64,
    pub dedup_hashes: VecDeque<([u8; 32], u64)>,
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self {
            items: VecDeque::new(),
            capacity: 50,
            dedup_window_secs: 300,
            dedup_hashes: VecDeque::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingItem {
    pub id: u64,
    pub content: String,
    pub source: String,
    pub timestamp: u64,
    pub content_hash: [u8; 32],
    pub importance: f64,
}

// ── Episodic Memory ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub episodes: Vec<EpisodeRecord>,
    pub max_episodes: usize,
    pub decay_days: u64,
}

impl Default for EpisodicMemory {
    fn default() -> Self {
        Self {
            episodes: Vec::new(),
            max_episodes: 1000,
            decay_days: 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeRecord {
    pub id: u64,
    pub summary: String,
    pub key_events: Vec<String>,
    pub start_time: u64,
    pub end_time: u64,
    pub valence: f64,
    pub importance: f64,
    pub access_count: u64,
    pub consolidated: bool,
}

// ── Semantic Memory ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    pub facts: Vec<SemanticFact>,
    pub max_facts: usize,
}

impl Default for SemanticMemory {
    fn default() -> Self {
        Self {
            facts: Vec::new(),
            max_facts: 5000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFact {
    pub id: u64,
    pub content: String,
    pub confidence: f64,
    pub source_episode_ids: Vec<u64>,
    pub access_count: u64,
    pub created_at: u64,
    pub last_verified: u64,
}

// ── Procedural Memory ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralMemory {
    pub procedures: Vec<ProcedureRecord>,
    pub max_procedures: usize,
}

impl Default for ProceduralMemory {
    fn default() -> Self {
        Self {
            procedures: Vec::new(),
            max_procedures: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureRecord {
    pub id: u64,
    pub name: String,
    pub trigger_pattern: String,
    pub steps: Vec<String>,
    pub success_rate: f64,
    pub invocation_count: u64,
    pub created_at: u64,
}

// ── Stats ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationStats {
    pub working_count: usize,
    pub working_capacity: usize,
    pub episodic_count: usize,
    pub episodic_max: usize,
    pub semantic_count: usize,
    pub semantic_max: usize,
    pub procedural_count: usize,
    pub procedural_max: usize,
    pub total: usize,
    pub cycle: u64,
}

// ── Pipeline ──

pub struct MemoryConsolidationPipeline {
    pub working: WorkingMemory,
    pub episodic: EpisodicMemory,
    pub semantic: SemanticMemory,
    pub procedural: ProceduralMemory,
    pub next_id: u64,
    pub cycle: u64,
}

impl MemoryConsolidationPipeline {
    pub fn new() -> Self {
        Self {
            working: WorkingMemory::default(),
            episodic: EpisodicMemory::default(),
            semantic: SemanticMemory::default(),
            procedural: ProceduralMemory::default(),
            next_id: 1,
            cycle: 0,
        }
    }

    /// Add a new observation to working memory with SHA-256 dedup within window.
    pub fn observe(&mut self, content: &str, source: &str, importance: f64) -> u64 {
        let now = now_secs();
        let hash = sha256(content);

        // Dedup: check dedup_hashes for matching hash within window
        let dedup_hit = self
            .working
            .dedup_hashes
            .iter()
            .any(|(h, t)| *h == hash && (now - *t) < self.working.dedup_window_secs);
        if dedup_hit {
            return 0;
        }

        let id = self.next_id;
        self.next_id += 1;

        self.working.items.push_back(WorkingItem {
            id,
            content: content.to_string(),
            source: source.to_string(),
            timestamp: now,
            content_hash: hash,
            importance,
        });

        // Record hash in dedup_hashes
        self.working.dedup_hashes.push_back((hash, now));

        // Prune dedup_hashes older than window
        while let Some(front) = self.working.dedup_hashes.front() {
            if now - front.1 >= self.working.dedup_window_secs {
                self.working.dedup_hashes.pop_front();
            } else {
                break;
            }
        }

        // Evict if over capacity
        if self.working.items.len() > self.working.capacity {
            self.consolidate_working_to_episodic(5);
        }

        id
    }

    /// Run one full consolidation cycle: Working→Episodic→Semantic→Procedural + decay + prune.
    pub fn tick(&mut self) {
        self.cycle += 1;

        // Step 1: Migrate 3 oldest Working items → Episodic
        self.consolidate_working_to_episodic(3);

        // Step 2: Consolidate expired episodes → Semantic
        self.consolidate_episodic_to_semantic();

        // Step 3: Detect repeated patterns → extract Procedures
        self.extract_procedures();

        // Step 4: Apply Ebbinghaus decay
        self.decay_all();

        // Step 5: Prune overspill
        self.prune();
    }

    /// Batch migrate Working→Episodic with compression.
    pub fn consolidate_working_to_episodic(&mut self, count: usize) {
        let now = now_secs();
        for _ in 0..count.min(self.working.items.len()) {
            if let Some(item) = self.working.items.pop_front() {
                let summary = if item.content.len() > 200 {
                    format!("{}...", &item.content[..197])
                } else {
                    item.content.clone()
                };

                let key_events = extract_key_events(&item.content);

                let ep_id = self.next_id;
                self.next_id += 1;
                let episode = EpisodeRecord {
                    id: ep_id,
                    summary,
                    key_events,
                    start_time: item.timestamp,
                    end_time: now,
                    valence: 0.0,
                    importance: item.importance,
                    access_count: 0,
                    consolidated: false,
                };

                self.episodic.episodes.push(episode);
            }
        }

        // Enforce capacity — keep HIGHEST importance
        if self.episodic.episodes.len() > self.episodic.max_episodes {
            self.episodic.episodes.sort_by(|a, b| {
                b.importance
                    .partial_cmp(&a.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.episodic.episodes.truncate(self.episodic.max_episodes);
        }
    }

    /// Find expired episodes and consolidate to Semantic memory.
    pub fn consolidate_episodic_to_semantic(&mut self) {
        let expired_ids: Vec<u64> = self
            .episodic
            .episodes
            .iter()
            .filter(|e| !e.consolidated && episode_expired(e, self.episodic.decay_days))
            .map(|e| e.id)
            .collect();

        for ep_id in expired_ids {
            self.consolidate_single_episode(ep_id);
        }
    }

    /// Extract facts from a single episode into Semantic memory.
    fn consolidate_single_episode(&mut self, episode_id: u64) {
        let idx = match self
            .episodic
            .episodes
            .iter()
            .position(|e| e.id == episode_id)
        {
            Some(i) => i,
            None => return,
        };

        let episode = &self.episodic.episodes[idx];
        let now = now_secs();
        let facts = extract_facts(&episode.summary);

        let ep_id = episode_id;
        for fact_content in facts {
            let existing = self
                .semantic
                .facts
                .iter_mut()
                .find(|f| f.content.to_lowercase() == fact_content.to_lowercase());

            if let Some(existing_fact) = existing {
                existing_fact.confidence = (existing_fact.confidence + 0.15).min(1.0);
                if !existing_fact.source_episode_ids.contains(&ep_id) {
                    existing_fact.source_episode_ids.push(ep_id);
                }
                existing_fact.access_count += 1;
                existing_fact.last_verified = now;
            } else {
                let fact_id = self.next_id;
                self.next_id += 1;
                self.semantic.facts.push(SemanticFact {
                    id: fact_id,
                    content: fact_content,
                    confidence: episode.importance * 0.7,
                    source_episode_ids: vec![ep_id],
                    access_count: 0,
                    created_at: now,
                    last_verified: now,
                });
            }
        }

        if let Some(ep) = self.episodic.episodes.get_mut(idx) {
            ep.consolidated = true;
        }

        let flagged = self.detect_contradictions();
        if !flagged.is_empty() {
            log::debug!(
                "[memory] {} low-confidence facts flagged for review",
                flagged.len()
            );
        }
    }

    /// Detect facts whose confidence has dropped below threshold.
    /// Low-confidence facts get their confidence halved; if below 0.05, removed.
    pub fn detect_contradictions(&mut self) -> Vec<u64> {
        let mut flagged = Vec::new();
        let mut to_remove = Vec::new();
        for fact in &self.semantic.facts {
            if fact.confidence < 0.2 {
                flagged.push(fact.id);
                if fact.confidence < 0.05 {
                    to_remove.push(fact.id);
                }
            }
        }
        // Remove critically low-confidence facts
        self.semantic.facts.retain(|f| !to_remove.contains(&f.id));
        // Halve confidence for borderline facts (handled on next decay cycle)
        for fact in &mut self.semantic.facts {
            if fact.confidence < 0.2 && fact.confidence >= 0.05 {
                fact.confidence *= 0.5;
            }
        }
        flagged
    }

    /// Scan semantic facts for repeated patterns → extract procedures.
    pub fn extract_procedures(&mut self) {
        if self.semantic.facts.len() < 3 {
            return;
        }

        // Find facts with overlapping 3-word phrases
        let mut phrase_freq: HashMap<String, Vec<u64>> = HashMap::new();
        for fact in &self.semantic.facts {
            let words: Vec<&str> = fact.content.split_whitespace().collect();
            for window in words.windows(3) {
                let phrase = window.join(" ");
                phrase_freq.entry(phrase).or_default().push(fact.id);
            }
        }

        // Find phrases shared by >=3 facts, not already a procedure
        for (phrase, ids) in &phrase_freq {
            if ids.len() < 3 {
                continue;
            }
            let already_procedure = self.procedural.procedures.iter().any(|p| p.name == *phrase);
            if already_procedure {
                continue;
            }

            let now = now_secs();
            let pid = self.next_id;
            self.next_id += 1;

            let steps: Vec<String> = ids
                .iter()
                .filter_map(|id| self.semantic.facts.iter().find(|f| f.id == *id))
                .map(|f| f.content.clone())
                .collect();

            self.procedural.procedures.push(ProcedureRecord {
                id: pid,
                name: phrase.clone(),
                trigger_pattern: phrase.clone(),
                steps,
                success_rate: 0.5,
                invocation_count: 0,
                created_at: now,
            });
        }

        // Enforce capacity — keep HIGHEST scored procedures
        if self.procedural.procedures.len() > self.procedural.max_procedures {
            self.procedural.procedures.sort_by(|a, b| {
                let a_score = a.success_rate * (a.invocation_count as f64).max(1.0);
                let b_score = b.success_rate * (b.invocation_count as f64).max(1.0);
                b_score
                    .partial_cmp(&a_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.procedural
                .procedures
                .truncate(self.procedural.max_procedures);
        }
    }

    /// Search within a specific memory tier, returning content + score pairs.
    pub fn search(&self, query: &str, tier: MemoryTier) -> Vec<(String, f64)> {
        let q = query.to_lowercase();
        let q_tokens: Vec<&str> = q.split_whitespace().collect();
        if q_tokens.is_empty() {
            return Vec::new();
        }

        let score_fn = |text: &str| -> f64 {
            let lower = text.to_lowercase();
            let matches = q_tokens.iter().filter(|t| lower.contains(*t)).count();
            matches as f64 / q_tokens.len() as f64
        };

        match tier {
            MemoryTier::Working => {
                let mut results: Vec<(String, f64)> = self
                    .working
                    .items
                    .iter()
                    .map(|item| (item.content.clone(), score_fn(&item.content)))
                    .filter(|(_, s)| *s > 0.0)
                    .collect();
                results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                results.truncate(10);
                results
            }
            MemoryTier::Episodic => {
                let mut results: Vec<(String, f64)> = self
                    .episodic
                    .episodes
                    .iter()
                    .map(|ep| (ep.summary.clone(), score_fn(&ep.summary)))
                    .filter(|(_, s)| *s > 0.0)
                    .collect();
                results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                results.truncate(10);
                results
            }
            MemoryTier::Semantic => {
                let mut results: Vec<(String, f64)> = self
                    .semantic
                    .facts
                    .iter()
                    .map(|f| (f.content.clone(), score_fn(&f.content) * f.confidence))
                    .filter(|(_, s)| *s > 0.0)
                    .collect();
                results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                results.truncate(10);
                results
            }
            MemoryTier::Procedural => {
                let mut results: Vec<(String, f64)> = self
                    .procedural
                    .procedures
                    .iter()
                    .map(|p| (p.name.clone(), score_fn(&p.name)))
                    .filter(|(_, s)| *s > 0.0)
                    .collect();
                results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                results.truncate(10);
                results
            }
        }
    }

    /// Search across all tiers, returning ranked results with tier weights.
    pub fn search_all(&self, query: &str) -> Vec<(MemoryTier, String, f64)> {
        let q = query.to_lowercase();
        let q_tokens: Vec<&str> = q.split_whitespace().collect();
        if q_tokens.is_empty() {
            return Vec::new();
        }

        let score_fn = |text: &str| -> f64 {
            let lower = text.to_lowercase();
            let matches = q_tokens.iter().filter(|t| lower.contains(*t)).count();
            matches as f64 / q_tokens.len() as f64
        };

        let mut results = Vec::new();

        for item in &self.working.items {
            let s = score_fn(&item.content);
            if s > 0.0 {
                results.push((MemoryTier::Working, item.content.clone(), s * 1.0));
            }
        }

        for ep in &self.episodic.episodes {
            let s = score_fn(&ep.summary);
            if s > 0.0 {
                results.push((MemoryTier::Episodic, ep.summary.clone(), s * 0.9));
            }
        }

        for fact in &self.semantic.facts {
            let s = score_fn(&fact.content);
            if s > 0.0 {
                results.push((
                    MemoryTier::Semantic,
                    fact.content.clone(),
                    s * 0.8 * fact.confidence,
                ));
            }
        }

        for proc in &self.procedural.procedures {
            let s = score_fn(&proc.name).max(score_fn(&proc.trigger_pattern));
            if s > 0.0 {
                results.push((MemoryTier::Procedural, proc.name.clone(), s * 0.7));
            }
        }

        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(20);
        results
    }

    /// Get statistics for all tiers.
    pub fn stats(&self) -> ConsolidationStats {
        ConsolidationStats {
            working_count: self.working.items.len(),
            working_capacity: self.working.capacity,
            episodic_count: self.episodic.episodes.len(),
            episodic_max: self.episodic.max_episodes,
            semantic_count: self.semantic.facts.len(),
            semantic_max: self.semantic.max_facts,
            procedural_count: self.procedural.procedures.len(),
            procedural_max: self.procedural.max_procedures,
            total: self.working.items.len()
                + self.episodic.episodes.len()
                + self.semantic.facts.len()
                + self.procedural.procedures.len(),
            cycle: self.cycle,
        }
    }

    /// Prune: keep only top items per tier by importance/confidence/success_rate.
    pub fn prune(&mut self) {
        // Working: drain to 80% capacity, keep highest importance
        let working_target = (self.working.capacity as f64 * 0.8) as usize;
        if self.working.items.len() > working_target {
            self.working.items.make_contiguous().sort_by(|a, b| {
                b.importance
                    .partial_cmp(&a.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.working.items.truncate(working_target);
        }

        // Episodic: keep top max_episodes by importance
        if self.episodic.episodes.len() > self.episodic.max_episodes {
            self.episodic.episodes.sort_by(|a, b| {
                b.importance
                    .partial_cmp(&a.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.episodic.episodes.truncate(self.episodic.max_episodes);
        }

        // Semantic: keep top max_facts by confidence
        if self.semantic.facts.len() > self.semantic.max_facts {
            self.semantic.facts.sort_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.semantic.facts.truncate(self.semantic.max_facts);
        }

        // Procedural: keep top max_procedures by success_rate
        if self.procedural.procedures.len() > self.procedural.max_procedures {
            self.procedural.procedures.sort_by(|a, b| {
                b.success_rate
                    .partial_cmp(&a.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.procedural
                .procedures
                .truncate(self.procedural.max_procedures);
        }
    }

    /// Apply Ebbinghaus decay to all tiers.
    pub fn decay_all(&mut self) {
        let now = now_secs();

        // Decay semantic facts
        for fact in &mut self.semantic.facts {
            let days_since_access = if fact.last_verified == 0 {
                0.0
            } else {
                (now - fact.last_verified) as f64 / 86400.0
            };
            fact.confidence *= ebbinghaus_retention(days_since_access, 7.0);

            // Boost confidence on access_count
            let access_boost = (fact.access_count as f64).min(10.0) * 0.05;
            fact.confidence = (fact.confidence + access_boost).min(1.0);
        }

        // Decay episodic importance
        for ep in &mut self.episodic.episodes {
            let days_since = (now - ep.end_time) as f64 / 86400.0;
            let decay = ebbinghaus_retention(days_since, 14.0);
            ep.importance *= decay;
        }

        // Decay working item importance (faster decay)
        for item in &mut self.working.items {
            let days_since = (now - item.timestamp) as f64 / 86400.0;
            let decay = ebbinghaus_retention(days_since, 3.0);
            item.importance *= decay;
        }
    }
}

impl Default for MemoryConsolidationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Ebbinghaus forgetting curve: R = e^(-t/S)
pub fn ebbinghaus_retention(days_since: f64, stability_days: f64) -> f64 {
    (-days_since / stability_days).exp()
}

/// Check if an episode has expired by TTL.
fn episode_expired(episode: &EpisodeRecord, decay_days: u64) -> bool {
    let now = now_secs();
    let age_secs = now - episode.end_time;
    age_secs > decay_days * 86400
}

/// SHA-256 hash of content bytes using the sha2 crate.
fn sha256(content: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

fn now_secs() -> u64 {
    crate::core::nt_core_time::unix_now_secs()
}

fn extract_key_events(content: &str) -> Vec<String> {
    let mut events = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.len() > 10 && trimmed.len() < 200 {
            let lower = trimmed.to_lowercase();
            if lower.contains("error")
                || lower.contains("result")
                || lower.contains("found")
                || lower.contains("done")
                || lower.contains("completed")
                || lower.contains("failed")
            {
                events.push(trimmed.to_string());
            }
        }
    }
    if events.is_empty() && !content.is_empty() {
        events.push(content.chars().take(100).collect());
    }
    events.truncate(5);
    events
}

fn extract_facts(summary: &str) -> Vec<String> {
    let mut facts = Vec::new();
    for line in summary.lines() {
        let trimmed = line.trim();
        if trimmed.len() > 15 && trimmed.len() < 300 {
            let lower = trimmed.to_lowercase();
            if lower.contains(" is ")
                || lower.contains(" uses ")
                || lower.contains(" has ")
                || lower.contains(" was ")
                || lower.contains(" are ")
                || lower.contains(" been ")
                || lower.contains(" contains ")
                || lower.contains(" requires ")
            {
                facts.push(trimmed.to_string());
            }
        }
    }
    if facts.is_empty() && !summary.is_empty() {
        facts.push(summary.chars().take(200).collect());
    }
    facts.truncate(10);
    facts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observe_adds_to_working() {
        let mut p = MemoryConsolidationPipeline::new();
        let id = p.observe("test observation", "user_message", 0.5);
        assert!(id > 0);
        assert_eq!(p.working.items.len(), 1);
        assert_eq!(p.working.items[0].source, "user_message");
        assert_eq!(p.working.items[0].importance, 0.5);
    }

    #[test]
    fn test_dedup_returns_zero_for_duplicate() {
        let mut p = MemoryConsolidationPipeline::new();
        let id1 = p.observe("duplicate content", "user_message", 0.3);
        let id2 = p.observe("duplicate content", "user_message", 0.3);
        assert!(id1 > 0);
        assert_eq!(id2, 0);
        assert_eq!(p.working.items.len(), 1);
    }

    #[test]
    fn test_dedup_different_content_allows() {
        let mut p = MemoryConsolidationPipeline::new();
        let id1 = p.observe("content A", "user_message", 0.3);
        let id2 = p.observe("content B", "user_message", 0.3);
        assert!(id1 > 0);
        assert!(id2 > 0);
        assert_eq!(p.working.items.len(), 2);
    }

    #[test]
    fn test_working_overflow_triggers_consolidation() {
        let mut p = MemoryConsolidationPipeline::new();
        p.working.capacity = 2;
        let _id1 = p.observe("item 1", "tool_call", 0.5);
        let _id2 = p.observe("item 2", "tool_call", 0.5);
        let id3 = p.observe("item 3", "tool_call", 0.5);
        assert!(id3 > 0);
        assert_eq!(p.working.items.len(), 2);
        assert_eq!(p.episodic.episodes.len(), 1);
    }

    #[test]
    fn test_consolidation_creates_episode() {
        let mut p = MemoryConsolidationPipeline::new();
        p.observe("first observation", "tool_call", 0.7);
        p.observe("second observation", "tool_call", 0.5);
        let len_before = p.episodic.episodes.len();
        p.consolidate_working_to_episodic(1);
        assert_eq!(p.working.items.len(), 1);
        assert_eq!(p.episodic.episodes.len(), len_before + 1);
        assert!(p
            .episodic
            .episodes
            .last()
            .unwrap()
            .summary
            .contains("first"));
    }

    #[test]
    fn test_episodic_to_semantic_on_expiry() {
        let mut p = MemoryConsolidationPipeline::new();
        let now = now_secs();
        let ep_id = p.next_id;
        p.next_id += 1;
        p.episodic.episodes.push(EpisodeRecord {
            id: ep_id,
            summary: "The system uses PostgreSQL for storage and is deployed on AWS.".to_string(),
            key_events: vec![],
            start_time: now - 800_000,
            end_time: now - 700_000,
            valence: 0.0,
            importance: 0.8,
            access_count: 0,
            consolidated: false,
        });
        p.consolidate_single_episode(ep_id);
        assert!(p.semantic.facts.len() >= 1);
        assert!(p.episodic.episodes[0].consolidated);
    }

    #[test]
    fn test_procedure_extraction_from_repeated_facts() {
        let mut p = MemoryConsolidationPipeline::new();
        let now = now_secs();
        let contents = [
            "The database uses PostgreSQL for storage.",
            "The database uses PostgreSQL for queries.",
            "The database uses PostgreSQL for replication.",
        ];
        for c in &contents {
            let fid = p.next_id;
            p.next_id += 1;
            p.semantic.facts.push(SemanticFact {
                id: fid,
                content: c.to_string(),
                confidence: 0.8,
                source_episode_ids: vec![1],
                access_count: 3,
                created_at: now,
                last_verified: now,
            });
        }
        p.extract_procedures();
        assert_eq!(p.procedural.procedures.len(), 1);
        assert!(p.procedural.procedures[0]
            .name
            .contains("The database uses PostgreSQL"));
    }

    #[test]
    fn test_ebbinghaus_retention_formula() {
        let r0 = ebbinghaus_retention(0.0, 7.0);
        assert!((r0 - 1.0).abs() < 1e-6);
        let r1 = ebbinghaus_retention(7.0, 7.0);
        assert!((r1 - (-1.0f64).exp()).abs() < 1e-6);
        let r2 = ebbinghaus_retention(14.0, 7.0);
        assert!((r2 - (-2.0f64).exp()).abs() < 1e-6);
        assert!(r0 > r1);
        assert!(r1 > r2);
    }

    #[test]
    fn test_decay_reduces_confidence() {
        let mut p = MemoryConsolidationPipeline::new();
        let now = now_secs();
        let fid = p.next_id;
        p.next_id += 1;
        p.semantic.facts.push(SemanticFact {
            id: fid,
            content: "test fact".to_string(),
            confidence: 1.0,
            source_episode_ids: vec![],
            access_count: 0,
            created_at: now - 86400 * 30,
            last_verified: now - 86400 * 30,
        });
        p.decay_all();
        assert!(p.semantic.facts[0].confidence < 1.0);
        assert!(p.semantic.facts[0].confidence > 0.0);
    }

    #[test]
    fn test_prune_keeps_top_items() {
        let mut p = MemoryConsolidationPipeline::new();
        p.working.capacity = 10;
        for i in 0..10 {
            p.observe(&format!("item {}", i), "test", i as f64 * 0.1);
        }
        p.prune();
        assert_eq!(p.working.items.len(), 8);
        assert!(p.working.items[0].importance >= p.working.items[7].importance);
    }

    #[test]
    fn test_search_returns_matching_results() {
        let mut p = MemoryConsolidationPipeline::new();
        p.observe("rust ownership and borrowing", "tool_call", 0.5);
        p.observe("python async programming", "tool_call", 0.5);
        let results = p.search("rust", MemoryTier::Working);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "rust ownership and borrowing");
        assert!(results[0].1 > 0.0);
    }

    #[test]
    fn test_search_all_merges_across_tiers() {
        let mut p = MemoryConsolidationPipeline::new();
        p.observe("working memory about Rust", "tool_call", 0.5);
        let now = now_secs();
        let ep_id = p.next_id;
        p.next_id += 1;
        p.episodic.episodes.push(EpisodeRecord {
            id: ep_id,
            summary: "episodic memory about Rust".to_string(),
            key_events: vec![],
            start_time: now - 1000,
            end_time: now,
            valence: 0.5,
            importance: 0.7,
            access_count: 0,
            consolidated: true,
        });
        let results = p.search_all("Rust");
        assert!(results.len() >= 2);
        assert_eq!(results[0].0, MemoryTier::Working);
    }

    #[test]
    fn test_stats_returns_correct_counts() {
        let mut p = MemoryConsolidationPipeline::new();
        p.observe("test data", "user", 0.5);
        let stats = p.stats();
        assert_eq!(stats.working_count, 1);
        assert_eq!(stats.total, 1);
    }

    #[test]
    fn test_empty_pipeline_stats() {
        let p = MemoryConsolidationPipeline::new();
        let stats = p.stats();
        assert_eq!(stats.working_count, 0);
        assert_eq!(stats.episodic_count, 0);
        assert_eq!(stats.semantic_count, 0);
        assert_eq!(stats.procedural_count, 0);
        assert_eq!(stats.total, 0);
    }

    #[test]
    fn test_tick_runs_without_error() {
        let mut p = MemoryConsolidationPipeline::new();
        p.observe("first", "test", 0.5);
        p.observe("second", "test", 0.5);
        p.tick();
        assert!(p.cycle >= 1);
    }

    #[test]
    fn test_tick_multiple_times_accumulates() {
        let mut p = MemoryConsolidationPipeline::new();
        for _ in 0..5 {
            p.tick();
        }
        assert_eq!(p.cycle, 5);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut p = MemoryConsolidationPipeline::new();
        p.observe("test data", "user", 0.5);
        let json = serde_json::to_string(&p.working).expect("serialize working");
        let _back: WorkingMemory = serde_json::from_str(&json).expect("deserialize working");
        let json_ep = serde_json::to_string(&p.episodic).expect("serialize episodic");
        let _back_ep: EpisodicMemory =
            serde_json::from_str(&json_ep).expect("deserialize episodic");
    }

    #[test]
    fn test_contradiction_detection() {
        let mut p = MemoryConsolidationPipeline::new();
        let now = now_secs();
        for i in 0..5 {
            let fid = p.next_id;
            p.next_id += 1;
            p.semantic.facts.push(SemanticFact {
                id: fid,
                content: format!("low confidence fact {}", i),
                confidence: 0.1,
                source_episode_ids: vec![],
                access_count: 0,
                created_at: now,
                last_verified: now,
            });
        }
        let flagged = p.detect_contradictions();
        assert_eq!(flagged.len(), 5);
    }
}

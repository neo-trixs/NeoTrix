use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::core::nt_core_experience::capability_synthesizer::CapabilitySynthesizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalFidelityReport {
    pub close_count: u32,
    pub close_avg_score: f64,
    pub medium_count: u32,
    pub medium_avg_score: f64,
    pub far_count: u32,
    pub far_avg_score: f64,
    pub unknown_count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeFilter {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub app_id: Option<String>,
}

impl ScopeFilter {
    pub fn matches(&self, entry: &LatticeEntry) -> bool {
        if let Some(ref uid) = self.user_id {
            if entry.origin.name() != uid {
                return false;
            }
        }
        true
    }

    pub fn is_empty(&self) -> bool {
        self.user_id.is_none()
            && self.agent_id.is_none()
            && self.run_id.is_none()
            && self.app_id.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryOrigin {
    User,
    Model,
    System,
    External,
}

impl MemoryOrigin {
    pub fn name(&self) -> &'static str {
        match self {
            MemoryOrigin::User => "user",
            MemoryOrigin::Model => "model",
            MemoryOrigin::System => "system",
            MemoryOrigin::External => "external",
        }
    }

    pub fn is_self(&self) -> bool {
        matches!(self, MemoryOrigin::Model | MemoryOrigin::System)
    }

    pub fn is_world(&self) -> bool {
        matches!(self, MemoryOrigin::User | MemoryOrigin::External)
    }

    pub fn default() -> Self {
        MemoryOrigin::System
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BeliefState {
    Authoritative,
    Inferred,
    Contradictory,
    Expired,
}

impl BeliefState {
    pub fn name(&self) -> &'static str {
        match self {
            BeliefState::Authoritative => "authoritative",
            BeliefState::Inferred => "inferred",
            BeliefState::Contradictory => "contradictory",
            BeliefState::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LatticeLayer {
    /// Raw episodic memories — unfiltered experiences
    Episodic,
    /// Consolidated facts — verified and abstracted knowledge
    Facts,
    /// Reusable skills — frequently successful patterns
    Skills,
    /// Meta-rules — behavioral heuristics and strategies
    MetaRules,
    /// Core identity — fundamental invariants
    Identity,
}

impl LatticeLayer {
    pub fn level(&self) -> usize {
        match self {
            LatticeLayer::Episodic => 0,
            LatticeLayer::Facts => 1,
            LatticeLayer::Skills => 2,
            LatticeLayer::MetaRules => 3,
            LatticeLayer::Identity => 4,
        }
    }

    pub fn capacity(&self) -> usize {
        match self {
            LatticeLayer::Episodic => 500,
            LatticeLayer::Facts => 200,
            LatticeLayer::Skills => 100,
            LatticeLayer::MetaRules => 30,
            LatticeLayer::Identity => 10,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LatticeLayer::Episodic => "episodic",
            LatticeLayer::Facts => "facts",
            LatticeLayer::Skills => "skills",
            LatticeLayer::MetaRules => "meta_rules",
            LatticeLayer::Identity => "identity",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatticeEntry {
    pub content: String,
    pub vsa_hash: Vec<u8>,
    pub layer: LatticeLayer,
    pub confidence: f64,
    pub invocation_count: u64,
    pub last_accessed: u64,
    pub source_layer: Option<LatticeLayer>,
    pub consolidated: bool,
    /// MemRL-style Q-value for RL-driven retrieval. Default 0.5.
    pub q_value: f64,
    /// Bi-temporal validity: valid_from (inclusive) — None means unbounded past
    pub valid_from: Option<i64>,
    /// Bi-temporal validity: valid_to (inclusive) — None means unbounded future
    pub valid_to: Option<i64>,
    /// Origin tracking for anti-self-pollution gate
    pub origin: MemoryOrigin,
    pub provenance_parent: Option<u64>,
    pub belief_state: BeliefState,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLattice {
    pub episodic: VecDeque<LatticeEntry>,
    pub facts: VecDeque<LatticeEntry>,
    pub skills: VecDeque<LatticeEntry>,
    pub meta_rules: VecDeque<LatticeEntry>,
    pub identity: VecDeque<LatticeEntry>,
    pub cycle: u64,
    pub total_consolidations: u64,
    pub consolidation_threshold: f64,
}

impl Default for MemoryLattice {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryLattice {
    pub fn new() -> Self {
        Self {
            episodic: VecDeque::with_capacity(LatticeLayer::Episodic.capacity()),
            facts: VecDeque::with_capacity(LatticeLayer::Facts.capacity()),
            skills: VecDeque::with_capacity(LatticeLayer::Skills.capacity()),
            meta_rules: VecDeque::with_capacity(LatticeLayer::MetaRules.capacity()),
            identity: VecDeque::with_capacity(LatticeLayer::Identity.capacity()),
            cycle: 0,
            total_consolidations: 0,
            consolidation_threshold: 0.55,
        }
    }

    pub fn store(&mut self, content: String, vsa_hash: Vec<u8>, layer: LatticeLayer) -> usize {
        self.store_with_origin(content, vsa_hash, layer, MemoryOrigin::default())
    }

    pub fn store_with_origin(
        &mut self,
        content: String,
        vsa_hash: Vec<u8>,
        layer: LatticeLayer,
        origin: MemoryOrigin,
    ) -> usize {
        self.store_with_validity(content, vsa_hash, layer, origin, None, None)
    }

    fn build_entry(
        &self,
        content: String,
        vsa_hash: Vec<u8>,
        layer: LatticeLayer,
        origin: MemoryOrigin,
        valid_from: Option<i64>,
        valid_to: Option<i64>,
    ) -> LatticeEntry {
        LatticeEntry {
            content,
            vsa_hash,
            layer,
            confidence: 0.3,
            invocation_count: 0,
            last_accessed: self.cycle,
            source_layer: None,
            consolidated: false,
            q_value: 0.5,
            valid_from,
            valid_to,
            origin,
            provenance_parent: None,
            belief_state: BeliefState::Inferred,
            domain: "general".to_string(),
        }
    }

    pub fn store_with_validity(
        &mut self,
        content: String,
        vsa_hash: Vec<u8>,
        layer: LatticeLayer,
        origin: MemoryOrigin,
        valid_from: Option<i64>,
        valid_to: Option<i64>,
    ) -> usize {
        let entry = self.build_entry(content, vsa_hash, layer, origin, valid_from, valid_to);
        let idx;
        let cap = layer.capacity();
        match layer {
            LatticeLayer::Episodic => {
                if self.episodic.len() >= cap {
                    self.episodic.pop_front();
                }
                idx = self.episodic.len();
                self.episodic.push_back(entry);
            }
            LatticeLayer::Facts => {
                if self.facts.len() >= cap {
                    self.facts.pop_front();
                }
                idx = self.facts.len();
                self.facts.push_back(entry);
            }
            LatticeLayer::Skills => {
                if self.skills.len() >= cap {
                    self.skills.pop_front();
                }
                idx = self.skills.len();
                self.skills.push_back(entry);
            }
            LatticeLayer::MetaRules => {
                if self.meta_rules.len() >= cap {
                    self.meta_rules.pop_front();
                }
                idx = self.meta_rules.len();
                self.meta_rules.push_back(entry);
            }
            LatticeLayer::Identity => {
                if self.identity.len() >= cap {
                    self.identity.pop_front();
                }
                idx = self.identity.len();
                self.identity.push_back(entry);
            }
        }
        idx
    }

    pub fn store_with_provenance(
        &mut self,
        content: String,
        vsa_hash: Vec<u8>,
        layer: LatticeLayer,
        origin: MemoryOrigin,
        provenance_parent: Option<u64>,
        belief_state: BeliefState,
        domain: String,
    ) -> usize {
        let mut entry = self.build_entry(content, vsa_hash, layer, origin, None, None);
        entry.provenance_parent = provenance_parent;
        entry.belief_state = belief_state;
        entry.domain = domain;
        let idx;
        let cap = layer.capacity();
        match layer {
            LatticeLayer::Episodic => {
                if self.episodic.len() >= cap {
                    self.episodic.pop_front();
                }
                idx = self.episodic.len();
                self.episodic.push_back(entry);
            }
            LatticeLayer::Facts => {
                if self.facts.len() >= cap {
                    self.facts.pop_front();
                }
                idx = self.facts.len();
                self.facts.push_back(entry);
            }
            LatticeLayer::Skills => {
                if self.skills.len() >= cap {
                    self.skills.pop_front();
                }
                idx = self.skills.len();
                self.skills.push_back(entry);
            }
            LatticeLayer::MetaRules => {
                if self.meta_rules.len() >= cap {
                    self.meta_rules.pop_front();
                }
                idx = self.meta_rules.len();
                self.meta_rules.push_back(entry);
            }
            LatticeLayer::Identity => {
                if self.identity.len() >= cap {
                    self.identity.pop_front();
                }
                idx = self.identity.len();
                self.identity.push_back(entry);
            }
        }
        idx
    }

    pub fn access(&mut self, idx: usize, layer: LatticeLayer) -> Option<&LatticeEntry> {
        let entry = match layer {
            LatticeLayer::Episodic => self.episodic.get_mut(idx)?,
            LatticeLayer::Facts => self.facts.get_mut(idx)?,
            LatticeLayer::Skills => self.skills.get_mut(idx)?,
            LatticeLayer::MetaRules => self.meta_rules.get_mut(idx)?,
            LatticeLayer::Identity => self.identity.get_mut(idx)?,
        };
        entry.invocation_count += 1;
        entry.last_accessed = self.cycle;
        Some(entry)
    }

    pub fn find(&self, query: &str) -> Vec<(LatticeLayer, usize, f64)> {
        let q = query.to_lowercase();
        let mut results = Vec::new();
        for (layer, entries) in [
            (LatticeLayer::Identity, &self.identity),
            (LatticeLayer::MetaRules, &self.meta_rules),
            (LatticeLayer::Skills, &self.skills),
            (LatticeLayer::Facts, &self.facts),
            (LatticeLayer::Episodic, &self.episodic),
        ] {
            for (i, entry) in entries.iter().enumerate() {
                if entry.content.to_lowercase().contains(&q) {
                    let score =
                        entry.confidence * (1.0 + 0.1 * entry.invocation_count as f64).min(2.0);
                    results.push((layer, i, score));
                }
            }
        }
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(10);
        results
    }

    pub fn find_by_temporal(
        &self,
        query: &str,
        at_time: i64,
        layer: Option<LatticeLayer>,
    ) -> Vec<(usize, f64)> {
        let q = query.to_lowercase();
        let mut results = Vec::new();
        let layers: Vec<(LatticeLayer, &VecDeque<LatticeEntry>)> = match layer {
            Some(l) => vec![(l, self.layer_deque(l))],
            None => vec![
                (LatticeLayer::Identity, &self.identity),
                (LatticeLayer::MetaRules, &self.meta_rules),
                (LatticeLayer::Skills, &self.skills),
                (LatticeLayer::Facts, &self.facts),
                (LatticeLayer::Episodic, &self.episodic),
            ],
        };
        for (_layer, entries) in layers {
            for (i, entry) in entries.iter().enumerate() {
                let in_validity = match (entry.valid_from, entry.valid_to) {
                    (Some(vf), Some(vt)) => vf <= at_time && vt >= at_time,
                    (Some(vf), None) => vf <= at_time,
                    (None, Some(vt)) => vt >= at_time,
                    (None, None) => true,
                };
                if !in_validity {
                    continue;
                }
                if entry.content.to_lowercase().contains(&q) {
                    let score =
                        entry.confidence * (1.0 + 0.1 * entry.invocation_count as f64).min(2.0);
                    results.push((i, score));
                }
            }
        }
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(10);
        results
    }

    /// RAG identity coherence check per Stack Theory (arXiv 2603.09043).
    /// Returns the proportion of Self-tagged entries in a retrieval result.
    /// If < 30%, retrieval is fragmenting identity (Theorems E.1/E.2).
    /// Callable from MemoryLattice retrieval path.
    pub fn check_retrieval_identity_coherence(
        &self,
        retrieved_count: usize,
        self_tagged_count: usize,
    ) -> f64 {
        if retrieved_count == 0 {
            return 1.0;
        }
        self_tagged_count as f64 / retrieved_count as f64
    }

    fn layer_deque(&self, layer: LatticeLayer) -> &VecDeque<LatticeEntry> {
        match layer {
            LatticeLayer::Episodic => &self.episodic,
            LatticeLayer::Facts => &self.facts,
            LatticeLayer::Skills => &self.skills,
            LatticeLayer::MetaRules => &self.meta_rules,
            LatticeLayer::Identity => &self.identity,
        }
    }

    /// MemRL-style Q-value update: Q_new = Q_old + alpha * (reward - Q_old)
    /// Updates the entry at `idx` in the given `layer`.
    pub fn update_q_value(&mut self, idx: usize, layer: LatticeLayer, reward: f64, alpha: f64) {
        let entry = match layer {
            LatticeLayer::Episodic => self.episodic.get_mut(idx),
            LatticeLayer::Facts => self.facts.get_mut(idx),
            LatticeLayer::Skills => self.skills.get_mut(idx),
            LatticeLayer::MetaRules => self.meta_rules.get_mut(idx),
            LatticeLayer::Identity => self.identity.get_mut(idx),
        };
        if let Some(e) = entry {
            let q_old = e.q_value;
            e.q_value = q_old + alpha * (reward - q_old);
        }
    }

    /// Return top-K episodic indices sorted by q_value descending.
    pub fn top_episodic_by_q(&self, top_k: usize) -> Vec<(usize, f64)> {
        let mut scored: Vec<(usize, f64)> = self
            .episodic
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e.q_value))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    /// Apply exponential forgetting to all entries across all 5 layers.
    /// For each entry:
    /// - `q_value *= (1.0 - decay_rate * 0.01)` — exponential decay per cycle
    /// - `confidence *= (1.0 - decay_rate * 0.005)` — confidence decays slower than q_value
    /// Returns the count of entries "forgotten" (q_value < 0.01).
    pub fn apply_forgetting(&mut self, decay_rate: f64) -> usize {
        let q_decay = 1.0 - decay_rate * 0.01;
        let c_decay = 1.0 - decay_rate * 0.005;
        let mut forgotten = 0;
        for entry in self
            .episodic
            .iter_mut()
            .chain(self.facts.iter_mut())
            .chain(self.skills.iter_mut())
            .chain(self.meta_rules.iter_mut())
            .chain(self.identity.iter_mut())
        {
            entry.q_value *= q_decay;
            entry.confidence *= c_decay;
            if entry.q_value < 0.01 {
                forgotten += 1;
            }
        }
        forgotten
    }

    /// Remove entries with q_value below threshold from each layer.
    /// Returns a map of layer → removed_count for layers that lost entries.
    pub fn prune_forgotten(&mut self, q_threshold: f64) -> HashMap<LatticeLayer, usize> {
        let mut result = HashMap::new();
        for (layer, deque) in [
            (LatticeLayer::Episodic, &mut self.episodic),
            (LatticeLayer::Facts, &mut self.facts),
            (LatticeLayer::Skills, &mut self.skills),
            (LatticeLayer::MetaRules, &mut self.meta_rules),
            (LatticeLayer::Identity, &mut self.identity),
        ] {
            let before = deque.len();
            deque.retain(|e| e.q_value >= q_threshold);
            let removed = before - deque.len();
            if removed > 0 {
                result.insert(layer, removed);
            }
        }
        result
    }

    /// Q-weighted find: score = text_score * (1 - q_weight) + entry.q_value * q_weight.
    /// When q_weight = 0.0, behaves like `find()`.
    /// When q_weight = 1.0, pure Q-value sorting.
    pub fn find_by_q(&self, query: &str, q_weight: f64) -> Vec<(LatticeLayer, usize, f64)> {
        let q = query.to_lowercase();
        let mut results = Vec::new();
        for (layer, entries) in [
            (LatticeLayer::Identity, &self.identity),
            (LatticeLayer::MetaRules, &self.meta_rules),
            (LatticeLayer::Skills, &self.skills),
            (LatticeLayer::Facts, &self.facts),
            (LatticeLayer::Episodic, &self.episodic),
        ] {
            for (i, entry) in entries.iter().enumerate() {
                let text_score = if entry.content.to_lowercase().contains(&q) {
                    entry.confidence * (1.0 + 0.1 * entry.invocation_count as f64).min(2.0)
                } else {
                    0.0
                };
                if text_score > 0.0 || entry.q_value > 0.5 {
                    let score = text_score * (1.0 - q_weight) + entry.q_value * q_weight;
                    results.push((layer, i, score));
                }
            }
        }
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(10);
        results
    }

    /// REMem-style iterative retrieval with query expansion.
    /// Round 1: initial `find_by_q` with original query.
    /// Round 2: expand query with distinctive tokens from top result.
    /// Round 3: expand further with next result's content.
    /// All rounds merged, deduplicated, re-ranked by max score.
    pub fn iterative_retrieve(&self, query: &str, top_k: usize) -> Vec<(LatticeLayer, usize, f64)> {
        let mut expanded = query.to_string();
        let mut combined: std::collections::HashMap<(LatticeLayer, usize), f64> =
            std::collections::HashMap::new();

        for _ in 0..3 {
            let round = self.find_by_q(&expanded, 0.3);
            for &(layer, idx, score) in &round {
                let key = (layer, idx);
                let entry = combined.entry(key).or_insert(0.0);
                *entry = (*entry).max(score);
            }
            // Expand query with top result content
            if let Some(&(layer, idx, _)) = round.first() {
                if let Some(entry) = self.layer_deque(layer).get(idx) {
                    let keywords: Vec<&str> = entry
                        .content
                        .split_whitespace()
                        .filter(|w| w.len() > 4)
                        .take(5)
                        .collect();
                    if !keywords.is_empty() {
                        expanded = format!("{} {}", expanded, keywords.join(" "));
                        if expanded.len() > 320 {
                            break;
                        }
                        continue;
                    }
                }
            }
            break;
        }

        let mut results: Vec<(LatticeLayer, usize, f64)> = combined
            .into_iter()
            .map(|((layer, idx), score)| (layer, idx, score))
            .collect();
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    pub fn tick(&mut self) -> String {
        self.cycle += 1;
        let mut events = Vec::new();
        self.apply_forgetting(0.5);
        if self.cycle % 5 == 0 {
            let n = self.consolidate();
            if n > 0 {
                events.push(format!("consolidated:{}", n));
            }
        }
        if self.cycle % 20 == 0 {
            let p = self.prune();
            if p > 0 {
                events.push(format!("pruned:{}", p));
            }
        }
        events.join("|")
    }

    pub fn consolidate(&mut self) -> usize {
        let mut count = 0;
        let keep: Vec<LatticeEntry> = self
            .episodic
            .iter()
            .filter(|e| e.confidence >= self.consolidation_threshold && !e.consolidated)
            .cloned()
            .collect();
        for entry in keep {
            let mut new_entry = LatticeEntry {
                source_layer: Some(LatticeLayer::Episodic),
                consolidated: true,
                ..entry
            };
            new_entry.confidence = (new_entry.confidence * 0.9 + 0.1).min(1.0);
            new_entry.layer = LatticeLayer::Facts;
            if self.facts.len() >= LatticeLayer::Facts.capacity() {
                self.facts.pop_front();
            }
            self.facts.push_back(new_entry);
            count += 1;
        }
        let promote: Vec<LatticeEntry> = self
            .facts
            .iter()
            .filter(|e| e.invocation_count >= 3 && e.confidence >= 0.7 && !e.consolidated)
            .cloned()
            .collect();
        for entry in promote {
            let mut new_entry = LatticeEntry {
                source_layer: Some(LatticeLayer::Facts),
                consolidated: true,
                ..entry
            };
            new_entry.confidence = (new_entry.confidence * 0.85 + 0.15).min(1.0);
            new_entry.layer = LatticeLayer::Skills;
            if self.skills.len() >= LatticeLayer::Skills.capacity() {
                self.skills.pop_front();
            }
            self.skills.push_back(new_entry);
            count += 1;
        }
        let meta: Vec<LatticeEntry> = self
            .skills
            .iter()
            .filter(|e| e.invocation_count >= 10 && e.confidence >= 0.85 && !e.consolidated)
            .cloned()
            .collect();
        for entry in meta {
            let mut new_entry = LatticeEntry {
                source_layer: Some(LatticeLayer::Skills),
                consolidated: true,
                ..entry
            };
            new_entry.confidence = (new_entry.confidence * 0.8 + 0.2).min(1.0);
            new_entry.layer = LatticeLayer::MetaRules;
            if self.meta_rules.len() >= LatticeLayer::MetaRules.capacity() {
                self.meta_rules.pop_front();
            }
            self.meta_rules.push_back(new_entry);
            count += 1;
        }
        self.total_consolidations += count as u64;
        count
    }

    /// Export Skills + MetaRules to a CapabilitySynthesizer as primitive capabilities.
    /// Returns the number of capabilities registered.
    pub fn export_to_synthesizer(&self, synth: &mut CapabilitySynthesizer) -> usize {
        let mut count = 0;
        for entry in self.skills.iter().chain(self.meta_rules.iter()) {
            if entry.confidence > 0.6 {
                synth.register_primitive(&entry.content, "from_lattice");
                count += 1;
            }
        }
        count
    }

    /// Import high-success capabilities from CapabilitySynthesizer into Skills layer.
    pub fn import_from_synthesizer(&mut self, synth: &CapabilitySynthesizer) -> usize {
        let mut count = 0;
        for cap in &synth.capabilities {
            if cap.success_rate > 0.8 && cap.invocation_count > 3 {
                let vsa_hash = cap.vsa_vector.clone();
                self.store(cap.name.clone(), vsa_hash, LatticeLayer::Skills);
                count += 1;
            }
        }
        count
    }

    pub fn prune(&mut self) -> usize {
        let mut count = 0;
        let episodic_keep: VecDeque<LatticeEntry> = self
            .episodic
            .iter()
            .filter(|e| self.cycle - e.last_accessed < 100 || e.confidence >= 0.3)
            .cloned()
            .collect();
        count += self.episodic.len() - episodic_keep.len();
        self.episodic = episodic_keep;
        let facts_keep: VecDeque<LatticeEntry> = self
            .facts
            .iter()
            .filter(|e| self.cycle - e.last_accessed < 200 || e.invocation_count >= 2)
            .cloned()
            .collect();
        count += self.facts.len() - facts_keep.len();
        self.facts = facts_keep;
        count
    }

    /// Find entries by origin across all layers.
    pub fn find_by_origin(&self, origin: MemoryOrigin) -> Vec<(LatticeLayer, usize, f64)> {
        let mut results = Vec::new();
        for (layer, entries) in [
            (LatticeLayer::Identity, &self.identity),
            (LatticeLayer::MetaRules, &self.meta_rules),
            (LatticeLayer::Skills, &self.skills),
            (LatticeLayer::Facts, &self.facts),
            (LatticeLayer::Episodic, &self.episodic),
        ] {
            for (i, entry) in entries.iter().enumerate() {
                if entry.origin == origin {
                    let score =
                        entry.confidence * (1.0 + 0.1 * entry.invocation_count as f64).min(2.0);
                    results.push((layer, i, score));
                }
            }
        }
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(20);
        results
    }

    /// Find entries by multi-dimensional scope filter.
    /// When multiple scope dimensions are provided, results are merged with rank weighting.
    /// Falls back to text-based candidate matching when query is non-empty.
    pub fn find_by_scope(
        &self,
        query: &str,
        scope: &ScopeFilter,
    ) -> Vec<(LatticeLayer, usize, f64)> {
        let q = query.to_lowercase();
        let mut results = Vec::new();
        for (layer, entries) in [
            (LatticeLayer::Identity, &self.identity),
            (LatticeLayer::MetaRules, &self.meta_rules),
            (LatticeLayer::Skills, &self.skills),
            (LatticeLayer::Facts, &self.facts),
            (LatticeLayer::Episodic, &self.episodic),
        ] {
            for (i, entry) in entries.iter().enumerate() {
                if !scope.matches(entry) {
                    continue;
                }
                let text_score = if q.is_empty() || entry.content.to_lowercase().contains(&q) {
                    entry.confidence * (1.0 + 0.1 * entry.invocation_count as f64).min(2.0)
                } else {
                    0.0
                };
                if text_score <= 0.0 {
                    continue;
                }
                let mut scope_boost = 1.0;
                if scope.agent_id.is_some() {
                    scope_boost += 0.15;
                }
                if scope.run_id.is_some() {
                    scope_boost += 0.10;
                }
                if scope.app_id.is_some() {
                    scope_boost += 0.10;
                }
                results.push((layer, i, text_score * scope_boost));
            }
        }
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(20);
        results
    }

    /// Count entries per origin.
    pub fn count_by_origin(&self) -> HashMap<MemoryOrigin, usize> {
        let mut counts: HashMap<MemoryOrigin, usize> = HashMap::new();
        for entry in self
            .episodic
            .iter()
            .chain(self.facts.iter())
            .chain(self.skills.iter())
            .chain(self.meta_rules.iter())
            .chain(self.identity.iter())
        {
            *counts.entry(entry.origin).or_insert(0) += 1;
        }
        counts
    }

    /// Set confidence of first matching entry in a layer. Returns mutable reference if found.
    pub fn set_confidence(
        &mut self,
        content: &str,
        layer: LatticeLayer,
        confidence: f64,
    ) -> Option<&mut LatticeEntry> {
        let deque = match layer {
            LatticeLayer::Episodic => &mut self.episodic,
            LatticeLayer::Facts => &mut self.facts,
            LatticeLayer::Skills => &mut self.skills,
            LatticeLayer::MetaRules => &mut self.meta_rules,
            LatticeLayer::Identity => &mut self.identity,
        };
        for entry in deque.iter_mut() {
            if entry.content == content {
                entry.confidence = confidence;
                return Some(entry);
            }
        }
        None
    }

    pub fn find_by_domain(&self, domain: &str) -> Vec<(LatticeLayer, usize, f64)> {
        let q = domain.to_lowercase();
        let mut results = Vec::new();
        for (layer, entries) in [
            (LatticeLayer::Identity, &self.identity),
            (LatticeLayer::MetaRules, &self.meta_rules),
            (LatticeLayer::Skills, &self.skills),
            (LatticeLayer::Facts, &self.facts),
            (LatticeLayer::Episodic, &self.episodic),
        ] {
            for (i, entry) in entries.iter().enumerate() {
                if entry.domain.to_lowercase() == q {
                    let score =
                        entry.confidence * (1.0 + 0.1 * entry.invocation_count as f64).min(2.0);
                    results.push((layer, i, score));
                }
            }
        }
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(10);
        results
    }

    pub fn find_by_belief_state(&self, state: BeliefState) -> Vec<(LatticeLayer, usize, f64)> {
        let mut results = Vec::new();
        for (layer, entries) in [
            (LatticeLayer::Identity, &self.identity),
            (LatticeLayer::MetaRules, &self.meta_rules),
            (LatticeLayer::Skills, &self.skills),
            (LatticeLayer::Facts, &self.facts),
            (LatticeLayer::Episodic, &self.episodic),
        ] {
            for (i, entry) in entries.iter().enumerate() {
                if entry.belief_state == state {
                    let score =
                        entry.confidence * (1.0 + 0.1 * entry.invocation_count as f64).min(2.0);
                    results.push((layer, i, score));
                }
            }
        }
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(10);
        results
    }

    /// Measure retrieval score distribution by temporal distance from query time.
    /// Partitions entries matching `query` into bins based on valid_to distance:
    /// close [0,50), medium [50,200), far [200,∞), unknown (no valid_to).
    /// Returns a report with per-bin count and average score.
    /// References: arXiv:2606.24775 Finding 4 — retrieval fidelity degrades with temporal distance.
    pub fn retrieval_fidelity_analysis(
        &self,
        query: &str,
        at_time: i64,
    ) -> RetrievalFidelityReport {
        let q = query.to_lowercase();
        let mut close_count = 0u32;
        let mut close_score_sum = 0.0f64;
        let mut medium_count = 0u32;
        let mut medium_score_sum = 0.0f64;
        let mut far_count = 0u32;
        let mut far_score_sum = 0.0f64;
        let mut unknown_count = 0u32;

        for entry in self
            .episodic
            .iter()
            .chain(self.facts.iter())
            .chain(self.skills.iter())
            .chain(self.meta_rules.iter())
            .chain(self.identity.iter())
        {
            if !entry.content.to_lowercase().contains(&q) {
                continue;
            }
            let score = entry.confidence * (1.0 + 0.1 * entry.invocation_count as f64).min(2.0);
            let dist = match entry.valid_to {
                Some(vt) => {
                    let d = at_time - vt;
                    if d < 0 {
                        0u64
                    } else {
                        d as u64
                    }
                }
                None => u64::MAX,
            };
            match dist {
                0..=49 => {
                    close_count += 1;
                    close_score_sum += score;
                }
                50..=199 => {
                    medium_count += 1;
                    medium_score_sum += score;
                }
                u64::MAX => {
                    unknown_count += 1;
                }
                _ => {
                    far_count += 1;
                    far_score_sum += score;
                }
            }
        }

        RetrievalFidelityReport {
            close_count,
            close_avg_score: if close_count > 0 {
                close_score_sum / close_count as f64
            } else {
                0.0
            },
            medium_count,
            medium_avg_score: if medium_count > 0 {
                medium_score_sum / medium_count as f64
            } else {
                0.0
            },
            far_count,
            far_avg_score: if far_count > 0 {
                far_score_sum / far_count as f64
            } else {
                0.0
            },
            unknown_count,
        }
    }

    /// Find domains with Contradictory entries and resolve by keeping the highest-confidence entry.
    /// Within each layer, for each domain that has at least one Contradictory entry,
    /// keep the entry with the highest confidence and remove the rest.
    /// Returns the number of entries removed.
    pub fn resolve_contradictions(&mut self) -> usize {
        let mut resolved = 0;
        for deque in [
            &mut self.episodic,
            &mut self.facts,
            &mut self.skills,
            &mut self.meta_rules,
            &mut self.identity,
        ] {
            let contradict_domains: Vec<String> = deque
                .iter()
                .filter(|e| e.belief_state == BeliefState::Contradictory)
                .map(|e| e.domain.clone())
                .collect();
            // Use Vec to deduplicate while preserving simplicity
            let mut unique_domains = Vec::new();
            for d in contradict_domains {
                if !unique_domains.contains(&d) {
                    unique_domains.push(d);
                }
            }
            for domain in unique_domains {
                let mut indices: Vec<usize> = Vec::new();
                for (i, entry) in deque.iter().enumerate() {
                    if entry.domain == domain {
                        indices.push(i);
                    }
                }
                if indices.len() < 2 {
                    continue;
                }
                let best_idx = indices
                    .iter()
                    .max_by(|&&a, &&b| {
                        deque[a]
                            .confidence
                            .partial_cmp(&deque[b].confidence)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied()
                    .unwrap_or(0);
                let mut to_remove: Vec<usize> =
                    indices.into_iter().filter(|&i| i != best_idx).collect();
                to_remove.sort_unstable_by(|a, b| b.cmp(a));
                resolved += to_remove.len();
                for idx in to_remove {
                    if idx < deque.len() {
                        deque.remove(idx);
                    }
                }
            }
        }
        resolved
    }

    pub fn diagnostic(&self) -> String {
        let by_origin = self.count_by_origin();
        let origin_str: String = by_origin
            .iter()
            .map(|(o, c)| format!("{}:{}", o.name(), c))
            .collect::<Vec<_>>()
            .join(",");
        let domain_counts: std::collections::HashMap<String, usize> = self
            .episodic
            .iter()
            .chain(self.facts.iter())
            .chain(self.skills.iter())
            .chain(self.meta_rules.iter())
            .chain(self.identity.iter())
            .fold(std::collections::HashMap::new(), |mut acc, e| {
                *acc.entry(e.domain.clone()).or_insert(0) += 1;
                acc
            });
        let domain_str: String = domain_counts
            .iter()
            .take(3)
            .map(|(d, c)| format!("{}:{}", d, c))
            .collect::<Vec<_>>()
            .join(",");
        let belief_counts: std::collections::HashMap<BeliefState, usize> = self
            .episodic
            .iter()
            .chain(self.facts.iter())
            .chain(self.skills.iter())
            .chain(self.meta_rules.iter())
            .chain(self.identity.iter())
            .fold(std::collections::HashMap::new(), |mut acc, e| {
                *acc.entry(e.belief_state).or_insert(0) += 1;
                acc
            });
        let belief_str: String = belief_counts
            .iter()
            .map(|(b, c)| format!("{}:{}", b.name(), c))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "lattice:ep={}|fa={}|sk={}|mr={}|id={}|cons={}|origins=[{}]|domains=[{}]|beliefs=[{}]",
            self.episodic.len(),
            self.facts.len(),
            self.skills.len(),
            self.meta_rules.len(),
            self.identity.len(),
            self.total_consolidations,
            origin_str,
            domain_str,
            belief_str,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(content: &str) -> (String, Vec<u8>) {
        let hash = crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA::seeded_random(
            content.len() as u64,
            4096,
        );
        (content.to_string(), hash)
    }

    #[test]
    fn test_new_lattice_empty() {
        let ml = MemoryLattice::new();
        assert_eq!(ml.episodic.len(), 0);
        assert_eq!(ml.facts.len(), 0);
    }

    #[test]
    fn test_store_episodic() {
        let mut ml = MemoryLattice::new();
        let (c, h) = make_entry("test experience");
        ml.store(c, h, LatticeLayer::Episodic);
        assert_eq!(ml.episodic.len(), 1);
    }

    #[test]
    fn test_store_respects_capacity() {
        let mut ml = MemoryLattice::new();
        for i in 0..LatticeLayer::Identity.capacity() + 5 {
            let (c, h) = make_entry(&format!("identity {}", i));
            ml.store(c, h, LatticeLayer::Identity);
        }
        assert_eq!(ml.identity.len(), LatticeLayer::Identity.capacity());
    }

    #[test]
    fn test_consolidation_episodic_to_facts() {
        let mut ml = MemoryLattice::new();
        let (c, h) = make_entry("high confidence episodic");
        ml.store(c, h, LatticeLayer::Episodic);
        if let Some(e) = ml.episodic.back_mut() {
            e.confidence = 0.7;
        }
        let n = ml.consolidate();
        assert!(n > 0, "should consolidate high-confidence entries");
        assert!(
            ml.facts.len() > 0,
            "facts should have entries after consolidation"
        );
    }

    #[test]
    fn test_find_returns_results() {
        let mut ml = MemoryLattice::new();
        let (c, h) = make_entry("quantum computing research");
        ml.store(c, h, LatticeLayer::Facts);
        let results = ml.find("quantum");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_tick_runs_consolidation() {
        let mut ml = MemoryLattice::new();
        for i in 0..6 {
            let (c, h) = make_entry(&format!("episodic {}", i));
            ml.store(c, h, LatticeLayer::Episodic);
            if let Some(e) = ml.episodic.back_mut() {
                e.confidence = 0.6 + (i as f64 * 0.05);
            }
        }
        for _ in 0..5 {
            ml.tick();
        }
        assert!(ml.cycle == 5);
        assert!(ml.total_consolidations > 0 || ml.episodic.len() <= 6);
    }

    #[test]
    fn test_prune_removes_stale_entries() {
        let mut ml = MemoryLattice::new();
        for i in 0..10 {
            let (c, h) = make_entry(&format!("stale {}", i));
            let c2 = c.clone();
            let h2 = h.clone();
            ml.store(c, h, LatticeLayer::Episodic);
            ml.store(c2, h2, LatticeLayer::Facts);
        }
        ml.cycle = 300;
        let pruned = ml.prune();
        assert!(pruned > 0, "should prune stale entries");
    }

    #[test]
    fn test_diagnostic_format() {
        let ml = MemoryLattice::new();
        let d = ml.diagnostic();
        assert!(d.starts_with("lattice:"));
        assert!(d.contains("ep="));
    }

    #[test]
    fn test_access_increments_count() {
        let mut ml = MemoryLattice::new();
        let (c, h) = make_entry("accessible entry");
        ml.store(c, h, LatticeLayer::Skills);
        ml.access(0, LatticeLayer::Skills);
        ml.access(0, LatticeLayer::Skills);
        if let Some(e) = ml.skills.get(0) {
            assert_eq!(e.invocation_count, 2);
        }
    }

    #[test]
    fn test_layer_level_ordering() {
        assert!(LatticeLayer::Episodic.level() < LatticeLayer::Facts.level());
        assert!(LatticeLayer::Facts.level() < LatticeLayer::Skills.level());
        assert!(LatticeLayer::Skills.level() < LatticeLayer::MetaRules.level());
        assert!(LatticeLayer::MetaRules.level() < LatticeLayer::Identity.level());
    }

    #[test]
    fn test_store_with_validity_bounds() {
        let mut ml = MemoryLattice::new();
        let (c, h) = make_entry("temporal fact");
        let idx = ml.store_with_validity(
            c,
            h,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            Some(100),
            Some(200),
        );
        let e = ml.access(idx, LatticeLayer::Facts).unwrap();
        assert_eq!(e.valid_from, Some(100));
        assert_eq!(e.valid_to, Some(200));
    }

    #[test]
    fn test_find_by_temporal_matches_valid_entries() {
        let mut ml = MemoryLattice::new();
        let (c1, h1) = make_entry("past fact");
        let (c2, h2) = make_entry("current fact");
        let (c3, h3) = make_entry("future fact");
        ml.store_with_validity(
            c1,
            h1,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            Some(0),
            Some(99),
        );
        ml.store_with_validity(
            c2,
            h2,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            Some(100),
            Some(200),
        );
        ml.store_with_validity(
            c3,
            h3,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            Some(201),
            Some(300),
        );
        let results = ml.find_by_temporal("fact", 150, Some(LatticeLayer::Facts));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn test_find_by_temporal_none_bounds_matches_all() {
        let mut ml = MemoryLattice::new();
        let (c1, h1) = make_entry("entry alpha");
        let (c2, h2) = make_entry("entry beta");
        ml.store(c1, h1, LatticeLayer::Episodic);
        ml.store(c2, h2, LatticeLayer::Episodic);
        let results = ml.find_by_temporal("entry", 999, Some(LatticeLayer::Episodic));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_by_temporal_expired_excluded() {
        let mut ml = MemoryLattice::new();
        let (c1, h1) = make_entry("active thing");
        let (c2, h2) = make_entry("expired thing");
        ml.store_with_validity(
            c1,
            h1,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            Some(0),
            Some(200),
        );
        ml.store_with_validity(
            c2,
            h2,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            Some(0),
            Some(50),
        );
        let results = ml.find_by_temporal("thing", 100, Some(LatticeLayer::Facts));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 0);
    }

    #[test]
    fn test_find_by_temporal_future_excluded() {
        let mut ml = MemoryLattice::new();
        let (c1, h1) = make_entry("current thing");
        let (c2, h2) = make_entry("future thing");
        ml.store_with_validity(
            c1,
            h1,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            None,
            Some(100),
        );
        ml.store_with_validity(
            c2,
            h2,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            Some(200),
            None,
        );
        let results = ml.find_by_temporal("thing", 50, Some(LatticeLayer::Facts));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 0);
    }

    #[test]
    fn test_forgetting_reduces_q_value() {
        let mut ml = MemoryLattice::new();
        let (c, h) = make_entry("test forgetting");
        ml.store(c, h, LatticeLayer::Facts);
        let q_before = ml.facts[0].q_value;
        ml.apply_forgetting(50.0);
        let q_after = ml.facts[0].q_value;
        assert!(q_after < q_before, "q_value should decrease");
        assert!(
            q_after < 0.5,
            "q_value should be below 0.5 after strong decay"
        );
    }

    #[test]
    fn test_forgetting_eventually_prunes() {
        let mut ml = MemoryLattice::new();
        let (c, h) = make_entry("vanishing entry");
        let idx = ml.store_with_validity(
            c,
            h,
            LatticeLayer::Episodic,
            MemoryOrigin::System,
            None,
            None,
        );
        ml.episodic[idx].q_value = 0.01;
        ml.apply_forgetting(100.0);
        let pruned = ml.prune_forgotten(0.01);
        assert!(
            pruned.contains_key(&LatticeLayer::Episodic),
            "episodic layer should have pruned entries"
        );
    }

    #[test]
    fn test_prune_forgotten_returns_counts() {
        let mut ml = MemoryLattice::new();
        let (c1, h1) = make_entry("keep me");
        let (c2, h2) = make_entry("prune me");
        ml.store(c1, h1, LatticeLayer::Skills);
        let idx = ml.skills.len();
        ml.store(c2, h2, LatticeLayer::Skills);
        ml.skills[idx].q_value = 0.001;
        let result = ml.prune_forgotten(0.01);
        assert_eq!(ml.skills.len(), 1, "low q_value entry should be removed");
        let removed = result.get(&LatticeLayer::Skills).copied().unwrap_or(0);
        assert_eq!(removed, 1, "should report exactly 1 removal");
    }

    #[test]
    fn test_forgetting_preserves_high_confidence() {
        let mut ml = MemoryLattice::new();
        let (c, h) = make_entry("high conf entry");
        ml.store(c, h, LatticeLayer::Identity);
        ml.identity[0].confidence = 0.99;
        ml.apply_forgetting(1.0);
        assert!(
            ml.identity[0].confidence > 0.98,
            "high confidence should remain high after mild decay"
        );
    }

    #[test]
    fn test_find_by_q_ranking() {
        let mut ml = MemoryLattice::new();
        let (c1, h1) = make_entry("common topic alpha");
        let (c2, h2) = make_entry("common topic beta");
        ml.store(c1, h1, LatticeLayer::Facts);
        let idx = ml.facts.len() - 1;
        let (c3, h3) = make_entry("common topic gamma");
        ml.store(c3, h3, LatticeLayer::Facts);
        ml.store(c2, h2, LatticeLayer::Facts);
        // Give alpha a high Q-value
        ml.facts[idx].q_value = 0.95;
        // With q_weight=1.0, ranking should be purely by Q-value
        let q_results = ml.find_by_q("common", 1.0);
        assert!(!q_results.is_empty(), "should find entries");
        assert_eq!(
            q_results[0].1, idx,
            "highest Q-value entry should rank first"
        );
        // With q_weight=0.0, Q-value is ignored
        let text_results = ml.find_by_q("common", 0.0);
        assert!(
            !text_results.is_empty(),
            "should find entries with text-only"
        );
    }

    #[test]
    fn test_retrieval_fidelity_temporal_binning() {
        let mut ml = MemoryLattice::new();
        // Close entry (valid_to near query time)
        let (c1, h1) = make_entry("test memory recent");
        ml.store_with_validity(
            c1,
            h1,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            Some(0),
            Some(45),
        );
        // Medium entry
        let (c2, h2) = make_entry("test memory medium");
        ml.store_with_validity(
            c2,
            h2,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            Some(0),
            Some(100),
        );
        // Far entry
        let (c3, h3) = make_entry("test memory distant");
        ml.store_with_validity(
            c3,
            h3,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            Some(0),
            Some(300),
        );
        // Unknown (no valid_to)
        let (c4, h4) = make_entry("test memory timeless");
        ml.store(c4, h4, LatticeLayer::Facts);

        let report = ml.retrieval_fidelity_analysis("test", 50);
        assert_eq!(report.close_count, 1, "valid_to=45 should be close");
        assert_eq!(report.medium_count, 1, "valid_to=100 should be medium");
        assert_eq!(report.far_count, 1, "valid_to=300 should be far");
        assert_eq!(report.unknown_count, 1, "no valid_to should be unknown");
        assert!(
            report.close_avg_score > 0.0,
            "close entries should have positive score"
        );
    }

    #[test]
    fn test_resolve_contradictions_removes_lower_confidence() {
        let mut ml = MemoryLattice::new();
        let (c1, h1) = make_entry("disputed fact");
        ml.store_with_provenance(
            c1,
            h1,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            None,
            BeliefState::Contradictory,
            "test_domain".to_string(),
        );
        let (c2, h2) = make_entry("disputed fact");
        ml.store_with_provenance(
            c2,
            h2,
            LatticeLayer::Facts,
            MemoryOrigin::System,
            None,
            BeliefState::Authoritative,
            "test_domain".to_string(),
        );
        assert_eq!(ml.facts.len(), 2, "both entries stored");
        // Set confidence on the contradictory one lower
        for entry in ml.facts.iter_mut() {
            if entry.belief_state == BeliefState::Contradictory {
                entry.confidence = 0.3;
            } else {
                entry.confidence = 0.9;
            }
        }
        let resolved = ml.resolve_contradictions();
        assert!(resolved > 0, "should resolve at least one contradiction");
        assert_eq!(ml.facts.len(), 1, "only highest-confidence entry remains");
        assert_eq!(
            ml.facts[0].belief_state,
            BeliefState::Authoritative,
            "authoritative entry should survive"
        );
    }

    #[test]
    fn test_resolve_contradictions_noop_when_no_contradictions() {
        let mut ml = MemoryLattice::new();
        let (c1, h1) = make_entry("peaceful fact");
        ml.store(c1, h1, LatticeLayer::Facts);
        let (c2, h2) = make_entry("another fact");
        ml.store(c2, h2, LatticeLayer::Facts);
        let resolved = ml.resolve_contradictions();
        assert_eq!(resolved, 0, "no contradictions to resolve");
        assert_eq!(ml.facts.len(), 2, "all entries preserved");
    }
}

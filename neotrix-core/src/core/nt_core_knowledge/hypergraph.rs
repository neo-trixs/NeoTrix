use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hyperedge {
    pub id: String,
    pub entities: Vec<String>,
    pub relation_type: NaryRelationType,
    pub weight: f64,
    pub confidence: f64,
    pub context: String,
    pub source_url: String,
    pub created_at: i64,
    /// OKH-RAG temporal ordering: timestamp or sequence number for order-aware traversal.
    /// Paths with non-decreasing temporal_order are preferred during beam search.
    pub temporal_order: Option<u64>,
    /// VSA fingerprint (512-bit = 64 bytes) derived from participant names for similarity search.
    pub vsa_fingerprint: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NaryRelationType {
    Interaction,
    Composition,
    CausalChain,
    TemporalSequence,
    CoOccurrence,
    Hierarchical,
    Comparative,
    Procedural,
    Custom(String),
}

impl NaryRelationType {
    pub fn name(&self) -> &str {
        match self {
            NaryRelationType::Interaction => "interaction",
            NaryRelationType::Composition => "composition",
            NaryRelationType::CausalChain => "causal_chain",
            NaryRelationType::TemporalSequence => "temporal_sequence",
            NaryRelationType::CoOccurrence => "co_occurrence",
            NaryRelationType::Hierarchical => "hierarchical",
            NaryRelationType::Comparative => "comparative",
            NaryRelationType::Procedural => "procedural",
            NaryRelationType::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphStore {
    hyperedges: Vec<Hyperedge>,
    entity_to_hyperedges: HashMap<String, Vec<usize>>,
    max_hyperedges: usize,
}

impl HypergraphStore {
    pub fn new(max_hyperedges: usize) -> Self {
        HypergraphStore {
            hyperedges: Vec::new(),
            entity_to_hyperedges: HashMap::new(),
            max_hyperedges,
        }
    }

    pub fn insert(&mut self, mut edge: Hyperedge) -> bool {
        if self.hyperedges.len() >= self.max_hyperedges {
            return false;
        }
        if edge.vsa_fingerprint.is_none() {
            let seed = edge
                .entities
                .iter()
                .flat_map(|p| p.bytes())
                .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            edge.vsa_fingerprint = Some(QuantizedVSA::seeded_random(seed, 64));
        }
        let idx = self.hyperedges.len();
        for entity in &edge.entities {
            self.entity_to_hyperedges
                .entry(entity.clone())
                .or_default()
                .push(idx);
        }
        self.hyperedges.push(edge);
        true
    }

    pub fn hyperedges_for_entity(&self, entity: &str) -> Vec<&Hyperedge> {
        self.entity_to_hyperedges
            .get(entity)
            .map(|indices| indices.iter().map(|&i| &self.hyperedges[i]).collect())
            .unwrap_or_default()
    }

    pub fn all_hyperedges(&self) -> &[Hyperedge] {
        &self.hyperedges
    }

    pub fn count(&self) -> usize {
        self.hyperedges.len()
    }

    pub fn entities_connected_to(&self, entity: &str, max_depth: usize) -> HashSet<String> {
        let mut visited_entities: HashSet<String> = HashSet::new();
        let mut visited_edges: HashSet<usize> = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut depths: HashMap<String, usize> = HashMap::new();

        queue.push_back(entity.to_string());
        depths.insert(entity.to_string(), 0);
        visited_entities.insert(entity.to_string());

        while let Some(current) = queue.pop_front() {
            let depth = depths[&current];
            if depth >= max_depth {
                continue;
            }
            if let Some(edge_indices) = self.entity_to_hyperedges.get(&current) {
                for &ei in edge_indices {
                    if visited_edges.contains(&ei) {
                        continue;
                    }
                    visited_edges.insert(ei);
                    let edge = &self.hyperedges[ei];
                    for other in &edge.entities {
                        if visited_entities.insert(other.clone()) {
                            depths.insert(other.clone(), depth + 1);
                            queue.push_back(other.clone());
                        }
                    }
                }
            }
        }

        visited_entities.remove(entity);
        visited_entities
    }

    pub fn traverse_to_leaf(
        &self,
        entity: &str,
        relation_type: &NaryRelationType,
    ) -> Vec<Vec<String>> {
        let mut paths: Vec<Vec<String>> = Vec::new();
        let mut current_path: Vec<String> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        self._dfs(
            entity,
            relation_type,
            &mut visited,
            &mut current_path,
            &mut paths,
            0,
            100,
        );
        paths
    }

    fn _dfs(
        &self,
        entity: &str,
        rtype: &NaryRelationType,
        visited: &mut HashSet<String>,
        current_path: &mut Vec<String>,
        paths: &mut Vec<Vec<String>>,
        depth: usize,
        max_depth: usize,
    ) {
        if depth >= max_depth {
            return;
        }
        if !visited.insert(entity.to_string()) {
            return;
        }
        current_path.push(entity.to_string());

        let mut found_next = false;
        if let Some(indices) = self.entity_to_hyperedges.get(entity) {
            for &ei in indices {
                let edge = &self.hyperedges[ei];
                if edge.relation_type != *rtype {
                    continue;
                }
                for other in &edge.entities {
                    if !visited.contains(other) {
                        found_next = true;
                        self._dfs(
                            other,
                            rtype,
                            visited,
                            current_path,
                            paths,
                            depth + 1,
                            max_depth,
                        );
                    }
                }
            }
        }

        if !found_next && current_path.len() > 1 {
            paths.push(current_path.clone());
        }

        current_path.pop();
        visited.remove(entity);
    }
}

pub struct NaryRelationExtractor;

impl NaryRelationExtractor {
    pub fn extract(text: &str) -> Vec<Hyperedge> {
        let mut edges = Vec::new();
        let sentences = text.split(|c| c == '.' || c == '!' || c == '?');

        for sentence in sentences {
            let trimmed = sentence.trim();
            if trimmed.len() < 20 {
                continue;
            }

            if let Some(edge) = Self::extract_composition(trimmed) {
                edges.push(edge);
            }
            if let Some(edge) = Self::extract_causal(trimmed) {
                edges.push(edge);
            }
            if let Some(edge) = Self::extract_interaction(trimmed) {
                edges.push(edge);
            }
            if let Some(edge) = Self::extract_temporal(trimmed) {
                edges.push(edge);
            }
        }

        edges
    }

    pub fn extract_with_context(text: &str, source_url: &str, timestamp: i64) -> Vec<Hyperedge> {
        let mut edges = Self::extract(text);
        for edge in &mut edges {
            if edge.context.is_empty() {
                edge.context = text.chars().take(200).collect();
            }
            if edge.source_url.is_empty() {
                edge.source_url = source_url.to_string();
            }
            if edge.created_at == 0 {
                edge.created_at = timestamp;
            }
        }
        edges
    }

    fn extract_composition(sentence: &str) -> Option<Hyperedge> {
        let indicators = [
            "consists of",
            "composed of",
            "contains",
            "includes",
            "comprises",
            "made up of",
        ];
        for ind in &indicators {
            if let Some(pos) = sentence.find(ind) {
                let subject = sentence[..pos].trim();
                let rest = sentence[pos + ind.len()..].trim();
                let parts: Vec<&str> = rest
                    .split(|c| c == ',' || c == ';' || c == '和')
                    .map(|s| {
                        s.trim()
                            .trim_matches(|c: char| c == ' ' || c == '。' || c == '、')
                    })
                    .filter(|s| !s.is_empty() && s.len() > 1)
                    .collect();
                if !parts.is_empty() {
                    let mut entities = vec![subject.to_string()];
                    for p in parts {
                        let cleaned = p.trim_end_matches('.').trim();
                        if !cleaned.is_empty() {
                            entities.push(cleaned.to_string());
                        }
                    }
                    if entities.len() >= 2 {
                        return Some(Hyperedge {
                            id: format!("comp-{}", fxhash(&entities)),
                            entities,
                            relation_type: NaryRelationType::Composition,
                            weight: 0.8,
                            confidence: 0.6,
                            context: sentence.to_string(),
                            source_url: String::new(),
                            temporal_order: None,
                            vsa_fingerprint: None,
                            created_at: 0,
                        });
                    }
                }
            }
        }
        None
    }

    fn extract_causal(sentence: &str) -> Option<Hyperedge> {
        let indicators = [
            "causes",
            "leads to",
            "results in",
            "triggers",
            "produces",
            "因为",
            "所以",
        ];
        for ind in &indicators {
            if let Some(pos) = sentence.find(ind) {
                let cause = sentence[..pos].trim();
                let effect = sentence[pos + ind.len()..].trim().trim_end_matches('.');

                if cause.len() > 2 && effect.len() > 2 {
                    let mut entities = vec![cause.to_string(), effect.to_string()];

                    if let Some(extra) = Self::find_additional_entity(sentence, pos) {
                        entities.push(extra);
                    }

                    return Some(Hyperedge {
                        id: format!("causal-{}", fxhash(&entities)),
                        entities,
                        relation_type: NaryRelationType::CausalChain,
                        weight: 0.7,
                        confidence: 0.5,
                        context: sentence.to_string(),
                        source_url: String::new(),
                        temporal_order: None,
                        created_at: 0,
                        vsa_fingerprint: None,
                    });
                }
            }
        }
        None
    }

    fn extract_interaction(sentence: &str) -> Option<Hyperedge> {
        let indicators = [
            "interacts with",
            "collaborates with",
            "binds to",
            "reacts with",
            "communicates with",
            "connects to",
            "couples with",
        ];
        for ind in &indicators {
            if let Some(pos) = sentence.find(ind) {
                let subject = sentence[..pos].trim();
                let rest = sentence[pos + ind.len()..].trim().trim_end_matches('.');
                let targets: Vec<&str> = rest
                    .split(|c| c == ',' || c == ';')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty() && s.len() > 1)
                    .collect();
                if !targets.is_empty() {
                    let mut entities = vec![subject.to_string()];
                    for t in targets {
                        entities.push(t.to_string());
                    }
                    return Some(Hyperedge {
                        id: format!("interact-{}", fxhash(&entities)),
                        entities,
                        relation_type: NaryRelationType::Interaction,
                        weight: 0.75,
                        confidence: 0.55,
                        context: sentence.to_string(),
                        source_url: String::new(),
                        temporal_order: None,
                        created_at: 0,
                        vsa_fingerprint: None,
                    });
                }
            }
        }
        None
    }

    fn extract_temporal(sentence: &str) -> Option<Hyperedge> {
        let indicators = [
            "then",
            "after",
            "before",
            "following",
            "subsequently",
            "first",
            "finally",
            "首先",
            "然后",
            "最后",
        ];
        let mut parts: Vec<&str> = Vec::new();

        let mut last_end = 0;
        for ind in &indicators {
            if let Some(pos) = sentence.find(ind) {
                if pos > last_end {
                    let segment = sentence[last_end..pos].trim();
                    if !segment.is_empty() && segment.len() > 3 {
                        parts.push(segment);
                    }
                }
                last_end = pos + ind.len();
            }
        }
        let remaining = sentence[last_end..].trim().trim_end_matches('.');
        if !remaining.is_empty() && remaining.len() > 3 {
            parts.push(remaining);
        }

        if parts.len() >= 3 {
            let entities: Vec<String> = parts.into_iter().map(|s| s.to_string()).collect();
            return Some(Hyperedge {
                id: format!("temporal-{}", fxhash(&entities)),
                entities,
                relation_type: NaryRelationType::TemporalSequence,
                weight: 0.6,
                confidence: 0.4,
                context: sentence.to_string(),
                source_url: String::new(),
                temporal_order: None,
                created_at: 0,
                vsa_fingerprint: None,
            });
        }
        None
    }

    fn find_additional_entity(sentence: &str, pos: usize) -> Option<String> {
        let before = &sentence[..pos];
        if let Some(via_pos) = before.rfind("via") {
            let via_entity = before[via_pos + 3..].trim();
            if via_entity.len() > 2 {
                return Some(via_entity.to_string());
            }
        }
        if let Some(through_pos) = before.rfind("through") {
            let through_entity = before[through_pos + 7..].trim();
            if through_entity.len() > 2 {
                return Some(through_entity.to_string());
            }
        }
        if let Some(by_pos) = before.rfind("by") {
            let by_entity = before[by_pos + 2..].trim();
            if by_entity.len() > 2 {
                return Some(by_entity.to_string());
            }
        }
        None
    }
}

fn fxhash(strings: &[String]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for s in strings {
        s.hash(&mut hasher);
    }
    hasher.finish()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperedgeTraversal {
    visited: HashSet<String>,
    path: Vec<String>,
    paths: Vec<Vec<String>>,
}

impl HyperedgeTraversal {
    pub fn new() -> Self {
        HyperedgeTraversal {
            visited: HashSet::new(),
            path: Vec::new(),
            paths: Vec::new(),
        }
    }

    pub fn beam_search(
        &mut self,
        store: &HypergraphStore,
        start: &str,
        target: &str,
        beam_width: usize,
    ) -> Vec<Vec<String>> {
        let mut candidates: Vec<(Vec<String>, f64)> = vec![(vec![start.to_string()], 1.0)];

        for _ in 0..10 {
            let mut new_candidates: Vec<(Vec<String>, f64)> = Vec::new();
            for (path, score) in &candidates {
                let tail = path.last().expect("BFS path always has start_node");
                let edges = store.hyperedges_for_entity(tail);
                for edge in edges {
                    for entity in &edge.entities {
                        if entity == tail || path.contains(entity) {
                            continue;
                        }
                        let mut new_path = path.clone();
                        new_path.push(entity.clone());
                        let new_score = score * edge.weight * edge.confidence;
                        new_candidates.push((new_path, new_score));
                    }
                }
            }

            candidates = new_candidates;
            candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            candidates.truncate(beam_width);

            for (path, _) in &candidates {
                if path.last().map(|s| s.as_str()) == Some(target) {
                    self.paths.push(path.clone());
                }
            }
        }

        self.paths.sort_by(|a, b| a.len().cmp(&b.len()));
        self.paths.clone()
    }

    pub fn shortest_path(
        &self,
        store: &HypergraphStore,
        start: &str,
        target: &str,
    ) -> Option<Vec<String>> {
        let mut queue: VecDeque<Vec<String>> = VecDeque::new();
        let mut visited_local: HashSet<String> = HashSet::new();
        queue.push_back(vec![start.to_string()]);
        visited_local.insert(start.to_string());

        while let Some(path) = queue.pop_front() {
            let tail = path.last().expect("BFS path always has start_node");
            if tail == target {
                return Some(path);
            }
            let edges = store.hyperedges_for_entity(tail);
            for edge in edges {
                for entity in &edge.entities {
                    if visited_local.insert(entity.clone()) {
                        let mut new_path = path.clone();
                        new_path.push(entity.clone());
                        queue.push_back(new_path);
                    }
                }
            }
        }

        None
    }

    /// Beam search with memory guidance and temporal ordering (HyperRAG-style).
    /// Extends the standard beam search with:
    /// - Memory-guided scoring from LLM parametric priors
    /// - Temporal consistency penalty for order-aware traversal
    pub fn memory_guided_beam_search(
        &mut self,
        store: &HypergraphStore,
        start: &str,
        target: &str,
        config: &BeamSearchConfig,
    ) -> Vec<Vec<String>> {
        let beam_width = config.beam_width;
        let max_depth = config.max_depth;

        // (path, edge_ids, temporal_orders, score)
        let mut candidates: Vec<(Vec<String>, Vec<u64>, Vec<Option<u64>>, f64)> =
            vec![(vec![start.to_string()], vec![], vec![], 1.0)];

        for _depth in 0..max_depth {
            let mut new_candidates: Vec<(Vec<String>, Vec<u64>, Vec<Option<u64>>, f64)> =
                Vec::new();
            for (path, edge_ids, temporal_orders, score) in &candidates {
                let tail = path.last().expect("beam_search path always has start_node");
                let edges = store.hyperedges_for_entity(tail);
                for edge in edges {
                    for entity in &edge.entities {
                        if entity == tail || path.contains(entity) {
                            continue;
                        }
                        let mut new_path = path.clone();
                        new_path.push(entity.clone());

                        let numeric_id = hyperedge_numeric_id(&edge.id);
                        let mut new_edge_ids = edge_ids.clone();
                        new_edge_ids.push(numeric_id);

                        let mut new_temporal_orders = temporal_orders.clone();
                        new_temporal_orders.push(edge.temporal_order);

                        let mut new_score = score * edge.weight * edge.confidence;

                        // Temporal consistency penalty
                        if config.temporal_penalty > 0.0 && new_temporal_orders.len() >= 2 {
                            let len = new_temporal_orders.len();
                            match (new_temporal_orders[len - 2], new_temporal_orders[len - 1]) {
                                (Some(t1), Some(t2)) if t2 < t1 => {
                                    new_score *= 1.0 - config.temporal_penalty;
                                }
                                _ => {}
                            }
                        }

                        // Memory-guided scoring: blend structural score with LLM prior
                        if let Some(ref memory) = config.memory_guidance {
                            new_score = memory.score_with_memory(&new_edge_ids, new_score);
                        }

                        new_candidates.push((
                            new_path,
                            new_edge_ids,
                            new_temporal_orders,
                            new_score,
                        ));
                    }
                }
            }

            candidates = new_candidates;
            candidates.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
            candidates.truncate(beam_width);

            for (path, _, _, _) in &candidates {
                if path.last().map(|s| s.as_str()) == Some(target) {
                    self.paths.push(path.clone());
                }
            }
        }

        self.paths.sort_by(|a, b| a.len().cmp(&b.len()));
        self.paths.clone()
    }
}

/// Deterministic hash of a hyperedge string ID to a numeric key for memory lookup.
pub fn hyperedge_numeric_id(id: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
}

/// Configuration for [`HyperedgeTraversal::memory_guided_beam_search`].
///
/// Wraps beam search parameters with optional memory guidance (HyperRAG-style)
/// and temporal ordering (OKH-RAG-style).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamSearchConfig {
    pub beam_width: usize,
    pub max_depth: usize,
    pub memory_guidance: Option<MemoryGuidedBeamSearch>,
    pub temporal_penalty: f64,
}

impl BeamSearchConfig {
    pub fn new(beam_width: usize, max_depth: usize) -> Self {
        BeamSearchConfig {
            beam_width,
            max_depth,
            memory_guidance: None,
            temporal_penalty: 0.0,
        }
    }

    /// Attach memory guidance to score paths using LLM parametric priors.
    pub fn with_memory_guidance(mut self, memory: MemoryGuidedBeamSearch) -> Self {
        self.memory_guidance = Some(memory);
        self
    }

    /// Apply a penalty (0.0–1.0) to paths that violate temporal ordering.
    pub fn with_temporal_penalty(mut self, penalty: f64) -> Self {
        self.temporal_penalty = penalty.clamp(0.0, 1.0);
        self
    }
}

/// Memory-guided beam search inspired by HyperRAG / HyperMemory (arXiv:2602.14470).
///
/// Blends structural graph scores with LLM parametric memory priors so that
/// traversals prefer hyperedges that the LLM has previously judged relevant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGuidedBeamSearch {
    pub llm_weight: f64,
    pub memory_hits: HashMap<u64, f64>,
    pub structural_weight: f64,
    pub max_entries: usize,
    access_order: VecDeque<u64>,
}

impl MemoryGuidedBeamSearch {
    pub fn new(llm_weight: f64, structural_weight: f64) -> Self {
        MemoryGuidedBeamSearch {
            llm_weight: llm_weight.clamp(0.0, 1.0),
            memory_hits: HashMap::new(),
            structural_weight,
            max_entries: 1000,
            access_order: VecDeque::new(),
        }
    }

    /// Score a path by combining per-edge memory priors with the structural score.
    ///
    /// `final = llm_weight * avg_memory + (1 - llm_weight) * structural_score`
    ///
    /// Falls back to structural-only when no memory entries match any edge.
    pub fn score_with_memory(&self, edge_ids: &[u64], structural_score: f64) -> f64 {
        if edge_ids.is_empty() {
            return structural_score;
        }
        let mut total_memory = 0.0f64;
        let mut hit_count = 0usize;
        for &eid in edge_ids {
            if let Some(&prior) = self.memory_hits.get(&eid) {
                total_memory += prior;
                hit_count += 1;
            }
        }
        if hit_count == 0 {
            return structural_score;
        }
        let avg_memory = total_memory / hit_count as f64;
        self.llm_weight * avg_memory + (1.0 - self.llm_weight) * structural_score
    }

    /// Set the maximum number of memory entries (LRU eviction threshold).
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Update memory with a feedback signal using exponential moving average.
    ///
    /// `memory_hits[e] = memory_hits[e] * 0.7 + feedback * 0.3`
    ///
    /// Evicts oldest entries when capacity exceeds `max_entries` (LRU).
    pub fn update_memory(&mut self, edge_ids: &[u64], feedback: f64) {
        let feedback = feedback.clamp(0.0, 1.0);
        for &eid in edge_ids {
            let current = self.memory_hits.get(&eid).copied().unwrap_or(0.5);
            let updated = current * 0.7 + feedback * 0.3;
            if !self.memory_hits.contains_key(&eid) {
                self.access_order.push_back(eid);
            }
            self.memory_hits.insert(eid, updated);
        }
        // LRU eviction: remove oldest entries when over capacity
        while self.memory_hits.len() > self.max_entries {
            if let Some(oldest) = self.access_order.pop_front() {
                self.memory_hits.remove(&oldest);
            } else {
                break;
            }
        }
    }
}

pub fn hyperedge_to_vsa_text(edge: &Hyperedge) -> String {
    let entities = edge.entities.join(", ");
    format!(
        "[{}] {}: {} (confidence: {:.2})",
        edge.relation_type.name(),
        entities,
        edge.context.chars().take(100).collect::<String>(),
        edge.confidence
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_retrieve() {
        let mut store = HypergraphStore::new(100);
        let edge = Hyperedge {
            id: "e1".into(),
            entities: vec!["A".into(), "B".into(), "C".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 1.0,
            confidence: 0.9,
            context: "test".into(),
            source_url: String::new(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        };
        assert!(store.insert(edge));
        assert_eq!(store.count(), 1);
        let edges = store.hyperedges_for_entity("A");
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn test_entities_connected_to() {
        let mut store = HypergraphStore::new(100);
        store.insert(Hyperedge {
            id: "e1".into(),
            entities: vec!["A".into(), "B".into(), "C".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 1.0,
            confidence: 1.0,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        store.insert(Hyperedge {
            id: "e2".into(),
            entities: vec!["C".into(), "D".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 1.0,
            confidence: 1.0,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        let connected = store.entities_connected_to("A", 2);
        assert!(connected.contains("B"));
        assert!(connected.contains("C"));
        assert!(connected.contains("D"));
        assert!(!connected.contains("A"));
    }

    #[test]
    fn test_shortest_path() {
        let mut store = HypergraphStore::new(100);
        store.insert(Hyperedge {
            id: "e1".into(),
            entities: vec!["A".into(), "B".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 1.0,
            confidence: 1.0,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        store.insert(Hyperedge {
            id: "e2".into(),
            entities: vec!["B".into(), "C".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 1.0,
            confidence: 1.0,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        let trav = HyperedgeTraversal::new();
        let path = trav.shortest_path(&store, "A", "C");
        assert!(path.is_some());
        assert_eq!(path.unwrap(), vec!["A", "B", "C"]);
    }

    #[test]
    fn test_extract_composition() {
        let text = "The solar system consists of Sun, Mercury, Venus, Earth, Mars.";
        let edges = NaryRelationExtractor::extract(text);
        assert!(!edges.is_empty());
        let comp = edges
            .iter()
            .find(|e| e.relation_type == NaryRelationType::Composition);
        assert!(comp.is_some());
        let e = comp.unwrap();
        assert_eq!(e.entities[0], "The solar system");
        assert!(e.entities.len() >= 3);
    }

    #[test]
    fn test_extract_causal() {
        let text = "Protein A causes activation of pathway B via receptor C.";
        let edges = NaryRelationExtractor::extract(text);
        let causal = edges
            .iter()
            .find(|e| e.relation_type == NaryRelationType::CausalChain);
        assert!(causal.is_some());
        let e = causal.unwrap();
        assert!(e.entities.len() >= 2);
    }

    #[test]
    fn test_beam_search() {
        let mut store = HypergraphStore::new(100);
        store.insert(Hyperedge {
            id: "e1".into(),
            entities: vec!["A".into(), "X".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 0.8,
            confidence: 0.9,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        store.insert(Hyperedge {
            id: "e2".into(),
            entities: vec!["X".into(), "Y".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 0.9,
            confidence: 0.8,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        store.insert(Hyperedge {
            id: "e3".into(),
            entities: vec!["Y".into(), "Z".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 0.7,
            confidence: 0.7,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        let mut trav = HyperedgeTraversal::new();
        let paths = trav.beam_search(&store, "A", "Z", 3);
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_extract_temporal() {
        let text = "First enzyme A binds to substrate B, then it activates pathway C, finally product D is released.";
        let edges = NaryRelationExtractor::extract(text);
        let temporal = edges
            .iter()
            .find(|e| e.relation_type == NaryRelationType::TemporalSequence);
        assert!(temporal.is_some());
    }

    #[test]
    fn test_max_capacity() {
        let mut store = HypergraphStore::new(2);
        for i in 0..5 {
            let ok = store.insert(Hyperedge {
                id: format!("e{}", i),
                entities: vec![format!("E{}", i)],
                relation_type: NaryRelationType::Interaction,
                weight: 1.0,
                confidence: 1.0,
                context: "".into(),
                source_url: "".into(),
                temporal_order: None,
                vsa_fingerprint: None,
                created_at: 0,
            });
            if i < 2 {
                assert!(ok);
            } else {
                assert!(!ok);
            }
        }
        assert_eq!(store.count(), 2);
    }

    #[test]
    fn test_dfs_traverse() {
        let mut store = HypergraphStore::new(100);
        store.insert(Hyperedge {
            id: "e1".into(),
            entities: vec!["Root".into(), "Child1".into(), "Child2".into()],
            relation_type: NaryRelationType::Hierarchical,
            weight: 1.0,
            confidence: 1.0,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        store.insert(Hyperedge {
            id: "e2".into(),
            entities: vec!["Child1".into(), "Grandchild".into()],
            relation_type: NaryRelationType::Hierarchical,
            weight: 1.0,
            confidence: 1.0,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        let paths = store.traverse_to_leaf("Root", &NaryRelationType::Hierarchical);
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_vsa_text_roundtrip() {
        let edge = Hyperedge {
            id: "t1".into(),
            entities: vec!["X".into(), "Y".into(), "Z".into()],
            relation_type: NaryRelationType::CausalChain,
            weight: 0.7,
            confidence: 0.5,
            context: "X causes Y through Z".into(),
            source_url: "http://example.com".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 1000,
        };
        let text = hyperedge_to_vsa_text(&edge);
        assert!(text.contains("causal_chain"));
        assert!(text.contains("X, Y, Z"));
        assert!(text.contains("0.50"));
    }

    #[test]
    fn test_custom_relation_type() {
        let rtype = NaryRelationType::Custom("synergistic".into());
        assert_eq!(rtype.name(), "synergistic");
    }

    #[test]
    fn test_extract_with_context_fills_source() {
        let edges = NaryRelationExtractor::extract_with_context(
            "The complex contains protein A, protein B, protein C.",
            "https://example.com",
            1000,
        );
        assert!(!edges.is_empty());
        for e in &edges {
            assert!(!e.source_url.is_empty());
            assert!(e.created_at > 0);
        }
    }

    #[test]
    fn test_memory_guidance_changes_ranking() {
        let mut store = HypergraphStore::new(100);
        store.insert(Hyperedge {
            id: "e1".into(),
            entities: vec!["A".into(), "X".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 0.5,
            confidence: 0.5,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        store.insert(Hyperedge {
            id: "e2".into(),
            entities: vec!["X".into(), "Y".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 0.5,
            confidence: 0.5,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });
        store.insert(Hyperedge {
            id: "e3".into(),
            entities: vec!["A".into(), "Z".into()],
            relation_type: NaryRelationType::Interaction,
            weight: 0.9,
            confidence: 0.9,
            context: "".into(),
            source_url: "".into(),
            temporal_order: None,
            vsa_fingerprint: None,
            created_at: 0,
        });

        // Without memory: path A→Z wins (higher weight*confidence = 0.81 vs 0.25*0.25*2=0.0625)
        let mut trav = HyperedgeTraversal::new();
        let config = BeamSearchConfig::new(5, 10);
        let results = trav.memory_guided_beam_search(&store, "A", "Y", &config);
        assert!(
            results.is_empty(),
            "A→Y should be unreachable without memory at these weights"
        );

        // With memory boosting e1+e2: A→X→Y should become reachable
        let mut memory = MemoryGuidedBeamSearch::new(0.8, 1.0);
        let e1_id = hyperedge_numeric_id("e1");
        let e2_id = hyperedge_numeric_id("e2");
        memory.update_memory(&[e1_id, e2_id], 0.95);
        let config2 = BeamSearchConfig::new(5, 10).with_memory_guidance(memory);
        let mut trav2 = HyperedgeTraversal::new();
        let results2 = trav2.memory_guided_beam_search(&store, "A", "Y", &config2);
        assert!(
            !results2.is_empty(),
            "Memory should boost A→X→Y path into beam"
        );
    }

    #[test]
    fn test_memory_update_ema() {
        let mut memory = MemoryGuidedBeamSearch::new(0.5, 1.0);
        let eid = hyperedge_numeric_id("test-edge");

        // Initial: no prior, update_memory uses default 0.5 * 0.7 + 0.9 * 0.3 = 0.62
        memory.update_memory(&[eid], 0.9);
        let score = memory.memory_hits.get(&eid).copied().unwrap_or(0.0);
        let expected = 0.5 * 0.7 + 0.9 * 0.3;
        assert!(
            (score - expected).abs() < 1e-10,
            "EMA round 1: got {} expected {}",
            score,
            expected
        );

        // Second update: 0.62 * 0.7 + 0.3 * 0.3 = 0.434 + 0.09 = 0.524
        memory.update_memory(&[eid], 0.3);
        let score2 = memory.memory_hits.get(&eid).copied().unwrap_or(0.0);
        let expected2 = expected * 0.7 + 0.3 * 0.3;
        assert!(
            (score2 - expected2).abs() < 1e-10,
            "EMA round 2: got {} expected {}",
            score2,
            expected2
        );
    }

    #[test]
    fn test_temporal_ordering_penalizes_backwards_paths() {
        let mut store = HypergraphStore::new(100);
        store.insert(Hyperedge {
            id: "t1".into(),
            entities: vec!["A".into(), "B".into()],
            relation_type: NaryRelationType::TemporalSequence,
            weight: 0.9,
            confidence: 0.9,
            context: "".into(),
            source_url: "".into(),
            temporal_order: Some(100),
            vsa_fingerprint: None,
            created_at: 0,
        });
        store.insert(Hyperedge {
            id: "t2".into(),
            entities: vec!["B".into(), "C".into()],
            relation_type: NaryRelationType::TemporalSequence,
            weight: 0.9,
            confidence: 0.9,
            context: "".into(),
            source_url: "".into(),
            vsa_fingerprint: None,
            temporal_order: Some(200),
            created_at: 0,
        });
        store.insert(Hyperedge {
            id: "t3".into(),
            entities: vec!["A".into(), "D".into()],
            relation_type: NaryRelationType::TemporalSequence,
            weight: 0.9,
            confidence: 0.9,
            context: "".into(),
            source_url: "".into(),
            temporal_order: Some(300),
            vsa_fingerprint: None,
            created_at: 0,
        });
        store.insert(Hyperedge {
            id: "t4".into(),
            entities: vec!["D".into(), "C".into()],
            relation_type: NaryRelationType::TemporalSequence,
            weight: 0.9,
            confidence: 0.9,
            context: "".into(),
            source_url: "".into(),
            temporal_order: Some(50),
            vsa_fingerprint: None,
            created_at: 0,
        });

        // Without temporal penalty: both A→B→C (100→200) and A→D→C (300→50) compete
        let mut trav = HyperedgeTraversal::new();
        let config = BeamSearchConfig::new(5, 10);
        let results = trav.memory_guided_beam_search(&store, "A", "C", &config);
        assert_eq!(
            results.len(),
            2,
            "Both paths should be found without temporal penalty"
        );

        // With temporal penalty: A→B→C should be preferred over A→D→C
        let mut trav2 = HyperedgeTraversal::new();
        let config2 = BeamSearchConfig::new(2, 10).with_temporal_penalty(0.5);
        let results2 = trav2.memory_guided_beam_search(&store, "A", "C", &config2);
        assert!(!results2.is_empty(), "Should find at least one path");
        // A→B→C has temporal_orders [100, 200] which is consistent (no penalty)
        // A→D→C has temporal_orders [300, 50] which is inconsistent (50% penalty)
        // So A→B→C should rank first
        let first_path = &results2[0];
        assert_eq!(
            first_path,
            &vec!["A", "B", "C"],
            "Temporally consistent path A→B→C should rank first"
        );
    }

    #[test]
    fn test_lru_eviction() {
        let mut memory = MemoryGuidedBeamSearch::new(0.5, 1.0);
        // Evict by setting max_entries low via non-public mechanism — use overflow
        // We test by inserting beyond the (now-reduced) effective capacity
        // Use an internal method: reconstruct with small capacity
        // Instead: fill with 1002 entries to trigger eviction from default 1000
        for i in 0..1002u64 {
            let fake_id = hyperedge_numeric_id(&format!("edge-{}", i));
            memory.update_memory(&[fake_id], 0.5);
        }
        assert!(
            memory.memory_hits.len() <= 1000,
            "Should evict to at most 1000 entries, got {}",
            memory.memory_hits.len()
        );
        // The first entries (edge-0, edge-1) should be evicted
        let evicted_id = hyperedge_numeric_id("edge-0");
        assert!(
            !memory.memory_hits.contains_key(&evicted_id),
            "Oldest entry (edge-0) should be evicted"
        );
    }

    #[test]
    fn test_score_with_memory_fallback() {
        let memory = MemoryGuidedBeamSearch::new(0.8, 1.0);
        // No memory entries exist
        let edge_ids = vec![hyperedge_numeric_id("unknown")];
        let score = memory.score_with_memory(&edge_ids, 0.5);
        assert!(
            (score - 0.5).abs() < 1e-10,
            "Should fall back to structural score, got {}",
            score
        );
    }

    #[test]
    fn test_score_with_memory_blend() {
        let mut memory = MemoryGuidedBeamSearch::new(0.5, 1.0);
        let eid = hyperedge_numeric_id("known-edge");
        memory.update_memory(&[eid], 0.9);
        // memory_hits[eid] = 0.5*0.7 + 0.9*0.3 = 0.62
        // score = 0.5 * 0.62 + 0.5 * 0.5 = 0.31 + 0.25 = 0.56
        let score = memory.score_with_memory(&[eid], 0.5);
        let expected = 0.5 * 0.62 + 0.5 * 0.5;
        assert!(
            (score - expected).abs() < 1e-10,
            "Memory blend: got {} expected {}",
            score,
            expected
        );
    }

    #[test]
    fn test_empty_path_memory_score() {
        let memory = MemoryGuidedBeamSearch::new(0.5, 1.0);
        let score = memory.score_with_memory(&[], 0.75);
        assert!(
            (score - 0.75).abs() < 1e-10,
            "Empty path should return structural score, got {}",
            score
        );
    }
}

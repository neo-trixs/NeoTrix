use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub struct RetrievalWeights {
    pub recency: f32,
    pub relevance: f32,
    pub importance: f32,
    pub lexical: f32,
}

impl Default for RetrievalWeights {
    fn default() -> Self {
        Self {
            recency: 0.5,
            relevance: 3.0,
            importance: 2.0,
            lexical: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryKind {
    Observation,
    Reflection,
    Plan,
    Social,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNode {
    pub id: u64,
    pub kind: MemoryKind,
    pub agent_id: String,
    pub created_tick: u64,
    pub last_accessed_tick: u64,
    pub description: String,
    pub importance: f32,
    pub keywords: Vec<String>,
    pub citations: Vec<u64>,
    pub embedding: Option<[f32; 16]>,
}

pub struct MemoryStream {
    nodes: Vec<MemoryNode>,
    next_id: u64,
    max_nodes: usize,
    recency_weight: f32,
    relevance_weight: f32,
    importance_weight: f32,
    recency_decay: f32,
}

impl MemoryStream {
    pub fn new(max_nodes: usize) -> Self {
        Self {
            nodes: Vec::new(),
            next_id: 0,
            max_nodes,
            recency_weight: 0.5,
            relevance_weight: 3.0,
            importance_weight: 2.0,
            recency_decay: 0.995,
        }
    }

    pub fn add(&mut self, mut node: MemoryNode) -> u64 {
        let id = self.next_id;
        node.id = id;
        self.next_id += 1;
        self.nodes.push(node);
        self.evict();
        id
    }

    pub fn retrieve(
        &mut self,
        query_embedding: &[f32; 16],
        current_tick: u64,
        top_k: usize,
    ) -> Vec<&MemoryNode> {
        let mut scored: Vec<(usize, f32)> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (i, self.score(node, query_embedding, current_tick)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        let indices: Vec<usize> = scored.into_iter().map(|(i, _)| i).collect();

        for &i in &indices {
            self.nodes[i].last_accessed_tick = current_tick;
        }

        indices.iter().map(|&i| &self.nodes[i]).collect()
    }

    fn score(&self, node: &MemoryNode, query_embedding: &[f32; 16], current_tick: u64) -> f32 {
        let age = current_tick.saturating_sub(node.last_accessed_tick);
        let recency = self.recency_decay.powi(age as i32);

        let relevance = match &node.embedding {
            Some(emb) => cosine_sim(emb, query_embedding),
            None => 0.0,
        };

        let importance = node.importance / 10.0;

        self.recency_weight * recency
            + self.relevance_weight * relevance
            + self.importance_weight * importance
    }

    pub fn recent(&self, n: usize) -> &[MemoryNode] {
        let start = self.nodes.len().saturating_sub(n);
        &self.nodes[start..]
    }

    pub fn recent_importance_sum(&self, n: usize) -> f32 {
        self.recent(n).iter().map(|node| node.importance).sum()
    }

    pub fn by_kind(&self, kind: &MemoryKind) -> Vec<&MemoryNode> {
        self.nodes.iter().filter(|n| &n.kind == kind).collect()
    }

    fn evict(&mut self) {
        if self.nodes.len() <= self.max_nodes {
            return;
        }

        let query_embedding = [0.0f32; 16];
        let current_tick = self
            .nodes
            .iter()
            .map(|n| n.last_accessed_tick)
            .max()
            .unwrap_or(0);

        let mut scored: Vec<(usize, f32)> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (i, self.score(node, &query_embedding, current_tick)))
            .collect();

        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let to_remove = self.nodes.len() - self.max_nodes;
        let remove_indices: std::collections::HashSet<usize> =
            scored.into_iter().take(to_remove).map(|(i, _)| i).collect();

        let mut idx = 0;
        self.nodes.retain(|_| {
            let keep = !remove_indices.contains(&idx);
            idx += 1;
            keep
        });
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn get(&self, id: u64) -> Option<&MemoryNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn retrieve_by_text(
        &mut self,
        query: &str,
        current_tick: u64,
        top_k: usize,
    ) -> Vec<&MemoryNode> {
        let weights = RetrievalWeights::default();
        let query_keywords = tokenize(query);
        let mut scored: Vec<(usize, f32)> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let s = self.dual_score(node, None, Some(&query_keywords), current_tick, &weights);
                (i, s)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        let indices: Vec<usize> = scored.into_iter().map(|(i, _)| i).collect();
        for &i in &indices {
            self.nodes[i].last_accessed_tick = current_tick;
        }
        indices.iter().map(|&i| &self.nodes[i]).collect()
    }

    pub fn retrieve_with_weights(
        &mut self,
        query_embedding: Option<&[f32; 16]>,
        query_text: Option<&str>,
        current_tick: u64,
        weights: &RetrievalWeights,
        top_k: usize,
    ) -> Vec<&MemoryNode> {
        let query_keywords = query_text.map(tokenize);
        let query_keywords_ref = query_keywords.as_deref();

        let mut scored: Vec<(usize, f32)> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let s = self.dual_score(node, query_embedding, query_keywords_ref, current_tick, weights);
                (i, s)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        let indices: Vec<usize> = scored.into_iter().map(|(i, _)| i).collect();
        for &i in &indices {
            self.nodes[i].last_accessed_tick = current_tick;
        }
        indices.iter().map(|&i| &self.nodes[i]).collect()
    }

    fn dual_score(
        &self,
        node: &MemoryNode,
        query_embedding: Option<&[f32; 16]>,
        query_keywords: Option<&[String]>,
        current_tick: u64,
        weights: &RetrievalWeights,
    ) -> f32 {
        let age = current_tick.saturating_sub(node.last_accessed_tick);
        let recency_score = self.recency_decay.powi(age as i32);

        let relevance_score = match (query_embedding, &node.embedding) {
            (Some(qe), Some(ne)) => cosine_sim(ne, qe),
            _ => 0.0,
        };

        let importance_score = node.importance / 10.0;

        let lexical_score = match query_keywords {
            Some(qk) => keyword_score(&node.keywords, qk),
            None => 0.0,
        };

        weights.recency * recency_score
            + weights.relevance * relevance_score
            + weights.importance * importance_score
            + weights.lexical * lexical_score
    }
}

fn keyword_score(memory_keywords: &[String], query_keywords: &[String]) -> f32 {
    if query_keywords.is_empty() {
        return 0.0;
    }
    let mem_set: HashSet<&str> = memory_keywords.iter().map(|s| s.as_str()).collect();
    let query_set: HashSet<&str> = query_keywords.iter().map(|s| s.as_str()).collect();
    let overlap = mem_set.intersection(&query_set).count();
    let total = mem_set.union(&query_set).count();
    if total == 0 {
        0.0
    } else {
        overlap as f32 / total as f32
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_string())
        .collect()
}

fn cosine_sim(a: &[f32; 16], b: &[f32; 16]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(kind: MemoryKind, importance: f32, tick: u64, embedding: Option<[f32; 16]>) -> MemoryNode {
        MemoryNode {
            id: 0,
            kind,
            agent_id: "agent_0".into(),
            created_tick: tick,
            last_accessed_tick: tick,
            description: "test".into(),
            importance,
            keywords: vec![],
            citations: vec![],
            embedding,
        }
    }

    #[test]
    fn add_and_retrieve() {
        let mut stream = MemoryStream::new(100);
        let node = make_node(MemoryKind::Observation, 5.0, 0, None);
        let id = stream.add(node);
        assert_eq!(id, 0);
        assert_eq!(stream.len(), 1);
    }

    #[test]
    fn importance_scales_score() {
        let emb = [1.0f32; 16];
        let mut stream = MemoryStream::new(100);
        stream.add(make_node(MemoryKind::Observation, 2.0, 0, Some(emb)));
        stream.add(make_node(MemoryKind::Observation, 9.0, 0, Some(emb)));

        let results = stream.retrieve(&emb, 0, 10);
        assert!(results.len() >= 2);
        assert!(results[0].importance > results[1].importance);
    }

    #[test]
    fn recency_boosts_score() {
        let emb = [1.0f32; 16];
        let mut stream = MemoryStream::new(100);
        stream.add(make_node(MemoryKind::Observation, 5.0, 0, Some(emb)));
        stream.add(make_node(MemoryKind::Observation, 5.0, 100, Some(emb)));

        let results = stream.retrieve(&emb, 200, 10);
        assert!(results.len() >= 2);
        assert!(results[0].created_tick > results[1].created_tick);
    }

    #[test]
    fn relevance_boosts_score() {
        let emb_a = [1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let emb_b = [0.0f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut stream = MemoryStream::new(100);
        stream.add(make_node(MemoryKind::Observation, 5.0, 0, Some(emb_a)));
        stream.add(make_node(MemoryKind::Observation, 5.0, 0, Some(emb_b)));

        let results = stream.retrieve(&emb_a, 0, 10);
        assert!(results.len() >= 2);
        assert!(results[0].embedding.unwrap()[0] == 1.0);
    }

    #[test]
    fn eviction_when_over_capacity() {
        let mut stream = MemoryStream::new(5);
        for i in 0..10 {
            let emb = [i as f32; 16];
            stream.add(make_node(MemoryKind::Observation, 5.0, i, Some(emb)));
        }
        assert_eq!(stream.len(), 5);
    }

    #[test]
    fn cosine_sim_identical() {
        let a = [1.0f32; 16];
        let sim = cosine_sim(&a, &a);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_sim_zero_vector() {
        let a = [1.0f32; 16];
        let b = [0.0f32; 16];
        assert_eq!(cosine_sim(&a, &b), 0.0);
    }

    #[test]
    fn cosine_sim_orthogonal() {
        let a = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!((cosine_sim(&a, &b)).abs() < 1e-6);
    }

    #[test]
    fn recent_method() {
        let mut stream = MemoryStream::new(100);
        for i in 0..5 {
            stream.add(make_node(MemoryKind::Observation, 5.0, i, None));
        }
        let recent = stream.recent(2);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn by_kind_filter() {
        let mut stream = MemoryStream::new(100);
        stream.add(make_node(MemoryKind::Observation, 5.0, 0, None));
        stream.add(make_node(MemoryKind::Reflection, 5.0, 0, None));
        stream.add(make_node(MemoryKind::Social, 5.0, 0, None));

        let reflections = stream.by_kind(&MemoryKind::Reflection);
        assert_eq!(reflections.len(), 1);
        assert_eq!(reflections[0].kind, MemoryKind::Reflection);
    }

    #[test]
    fn get_by_id() {
        let mut stream = MemoryStream::new(100);
        let id = stream.add(make_node(MemoryKind::Observation, 5.0, 0, None));
        assert!(stream.get(id).is_some());
        assert!(stream.get(999).is_none());
    }

    #[test]
    fn retrieve_by_text_finds_matching_keywords() {
        let mut stream = MemoryStream::new(100);
        let mut n1 = make_node(MemoryKind::Observation, 5.0, 0, None);
        n1.keywords = vec!["rust".into(), "compiler".into()];
        let mut n2 = make_node(MemoryKind::Observation, 5.0, 0, None);
        n2.keywords = vec!["python".into(), "script".into()];
        stream.add(n1);
        stream.add(n2);

        let results = stream.retrieve_by_text("rust compiler", 0, 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].keywords[0], "rust");
    }

    #[test]
    fn retrieve_by_text_no_matches_returns_empty() {
        let mut stream = MemoryStream::new(100);
        let mut n = make_node(MemoryKind::Observation, 5.0, 0, None);
        n.keywords = vec!["rust".into()];
        stream.add(n);

        let results = stream.retrieve_by_text("quantum physics", 0, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(keyword_score(&results[0].keywords, &tokenize("quantum physics")), 0.0);
    }

    #[test]
    fn custom_weights_change_ranking() {
        let emb = [1.0f32; 16];
        let mut stream = MemoryStream::new(100);
        let mut n1 = make_node(MemoryKind::Observation, 5.0, 0, Some(emb));
        n1.keywords = vec!["alpha".into(), "beta".into()];
        let mut n2 = make_node(MemoryKind::Observation, 5.0, 0, Some(emb));
        n2.keywords = vec!["gamma".into()];
        stream.add(n1);
        stream.add(n2);

        let high_lexical = RetrievalWeights {
            recency: 0.0, relevance: 0.0, importance: 0.0, lexical: 1.0,
        };
        let results = stream.retrieve_with_weights(
            None, Some("alpha"), 0, &high_lexical, 10,
        );
        assert_eq!(results[0].keywords[0], "alpha");
    }

    #[test]
    fn dual_score_combines_both_signals() {
        let emb_a = [1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let emb_b = [0.0f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut stream = MemoryStream::new(100);
        let mut n1 = make_node(MemoryKind::Observation, 5.0, 0, Some(emb_a));
        n1.keywords = vec!["alpha".into()];
        let mut n2 = make_node(MemoryKind::Observation, 5.0, 0, Some(emb_b));
        n2.keywords = vec!["beta".into()];
        stream.add(n1);
        stream.add(n2);

        let results = stream.retrieve_with_weights(
            Some(&emb_a), Some("alpha"), 0,
            &RetrievalWeights::default(), 10,
        );
        assert_eq!(results[0].id, 0);
    }

    #[test]
    fn tokenize_handles_punctuation_and_case() {
        let tokens = tokenize("Hello, World! This is a TEST.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        assert!(!tokens.iter().any(|t| t.len() < 2));
    }

    #[test]
    fn keyword_score_partial_overlap() {
        let mem = vec!["a".into(), "b".into(), "c".into()];
        let query = vec!["b".into(), "c".into(), "d".into()];
        let score = keyword_score(&mem, &query);
        assert!((score - 0.5).abs() < 1e-6);
    }

    #[test]
    fn keyword_score_empty_query() {
        let mem = vec!["a".into()];
        assert_eq!(keyword_score(&mem, &[]), 0.0);
    }
}

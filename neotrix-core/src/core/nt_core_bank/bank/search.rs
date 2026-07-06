use std::collections::HashSet;
use std::sync::atomic::Ordering;

use chrono::Utc;

use crate::core::nt_core_knowledge::TaskType;
use crate::core::nt_core_bank::iteration::{Bm25Document, Bm25Index, rrf_fuse};
use crate::core::nt_core_bank::{ReasoningBank, ReasoningMemory, T3ViewType};
use crate::core::nt_core_kron::KroneckerCleanup;

impl ReasoningBank {
    fn rebuild_bm25(&self) {
        if !self.bm25_dirty.load(Ordering::SeqCst) { return; }
        let docs: Vec<Bm25Document> = self.memories.iter().map(|m| {
            let mut text = format!("{} {:?}", m.task_description, m.task_type);
            if let Some(ref v) = m.t3_views.struct_view { text.push_str(&format!(" struct:{}", v)); }
            if let Some(ref v) = m.t3_views.semantic_view { text.push_str(&format!(" semantic:{}", v)); }
            if let Some(ref v) = m.t3_views.reflect_view { text.push_str(&format!(" reflect:{}", v)); }
            Bm25Document { id: m.id.clone(), text }
        }).collect();
        if let Ok(mut bm25) = self.bm25.write() { *bm25 = Bm25Index::build(&docs); }
        self.bm25_dirty.store(false, Ordering::SeqCst);
    }

    fn bm25_search(&self, task: &str, k: usize) -> Vec<(f64, String)> {
        self.rebuild_bm25();
        if let Ok(bm25) = self.bm25.read() { bm25.search(task, k) } else { Vec::new() }
    }

    pub(crate) fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 && norm_b == 0.0 { return 1.0; }
        if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
        dot / (norm_a * norm_b)
    }

    pub fn retrieve_by_wh(&self, query: &str, k: usize) -> Vec<(f64, String)> {
        match self.wh_index {
            Some(ref wh) => wh.search(query, k),
            None => Vec::new(),
        }
    }

    pub fn retrieve_relevant(
        &self,
        task: &str,
        task_type: Option<TaskType>,
        k: usize,
    ) -> Vec<ReasoningMemory> {
        if let Some(ref kronecker) = self.kronecker {
            let result = self.retrieve_with_kronecker(task, task_type, k, kronecker);
            if !result.is_empty() {
                return result;
            }
        }
        let now = Utc::now().timestamp();
        let bm25_results = self.bm25_search(task, k * 2);
        let embed_results = self.vector_search_by_text(task, task_type, k * 2);
        let wh_results = self.retrieve_by_wh(task, k * 2);

        let mut sources: Vec<Vec<(f64, String)>> = Vec::new();
        if !bm25_results.is_empty() { sources.push(bm25_results); }
        if !embed_results.is_empty() { sources.push(embed_results); }
        if !wh_results.is_empty() { sources.push(wh_results); }

        let fused_ids: Vec<String> = if sources.len() >= 2 {
            rrf_fuse(&sources).into_iter().map(|(_, id)| id).collect()
        } else if sources.len() == 1 {
            sources[0].iter().map(|(_, id)| id.clone()).collect()
        } else {
            Vec::new()
        };

        let candidate_indices: Vec<usize> = if let Some(tt) = task_type {
            self.task_type_index.get(&tt).cloned().unwrap_or_default()
        } else {
            (0..self.memories.len()).collect()
        };

        let id_set: HashSet<String> = fused_ids.iter().cloned().collect();
        let mut result_ids: Vec<String> = if let Some(_tt) = task_type {
            let candidate_set: HashSet<&str> = candidate_indices.iter()
                .filter_map(|&idx| self.memories.get(idx))
                .map(|m| m.id.as_str())
                .collect();
            fused_ids.into_iter().filter(|id: &String| candidate_set.contains(id.as_str())).collect()
        } else {
            fused_ids
        };
        if result_ids.len() < k {
            let mut extra_ids: Vec<(f64, String)> = candidate_indices.iter()
                .filter_map(|&idx| self.memories.get(idx))
                .filter(|m| !id_set.contains(&m.id))
                .map(|m| {
                    let age = (now - m.timestamp).max(0);
                    let recency = (-(age as f64) / 604800.0).exp();
                    (recency + m.reward * 0.3, m.id.clone())
                }).collect();
            extra_ids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            result_ids.extend(extra_ids.into_iter().take(k.saturating_sub(result_ids.len())).map(|(_, id)| id));
        }

        let final_ids: Vec<String> = result_ids.into_iter().take(k).collect();
        let mut result: Vec<ReasoningMemory> = Vec::new();
        for id in &final_ids {
            if let Some(m) = self.memories.iter().find(|m| m.id == *id) {
                let mut mc = m.clone();
                mc.lifecycle.access_count += 1;
                mc.lifecycle.last_accessed = now;
                result.push(mc);
            }
        }
        result
    }

    fn retrieve_with_kronecker(
        &self,
        task: &str,
        task_type: Option<TaskType>,
        k: usize,
        kronecker: &KroneckerCleanup,
    ) -> Vec<ReasoningMemory> {
        use crate::core::nt_core_embed::TextEmbedder;

        let mut embedder = TextEmbedder::new();
        let query_emb = embedder.embed(task);
        if query_emb.len() != kronecker.dim() {
            return Vec::new();
        }

        let candidate_indices: Vec<usize> = if let Some(tt) = task_type {
            self.task_type_index.get(&tt).cloned().unwrap_or_default()
        } else {
            (0..self.memories.len()).collect()
        };

        let items: Vec<(String, Vec<f64>)> = candidate_indices
            .iter()
            .filter_map(|&idx| self.memories.get(idx))
            .filter_map(|m| m.embedding.as_ref().map(|emb| (m.id.clone(), emb.clone())))
            .collect();

        if items.is_empty() {
            return Vec::new();
        }

        let results = kronecker.cleanup(&query_emb, &items, k);

        let now = Utc::now().timestamp();
        results
            .into_iter()
            .filter_map(|(id, _score)| {
                self.memories.iter().find(|m| m.id == id).map(|m| {
                    let mut mc = m.clone();
                    mc.lifecycle.access_count += 1;
                    mc.lifecycle.last_accessed = now;
                    mc
                })
            })
            .collect()
    }

    fn vector_search_by_text(
        &self,
        _task: &str,
        task_type: Option<TaskType>,
        k: usize,
    ) -> Vec<(f64, String)> {
        let candidate_indices: Vec<usize> = if let Some(tt) = task_type {
            self.task_type_index.get(&tt).cloned().unwrap_or_default()
        } else {
            (0..self.memories.len()).collect()
        };
        let mut scored: Vec<(f64, &ReasoningMemory)> = candidate_indices.iter()
            .filter_map(|&idx| self.memories.get(idx))
            .filter_map(|m| m.embedding.as_ref().map(|emb| {
                let norm: f64 = emb.iter().map(|x| x * x).sum::<f64>().sqrt();
                (norm, m)
            }))
            .filter(|(score, _)| *score > 0.0)
            .map(|(norm, m)| {
                let type_bonus = if let Some(tt) = task_type { if m.task_type == tt { 0.2 } else { 0.0 } } else { 0.0 };
                (norm + type_bonus, m)
            }).collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).map(|(s, m)| (s, m.id.clone())).collect()
    }

    pub fn retrieve_relevant_by_embedding(
        &self,
        task_embedding: &[f64],
        task_type: Option<TaskType>,
        k: usize,
    ) -> Vec<ReasoningMemory> {
        if task_embedding.is_empty() { return Vec::new(); }
        let candidate_indices: Vec<usize> = if let Some(tt) = task_type {
            self.task_type_index.get(&tt).cloned().unwrap_or_default()
        } else {
            (0..self.memories.len()).collect()
        };
        let mut scored: Vec<(f64, &ReasoningMemory)> = candidate_indices.iter()
            .filter_map(|&idx| self.memories.get(idx))
            .filter_map(|m| m.embedding.as_ref().map(|emb| (Self::cosine_similarity(task_embedding, emb), m)))
            .filter(|(score, _)| *score > 0.0)
            .map(|(sim, m)| {
                let type_bonus = if let Some(tt) = task_type { if m.task_type == tt { 0.2 } else { 0.0 } } else { 0.0 };
                (sim + type_bonus, m)
            }).collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).map(|(_, m)| m.clone()).collect()
    }

    fn graph_search_by_entities(
        &self,
        task: &str,
        task_type: Option<TaskType>,
        k: usize,
    ) -> Vec<(f64, String)> {
        let query_tokens: HashSet<String> = task.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() >= 3)
            .map(|s| s.to_string()).collect();
        if query_tokens.is_empty() { return Vec::new(); }

        let candidate_indices: Vec<usize> = if let Some(tt) = task_type {
            self.task_type_index.get(&tt).cloned().unwrap_or_default()
        } else {
            (0..self.memories.len()).collect()
        };

        let now = Utc::now().timestamp();
        let mut scored: Vec<(f64, usize)> = candidate_indices.iter()
            .filter_map(|&idx| self.memories.get(idx).map(|m| (idx, m)))
            .map(|(idx, m)| {
                let mem_tokens: HashSet<String> = m.task_description.to_lowercase()
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|s| s.len() >= 3)
                    .map(|s| s.to_string()).collect();
                let intersection = query_tokens.intersection(&mem_tokens).count();
                let union = query_tokens.union(&mem_tokens).count();
                let jaccard = if union > 0 { intersection as f64 / union as f64 } else { 0.0 };
                let recency = (-((now - m.timestamp).max(0) as f64) / 604800.0).exp();
                (jaccard * 3.0 + recency + m.reward * 0.5, idx)
            })
            .filter(|(s, _)| *s > 0.0).collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).map(|(s, idx)| (s, self.memories[idx].id.clone())).collect()
    }

    fn t3_search(&self, query: &str, view_type: T3ViewType, _tt: Option<TaskType>, k: usize) -> Vec<(f64, String)> {
        let qv: Vec<&str> = query.split_whitespace().collect();
        if qv.is_empty() { return Vec::new(); }
        let mut scored: Vec<(f64, String)> = self.memories.iter().filter_map(|m| {
            let text = m.t3_views.get(view_type)?;
            let matches = qv.iter().filter(|w| text.to_lowercase().contains(&w.to_lowercase())).count();
            if matches == 0 { return None; }
            Some((matches as f64 / qv.len() as f64 + m.reward * 0.2, m.id.clone()))
        }).collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).collect()
    }

    pub fn retrieve_by_view(
        &self,
        query: &str,
        view_type: T3ViewType,
        task_type: Option<TaskType>,
        k: usize,
    ) -> Vec<ReasoningMemory> {
        let bm25_results = self.bm25_search(query, k * 2);
        let embed_results = self.vector_search_by_text(query, task_type, k * 2);
        let t3_results = self.t3_search(query, view_type, task_type, k * 2);

        let mut sources: Vec<Vec<(f64, String)>> = Vec::new();
        if !bm25_results.is_empty() { sources.push(bm25_results); }
        if !embed_results.is_empty() { sources.push(embed_results); }
        if !t3_results.is_empty() { sources.push(t3_results); }

        let fused_ids: Vec<String> = if sources.is_empty() {
            Vec::new()
        } else if sources.len() == 1 {
            sources[0].iter().map(|(_, id)| id.clone()).collect()
        } else {
            rrf_fuse(&sources).into_iter().map(|(_, id)| id).collect()
        };

        let now = Utc::now().timestamp();
        let candidate_indices: Vec<usize> = if let Some(tt) = task_type {
            self.task_type_index.get(&tt).cloned().unwrap_or_default()
        } else {
            (0..self.memories.len()).collect()
        };

        let mut result_ids = fused_ids;
        let id_set: HashSet<String> = result_ids.iter().cloned().collect();
        if result_ids.len() < k {
            let mut extra_ids: Vec<(f64, String)> = candidate_indices.iter()
                .filter_map(|&idx| self.memories.get(idx))
                .filter(|m| !id_set.contains(&m.id))
                .map(|m| {
                    let age = (now - m.timestamp).max(0);
                    ((-(age as f64) / 604800.0).exp() + m.reward * 0.3, m.id.clone())
                }).collect();
            extra_ids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            result_ids.extend(extra_ids.into_iter().take(k.saturating_sub(result_ids.len())).map(|(_, id)| id));
        }

        let mut result: Vec<ReasoningMemory> = result_ids.into_iter().take(k)
            .filter_map(|id| self.memories.iter().find(|m| m.id == id))
            .cloned().collect();
        for m in &mut result { ReasoningMemory::touch(m); }
        result
    }

    pub fn retrieve_all_views(
        &self,
        query: &str,
        task_type: Option<TaskType>,
        k_per_view: usize,
    ) -> Vec<ReasoningMemory> {
        let mut all = Vec::new();
        let mut seen = HashSet::new();
        for &view_type in T3ViewType::all() {
            for mem in self.retrieve_by_view(query, view_type, task_type, k_per_view) {
                if seen.insert(mem.id.clone()) { all.push(mem); }
            }
        }
        all
    }

    pub fn multi_modal_search(
        &self,
        query: &str,
        bm25_weight: f64,
        vector_weight: f64,
        graph_weight: f64,
    ) -> Vec<ReasoningMemory> {
        let now = Utc::now().timestamp();
        let k = 20;
        let bm25_results = self.bm25_search(query, k);
        let vec_results = self.vector_search_by_text(query, None, k);
        let graph_results = self.graph_search_by_entities(query, None, k);

        let weighted_bm25: Vec<(f64, String)> = bm25_results.into_iter().map(|(s, id)| (s * bm25_weight, id)).collect();
        let weighted_vec: Vec<(f64, String)> = vec_results.into_iter().map(|(s, id)| (s * vector_weight, id)).collect();
        let weighted_graph: Vec<(f64, String)> = graph_results.into_iter().map(|(s, id)| (s * graph_weight, id)).collect();

        let mut sources: Vec<Vec<(f64, String)>> = Vec::new();
        if !weighted_bm25.is_empty() { sources.push(weighted_bm25); }
        if !weighted_vec.is_empty() { sources.push(weighted_vec); }
        if !weighted_graph.is_empty() { sources.push(weighted_graph); }

        let fused_ids: Vec<String> = if sources.len() >= 2 {
            rrf_fuse(&sources).into_iter().map(|(_, id)| id).collect()
        } else if sources.len() == 1 {
            sources[0].iter().map(|(_, id)| id.clone()).collect()
        } else {
            return Vec::new();
        };

        let mut result: Vec<ReasoningMemory> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for id in &fused_ids {
            if !seen.insert(id.clone()) { continue; }
            if let Some(m) = self.memories.iter().find(|m| m.id == *id) {
                let mut mc = m.clone();
                mc.lifecycle.access_count += 1;
                mc.lifecycle.last_accessed = now;
                result.push(mc);
            }
        }
        result
    }

    pub fn get_successes(&self) -> Vec<ReasoningMemory> {
        self.memories.iter().filter(|m| m.success).cloned().collect()
    }

    pub fn enable_hypergraph(&mut self, initial_capacity: usize) {
        let graph = crate::core::nt_core_graph::HyperGraph::with_capacity(initial_capacity);
        self.hypergraph = Some(graph);
    }

    pub fn index_memory(&mut self, memory_id: &str) -> Result<(), String> {
        let graph = self.hypergraph.as_mut().ok_or_else(|| "hypergraph not enabled".to_string())?;
        let mem = self.memories.iter().find(|m| m.id == memory_id).cloned()
            .ok_or_else(|| format!("memory not found: {}", memory_id))?;

        let node_type = match mem.task_type {
            TaskType::Learning | TaskType::Research | TaskType::Reflection => crate::core::nt_core_graph::HyperNodeType::Concept,
            TaskType::CodeAnalysis | TaskType::CodeReview | TaskType::CodeGeneration => crate::core::nt_core_graph::HyperNodeType::Pattern,
            TaskType::UIDesign | TaskType::Security => crate::core::nt_core_graph::HyperNodeType::Skill,
            TaskType::Planning => crate::core::nt_core_graph::HyperNodeType::Goal,
            _ => crate::core::nt_core_graph::HyperNodeType::Memory,
        };

        let mut node = crate::core::nt_core_graph::HyperNode::new(&mem.id, node_type, &mem.task_description, mem.reward);
        if let Some(ref emb) = mem.embedding { node.embedding = emb.clone(); }
        graph.add_node(node);

        let existing_ids: Vec<String> = graph.nodes.keys().cloned().collect();
        for other_id in &existing_ids {
            if other_id == memory_id { continue; }
            if let Some(other_node) = graph.nodes.get(other_id) {
                let mut strength = 0.0;
                if let Some(ref emb) = mem.embedding {
                    if !other_node.embedding.is_empty() {
                        strength = crate::core::nt_core_graph::HyperGraph::cosine_similarity(emb, &other_node.embedding);
                    }
                }
                if strength == 0.0 {
                    if let Some(other_mem) = self.memories.iter().find(|m| m.id == *other_id) {
                        if other_mem.task_type == mem.task_type { strength = 0.5; }
                    }
                }
                if strength > 0.3 {
                    graph.add_edge(memory_id, other_id, crate::core::nt_core_graph::EdgeRelation::SimilarTo, strength);
                }
            }
        }
        Ok(())
    }

    pub fn hypergraph_traverse(&self, start_memory_id: &str, depth: usize) -> Vec<String> {
        let graph = match self.hypergraph { Some(ref g) => g, None => return Vec::new() };
        let nodes = graph.traverse(start_memory_id, depth);
        nodes.iter().map(|n| n.id.clone()).collect()
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}

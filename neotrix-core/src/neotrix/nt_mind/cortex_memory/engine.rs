use std::collections::{HashMap, VecDeque, HashSet};
use crate::neotrix::nt_mind::embedding::TextEmbedder;
use crate::neotrix::signal::ops::cosine_similarity;
use super::types::*;

// CortexMemory — 类人脑多维度存储系统
// ============================================================

/// 大脑皮层记忆系统
/// 比人脑更好的设计：
/// - 精确的多维索引 (timeline × dimension × modality)
/// - 向量嵌入联想检索
/// - 自动短期→长期巩固
/// - 跨模态关联
pub struct CortexMemory {
    /// 短期缓冲（感觉记忆，最近 N 条）
    pub(crate) nt_world_sense_buffer: VecDeque<MemoryTrace>,
    /// 长期存储（所有持久化记忆）
    pub(crate) long_term: Vec<MemoryTrace>,
    /// 最大短期容量
    nt_world_sense_capacity: usize,
    /// 最大长期容量
    long_term_capacity: usize,
    /// 维度索引 (dimension → trace ids)
    dimension_index: HashMap<DimensionTag, HashSet<String>>,
    /// 模态索引 (modality name → trace ids)
    modality_index: HashMap<String, HashSet<String>>,
    /// 来源索引 (source type → trace ids)
    source_index: HashMap<String, HashSet<String>>,
    /// 时间线索引 (epoch → trace ids)
    timeline_index: HashMap<String, HashSet<String>>,
    /// 嵌入器
    embedder: TextEmbedder,
    /// 所有 trace 的 id→trace 映射
    trace_map: HashMap<String, MemoryTrace>,
    /// 超边索引: hyperedge_id → trace_ids (HyperMem n-ary relationships)
    hyperedge_index: HashMap<String, Vec<String>>,
    /// 层级索引: layer → trace_ids
    layer_index: HashMap<MemoryLayer, HashSet<String>>,
}

impl CortexMemory {
    pub fn new(nt_world_sense_capacity: usize, long_term_capacity: usize) -> Self {
        Self {
            nt_world_sense_buffer: VecDeque::with_capacity(nt_world_sense_capacity),
            long_term: Vec::with_capacity(long_term_capacity),
            nt_world_sense_capacity,
            long_term_capacity,
            dimension_index: HashMap::new(),
            modality_index: HashMap::new(),
            source_index: HashMap::new(),
            timeline_index: HashMap::new(),
            embedder: TextEmbedder::new(),
            trace_map: HashMap::new(),
            hyperedge_index: HashMap::new(),
            layer_index: HashMap::new(),
        }
    }

    // ==================== 存储 ====================

    /// 存储一条记忆痕迹（自动计算 embedding + 索引）
    pub fn store(&mut self, mut trace: MemoryTrace) -> String {
        let id = trace.id.clone();

        if trace.embedding.is_none() {
            let text_for_embed = format!("{} {} {}", trace.title, trace.summary, trace.tags.join(" "));
            trace.embedding = Some(self.embedder.embed(&text_for_embed));
        }

        self.trace_map.insert(id.clone(), trace.clone());

        if self.nt_world_sense_buffer.len() >= self.nt_world_sense_capacity {
            self.nt_world_sense_buffer.pop_front();
        }
        self.nt_world_sense_buffer.push_back(trace.clone());

        self.layer_index.entry(MemoryLayer::Sensory).or_default().insert(id.clone());

        for dim in &trace.dimensions {
            self.dimension_index.entry(*dim).or_default().insert(id.clone());
        }

        self.modality_index.entry(trace.modality.name().to_string())
            .or_default().insert(id.clone());

        self.source_index.entry(trace.source_type.clone())
            .or_default().insert(id.clone());

        id
    }

    /// 巩固：短期 → 长期（带重要性过滤），同时 Sensory → Topic 层级提升
    pub fn consolidate(&mut self, importance_threshold: f64) -> usize {
        let mut consolidated = 0usize;
        let mut to_keep: VecDeque<MemoryTrace> = VecDeque::new();

        while let Some(trace) = self.nt_world_sense_buffer.pop_front() {
            if trace.importance >= importance_threshold {
                if self.long_term.len() >= self.long_term_capacity {
                    self.long_term.remove(0);
                }
                self.layer_index.entry(MemoryLayer::Sensory).or_default().remove(&trace.id);
                self.layer_index.entry(MemoryLayer::Topic).or_default().insert(trace.id.clone());
                let epoch = self.epoch_for_timestamp(trace.timestamp);
                self.timeline_index.entry(epoch).or_default()
                    .insert(trace.id.clone());
                self.long_term.push(trace);
                consolidated += 1;
            } else {
                to_keep.push_back(trace);
            }
        }

        self.nt_world_sense_buffer = to_keep;
        consolidated
    }

    /// 强制所有短期 → 长期
    pub fn consolidate_all(&mut self) -> usize {
        let count = self.nt_world_sense_buffer.len();
        while let Some(trace) = self.nt_world_sense_buffer.pop_front() {
            if self.long_term.len() >= self.long_term_capacity {
                self.long_term.remove(0);
            }
            let epoch = self.epoch_for_timestamp(trace.timestamp);
            self.timeline_index.entry(epoch).or_default()
                .insert(trace.id.clone());
            self.long_term.push(trace);
        }
        count
    }

    // ==================== CMS — Continuum Memory System ====================

    pub fn consolidate_layer(&mut self, from: MemoryLayer, threshold: f64) -> usize {
        let next = match from.next() {
            Some(l) => l,
            None => return 0,
        };
        let ids: Vec<String> = self.layer_index.get(&from)
            .map(|ids| ids.iter().cloned().collect())
            .unwrap_or_default();
        let mut promoted = 0usize;
        for id in &ids {
            if let Some(trace) = self.trace_map.get(id) {
                if trace.importance >= threshold {
                    self.layer_index.entry(from).or_default().remove(id);
                    self.layer_index.entry(next).or_default().insert(id.clone());
                    promoted += 1;
                }
            }
        }
        promoted
    }

    pub fn consolidate_cms(&mut self, iteration: u64, config: &CmsConfig) -> CmsResult {
        let mut result = CmsResult::default();
        let _old_topic = self.layer_index.get(&MemoryLayer::Topic).map(|s| s.len()).unwrap_or(0);
        result.nt_world_sense_to_topic = self.consolidate(config.topic_threshold);
        if iteration.is_multiple_of(config.topic_frequency as u64) {
            result.topic_to_event = self.consolidate_layer(MemoryLayer::Topic, config.event_threshold);
        }
        if iteration.is_multiple_of(config.event_frequency as u64) {
            result.event_to_fact = self.consolidate_layer(MemoryLayer::Event, config.fact_threshold);
        }
        result.topic_layer_size = self.layer_index.get(&MemoryLayer::Topic).map(|s| s.len()).unwrap_or(0);
        result.event_layer_size = self.layer_index.get(&MemoryLayer::Event).map(|s| s.len()).unwrap_or(0);
        result.fact_layer_size = self.layer_index.get(&MemoryLayer::Fact).map(|s| s.len()).unwrap_or(0);
        result
    }

    // ==================== 检索 ====================

    /// 按维度检索（多维度视角查询）
    pub fn query_by_dimension(&self, dim: DimensionTag, limit: usize) -> Vec<&MemoryTrace> {
        self.dimension_index.get(&dim)
            .map(|ids| {
                let mut traces: Vec<&MemoryTrace> = ids.iter()
                    .filter_map(|id| self.trace_map.get(id))
                    .collect();
                traces.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
                traces.truncate(limit);
                traces
            })
            .unwrap_or_default()
    }

    /// 按多维度组合检索
    pub fn query_by_dimensions(&self, dims: &[DimensionTag], limit: usize) -> Vec<&MemoryTrace> {
        let mut all_ids: HashSet<String> = HashSet::new();
        for dim in dims {
            if let Some(ids) = self.dimension_index.get(dim) {
                all_ids.extend(ids.iter().cloned());
            }
        }
        let mut traces: Vec<&MemoryTrace> = all_ids.iter()
            .filter_map(|id| self.trace_map.get(id))
            .collect();
        traces.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        traces.truncate(limit);
        traces
    }

    /// 按模态检索
    pub fn query_by_modality(&self, modality: &str, limit: usize) -> Vec<&MemoryTrace> {
        self.modality_index.get(modality)
            .map(|ids| {
                let mut traces: Vec<&MemoryTrace> = ids.iter()
                    .filter_map(|id| self.trace_map.get(id))
                    .collect();
                traces.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
                traces.truncate(limit);
                traces
            })
            .unwrap_or_default()
    }

    /// 按关键词联想检索（embedding 相似度）
    pub fn recall(&mut self, query: &str, limit: usize) -> Vec<(&MemoryTrace, f64)> {
        let qv = self.embedder.embed(query);
        let mut scored: Vec<(&MemoryTrace, f64)> = self.trace_map.values()
            .filter_map(|trace| {
                trace.embedding.as_ref().map(|ev| {
                    let sim = cosine_similarity(&qv, ev);
                    (trace, sim * 0.7 + trace.importance * 0.3)
                })
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored
    }

    /// 时间线查询（按 epoch）
    pub fn query_timeline(&self, epoch: &str, limit: usize) -> Vec<&MemoryTrace> {
        self.timeline_index.get(epoch)
            .map(|ids| {
                let mut traces: Vec<&MemoryTrace> = ids.iter()
                    .filter_map(|id| self.trace_map.get(id))
                    .collect();
                traces.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
                traces.truncate(limit);
                traces
            })
            .unwrap_or_default()
    }

    /// 按来源类型查询
    pub fn query_by_source(&self, source_type: &str, limit: usize) -> Vec<&MemoryTrace> {
        self.source_index.get(source_type)
            .map(|ids| {
                let mut traces: Vec<&MemoryTrace> = ids.iter()
                    .filter_map(|id| self.trace_map.get(id))
                    .collect();
                traces.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
                traces.truncate(limit);
                traces
            })
            .unwrap_or_default()
    }

    /// 关联检索：给定一条记忆，找出相关联的记忆
    pub fn get_associations(&self, trace_id: &str, limit: usize) -> Vec<&MemoryTrace> {
        self.trace_map.get(trace_id)
            .map(|trace| {
                let mut assoc: Vec<&MemoryTrace> = trace.associations.iter()
                    .filter_map(|aid| self.trace_map.get(aid))
                    .collect();
                assoc.truncate(limit);
                assoc
            })
            .unwrap_or_default()
    }

    // ==================== 时间线管理 ====================

    fn epoch_for_timestamp(&self, _ts: i64) -> String {
        "recent".to_string()
    }

    // ==================== 维度视角查询 ====================

    /// 多维度链查询 — 核心功能
    /// 返回指定维度链中的所有记忆，按重要性排序
    pub fn dimension_chain(&self, category: &str, limit: usize) -> Vec<&MemoryTrace> {
        let dims: Vec<DimensionTag> = DimensionTag::all().into_iter()
            .filter(|d| d.category() == category)
            .collect();
        self.query_by_dimensions(&dims, limit)
    }

    /// Iterate over all stored traces (nt_world_sense + long-term)
    pub fn all_traces(&self) -> Vec<&MemoryTrace> {
        self.trace_map.values().collect()
    }

    // ==================== 统计 ====================

    pub fn stats(&self) -> CortexStats {
        let per_dim: HashMap<String, usize> = self.dimension_index.iter()
            .map(|(k, v)| (format!("{:?}", k), v.len()))
            .collect();
        let per_modality: HashMap<String, usize> = self.modality_index.iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        CortexStats {
            nt_world_sense_count: self.nt_world_sense_buffer.len(),
            long_term_count: self.long_term.len(),
            total_traces: self.trace_map.len(),
            per_dimension: per_dim,
            per_modality,
        }
    }

    /// 生成维度链报告（markdown 格式）
    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str("🧠 CortexMemory 多维度知识报告\n");
        r.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        r.push_str(&format!("短期缓冲: {} / {}\n", self.nt_world_sense_buffer.len(), self.nt_world_sense_capacity));
        r.push_str(&format!("长期记忆: {} / {}\n", self.long_term.len(), self.long_term_capacity));
        r.push_str(&format!("总痕迹数: {}\n", self.trace_map.len()));

        let categories = ["时间链", "文明链", "科技链", "物种链", "地理链", "宇宙链", "知识链"];
        for cat in &categories {
            let traces = self.dimension_chain(cat, 5);
            if !traces.is_empty() {
                r.push_str(&format!("\n📌 {} ({}条)\n", cat, traces.len()));
                for t in traces {
                    let dims: Vec<String> = t.dimensions.iter().map(|d| format!("{:?}", d)).collect();
                    r.push_str(&format!("  • {} [{:?}] ← {}\n    {}\n",
                        t.title, t.modality, t.source, dims.join(", ")));
                }
            }
        }
        r
    }

    // ==================== 持久化 ====================

    pub fn export_json(&self) -> serde_json::Value {
        let traces: Vec<&MemoryTrace> = self.long_term.iter().chain(self.nt_world_sense_buffer.iter()).collect();
        let json_traces: Vec<serde_json::Value> = traces.iter().map(|t| {
            serde_json::json!({
                "id": t.id,
                "timestamp": t.timestamp,
                "source": t.source,
                "source_type": t.source_type,
                "title": t.title,
                "summary": t.summary,
                "modality": t.modality.name(),
                "dimensions": t.dimensions.iter().map(|d| format!("{:?}", d)).collect::<Vec<_>>(),
                "importance": t.importance,
                "tags": t.tags,
                "content_length": t.content_length,
            })
        }).collect();

        serde_json::json!({
            "nt_world_sense_count": self.nt_world_sense_buffer.len(),
            "long_term_count": self.long_term.len(),
            "total_traces": self.trace_map.len(),
            "traces": json_traces,
        })
    }

    // ==================== HyperMem 超图记忆 (EverOS) ====================

    pub fn store_with_layer(&mut self, trace: MemoryTrace, layer: MemoryLayer) -> String {
        let id = self.store(trace);
        self.layer_index.entry(MemoryLayer::Sensory).or_default().remove(&id);
        self.layer_index.entry(layer).or_default().insert(id.clone());
        id
    }

    pub fn create_hyperedge(&mut self, hyperedge_id: &str, trace_ids: &[&str]) -> bool {
        if self.hyperedge_index.contains_key(hyperedge_id) {
            return false;
        }
        let ids: Vec<&str> = trace_ids.to_vec();
        let existing: Vec<String> = ids.iter()
            .filter(|id| self.trace_map.contains_key(**id))
            .map(|id| id.to_string())
            .collect();
        if existing.is_empty() {
            return false;
        }
        self.hyperedge_index.insert(hyperedge_id.to_string(), existing);
        true
    }

    pub fn get_hyperedges(&self, trace_id: &str) -> Vec<&str> {
        self.hyperedge_index.iter()
            .filter(|(_, ids)| ids.contains(&trace_id.to_string()))
            .map(|(id, _)| id.as_str())
            .collect()
    }

    pub fn retrieve_coarse_to_fine(&mut self, query: &str, k: usize) -> (Vec<MemoryTrace>, Vec<MemoryTrace>, Vec<MemoryTrace>) {
        let query_embed = self.embedder.embed(query);
        let mut topic_scores: Vec<(f64, &MemoryTrace)> = Vec::new();
        let mut event_scores: Vec<(f64, &MemoryTrace)> = Vec::new();
        let mut fact_scores: Vec<(f64, &MemoryTrace)> = Vec::new();

        for trace in self.long_term.iter().chain(self.nt_world_sense_buffer.iter()) {
            if let Some(ref emb) = trace.embedding {
                let sim = cosine_similarity(&query_embed, emb);
                match self.detect_layer(&trace.id) {
                    MemoryLayer::Topic => topic_scores.push((sim, trace)),
                    MemoryLayer::Event => event_scores.push((sim, trace)),
                    _ => fact_scores.push((sim, trace)),
                }
            }
        }

        topic_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        event_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        fact_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let topic: Vec<MemoryTrace> = topic_scores.into_iter().take(k).map(|(_, t)| t.clone()).collect();
        let event: Vec<MemoryTrace> = event_scores.into_iter().take(k).map(|(_, t)| t.clone()).collect();
        let fact: Vec<MemoryTrace> = fact_scores.into_iter().take(k).map(|(_, t)| t.clone()).collect();
        (topic, event, fact)
    }

    pub fn promote_layer(&mut self, trace_id: &str, target_layer: MemoryLayer) -> bool {
        let current = self.detect_layer(trace_id);
        if current == target_layer {
            return false;
        }
        self.layer_index.entry(current).or_default().remove(trace_id);
        self.layer_index.entry(target_layer).or_default().insert(trace_id.to_string());
        true
    }

    fn detect_layer(&self, trace_id: &str) -> MemoryLayer {
        for layer in MemoryLayer::all() {
            if self.layer_index.get(&layer)
                .map(|ids| ids.contains(trace_id))
                .unwrap_or(false)
            {
                return layer;
            }
        }
        if self.trace_map.contains_key(trace_id) {
            return MemoryLayer::Sensory;
        }
        MemoryLayer::Fact
    }
}

/// 从 KnowledgeSource 来源自动注入 cortex
pub fn inject_from_web_miner(
    cortex: &mut CortexMemory,
    source_url: &str,
    _source_name: &str,
    source_type: &str,
    title: &str,
    summary: &str,
    edits: &[(String, f64)],
) -> String {
    let dims = DimensionTag::detect(title, summary);
    let mut tags: Vec<String> = vec![
        source_type.to_string(),
        format!("{}_sources", source_type),
    ];
    for (dim, _) in edits {
        tags.push(dim.clone());
    }

    let trace = MemoryTrace::new(
        title,
        source_url,
        summary,
        Modality::Text,
        dims,
    )
    .with_tags(tags)
    .with_importance(0.6 + edits.len() as f64 * 0.03);

    cortex.store(trace)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}

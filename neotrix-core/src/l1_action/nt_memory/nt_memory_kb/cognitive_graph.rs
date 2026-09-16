//! Cognitive Event Graph — 吸收自 agentic-memory
//!
//! 认知事件图: facts, decisions, inferences, corrections, skills, episodes
//! 16查询类型: traversal, pattern matching, temporal comparison, causal impact,
//! reasoning gap detection, analogical reasoning, consolidation, drift detection
//! Supersession: corrections don't delete, they supersede (保留历史)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 认知事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    Fact,
    Decision,
    Inference,
    Correction,
    Skill,
    Episode,
}

/// 认知事件节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEvent {
    pub id: String,
    pub event_type: EventType,
    pub content: String,
    pub timestamp: u64,
    pub confidence: f64,               // 0.0 - 1.0
    pub superseded_by: Option<String>, // 被哪个事件取代
    pub tags: Vec<String>,
}

/// 边类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    CausedBy,     // 因果关系
    InferredFrom, // 推理来源
    Supersedes,   // 取代关系
    Temporal,     // 时间顺序
    RelatedTo,    // 相关关系
}

/// 认知图边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEdge {
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub weight: f64,
}

/// 认知事件图
pub struct CognitiveEventGraph {
    events: HashMap<String, CognitiveEvent>,
    edges: Vec<CognitiveEdge>,
    tag_index: HashMap<String, Vec<String>>, // tag → event_ids
    type_index: HashMap<EventType, Vec<String>>, // type → event_ids
}

impl CognitiveEventGraph {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            edges: Vec::new(),
            tag_index: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// 添加事件
    pub fn add_event(&mut self, event: CognitiveEvent) {
        let id = event.id.clone();
        let event_type = event.event_type;
        let tags = event.tags.clone();

        self.events.insert(id.clone(), event);
        self.type_index
            .entry(event_type)
            .or_default()
            .push(id.clone());

        for tag in tags {
            self.tag_index.entry(tag).or_default().push(id.clone());
        }
    }

    /// 添加边
    pub fn add_edge(&mut self, edge: CognitiveEdge) {
        self.edges.push(edge);
    }

    /// Supersession: 用新事件取代旧事件（保留历史）
    pub fn supersede(&mut self, old_id: &str, new_event: CognitiveEvent) {
        let new_id = new_event.id.clone();

        // 标记旧事件被取代
        if let Some(old) = self.events.get_mut(old_id) {
            old.superseded_by = Some(new_id.clone());
        }

        // 添加取代边
        self.edges.push(CognitiveEdge {
            source: new_id.clone(),
            target: old_id.to_string(),
            edge_type: EdgeType::Supersedes,
            weight: 1.0,
        });

        // 添加新事件
        self.add_event(new_event);
    }

    /// 查询: 获取当前有效版本（未被取代的）
    pub fn resolve(&self, id: &str) -> Option<&CognitiveEvent> {
        let event = self.events.get(id)?;
        if event.superseded_by.is_some() {
            // 被取代了，递归查找最终版本
            let mut current = event;
            while let Some(ref next_id) = current.superseded_by {
                current = self.events.get(next_id)?;
            }
            Some(current)
        } else {
            Some(event)
        }
    }

    /// 查询: 按类型检索
    pub fn by_type(&self, event_type: EventType) -> Vec<&CognitiveEvent> {
        self.type_index
            .get(&event_type)
            .map(|ids| ids.iter().filter_map(|id| self.resolve(id)).collect())
            .unwrap_or_default()
    }

    /// 查询: 按标签检索
    pub fn by_tag(&self, tag: &str) -> Vec<&CognitiveEvent> {
        self.tag_index
            .get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.resolve(id)).collect())
            .unwrap_or_default()
    }

    /// 查询: 因果链追踪
    pub fn causal_chain(&self, event_id: &str) -> Vec<CognitiveEvent> {
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.trace_causal(event_id, &mut chain, &mut visited);
        chain
    }

    fn trace_causal(
        &self,
        id: &str,
        chain: &mut Vec<CognitiveEvent>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if visited.contains(id) {
            return;
        }
        visited.insert(id.to_string());

        if let Some(event) = self.resolve(id) {
            chain.push(event.clone());
        }

        // 查找因果边 (target → source 表示 source caused by target)
        for edge in &self.edges {
            if edge.target == id && edge.edge_type == EdgeType::CausedBy {
                self.trace_causal(&edge.source, chain, visited);
            }
        }
    }

    /// 查询: 推理缺口检测
    pub fn reasoning_gaps(&self) -> Vec<String> {
        let mut gaps = Vec::new();
        let _facts: Vec<&CognitiveEvent> = self.by_type(EventType::Fact);
        let inferences: Vec<&CognitiveEvent> = self.by_type(EventType::Inference);

        // 简单启发: 如果有推理但缺少某些前提事实
        for inf in &inferences {
            let deps: Vec<&str> = self
                .edges
                .iter()
                .filter(|e| e.source == inf.id && e.edge_type == EdgeType::InferredFrom)
                .map(|e| e.target.as_str())
                .collect();

            if deps.is_empty() {
                gaps.push(format!(
                    "Inference '{}' has no inferred-from dependencies",
                    inf.id
                ));
            }
        }

        gaps
    }

    /// 查询: 漂移检测 (confidence 低的事件)
    pub fn drift_detection(&self, min_confidence: f64) -> Vec<&CognitiveEvent> {
        self.events
            .values()
            .filter(|e| e.confidence < min_confidence && e.superseded_by.is_none())
            .collect()
    }

    /// 统计
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            total_events: self.events.len(),
            total_edges: self.edges.len(),
            active_events: self
                .events
                .values()
                .filter(|e| e.superseded_by.is_none())
                .count(),
            type_distribution: self
                .type_index
                .iter()
                .map(|(k, v)| (format!("{:?}", k), v.len()))
                .collect(),
        }
    }
}

impl Default for CognitiveEventGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_events: usize,
    pub total_edges: usize,
    pub active_events: usize,
    pub type_distribution: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_event(id: &str, event_type: EventType, content: &str) -> CognitiveEvent {
        CognitiveEvent {
            id: id.to_string(),
            event_type,
            content: content.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            confidence: 0.9,
            superseded_by: None,
            tags: vec![],
        }
    }

    #[test]
    fn test_add_and_resolve() {
        let mut graph = CognitiveEventGraph::new();
        graph.add_event(make_event("e1", EventType::Fact, "Water is wet"));

        let resolved = graph.resolve("e1");
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().content, "Water is wet");
    }

    #[test]
    fn test_supersession() {
        let mut graph = CognitiveEventGraph::new();
        graph.add_event(make_event("e1", EventType::Fact, "Old fact"));
        graph.supersede("e1", make_event("e2", EventType::Fact, "New fact"));

        // e1 should be superseded
        let e1 = graph.resolve("e1").unwrap();
        assert_eq!(e1.content, "New fact"); // resolves to latest

        // e2 should be current
        let e2 = graph.resolve("e2").unwrap();
        assert_eq!(e2.content, "New fact");
    }

    #[test]
    fn test_causal_chain() {
        let mut graph = CognitiveEventGraph::new();
        graph.add_event(make_event("e1", EventType::Fact, "Observation"));
        graph.add_event(make_event("e2", EventType::Inference, "Inference from e1"));
        graph.add_edge(CognitiveEdge {
            source: "e2".to_string(),
            target: "e1".to_string(),
            edge_type: EdgeType::InferredFrom,
            weight: 1.0,
        });

        let chain = graph.causal_chain("e2");
        assert_eq!(chain.len(), 2);
    }
}

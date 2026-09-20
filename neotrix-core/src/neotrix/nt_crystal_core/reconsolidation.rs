//! ReconsolidationEngine — 检索驱动记忆重组
//!
//! 基于 REALM (2026) 的核心洞察：检索不是终点，而是记忆进化的驱动力。
//!
//! 闭环: 检索质量 → 图谱进化

use serde::{Deserialize, Serialize};

use super::multi_graph_memory::{MemoryNodeId, MultiGraphMemory};

// ═══════════════════════════════════════════════════════════════
// 重组策略
// ═══════════════════════════════════════════════════════════════

/// 重组策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReconsolidationStrategy {
    /// 边强化: 成功检索的路径加强
    StrengthenPath { factor: f64 },
    /// 结构发现: 检索中发现的新关系
    DiscoverStructure { min_confidence: f64 },
    /// 冲突消解: 检索中发现的矛盾
    ResolveConflict { policy: ConflictPolicy },
    /// 节点分裂: 一个节点分化为多个
    SplitNode { threshold: f64 },
    /// 节点合并: 多个相似节点合并
    MergeNodes { similarity_threshold: f64 },
}

/// 冲突消解策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictPolicy {
    /// 保留最新的
    KeepLatest,
    /// 保留置信度最高的
    KeepHighestConfidence,
    /// 保留被引用最多的
    KeepMostReferenced,
    /// 人工决策
    ManualDecision,
}

// ═══════════════════════════════════════════════════════════════
// 重组事件
// ═══════════════════════════════════════════════════════════════

/// 检索反馈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalFeedback {
    /// 检索质量分数 (0-1)
    pub quality_score: f64,
    /// 用户是否满意
    pub user_satisfied: bool,
    /// 证据链是否完整
    pub evidence_complete: bool,
    /// 是否发现冲突
    pub conflicts_found: bool,
}

/// 图谱变异操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphMutation {
    /// 添加边
    AddEdge {
        source: MemoryNodeId,
        target: MemoryNodeId,
        edge_type: String,
    },
    /// 移除边
    RemoveEdge {
        source: MemoryNodeId,
        target: MemoryNodeId,
    },
    /// 更新边强度
    UpdateEdgeStrength {
        source: MemoryNodeId,
        target: MemoryNodeId,
        new_strength: f64,
    },
    /// 分裂节点
    SplitNode {
        node_id: MemoryNodeId,
        new_node_ids: Vec<MemoryNodeId>,
    },
    /// 合并节点
    MergeNodes {
        node_ids: Vec<MemoryNodeId>,
        merged_id: MemoryNodeId,
    },
}

/// 重组事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconsolidationEvent {
    /// 原始查询
    pub query: String,
    /// 检索到的节点 ID
    pub retrieved_ids: Vec<MemoryNodeId>,
    /// 反馈信号
    pub feedback: RetrievalFeedback,
    /// 图谱变异
    pub changes: Vec<GraphMutation>,
    /// 时间戳
    pub timestamp: u64,
    /// 使用的策略
    pub strategy: ReconsolidationStrategy,
}

// ═══════════════════════════════════════════════════════════════
// 重组引擎
// ═══════════════════════════════════════════════════════════════

/// 检索驱动记忆重组引擎
pub struct ReconsolidationEngine {
    /// 重组历史
    pub history: Vec<ReconsolidationEvent>,
    /// 默认策略
    pub default_strategy: ReconsolidationStrategy,
    /// 重组阈值 (低于此质量分数触发重组)
    pub reconsolidation_threshold: f64,
}

impl ReconsolidationEngine {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            default_strategy: ReconsolidationStrategy::StrengthenPath { factor: 0.1 },
            reconsolidation_threshold: 0.5,
        }
    }

    /// 执行重组
    pub fn reconsolidate(
        &mut self,
        query: &str,
        retrieved_ids: &[MemoryNodeId],
        feedback: &RetrievalFeedback,
        graph: &mut MultiGraphMemory,
    ) -> ReconsolidationEvent {
        let mut changes = Vec::new();

        // 根据反馈选择策略
        let strategy = if feedback.conflicts_found {
            ReconsolidationStrategy::ResolveConflict {
                policy: ConflictPolicy::KeepLatest,
            }
        } else if feedback.quality_score < self.reconsolidation_threshold {
            ReconsolidationStrategy::DiscoverStructure {
                min_confidence: 0.6,
            }
        } else {
            self.default_strategy.clone()
        };

        match &strategy {
            ReconsolidationStrategy::StrengthenPath { factor } => {
                // 强化检索到的节点之间的边
                for i in 0..retrieved_ids.len() {
                    for j in (i + 1)..retrieved_ids.len() {
                        // 查找并强化现有边
                        let mut found = false;
                        for edge in &mut graph.semantic.edges {
                            if (edge.0 == retrieved_ids[i] && edge.1 == retrieved_ids[j])
                                || (edge.0 == retrieved_ids[j] && edge.1 == retrieved_ids[i])
                            {
                                edge.2.strength += factor;
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            // 创建新边
                            graph.semantic.add_edge(
                                &retrieved_ids[i],
                                &retrieved_ids[j],
                                super::multi_graph_memory::SemanticEdge {
                                    similarity: 0.5,
                                    strength: *factor,
                                },
                            );
                        }
                        changes.push(GraphMutation::UpdateEdgeStrength {
                            source: retrieved_ids[i].clone(),
                            target: retrieved_ids[j].clone(),
                            new_strength: factor + 0.5,
                        });
                    }
                }
            }
            ReconsolidationStrategy::DiscoverStructure { .. } => {
                // 发现新关系 — 在检索结果中寻找未连接的节点对
                for i in 0..retrieved_ids.len() {
                    for j in (i + 1)..retrieved_ids.len() {
                        let has_edge = graph.semantic.edges.iter().any(|(s, t, _)| {
                            (s == &retrieved_ids[i] && t == &retrieved_ids[j])
                                || (s == &retrieved_ids[j] && t == &retrieved_ids[i])
                        });

                        if !has_edge {
                            // 检查语义相似度
                            if let (Some(item_a), Some(item_b)) = (
                                graph.semantic.nodes.get(&retrieved_ids[i]),
                                graph.semantic.nodes.get(&retrieved_ids[j]),
                            ) {
                                let sim = cosine_similarity(&item_a.embedding, &item_b.embedding);
                                if sim > 0.5 {
                                    graph.semantic.add_edge(
                                        &retrieved_ids[i],
                                        &retrieved_ids[j],
                                        super::multi_graph_memory::SemanticEdge {
                                            similarity: sim,
                                            strength: sim * 0.3,
                                        },
                                    );
                                    changes.push(GraphMutation::AddEdge {
                                        source: retrieved_ids[i].clone(),
                                        target: retrieved_ids[j].clone(),
                                        edge_type: "semantic_discovered".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            ReconsolidationStrategy::MergeNodes {
                similarity_threshold,
            } => {
                // 合并高相似度节点
                let threshold = *similarity_threshold;
                let mut to_merge: Vec<(MemoryNodeId, MemoryNodeId)> = Vec::new();
                for i in 0..retrieved_ids.len() {
                    for j in (i + 1)..retrieved_ids.len() {
                        if let (Some(item_a), Some(item_b)) = (
                            graph.semantic.nodes.get(&retrieved_ids[i]),
                            graph.semantic.nodes.get(&retrieved_ids[j]),
                        ) {
                            let sim = cosine_similarity(&item_a.embedding, &item_b.embedding);
                            if sim > threshold {
                                to_merge.push((retrieved_ids[i].clone(), retrieved_ids[j].clone()));
                            }
                        }
                    }
                }
                // 实际合并由调用方处理
                for (a, b) in to_merge {
                    changes.push(GraphMutation::MergeNodes {
                        node_ids: vec![a, b],
                        merged_id: MemoryNodeId::new(),
                    });
                }
            }
            _ => {}
        }

        let event = ReconsolidationEvent {
            query: query.to_string(),
            retrieved_ids: retrieved_ids.to_vec(),
            feedback: feedback.clone(),
            changes,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            strategy,
        };

        self.history.push(event.clone());
        event
    }

    /// 获取最近 N 次重组事件
    pub fn recent_events(&self, n: usize) -> &[ReconsolidationEvent] {
        let start = self.history.len().saturating_sub(n);
        &self.history[start..]
    }

    /// 获取重组统计
    pub fn stats(&self) -> ReconsolidationStats {
        let total_mutations: usize = self.history.iter().map(|e| e.changes.len()).sum();
        let avg_quality: f64 = if self.history.is_empty() {
            0.0
        } else {
            self.history.iter().map(|e| e.feedback.quality_score).sum::<f64>()
                / self.history.len() as f64
        };

        ReconsolidationStats {
            total_reconsolidations: self.history.len(),
            total_mutations,
            avg_quality_score: avg_quality,
        }
    }
}

/// 重组统计
#[derive(Debug, Clone)]
pub struct ReconsolidationStats {
    pub total_reconsolidations: usize,
    pub total_mutations: usize,
    pub avg_quality_score: f64,
}

// ═══════════════════════════════════════════════════════════════
// 工具函数
// ═══════════════════════════════════════════════════════════════

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_crystal_core::multi_graph_memory::{MemoryItem, MemoryType};
    use std::collections::HashMap;

    fn make_item(content: &str, ts: u64) -> MemoryItem {
        let embedding: Vec<f64> = (0..8).map(|i| (ts as f64 + i as f64).sin()).collect();
        MemoryItem {
            id: MemoryNodeId::new(),
            content: content.to_string(),
            embedding,
            timestamp: ts,
            metadata: HashMap::new(),
            entities: Vec::new(),
            memory_type: MemoryType::Episodic,
        }
    }

    #[test]
    fn test_reconsolidation_strengthen() {
        let mut graph = MultiGraphMemory::new();
        let item1 = make_item("Rust is fast", 1000);
        let item2 = make_item("Rust is safe", 2000);
        let id1 = graph.add(item1);
        let id2 = graph.add(item2);

        let mut engine = ReconsolidationEngine::new();
        let feedback = RetrievalFeedback {
            quality_score: 0.8,
            user_satisfied: true,
            evidence_complete: true,
            conflicts_found: false,
        };

        let event = engine.reconsolidate("Rust", &[id1, id2], &feedback, &mut graph);
        assert!(!event.changes.is_empty());
        assert_eq!(engine.history.len(), 1);
    }

    #[test]
    fn test_reconsolidation_discover() {
        let mut graph = MultiGraphMemory::new();
        let item1 = make_item("Rust ownership", 1000);
        let item2 = make_item("Rust borrowing", 2000);
        let id1 = graph.add(item1);
        let id2 = graph.add(item2);

        let mut engine = ReconsolidationEngine::new();
        let feedback = RetrievalFeedback {
            quality_score: 0.3,
            user_satisfied: false,
            evidence_complete: false,
            conflicts_found: false,
        };

        let event = engine.reconsolidate("ownership", &[id1, id2], &feedback, &mut graph);
        // 应该发现新关系
        assert!(!event.changes.is_empty());
    }
}

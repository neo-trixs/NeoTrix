/// KnowledgeNode — 意识体自我理解的知识结构
///
/// 从进化档案 + 执行迹 + 设计标记中蒸馏的结构化知识：
/// - **PrincipleNode**: 设计原则（"不要随机变异，要反射式分析"）
/// - **PatternNode**: 可重复模式（"ECE 上升通常跟随 meta-accuracy 下降"）
/// - **DecisionNode**: 决策记录（"为什么选 Wave A 先于 Wave B"）
/// - **AntipatternNode**: 反模式（"空数据源的死代码是最危险的"）
///
/// 所有节点通过语义关系互连。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 知识节点类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    /// 设计原则 — 因果性的"如果X则Y"陈述
    Principle,
    /// 可重复模式 — 从执行迹中统计发现的相关性
    Pattern,
    /// 决策记录 — 特定情境下做出的选择
    Decision,
    /// 反模式 — 已被识别的有害模式
    Antipattern,
    /// 设计令牌 — 从 DESIGN.md / 设计系统提取的单一设计决策
    DesignToken,
    /// 布局模式 — 可复用的 UI 布局 + 间距风格
    LayoutPattern,
    /// 组件规范 — 具有视觉变体的完整组件定义
    ComponentSpec,
}

impl NodeType {
    pub fn name(&self) -> &'static str {
        match self {
            NodeType::Principle => "principle",
            NodeType::Pattern => "pattern",
            NodeType::Decision => "decision",
            NodeType::Antipattern => "antipattern",
            NodeType::DesignToken => "design_token",
            NodeType::LayoutPattern => "layout_pattern",
            NodeType::ComponentSpec => "component_spec",
        }
    }
}

/// 语义关系类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// A 是 B 的前置条件
    Precedes,
    /// A 是 B 的延迟后果
    Follows,
    /// A 替换/升级了 B
    Supersedes,
    /// A 与 B 矛盾
    Contradicts,
    /// A 提供了 B 的证据
    Supports,
    /// A 为 B 提供了反证
    Refutes,
}

impl RelationType {
    pub fn name(&self) -> &'static str {
        match self {
            RelationType::Precedes => "precedes",
            RelationType::Follows => "follows",
            RelationType::Supersedes => "supersedes",
            RelationType::Contradicts => "contradicts",
            RelationType::Supports => "supports",
            RelationType::Refutes => "refutes",
        }
    }
}

/// 两个知识节点之间的语义边。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub source_id: u64,
    pub target_id: u64,
    pub relation: RelationType,
    pub weight: f64,
    pub discovered_at: u64,
}

/// 单个知识节点。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: u64,
    pub node_type: NodeType,
    /// 核心陈述（一句话原则/模式/决策/反模式）
    pub title: String,
    /// 详细描述
    pub description: String,
    /// 置信度 (0.0 ~ 1.0)
    pub confidence: f64,
    /// 证据计数（支撑该节点的实例数量）
    pub evidence_count: u64,
    /// 反例计数
    pub counterexample_count: u64,
    /// 创建时的 cycle
    pub created_at_cycle: u64,
    /// 最后验证 cycle
    pub last_validated_cycle: u64,
    /// 引用来源节点 ID（哪些执行迹/经验节点支持此知识）
    pub source_ids: Vec<u64>,
    /// 关联的设计基元
    pub related_primitives: Vec<String>,
}

/// 知识图谱 — 所有 KnowledgeNode 的带索引存储。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
    next_id: u64,
    /// 按 type 索引
    by_type: HashMap<NodeType, Vec<usize>>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            next_id: 1,
            by_type: HashMap::new(),
        }
    }

    /// 添加一个知识节点。
    pub fn add_node(
        &mut self,
        node_type: NodeType,
        title: String,
        description: String,
        confidence: f64,
        source_ids: Vec<u64>,
        related_primitives: Vec<String>,
        cycle: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let idx = self.nodes.len();
        self.nodes.push(KnowledgeNode {
            id,
            node_type,
            title,
            description,
            confidence,
            evidence_count: 1,
            counterexample_count: 0,
            created_at_cycle: cycle,
            last_validated_cycle: cycle,
            source_ids,
            related_primitives,
        });
        self.by_type.entry(node_type).or_default().push(idx);
        id
    }

    /// 在两个节点之间添加语义边。
    pub fn add_edge(
        &mut self,
        source_id: u64,
        target_id: u64,
        relation: RelationType,
        weight: f64,
        cycle: u64,
    ) {
        self.edges.push(KnowledgeEdge {
            source_id,
            target_id,
            relation,
            weight,
            discovered_at: cycle,
        });
    }

    /// 按类型查找所有节点。
    pub fn find_by_type(&self, node_type: NodeType) -> Vec<&KnowledgeNode> {
        self.by_type
            .get(&node_type)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.nodes.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 通过 ID 查找节点。
    pub fn find_by_id(&self, id: u64) -> Option<&KnowledgeNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// 查找从 source 节点出发的所有出边。
    pub fn outgoing_edges(&self, source_id: u64) -> Vec<&KnowledgeEdge> {
        self.edges.iter().filter(|e| e.source_id == source_id).collect()
    }

    /// 查找指向 target 节点的所有入边。
    pub fn incoming_edges(&self, target_id: u64) -> Vec<&KnowledgeEdge> {
        self.edges.iter().filter(|e| e.target_id == target_id).collect()
    }

    /// 增加证据计数（当相同模式再次出现时）。
    pub fn add_evidence(&mut self, id: u64, cycle: u64) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.evidence_count += 1;
            node.last_validated_cycle = cycle;
            node.confidence = (node.confidence + 0.05).min(1.0);
        }
    }

    /// 增加反例计数。
    pub fn add_counterexample(&mut self, id: u64, cycle: u64) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.counterexample_count += 1;
            node.last_validated_cycle = cycle;
            node.confidence = (node.confidence * 0.7).max(0.05);
        }
    }

    /// 高置信度原则（>0.7 置信度）。
    pub fn high_confidence_principles(&self) -> Vec<&KnowledgeNode> {
        self.find_by_type(NodeType::Principle)
            .into_iter()
            .filter(|n| n.confidence > 0.7)
            .collect()
    }

    /// 获取统计摘要。
    pub fn summary(&self) -> String {
        let principles = self.find_by_type(NodeType::Principle).len();
        let patterns = self.find_by_type(NodeType::Pattern).len();
        let decisions = self.find_by_type(NodeType::Decision).len();
        let antipatterns = self.find_by_type(NodeType::Antipattern).len();
        format!(
            "graph: {} nodes ({}P + {}Pa + {}D + {}A), {} edges, next_id={}",
            self.nodes.len(),
            principles, patterns, decisions, antipatterns,
            self.edges.len(),
            self.next_id,
        )
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_find_principles() {
        let mut g = KnowledgeGraph::new();
        let id = g.add_node(
            NodeType::Principle,
            "反射式变异优于随机变异".into(),
            "GEPA 证明基于执行迹的定向修复远胜随机参数搜索".into(),
            0.85,
            vec![],
            vec!["mcts".into(), "reflect".into()],
            100,
        );
        assert!(id > 0);
        let principles = g.find_by_type(NodeType::Principle);
        assert_eq!(principles.len(), 1);
        assert_eq!(principles[0].title, "反射式变异优于随机变异");
    }

    #[test]
    fn test_edges() {
        let mut g = KnowledgeGraph::new();
        let a = g.add_node(NodeType::Principle, "原则A".into(), "".into(), 0.8, vec![], vec![], 1);
        let b = g.add_node(NodeType::Pattern, "模式B".into(), "".into(), 0.6, vec![], vec![], 2);
        g.add_edge(a, b, RelationType::Supports, 0.9, 3);
        assert_eq!(g.outgoing_edges(a).len(), 1);
        assert_eq!(g.incoming_edges(b).len(), 1);
    }

    #[test]
    fn test_evidence_increases_confidence() {
        let mut g = KnowledgeGraph::new();
        let id = g.add_node(NodeType::Principle, "test".into(), "".into(), 0.5, vec![], vec![], 1);
        g.add_evidence(id, 2);
        let node = g.find_by_id(id).unwrap();
        assert_eq!(node.evidence_count, 1);
        assert!((node.confidence - 0.55).abs() < 1e-6);
    }

    #[test]
    fn test_counterexample_decreases_confidence() {
        let mut g = KnowledgeGraph::new();
        let id = g.add_node(NodeType::Antipattern, "bad".into(), "".into(), 0.8, vec![], vec![], 1);
        g.add_counterexample(id, 2);
        let node = g.find_by_id(id).unwrap();
        assert!((node.confidence - 0.56).abs() < 1e-6); // 0.8 * 0.7
    }

    #[test]
    fn test_high_confidence_principles_filter() {
        let mut g = KnowledgeGraph::new();
        g.add_node(NodeType::Principle, "high".into(), "".into(), 0.9, vec![], vec![], 1);
        g.add_node(NodeType::Principle, "low".into(), "".into(), 0.5, vec![], vec![], 2);
        assert_eq!(g.high_confidence_principles().len(), 1);
    }

    #[test]
    fn test_empty_graph_summary() {
        let g = KnowledgeGraph::new();
        assert!(g.summary().contains("0 nodes"));
    }

    #[test]
    fn test_node_type_names() {
        assert_eq!(NodeType::Principle.name(), "principle");
        assert_eq!(NodeType::Antipattern.name(), "antipattern");
        assert_eq!(RelationType::Precedes.name(), "precedes");
        assert_eq!(RelationType::Contradicts.name(), "contradicts");
    }
}

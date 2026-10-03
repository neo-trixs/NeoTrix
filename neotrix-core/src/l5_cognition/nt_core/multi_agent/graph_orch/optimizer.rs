//! DAG Optimizer — analyzes and transforms DAGs for better execution.
//!
//! Actions: merge sequential single-task nodes, identify critical path,
//! suggest parallelization opportunities.
//! Follows R-P126 (graph-based scheduling optimization).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::dag::{Dag, DagEdge, DagNode, EdgeType, NodeType};

/// Result of a critical path analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPath {
    /// Ordered node ids on the critical path
    pub path: Vec<String>,
    /// Total depth (number of sequential hops)
    pub total_depth: u32,
}

/// A suggestion for parallelizing independent nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelizationSuggestion {
    /// Nodes that can run in parallel
    pub parallel_group: Vec<String>,
    /// Reason for the suggestion
    pub reason: String,
}

/// Optimizer that analyzes and transforms DAGs for improved scheduling.
pub struct DagOptimizer;

impl DagOptimizer {
    /// Optimize a DAG in place: merge sequential single-task nodes,
    /// then return suggestions for parallelization.
    ///
    /// Returns the critical path and parallelization suggestions.
    pub fn optimize(
        dag: &mut Dag,
    ) -> Result<(CriticalPath, Vec<ParallelizationSuggestion>), String> {
        Self::merge_sequential_single_task_nodes(dag)?;
        let critical = Self::identify_critical_path(dag);
        let suggestions = Self::suggest_parallelization(dag);
        Ok((critical, suggestions))
    }

    /// Merge chains of Sequential single-Task nodes into single nodes.
    ///
    /// If node A -> Sequential -> node B, and both are Task type with
    /// no branching, they can be merged. Merged node id = "merged_{first}_{last}".
    pub fn merge_sequential_single_task_nodes(dag: &mut Dag) -> Result<(), String> {
        // ⚠️ 2026-10-02 修正（**真实缺陷**，由 177 个从未运行的测试抓出）：
        // 原实现**一次性**算出全部候选，再依次执行：
        //     let candidates = find_merge_candidates(dag);   // => [(a,b), (b,c)]
        //     for (from, to) in candidates { merge_pair(dag, &from, &to)?; }
        // 但每次合并都会**使后续候选失效** —— 合并 (a,b) 后 `b` 已被吸收，
        // 候选 (b,c) 指向一个不存在的节点。
        // ⇒ 线性链 a→b→c 只能合并掉一半（实测 `node_count()` 得 2，期望 1），
        //   并连带让 `parallelization_no_suggestions_for_linear` 失败
        //   （残留下来的节点落在同一 depth，被误判为「可并行」）。
        //
        // 修法：**每合并一对就重算候选**，而不是把候选表当快照用。
        loop {
            let Some((from_id, to_id)) = Self::find_merge_candidates(dag).into_iter().next() else {
                break;
            };
            Self::merge_pair(dag, &from_id, &to_id)?;
        }
        Ok(())
    }

    /// Find pairs of nodes that can be merged:
    /// - Both are Task nodes
    /// - Connected by a single Sequential edge
    /// - `from` has exactly one outgoing edge
    /// - `to` has exactly one incoming edge
    fn find_merge_candidates(dag: &Dag) -> Vec<(String, String)> {
        let mut candidates = Vec::new();

        for edge in &dag.edges {
            if edge.edge_type != EdgeType::Sequential {
                continue;
            }

            let from_node = match dag.get_node(&edge.from) {
                Some(n) => n,
                None => continue,
            };
            let to_node = match dag.get_node(&edge.to) {
                Some(n) => n,
                None => continue,
            };

            if from_node.node_type != NodeType::Task || to_node.node_type != NodeType::Task {
                continue;
            }

            // from must have exactly one outgoing edge
            if dag.outgoing_edges(&edge.from).len() != 1 {
                continue;
            }
            // to must have exactly one incoming edge
            if dag.incoming_edges(&edge.to).len() != 1 {
                continue;
            }

            candidates.push((edge.from.clone(), edge.to.clone()));
        }

        candidates
    }

    /// Merge two nodes: combine metadata, rewire edges, remove the second node.
    fn merge_pair(dag: &mut Dag, from_id: &str, to_id: &str) -> Result<(), String> {
        let new_id = format!("merged_{}_{}", from_id, to_id);

        // Build merged metadata
        let mut metadata = HashMap::new();
        if let Some(from_node) = dag.get_node(from_id) {
            metadata.extend(from_node.metadata.clone());
            metadata.insert("merged_from".to_string(), from_id.to_string());
        }
        if let Some(to_node) = dag.get_node(to_id) {
            metadata.extend(to_node.metadata.clone());
            metadata.insert("merged_to".to_string(), to_id.to_string());
        }

        let merged_node = DagNode::new(&new_id, NodeType::Task).with_metadata("merged", "true");
        // Copy merged metadata
        let mut final_node = merged_node;
        for (k, v) in metadata {
            final_node.metadata.insert(k, v);
        }

        dag.add_node(final_node)?;

        // Rewire edges from `from_id` to `new_id` (incoming)
        let incoming_from: Vec<DagEdge> = dag
            .incoming_edges(from_id)
            .into_iter()
            .map(|e| DagEdge::new(&e.from, &new_id, e.edge_type))
            .collect();
        // Rewire edges from `to_id` from `new_id` (outgoing)
        let outgoing_to: Vec<DagEdge> = dag
            .outgoing_edges(to_id)
            .into_iter()
            .map(|e| DagEdge::new(&new_id, &e.to, e.edge_type))
            .collect();

        // Remove old nodes and their edges
        dag.nodes.remove(from_id);
        dag.nodes.remove(to_id);
        dag.edges
            .retain(|e| e.from != from_id && e.to != from_id && e.from != to_id && e.to != to_id);

        // Add re-wired edges
        for e in incoming_from {
            let _ = dag.add_edge(e);
        }
        for e in outgoing_to {
            let _ = dag.add_edge(e);
        }

        Ok(())
    }

    /// Identify the critical path (longest path through the DAG).
    ///
    /// Uses dynamic programming on the topological order.
    pub fn identify_critical_path(dag: &Dag) -> CriticalPath {
        let order = match dag.topological_sort() {
            Ok(o) => o,
            Err(_) => {
                return CriticalPath {
                    path: Vec::new(),
                    total_depth: 0,
                }
            }
        };

        // dist[node] = longest path from any root to this node
        let mut dist: HashMap<&str, u32> = HashMap::new();
        let mut predecessor: HashMap<&str, Option<&str>> = HashMap::new();

        for id in &order {
            let max_incoming = dag
                .incoming_edges(id)
                .iter()
                .filter(|e| {
                    // ⚠️ 2026-10-02：同 `scheduler.rs` 的修正 ——
                    // `Parallel` 也是**依赖边**（其自述「both branches execute
                    // concurrently」指 b/c 相互并发，不是「b 不必等 a」）。
                    // 只认 Sequential 会把 Parallel 分支算成同层。
                    // 排除 `Conditional`（可能不执行，不能作顺序约束）。
                    matches!(
                        e.edge_type,
                        EdgeType::Sequential | EdgeType::Parallel
                    )
                })
                .filter_map(|e| {
                    let d = dist.get(e.from.as_str())?;
                    Some((d + 1, e.from.as_str()))
                })
                .max_by_key(|(d, _)| *d);

            match max_incoming {
                Some((d, pred)) => {
                    dist.insert(id.as_str(), d);
                    predecessor.insert(id.as_str(), Some(pred));
                }
                None => {
                    dist.insert(id.as_str(), 0);
                    predecessor.insert(id.as_str(), None);
                }
            }
        }

        // Find the node with maximum distance
        //
        // ⚠️ 2026-10-02 修正（**真实缺陷**，由 177 个从未运行的测试抓出）：
        // 原实现 `.unwrap_or("")` 用**空串当哨兵**。空 DAG 时 `dist` 为空 ⇒
        // `end_node = ""` ⇒ 下面回溯循环 `current = Some("")` 仍会执行一次
        // ⇒ **把 `""` 压进 `path`** ⇒ 返回 `path == [""]`（长度 1）而非空。
        // ⇒ 任何调用方 `path.first()` 会拿到 `Some("")` —— 一个**幽灵节点**。
        // 修法：空 DAG 直接返回空路径，不引入哨兵。
        let Some(end_node) = dist.iter().max_by_key(|(_, &d)| d).map(|(&id, _)| id) else {
            return CriticalPath {
                path: Vec::new(),
                total_depth: 0,
            };
        };

        // Trace back
        let mut path = Vec::new();
        let mut current = Some(end_node);
        while let Some(node) = current {
            path.push(node.to_string());
            current = predecessor.get(node).and_then(|p| *p);
        }
        path.reverse();

        let total_depth = dist.get(end_node).copied().unwrap_or(0);

        CriticalPath { path, total_depth }
    }

    /// Suggest parallelization opportunities.
    ///
    /// Finds nodes at the same depth with no data dependency between them.
    pub fn suggest_parallelization(dag: &Dag) -> Vec<ParallelizationSuggestion> {
        let order = match dag.topological_sort() {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        // Compute depth
        let mut depth: HashMap<&str, u32> = HashMap::new();
        for id in &order {
            let max_pred = dag
                .incoming_edges(id)
                .iter()
                .filter(|e| {
                    // ⚠️ 2026-10-02：同 `scheduler.rs` 的修正 ——
                    // `Parallel` 也是**依赖边**（其自述「both branches execute
                    // concurrently」指 b/c 相互并发，不是「b 不必等 a」）。
                    // 只认 Sequential 会把 Parallel 分支算成同层。
                    // 排除 `Conditional`（可能不执行，不能作顺序约束）。
                    matches!(
                        e.edge_type,
                        EdgeType::Sequential | EdgeType::Parallel
                    )
                })
                .filter_map(|e| depth.get(e.from.as_str()))
                .copied()
                .max();
            depth.insert(id.as_str(), max_pred.map_or(0, |d| d + 1));
        }

        // Group by depth
        let mut by_depth: HashMap<u32, Vec<&str>> = HashMap::new();
        for id in &order {
            let d = depth[id.as_str()];
            by_depth.entry(d).or_default().push(id);
        }

        let mut suggestions = Vec::new();
        for (d, nodes) in &by_depth {
            if nodes.len() > 1 {
                suggestions.push(ParallelizationSuggestion {
                    parallel_group: nodes.iter().map(|s| s.to_string()).collect(),
                    reason: format!(
                        "These {} nodes are at depth {} with no dependencies between them",
                        nodes.len(),
                        d
                    ),
                });
            }
        }

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::super::dag::{DagEdge, DagNode, EdgeType, NodeType};
    use super::*;

    fn linear_task_dag() -> Dag {
        let mut dag = Dag::new("linear");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "c", EdgeType::Sequential))
            .unwrap();
        dag
    }

    #[test]
    fn merge_sequential_tasks() {
        let mut dag = linear_task_dag();
        DagOptimizer::merge_sequential_single_task_nodes(&mut dag).unwrap();
        // All three should merge into one
        assert_eq!(dag.node_count(), 1);
        let merged_id = dag.nodes.keys().next().unwrap();
        assert!(merged_id.starts_with("merged_"));
    }

    #[test]
    fn merge_preserves_non_task_nodes() {
        let mut dag = Dag::new("mixed");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("decision", NodeType::Decision))
            .unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_edge(DagEdge::new("a", "decision", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("decision", "b", EdgeType::Sequential))
            .unwrap();

        DagOptimizer::merge_sequential_single_task_nodes(&mut dag).unwrap();
        // Decision node prevents merging
        assert_eq!(dag.node_count(), 3);
    }

    #[test]
    fn critical_path_linear() {
        let dag = linear_task_dag();
        let cp = DagOptimizer::identify_critical_path(&dag);
        assert_eq!(cp.path, vec!["a", "b", "c"]);
        assert_eq!(cp.total_depth, 2);
    }

    #[test]
    fn critical_path_diamond() {
        let mut dag = Dag::new("diamond");
        dag.add_node(DagNode::new("a", NodeType::Split)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("d", NodeType::Join)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Parallel))
            .unwrap();
        dag.add_edge(DagEdge::new("a", "c", EdgeType::Parallel))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "d", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("c", "d", EdgeType::Sequential))
            .unwrap();

        let cp = DagOptimizer::identify_critical_path(&dag);
        assert!(cp.path.contains(&"a".to_string()));
        assert!(cp.path.contains(&"d".to_string()));
    }

    #[test]
    fn suggest_parallelization_diamond() {
        let mut dag = Dag::new("diamond");
        dag.add_node(DagNode::new("a", NodeType::Split)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("d", NodeType::Join)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("a", "c", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "d", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("c", "d", EdgeType::Sequential))
            .unwrap();

        let suggestions = DagOptimizer::suggest_parallelization(&dag);
        assert!(!suggestions.is_empty());
        // b and c should be in a parallel group
        let group: Vec<&str> = suggestions[0]
            .parallel_group
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert!(group.contains(&"b"));
        assert!(group.contains(&"c"));
    }

    #[test]
    fn optimize_full_pipeline() {
        let mut dag = linear_task_dag();
        let (cp, suggestions) = DagOptimizer::optimize(&mut dag).unwrap();
        // After merge, critical path is the single merged node
        assert_eq!(cp.path.len(), 1);
        assert!(suggestions.is_empty()); // single node, no parallelization
    }

    #[test]
    fn critical_path_empty_dag() {
        let dag = Dag::new("empty");
        let cp = DagOptimizer::identify_critical_path(&dag);
        assert!(cp.path.is_empty());
        assert_eq!(cp.total_depth, 0);
    }

    #[test]
    fn parallelization_single_node_no_suggestion() {
        let mut dag = Dag::new("single");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        let suggestions = DagOptimizer::suggest_parallelization(&dag);
        assert!(suggestions.is_empty());
    }

    /// ✅ 2026-10-02 已裁决并重写（原先与 `merge_sequential_tasks` 断言相反）。
    ///
    /// **矛盾根源**：原夹具是 a→b→c（全 `Task`、全 `Sequential`），
    /// 却期望合并后 `merged` **仍有出边指向 c** ⇒ 要求 c **未**被合并；
    /// 而 `merge_sequential_tasks` 对**同一 DAG** 要求 `node_count() == 1`
    /// ⇒ 要求 c **已**被合并。两者不可同时满足。
    ///
    /// **裁决**：`merge_pair`（:128）把合并节点建为 `NodeType::Task`
    /// ⇒ 合并后**仍可继续合并** ⇒ 线性链**完全折叠**是本函数的契约
    ///（这也正是 `parallelization_no_suggestions_for_linear` 依赖的行为）。
    /// ⇒ 2 : 1 多数方。
    ///
    /// **重写而非忽略**：本测试的名字是「出边被保留」，而该意图
    /// **确实被实现满足** —— `merge_pair`(:138-144) 把 `to_id` 的出边
    /// **重连**到新节点，而非丢弃。原夹具只是恰好让链折完、没留下出边。
    /// ⇒ 现把 `c` 改为 `NodeType::Join`（**不可合并**），
    ///    链停在 2 个节点 ⇒ 这样它才**真正测到出边被重连**。
    #[test]    fn merge_preserves_outgoing_edges() {
        let mut dag = Dag::new("merge-out");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        // ⚠️ 2026-10-02：`c` 改为 **Join**（原为 Task）。
        // 原为 Task 时 a→b→c 会**整链折叠**成一个节点，于是「出边还在」
        // 根本无法成立 —— 那是夹具选错，不是实现丢失出边。
        // 改为 Join 后 `c` 不可合并 ⇒ 链停在 2 个节点 ⇒
        // 本测试才真正验证「`to_id` 的出边被**重连**到 merged 节点」。
                // ⚠️ 2026-10-02：`c` 改为 **Join**（原为 Task）。
        // 原为 Task 时 a→b→c 会**整链折叠**成一个节点，于是「出边还在」
        // 根本无法成立 —— 那是夹具选错，不是实现丢失出边。
        // 改为 Join 后 `c` 不可合并 ⇒ 链停在 2 个节点 ⇒
        // 本测试才真正验证「`to_id` 的出边被**重连**到 merged 节点」。
        dag.add_node(DagNode::new("c", NodeType::Join)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "c", EdgeType::Sequential))
            .unwrap();

        DagOptimizer::merge_sequential_single_task_nodes(&mut dag).unwrap();
        assert_eq!(dag.node_count(), 2, "c 为 Join 不可合并 ⇒ 链应停在 2 个节点");

        // ⚠️ 2026-10-02 修正：原代码用 `dag.nodes.keys().next().unwrap()`
        // 取「第一个」节点当 merged —— 但 `nodes` 是 **HashMap，迭代顺序不确定**，
        // 合并后有 2 个节点（merged_ab 与 c）⇒ 取到 `c` 就得到 0 条出边。
        // ⇒ 这**又是一个「顺序不确定」型缺陷**（本会话第 N 次：
        //   `nt_nondet.py` 扫的就是这一类）。
        // 修法：按**语义**定位 merged 节点（id 以 `merged_` 开头），
        // 而不是依赖 HashMap 顺序。
        let merged_id = dag
            .nodes
            .keys()
            .find(|id| id.starts_with("merged_"))
            .expect("应存在 merged_ 前缀的合并节点");
        let out = dag.outgoing_edges(merged_id);
        assert_eq!(out.len(), 1, "merged 节点应保留指向 c 的出边");
        assert_eq!(out[0].to, "c");
    }

    #[test]
    fn merge_no_mergeable_pairs() {
        let mut dag = Dag::new("no-merge");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Decision))
            .unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        DagOptimizer::merge_sequential_single_task_nodes(&mut dag).unwrap();
        assert_eq!(dag.node_count(), 2);
    }

    #[test]
    fn critical_path_single_node() {
        let mut dag = Dag::new("single");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        let cp = DagOptimizer::identify_critical_path(&dag);
        assert_eq!(cp.path, vec!["a"]);
        assert_eq!(cp.total_depth, 0);
    }

    #[test]
    fn optimize_diamond_preserves_structure() {
        let mut dag = Dag::new("diamond");
        dag.add_node(DagNode::new("a", NodeType::Split)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("d", NodeType::Join)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("a", "c", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("b", "d", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("c", "d", EdgeType::Sequential))
            .unwrap();

        let (cp, suggestions) = DagOptimizer::optimize(&mut dag).unwrap();
        assert!(cp.path.len() >= 2);
        // b and c are at same depth
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn parallelization_no_suggestions_for_linear() {
        let mut dag = linear_task_dag();
        DagOptimizer::merge_sequential_single_task_nodes(&mut dag).unwrap();
        let suggestions = DagOptimizer::suggest_parallelization(&dag);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn critical_path_serialization() {
        let cp = CriticalPath {
            path: vec!["a".into(), "b".into()],
            total_depth: 1,
        };
        let json = serde_json::to_string(&cp).unwrap();
        let back: CriticalPath = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, vec!["a", "b"]);
        assert_eq!(back.total_depth, 1);
    }

    #[test]
    fn parallelization_suggestion_serialization() {
        let s = ParallelizationSuggestion {
            parallel_group: vec!["a".into(), "b".into()],
            reason: "same depth".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ParallelizationSuggestion = serde_json::from_str(&json).unwrap();
        assert_eq!(back.parallel_group.len(), 2);
    }
}

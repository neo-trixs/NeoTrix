//! DAG Scheduler — maps a DAG into a turn-based execution schedule.
//!
//! Respects topological ordering and maximizes parallelism for independent nodes.
//! Follows R-P126 (graph-based scheduling).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::dag::{Dag, EdgeType};

/// A single entry in the execution schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    /// Node id to execute
    pub node_id: String,
    /// Turn number (0-indexed) when this node starts
    pub start_turn: u32,
    /// Turn number when this node completes (inclusive)
    pub end_turn: u32,
    /// Agent handle assigned to execute this node (None = unassigned)
    pub assigned_agent: Option<String>,
}

impl ScheduleEntry {
    pub fn new(node_id: impl Into<String>, start_turn: u32, end_turn: u32) -> Self {
        Self {
            node_id: node_id.into(),
            start_turn,
            end_turn,
            assigned_agent: None,
        }
    }

    pub fn with_agent(mut self, agent: impl Into<String>) -> Self {
        self.assigned_agent = Some(agent.into());
        self
    }

    /// Duration in turns
    pub fn duration(&self) -> u32 {
        self.end_turn - self.start_turn + 1
    }
}

/// Scheduler that turns a DAG into an ordered list of schedule entries.
///
/// Nodes at the same topological depth (independent branches) are placed
/// in the same turn, enabling parallel execution.
pub struct DagScheduler;

impl DagScheduler {
    /// Schedule a DAG into turn-based entries.
    ///
    /// Uses topological sort, then assigns parallel nodes to the same turn.
    /// All edges are treated as Sequential for scheduling; Parallel edges
    /// just mean the nodes are at the same depth.
    pub fn schedule(dag: &Dag) -> Result<Vec<ScheduleEntry>, String> {
        let order = dag.topological_sort()?;

        // Compute depth (longest path from root) for each node
        let mut depth: HashMap<&str, u32> = HashMap::new();
        for id in &order {
            let max_pred_depth = dag
                .incoming_edges(id)
                .iter()
                // ⚠️ 2026-10-02 修正（**真实缺陷**，由 177 个从未运行的测试抓出）：
                // 原代码只认 `Sequential` 边 ⇒ **Parallel 依赖边被忽略**。
                // 而 `EdgeType::Parallel` 的自述是「both branches execute
                // concurrently」—— 说的是**b 与 c 相互并发**，
                // **不是**「b 不必等 a」⇒ a→b 仍是**依赖边**，必须计入深度。
                // 后果（实测）：diamond `a -Parallel-> {b,c} -Sequential-> d`
                //   被算成 depth[a]=0, depth[b]=0, depth[c]=0, depth[d]=1
                //   ⇒ d 被排到 turn 1（期望 turn 2），`makespan` 随之少算 1（2 vs 3）。
                //
                // 计入 `Sequential | Parallel`；**排除 `Conditional`**
                // ——其自述「runs only if condition is met」⇒ 该依赖**可能不发生**，
                // 不能作为顺序约束的依据（否则会过度串行化）。
                .filter(|e| {
                    matches!(
                        e.edge_type,
                        EdgeType::Sequential | EdgeType::Parallel
                    )
                })
                .filter_map(|e| depth.get(e.from.as_str()))
                .copied()
                .max();
            depth.insert(id.as_str(), max_pred_depth.map_or(0, |d| d + 1));
        }

        // Group nodes by depth (turn number)
        let mut turns: HashMap<u32, Vec<&str>> = HashMap::new();
        for id in &order {
            let turn = depth[id.as_str()];
            turns.entry(turn).or_default().push(id);
        }

        let max_turn = turns.keys().copied().max().unwrap_or(0);
        let mut entries = Vec::new();

        for turn in 0..=max_turn {
            if let Some(nodes) = turns.get(&turn) {
                // ⚠️ 2026-10-02 修正（E0277 ×2）：`nodes: &Vec<&str>`，
                // `for x in nodes` 已得 `&&str`，原代码再剥一层写成 `for &&node_id`
                // ⇒ 局部变量成了 `str`（unsized）⇒ 两条错误。
                // 改为只迭代一层，`node_id` 为 `&str`，正好满足 `impl Into<String>`。
                for &node_id in nodes {
                    entries.push(ScheduleEntry::new(node_id, turn, turn));
                }
            }
        }

        Ok(entries)
    }

    /// Compute total makespan (number of turns)
    pub fn makespan(entries: &[ScheduleEntry]) -> u32 {
        entries
            .iter()
            .map(|e| e.end_turn)
            .max()
            .map_or(0, |m| m + 1)
    }

    /// Get entries for a specific turn
    pub fn entries_for_turn(entries: &[ScheduleEntry], turn: u32) -> Vec<&ScheduleEntry> {
        entries.iter().filter(|e| e.start_turn == turn).collect()
    }

    /// Get entries assigned to a specific agent
    pub fn entries_for_agent<'a>(
        entries: &'a [ScheduleEntry],
        agent: &str,
    ) -> Vec<&'a ScheduleEntry> {
        entries
            .iter()
            .filter(|e| e.assigned_agent.as_deref() == Some(agent))
            .collect()
    }

    /// Assign agents round-robin from a list of available agent handles
    pub fn assign_agents(entries: &mut [ScheduleEntry], agents: &[String]) {
        if agents.is_empty() {
            return;
        }
        for (i, entry) in entries.iter_mut().enumerate() {
            entry.assigned_agent = Some(agents[i % agents.len()].clone());
        }
    }
}

#[cfg(test)]
mod tests {
    // ⚠️ 2026-10-02 修正：漏了 `DagEdge` —— 同文件测试里 8 处 `DagEdge::new(..)`
    // 全部无法解析。`optimizer.rs` 的同类导入本就含 DagEdge，`dag.rs` 靠 `use super::*`
    // 拿到 ⇒ **只有本处漏了**。这正是「孤儿测试从未编译」的直接后果：
    // 编译错误从未被任何人看见。
    use super::super::dag::{DagEdge, DagNode, NodeType};
    use super::*;

    fn linear_dag() -> Dag {
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

    fn diamond_dag() -> Dag {
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
        dag
    }

    #[test]
    fn schedule_linear() {
        let dag = linear_dag();
        let entries = DagScheduler::schedule(&dag).unwrap();
        assert_eq!(entries.len(), 3);
        // Each node gets its own turn
        assert_eq!(entries[0].start_turn, 0);
        assert_eq!(entries[1].start_turn, 1);
        assert_eq!(entries[2].start_turn, 2);
    }

    #[test]
    fn schedule_diamond() {
        let dag = diamond_dag();
        let entries = DagScheduler::schedule(&dag).unwrap();
        assert_eq!(entries.len(), 4);

        // a is turn 0
        let a = entries.iter().find(|e| e.node_id == "a").unwrap();
        assert_eq!(a.start_turn, 0);

        // b and c are turn 1 (parallel)
        let b = entries.iter().find(|e| e.node_id == "b").unwrap();
        let c = entries.iter().find(|e| e.node_id == "c").unwrap();
        assert_eq!(b.start_turn, 1);
        assert_eq!(c.start_turn, 1);

        // d is turn 2
        let d = entries.iter().find(|e| e.node_id == "d").unwrap();
        assert_eq!(d.start_turn, 2);
    }

    #[test]
    fn makespan_calculation() {
        let dag = diamond_dag();
        let entries = DagScheduler::schedule(&dag).unwrap();
        assert_eq!(DagScheduler::makespan(&entries), 3);
    }

    #[test]
    fn entries_for_turn_filter() {
        let dag = diamond_dag();
        let entries = DagScheduler::schedule(&dag).unwrap();
        let turn1 = DagScheduler::entries_for_turn(&entries, 1);
        assert_eq!(turn1.len(), 2);
    }

    #[test]
    fn assign_agents_round_robin() {
        let dag = diamond_dag();
        let mut entries = DagScheduler::schedule(&dag).unwrap();
        let agents = vec!["agent-0".to_string(), "agent-1".to_string()];
        DagScheduler::assign_agents(&mut entries, &agents);
        for (i, entry) in entries.iter().enumerate() {
            assert_eq!(
                entry.assigned_agent.as_deref(),
                Some(agents[i % 2].as_str())
            );
        }
    }

    #[test]
    fn assign_agents_empty_list() {
        let dag = diamond_dag();
        let mut entries = DagScheduler::schedule(&dag).unwrap();
        DagScheduler::assign_agents(&mut entries, &[]);
        for entry in &entries {
            assert!(entry.assigned_agent.is_none());
        }
    }

    #[test]
    fn entries_for_agent() {
        let dag = linear_dag();
        let mut entries = DagScheduler::schedule(&dag).unwrap();
        entries[0].assigned_agent = Some("coder".to_string());
        entries[1].assigned_agent = Some("reviewer".to_string());
        entries[2].assigned_agent = Some("coder".to_string());

        let coder_entries = DagScheduler::entries_for_agent(&entries, "coder");
        assert_eq!(coder_entries.len(), 2);

        let reviewer_entries = DagScheduler::entries_for_agent(&entries, "reviewer");
        assert_eq!(reviewer_entries.len(), 1);
    }

    #[test]
    fn schedule_empty_dag() {
        let dag = Dag::new("empty");
        let entries = DagScheduler::schedule(&dag).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn schedule_entry_builder() {
        let entry = ScheduleEntry::new("n1", 0, 2).with_agent("worker-1");
        assert_eq!(entry.node_id, "n1");
        assert_eq!(entry.duration(), 3);
        assert_eq!(entry.assigned_agent.as_deref(), Some("worker-1"));
    }

    #[test]
    fn schedule_entry_serialization_roundtrip() {
        let entry = ScheduleEntry::new("n1", 0, 2).with_agent("agent-1");
        let json = serde_json::to_string(&entry).unwrap();
        let back: ScheduleEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.node_id, "n1");
        assert_eq!(back.start_turn, 0);
    }

    #[test]
    fn schedule_linear_parallelism() {
        // Two independent chains: a->b, c->d
        let mut dag = Dag::new("parallel-chains");
        dag.add_node(DagNode::new("a", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("b", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("c", NodeType::Task)).unwrap();
        dag.add_node(DagNode::new("d", NodeType::Task)).unwrap();
        dag.add_edge(DagEdge::new("a", "b", EdgeType::Sequential))
            .unwrap();
        dag.add_edge(DagEdge::new("c", "d", EdgeType::Sequential))
            .unwrap();

        let entries = DagScheduler::schedule(&dag).unwrap();
        assert_eq!(entries.len(), 4);
        // a and c should be at turn 0
        let turn0 = DagScheduler::entries_for_turn(&entries, 0);
        assert_eq!(turn0.len(), 2);
        // b and d should be at turn 1
        let turn1 = DagScheduler::entries_for_turn(&entries, 1);
        assert_eq!(turn1.len(), 2);
    }

    #[test]
    fn makespan_empty() {
        assert_eq!(DagScheduler::makespan(&[]), 0);
    }

    #[test]
    fn entries_for_turn_none_matching() {
        let dag = linear_dag();
        let entries = DagScheduler::schedule(&dag).unwrap();
        let turn99 = DagScheduler::entries_for_turn(&entries, 99);
        assert!(turn99.is_empty());
    }

    #[test]
    fn entries_for_agent_none_matching() {
        let dag = linear_dag();
        let entries = DagScheduler::schedule(&dag).unwrap();
        let ghost = DagScheduler::entries_for_agent(&entries, "nonexistent");
        assert!(ghost.is_empty());
    }

    #[test]
    fn assign_agents_single() {
        let dag = linear_dag();
        let mut entries = DagScheduler::schedule(&dag).unwrap();
        DagScheduler::assign_agents(&mut entries, &["solo".to_string()]);
        for entry in &entries {
            assert_eq!(entry.assigned_agent.as_deref(), Some("solo"));
        }
    }

    #[test]
    fn schedule_entry_duration() {
        let entry = ScheduleEntry::new("n", 3, 7);
        assert_eq!(entry.duration(), 5); // 7 - 3 + 1
    }
}

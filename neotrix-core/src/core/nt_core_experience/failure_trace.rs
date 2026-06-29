use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TraceNodeType {
    Hypothesis,
    Experiment,
    DeadEnd,
    Success,
    RejectedPath,
    Insight,
}

#[derive(Debug, Clone)]
pub struct TraceNode {
    pub id: u64,
    pub node_type: TraceNodeType,
    pub context: Vec<u8>,
    pub action: Vec<u8>,
    pub outcome: Vec<u8>,
    pub parent_id: Option<u64>,
    pub children: Vec<u64>,
    pub failure_reason: Option<String>,
    pub quality_score: f64,
    pub depth: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct FailureTrace {
    pub id: u64,
    pub root_hypothesis: String,
    pub attempted_actions: Vec<String>,
    pub failure_pattern: String,
    pub outcome_vector: Vec<u8>,
    pub derived_heuristic: Option<u64>,
    pub recovery_attempts: u32,
    pub was_eventually_solved: bool,
    pub solution_path: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct VsaFailureCluster {
    pub centroid: Vec<u8>,
    pub member_ids: Vec<u64>,
    pub node_type: TraceNodeType,
    pub avg_severity: f64,
    pub count: usize,
    pub first_seen: u64,
    pub last_seen: u64,
}

#[derive(Debug, Clone)]
pub struct ExplorationGraph {
    nodes: Vec<TraceNode>,
    failures: Vec<FailureTrace>,
    next_id: u64,
    max_nodes: usize,
    cycle: u64,
    pub cluster_cache: Vec<VsaFailureCluster>,
    pub cluster_dirty: bool,
}

impl ExplorationGraph {
    pub fn new(max_nodes: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(max_nodes),
            failures: Vec::new(),
            next_id: 1,
            max_nodes,
            cycle: 0,
            cluster_cache: Vec::new(),
            cluster_dirty: true,
        }
    }

    pub fn add_node(
        &mut self,
        node_type: TraceNodeType,
        context_str: &str,
        action_str: &str,
        outcome_str: &str,
        parent_id: Option<u64>,
        failure_reason: Option<String>,
        quality_score: f64,
    ) -> u64 {
        self.cycle += 1;
        if self.nodes.len() >= self.max_nodes {
            self.prune_old();
        }

        let id = self.next_id;
        self.next_id += 1;

        let is_dead_end =
            node_type == TraceNodeType::DeadEnd || node_type == TraceNodeType::RejectedPath;

        let depth = parent_id
            .and_then(|pid| self.nodes.iter().find(|n| n.id == pid).map(|n| n.depth + 1))
            .unwrap_or(0);

        let node = TraceNode {
            id,
            node_type,
            context: QuantizedVSA::seeded_random(self.stable_hash(context_str), 4096),
            action: QuantizedVSA::seeded_random(self.stable_hash(action_str), 4096),
            outcome: QuantizedVSA::seeded_random(self.stable_hash(outcome_str), 4096),
            parent_id,
            children: Vec::new(),
            failure_reason,
            quality_score,
            depth,
            timestamp: self.cycle,
        };

        if let Some(pid) = parent_id {
            if let Some(parent) = self.nodes.iter_mut().find(|n| n.id == pid) {
                parent.children.push(id);
            }
        }

        if is_dead_end {
            self.record_failure_from_node(&node, context_str);
        }

        self.nodes.push(node);
        id
    }

    fn record_failure_from_node(&mut self, node: &TraceNode, context_str: &str) {
        let mut path = Vec::new();
        let mut current = Some(node.id);
        let nodes_snapshot: Vec<_> = self.nodes.iter().map(|n| (n.id, n.parent_id)).collect();
        while let Some(cid) = current {
            path.push(cid);
            current = nodes_snapshot
                .iter()
                .find(|(id, _)| *id == cid)
                .and_then(|(_, p)| *p);
        }
        path.reverse();

        let actions: Vec<String> = path
            .iter()
            .filter_map(|id| {
                let node_info = nodes_snapshot.iter().find(|(nid, _)| nid == id)?;
                Some(format!(
                    "{:?}:{:x}",
                    self.node_type_from_id(*id),
                    node_info.0
                ))
            })
            .collect();

        let trace = FailureTrace {
            id: self.next_id,
            root_hypothesis: context_str.to_string(),
            attempted_actions: actions,
            failure_pattern: node.failure_reason.clone().unwrap_or_default(),
            outcome_vector: node.outcome.clone(),
            derived_heuristic: None,
            recovery_attempts: 0,
            was_eventually_solved: false,
            solution_path: Vec::new(),
        };
        self.failures.push(trace);
    }

    fn node_type_from_id(&self, id: u64) -> TraceNodeType {
        self.nodes
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.node_type.clone())
            .unwrap_or(TraceNodeType::Hypothesis)
    }

    pub fn mark_resolved(&mut self, failure_id: u64, solution_node_ids: &[u64]) {
        if let Some(trace) = self.failures.iter_mut().find(|t| t.id == failure_id) {
            trace.was_eventually_solved = true;
            trace.solution_path = solution_node_ids.to_vec();
        }
    }

    pub fn find_similar_failures(&self, context_str: &str, top_k: usize) -> Vec<&FailureTrace> {
        let query = QuantizedVSA::seeded_random(self.stable_hash(context_str), 4096);
        let mut scored: Vec<(f64, usize)> = self
            .failures
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let sim = QuantizedVSA::similarity(&f.outcome_vector, &query);
                (sim, i)
            })
            .filter(|(s, _)| *s > 0.3)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(top_k)
            .map(|(_, i)| &self.failures[i])
            .collect()
    }

    pub fn extract_anti_patterns(&self) -> Vec<(String, String, f64)> {
        let mut patterns: HashMap<String, (usize, f64)> = HashMap::new();
        for f in &self.failures {
            let entry = patterns
                .entry(f.failure_pattern.clone())
                .or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += f.was_eventually_solved as u64 as f64;
        }

        let mut result: Vec<(String, String, f64)> = patterns
            .into_iter()
            .map(|(pattern, (count, solved))| {
                let freq = count as f64 / self.failures.len().max(1) as f64;
                let solve_rate = solved / count.max(1) as f64;
                let anti_pattern_desc = format!(
                    "此失败模式出现{}次, 最终解决率{:.1}%",
                    count,
                    solve_rate * 100.0
                );
                (pattern, anti_pattern_desc, freq * (1.0 - solve_rate))
            })
            .collect();
        result.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    /// Cluster failure traces by VSA Hamming similarity.
    /// threshold = 0.78 (matching RecurrenceDetector convention).
    /// min_cluster_size = 3.
    pub fn cluster_failures(
        &self,
        threshold: f64,
        min_cluster_size: usize,
    ) -> Vec<VsaFailureCluster> {
        let failed: Vec<&TraceNode> = self
            .nodes
            .iter()
            .filter(|n| n.quality_score < 0.5)
            .collect();
        if failed.len() < min_cluster_size {
            return Vec::new();
        }

        let n = failed.len();
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for i in 0..n {
            for j in (i + 1)..n {
                let sim = QuantizedVSA::similarity(&failed[i].outcome, &failed[j].outcome);
                if sim >= threshold && failed[i].node_type == failed[j].node_type {
                    adj[i].push(j);
                    adj[j].push(i);
                }
            }
        }

        let mut visited = vec![false; n];
        let mut clusters: Vec<Vec<usize>> = Vec::new();
        for i in 0..n {
            if !visited[i] {
                let mut stack = vec![i];
                let mut cluster = Vec::new();
                while let Some(idx) = stack.pop() {
                    if visited[idx] {
                        continue;
                    }
                    visited[idx] = true;
                    cluster.push(idx);
                    for &neighbor in &adj[idx] {
                        if !visited[neighbor] {
                            stack.push(neighbor);
                        }
                    }
                }
                clusters.push(cluster);
            }
        }

        clusters
            .into_iter()
            .filter(|c| c.len() >= min_cluster_size)
            .map(|members| {
                let member_ids: Vec<u64> = members.iter().map(|&i| failed[i].id).collect();
                let node_type = failed[members[0]].node_type.clone();
                let vsa_refs: Vec<&[u8]> = members
                    .iter()
                    .map(|&i| failed[i].outcome.as_slice())
                    .collect();
                let centroid = QuantizedVSA::majority_bundle(&vsa_refs);
                let avg_severity = members
                    .iter()
                    .map(|&i| 1.0 - failed[i].quality_score)
                    .sum::<f64>()
                    / members.len() as f64;
                let first_seen = members
                    .iter()
                    .map(|&i| failed[i].timestamp)
                    .min()
                    .unwrap_or(0);
                let last_seen = members
                    .iter()
                    .map(|&i| failed[i].timestamp)
                    .max()
                    .unwrap_or(0);
                VsaFailureCluster {
                    centroid,
                    member_ids,
                    node_type,
                    avg_severity,
                    count: members.len(),
                    first_seen,
                    last_seen,
                }
            })
            .collect()
    }

    /// Top-K anti-patterns by frequency from clusters
    pub fn top_clusters(&self, k: usize) -> Vec<VsaFailureCluster> {
        let mut sorted = self.cluster_cache.clone();
        sorted.sort_by(|a, b| b.count.cmp(&a.count));
        sorted.into_iter().take(k).collect()
    }

    /// Clear old cluster cache and recompute
    pub fn recompute_clusters(&mut self) {
        self.cluster_cache = self.cluster_failures(0.78, 3);
        self.cluster_dirty = false;
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    pub fn get_node(&self, id: u64) -> Option<&TraceNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_failure(&self, id: u64) -> Option<&FailureTrace> {
        self.failures.iter().find(|f| f.id == id)
    }

    pub fn attach_heuristic(&mut self, failure_id: u64, heuristic_id: u64) {
        if let Some(trace) = self.failures.iter_mut().find(|t| t.id == failure_id) {
            trace.derived_heuristic = Some(heuristic_id);
        }
    }

    fn prune_old(&mut self) {
        let keep = self.max_nodes * 3 / 4;
        if self.nodes.len() > keep {
            let mut with_scores: Vec<(f64, usize)> = self
                .nodes
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    let has_failure = self.failures.iter().any(|f| {
                        f.solution_path.contains(&n.id)
                            || f.attempted_actions
                                .iter()
                                .any(|a| a.contains(&n.id.to_string()))
                    });
                    let score = if has_failure { 10.0 } else { n.quality_score };
                    let recency = if n.timestamp == self.cycle { 5.0 } else { 1.0 };
                    (score * recency, i)
                })
                .collect();
            with_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            let keep_ids: std::collections::HashSet<u64> = with_scores
                .iter()
                .take(keep)
                .map(|(_, i)| self.nodes[*i].id)
                .collect();
            self.nodes.retain(|n| keep_ids.contains(&n.id));
        }
    }

    fn stable_hash(&self, s: &str) -> u64 {
        let mut h: u64 = 0xfa11_0000_0000_0000;
        for b in s.bytes() {
            h = h.wrapping_mul(0x9e3779b97f4a7c15);
            h ^= b as u64;
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_success_node() {
        let mut graph = ExplorationGraph::new(100);
        let id = graph.add_node(
            TraceNodeType::Success,
            "solve x",
            "compute",
            "answer=42",
            None,
            None,
            1.0,
        );
        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node(id).is_some());
    }

    #[test]
    fn test_dead_end_creates_failure() {
        let mut graph = ExplorationGraph::new(100);
        graph.add_node(
            TraceNodeType::Hypothesis,
            "try method A",
            "init",
            "",
            None,
            None,
            0.5,
        );
        graph.add_node(
            TraceNodeType::DeadEnd,
            "method A fails",
            "execute A",
            "error",
            Some(1),
            Some("divergence detected".into()),
            0.0,
        );
        assert_eq!(graph.failure_count(), 1);
    }

    #[test]
    fn test_find_similar_failures() {
        let mut graph = ExplorationGraph::new(100);
        graph.add_node(
            TraceNodeType::DeadEnd,
            "timeout",
            "wait",
            "fail",
            None,
            Some("timeout".into()),
            0.0,
        );
        graph.add_node(
            TraceNodeType::DeadEnd,
            "crash",
            "run",
            "fail",
            None,
            Some("crash".into()),
            0.0,
        );
        let similar = graph.find_similar_failures("timeout occurred", 2);
        assert!(!similar.is_empty());
    }

    #[test]
    fn test_extract_anti_patterns() {
        let mut graph = ExplorationGraph::new(100);
        graph.add_node(
            TraceNodeType::DeadEnd,
            "t1",
            "a1",
            "fail",
            None,
            Some("timeout".into()),
            0.0,
        );
        graph.add_node(
            TraceNodeType::DeadEnd,
            "t2",
            "a2",
            "fail",
            None,
            Some("timeout".into()),
            0.0,
        );
        graph.add_node(
            TraceNodeType::DeadEnd,
            "t3",
            "a3",
            "fail",
            None,
            Some("crash".into()),
            0.0,
        );
        let patterns = graph.extract_anti_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns[0].2 > 0.0);
    }

    #[test]
    fn test_mark_resolved() {
        let mut graph = ExplorationGraph::new(100);
        graph.add_node(
            TraceNodeType::DeadEnd,
            "bug",
            "run",
            "fail",
            None,
            Some("bug".into()),
            0.0,
        );
        assert_eq!(graph.failure_count(), 1);
        graph.add_node(
            TraceNodeType::Success,
            "fix",
            "patch",
            "pass",
            Some(1),
            None,
            1.0,
        );
        graph.mark_resolved(0, &[2]);
        let f = graph.get_failure(0).unwrap();
        assert!(f.was_eventually_solved);
    }

    #[test]
    fn test_attach_heuristic() {
        let mut graph = ExplorationGraph::new(100);
        graph.add_node(
            TraceNodeType::DeadEnd,
            "test",
            "run",
            "fail",
            None,
            Some("err".into()),
            0.0,
        );
        graph.attach_heuristic(0, 42);
        assert_eq!(graph.get_failure(0).unwrap().derived_heuristic, Some(42));
    }

    #[test]
    fn test_node_with_parent() {
        let mut graph = ExplorationGraph::new(100);
        let parent = graph.add_node(
            TraceNodeType::Hypothesis,
            "root",
            "think",
            "",
            None,
            None,
            0.5,
        );
        let child = graph.add_node(
            TraceNodeType::Experiment,
            "child",
            "try",
            "",
            Some(parent),
            None,
            0.6,
        );
        let p = graph.get_node(parent).unwrap();
        assert_eq!(p.children, vec![child]);
        let c = graph.get_node(child).unwrap();
        assert_eq!(c.parent_id, Some(parent));
    }
}

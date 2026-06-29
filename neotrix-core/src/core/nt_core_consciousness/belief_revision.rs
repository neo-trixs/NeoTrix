use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeRelation {
    Supports,
    Opposes,
    Implies,
}

#[derive(Debug, Clone)]
pub struct BeliefEdge {
    pub from_id: usize,
    pub to_id: usize,
    pub relation: EdgeRelation,
    pub strength: f64,
}

#[derive(Debug, Clone)]
pub struct BeliefNode {
    pub id: usize,
    pub belief_vsa: Vec<u8>,
    pub confidence: f64,
    pub timestamp: u64,
    pub source_id: String,
    pub entrenchment: f64,
}

#[derive(Debug, Clone)]
pub struct BeliefGraph {
    pub nodes: Vec<BeliefNode>,
    pub edges: Vec<BeliefEdge>,
    next_id: usize,
}

impl BeliefGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_belief(&mut self, vsa: Vec<u8>, confidence: f64, source: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(BeliefNode {
            id,
            belief_vsa: vsa,
            confidence,
            timestamp: 0,
            source_id: source.to_string(),
            entrenchment: 0.5,
        });
        id
    }

    pub fn remove_belief(&mut self, id: usize) -> bool {
        let before = self.nodes.len();
        self.nodes.retain(|n| n.id != id);
        self.edges.retain(|e| e.from_id != id && e.to_id != id);
        before != self.nodes.len()
    }

    pub fn add_edge(&mut self, from: usize, to: usize, relation: EdgeRelation, strength: f64) {
        self.edges.push(BeliefEdge {
            from_id: from,
            to_id: to,
            relation,
            strength,
        });
    }

    pub fn get_node(&self, id: usize) -> Option<&BeliefNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut BeliefNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn find_supporters(&self, id: usize) -> Vec<&BeliefEdge> {
        self.edges
            .iter()
            .filter(|e| e.to_id == id && e.relation == EdgeRelation::Supports)
            .collect()
    }

    pub fn find_opposers(&self, id: usize) -> Vec<&BeliefEdge> {
        self.edges
            .iter()
            .filter(|e| e.to_id == id && e.relation == EdgeRelation::Opposes)
            .collect()
    }

    pub fn find_implications(&self, id: usize) -> Vec<&BeliefEdge> {
        self.edges
            .iter()
            .filter(|e| e.from_id == id && e.relation == EdgeRelation::Implies)
            .collect()
    }

    pub fn all_beliefs(&self) -> &[BeliefNode] {
        &self.nodes
    }
}

#[derive(Debug, Clone)]
pub struct DissonanceCluster {
    pub involved_ids: Vec<usize>,
    pub severity: f64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct DissonanceDetector {
    pub dissonance_threshold: f64,
    pub total_detected: u64,
}

impl DissonanceDetector {
    pub fn new(threshold: f64) -> Self {
        Self {
            dissonance_threshold: threshold,
            total_detected: 0,
        }
    }

    pub fn detect(&mut self, graph: &BeliefGraph) -> Vec<DissonanceCluster> {
        let mut clusters = Vec::new();
        let mut visited: Vec<Vec<usize>> = Vec::new();

        for edge in &graph.edges {
            if edge.relation != EdgeRelation::Opposes {
                continue;
            }
            let a = edge.from_id;
            let b = edge.to_id;

            let has_reverse = graph
                .edges
                .iter()
                .any(|e| e.from_id == b && e.to_id == a && e.relation == EdgeRelation::Opposes);

            if has_reverse {
                let mut ids = vec![a, b];
                ids.sort();
                if !visited.contains(&ids) {
                    visited.push(ids.clone());
                    let mut total_strength = 0.0;
                    for e in &graph.edges {
                        if (e.from_id == a && e.to_id == b || e.from_id == b && e.to_id == a)
                            && e.relation == EdgeRelation::Opposes
                        {
                            total_strength += e.strength;
                        }
                    }
                    let severity = total_strength / 2.0;
                    if severity > self.dissonance_threshold {
                        clusters.push(DissonanceCluster {
                            involved_ids: ids,
                            severity,
                            description: format!(
                                "Mutual opposition between belief {} and belief {}",
                                a, b
                            ),
                        });
                    }
                }
            }
        }

        let mut extended_clusters = self.detect_extended_cycles(graph);
        clusters.append(&mut extended_clusters);

        self.total_detected += clusters.len() as u64;
        clusters
    }

    fn detect_extended_cycles(&self, graph: &BeliefGraph) -> Vec<DissonanceCluster> {
        let mut clusters = Vec::new();
        let node_ids: Vec<usize> = graph.nodes.iter().map(|n| n.id).collect();
        let n = node_ids.len();
        if n < 3 {
            return clusters;
        }
        let id_to_idx: HashMap<usize, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (*id, i))
            .collect();
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for e in &graph.edges {
            if e.relation == EdgeRelation::Opposes {
                if let (Some(&fi), Some(&ti)) = (id_to_idx.get(&e.from_id), id_to_idx.get(&e.to_id))
                {
                    adj[fi].push(ti);
                }
            }
        }
        let mut visited_global = vec![false; n];
        for start in 0..n {
            if visited_global[start] {
                continue;
            }
            let mut path = Vec::new();
            let mut in_stack = vec![false; n];
            self.find_cycles(
                start,
                &adj,
                &mut visited_global,
                &mut in_stack,
                &mut path,
                &mut clusters,
                &graph.edges,
                &node_ids,
            );
        }
        clusters
    }

    fn find_cycles(
        &self,
        u: usize,
        adj: &[Vec<usize>],
        visited_global: &mut [bool],
        in_stack: &mut [bool],
        path: &mut Vec<usize>,
        clusters: &mut Vec<DissonanceCluster>,
        edges: &[BeliefEdge],
        node_ids: &[usize],
    ) {
        visited_global[u] = true;
        in_stack[u] = true;
        path.push(u);

        for &v in &adj[u] {
            if !in_stack[v] {
                if !visited_global[v] {
                    self.find_cycles(
                        v,
                        adj,
                        visited_global,
                        in_stack,
                        path,
                        clusters,
                        edges,
                        node_ids,
                    );
                }
            } else if path.len() >= 2 {
                let pos = path.iter().position(|&x| x == v);
                if let Some(p) = pos {
                    let cycle: Vec<usize> = path[p..].to_vec();
                    if cycle.len() >= 3 {
                        let involved_ids: Vec<usize> = cycle.iter().map(|&i| node_ids[i]).collect();
                        let mut total_oppose = 0.0;
                        let mut oppose_count = 0;
                        for e in edges {
                            if e.relation == EdgeRelation::Opposes {
                                let fi = node_ids.iter().position(|&x| x == e.from_id);
                                let ti = node_ids.iter().position(|&x| x == e.to_id);
                                if let (Some(f), Some(t)) = (fi, ti) {
                                    if cycle.contains(&f) && cycle.contains(&t) {
                                        total_oppose += e.strength;
                                        oppose_count += 1;
                                    }
                                }
                            }
                        }
                        if oppose_count > 0 {
                            let severity = total_oppose / cycle.len() as f64;
                            if severity > self.dissonance_threshold {
                                let desc = format!(
                                    "Extended dissonance cycle: {}",
                                    involved_ids
                                        .iter()
                                        .map(|id| id.to_string())
                                        .collect::<Vec<_>>()
                                        .join(" ↔ ")
                                );
                                let mut sorted = involved_ids.clone();
                                sorted.sort();
                                if !clusters.iter().any(|c| c.involved_ids == sorted) {
                                    clusters.push(DissonanceCluster {
                                        involved_ids: sorted,
                                        severity,
                                        description: desc,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        path.pop();
        in_stack[u] = false;
    }
}

#[derive(Debug, Clone)]
pub enum RevisionOp {
    Expansion {
        new_belief: Vec<u8>,
        confidence: f64,
        source: String,
    },
    Contraction {
        target_id: usize,
    },
    Revision {
        new_belief: Vec<u8>,
        old_belief_id: usize,
        confidence: f64,
        source: String,
    },
}

#[derive(Debug, Clone)]
pub struct MinimalChangeEnforcer {
    pub max_edges_to_remove: usize,
}

impl MinimalChangeEnforcer {
    pub fn score_changes(&self, before: &BeliefGraph, after: &BeliefGraph) -> f64 {
        let node_diff = if before.node_count() + 1 > 0 {
            (before.node_count() as isize - after.node_count() as isize).unsigned_abs() as f64
                / (before.node_count().max(1) as f64)
        } else {
            0.0
        };
        let edge_diff = if before.edge_count() + 1 > 0 {
            (before.edge_count() as isize - after.edge_count() as isize).unsigned_abs() as f64
                / (before.edge_count().max(1) as f64)
        } else {
            0.0
        };
        (node_diff + edge_diff) / 2.0
    }

    pub fn select_minimal(&self, graph: &BeliefGraph, candidates: &[RevisionOp]) -> usize {
        if candidates.is_empty() {
            return 0;
        }
        let mut best_idx = 0;
        let mut best_score = f64::MAX;
        for (i, op) in candidates.iter().enumerate() {
            let after = self.simulate(graph, op);
            let score = self.score_changes(graph, &after);
            if score < best_score {
                best_score = score;
                best_idx = i;
            }
        }
        best_idx
    }

    fn simulate(&self, graph: &BeliefGraph, op: &RevisionOp) -> BeliefGraph {
        let mut sim = graph.clone();
        match op {
            RevisionOp::Expansion {
                new_belief,
                confidence,
                source,
            } => {
                sim.add_belief(new_belief.clone(), *confidence, source);
            }
            RevisionOp::Contraction { target_id } => {
                sim.remove_belief(*target_id);
            }
            RevisionOp::Revision {
                new_belief,
                old_belief_id,
                confidence,
                source,
            } => {
                sim.remove_belief(*old_belief_id);
                sim.add_belief(new_belief.clone(), *confidence, source);
            }
        }
        sim
    }
}

#[derive(Debug, Clone)]
pub struct BeliefRevisionEngine {
    pub graph: BeliefGraph,
    pub detector: DissonanceDetector,
    pub enforcer: MinimalChangeEnforcer,
    pub revision_count: u64,
    pub expansion_count: u64,
    pub contraction_count: u64,
    pub dissonance_history: VecDeque<DissonanceCluster>,
    max_history: usize,
}

impl BeliefRevisionEngine {
    pub fn new(dissonance_threshold: f64) -> Self {
        Self {
            graph: BeliefGraph::new(),
            detector: DissonanceDetector::new(dissonance_threshold),
            enforcer: MinimalChangeEnforcer {
                max_edges_to_remove: 5,
            },
            revision_count: 0,
            expansion_count: 0,
            contraction_count: 0,
            dissonance_history: VecDeque::new(),
            max_history: 100,
        }
    }

    pub fn expand(&mut self, op: RevisionOp) -> Result<usize, String> {
        match op {
            RevisionOp::Expansion {
                new_belief,
                confidence,
                source,
            } => {
                if confidence < 0.3 {
                    return Err(format!(
                        "Confidence {:.2} below expansion threshold 0.3",
                        confidence
                    ));
                }
                let id = self.graph.add_belief(new_belief, confidence, &source);
                self.expansion_count += 1;
                Ok(id)
            }
            _ => Err("expand called with non-Expansion op".to_string()),
        }
    }

    pub fn contract(&mut self, op: RevisionOp) -> Result<bool, String> {
        match op {
            RevisionOp::Contraction { target_id } => {
                let entrenchment = self
                    .graph
                    .get_node(target_id)
                    .map(|n| n.entrenchment)
                    .unwrap_or(0.0);
                if entrenchment >= 0.8 {
                    return Err(format!(
                        "Cannot contract belief {}: entrenchment {:.2} >= 0.8",
                        target_id, entrenchment
                    ));
                }
                let implications: Vec<usize> = self
                    .graph
                    .find_implications(target_id)
                    .iter()
                    .map(|e| e.to_id)
                    .collect();
                for impl_id in implications {
                    self.graph.remove_belief(impl_id);
                }
                let removed = self.graph.remove_belief(target_id);
                if removed {
                    self.contraction_count += 1;
                }
                Ok(removed)
            }
            _ => Err("contract called with non-Contraction op".to_string()),
        }
    }

    pub fn revise(&mut self, op: RevisionOp) -> Result<usize, String> {
        match op {
            RevisionOp::Revision {
                new_belief,
                old_belief_id,
                confidence,
                source,
            } => {
                self.graph.remove_belief(old_belief_id);
                let id = self.graph.add_belief(new_belief, confidence, &source);
                self.revision_count += 1;
                Ok(id)
            }
            _ => Err("revise called with non-Revision op".to_string()),
        }
    }

    pub fn detect_dissonance(&mut self) -> Vec<DissonanceCluster> {
        let clusters = self.detector.detect(&self.graph);
        for c in &clusters {
            self.dissonance_history.push_back(c.clone());
            if self.dissonance_history.len() > self.max_history {
                self.dissonance_history.pop_front();
            }
        }
        clusters
    }

    pub fn auto_resolve(&mut self, cluster: &DissonanceCluster) {
        let involved: Vec<usize> = cluster.involved_ids.clone();
        if involved.len() < 2 {
            return;
        }
        let mut pairs: Vec<(usize, usize)> = Vec::new();
        for i in 0..involved.len() {
            for j in (i + 1)..involved.len() {
                let has_oppose = self.graph.edges.iter().any(|e| {
                    (e.from_id == involved[i] && e.to_id == involved[j]
                        || e.from_id == involved[j] && e.to_id == involved[i])
                        && e.relation == EdgeRelation::Opposes
                });
                if has_oppose {
                    pairs.push((involved[i], involved[j]));
                }
            }
        }
        for (a_id, b_id) in pairs {
            let a_entrench = self
                .graph
                .get_node(a_id)
                .map(|n| n.entrenchment)
                .unwrap_or(0.0);
            let b_entrench = self
                .graph
                .get_node(b_id)
                .map(|n| n.entrenchment)
                .unwrap_or(0.0);
            if a_entrench < b_entrench {
                self.graph.remove_belief(a_id);
            } else {
                self.graph.remove_belief(b_id);
            }
        }
    }

    pub fn apply(&mut self, op: RevisionOp) -> Result<usize, String> {
        match &op {
            RevisionOp::Expansion { .. } => self.expand(op).map(|_| 0),
            RevisionOp::Contraction { .. } => self.contract(op).map(|_| 0),
            RevisionOp::Revision { .. } => self.revise(op),
        }
    }

    pub fn epistemic_report(&self) -> HashMap<String, f64> {
        let mut report = HashMap::new();
        report.insert("node_count".to_string(), self.graph.node_count() as f64);
        report.insert("edge_count".to_string(), self.graph.edge_count() as f64);
        let avg_conf: f64 = if self.graph.nodes.is_empty() {
            0.0
        } else {
            self.graph.nodes.iter().map(|n| n.confidence).sum::<f64>()
                / self.graph.nodes.len() as f64
        };
        report.insert("avg_confidence".to_string(), avg_conf);
        let avg_entr: f64 = if self.graph.nodes.is_empty() {
            0.0
        } else {
            self.graph.nodes.iter().map(|n| n.entrenchment).sum::<f64>()
                / self.graph.nodes.len() as f64
        };
        report.insert("avg_entrenchment".to_string(), avg_entr);
        report.insert(
            "dissonance_count".to_string(),
            self.dissonance_history.len() as f64,
        );
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::QuantizedVSA;

    fn test_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, 64)
    }

    #[test]
    fn test_belief_node_creation() {
        let vsa = test_vsa(1);
        let node = BeliefNode {
            id: 1,
            belief_vsa: vsa.clone(),
            confidence: 0.8,
            timestamp: 100,
            source_id: "test".to_string(),
            entrenchment: 0.5,
        };
        assert_eq!(node.id, 1);
        assert_eq!(node.belief_vsa, vsa);
        assert!((node.confidence - 0.8).abs() < 1e-9);
        assert!((node.entrenchment - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_belief_graph_add_node() {
        let mut g = BeliefGraph::new();
        let id1 = g.add_belief(test_vsa(1), 0.9, "source_a");
        let id2 = g.add_belief(test_vsa(2), 0.7, "source_b");
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(g.node_count(), 2);
    }

    #[test]
    fn test_belief_graph_add_edge() {
        let mut g = BeliefGraph::new();
        let a = g.add_belief(test_vsa(1), 0.9, "src");
        let b = g.add_belief(test_vsa(2), 0.8, "src");
        g.add_edge(a, b, EdgeRelation::Supports, 0.75);
        assert_eq!(g.edge_count(), 1);
        let supporters = g.find_supporters(b);
        assert_eq!(supporters.len(), 1);
        assert_eq!(supporters[0].from_id, a);
    }

    #[test]
    fn test_belief_graph_remove() {
        let mut g = BeliefGraph::new();
        let a = g.add_belief(test_vsa(1), 0.9, "src");
        let b = g.add_belief(test_vsa(2), 0.8, "src");
        g.add_edge(a, b, EdgeRelation::Supports, 0.5);
        assert!(g.remove_belief(a));
        assert_eq!(g.node_count(), 1);
        assert_eq!(g.edge_count(), 0);
        assert!(!g.remove_belief(999));
    }

    #[test]
    fn test_find_supporters() {
        let mut g = BeliefGraph::new();
        let a = g.add_belief(test_vsa(1), 0.9, "src");
        let b = g.add_belief(test_vsa(2), 0.8, "src");
        let c = g.add_belief(test_vsa(3), 0.7, "src");
        g.add_edge(a, c, EdgeRelation::Supports, 0.6);
        g.add_edge(b, c, EdgeRelation::Supports, 0.4);
        let supporters = g.find_supporters(c);
        assert_eq!(supporters.len(), 2);
        assert!(supporters.iter().any(|e| e.from_id == a));
        assert!(supporters.iter().any(|e| e.from_id == b));
    }

    #[test]
    fn test_find_opposers() {
        let mut g = BeliefGraph::new();
        let a = g.add_belief(test_vsa(1), 0.9, "src");
        let b = g.add_belief(test_vsa(2), 0.8, "src");
        g.add_edge(a, b, EdgeRelation::Opposes, 0.7);
        let opposers = g.find_opposers(b);
        assert_eq!(opposers.len(), 1);
        assert_eq!(opposers[0].from_id, a);
    }

    #[test]
    fn test_dissonance_detector_no_conflict() {
        let mut g = BeliefGraph::new();
        let a = g.add_belief(test_vsa(1), 0.9, "src");
        let b = g.add_belief(test_vsa(2), 0.8, "src");
        g.add_edge(a, b, EdgeRelation::Supports, 0.5);
        let mut detector = DissonanceDetector::new(0.3);
        let clusters = detector.detect(&g);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_dissonance_detector_detects() {
        let mut g = BeliefGraph::new();
        let a = g.add_belief(test_vsa(1), 0.9, "src");
        let b = g.add_belief(test_vsa(2), 0.8, "src");
        g.add_edge(a, b, EdgeRelation::Opposes, 0.8);
        g.add_edge(b, a, EdgeRelation::Opposes, 0.7);
        let mut detector = DissonanceDetector::new(0.3);
        let clusters = detector.detect(&g);
        assert_eq!(clusters.len(), 1);
        assert!(clusters[0].severity > 0.3);
        assert!(clusters[0].involved_ids.contains(&a));
        assert!(clusters[0].involved_ids.contains(&b));
    }

    #[test]
    fn test_revision_expansion() {
        let mut engine = BeliefRevisionEngine::new(0.3);
        let op = RevisionOp::Expansion {
            new_belief: test_vsa(10),
            confidence: 0.8,
            source: "perception".to_string(),
        };
        let result = engine.expand(op);
        assert!(result.is_ok());
        assert_eq!(engine.graph.node_count(), 1);
        assert_eq!(engine.expansion_count, 1);
    }

    #[test]
    fn test_revision_contraction() {
        let mut engine = BeliefRevisionEngine::new(0.3);
        let id = engine.graph.add_belief(test_vsa(1), 0.8, "src");
        if let Some(n) = engine.graph.get_node_mut(id) {
            n.entrenchment = 0.3;
        }
        let op = RevisionOp::Contraction { target_id: id };
        let result = engine.contract(op);
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(engine.graph.node_count(), 0);
    }

    #[test]
    fn test_revision_revision() {
        let mut engine = BeliefRevisionEngine::new(0.3);
        let old_id = engine.graph.add_belief(test_vsa(1), 0.8, "src");
        let op = RevisionOp::Revision {
            new_belief: test_vsa(2),
            old_belief_id: old_id,
            confidence: 0.9,
            source: "update".to_string(),
        };
        let result = engine.revise(op);
        assert!(result.is_ok());
        let new_id = result.unwrap();
        assert_ne!(new_id, old_id);
        assert_eq!(engine.graph.node_count(), 1);
        assert_eq!(engine.revision_count, 1);
    }

    #[test]
    fn test_expansion_low_confidence() {
        let mut engine = BeliefRevisionEngine::new(0.3);
        let op = RevisionOp::Expansion {
            new_belief: test_vsa(5),
            confidence: 0.2,
            source: "noise".to_string(),
        };
        let result = engine.expand(op);
        assert!(result.is_err());
        assert_eq!(engine.graph.node_count(), 0);
    }

    #[test]
    fn test_contraction_high_entrenchment() {
        let mut engine = BeliefRevisionEngine::new(0.3);
        let id = engine.graph.add_belief(test_vsa(1), 0.9, "core");
        if let Some(n) = engine.graph.get_node_mut(id) {
            n.entrenchment = 0.9;
        }
        let op = RevisionOp::Contraction { target_id: id };
        let result = engine.contract(op);
        assert!(result.is_err());
        assert_eq!(engine.graph.node_count(), 1);
    }

    #[test]
    fn test_auto_resolve() {
        let mut engine = BeliefRevisionEngine::new(0.3);
        let a = engine.graph.add_belief(test_vsa(1), 0.9, "src");
        let b = engine.graph.add_belief(test_vsa(2), 0.8, "src");
        if let Some(n) = engine.graph.get_node_mut(a) {
            n.entrenchment = 0.7;
        }
        if let Some(n) = engine.graph.get_node_mut(b) {
            n.entrenchment = 0.3;
        }
        engine.graph.add_edge(a, b, EdgeRelation::Opposes, 0.8);
        engine.graph.add_edge(b, a, EdgeRelation::Opposes, 0.7);
        let cluster = DissonanceCluster {
            involved_ids: vec![a, b],
            severity: 0.75,
            description: "test conflict".to_string(),
        };
        engine.auto_resolve(&cluster);
        assert_eq!(engine.graph.node_count(), 1);
        assert!(engine.graph.get_node(a).is_some());
    }

    #[test]
    fn test_minimal_change_enforcer_score() {
        let mut before = BeliefGraph::new();
        before.add_belief(test_vsa(1), 0.9, "src");
        before.add_belief(test_vsa(2), 0.8, "src");
        let mut after = before.clone();
        after.remove_belief(1);
        let enforcer = MinimalChangeEnforcer {
            max_edges_to_remove: 5,
        };
        let score = enforcer.score_changes(&before, &after);
        assert!(score > 0.0);
        assert!(score <= 1.0);

        let same_score = enforcer.score_changes(&before, &before);
        assert!((same_score - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_epistemic_report() {
        let mut engine = BeliefRevisionEngine::new(0.3);
        engine.graph.add_belief(test_vsa(1), 0.9, "src");
        engine.graph.add_belief(test_vsa(2), 0.5, "src");
        let report = engine.epistemic_report();
        assert_eq!(*report.get("node_count").unwrap() as usize, 2);
        assert!((*report.get("avg_confidence").unwrap() - 0.7).abs() < 1e-9);
        assert!((*report.get("avg_entrenchment").unwrap() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_dissonance_history() {
        let mut engine = BeliefRevisionEngine::new(0.3);
        let a = engine.graph.add_belief(test_vsa(1), 0.9, "src");
        let b = engine.graph.add_belief(test_vsa(2), 0.8, "src");
        engine.graph.add_edge(a, b, EdgeRelation::Opposes, 0.8);
        engine.graph.add_edge(b, a, EdgeRelation::Opposes, 0.7);
        let clusters = engine.detect_dissonance();
        assert!(!clusters.is_empty());
        assert_eq!(engine.dissonance_history.len(), clusters.len());
    }

    #[test]
    fn test_complex_cycle_detection() {
        let mut g = BeliefGraph::new();
        let a = g.add_belief(test_vsa(1), 0.9, "src");
        let b = g.add_belief(test_vsa(2), 0.8, "src");
        let c = g.add_belief(test_vsa(3), 0.7, "src");
        g.add_edge(a, b, EdgeRelation::Opposes, 0.6);
        g.add_edge(b, c, EdgeRelation::Opposes, 0.5);
        g.add_edge(c, a, EdgeRelation::Opposes, 0.4);
        let mut detector = DissonanceDetector::new(0.2);
        let clusters = detector.detect(&g);
        assert!(!clusters.is_empty());
        let found = clusters.iter().any(|cl| cl.involved_ids.len() >= 3);
        assert!(found);
    }

    #[test]
    fn test_find_implications() {
        let mut g = BeliefGraph::new();
        let a = g.add_belief(test_vsa(1), 0.9, "src");
        let b = g.add_belief(test_vsa(2), 0.8, "src");
        let c = g.add_belief(test_vsa(3), 0.7, "src");
        g.add_edge(a, b, EdgeRelation::Implies, 0.8);
        g.add_edge(a, c, EdgeRelation::Implies, 0.6);
        let implications = g.find_implications(a);
        assert_eq!(implications.len(), 2);
        assert!(implications.iter().any(|e| e.to_id == b));
        assert!(implications.iter().any(|e| e.to_id == c));
    }

    #[test]
    fn test_contraction_cascade_implications() {
        let mut engine = BeliefRevisionEngine::new(0.3);
        let a = engine.graph.add_belief(test_vsa(1), 0.9, "src");
        let b = engine.graph.add_belief(test_vsa(2), 0.8, "src");
        let c = engine.graph.add_belief(test_vsa(3), 0.7, "src");
        engine.graph.add_edge(a, b, EdgeRelation::Implies, 0.8);
        engine.graph.add_edge(b, c, EdgeRelation::Implies, 0.6);
        if let Some(n) = engine.graph.get_node_mut(a) {
            n.entrenchment = 0.2;
        }
        let op = RevisionOp::Contraction { target_id: a };
        let result = engine.contract(op);
        assert!(result.is_ok());
        assert!(engine.graph.get_node(a).is_none());
    }

    #[test]
    fn test_apply_dispatch() {
        let mut engine = BeliefRevisionEngine::new(0.3);
        let op = RevisionOp::Expansion {
            new_belief: test_vsa(10),
            confidence: 0.8,
            source: "test".to_string(),
        };
        assert!(engine.apply(op).is_ok());
        assert_eq!(engine.expansion_count, 1);
    }

    #[test]
    fn test_dissonance_detector_accumulates_total() {
        let mut g = BeliefGraph::new();
        let a = g.add_belief(test_vsa(1), 0.9, "src");
        let b = g.add_belief(test_vsa(2), 0.8, "src");
        g.add_edge(a, b, EdgeRelation::Opposes, 0.8);
        g.add_edge(b, a, EdgeRelation::Opposes, 0.7);
        let mut detector = DissonanceDetector::new(0.3);
        let before = detector.total_detected;
        detector.detect(&g);
        assert!(detector.total_detected > before);
    }

    #[test]
    fn test_get_node_and_get_node_mut() {
        let mut g = BeliefGraph::new();
        let id = g.add_belief(test_vsa(1), 0.8, "src");
        let node = g.get_node(id);
        assert!(node.is_some());
        assert_eq!(node.unwrap().id, id);
        let node_mut = g.get_node_mut(id);
        assert!(node_mut.is_some());
        node_mut.unwrap().confidence = 0.95;
        assert!((g.get_node(id).unwrap().confidence - 0.95).abs() < 1e-9);
        assert!(g.get_node(999).is_none());
    }

    #[test]
    fn test_all_beliefs() {
        let mut g = BeliefGraph::new();
        g.add_belief(test_vsa(1), 0.9, "src");
        g.add_belief(test_vsa(2), 0.8, "src");
        assert_eq!(g.all_beliefs().len(), 2);
    }

    #[test]
    fn test_select_minimal() {
        let mut g = BeliefGraph::new();
        g.add_belief(test_vsa(1), 0.9, "src");
        g.add_belief(test_vsa(2), 0.8, "src");
        let enforcer = MinimalChangeEnforcer {
            max_edges_to_remove: 5,
        };
        let candidates = vec![
            RevisionOp::Expansion {
                new_belief: test_vsa(3),
                confidence: 0.7,
                source: "test".to_string(),
            },
            RevisionOp::Contraction { target_id: 1 },
        ];
        let idx = enforcer.select_minimal(&g, &candidates);
        assert!(idx < candidates.len());
    }
}

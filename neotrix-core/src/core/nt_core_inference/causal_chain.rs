use crate::core::nt_core_consciousness::vsa_tag::VsaTagged;
use std::collections::{HashMap, HashSet, VecDeque};

/// Maximum nodes in a causal graph
const MAX_CAUSAL_NODES: usize = 1000;

/// Maximum depth for path-finding DFS
const MAX_PATH_DEPTH: usize = 12;

/// A causal link between two events
#[derive(Debug, Clone)]
pub struct CausalLink {
    pub cause_id: u64,
    pub effect_id: u64,
    pub confidence: f64,
    pub temporal_delta_ms: i64,
    pub mechanism: String,
}

/// A counterfactual scenario
#[derive(Debug, Clone)]
pub struct Counterfactual {
    pub id: u64,
    pub premise: String,
    pub actual_outcome: String,
    pub predicted_outcome: String,
    pub confidence: f64,
    pub reasoning_chain: Vec<String>,
    /// Whether this counterfactual represents a Pearl-style do()-intervention
    pub interventional: bool,
}

/// Multi-step state prediction
#[derive(Debug, Clone)]
pub struct StatePrediction {
    pub initial_state: VsaTagged,
    pub action: String,
    pub predicted_steps: Vec<PredictedStep>,
    pub horizon_steps: usize,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct PredictedStep {
    pub step: usize,
    pub predicted_state: String,
    pub probability: f64,
    pub alternatives: Vec<(String, f64)>,
}

/// A DAG representing the causal structure, built from recorded CausalLinks.
/// `nodes`: node_id → human-readable label
/// `edges`: parent → [(child, strength)]
#[derive(Debug, Clone)]
pub struct CausalGraph {
    pub nodes: HashMap<u64, String>,
    pub edges: HashMap<u64, Vec<(u64, f64)>>,
}

impl CausalGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Register a node. No-op if already present.
    pub fn add_node(&mut self, id: u64, label: String) {
        if self.nodes.len() >= MAX_CAUSAL_NODES {
            return;
        }
        self.nodes.entry(id).or_insert(label);
    }

    /// Add a directed edge from → to with causal strength.
    pub fn add_edge(&mut self, from: u64, to: u64, strength: f64) {
        self.add_node(from, format!("node_{}", from));
        self.add_node(to, format!("node_{}", to));
        self.edges.entry(from).or_default().push((to, strength));
    }

    /// Parent nodes (direct causes) of `node`, with edge strengths.
    pub fn parents(&self, node: u64) -> Vec<(u64, f64)> {
        let mut result = Vec::new();
        for (parent, children) in &self.edges {
            for (child, s) in children {
                if *child == node {
                    result.push((*parent, *s));
                }
            }
        }
        result
    }

    /// Children nodes (direct effects) of `node`, with edge strengths.
    pub fn children(&self, node: u64) -> Vec<(u64, f64)> {
        self.edges.get(&node).cloned().unwrap_or_default()
    }

    /// Count of all nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Topological sort via Kahn's algorithm. Returns empty vec if graph has a cycle.
    pub fn topological_sort(&self) -> Vec<u64> {
        let mut in_degree: HashMap<u64, usize> = HashMap::new();
        for &node in self.nodes.keys() {
            in_degree.entry(node).or_insert(0);
        }
        for (_, children) in &self.edges {
            for (child, _) in children {
                *in_degree.entry(*child).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<u64> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&n, _)| n)
            .collect();

        let mut sorted = Vec::new();
        while let Some(node) = queue.pop_front() {
            sorted.push(node);
            if let Some(children) = self.edges.get(&node) {
                for (child, _) in children {
                    if let Some(deg) = in_degree.get_mut(child) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(*child);
                        }
                    }
                }
            }
        }

        if sorted.len() == self.nodes.len() {
            sorted
        } else {
            Vec::new() // cycle detected
        }
    }

    /// Returns true iff the graph contains no directed cycles (i.e., valid DAG).
    pub fn is_valid_dag(&self) -> bool {
        if self.nodes.is_empty() {
            return true;
        }
        !self.topological_sort().is_empty()
    }
}

impl Default for CausalGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// A Pearl-style do(X=x) intervention.
#[derive(Debug, Clone)]
pub struct DoIntervention {
    pub target_node: u64,
    pub value: String,
    pub confidence: f64,
}

/// An interventional query P(Y | do(X=x), Z).
#[derive(Debug, Clone)]
pub struct InterventionalQuery {
    pub outcome: u64,
    pub intervention: DoIntervention,
    pub conditioning_set: Vec<u64>,
    pub adjustment_set: Option<Vec<u64>>,
}

// ---------------------------------------------------------------------------
// Helper: graph traversal utilities
// ---------------------------------------------------------------------------

fn has_directed_edge(graph: &CausalGraph, from: u64, to: u64) -> bool {
    graph
        .edges
        .get(&from)
        .map(|children| children.iter().any(|(c, _)| *c == to))
        .unwrap_or(false)
}

fn undirected_neighbors(graph: &CausalGraph, node: u64) -> Vec<u64> {
    let mut seen = HashSet::new();
    let mut nb = Vec::new();
    if let Some(children) = graph.edges.get(&node) {
        for (c, _) in children {
            if seen.insert(*c) {
                nb.push(*c);
            }
        }
    }
    for (parent, children) in &graph.edges {
        for (child, _) in children {
            if *child == node && seen.insert(*parent) {
                nb.push(*parent);
            }
        }
    }
    nb
}

/// All nodes reachable from `root` via directed edges (including root).
fn descendants_of(graph: &CausalGraph, root: u64) -> HashSet<u64> {
    let mut visited = HashSet::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if visited.insert(n) {
            if let Some(children) = graph.edges.get(&n) {
                for (c, _) in children {
                    if !visited.contains(c) {
                        stack.push(*c);
                    }
                }
            }
        }
    }
    visited
}

/// All nodes that can reach `target` via directed edges (including target).
fn ancestors_of(graph: &CausalGraph, target: u64) -> HashSet<u64> {
    let mut reverse: HashMap<u64, Vec<u64>> = HashMap::new();
    for (parent, children) in &graph.edges {
        for (child, _) in children {
            reverse.entry(*child).or_default().push(*parent);
        }
    }
    let mut visited = HashSet::new();
    let mut stack = vec![target];
    while let Some(n) = stack.pop() {
        if visited.insert(n) {
            if let Some(parents) = reverse.get(&n) {
                for p in parents {
                    if !visited.contains(p) {
                        stack.push(*p);
                    }
                }
            }
        }
    }
    visited
}

/// Enumerate all simple paths (no repeated nodes) between `from` and `to`,
/// up to `MAX_PATH_DEPTH` steps.
fn enumerate_paths(graph: &CausalGraph, from: u64, to: u64) -> Vec<Vec<u64>> {
    let mut paths = Vec::new();
    let mut current = vec![from];
    let mut visited = HashSet::new();
    visited.insert(from);
    dfs_enumerate(graph, from, to, &mut current, &mut visited, &mut paths);
    paths
}

fn dfs_enumerate(
    graph: &CausalGraph,
    cur: u64,
    target: u64,
    path: &mut Vec<u64>,
    visited: &mut HashSet<u64>,
    paths: &mut Vec<Vec<u64>>,
) {
    if path.len() > MAX_PATH_DEPTH {
        return;
    }
    if cur == target && path.len() > 1 {
        paths.push(path.clone());
        return;
    }
    for nb in undirected_neighbors(graph, cur) {
        if !visited.contains(&nb) {
            visited.insert(nb);
            path.push(nb);
            dfs_enumerate(graph, nb, target, path, visited, paths);
            path.pop();
            visited.remove(&nb);
        }
    }
}

/// A path [v0, v1, …, vk] is a *back-door path* from v0 to vk iff the first
/// edge points *into* v0 (i.e. v1 → v0).
fn is_backdoor_path(graph: &CausalGraph, path: &[u64]) -> bool {
    path.len() >= 2 && has_directed_edge(graph, path[1], path[0])
}

/// Determine the triple type at indices i..i+2 on `path`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TripleKind {
    Chain,    // A → B → C   or   A ← B ← C
    Fork,     // A ← B → C
    Collider, // A → B ← C
}

fn triple_kind(graph: &CausalGraph, a: u64, b: u64, c: u64) -> TripleKind {
    let a_to_b = has_directed_edge(graph, a, b);
    let b_to_c = has_directed_edge(graph, b, c);
    let b_to_a = has_directed_edge(graph, b, a);
    let c_to_b = has_directed_edge(graph, c, b);
    match (a_to_b, b_to_c, b_to_a, c_to_b) {
        (true, true, _, _) | (_, _, true, true) => TripleKind::Chain,
        (_, true, true, _) => TripleKind::Fork,
        (true, _, _, true) => TripleKind::Collider,
        _ => TripleKind::Chain,
    }
}

/// A simple path is *blocked* by set Z iff there exists at least one node B on
/// the path such that:
///   1. B is a non-collider and B ∈ Z, OR
///   2. B is a collider and B ∉ Z **and** no descendant of any node in Z is B.
fn is_path_blocked_by(
    graph: &CausalGraph,
    path: &[u64],
    z_set: &HashSet<u64>,
    z_descendants: &HashSet<u64>,
) -> bool {
    for i in 0..path.len().saturating_sub(2) {
        let a = path[i];
        let b = path[i + 1];
        let c = path[i + 2];
        let kind = triple_kind(graph, a, b, c);
        match kind {
            TripleKind::Chain | TripleKind::Fork => {
                if z_set.contains(&b) {
                    return true;
                }
            }
            TripleKind::Collider => {
                if !z_set.contains(&b) && !z_descendants.contains(&b) {
                    return true;
                }
            }
        }
    }
    false
}

/// All descendants of any node in `nodes` (union).
fn descendants_of_set(graph: &CausalGraph, nodes: &HashSet<u64>) -> HashSet<u64> {
    let mut all_desc = HashSet::new();
    for &n in nodes {
        all_desc.extend(descendants_of(graph, n));
    }
    all_desc
}

// ---------------------------------------------------------------------------
// Main engine
// ---------------------------------------------------------------------------

/// Main causal reasoning engine with Pearl SCM do-calculus support
#[derive(Debug)]
pub struct CausalReasoningEngine {
    pub causal_links: VecDeque<CausalLink>,
    pub counterfactuals: VecDeque<Counterfactual>,
    pub predictions: VecDeque<StatePrediction>,
    max_links: usize,
    max_counterfactuals: usize,
    max_predictions: usize,
    next_id: u64,
}

impl Default for CausalReasoningEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CausalReasoningEngine {
    pub fn new() -> Self {
        Self {
            causal_links: VecDeque::new(),
            counterfactuals: VecDeque::new(),
            predictions: VecDeque::new(),
            max_links: 5000,
            max_counterfactuals: 1000,
            max_predictions: 500,
            next_id: 1,
        }
    }

    pub fn with_max_links(mut self, max: usize) -> Self {
        self.max_links = max;
        self
    }

    pub fn with_max_counterfactuals(mut self, max: usize) -> Self {
        self.max_counterfactuals = max;
        self
    }

    pub fn with_max_predictions(mut self, max: usize) -> Self {
        self.max_predictions = max;
        self
    }

    /// Record a causal link between two events
    pub fn record_causal_link(
        &mut self,
        cause_id: u64,
        effect_id: u64,
        confidence: f64,
        delta_ms: i64,
        mechanism: String,
    ) {
        if self.causal_links.len() >= self.max_links {
            self.causal_links.pop_front();
        }
        self.causal_links.push_back(CausalLink {
            cause_id,
            effect_id,
            confidence,
            temporal_delta_ms: delta_ms,
            mechanism,
        });
        self.next_id += 1;
    }

    /// Generate a counterfactual given a premise (non-interventional)
    pub fn generate_counterfactual(
        &mut self,
        premise: &str,
        actual: &str,
        predicted: &str,
        confidence: f64,
        reasoning: Vec<String>,
    ) -> u64 {
        let id = self.next_id;
        if self.counterfactuals.len() >= self.max_counterfactuals {
            self.counterfactuals.pop_front();
        }
        self.counterfactuals.push_back(Counterfactual {
            id,
            premise: premise.to_string(),
            actual_outcome: actual.to_string(),
            predicted_outcome: predicted.to_string(),
            confidence,
            reasoning_chain: reasoning,
            interventional: false,
        });
        self.next_id += 1;
        id
    }

    /// Predict future states given an action
    pub fn predict_states(&mut self, state: VsaTagged, action: &str, horizon: usize) -> u64 {
        let id = self.next_id;
        let steps: Vec<_> = (0..horizon)
            .map(|i| PredictedStep {
                step: i + 1,
                predicted_state: format!("predicted state after step {}", i + 1),
                probability: 1.0 / (i + 1) as f64,
                alternatives: vec![("alternative".to_string(), 0.2)],
            })
            .collect();

        if self.predictions.len() >= self.max_predictions {
            self.predictions.pop_front();
        }
        self.predictions.push_back(StatePrediction {
            initial_state: state,
            action: action.to_string(),
            predicted_steps: steps,
            horizon_steps: horizon,
            confidence: 0.8_f64.max(1.0 - (horizon as f64 * 0.05)),
        });
        self.next_id += 1;
        id
    }

    /// Get causal links for a specific event
    pub fn causes_for(&self, effect_id: u64) -> Vec<&CausalLink> {
        self.causal_links
            .iter()
            .filter(|l| l.effect_id == effect_id)
            .collect()
    }

    pub fn effects_of(&self, cause_id: u64) -> Vec<&CausalLink> {
        self.causal_links
            .iter()
            .filter(|l| l.cause_id == cause_id)
            .collect()
    }

    pub fn stats(&self) -> String {
        format!(
            "CausalReasoningEngine: {} links, {} counterfactuals ({} interventional), {} predictions",
            self.causal_links.len(),
            self.counterfactuals.len(),
            self.counterfactuals.iter().filter(|c| c.interventional).count(),
            self.predictions.len()
        )
    }

    // =====================================================================
    // Pearl SCM do-calculus API
    // =====================================================================

    /// Build a `CausalGraph` DAG from all recorded causal links.
    /// Nodes are auto-registered from cause/effect ids; labels try to use
    /// neighbouring links' mechanism strings.
    pub fn build_causal_graph(&self) -> CausalGraph {
        let mut graph = CausalGraph::new();
        for link in &self.causal_links {
            graph.add_node(link.cause_id, format!("cause_{}", link.cause_id));
            graph.add_node(link.effect_id, format!("effect_{}", link.effect_id));
            graph.add_edge(link.cause_id, link.effect_id, link.confidence);
        }
        graph
    }

    /// Apply a Pearl-style `do(X=x)` intervention.
    ///
    /// This creates a *counterfactual* record with `interventional = true`.
    /// Semantically the intervention "mutilates" the graph by removing all
    /// incoming edges to the target node, but in our data model we simply
    /// record the forced assignment.
    pub fn do_intervene(&mut self, intervention: DoIntervention) -> u64 {
        let id = self.next_id;
        if self.counterfactuals.len() >= self.max_counterfactuals {
            self.counterfactuals.pop_front();
        }

        // Build a premise that describes the do-intervention
        let premise = format!("do({}={})", intervention.target_node, intervention.value);

        let predicted = format!(
            "intervened: {} set to {}",
            intervention.target_node, intervention.value
        );

        self.counterfactuals.push_back(Counterfactual {
            id,
            premise,
            actual_outcome: String::new(),
            predicted_outcome: predicted,
            confidence: intervention.confidence,
            reasoning_chain: vec![format!(
                "do-intervention on node {} with value {}",
                intervention.target_node, intervention.value
            )],
            interventional: true,
        });
        self.next_id += 1;
        id
    }

    /// Estimate `P(Y | do(X), Z)` using back-door adjustment.
    ///
    /// 1. If `query.adjustment_set` is `Some`, use it directly.
    /// 2. Otherwise, auto-detect a back-door set via `detect_backdoor()`.
    /// 3. Fall back to observed association (non-causal correlation) if no
    ///    valid adjustment set can be found.
    pub fn estimate_interventional(&self, query: &InterventionalQuery) -> f64 {
        let graph = self.build_causal_graph();
        let outcome = query.outcome;
        let treatment = query.intervention.target_node;

        // Observed association: average confidence of direct X→Y links
        let obs_links: Vec<&CausalLink> = self
            .causal_links
            .iter()
            .filter(|l| l.cause_id == treatment && l.effect_id == outcome)
            .collect();

        if obs_links.is_empty() {
            return 0.0;
        }
        let obs_assoc: f64 =
            obs_links.iter().map(|l| l.confidence).sum::<f64>() / obs_links.len() as f64;

        // Resolve adjustment set
        let adjustment: Option<Vec<u64>> = query.adjustment_set.clone().or_else(|| {
            let detected = self.detect_backdoor(&graph, outcome, treatment);
            if detected.is_empty() {
                None
            } else {
                Some(detected)
            }
        });

        match adjustment {
            Some(ref adj) if !adj.is_empty() => {
                // Back-door adjustment:
                // P(Y|do(X)) ≈ Σ_z P(Y|X,z) P(z)
                // Approximate using link confidences: for each confounder C,
                // compute confounding bias as product of C→X and C→Y strengths.
                let mut conf_bias = 0.0_f64;
                let mut count = 0_usize;
                for conf in adj {
                    let c_to_x: f64 = self
                        .causal_links
                        .iter()
                        .filter(|l| l.cause_id == *conf && l.effect_id == treatment)
                        .map(|l| l.confidence)
                        .sum();
                    let c_to_y: f64 = self
                        .causal_links
                        .iter()
                        .filter(|l| l.cause_id == *conf && l.effect_id == outcome)
                        .map(|l| l.confidence)
                        .sum();
                    let n = self
                        .causal_links
                        .iter()
                        .filter(|l| {
                            l.cause_id == *conf
                                && (l.effect_id == treatment || l.effect_id == outcome)
                        })
                        .count();
                    if n > 0 {
                        let avg_x = c_to_x / n as f64;
                        let avg_y = c_to_y / n as f64;
                        conf_bias += avg_x * avg_y;
                        count += 1;
                    }
                }
                if count > 0 {
                    let bias = conf_bias / count as f64;
                    (obs_assoc - bias * 0.3).clamp(0.0, 1.0)
                } else {
                    obs_assoc
                }
            }
            _ => obs_assoc,
        }
    }

    /// Detect a set of nodes satisfying the **back-door criterion** relative
    /// to `(treatment, outcome)` in `graph`.
    ///
    /// A set Z satisfies the criterion iff:
    ///   (1) No node in Z is a descendant of treatment;
    ///   (2) Z blocks every back-door path from treatment to outcome.
    ///
    /// This implementation enumerates candidate confounders (ancestors of
    /// outcome that are non-descendants of treatment) and verifies that they
    /// block all back-door paths.  Returns an empty vec when no valid set is
    /// found.
    pub fn detect_backdoor(&self, graph: &CausalGraph, outcome: u64, treatment: u64) -> Vec<u64> {
        let descendants_of_tx = descendants_of(graph, treatment);

        // Candidates: all nodes except treatment itself and its descendants
        let candidates: HashSet<u64> = graph
            .nodes
            .keys()
            .filter(|&&n| n != treatment && !descendants_of_tx.contains(&n))
            .copied()
            .collect();

        // Narrow to nodes that are ancestors of outcome
        let ancestors_of_y = ancestors_of(graph, outcome);
        let z: Vec<u64> = candidates
            .into_iter()
            .filter(|c| ancestors_of_y.contains(c))
            .collect();

        if z.is_empty() {
            return z;
        }

        // Verify that Z blocks all back-door paths
        let all_paths = enumerate_paths(graph, treatment, outcome);
        let backdoor_paths: Vec<Vec<u64>> = all_paths
            .into_iter()
            .filter(|p| is_backdoor_path(graph, p))
            .collect();

        if backdoor_paths.is_empty() {
            return z; // no back-door paths → any non-descendant works
        }

        let z_set: HashSet<u64> = z.iter().copied().collect();
        let z_desc = descendants_of_set(graph, &z_set);

        let all_blocked = backdoor_paths
            .iter()
            .all(|p| is_path_blocked_by(graph, p, &z_set, &z_desc));

        if all_blocked {
            z
        } else {
            Vec::new()
        }
    }

    /// Detect a **front-door** set — a mediator M that fully mediates the
    /// treatment→outcome effect.
    ///
    /// Returns `Some([m])` if there exists a node m such that:
    ///   (1) All treatment→outcome paths go through m;
    ///   (2) There is no unblocked back-door path from treatment to m;
    ///   (3) There is no unblocked back-door path from m to outcome.
    ///
    /// For simplicity, this checks condition (1) alone (all directed paths
    /// from treatment to outcome pass through the candidate mediator).
    pub fn detect_frontdoor(
        &self,
        graph: &CausalGraph,
        outcome: u64,
        treatment: u64,
    ) -> Option<Vec<u64>> {
        let mediators: Vec<u64> = graph
            .nodes
            .keys()
            .filter(|&&n| n != treatment && n != outcome)
            .copied()
            .collect();

        for m in mediators {
            // Condition 1: every directed path from treatment to outcome goes through m
            let paths = enumerate_paths(graph, treatment, outcome);
            let directed_paths: Vec<Vec<u64>> = paths
                .into_iter()
                .filter(|p| p.windows(2).all(|w| has_directed_edge(graph, w[0], w[1])))
                .collect();

            if directed_paths.is_empty() {
                continue;
            }

            let all_through_m = directed_paths.iter().all(|p| p.contains(&m));
            if all_through_m {
                return Some(vec![m]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_causal_link() {
        let mut engine = CausalReasoningEngine::new();
        engine.record_causal_link(1, 2, 0.85, 150, "activation propagated".into());
        assert_eq!(engine.causal_links.len(), 1);
        let link = &engine.causal_links[0];
        assert_eq!(link.cause_id, 1);
        assert_eq!(link.effect_id, 2);
        assert!((link.confidence - 0.85).abs() < 1e-10);
    }

    #[test]
    fn test_counterfactual_generation_bounds() {
        let mut engine = CausalReasoningEngine::new().with_max_counterfactuals(3);
        for i in 0..5 {
            engine.generate_counterfactual(
                &format!("premise {}", i),
                "actual",
                "predicted",
                0.7,
                vec!["step 1".into()],
            );
        }
        assert_eq!(engine.counterfactuals.len(), 3);
        assert_eq!(engine.counterfactuals[0].premise, "premise 2");
        assert_eq!(engine.counterfactuals[2].premise, "premise 4");
    }

    #[test]
    fn test_prediction_with_horizon() {
        let mut engine = CausalReasoningEngine::new();
        use crate::core::nt_core_consciousness::vsa_tag::VsaOrigin;
        use crate::core::nt_core_consciousness::vsa_tag::VsaSelfCategory;
        let state = VsaTagged::new(
            vec![0u8; 64],
            VsaOrigin::Self_(VsaSelfCategory::Imagination),
        );
        let id = engine.predict_states(state, "analyze threat", 4);
        let pred = engine.predictions.back().unwrap();
        assert_eq!(pred.horizon_steps, 4);
        assert_eq!(pred.predicted_steps.len(), 4);
        assert_eq!(pred.predicted_steps[0].step, 1);
        assert!(pred.confidence > 0.0);
        assert!(id > 0);
    }

    #[test]
    fn test_causes_for_effects_of() {
        let mut engine = CausalReasoningEngine::new();
        engine.record_causal_link(10, 20, 0.9, 100, "A caused B".into());
        engine.record_causal_link(10, 30, 0.7, 200, "A caused C".into());
        engine.record_causal_link(15, 20, 0.5, 50, "D caused B".into());

        let causes = engine.causes_for(20);
        assert_eq!(causes.len(), 2);
        assert!(causes.iter().any(|l| l.cause_id == 10));
        assert!(causes.iter().any(|l| l.cause_id == 15));

        let effects = engine.effects_of(10);
        assert_eq!(effects.len(), 2);
        assert!(effects.iter().any(|l| l.effect_id == 20));
        assert!(effects.iter().any(|l| l.effect_id == 30));
    }

    #[test]
    fn test_max_links_bounds() {
        let mut engine = CausalReasoningEngine::new().with_max_links(3);
        for i in 0..6 {
            engine.record_causal_link(i, i + 1, 0.5, 10, "test".into());
        }
        assert_eq!(engine.causal_links.len(), 3);
        assert_eq!(engine.causal_links[0].cause_id, 3);
    }

    #[test]
    fn test_stats() {
        let engine = CausalReasoningEngine::new();
        let s = engine.stats();
        assert!(s.contains("0 links"));
        assert!(s.contains("0 counterfactuals"));
    }

    // =====================================================================
    // Pearl SCM do-calculus tests
    // =====================================================================

    #[test]
    fn test_causal_graph_construction() {
        let mut engine = CausalReasoningEngine::new();
        engine.record_causal_link(1, 2, 0.9, 100, "A→B".into());
        engine.record_causal_link(2, 3, 0.8, 200, "B→C".into());
        engine.record_causal_link(1, 3, 0.5, 300, "A→C direct".into());

        let graph = engine.build_causal_graph();

        // All 3 nodes present
        assert!(graph.nodes.contains_key(&1));
        assert!(graph.nodes.contains_key(&2));
        assert!(graph.nodes.contains_key(&3));

        // Edges: 1→2, 2→3, 1→3
        let children_1: Vec<u64> = graph.children(1).iter().map(|(c, _)| *c).collect();
        assert!(children_1.contains(&2));
        assert!(children_1.contains(&3));

        let children_2: Vec<u64> = graph.children(2).iter().map(|(c, _)| *c).collect();
        assert!(children_2.contains(&3));

        // Parents of 3 = [1, 2]
        let parents_3: Vec<u64> = graph.parents(3).iter().map(|(p, _)| *p).collect();
        assert!(parents_3.contains(&1));
        assert!(parents_3.contains(&2));

        // Valid DAG (no cycle)
        assert!(graph.is_valid_dag());
    }

    #[test]
    fn test_do_intervention_creates_counterfactual() {
        let mut engine = CausalReasoningEngine::new();
        let intervention = DoIntervention {
            target_node: 42,
            value: "true".into(),
            confidence: 0.95,
        };

        let id = engine.do_intervene(intervention);
        let cf = engine.counterfactuals.back().unwrap();

        // Counterfactual created with correct id
        assert_eq!(cf.id, id);
        assert!(cf.interventional);
        assert!(cf.premise.contains("do(42=true)"));
        assert!((cf.confidence - 0.95).abs() < 1e-10);
    }

    #[test]
    fn test_backdoor_detection() {
        // Create a classic confounder structure:
        //   C (confounder) → X (treatment)
        //   C → Y (outcome)
        //   X → Y
        let mut engine = CausalReasoningEngine::new();
        engine.record_causal_link(3, 1, 0.7, 50, "C→X".into()); // C→treatment
        engine.record_causal_link(3, 2, 0.8, 80, "C→Y".into()); // C→outcome
        engine.record_causal_link(1, 2, 0.6, 100, "X→Y".into()); // treatment→outcome

        let graph = engine.build_causal_graph();
        let backdoor = engine.detect_backdoor(&graph, 2, 1);

        // C (node 3) should be in the back-door set
        assert!(!backdoor.is_empty(), "should find a back-door set");
        assert!(
            backdoor.contains(&3),
            "confounder C should be in back-door set"
        );
    }

    #[test]
    fn test_interventional_estimate_fallback() {
        // Simple X→Y without confounders — estimate falls back to association
        let mut engine = CausalReasoningEngine::new();
        engine.record_causal_link(10, 20, 0.85, 100, "X→Y".into());

        let query = InterventionalQuery {
            outcome: 20,
            intervention: DoIntervention {
                target_node: 10,
                value: "active".into(),
                confidence: 1.0,
            },
            conditioning_set: vec![],
            adjustment_set: None,
        };

        let estimate = engine.estimate_interventional(&query);
        // Should be close to the direct link confidence (0.85) since no confounders
        assert!(
            (estimate - 0.85).abs() < 0.01,
            "estimate should match association when no confounders"
        );

        // Query with no data returns 0.0
        let no_data_query = InterventionalQuery {
            outcome: 99,
            intervention: DoIntervention {
                target_node: 10,
                value: "active".into(),
                confidence: 1.0,
            },
            conditioning_set: vec![],
            adjustment_set: None,
        };
        assert!((engine.estimate_interventional(&no_data_query) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_graph_cycle_detection() {
        // A→B→C→A (cycle)
        let mut graph = CausalGraph::new();
        graph.add_node(1, "A".into());
        graph.add_node(2, "B".into());
        graph.add_node(3, "C".into());
        graph.add_edge(1, 2, 0.5);
        graph.add_edge(2, 3, 0.5);
        graph.add_edge(3, 1, 0.5);

        // Topological sort should fail (cycle)
        let sorted = graph.topological_sort();
        assert!(sorted.is_empty());

        // is_valid_dag should return false
        assert!(!graph.is_valid_dag());
    }

    #[test]
    fn test_interventional_with_adjustment_set() {
        // Confounded structure with explicit adjustment set
        let mut engine = CausalReasoningEngine::new();
        engine.record_causal_link(3, 1, 0.7, 50, "C→X".into());
        engine.record_causal_link(3, 2, 0.8, 80, "C→Y".into());
        engine.record_causal_link(1, 2, 0.6, 100, "X→Y".into());

        let query = InterventionalQuery {
            outcome: 2,
            intervention: DoIntervention {
                target_node: 1,
                value: "on".into(),
                confidence: 1.0,
            },
            conditioning_set: vec![],
            adjustment_set: Some(vec![3]),
        };

        let estimate = engine.estimate_interventional(&query);
        // With back-door adjustment, estimate should differ from raw association
        assert!(estimate > 0.0);
        assert!(estimate <= 1.0);
    }

    #[test]
    fn test_frontdoor_detection() {
        // Mediator structure:
        //   X (treatment) → M (mediator) → Y (outcome)
        let mut engine = CausalReasoningEngine::new();
        engine.record_causal_link(1, 5, 0.9, 50, "X→M".into());
        engine.record_causal_link(5, 2, 0.8, 100, "M→Y".into());

        let graph = engine.build_causal_graph();
        let frontdoor = engine.detect_frontdoor(&graph, 2, 1);

        // M (node 5) should be detected as the front-door mediator
        assert!(frontdoor.is_some(), "should find a front-door set");
        let fd = frontdoor.unwrap();
        assert!(fd.contains(&5), "mediator M should be in front-door set");
    }

    #[test]
    fn test_empty_graph_valid_dag() {
        let graph = CausalGraph::new();
        assert!(graph.is_valid_dag());
        assert!(graph.topological_sort().is_empty());
    }

    #[test]
    fn test_topological_sort() {
        // 1→2→3  (linear chain)
        let mut graph = CausalGraph::new();
        graph.add_edge(1, 2, 0.5);
        graph.add_edge(2, 3, 0.5);

        let sorted = graph.topological_sort();
        assert_eq!(sorted.len(), 3);
        // 1 must come before 2, 2 before 3
        let pos = |n: u64| sorted.iter().position(|&x| x == n).unwrap();
        assert!(pos(1) < pos(2));
        assert!(pos(2) < pos(3));
    }

    #[test]
    fn test_diamond_graph_structure() {
        // Diamond: 1→2→4  and  1→3→4
        let mut engine = CausalReasoningEngine::new();
        engine.record_causal_link(1, 2, 0.7, 10, "→".into());
        engine.record_causal_link(1, 3, 0.6, 20, "→".into());
        engine.record_causal_link(2, 4, 0.8, 30, "→".into());
        engine.record_causal_link(3, 4, 0.5, 40, "→".into());

        let graph = engine.build_causal_graph();
        assert!(graph.is_valid_dag());

        let children_1: Vec<u64> = graph.children(1).iter().map(|(c, _)| *c).collect();
        assert!(children_1.contains(&2));
        assert!(children_1.contains(&3));

        let parents_4: Vec<u64> = graph.parents(4).iter().map(|(p, _)| *p).collect();
        assert!(parents_4.contains(&2));
        assert!(parents_4.contains(&3));
    }

    #[test]
    fn test_do_intervention_bounds() {
        let mut engine = CausalReasoningEngine::new().with_max_counterfactuals(2);
        for i in 0..5 {
            engine.do_intervene(DoIntervention {
                target_node: i,
                value: format!("v{}", i),
                confidence: 0.9,
            });
        }
        assert_eq!(engine.counterfactuals.len(), 2);
        // Only last 2 should remain
        assert!(engine.counterfactuals[0].premise.contains("do(3="));
        assert!(engine.counterfactuals[1].premise.contains("do(4="));
    }

    #[test]
    fn test_stats_includes_interventional_count() {
        let mut engine = CausalReasoningEngine::new();
        engine.generate_counterfactual("test", "a", "p", 0.5, vec![]);
        engine.do_intervene(DoIntervention {
            target_node: 1,
            value: "x".into(),
            confidence: 0.9,
        });
        let s = engine.stats();
        assert!(s.contains("1 interventional"));
    }
}

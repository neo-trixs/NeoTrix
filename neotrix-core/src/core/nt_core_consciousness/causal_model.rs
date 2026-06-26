use std::collections::{HashMap, HashSet, VecDeque};

/// A node in the causal graph — represents a variable
#[derive(Debug, Clone)]
pub struct CausalNode {
    pub id: usize,
    pub name: String,
    pub value: f64,
    pub intervened: bool,
}

/// A directed edge in the causal graph — represents causal influence
#[derive(Debug, Clone)]
pub struct CausalEdge {
    pub from: usize,
    pub to: usize,
    pub strength: f64,
    pub delay: usize,
}

/// A recorded counterfactual: "if I had done X instead, Y would have happened"
#[derive(Debug, Clone)]
pub struct Counterfactual {
    pub id: usize,
    pub fact: String,
    pub intervention: String,
    pub actual_outcome: String,
    pub counterfactual_outcome: String,
    pub confidence: f64,
    pub tick: u64,
}

/// Implements Pearl's do-calculus operations on a causal DAG
#[derive(Debug, Clone)]
pub struct DoCalculus {
    pub nodes: Vec<CausalNode>,
    pub edges: Vec<CausalEdge>,
    next_id: usize,
}

impl DoCalculus {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_node(&mut self, name: &str, value: f64) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(CausalNode {
            id,
            name: name.to_string(),
            value,
            intervened: false,
        });
        id
    }

    pub fn add_edge(&mut self, from: usize, to: usize, strength: f64, delay: usize) {
        self.edges.push(CausalEdge {
            from,
            to,
            strength: strength.clamp(-1.0, 1.0),
            delay,
        });
    }

    /// Forces a node to a specific value (do-operation)
    pub fn do_intervention(&mut self, node_id: usize, value: f64) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.value = value;
            node.intervened = true;
        }
    }

    /// Propagates an intervention through the graph and predicts effect on target after `steps` ticks
    pub fn predict_effect(&self, intervention_id: usize, target_id: usize, steps: usize) -> f64 {
        let mut state: HashMap<usize, f64> = self.nodes.iter().map(|n| (n.id, n.value)).collect();
        let base_state: HashMap<usize, f64> = state.clone();

        if let Some(node) = self.nodes.iter().find(|n| n.id == intervention_id) {
            state.insert(intervention_id, node.value);
        }

        let mut incoming: HashMap<usize, Vec<(usize, f64, usize)>> = HashMap::new();
        for edge in &self.edges {
            incoming
                .entry(edge.to)
                .or_default()
                .push((edge.from, edge.strength, edge.delay));
        }

        let mut changed_at: HashMap<usize, usize> = HashMap::new();
        changed_at.insert(intervention_id, 0);

        for t in 1..=steps {
            let mut new_state = state.clone();
            for (&node_id, _) in &state {
                if node_id == intervention_id {
                    continue;
                }
                if let Some(incoming_edges) = incoming.get(&node_id) {
                    let mut total = 0.0;
                    let mut count = 0usize;
                    for &(from_id, strength, delay) in incoming_edges {
                        if delay <= t {
                            let base_val = base_state.get(&from_id).copied().unwrap_or(0.0);
                            let current_val = state.get(&from_id).copied().unwrap_or(0.0);
                            if let Some(&changed) = changed_at.get(&from_id) {
                                if changed + delay <= t {
                                    total += strength * current_val;
                                } else {
                                    total += strength * base_val;
                                }
                            } else {
                                total += strength * base_val;
                            }
                            count += 1;
                        }
                    }
                    if count > 0 {
                        let new_val = total / count as f64;
                        if (new_val - state[&node_id]).abs() > 1e-9 {
                            changed_at.insert(node_id, t);
                        }
                        new_state.insert(node_id, new_val);
                    }
                }
            }
            state = new_state;
        }

        state.get(&target_id).copied().unwrap_or(0.0)
    }

    /// Computes P(effect | do(cause=value), given=conditions)
    pub fn conditional_probability(
        &self,
        cause_id: usize,
        effect_id: usize,
        given: &[(usize, f64)],
    ) -> f64 {
        let mut state: HashMap<usize, f64> = self.nodes.iter().map(|n| (n.id, n.value)).collect();

        for &(gid, gv) in given {
            state.insert(gid, gv);
        }

        if let Some(node) = self.nodes.iter().find(|n| n.id == cause_id) {
            state.insert(cause_id, node.value);
        }

        let max_delay = self.edges.iter().map(|e| e.delay).max().unwrap_or(1);
        let mut incoming: HashMap<usize, Vec<(usize, f64)>> = HashMap::new();
        for edge in &self.edges {
            incoming
                .entry(edge.to)
                .or_default()
                .push((edge.from, edge.strength));
        }

        for _ in 0..=max_delay {
            let mut new_state = state.clone();
            for (&node_id, _) in &state {
                if node_id == cause_id {
                    continue;
                }
                if given.iter().any(|(id, _)| *id == node_id) {
                    continue;
                }
                if let Some(incoming_edges) = incoming.get(&node_id) {
                    let mut total = 0.0;
                    let mut count = 0usize;
                    for &(from_id, strength) in incoming_edges {
                        total += strength * state.get(&from_id).copied().unwrap_or(0.0);
                        count += 1;
                    }
                    if count > 0 {
                        new_state.insert(node_id, total / count as f64);
                    }
                }
            }
            state = new_state;
        }

        state.get(&effect_id).copied().unwrap_or(0.0)
    }

    /// Checks if there's a valid backdoor adjustment set (no unblocked confounders)
    pub fn backdoor_criterion(&self, cause_id: usize, effect_id: usize) -> bool {
        let cause_incoming: Vec<usize> = self
            .edges
            .iter()
            .filter(|e| e.to == cause_id)
            .map(|e| e.from)
            .collect();

        if cause_incoming.is_empty() {
            return true;
        }

        for &confounder in &cause_incoming {
            if self.has_path(confounder, effect_id) {
                return false;
            }
        }

        true
    }

    /// Checks if there exists a mediator satisfying the frontdoor criterion
    pub fn frontdoor_criterion(&self, cause_id: usize, effect_id: usize) -> bool {
        let mediators: Vec<usize> = self
            .nodes
            .iter()
            .map(|n| n.id)
            .filter(|&id| {
                id != cause_id
                    && id != effect_id
                    && self.has_path(cause_id, id)
                    && self.has_path(id, effect_id)
            })
            .collect();

        if mediators.is_empty() {
            return false;
        }

        for &m in &mediators {
            let m_backdoor: Vec<usize> = self
                .edges
                .iter()
                .filter(|e| e.to == m)
                .map(|e| e.from)
                .filter(|&f| f != cause_id)
                .collect();

            let has_unblocked = m_backdoor
                .iter()
                .any(|&conf| !self.has_path(cause_id, conf) && self.has_path(conf, effect_id));

            if !has_unblocked {
                return true;
            }
        }

        false
    }

    /// DFS to check if there's a directed path from start to end
    pub fn has_path(&self, start: usize, end: usize) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);

        while let Some(node) = queue.pop_front() {
            if node == end {
                return true;
            }
            if !visited.insert(node) {
                continue;
            }
            for edge in &self.edges {
                if edge.from == node && !visited.contains(&edge.to) {
                    queue.push_back(edge.to);
                }
            }
        }

        false
    }

    /// Generates DOT format output for graph visualization
    pub fn graphviz(&self) -> String {
        let mut dot = String::from("digraph CausalModel {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n\n");

        for node in &self.nodes {
            let label = format!("{}\\n({:.3})", node.name, node.value);
            if node.intervened {
                dot.push_str(&format!(
                    "  {} [label=\"{}\", fillcolor=lightcoral];\n",
                    node.id, label
                ));
            } else {
                dot.push_str(&format!("  {} [label=\"{}\"];\n", node.id, label));
            }
        }

        dot.push('\n');
        for edge in &self.edges {
            dot.push_str(&format!(
                "  {} -> {} [label=\"s={:.2},d={}\"];\n",
                edge.from, edge.to, edge.strength, edge.delay
            ));
        }

        dot.push_str("}\n");
        dot
    }
}

/// Higher-level causal reasoner that wraps DoCalculus and adds counterfactual reasoning
#[derive(Debug, Clone)]
pub struct CausalReasoner {
    pub model: DoCalculus,
    pub counterfactuals: Vec<Counterfactual>,
    max_history: usize,
}

impl CausalReasoner {
    pub fn new() -> Self {
        Self {
            model: DoCalculus::new(),
            counterfactuals: Vec::new(),
            max_history: 100,
        }
    }

    /// Records and evaluates a counterfactual: "if I had done X instead, Y would have happened"
    pub fn counterfactual(
        &mut self,
        fact: &str,
        intervention: &str,
        outcome: &str,
        tick: u64,
    ) -> Counterfactual {
        let id = self.counterfactuals.len() + 1;
        let confidence = 0.75;

        let cf = Counterfactual {
            id,
            fact: fact.to_string(),
            intervention: intervention.to_string(),
            actual_outcome: outcome.to_string(),
            counterfactual_outcome: format!("if {} then {}", intervention, outcome),
            confidence,
            tick,
        };

        self.counterfactuals.push(cf.clone());
        if self.counterfactuals.len() > self.max_history {
            self.counterfactuals.remove(0);
        }

        cf
    }

    /// Finds the node with the strongest causal link to the given effect
    pub fn most_likely_cause(&self, effect_id: usize) -> Option<(usize, f64)> {
        let mut candidates: Vec<(usize, f64)> = Vec::new();

        for edge in &self.model.edges {
            if edge.to == effect_id {
                candidates.push((edge.from, edge.strength.abs()));
            }
        }

        candidates
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut dc = DoCalculus::new();
        let id = dc.add_node("X", 1.0);
        assert_eq!(id, 1);
        assert_eq!(dc.nodes.len(), 1);
        assert_eq!(dc.nodes[0].name, "X");
        assert!((dc.nodes[0].value - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_add_edge() {
        let mut dc = DoCalculus::new();
        let n1 = dc.add_node("A", 1.0);
        let n2 = dc.add_node("B", 0.0);
        dc.add_edge(n1, n2, 0.8, 1);
        assert_eq!(dc.edges.len(), 1);
        assert_eq!(dc.edges[0].from, n1);
        assert_eq!(dc.edges[0].to, n2);
        assert!((dc.edges[0].strength - 0.8).abs() < 1e-9);
        assert_eq!(dc.edges[0].delay, 1);
    }

    #[test]
    fn test_do_intervention_changes_value() {
        let mut dc = DoCalculus::new();
        let id = dc.add_node("X", 0.0);
        assert!((dc.nodes[0].value - 0.0).abs() < 1e-9);
        assert!(!dc.nodes[0].intervened);

        dc.do_intervention(id, 5.0);
        assert!((dc.nodes[0].value - 5.0).abs() < 1e-9);
        assert!(dc.nodes[0].intervened);
    }

    #[test]
    fn test_predict_effect() {
        let mut dc = DoCalculus::new();
        let x = dc.add_node("X", 1.0);
        let y = dc.add_node("Y", 0.0);
        dc.add_edge(x, y, 1.0, 1);

        dc.do_intervention(x, 5.0);
        let predicted = dc.predict_effect(x, y, 2);

        assert!((predicted - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_conditional_probability() {
        let mut dc = DoCalculus::new();
        let x = dc.add_node("X", 1.0);
        let y = dc.add_node("Y", 0.0);
        dc.add_edge(x, y, 1.0, 0);

        let p = dc.conditional_probability(x, y, &[]);
        assert!((p - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_backdoor_criterion() {
        let mut dc = DoCalculus::new();
        let x = dc.add_node("X", 1.0);
        let y = dc.add_node("Y", 0.0);
        dc.add_edge(x, y, 1.0, 1);

        assert!(dc.backdoor_criterion(x, y));
    }

    #[test]
    fn test_counterfactual() {
        let mut cr = CausalReasoner::new();
        let cf = cr.counterfactual("I chose A", "choose B", "outcome O2", 42);

        assert_eq!(cf.fact, "I chose A");
        assert_eq!(cf.intervention, "choose B");
        assert_eq!(cf.tick, 42);
        assert_eq!(cr.counterfactuals.len(), 1);
    }

    #[test]
    fn test_most_likely_cause() {
        let mut cr = CausalReasoner::new();
        let x = cr.model.add_node("X", 1.0);
        let y = cr.model.add_node("Y", 0.0);
        let z = cr.model.add_node("Z", 0.0);
        cr.model.add_edge(x, y, 0.9, 1);
        cr.model.add_edge(z, y, 0.3, 1);

        let result = cr.most_likely_cause(y);
        assert!(result.is_some());
        let (cause_id, strength) = result.unwrap();
        assert_eq!(cause_id, x);
        assert!((strength - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_graphviz_output() {
        let mut dc = DoCalculus::new();
        let x = dc.add_node("X", 1.0);
        let y = dc.add_node("Y", 0.0);
        dc.add_edge(x, y, 0.8, 1);

        let dot = dc.graphviz();
        assert!(dot.starts_with("digraph CausalModel {"));
        assert!(dot.contains("X"));
        assert!(dot.contains("Y"));
        assert!(dot.contains("s=0.80"));
        assert!(dot.contains("d=1"));
        assert!(dot.ends_with("}\n"));
    }

    #[test]
    fn test_frontdoor_criterion() {
        let mut dc = DoCalculus::new();
        let x = dc.add_node("X", 1.0);
        let m = dc.add_node("M", 0.0);
        let y = dc.add_node("Y", 0.0);
        dc.add_edge(x, m, 1.0, 1);
        dc.add_edge(m, y, 1.0, 1);

        assert!(dc.frontdoor_criterion(x, y));
    }
}

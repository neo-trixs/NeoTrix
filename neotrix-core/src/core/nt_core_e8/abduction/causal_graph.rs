#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalNode {
    pub id: usize,
    pub description: String,
    pub confidence: f64,
    pub observed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub from: usize,
    pub to: usize,
    pub relation: String,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraph {
    pub nodes: Vec<CausalNode>,
    pub edges: Vec<CausalEdge>,
    pub adjacency: HashMap<usize, Vec<(usize, f64)>>,
    next_id: usize,
}

impl CausalGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            adjacency: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_node(&mut self, desc: String, confidence: f64) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let confidence = confidence.max(0.0).min(1.0);
        self.nodes.push(CausalNode {
            id,
            description: desc,
            confidence,
            observed: false,
        });
        self.adjacency.entry(id).or_default();
        id
    }

    pub fn add_edge(&mut self, from: usize, to: usize, relation: String, strength: f64) {
        let strength = strength.max(0.0).min(1.0);
        self.edges.push(CausalEdge {
            from,
            to,
            relation,
            strength,
        });
        self.adjacency.entry(from).or_default().push((to, strength));
    }

    pub fn find_causes(&self, node: usize) -> Vec<(usize, f64)> {
        let mut causes = Vec::new();
        for edge in &self.edges {
            if edge.to == node {
                causes.push((edge.from, edge.strength));
            }
        }
        causes
    }

    pub fn find_effects(&self, node: usize) -> Vec<(usize, f64)> {
        self.adjacency.get(&node).cloned().unwrap_or_default()
    }

    pub fn find_abductive_explanations(
        &self,
        observation: usize,
        max_depth: usize,
    ) -> Vec<Vec<usize>> {
        let mut results = Vec::new();
        let mut current_path = vec![observation];
        self.dfs_backward(observation, max_depth, &mut current_path, &mut results);
        results
    }

    fn dfs_backward(
        &self,
        node: usize,
        max_depth: usize,
        current_path: &mut Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        if current_path.len() > max_depth {
            return;
        }
        let causes = self.find_causes(node);
        if causes.is_empty() {
            results.push(current_path.clone());
            return;
        }
        for (cause, _) in &causes {
            if current_path.contains(cause) {
                results.push(current_path.clone());
                continue;
            }
            current_path.push(*cause);
            self.dfs_backward(*cause, max_depth, current_path, results);
            current_path.pop();
        }
    }

    pub fn subgraph(&self, nodes: &[usize]) -> CausalGraph {
        let node_set: std::collections::HashSet<usize> = nodes.iter().copied().collect();
        let mut sg = CausalGraph::new();
        let mut id_map = HashMap::new();
        for &old_id in nodes {
            if let Some(n) = self.nodes.iter().find(|n| n.id == old_id) {
                let new_id = sg.add_node(n.description.clone(), n.confidence);
                id_map.insert(old_id, new_id);
            }
        }
        for edge in &self.edges {
            if node_set.contains(&edge.from) && node_set.contains(&edge.to) {
                if let (Some(&nf), Some(&nt)) = (id_map.get(&edge.from), id_map.get(&edge.to)) {
                    sg.add_edge(nf, nt, edge.relation.clone(), edge.strength);
                }
            }
        }
        sg
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Load a Cortex-Brain causal graph exported by the external brain
    /// (`/Volumes/NeoTrixBrain/working/causal_graph.json`).
    ///
    /// Schema: `{ "nodes": [{ "id": "<desc>" }], "links": [{ "s": "<src>", "t": "<tgt>", "v": <strength> }], "ts": <epoch> }`.
    /// Node confidence defaults to 0.8 (external-brain claim, not yet locally observed);
    /// edges are labelled `"causal"` so the abduction engine can reason over them.
    ///
    /// Returns an error only on I/O or parse failure — a missing file is a caller concern,
    /// not a crash (Dark Forest: connect-or-no-op, never panic on absent peripheral).
    pub fn from_cortex_json(path: &Path) -> Result<CausalGraph, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("cortex causal graph read {} failed: {}", path.display(), e))?;
        let data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("cortex causal graph parse {} failed: {}", path.display(), e))?;

        let mut graph = CausalGraph::new();
        let empty = serde_json::Value::Null;
        let nodes = data.get("nodes").unwrap_or(&empty).as_array()
            .ok_or_else(|| format!("cortex causal graph {} missing 'nodes' array", path.display()))?;

        let mut id_map: HashMap<String, usize> = HashMap::new();
        for n in nodes {
            let id = n.get("id").and_then(|v| v.as_str())
                .ok_or_else(|| format!("cortex causal graph {} node missing 'id'", path.display()))?
                .to_string();
            let idx = graph.add_node(id.clone(), 0.8);
            id_map.insert(id, idx);
        }

        let links = data.get("links").unwrap_or(&empty).as_array()
            .ok_or_else(|| format!("cortex causal graph {} missing 'links' array", path.display()))?;
        for l in links {
            let s = l.get("s").and_then(|v| v.as_str())
                .ok_or_else(|| format!("cortex causal graph {} link missing 's'", path.display()))?
                .to_string();
            let t = l.get("t").and_then(|v| v.as_str())
                .ok_or_else(|| format!("cortex causal graph {} link missing 't'", path.display()))?
                .to_string();
            let v = l.get("v").and_then(|x| x.as_f64()).unwrap_or(0.5);
            if let (Some(&a), Some(&b)) = (id_map.get(&s), id_map.get(&t)) {
                graph.add_edge(a, b, "causal".to_string(), v);
            }
        }

        Ok(graph)
    }
}

impl Default for CausalGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let g = CausalGraph::new();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut g = CausalGraph::new();
        let id = g.add_node("test".into(), 0.9);
        assert_eq!(id, 0);
        assert_eq!(g.node_count(), 1);
        assert!((g.nodes[0].confidence - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_add_edge() {
        let mut g = CausalGraph::new();
        let a = g.add_node("A".into(), 1.0);
        let b = g.add_node("B".into(), 1.0);
        g.add_edge(a, b, "causes".into(), 0.8);
        assert_eq!(g.edge_count(), 1);
        let effects = g.find_effects(a);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].0, b);
    }

    #[test]
    fn test_find_causes() {
        let mut g = CausalGraph::new();
        let a = g.add_node("A".into(), 1.0);
        let b = g.add_node("B".into(), 1.0);
        g.add_edge(a, b, "causes".into(), 0.7);
        let causes = g.find_causes(b);
        assert_eq!(causes.len(), 1);
        assert_eq!(causes[0].0, a);
    }

    #[test]
    fn test_abductive_explanations_single_hop() {
        let mut g = CausalGraph::new();
        let a = g.add_node("root".into(), 1.0);
        let b = g.add_node("obs".into(), 1.0);
        g.add_edge(a, b, "causes".into(), 0.9);
        let paths = g.find_abductive_explanations(b, 5);
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|p| p.contains(&a) && p.contains(&b)));
    }

    #[test]
    fn test_abductive_explanations_multi_hop() {
        let mut g = CausalGraph::new();
        let r1 = g.add_node("root1".into(), 0.9);
        let r2 = g.add_node("root2".into(), 0.8);
        let mid = g.add_node("mid".into(), 0.7);
        let obs = g.add_node("obs".into(), 1.0);
        g.add_edge(r1, mid, "causes".into(), 0.8);
        g.add_edge(r2, mid, "causes".into(), 0.6);
        g.add_edge(mid, obs, "causes".into(), 0.9);
        let paths = g.find_abductive_explanations(obs, 5);
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|p| p.len() == 3));
    }

    #[test]
    fn test_abductive_explanations_max_depth() {
        let mut g = CausalGraph::new();
        let a = g.add_node("a".into(), 1.0);
        let b = g.add_node("b".into(), 1.0);
        let c = g.add_node("c".into(), 1.0);
        let d = g.add_node("d".into(), 1.0);
        g.add_edge(a, b, "".into(), 1.0);
        g.add_edge(b, c, "".into(), 1.0);
        g.add_edge(c, d, "".into(), 1.0);
        let paths = g.find_abductive_explanations(d, 2);
        for p in &paths {
            assert!(p.len() <= 3);
        }
    }

    #[test]
    fn test_subgraph() {
        let mut g = CausalGraph::new();
        let a = g.add_node("A".into(), 1.0);
        let b = g.add_node("B".into(), 1.0);
        let c = g.add_node("C".into(), 1.0);
        g.add_edge(a, b, "x".into(), 0.5);
        g.add_edge(b, c, "y".into(), 0.6);
        let sg = g.subgraph(&[a, b]);
        assert_eq!(sg.node_count(), 2);
        assert_eq!(sg.edge_count(), 1);
    }

    #[test]
    fn test_no_explanations_for_orphan() {
        let mut g = CausalGraph::new();
        let n = g.add_node("lonely".into(), 1.0);
        let paths = g.find_abductive_explanations(n, 5);
        assert!(!paths.is_empty());
        assert_eq!(paths[0], vec![n]);
    }

    #[test]
    fn test_from_cortex_json_inline() {
        let json = serde_json::json!({
            "nodes": [
                {"id": "Nomad Remedy [home]: Honey"},
                {"id": "Cough (Cold, cough & allergy)"},
                {"id": "symptom relief expected"}
            ],
            "links": [
                {"s": "Nomad Remedy [home]: Honey", "t": "Cough (Cold, cough & allergy)", "v": 0.7},
                {"s": "Cough (Cold, cough & allergy)", "t": "symptom relief expected", "v": 0.56}
            ],
            "ts": 1700000000i64
        });
        let tmp = std::env::temp_dir().join("cortex_test_causal.json");
        std::fs::write(&tmp, serde_json::to_string(&json).unwrap()).unwrap();

        let g = CausalGraph::from_cortex_json(&tmp).expect("load should succeed");
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
        // Honey -> Cough -> relief: abduction from the leaf should surface Honey
        let relief = g.nodes.iter().position(|n| n.description.contains("relief expected")).unwrap();
        let paths = g.find_abductive_explanations(relief, 5);
        assert!(paths.iter().any(|p| p.len() == 3));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_from_cortex_json_missing_file_errors() {
        let res = CausalGraph::from_cortex_json(std::path::Path::new("/nonexistent/cortex.json"));
        assert!(res.is_err());
    }
}

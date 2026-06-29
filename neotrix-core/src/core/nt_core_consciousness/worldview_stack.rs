use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WorldviewLayer {
    Facts,
    Generalizations,
    Principles,
    Values,
    Metaphysics,
}

impl WorldviewLayer {
    fn index(&self) -> usize {
        match self {
            WorldviewLayer::Facts => 0,
            WorldviewLayer::Generalizations => 1,
            WorldviewLayer::Principles => 2,
            WorldviewLayer::Values => 3,
            WorldviewLayer::Metaphysics => 4,
        }
    }

    #[allow(dead_code)]
    fn from_index(i: usize) -> Self {
        match i {
            0 => WorldviewLayer::Facts,
            1 => WorldviewLayer::Generalizations,
            2 => WorldviewLayer::Principles,
            3 => WorldviewLayer::Values,
            _ => WorldviewLayer::Metaphysics,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorldviewNode {
    pub node_vsa: Vec<u8>,
    pub label: String,
    pub layer: WorldviewLayer,
    pub parent_nodes: Vec<usize>,
    pub child_nodes: Vec<usize>,
    pub confidence: f64,
    pub stability: f64,
    pub last_updated: u64,
    pub grounding_strength: f64,
}

#[derive(Debug, Clone)]
pub struct WorldviewStack {
    pub nodes: Vec<WorldviewNode>,
    pub max_nodes: usize,
    pub min_confidence_for_promotion: f64,
    pub upward_decay: f64,
    pub coherence_threshold: f64,
    pub cycle_count: u64,
    pub self_layer_key: Vec<u8>,
}

impl WorldviewStack {
    pub fn new() -> Self {
        let self_layer_key = QuantizedVSA::seeded_random(42, VSA_DIM);
        WorldviewStack {
            nodes: Vec::new(),
            max_nodes: 200,
            min_confidence_for_promotion: 0.6,
            upward_decay: 0.15,
            coherence_threshold: 0.4,
            cycle_count: 0,
            self_layer_key,
        }
    }

    pub fn add_node(
        &mut self,
        node_vsa: Vec<u8>,
        label: &str,
        layer: WorldviewLayer,
        parent_ids: &[usize],
        confidence: f64,
    ) -> usize {
        if self.nodes.len() >= self.max_nodes {
            let oldest = self
                .nodes
                .iter()
                .enumerate()
                .min_by_key(|(_, n)| n.last_updated)
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.nodes.remove(oldest);
            for node in &mut self.nodes {
                node.parent_nodes.retain(|&p| p != oldest);
                node.child_nodes.retain(|&c| c != oldest);
                for p in &mut node.parent_nodes {
                    if *p > oldest {
                        *p -= 1;
                    }
                }
                for c in &mut node.child_nodes {
                    if *c > oldest {
                        *c -= 1;
                    }
                }
            }
        }

        let layer_index = layer.index();
        let decayed_confidence = confidence * (1.0 - self.upward_decay).powi(layer_index as i32);
        let id = self.nodes.len();

        let child_ids: Vec<usize> = parent_ids
            .iter()
            .filter(|&&p| p < self.nodes.len())
            .copied()
            .collect();

        let grounding = if child_ids.is_empty() {
            0.0
        } else {
            let total: f64 = child_ids.iter().map(|&c| self.nodes[c].confidence).sum();
            total / child_ids.len() as f64
        };

        for &child in &child_ids {
            if !self.nodes[child].parent_nodes.contains(&id) {
                self.nodes[child].parent_nodes.push(id);
            }
        }

        // Adjust parent_ids to point to current indices after possible removals
        // parent_ids are assumed to refer to nodes already in the stack

        self.nodes.push(WorldviewNode {
            node_vsa,
            label: label.to_string(),
            layer,
            parent_nodes: parent_ids.to_vec(),
            child_nodes: child_ids,
            confidence: decayed_confidence.clamp(0.0, 1.0),
            stability: 0.5,
            last_updated: self.cycle_count,
            grounding_strength: grounding,
        });

        id
    }

    pub fn induce_generalization(&mut self, fact_ids: &[usize]) -> Option<usize> {
        if fact_ids.len() < 2 {
            return None;
        }

        let facts: Vec<&WorldviewNode> = fact_ids
            .iter()
            .filter_map(|&id| self.nodes.get(id))
            .filter(|n| n.layer == WorldviewLayer::Facts)
            .collect();

        if facts.len() < 2 {
            return None;
        }

        let mut min_sim = 1.0f64;
        for i in 0..facts.len() {
            for j in (i + 1)..facts.len() {
                let sim = QuantizedVSA::similarity(&facts[i].node_vsa, &facts[j].node_vsa);
                if sim < min_sim {
                    min_sim = sim;
                }
            }
        }

        if min_sim < self.coherence_threshold {
            return None;
        }

        let vsas: Vec<&[u8]> = facts.iter().map(|n| n.node_vsa.as_slice()).collect();
        let bundled = QuantizedVSA::bundle(&vsas);

        let avg_confidence: f64 =
            facts.iter().map(|n| n.confidence).sum::<f64>() / facts.len() as f64;
        let label = format!(
            "generalization({})",
            facts
                .iter()
                .map(|n| n.label.as_str())
                .collect::<Vec<&str>>()
                .join("+")
        );

        let id = self.add_node(
            bundled,
            &label,
            WorldviewLayer::Generalizations,
            &[],
            avg_confidence,
        );

        // link back to facts as children
        for &fid in fact_ids {
            if fid < self.nodes.len() {
                if !self.nodes[fid].parent_nodes.contains(&id) {
                    self.nodes[fid].parent_nodes.push(id);
                }
                if !self.nodes[id].child_nodes.contains(&fid) {
                    self.nodes[id].child_nodes.push(fid);
                }
            }
        }

        Some(id)
    }

    pub fn find_contradictions(&self) -> Vec<(usize, usize, f64)> {
        let mut result = Vec::new();
        let len = self.nodes.len();
        for i in 0..len {
            for j in (i + 1)..len {
                if self.nodes[i].layer == self.nodes[j].layer {
                    let sim =
                        QuantizedVSA::similarity(&self.nodes[i].node_vsa, &self.nodes[j].node_vsa);
                    if sim < self.coherence_threshold {
                        result.push((i, j, sim));
                    }
                }
            }
        }
        result
    }

    pub fn resolve_contradiction(&mut self, node_a: usize, node_b: usize) -> Option<usize> {
        if node_a >= self.nodes.len() || node_b >= self.nodes.len() {
            return None;
        }
        if self.nodes[node_a].layer != self.nodes[node_b].layer {
            return None;
        }

        let (keep, remove) = if self.nodes[node_a].stability >= self.nodes[node_b].stability {
            (node_a, node_b)
        } else {
            (node_b, node_a)
        };

        // reroute references from remove to keep
        for node in &mut self.nodes {
            for p in &mut node.parent_nodes {
                if *p == remove {
                    *p = keep;
                } else if *p > remove {
                    *p -= 1;
                }
            }
            for c in &mut node.child_nodes {
                if *c == remove {
                    *c = keep;
                } else if *c > remove {
                    *c -= 1;
                }
            }
        }

        // deduplicate
        if let Some(n) = self.nodes.get_mut(keep) {
            n.parent_nodes.sort();
            n.parent_nodes.dedup();
            n.parent_nodes.retain(|&x| x != keep);
            n.child_nodes.sort();
            n.child_nodes.dedup();
            n.child_nodes.retain(|&x| x != keep);
        }

        self.nodes.remove(remove);
        Some(keep)
    }

    pub fn layer_nodes(&self, layer: WorldviewLayer) -> Vec<&WorldviewNode> {
        self.nodes.iter().filter(|n| n.layer == layer).collect()
    }

    pub fn traverse_up(&self, start_id: usize) -> Vec<&WorldviewNode> {
        if start_id >= self.nodes.len() {
            return Vec::new();
        }
        let mut result = Vec::new();
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = vec![start_id];
        while let Some(id) = stack.pop() {
            if id >= self.nodes.len() || visited[id] {
                continue;
            }
            visited[id] = true;
            result.push(&self.nodes[id]);
            for &p in &self.nodes[id].parent_nodes {
                if p < self.nodes.len() && !visited[p] {
                    stack.push(p);
                }
            }
        }
        result.reverse();
        result
    }

    pub fn traverse_down(&self, start_id: usize) -> Vec<&WorldviewNode> {
        if start_id >= self.nodes.len() {
            return Vec::new();
        }
        let mut result = Vec::new();
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = vec![start_id];
        while let Some(id) = stack.pop() {
            if id >= self.nodes.len() || visited[id] {
                continue;
            }
            visited[id] = true;
            result.push(&self.nodes[id]);
            for &c in &self.nodes[id].child_nodes {
                if c < self.nodes.len() && !visited[c] {
                    stack.push(c);
                }
            }
        }
        result
    }

    pub fn coherence(&self) -> f64 {
        let n = self.nodes.len();
        if n < 2 {
            return 1.0;
        }
        let mut total = 0u64;
        let mut coherent = 0u64;
        for i in 0..n {
            for j in (i + 1)..n {
                if self.nodes[i].layer == self.nodes[j].layer {
                    total += 1;
                    let sim =
                        QuantizedVSA::similarity(&self.nodes[i].node_vsa, &self.nodes[j].node_vsa);
                    if sim >= self.coherence_threshold {
                        coherent += 1;
                    }
                }
            }
        }
        if total == 0 {
            return 1.0;
        }
        coherent as f64 / total as f64
    }

    pub fn layer_count(&self, layer: WorldviewLayer) -> usize {
        self.nodes.iter().filter(|n| n.layer == layer).count()
    }

    pub fn total_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn reset(&mut self) {
        self.nodes.clear();
        self.cycle_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fact_vsa(label: &str) -> Vec<u8> {
        let seed = label
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    #[test]
    fn test_new_stack_defaults() {
        let ws = WorldviewStack::new();
        assert_eq!(ws.max_nodes, 200);
        assert_eq!(ws.nodes.len(), 0);
        assert!((ws.min_confidence_for_promotion - 0.6).abs() < 1e-9);
        assert!((ws.upward_decay - 0.15).abs() < 1e-9);
        assert!((ws.coherence_threshold - 0.4).abs() < 1e-9);
        assert_eq!(ws.cycle_count, 0);
    }

    #[test]
    fn test_add_node() {
        let mut ws = WorldviewStack::new();
        let vsa = make_fact_vsa("water boils at 100C");
        let id = ws.add_node(
            vsa.clone(),
            "water boils at 100C",
            WorldviewLayer::Facts,
            &[],
            0.9,
        );
        assert_eq!(id, 0);
        assert_eq!(ws.nodes.len(), 1);
        assert_eq!(ws.nodes[0].node_vsa, vsa);
        assert_eq!(ws.nodes[0].label, "water boils at 100C");
        assert_eq!(ws.nodes[0].layer, WorldviewLayer::Facts);
        assert!((ws.nodes[0].confidence - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_add_node_layers() {
        let mut ws = WorldviewStack::new();
        let f_id = ws.add_node(
            make_fact_vsa("fact"),
            "fact",
            WorldviewLayer::Facts,
            &[],
            0.9,
        );
        let g_id = ws.add_node(
            make_fact_vsa("gen"),
            "gen",
            WorldviewLayer::Generalizations,
            &[],
            0.8,
        );
        let p_id = ws.add_node(
            make_fact_vsa("principle"),
            "principle",
            WorldviewLayer::Principles,
            &[],
            0.7,
        );
        let v_id = ws.add_node(
            make_fact_vsa("value"),
            "value",
            WorldviewLayer::Values,
            &[],
            0.6,
        );
        let m_id = ws.add_node(
            make_fact_vsa("meta"),
            "meta",
            WorldviewLayer::Metaphysics,
            &[],
            0.5,
        );
        assert_eq!(ws.nodes[f_id].layer, WorldviewLayer::Facts);
        assert_eq!(ws.nodes[g_id].layer, WorldviewLayer::Generalizations);
        assert_eq!(ws.nodes[p_id].layer, WorldviewLayer::Principles);
        assert_eq!(ws.nodes[v_id].layer, WorldviewLayer::Values);
        assert_eq!(ws.nodes[m_id].layer, WorldviewLayer::Metaphysics);
    }

    #[test]
    fn test_add_node_parent_child_links() {
        let mut ws = WorldviewStack::new();
        let f1 = ws.add_node(make_fact_vsa("f1"), "f1", WorldviewLayer::Facts, &[], 0.9);
        let f2 = ws.add_node(make_fact_vsa("f2"), "f2", WorldviewLayer::Facts, &[], 0.9);
        let gen = ws.add_node(
            make_fact_vsa("gen"),
            "gen",
            WorldviewLayer::Generalizations,
            &[f1, f2],
            0.8,
        );
        assert!(ws.nodes[gen].child_nodes.contains(&f1));
        assert!(ws.nodes[gen].child_nodes.contains(&f2));
        assert!(ws.nodes[f1].parent_nodes.contains(&gen));
        assert!(ws.nodes[f2].parent_nodes.contains(&gen));
    }

    #[test]
    fn test_induce_generalization_from_similar_facts() {
        let mut ws = WorldviewStack::new();
        // Use same seed so facts are identical → high similarity
        let vsa1 = QuantizedVSA::seeded_random(100, VSA_DIM);
        let vsa2 = QuantizedVSA::seeded_random(100, VSA_DIM);
        let f1 = ws.add_node(vsa1, "fact1", WorldviewLayer::Facts, &[], 0.9);
        let f2 = ws.add_node(vsa2, "fact2", WorldviewLayer::Facts, &[], 0.9);
        let gen = ws.induce_generalization(&[f1, f2]);
        assert!(gen.is_some());
        let gen_id = gen.unwrap();
        assert_eq!(ws.nodes[gen_id].layer, WorldviewLayer::Generalizations);
        assert!(ws.nodes[gen_id].child_nodes.contains(&f1));
        assert!(ws.nodes[gen_id].child_nodes.contains(&f2));
    }

    #[test]
    fn test_induce_generalization_no_match() {
        let mut ws = WorldviewStack::new();
        let vsa1 = QuantizedVSA::seeded_random(100, VSA_DIM);
        let vsa2 = QuantizedVSA::seeded_random(999, VSA_DIM);
        let f1 = ws.add_node(vsa1, "fact1", WorldviewLayer::Facts, &[], 0.9);
        let f2 = ws.add_node(vsa2, "fact2", WorldviewLayer::Facts, &[], 0.9);
        let gen = ws.induce_generalization(&[f1, f2]);
        assert!(gen.is_none());
    }

    #[test]
    fn test_find_contradictions() {
        let mut ws = WorldviewStack::new();
        let vsa1 = QuantizedVSA::seeded_random(100, VSA_DIM);
        let vsa2 = QuantizedVSA::seeded_random(999, VSA_DIM);
        ws.add_node(vsa1, "belief_a", WorldviewLayer::Values, &[], 0.8);
        ws.add_node(vsa2, "belief_b", WorldviewLayer::Values, &[], 0.8);
        let contradictions = ws.find_contradictions();
        assert_eq!(contradictions.len(), 1);
        assert_eq!(contradictions[0].0, 0);
        assert_eq!(contradictions[0].1, 1);
    }

    #[test]
    fn test_find_no_contradictions_when_coherent() {
        let mut ws = WorldviewStack::new();
        let vsa = QuantizedVSA::seeded_random(100, VSA_DIM);
        ws.add_node(vsa.clone(), "belief_a", WorldviewLayer::Values, &[], 0.8);
        ws.add_node(vsa, "belief_b", WorldviewLayer::Values, &[], 0.8);
        let contradictions = ws.find_contradictions();
        assert_eq!(contradictions.len(), 0);
    }

    #[test]
    fn test_resolve_contradiction_keeps_higher_stability() {
        let mut ws = WorldviewStack::new();
        let vsa1 = QuantizedVSA::seeded_random(100, VSA_DIM);
        let vsa2 = QuantizedVSA::seeded_random(999, VSA_DIM);
        let id_a = ws.add_node(vsa1, "a", WorldviewLayer::Values, &[], 0.8);
        let id_b = ws.add_node(vsa2, "b", WorldviewLayer::Values, &[], 0.8);
        ws.nodes[id_a].stability = 0.9;
        ws.nodes[id_b].stability = 0.3;
        let kept = ws.resolve_contradiction(id_a, id_b);
        assert!(kept.is_some());
        assert_eq!(ws.nodes.len(), 1);
        assert_eq!(ws.nodes[0].label, "a");
    }

    #[test]
    fn test_traverse_up() {
        let mut ws = WorldviewStack::new();
        let f1 = ws.add_node(make_fact_vsa("f1"), "f1", WorldviewLayer::Facts, &[], 0.9);
        let gen = ws.add_node(
            make_fact_vsa("gen"),
            "gen",
            WorldviewLayer::Generalizations,
            &[f1],
            0.8,
        );
        let prin = ws.add_node(
            make_fact_vsa("prin"),
            "prin",
            WorldviewLayer::Principles,
            &[gen],
            0.7,
        );
        let path = ws.traverse_up(prin);
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].label, "gen");
        assert_eq!(path[1].label, "prin");
    }

    #[test]
    fn test_traverse_down() {
        let mut ws = WorldviewStack::new();
        let f1 = ws.add_node(make_fact_vsa("f1"), "f1", WorldviewLayer::Facts, &[], 0.9);
        let gen = ws.add_node(
            make_fact_vsa("gen"),
            "gen",
            WorldviewLayer::Generalizations,
            &[f1],
            0.8,
        );
        let path = ws.traverse_down(gen);
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].label, "gen");
        assert_eq!(path[1].label, "f1");
    }

    #[test]
    fn test_coherence_score() {
        let mut ws = WorldviewStack::new();
        let vsa = QuantizedVSA::seeded_random(100, VSA_DIM);
        ws.add_node(vsa.clone(), "a", WorldviewLayer::Values, &[], 0.8);
        ws.add_node(vsa, "b", WorldviewLayer::Values, &[], 0.8);
        let c = ws.coherence();
        assert!((c - 1.0).abs() < 1e-9);
    }
}

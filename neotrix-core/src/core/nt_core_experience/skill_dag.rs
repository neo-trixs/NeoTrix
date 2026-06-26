use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Debug)]
pub struct SkillNode {
    pub id: u64,
    pub name: String,
    pub query: Vec<u8>,
    pub success_criteria: Vec<u8>,
    pub prerequisites: Vec<u64>,
    pub depth: usize,
    pub pass_count: usize,
    pub fail_count: usize,
    pub mutation_count: usize,
    pub avg_success_similarity: f64,
}

impl SkillNode {
    pub fn success_rate(&self) -> f64 {
        let total = self.pass_count + self.fail_count;
        if total == 0 {
            0.5
        } else {
            self.pass_count as f64 / total as f64
        }
    }

    pub fn is_mastered(&self) -> bool {
        self.pass_count >= 10 && self.success_rate() > 0.8
    }
}

pub struct SkillDagArchive {
    pub nodes: HashMap<u64, SkillNode>,
    pub edges: HashMap<u64, Vec<u64>>,
    pub reverse_edges: HashMap<u64, Vec<u64>>,
    next_id: u64,
}

impl SkillDagArchive {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
            next_id: 1,
        }
    }

    fn random_vsa(&self) -> Vec<u8> {
        QuantizedVSA::seeded_random(self.next_id, 4096)
    }

    pub fn add_skill(&mut self, name: &str, prerequisites: &[u64]) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let query = self.random_vsa();
        let success_criteria = self.random_vsa();
        let depth = if prerequisites.is_empty() {
            0
        } else {
            prerequisites
                .iter()
                .filter_map(|p| self.nodes.get(p))
                .map(|n| n.depth)
                .max()
                .unwrap_or(0)
                + 1
        };
        let node = SkillNode {
            id,
            name: name.to_string(),
            query,
            success_criteria,
            prerequisites: prerequisites.to_vec(),
            depth,
            pass_count: 0,
            fail_count: 0,
            mutation_count: 0,
            avg_success_similarity: 0.0,
        };
        self.nodes.insert(id, node);
        self.edges.entry(id).or_default();
        for p in prerequisites {
            self.edges.entry(*p).or_default().push(id);
            self.reverse_edges.entry(id).or_default().push(*p);
        }
        id
    }

    pub fn record_outcome(&mut self, id: u64, success: bool, similarity: f64) {
        if let Some(node) = self.nodes.get_mut(&id) {
            if success {
                node.pass_count += 1;
            } else {
                node.fail_count += 1;
            }
            let alpha = 0.9;
            node.avg_success_similarity =
                alpha * node.avg_success_similarity + (1.0 - alpha) * similarity;
        }
    }

    pub fn discover_skill(&self) -> u64 {
        let mut frontier: Vec<u64> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.is_mastered())
            .map(|(id, _)| *id)
            .collect();
        frontier.sort_by(|a, b| {
            self.nodes
                .get(b)
                .map(|n| n.depth)
                .unwrap_or(0)
                .cmp(&self.nodes.get(a).map(|n| n.depth).unwrap_or(0))
        });
        let _seed = frontier.first().copied().unwrap_or(0);
        let id = self.next_id + 1;
        id
    }

    pub fn evolve_skill(&mut self, id: u64, candidate_name: &str) -> u64 {
        let prereqs: Vec<u64> = self
            .nodes
            .get(&id)
            .map(|n| n.prerequisites.clone())
            .unwrap_or_default();
        let parent_mutations = self
            .nodes
            .get(&id)
            .map(|n| n.mutation_count + 1)
            .unwrap_or(1);
        let child_id = self.add_skill(&format!("{}_evolved_{}", candidate_name, id), &prereqs);
        if let Some(child) = self.nodes.get_mut(&child_id) {
            child.mutation_count = parent_mutations;
        }
        child_id
    }

    pub fn dag_diversity(&self) -> f64 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        let depths: Vec<usize> = self.nodes.values().map(|n| n.depth).collect();
        let mean = depths.iter().sum::<usize>() as f64 / depths.len() as f64;
        let variance = depths
            .iter()
            .map(|d| (*d as f64 - mean).powi(2))
            .sum::<f64>()
            / depths.len() as f64;
        variance.sqrt() / (mean + 1.0)
    }

    pub fn topological_sort(&self) -> Vec<u64> {
        let mut in_degree: HashMap<u64, usize> = self.nodes.keys().map(|k| (*k, 0)).collect();
        for (_, children) in &self.edges {
            for child in children {
                *in_degree.entry(*child).or_insert(0) += 1;
            }
        }
        let mut queue: VecDeque<u64> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(id, _)| *id)
            .collect();
        let mut result = Vec::new();
        while let Some(id) = queue.pop_front() {
            result.push(id);
            if let Some(children) = self.edges.get(&id) {
                for child in children {
                    if let Some(d) = in_degree.get_mut(child) {
                        *d -= 1;
                        if *d == 0 {
                            queue.push_back(*child);
                        }
                    }
                }
            }
        }
        result
    }

    pub fn shortest_path_to(&self, from: u64, to: u64) -> Option<Vec<u64>> {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return None;
        }
        let mut visited: HashSet<u64> = HashSet::new();
        let mut queue: VecDeque<(u64, Vec<u64>)> = VecDeque::new();
        queue.push_back((from, vec![from]));
        while let Some((current, path)) = queue.pop_front() {
            if current == to {
                return Some(path);
            }
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            if let Some(children) = self.edges.get(&current) {
                for child in children {
                    if !visited.contains(child) {
                        let mut new_path = path.clone();
                        new_path.push(*child);
                        queue.push_back((*child, new_path));
                    }
                }
            }
        }
        None
    }

    pub fn stat_summary(&self) -> DagStats {
        let mastered = self.nodes.values().filter(|n| n.is_mastered()).count();
        let total_depth: usize = self.nodes.values().map(|n| n.depth).sum();
        let avg_depth = if self.nodes.is_empty() {
            0.0
        } else {
            total_depth as f64 / self.nodes.len() as f64
        };
        let avg_success = if self.nodes.is_empty() {
            0.0
        } else {
            self.nodes.values().map(|n| n.success_rate()).sum::<f64>() / self.nodes.len() as f64
        };
        DagStats {
            total_nodes: self.nodes.len(),
            mastered_nodes: mastered,
            max_depth: self.nodes.values().map(|n| n.depth).max().unwrap_or(0),
            avg_depth,
            avg_success_rate: avg_success,
            dag_diversity: self.dag_diversity(),
            total_mutations: self.nodes.values().map(|n| n.mutation_count).sum(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DagStats {
    pub total_nodes: usize,
    pub mastered_nodes: usize,
    pub max_depth: usize,
    pub avg_depth: f64,
    pub avg_success_rate: f64,
    pub dag_diversity: f64,
    pub total_mutations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_skill_no_prereqs() {
        let mut dag = SkillDagArchive::new();
        let id = dag.add_skill("grasp", &[]);
        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.nodes[&id].depth, 0);
        assert_eq!(dag.nodes[&id].name, "grasp");
    }

    #[test]
    fn test_add_skill_with_prereqs() {
        let mut dag = SkillDagArchive::new();
        let grasp = dag.add_skill("grasp", &[]);
        let lift = dag.add_skill("lift", &[grasp]);
        assert_eq!(dag.nodes[&lift].depth, 1);
        assert_eq!(dag.nodes[&lift].prerequisites, vec![grasp]);
    }

    #[test]
    fn test_skill_depth_chain() {
        let mut dag = SkillDagArchive::new();
        let a = dag.add_skill("a", &[]);
        let b = dag.add_skill("b", &[a]);
        let c = dag.add_skill("c", &[b]);
        assert_eq!(dag.nodes[&a].depth, 0);
        assert_eq!(dag.nodes[&b].depth, 1);
        assert_eq!(dag.nodes[&c].depth, 2);
    }

    #[test]
    fn test_success_rate() {
        let mut dag = SkillDagArchive::new();
        let id = dag.add_skill("test", &[]);
        for _ in 0..8 {
            dag.record_outcome(id, true, 0.9);
        }
        for _ in 0..2 {
            dag.record_outcome(id, false, 0.3);
        }
        assert!((dag.nodes[&id].success_rate() - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_is_mastered() {
        let mut dag = SkillDagArchive::new();
        let id = dag.add_skill("test", &[]);
        for _ in 0..10 {
            dag.record_outcome(id, true, 0.9);
        }
        assert!(dag.nodes[&id].is_mastered());
    }

    #[test]
    fn test_not_mastered_with_low_success() {
        let mut dag = SkillDagArchive::new();
        let id = dag.add_skill("test", &[]);
        for _ in 0..10 {
            dag.record_outcome(id, false, 0.3);
        }
        assert!(!dag.nodes[&id].is_mastered());
    }

    #[test]
    fn test_evolve_skill() {
        let mut dag = SkillDagArchive::new();
        let parent = dag.add_skill("parent", &[]);
        let child = dag.evolve_skill(parent, "parent");
        assert_eq!(dag.nodes[&child].name, "parent_evolved_1");
        assert!(dag.nodes[&child].mutation_count >= 1);
    }

    #[test]
    fn test_dag_diversity() {
        let mut dag = SkillDagArchive::new();
        let diversity_empty = dag.dag_diversity();
        assert!((diversity_empty - 0.0).abs() < 1e-9);
        let a = dag.add_skill("a", &[]);
        let _b = dag.add_skill("b", &[a]);
        let diversity_nonempty = dag.dag_diversity();
        assert!(diversity_nonempty > 0.0);
    }

    #[test]
    fn test_topological_sort() {
        let mut dag = SkillDagArchive::new();
        let a = dag.add_skill("a", &[]);
        let b = dag.add_skill("b", &[a]);
        let _c = dag.add_skill("c", &[b]);
        let sorted = dag.topological_sort();
        assert_eq!(sorted.len(), 3);
        let pos_a = sorted.iter().position(|x| *x == a).unwrap();
        let pos_b = sorted.iter().position(|x| *x == b).unwrap();
        assert!(pos_a < pos_b);
    }

    #[test]
    fn test_shortest_path() {
        let mut dag = SkillDagArchive::new();
        let a = dag.add_skill("a", &[]);
        let b = dag.add_skill("b", &[a]);
        let c = dag.add_skill("c", &[b]);
        let path = dag.shortest_path_to(a, c);
        assert!(path.is_some());
        assert_eq!(path.unwrap(), vec![a, b, c]);
    }

    #[test]
    fn test_shortest_path_nonexistent() {
        let dag = SkillDagArchive::new();
        assert!(dag.shortest_path_to(1, 99).is_none());
    }

    #[test]
    fn test_stat_summary() {
        let mut dag = SkillDagArchive::new();
        let a = dag.add_skill("a", &[]);
        let _b = dag.add_skill("b", &[a]);
        let stats = dag.stat_summary();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.max_depth, 1);
        assert!(stats.avg_depth > 0.0);
    }

    #[test]
    fn test_avg_success_similarity_ema() {
        let mut dag = SkillDagArchive::new();
        let id = dag.add_skill("test", &[]);
        dag.record_outcome(id, true, 0.8);
        dag.record_outcome(id, true, 1.0);
        let sim = dag.nodes[&id].avg_success_similarity;
        assert!(sim > 0.8 && sim < 1.0);
    }

    #[test]
    fn test_multiple_children() {
        let mut dag = SkillDagArchive::new();
        let root = dag.add_skill("root", &[]);
        let _a = dag.add_skill("a", &[root]);
        let _b = dag.add_skill("b", &[root]);
        assert_eq!(dag.nodes[&root].depth, 0);
        assert_eq!(dag.edges[&root].len(), 2);
        assert_eq!(dag.nodes.len(), 3);
    }
}

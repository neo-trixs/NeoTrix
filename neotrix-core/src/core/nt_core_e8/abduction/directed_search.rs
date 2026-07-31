#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use std::collections::{HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::causal_graph::{CausalGraph, CausalNode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchStrategy {
    Beam { width: usize },
    BFS,
    DFS,
    MCTS { simulations: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectedSearch {
    pub strategy: SearchStrategy,
    pub visited: HashSet<usize>,
    pub frontier: VecDeque<usize>,
    pub max_depth: usize,
}

impl DirectedSearch {
    pub fn new(strategy: SearchStrategy, max_depth: usize) -> Self {
        Self {
            strategy,
            visited: HashSet::new(),
            frontier: VecDeque::new(),
            max_depth,
        }
    }

    pub fn search(&mut self, graph: &CausalGraph, start: usize, goal: impl Fn(&CausalNode) -> bool) -> Vec<Vec<usize>> {
        self.reset();
        match &self.strategy {
            SearchStrategy::Beam { width } => self.beam_search(graph, start, &goal, *width),
            SearchStrategy::BFS => self.bfs_search(graph, start, &goal),
            SearchStrategy::DFS => self.dfs_search(graph, start, &goal),
            SearchStrategy::MCTS { .. } => self.dfs_search(graph, start, &goal),
        }
    }

    pub fn bfs_search(&mut self, graph: &CausalGraph, start: usize, goal: &impl Fn(&CausalNode) -> bool) -> Vec<Vec<usize>> {
        let mut results = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(vec![start]);

        while let Some(path) = queue.pop_front() {
            if path.len() > self.max_depth {
                continue;
            }
            if let Some(node) = graph.nodes.iter().find(|n| n.id == path.last().copied().unwrap_or(start)) {
                if goal(node) {
                    results.push(path.clone());
                }
            }
                let last = path.last().copied().unwrap_or(start);
            if let Some(neighbors) = graph.adjacency.get(&last) {
                for &(next, _) in neighbors {
                    if !path.contains(&next) {
                        let mut new_path = path.clone();
                        new_path.push(next);
                        queue.push_back(new_path);
                    }
                }
            }
        }

        results
    }

    pub fn dfs_search(&mut self, graph: &CausalGraph, start: usize, goal: &impl Fn(&CausalNode) -> bool) -> Vec<Vec<usize>> {
        let mut results = Vec::new();
        let mut current_path = vec![start];
        self.dfs_recursive(graph, start, goal, &mut current_path, &mut results);
        results
    }

    fn dfs_recursive(&self, graph: &CausalGraph, node: usize, goal: &impl Fn(&CausalNode) -> bool, current_path: &mut Vec<usize>, results: &mut Vec<Vec<usize>>) {
        if current_path.len() > self.max_depth {
            return;
        }
        if let Some(n) = graph.nodes.iter().find(|n| n.id == node) {
            if goal(n) {
                results.push(current_path.clone());
            }
        }
        if let Some(neighbors) = graph.adjacency.get(&node) {
            for &(next, _) in neighbors {
                if !current_path.contains(&next) {
                    current_path.push(next);
                    self.dfs_recursive(graph, next, goal, current_path, results);
                    current_path.pop();
                }
            }
        }
    }

    pub fn beam_search(&mut self, graph: &CausalGraph, start: usize, goal: &impl Fn(&CausalNode) -> bool, width: usize) -> Vec<Vec<usize>> {
        let mut results = Vec::new();
        let mut candidates = vec![vec![start]];

        for _depth in 0..self.max_depth {
            if candidates.is_empty() {
                break;
            }
            let mut all_extensions = Vec::new();
            for path in &candidates {
            let last = path.last().copied().unwrap_or(start);
                if let Some(n) = graph.nodes.iter().find(|n| n.id == last) {
                    if goal(n) {
                        results.push(path.clone());
                    }
                }
                if let Some(neighbors) = graph.adjacency.get(&last) {
                    for &(next, _) in neighbors {
                        if !path.contains(&next) {
                            let mut new_path = path.clone();
                            new_path.push(next);
                            let score = graph
                                .nodes
                                .iter()
                                .find(|n| n.id == next)
                                .map(|n| n.confidence)
                                .unwrap_or(0.0);
                            all_extensions.push((new_path, score));
                        }
                    }
                }
            }
            all_extensions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            candidates = all_extensions.into_iter().take(width).map(|(p, _)| p).collect();
        }

        results
    }

    pub fn reset(&mut self) {
        self.visited.clear();
        self.frontier.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_graph() -> CausalGraph {
        let mut g = CausalGraph::new();
        let _ = g.add_node("start".into(), 0.9);
        let _ = g.add_node("mid".into(), 0.7);
        let _ = g.add_node("goal".into(), 0.5);
        let _ = g.add_node("dead_end".into(), 0.3);
        g.add_edge(0, 1, "causes".into(), 0.8);
        g.add_edge(1, 2, "causes".into(), 0.6);
        g.add_edge(0, 3, "causes".into(), 0.4);
        g
    }

    #[test]
    fn test_bfs_search() {
        let g = make_test_graph();
        let mut s = DirectedSearch::new(SearchStrategy::BFS, 10);
        let results = s.bfs_search(&g, 0, &|n| n.id == 2);
        assert!(!results.is_empty());
        assert!(results.iter().any(|p| p == &vec![0, 1, 2]));
    }

    #[test]
    fn test_dfs_search() {
        let g = make_test_graph();
        let mut s = DirectedSearch::new(SearchStrategy::DFS, 10);
        let results = s.dfs_search(&g, 0, &|n| n.id == 2);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_beam_search() {
        let g = make_test_graph();
        let mut s = DirectedSearch::new(SearchStrategy::Beam { width: 2 }, 10);
        let results = s.beam_search(&g, 0, &|n| n.id == 2, 2);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_no_path() {
        let g = make_test_graph();
        let mut s = DirectedSearch::new(SearchStrategy::BFS, 10);
        let results = s.bfs_search(&g, 3, &|n| n.id == 0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_multiple_paths() {
        let mut g = CausalGraph::new();
        let s = g.add_node("start".into(), 1.0);
        let m1 = g.add_node("mid1".into(), 0.8);
        let m2 = g.add_node("mid2".into(), 0.7);
        let gl = g.add_node("goal".into(), 1.0);
        g.add_edge(s, m1, "causes".into(), 0.5);
        g.add_edge(s, m2, "causes".into(), 0.5);
        g.add_edge(m1, gl, "causes".into(), 0.5);
        g.add_edge(m2, gl, "causes".into(), 0.5);
        let mut srch = DirectedSearch::new(SearchStrategy::BFS, 10);
        let results = srch.bfs_search(&g, 0, &|n| n.id == 3);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_reset() {
        let mut s = DirectedSearch::new(SearchStrategy::BFS, 10);
        s.visited.insert(1);
        s.frontier.push_back(2);
        s.reset();
        assert!(s.visited.is_empty());
        assert!(s.frontier.is_empty());
    }

    #[test]
    fn test_search_dispatch() {
        let g = make_test_graph();
        let mut s = DirectedSearch::new(SearchStrategy::BFS, 10);
        let results = s.search(&g, 0, |n| n.id == 2);
        assert!(!results.is_empty());
    }
}

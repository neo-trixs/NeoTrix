use std::collections::HashMap;

use rusqlite::Connection;

use super::nt_memory_types::KnowledgeEdge;

#[derive(Debug)]
pub struct GraphCache {
    forward: HashMap<String, Vec<KnowledgeEdge>>,
    backward: HashMap<String, Vec<KnowledgeEdge>>,
    pub edge_count: usize,
    pub node_count: usize,
}

impl GraphCache {
    pub fn new(conn: &Connection) -> rusqlite::Result<Self> {
        let edges = super::nt_memory_store::get_all_edges(conn)?;
        let mut forward: HashMap<String, Vec<KnowledgeEdge>> = HashMap::new();
        let mut backward: HashMap<String, Vec<KnowledgeEdge>> = HashMap::new();
        let mut nodes = std::collections::HashSet::new();
        for edge in &edges {
            forward.entry(edge.source_id.clone()).or_default().push(edge.clone());
            backward.entry(edge.target_id.clone()).or_default().push(edge.clone());
            nodes.insert(edge.source_id.clone());
            nodes.insert(edge.target_id.clone());
        }
        Ok(Self {
            edge_count: edges.len(),
            node_count: nodes.len(),
            forward,
            backward,
        })
    }

    pub fn empty() -> Self {
        Self {
            forward: HashMap::new(),
            backward: HashMap::new(),
            edge_count: 0,
            node_count: 0,
        }
    }

    pub fn neighbors(&self, node_id: &str) -> Vec<&KnowledgeEdge> {
        let mut all: Vec<&KnowledgeEdge> = Vec::new();
        if let Some(fwd) = self.forward.get(node_id) {
            all.extend(fwd.iter());
        }
        if let Some(bwd) = self.backward.get(node_id) {
            all.extend(bwd.iter());
        }
        all
    }

    pub fn adjacent_ids(&self, node_id: &str) -> Vec<(&str, &KnowledgeEdge)> {
        let mut result: Vec<(&str, &KnowledgeEdge)> = Vec::new();
        if let Some(fwd) = self.forward.get(node_id) {
            for e in fwd {
                result.push((e.target_id.as_str(), e));
            }
        }
        if let Some(bwd) = self.backward.get(node_id) {
            for e in bwd {
                result.push((e.source_id.as_str(), e));
            }
        }
        result
    }

    pub fn insert_edge(&mut self, edge: KnowledgeEdge) {
        self.forward.entry(edge.source_id.clone()).or_default().push(edge.clone());
        self.backward.entry(edge.target_id.clone()).or_default().push(edge);
        self.edge_count += 1;
    }

    pub fn rebuild(&mut self, conn: &Connection) -> rusqlite::Result<()> {
        *self = Self::new(conn)?;
        Ok(())
    }
}

pub fn weighted_shortest_path(
    cache: &GraphCache,
    from_id: &str,
    to_id: &str,
) -> Option<(Vec<String>, Vec<KnowledgeEdge>, f64)> {
    if from_id == to_id {
        return Some((vec![from_id.to_string()], vec![], 0.0));
    }
    use std::collections::BinaryHeap;
    use std::cmp::Ordering;

    #[derive(Clone, PartialEq)]
    struct State {
        cost: f64,
        node: String,
        prev_node: String,
        prev_edge: String,
    }
    impl Eq for State {}
    impl PartialOrd for State {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for State {
        fn cmp(&self, other: &Self) -> Ordering {
            other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
        }
    }

    let mut best_cost: HashMap<String, f64> = HashMap::new();
    let mut prev: HashMap<String, (String, String)> = HashMap::new();
    let mut heap: BinaryHeap<State> = BinaryHeap::new();

    best_cost.insert(from_id.to_string(), 0.0);
    heap.push(State { cost: 0.0, node: from_id.to_string(), prev_node: String::new(), prev_edge: String::new() });

    while let Some(State { cost, node, .. }) = heap.pop() {
        if node == to_id {
            let mut path_nodes = vec![to_id.to_string()];
            let mut path_edges: Vec<KnowledgeEdge> = Vec::new();
            let mut cur = to_id.to_string();
            while cur != from_id {
                if let Some((p, eid)) = prev.get(&cur) {
                    // Recover the full edge that led into `cur` from `p`,
                    // by matching its id among the neighbors of `p`.
                    if let Some(edge) = cache
                        .neighbors(p)
                        .into_iter()
                        .find(|e| &e.id == eid)
                    {
                        path_edges.push(edge.clone());
                    }
                    cur = p.clone();
                    path_nodes.push(cur.clone());
                } else {
                    break;
                }
            }
            path_nodes.reverse();
            path_edges.reverse();
            return Some((path_nodes, path_edges, cost));
        }
        if let Some(&bc) = best_cost.get(&node) {
            if cost > bc {
                continue;
            }
        }
        for (next_id, edge) in cache.adjacent_ids(&node) {
            let next_cost = cost + edge.weight;
            let next_key = next_id.to_string();
            if next_cost < *best_cost.get(&next_key).unwrap_or(&f64::INFINITY) {
                best_cost.insert(next_key.clone(), next_cost);
                prev.insert(next_key.clone(), (node.clone(), edge.id.clone()));
                heap.push(State {
                    cost: next_cost,
                    node: next_key,
                    prev_node: String::new(),
                    prev_edge: String::new(),
                });
            }
        }
    }
    None
}

pub fn all_paths(
    cache: &GraphCache,
    from_id: &str,
    to_id: &str,
    max_paths: usize,
    max_depth: usize,
) -> Vec<(Vec<String>, f64)> {
    let mut results: Vec<(Vec<String>, f64)> = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut current_path: Vec<String> = Vec::new();
    visited.insert(from_id.to_string());
    current_path.push(from_id.to_string());
    dfs_all_paths(cache, from_id, to_id, max_paths, max_depth, &mut visited, &mut current_path, 0.0, &mut results);
    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    results
}

fn dfs_all_paths(
    cache: &GraphCache,
    current: &str,
    target: &str,
    max_paths: usize,
    max_depth: usize,
    visited: &mut std::collections::HashSet<String>,
    path: &mut Vec<String>,
    cost: f64,
    results: &mut Vec<(Vec<String>, f64)>,
) {
    if results.len() >= max_paths {
        return;
    }
    if path.len() > max_depth {
        return;
    }
    if current == target && path.len() > 1 {
        results.push((path.clone(), cost));
        return;
    }
    for (next_id, edge) in cache.adjacent_ids(current) {
        let next_key = next_id.to_string();
        if !visited.contains(&next_key) {
            visited.insert(next_key.clone());
            path.push(next_key.clone());
            dfs_all_paths(cache, &next_key, target, max_paths, max_depth, visited, path, cost + edge.weight, results);
            path.pop();
            visited.remove(&next_key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::RelationType;

    fn edge(id: &str, src: &str, dst: &str, weight: f64) -> KnowledgeEdge {
        KnowledgeEdge {
            id: id.into(), source_id: src.into(), target_id: dst.into(),
            relation_type: RelationType::References, weight,
            description: None, created_at: 0, metadata: None,
        }
    }

    #[test]
    fn test_weighted_shortest_path_returns_edges() {
        // Regression: Dijkstra recorded (prev_node, edge_id) in `prev` but
        // discarded the id during backtracking, so path_edges was always
        // empty for any non-trivial path. Edges are now recovered by id.
        let mut cache = GraphCache::empty();
        cache.insert_edge(edge("e-ab", "a", "b", 1.0));
        cache.insert_edge(edge("e-bc", "b", "c", 1.0));
        cache.insert_edge(edge("e-ac", "a", "c", 10.0));

        let (nodes, edges, cost) = weighted_shortest_path(&cache, "a", "c").unwrap();
        assert_eq!(nodes, vec!["a", "b", "c"], "must take the cheap hop");
        assert_eq!(cost, 2.0);
        assert_eq!(edges.len(), 2, "both edges on the path must be returned");
        let ids: Vec<&str> = edges.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["e-ab", "e-bc"]);
        assert_eq!(edges[0].source_id, "a");
        assert_eq!(edges[0].target_id, "b");
    }

    #[test]
    fn test_weighted_shortest_path_same_node() {
        let cache = GraphCache::empty();
        let (nodes, edges, cost) = weighted_shortest_path(&cache, "x", "x").unwrap();
        assert_eq!(nodes, vec!["x"]);
        assert!(edges.is_empty());
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_weighted_shortest_path_no_route() {
        let mut cache = GraphCache::empty();
        cache.insert_edge(edge("e-ab", "a", "b", 1.0));
        assert!(weighted_shortest_path(&cache, "a", "z").is_none());
    }
}

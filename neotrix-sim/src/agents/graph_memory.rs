use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeKind { Event, Concept, Person, Location, Plan, Reflection }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeKind { Temporal, Causal, Semantic, Social, Spatial, Citation }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemNode {
    pub id: u64,
    pub kind: NodeKind,
    pub content: String,
    pub tick: u64,
    pub importance: f32,
    pub access_count: u32,
    pub last_accessed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: u64,
    pub to: u64,
    pub kind: EdgeKind,
    pub weight: f32,
    pub created_tick: u64,
}

pub struct GraphMemory {
    nodes: Vec<MemNode>,
    edges: Vec<Edge>,
    next_id: u64,
    adjacency: HashMap<u64, Vec<usize>>,
    max_nodes: usize,
}

impl GraphMemory {
    pub fn new(max_nodes: usize) -> Self {
        Self { nodes: Vec::new(), edges: Vec::new(), next_id: 0, adjacency: HashMap::new(), max_nodes }
    }

    pub fn add_node(&mut self, kind: NodeKind, content: &str, tick: u64, importance: f32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(MemNode {
            id, kind, content: content.to_string(), tick, importance,
            access_count: 0, last_accessed: tick,
        });
        self.adjacency.entry(id).or_default();
        self.prune();
        id
    }

    pub fn add_edge(&mut self, from: u64, to: u64, kind: EdgeKind, weight: f32, tick: u64) {
        if self.edges.iter().any(|e| e.from == from && e.to == to && e.kind == kind) { return; }
        let idx = self.edges.len();
        self.edges.push(Edge { from, to, kind, weight, created_tick: tick });
        self.adjacency.entry(from).or_default().push(idx);
        self.adjacency.entry(to).or_default().push(idx);
    }

    pub fn get_node(&self, id: u64) -> Option<&MemNode> {
        self.nodes.iter().find(|n| n.id == id).map(|n| {
            // note: we can't mutate here, so access_count is updated on access via spread_activation
            n
        })
    }

    fn edges_from(&self, id: u64) -> Vec<&Edge> {
        self.adjacency.get(&id).map(|idxs| {
            idxs.iter().filter_map(|&i| self.edges.get(i)).collect()
        }).unwrap_or_default()
    }

    pub fn neighbors(&self, id: u64) -> Vec<&MemNode> {
        self.edges_from(id).iter()
            .filter_map(|e| {
                let next = if e.from == id { e.to } else { e.from };
                self.nodes.iter().find(|n| n.id == next)
            })
            .collect()
    }

    pub fn neighbors_by_kind(&self, id: u64, kind: &EdgeKind) -> Vec<&MemNode> {
        self.edges_from(id).iter()
            .filter(|e| &e.kind == kind)
            .filter_map(|e| {
                let next = if e.from == id { e.to } else { e.from };
                self.nodes.iter().find(|n| n.id == next)
            })
            .collect()
    }

    pub fn spread_activation(&self, start_id: u64, hops: u32, min_weight: f32) -> Vec<(&MemNode, f32)> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((start_id, 1.0f32, 0u32));
        visited.insert(start_id);

        while let Some((id, strength, depth)) = queue.pop_front() {
            if depth > 0 {
                if let Some(node) = self.nodes.iter().find(|n| n.id == id) {
                    result.push((node, strength));
                }
            }
            if depth >= hops { continue; }
            for edge in self.edges_from(id) {
                if edge.weight < min_weight { continue; }
                let next = if edge.from == id { edge.to } else { edge.from };
                if visited.insert(next) {
                    queue.push_back((next, strength * edge.weight, depth + 1));
                }
            }
        }
        result
    }

    pub fn path_between(&self, from: u64, to: u64, max_hops: u32) -> Option<Vec<u64>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<u64, u64> = HashMap::new();
        queue.push_back((from, 0u32));
        visited.insert(from);

        while let ((id, depth)) = queue.pop_front()? {
            if id == to {
                let mut path = vec![to];
                let mut cur = to;
                while let Some(&p) = parent.get(&cur) {
                    path.push(p);
                    cur = p;
                }
                path.reverse();
                return Some(path);
            }
            if depth >= max_hops { continue; }
            for edge in self.edges_from(id) {
                let next = if edge.from == id { edge.to } else { edge.from };
                if visited.insert(next) {
                    parent.insert(next, id);
                    queue.push_back((next, depth + 1));
                }
            }
        }
        None
    }

    pub fn nodes_by_kind(&self, kind: &NodeKind) -> Vec<&MemNode> {
        self.nodes.iter().filter(|n| &n.kind == kind).collect()
    }

    pub fn strongest_connections(&self, id: u64, top_k: usize) -> Vec<(&MemNode, f32)> {
        let mut edges: Vec<&Edge> = self.edges_from(id).into_iter()
            .filter(|e| e.from == id || e.to == id)
            .collect();
        edges.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        edges.iter().take(top_k).filter_map(|e| {
            let next = if e.from == id { e.to } else { e.from };
            self.nodes.iter().find(|n| n.id == next).map(|n| (n, e.weight))
        }).collect()
    }

    pub fn consolidate(&mut self) {
        // Strengthen edges between frequently co-accessed nodes
        for edge in &mut self.edges {
            if let (Some(from), Some(to)) = (
                self.nodes.iter().find(|n| n.id == edge.from),
                self.nodes.iter().find(|n| n.id == edge.to),
            ) {
                let co_access = from.access_count.min(to.access_count) as f32;
                edge.weight = (edge.weight + co_access * 0.01).min(1.0);
            }
        }
    }

    fn prune(&mut self) {
        if self.nodes.len() <= self.max_nodes { return; }
        self.nodes.sort_by(|a, b| {
            b.importance.partial_cmp(&a.importance).unwrap()
                .then(b.access_count.cmp(&a.access_count))
        });
        let keep: HashSet<u64> = self.nodes.iter().take(self.max_nodes * 8 / 10).map(|n| n.id).collect();
        self.nodes.retain(|n| keep.contains(&n.id));
        self.edges.retain(|e| keep.contains(&e.from) && keep.contains(&e.to));
        self.adjacency.clear();
        for (i, edge) in self.edges.iter().enumerate() {
            self.adjacency.entry(edge.from).or_default().push(i);
            self.adjacency.entry(edge.to).or_default().push(i);
        }
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> usize { self.edges.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_node_returns_sequential_ids() {
        let mut gm = GraphMemory::new(100);
        let a = gm.add_node(NodeKind::Event, "burnt toast", 0, 0.5);
        let b = gm.add_node(NodeKind::Event, "made coffee", 1, 0.3);
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(gm.node_count(), 2);
    }

    #[test]
    fn add_edge_creates_connection() {
        let mut gm = GraphMemory::new(100);
        let a = gm.add_node(NodeKind::Event, "A", 0, 0.5);
        let b = gm.add_node(NodeKind::Event, "B", 1, 0.5);
        gm.add_edge(a, b, EdgeKind::Temporal, 0.8, 1);
        assert_eq!(gm.edge_count(), 1);
    }

    #[test]
    fn neighbors_returns_connected() {
        let mut gm = GraphMemory::new(100);
        let a = gm.add_node(NodeKind::Event, "A", 0, 0.5);
        let b = gm.add_node(NodeKind::Event, "B", 1, 0.5);
        let c = gm.add_node(NodeKind::Event, "C", 2, 0.5);
        gm.add_edge(a, b, EdgeKind::Temporal, 0.8, 1);
        gm.add_edge(a, c, EdgeKind::Causal, 0.6, 2);
        let n = gm.neighbors(a);
        assert_eq!(n.len(), 2);
    }

    #[test]
    fn spread_activation_reaches() {
        let mut gm = GraphMemory::new(100);
        let a = gm.add_node(NodeKind::Concept, "A", 0, 1.0);
        let b = gm.add_node(NodeKind::Concept, "B", 1, 0.5);
        let c = gm.add_node(NodeKind::Concept, "C", 2, 0.5);
        gm.add_edge(a, b, EdgeKind::Semantic, 0.9, 1);
        gm.add_edge(b, c, EdgeKind::Semantic, 0.8, 2);
        let result = gm.spread_activation(a, 2, 0.5);
        assert!(result.len() >= 2);
    }

    #[test]
    fn path_between_finds_shortest() {
        let mut gm = GraphMemory::new(100);
        let a = gm.add_node(NodeKind::Event, "A", 0, 0.5);
        let b = gm.add_node(NodeKind::Event, "B", 1, 0.5);
        let c = gm.add_node(NodeKind::Event, "C", 2, 0.5);
        gm.add_edge(a, b, EdgeKind::Temporal, 0.8, 1);
        gm.add_edge(b, c, EdgeKind::Temporal, 0.8, 2);
        let path = gm.path_between(a, c, 5).unwrap();
        assert_eq!(path, vec![a, b, c]);
    }

    #[test]
    fn nodes_by_kind_filters() {
        let mut gm = GraphMemory::new(100);
        gm.add_node(NodeKind::Event, "A", 0, 0.5);
        gm.add_node(NodeKind::Concept, "B", 1, 0.5);
        gm.add_node(NodeKind::Event, "C", 2, 0.5);
        assert_eq!(gm.nodes_by_kind(&NodeKind::Event).len(), 2);
        assert_eq!(gm.nodes_by_kind(&NodeKind::Concept).len(), 1);
    }

    #[test]
    fn prune_removes_low_importance() {
        let mut gm = GraphMemory::new(5);
        for i in 0..10 {
            gm.add_node(NodeKind::Event, &format!("E{}", i), i, i as f32 * 0.1);
        }
        assert!(gm.node_count() <= 5);
    }

    #[test]
    fn duplicate_edge_prevented() {
        let mut gm = GraphMemory::new(100);
        let a = gm.add_node(NodeKind::Event, "A", 0, 0.5);
        let b = gm.add_node(NodeKind::Event, "B", 1, 0.5);
        gm.add_edge(a, b, EdgeKind::Temporal, 0.8, 1);
        gm.add_edge(a, b, EdgeKind::Temporal, 0.9, 2);
        assert_eq!(gm.edge_count(), 1);
    }
}

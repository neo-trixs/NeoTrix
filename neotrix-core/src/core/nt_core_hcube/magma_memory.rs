#![forbid(unsafe_code)]

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphType {
    Semantic,
    Temporal,
    Causal,
    Entity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relation {
    IsA,
    Has,
    PartOf,
    SimilarTo,
    Before,
    After,
    During,
    Concurrent,
    Causes,
    Prevents,
    Enables,
    Contributes,
    SameAs,
    RelatedTo,
    References,
    DependsOn,
}

impl Relation {
    pub fn graph_type(&self) -> GraphType {
        match self {
            Relation::IsA | Relation::Has | Relation::PartOf | Relation::SimilarTo => {
                GraphType::Semantic
            }
            Relation::Before | Relation::After | Relation::During | Relation::Concurrent => {
                GraphType::Temporal
            }
            Relation::Causes | Relation::Prevents | Relation::Enables | Relation::Contributes => {
                GraphType::Causal
            }
            Relation::SameAs | Relation::RelatedTo | Relation::References | Relation::DependsOn => {
                GraphType::Entity
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: u64,
    pub target: u64,
    pub relation: Relation,
    pub weight: f64,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct MemoryGraph {
    pub edges: Vec<GraphEdge>,
    pub graph_type: GraphType,
}

impl MemoryGraph {
    pub fn new(graph_type: GraphType) -> Self {
        Self {
            edges: Vec::new(),
            graph_type,
        }
    }

    pub fn add_edge(&mut self, source: u64, target: u64, relation: Relation, weight: f64) {
        let edge = GraphEdge {
            source,
            target,
            relation,
            weight: weight.clamp(0.0, 1.0),
            created_at: 0,
        };
        self.edges.push(edge);
    }

    pub fn query(&self, source: u64, relation: Option<Relation>) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|e| e.source == source && relation.map_or(true, |r| e.relation == r))
            .collect()
    }

    pub fn query_target(&self, target: u64, relation: Option<Relation>) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|e| e.target == target && relation.map_or(true, |r| e.relation == r))
            .collect()
    }

    pub fn all_relations(&self) -> Vec<Relation> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for edge in &self.edges {
            if seen.insert(edge.relation) {
                result.push(edge.relation);
            }
        }
        result
    }

    pub fn node_count(&self) -> usize {
        let mut nodes = HashSet::new();
        for edge in &self.edges {
            nodes.insert(edge.source);
            nodes.insert(edge.target);
        }
        nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PolicyType {
    BroadFirst,
    SpecificFirst { graph: GraphType },
    ConfidenceFirst,
}

#[derive(Debug, Clone)]
pub struct PolicyGuidedTraversal {
    pub graphs: Vec<MemoryGraph>,
    pub default_policy: PolicyType,
}

impl PolicyGuidedTraversal {
    pub fn new() -> Self {
        Self {
            graphs: vec![
                MemoryGraph::new(GraphType::Semantic),
                MemoryGraph::new(GraphType::Temporal),
                MemoryGraph::new(GraphType::Causal),
                MemoryGraph::new(GraphType::Entity),
            ],
            default_policy: PolicyType::BroadFirst,
        }
    }

    pub fn add_to_graph(
        &mut self,
        gt: GraphType,
        source: u64,
        target: u64,
        relation: Relation,
        weight: f64,
    ) {
        if let Some(g) = self.graphs.iter_mut().find(|g| g.graph_type == gt) {
            g.add_edge(source, target, relation, weight);
        }
    }

    fn graph_index(gt: GraphType) -> usize {
        match gt {
            GraphType::Semantic => 0,
            GraphType::Temporal => 1,
            GraphType::Causal => 2,
            GraphType::Entity => 3,
        }
    }

    pub fn traverse(
        &self,
        source: u64,
        policy: Option<PolicyType>,
    ) -> Vec<(GraphType, &GraphEdge)> {
        let policy = policy.unwrap_or(self.default_policy);
        match policy {
            PolicyType::BroadFirst => {
                let mut results = Vec::new();
                for g in &self.graphs {
                    for edge in g.query(source, None) {
                        results.push((g.graph_type, edge));
                    }
                }
                results
            }
            PolicyType::SpecificFirst { graph } => {
                let mut results = Vec::new();
                let prio_idx = Self::graph_index(graph);
                for edge in self.graphs[prio_idx].query(source, None) {
                    results.push((self.graphs[prio_idx].graph_type, edge));
                }
                for (i, g) in self.graphs.iter().enumerate() {
                    if i == prio_idx {
                        continue;
                    }
                    for edge in g.query(source, None) {
                        results.push((g.graph_type, edge));
                    }
                }
                results
            }
            PolicyType::ConfidenceFirst => {
                let mut avg_weights: Vec<(usize, f64)> = self
                    .graphs
                    .iter()
                    .enumerate()
                    .map(|(i, g)| {
                        let n = g.edges.len();
                        if n == 0 {
                            (i, 0.0)
                        } else {
                            let sum: f64 = g.edges.iter().map(|e| e.weight).sum();
                            (i, sum / n as f64)
                        }
                    })
                    .collect();
                avg_weights
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let mut results = Vec::new();
                for (i, _) in avg_weights {
                    if self.graphs[i].edge_count() > 0 {
                        for edge in self.graphs[i].query(source, None) {
                            results.push((self.graphs[i].graph_type, edge));
                        }
                    }
                }
                results
            }
        }
    }

    pub fn fused_query(&self, source: u64, policy: Option<PolicyType>) -> Vec<u64> {
        let mut targets = HashSet::new();
        for (_, edge) in self.traverse(source, policy) {
            targets.insert(edge.target);
        }
        let mut result: Vec<u64> = targets.into_iter().collect();
        result.sort();
        result
    }

    pub fn graph(&self, gt: GraphType) -> Option<&MemoryGraph> {
        self.graphs.iter().find(|g| g.graph_type == gt)
    }
}

impl Default for PolicyGuidedTraversal {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MagmaMemoryStore {
    pub traversal: PolicyGuidedTraversal,
}

impl MagmaMemoryStore {
    pub fn new() -> Self {
        Self {
            traversal: PolicyGuidedTraversal::new(),
        }
    }

    pub fn store_relation(
        &mut self,
        gt: GraphType,
        source: u64,
        target: u64,
        relation: Relation,
        weight: f64,
    ) {
        self.traversal
            .add_to_graph(gt, source, target, relation, weight);
    }

    pub fn query_relations(
        &self,
        source: u64,
        policy: Option<PolicyType>,
    ) -> Vec<(GraphType, &GraphEdge)> {
        self.traversal.traverse(source, policy)
    }
}

impl Default for MagmaMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_type_derives() {
        let types = [
            GraphType::Semantic,
            GraphType::Temporal,
            GraphType::Causal,
            GraphType::Entity,
        ];
        assert_eq!(types.len(), 4);
    }

    #[test]
    fn test_relation_graph_type_mapping() {
        assert_eq!(Relation::IsA.graph_type(), GraphType::Semantic);
        assert_eq!(Relation::Before.graph_type(), GraphType::Temporal);
        assert_eq!(Relation::Causes.graph_type(), GraphType::Causal);
        assert_eq!(Relation::SameAs.graph_type(), GraphType::Entity);
    }

    #[test]
    fn test_memory_graph_add_and_query() {
        let mut g = MemoryGraph::new(GraphType::Semantic);
        g.add_edge(1, 2, Relation::IsA, 0.9);
        g.add_edge(1, 3, Relation::Has, 0.8);

        let results = g.query(1, None);
        assert_eq!(results.len(), 2);

        let filtered = g.query(1, Some(Relation::IsA));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].target, 2);
    }

    #[test]
    fn test_memory_graph_query_target() {
        let mut g = MemoryGraph::new(GraphType::Causal);
        g.add_edge(10, 20, Relation::Causes, 1.0);
        g.add_edge(15, 20, Relation::Contributes, 0.7);

        let results = g.query_target(20, None);
        assert_eq!(results.len(), 2);

        let filtered = g.query_target(20, Some(Relation::Causes));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source, 10);
    }

    #[test]
    fn test_weight_clamped() {
        let mut g = MemoryGraph::new(GraphType::Semantic);
        g.add_edge(1, 2, Relation::IsA, 1.5);
        assert!((g.edges[0].weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_all_relations() {
        let mut g = MemoryGraph::new(GraphType::Temporal);
        g.add_edge(1, 2, Relation::Before, 0.5);
        g.add_edge(2, 3, Relation::After, 0.6);
        g.add_edge(3, 4, Relation::Before, 0.7);
        let rels = g.all_relations();
        assert_eq!(rels.len(), 2);
    }

    #[test]
    fn test_node_and_edge_count() {
        let mut g = MemoryGraph::new(GraphType::Entity);
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
        g.add_edge(1, 2, Relation::SameAs, 1.0);
        g.add_edge(2, 3, Relation::DependsOn, 0.5);
        assert_eq!(g.edge_count(), 2);
        assert_eq!(g.node_count(), 3);
    }

    #[test]
    fn test_policy_guided_traversal_new() {
        let pt = PolicyGuidedTraversal::new();
        assert_eq!(pt.graphs.len(), 4);
    }

    #[test]
    fn test_add_to_graph_dispatches_correctly() {
        let mut pt = PolicyGuidedTraversal::new();
        pt.add_to_graph(GraphType::Semantic, 1, 2, Relation::IsA, 0.9);
        pt.add_to_graph(GraphType::Temporal, 1, 3, Relation::Before, 0.8);
        assert_eq!(pt.graphs[0].edge_count(), 1);
        assert_eq!(pt.graphs[1].edge_count(), 1);
    }

    #[test]
    fn test_broad_first_policy() {
        let mut pt = PolicyGuidedTraversal::new();
        pt.add_to_graph(GraphType::Semantic, 1, 2, Relation::IsA, 0.9);
        pt.add_to_graph(GraphType::Temporal, 1, 3, Relation::Before, 0.8);
        let results = pt.traverse(1, Some(PolicyType::BroadFirst));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_specific_first_policy() {
        let mut pt = PolicyGuidedTraversal::new();
        pt.add_to_graph(GraphType::Semantic, 1, 2, Relation::IsA, 0.9);
        pt.add_to_graph(GraphType::Temporal, 1, 3, Relation::Before, 0.8);
        pt.add_to_graph(GraphType::Causal, 1, 4, Relation::Causes, 0.7);
        let results = pt.traverse(
            1,
            Some(PolicyType::SpecificFirst {
                graph: GraphType::Temporal,
            }),
        );
        assert!(results.len() >= 1);
        assert_eq!(results[0].0, GraphType::Temporal);
    }

    #[test]
    fn test_confidence_first_policy() {
        let mut pt = PolicyGuidedTraversal::new();
        pt.add_to_graph(GraphType::Semantic, 1, 2, Relation::IsA, 1.0);
        pt.add_to_graph(GraphType::Temporal, 1, 3, Relation::Before, 0.1);
        pt.add_to_graph(GraphType::Causal, 1, 4, Relation::Causes, 0.1);
        let results = pt.traverse(1, Some(PolicyType::ConfidenceFirst));
        assert_eq!(results[0].0, GraphType::Semantic);
    }

    #[test]
    fn test_fused_query() {
        let mut pt = PolicyGuidedTraversal::new();
        pt.add_to_graph(GraphType::Semantic, 1, 2, Relation::IsA, 0.9);
        pt.add_to_graph(GraphType::Temporal, 1, 2, Relation::Before, 0.8);
        pt.add_to_graph(GraphType::Entity, 1, 3, Relation::RelatedTo, 0.7);
        let targets = pt.fused_query(1, Some(PolicyType::BroadFirst));
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn test_graph_accessor() {
        let pt = PolicyGuidedTraversal::new();
        assert!(pt.graph(GraphType::Semantic).is_some());
        assert!(pt.graph(GraphType::Temporal).is_some());
        assert!(pt.graph(GraphType::Causal).is_some());
        assert!(pt.graph(GraphType::Entity).is_some());
    }

    #[test]
    fn test_magma_memory_store() {
        let mut store = MagmaMemoryStore::new();
        store.store_relation(GraphType::Semantic, 42, 100, Relation::IsA, 0.95);
        store.store_relation(GraphType::Causal, 42, 101, Relation::Causes, 0.85);
        let results = store.query_relations(42, Some(PolicyType::BroadFirst));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_default_policy_used() {
        let mut pt = PolicyGuidedTraversal::new();
        pt.default_policy = PolicyType::SpecificFirst {
            graph: GraphType::Semantic,
        };
        pt.add_to_graph(GraphType::Semantic, 1, 2, Relation::IsA, 0.9);
        pt.add_to_graph(GraphType::Temporal, 1, 3, Relation::Before, 0.8);
        let results = pt.traverse(1, None);
        assert_eq!(results[0].0, GraphType::Semantic);
    }

    #[test]
    fn test_empty_traverse() {
        let pt = PolicyGuidedTraversal::new();
        let results = pt.traverse(99, Some(PolicyType::BroadFirst));
        assert!(results.is_empty());
    }
}

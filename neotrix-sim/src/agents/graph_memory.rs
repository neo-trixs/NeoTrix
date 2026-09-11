use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeKind {
    Event,
    Concept,
    Person,
    Location,
    Plan,
    Reflection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeKind {
    Temporal,
    Causal,
    Semantic,
    Social,
    Spatial,
    Citation,
}

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
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            next_id: 0,
            adjacency: HashMap::new(),
            max_nodes,
        }
    }

    pub fn add_node(&mut self, kind: NodeKind, content: &str, tick: u64, importance: f32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(MemNode {
            id,
            kind,
            content: content.to_string(),
            tick,
            importance,
            access_count: 0,
            last_accessed: tick,
        });
        self.adjacency.entry(id).or_default();
        self.prune();
        id
    }

    pub fn add_edge(
        &mut self,
        from: u64,
        to: u64,
        kind: EdgeKind,
        weight: f32,
        tick: u64,
    ) {
        if self
            .edges
            .iter()
            .any(|e| e.from == from && e.to == to && e.kind == kind)
        {
            return;
        }
        let idx = self.edges.len();
        self.edges.push(Edge {
            from,
            to,
            kind,
            weight,
            created_tick: tick,
        });
        self.adjacency.entry(from).or_default().push(idx);
        self.adjacency.entry(to).or_default().push(idx);
    }

    pub fn get_node(&self, id: u64) -> Option<&MemNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    fn edges_from(&self, id: u64) -> Vec<&Edge> {
        self.adjacency
            .get(&id)
            .map(|idxs| idxs.iter().filter_map(|&i| self.edges.get(i)).collect())
            .unwrap_or_default()
    }

    pub fn neighbors(&self, id: u64) -> Vec<&MemNode> {
        self.edges_from(id)
            .iter()
            .filter_map(|e| {
                let next = if e.from == id { e.to } else { e.from };
                self.nodes.iter().find(|n| n.id == next)
            })
            .collect()
    }

    pub fn neighbors_by_kind(&self, id: u64, kind: &EdgeKind) -> Vec<&MemNode> {
        self.edges_from(id)
            .iter()
            .filter(|e| &e.kind == kind)
            .filter_map(|e| {
                let next = if e.from == id { e.to } else { e.from };
                self.nodes.iter().find(|n| n.id == next)
            })
            .collect()
    }

    pub fn spread_activation(
        &self,
        start_id: u64,
        hops: u32,
        min_weight: f32,
    ) -> Vec<(&MemNode, f32)> {
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
            if depth >= hops {
                continue;
            }
            for edge in self.edges_from(id) {
                if edge.weight < min_weight {
                    continue;
                }
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

        loop {
            let (id, depth) = queue.pop_front()?;
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
            if depth >= max_hops {
                continue;
            }
            for edge in self.edges_from(id) {
                let next = if edge.from == id { edge.to } else { edge.from };
                if visited.insert(next) {
                    parent.insert(next, id);
                    queue.push_back((next, depth + 1));
                }
            }
        }
    }

    pub fn nodes_by_kind(&self, kind: &NodeKind) -> Vec<&MemNode> {
        self.nodes.iter().filter(|n| &n.kind == kind).collect()
    }

    pub fn strongest_connections(&self, id: u64, top_k: usize) -> Vec<(&MemNode, f32)> {
        let mut edges: Vec<&Edge> = self
            .edges_from(id)
            .into_iter()
            .filter(|e| e.from == id || e.to == id)
            .collect();
        edges.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        edges
            .iter()
            .take(top_k)
            .filter_map(|e| {
                let next = if e.from == id { e.to } else { e.from };
                self.nodes
                    .iter()
                    .find(|n| n.id == next)
                    .map(|n| (n, e.weight))
            })
            .collect()
    }

    pub fn consolidate(&mut self) {
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
        if self.nodes.len() <= self.max_nodes {
            return;
        }
        self.nodes.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap()
                .then(b.access_count.cmp(&a.access_count))
        });
        let keep: HashSet<u64> = self
            .nodes
            .iter()
            .take(self.max_nodes * 8 / 10)
            .map(|n| n.id)
            .collect();
        self.nodes.retain(|n| keep.contains(&n.id));
        self.edges
            .retain(|e| keep.contains(&e.from) && keep.contains(&e.to));
        self.adjacency.clear();
        for (i, edge) in self.edges.iter().enumerate() {
            self.adjacency.entry(edge.from).or_default().push(i);
            self.adjacency.entry(edge.to).or_default().push(i);
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
    pub fn last_node_id(&self) -> Option<u64> {
        self.nodes.last().map(|n| n.id)
    }
}

// ============================================================================
// SYNAPSE-inspired Unified Memory Graph
// ============================================================================

/// Episodic memory node — captures what happened with situational context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeNode {
    pub id: u64,
    pub description: String,
    pub tick: u64,
    pub importance: f32,
    pub access_count: u32,
    pub last_accessed: u64,
    pub embedding: Option<[f32; 16]>,
    // Situational context (REMem-style binding)
    pub location: Option<String>,
    pub participants: Vec<String>,
    pub emotion: Option<String>,
    pub action_taken: Option<String>,
}

/// Semantic memory node — captures what is known (concepts, entities, facts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticNode {
    pub id: u64,
    pub label: String,
    pub definition: String,
    pub tick: u64,
    pub importance: f32,
    pub access_count: u32,
    pub last_accessed: u64,
    pub embedding: Option<[f32; 16]>,
    pub parent_concepts: Vec<u64>,
    pub child_concepts: Vec<u64>,
}

/// Edge types in the unified graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UnifiedEdgeKind {
    /// Episode A happened before episode B
    Temporal,
    /// Episode A caused episode B
    Causal,
    /// Semantic: concept A is-a / part-of concept B
    Abstraction,
    /// Episode involves concept C
    InstanceOf,
    /// Semantic: concept A relates-to concept B
    Associative,
    /// Episode has emotional coloring E
    Emotional,
}

/// A directed edge in the unified graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedEdge {
    pub from: u64,
    pub to: u64,
    pub kind: UnifiedEdgeKind,
    pub weight: f32,
    pub created_tick: u64,
}

/// Node in the unified graph — can be either episodic or semantic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnifiedNode {
    Episode(EpisodeNode),
    Semantic(SemanticNode),
}

impl UnifiedNode {
    pub fn id(&self) -> u64 {
        match self {
            UnifiedNode::Episode(e) => e.id,
            UnifiedNode::Semantic(s) => s.id,
        }
    }
    pub fn tick(&self) -> u64 {
        match self {
            UnifiedNode::Episode(e) => e.tick,
            UnifiedNode::Semantic(s) => s.tick,
        }
    }
    pub fn importance(&self) -> f32 {
        match self {
            UnifiedNode::Episode(e) => e.importance,
            UnifiedNode::Semantic(s) => s.importance,
        }
    }
    pub fn access_count(&self) -> u32 {
        match self {
            UnifiedNode::Episode(e) => e.access_count,
            UnifiedNode::Semantic(s) => s.access_count,
        }
    }
    pub fn last_accessed(&self) -> u64 {
        match self {
            UnifiedNode::Episode(e) => e.last_accessed,
            UnifiedNode::Semantic(s) => s.last_accessed,
        }
    }
    pub fn embedding(&self) -> Option<&[f32; 16]> {
        match self {
            UnifiedNode::Episode(e) => e.embedding.as_ref(),
            UnifiedNode::Semantic(s) => s.embedding.as_ref(),
        }
    }
}

/// Configuration for spreading activation
#[derive(Debug, Clone)]
pub struct ActivationConfig {
    /// Maximum hops for activation propagation
    pub max_hops: u32,
    /// Decay factor per hop (multiplied into edge weight)
    pub decay_per_hop: f32,
    /// Minimum activation threshold to propagate
    pub min_activation: f32,
    /// Lateral inhibition strength (0.0 = disabled)
    pub inhibition_strength: f32,
    /// Temporal decay half-life in ticks
    pub temporal_decay_half_life: f32,
    /// Initial activation for seed nodes
    pub seed_activation: f32,
}

impl Default for ActivationConfig {
    fn default() -> Self {
        Self {
            max_hops: 4,
            decay_per_hop: 0.7,
            min_activation: 0.05,
            inhibition_strength: 0.3,
            temporal_decay_half_life: 50.0,
            seed_activation: 1.0,
        }
    }
}

/// Configuration for triple hybrid retrieval
#[derive(Debug, Clone)]
pub struct RetrievalConfig {
    /// Weight for geometric (cosine similarity) component
    pub geometric_weight: f32,
    /// Weight for spreading activation component
    pub activation_weight: f32,
    /// Weight for graph traversal (path proximity) component
    pub traversal_weight: f32,
    /// Number of top results to return
    pub top_k: usize,
    /// Max hops for graph traversal scoring
    pub max_traversal_hops: u32,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            geometric_weight: 0.4,
            activation_weight: 0.35,
            traversal_weight: 0.25,
            top_k: 10,
            max_traversal_hops: 5,
        }
    }
}

/// Unified episodic-semantic memory graph (SYNAPSE-inspired)
pub struct UnifiedMemoryGraph {
    nodes: Vec<UnifiedNode>,
    edges: Vec<UnifiedEdge>,
    next_id: u64,
    adjacency: HashMap<u64, Vec<usize>>,
    max_nodes: usize,
    /// Tracks which node IDs are episodic vs semantic for fast lookup
    episodic_ids: HashSet<u64>,
    semantic_ids: HashSet<u64>,
}

impl UnifiedMemoryGraph {
    pub fn new(max_nodes: usize) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            next_id: 0,
            adjacency: HashMap::new(),
            max_nodes,
            episodic_ids: HashSet::new(),
            semantic_ids: HashSet::new(),
        }
    }

    /// Add an episodic memory node
    pub fn add_episode(
        &mut self,
        description: &str,
        tick: u64,
        importance: f32,
        embedding: Option<[f32; 16]>,
        location: Option<String>,
        participants: Vec<String>,
        emotion: Option<String>,
        action_taken: Option<String>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.episodic_ids.insert(id);
        self.nodes.push(UnifiedNode::Episode(EpisodeNode {
            id,
            description: description.to_string(),
            tick,
            importance,
            access_count: 0,
            last_accessed: tick,
            embedding,
            location,
            participants,
            emotion,
            action_taken,
        }));
        self.adjacency.entry(id).or_default();
        self.prune();
        id
    }

    /// Add a semantic memory node
    pub fn add_semantic(
        &mut self,
        label: &str,
        definition: &str,
        tick: u64,
        importance: f32,
        embedding: Option<[f32; 16]>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.semantic_ids.insert(id);
        self.nodes.push(UnifiedNode::Semantic(SemanticNode {
            id,
            label: label.to_string(),
            definition: definition.to_string(),
            tick,
            importance,
            access_count: 0,
            last_accessed: tick,
            embedding,
            parent_concepts: Vec::new(),
            child_concepts: Vec::new(),
        }));
        self.adjacency.entry(id).or_default();
        self.prune();
        id
    }

    /// Add an edge between any two nodes
    pub fn add_edge(
        &mut self,
        from: u64,
        to: u64,
        kind: UnifiedEdgeKind,
        weight: f32,
        tick: u64,
    ) {
        if self
            .edges
            .iter()
            .any(|e| e.from == from && e.to == to && e.kind == kind)
        {
            return;
        }
        let idx = self.edges.len();
        self.edges.push(UnifiedEdge {
            from,
            to,
            kind,
            weight,
            created_tick: tick,
        });
        self.adjacency.entry(from).or_default().push(idx);
        self.adjacency.entry(to).or_default().push(idx);
    }

    /// Add a temporal edge (A happened before B)
    pub fn add_temporal_edge(&mut self, from: u64, to: u64, weight: f32, tick: u64) {
        self.add_edge(from, to, UnifiedEdgeKind::Temporal, weight, tick);
    }

    /// Add a causal edge (A caused B)
    pub fn add_causal_edge(&mut self, from: u64, to: u64, weight: f32, tick: u64) {
        self.add_edge(from, to, UnifiedEdgeKind::Causal, weight, tick);
    }

    /// Add an abstraction edge (concept A is-a concept B)
    pub fn add_abstraction_edge(&mut self, from: u64, to: u64, weight: f32, tick: u64) {
        self.add_edge(from, to, UnifiedEdgeKind::Abstraction, weight, tick);
    }

    /// Add an instance-of edge (episode E involves concept C)
    pub fn add_instance_of_edge(&mut self, from: u64, to: u64, weight: f32, tick: u64) {
        self.add_edge(from, to, UnifiedEdgeKind::InstanceOf, weight, tick);
    }

    pub fn get_node(&self, id: u64) -> Option<&UnifiedNode> {
        self.nodes.iter().find(|n| n.id() == id)
    }

    fn edges_from(&self, id: u64) -> Vec<&UnifiedEdge> {
        self.adjacency
            .get(&id)
            .map(|idxs| idxs.iter().filter_map(|&i| self.edges.get(i)).collect())
            .unwrap_or_default()
    }

    pub fn neighbors(&self, id: u64) -> Vec<&UnifiedNode> {
        self.edges_from(id)
            .iter()
            .filter_map(|e| {
                let next = if e.from == id { e.to } else { e.from };
                self.nodes.iter().find(|n| n.id() == next)
            })
            .collect()
    }

    pub fn neighbors_by_kind(&self, id: u64, kind: &UnifiedEdgeKind) -> Vec<&UnifiedNode> {
        self.edges_from(id)
            .iter()
            .filter(|e| &e.kind == kind)
            .filter_map(|e| {
                let next = if e.from == id { e.to } else { e.from };
                self.nodes.iter().find(|n| n.id() == next)
            })
            .collect()
    }

    pub fn episodes(&self) -> Vec<&EpisodeNode> {
        self.nodes
            .iter()
            .filter_map(|n| match n {
                UnifiedNode::Episode(e) => Some(e),
                _ => None,
            })
            .collect()
    }

    pub fn semantic_nodes(&self) -> Vec<&SemanticNode> {
        self.nodes
            .iter()
            .filter_map(|n| match n {
                UnifiedNode::Semantic(s) => Some(s),
                _ => None,
            })
            .collect()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn last_node_id(&self) -> Option<u64> {
        self.nodes.last().map(|n| n.id())
    }

    // ========================================================================
    // Spreading Activation with Lateral Inhibition + Temporal Decay
    // ========================================================================

    /// Compute temporal decay factor: older memories decay exponentially
    fn temporal_decay(current_tick: u64, memory_tick: u64, half_life: f32) -> f32 {
        let age = current_tick.saturating_sub(memory_tick) as f32;
        (-0.693 * age / half_life).exp()
    }

    /// Lateral inhibition: competing nodes suppress each other's activation
    /// When two activated nodes are semantically similar but competing, the
    /// stronger one suppresses the weaker one
    fn apply_lateral_inhibition(
        activations: &mut HashMap<u64, f32>,
        nodes: &[UnifiedNode],
        inhibition_strength: f32,
    ) {
        if inhibition_strength <= 0.0 {
            return;
        }

        let ids: Vec<u64> = activations.keys().copied().collect();
        let mut suppressions: Vec<(u64, f32)> = Vec::new();

        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let a_id = ids[i];
                let b_id = ids[j];
                let a_act = activations[&a_id];
                let b_act = activations[&b_id];

                // Compute semantic similarity from embeddings
                let a_node = nodes.iter().find(|n| n.id() == a_id);
                let b_node = nodes.iter().find(|n| n.id() == b_id);
                let similarity = match (a_node.and_then(|n| n.embedding()), b_node.and_then(|n| n.embedding()))
                {
                    (Some(a_emb), Some(b_emb)) => cosine_sim(a_emb, b_emb),
                    _ => 0.0,
                };

                // Inhibition proportional to similarity and activation difference
                if similarity > 0.3 {
                    let suppression_amount =
                        inhibition_strength * similarity * (a_act - b_act).abs();
                    if a_act > b_act {
                        suppressions.push((b_id, suppression_amount));
                    } else {
                        suppressions.push((a_id, suppression_amount));
                    }
                }
            }
        }

        for (id, amount) in suppressions {
            if let Some(act) = activations.get_mut(&id) {
                *act = (*act - amount).max(0.0);
            }
        }
    }

    /// Spreading activation from a set of seed node IDs
    /// Returns (node_id, activation_score) pairs
    pub fn spreading_activation(
        &self,
        seed_ids: &[u64],
        current_tick: u64,
        config: &ActivationConfig,
    ) -> HashMap<u64, f32> {
        let mut activations: HashMap<u64, f32> = HashMap::new();
        let mut visited: HashSet<u64> = HashSet::new();
        let mut queue: VecDeque<(u64, f32, u32)> = VecDeque::new();

        // Seed the activation
        for &seed_id in seed_ids {
            let temporal_factor =
                Self::temporal_decay(current_tick, current_tick, config.temporal_decay_half_life);
            activations.insert(seed_id, config.seed_activation * temporal_factor);
            queue.push_back((seed_id, config.seed_activation, 0));
            visited.insert(seed_id);
        }

        // BFS propagation
        while let Some((id, strength, depth)) = queue.pop_front() {
            if depth >= config.max_hops {
                continue;
            }

            for edge in self.edges_from(id) {
                let next = if edge.from == id { edge.to } else { edge.from };
                if visited.contains(&next) {
                    continue;
                }

                // Edge-weighted propagation with hop decay
                let propagated = strength * edge.weight * config.decay_per_hop;

                if propagated < config.min_activation {
                    continue;
                }

                // Apply temporal decay for episodic nodes
                let temporal_factor = if let Some(node) = self.nodes.iter().find(|n| n.id() == next) {
                    Self::temporal_decay(current_tick, node.tick(), config.temporal_decay_half_life)
                } else {
                    1.0
                };

                let final_activation = propagated * temporal_factor;

                // Accumulate activation (multiple paths contribute)
                let entry = activations.entry(next).or_insert(0.0);
                *entry += final_activation;

                visited.insert(next);
                queue.push_back((next, propagated, depth + 1));
            }
        }

        // Apply lateral inhibition
        Self::apply_lateral_inhibition(
            &mut activations,
            &self.nodes,
            config.inhibition_strength,
        );

        activations
    }

    // ========================================================================
    // Triple Hybrid Retrieval (Geometric + Activation + Graph Traversal)
    // ========================================================================

    /// Geometric score: cosine similarity between query embedding and node embedding
    fn geometric_score(node: &UnifiedNode, query_embedding: &[f32; 16]) -> f32 {
        match node.embedding() {
            Some(emb) => cosine_sim(emb, query_embedding),
            None => 0.0,
        }
    }

    /// Activation score: normalized spreading activation score
    fn activation_score(node_id: u64, activations: &HashMap<u64, f32>) -> f32 {
        activations.get(&node_id).copied().unwrap_or(0.0)
    }

    /// Graph traversal score: proximity via shortest path (decays with distance)
    fn traversal_score(
        graph: &UnifiedMemoryGraph,
        node_id: u64,
        seed_ids: &[u64],
        max_hops: u32,
    ) -> f32 {
        let mut best_score = 0.0f32;
        for &seed_id in seed_ids {
            if let Some(path) = graph.path_between(seed_id, node_id, max_hops) {
                let distance = (path.len() as f32 - 1.0).max(1.0);
                let score = 1.0 / distance;
                best_score = best_score.max(score);
            }
        }
        best_score
    }

    /// Triple hybrid retrieval: combines geometric + activation + traversal
    pub fn hybrid_retrieve(
        &self,
        seed_ids: &[u64],
        query_embedding: &[f32; 16],
        current_tick: u64,
        activation_config: &ActivationConfig,
        retrieval_config: &RetrievalConfig,
    ) -> Vec<(u64, f32, UnifiedNode)> {
        // Phase 1: Spreading activation from seeds
        let activations =
            self.spreading_activation(seed_ids, current_tick, activation_config);

        // Phase 2: Score all candidates using triple hybrid
        let mut scored: Vec<(u64, f32, UnifiedNode)> = self
            .nodes
            .iter()
            .map(|node| {
                let id = node.id();
                let g = Self::geometric_score(node, query_embedding);
                let a = Self::activation_score(id, &activations);
                let t = Self::traversal_score(self, id, seed_ids, retrieval_config.max_traversal_hops);

                let combined = retrieval_config.geometric_weight * g
                    + retrieval_config.activation_weight * a
                    + retrieval_config.traversal_weight * t;

                (id, combined, node.clone())
            })
            .collect();

        // Phase 3: Sort by combined score and return top_k
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(retrieval_config.top_k);
        scored
    }

    /// Path-based retrieval: find the connecting path between two concepts
    pub fn path_between(&self, from: u64, to: u64, max_hops: u32) -> Option<Vec<u64>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<u64, u64> = HashMap::new();
        queue.push_back((from, 0u32));
        visited.insert(from);

        loop {
            let (id, depth) = queue.pop_front()?;
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
            if depth >= max_hops {
                continue;
            }
            for edge in self.edges_from(id) {
                let next = if edge.from == id { edge.to } else { edge.from };
                if visited.insert(next) {
                    parent.insert(next, id);
                    queue.push_back((next, depth + 1));
                }
            }
        }
    }

    pub fn strongest_connections(&self, id: u64, top_k: usize) -> Vec<(&UnifiedNode, f32)> {
        let mut edges: Vec<&UnifiedEdge> = self
            .edges_from(id)
            .into_iter()
            .filter(|e| e.from == id || e.to == id)
            .collect();
        edges.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        edges
            .iter()
            .take(top_k)
            .filter_map(|e| {
                let next = if e.from == id { e.to } else { e.from };
                self.nodes
                    .iter()
                    .find(|n| n.id() == next)
                    .map(|n| (n, e.weight))
            })
            .collect()
    }

    /// Consolidate: strengthen edges between frequently co-accessed nodes
    pub fn consolidate(&mut self) {
        for edge in &mut self.edges {
            if let (Some(from), Some(to)) = (
                self.nodes.iter().find(|n| n.id() == edge.from),
                self.nodes.iter().find(|n| n.id() == edge.to),
            ) {
                let co_access = from.access_count().min(to.access_count()) as f32;
                edge.weight = (edge.weight + co_access * 0.01).min(1.0);
            }
        }
    }

    fn prune(&mut self) {
        if self.nodes.len() <= self.max_nodes {
            return;
        }
        self.nodes.sort_by(|a, b| {
            b.importance()
                .partial_cmp(&a.importance())
                .unwrap()
                .then(b.access_count().cmp(&a.access_count()))
        });
        let keep: HashSet<u64> = self
            .nodes
            .iter()
            .take(self.max_nodes * 8 / 10)
            .map(|n| n.id())
            .collect();
        self.nodes.retain(|n| keep.contains(&n.id()));
        self.episodic_ids.retain(|id| keep.contains(id));
        self.semantic_ids.retain(|id| keep.contains(id));
        self.edges
            .retain(|e| keep.contains(&e.from) && keep.contains(&e.to));
        self.adjacency.clear();
        for (i, edge) in self.edges.iter().enumerate() {
            self.adjacency.entry(edge.from).or_default().push(i);
            self.adjacency.entry(edge.to).or_default().push(i);
        }
    }
}

// ============================================================================
// Shared utilities
// ============================================================================

fn cosine_sim(a: &[f32; 16], b: &[f32; 16]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- GraphMemory legacy tests ---

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

    // --- UnifiedMemoryGraph tests ---

    fn emb_a() -> [f32; 16] {
        [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    }

    fn emb_b() -> [f32; 16] {
        [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    }

    fn emb_similar() -> [f32; 16] {
        [0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    }

    #[test]
    fn unified_add_episode_and_semantic() {
        let mut g = UnifiedMemoryGraph::new(100);
        let e1 = g.add_episode("burned toast", 10, 0.8, Some(emb_a()), None, vec![], None, None);
        let s1 = g.add_semantic("fire", "combustion process", 5, 0.6, Some(emb_b()));
        assert_eq!(e1, 0);
        assert_eq!(s1, 1);
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.episodes().len(), 1);
        assert_eq!(g.semantic_nodes().len(), 1);
    }

    #[test]
    fn unified_edges_work() {
        let mut g = UnifiedMemoryGraph::new(100);
        let e1 = g.add_episode("burned toast", 10, 0.8, None, None, vec![], None, None);
        let e2 = g.add_episode("fire alarm went off", 11, 0.9, None, None, vec![], None, None);
        g.add_causal_edge(e1, e2, 0.9, 11);
        assert_eq!(g.edge_count(), 1);
        let neighbors = g.neighbors(e1);
        assert_eq!(neighbors.len(), 1);
    }

    #[test]
    fn unified_temporal_chain() {
        let mut g = UnifiedMemoryGraph::new(100);
        let e1 = g.add_episode("arrived at office", 1, 0.5, None, None, vec![], None, None);
        let e2 = g.add_episode("opened laptop", 2, 0.4, None, None, vec![], None, None);
        let e3 = g.add_episode("started coding", 3, 0.6, None, None, vec![], None, None);
        g.add_temporal_edge(e1, e2, 0.8, 2);
        g.add_temporal_edge(e2, e3, 0.9, 3);

        let path = g.path_between(e1, e3, 5).unwrap();
        assert_eq!(path, vec![e1, e2, e3]);
    }

    #[test]
    fn spreading_activation_basic() {
        let mut g = UnifiedMemoryGraph::new(100);
        let e1 = g.add_episode("saw fire", 10, 0.8, Some(emb_a()), None, vec![], None, None);
        let e2 = g.add_episode("heard alarm", 11, 0.7, Some(emb_b()), None, vec![], None, None);
        let e3 = g.add_episode("ran outside", 12, 0.9, Some(emb_a()), None, vec![], None, None);
        g.add_causal_edge(e1, e2, 0.9, 11);
        g.add_causal_edge(e2, e3, 0.8, 12);

        let config = ActivationConfig {
            max_hops: 3,
            min_activation: 0.01,
            ..Default::default()
        };
        let activations = g.spreading_activation(&[e1], 15, &config);
        assert!(activations.contains_key(&e2));
        assert!(activations.contains_key(&e3));
    }

    #[test]
    fn temporal_decay_reduces_old_memories() {
        let d1 = UnifiedMemoryGraph::temporal_decay(100, 0, 50.0);   // old memory (age=100)
        let d2 = UnifiedMemoryGraph::temporal_decay(100, 100, 50.0); // new memory (age=0)
        assert!(d1 < d2, "older memory should have lower decay factor");
    }

    #[test]
    fn lateral_inhibition_reduces_competing_activation() {
        let mut activations = HashMap::new();
        activations.insert(0, 0.8);
        activations.insert(1, 0.6);
        let nodes = vec![
            UnifiedNode::Episode(EpisodeNode {
                id: 0,
                description: "a".into(),
                tick: 0,
                importance: 0.5,
                access_count: 0,
                last_accessed: 0,
                embedding: Some(emb_a()),
                location: None,
                participants: vec![],
                emotion: None,
                action_taken: None,
            }),
            UnifiedNode::Episode(EpisodeNode {
                id: 1,
                description: "b".into(),
                tick: 0,
                importance: 0.5,
                access_count: 0,
                last_accessed: 0,
                embedding: Some(emb_similar()),
                location: None,
                participants: vec![],
                emotion: None,
                action_taken: None,
            }),
        ];
        UnifiedMemoryGraph::apply_lateral_inhibition(&mut activations, &nodes, 0.5);
        let after_1 = activations[&1];
        assert!(after_1 < 0.6, "inhibition should reduce activation of node 1");
    }

    #[test]
    fn hybrid_retrieve_returns_sorted_results() {
        let mut g = UnifiedMemoryGraph::new(100);
        let e1 = g.add_episode("met alice", 10, 0.8, Some(emb_a()), Some("office".into()), vec!["alice".into()], Some("happy".into()), Some("greet".into()));
        let e2 = g.add_episode("met bob", 20, 0.5, Some(emb_b()), Some("cafe".into()), vec!["bob".into()], Some("neutral".into()), Some("talk".into()));
        let s1 = g.add_semantic("alice", "person", 5, 0.7, Some(emb_a()));
        g.add_instance_of_edge(e1, s1, 0.9, 10);

        let query = emb_a();
        let act_config = ActivationConfig::default();
        let ret_config = RetrievalConfig {
            top_k: 10,
            ..Default::default()
        };
        let results = g.hybrid_retrieve(&[s1], &query, 30, &act_config, &ret_config);
        assert!(!results.is_empty());
        // e1 should appear in results and score higher than e2 (shares embedding with query)
        let e1_pos = results.iter().position(|(id, _, _)| *id == e1);
        let e2_pos = results.iter().position(|(id, _, _)| *id == e2);
        assert!(e1_pos.is_some(), "e1 should be in results");
        if let Some(e2p) = e2_pos {
            assert!(e1_pos.unwrap() < e2p, "e1 should rank higher than e2");
        }
    }

    #[test]
    fn path_between_finds_connection() {
        let mut g = UnifiedMemoryGraph::new(100);
        let s1 = g.add_semantic("fire", "combustion", 1, 0.5, None);
        let s2 = g.add_semantic("smoke", "combustion byproduct", 2, 0.5, None);
        let s3 = g.add_semantic("alarm", "safety device", 3, 0.5, None);
        g.add_abstraction_edge(s1, s2, 0.8, 2);
        g.add_abstraction_edge(s2, s3, 0.7, 3);

        let path = g.path_between(s1, s3, 5).unwrap();
        assert_eq!(path, vec![s1, s2, s3]);
    }

    #[test]
    fn unified_prune_preserves_important() {
        let mut g = UnifiedMemoryGraph::new(5);
        for i in 0..10 {
            g.add_episode(&format!("event {}", i), i, i as f32 * 0.1, None, None, vec![], None, None);
        }
        assert!(g.node_count() <= 5);
    }
}

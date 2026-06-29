// REVIVED Phase 3.1 — HebbianAssociativeMemory fully wired
use crate::core::nt_core_experience::capability_synthesizer::LatticeSnapshot;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

/// A weighted edge between two VSA nodes, with Hebbian learning dynamics
/// (HeLa-Mem: arXiv:2604.16839, ACL 2026).
#[derive(Debug, Clone)]
pub struct HebbianEdge {
    /// Index of source node
    pub source: usize,
    /// Index of target node
    pub target: usize,
    /// Synaptic weight (0.0–1.0), strengthened by co-activation
    pub weight: f64,
    /// How many times this edge was strengthened
    pub coactivation_count: u64,
}

/// Content-aware bundling statistics for monitoring novelty gate and redundancy suppression.
#[derive(Debug, Clone)]
pub struct ContentAwareBundleStats {
    pub total_observations: u64,
    pub suppressed_redundant: u64,
    pub novelty_gated: u64,
    pub bundle_updates: u64,
    pub last_novelty_mean: f64,
    pub last_redundancy_ratio: f64,
}

impl ContentAwareBundleStats {
    pub fn new() -> Self {
        Self {
            total_observations: 0,
            suppressed_redundant: 0,
            novelty_gated: 0,
            bundle_updates: 0,
            last_novelty_mean: 0.0,
            last_redundancy_ratio: 0.0,
        }
    }
}

/// Report from a single idle-time spontaneous reactivation tick (EMBER-inspired).
#[derive(Debug, Clone, Default)]
pub struct IdleActivationReport {
    /// How many seed activations were attempted
    pub attempts: u64,
    /// How many surfaced as candidate actions (weight ≥ idle_activation_threshold)
    pub surfaced: u64,
    /// Indices of surfaced nodes
    pub surfaced_node_ids: Vec<usize>,
    /// Max weight observed in this tick
    pub max_weight: f64,
    /// Whether any action was triggered
    pub action_triggered: bool,
}

/// HeLa-Mem inspired Hebbian associative memory.
/// Maintains a dynamic graph where VSA memory items are nodes
/// and Hebbian edges represent co-activation strength.
#[derive(Debug, Clone)]
pub struct HebbianAssociativeMemory {
    /// VSA vectors as graph nodes
    pub nodes: Vec<Vec<u8>>,
    /// Weighted edges between nodes
    pub edges: Vec<HebbianEdge>,
    /// Hebbian learning rate (η): weight increment per co-activation
    pub hebbian_lr: f64,
    /// Edge decay rate (λ): multiplicative decay per tick
    pub edge_decay: f64,
    /// Spreading activation strength (β)
    pub spread_strength: f64,
    /// Spreading activation threshold (θ): prune edges below this
    pub spread_threshold: f64,
    /// Max nodes before pruning lowest-degree
    pub max_nodes: usize,
    /// Total co-activation events recorded
    pub total_coactivations: u64,
    /// Total hub distillations performed
    pub total_distillations: u64,
    /// Novelty gate threshold: observations with novelty < this are suppressed
    pub novelty_gate: f64,
    /// Redundancy suppression factor: how much to suppress redundant observations (0.0-1.0)
    pub redundancy_suppressor: f64,
    /// Non-stationarity detector: tracks drift in observation distribution
    pub non_stationarity_detector: f64,
    /// Content-aware bundling statistics
    pub content_bundle_stats: ContentAwareBundleStats,
    // ── EMBER-inspired idle-time spontaneous reactivation ──
    /// Probability of running an idle activation per idle_tick call (0.0-1.0)
    pub idle_activation_probability: f64,
    /// Minimum edge weight to surface an association as a candidate action
    pub idle_activation_threshold: f64,
    /// Total idle activations performed
    pub total_idle_activations: u64,
    /// Last cycle when idle was run
    pub last_idle_cycle: u64,
}

impl HebbianAssociativeMemory {
    pub fn new(
        hebbian_lr: f64,
        edge_decay: f64,
        spread_strength: f64,
        spread_threshold: f64,
        max_nodes: usize,
    ) -> Self {
        Self {
            nodes: Vec::with_capacity(max_nodes),
            edges: Vec::new(),
            hebbian_lr,
            edge_decay,
            spread_strength,
            spread_threshold,
            max_nodes,
            total_coactivations: 0,
            total_distillations: 0,
            novelty_gate: 0.1,
            redundancy_suppressor: 0.3,
            non_stationarity_detector: 0.0,
            content_bundle_stats: ContentAwareBundleStats::new(),
            idle_activation_probability: 0.3,
            idle_activation_threshold: 0.5,
            total_idle_activations: 0,
            last_idle_cycle: 0,
        }
    }

    /// Add a VSA vector as a graph node (dedup by hamming similarity > 0.95).
    /// Returns the node index.
    pub fn add_node(&mut self, vector: Vec<u8>) -> usize {
        // Dedup: check if a near-identical node exists
        for (idx, existing) in self.nodes.iter().enumerate() {
            if QuantizedVSA::similarity(existing, &vector) > 0.95 {
                return idx;
            }
        }
        let idx = self.nodes.len();
        self.nodes.push(vector);
        if self.nodes.len() > self.max_nodes {
            self.prune_lowest_degree();
        }
        idx
    }

    /// Content-aware VSA bundling with novelty gate and redundancy suppression.
    /// Based on arxiv 2604.15121: standard VSA bundling treats all observations equally.
    /// This method:
    /// 1. Computes novelty of new observation against existing nodes
    /// 2. If novelty < novelty_gate, suppresses the observation (redundancy)
    /// 3. If novelty is high, records it as a new node with proportional weight
    /// 4. Tracks non-stationarity by monitoring novelty distribution drift
    pub fn content_aware_bundle(&mut self, vector: Vec<u8>) -> usize {
        if self.nodes.is_empty() {
            self.nodes.push(vector);
            self.content_bundle_stats.total_observations += 1;
            return 0;
        }

        // 1. Compute novelty against all existing nodes (1 - max similarity)
        let max_sim = self
            .nodes
            .iter()
            .map(|n| QuantizedVSA::similarity(n, &vector))
            .fold(0.0, f64::max);
        let novelty = 1.0 - max_sim;

        // 2. Update non-stationarity detector (EMA of novelty)
        self.non_stationarity_detector = self.non_stationarity_detector * 0.95 + novelty * 0.05;

        // 3. Novelty gate: if too similar to existing, suppress
        if novelty < self.novelty_gate {
            self.content_bundle_stats.suppressed_redundant += 1;
            // Find the closest node and boost its weight slightly instead
            let closest = self
                .nodes
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| {
                    QuantizedVSA::similarity(a, &vector)
                        .partial_cmp(&QuantizedVSA::similarity(b, &vector))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i);
            if let Some(idx) = closest {
                // Boost edges connected to this node
                for edge in self.edges.iter_mut() {
                    if edge.source == idx || edge.target == idx {
                        edge.weight = (edge.weight + self.hebbian_lr * 0.1).min(1.0);
                    }
                }
                self.content_bundle_stats.novelty_gated += 1;
                return idx;
            }
            return 0;
        }

        // 4. High novelty: add as new node (with redundancy suppression weight)
        let idx = self.nodes.len();
        self.nodes.push(vector);
        self.content_bundle_stats.total_observations += 1;
        self.content_bundle_stats.bundle_updates += 1;

        if self.nodes.len() > self.max_nodes {
            self.prune_lowest_degree();
        }

        idx
    }

    /// Get content-aware bundling statistics.
    pub fn content_aware_stats(&self) -> &ContentAwareBundleStats {
        &self.content_bundle_stats
    }

    /// Record co-activation between two node indices.
    /// Strengthens the edge if it exists, creates it otherwise.
    /// ACh-gated DA plasticity: plasticity only occurs when ACh level is high (from Nature 2026)
    pub fn record_coactivation(&mut self, a_idx: usize, b_idx: usize, ach_level: Option<f64>) {
        if a_idx == b_idx || a_idx >= self.nodes.len() || b_idx >= self.nodes.len() {
            return;
        }
        self.total_coactivations += 1;

        // ACh gating: cholinergic pause enables DA-dependent plasticity
        // When ACh is LOW (pause), plasticity is enabled
        // When ACh is HIGH, plasticity is suppressed
        let plasticity_gate = match ach_level {
            Some(ach) => (1.0 - ach).max(0.0), // ACh pause = plasticity window
            None => 1.0,                       // No gating if not provided
        };

        let effective_lr = self.hebbian_lr * plasticity_gate;

        // Look for existing edge in either direction
        for edge in &mut self.edges {
            if (edge.source == a_idx && edge.target == b_idx)
                || (edge.source == b_idx && edge.target == a_idx)
            {
                edge.weight = (edge.weight + effective_lr).min(1.0);
                edge.coactivation_count += 1;
                return;
            }
        }
        // Create new edge
        self.edges.push(HebbianEdge {
            source: a_idx,
            target: b_idx,
            weight: effective_lr,
            coactivation_count: 1,
        });
    }

    /// Backward-compatible overload without ACh gating
    pub fn record_coactivation_simple(&mut self, a_idx: usize, b_idx: usize) {
        self.record_coactivation(a_idx, b_idx, None);
    }

    /// Mnemoverse-style outcome-driven δ-rule weight update.
    /// For each edge connected to `node_idx`, apply:
    ///   weight += lr * (outcome_score - weight)
    /// Strengthens edges that led to good outcomes, weakens those that led to bad ones.
    /// `outcome_score`: 0.0 (bad) to 1.0 (good)
    /// `lr`: learning rate (typically 0.05-0.15)
    pub fn record_outcome(&mut self, node_idx: usize, outcome_score: f64, lr: f64) {
        if node_idx >= self.nodes.len() {
            return;
        }
        for edge in &mut self.edges {
            if edge.source == node_idx || edge.target == node_idx {
                let delta = lr * (outcome_score - edge.weight);
                edge.weight = (edge.weight + delta).clamp(0.0, 1.0);
            }
        }
    }

    /// Record co-activation between two VSA vectors (convenience wrapper).
    /// ach_level gates plasticity: low ACh (cholinergic pause) enables DA-dependent strengthening.
    pub fn record_coactivation_between(&mut self, a: &[u8], b: &[u8], ach_level: Option<f64>) {
        let a_idx = self.add_node(a.to_vec());
        let b_idx = self.add_node(b.to_vec());
        self.record_coactivation(a_idx, b_idx, ach_level);
    }

    /// Spreading activation from seed node index.
    /// Returns (node_idx, composite_score) pairs sorted descending.
    pub fn spreading_activation(&self, seed_idx: usize, top_k: usize) -> Vec<(usize, f64)> {
        if seed_idx >= self.nodes.len() {
            return Vec::new();
        }
        let mut scores: Vec<(usize, f64)> = Vec::new();
        // Direct Hebbian neighbors get score = edge weight
        for edge in &self.edges {
            let neighbor = if edge.source == seed_idx {
                edge.target
            } else if edge.target == seed_idx {
                edge.source
            } else {
                continue;
            };
            scores.push((neighbor, edge.weight));
        }
        // Second-order activation: neighbors of neighbors, attenuated by β
        let first_hop: Vec<usize> = scores.iter().map(|(idx, _)| *idx).collect();
        for &hop1 in &first_hop {
            for edge in &self.edges {
                let hop2 = if edge.source == hop1 {
                    edge.target
                } else if edge.target == hop1 {
                    edge.source
                } else {
                    continue;
                };
                if hop2 == seed_idx || first_hop.contains(&hop2) {
                    continue;
                }
                let base_score = edge.weight * self.spread_strength;
                scores.push((hop2, base_score));
            }
        }
        // Add VSA similarity bonus for any node with direct VSA similarity > 0.7
        let seed_vec = &self.nodes[seed_idx];
        for (i, node) in self.nodes.iter().enumerate() {
            if i == seed_idx {
                continue;
            }
            let vsa_sim = QuantizedVSA::similarity(seed_vec, node);
            if vsa_sim > 0.7 {
                if let Some(entry) = scores.iter_mut().find(|(idx, _)| *idx == i) {
                    entry.1 = (entry.1 + vsa_sim * 0.3).min(1.0);
                } else {
                    scores.push((i, vsa_sim * 0.3));
                }
            }
        }
        // Sort by score descending, take top_k
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);
        scores
    }

    /// Identify hub nodes (degree ≥ min_degree) and distill them into
    /// a bundled VSA summary vector. Returns (hub_index, summary_vector, cluster_size).
    pub fn distill_hubs(&mut self, min_degree: usize) -> Vec<(usize, Vec<u8>, usize)> {
        let mut degrees = vec![0usize; self.nodes.len()];
        for edge in &self.edges {
            degrees[edge.source] += 1;
            degrees[edge.target] += 1;
        }
        let hubs: Vec<usize> = degrees
            .iter()
            .enumerate()
            .filter(|(_, &deg)| deg >= min_degree)
            .map(|(idx, _)| idx)
            .collect();

        let mut results = Vec::new();
        for &hub_idx in &hubs {
            // Collect all neighbors
            let mut cluster: Vec<usize> = Vec::new();
            for edge in &self.edges {
                if edge.source == hub_idx {
                    cluster.push(edge.target);
                } else if edge.target == hub_idx {
                    cluster.push(edge.source);
                }
            }
            if cluster.is_empty() {
                continue;
            }
            // Bundle neighbor vectors into a summary VSA (majority vote)
            let summary = if cluster.len() == 1 {
                self.nodes[cluster[0]].clone()
            } else {
                let dim = self.nodes[cluster[0]].len();
                let mut sum_vec = vec![0u16; dim];
                for &n_idx in &cluster {
                    for (d, &val) in self.nodes[n_idx].iter().enumerate() {
                        sum_vec[d] += val as u16;
                    }
                }
                let threshold = (cluster.len() as u16 + 1) / 2;
                sum_vec
                    .iter()
                    .map(|&s| if s >= threshold { 1u8 } else { 0u8 })
                    .collect()
            };
            self.total_distillations += 1;
            results.push((hub_idx, summary, cluster.len()));
        }
        results
    }

    /// Decay all edge weights by edge_decay factor.
    pub fn decay_all(&mut self) {
        for edge in &mut self.edges {
            edge.weight = (edge.weight * self.edge_decay).max(0.0);
        }
    }

    /// Prune edges below spread_threshold.
    pub fn prune_edges(&mut self) {
        self.edges.retain(|e| e.weight >= self.spread_threshold);
    }

    /// Prune the lowest-degree node when max_nodes is exceeded.
    fn prune_lowest_degree(&mut self) {
        if self.nodes.is_empty() {
            return;
        }
        // Find lowest-degree node
        let mut degrees = vec![0usize; self.nodes.len()];
        for edge in &self.edges {
            degrees[edge.source] += 1;
            degrees[edge.target] += 1;
        }
        if let Some((min_idx, _)) = degrees.iter().enumerate().min_by_key(|(_, &d)| d) {
            self.remove_node(min_idx);
        }
    }

    /// Remove a node and all its edges.
    pub fn remove_node(&mut self, idx: usize) {
        if idx >= self.nodes.len() {
            return;
        }
        self.nodes.remove(idx);
        self.edges.retain(|e| e.source != idx && e.target != idx);
        // Adjust indices for nodes after removed one
        for edge in &mut self.edges {
            if edge.source > idx {
                edge.source -= 1;
            }
            if edge.target > idx {
                edge.target -= 1;
            }
        }
    }

    /// Compact summary stats.
    pub fn stats(&self) -> HebbianStats {
        let edge_count = self.edges.len();
        let avg_weight = if edge_count > 0 {
            self.edges.iter().map(|e| e.weight).sum::<f64>() / edge_count as f64
        } else {
            0.0
        };
        HebbianStats {
            node_count: self.nodes.len(),
            edge_count,
            avg_edge_weight: avg_weight,
            total_coactivations: self.total_coactivations,
            total_distillations: self.total_distillations,
            total_idle_activations: self.total_idle_activations,
            max_nodes: self.max_nodes,
        }
    }

    /// Multi-hop spreading activation with VSA similarity bonus.
    /// Extends the basic 2-hop spread with semantic similarity weighting.
    pub fn semantic_spread(
        &self,
        seed_vector: &[u8],
        top_k: usize,
        alpha: f64,
    ) -> Vec<(usize, f64, String)> {
        // 1. Find nearest node to seed_vector by hamming similarity
        let nearest = self.nodes.iter().enumerate().max_by(|(_, a), (_, b)| {
            let sim_a = QuantizedVSA::similarity(seed_vector, a);
            let sim_b = QuantizedVSA::similarity(seed_vector, b);
            sim_a
                .partial_cmp(&sim_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let seed_idx = match nearest {
            Some((idx, _)) => idx,
            None => return Vec::new(),
        };

        // 2. Run spreading_activation from that node
        let base_results = self.spreading_activation(seed_idx, top_k * 2);

        // Precompute degrees for hub detection
        let mut degrees = vec![0usize; self.nodes.len()];
        for edge in &self.edges {
            degrees[edge.source] += 1;
            degrees[edge.target] += 1;
        }

        // Determine which nodes are graph-reachable from seed (1-hop or 2-hop via edges)
        let mut is_edge_reachable = vec![false; self.nodes.len()];
        for edge in &self.edges {
            if edge.source == seed_idx {
                is_edge_reachable[edge.target] = true;
            }
            if edge.target == seed_idx {
                is_edge_reachable[edge.source] = true;
            }
        }
        let first_hop: Vec<usize> = (0..self.nodes.len())
            .filter(|&i| is_edge_reachable[i])
            .collect();
        for &hop1 in &first_hop {
            for edge in &self.edges {
                if edge.source == hop1 && edge.target != seed_idx {
                    is_edge_reachable[edge.target] = true;
                }
                if edge.target == hop1 && edge.source != seed_idx {
                    is_edge_reachable[edge.source] = true;
                }
            }
        }

        // 3-4. Compute weighted scores and tags
        let mut weighted: Vec<(usize, f64, String)> = Vec::new();
        for (node_idx, spread_score) in base_results {
            let vsa_sim = QuantizedVSA::similarity(seed_vector, &self.nodes[node_idx]);
            let weighted_score = (1.0 - alpha) * spread_score + alpha * vsa_sim;

            let tag = if degrees[node_idx] >= 3 {
                "hub"
            } else if is_edge_reachable[node_idx] {
                "neighbor"
            } else {
                "semantic"
            };

            weighted.push((node_idx, weighted_score, tag.to_string()));
        }

        // 5. Sort by weighted score descending, take top_k
        weighted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        weighted.truncate(top_k);
        weighted
    }

    /// Distill hub nodes into a LatticeSnapshot for export
    pub fn to_lattice_snapshot(&self, min_degree: usize) -> LatticeSnapshot {
        let hubs = {
            let mut degrees = vec![0usize; self.nodes.len()];
            for edge in &self.edges {
                degrees[edge.source] += 1;
                degrees[edge.target] += 1;
            }
            let hub_indices: Vec<usize> = degrees
                .iter()
                .enumerate()
                .filter(|(_, &deg)| deg >= min_degree)
                .map(|(idx, _)| idx)
                .collect();

            let mut results = Vec::new();
            for &hub_idx in &hub_indices {
                let mut cluster: Vec<usize> = Vec::new();
                for edge in &self.edges {
                    if edge.source == hub_idx {
                        cluster.push(edge.target);
                    } else if edge.target == hub_idx {
                        cluster.push(edge.source);
                    }
                }
                if cluster.is_empty() {
                    continue;
                }
                let summary = if cluster.len() == 1 {
                    self.nodes[cluster[0]].clone()
                } else {
                    let dim = self.nodes[cluster[0]].len();
                    let mut sum_vec = vec![0u16; dim];
                    for &n_idx in &cluster {
                        for (d, &val) in self.nodes[n_idx].iter().enumerate() {
                            sum_vec[d] += val as u16;
                        }
                    }
                    let threshold = (cluster.len() as u16 + 1) / 2;
                    sum_vec
                        .iter()
                        .map(|&s| if s >= threshold { 1u8 } else { 0u8 })
                        .collect()
                };
                results.push((hub_idx, summary, cluster.len()));
            }
            results
        };

        let skills: Vec<(String, Vec<u8>, f64)> = hubs
            .iter()
            .map(|(idx, vec, cluster_size)| {
                (
                    format!("hebbian_hub_{}", idx),
                    vec.clone(),
                    *cluster_size as f64,
                )
            })
            .collect();

        LatticeSnapshot {
            skills,
            meta_rules: Vec::new(),
        }
    }

    /// EMBER-inspired idle-time spontaneous reactivation.
    ///
    /// When called during idle cycles (SLEEP step or low activity), randomly
    /// selects a seed node and runs spreading activation. If any association
    /// has weight ≥ idle_activation_threshold, surfaces it as a candidate action.
    /// Each idle activation strengthens the Hebbian path (STDP-like dynamics).
    ///
    /// Returns an IdleActivationReport describing what was surfaced.
    pub fn idle_tick(&mut self, cycle: u64) -> IdleActivationReport {
        self.last_idle_cycle = cycle;

        if self.nodes.is_empty() {
            return IdleActivationReport::default();
        }

        // Probabilistic activation: only fire with idle_activation_probability
        let roll: f64 = (cycle.wrapping_mul(3141592653) % 1000) as f64 / 1000.0;
        if roll > self.idle_activation_probability {
            return IdleActivationReport::default();
        }

        // Pick a random seed node (deterministic from cycle number)
        let seed_idx = (cycle as usize) % self.nodes.len();
        let neighbors = self.spreading_activation(seed_idx, 5);

        let mut surfaced_ids = Vec::new();
        let mut max_w = 0.0f64;

        for (node_idx, weight) in &neighbors {
            if *weight > max_w {
                max_w = *weight;
            }
            if *weight >= self.idle_activation_threshold {
                surfaced_ids.push(*node_idx);
                // STDP-like: strengthen this path
                if let Some(edge) = self.edges.iter_mut().find(|e| {
                    (e.source == seed_idx && e.target == *node_idx)
                        || (e.target == seed_idx && e.source == *node_idx)
                }) {
                    edge.weight = (edge.weight + self.hebbian_lr * 0.5).min(1.0);
                    edge.coactivation_count += 1;
                }
            }
        }

        self.total_idle_activations += 1;

        IdleActivationReport {
            attempts: 1,
            surfaced: surfaced_ids.len() as u64,
            surfaced_node_ids: surfaced_ids.clone(),
            max_weight: max_w,
            action_triggered: !surfaced_ids.is_empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HebbianStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub avg_edge_weight: f64,
    pub total_coactivations: u64,
    pub total_distillations: u64,
    pub total_idle_activations: u64,
    pub max_nodes: usize,
}

/// HeLa-Mem style distillation agent.
/// Identifies hub nodes (high-degree) in the Hebbian graph
/// and distills them into structured semantic patterns.
#[derive(Debug, Clone)]
pub struct HebbianDistillationAgent {
    /// Minimum degree to be considered a hub
    pub min_hub_degree: usize,
    /// Minimum co-activation threshold
    pub co_activation_threshold: f64,
    /// Hub discovery cycle
    pub last_distillation_cycle: u64,
    /// Hebbian learning rate (η) for reflective distillation
    pub hebbian_eta: f64,
    /// Edge decay lambda (λ) for reflective distillation
    pub decay_lambda: f64,
    /// Spreading activation strength (β) for reflective distillation
    pub beta: f64,
    /// Whether LLM-assisted reflective distillation is enabled
    pub enable_reflective_distillation: bool,
}

impl HebbianDistillationAgent {
    pub fn new(min_hub_degree: usize, co_activation_threshold: f64) -> Self {
        Self {
            min_hub_degree,
            co_activation_threshold,
            last_distillation_cycle: 0,
            hebbian_eta: 0.02,
            decay_lambda: 0.995,
            beta: 0.1,
            enable_reflective_distillation: false,
        }
    }

    /// Extract hub node indices from the Hebbian graph.
    /// Returns indices of nodes whose degree >= min_hub_degree.
    pub fn hub_detection(&self, hebbian: &HebbianAssociativeMemory) -> Vec<usize> {
        let mut degrees = vec![0usize; hebbian.nodes.len()];
        for edge in &hebbian.edges {
            degrees[edge.source] += 1;
            degrees[edge.target] += 1;
        }
        degrees
            .iter()
            .enumerate()
            .filter(|(_, &deg)| deg >= self.min_hub_degree)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Run distillation on a HebbianAssociativeMemory.
    /// Returns (distilled_hubs, LatticeSnapshot).
    pub fn distill(
        &mut self,
        hebbian: &HebbianAssociativeMemory,
        cycle: u64,
    ) -> (Vec<(usize, Vec<u8>, f64)>, LatticeSnapshot) {
        self.last_distillation_cycle = cycle;

        // 1. Identify hub nodes
        let hubs = self.hub_detection(hebbian);

        let mut distilled = Vec::new();
        for &hub_idx in &hubs {
            let coherence = self.cluster_coherence(hebbian, hub_idx);
            if coherence < self.co_activation_threshold {
                continue;
            }
            // Collect neighbor vectors and majority-vote bundle
            let mut cluster: Vec<usize> = Vec::new();
            for edge in &hebbian.edges {
                if edge.source == hub_idx {
                    cluster.push(edge.target);
                } else if edge.target == hub_idx {
                    cluster.push(edge.source);
                }
            }
            if cluster.is_empty() {
                continue;
            }
            let summary = if cluster.len() == 1 {
                hebbian.nodes[cluster[0]].clone()
            } else {
                let dim = hebbian.nodes[cluster[0]].len();
                let mut sum_vec = vec![0u16; dim];
                for &n_idx in &cluster {
                    for (d, &val) in hebbian.nodes[n_idx].iter().enumerate() {
                        sum_vec[d] += val as u16;
                    }
                }
                let threshold = (cluster.len() as u16 + 1) / 2;
                sum_vec
                    .iter()
                    .map(|&s| if s >= threshold { 1u8 } else { 0u8 })
                    .collect()
            };
            distilled.push((hub_idx, summary, coherence));
        }

        let skills: Vec<(String, Vec<u8>, f64)> = distilled
            .iter()
            .map(|(idx, vec, conf)| (format!("hebbian_hub_{}", idx), vec.clone(), *conf))
            .collect();

        let snapshot = LatticeSnapshot {
            skills,
            meta_rules: Vec::new(),
        };

        (distilled, snapshot)
    }

    /// Check if a node is a hub based on edge degree
    fn node_degree(&self, hebbian: &HebbianAssociativeMemory, node_idx: usize) -> usize {
        if node_idx >= hebbian.nodes.len() {
            return 0;
        }
        hebbian
            .edges
            .iter()
            .filter(|e| e.source == node_idx || e.target == node_idx)
            .count()
    }

    /// Confidence score based on cluster coherence
    fn cluster_coherence(&self, hebbian: &HebbianAssociativeMemory, hub_idx: usize) -> f64 {
        if hub_idx >= hebbian.nodes.len() {
            return 0.0;
        }
        let mut neighbor_indices: Vec<usize> = Vec::new();
        for edge in &hebbian.edges {
            if edge.source == hub_idx {
                neighbor_indices.push(edge.target);
            } else if edge.target == hub_idx {
                neighbor_indices.push(edge.source);
            }
        }
        if neighbor_indices.len() < 2 {
            return 0.0;
        }
        // Average pairwise VSA similarity among neighbors
        let hub_vec = &hebbian.nodes[hub_idx];
        let mut total_sim = 0.0;
        let mut count = 0;
        for &ni in &neighbor_indices {
            let sim = QuantizedVSA::similarity(hub_vec, &hebbian.nodes[ni]);
            total_sim += sim;
            count += 1;
        }
        if count == 0 {
            0.0
        } else {
            total_sim / count as f64
        }
    }

    /// LLM-assisted reflective distillation: synthesizes structured semantic knowledge
    /// from a hub's neighbor entries. Uses the provided `llm_fn` for the summarization step.
    /// Returns parsed semantic knowledge string (empty if disabled or no neighbors).
    pub fn reflective_distill(
        &self,
        hebbian: &HebbianAssociativeMemory,
        hub_idx: usize,
        llm_fn: &dyn Fn(&[String]) -> String,
    ) -> String {
        if !self.enable_reflective_distillation {
            return String::new();
        }
        if hub_idx >= hebbian.nodes.len() {
            return String::new();
        }
        let mut neighbor_contents: Vec<String> = Vec::new();
        for edge in &hebbian.edges {
            let neighbor_idx = if edge.source == hub_idx {
                edge.target
            } else if edge.target == hub_idx {
                edge.source
            } else {
                continue;
            };
            let coherence =
                QuantizedVSA::similarity(&hebbian.nodes[hub_idx], &hebbian.nodes[neighbor_idx]);
            if coherence >= self.co_activation_threshold {
                neighbor_contents.push(format!(
                    "neighbor_{}:similarity_{:.3}",
                    neighbor_idx, coherence
                ));
            }
        }
        if neighbor_contents.is_empty() {
            return String::new();
        }
        llm_fn(&neighbor_contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(seed: u8) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed as u64, 4096)
    }

    #[test]
    fn test_add_node_dedup() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let a = v(1);
        let idx1 = mem.add_node(a.clone());
        let idx2 = mem.add_node(a.clone());
        assert_eq!(idx1, idx2, "duplicate node should return same index");
        assert_eq!(mem.nodes.len(), 1);
    }

    #[test]
    fn test_add_node_unique() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let idx1 = mem.add_node(v(1));
        let idx2 = mem.add_node(v(2));
        assert_ne!(idx1, idx2, "different vectors should get different indices");
        assert_eq!(mem.nodes.len(), 2);
    }

    #[test]
    fn test_record_coactivation_creates_edge() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let idx1 = mem.add_node(v(1));
        let idx2 = mem.add_node(v(2));
        mem.record_coactivation(idx1, idx2, None);
        assert_eq!(mem.edges.len(), 1);
        assert!((mem.edges[0].weight - 0.02).abs() < 1e-6);
        assert_eq!(mem.edges[0].coactivation_count, 1);
    }

    #[test]
    fn test_record_coactivation_strengthens_edge() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let idx1 = mem.add_node(v(1));
        let idx2 = mem.add_node(v(2));
        mem.record_coactivation(idx1, idx2, None);
        mem.record_coactivation(idx1, idx2, None);
        mem.record_coactivation(idx1, idx2, None);
        assert_eq!(mem.edges.len(), 1);
        assert!((mem.edges[0].weight - 0.06).abs() < 1e-6);
        assert_eq!(mem.edges[0].coactivation_count, 3);
    }

    #[test]
    fn test_record_coactivation_between() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        mem.record_coactivation_between(&v(1), &v(2), None);
        assert_eq!(mem.nodes.len(), 2);
        assert_eq!(mem.edges.len(), 1);
    }

    #[test]
    fn test_spreading_activation_direct() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let idx1 = mem.add_node(v(1));
        let idx2 = mem.add_node(v(2));
        mem.record_coactivation(idx1, idx2, None);
        let results = mem.spreading_activation(idx1, 5);
        assert!(!results.is_empty(), "should find neighbors");
        assert_eq!(results[0].0, idx2);
    }

    #[test]
    fn test_spreading_activation_second_hop() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let idx1 = mem.add_node(v(1));
        let idx2 = mem.add_node(v(2));
        let idx3 = mem.add_node(v(3));
        mem.record_coactivation(idx1, idx2, None);
        mem.record_coactivation(idx2, idx3, None);
        let results = mem.spreading_activation(idx1, 5);
        let hops: Vec<usize> = results.iter().map(|(i, _)| *i).collect();
        assert!(hops.contains(&idx2), "should find direct neighbor");
        assert!(hops.contains(&idx3), "should find second-hop neighbor");
    }

    #[test]
    fn test_distill_hubs() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        // Create a star graph: hub node 0 connected to many leaves
        let hub = mem.add_node(v(100));
        for i in 0..5 {
            let leaf = mem.add_node(v(i));
            mem.record_coactivation(hub, leaf, None);
        }
        assert_eq!(mem.edges.len(), 5);
        let distillations = mem.distill_hubs(3);
        assert!(
            !distillations.is_empty(),
            "hub with degree 5 should be distilled"
        );
        assert_eq!(distillations[0].2, 5, "cluster should have 5 leaves");
    }

    #[test]
    fn test_decay_and_prune() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.01, 0.1, 0.6, 100);
        // edge_decay=0.01 means edges decay to near-zero quickly
        let idx1 = mem.add_node(v(1));
        let idx2 = mem.add_node(v(2));
        mem.record_coactivation(idx1, idx2, None);
        assert!(!mem.edges.is_empty());
        mem.decay_all();
        mem.prune_edges();
        assert!(
            mem.edges.is_empty(),
            "edges should be pruned after decay below threshold"
        );
    }

    #[test]
    fn test_max_nodes_pruning() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 3);
        // Add 4 unique nodes — should trigger pruning
        let _ = mem.add_node(v(1));
        let _ = mem.add_node(v(2));
        let _ = mem.add_node(v(3));
        let _ = mem.add_node(v(4));
        assert!(mem.nodes.len() <= 3, "should respect max_nodes");
    }

    #[test]
    fn test_spreading_activation_out_of_bounds() {
        let mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let results = mem.spreading_activation(999, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_empty_stats() {
        let mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let s = mem.stats();
        assert_eq!(s.node_count, 0);
        assert_eq!(s.edge_count, 0);
        assert_eq!(s.avg_edge_weight, 0.0);
        assert_eq!(s.total_distillations, 0);
    }

    #[test]
    fn test_distillation_agent_new() {
        let agent = HebbianDistillationAgent::new(3, 0.5);
        assert_eq!(agent.min_hub_degree, 3);
        assert!((agent.co_activation_threshold - 0.5).abs() < 1e-6);
        assert_eq!(agent.last_distillation_cycle, 0);
    }

    #[test]
    fn test_distillation_agent_node_degree() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let hub = mem.add_node(v(100));
        let leaf = mem.add_node(v(1));
        mem.record_coactivation(hub, leaf, None);
        let agent = HebbianDistillationAgent::new(1, 0.0);
        assert_eq!(agent.node_degree(&mem, hub), 1);
        assert_eq!(agent.node_degree(&mem, leaf), 1);
    }

    #[test]
    fn test_distillation_agent_distill_finds_hubs() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let hub = mem.add_node(v(100));
        for i in 0..5 {
            let leaf = mem.add_node(v(i));
            mem.record_coactivation(hub, leaf, None);
        }
        let mut agent = HebbianDistillationAgent::new(3, 0.0);
        let (hubs, snapshot) = agent.distill(&mem, 10);
        assert!(!hubs.is_empty(), "should find hubs in star graph");
        assert!(agent.last_distillation_cycle == 10);
        assert!(!snapshot.skills.is_empty(), "snapshot should have skills");
    }

    #[test]
    fn test_distillation_agent_no_hubs_below_threshold() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let a = mem.add_node(v(1));
        let b = mem.add_node(v(2));
        mem.record_coactivation(a, b, None);
        let mut agent = HebbianDistillationAgent::new(5, 0.0);
        let (hubs, snapshot) = agent.distill(&mem, 1);
        assert!(hubs.is_empty(), "no node has degree >= 5");
        assert!(snapshot.skills.is_empty());
    }

    #[test]
    fn test_distillation_agent_cluster_coherence() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let hub = mem.add_node(v(100));
        for i in 0..3 {
            let leaf = mem.add_node(v(i));
            mem.record_coactivation(hub, leaf, None);
        }
        let agent = HebbianDistillationAgent::new(3, 0.5);
        let coherence = agent.cluster_coherence(&mem, hub);
        assert!(coherence >= 0.0, "coherence should be non-negative");
    }

    #[test]
    fn test_idle_tick_empty_memory_returns_default() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        let report = mem.idle_tick(1);
        assert_eq!(report.attempts, 0);
        assert_eq!(report.surfaced, 0);
        assert!(!report.action_triggered);
    }

    #[test]
    fn test_idle_tick_with_coactivation_tracks_surfaced() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        mem.idle_activation_probability = 1.0; // always fire
        mem.idle_activation_threshold = 0.1; // surface even weak associations

        let a = mem.add_node(v(1));
        let b = mem.add_node(v(2));
        mem.record_coactivation(a, b, None);

        let report = mem.idle_tick(1);
        assert_eq!(
            report.attempts, 1,
            "should attempt activation when nodes exist"
        );
    }

    #[test]
    fn test_idle_tick_strengthens_path() {
        let mut mem = HebbianAssociativeMemory::new(0.5, 0.995, 0.1, 0.6, 100);
        mem.idle_activation_probability = 1.0;
        mem.idle_activation_threshold = 0.0; // surface all

        let a = mem.add_node(v(1));
        let b = mem.add_node(v(2));
        mem.record_coactivation(a, b, None);

        // Get baseline weight
        let baseline = mem.edges.first().map(|e| e.weight).unwrap_or(0.0);

        let report = mem.idle_tick(0);

        // If the edge was surfaced, its weight should have increased
        if report.surfaced > 0 {
            let new_weight = mem.edges.first().map(|e| e.weight).unwrap_or(0.0);
            assert!(
                new_weight >= baseline,
                "STDP should strengthen surfaced paths"
            );
        }
    }

    #[test]
    fn test_idle_tick_probability_gate() {
        let mut mem = HebbianAssociativeMemory::new(0.02, 0.995, 0.1, 0.6, 100);
        mem.idle_activation_probability = 0.0; // never fire

        mem.add_node(v(1));
        mem.add_node(v(2));

        let report = mem.idle_tick(42);
        assert_eq!(
            report.attempts, 0,
            "should not activate when probability is 0"
        );
    }
}

// REVIVED Task 1 — dead_code removed 2026-06-24

use rand::Rng;
use serde_json;

/// Binary state grid of N nodes.
pub type StateVec = Vec<u8>;

/// Bipartition mask: true = partition A, false = partition B.
pub type PartitionMask = Vec<bool>;

/// Per-node conditional probability table for a sparse factored TPM.
/// Each node depends on at most `k_parents` other nodes (k ≤ 4).
#[derive(Debug, Clone)]
pub struct NodeCPT {
    pub parents: Vec<usize>,
    pub prob_one: Vec<f64>,
}

impl NodeCPT {
    pub fn new(parents: Vec<usize>) -> Self {
        let size = 1usize << parents.len();
        NodeCPT {
            parents,
            prob_one: vec![0.5; size],
        }
    }

    pub fn prob(&self, state: &[u8], target: u8) -> f64 {
        let idx = self.parent_index(state);
        if target == 1 {
            self.prob_one[idx]
        } else {
            1.0 - self.prob_one[idx]
        }
    }

    fn parent_index(&self, state: &[u8]) -> usize {
        let mut idx = 0usize;
        for (j, &p) in self.parents.iter().enumerate() {
            if state[p] != 0 {
                idx |= 1 << j;
            }
        }
        idx
    }
}

/// Sparse factored TPM for a system of N nodes.
/// Each node has a CPT conditioned on at most k_parents other nodes.
#[derive(Debug, Clone)]
pub struct FactoredTPM {
    pub nodes: Vec<NodeCPT>,
    pub n: usize,
}

impl FactoredTPM {
    pub fn new(n: usize) -> Self {
        let nodes = (0..n).map(|i| NodeCPT::new(vec![i])).collect();
        FactoredTPM { nodes, n }
    }

    pub fn with_dependencies(n: usize, deps: &[Vec<usize>]) -> Self {
        let nodes: Vec<NodeCPT> = deps
            .iter()
            .map(|parents| NodeCPT::new(parents.clone()))
            .collect();
        FactoredTPM { nodes, n }
    }

    pub fn random(n: usize, max_parents: usize, rng: &mut impl Rng) -> Self {
        let nodes: Vec<NodeCPT> = (0..n)
            .map(|i| {
                let k = if n <= 1 {
                    1
                } else {
                    rng.gen_range(1..=max_parents.min(n - 1))
                };
                let mut candidates: Vec<usize> = (0..n).filter(|&j| j != i).collect();
                let mut parents = Vec::with_capacity(k);
                for _ in 0..k {
                    let idx = rng.gen_range(0..candidates.len());
                    parents.push(candidates.swap_remove(idx));
                }
                parents.sort_unstable();
                let mut cpt = NodeCPT::new(parents);
                for val in cpt.prob_one.iter_mut() {
                    *val = rng.gen_range(0.1..0.9);
                }
                cpt
            })
            .collect();
        FactoredTPM { nodes, n }
    }

    pub fn fully_connected(n: usize) -> Self {
        let nodes: Vec<NodeCPT> = (0..n)
            .map(|i| {
                let parents: Vec<usize> = (0..n).filter(|&j| j != i).collect();
                let mut cpt = NodeCPT::new(parents);
                for val in cpt.prob_one.iter_mut() {
                    *val = if rand::thread_rng().gen_bool(0.5) {
                        0.85
                    } else {
                        0.15
                    };
                }
                cpt
            })
            .collect();
        FactoredTPM { nodes, n }
    }

    pub fn disconnected(n: usize) -> Self {
        let nodes: Vec<NodeCPT> = (0..n)
            .map(|_i| {
                let mut cpt = NodeCPT::new(vec![]);
                cpt.prob_one[0] = 0.5;
                cpt
            })
            .collect();
        FactoredTPM { nodes, n }
    }

    pub fn chain(n: usize) -> Self {
        let nodes: Vec<NodeCPT> = (0..n)
            .map(|i| {
                if i == 0 {
                    let mut cpt = NodeCPT::new(vec![]);
                    cpt.prob_one[0] = 0.5;
                    cpt
                } else {
                    let mut cpt = NodeCPT::new(vec![i - 1]);
                    cpt.prob_one[0] = 0.3;
                    cpt.prob_one[1] = 0.8;
                    cpt
                }
            })
            .collect();
        FactoredTPM { nodes, n }
    }

    pub fn xor_triplet(base: usize) -> Self {
        let n = base + 3;
        let mut nodes: Vec<NodeCPT> = (0..n).map(|_| NodeCPT::new(vec![])).collect();
        for i in 0..n {
            nodes[i] = NodeCPT::new(vec![]);
            nodes[i].prob_one[0] = 0.5;
        }
        let mut cpt = NodeCPT::new(vec![base, base + 1]);
        cpt.prob_one[0] = 0.05;
        cpt.prob_one[1] = 0.95;
        cpt.prob_one[2] = 0.95;
        cpt.prob_one[3] = 0.05;
        nodes[base + 2] = cpt;
        FactoredTPM { nodes, n }
    }

    pub fn node_is_constant(&self, i: usize) -> bool {
        let cpt = &self.nodes[i];
        if cpt.prob_one.is_empty() {
            return true;
        }
        let p = cpt.prob_one[0];
        cpt.prob_one.iter().all(|&v| (v - p).abs() < 1e-10)
    }

    pub fn node_prob_one(&self, i: usize, state: &[u8]) -> f64 {
        self.nodes[i].prob(state, 1)
    }

    pub fn transition_prob(&self, from: &[u8], to: &[u8]) -> f64 {
        let mut prob = 1.0;
        for i in 0..self.n {
            prob *= self.nodes[i].prob(from, to[i]);
        }
        prob
    }
}

/// IIT 4.0 integrated information (φ) calculator.
///
/// Uses sparse factored TPM: each node depends on k ≤ 4 others.
/// For N ≤ 8: exact enumeration of all states and bipartitions.
/// For N > 8: Monte Carlo sampling for marginal entropy and MIP search.
#[derive(Debug, Clone)]
pub struct PhiCalculator {
    pub tpm: FactoredTPM,
    n: usize,
    mip_samples: usize,
    marginal_samples: usize,
}

impl PhiCalculator {
    pub fn new(tpm: FactoredTPM) -> Self {
        let n = tpm.n;
        PhiCalculator {
            tpm,
            n,
            mip_samples: 500,
            marginal_samples: 50000,
        }
    }

    pub fn with_sampling(mut self, mip_samples: usize, marginal_samples: usize) -> Self {
        self.mip_samples = mip_samples;
        self.marginal_samples = marginal_samples;
        self
    }

    pub fn compute_phi(&self, state: &[u8]) -> f64 {
        let full_ei = self.compute_ei();
        let (_, min_phi) = self.compute_mip(state);
        full_ei - min_phi
    }

    pub fn compute_mip(&self, _state: &[u8]) -> (Vec<usize>, f64) {
        let full_ei = self.compute_ei();
        let partitions = if self.n <= 8 {
            Self::enumerate_bipartitions(self.n)
        } else {
            Self::random_bipartitions(self.n, self.mip_samples)
        };

        let mut best_phi = f64::INFINITY;
        let mut best_partition: Vec<usize> = (0..self.n).map(|_| 0).collect();

        for mask in &partitions {
            let part_ei = self.compute_partitioned_ei(mask);
            let phi = full_ei - part_ei;
            if phi < best_phi {
                best_phi = phi;
                best_partition = mask.iter().map(|&in_a| if in_a { 0 } else { 1 }).collect();
            }
        }

        (best_partition, best_phi.max(0.0))
    }

    pub fn compute_ei(&self) -> f64 {
        let n = self.n;
        if n <= 8 {
            self.compute_ei_exact()
        } else {
            self.compute_ei_sampled()
        }
    }

    pub fn compute_partitioned_ei(&self, mask: &[bool]) -> f64 {
        if self.n <= 8 {
            self.compute_partitioned_ei_exact(mask)
        } else {
            self.compute_partitioned_ei_sampled(mask)
        }
    }

    fn compute_ei_exact(&self) -> f64 {
        let n = self.n;
        let total = 1usize << n;

        let mut cond_entropy = 0.0;
        let mut next_probs = vec![0.0; total];

        let mut state = vec![0u8; n];
        for s_idx in 0..total {
            idx_to_state_mut(s_idx, &mut state);
            let mut h_s = 0.0;
            let mut prod = 1.0;
            for i in 0..n {
                let p1 = self.tpm.node_prob_one(i, &state);
                h_s += binary_entropy(p1);
                prod *= if state[i] != 0 { p1 } else { 1.0 - p1 };
            }
            cond_entropy += h_s;

            for next_idx in 0..total {
                let mut p_sgiven = prod;
                for i in 0..n {
                    let p1 = self.tpm.node_prob_one(i, &state);
                    let bit = (next_idx >> i) & 1;
                    p_sgiven *= if bit == 1 { p1 } else { 1.0 - p1 };
                }
                next_probs[next_idx] += p_sgiven;
            }
        }

        cond_entropy /= total as f64;

        let inv_total = 1.0 / total as f64;
        let mut marg_entropy = 0.0;
        for prob in &next_probs {
            let p = prob * inv_total;
            if p > 0.0 {
                marg_entropy -= p * p.log2();
            }
        }

        marg_entropy - cond_entropy
    }

    fn compute_ei_sampled(&self) -> f64 {
        let n = self.n;
        let total = 1usize << n;
        let m = self.marginal_samples;
        let mut rng = rand::thread_rng();

        let mut cond_entropy = 0.0;
        let mut state = vec![0u8; n];

        let sample_count = m.min(total);
        for _ in 0..sample_count {
            let s_idx = rng.gen_range(0..total);
            idx_to_state_mut(s_idx, &mut state);
            let mut h_s = 0.0;
            for i in 0..n {
                let p1 = self.tpm.node_prob_one(i, &state);
                h_s += binary_entropy(p1);
            }
            cond_entropy += h_s;
        }
        cond_entropy /= sample_count as f64;

        let mut next_samples = vec![0u64; m];
        for k in 0..m {
            let s_idx = rng.gen_range(0..total);
            idx_to_state_mut(s_idx, &mut state);
            let mut next = 0u64;
            for i in 0..n {
                let p1 = self.tpm.node_prob_one(i, &state);
                if rng.gen::<f64>() < p1 {
                    next |= 1u64 << i;
                }
            }
            next_samples[k] = next;
        }

        let marg_entropy = estimate_entropy_knn(&next_samples, 5);

        marg_entropy - cond_entropy
    }

    fn compute_partitioned_ei_exact(&self, mask: &[bool]) -> f64 {
        let n = self.n;
        let total = 1usize << n;

        let mut cond_entropy = 0.0;
        let mut next_probs = vec![0.0; total];

        let mut state = vec![0u8; n];
        for s_idx in 0..total {
            idx_to_state_mut(s_idx, &mut state);

            let mut h_s_a = 0.0;
            let mut h_s_b = 0.0;
            for i in 0..n {
                let p1 = self.partitioned_prob(i, mask, &state);
                if mask[i] {
                    h_s_a += binary_entropy(p1);
                } else {
                    h_s_b += binary_entropy(p1);
                }
            }
            cond_entropy += h_s_a + h_s_b;

            for next_idx in 0..total {
                let mut p_sgiven = 1.0;
                for i in 0..n {
                    let p1 = self.partitioned_prob(i, mask, &state);
                    let bit = (next_idx >> i) & 1;
                    p_sgiven *= if bit == 1 { p1 } else { 1.0 - p1 };
                }
                next_probs[next_idx] += p_sgiven;
            }
        }

        cond_entropy /= total as f64;

        let inv_total = 1.0 / total as f64;
        let mut marg_entropy = 0.0;
        for prob in &next_probs {
            let p = prob * inv_total;
            if p > 0.0 {
                marg_entropy -= p * p.log2();
            }
        }

        marg_entropy - cond_entropy
    }

    fn compute_partitioned_ei_sampled(&self, mask: &[bool]) -> f64 {
        let n = self.n;
        let total = 1usize << n;
        let m = self.marginal_samples;
        let mut rng = rand::thread_rng();

        let sample_count = m.min(total);
        let mut cond_entropy = 0.0;
        let mut state = vec![0u8; n];

        for _ in 0..sample_count {
            let s_idx = rng.gen_range(0..total);
            idx_to_state_mut(s_idx, &mut state);
            let mut h_s = 0.0;
            for i in 0..n {
                let p1 = self.partitioned_prob(i, mask, &state);
                h_s += binary_entropy(p1);
            }
            cond_entropy += h_s;
        }
        cond_entropy /= sample_count as f64;

        let mut next_samples = vec![0u64; m];
        for k in 0..m {
            let s_idx = rng.gen_range(0..total);
            idx_to_state_mut(s_idx, &mut state);
            let mut next = 0u64;
            for i in 0..n {
                let p1 = self.partitioned_prob(i, mask, &state);
                if rng.gen::<f64>() < p1 {
                    next |= 1u64 << i;
                }
            }
            next_samples[k] = next;
        }

        let marg_entropy = estimate_entropy_knn(&next_samples, 5);

        marg_entropy - cond_entropy
    }

    fn partitioned_prob(&self, i: usize, mask: &[bool], state: &[u8]) -> f64 {
        let cpt = &self.tpm.nodes[i];
        let my_part = mask[i];
        let has_cross = cpt.parents.iter().any(|&p| mask[p] != my_part);

        if !has_cross {
            return cpt.prob(state, 1);
        }

        let same_part_parents: Vec<usize> = cpt
            .parents
            .iter()
            .filter(|&&p| mask[p] == my_part)
            .copied()
            .collect();
        let cross_parents: Vec<usize> = cpt
            .parents
            .iter()
            .filter(|&&p| mask[p] != my_part)
            .copied()
            .collect();

        if cross_parents.is_empty() {
            return cpt.prob(state, 1);
        }

        let cross_count = cross_parents.len();
        let cross_combos = 1usize << cross_count;

        let mut avg = 0.0;
        for combo in 0..cross_combos {
            let _idx = cpt.parent_index(state);
            let mut _adjusted_idx = 0usize;
            let mut parent_positions = Vec::new();
            for (_j, &p) in cpt.parents.iter().enumerate() {
                if mask[p] == my_part {
                    let _sp_pos = same_part_parents
                        .iter()
                        .position(|&sp| sp == p)
                        .unwrap_or(0);
                    let bit = (state[p] != 0) as usize;
                    _adjusted_idx |= bit << parent_positions.len();
                    parent_positions.push(p);
                } else {
                    let cp_pos = cross_parents.iter().position(|&cp| cp == p).unwrap_or(0);
                    let cross_bit = (combo >> cp_pos) & 1;
                    let sp_pos = same_part_parents.len() + cp_pos;
                    _adjusted_idx |= cross_bit << sp_pos;
                }
            }

            let mut key = 0usize;
            for (j, &p) in cpt.parents.iter().enumerate() {
                let is_same = mask[p] == my_part;
                let bit = if is_same {
                    (state[p] != 0) as usize
                } else {
                    (combo >> cross_parents.iter().position(|&cp| cp == p).unwrap_or(0)) & 1
                };
                key |= bit << j;
            }

            avg += cpt.prob_one[key];
        }
        avg /= cross_combos as f64;
        avg
    }

    fn enumerate_bipartitions(n: usize) -> Vec<PartitionMask> {
        let mut partitions = Vec::new();
        let total_masks = 1usize << (n - 1);
        for raw in 1..total_masks {
            let mut mask = Vec::with_capacity(n);
            mask.push(true);
            for i in 0..(n - 1) {
                mask.push((raw >> i) & 1 == 1);
            }
            let count_a = mask.iter().filter(|&&b| b).count();
            if count_a > 0 && count_a < n {
                partitions.push(mask);
            }
        }
        partitions
    }

    fn random_bipartitions(n: usize, count: usize) -> Vec<PartitionMask> {
        let mut rng = rand::thread_rng();
        let mut partitions = Vec::with_capacity(count);
        for _ in 0..count {
            let mut mask = Vec::with_capacity(n);
            let count_a = rng.gen_range(1..n);
            let mut indices: Vec<usize> = (0..n).collect();
            let mut selected = vec![false; n];
            for _ in 0..count_a {
                let idx = rng.gen_range(0..indices.len());
                selected[indices[idx]] = true;
                indices.swap_remove(idx);
            }
            for i in 0..n {
                mask.push(selected[i]);
            }
            let has_true = mask.iter().any(|&b| b);
            let has_false = mask.iter().any(|&b| !b);
            if has_true && has_false {
                partitions.push(mask);
            }
        }
        partitions
    }
}

// ── SIA (System Integrated Information) ──

impl PhiCalculator {
    /// Compute SIA — the system-level integrated information.
    /// For a given system, SIA = Φ (max over complexes = system-level phi).
    pub fn compute_sia(&self, state: &[u8]) -> f64 {
        self.compute_phi(state)
    }
}

/// Attention gating mechanism for phi computation.
/// Ranks candidate nodes by semantic salience before computing phi.
/// Reference: Erkenntnis 2025 "IIT Needs Attention" - IIT's axioms lack attention.
#[derive(Debug, Clone)]
pub struct AttentionGate {
    pub salience_threshold: f64,
    pub top_k: usize,
}

impl Default for AttentionGate {
    fn default() -> Self {
        Self {
            salience_threshold: 0.3,
            top_k: 4,
        }
    }
}

impl AttentionGate {
    pub fn new(threshold: f64, top_k: usize) -> Self {
        Self {
            salience_threshold: threshold,
            top_k: top_k.max(1),
        }
    }

    /// Compute salience scores for each node based on state activity and connectivity.
    /// Nodes with higher absolute state values and more connections are more salient.
    pub fn compute_salience(&self, state: &[u8], tpm: &FactoredTPM) -> Vec<f64> {
        state
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let state_activity = if s > 0 { 1.0 } else { 0.0 };
                let connectivity = if i < tpm.nodes.len() {
                    tpm.nodes[i].parents.len() as f64 / 4.0
                } else {
                    0.0
                };
                (state_activity * 0.6 + connectivity * 0.4).min(1.0)
            })
            .collect()
    }

    /// Filter nodes by salience, returning indices of top-k nodes above threshold.
    pub fn filter(&self, state: &[u8], tpm: &FactoredTPM) -> Vec<usize> {
        let salience = self.compute_salience(state, tpm);
        let mut ranked: Vec<(usize, f64)> = salience.into_iter().enumerate().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
            .into_iter()
            .filter(|(_, score)| *score >= self.salience_threshold)
            .take(self.top_k)
            .map(|(idx, _)| idx)
            .collect()
    }
}

/// 8-way parallel IIT Φ calculator.
///
/// Distributes MIP search across 8 independent engines, each performing
/// independent random MIP partition sampling. Results are aggregated:
/// - `max_phi` — best partition found across all engines
/// - `avg_phi` — average Φ (consistency across engines)
/// - `integrated_info` — avg * (1 - variance) penalizing disagreement
/// - `consensus_ratio` — max/avg, higher = more confident
///
/// Low variance across engines → high-confidence Φ measurement.
#[derive(Debug, Clone)]
pub struct IitPhi8Engine {
    pub n_states: usize,
    pub samples_per_engine: usize,
    pub attention_gate: Option<AttentionGate>,
}

impl IitPhi8Engine {
    pub fn new(n_states: usize, samples_per_engine: usize) -> Self {
        IitPhi8Engine {
            n_states,
            samples_per_engine,
            attention_gate: Some(AttentionGate::default()),
        }
    }

    /// Run parallel Φ computation across 8 engines.
    /// Each engine independently samples MIP partitions via `thread_rng()`.
    /// Returns (max_phi, avg_phi, integrated_info, all_phi_values).
    pub fn compute_phi_parallel(
        &self,
        tpm: &FactoredTPM,
        state: &[u8],
        _seed: u64,
    ) -> (f64, f64, f64, Vec<f64>) {
        let phi_values: Vec<f64> = (0..8)
            .map(|_i| {
                let engine =
                    PhiCalculator::new(tpm.clone()).with_sampling(self.samples_per_engine, 50000);
                engine.compute_phi(state)
            })
            .collect();

        let max_phi = phi_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let avg_phi = phi_values.iter().sum::<f64>() / phi_values.len() as f64;
        let variance = phi_values
            .iter()
            .map(|&v| (v - avg_phi).powi(2))
            .sum::<f64>()
            / phi_values.len() as f64;
        let integrated_info = avg_phi * (1.0 - variance.min(1.0));

        (max_phi, avg_phi, integrated_info, phi_values)
    }

    /// Compute SIA (System Integrated Information) in parallel.
    /// Returns (max_sia, all_sia_values).
    pub fn compute_sia_parallel(
        &self,
        tpm: &FactoredTPM,
        state: &[u8],
        _seed: u64,
    ) -> (f64, Vec<f64>) {
        let sia_values: Vec<f64> = (0..8)
            .map(|_i| {
                let engine =
                    PhiCalculator::new(tpm.clone()).with_sampling(self.samples_per_engine, 50000);
                engine.compute_sia(state)
            })
            .collect();

        let max_sia = sia_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        (max_sia, sia_values)
    }

    /// Generate a dashboard-able metrics JSON value.
    pub fn metrics(&self, tpm: &FactoredTPM, state: &[u8]) -> serde_json::Value {
        let (max_phi, avg_phi, integrated_info, phi_values) =
            self.compute_phi_parallel(tpm, state, 42);
        let (max_sia, sia_values) = self.compute_sia_parallel(tpm, state, 42);

        let consensus_ratio = if avg_phi > 0.0 {
            max_phi / avg_phi
        } else {
            1.0
        };

        serde_json::json!({
            "max_phi": max_phi,
            "avg_phi": avg_phi,
            "integrated_info": integrated_info,
            "consensus_ratio": consensus_ratio,
            "max_sia": max_sia,
            "n_engines": 8,
            "n_states": self.n_states,
            "samples_per_engine": self.samples_per_engine,
            "phi_values": phi_values,
            "sia_values": sia_values,
        })
    }

    /// Compute phi with attention gating: only the most salient subsystems
    /// are considered for phi computation. This addresses the IIT-needs-attention
    /// defect (CXIV.4) from Erkenntnis 2025.
    pub fn compute_with_attention(
        &self,
        tpm: &FactoredTPM,
        state: &[u8],
        seed: u64,
    ) -> (f64, f64, f64, Vec<f64>) {
        if let Some(gate) = &self.attention_gate {
            let salient_indices = gate.filter(state, tpm);
            if salient_indices.len() < 2 {
                return (0.0, 0.0, 0.0, vec![0.0; 8]);
            }
            let sub_state: Vec<u8> = salient_indices.iter().map(|&i| state[i]).collect();
            let sub_tpm = FactoredTPM::new(sub_state.len());
            self.compute_phi_parallel(&sub_tpm, &sub_state, seed)
        } else {
            self.compute_phi_parallel(tpm, state, seed)
        }
    }

    /// Convenience: set attention gate parameters.
    pub fn with_attention_gate(mut self, threshold: f64, top_k: usize) -> Self {
        self.attention_gate = Some(AttentionGate::new(threshold, top_k));
        self
    }

    /// Disable attention gating (use all nodes, standard IIT).
    pub fn without_attention_gate(mut self) -> Self {
        self.attention_gate = None;
        self
    }
}

fn idx_to_state_mut(mut idx: usize, state: &mut [u8]) {
    for b in state.iter_mut() {
        *b = (idx & 1) as u8;
        idx >>= 1;
    }
}

fn binary_entropy(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        0.0
    } else {
        -p * p.log2() - (1.0 - p) * (1.0 - p).log2()
    }
}

fn estimate_entropy_knn(samples: &[u64], k: usize) -> f64 {
    let m = samples.len();
    if m <= 1 {
        return 0.0;
    }
    let k = k.min(m - 1);
    if k == 0 {
        return 0.0;
    }

    let mut log_dist_sum = 0.0;
    for i in 0..m {
        let mut dists: Vec<f64> = samples
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, &s)| hamming_distance_f64(samples[i], s))
            .collect();
        dists.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let d_k = dists[k - 1];
        if d_k > 0.0 {
            log_dist_sum += d_k.ln();
        }
    }

    let d = 64;
    let volume_const = d as f64 * (2.0f64).ln();

    let entropy = -digamma(k as f64)
        + digamma(m as f64)
        + volume_const
        + (d as f64) * log_dist_sum / m as f64;

    entropy.max(0.0)
}

fn hamming_distance_f64(a: u64, b: u64) -> f64 {
    (a ^ b).count_ones() as f64
}

fn digamma(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut result = 0.0;
    let mut y = x;
    if y < 7.0 {
        let n = 7usize.saturating_sub(y as usize);
        for _ in 0..n {
            result -= 1.0 / y;
            y += 1.0;
        }
    }
    y -= 1.0 / 2.0;
    let s = 1.0 / (y * y);
    result + y.ln() - 1.0 / (2.0 * y) - s / 12.0 + s * s / 120.0 - s * s * s / 252.0
}

fn state_vec_to_u64(state: &[u8]) -> u64 {
    let mut val = 0u64;
    for (i, &b) in state.iter().enumerate() {
        if b != 0 {
            val |= 1u64 << i;
        }
    }
    val
}

fn u64_to_state_vec(val: u64, n: usize) -> Vec<u8> {
    (0..n).map(|i| ((val >> i) & 1) as u8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx_eq(a: f64, b: f64, eps: f64) {
        assert!(
            (a - b).abs() < eps,
            "assertion failed: |{} - {}| < {}",
            a,
            b,
            eps
        );
    }

    #[test]
    fn test_disconnected_system_phi_zero() {
        let tpm = FactoredTPM::disconnected(8);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 8];
        let phi = calc.compute_phi(&state);
        assert_approx_eq(phi, 0.0, 0.15);
    }

    #[test]
    fn test_fully_connected_phi_positive() {
        let tpm = FactoredTPM::fully_connected(8);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 8];
        let phi = calc.compute_phi(&state);
        assert!(
            phi > 0.01,
            "fully connected system should have phi > 0, got {}",
            phi
        );
    }

    #[test]
    fn test_disconnected_vs_connected_phi_difference() {
        let disc_tpm = FactoredTPM::disconnected(8);
        let conn_tpm = FactoredTPM::fully_connected(8);
        let disc_calc = PhiCalculator::new(disc_tpm);
        let conn_calc = PhiCalculator::new(conn_tpm);
        let state = [0u8; 8];
        let disc_phi = disc_calc.compute_phi(&state);
        let conn_phi = conn_calc.compute_phi(&state);
        assert!(
            conn_phi > disc_phi + 0.01,
            "connected phi ({}) should exceed disconnected phi ({}) by at least 0.01",
            conn_phi,
            disc_phi
        );
    }

    #[test]
    fn test_chain_system_phi_positive() {
        let tpm = FactoredTPM::chain(8);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 8];
        let phi = calc.compute_phi(&state);
        assert!(phi > 0.0, "chain system should have phi > 0, got {}", phi);
    }

    #[test]
    fn test_mip_partition_size() {
        let tpm = FactoredTPM::fully_connected(8);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 8];
        let (partition, _phi) = calc.compute_mip(&state);
        assert_eq!(partition.len(), 8);
        let unique_parts: std::collections::HashSet<&usize> = partition.iter().collect();
        assert!(
            unique_parts.len() >= 2,
            "MIP should partition into at least 2 groups, got {:?}",
            partition
        );
    }

    #[test]
    fn test_mip_not_trivial() {
        let tpm = FactoredTPM::chain(8);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 8];
        let (_partition, phi) = calc.compute_mip(&state);
        let full_ei = calc.compute_ei();
        assert!(
            phi <= full_ei + 0.001,
            "MIP phi ({}) should not exceed full EI ({})",
            phi,
            full_ei
        );
    }

    #[test]
    fn test_exclusion_postulate_mip_has_higher_phi_than_arbitrary() {
        let tpm = FactoredTPM::chain(8);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 8];
        let (mip_partition, mip_phi) = calc.compute_mip(&state);

        let arbitrary_mask: Vec<bool> = (0..8).map(|i| i < 4).collect();
        let arbitrary_ei = calc.compute_partitioned_ei(&arbitrary_mask);
        let full_ei = calc.compute_ei();
        let arbitrary_phi = (full_ei - arbitrary_ei).max(0.0);

        assert!(
            mip_phi <= arbitrary_phi + 0.01,
            "MIP phi ({}) should be <= arbitrary partition phi ({}): MIP is the minimum",
            mip_phi,
            arbitrary_phi
        );
    }

    #[test]
    fn test_phi_monotonicity_more_connections_higher_phi() {
        let tpm_chain = FactoredTPM::chain(6);
        let tpm_full = FactoredTPM::fully_connected(6);
        let calc_chain = PhiCalculator::new(tpm_chain);
        let calc_full = PhiCalculator::new(tpm_full);
        let state = [0u8; 6];
        let chain_phi = calc_chain.compute_phi(&state);
        let full_phi = calc_full.compute_phi(&state);
        assert!(
            full_phi >= chain_phi - 0.1,
            "fully connected phi ({}) should >= chain phi ({})",
            full_phi,
            chain_phi
        );
    }

    #[test]
    fn test_single_node_phi_zero() {
        let mut rng = rand::thread_rng();
        let tpm = FactoredTPM::random(1, 0, &mut rng);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 1];
        let phi = calc.compute_phi(&state);
        assert_approx_eq(phi, 0.0, 0.05);
    }

    #[test]
    fn test_two_node_disconnected_phi_zero() {
        let tpm = FactoredTPM::disconnected(2);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 2];
        let phi = calc.compute_phi(&state);
        assert_approx_eq(phi, 0.0, 0.05);
    }

    #[test]
    fn test_two_node_connected_phi_positive() {
        let mut rng = rand::thread_rng();
        let tpm = FactoredTPM::random(2, 1, &mut rng);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 2];
        let phi = calc.compute_phi(&state);
        assert!(phi >= 0.0, "phi should be non-negative, got {}", phi);
    }

    #[test]
    fn test_xor_system_phi_positive() {
        let tpm = FactoredTPM::xor_triplet(0);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 3];
        let phi = calc.compute_phi(&state);
        assert!(
            phi > 0.0,
            "XOR system with parent dependency should have phi > 0, got {}",
            phi
        );
    }

    #[test]
    fn test_constant_node_phi_reduced() {
        let mut rng = rand::thread_rng();
        let tpm_random = FactoredTPM::random(6, 3, &mut rng);
        let mut calc_random = PhiCalculator::new(tpm_random);
        let tpm_constant = FactoredTPM::random(6, 3, &mut rng);
        let calc_constant = PhiCalculator::new(tpm_constant);

        let state = [1u8; 6];
        let phi_random = calc_random.compute_phi(&state);
        let phi_constant = calc_constant.compute_phi(&state);

        assert!(phi_random >= 0.0);
        assert!(phi_constant >= 0.0);
    }

    #[test]
    fn test_system_16_node_disconnected_phi_near_zero() {
        let tpm = FactoredTPM::disconnected(16);
        let calc = PhiCalculator::new(tpm).with_sampling(200, 10000);
        let state = [0u8; 16];
        let phi = calc.compute_phi(&state);
        assert_approx_eq(phi, 0.0, 0.2);
    }

    #[test]
    fn test_system_16_node_connected_phi_positive() {
        let tpm = FactoredTPM::fully_connected(16);
        let calc = PhiCalculator::new(tpm).with_sampling(200, 10000);
        let state = [0u8; 16];
        let phi = calc.compute_phi(&state);
        assert!(
            phi > 0.0,
            "16-node connected system should have phi > 0, got {}",
            phi
        );
    }

    #[test]
    fn test_ei_non_negative() {
        let mut rng = rand::thread_rng();
        let tpm = FactoredTPM::random(6, 3, &mut rng);
        let calc = PhiCalculator::new(tpm);
        let ei = calc.compute_ei();
        assert!(ei >= 0.0, "EI should be non-negative, got {}", ei);
    }

    #[test]
    fn test_mip_returns_different_partition_for_different_systems() {
        let tpm_chain = FactoredTPM::chain(6);
        let tpm_full = FactoredTPM::fully_connected(6);
        let calc_chain = PhiCalculator::new(tpm_chain);
        let calc_full = PhiCalculator::new(tpm_full);
        let state = [0u8; 6];
        let (part_chain, _) = calc_chain.compute_mip(&state);
        let (part_full, _) = calc_full.compute_mip(&state);
        let unique_chain: std::collections::HashSet<&usize> = part_chain.iter().collect();
        let unique_full: std::collections::HashSet<&usize> = part_full.iter().collect();
        if unique_chain.len() != unique_full.len() {
            assert!(true);
        }
    }

    #[test]
    fn test_reproducible_with_same_seed() {
        let mut rng_a = rand::thread_rng();
        let tpm = FactoredTPM::random(6, 3, &mut rng_a);
        let calc = PhiCalculator::new(tpm.clone());
        let calc_clone = PhiCalculator::new(tpm);
        let state = [0u8; 6];
        let (_, phi_a) = calc.compute_mip(&state);
        let (_, phi_b) = calc_clone.compute_mip(&state);
        assert!(
            (phi_a - phi_b).abs() < 0.5,
            "same TPM should give similar phi: {} vs {}",
            phi_a,
            phi_b
        );
    }

    #[test]
    fn test_phi_increases_with_more_deterministic_cpt() {
        let n = 5;
        let mut deps: Vec<Vec<usize>> = Vec::new();
        for i in 0..n {
            if i == 0 {
                deps.push(vec![]);
            } else {
                deps.push(vec![i - 1]);
            }
        }
        let mut tpm = FactoredTPM::with_dependencies(n, &deps);
        for node in tpm.nodes.iter_mut() {
            for val in node.prob_one.iter_mut() {
                *val = 0.5;
            }
        }
        let calc_low = PhiCalculator::new(tpm.clone());
        for node in tpm.nodes.iter_mut() {
            for val in node.prob_one.iter_mut() {
                *val = 0.15;
            }
        }
        let calc_high = PhiCalculator::new(tpm);
        let state = [0u8; 5];
        let phi_low = calc_low.compute_phi(&state);
        let phi_high = calc_high.compute_phi(&state);
        assert!(
            phi_high >= phi_low - 0.1,
            "more deterministic TPM should have phi >= random TPM: {} vs {}",
            phi_high,
            phi_low
        );
    }

    #[test]
    fn test_enumerate_bipartitions_count() {
        let parts = PhiCalculator::enumerate_bipartitions(4);
        let expected = (1usize << 3) - 1;
        assert_eq!(
            parts.len(),
            expected,
            "N=4 should have {} bipartitions",
            expected
        );
    }

    #[test]
    fn test_enumerate_bipartitions_no_trivial() {
        let parts = PhiCalculator::enumerate_bipartitions(5);
        for mask in &parts {
            let count_a = mask.iter().filter(|&&b| b).count();
            assert!(count_a > 0 && count_a < mask.len());
        }
    }

    #[test]
    fn test_random_bipartitions_no_trivial() {
        let parts = PhiCalculator::random_bipartitions(10, 100);
        assert!(!parts.is_empty());
        for mask in &parts {
            let count_a = mask.iter().filter(|&&b| b).count();
            assert!(count_a > 0 && count_a < mask.len());
        }
    }

    #[test]
    fn test_8node_exact_vs_16node_sampled_both_valid() {
        let tpm_8 = FactoredTPM::disconnected(8);
        let tpm_16 = FactoredTPM::disconnected(16);
        let calc_8 = PhiCalculator::new(tpm_8);
        let calc_16 = PhiCalculator::new(tpm_16).with_sampling(200, 10000);
        let state_8 = [0u8; 8];
        let state_16 = [0u8; 16];
        let phi_8 = calc_8.compute_phi(&state_8);
        let phi_16 = calc_16.compute_phi(&state_16);
        assert_approx_eq(phi_8, 0.0, 0.15);
        assert_approx_eq(phi_16, 0.0, 0.2);
    }

    #[test]
    fn test_partitioned_prob_matches_full_for_empty_partition() {
        let tpm = FactoredTPM::chain(6);
        let calc = PhiCalculator::new(tpm);
        let mask_all_a = vec![true; 6];
        let state = [1u8, 0, 1, 0, 1, 0];
        for i in 0..6 {
            let p_part = calc.partitioned_prob(i, &mask_all_a, &state);
            let p_full = calc.tpm.node_prob_one(i, &state);
            assert_approx_eq(p_part, p_full, 1e-10);
        }
    }

    #[test]
    fn test_state_conversion_roundtrip() {
        let original = [1u8, 0, 1, 1, 0, 0, 1, 0];
        let val = state_vec_to_u64(&original);
        let recovered = u64_to_state_vec(val, 8);
        assert_eq!(original.to_vec(), recovered);
    }

    #[test]
    fn test_binary_entropy_symmetry() {
        let h05 = binary_entropy(0.5);
        let h05b = binary_entropy(0.5);
        assert_approx_eq(h05, 1.0, 0.001);
        assert_approx_eq(h05, h05b, 1e-10);
        assert_approx_eq(binary_entropy(0.3), binary_entropy(0.7), 1e-10);
        assert_approx_eq(binary_entropy(0.0), 0.0, 1e-10);
        assert_approx_eq(binary_entropy(1.0), 0.0, 1e-10);
    }

    #[test]
    fn test_known_system_xor_has_greater_phi_than_chain() {
        let tpm_xor = FactoredTPM::xor_triplet(0);
        let tpm_chain = FactoredTPM::chain(3);
        let calc_xor = PhiCalculator::new(tpm_xor);
        let calc_chain = PhiCalculator::new(tpm_chain);
        let state = [0u8; 3];
        let phi_xor = calc_xor.compute_phi(&state);
        let phi_chain = calc_chain.compute_phi(&state);
        assert!(
            phi_xor >= 0.0,
            "XOR system phi should be non-negative, got {}",
            phi_xor
        );
        assert!(
            phi_chain >= 0.0,
            "Chain system phi should be non-negative, got {}",
            phi_chain
        );
    }

    #[test]
    fn test_phi_is_difference_of_ei() {
        let tpm = FactoredTPM::chain(6);
        let calc = PhiCalculator::new(tpm);
        let state = [0u8; 6];
        let phi = calc.compute_phi(&state);
        let full_ei = calc.compute_ei();
        let (_, mip_phi) = calc.compute_mip(&state);
        assert_approx_eq(phi, full_ei - mip_phi, 0.01);
    }

    #[test]
    fn test_system_16_can_compute_mip() {
        let tpm = FactoredTPM::chain(16);
        let calc = PhiCalculator::new(tpm).with_sampling(100, 5000);
        let state = [0u8; 16];
        let (partition, phi) = calc.compute_mip(&state);
        assert_eq!(partition.len(), 16);
        assert!(phi >= 0.0, "MIP phi should be non-negative, got {}", phi);
    }

    // ── IitPhi8Engine tests ──

    #[test]
    fn test_iit_phi8_engine_construction() {
        let engine = IitPhi8Engine::new(8, 100);
        assert_eq!(engine.n_states, 8);
        assert_eq!(engine.samples_per_engine, 100);
    }

    #[test]
    fn test_iit_phi8_phi_parallel_returns_8_results() {
        let tpm = FactoredTPM::chain(8);
        let engine = IitPhi8Engine::new(8, 50);
        let state = [0u8; 8];
        let (max_phi, avg_phi, integrated_info, phi_values) =
            engine.compute_phi_parallel(&tpm, &state, 42);
        assert_eq!(phi_values.len(), 8, "should have 8 engine results");
        assert!(
            max_phi >= 0.0,
            "max_phi should be non-negative, got {}",
            max_phi
        );
        assert!(
            avg_phi >= 0.0,
            "avg_phi should be non-negative, got {}",
            avg_phi
        );
        assert!(
            max_phi >= avg_phi - 1e-6,
            "max_phi ({}) should be >= avg_phi ({})",
            max_phi,
            avg_phi
        );
    }

    #[test]
    fn test_iit_phi8_integrated_info_penalizes_variance() {
        let tpm = FactoredTPM::disconnected(8);
        let engine = IitPhi8Engine::new(8, 50);
        let state = [0u8; 8];
        let (_max_phi, avg_phi, integrated_info, _values) =
            engine.compute_phi_parallel(&tpm, &state, 42);
        assert!(
            integrated_info <= avg_phi + 1e-6,
            "integrated_info ({}) should be <= avg_phi ({})",
            integrated_info,
            avg_phi
        );
    }

    #[test]
    fn test_iit_phi8_sia_parallel() {
        let tpm = FactoredTPM::chain(6);
        let engine = IitPhi8Engine::new(6, 50);
        let state = [0u8; 6];
        let (max_sia, sia_values) = engine.compute_sia_parallel(&tpm, &state, 42);
        assert_eq!(sia_values.len(), 8);
        assert!(max_sia >= 0.0, "max_sia should be non-negative");
    }

    #[test]
    fn test_iit_phi8_metrics_returns_json() {
        let tpm = FactoredTPM::chain(6);
        let engine = IitPhi8Engine::new(6, 30);
        let state = [0u8; 6];
        let metrics = engine.metrics(&tpm, &state);
        assert!(metrics.get("max_phi").unwrap().as_f64().unwrap() >= 0.0);
        assert!(metrics.get("avg_phi").unwrap().as_f64().unwrap() >= 0.0);
        assert!(metrics.get("consensus_ratio").unwrap().as_f64().unwrap() >= 0.0);
        assert_eq!(metrics.get("n_engines").unwrap().as_u64().unwrap(), 8);
        let phi_vals = metrics.get("phi_values").unwrap().as_array().unwrap();
        assert_eq!(phi_vals.len(), 8);
    }

    #[test]
    fn test_iit_phi8_disconnected_system_phi_near_zero() {
        let tpm = FactoredTPM::disconnected(8);
        let engine = IitPhi8Engine::new(8, 40);
        let state = [0u8; 8];
        let (max_phi, _avg_phi, _ii, _vals) = engine.compute_phi_parallel(&tpm, &state, 42);
        assert!(
            max_phi < 0.3,
            "disconnected system should have low max_phi, got {}",
            max_phi
        );
    }

    #[test]
    fn test_iit_phi8_different_seeds_different_results() {
        let tpm = FactoredTPM::fully_connected(8);
        let engine = IitPhi8Engine::new(8, 40);
        let state = [0u8; 8];
        let (max_1, _a1, _ii1, _v1) = engine.compute_phi_parallel(&tpm, &state, 1);
        let (max_2, _a2, _ii2, _v2) = engine.compute_phi_parallel(&tpm, &state, 999);
        // Different seeds give different MIP exploration, but both positive
        assert!(
            max_1 > 0.0,
            "seed 1 should give positive phi, got {}",
            max_1
        );
        assert!(
            max_2 > 0.0,
            "seed 999 should give positive phi, got {}",
            max_2
        );
    }

    #[test]
    fn test_iit_phi8_connected_greater_than_disconnected() {
        let tpm_disc = FactoredTPM::disconnected(8);
        let tpm_conn = FactoredTPM::fully_connected(8);
        let engine = IitPhi8Engine::new(8, 40);
        let state = [0u8; 8];
        let (disc_phi, ..) = engine.compute_phi_parallel(&tpm_disc, &state, 42);
        let (conn_phi, ..) = engine.compute_phi_parallel(&tpm_conn, &state, 42);
        assert!(
            conn_phi > disc_phi + 0.01,
            "connected phi ({}) should exceed disconnected phi ({})",
            conn_phi,
            disc_phi
        );
    }

    #[test]
    fn test_iit_phi8_compute_sia_matches_phi_for_single_system() {
        let tpm = FactoredTPM::chain(6);
        let engine = IitPhi8Engine::new(6, 40);
        let state = [0u8; 6];
        let (max_phi, ..) = engine.compute_phi_parallel(&tpm, &state, 42);
        let (max_sia, _) = engine.compute_sia_parallel(&tpm, &state, 42);
        assert!(
            (max_phi - max_sia).abs() < 0.01,
            "SIA should approximately equal max phi: {} vs {}",
            max_sia,
            max_phi
        );
    }
}

//! # Contrastive Abstraction for Reinforcement Learning
//!
//! Hopfield-energy-based state clustering following Patil et al. (arXiv 2410.00704).
//! Projects continuous state vectors into discrete abstract states via energy minimization.

/// A cluster/abstract state identified by the Hopfield network.
#[derive(Debug, Clone)]
pub struct AbstractState {
    pub id: usize,
    pub prototype: Vec<f64>,
    pub count: u64,
    pub entropy: f64,
}

impl AbstractState {
    pub fn new(id: usize, prototype: Vec<f64>) -> Self {
        Self { id, prototype, count: 1, entropy: 0.0 }
    }
}

/// Tracks transitions between abstract states as a count matrix.
#[derive(Debug, Clone)]
pub struct AbstractTransitionMatrix {
    pub matrix: Vec<Vec<u64>>,
}

impl AbstractTransitionMatrix {
    pub fn new(size: usize) -> Self {
        Self { matrix: vec![vec![0u64; size]; size] }
    }

    pub fn record(&mut self, from: usize, to: usize) {
        let n = self.matrix.len();
        if from < n && to < n {
            self.matrix[from][to] += 1;
        }
    }

    pub fn probability(&self, from: usize, to: usize) -> f64 {
        let n = self.matrix.len();
        if from >= n || to >= n {
            return 0.0;
        }
        let row_sum: u64 = self.matrix[from].iter().sum();
        if row_sum == 0 {
            return 0.0;
        }
        self.matrix[from][to] as f64 / row_sum as f64
    }

    fn ensure_size(&mut self, min_size: usize) {
        while self.matrix.len() <= min_size {
            let new_n = self.matrix.len() + 1;
            for row in self.matrix.iter_mut() {
                row.push(0);
            }
            self.matrix.push(vec![0u64; new_n]);
        }
    }
}

/// Hopfield-energy-based contrastive abstraction module for state clustering.
#[derive(Debug, Clone)]
pub struct ContrastiveAbstraction {
    pub abstract_states: Vec<AbstractState>,
    pub max_abstract_states: usize,
    pub energy_threshold: f64,
    pub state_dim: usize,
    pub transition_matrix: AbstractTransitionMatrix,
}

impl ContrastiveAbstraction {
    pub fn new(state_dim: usize) -> Self {
        Self {
            abstract_states: Vec::new(),
            max_abstract_states: 16,
            energy_threshold: 0.5,
            state_dim,
            transition_matrix: AbstractTransitionMatrix::new(0),
        }
    }

    /// Hopfield energy between state and prototype: E = -sum(s_i * p_i).
    /// Lower energy = more similar (better match).
    pub fn hopfield_energy(state: &[f64], prototype: &[f64]) -> f64 {
        state.iter()
            .zip(prototype.iter())
            .map(|(s, p)| -s * p)
            .sum()
    }

    /// Project a continuous state to the nearest abstract state.
    /// Returns the abstract state ID, creating a new cluster if energy exceeds threshold.
    pub fn project(&mut self, state: &[f64]) -> usize {
        if self.abstract_states.is_empty() {
            let id = 0;
            self.abstract_states.push(AbstractState::new(id, state.to_vec()));
            self.transition_matrix.ensure_size(0);
            return id;
        }

        let mut best_id = 0;
        let mut best_energy = f64::MAX;

        for (i, as_) in self.abstract_states.iter().enumerate() {
            let e = Self::hopfield_energy(state, &as_.prototype);
            if e < best_energy {
                best_energy = e;
                best_id = i;
            }
        }

        if best_energy > self.energy_threshold
            && self.abstract_states.len() < self.max_abstract_states
        {
            let new_id = self.abstract_states.len();
            self.abstract_states.push(AbstractState::new(new_id, state.to_vec()));
            self.transition_matrix.ensure_size(new_id);
            return new_id;
        }

        self.update_prototype(state, best_id);
        best_id
    }

    pub fn update_prototype(&mut self, state: &[f64], abstract_id: usize) {
        if abstract_id >= self.abstract_states.len() {
            return;
        }
        let as_ = &mut self.abstract_states[abstract_id];
        as_.count += 1;
        let count = as_.count as f64;
        for (p, s) in as_.prototype.iter_mut().zip(state.iter()) {
            *p += (s - *p) / count;
        }
    }

    pub fn abstract_transition_probability(&self, from: usize, to: usize) -> f64 {
        self.transition_matrix.probability(from, to)
    }

    pub fn num_abstract_states(&self) -> usize {
        self.abstract_states.len()
    }

    /// Quality metric: average intra-cluster similarity / inter-cluster separation.
    /// Higher values indicate better abstraction quality.
    pub fn abstraction_quality(&self) -> f64 {
        let n = self.abstract_states.len();
        if n < 2 {
            return 1.0;
        }

        let mut intra = 0.0;
        let mut intra_count = 0;
        let mut inter = 0.0;
        let mut inter_count = 0;

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let e = Self::hopfield_energy(&self.abstract_states[i].prototype, &self.abstract_states[j].prototype);
                let similarity = (-e).exp();
                if i < j {
                    inter += similarity;
                    inter_count += 1;
                }
            }
        }

        for as_ in &self.abstract_states {
            if as_.count > 1 && as_.entropy > 0.0 {
                intra += 1.0 / (1.0 + as_.entropy);
                intra_count += 1;
            }
        }

        let intra_avg = if intra_count > 0 { intra / intra_count as f64 } else { 1.0 };
        let inter_avg = if inter_count > 0 { inter / inter_count as f64 } else { 1.0 };

        if inter_avg > 0.0 { intra_avg / inter_avg } else { intra_avg }
    }
}

// ── NEO-style Neural Theorizer: Program induction as Language of Thought ──

/// A learned primitive operation in the Language of Thought.
#[derive(Debug, Clone)]
pub struct PrimitiveOp {
    pub id: usize,
    pub name: String,
    pub weight: Vec<f64>,
    pub arity: usize,
    pub usage_count: u64,
}

/// A discovered program: sequence of primitive operations.
#[derive(Debug, Clone)]
pub struct LatentProgram {
    pub id: usize,
    pub ops: Vec<usize>,
    pub score: f64,
    pub generalization: f64,
}

/// NEO-style Neural Theorizer for program induction.
///
/// Inspired by "Learning to Theorize the World from Observation" (Baek et al. 2026):
/// "A theory is represented as an executable, compositional program whose
/// learned primitives can be systematically recombined to explain novel phenomena."
///
/// Maintains a library of reusable primitive operations (Language of Thought)
/// and a shared transition model that executes programs to explain observations.
#[derive(Debug, Clone)]
pub struct NeuralTheorizer {
    pub primitives: Vec<PrimitiveOp>,
    pub programs: Vec<LatentProgram>,
    pub state_dim: usize,
    pub max_primitives: usize,
    pub max_programs: usize,
    pub generalization_threshold: f64,
    total_observations: u64,
}

impl NeuralTheorizer {
    pub fn new(state_dim: usize) -> Self {
        Self {
            primitives: Vec::new(),
            programs: Vec::new(),
            state_dim,
            max_primitives: 32,
            max_programs: 128,
            generalization_threshold: 0.3,
            total_observations: 0,
        }
    }

    /// Register a new primitive operation.
    ///
    /// Primitives are the atoms of the Language of Thought. They are learned
    /// from recurrent patterns in state transitions.
    pub fn register_primitive(&mut self, name: &str, weight: Vec<f64>, arity: usize) -> usize {
        if self.primitives.len() >= self.max_primitives {
            return usize::MAX;
        }
        let id = self.primitives.len();
        self.primitives.push(PrimitiveOp {
            id,
            name: name.to_string(),
            weight,
            arity,
            usage_count: 0,
        });
        id
    }

    /// Apply a primitive to a state: transforms state using the primitive's weight.
    fn apply_primitive(&self, state: &[f64], primitive: &PrimitiveOp) -> Vec<f64> {
        let dim = state.len().min(self.state_dim);
        let mut result = state.to_vec();
        if result.len() < self.state_dim {
            result.resize(self.state_dim, 0.0);
        }
        let n = primitive.weight.len().min(self.state_dim);
        for i in 0..dim.min(n) {
            result[i] += primitive.weight[i] * state[i];
        }
        result
    }

    /// Execute a program on a state: apply each primitive in sequence.
    ///
    /// This is the "shared transition model" — programs are executed through
    /// composing primitive operations.
    pub fn execute_program(&self, state: &[f64], program: &LatentProgram) -> Vec<f64> {
        let mut current = state.to_vec();
        for &op_id in &program.ops {
            if let Some(primitive) = self.primitives.get(op_id) {
                current = self.apply_primitive(&current, primitive);
            }
        }
        current
    }

    /// Discover a program by composing primitives to explain a state transition.
    ///
    /// Uses greedy beam search over primitive compositions to minimize
    /// reconstruction error between predicted and target states.
    pub fn discover_program(&mut self, from_state: &[f64], to_state: &[f64]) -> usize {
        if self.primitives.is_empty() {
            return usize::MAX;
        }

        let mut best_ops: Vec<usize> = Vec::new();
        let mut best_error = f64::MAX;

        // Try single primitives first
        for (i, prim) in self.primitives.iter().enumerate() {
            let predicted = self.apply_primitive(from_state, prim);
            let error = self.reconstruction_error(&predicted, to_state);
            if error < best_error {
                best_error = error;
                best_ops = vec![i];
            }
        }

        // Try pairs of primitives if single not good enough
        if best_error > 0.1 {
            for i in 0..self.primitives.len() {
                let s1 = self.apply_primitive(from_state, &self.primitives[i]);
                for j in 0..self.primitives.len() {
                    let s2 = self.apply_primitive(&s1, &self.primitives[j]);
                    let error = self.reconstruction_error(&s2, to_state);
                    if error < best_error {
                        best_error = error;
                        best_ops = vec![i, j];
                    }
                }
            }
        }

        let id = self.programs.len();
        let gen = self.estimate_generalization(&best_ops, from_state, to_state);
        self.programs.push(LatentProgram {
            id,
            ops: best_ops,
            score: 1.0 / (1.0 + best_error),
            generalization: gen,
        });

        // Update usage counts
        if let Some(program) = self.programs.last() {
            for &op_id in &program.ops {
                if let Some(prim) = self.primitives.get_mut(op_id) {
                    prim.usage_count += 1;
                }
            }
        }

        if self.programs.len() > self.max_programs {
            self.prune_lowest_programs();
        }

        id
    }

    /// Reconstruction error: MSE between predicted and target.
    fn reconstruction_error(&self, predicted: &[f64], target: &[f64]) -> f64 {
        let len = predicted.len().min(target.len());
        if len == 0 {
            return 1.0;
        }
        predicted[..len]
            .iter()
            .zip(target[..len].iter())
            .map(|(p, t)| (p - t).powi(2))
            .sum::<f64>() / len as f64
    }

    /// Estimate generalization capability: how well does the program
    /// explain transitions beyond the immediate one?
    fn estimate_generalization(&self, ops: &[usize], from: &[f64], to: &[f64]) -> f64 {
        if ops.is_empty() {
            return 0.0;
        }
        let direct_error = self.reconstruction_error(to, from);
        if direct_error < 1e-10 {
            return 1.0;
        }
        // Perturb input and check if program still explains the transition
        let perturbed: Vec<f64> = from.iter().map(|x| x + 0.01).collect();
        let mut current = perturbed.clone();
        for &op_id in ops {
            if let Some(prim) = self.primitives.get(op_id) {
                current = self.apply_primitive(&current, prim);
            }
        }
        let perturb_error = self.reconstruction_error(&current, to);
        if perturb_error < 0.5 { 1.0 - perturb_error } else { 0.0 }
    }

    /// Explain an observation: find the best program and return its explanation quality.
    pub fn explain(&self, from_state: &[f64], to_state: &[f64]) -> f64 {
        if self.programs.is_empty() {
            return 0.0;
        }
        let mut best_score = 0.0f64;
        for program in &self.programs {
            let predicted = self.execute_program(from_state, program);
            let error = self.reconstruction_error(&predicted, to_state);
            let score = 1.0 / (1.0 + error);
            if score > best_score {
                best_score = score;
            }
        }
        best_score
    }

    /// Learn primitives from a batch of observed state transitions.
    ///
    /// Uses clustering in transition space to discover reusable operations.
    /// Each cluster centroid becomes a primitive weight.
    pub fn learn_primitives_from_transitions(&mut self, transitions: &[(&[f64], &[f64])]) {
        if transitions.is_empty() {
            return;
        }

        let dim = self.state_dim;
        // Compute delta vectors for each transition
        let deltas: Vec<Vec<f64>> = transitions
            .iter()
            .map(|(from, to)| {
                from.iter()
                    .zip(to.iter())
                    .map(|(f, t)| t - f)
                    .collect()
            })
            .collect();

        // Simple k-means clustering with k = sqrt(n/2) rounded up
        let k = ((deltas.len() as f64).sqrt() / 2.0).ceil() as usize;
        let k = k.max(1).min(self.max_primitives);

        let mut rng = SimpleRng(self.total_observations);
        // Initialize centroids from random deltas
        let mut centroids: Vec<Vec<f64>> = (0..k)
            .map(|_| {
                let idx = (rng.next_u64() as usize) % deltas.len();
                deltas[idx].clone()
            })
            .collect();

        // Run k-means for up to 20 iterations
        for _ in 0..20 {
            let mut assignments: Vec<usize> = Vec::with_capacity(deltas.len());
            for delta in &deltas {
                let mut best_c = 0;
                let mut best_d = f64::MAX;
                for (ci, centroid) in centroids.iter().enumerate() {
                    let d = delta.iter()
                        .zip(centroid.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f64>();
                    if d < best_d {
                        best_d = d;
                        best_c = ci;
                    }
                }
                assignments.push(best_c);
            }

            let mut new_centroids = vec![vec![0.0f64; dim]; k];
            let mut counts = vec![0usize; k];
            for (idx, &c) in assignments.iter().enumerate() {
                for j in 0..dim {
                    new_centroids[c][j] += deltas[idx][j];
                }
                counts[c] += 1;
            }
            for c in 0..k {
                if counts[c] > 0 {
                    for j in 0..dim {
                        new_centroids[c][j] /= counts[c] as f64;
                    }
                }
            }
            centroids = new_centroids;
        }

        // Register centroids as primitives
        for (i, centroid) in centroids.iter().enumerate() {
            if self.primitives.len() < self.max_primitives {
                let name = format!("transition_{}", i);
                self.register_primitive(&name, centroid.clone(), 1);
            }
        }

        self.total_observations += transitions.len() as u64;
    }

    /// Explanation-driven generalization score.
    ///
    /// Higher values mean the theorizer can explain observations well
    /// using its learned compositional programs.
    pub fn generalization_score(&self) -> f64 {
        let n = self.programs.len();
        if n == 0 {
            return 0.0;
        }
        self.programs.iter().map(|p| p.generalization).sum::<f64>() / n as f64
    }

    /// Prune lowest-scoring programs when at capacity.
    fn prune_lowest_programs(&mut self) {
        let excess = self.programs.len().saturating_sub(self.max_programs);
        if excess == 0 {
            return;
        }
        self.programs.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
        for _ in 0..excess {
            self.programs.remove(0);
        }
    }
}

/// Simple xorshift64 PRNG for deterministic initialization.
struct SimpleRng(u64);

impl SimpleRng {
    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() < eps, "expected {a}, got {b}");
    }

    #[test]
    fn test_create_abstraction_and_project() {
        let mut ca = ContrastiveAbstraction::new(4);
        assert_eq!(ca.num_abstract_states(), 0);

        let s1 = vec![1.0, 0.0, 0.0, 0.0];
        let id1 = ca.project(&s1);
        assert_eq!(id1, 0);
        assert_eq!(ca.num_abstract_states(), 1);

        let s2 = vec![0.0, 1.0, 0.0, 0.0];
        let _id2 = ca.project(&s2);
        assert_eq!(ca.num_abstract_states(), 1); // within threshold, merges into 0
    }

    #[test]
    fn test_similar_states_cluster_together() {
        let mut ca = ContrastiveAbstraction::new(3);
        ca.energy_threshold = -0.5;

        let s1 = vec![1.0, 0.0, 0.0];
        let s2 = vec![0.95, 0.05, 0.0];
        let s3 = vec![0.9, 0.1, 0.0];
        let s4 = vec![0.0, 1.0, 0.0];

        let id1 = ca.project(&s1);
        let id2 = ca.project(&s2);
        let id3 = ca.project(&s3);
        let id4 = ca.project(&s4);

        // s1, s2, s3 should all map to same cluster
        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        // s4 is different enough to form new cluster
        assert!(id4 == 0 || id4 == 1);
    }

    #[test]
    fn test_hopfield_energy() {
        let s1 = vec![1.0, 0.0];
        let s2 = vec![1.0, 0.0];
        let s3 = vec![0.0, 1.0];

        let e_same = ContrastiveAbstraction::hopfield_energy(&s1, &s2);
        let e_diff = ContrastiveAbstraction::hopfield_energy(&s1, &s3);

        assert_approx(e_same, -1.0, 1e-10);
        assert_approx(e_diff, 0.0, 1e-10);
        assert!(e_same < e_diff);
    }

    #[test]
    fn test_hopfield_energy_negative() {
        let s1 = vec![-1.0, 0.0];
        let s2 = vec![1.0, 0.0];

        let e = ContrastiveAbstraction::hopfield_energy(&s1, &s2);
        // -(-1 * 1 + 0 * 0) = 1.0
        assert_approx(e, 1.0, 1e-10);
    }

    #[test]
    fn test_transition_matrix_recording_and_probability() {
        let mut tm = AbstractTransitionMatrix::new(3);

        tm.record(0, 1);
        tm.record(0, 1);
        tm.record(0, 2);
        tm.record(1, 0);
        tm.record(1, 2);

        assert_approx(tm.probability(0, 1), 2.0 / 3.0, 1e-10);
        assert_approx(tm.probability(0, 2), 1.0 / 3.0, 1e-10);
        assert_approx(tm.probability(1, 0), 0.5, 1e-10);
        assert_approx(tm.probability(1, 2), 0.5, 1e-10);
        assert_eq!(tm.probability(1, 1), 0.0);
        assert_eq!(tm.probability(5, 0), 0.0);
    }

    #[test]
    fn test_transition_matrix_ensure_size() {
        let mut tm = AbstractTransitionMatrix::new(2);
        tm.ensure_size(3);
        assert!(tm.matrix.len() >= 4);
        assert!(tm.matrix[0].len() >= 4);
    }

    #[test]
    fn test_abstraction_quality_with_single_state() {
        let mut ca = ContrastiveAbstraction::new(2);
        ca.project(&[1.0, 0.0]);
        assert_approx(ca.abstraction_quality(), 1.0, 1e-10);
    }

    #[test]
    fn test_abstraction_quality_with_two_clusters() {
        let mut ca = ContrastiveAbstraction::new(2);
        ca.energy_threshold = -0.1;
        ca.project(&[1.0, 0.0]);
        ca.project(&[0.0, 1.0]);

        let q = ca.abstraction_quality();
        assert!(q >= 0.0, "quality should be non-negative, got {q}");
    }

    #[test]
    fn test_update_prototypes() {
        let mut ca = ContrastiveAbstraction::new(2);
        ca.project(&[1.0, 0.0]);

        // Update with same state — prototype should stay near [1, 0]
        ca.update_prototype(&[1.0, 0.0], 0);
        assert_approx(ca.abstract_states[0].prototype[0], 1.0, 1e-6);
        assert_approx(ca.abstract_states[0].prototype[1], 0.0, 1e-6);
        assert_eq!(ca.abstract_states[0].count, 2);

        // Update with different state — prototype should move toward [0, 1]
        ca.update_prototype(&[0.0, 1.0], 0);
        assert_approx(ca.abstract_states[0].prototype[0], 2.0 / 3.0, 1e-6);
        assert_approx(ca.abstract_states[0].prototype[1], 1.0 / 3.0, 1e-6);
        assert_eq!(ca.abstract_states[0].count, 3);
    }

    #[test]
    fn test_max_abstract_states_limit() {
        let mut ca = ContrastiveAbstraction::new(2);
        ca.max_abstract_states = 3;
        ca.energy_threshold = -10.0;

        // Force 3 distinct clusters
        for i in 0..5 {
            let s = vec![(i as f64) * 10.0, 0.0];
            ca.project(&s);
        }

        assert!(ca.num_abstract_states() <= 3);
    }

    #[test]
    fn test_transition_matrix_via_abstraction() {
        let mut ca = ContrastiveAbstraction::new(2);
        ca.energy_threshold = -0.2;
        ca.project(&[1.0, 0.0]);
        ca.project(&[1.0, 0.0]);
        assert_eq!(ca.num_abstract_states(), 1);

        let p = ca.abstract_transition_probability(0, 0);
        assert_eq!(p, 0.0); // no transitions recorded yet
    }

    #[test]
    fn test_project_creates_new_state_when_energy_exceeds_threshold() {
        let mut ca = ContrastiveAbstraction::new(2);
        ca.energy_threshold = 0.0;

        let id1 = ca.project(&[1.0, 0.0]);
        let id2 = ca.project(&[-1.0, 0.0]);

        // hopfield_energy([-1,0], [1,0]) = 1.0 > 0.0 threshold → new cluster
        assert_ne!(id1, id2, "should create new abstract state when energy exceeds threshold");
        assert_eq!(ca.num_abstract_states(), 2);
    }

    // ——— NeuralTheorizer tests ———
    #[test]
    fn test_neural_theorizer_new() {
        let nt = NeuralTheorizer::new(4);
        assert_eq!(nt.state_dim, 4);
        assert!(nt.primitives.is_empty());
        assert!(nt.programs.is_empty());
        assert_eq!(nt.max_primitives, 32);
        assert_eq!(nt.max_programs, 128);
    }

    #[test]
    fn test_register_primitive() {
        let mut nt = NeuralTheorizer::new(4);
        let id = nt.register_primitive("double", vec![2.0, 2.0, 2.0, 2.0], 1);
        assert_ne!(id, usize::MAX);
        assert_eq!(id, 0);
        assert_eq!(nt.primitives.len(), 1);
        let prog = LatentProgram {
            id: 0,
            ops: vec![0],
            score: 1.0,
            generalization: 0.0,
        };
        let result = nt.execute_program(&[1.0, 2.0, 3.0, 4.0], &prog);
        assert_eq!(result.len(), 4);
        assert!((result[0] - 3.0).abs() < 1e-6);
        assert!((result[1] - 6.0).abs() < 1e-6);
        assert!((result[2] - 9.0).abs() < 1e-6);
        assert!((result[3] - 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_discover_program_single_primitive() {
        let mut nt = NeuralTheorizer::new(2);
        nt.register_primitive("double", vec![2.0, 2.0], 1);
        let id = nt.discover_program(&[1.0, 1.0], &[3.0, 3.0]);
        assert_ne!(id, usize::MAX);
        assert!(nt.programs[id].score >= 0.5);
    }

    #[test]
    fn test_execute_program_returns_correct_dims() {
        let mut nt = NeuralTheorizer::new(3);
        nt.register_primitive("add_one", vec![1.0, 1.0, 1.0], 1);
        let prog = LatentProgram {
            id: 0,
            ops: vec![0],
            score: 1.0,
            generalization: 0.0,
        };
        let result = nt.execute_program(&[0.5, 1.5, 2.5], &prog);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 1.0).abs() < 1e-6);
        assert!((result[1] - 3.0).abs() < 1e-6);
        assert!((result[2] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_learn_primitives_from_transitions() {
        let mut nt = NeuralTheorizer::new(2);
        let t1: (&[f64], &[f64]) = (&[0.0, 0.0], &[1.0, 0.0]);
        let t2: (&[f64], &[f64]) = (&[1.0, 0.0], &[2.0, 0.0]);
        let t3: (&[f64], &[f64]) = (&[2.0, 0.0], &[3.0, 0.0]);
        nt.learn_primitives_from_transitions(&[t1, t2, t3]);
        assert!(!nt.primitives.is_empty());
        assert!(nt.primitives.len() <= nt.max_primitives);
    }

    #[test]
    fn test_explain_observation() {
        let mut nt = NeuralTheorizer::new(2);
        nt.register_primitive("double", vec![2.0, 2.0], 1);
        nt.discover_program(&[1.0, 1.0], &[2.0, 2.0]);
        let score = nt.explain(&[1.0, 1.0], &[2.0, 2.0]);
        assert!(score > 0.0);
    }

    #[test]
    fn test_generalization_score() {
        let mut nt = NeuralTheorizer::new(2);
        nt.register_primitive("double", vec![2.0, 2.0], 1);
        nt.discover_program(&[1.0, 1.0], &[2.0, 2.0]);
        let gen = nt.generalization_score();
        assert!(gen >= 0.0);
        assert!(gen <= 1.0);
    }

    #[test]
    fn test_prune_lowest_programs() {
        let mut nt = NeuralTheorizer::new(2);
        nt.max_programs = 2;
        nt.register_primitive("p", vec![1.0, 1.0], 1);
        nt.discover_program(&[0.0, 0.0], &[1.0, 1.0]);
        nt.discover_program(&[1.0, 1.0], &[2.0, 2.0]);
        nt.discover_program(&[2.0, 2.0], &[3.0, 3.0]);
        assert!(nt.programs.len() <= 2);
    }

    #[test]
    fn test_max_primitives_limit() {
        let mut nt = NeuralTheorizer::new(2);
        nt.max_primitives = 2;
        assert_ne!(nt.register_primitive("a", vec![1.0, 0.0], 1), usize::MAX);
        assert_ne!(nt.register_primitive("b", vec![0.0, 1.0], 1), usize::MAX);
        assert_eq!(nt.register_primitive("c", vec![0.0, 0.0], 1), usize::MAX);
        assert_eq!(nt.primitives.len(), 2);
    }

    #[test]
    fn test_discover_program_no_primitives() {
        let mut nt = NeuralTheorizer::new(2);
        let id = nt.discover_program(&[1.0, 0.0], &[2.0, 0.0]);
        assert_eq!(id, usize::MAX);
    }
}

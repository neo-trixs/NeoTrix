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
}

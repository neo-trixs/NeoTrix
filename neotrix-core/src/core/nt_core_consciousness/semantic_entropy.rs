use std::collections::VecDeque;

/// GWA-inspired semantic entropy tracker (Eq. 2-4 from arXiv:2604.08206).
/// Measures semantic diversity H(W) over recent thought vectors and
/// dynamically regulates temperature to break reasoning deadlocks.
#[derive(Debug, Clone)]
pub struct SemanticEntropyTracker {
    /// Recent thought embeddings (d-dimensional vectors)
    thought_history: VecDeque<Vec<f64>>,
    /// Maximum history window
    max_history: usize,
    /// Number of semantic clusters (K)
    n_clusters: usize,
    /// Cluster centroids (dynamically updated)
    centroids: Vec<Vec<f64>>,
    /// Baseline sampling temperature
    pub t_base: f64,
    /// Maximum exploratory variance
    pub alpha: f64,
    /// Sensitivity to entropy change
    pub beta: f64,
    /// Current semantic entropy H(W)
    current_entropy: f64,
    /// Current dynamic temperature
    current_temperature: f64,
    /// Dimensionality of embedding space
    embedding_dim: usize,
}

impl Default for SemanticEntropyTracker {
    fn default() -> Self {
        Self::new(100, 5)
    }
}

impl SemanticEntropyTracker {
    pub fn new(max_history: usize, n_clusters: usize) -> Self {
        let dim = 64; // default VSA-derived embedding dimension
        let centroids = (0..n_clusters)
            .map(|i| {
                let mut c = vec![0.0; dim];
                for j in 0..dim {
                    c[j] = (i as f64 * 0.1 + j as f64 * 0.01).sin();
                }
                let norm: f64 = c.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
                for x in &mut c {
                    *x /= norm;
                }
                c
            })
            .collect();
        Self {
            thought_history: VecDeque::with_capacity(max_history),
            max_history,
            n_clusters,
            centroids,
            t_base: 1.0,
            alpha: 2.0,
            beta: 3.0,
            current_entropy: 1.0,
            current_temperature: 1.0,
            embedding_dim: dim,
        }
    }

    /// Record a thought vector and recompute entropy + dynamic temperature.
    /// `thought_vec` should be a f64 slice of fixed dimension.
    pub fn record_thought(&mut self, thought_vec: &[f64]) {
        // Pad/truncate to embedding_dim
        let mut tv: Vec<f64> = Vec::with_capacity(self.embedding_dim);
        for (i, &v) in thought_vec.iter().enumerate() {
            if i >= self.embedding_dim {
                break;
            }
            tv.push(v);
        }
        while tv.len() < self.embedding_dim {
            tv.push(0.0);
        }
        // Normalize
        let norm: f64 = tv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
        for x in &mut tv {
            *x /= norm;
        }

        self.thought_history.push_back(tv);
        while self.thought_history.len() > self.max_history {
            self.thought_history.pop_front();
        }

        self.compute_entropy_and_temperature();
    }

    /// Eq 2-3: Compute p(x_k) via softmax over cluster distances, then H(W).
    fn compute_entropy_and_temperature(&mut self) {
        let history: Vec<&[f64]> = self.thought_history.iter().map(|v| v.as_slice()).collect();
        if history.is_empty() || self.centroids.is_empty() {
            self.current_entropy = 1.0;
            self.current_temperature = self.t_base;
            return;
        }

        // Find cluster membership for latest thought
        let latest = history.last().copied().unwrap_or(&[]);
        if latest.is_empty() {
            self.current_entropy = 1.0;
            self.current_temperature = self.t_base;
            return;
        }

        // Compute distances to each centroid (Eq 2: d_t,k = 1 - cos_sim)
        let tau = 1.0; // temperature scaling for softmax
        let mut distances = Vec::with_capacity(self.n_clusters);
        for centroid in &self.centroids {
            let dot: f64 = latest.iter().zip(centroid.iter()).map(|(a, b)| a * b).sum();
            let n1: f64 = latest.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
            let n2: f64 = centroid
                .iter()
                .map(|x| x * x)
                .sum::<f64>()
                .sqrt()
                .max(1e-10);
            let cos_sim = (dot / (n1 * n2)).clamp(-1.0, 1.0);
            distances.push(1.0 - cos_sim);
        }

        // Softmax probabilities p(x_k) = exp(-d_t,k/tau) / sum(exp(-d_j/tau))
        let max_d = distances.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = distances
            .iter()
            .map(|d| ((-d + max_d) / tau).exp())
            .collect();
        let sum_exp: f64 = exps.iter().sum();
        let probs: Vec<f64> = if sum_exp > 1e-30 {
            exps.iter().map(|e| e / sum_exp).collect()
        } else {
            vec![1.0 / self.n_clusters as f64; self.n_clusters]
        };

        // Shannon entropy H(W) = -sum(p_k * log(p_k))
        let entropy: f64 = probs
            .iter()
            .map(|p| if *p > 1e-10 { -p * p.ln() } else { 0.0 })
            .sum();
        let max_entropy = (self.n_clusters as f64).ln();
        self.current_entropy = if max_entropy > 0.0 {
            (entropy / max_entropy).clamp(0.0, 1.0)
        } else {
            1.0
        };

        // Eq 4: T_gen = T_base + alpha * exp(-beta * H(W))
        self.current_temperature =
            self.t_base + self.alpha * (-self.beta * self.current_entropy).exp();
    }

    pub fn current_entropy(&self) -> f64 {
        self.current_entropy
    }

    pub fn current_temperature(&self) -> f64 {
        self.current_temperature
    }

    /// Get the dynamic temperature for use in CompetitiveSelection or FreeEnergyCuriosity
    pub fn dynamic_temperature(&self) -> f64 {
        self.current_temperature
    }

    pub fn with_t_base(mut self, t: f64) -> Self {
        self.t_base = t;
        self
    }

    pub fn with_alpha(mut self, a: f64) -> Self {
        self.alpha = a;
        self
    }

    pub fn with_beta(mut self, b: f64) -> Self {
        self.beta = b;
        self
    }
}

use serde::{Deserialize, Serialize};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SIGReg {
    pub seed: u64,
    pub num_projections: usize,
    pub projections: Vec<Vec<f64>>,
    pub target_mean: f64,
    pub target_var: f64,
}

impl SIGReg {
    pub fn new(dim: usize, num_projections: usize, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut projections = Vec::with_capacity(num_projections);

        for _ in 0..num_projections {
            let mut v: Vec<f64> = (0..dim).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect();
            let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-8 {
                for val in v.iter_mut() {
                    *val /= norm;
                }
            }
            projections.push(v);
        }

        Self {
            seed,
            num_projections,
            projections,
            target_mean: 0.0,
            target_var: 1.0,
        }
    }

    pub fn compute_loss(&self, embeddings: &[Vec<f64>]) -> f64 {
        let n = embeddings.len();
        if n <= 1 || self.num_projections == 0 {
            return 0.0;
        }

        let mut total_loss = 0.0;
        for p in &self.projections {
            let projected: Vec<f64> = embeddings
                .iter()
                .map(|emb| emb.iter().zip(p.iter()).map(|(a, b)| a * b).sum())
                .collect();

            let mean: f64 = projected.iter().sum::<f64>() / n as f64;
            let var: f64 = projected
                .iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>()
                / n as f64;

            total_loss += (mean - self.target_mean).powi(2)
                + (var - self.target_var).powi(2);
        }

        total_loss / self.num_projections as f64
    }

    pub fn compute_loss_matrix(&self, embeddings: &[Vec<f64>]) -> f64 {
        let n = embeddings.len();
        if n <= 1 {
            return 0.0;
        }
        let d = embeddings[0].len();
        if d == 0 {
            return 0.0;
        }

        let means: Vec<f64> = (0..d)
            .map(|i| embeddings.iter().map(|v| v[i]).sum::<f64>() / n as f64)
            .collect();

        let mut frob_sq = 0.0;
        for i in 0..d {
            for j in 0..d {
                let cov_ij: f64 = embeddings
                    .iter()
                    .map(|v| (v[i] - means[i]) * (v[j] - means[j]))
                    .sum::<f64>()
                    / (n - 1) as f64;
                let target = if i == j { 1.0 } else { 0.0 };
                frob_sq += (cov_ij - target).powi(2);
            }
        }

        frob_sq
    }
}

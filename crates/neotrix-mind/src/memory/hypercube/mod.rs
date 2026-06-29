// TODO: full HyperCube migration — currently a stub for system1/self_reasoner
pub const VSA_DIM: usize = 4096;
pub type QuantizedVSA = Vec<f64>;

use rand::Rng;

#[derive(Debug, Clone)]
pub struct HyperCube;

impl HyperCube {
    pub fn new(_dim: usize) -> Self {
        Self
    }

    pub fn bind(a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter().zip(b).map(|(x, y)| x * y).collect()
    }

    pub fn bundle(a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter().zip(b).map(|(x, y)| x + y).collect()
    }

    pub fn similarity(a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }

    pub fn permute(v: &[f64], _n: usize) -> Vec<f64> {
        v.to_vec()
    }

    pub fn seeded_vector(&self, seed: u64) -> Vec<f64> {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        (0..VSA_DIM).map(|_| rng.gen_range(-1.0_f64..1.0_f64)).collect()
    }
}

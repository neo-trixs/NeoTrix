// REVIVED Task 2 — dead_code removed

use rand::Rng;

pub trait VsaModel: Send + Sync + std::fmt::Debug {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64>;
    fn bundle(&self, vecs: &[&[f64]]) -> Vec<f64>;
    fn similarity(&self, a: &[f64], b: &[f64]) -> f64;
    fn invert(&self, v: &[f64]) -> Vec<f64>;
    fn cleanup(&self, v: &[f64], codebook: &[Vec<f64>]) -> Vec<f64>;
    fn dimension(&self) -> usize;
    fn name(&self) -> &str;

    /// Bidirectional recurrent binding: iteratively refines the binding
    /// through bottom-up, top-down, and lateral passes.
    /// Default: iterative bind+bundle cycles (Nature Comms 2026).
    fn bidirectional_bind(&self, a: &[f64], b: &[f64], iterations: usize) -> Vec<f64> {
        let mut bound = self.bind(a, b);
        for _ in 0..iterations {
            let a_to_b = self.bind(a, &bound);
            let b_to_a = self.bind(b, &bound);
            bound = self.bundle(&[&a_to_b, &b_to_a]);
        }
        bound
    }

    /// Soft weighted binding — attention-weighted superposition.
    /// Unlike deterministic bind (binary/hard), produces a weighted
    /// mixture controlled by attention values (AAAI 2026 Bridge).
    fn soft_bind(&self, items: &[(&[f64], f64)]) -> Vec<f64> {
        if items.is_empty() {
            return vec![0.0; self.dimension()];
        }
        let total_weight: f64 = items.iter().map(|(_, w)| w).sum();
        if total_weight == 0.0 {
            return vec![0.0; self.dimension()];
        }
        let dim = self.dimension();
        let mut result = vec![0.0; dim];
        for (v, w) in items {
            let weight = w / total_weight;
            for i in 0..dim {
                result[i] += v[i] * weight;
            }
        }
        result
    }

    /// Orthogonal Subspace Carving bind (arXiv 2606.11391).
    /// Projects filler onto the null space of the role basis before
    /// bundling, eliminating cross-talk noise in high-superposition regimes.
    /// Unlike standard bind (produces a new compound vector), OSC bind
    /// returns a filtered vector suitable for bundling into a memory tensor
    /// with reduced interference.
    fn osc_bind(&self, filler: &[f64], role: &[f64]) -> Vec<f64> {
        let _dim = self.dimension();
        let dot: f64 = filler.iter().zip(role.iter()).map(|(a, b)| a * b).sum();
        let role_norm_sq: f64 = role.iter().map(|x| x * x).sum();
        if role_norm_sq < 1e-30 {
            return filler.to_vec();
        }
        let projection_scale = dot / role_norm_sq;
        filler
            .iter()
            .zip(role.iter())
            .map(|(f, r)| f - projection_scale * r)
            .collect()
    }
}

#[derive(Debug)]
pub struct MapModel {
    dim: usize,
}

impl MapModel {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl VsaModel for MapModel {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).collect()
    }
    fn bundle(&self, vecs: &[&[f64]]) -> Vec<f64> {
        let n = vecs.len() as f64;
        (0..self.dim)
            .map(|i| vecs.iter().map(|v| v[i]).sum::<f64>() / n)
            .collect()
    }
    fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if na == 0.0 || nb == 0.0 {
            0.0
        } else {
            dot / (na * nb)
        }
    }
    fn invert(&self, v: &[f64]) -> Vec<f64> {
        v.iter().map(|x| -x).collect()
    }
    fn cleanup(&self, v: &[f64], codebook: &[Vec<f64>]) -> Vec<f64> {
        let mut best = 0usize;
        let mut best_sim = -1.0f64;
        for (i, c) in codebook.iter().enumerate() {
            let s = self.similarity(v, c);
            if s > best_sim {
                best_sim = s;
                best = i;
            }
        }
        codebook[best].clone()
    }
    fn dimension(&self) -> usize {
        self.dim
    }
    fn name(&self) -> &str {
        "MAP"
    }
}

#[derive(Debug)]
pub struct BscModel {
    dim: usize,
}

impl BscModel {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl VsaModel for BscModel {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| if (*x > 0.0) != (*y > 0.0) { -1.0 } else { 1.0 })
            .collect()
    }
    fn bundle(&self, vecs: &[&[f64]]) -> Vec<f64> {
        (0..self.dim)
            .map(|i| {
                let sum: f64 = vecs
                    .iter()
                    .map(|v| if v[i] > 0.0 { 1.0 } else { -1.0 })
                    .sum();
                if sum > 0.0 {
                    1.0
                } else {
                    -1.0
                }
            })
            .collect()
    }
    fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let hamming: f64 = a
            .iter()
            .zip(b.iter())
            .filter(|(x, y)| (**x > 0.0) != (**y > 0.0))
            .count() as f64;
        1.0 - hamming / self.dim as f64
    }
    fn invert(&self, v: &[f64]) -> Vec<f64> {
        v.iter().map(|x| -x).collect()
    }
    fn cleanup(&self, v: &[f64], codebook: &[Vec<f64>]) -> Vec<f64> {
        let mut best = 0usize;
        let mut best_sim = -1.0;
        for (i, c) in codebook.iter().enumerate() {
            let s = self.similarity(v, c);
            if s > best_sim {
                best_sim = s;
                best = i;
            }
        }
        codebook[best].clone()
    }
    fn dimension(&self) -> usize {
        self.dim
    }
    fn name(&self) -> &str {
        "BSC"
    }
}

#[derive(Debug)]
pub struct HrrModel {
    dim: usize,
}

impl HrrModel {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
    fn circular_conv(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        let n = a.len();
        let mut out = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                out[i] += a[j] * b[(i - j + n) % n];
            }
        }
        out
    }
    fn circular_corr(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        let n = a.len();
        let mut out = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                out[i] += a[j] * b[(i + j) % n];
            }
        }
        out
    }
}

impl VsaModel for HrrModel {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        self.circular_conv(a, b)
    }
    fn bundle(&self, vecs: &[&[f64]]) -> Vec<f64> {
        let n = vecs.len() as f64;
        (0..self.dim)
            .map(|i| vecs.iter().map(|v| v[i]).sum::<f64>() / n)
            .collect()
    }
    fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if na == 0.0 || nb == 0.0 {
            0.0
        } else {
            dot / (na * nb)
        }
    }
    fn invert(&self, v: &[f64]) -> Vec<f64> {
        let n = v.len();
        let mut inv = vec![0.0; n];
        for i in 0..n {
            inv[i] = v[(n - i) % n];
        }
        inv
    }
    fn cleanup(&self, v: &[f64], codebook: &[Vec<f64>]) -> Vec<f64> {
        let mut best = 0usize;
        let mut best_sim = -1.0;
        for (i, c) in codebook.iter().enumerate() {
            let s = self.similarity(v, c);
            if s > best_sim {
                best_sim = s;
                best = i;
            }
        }
        codebook[best].clone()
    }
    fn dimension(&self) -> usize {
        self.dim
    }
    fn name(&self) -> &str {
        "HRR"
    }
}

#[derive(Debug)]
pub struct FhrrModel {
    dim: usize,
}

impl FhrrModel {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl VsaModel for FhrrModel {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x + y).rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI)
            .collect()
    }
    fn bundle(&self, vecs: &[&[f64]]) -> Vec<f64> {
        let n = vecs.len() as f64;
        (0..self.dim)
            .map(|i| {
                let sum_sin: f64 = vecs.iter().map(|v| v[i].sin()).sum();
                let sum_cos: f64 = vecs.iter().map(|v| v[i].cos()).sum();
                (sum_sin / n).atan2(sum_cos / n)
            })
            .collect()
    }
    fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).cos()).sum();
        dot / self.dim as f64
    }
    fn invert(&self, v: &[f64]) -> Vec<f64> {
        v.iter()
            .map(|x| (-x).rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI)
            .collect()
    }
    fn cleanup(&self, v: &[f64], codebook: &[Vec<f64>]) -> Vec<f64> {
        let mut best = 0usize;
        let mut best_sim = -1.0;
        for (i, c) in codebook.iter().enumerate() {
            let s = self.similarity(v, c);
            if s > best_sim {
                best_sim = s;
                best = i;
            }
        }
        codebook[best].clone()
    }
    fn dimension(&self) -> usize {
        self.dim
    }
    fn name(&self) -> &str {
        "FHRR"
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SupportedModel {
    Map,
    Bsc,
    Hrr,
    Fhrr,
}

pub struct ModelFactory;

impl ModelFactory {
    pub fn create(model: SupportedModel, dim: usize) -> Box<dyn VsaModel> {
        match model {
            SupportedModel::Map => Box::new(MapModel::new(dim)),
            SupportedModel::Bsc => Box::new(BscModel::new(dim)),
            SupportedModel::Hrr => Box::new(HrrModel::new(dim)),
            SupportedModel::Fhrr => Box::new(FhrrModel::new(dim)),
        }
    }

    pub fn create_by_name(name: &str, dim: usize) -> Option<Box<dyn VsaModel>> {
        match name.to_ascii_lowercase().as_str() {
            "map" => Some(Box::new(MapModel::new(dim))),
            "bsc" => Some(Box::new(BscModel::new(dim))),
            "hrr" => Some(Box::new(HrrModel::new(dim))),
            "fhrr" => Some(Box::new(FhrrModel::new(dim))),
            _ => None,
        }
    }

    pub fn random_vector(dim: usize, rng: &mut impl Rng) -> Vec<f64> {
        (0..dim).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_models() -> Vec<(SupportedModel, &'static str)> {
        vec![
            (SupportedModel::Map, "MAP"),
            (SupportedModel::Bsc, "BSC"),
            (SupportedModel::Hrr, "HRR"),
            (SupportedModel::Fhrr, "FHRR"),
        ]
    }

    #[test]
    fn test_bind_unbind_roundtrip_map() {
        let model = MapModel::new(128);
        let mut rng = rand::thread_rng();
        let a = ModelFactory::random_vector(128, &mut rng);
        let b = ModelFactory::random_vector(128, &mut rng);
        let bound = model.bind(&a, &b);
        let inv_b = model.invert(&b);
        let unbound = model.bind(&bound, &inv_b);
        let sim = model.similarity(&a, &unbound);
        assert!(sim > 0.5, "MAP unbind roundtrip sim={}", sim);
    }

    #[test]
    fn test_bind_unbind_roundtrip_bsc() {
        let model = BscModel::new(128);
        let mut rng = rand::thread_rng();
        let a = ModelFactory::random_vector(128, &mut rng);
        let b = ModelFactory::random_vector(128, &mut rng);
        let bound = model.bind(&a, &b);
        let unbound = model.bind(&bound, &b);
        let sim = model.similarity(&a, &unbound);
        assert!(sim > 0.5, "BSC unbind roundtrip sim={}", sim);
    }

    #[test]
    fn test_bundle_identity_map() {
        let model = MapModel::new(64);
        let mut rng = rand::thread_rng();
        let a = ModelFactory::random_vector(64, &mut rng);
        let bundle = model.bundle(&[&a, &a, &a]);
        let sim = model.similarity(&a, &bundle);
        assert!(sim > 0.8, "MAP bundle same vector sim={}", sim);
    }

    #[test]
    fn test_similarity_self() {
        for (variant, _) in test_models() {
            let model = ModelFactory::create(variant, 64);
            let mut rng = rand::thread_rng();
            let v = ModelFactory::random_vector(64, &mut rng);
            let sim = model.similarity(&v, &v);
            assert!(
                (sim - 1.0).abs() < 1e-6,
                "{} self-sim={}",
                model.name(),
                sim
            );
        }
    }

    #[test]
    fn test_invert_twice_identity() {
        let model = MapModel::new(64);
        let mut rng = rand::thread_rng();
        let v = ModelFactory::random_vector(64, &mut rng);
        let inv = model.invert(&v);
        let inv2 = model.invert(&inv);
        let sim = model.similarity(&v, &inv2);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cleanup_returns_closest() {
        let model = MapModel::new(64);
        let mut rng = rand::thread_rng();
        let a = ModelFactory::random_vector(64, &mut rng);
        let b = ModelFactory::random_vector(64, &mut rng);
        let codebook = vec![a.clone(), b.clone()];
        let noisy: Vec<f64> = a.iter().map(|x| x + rng.gen::<f64>() * 0.1).collect();
        let cleaned = model.cleanup(&noisy, &codebook);
        let sim_to_a = model.similarity(&cleaned, &a);
        assert!(
            sim_to_a > 0.5,
            "cleanup should return closest, sim={}",
            sim_to_a
        );
    }

    #[test]
    fn test_factory_by_name() {
        for name in &["map", "bsc", "hrr", "fhrr"] {
            let model = ModelFactory::create_by_name(name, 64);
            assert!(model.is_some(), "factory should create {}", name);
        }
        assert!(ModelFactory::create_by_name("unknown", 64).is_none());
    }

    #[test]
    fn test_hrr_bind_similarity() {
        let model = HrrModel::new(32);
        let mut rng = rand::thread_rng();
        let a = ModelFactory::random_vector(32, &mut rng);
        let b = ModelFactory::random_vector(32, &mut rng);
        let bound = model.bind(&a, &b);
        let inv = model.invert(&b);
        let unbound = model.bind(&bound, &inv);
        let sim = model.similarity(&a, &unbound);
        assert!(sim > 0.3, "HRR unbind sim={}", sim);
    }

    #[test]
    fn test_bsc_majority_bundle() {
        let model = BscModel::new(128);
        let mut rng = rand::thread_rng();
        let a = ModelFactory::random_vector(128, &mut rng);
        let all: Vec<&[f64]> = vec![&a; 5];
        let bundle = model.bundle(&all);
        let sim = model.similarity(&a, &bundle);
        assert!(sim > 0.5, "BSC majority bundle sim={}", sim);
    }

    #[test]
    fn test_fhrr_phase_wrapping() {
        let model = FhrrModel::new(64);
        let mut rng = rand::thread_rng();
        let a: Vec<f64> = (0..64)
            .map(|_| rng.gen::<f64>() * 2.0 * std::f64::consts::PI - std::f64::consts::PI)
            .collect();
        let b: Vec<f64> = (0..64)
            .map(|_| rng.gen::<f64>() * 2.0 * std::f64::consts::PI - std::f64::consts::PI)
            .collect();
        let bound = model.bind(&a, &b);
        for &x in &bound {
            assert!(x >= -std::f64::consts::PI && x <= std::f64::consts::PI);
        }
    }

    #[test]
    fn test_empty_codebook_cleanup() {
        let model = MapModel::new(64);
        let mut rng = rand::thread_rng();
        let v = ModelFactory::random_vector(64, &mut rng);
        let result = model.cleanup(&v, &[]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_dimension_consistency() {
        for name in &["map", "bsc", "hrr", "fhrr"] {
            let model = ModelFactory::create_by_name(name, 256).unwrap();
            assert_eq!(model.dimension(), 256);
        }
    }

    #[test]
    fn test_bundle_three_vectors_map() {
        let model = MapModel::new(64);
        let mut rng = rand::thread_rng();
        let a = ModelFactory::random_vector(64, &mut rng);
        let b = ModelFactory::random_vector(64, &mut rng);
        let c = ModelFactory::random_vector(64, &mut rng);
        let bundle = model.bundle(&[&a, &b, &c]);
        assert_eq!(bundle.len(), 64);
        let sim_ab = model.similarity(&a, &b);
        let sim_abundle = model.similarity(&a, &bundle);
        assert!(sim_abundle > sim_ab * 0.3);
    }

    #[test]
    fn test_different_models_different_bindings() {
        let map = MapModel::new(64);
        let bsc = BscModel::new(64);
        let mut rng = rand::thread_rng();
        let a = ModelFactory::random_vector(64, &mut rng);
        let b = ModelFactory::random_vector(64, &mut rng);
        let map_bound = map.bind(&a, &b);
        let bsc_bound = bsc.bind(&a, &b);
        let sim = map.similarity(&map_bound, &bsc_bound);
        assert!(
            sim < 0.9,
            "different models should give different results, sim={}",
            sim
        );
    }
}

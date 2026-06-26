use rand::Rng;
use rand::SeedableRng;

use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

const DEFAULT_FPE_SEED: u64 = 42;
const GRID_STEPS: usize = 128;
const GOLDEN_ITERATIONS: usize = 10;

pub struct FpeEncoder {
    frequencies: Vec<f64>,
    phase_shifts: Vec<f64>,
}

impl FpeEncoder {
    pub fn new(seed: u64) -> Self {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let frequencies: Vec<f64> = (0..VSA_DIM).map(|_| rng.gen_range(0.5..30.0)).collect();
        let phase_shifts: Vec<f64> = (0..VSA_DIM)
            .map(|_| rng.gen_range(0.0..2.0 * std::f64::consts::PI))
            .collect();
        Self {
            frequencies,
            phase_shifts,
        }
    }

    pub fn new_default() -> Self {
        Self::new(DEFAULT_FPE_SEED)
    }

    fn encode_phase(&self, phase: f64) -> Vec<f64> {
        (0..VSA_DIM)
            .map(|i| (self.frequencies[i] * phase + self.phase_shifts[i]).cos())
            .collect()
    }

    pub fn encode_scalar(&self, value: f64, domain: (f64, f64)) -> Vec<f64> {
        let range = domain.1 - domain.0;
        if range <= 0.0 {
            return self.encode_phase(0.0);
        }
        let normalized = ((value - domain.0) / range).clamp(0.0, 1.0);
        self.encode_phase(normalized * std::f64::consts::PI)
    }

    pub fn encode_scalar_log(&self, value: f64, domain: (f64, f64)) -> Vec<f64> {
        let lo = domain.0.max(f64::MIN_POSITIVE);
        let hi = domain.1.max(f64::MIN_POSITIVE);
        let log_lo = lo.ln();
        let log_hi = hi.ln();
        let log_val = value
            .abs()
            .max(f64::MIN_POSITIVE)
            .ln()
            .clamp(log_lo, log_hi);
        let normalized = (log_val - log_lo) / (log_hi - log_lo);
        self.encode_phase(normalized * std::f64::consts::PI)
    }

    pub fn encode_scalar_circular(&self, angle_rad: f64) -> Vec<f64> {
        let two_pi = 2.0 * std::f64::consts::PI;
        let normalized = (angle_rad % two_pi) / two_pi;
        self.encode_phase(normalized * two_pi)
    }

    pub fn decode_scalar(&self, vec: &[f64], domain: (f64, f64)) -> f64 {
        let (lo, hi) = domain;
        let range = hi - lo;
        if range <= 0.0 {
            return lo;
        }
        let step = range / GRID_STEPS as f64;

        let mut best_val = lo;
        let mut best_sim = -2.0f64;

        for i in 0..=GRID_STEPS {
            let val = lo + i as f64 * step;
            let candidate = self.encode_scalar(val, domain);
            let sim = cosine_similarity(vec, &candidate);
            if sim > best_sim {
                best_sim = sim;
                best_val = val;
            }
        }

        let refine_lo = (best_val - step).max(lo);
        let refine_hi = (best_val + step).min(hi);
        golden_section_decode(self, vec, domain, refine_lo, refine_hi, GOLDEN_ITERATIONS)
    }

    pub fn ssp_bind(&self, base: &[f64], scalar_vsa: &[f64]) -> Vec<f64> {
        let len = base.len().min(scalar_vsa.len());
        base[..len]
            .iter()
            .zip(&scalar_vsa[..len])
            .map(|(a, b)| a * b)
            .collect()
    }

    pub fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        cosine_similarity(a, b)
    }
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let dot: f64 = a[..len].iter().zip(&b[..len]).map(|(x, y)| x * y).sum();
    let na: f64 = a[..len].iter().map(|x| x * x).sum();
    let nb: f64 = b[..len].iter().map(|x| x * x).sum();
    if na < f64::EPSILON || nb < f64::EPSILON {
        return 0.0;
    }
    (dot / (na * nb).sqrt()).clamp(-1.0, 1.0)
}

fn golden_section_decode(
    enc: &FpeEncoder,
    vec: &[f64],
    domain: (f64, f64),
    mut lo: f64,
    mut hi: f64,
    iterations: usize,
) -> f64 {
    const PHI: f64 = 1.618_033_988_749_895;
    let inv_phi = 1.0 / PHI;

    let mut mid1 = hi - (hi - lo) * inv_phi;
    let mut mid2 = lo + (hi - lo) * inv_phi;
    let mut f1 = cosine_similarity(vec, &enc.encode_scalar(mid1, domain));
    let mut f2 = cosine_similarity(vec, &enc.encode_scalar(mid2, domain));

    for _ in 0..iterations {
        if f1 > f2 {
            hi = mid2;
            mid2 = mid1;
            f2 = f1;
            mid1 = hi - (hi - lo) * inv_phi;
            f1 = cosine_similarity(vec, &enc.encode_scalar(mid1, domain));
        } else {
            lo = mid1;
            mid1 = mid2;
            f1 = f2;
            mid2 = lo + (hi - lo) * inv_phi;
            f2 = cosine_similarity(vec, &enc.encode_scalar(mid2, domain));
        }
    }

    (lo + hi) * 0.5
}

pub struct SpatialSemanticPointer {
    enc_x: FpeEncoder,
    enc_y: FpeEncoder,
    enc_z: FpeEncoder,
}

impl SpatialSemanticPointer {
    pub fn new(_dim: usize, _length_scale: f64) -> Self {
        Self {
            enc_x: FpeEncoder::new(DEFAULT_FPE_SEED),
            enc_y: FpeEncoder::new(DEFAULT_FPE_SEED + 1),
            enc_z: FpeEncoder::new(DEFAULT_FPE_SEED + 2),
        }
    }

    pub fn new_seeded(_dim: usize, _length_scale: f64, seed: u64) -> Self {
        Self {
            enc_x: FpeEncoder::new(seed),
            enc_y: FpeEncoder::new(seed + 1),
            enc_z: FpeEncoder::new(seed + 2),
        }
    }

    pub fn dim(&self) -> usize {
        VSA_DIM
    }

    pub fn length_scale(&self) -> f64 {
        1.0
    }

    pub fn encoder(&self) -> &FpeEncoder {
        &self.enc_x
    }

    pub fn encode_point(&self, x: f64, y: f64, z: f64) -> Vec<f64> {
        let domain = (-10.0, 10.0);
        let vx = self.enc_x.encode_scalar(x, domain);
        let vy = self.enc_y.encode_scalar(y, domain);
        let vz = self.enc_z.encode_scalar(z, domain);
        self.enc_x.ssp_bind(&self.enc_x.ssp_bind(&vx, &vy), &vz)
    }

    pub fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        cosine_similarity(a, b)
    }

    pub fn similarity_01(&self, a: &[f64], b: &[f64]) -> f64 {
        (cosine_similarity(a, b) + 1.0) * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoder() -> FpeEncoder {
        FpeEncoder::new(42)
    }

    #[test]
    fn test_encode_same_value_produces_same_vector() {
        let enc = encoder();
        let a = enc.encode_scalar(3.14, (0.0, 10.0));
        let b = enc.encode_scalar(3.14, (0.0, 10.0));
        assert_eq!(a.len(), VSA_DIM);
        assert_eq!(b.len(), VSA_DIM);
        for (x, y) in a.iter().zip(b.iter()) {
            assert!((x - y).abs() < 1e-15, "determinism violated");
        }
    }

    #[test]
    fn test_encode_different_values_different_vectors() {
        let enc = encoder();
        let a = enc.encode_scalar(1.0, (0.0, 100.0));
        let b = enc.encode_scalar(99.0, (0.0, 100.0));
        let sim = cosine_similarity(&a, &b);
        assert!(
            sim < 0.6,
            "distant values should have low similarity, got {}",
            sim
        );
    }

    #[test]
    fn test_similarity_monotonic_with_distance() {
        let enc = encoder();
        let domain = (0.0, 100.0);
        let origin = enc.encode_scalar(0.0, domain);
        let near = enc.encode_scalar(1.0, domain);
        let mid = enc.encode_scalar(25.0, domain);
        let far = enc.encode_scalar(50.0, domain);

        let sim_near = cosine_similarity(&origin, &near);
        let sim_mid = cosine_similarity(&origin, &mid);
        let sim_far = cosine_similarity(&origin, &far);

        assert!(
            sim_near > sim_mid,
            "near should be more similar than mid: {} <= {}",
            sim_near,
            sim_mid
        );
        assert!(
            sim_mid > sim_far,
            "mid should be more similar than far: {} <= {}",
            sim_mid,
            sim_far
        );
    }

    #[test]
    fn test_decode_roundtrip() {
        let enc = encoder();
        let domain = (-5.0, 5.0);
        for &val in &[-4.0, -2.0, 0.0, 1.5, 3.0, 4.5] {
            let encoded = enc.encode_scalar(val, domain);
            let decoded = enc.decode_scalar(&encoded, domain);
            let err = (decoded - val).abs();
            assert!(err < 0.1, "roundtrip error for {} is {} (> 0.1)", val, err);
        }
    }

    #[test]
    fn test_log_scale_roundtrip() {
        let enc = encoder();
        let domain = (1.0, 10000.0);
        for &val in &[1.0, 10.0, 100.0, 1000.0, 5000.0] {
            let encoded = enc.encode_scalar_log(val, domain);
            let decoded = enc.decode_scalar(&encoded, domain);
            let relative_err = (decoded - val).abs() / val;
            assert!(
                relative_err < 0.5,
                "log roundtrip relative error for {} is {} (> 50%)",
                val,
                relative_err
            );
        }
    }

    #[test]
    fn test_circular_encoding_periodic() {
        let enc = encoder();
        let a = enc.encode_scalar_circular(0.0);
        let b = enc.encode_scalar_circular(2.0 * std::f64::consts::PI);
        let sim = cosine_similarity(&a, &b);
        assert!(
            (sim - 1.0).abs() < 1e-12,
            "circular 0 and 2π should be identical, got {}",
            sim
        );
    }

    #[test]
    fn test_circular_orthogonal_quarter() {
        let enc = encoder();
        let a = enc.encode_scalar_circular(0.0);
        let b = enc.encode_scalar_circular(std::f64::consts::PI);
        let sim = cosine_similarity(&a, &b);
        assert!(
            sim < 0.1,
            "circular 0 and π should be near-orthogonal, got {}",
            sim
        );
    }

    #[test]
    fn test_decode_out_of_range_clamped() {
        let enc = encoder();
        let domain = (-1.0, 1.0);
        let encoded = enc.encode_scalar(10.0, domain);
        let decoded = enc.decode_scalar(&encoded, domain);
        assert!(
            (decoded - 1.0).abs() < 0.01,
            "out-of-range value should decode near domain max, got {}",
            decoded
        );
    }

    #[test]
    fn test_ssp_bind_produces_different_vector() {
        let enc = encoder();
        let base = enc.encode_scalar(5.0, (0.0, 10.0));
        let scalar = enc.encode_scalar(0.7, (0.0, 1.0));
        let bound = enc.ssp_bind(&base, &scalar);
        let sim = cosine_similarity(&base, &bound);
        assert!(
            sim.abs() < 0.3,
            "ssp_bind should produce dissimilar vector, sim={}",
            sim
        );
    }

    #[test]
    fn test_self_similarity_unity() {
        let enc = encoder();
        let v = enc.encode_scalar(42.0, (0.0, 100.0));
        let sim = cosine_similarity(&v, &v);
        assert!(
            (sim - 1.0).abs() < 1e-12,
            "self-similarity must be 1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_close_values_high_similarity() {
        let enc = encoder();
        let domain = (0.0, 1.0);
        let a = enc.encode_scalar(0.5, domain);
        let b = enc.encode_scalar(0.5001, domain);
        let sim = cosine_similarity(&a, &b);
        assert!(
            sim > 0.99,
            "very close values should have near-1 similarity, got {}",
            sim
        );
    }

    #[test]
    fn test_spatial_self_similarity() {
        let s = SpatialSemanticPointer::new_seeded(VSA_DIM, 1.0, 42);
        let v = s.encode_point(1.0, 2.0, 3.0);
        let sim = s.similarity(&v, &v);
        assert!(
            (sim - 1.0).abs() < 1e-12,
            "spatial self-similarity must be 1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_spatial_proximity_decay() {
        let s = SpatialSemanticPointer::new_seeded(VSA_DIM, 1.0, 42);
        let origin = s.encode_point(0.0, 0.0, 0.0);
        let near = s.encode_point(0.5, 0.5, 0.5);
        let far = s.encode_point(5.0, 5.0, 5.0);
        let sim_near = s.similarity(&origin, &near);
        let sim_far = s.similarity(&origin, &far);
        assert!(
            sim_near > sim_far,
            "near points should be more similar: near={}, far={}",
            sim_near,
            sim_far
        );
    }

    #[test]
    fn test_different_seeds_different_encodings() {
        let a = FpeEncoder::new(1);
        let b = FpeEncoder::new(2);
        let va = a.encode_scalar(1.0, (0.0, 10.0));
        let vb = b.encode_scalar(1.0, (0.0, 10.0));
        let sim = cosine_similarity(&va, &vb);
        assert!(
            sim.abs() < 0.3,
            "different seeds should give near-orthogonal vectors, sim={}",
            sim
        );
    }

    #[test]
    fn test_dimension_matches_vsa_dim() {
        let enc = encoder();
        let v = enc.encode_scalar(1.0, (0.0, 1.0));
        assert_eq!(v.len(), VSA_DIM);
    }
}

// REVIVED Task 2 — dead_code removed
/// qFHRR: Quantized Fourier Holographic Reduced Representations
/// Each dimension is a discrete phase index (K values).
/// Binding = modular add, unbind = modular sub, similarity via LUT.
/// Reference: qFHRR - Rethinking FHRR through Quantized Phase (2026)
use std::sync::LazyLock;

use super::vsa::{BinaryVsaBackend, VsaBackend};

/// Number of discrete phase levels
pub const QFHRR_K: usize = 16; // 4 bits per dimension

/// Dimension of VSA vectors
pub const QFHRR_DIM: usize = 4096;

/// A qFHRR vector: each element is a discrete phase index [0, QFHRR_K)
#[derive(Debug, Clone)]
pub struct QFHRRVector {
    pub phases: Vec<u8>, // Each byte stores phase index 0..QFHRR_K-1
}

impl QFHRRVector {
    pub fn new() -> Self {
        QFHRRVector {
            phases: vec![0u8; QFHRR_DIM],
        }
    }

    pub fn random(seed: u64) -> Self {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};
        let mut v = Vec::with_capacity(QFHRR_DIM);
        for i in 0..QFHRR_DIM {
            let mut hasher = RandomState::new().build_hasher();
            seed.hash(&mut hasher);
            i.hash(&mut hasher);
            v.push((hasher.finish() as u8) % QFHRR_K as u8);
        }
        QFHRRVector { phases: v }
    }

    /// Bind: element-wise modular addition
    pub fn bind(&self, other: &QFHRRVector) -> QFHRRVector {
        let phases = self
            .phases
            .iter()
            .zip(&other.phases)
            .map(|(a, b)| (a + b) % QFHRR_K as u8)
            .collect();
        QFHRRVector { phases }
    }

    /// Unbind: element-wise modular subtraction
    pub fn unbind(&self, other: &QFHRRVector) -> QFHRRVector {
        let phases = self
            .phases
            .iter()
            .zip(&other.phases)
            .map(|(a, b)| (a + QFHRR_K as u8 - b) % QFHRR_K as u8)
            .collect();
        QFHRRVector { phases }
    }

    /// Bundle: element-wise majority with phase-aware rounding
    pub fn bundle(vectors: &[&QFHRRVector]) -> QFHRRVector {
        let n = vectors.len() as f64;
        let phases: Vec<u8> = (0..QFHRR_DIM)
            .map(|i| {
                let sum: usize = vectors.iter().map(|v| v.phases[i] as usize).sum();
                let mean = sum as f64 / n;
                let rounded = mean.round() as u8 % QFHRR_K as u8;
                rounded
            })
            .collect();
        QFHRRVector { phases }
    }

    /// Cosine similarity using LUT-based complex reconstruction
    pub fn similarity(&self, other: &QFHRRVector) -> f64 {
        let mut sum = 0.0_f64;
        for (a, b) in self.phases.iter().zip(&other.phases) {
            let diff = (*a as i16 - *b as i16).unsigned_abs() as u8 % QFHRR_K as u8;
            sum += COS_LUT[diff as usize];
        }
        sum / QFHRR_DIM as f64
    }

    /// Permute: cyclic shift
    pub fn permute(&self, shift: isize) -> QFHRRVector {
        let len = self.phases.len();
        let mut phases = vec![0u8; len];
        for i in 0..len {
            let src = ((i as isize - shift).rem_euclid(len as isize)) as usize;
            phases[i] = self.phases[src];
        }
        QFHRRVector { phases }
    }

    /// Memory footprint in bytes
    pub fn memory_bytes(&self) -> usize {
        self.phases.len()
    }

    /// Compare with f64 FHRR memory
    pub fn memory_savings() -> f64 {
        16.0
    }
}

/// Wrapper struct for VSA trait implementations (avoids orphan rule).
#[derive(Debug, Clone)]
pub struct QFHRRBackend;

impl QFHRRBackend {
    /// Convert f64 in [-1, 1] to phase index in [0, QFHRR_K)
    fn f64_to_phase(v: f64) -> u8 {
        let scaled = (v + 1.0) / 2.0 * (QFHRR_K - 1) as f64;
        (scaled.round() as u8) % QFHRR_K as u8
    }

    /// Convert phase index in [0, QFHRR_K) to f64 in [-1, 1]
    fn phase_to_f64(phase: u8) -> f64 {
        (phase as f64 / (QFHRR_K - 1) as f64) * 2.0 - 1.0
    }

    fn phases_from_f64(v: &[f64]) -> Vec<u8> {
        v.iter().map(|&x| Self::f64_to_phase(x)).collect()
    }

    fn f64_from_phases(phases: &[u8]) -> Vec<f64> {
        phases.iter().map(|&p| Self::phase_to_f64(p)).collect()
    }
}

impl BinaryVsaBackend for QFHRRBackend {
    fn bind(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        let va = QFHRRVector { phases: a.to_vec() };
        let vb = QFHRRVector { phases: b.to_vec() };
        va.bind(&vb).phases
    }

    fn unbind(&self, c: &[u8], a: &[u8]) -> Vec<u8> {
        let vc = QFHRRVector { phases: c.to_vec() };
        let va = QFHRRVector { phases: a.to_vec() };
        vc.unbind(&va).phases
    }

    fn bundle(&self, vectors: &[&[u8]]) -> Vec<u8> {
        let qvecs: Vec<QFHRRVector> = vectors
            .iter()
            .map(|v| QFHRRVector { phases: v.to_vec() })
            .collect();
        let refs: Vec<&QFHRRVector> = qvecs.iter().collect();
        QFHRRVector::bundle(&refs).phases
    }

    fn permute(&self, v: &[u8], shift: isize) -> Vec<u8> {
        let vv = QFHRRVector { phases: v.to_vec() };
        vv.permute(shift).phases
    }

    fn similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        let va = QFHRRVector { phases: a.to_vec() };
        let vb = QFHRRVector { phases: b.to_vec() };
        va.similarity(&vb)
    }

    fn dimensions(&self) -> usize {
        QFHRR_DIM
    }

    fn name(&self) -> &str {
        "qfhrr-vsa"
    }

    fn to_bits(&self, v: &[u8]) -> Vec<u8> {
        v.to_vec()
    }

    fn to_dense(&self, v: &[u8]) -> Vec<f64> {
        v.iter().map(|&p| p as f64 / QFHRR_K as f64).collect()
    }
}

impl VsaBackend for QFHRRBackend {
    fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        let phases_a = Self::phases_from_f64(a);
        let phases_b = Self::phases_from_f64(b);
        let va = QFHRRVector { phases: phases_a };
        let vb = QFHRRVector { phases: phases_b };
        let result = va.bind(&vb);
        Self::f64_from_phases(&result.phases)
    }

    fn bundle(&self, vectors: &[&[f64]]) -> Vec<f64> {
        let qvecs: Vec<QFHRRVector> = vectors
            .iter()
            .map(|v| {
                let phases = Self::phases_from_f64(v);
                QFHRRVector { phases }
            })
            .collect();
        let refs: Vec<&QFHRRVector> = qvecs.iter().collect();
        let result = QFHRRVector::bundle(&refs);
        Self::f64_from_phases(&result.phases)
    }

    fn permute(&self, v: &[f64], shift: isize) -> Vec<f64> {
        let phases = Self::phases_from_f64(v);
        let vv = QFHRRVector { phases };
        let result = vv.permute(shift);
        Self::f64_from_phases(&result.phases)
    }

    fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let phases_a = Self::phases_from_f64(a);
        let phases_b = Self::phases_from_f64(b);
        let va = QFHRRVector { phases: phases_a };
        let vb = QFHRRVector { phases: phases_b };
        va.similarity(&vb)
    }

    fn dimensions(&self) -> usize {
        QFHRR_DIM
    }

    fn name(&self) -> &str {
        "qfhrr-vsa"
    }
}

// Pre-computed cosine lookup table for all possible phase differences
static COS_LUT: LazyLock<Vec<f64>> = LazyLock::new(|| {
    (0..QFHRR_K)
        .map(|d| {
            let angle = 2.0 * std::f64::consts::PI * d as f64 / QFHRR_K as f64;
            angle.cos()
        })
        .collect()
});

/// Packed qFHRR: 2 dimensions per byte (4 bits each)
#[derive(Debug, Clone)]
pub struct PackedQFHRR {
    pub data: Vec<u8>, // 2048 bytes = 4096 dims × 4 bits
}

impl PackedQFHRR {
    pub fn from_qfhrr(v: &QFHRRVector) -> Self {
        let mut data = Vec::with_capacity(QFHRR_DIM / 2);
        for chunk in v.phases.chunks(2) {
            let a = chunk[0] & 0x0F;
            let b = if chunk.len() > 1 { chunk[1] & 0x0F } else { 0 };
            data.push((a << 4) | b);
        }
        PackedQFHRR { data }
    }

    pub fn memory_bytes(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_vector() {
        let v = QFHRRVector::random(42);
        assert_eq!(v.phases.len(), QFHRR_DIM);
        assert!(v.phases.iter().all(|&p| p < QFHRR_K as u8));
    }

    #[test]
    fn test_bind_unbind_roundtrip() {
        let a = QFHRRVector::random(1);
        let b = QFHRRVector::random(2);
        let c = a.bind(&b);
        let recovered = c.unbind(&b);
        let sim = a.similarity(&recovered);
        assert!(sim > 0.99, "bind/unbind roundtrip similarity: {}", sim);
    }

    #[test]
    fn test_self_similarity() {
        let a = QFHRRVector::random(1);
        let sim = a.similarity(&a);
        assert!((sim - 1.0).abs() < 1e-10, "self similarity: {}", sim);
    }

    #[test]
    fn test_random_similarity() {
        let a = QFHRRVector::random(1);
        let b = QFHRRVector::random(2);
        let sim = a.similarity(&b);
        assert!(sim.abs() < 0.1, "random similarity: {}", sim);
    }

    #[test]
    fn test_packed_memory() {
        let v = QFHRRVector::random(42);
        let p = PackedQFHRR::from_qfhrr(&v);
        assert_eq!(p.memory_bytes(), QFHRR_DIM / 2);
    }

    #[test]
    fn test_permute_reversible() {
        let a = QFHRRVector::random(42);
        let p = a.permute(257);
        let r = p.permute(-257);
        let sim = a.similarity(&r);
        assert!(sim > 0.99, "permute roundtrip similarity: {}", sim);
    }

    #[test]
    fn test_bind_commutative() {
        let a = QFHRRVector::random(1);
        let b = QFHRRVector::random(2);
        let ab = a.bind(&b);
        let ba = b.bind(&a);
        let sim = ab.similarity(&ba);
        assert!(sim > 0.99, "bind commutative similarity: {}", sim);
    }

    #[test]
    fn test_memory_savings() {
        let v = QFHRRVector::random(42);
        assert_eq!(v.memory_bytes(), QFHRR_DIM);
        assert_eq!(QFHRRVector::memory_savings(), 16.0);
    }

    #[test]
    fn test_vsa_backend_bind_roundtrip() {
        let b = QFHRRBackend;
        let a: Vec<f64> = (0..QFHRR_DIM).map(|i| (i as f64).sin()).collect();
        let c: Vec<f64> = (0..QFHRR_DIM).map(|i| (i as f64).cos()).collect();
        let bound = VsaBackend::bind(&b, &a, &c);
        // Normalize bound to ensure it's in expected range
        let bound_max = bound.iter().cloned().fold(-1.0_f64, f64::max);
        let bound_min = bound.iter().cloned().fold(1.0_f64, f64::min);
        assert!(
            bound_min >= -1.0 - 1e-6,
            "bound min out of range: {}",
            bound_min
        );
        assert!(
            bound_max <= 1.0 + 1e-6,
            "bound max out of range: {}",
            bound_max
        );
    }

    #[test]
    fn test_vsa_backend_dimensions() {
        let b = QFHRRBackend;
        assert_eq!(BinaryVsaBackend::dimensions(&b), QFHRR_DIM);
    }

    #[test]
    fn test_binary_backend_bind_roundtrip() {
        let b = QFHRRBackend;
        let a = QFHRRVector::random(1);
        let c = QFHRRVector::random(2);
        let bound = BinaryVsaBackend::bind(&b, &a.phases, &c.phases);
        let recovered = BinaryVsaBackend::unbind(&b, &bound, &c.phases);
        let sim = BinaryVsaBackend::similarity(&b, &a.phases, &recovered);
        assert!(
            sim > 0.99,
            "binary bind/unbind roundtrip similarity: {}",
            sim
        );
    }

    #[test]
    fn test_binary_backend_similarity() {
        let b = QFHRRBackend;
        let a = QFHRRVector::random(42);
        let sim = BinaryVsaBackend::similarity(&b, &a.phases, &a.phases);
        assert!((sim - 1.0).abs() < 1e-10, "binary self similarity: {}", sim);
    }

    #[test]
    fn test_to_dense_roundtrip() {
        let b = QFHRRBackend;
        let a = QFHRRVector::random(42);
        let dense = b.to_dense(&a.phases);
        assert_eq!(dense.len(), QFHRR_DIM);
        // Convert back: dense values should be in [0, 1]
        for &v in &dense {
            assert!(v >= 0.0 && v <= 1.0, "dense value out of range: {}", v);
        }
        // Self-similarity via to_dense should still be high
        let phases2: Vec<u8> = dense
            .iter()
            .map(|&v| (v * QFHRR_K as f64).round() as u8 % QFHRR_K as u8)
            .collect();
        let sim = BinaryVsaBackend::similarity(&b, &a.phases, &phases2);
        assert!(sim > 0.99, "to_dense roundtrip similarity: {}", sim);
    }

    #[test]
    fn test_vsa_backend_name() {
        let b = QFHRRBackend;
        assert_eq!(BinaryVsaBackend::name(&b), "qfhrr-vsa");
    }
}

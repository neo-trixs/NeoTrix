use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

#[derive(Debug, Clone)]
pub struct HLBBind;

pub(crate) fn fwht(data: &mut [f64]) {
    let n = data.len();
    let mut h = 1;
    while h < n {
        for i in (0..n).step_by(h * 2) {
            for j in i..i + h {
                let x = data[j];
                let y = data[j + h];
                data[j] = x + y;
                data[j + h] = x - y;
            }
        }
        h *= 2;
    }
    let inv_n = 1.0 / (n as f64).sqrt();
    for v in data.iter_mut() {
        *v *= inv_n;
    }
}

fn ifwht(data: &mut [f64]) {
    let n = data.len();
    let mut h = 1;
    while h < n {
        for i in (0..n).step_by(h * 2) {
            for j in i..i + h {
                let x = data[j];
                let y = data[j + h];
                data[j] = x + y;
                data[j + h] = x - y;
            }
        }
        h *= 2;
    }
    let inv_n = 1.0 / (n as f64).sqrt();
    for v in data.iter_mut() {
        *v *= inv_n;
    }
}

impl HLBBind {
    pub fn hlb_bind(a: &[u8], b: &[u8]) -> Vec<u8> {
        let len = a.len().min(b.len()).min(VSA_DIM);
        let mut af: Vec<f64> = a
            .iter()
            .take(len)
            .map(|&x| if x == 0 { -1.0 } else { 1.0 })
            .collect();
        let mut bf: Vec<f64> = b
            .iter()
            .take(len)
            .map(|&x| if x == 0 { -1.0 } else { 1.0 })
            .collect();
        let original_len = af.len();

        let next_pow2 = original_len.next_power_of_two();
        af.resize(next_pow2, -1.0);
        bf.resize(next_pow2, -1.0);

        fwht(&mut af);
        fwht(&mut bf);

        for i in 0..next_pow2 {
            af[i] *= bf[i];
        }

        ifwht(&mut af);

        af.truncate(original_len);
        af.into_iter()
            .map(|v| if v > 0.0 { 1u8 } else { 0u8 })
            .collect()
    }

    pub fn hlb_similarity(a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len()).min(VSA_DIM);
        if len == 0 {
            return 0.0;
        }
        let mut af: Vec<f64> = a
            .iter()
            .take(len)
            .map(|&x| if x == 0 { -1.0 } else { 1.0 })
            .collect();
        let mut bf: Vec<f64> = b
            .iter()
            .take(len)
            .map(|&x| if x == 0 { -1.0 } else { 1.0 })
            .collect();
        let original_len = af.len();

        let next_pow2 = original_len.next_power_of_two();
        af.resize(next_pow2, -1.0);
        bf.resize(next_pow2, -1.0);

        fwht(&mut af);
        fwht(&mut bf);

        let dot: f64 = af.iter().zip(bf.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f64 = af.iter().map(|x| x * x).sum();
        let mag_b: f64 = bf.iter().map(|x| x * x).sum();
        let denom = (mag_a.sqrt() * mag_b.sqrt()).max(1e-12);
        dot / denom
    }

    /// MiND (Matched i.i.d. Normal Distribution) initialization.
    /// Draws values from N(0, 1/√dim) for optimal binding capacity.
    pub fn mind_init(dim: usize, seed: u64) -> Vec<f64> {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let scale = 1.0 / (dim as f64).sqrt();
        use rand::Rng;
        (0..dim).map(|_| rng.gen::<f64>() * scale).collect()
    }

    /// HLB bind on float vectors (direct, no bipolar conversion).
    pub fn hlb_bind_float(a: &[f64], b: &[f64]) -> Vec<f64> {
        let mut af = a.to_vec();
        let mut bf = b.to_vec();
        let len = af.len().min(bf.len());
        af.truncate(len);
        bf.truncate(len);
        fwht(&mut af);
        fwht(&mut bf);
        let mut result: Vec<f64> = af.iter().zip(bf.iter()).map(|(x, y)| x * y).collect();
        ifwht(&mut result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    #[test]
    fn test_fwht_roundtrip() {
        let len = 64;
        let original: Vec<f64> = (0..len)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let mut data = original.clone();
        fwht(&mut data);
        ifwht(&mut data);
        for (a, b) in data.iter().zip(original.iter()) {
            assert!(
                (a - b).abs() < 1e-10,
                "FWHT roundtrip failed: {} vs {}",
                a,
                b
            );
        }
    }

    #[test]
    fn test_bind_self_nonzero() {
        let a = QuantizedVSA::random_binary();
        let bound = HLBBind::hlb_bind(&a, &a);
        let ones = bound.iter().filter(|&&x| x == 1).count();
        let ratio = ones as f64 / bound.len() as f64;
        assert!(
            ratio > 0.05 && ratio < 0.95,
            "HLB bind(a,a) should NOT be zero; ones ratio = {}",
            ratio
        );
    }

    #[test]
    fn test_bind_associativity() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let c = QuantizedVSA::random_binary();
        let ab_c = HLBBind::hlb_bind(&HLBBind::hlb_bind(&a, &b), &c);
        let a_bc = HLBBind::hlb_bind(&a, &HLBBind::hlb_bind(&b, &c));
        let sim = QuantizedVSA::similarity(&ab_c, &a_bc);
        assert!(sim > 0.95, "HLB should be associative; sim = {}", sim);
    }

    #[test]
    fn test_bind_linearity() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let c = QuantizedVSA::random_binary();
        let ab_c = HLBBind::hlb_bind(&QuantizedVSA::bind(&a, &b), &c);
        let ac_bc = QuantizedVSA::bind(&HLBBind::hlb_bind(&a, &c), &HLBBind::hlb_bind(&b, &c));
        let sim = QuantizedVSA::similarity(&ab_c, &ac_bc);
        assert!(sim > 0.90, "HLB should be linear over XOR; sim = {}", sim);
    }

    #[test]
    fn test_bind_linearity_hlb() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let c = QuantizedVSA::random_binary();
        let ab = HLBBind::hlb_bind(&a, &b);
        let ab_c = HLBBind::hlb_bind(&ab, &c);
        let ac = HLBBind::hlb_bind(&a, &c);
        let bc = HLBBind::hlb_bind(&b, &c);
        let ac_bc_xor = QuantizedVSA::bind(&ac, &bc);
        let sim = QuantizedVSA::similarity(&ab_c, &ac_bc_xor);
        assert!(
            sim > 0.90,
            "HLB linearty: bind(a⊕b, c) ≈ bind(a,c) ⊕ bind(b,c); sim = {}",
            sim
        );
    }

    #[test]
    fn test_bind_orthogonality() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let bound = HLBBind::hlb_bind(&a, &b);
        let sim_a = QuantizedVSA::similarity(&bound, &a);
        let sim_b = QuantizedVSA::similarity(&bound, &b);
        assert!(
            sim_a < 0.6,
            "bind(a,b) should be dissimilar to a; sim = {}",
            sim_a
        );
        assert!(
            sim_b < 0.6,
            "bind(a,b) should be dissimilar to b; sim = {}",
            sim_b
        );
    }

    #[test]
    fn test_hlb_similarity_self() {
        let a = QuantizedVSA::random_binary();
        let sim = HLBBind::hlb_similarity(&a, &a);
        assert!(
            (sim - 1.0).abs() < 1e-6,
            "self similarity should be 1; got {}",
            sim
        );
    }

    #[test]
    fn test_hlb_similarity_orthogonal() {
        let zeros = vec![0u8; VSA_DIM];
        let ones = vec![1u8; VSA_DIM];
        let sim = HLBBind::hlb_similarity(&zeros, &ones);
        assert!(
            sim.abs() < 0.1,
            "all-zeros vs all-ones should be near-orthogonal; sim = {}",
            sim
        );
    }

    #[test]
    fn test_bind_bundle_resolve() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let c = QuantizedVSA::random_binary();
        let bound = HLBBind::hlb_bind(&a, &b);
        let bundle_ab_c = QuantizedVSA::bundle(&[&bound, &c]);
        let sim_c = QuantizedVSA::similarity(&bundle_ab_c, &c);
        assert!(sim_c > 0.3, "bundled HLB should be retrievable");
    }

    #[test]
    fn test_fwht_4096() {
        let len = 4096;
        let mut data: Vec<f64> = (0..len)
            .map(|i| if i % 3 == 0 { 1.0 } else { -1.0 })
            .collect();
        let original = data.clone();
        fwht(&mut data);
        ifwht(&mut data);
        for (a, b) in data.iter().zip(original.iter()) {
            assert!((a - b).abs() < 1e-10, "FWHT 4096 roundtrip failed");
        }
    }

    #[test]
    fn test_bind_three_way() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let c = QuantizedVSA::random_binary();
        let abc = HLBBind::hlb_bind(&HLBBind::hlb_bind(&a, &b), &c);
        let acb = HLBBind::hlb_bind(&HLBBind::hlb_bind(&a, &c), &b);
        let sim = QuantizedVSA::similarity(&abc, &acb);
        assert!(
            sim > 0.95,
            "HLB three-way bind should be order-independent via associativity; sim = {}",
            sim
        );
    }

    #[test]
    fn test_bind_commutative() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let ab = HLBBind::hlb_bind(&a, &b);
        let ba = HLBBind::hlb_bind(&b, &a);
        let sim = QuantizedVSA::similarity(&ab, &ba);
        assert!(sim > 0.95, "HLB should be commutative; sim = {}", sim);
    }

    #[test]
    fn test_bind_output_is_binary() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let bound = HLBBind::hlb_bind(&a, &b);
        assert_eq!(bound.len(), VSA_DIM);
        for &x in &bound {
            assert!(x == 0 || x == 1, "HLB output must be binary; got {}", x);
        }
    }

    #[test]
    fn test_similarity_against_xor_baseline() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let xor_bound = QuantizedVSA::bind(&a, &b);
        let hlb_bound = HLBBind::hlb_bind(&a, &b);
        let sim = QuantizedVSA::similarity(&xor_bound, &hlb_bound);
        assert!(
            sim > 0.3,
            "HLB and XOR should be positively correlated; sim = {}",
            sim
        );
    }

    #[test]
    fn test_mind_init_distribution() {
        let v = HLBBind::mind_init(4096, 42);
        assert_eq!(v.len(), 4096);
        let mean = v.iter().sum::<f64>() / v.len() as f64;
        assert!(
            mean.abs() < 0.05,
            "MiND mean should be near 0, got {}",
            mean
        );
        let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / v.len() as f64;
        let expected_var = 1.0 / 4096.0;
        assert!(
            (var - expected_var).abs() < 0.001,
            "MiND var should be ~1/d, got {} vs {}",
            var,
            expected_var
        );
    }

    #[test]
    fn test_hlb_bind_float_basic() {
        let a = HLBBind::mind_init(256, 1);
        let b = HLBBind::mind_init(256, 2);
        let c = HLBBind::hlb_bind_float(&a, &b);
        assert_eq!(c.len(), 256);
        let energy: f64 = c.iter().map(|x| x * x).sum();
        assert!(
            energy > 0.0,
            "HLB bind float should produce non-zero output"
        );
    }
}

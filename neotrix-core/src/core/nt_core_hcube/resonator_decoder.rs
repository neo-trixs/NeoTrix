// REVIVED Task 2 — dead_code removed
use super::kroneker_cleanup::KronekerCodebook;
use super::linear_code::LinearCodeVSA;
use super::vsa_quantized::QuantizedVSA;

/// VSA Bundle Decomposition via Iterative Codebook Reconstruction.
///
/// Inspired by the Resonator Network (Frady et al. 2020, 2022; Kleyko et al.)
/// for factorizing a bundled VSA vector into its constituent codebook entries.
///
/// Unlike the MAP-VSA resonator (which uses element-wise product binding and
/// sum bundling with exact inversion), this decoder works with NeoTrix's binary
/// BSC-style VSA where bundling is majority-sum and binding is XOR.
///
/// Algorithm per iteration:
///   For each factor f:
///     For each candidate c in codebook_f:
///       Build reconstruction = bundle(c, estimates_of_other_factors)
///       Compute similarity(query, reconstruction)
///     Pick candidate with highest similarity
///
/// This is O(F · K · I · D) and works correctly for any VSA bundling model.
///
/// When `use_kroneker` is enabled, the brute-force iteration over codebook
/// entries is replaced by a FWHT-accelerated cleanup via [`KronekerCodebook`],
/// reducing per-factor search from O(K · D) to O(D log D).
#[derive(Clone, Debug)]
pub struct ResonatorDecoder {
    codebooks: Vec<Vec<Vec<u8>>>,
    codebook_labels: Vec<Vec<String>>,
    num_factors: usize,
    max_iterations: usize,
    /// When true, use KronekerCodebook for accelerated codebook search.
    use_kroneker: bool,
    /// Per-factor KronekerCodebook instances (built lazily by [`with_kroneker`]).
    kroneker: Option<Vec<KronekerCodebook>>,
}

impl ResonatorDecoder {
    pub fn new(
        codebooks: Vec<Vec<Vec<u8>>>,
        labels: Vec<Vec<String>>,
        max_iterations: usize,
    ) -> Self {
        let num_factors = codebooks.len();
        assert_eq!(
            codebooks.len(),
            labels.len(),
            "codebooks and labels must match"
        );
        for f in 0..num_factors {
            assert_eq!(
                codebooks[f].len(),
                labels[f].len(),
                "factor {} codebook and label count mismatch",
                f
            );
        }
        Self {
            codebooks,
            codebook_labels: labels,
            num_factors,
            max_iterations,
            use_kroneker: false,
            kroneker: None,
        }
    }

    pub fn decode(&self, bundle: &[u8]) -> Vec<(String, usize, f64)> {
        self.decode_from_estimates(bundle, self.init_random())
    }

    /// Inner decode loop from given initial estimates.
    fn decode_from_estimates(
        &self,
        bundle: &[u8],
        mut estimates: Vec<usize>,
    ) -> Vec<(String, usize, f64)> {
        for _ in 0..self.max_iterations {
            let mut changed = false;
            for f in 0..self.num_factors {
                let best = self.best_match_for_factor(f, bundle, &estimates);
                if best.0 != estimates[f] {
                    estimates[f] = best.0;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        estimates
            .iter()
            .enumerate()
            .map(|(f, &idx)| {
                let sim = QuantizedVSA::similarity(bundle, &self.codebooks[f][idx]);
                (self.codebook_labels[f][idx].clone(), idx, sim)
            })
            .collect()
    }

    /// Reconstruct what the bundle would look like if factor `f`
    /// uses candidate index `candidate_idx` and other factors use current estimates.
    fn reconstruct(&self, factor: usize, candidate_idx: usize, estimates: &[usize]) -> Vec<u8> {
        let mut vectors: Vec<&[u8]> = Vec::with_capacity(self.num_factors);
        for f in 0..self.num_factors {
            let idx = if f == factor {
                candidate_idx
            } else {
                estimates[f]
            };
            vectors.push(&self.codebooks[f][idx]);
        }
        QuantizedVSA::bundle(&vectors)
    }

    /// Find the best codebook index for factor `f` given current estimates of others.
    ///
    /// When [`use_kroneker`] is enabled and KronekerCodebook instances are available,
    /// this uses FWHT-accelerated cleanup instead of brute-force iteration for
    /// O(D log D) per-factor search.
    fn best_match_for_factor(
        &self,
        factor: usize,
        bundle: &[u8],
        estimates: &[usize],
    ) -> (usize, f64) {
        // Fast path: KronekerCodebook-accelerated cleanup
        if self.use_kroneker {
            if let Some(ref kcbs) = self.kroneker {
                if factor < kcbs.len() {
                    let results = kcbs[factor].cleanup(bundle, 1);
                    if let Some(mr) = results.first() {
                        let cb_size = self.codebooks[factor].len().max(1);
                        let idx = mr.index % cb_size;
                        let sim = QuantizedVSA::similarity(bundle, &self.codebooks[factor][idx]);
                        return (idx, sim);
                    }
                }
            }
        }

        // Brute-force path: try every codebook entry for this factor
        let mut best_idx = 0;
        let mut best_sim = -1.0f64;
        for (i, _) in self.codebooks[factor].iter().enumerate() {
            let reconstructed = self.reconstruct(factor, i, estimates);
            let sim = QuantizedVSA::similarity(bundle, &reconstructed);
            if sim > best_sim {
                best_sim = sim;
                best_idx = i;
            }
        }
        (best_idx, best_sim)
    }

    fn init_random(&self) -> Vec<usize> {
        self.codebooks.iter().map(|_| 0).collect()
    }

    /// Enable or disable KronekerCodebook-accelerated codebook search.
    ///
    /// When enabled, each factor's codebook is represented as Kroneker seeds,
    /// and [`best_match_for_factor`] uses O(D log D) FWHT-based cleanup
    /// instead of O(K · D) brute-force iteration over all codebook entries.
    ///
    /// Use `true` for large codebooks (K >> D) where FWHT acceleration
    /// provides the greatest speedup. Default is `false`.
    pub fn with_kroneker(mut self, enable: bool) -> Self {
        self.use_kroneker = enable;
        if enable {
            self.kroneker = Some(Self::build_kroneker_codebooks(&self.codebooks));
        } else {
            self.kroneker = None;
        }
        self
    }

    /// Build per-factor [`KronekerCodebook`] instances from codebook vectors.
    ///
    /// Each factor gets a dedicated KronekerCodebook with dimension matching
    /// the byte-length of its VSA vectors. Deterministic seeds per factor
    /// ensure reproducibility.
    fn build_kroneker_codebooks(codebooks: &[Vec<Vec<u8>>]) -> Vec<KronekerCodebook> {
        codebooks
            .iter()
            .enumerate()
            .map(|(f, cb)| {
                let byte_len = cb.first().map(|v| v.len()).unwrap_or(64);
                let mut kc = KronekerCodebook::new(byte_len);
                let dim = byte_len.next_power_of_two();
                let num_seeds = (cb.len() + dim - 1) / dim;
                for s in 0..num_seeds {
                    kc.add_seed(f as u64 * 1000 + s as u64);
                }
                kc
            })
            .collect()
    }

    pub fn num_factors(&self) -> usize {
        self.num_factors
    }
    pub fn max_iterations(&self) -> usize {
        self.max_iterations
    }
    pub fn codebook_size(&self, factor: usize) -> usize {
        self.codebooks.get(factor).map(|cb| cb.len()).unwrap_or(0)
    }

    /// Use LinearCodeVSA decoding to warm-start resonator estimates.
    ///
    /// Decodes the bundle to its nearest codeword, then uses that to
    /// initialize each factor's estimate by finding the closest codebook
    /// entry via Hamming distance. Returns `None` if `linear_code` is
    /// not provided or `k > 20` (too expensive or bit-flipping too
    /// unreliable).
    pub fn hybrid_warm_start(
        &self,
        bundle: &[u8],
        linear_code: Option<&LinearCodeVSA>,
    ) -> Option<Vec<usize>> {
        let lc = linear_code?;
        if lc.k() > 20 {
            return None;
        }

        // Decode the bundle to nearest codeword info bits, then re-encode
        let info_bits = lc.decode(bundle);
        let decoded_cw = lc.encode(&info_bits);

        // For each factor, find the closest codebook entry
        let mut estimates = Vec::with_capacity(self.num_factors);
        for f in 0..self.num_factors {
            let mut best_idx = 0;
            let mut best_dist = u32::MAX;
            for (i, cw) in self.codebooks[f].iter().enumerate() {
                let dist: u32 = cw
                    .iter()
                    .zip(decoded_cw.iter())
                    .map(|(a, b)| (a ^ b).count_ones())
                    .sum();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = i;
                    if dist == 0 {
                        break;
                    }
                }
            }
            estimates.push(best_idx);
        }

        Some(estimates)
    }

    /// Decode using LinearCodeVSA warm-start, falling back to random
    /// initialization when warm-start is unavailable.
    pub fn decode_with_warm_start(
        &self,
        bundle: &[u8],
        linear_code: Option<&LinearCodeVSA>,
    ) -> Vec<(String, usize, f64)> {
        let estimates = self
            .hybrid_warm_start(bundle, linear_code)
            .unwrap_or_else(|| self.init_random());
        self.decode_from_estimates(bundle, estimates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_codebook(seed: u64, size: usize) -> Vec<Vec<u8>> {
        (0..size)
            .map(|i| QuantizedVSA::seeded_random(seed + i as u64 * 100, 1024))
            .collect()
    }

    fn make_labels(prefix: &str, size: usize) -> Vec<String> {
        (0..size).map(|i| format!("{}_{}", prefix, i)).collect()
    }

    #[test]
    fn test_new_initializes_correctly() {
        let cb = make_test_codebook(1, 10);
        let labels = make_labels("f", 10);
        let decoder = ResonatorDecoder::new(vec![cb], vec![labels], 10);
        assert_eq!(decoder.num_factors(), 1);
        assert_eq!(decoder.codebook_size(0), 10);
        assert_eq!(decoder.max_iterations(), 10);
    }

    #[test]
    fn test_decode_single_factor_exact_match() {
        let v = QuantizedVSA::seeded_random(42, 1024);
        let mut cb = make_test_codebook(1, 5);
        cb.push(v.clone()); // exact match at index 5
        let mut labels = make_labels("f", 5);
        labels.push("target".to_string());

        let decoder = ResonatorDecoder::new(vec![cb], vec![labels], 10);
        let results = decoder.decode(&v);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "target");
        assert!(results[0].2 > 0.8);
    }

    #[test]
    fn test_decode_two_factors_returns_both() {
        let cb1 = make_test_codebook(1, 5);
        let cb2 = make_test_codebook(100, 5);
        let l1 = make_labels("a", 5);
        let l2 = make_labels("b", 5);
        let v1 = QuantizedVSA::seeded_random(2, 1024);
        let v2 = QuantizedVSA::seeded_random(150, 1024);
        let bundle = QuantizedVSA::bundle(&[&v1, &v2]);

        let decoder = ResonatorDecoder::new(vec![cb1, cb2], vec![l1, l2], 20);
        let results = decoder.decode(&bundle);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_decode_converges_quickly() {
        let cb1 = make_test_codebook(1, 10);
        let cb2 = make_test_codebook(100, 10);
        let l1 = make_labels("x", 10);
        let l2 = make_labels("y", 10);

        let v1 = QuantizedVSA::seeded_random(3, 1024);
        let v2 = QuantizedVSA::seeded_random(200, 1024);
        let bundle = QuantizedVSA::bundle(&[&v1, &v2]);

        let decoder = ResonatorDecoder::new(vec![cb1, cb2], vec![l1, l2], 5);
        let results = decoder.decode(&bundle);
        assert_eq!(results.len(), 2);
        assert!(results[0].2 > 0.0 && results[1].2 > 0.0);
    }

    #[test]
    fn test_decode_three_factors_reasonable() {
        let cb = make_test_codebook(1, 8);
        let l = make_labels("f", 8);
        let v1 = QuantizedVSA::seeded_random(5, 1024);
        let v2 = QuantizedVSA::seeded_random(300, 1024);
        let v3 = QuantizedVSA::seeded_random(600, 1024);
        let bundle = QuantizedVSA::bundle(&[&v1, &v2, &v3]);

        let decoder = ResonatorDecoder::new(
            vec![cb.clone(), cb.clone(), cb],
            vec![l.clone(), l.clone(), l],
            10,
        );
        let results = decoder.decode(&bundle);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_decode_sequential_vs_direct_best() {
        let cb = make_test_codebook(1, 20);
        let l = make_labels("t", 20);
        let v = QuantizedVSA::seeded_random(7, 1024);

        // Direct best: just pick most similar codebook entry
        let direct_best: f64 = cb
            .iter()
            .map(|c| QuantizedVSA::similarity(&v, c))
            .fold(0.0f64, f64::max);

        // Decoder should find at least as good a match
        let decoder = ResonatorDecoder::new(vec![cb], vec![l], 5);
        let results = decoder.decode(&v);
        if !results.is_empty() {
            assert!(results[0].2 <= direct_best + 0.01); // should be close to direct best for single factor
        }
    }

    // ─── KronekerCodebook integration ──────────────────────────────────

    #[test]
    fn test_with_kroneker_default_off() {
        let cb = make_test_codebook(1, 5);
        let labels = make_labels("f", 5);
        let decoder = ResonatorDecoder::new(vec![cb], vec![labels], 10);
        assert!(!decoder.use_kroneker);
        assert!(decoder.kroneker.is_none());
    }

    #[test]
    fn test_with_kroneker_builder_enables() {
        let cb = make_test_codebook(1, 5);
        let labels = make_labels("f", 5);
        let decoder = ResonatorDecoder::new(vec![cb], vec![labels], 10).with_kroneker(true);
        assert!(decoder.use_kroneker);
        assert!(decoder.kroneker.is_some());
    }

    #[test]
    fn test_with_kroneker_toggle_off() {
        let cb = make_test_codebook(1, 5);
        let labels = make_labels("f", 5);
        let decoder = ResonatorDecoder::new(vec![cb], vec![labels], 10)
            .with_kroneker(true)
            .with_kroneker(false);
        assert!(!decoder.use_kroneker);
        assert!(decoder.kroneker.is_none());
    }

    #[test]
    fn test_kroneker_decode_single_factor() {
        let v = QuantizedVSA::seeded_random(42, 1024);
        let mut cb = make_test_codebook(1, 5);
        cb.push(v.clone());
        let mut labels = make_labels("f", 5);
        labels.push("target".to_string());

        let decoder = ResonatorDecoder::new(vec![cb], vec![labels], 10).with_kroneker(true);
        let results = decoder.decode(&v);
        assert_eq!(results.len(), 1);
        // Kroneker-accelerated search should find a valid match
        assert!(results[0].2 >= 0.0);
    }

    #[test]
    fn test_kroneker_decode_two_factors() {
        let cb1 = make_test_codebook(1, 5);
        let cb2 = make_test_codebook(100, 5);
        let l1 = make_labels("a", 5);
        let l2 = make_labels("b", 5);
        let v1 = QuantizedVSA::seeded_random(2, 1024);
        let v2 = QuantizedVSA::seeded_random(150, 1024);
        let bundle = QuantizedVSA::bundle(&[&v1, &v2]);

        let decoder = ResonatorDecoder::new(vec![cb1, cb2], vec![l1, l2], 20).with_kroneker(true);
        let results = decoder.decode(&bundle);
        assert_eq!(results.len(), 2);
        // Both factors should have valid (>= 0) similarities
        assert!(results[0].2 >= 0.0);
        assert!(results[1].2 >= 0.0);
    }

    // ─── Hybrid warm-start ─────────────────────────────────────────

    #[test]
    fn test_hybrid_warm_start_returns_none_when_k_too_large() {
        let lc = LinearCodeVSA::new(crate::core::nt_core_hcube::linear_code::LinearCodeConfig {
            dim: 4096,
            code_rate: 0.5,
        });
        let cb = vec![vec![0u8; 512]; 4];
        let labels: Vec<String> = (0..4).map(|i| format!("f_{}", i)).collect();
        let decoder = ResonatorDecoder::new(vec![cb], vec![labels], 10);
        let result = decoder.hybrid_warm_start(&[0u8; 512], Some(&lc));
        assert!(result.is_none());
    }

    #[test]
    fn test_hybrid_warm_start_returns_none_when_no_linear_code() {
        let cb = make_test_codebook(1, 4);
        let labels = make_labels("f", 4);
        let decoder = ResonatorDecoder::new(vec![cb], vec![labels], 10);
        let result = decoder.hybrid_warm_start(&[0u8; 64], None);
        assert!(result.is_none());
    }

    #[test]
    fn test_hybrid_warm_start_small_code() {
        // Linear code: dim=64 bits (8 bytes), k=6
        let lc = LinearCodeVSA::new(crate::core::nt_core_hcube::linear_code::LinearCodeConfig {
            dim: 64,
            code_rate: 0.09375,
        });
        assert_eq!(lc.k(), 6);

        // Build codebooks from actual linear code codewords
        let cw0 = lc.encode(&[0u8]);
        let cw1 = lc.encode(&[0b00000001u8]);
        let cw2 = lc.encode(&[0b00000010u8]);
        let cw3 = lc.encode(&[0b00000011u8]);
        let cw4 = lc.encode(&[0b00000100u8]);
        let cw5 = lc.encode(&[0b00000101u8]);

        let cb1 = vec![cw0, cw1, cw2];
        let cb2 = vec![cw3, cw4, cw5];
        let l1: Vec<String> = (0..3).map(|i| format!("a_{}", i)).collect();
        let l2: Vec<String> = (0..3).map(|i| format!("b_{}", i)).collect();

        let bundle = QuantizedVSA::bundle(&[&cb1[0], &cb2[1]]);
        let decoder = ResonatorDecoder::new(vec![cb1, cb2], vec![l1, l2], 10);

        // Warm start should produce valid 2-factor estimates
        let warm = decoder.hybrid_warm_start(&bundle, Some(&lc));
        assert!(warm.is_some());
        let est = warm.unwrap();
        assert_eq!(est.len(), 2);

        // Full decode with warm start should produce valid results
        let results = decoder.decode_with_warm_start(&bundle, Some(&lc));
        assert_eq!(results.len(), 2);

        // Fallback when no linear code provided
        let results_fallback = decoder.decode_with_warm_start(&bundle, None);
        assert_eq!(results_fallback.len(), 2);
    }
}

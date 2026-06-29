use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

/// Element-wise amplify via thresholding.
/// For bipolar binary VSA: uses a threshold-based "amplification" where
/// values below threshold are weakened.
pub fn amplify(a: &[u8], p: f64) -> Vec<u8> {
    let thresh = (1.0 - p.clamp(0.0, 1.0)) * 128.0;
    let mut result = a.to_vec();
    for x in &mut result {
        if (*x as f64) < thresh && *x > 0 {
            *x = 0;
        }
    }
    result
}

/// Blend: weighted majority between two vectors.
/// t=0 returns a, t=1 returns b.
pub fn blend(a: &[u8], b: &[u8], t: f64) -> Vec<u8> {
    let threshold = (t * 255.0) as u8;
    let mut result = Vec::with_capacity(VSA_DIM);
    for (x, y) in a.iter().zip(b.iter()) {
        let v = if *x > threshold { *x } else { *y };
        result.push(v);
    }
    result
}

/// Attend: soft attention over a set of items weighted by similarity to query.
/// Returns the best-matching item (winner-take-all for binary VSA).
pub fn attend(query: &[u8], items: &[&[u8]]) -> Vec<u8> {
    if items.is_empty() {
        return vec![0; VSA_DIM];
    }
    let best = items
        .iter()
        .map(|item| QuantizedVSA::similarity(query, item))
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    items[best].to_vec()
}

/// Analogy: pattern transfer A:B::C:? → returns D such that D ⊗ C ≈ B ⊗ A.
/// In XOR binding VSA: D = B ⊕ A ⊕ C (since bind is self-inverse).
pub fn analogy(a: &[u8], b: &[u8], c: &[u8]) -> Vec<u8> {
    let ba = QuantizedVSA::bind(b, a);
    QuantizedVSA::bind(&ba, c)
}

/// Resonance: iterative cleanup by finding nearest codebook neighbor.
pub fn resonance(vec: &[u8], codebook: &[&[u8]], steps: usize) -> Vec<u8> {
    if codebook.is_empty() {
        return vec![0; VSA_DIM];
    }
    let mut result = vec.to_vec();
    for _ in 0..steps {
        let best = codebook
            .iter()
            .map(|c| QuantizedVSA::similarity(&result, c))
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i);
        if let Some(idx) = best {
            let sim = QuantizedVSA::similarity(&result, codebook[idx]);
            if sim > 0.0 {
                result = codebook[idx].to_vec();
            }
        }
    }
    result
}

/// Project: not directly meaningful in binary VSA; delegates to resonance.
pub fn project(vec: &[u8], basis: &[&[u8]]) -> Vec<u8> {
    resonance(vec, basis, 3)
}

/// Conditional bind: if condition > threshold, bind(a, b) else bind(negate(a), b).
pub fn conditional_bind(a: &[u8], b: &[u8], condition: f64, threshold: f64) -> Vec<u8> {
    if condition > threshold {
        QuantizedVSA::bind(a, b)
    } else {
        QuantizedVSA::bind(&QuantizedVSA::negate(a), b)
    }
}

/// Difference: Hamming distance between two vectors as normalized float.
pub fn difference(a: &[u8], b: &[u8]) -> f64 {
    let hd = QuantizedVSA::hamming_distance(a, b);
    hd as f64 / VSA_DIM as f64
}

/// Complexity: entropy of byte distribution over 16 bins.
pub fn complexity(vec: &[u8]) -> f64 {
    let n_bins = 16usize;
    let mut hist = vec![0usize; n_bins];
    for &x in vec.iter() {
        let bin = (x as usize) * n_bins / 256;
        let bin = bin.min(n_bins - 1);
        hist[bin] += 1;
    }
    let total = vec.len() as f64;
    let entropy: f64 = hist
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / total;
            -p * p.log2()
        })
        .sum();
    entropy
}

/// Segment: resonator-style decomposition of composite into codebook parts.
pub fn segment(composite: &[u8], codebook: &[&[u8]], max_steps: usize) -> Vec<Vec<u8>> {
    if codebook.is_empty() {
        return Vec::new();
    }
    let mut remaining = composite.to_vec();
    let mut found: Vec<Vec<u8>> = Vec::new();
    let mut used = vec![false; codebook.len()];

    for _ in 0..max_steps {
        let mut best_idx = None;
        let mut best_sim = f64::NEG_INFINITY;
        for (i, item) in codebook.iter().enumerate() {
            if used[i] {
                continue;
            }
            let sim = QuantizedVSA::similarity(&remaining, item);
            if sim > best_sim {
                best_sim = sim;
                best_idx = Some(i);
            }
        }
        match best_idx {
            None => break,
            Some(idx) => {
                if best_sim < 0.05 {
                    break;
                }
                found.push(codebook[idx].to_vec());
                used[idx] = true;
                remaining = QuantizedVSA::bind(&remaining, &QuantizedVSA::negate(codebook[idx]));
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_vector() -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..VSA_DIM).map(|_| rng.gen()).collect()
    }

    fn constant_vector(value: u8) -> Vec<u8> {
        vec![value; VSA_DIM]
    }

    #[test]
    fn test_amplify_preserves_structure() {
        let v = random_vector();
        let p1 = amplify(&v, 1.0);
        let p0 = amplify(&v, 0.0);
        assert_eq!(v.len(), p1.len());
        assert_eq!(v.len(), p0.len());
    }

    #[test]
    fn test_blend_extremes() {
        let a = constant_vector(0);
        let b = constant_vector(255);
        let at_zero = blend(&a, &b, 0.0);
        let at_one = blend(&a, &b, 1.0);
        assert_eq!(at_zero, a);
        assert_eq!(at_one, b);
    }

    #[test]
    fn test_attend_selects_best_match() {
        let query = constant_vector(128);
        let item_a = constant_vector(128);
        let item_b = constant_vector(0);
        let result = attend(&query, &[item_a.as_slice(), item_b.as_slice()]);
        assert_eq!(result, item_a);
    }

    #[test]
    fn test_analogy_roundtrip() {
        let a = random_vector();
        let b = random_vector();
        let c = random_vector();
        let d = analogy(&a, &b, &c);
        let check = QuantizedVSA::bind(&d, &c);
        let target = QuantizedVSA::bind(&b, &a);
        let sim = QuantizedVSA::similarity(&check, &target);
        assert!(
            sim.abs() > 0.9,
            "bind(D, C) should closely match bind(B, A); sim = {}",
            sim
        );
    }

    #[test]
    fn test_difference_symmetric() {
        let a = random_vector();
        let b = random_vector();
        let d1 = difference(&a, &b);
        let d2 = difference(&b, &a);
        assert!((d1 - d2).abs() < 1e-10, "difference must be symmetric");
    }

    #[test]
    fn test_conditional_bind_selects_correct_branch() {
        let a = random_vector();
        let b = random_vector();
        let pos = conditional_bind(&a, &b, 1.0, 0.0);
        let neg = conditional_bind(&a, &b, -1.0, 0.0);
        let direct = QuantizedVSA::bind(&a, &b);
        let negated = QuantizedVSA::bind(&QuantizedVSA::negate(&a), &b);
        assert!(
            QuantizedVSA::similarity(&pos, &direct) > 0.99,
            "positive condition should match direct bind"
        );
        assert!(
            QuantizedVSA::similarity(&neg, &negated) > 0.99,
            "negative condition should match negated bind"
        );
    }

    #[test]
    fn test_complexity_range() {
        let uniform = constant_vector(0);
        let random = random_vector();
        let c_uniform = complexity(&uniform);
        let c_random = complexity(&random);
        assert!(
            c_uniform < 0.1,
            "uniform vector should have near-zero entropy"
        );
        assert!(
            c_random > 1.0,
            "random vector should have significant entropy"
        );
    }

    #[test]
    fn test_resonance_converges() {
        let target = random_vector();
        let noisy = QuantizedVSA::bundle(&[target.as_slice(), &constant_vector(0)]);
        let cleaned = resonance(&noisy, &[target.as_slice()], 3);
        let sim = QuantizedVSA::similarity(&cleaned, &target);
        assert!(sim > 0.0, "resonance should find codebook entry");
    }

    #[test]
    fn test_segment_returns_constituents() {
        let a = random_vector();
        let b = random_vector();
        let composite = QuantizedVSA::bundle(&[a.as_slice(), b.as_slice()]);
        let parts = segment(&composite, &[a.as_slice(), b.as_slice()], 5);
        assert!(!parts.is_empty(), "segment should find constituents");
    }
}

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncodingMode {
    Correlated,
    Orthogonal,
}

#[derive(Debug, Clone)]
pub struct AdaptiveVsaEncoder {
    dim: usize,
    base_seed: u64,
    kernel_width: usize,
}

impl Default for AdaptiveVsaEncoder {
    fn default() -> Self {
        Self::new(VSA_DIM, 42, VSA_DIM)
    }
}

impl AdaptiveVsaEncoder {
    pub fn new(dim: usize, base_seed: u64, kernel_width: usize) -> Self {
        let kw = kernel_width.clamp(1, dim);
        Self {
            dim,
            base_seed,
            kernel_width: kw,
        }
    }

    pub fn with_mode(dim: usize, base_seed: u64, mode: EncodingMode) -> Self {
        let kw = match mode {
            EncodingMode::Correlated => dim / 16,
            EncodingMode::Orthogonal => dim,
        };
        Self::new(dim, base_seed, kw)
    }

    pub fn kernel_width(&self) -> usize {
        self.kernel_width
    }

    pub fn set_kernel_width(&mut self, kw: usize) {
        self.kernel_width = kw.clamp(1, self.dim);
    }

    pub fn set_mode(&mut self, mode: EncodingMode) {
        match mode {
            EncodingMode::Correlated => self.kernel_width = self.dim / 16,
            EncodingMode::Orthogonal => self.kernel_width = self.dim,
        }
    }

    pub fn encode(&self, text: &str, mode: EncodingMode) -> Vec<u8> {
        let effective_kw = match mode {
            EncodingMode::Correlated => self.dim / 16,
            EncodingMode::Orthogonal => self.dim,
        };
        self.encode_with_kernel(text, effective_kw)
    }

    pub fn encode_adaptive(&self, text: &str, is_learning_task: bool) -> Vec<u8> {
        if is_learning_task {
            self.encode(text, EncodingMode::Correlated)
        } else {
            self.encode(text, EncodingMode::Orthogonal)
        }
    }

    fn encode_with_kernel(&self, text: &str, kernel_width: usize) -> Vec<u8> {
        let words: Vec<&str> = text.split_whitespace().filter(|w| !w.is_empty()).collect();
        if words.is_empty() {
            return vec![0; self.dim];
        }

        if kernel_width >= self.dim {
            self.encode_orthogonal(&words)
        } else {
            self.encode_correlated(&words, kernel_width)
        }
    }

    fn encode_orthogonal(&self, words: &[&str]) -> Vec<u8> {
        let mut accum = vec![0u8; self.dim];
        let mut count = 0usize;
        for (i, word) in words.iter().enumerate() {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            word.hash(&mut h);
            let seed = h.finish();
            let mut v = QuantizedVSA::seeded_random(seed.wrapping_add(self.base_seed), self.dim);
            if i > 0 {
                v = QuantizedVSA::permute(&v, i as isize);
            }
            for (j, &b) in v.iter().enumerate() {
                accum[j] = accum[j].saturating_add(b);
            }
            count += 1;
        }
        let threshold = (count as u16 + 1) / 2;
        accum
            .iter_mut()
            .for_each(|x| *x = if *x as u16 >= threshold { 1 } else { 0 });
        accum
    }

    fn encode_correlated(&self, words: &[&str], kernel_width: usize) -> Vec<u8> {
        let mut accum = vec![0u8; self.dim];
        let mut count = 0usize;
        let half = (kernel_width / 2) as isize;
        for (i, word) in words.iter().enumerate() {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            word.hash(&mut h);
            let seed = h.finish();

            let pos = (seed.wrapping_add(self.base_seed) as usize) % self.dim;
            let start = (pos as isize - half).rem_euclid(self.dim as isize) as usize;
            let end = (pos as isize + half).rem_euclid(self.dim as isize) as usize;

            if start < end {
                for j in start..end {
                    accum[j] = accum[j].saturating_add(1);
                }
            } else {
                for j in start..self.dim {
                    accum[j] = accum[j].saturating_add(1);
                }
                for j in 0..end {
                    accum[j] = accum[j].saturating_add(1);
                }
            }

            if i > 0 {
                accum = QuantizedVSA::permute(&accum, i as isize);
            }
            count += 1;
        }
        let threshold = (count as u16 + 1) / 2;
        accum
            .iter_mut()
            .for_each(|x| *x = if *x as u16 >= threshold { 1 } else { 0 });
        accum
    }

    pub fn encode_with_tag(&self, text: &str, tag: &str) -> Vec<u8> {
        let is_learning = matches!(
            tag,
            "learning" | "classification" | "clustering" | "pattern_recognition"
        );
        self.encode_adaptive(text, is_learning)
    }

    pub fn similarity(a: &[u8], b: &[u8]) -> f64 {
        QuantizedVSA::similarity(a, b)
    }

    pub fn dim(&self) -> usize {
        self.dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoder() -> AdaptiveVsaEncoder {
        AdaptiveVsaEncoder::new(VSA_DIM, 42, VSA_DIM)
    }

    #[test]
    fn test_orthogonal_deterministic() {
        let e = encoder();
        let v1 = e.encode("hello world", EncodingMode::Orthogonal);
        let v2 = e.encode("hello world", EncodingMode::Orthogonal);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_correlated_deterministic() {
        let e = encoder();
        let v1 = e.encode("hello world", EncodingMode::Correlated);
        let v2 = e.encode("hello world", EncodingMode::Correlated);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_correlated_and_orthogonal_differ() {
        let e = encoder();
        let v_corr = e.encode("test sentence", EncodingMode::Correlated);
        let v_orth = e.encode("test sentence", EncodingMode::Orthogonal);
        assert_ne!(v_corr, v_orth, "modes must produce different encodings");
    }

    #[test]
    fn test_correlated_similar_inputs() {
        let e = encoder();
        let v1 = e.encode("cat sat on mat", EncodingMode::Correlated);
        let v2 = e.encode("cat sat on rug", EncodingMode::Correlated);
        let v3 = e.encode("quantum superposition", EncodingMode::Correlated);
        let sim_similar = AdaptiveVsaEncoder::similarity(&v1, &v2);
        let sim_different = AdaptiveVsaEncoder::similarity(&v1, &v3);
        assert!(
            sim_similar > sim_different,
            "correlated mode: similar inputs should be more similar ({} > {})",
            sim_similar,
            sim_different
        );
    }

    #[test]
    fn test_orthogonal_lower_similarity() {
        let e = encoder();
        let v1 = e.encode("cat sat on mat", EncodingMode::Orthogonal);
        let v2 = e.encode("cat sat on rug", EncodingMode::Orthogonal);
        let v3 = e.encode("quantum superposition", EncodingMode::Orthogonal);
        let sim_similar = AdaptiveVsaEncoder::similarity(&v1, &v2);
        let sim_different = AdaptiveVsaEncoder::similarity(&v1, &v3);
        assert!(
            sim_similar > sim_different,
            "orthogonal mode: similar inputs should still be more similar than different ones"
        );
    }

    #[test]
    fn test_kernel_width_bounds() {
        let mut e = encoder();
        e.set_kernel_width(0);
        assert_eq!(e.kernel_width(), 1);
        e.set_kernel_width(10000);
        assert_eq!(e.kernel_width(), VSA_DIM);
    }

    #[test]
    fn test_adaptive_learning_vs_cognitive() {
        let e = encoder();
        let v_learning = e.encode_adaptive("classify this pattern", true);
        let v_cognitive = e.encode_adaptive("classify this pattern", false);
        assert_ne!(v_learning, v_cognitive);
    }

    #[test]
    fn test_encode_with_tag() {
        let e = encoder();
        let v_learning = e.encode_with_tag("find patterns in data", "classification");
        let v_cog = e.encode_with_tag("find patterns in data", "reasoning");
        assert_ne!(v_learning, v_cog);
    }

    #[test]
    fn test_empty_text() {
        let e = encoder();
        let v = e.encode("", EncodingMode::Orthogonal);
        assert_eq!(v.len(), VSA_DIM);
        assert!(v.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_mode_switching() {
        let mut e = encoder();
        e.set_mode(EncodingMode::Correlated);
        assert!(e.kernel_width() < VSA_DIM);
        e.set_mode(EncodingMode::Orthogonal);
        assert_eq!(e.kernel_width(), VSA_DIM);
    }

    #[test]
    fn test_with_mode_constructor() {
        let e_corr = AdaptiveVsaEncoder::with_mode(VSA_DIM, 42, EncodingMode::Correlated);
        let e_orth = AdaptiveVsaEncoder::with_mode(VSA_DIM, 42, EncodingMode::Orthogonal);
        assert!(e_corr.kernel_width() < e_orth.kernel_width());
    }

    #[test]
    fn test_correlated_more_overlap() {
        let e_corr = AdaptiveVsaEncoder::with_mode(VSA_DIM, 42, EncodingMode::Correlated);
        let e_orth = AdaptiveVsaEncoder::with_mode(VSA_DIM, 42, EncodingMode::Orthogonal);
        let a = "apple banana cherry";
        let b = "apple banana date";
        let sim_corr = AdaptiveVsaEncoder::similarity(
            &e_corr.encode(a, EncodingMode::Correlated),
            &e_corr.encode(b, EncodingMode::Correlated),
        );
        let sim_orth = AdaptiveVsaEncoder::similarity(
            &e_orth.encode(a, EncodingMode::Orthogonal),
            &e_orth.encode(b, EncodingMode::Orthogonal),
        );
        assert!(
            sim_corr > sim_orth,
            "correlated mode should yield higher similarity for similar inputs (corr={}, orth={})",
            sim_corr,
            sim_orth
        );
    }
}

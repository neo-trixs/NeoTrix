#![allow(dead_code)]
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::collections::HashMap;

const VSA_DIM: usize = 4096;
const VSA_WORDS: usize = VSA_DIM / 64;

fn random_vsa_vector(rng: &mut StdRng) -> [u64; VSA_WORDS] {
    let mut vec = [0u64; VSA_WORDS];
    for w in &mut vec {
        *w = rng.gen();
    }
    vec
}

pub struct ClassicalVsaEncoder {
    vsa_dim: usize,
    ngram_n: usize,
    seed: u64,
    char_codebook: HashMap<char, [u64; VSA_WORDS]>,
    role_vectors: Vec<[u64; VSA_WORDS]>,
}

impl ClassicalVsaEncoder {
    pub fn new(vsa_dim: usize, ngram_n: usize, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(1));
        let role_vectors = (0..ngram_n).map(|_| random_vsa_vector(&mut rng)).collect();
        Self {
            vsa_dim,
            ngram_n,
            seed,
            char_codebook: HashMap::new(),
            role_vectors,
        }
    }

    pub fn build_char_codebook(&mut self, chars: &[char]) {
        let mut rng = StdRng::seed_from_u64(self.seed.wrapping_add(2));
        for &c in chars {
            self.char_codebook
                .entry(c)
                .or_insert_with(|| random_vsa_vector(&mut rng));
        }
    }

    fn get_or_create_char(&mut self, c: char) -> [u64; VSA_WORDS] {
        let seed = self.seed;
        *self.char_codebook.entry(c).or_insert_with(|| {
            let mut rng = StdRng::seed_from_u64(seed.wrapping_add(3).wrapping_add(c as u64));
            random_vsa_vector(&mut rng)
        })
    }

    fn clean_text(text: &str) -> String {
        text.chars().filter(|c| !c.is_whitespace()).collect()
    }

    fn xor_vectors(a: &[u64; VSA_WORDS], b: &[u64; VSA_WORDS]) -> [u64; VSA_WORDS] {
        let mut result = [0u64; VSA_WORDS];
        for i in 0..VSA_WORDS {
            result[i] = a[i] ^ b[i];
        }
        result
    }

    fn bundle(vectors: &[[u64; VSA_WORDS]]) -> [u64; VSA_WORDS] {
        let n = vectors.len();
        if n == 0 {
            return [0u64; VSA_WORDS];
        }
        let half = (n / 2) as i32;
        let mut counts = [0i32; VSA_DIM];
        for vec in vectors {
            for (w, &word) in vec.iter().enumerate() {
                for b in 0..64 {
                    if (word >> b) & 1 == 1 {
                        counts[w * 64 + b] += 1;
                    }
                }
            }
        }
        let mut result = [0u64; VSA_WORDS];
        for i in 0..VSA_DIM {
            if counts[i] > half {
                result[i / 64] |= 1u64 << (i % 64);
            }
        }
        result
    }

    pub fn encode(&mut self, text: &str) -> Vec<u64> {
        let cleaned = Self::clean_text(text);
        if cleaned.is_empty() {
            return vec![0u64; VSA_WORDS];
        }
        let chars: Vec<char> = cleaned.chars().collect();
        let mut phrase_vectors = Vec::new();
        if chars.len() < self.ngram_n {
            let mut bound_chars = Vec::with_capacity(chars.len());
            for (i, &c) in chars.iter().enumerate() {
                let cv = self.get_or_create_char(c);
                bound_chars.push(Self::xor_vectors(&cv, &self.role_vectors[i]));
            }
            phrase_vectors.push(Self::bundle(&bound_chars));
        } else {
            for ng in chars.windows(self.ngram_n) {
                let mut bound_chars = Vec::with_capacity(self.ngram_n);
                for (i, &c) in ng.iter().enumerate() {
                    let cv = self.get_or_create_char(c);
                    bound_chars.push(Self::xor_vectors(&cv, &self.role_vectors[i]));
                }
                phrase_vectors.push(Self::bundle(&bound_chars));
            }
        }
        let doc_vec = Self::bundle(&phrase_vectors);
        doc_vec.to_vec()
    }

    pub fn similarity(a: &[u64], b: &[u64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let total_bits = (a.len() * 64) as f64;
        let mut hamming = 0u64;
        for (wa, wb) in a.iter().zip(b.iter()) {
            hamming += (wa ^ wb).count_ones() as u64;
        }
        1.0 - 2.0 * hamming as f64 / total_bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_encoder() -> ClassicalVsaEncoder {
        ClassicalVsaEncoder::new(VSA_DIM, 3, 42)
    }

    #[test]
    fn test_similar_texts_high_similarity() {
        let mut e = init_encoder();
        let a = e.encode("子曰学而时习之");
        let b = e.encode("子曰学而时习之不亦说乎");
        let sim = ClassicalVsaEncoder::similarity(&a, &b);
        assert!(sim > 0.6, "similarity {} should be > 0.6", sim);
    }

    #[test]
    fn test_different_texts_low_similarity() {
        let mut e = init_encoder();
        let a = e.encode("天地玄黄宇宙洪荒");
        let b = e.encode("人之初性本善");
        let sim = ClassicalVsaEncoder::similarity(&a, &b);
        assert!(sim < 0.4, "similarity {} should be < 0.4", sim);
    }

    #[test]
    fn test_empty_text_returns_zero_vector() {
        let mut e = init_encoder();
        let result = e.encode("");
        assert_eq!(result.len(), VSA_WORDS);
        assert!(
            result.iter().all(|&w| w == 0),
            "empty text should produce zero vector"
        );
    }

    #[test]
    fn test_ngram_permutation_changes_vector() {
        let mut e = init_encoder();
        let original = e.encode("学而时习之");
        let permuted = e.encode("时习之学而");
        let sim = ClassicalVsaEncoder::similarity(&original, &permuted);
        assert!(
            sim < 0.9,
            "permuted text similarity {} should be < 0.9",
            sim
        );
    }

    #[test]
    fn test_identical_texts_max_similarity() {
        let mut e = init_encoder();
        let a = e.encode("有朋自远方来");
        let b = e.encode("有朋自远方来");
        let sim = ClassicalVsaEncoder::similarity(&a, &b);
        assert!(
            (sim - 1.0).abs() < 1e-10,
            "identical texts should have similarity 1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_build_char_codebook_prepopulates() {
        let mut e = init_encoder();
        assert!(e.char_codebook.is_empty());
        let chars: Vec<char> = "天地玄黄宇宙洪荒".chars().collect();
        e.build_char_codebook(&chars);
        assert_eq!(e.char_codebook.len(), 8);
    }
}

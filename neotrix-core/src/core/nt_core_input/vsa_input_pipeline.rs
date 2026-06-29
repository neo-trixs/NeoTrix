use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Semantic type of input for VSA encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSemanticType {
    UserQuery,
    SystemResponse,
    ToolOutput,
    DecisionChoice,
    KnowledgeRetrieval,
    ErrorSignal,
    CuriositySignal,
    ReflectionNote,
    ExplorationResult,
}

impl InputSemanticType {
    /// Each type gets a unique basis vector seed
    pub fn basis_seed(&self) -> u64 {
        match self {
            InputSemanticType::UserQuery => 1001,
            InputSemanticType::SystemResponse => 2001,
            InputSemanticType::ToolOutput => 3001,
            InputSemanticType::DecisionChoice => 4001,
            InputSemanticType::KnowledgeRetrieval => 5001,
            InputSemanticType::ErrorSignal => 6001,
            InputSemanticType::CuriositySignal => 7001,
            InputSemanticType::ReflectionNote => 8001,
            InputSemanticType::ExplorationResult => 9001,
        }
    }
}

/// A properly VSA-encoded piece of input
#[derive(Debug, Clone)]
pub struct VsaInput {
    pub content_hash: u64,
    pub content: String,
    pub vector: Vec<u8>,
    pub semantic_type: InputSemanticType,
    pub importance: f64,
    pub coherence: f64,
}

/// N-gram based VSA encoder for semantic text-to-VSA encoding.
///
/// Encodes text by:
/// 1. Splitting into overlapping character n-grams
/// 2. Hashing each n-gram → deterministic VSA seed → 4096-byte VSA
/// 3. XOR-binding each n-gram VSA to its position vector
/// 4. Majority-vote bundling all position-bound n-gram vectors
///
/// This produces VSA vectors where similar texts have similar vectors
/// because they share overlapping n-grams.
#[derive(Debug, Clone)]
pub struct NgramVsaEncoder {
    dim: usize,
    ngram_size: usize,
    position_basis: Vec<Vec<u8>>,
}

impl NgramVsaEncoder {
    pub fn new(dim: usize, ngram_size: usize) -> Self {
        Self {
            dim,
            ngram_size,
            position_basis: Vec::new(),
        }
    }

    fn hash_ngram(ngram: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        ngram.hash(&mut hasher);
        hasher.finish()
    }

    /// Ensure position vectors exist up to (and including) the given index.
    fn ensure_position(&mut self, pos: usize) {
        while self.position_basis.len() <= pos {
            let seed = self.position_basis.len() as u64;
            self.position_basis
                .push(QuantizedVSA::seeded_random(seed, self.dim));
        }
    }

    /// Majority-vote bundling of N binary VSA vectors.
    /// Each dimension: 255 if > half of vectors have value > 127, else 0.
    pub fn majority_bundle(vectors: &[Vec<u8>]) -> Vec<u8> {
        if vectors.is_empty() {
            return Vec::new();
        }
        let dim = vectors[0].len();
        let n = vectors.len();
        let mut result = vec![0u8; dim];
        for i in 0..dim {
            let ones = vectors.iter().filter(|v| v[i] > 127).count();
            result[i] = if ones > n / 2 { 255 } else { 0 };
        }
        result
    }

    /// Encode text into a VSA vector using n-gram based semantic encoding.
    ///
    /// Process:
    /// 1. Split text into overlapping n-grams (character trigrams by default)
    /// 2. For each n-gram at position i: hash → seeded VSA → XOR with position vector
    /// 3. Majority-vote bundle all position-bound vectors
    pub fn encode_text(&mut self, text: &str) -> Vec<u8> {
        if text.is_empty() {
            return QuantizedVSA::seeded_random(0, self.dim);
        }

        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let ng_size = self.ngram_size;

        if n < ng_size {
            let ngram: String = chars.iter().collect();
            let hash = Self::hash_ngram(&ngram);
            return QuantizedVSA::seeded_random(hash, self.dim);
        }

        let max_pos = n - ng_size + 1;
        self.ensure_position(max_pos - 1);

        let mut bound_vectors: Vec<Vec<u8>> = Vec::with_capacity(max_pos);

        for pos in 0..max_pos {
            let ngram: String = chars[pos..pos + ng_size].iter().collect();
            let hash = Self::hash_ngram(&ngram);
            let ngram_vsa = QuantizedVSA::seeded_random(hash, self.dim);

            let bound: Vec<u8> = ngram_vsa
                .iter()
                .zip(self.position_basis[pos].iter())
                .map(|(a, b)| a ^ b)
                .collect();

            bound_vectors.push(bound);
        }

        Self::majority_bundle(&bound_vectors)
    }

    /// Batch encode texts with content-hash dedup.
    pub fn encode_batch(&mut self, texts: &[&str]) -> Vec<Vec<u8>> {
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        for text in texts {
            let hash = Self::hash_ngram(text);
            if seen.contains(&hash) {
                continue;
            }
            seen.insert(hash);
            results.push(self.encode_text(text));
        }
        results
    }
}

impl Default for NgramVsaEncoder {
    fn default() -> Self {
        Self::new(VSA_DIM, 3)
    }
}

/// Real VSA input pipeline — converts text to VSA vectors
pub struct VsaInputPipeline {
    recent_inputs: Vec<VsaInput>,
    max_recent: usize,
    cycle: u64,
    /// N-gram semantic encoder for text-to-VSA
    ngram_encoder: NgramVsaEncoder,
    /// History of recently encoded items via ngram: (text, vector, content_hash)
    last_encoded: Vec<(String, Vec<u8>, u64)>,
}

impl VsaInputPipeline {
    pub fn new() -> Self {
        Self {
            recent_inputs: Vec::with_capacity(100),
            max_recent: 100,
            cycle: 0,
            ngram_encoder: NgramVsaEncoder::new(VSA_DIM, 3),
            last_encoded: Vec::with_capacity(64),
        }
    }

    /// Encode text content into a VSA vector with semantic type.
    /// Uses content hash → seed for reproducible VSA encoding,
    /// type basis vector → XOR binding for type distinction,
    /// and text length as importance heuristic.
    pub fn encode(&mut self, content: &str, semantic_type: InputSemanticType) -> VsaInput {
        self.cycle += 1;
        let content_hash = self.simple_hash(content);

        let content_vsa = QuantizedVSA::seeded_random(content_hash, VSA_DIM);
        let basis = QuantizedVSA::seeded_random(semantic_type.basis_seed(), VSA_DIM);

        let vector: Vec<u8> = content_vsa
            .iter()
            .zip(basis.iter())
            .map(|(a, b)| a ^ b)
            .collect();

        let length_factor = (content.len() as f64 / 1000.0).min(1.0);
        let type_importance = match semantic_type {
            InputSemanticType::ErrorSignal => 0.9,
            InputSemanticType::UserQuery => 0.8,
            InputSemanticType::DecisionChoice => 0.7,
            InputSemanticType::CuriositySignal => 0.6,
            InputSemanticType::KnowledgeRetrieval => 0.5,
            InputSemanticType::ExplorationResult => 0.4,
            InputSemanticType::SystemResponse => 0.3,
            InputSemanticType::ReflectionNote => 0.3,
            InputSemanticType::ToolOutput => 0.2,
        };
        let importance = (length_factor * 0.3 + type_importance * 0.7).clamp(0.0, 1.0);

        let coherence = if self.recent_inputs.is_empty() {
            1.0
        } else {
            let window: Vec<&VsaInput> = self.recent_inputs.iter().rev().take(3).collect();
            let avg_sim: f64 = if window.is_empty() {
                QuantizedVSA::similarity(&vector, &self.recent_inputs[0].vector)
            } else {
                window
                    .iter()
                    .map(|v| QuantizedVSA::similarity(&vector, &v.vector))
                    .sum::<f64>()
                    / window.len() as f64
            };
            avg_sim
        };

        let input = VsaInput {
            content_hash,
            content: content.to_string(),
            vector,
            semantic_type,
            importance,
            coherence,
        };

        self.recent_inputs.push(input.clone());
        if self.recent_inputs.len() > self.max_recent {
            self.recent_inputs.remove(0);
        }

        input
    }

    /// Encode text using n-gram semantic VSA encoder.
    /// Returns (VSA vector, content_hash).
    /// Unlike `encode()`, this produces semantically meaningful vectors
    /// where similar texts have similar vectors.
    pub fn process_text_input(&mut self, text: &str, _source: &str) -> (Vec<u8>, u64) {
        let hash = NgramVsaEncoder::hash_ngram(text);
        let vector = self.ngram_encoder.encode_text(text);
        (vector, hash)
    }

    /// Encode text via n-gram encoder and store in history.
    pub fn encode_and_record(&mut self, text: &str, source: &str) -> Vec<u8> {
        let (vector, hash) = self.process_text_input(text, source);
        self.last_encoded
            .push((text.to_string(), vector.clone(), hash));
        if self.last_encoded.len() > 100 {
            self.last_encoded.remove(0);
        }
        vector
    }

    /// Access the ngram encoder (for use by callers who need direct access)
    pub fn ngram_encoder_mut(&mut self) -> &mut NgramVsaEncoder {
        &mut self.ngram_encoder
    }

    pub fn ngram_encoder_ref(&self) -> &NgramVsaEncoder {
        &self.ngram_encoder
    }

    /// Recently encoded items via ngram encoder
    pub fn last_encoded_items(&self) -> &[(String, Vec<u8>, u64)] {
        &self.last_encoded
    }

    /// Batch encode multiple inputs, deduplicated by content_hash
    pub fn encode_batch(&mut self, items: &[(&str, InputSemanticType)]) -> Vec<VsaInput> {
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        for (content, st) in items {
            let hash = self.simple_hash(content);
            if seen.contains(&hash) {
                continue;
            }
            seen.insert(hash);
            results.push(self.encode(content, *st));
        }
        results
    }

    /// Get the most recent n inputs
    pub fn recent(&self, n: usize) -> &[VsaInput] {
        let len = self.recent_inputs.len();
        let start = if len > n { len - n } else { 0 };
        &self.recent_inputs[start..]
    }

    /// Clear recent history
    pub fn clear(&mut self) {
        self.recent_inputs.clear();
    }

    pub fn recent_count(&self) -> usize {
        self.recent_inputs.len()
    }

    pub fn pipeline_cycle(&self) -> u64 {
        self.cycle
    }

    /// Simple hash for content (Jenkins-like)
    fn simple_hash(&self, content: &str) -> u64 {
        let mut hash: u64 = 0;
        for (i, b) in content.bytes().enumerate() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u64);
            if i % 4 == 0 {
                hash ^= hash >> 7;
            }
        }
        hash
    }
}

impl Default for VsaInputPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_returns_vsa_input() {
        let mut pipeline = VsaInputPipeline::new();
        let input = pipeline.encode("hello world", InputSemanticType::UserQuery);
        assert_eq!(input.content, "hello world");
        assert_eq!(input.semantic_type, InputSemanticType::UserQuery);
        assert_eq!(input.vector.len(), VSA_DIM);
        assert!(input.importance >= 0.0 && input.importance <= 1.0);
        assert!(input.coherence >= 0.0 && input.coherence <= 1.0);
        assert!(input.content_hash != 0);
    }

    #[test]
    fn test_different_inputs_different_vectors() {
        let mut pipeline = VsaInputPipeline::new();
        let a = pipeline.encode("alpha", InputSemanticType::UserQuery);
        let b = pipeline.encode("beta", InputSemanticType::UserQuery);
        assert_ne!(a.content_hash, b.content_hash);
        assert_ne!(a.vector, b.vector);
    }

    #[test]
    fn test_same_input_same_vector() {
        let mut pipeline = VsaInputPipeline::new();
        let a = pipeline.encode("deterministic test", InputSemanticType::KnowledgeRetrieval);
        let b = pipeline.encode("deterministic test", InputSemanticType::KnowledgeRetrieval);
        assert_eq!(a.content_hash, b.content_hash);
        assert_eq!(a.vector, b.vector);
        assert!((a.importance - b.importance).abs() < 1e-12);
    }

    #[test]
    fn test_different_types_different_vectors() {
        let mut pipeline = VsaInputPipeline::new();
        let a = pipeline.encode("same text", InputSemanticType::UserQuery);
        let b = pipeline.encode("same text", InputSemanticType::ToolOutput);
        assert_ne!(a.semantic_type, b.semantic_type);
        assert_ne!(a.vector, b.vector);
    }

    #[test]
    fn test_batch_dedup() {
        let mut pipeline = VsaInputPipeline::new();
        let items = &[
            ("dup", InputSemanticType::UserQuery),
            ("dup", InputSemanticType::SystemResponse),
            ("unique", InputSemanticType::UserQuery),
        ];
        let results = pipeline.encode_batch(items);
        assert_eq!(results.len(), 2, "batch should dedup the two 'dup' inputs");
        assert_eq!(results[0].content_hash, pipeline.simple_hash("dup"));
        assert_eq!(results[1].content_hash, pipeline.simple_hash("unique"));
    }

    #[test]
    fn test_recent_returns_last_n() {
        let mut pipeline = VsaInputPipeline::new();
        pipeline.encode("first", InputSemanticType::UserQuery);
        pipeline.encode("second", InputSemanticType::UserQuery);
        pipeline.encode("third", InputSemanticType::UserQuery);
        pipeline.encode("fourth", InputSemanticType::UserQuery);

        let recent = pipeline.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].content, "second");
        assert_eq!(recent[1].content, "third");
        assert_eq!(recent[2].content, "fourth");
    }

    // ─── NgramVsaEncoder tests ───

    #[test]
    fn test_ngram_encoder_deterministic() {
        let mut enc = NgramVsaEncoder::new(256, 3);
        let a = enc.encode_text("hello world");
        let b = enc.encode_text("hello world");
        assert_eq!(a, b, "same text must produce identical VSA vector");
    }

    #[test]
    fn test_ngram_encoder_different_texts_different() {
        let mut enc = NgramVsaEncoder::new(256, 3);
        let a = enc.encode_text("the quick brown fox");
        let b = enc.encode_text("jumps over lazy dog");
        let sim = QuantizedVSA::similarity(&a, &b);
        assert!(
            sim < 0.7,
            "different texts should have similarity < 0.7, got {}",
            sim
        );
    }

    #[test]
    fn test_ngram_encoder_similar_texts_similar() {
        let mut enc = NgramVsaEncoder::new(256, 3);
        let a = enc.encode_text("hello world");
        let b = enc.encode_text("hello word");
        let sim = QuantizedVSA::similarity(&a, &b);
        assert!(
            sim > 0.5,
            "similar texts should have similarity > 0.5, got {}",
            sim
        );
    }

    #[test]
    fn test_process_text_input_returns_vector() {
        let mut pipeline = VsaInputPipeline::new();
        let (vector, hash) = pipeline.process_text_input("hello world", "test");
        assert_eq!(vector.len(), VSA_DIM);
        assert_ne!(hash, 0);
    }

    #[test]
    fn test_encode_batch_dedup() {
        let mut enc = NgramVsaEncoder::new(256, 3);
        let texts = &["hello", "world", "hello"];
        let results = enc.encode_batch(texts);
        assert_eq!(results.len(), 2, "batch should dedup 'hello'");
    }

    #[test]
    fn test_encode_and_record_stores_history() {
        let mut pipeline = VsaInputPipeline::new();
        let v1 = pipeline.encode_and_record("hello world", "test");
        let v2 = pipeline.encode_and_record("goodbye world", "test");
        assert_eq!(pipeline.last_encoded_items().len(), 2);
        assert_eq!(pipeline.last_encoded_items()[0].0, "hello world");
        assert_eq!(pipeline.last_encoded_items()[1].0, "goodbye world");
        assert_eq!(pipeline.last_encoded_items()[0].1, v1);
        assert_eq!(pipeline.last_encoded_items()[1].1, v2);
    }

    #[test]
    fn test_ngram_encoder_empty_text() {
        let mut enc = NgramVsaEncoder::new(256, 3);
        let v = enc.encode_text("");
        assert_eq!(v.len(), 256);
    }

    #[test]
    fn test_ngram_encoder_short_text() {
        let mut enc = NgramVsaEncoder::new(256, 3);
        let v = enc.encode_text("ab");
        assert_eq!(v.len(), 256);
        // text shorter than ngram_size should still produce deterministic output
        let v2 = enc.encode_text("ab");
        assert_eq!(v, v2);
    }

    #[test]
    fn test_majority_bundle_basic() {
        let v1 = vec![255u8, 0, 255, 0];
        let v2 = vec![255u8, 255, 0, 0];
        let v3 = vec![0u8, 255, 255, 0];
        let result = NgramVsaEncoder::majority_bundle(&[v1, v2, v3]);
        // majority per dim: dim0=2/3 ones → 255, dim1=2/3 → 255, dim2=2/3 → 255, dim3=0/3 → 0
        assert_eq!(result, vec![255u8, 255, 255, 0]);
    }

    #[test]
    fn test_ngram_encoder_4096_dim() {
        let mut enc = NgramVsaEncoder::new(VSA_DIM, 3);
        let v =
            enc.encode_text("This is a longer test sentence for the 4096-dimension VSA encoder.");
        assert_eq!(v.len(), VSA_DIM);
    }
}

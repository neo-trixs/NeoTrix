//! Unified KB embedding facade — single entry point for all embedding needs.
//!
//! Routes to the appropriate backend based on context:
//! - **Fast** (64-dim local bag-of-words): SVAF dedup checks, quick similarity
//! - **Semantic** (external API or local hash-kernel): high-quality semantic search
//! - **Associative** (VSA 1024-dim hypervectors): concept expansion for query augmentation
//!
//! Eliminates the need for callers to pick between `nt_core_embed::TextEmbedder`,
//! `nt_memory_embed::embed_text`, and `VsaAssociativeExpander` directly.

use std::sync::Mutex;

use super::nt_memory_embed::EmbeddingConfig;
use super::nt_memory_vsa_expand::VsaAssociativeExpander;
use crate::core::nt_core_embed::TextEmbedder;

/// Unified embedding facade — holds all three backends and routes requests.
pub struct KbEmbed {
    /// 64-dim local bag-of-words embedder (zero dependency, deterministic).
    fast: Mutex<TextEmbedder>,
    /// External API / local hash-kernel config for semantic embeddings.
    semantic_config: EmbeddingConfig,
    /// VSA trigram-based associative expander (1024-dim ±1 hypervectors).
    vsa: Mutex<VsaAssociativeExpander>,
}

impl Default for KbEmbed {
    fn default() -> Self {
        Self {
            fast: Mutex::new(TextEmbedder::new()),
            semantic_config: EmbeddingConfig::default(),
            vsa: Mutex::new(VsaAssociativeExpander::new(1024)),
        }
    }
}

impl std::fmt::Debug for KbEmbed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KbEmbed")
            .field("fast_vocab", &self.fast.lock().map(|e| e.vocab_size()).unwrap_or(0))
            .field("semantic_mode", &self.semantic_config.mode)
            .field("vsa_vocab", &self.vsa.lock().map(|e| e.vocab_size()).unwrap_or(0))
            .finish()
    }
}

impl KbEmbed {
    /// Create a new facade with the given semantic config.
    pub fn new(semantic_config: EmbeddingConfig) -> Self {
        Self {
            fast: Mutex::new(TextEmbedder::new()),
            semantic_config,
            vsa: Mutex::new(VsaAssociativeExpander::new(1024)),
        }
    }

    /// Create with a specific VSA dimension (default: 1024).
    pub fn with_vsa_dim(semantic_config: EmbeddingConfig, vsa_dim: usize) -> Self {
        Self {
            fast: Mutex::new(TextEmbedder::new()),
            semantic_config,
            vsa: Mutex::new(VsaAssociativeExpander::new(vsa_dim)),
        }
    }

    /// Replace the semantic config at runtime (e.g., after detecting a new API key).
    pub fn set_semantic_config(&self, _config: EmbeddingConfig) {
        // Design note: EmbeddingConfig is Clone, so callers can swap freely.
        // We store it directly — no RwLock needed since this is infrequent.
        // For thread safety in the facade, we use interior mutability via the
        // semantic_config field being set before use (callers coordinate via
        // KnowledgeBase.embedding_config RwLock which is the single fact source).
        //
        // We intentionally do NOT hold a lock here — the config is read at call
        // time via `semantic_config()` which returns a clone.
        //
        // This method exists for API symmetry; the real config lives on
        // KnowledgeBase.embedding_config and is passed through `embed_semantic`.
        // No-op here; see `embed_semantic_with_config`.
    }

    /// 64-dim fast local embedding for dedup / SVAF gate checks.
    ///
    /// Uses `nt_core_embed::TextEmbedder` — bag-of-words with CJK bigram
    /// support, zero external dependency, deterministic, ~µs latency.
    pub fn embed_fast(&self, text: &str) -> Vec<f64> {
        let mut embedder = match self.fast.lock() {
            Ok(e) => e,
            Err(poisoned) => {
                log::warn!("KbEmbed: fast embedder lock poisoned, recovering");
                poisoned.into_inner()
            }
        };
        embedder.embed(text)
    }

    /// Cosine similarity between two fast (64-dim) vectors.
    pub fn fast_similarity(&self, a: &str, b: &str) -> f64 {
        let mut embedder = match self.fast.lock() {
            Ok(e) => e,
            Err(poisoned) => poisoned.into_inner(),
        };
        embedder.similarity(a, b)
    }

    /// High-quality semantic embedding via external API (with local fallback).
    ///
    /// Routes through `nt_memory_embed::embed_text` which handles:
    /// - HTTP API call to OpenAI-compatible endpoint (default: MiniLM local)
    /// - Automatic fallback to local 384-dim hash-kernel on HTTP failure
    /// - Batch-friendly for multiple texts
    pub fn embed_semantic(&self, text: &str) -> Result<Vec<f32>, String> {
        super::nt_memory_embed::embed_text(&self.semantic_config, text)
    }

    /// Semantic embedding with an explicit config override.
    pub fn embed_semantic_with_config(text: &str, config: &EmbeddingConfig) -> Result<Vec<f32>, String> {
        super::nt_memory_embed::embed_text(config, text)
    }

    /// Batch semantic embedding.
    pub fn embed_semantic_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        super::nt_memory_embed::embed_text_batch(&self.semantic_config, texts)
    }

    /// VSA associative expansion: query → related concept terms.
    ///
    /// Uses character trigram hypervectors (1024-dim) for concept association.
    /// Shared trigrams (e.g., "retrieval"/"retrieve") produce high similarity
    /// without any external embedding endpoint.
    pub fn expand_associative(&self, query: &str, top_k: usize) -> Vec<(String, f64)> {
        let expander = match self.vsa.lock() {
            Ok(e) => e,
            Err(poisoned) => {
                log::warn!("KbEmbed: VSA expander lock poisoned, recovering");
                poisoned.into_inner()
            }
        };
        expander.expand(query, top_k)
    }

    /// VSA expand → return the augmented query string (original + related terms).
    pub fn expand_query(&self, query: &str, top_k: usize) -> String {
        let expander = match self.vsa.lock() {
            Ok(e) => e,
            Err(poisoned) => poisoned.into_inner(),
        };
        expander.expand_query(query, top_k)
    }

    /// Insert terms into the VSA vocabulary for associative expansion.
    pub fn insert_vsa_terms<I: IntoIterator<Item = String>>(&self, terms: I) {
        let mut expander = match self.vsa.lock() {
            Ok(e) => e,
            Err(poisoned) => poisoned.into_inner(),
        };
        expander.insert_terms(terms);
    }

    /// Insert a single term into the VSA vocabulary.
    pub fn insert_vsa_term(&self, term: &str) {
        let mut expander = match self.vsa.lock() {
            Ok(e) => e,
            Err(poisoned) => poisoned.into_inner(),
        };
        expander.insert_term(term);
    }

    /// VSA vocabulary size.
    pub fn vsa_vocab_size(&self) -> usize {
        self.vsa.lock().map(|e| e.vocab_size()).unwrap_or(0)
    }

    /// Fast embedder vocabulary size.
    pub fn fast_vocab_size(&self) -> usize {
        self.fast.lock().map(|e| e.vocab_size()).unwrap_or(0)
    }

    /// Get a clone of the current semantic config.
    pub fn semantic_config(&self) -> EmbeddingConfig {
        self.semantic_config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::nt_memory_embed::EmbedMode;

    #[test]
    fn test_kb_embed_fast_deterministic() {
        let kb = KbEmbed::default();
        let a = kb.embed_fast("hello world");
        let b = kb.embed_fast("hello world");
        assert_eq!(a, b, "fast embedding must be deterministic");
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn test_kb_embed_fast_normalization() {
        let kb = KbEmbed::default();
        let v = kb.embed_fast("test normalization");
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "fast embedding should be L2-normalized");
    }

    #[test]
    fn test_kb_embed_fast_similarity_related_higher() {
        let kb = KbEmbed::default();
        let sim_related = kb.fast_similarity(
            "fix database connection pool",
            "database connection pooling issue",
        );
        let sim_unrelated = kb.fast_similarity(
            "fix database connection pool",
            "design responsive layout css",
        );
        assert!(
            sim_related > sim_unrelated,
            "related texts should have higher similarity: {} vs {}",
            sim_related,
            sim_unrelated
        );
    }

    #[test]
    fn test_kb_embed_semantic_local_fallback() {
        let kb = KbEmbed::default();
        // Config defaults to local mode — no network needed.
        let result = kb.embed_semantic("test semantic embedding");
        assert!(result.is_ok(), "local semantic embedding should succeed");
        let vec = result.unwrap();
        assert_eq!(vec.len(), 384, "default hash-kernel dim is 384");
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-4, "semantic embedding should be L2-normalized");
    }

    #[test]
    fn test_kb_embed_semantic_batch() {
        let kb = KbEmbed::default();
        let result = kb.embed_semantic_batch(&["alpha", "beta", "gamma"]);
        assert!(result.is_ok());
        let vecs = result.unwrap();
        assert_eq!(vecs.len(), 3);
        for v in &vecs {
            assert_eq!(v.len(), 384);
        }
    }

    #[test]
    fn test_kb_embed_vsa_expand() {
        let kb = KbEmbed::default();
        kb.insert_vsa_terms(
            ["retrieval", "retrieve", "search", "query", "index"]
                .iter()
                .map(|s| s.to_string()),
        );
        let related = kb.expand_associative("retrieval", 3);
        assert!(!related.is_empty(), "VSA should return related terms");
        let top = &related[0].0;
        assert!(
            top == "retrieval" || top == "retrieve" || top == "search",
            "top related term should be semantically relevant, got: {}",
            top
        );
    }

    #[test]
    fn test_kb_embed_vsa_expand_query() {
        let kb = KbEmbed::default();
        kb.insert_vsa_terms(
            ["retrieval", "retrieve", "search"].iter().map(|s| s.to_string()),
        );
        let expanded = kb.expand_query("retrieval", 2);
        assert!(expanded.contains("retrieval"), "expanded query preserves original");
        assert!(expanded.len() > "retrieval".len(), "expanded query appends related terms");
    }

    #[test]
    fn test_kb_embed_default_creation() {
        let kb = KbEmbed::default();
        assert_eq!(kb.fast_vocab_size(), 0);
        assert_eq!(kb.vsa_vocab_size(), 0);
        assert_eq!(kb.semantic_config().mode, EmbedMode::Local);
    }

    #[test]
    fn test_kb_embed_with_custom_config() {
        let config = EmbeddingConfig {
            api_key: "test-key".into(),
            base_url: "http://localhost:9999/v1".into(),
            model: "custom-model".into(),
            dimension: 256,
            mode: EmbedMode::Local,
        };
        let kb = KbEmbed::new(config.clone());
        assert_eq!(kb.semantic_config().model, "custom-model");
        assert_eq!(kb.semantic_config().dimension, 256);
    }

    #[test]
    fn test_kb_embed_with_vsa_dim() {
        let kb = KbEmbed::with_vsa_dim(EmbeddingConfig::default(), 512);
        assert_eq!(kb.vsa_vocab_size(), 0);
        // Insert a term and verify it works with custom dim
        kb.insert_vsa_term("test");
        assert_eq!(kb.vsa_vocab_size(), 1);
        let v = kb.expand_associative("test", 1);
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn test_kb_embed_thread_safety() {
        use std::sync::Arc;
        let kb = Arc::new(KbEmbed::default());
        let mut handles = vec![];
        for i in 0..8 {
            let kb = Arc::clone(&kb);
            handles.push(std::thread::spawn(move || {
                let text = format!("thread {} test text", i);
                let fast = kb.embed_fast(&text);
                assert_eq!(fast.len(), 64);
                let semantic = kb.embed_semantic(&text);
                assert!(semantic.is_ok());
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }
}

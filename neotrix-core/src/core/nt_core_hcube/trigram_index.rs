// REVIVED Task 2 — dead_code removed
use std::collections::HashMap;

/// Trigram hash: 3-byte sequence → 64-bit hash
fn trigram_hash(trigram: &[u8]) -> u64 {
    debug_assert!(trigram.len() == 3);
    let mut h = 0x9E3779B97F4A7C15u64;
    for &b in trigram {
        h = h.wrapping_mul(31).wrapping_add(b as u64);
    }
    h
}

/// Extract all trigrams from a byte sequence with positional encoding.
fn extract_trigrams(text: &str) -> Vec<u64> {
    let bytes = text.as_bytes();
    if bytes.len() < 3 {
        return Vec::new();
    }
    bytes.windows(3).map(|tri| trigram_hash(tri)).collect()
}

/// Indexed document for trigram search
#[derive(Debug, Clone)]
pub struct IndexedDocument {
    pub doc_id: usize,
    pub text: String,
}

/// Trigram inverted index for fast grep-like search.
///
/// Maps trigram hashes to document IDs, enabling sublinear
/// candidate generation for regex and substring search.
#[derive(Debug, Clone)]
pub struct TrigramInvertedIndex {
    index: HashMap<u64, Vec<usize>>,
    docs: Vec<IndexedDocument>,
}

impl Default for TrigramInvertedIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl TrigramInvertedIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            docs: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.docs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    pub fn doc(&self, doc_id: usize) -> Option<&IndexedDocument> {
        self.docs.get(doc_id)
    }

    pub fn all_docs(&self) -> &[IndexedDocument] {
        &self.docs
    }

    /// Insert a document into the index.
    pub fn insert(&mut self, text: &str) -> usize {
        let doc_id = self.docs.len();
        self.docs.push(IndexedDocument {
            doc_id,
            text: text.to_string(),
        });
        let trigrams = extract_trigrams(text);
        for hash in trigrams {
            self.index.entry(hash).or_default().push(doc_id);
        }
        doc_id
    }

    /// LIKE search: find documents containing `substring`.
    /// Uses trigram filtering for candidate generation, then verifies with contains().
    pub fn like_search(&self, substring: &str) -> Vec<usize> {
        if substring.is_empty() {
            return (0..self.docs.len()).collect();
        }
        let query_trigrams = extract_trigrams(substring);
        if query_trigrams.is_empty() {
            // Single/double character substring: linear scan
            return self
                .docs
                .iter()
                .filter(|d| d.text.contains(substring))
                .map(|d| d.doc_id)
                .collect();
        }
        // Find documents containing all query trigrams
        let candidates = self.intersect_trigrams(&query_trigrams);
        // Verify with actual substring match
        candidates
            .into_iter()
            .filter(|&doc_id| self.docs[doc_id].text.contains(substring))
            .collect()
    }

    /// Regex search: compile pattern, extract required trigrams, filter candidates, verify.
    pub fn regex_search(&self, pattern: &str) -> Result<Vec<usize>, regex::Error> {
        let re = regex::Regex::new(pattern)?;
        let required = Self::pattern_trigrams(pattern);
        let candidates = if required.is_empty() {
            (0..self.docs.len()).collect()
        } else {
            self.intersect_trigrams(&required)
        };
        Ok(candidates
            .into_iter()
            .filter(|&doc_id| re.is_match(&self.docs[doc_id].text))
            .collect())
    }

    /// Naive trigram extraction from regex pattern string.
    /// Only extracts literal substrings of length ≥ 3 that must appear in matches.
    fn pattern_trigrams(pattern: &str) -> Vec<u64> {
        let bytes = pattern.as_bytes();
        let mut trigrams = Vec::new();
        let mut i = 0;
        while i + 3 <= bytes.len() {
            if bytes[i..i + 3]
                .iter()
                .all(|&b| b.is_ascii_alphanumeric() || b == b'_')
            {
                trigrams.push(trigram_hash(&bytes[i..i + 3]));
            }
            i += 1;
        }
        trigrams
    }

    /// Find doc IDs present in all trigram posting lists (AND intersection).
    fn intersect_trigrams(&self, trigrams: &[u64]) -> Vec<usize> {
        if trigrams.is_empty() {
            return Vec::new();
        }
        let mut sorted: Vec<Vec<usize>> = trigrams
            .iter()
            .filter_map(|h| self.index.get(h))
            .cloned()
            .collect();
        if sorted.is_empty() {
            return Vec::new();
        }
        // Sort by list length (smallest first for efficiency)
        sorted.sort_by_key(|v| v.len());
        let mut result: Vec<usize> = sorted[0].clone();
        result.sort_unstable();
        for list in &sorted[1..] {
            let mut list_sorted = list.clone();
            list_sorted.sort_unstable();
            result.retain(|id| list_sorted.binary_search(id).is_ok());
        }
        result
    }

    /// Clear the index.
    pub fn clear(&mut self) {
        self.index.clear();
        self.docs.clear();
    }
}

// ─── VSA HNSW Index (v0: brute-force) ──────────────────────────

/// Approximate nearest neighbor index for VSA vectors.
#[derive(Debug, Clone)]
pub struct VsaHnswIndex {
    vectors: Vec<Vec<f32>>,
    generators: Vec<u64>,
    dim: usize,
}

impl VsaHnswIndex {
    pub fn new(dim: usize) -> Self {
        Self {
            vectors: Vec::new(),
            generators: Vec::new(),
            dim,
        }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn insert(&mut self, vec: &[f32], generator: u64) {
        self.vectors.push(vec.to_vec());
        self.generators.push(generator);
    }

    /// Brute-force k-NN search by cosine similarity.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(u64, f32)> {
        let mut scored: Vec<(usize, f32)> = self
            .vectors
            .iter()
            .map(|v| cosine_similarity(query, v))
            .enumerate()
            .collect();
        scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
            .into_iter()
            .map(|(i, s)| (self.generators[i], s))
            .collect()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

/// Simple hash-based VSA vector from text (no real FFT).
fn sim_hash(query: &str, dim: usize) -> Vec<f32> {
    let mut vec = vec![0.0f32; dim];
    let bytes = query.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        let idx = i % dim;
        let val = (b as f32 - 128.0) / 128.0;
        vec[idx] += val;
    }
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vec {
            *v /= norm;
        }
    }
    vec
}

// ─── HybridRetriever (RRF Fusion) ──────────────────────────────

/// Fuses trigram and VSA search results using Reciprocal Rank Fusion.
#[derive(Debug, Clone)]
pub struct HybridRetriever {
    trigram_index: TrigramInvertedIndex,
    vsa_index: VsaHnswIndex,
}

impl HybridRetriever {
    pub fn new(trigram: TrigramInvertedIndex, vsa: VsaHnswIndex) -> Self {
        Self {
            trigram_index: trigram,
            vsa_index: vsa,
        }
    }

    pub fn search(
        &self,
        query: &str,
        k: usize,
        _alpha: f32,
        trigram_k: usize,
        vsa_k: usize,
    ) -> Vec<(u64, f32, String)> {
        let trigram_ids = self.trigram_index.like_search(query);
        let vsa_vec = sim_hash(query, self.vsa_index.dim());
        let vsa_results = self.vsa_index.search(&vsa_vec, vsa_k);

        const RRF_K: f32 = 60.0;
        let mut fused: HashMap<usize, (f32, String)> = HashMap::new();

        for (rank, &doc_id) in trigram_ids.iter().take(trigram_k).enumerate() {
            let rrf = 1.0 / (RRF_K + rank as f32);
            let text = self
                .trigram_index
                .doc(doc_id)
                .map(|d| d.text.clone())
                .unwrap_or_default();
            fused
                .entry(doc_id)
                .and_modify(|(s, _)| *s += rrf)
                .or_insert((rrf, text));
        }

        for (rank, (gen, _vsa_score)) in vsa_results.iter().enumerate() {
            let doc_id = *gen as usize;
            let rrf = 1.0 / (RRF_K + rank as f32);
            let text = self
                .trigram_index
                .doc(doc_id)
                .map(|d| d.text.clone())
                .unwrap_or_default();
            fused
                .entry(doc_id)
                .and_modify(|(s, _)| *s += rrf)
                .or_insert((rrf, text));
        }

        let mut result: Vec<(u64, f32, String)> = fused
            .into_iter()
            .map(|(doc_id, (score, text))| (doc_id as u64, score, text))
            .collect();
        result.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result.truncate(k);
        result
    }
}

// ─── AgenticSearcher ───────────────────────────────────────────

/// Plan-Retrieve-Evaluate search agent (v0: direct retrieval stub).
#[derive(Debug, Clone)]
pub struct AgenticSearcher {
    retriever: HybridRetriever,
}

impl AgenticSearcher {
    pub fn new(retriever: HybridRetriever) -> Self {
        Self { retriever }
    }

    pub fn search(&self, query: &str, k: usize) -> Vec<(u64, f32, String)> {
        self.retriever.search(query, k, 0.7, 20, 20)
    }

    /// Plan phase stub: returns the query as the only plan step.
    pub fn plan(&self, query: &str) -> Vec<String> {
        vec![query.to_string()]
    }

    /// Evaluate phase stub: sorts results by score descending.
    pub fn evaluate(
        &self,
        results: &[(u64, f32, String)],
        _query: &str,
    ) -> Vec<(u64, f32, String)> {
        let mut r = results.to_vec();
        r.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigram_hash_deterministic() {
        let h1 = trigram_hash(b"hel");
        let h2 = trigram_hash(b"hel");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_trigram_hash_differs() {
        let h1 = trigram_hash(b"hel");
        let h2 = trigram_hash(b"wor");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_insert_and_count() {
        let mut idx = TrigramInvertedIndex::new();
        assert!(idx.is_empty());
        idx.insert("hello world");
        idx.insert("world peace");
        assert_eq!(idx.len(), 2);
    }

    #[test]
    fn test_like_search_basic() {
        let mut idx = TrigramInvertedIndex::new();
        idx.insert("hello world this is a test");
        idx.insert("world peace and harmony");
        idx.insert("goodbye cruel world");
        let results = idx.like_search("world");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_like_search_substring() {
        let mut idx = TrigramInvertedIndex::new();
        idx.insert("hello world");
        idx.insert("goodbye");
        let results = idx.like_search("ello");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0);
    }

    #[test]
    fn test_like_search_no_results() {
        let mut idx = TrigramInvertedIndex::new();
        idx.insert("hello world");
        let results = idx.like_search("xyz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_like_search_short_pattern() {
        let mut idx = TrigramInvertedIndex::new();
        idx.insert("hello world");
        idx.insert("hi");
        let results = idx.like_search("hi");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_regex_search() {
        let mut idx = TrigramInvertedIndex::new();
        idx.insert("hello world 42");
        idx.insert("goodbye world");
        idx.insert("hello there");
        let results = idx.regex_search("hello.*\\d+").expect("valid regex");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0);
    }

    #[test]
    fn test_regex_invalid_pattern() {
        let idx = TrigramInvertedIndex::new();
        let result = idx.regex_search("[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_clear() {
        let mut idx = TrigramInvertedIndex::new();
        idx.insert("test");
        assert!(!idx.is_empty());
        idx.clear();
        assert!(idx.is_empty());
    }

    #[test]
    fn test_multiple_inserts_same_doc() {
        let mut idx = TrigramInvertedIndex::new();
        let id1 = idx.insert("hello world");
        let id2 = idx.insert("hello world again");
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(idx.len(), 2);
    }

    #[test]
    fn test_like_search_case_sensitive() {
        let mut idx = TrigramInvertedIndex::new();
        idx.insert("Hello World");
        idx.insert("hello world");
        // 'ello' only matches the lowercase one due to trigram extraction
        // (since 'H' and 'h' have different ASCII values)
        let results = idx.like_search("ello");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_empty_trigram_insert() {
        let mut idx = TrigramInvertedIndex::new();
        idx.insert("a"); // less than 3 bytes
        assert_eq!(idx.len(), 1);
        let results = idx.like_search("a");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_vsa_hnsw_insert_search() {
        let mut idx = VsaHnswIndex::new(8);
        idx.insert(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 100);
        idx.insert(&[0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 200);
        idx.insert(&[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], 300);
        idx.insert(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0], 400);
        idx.insert(&[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0], 500);
        let results = idx.search(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 3);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 100);
        assert!(results[0].1 > 0.99);
    }

    #[test]
    fn test_hybrid_retriever() {
        let mut trigram = TrigramInvertedIndex::new();
        trigram.insert("hello world this is a test document");
        trigram.insert("another document about machine learning");
        trigram.insert("neural networks and deep learning");
        let mut vsa = VsaHnswIndex::new(8);
        vsa.insert(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0);
        vsa.insert(&[0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 1);
        vsa.insert(&[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], 2);
        let hybrid = HybridRetriever::new(trigram, vsa);
        let results = hybrid.search("hello world", 5, 0.7, 10, 10);
        assert!(!results.is_empty());
        for (_doc_id, score, text) in &results {
            assert!(*score > 0.0);
            assert!(!text.is_empty());
        }
    }

    #[test]
    fn test_agentic_searcher() {
        let mut trigram = TrigramInvertedIndex::new();
        trigram.insert("test document one");
        trigram.insert("test document two");
        let mut vsa = VsaHnswIndex::new(8);
        vsa.insert(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0);
        vsa.insert(&[0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 1);
        let hybrid = HybridRetriever::new(trigram, vsa);
        let searcher = AgenticSearcher::new(hybrid);
        let plan = searcher.plan("test");
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0], "test");
        let results = searcher.search("test", 5);
        assert!(!results.is_empty());
        for (_doc_id, score, text) in &results {
            assert!(*score > 0.0);
            assert!(!text.is_empty());
        }
        let evaluated = searcher.evaluate(&results, "test");
        assert_eq!(evaluated.len(), results.len());
        for w in evaluated.windows(2) {
            assert!(w[0].1 >= w[1].1);
        }
    }
}

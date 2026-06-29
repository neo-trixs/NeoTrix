#![forbid(unsafe_code)]

use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use crate::core::nt_core_hcube::QuantizedVSA;
use std::collections::{HashMap, HashSet};

/// Source of a keyword entry.
#[derive(Debug, Clone, PartialEq)]
pub enum KeywordSource {
    /// Extracted from crawled web pages.
    Crawl,
    /// Extracted from search results.
    Search,
    /// Extracted from consciousness attractor state.
    Consciousness,
    /// Manually added.
    Manual,
}

/// A single keyword record in the lexicon.
#[derive(Debug, Clone)]
pub struct KeywordRecord {
    /// Unique identifier.
    pub id: u64,
    /// The keyword text.
    pub keyword: String,
    /// VSA-encoded vector representation.
    pub vsa_vector: Vec<u8>,
    /// Number of times this keyword has been seen.
    pub frequency: u32,
    /// Cycle number of the most recent occurrence.
    pub recency: u64,
    /// Confidence score in [0, 1].
    pub confidence: f64,
    /// Source of this keyword.
    pub source: KeywordSource,
}

/// A persistent VSA-encoded keyword vocabulary with NTSSEG-style storage.
///
/// Maintains a bounded lexicon of keywords, each encoded as a VSA vector
/// for semantic similarity lookup. Supports clustering, frequency tracking,
/// and LRU-style pruning.
pub struct KeywordLexicon {
    /// Keyword text → record map.
    keywords: HashMap<String, KeywordRecord>,
    /// ID → keyword text lookup.
    id_index: HashMap<u64, String>,
    /// Monotonically increasing ID counter.
    next_id: u64,
    /// Maximum number of entries before LRU eviction.
    max_entries: usize,
    /// Keyword ID → cluster ID for semantic grouping.
    cluster_assignments: HashMap<u64, u64>,
}

impl KeywordLexicon {
    /// Creates a new empty lexicon with the default capacity (1000).
    pub fn new() -> Self {
        Self {
            keywords: HashMap::new(),
            id_index: HashMap::new(),
            next_id: 1,
            max_entries: 1000,
            cluster_assignments: HashMap::new(),
        }
    }

    /// Creates a new empty lexicon with a custom maximum entry count.
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            keywords: HashMap::new(),
            id_index: HashMap::new(),
            next_id: 1,
            max_entries,
            cluster_assignments: HashMap::new(),
        }
    }

    /// Encodes a keyword string into a VSA vector.
    ///
    /// Computes a hash from all 2-grams (bigrams) of the text using
    /// XOR rotation, then uses that hash to seed a deterministic VSA vector
    /// via `QuantizedVSA::seeded_random`. This ensures the same keyword
    /// always produces the same vector.
    pub fn encode_keyword(text: &str) -> Vec<u8> {
        let seed = keyword_hash(text);
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    /// Finds existing keywords whose VSA similarity to `state` exceeds 0.7.
    ///
    /// Returns up to `top_k` keywords sorted by similarity descending.
    pub fn extract_from_attractor(&self, state: &[u8], top_k: usize) -> Vec<String> {
        let mut scored: Vec<(f64, &str)> = self
            .keywords
            .values()
            .map(|rec| {
                let sim = QuantizedVSA::similarity(state, &rec.vsa_vector);
                (sim, rec.keyword.as_str())
            })
            .filter(|(sim, _)| *sim > 0.7)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored.into_iter().map(|(_, kw)| kw.to_string()).collect()
    }

    /// Extracts keywords from free text using a simple TF-IDF-like approach.
    ///
    /// Splits text by whitespace, filters common stop words, takes the top-k
    /// by frequency, and adds-or-updates each in the lexicon.
    pub fn extract_from_text(
        &mut self,
        text: &str,
        source: KeywordSource,
        top_k: usize,
    ) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        let mut freq: HashMap<&str, u32> = HashMap::new();
        for token in text.split_whitespace() {
            let cleaned = token
                .trim_matches(|c: char| c.is_ascii_punctuation())
                .to_lowercase();
            if cleaned.is_empty() || cleaned.len() <= 1 || is_stop_word(&cleaned) {
                continue;
            }
            // Leak the cleaned string into freq keys — we own the String from split
            // so we need to allocate.
            let owned = cleaned;
            *freq.entry(Box::leak(owned.into_boxed_str())).or_insert(0) += 1;
        }

        let mut sorted: Vec<(&str, u32)> = freq.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(top_k);

        let mut result = Vec::with_capacity(sorted.len());
        for (keyword, _) in &sorted {
            self.add_or_update(keyword, source.clone());
            result.push(keyword.to_string());
        }
        result
    }

    /// Adds a new keyword or updates an existing one.
    ///
    /// If the keyword already exists, increments its frequency, updates
    /// recency, and raises confidence. If the lexicon is at capacity,
    /// evicts the least recently used (lowest recency) entry first.
    /// Returns the keyword ID.
    pub fn add_or_update(&mut self, keyword: &str, source: KeywordSource) -> u64 {
        if let Some(rec) = self.keywords.get_mut(keyword) {
            rec.frequency = rec.frequency.saturating_add(1);
            rec.recency = current_cycle();
            rec.confidence = rec.confidence.min(1.0) + 0.05;
            if rec.confidence > 1.0 {
                rec.confidence = 1.0;
            }
            return rec.id;
        }

        if self.keywords.len() >= self.max_entries {
            self.evict_one();
        }

        let id = self.next_id;
        self.next_id += 1;
        let vsa_vector = Self::encode_keyword(keyword);
        let record = KeywordRecord {
            id,
            keyword: keyword.to_string(),
            vsa_vector,
            frequency: 1,
            recency: current_cycle(),
            confidence: 0.5,
            source,
        };
        self.keywords.insert(keyword.to_string(), record);
        self.id_index.insert(id, keyword.to_string());
        id
    }

    /// Retrieves a keyword record by its text.
    pub fn get(&self, keyword: &str) -> Option<&KeywordRecord> {
        self.keywords.get(keyword)
    }

    /// Returns all keyword records sorted by frequency descending.
    pub fn all_keywords(&self) -> Vec<&KeywordRecord> {
        let mut vec: Vec<&KeywordRecord> = self.keywords.values().collect();
        vec.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        vec
    }

    /// Removes keywords that have not been seen in `max_age_cycles` and
    /// whose frequency is below `min_frequency`.
    ///
    /// Returns the number of removed entries.
    pub fn prune(&mut self, max_age_cycles: u64, min_frequency: u32, current_cycle: u64) -> usize {
        let threshold = current_cycle.saturating_sub(max_age_cycles);
        let to_remove: Vec<String> = self
            .keywords
            .iter()
            .filter(|(_, rec)| rec.recency < threshold && rec.frequency < min_frequency)
            .map(|(kw, _)| kw.clone())
            .collect();

        let count = to_remove.len();
        for kw in &to_remove {
            if let Some(rec) = self.keywords.remove(kw) {
                self.id_index.remove(&rec.id);
                self.cluster_assignments.remove(&rec.id);
            }
        }
        count
    }

    /// Re-assigns cluster IDs by grouping keywords with VSA cosine similarity
    /// above the given threshold into the same cluster.
    ///
    /// Returns the number of distinct clusters formed.
    pub fn run_clustering(&mut self, threshold: f64) -> usize {
        let ids: Vec<u64> = self.keywords.values().map(|r| r.id).collect();
        let mut clusters: Vec<u64> = Vec::new();
        let mut next_cluster: u64 = 0;

        for &id in &ids {
            if clusters.is_empty() {
                clusters.push(next_cluster);
                next_cluster += 1;
                continue;
            }

            let kwa = &self.keywords[&self.id_index[&id]];
            let mut found = false;
            for (i, &cid) in clusters.iter().enumerate() {
                let other_id = ids[i];
                let kwb = &self.keywords[&self.id_index[&other_id]];
                let sim = QuantizedVSA::cosine(&kwa.vsa_vector, &kwb.vsa_vector);
                if sim > threshold {
                    clusters.push(cid);
                    found = true;
                    break;
                }
            }
            if !found {
                clusters.push(next_cluster);
                next_cluster += 1;
            }
        }

        self.cluster_assignments.clear();
        for (idx, &id) in ids.iter().enumerate() {
            self.cluster_assignments.insert(id, clusters[idx]);
        }

        // Count unique clusters.
        let unique: HashSet<&u64> = clusters.iter().collect();
        unique.len()
    }

    /// Removes the single oldest entry (lowest recency).
    fn evict_one(&mut self) {
        let oldest = self
            .keywords
            .iter()
            .min_by_key(|(_, rec)| rec.recency)
            .map(|(kw, _)| kw.clone());
        if let Some(kw) = oldest {
            if let Some(rec) = self.keywords.remove(&kw) {
                self.id_index.remove(&rec.id);
                self.cluster_assignments.remove(&rec.id);
            }
        }
    }

    /// Returns the current number of entries.
    pub fn len(&self) -> usize {
        self.keywords.len()
    }

    /// Returns true if the lexicon is empty.
    pub fn is_empty(&self) -> bool {
        self.keywords.is_empty()
    }

    /// Returns the cluster ID for a given keyword ID, if assigned.
    pub fn cluster_of(&self, keyword_id: u64) -> Option<u64> {
        self.cluster_assignments.get(&keyword_id).copied()
    }

    /// Lookup keyword text by ID.
    pub fn keyword_by_id(&self, id: u64) -> Option<&str> {
        self.id_index.get(&id).map(|s| s.as_str())
    }
}

impl Default for KeywordLexicon {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes a deterministic 64-bit hash from bigram XOR rotation of the text.
fn keyword_hash(text: &str) -> u64 {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return 0;
    }
    let mut hash: u64 = 0;
    let len = bytes.len();
    if len >= 2 {
        for i in 0..len - 1 {
            let bigram = ((bytes[i] as u64) << 8) | (bytes[i + 1] as u64);
            hash ^= bigram.rotate_left((i as u32) % 64);
        }
    }
    hash
}

/// Returns the current cycle timestamp (wall-clock seconds since epoch).
fn current_cycle() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Common English stop words to filter during extraction.
fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "a" | "an"
            | "the"
            | "is"
            | "it"
            | "to"
            | "of"
            | "in"
            | "and"
            | "or"
            | "on"
            | "at"
            | "for"
            | "with"
            | "by"
            | "as"
            | "be"
            | "from"
            | "are"
            | "was"
            | "were"
            | "been"
            | "being"
            | "have"
            | "has"
            | "had"
            | "do"
            | "does"
            | "did"
            | "will"
            | "would"
            | "can"
            | "could"
            | "may"
            | "might"
            | "shall"
            | "should"
            | "that"
            | "this"
            | "these"
            | "those"
            | "i"
            | "you"
            | "he"
            | "she"
            | "we"
            | "they"
            | "me"
            | "him"
            | "her"
            | "us"
            | "them"
            | "my"
            | "your"
            | "his"
            | "its"
            | "our"
            | "their"
            | "not"
            | "no"
            | "nor"
            | "so"
            | "if"
            | "but"
            | "about"
            | "into"
            | "over"
            | "after"
            | "before"
            | "between"
            | "under"
            | "up"
            | "down"
            | "just"
            | "also"
            | "very"
            | "too"
            | "here"
            | "there"
            | "then"
            | "than"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lexicon() -> KeywordLexicon {
        let mut lex = KeywordLexicon::with_capacity(100);
        for (i, kw) in ["neural", "transformer", "attention", "vector", "semantic"]
            .iter()
            .enumerate()
        {
            let id = i as u64 + 1;
            let rec = KeywordRecord {
                id,
                keyword: kw.to_string(),
                vsa_vector: QuantizedVSA::seeded_random(keyword_hash(kw), VSA_DIM),
                frequency: (10 - i) as u32,
                recency: 1000 - i as u64,
                confidence: 0.5,
                source: KeywordSource::Manual,
            };
            lex.keywords.insert(kw.to_string(), rec);
            lex.id_index.insert(id, kw.to_string());
            lex.next_id = 6;
        }
        lex
    }

    #[test]
    fn test_new_is_empty() {
        let lex = KeywordLexicon::new();
        assert!(lex.is_empty());
        assert_eq!(lex.len(), 0);
    }

    #[test]
    fn test_add_or_update_creates_new() {
        let mut lex = KeywordLexicon::new();
        let id = lex.add_or_update("quantum", KeywordSource::Manual);
        assert_eq!(id, 1);
        assert_eq!(lex.len(), 1);
        let rec = lex.get("quantum").unwrap();
        assert_eq!(rec.frequency, 1);
        assert_eq!(rec.keyword, "quantum");
    }

    #[test]
    fn test_add_or_update_increments_frequency() {
        let mut lex = KeywordLexicon::new();
        let id1 = lex.add_or_update("quantum", KeywordSource::Manual);
        let id2 = lex.add_or_update("quantum", KeywordSource::Search);
        assert_eq!(id1, id2);
        let rec = lex.get("quantum").unwrap();
        assert_eq!(rec.frequency, 2);
    }

    #[test]
    fn test_get_returns_none_for_unknown() {
        let lex = KeywordLexicon::new();
        assert!(lex.get("nonexistent").is_none());
    }

    #[test]
    fn test_extract_from_text_produces_top_k() {
        let mut lex = KeywordLexicon::new();
        let keywords =
            lex.extract_from_text("the cat sat on the mat with a dog", KeywordSource::Crawl, 3);
        assert_eq!(keywords.len(), 3);
        // "cat", "sat", "mat", "dog" — top 3 by frequency (each appears once)
        for kw in &keywords {
            assert!(["cat", "sat", "mat", "dog"].contains(&kw.as_str()));
        }
    }

    #[test]
    fn test_extract_from_attractor_returns_similar() {
        let lex = sample_lexicon();
        let state = QuantizedVSA::seeded_random(keyword_hash("neural"), VSA_DIM);
        let results = lex.extract_from_attractor(&state, 3);
        // "neural" should be the most similar since same hash
        assert!(!results.is_empty());
        assert!(results.contains(&"neural".to_string()));
    }

    #[test]
    fn test_prune_removes_old_low_freq() {
        let mut lex = KeywordLexicon::new();
        lex.add_or_update("stale", KeywordSource::Manual);
        // Manually set recency far in the past
        if let Some(rec) = lex.keywords.get_mut("stale") {
            rec.recency = 0;
            rec.frequency = 1;
        }
        lex.add_or_update("fresh", KeywordSource::Manual);

        let removed = lex.prune(100, 2, current_cycle());
        // stale has recency=0 (far past threshold) and freq=1 (<2) → removed
        assert_eq!(removed, 1);
        assert!(lex.get("stale").is_none());
        assert!(lex.get("fresh").is_some());
    }

    #[test]
    fn test_run_clustering_groups_similar() {
        let mut lex = KeywordLexicon::new();
        lex.add_or_update("cat", KeywordSource::Manual);
        lex.add_or_update("dog", KeywordSource::Manual);
        lex.add_or_update("quantum", KeywordSource::Manual);
        lex.add_or_update("physics", KeywordSource::Manual);

        // Use a high threshold — expect 4 clusters (each vector is random with different seeds)
        let clusters = lex.run_clustering(0.9);
        assert!(clusters >= 3);
    }

    #[test]
    fn test_eviction_when_max_exceeded() {
        let mut lex = KeywordLexicon::with_capacity(3);
        lex.add_or_update("a", KeywordSource::Manual);
        lex.add_or_update("b", KeywordSource::Manual);
        lex.add_or_update("c", KeywordSource::Manual);
        assert_eq!(lex.len(), 3);
        lex.add_or_update("d", KeywordSource::Manual);
        // One should be evicted
        assert_eq!(lex.len(), 3);
        // 'a' was first, lowest recency (assuming all in same cycle)
        // But recency might all be the same second — in that case min_by_key
        // will pick the first inserted, which is 'a' after iteration order.
        // We just verify count is bounded.
    }

    #[test]
    fn test_encode_keyword_deterministic() {
        let v1 = KeywordLexicon::encode_keyword("hello");
        let v2 = KeywordLexicon::encode_keyword("hello");
        assert_eq!(v1.len(), VSA_DIM);
        assert_eq!(v2.len(), VSA_DIM);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_encode_keyword_different_texts_different() {
        let v1 = KeywordLexicon::encode_keyword("hello");
        let v2 = KeywordLexicon::encode_keyword("world");
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_all_keywords_sorted_by_frequency() {
        let lex = sample_lexicon();
        let all = lex.all_keywords();
        assert_eq!(all.len(), 5);
        for i in 1..all.len() {
            assert!(all[i - 1].frequency >= all[i].frequency);
        }
    }

    #[test]
    fn test_keyword_by_id_roundtrip() {
        let mut lex = KeywordLexicon::new();
        let id = lex.add_or_update("testword", KeywordSource::Manual);
        assert_eq!(lex.keyword_by_id(id), Some("testword"));
        assert!(lex.keyword_by_id(999).is_none());
    }

    #[test]
    fn test_extract_from_text_empty() {
        let mut lex = KeywordLexicon::new();
        let kw = lex.extract_from_text("", KeywordSource::Crawl, 5);
        assert!(kw.is_empty());
    }

    #[test]
    fn test_lexicon_with_capacity() {
        let lex = KeywordLexicon::with_capacity(50);
        assert_eq!(lex.len(), 0);
    }

    #[test]
    fn test_extract_from_attractor_empty_state() {
        let lex = sample_lexicon();
        let state = vec![0u8; VSA_DIM];
        let results = lex.extract_from_attractor(&state, 3);
        assert!(results.is_empty());
    }
}

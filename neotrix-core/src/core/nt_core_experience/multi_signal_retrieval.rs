use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

static STOP_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "as", "is", "was", "are", "were", "be", "been", "being", "have", "has", "had", "do",
    "does", "did", "will", "would", "can", "could", "shall", "should", "may", "might", "it", "its",
    "this", "that", "these", "those", "i", "me", "my", "we", "our", "you", "your", "he", "him",
    "his", "she", "her", "they", "them", "their", "not", "no", "nor", "so", "if",
];

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub id: u64,
    pub text: String,
    pub embedding: Option<Vec<u8>>,
    pub keywords: Vec<String>,
    pub entities: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct RetrievalSignals {
    pub semantic_score: f64,
    pub keyword_score: f64,
    pub entity_score: f64,
}

#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub entry: MemoryEntry,
    pub signals: RetrievalSignals,
    pub combined_score: f64,
}

impl RetrievalSignals {
    pub fn fuse(&self, weights: &[f64; 3]) -> f64 {
        weights[0] * self.semantic_score
            + weights[1] * self.keyword_score
            + weights[2] * self.entity_score
    }
}

pub struct MultiSignalRetriever {
    entries: Vec<MemoryEntry>,
    weights: [f64; 3],
    entity_boost_factor: f64,
}

impl Default for MultiSignalRetriever {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiSignalRetriever {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            weights: [0.4, 0.35, 0.25],
            entity_boost_factor: 1.5,
        }
    }

    pub fn with_weights(weights: [f64; 3]) -> Self {
        Self {
            entries: Vec::new(),
            weights,
            entity_boost_factor: 1.5,
        }
    }

    pub fn set_weight(&mut self, signal: usize, weight: f64) {
        if signal < 3 {
            self.weights[signal] = weight.clamp(0.0, 1.0);
        }
    }

    pub fn weights(&self) -> &[f64; 3] {
        &self.weights
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize_weights(&mut self) {
        let sum: f64 = self.weights.iter().sum();
        if sum > 0.0 {
            for w in &mut self.weights {
                *w /= sum;
            }
        }
    }

    pub fn add_entry(&mut self, entry: MemoryEntry) {
        self.entries.push(entry);
    }

    pub fn add_entry_with_embedding(&mut self, text: String, embedding: Vec<u8>) {
        let id = self.entries.len() as u64;
        let keywords = self.extract_keywords(&text);
        let entities = self.extract_entities(&text);
        self.entries.push(MemoryEntry {
            id,
            text,
            embedding: Some(embedding),
            keywords,
            entities,
            timestamp: chrono::Utc::now(),
        });
    }

    pub fn search(&self, query: &str, query_vsa: Option<&[u8]>, k: usize) -> Vec<RetrievalResult> {
        if self.entries.is_empty() || k == 0 {
            return Vec::new();
        }

        let query_terms = self.extract_keywords(query);
        let query_entities = self.extract_entities(query);

        let mut scored: Vec<RetrievalResult> = self
            .entries
            .iter()
            .map(|entry| {
                let semantic_score = match query_vsa {
                    Some(qvsa) => self.semantic_score(entry, qvsa),
                    None => 0.0,
                };
                let keyword_score = self.keyword_score(entry, &query_terms);
                let entity_score = self.entity_score(entry, &query_entities);

                // Apply entity boost when query entities match entry entities exactly
                let entity_boost = if entity_score > 0.5 {
                    self.entity_boost_factor
                } else {
                    1.0
                };

                let signals = RetrievalSignals {
                    semantic_score,
                    keyword_score,
                    entity_score,
                };
                let combined_score = signals.fuse(&self.weights) * entity_boost;

                RetrievalResult {
                    entry: entry.clone(),
                    signals,
                    combined_score,
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(k);
        scored
    }

    /// Update weights based on retrieval success feedback.
    /// `reward` > 0 means the top-k results were relevant.
    /// Uses simple delta rule for online weight adaptation.
    pub fn update_weights_from_feedback(&mut self, reward: f64, signals: &RetrievalSignals) {
        let lr = 0.01;
        let pred = signals.fuse(&self.weights);
        let error = reward - pred;
        let s = [
            signals.semantic_score,
            signals.keyword_score,
            signals.entity_score,
        ];
        for i in 0..3 {
            self.weights[i] = (self.weights[i] + lr * error * s[i]).clamp(0.01, 0.99);
        }
        self.normalize_weights();
    }

    fn semantic_score(&self, entry: &MemoryEntry, query_vsa: &[u8]) -> f64 {
        match &entry.embedding {
            Some(entry_vsa) => QuantizedVSA::similarity(entry_vsa, query_vsa),
            None => 0.0,
        }
    }

    fn keyword_score(&self, entry: &MemoryEntry, query_terms: &[String]) -> f64 {
        if query_terms.is_empty() {
            return 0.0;
        }

        let k1 = 1.2;
        let b = 0.75;

        let doc_len = entry.text.split_whitespace().count() as f64;
        let avgdl = if self.entries.is_empty() {
            doc_len
        } else {
            let total: usize = self
                .entries
                .iter()
                .map(|e| e.text.split_whitespace().count())
                .sum();
            total as f64 / self.entries.len() as f64
        };

        let n = self.entries.len() as f64;
        let mut total_score = 0.0;

        for term in query_terms {
            let tf = entry.text.matches(term.as_str()).count() as f64;
            if tf == 0.0 {
                continue;
            }

            let doc_freq = self
                .entries
                .iter()
                .filter(|e| e.text.contains(term.as_str()))
                .count() as f64;

            let idf = if doc_freq == 0.0 {
                0.0
            } else {
                ((n - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln() + 1.0
            };

            let score =
                idf * (k1 + 1.0) * tf / (k1 * (1.0 - b + b * doc_len / avgdl.max(1.0)) + tf);
            total_score += score;
        }

        total_score
    }

    fn entity_score(&self, entry: &MemoryEntry, query_entities: &[String]) -> f64 {
        if query_entities.is_empty() || entry.entities.is_empty() {
            return 0.0;
        }

        let mut intersection = 0;
        for qe in query_entities {
            for ee in &entry.entities {
                if qe.eq_ignore_ascii_case(ee) || levenshtein_distance(qe, ee) < 2 {
                    intersection += 1;
                    break;
                }
            }
        }

        let union = query_entities.len() + entry.entities.len() - intersection;
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    fn extract_keywords(&self, text: &str) -> Vec<String> {
        text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .filter(|s| !STOP_WORDS.contains(&s.as_str()) && s.len() >= 2)
            .collect()
    }

    fn extract_entities(&self, text: &str) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut entities = Vec::new();
        let mut i = 0;

        while i < words.len() {
            let word = words[i].trim_matches(|c: char| c.is_ascii_punctuation());
            if word.is_empty() {
                i += 1;
                continue;
            }

            if word.chars().next().map_or(false, |c| c.is_uppercase()) {
                let mut seq = vec![word.to_string()];
                i += 1;

                while i < words.len() {
                    let next = words[i].trim_matches(|c: char| c.is_ascii_punctuation());
                    if next.is_empty() {
                        i += 1;
                        continue;
                    }
                    if next.chars().next().map_or(false, |c| c.is_uppercase()) {
                        seq.push(next.to_string());
                        i += 1;
                    } else {
                        break;
                    }
                }

                if seq.len() >= 2 {
                    entities.push(seq.join(" "));
                }
            } else {
                i += 1;
            }
        }

        entities
    }
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    if a_len > b_len {
        return levenshtein_distance(b, a);
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    let mut prev: Vec<usize> = (0..=a_len).collect();
    let mut curr = vec![0usize; a_len + 1];

    for j in 1..=b_len {
        curr[0] = j;
        for i in 1..=a_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[i] = (curr[i - 1] + 1).min(prev[i] + 1).min(prev[i - 1] + cost);
        }
        (prev, curr) = (curr, prev);
    }

    prev[a_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: u64, text: &str, embedding: Option<Vec<u8>>) -> MemoryEntry {
        let kw: Vec<String> = text
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .filter(|w| !STOP_WORDS.contains(&w.as_str()))
            .collect();

        let words: Vec<&str> = text.split_whitespace().collect();
        let mut entities = Vec::new();
        let mut i = 0;
        while i < words.len() {
            let w = words[i].trim_matches(|c: char| c.is_ascii_punctuation());
            if !w.is_empty() && w.chars().next().map_or(false, |c| c.is_uppercase()) {
                let mut seq = vec![w.to_string()];
                i += 1;
                while i < words.len() {
                    let n = words[i].trim_matches(|c: char| c.is_ascii_punctuation());
                    if n.is_empty() {
                        i += 1;
                        continue;
                    }
                    if n.chars().next().map_or(false, |c| c.is_uppercase()) {
                        seq.push(n.to_string());
                        i += 1;
                    } else {
                        break;
                    }
                }
                if seq.len() >= 2 {
                    entities.push(seq.join(" "));
                }
            } else {
                i += 1;
            }
        }

        MemoryEntry {
            id,
            text: text.to_string(),
            embedding,
            keywords: kw,
            entities,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_add_and_search_semantic() {
        let mut retriever = MultiSignalRetriever::new();
        retriever.add_entry(make_entry(0, "The quantum computer processed data", None));
        retriever.add_entry(make_entry(1, "The cat sat on the mat", None));

        let results = retriever.search("quantum computer", None, 5);
        assert_eq!(results.len(), 2);
        assert!(results[0].signals.keyword_score > 0.0);
    }

    #[test]
    fn test_keyword_matching_boost() {
        let mut retriever = MultiSignalRetriever::new();
        retriever.add_entry(make_entry(0, "The fox jumped over the fence", None));
        retriever.add_entry(make_entry(1, "The brown fox ate grapes", None));
        retriever.add_entry(make_entry(2, "The weather is nice today", None));

        let results = retriever.search("brown fox", None, 5);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].entry.id, 1);
        assert!(results[0].signals.keyword_score > results[2].signals.keyword_score);
    }

    #[test]
    fn test_entity_matching() {
        let mut retriever = MultiSignalRetriever::new();
        retriever.add_entry(make_entry(
            0,
            "Alice met Bob at the Google office in Mountain View",
            None,
        ));
        retriever.add_entry(make_entry(
            1,
            "The quick brown fox jumps over the lazy dog",
            None,
        ));

        let results = retriever.search("Alice Bob Google", None, 5);
        assert_eq!(results.len(), 2);
        assert!(results[0].signals.entity_score > results[1].signals.entity_score);
    }

    #[test]
    fn test_fusion_weighting() {
        let signals_a = RetrievalSignals {
            semantic_score: 0.9,
            keyword_score: 0.3,
            entity_score: 0.8,
        };
        let signals_b = RetrievalSignals {
            semantic_score: 0.2,
            keyword_score: 0.9,
            entity_score: 0.1,
        };

        let fused_a = signals_a.fuse(&[0.5, 0.3, 0.2]);
        let fused_b = signals_b.fuse(&[0.5, 0.3, 0.2]);

        let expected_a = 0.5 * 0.9 + 0.3 * 0.3 + 0.2 * 0.8;
        let expected_b = 0.5 * 0.2 + 0.3 * 0.9 + 0.2 * 0.1;

        assert!((fused_a - expected_a).abs() < 1e-10);
        assert!((fused_b - expected_b).abs() < 1e-10);
    }

    #[test]
    fn test_search_returns_top_k() {
        let mut retriever = MultiSignalRetriever::new();
        for i in 0..10 {
            retriever.add_entry(make_entry(
                i as u64,
                &format!("document number {} about machine learning", i),
                None,
            ));
        }

        let results = retriever.search("machine learning", None, 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_empty_retriever_returns_empty() {
        let retriever: MultiSignalRetriever = MultiSignalRetriever::new();
        let results = retriever.search("anything", None, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_entity_boost_ranks_entity_match_higher() {
        let mut retriever = MultiSignalRetriever::new();
        retriever.add_entry(make_entry(
            0,
            "John works at Acme Corporation in Chicago",
            None,
        ));
        retriever.add_entry(make_entry(1, "The weather in Chicago is cold today", None));

        let results = retriever.search("John Acme Corporation", None, 5);
        assert_eq!(results.len(), 2);
        assert!(results[0].signals.entity_score > 0.5);
        assert!(results[0].combined_score > results[1].combined_score);
    }

    #[test]
    fn test_update_weights_from_feedback_shifts_toward_high_signal() {
        let mut retriever = MultiSignalRetriever::new();
        let original = *retriever.weights();

        let signals = RetrievalSignals {
            semantic_score: 0.9,
            keyword_score: 0.1,
            entity_score: 0.1,
        };
        retriever.update_weights_from_feedback(1.0, &signals);

        let updated = retriever.weights();
        assert!(updated[0] > original[0]);
        assert!(updated[1] < original[1]);
        assert!(updated[2] < original[2]);
    }

    #[test]
    fn test_normalize_weights_sums_to_one() {
        let mut retriever = MultiSignalRetriever::with_weights([0.8, 0.3, 0.1]);
        let sum_before: f64 = retriever.weights().iter().sum();
        assert!((sum_before - 1.2).abs() < 1e-10);

        retriever.normalize_weights();
        let sum_after: f64 = retriever.weights().iter().sum();
        assert!((sum_after - 1.0).abs() < 1e-10);

        for w in retriever.weights() {
            assert!((0.0..=1.0).contains(w));
        }
    }

    #[test]
    fn test_combined_score_includes_entity_boost_when_entity_score_high() {
        let mut retriever = MultiSignalRetriever::new();
        retriever.add_entry(make_entry(
            0,
            "Alice and Bob discussed the merger at Goldman Sachs",
            None,
        ));
        retriever.add_entry(make_entry(1, "Alice likes coffee and Bob likes tea", None));

        let results = retriever.search("Alice Bob Goldman Sachs", None, 5);
        assert_eq!(results.len(), 2);

        let goldman = &results[0];
        assert!(goldman.signals.entity_score > results[1].signals.entity_score);

        let expected_boosted =
            goldman.signals.fuse(retriever.weights()) * retriever.entity_boost_factor;
        assert!((goldman.combined_score - expected_boosted).abs() < 1e-10);
    }
}

use std::collections::HashMap;
use crate::core::nt_core_bank::ReasoningBankStats;

#[derive(Debug, Clone)]
pub struct MemoryIterationResult {
    pub before: ReasoningBankStats,
    pub after: ReasoningBankStats,
    pub merged_count: usize,
    pub pruned_count: usize,
    pub replayed_count: usize,
    pub promoted_count: usize,
    pub expired_count: usize,
}

pub(crate) const K1: f64 = 1.5;
pub(crate) const B: f64 = 0.75;
pub(crate) const RRF_K: f64 = 60.0;

#[derive(Debug, Clone)]
pub(crate) struct Bm25Document {
    pub(crate) id: String,
    pub(crate) text: String,
}

#[derive(Debug, Clone)]
struct DocEntry {
    doc_id: String,
    field_length: usize,
    term_freqs: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub(crate) struct Bm25Index {
    #[allow(dead_code)]
    df: HashMap<String, usize>,
    idf: HashMap<String, f64>,
    docs: Vec<DocEntry>,
    avg_doc_len: f64,
    n_docs: usize,
}

impl Bm25Index {
    pub(crate) fn empty() -> Self {
        Self { df: HashMap::new(), idf: HashMap::new(), docs: Vec::new(), avg_doc_len: 0.0, n_docs: 0 }
    }

    pub(crate) fn build(docs: &[Bm25Document]) -> Self {
        let entries: Vec<DocEntry> = docs.iter().map(|d| {
            let tokens = tokenize(&d.text);
            let mut tf: HashMap<String, f64> = HashMap::new();
            for t in &tokens {
                *tf.entry(t.clone()).or_insert(0.0) += 1.0;
            }
            DocEntry { doc_id: d.id.clone(), field_length: tokens.len(), term_freqs: tf }
        }).collect();

        let n = entries.len();
        let mut df: HashMap<String, usize> = HashMap::new();
        for entry in &entries {
            for term in entry.term_freqs.keys() {
                *df.entry(term.clone()).or_insert(0) += 1;
            }
        }

        let avgdl = if n > 0 {
            entries.iter().map(|e| e.field_length as f64).sum::<f64>() / n as f64
        } else {
            0.0
        };

        let idf: HashMap<String, f64> = df.iter().map(|(term, &doc_freq)| {
            (term.clone(), ((n as f64 - doc_freq as f64 + 0.5) / (doc_freq as f64 + 0.5) + 1.0).ln())
        }).collect();

        Self { df, idf, docs: entries, avg_doc_len: avgdl, n_docs: n }
    }

    pub(crate) fn search(&self, query: &str, k: usize) -> Vec<(f64, String)> {
        if self.n_docs == 0 { return Vec::new(); }
        let query_terms = tokenize(query);
        if query_terms.is_empty() { return Vec::new(); }

        let mut scores: Vec<(f64, usize)> = (0..self.docs.len()).map(|i| {
            let entry = &self.docs[i];
            let mut score = 0.0;
            for qt in &query_terms {
                let idf = self.idf.get(qt.as_str()).copied().unwrap_or(0.0);
                if idf <= 0.0 { continue; }
                let tf = entry.term_freqs.get(qt.as_str()).copied().unwrap_or(0.0);
                if tf <= 0.0 { continue; }
                score += idf * (tf * (K1 + 1.0)) / (tf + K1 * (1.0 - B + B * entry.field_length as f64 / self.avg_doc_len));
            }
            (score, i)
        }).collect();

        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scores.into_iter().filter(|(s, _)| *s > 0.0).take(k).map(|(s, i)| (s, self.docs[i].doc_id.clone())).collect()
    }
}

pub(crate) fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|s| !s.is_empty() && s.len() >= 2)
        .map(|s| s.to_string())
        .collect()
}

pub(crate) fn rrf_fuse(results: &[Vec<(f64, String)>]) -> Vec<(f64, String)> {
    if results.is_empty() { return Vec::new(); }
    let mut agg: HashMap<String, f64> = HashMap::new();
    for ranklist in results {
        for (rank, (_score, id)) in ranklist.iter().enumerate() {
            *agg.entry(id.clone()).or_insert(0.0) += 1.0 / (RRF_K + rank as f64 + 1.0);
        }
    }
    let mut fused: Vec<(f64, String)> = agg.into_iter().map(|(id, score)| (score, id)).collect();
    fused.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize("hello world");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_filters_short_words() {
        let tokens = tokenize("a an the cat");
        assert!(!tokens.contains(&"a".to_string()));
        assert!(tokens.contains(&"the".to_string()));
        assert!(tokens.contains(&"cat".to_string()));
    }

    #[test]
    fn test_tokenize_lowercases() {
        let tokens = tokenize("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_punctuation() {
        let tokens = tokenize("hello, world!");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_hyphenated() {
        let tokens = tokenize("state-of-the-art");
        assert!(tokens.contains(&"state-of-the-art".to_string()) || tokens.contains(&"state".to_string()));
    }

    #[test]
    fn test_bm25_search_returns_results() {
        let docs = vec![
            Bm25Document { id: "1".into(), text: "rust programming language".into() },
            Bm25Document { id: "2".into(), text: "python programming language".into() },
        ];
        let index = Bm25Index::build(&docs);
        let results = index.search("rust", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].1, "1");
    }

    #[test]
    fn test_bm25_search_empty_index() {
        let index = Bm25Index::build(&[]);
        let results = index.search("rust", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_bm25_search_no_match() {
        let docs = vec![
            Bm25Document { id: "1".into(), text: "rust programming".into() },
        ];
        let index = Bm25Index::build(&docs);
        let results = index.search("golang", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_bm25_empty_query() {
        let docs = vec![Bm25Document { id: "1".into(), text: "hello world".into() }];
        let index = Bm25Index::build(&docs);
        let results = index.search("", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rrf_fuse_single_list() {
        let list = vec![vec![(10.0, "a".into()), (5.0, "b".into())]];
        let fused = rrf_fuse(&list);
        assert_eq!(fused.len(), 2);
    }

    #[test]
    fn test_rrf_fuse_empty() {
        let fused = rrf_fuse(&[]);
        assert!(fused.is_empty());
    }

    #[test]
    fn test_rrf_fuse_ranks_high_in_both() {
        let list1 = vec![(10.0, "a".into()), (5.0, "b".into())];
        let list2 = vec![(8.0, "a".into()), (6.0, "c".into())];
        let fused = rrf_fuse(&[list1, list2]);
        assert!(fused[0].1 == "a" || fused[0].1 == "a");
    }

    #[test]
    fn test_bm25_build_and_search_multiple_terms() {
        let docs = vec![
            Bm25Document { id: "d1".into(), text: "the quick brown fox".into() },
            Bm25Document { id: "d2".into(), text: "lazy dog sleeping".into() },
        ];
        let index = Bm25Index::build(&docs);
        let results = index.search("quick fox", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "d1");
    }
}

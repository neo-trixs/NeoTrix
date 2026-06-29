use std::collections::HashMap;

const K1: f64 = 1.5;
const B: f64 = 0.75;
const RRF_K: f64 = 60.0;

#[derive(Debug, Clone)]
pub struct Bm25Document {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone)]
struct DocEntry {
    doc_id: String,
    field_length: usize,
    term_freqs: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct Bm25Index {
    idf: HashMap<String, f64>,
    docs: Vec<DocEntry>,
    avg_doc_len: f64,
    n_docs: usize,
}

impl Bm25Index {
    pub fn empty() -> Self {
        Self {
            idf: HashMap::new(),
            docs: Vec::new(),
            avg_doc_len: 0.0,
            n_docs: 0,
        }
    }

    pub fn build(docs: &[Bm25Document]) -> Self {
        let entries: Vec<DocEntry> = docs
            .iter()
            .map(|d| {
                let tokens = tokenize(&d.text);
                let mut tf: HashMap<String, f64> = HashMap::new();
                for t in &tokens {
                    *tf.entry(t.clone()).or_insert(0.0) += 1.0;
                }
                DocEntry {
                    doc_id: d.id.clone(),
                    field_length: tokens.len(),
                    term_freqs: tf,
                }
            })
            .collect();

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

        let idf: HashMap<String, f64> = df
            .iter()
            .map(|(term, &doc_freq)| {
                (
                    term.clone(),
                    ((n as f64 - doc_freq as f64 + 0.5) / (doc_freq as f64 + 0.5) + 1.0).ln(),
                )
            })
            .collect();

        Self {
            idf,
            docs: entries,
            avg_doc_len: avgdl,
            n_docs: n,
        }
    }

    pub fn search(&self, query: &str, k: usize) -> Vec<(f64, String)> {
        if self.n_docs == 0 {
            return Vec::new();
        }
        let query_terms = tokenize(query);
        if query_terms.is_empty() {
            return Vec::new();
        }

        let mut scores: Vec<(f64, usize)> = (0..self.docs.len())
            .map(|i| {
                let entry = &self.docs[i];
                let mut score = 0.0;
                for qt in &query_terms {
                    let idf = self.idf.get(qt.as_str()).copied().unwrap_or(0.0);
                    if idf <= 0.0 {
                        continue;
                    }
                    let tf = entry.term_freqs.get(qt.as_str()).copied().unwrap_or(0.0);
                    if tf <= 0.0 {
                        continue;
                    }
                    score += idf * (tf * (K1 + 1.0))
                        / (tf + K1 * (1.0 - B + B * entry.field_length as f64 / self.avg_doc_len));
                }
                (score, i)
            })
            .collect();

        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scores
            .into_iter()
            .filter(|(s, _)| *s > 0.0)
            .take(k)
            .map(|(s, i)| (s, self.docs[i].doc_id.clone()))
            .collect()
    }

    pub fn n_docs(&self) -> usize {
        self.n_docs
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|s| !s.is_empty() && s.len() >= 2)
        .map(|s| s.to_string())
        .collect()
}

pub fn rrf_fuse(results: &[Vec<(f64, String)>]) -> Vec<(f64, String)> {
    if results.is_empty() {
        return Vec::new();
    }
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

    fn make_test_docs() -> Vec<Bm25Document> {
        vec![
            Bm25Document {
                id: "1".into(),
                text: "Rust memory safety ownership borrowing lifetimes".into(),
            },
            Bm25Document {
                id: "2".into(),
                text: "async await tokio async runtime concurrency".into(),
            },
            Bm25Document {
                id: "3".into(),
                text: "React hooks useState useEffect component lifecycle".into(),
            },
            Bm25Document {
                id: "4".into(),
                text: "Python async await asyncio event loop concurrency".into(),
            },
            Bm25Document {
                id: "5".into(),
                text: "TypeScript types interfaces generics type safety".into(),
            },
        ]
    }

    #[test]
    fn test_bm25_basic_search() {
        let docs = make_test_docs();
        let index = Bm25Index::build(&docs);
        let results = index.search("async concurrency", 3);
        assert!(!results.is_empty(), "should find async docs");
        assert_eq!(
            results[0].1, "2",
            "doc 2 should be top for async concurrency"
        );
    }

    #[test]
    fn test_bm25_memory_safety() {
        let docs = make_test_docs();
        let index = Bm25Index::build(&docs);
        let results = index.search("memory ownership", 3);
        assert!(!results.is_empty());
        assert_eq!(
            results[0].1, "1",
            "doc 1 should be top for memory ownership"
        );
    }

    #[test]
    fn test_bm25_empty_index() {
        let index = Bm25Index::empty();
        let results = index.search("anything", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_tokenize_splits() {
        let tokens = tokenize("Rust-style async/await + tokio");
        assert!(tokens.contains(&"rust-style".to_string()));
        assert!(tokens.contains(&"async".to_string()));
        assert!(tokens.contains(&"tokio".to_string()));
    }

    #[test]
    fn test_rrf_fuse_merges_rankings() {
        let v1: Vec<(f64, String)> = vec![(0.9, "a".into()), (0.8, "b".into()), (0.7, "c".into())];
        let v2: Vec<(f64, String)> =
            vec![(0.95, "b".into()), (0.85, "a".into()), (0.6, "d".into())];
        let fused = rrf_fuse(&[v1, v2]);
        assert!(!fused.is_empty());
        let top = fused[0].1.clone();
        assert!(
            top == "a" || top == "b",
            "a or b should be top, got {}",
            top
        );
    }
}

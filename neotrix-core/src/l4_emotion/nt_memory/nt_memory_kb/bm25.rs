use std::collections::HashMap;

const K1: f64 = 1.5;
const B: f64 = 0.75;
// 本模块原有的 `const RRF_K: f64 = 60.0` 已删除（2026-09-30）：真身统一在
// `neotrix_types::core::nt_core_bank::RRF_K`（全仓曾有 5 份同值副本）。
// 它此前只被本模块那份 `rrf_fuse` 副本使用；该副本收敛为引用 types 实现后，
// 此处即成零使用的重复常量。RRF_K 是公开标准常数（Cormack et al. 2009），
// 不属于 BM25 的 K1/B 参数，故 K1/B 保留本地定义（它们是本实现的调参面）。

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
    recall_weight: f64,
}

#[derive(Debug, Clone)]
pub struct Bm25Index {
    df: HashMap<String, usize>,
    docs: Vec<DocEntry>,
    avg_doc_len: f64,
    n_docs: usize,
}

impl Bm25Index {
    pub fn empty() -> Self {
        Self {
            df: HashMap::new(),
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
                    recall_weight: 1.0,
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

        Self {
            df,
            docs: entries,
            avg_doc_len: avgdl,
            n_docs: n,
        }
    }

    pub fn add_document(&mut self, doc: &Bm25Document) {
        let tokens = tokenize(&doc.text);
        let mut tf: HashMap<String, f64> = HashMap::new();
        for t in &tokens {
            *tf.entry(t.clone()).or_insert(0.0) += 1.0;
        }
        for term in tf.keys() {
            *self.df.entry(term.clone()).or_insert(0) += 1;
        }
        let field_len = tokens.len();
        let n = self.n_docs;
        self.avg_doc_len = if n == 0 {
            field_len as f64
        } else {
            (self.avg_doc_len * n as f64 + field_len as f64) / (n + 1) as f64
        };
        self.docs.push(DocEntry {
            doc_id: doc.id.clone(),
            field_length: field_len,
            term_freqs: tf,
            recall_weight: 1.0,
        });
        self.n_docs += 1;
    }

    pub fn merge(&mut self, other: Self) {
        if other.n_docs == 0 {
            return;
        }
        let total_docs = self.n_docs + other.n_docs;
        let avg_sum =
            self.avg_doc_len * self.n_docs as f64 + other.avg_doc_len * other.n_docs as f64;
        self.avg_doc_len = avg_sum / total_docs as f64;
        for (term, count) in other.df {
            *self.df.entry(term).or_insert(0) += count;
        }
        self.docs.extend(other.docs);
        self.n_docs = total_docs;
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
                    let doc_freq = self.df.get(qt.as_str()).copied().unwrap_or(0);
                    if doc_freq == 0 {
                        continue;
                    }
                    let idf = ((self.n_docs as f64 - doc_freq as f64 + 0.5)
                        / (doc_freq as f64 + 0.5)
                        + 1.0)
                        .ln();
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
            .map(|(s, i)| (s * self.docs[i].recall_weight, self.docs[i].doc_id.clone()))
            .collect()
    }

    pub fn n_docs(&self) -> usize {
        self.n_docs
    }
}

// `tokenize` 与 `rrf_fuse` 此前与 `neotrix-types` 各有一份**逐字相同**的副本
// （`nt_diverge.py` 实测 owner 同为 `(free)`、归一化后全等）⇒ 收敛为单一真身。
// 必要性：逐字相同的副本**只会各自漂移**（本会话已实测 `now_ts` 13 份里 2 份 panic）。
// 本模块下方的 `mod tests` **保留未删** —— 它验的是行为，改实现不该让测试消失。
// `pub` 而非 `use`：`rrf_fuse` 经 `bm25::rrf_fuse` 被 `nt_pure_fns.rs` 调用，
// 是本模块的公开 API（首版写成私有 `use` 触发 E0603，由编译器抓出）。
pub use neotrix_types::core::nt_core_bank::{rrf_fuse, tokenize};


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

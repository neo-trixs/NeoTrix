use std::collections::HashMap;
use crate::core::nt_core_bank::{MemoryDetailedStats, ReasoningBankStats};

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

#[derive(Debug, Clone)]
pub struct ConsolidationReport {
    pub iteration_result: MemoryIterationResult,
    pub lifecycle_promotions: usize,
    pub lifecycle_demotions: usize,
    pub lifecycle_archives: usize,
    pub lifecycle_evictions: usize,
    pub decays_applied: usize,
    pub tier_stats_before: MemoryDetailedStats,
    pub tier_stats_after: MemoryDetailedStats,
}

impl std::ops::Deref for ConsolidationReport {
    type Target = MemoryIterationResult;
    fn deref(&self) -> &Self::Target {
        &self.iteration_result
    }
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
    idf: HashMap<String, f64>,
    docs: Vec<DocEntry>,
    avg_doc_len: f64,
    n_docs: usize,
}

impl Bm25Index {
    pub(crate) fn empty() -> Self {
        Self { idf: HashMap::new(), docs: Vec::new(), avg_doc_len: 0.0, n_docs: 0 }
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

        Self { idf, docs: entries, avg_doc_len: avgdl, n_docs: n }
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

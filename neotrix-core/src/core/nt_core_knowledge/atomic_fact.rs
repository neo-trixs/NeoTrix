use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

/// An atomic fact: a single unit of information that cannot be further decomposed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicFact {
    pub id: u64,
    pub text: String,
    pub vsa_hash: Vec<u8>,
    pub source: String,
    pub confidence: f64,
    pub contradictions: Vec<u64>,
    pub supporting_sources: Vec<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactStats {
    pub total_facts: usize,
    pub unique_sources: usize,
    pub contradictions: usize,
    pub average_confidence: f64,
}

/// Decomposes text into atomic facts using VSA-based sentence splitting and dedup.
#[derive(Debug, Clone)]
pub struct AtomicFactDecomposer {
    facts: Vec<AtomicFact>,
    next_id: u64,
    pub(crate) min_confidence: f64,
    max_facts_per_source: usize,
    redundancy_threshold: f64,
}

impl AtomicFactDecomposer {
    pub fn new() -> Self {
        Self {
            facts: Vec::new(),
            next_id: 1,
            min_confidence: 0.7,
            max_facts_per_source: 50,
            redundancy_threshold: 0.85,
        }
    }

    pub fn with_min_confidence(mut self, v: f64) -> Self {
        self.min_confidence = v;
        self
    }

    pub fn with_max_facts_per_source(mut self, v: usize) -> Self {
        self.max_facts_per_source = v;
        self
    }

    pub fn with_redundancy_threshold(mut self, v: f64) -> Self {
        self.redundancy_threshold = v;
        self
    }

    fn sentence_boundaries(text: &str) -> Vec<usize> {
        let mut boundaries = Vec::new();
        let mut in_period = false;
        for (i, ch) in text.char_indices() {
            if ch == '.' || ch == '!' || ch == '?' {
                in_period = true;
            } else if in_period {
                if ch.is_whitespace() || ch == '"' || ch == '\'' {
                    boundaries.push(i);
                }
                in_period = false;
            }
        }
        if !boundaries.is_empty() && boundaries.last().copied().unwrap_or(0) < text.len() {
            boundaries.push(text.len());
        }
        boundaries
    }

    fn split_sentences(text: &str) -> Vec<String> {
        let text = text.trim();
        if text.is_empty() {
            return Vec::new();
        }
        let boundaries = Self::sentence_boundaries(text);
        if boundaries.is_empty() {
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                return Vec::new();
            }
            return vec![trimmed];
        }
        let mut sentences = Vec::new();
        let mut start = 0;
        for &b in &boundaries {
            let s = text[start..b].trim().to_string();
            if !s.is_empty() {
                sentences.push(s);
            }
            start = b;
        }
        let remaining = text[start..].trim().to_string();
        if !remaining.is_empty() {
            sentences.push(remaining);
        }
        sentences
    }

    pub(crate) fn hash_text(text: &str) -> Vec<u8> {
        let seed: u64 = text
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        QuantizedVSA::seeded_random(seed, 64)
    }

    fn vsa_similarity(a: &[u8], b: &[u8]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dist: u32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x ^ y).count_ones())
            .sum();
        let total_bits = a.len() as f64;
        1.0 - dist as f64 / total_bits
    }

    pub fn decompose(&mut self, text: &str, source: &str, confidence: f64) -> Vec<AtomicFact> {
        let sentences = Self::split_sentences(text);
        let mut created = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for sentence in sentences {
            let trimmed = sentence.trim().to_string();
            if trimmed.len() < 3 {
                continue;
            }
            let confidence = confidence.clamp(0.0, 1.0);
            if confidence < self.min_confidence {
                continue;
            }

            let vsa_hash = Self::hash_text(&trimmed);
            let is_duplicate = self
                .facts
                .iter()
                .any(|f| Self::vsa_similarity(&f.vsa_hash, &vsa_hash) >= self.redundancy_threshold);
            if is_duplicate {
                continue;
            }
            let source_count = self.facts.iter().filter(|f| f.source == source).count();
            if source_count >= self.max_facts_per_source {
                continue;
            }

            let fact = AtomicFact {
                id: self.next_id,
                text: trimmed,
                vsa_hash,
                source: source.to_string(),
                confidence,
                contradictions: Vec::new(),
                supporting_sources: vec![source.to_string()],
                created_at: now,
            };
            self.next_id += 1;
            self.facts.push(fact.clone());
            created.push(fact);
        }
        created
    }

    pub fn find_contradictions(&mut self, threshold_similarity: f64) -> usize {
        let mut count = 0;
        let ids: Vec<u64> = self.facts.iter().map(|f| f.id).collect();
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let fi = &self.facts[i as usize];
                let fj = &self.facts[j as usize];
                if fi.source == fj.source {
                    continue;
                }
                let sim = Self::vsa_similarity(&fi.vsa_hash, &fj.vsa_hash);
                if sim >= threshold_similarity {
                    if let Some(fi_mut) = self.facts.iter_mut().find(|f| f.id == ids[i]) {
                        if !fi_mut.contradictions.contains(&ids[j]) {
                            fi_mut.contradictions.push(ids[j]);
                            count += 1;
                        }
                    }
                    if let Some(fj_mut) = self.facts.iter_mut().find(|f| f.id == ids[j]) {
                        if !fj_mut.contradictions.contains(&ids[i]) {
                            fj_mut.contradictions.push(ids[i]);
                        }
                    }
                }
            }
        }
        count
    }

    pub fn facts_for_source(&self, source: &str) -> Vec<&AtomicFact> {
        self.facts.iter().filter(|f| f.source == source).collect()
    }

    pub fn all_facts(&self) -> &[AtomicFact] {
        &self.facts
    }

    pub fn redundancy_filter(&mut self) -> usize {
        let mut removed = 0;
        let mut keep = Vec::new();
        let mut seen: Vec<Vec<u8>> = Vec::new();

        for fact in self.facts.drain(..) {
            let is_redundant = seen
                .iter()
                .any(|h| Self::vsa_similarity(h, &fact.vsa_hash) >= self.redundancy_threshold);
            if is_redundant {
                removed += 1;
            } else {
                seen.push(fact.vsa_hash.clone());
                keep.push(fact);
            }
        }
        self.facts = keep;
        removed
    }

    pub fn stats(&self) -> FactStats {
        let unique_sources: HashSet<&str> = self.facts.iter().map(|f| f.source.as_str()).collect();
        let contradictions = self.facts.iter().map(|f| f.contradictions.len()).sum();
        let avg_conf = if self.facts.is_empty() {
            0.0
        } else {
            self.facts.iter().map(|f| f.confidence).sum::<f64>() / self.facts.len() as f64
        };
        FactStats {
            total_facts: self.facts.len(),
            unique_sources: unique_sources.len(),
            contradictions,
            average_confidence: avg_conf,
        }
    }
}

impl Default for AtomicFactDecomposer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_multi_sentence() {
        let mut d = AtomicFactDecomposer::new();
        let text = "Gradient descent converges for convex functions. The learning rate must be chosen carefully. Adam optimizer combines momentum with adaptive scaling.";
        let facts = d.decompose(text, "optimizer_paper", 0.85);
        assert_eq!(facts.len(), 3);
        assert!(facts[0].text.contains("Gradient descent"));
        assert!(facts[1].text.contains("learning rate"));
        assert!(facts[2].text.contains("Adam optimizer"));
    }

    #[test]
    fn test_dedup_by_vsa_similarity() {
        let mut d = AtomicFactDecomposer::new();
        d.redundancy_threshold = 0.8;
        let facts1 = d.decompose(
            "Gradient descent converges for convex functions.",
            "src_a",
            0.9,
        );
        let facts2 = d.decompose(
            "Gradient descent converges for convex functions.",
            "src_b",
            0.9,
        );
        assert_eq!(facts1.len(), 1);
        assert!(facts2.is_empty());
    }

    #[test]
    fn test_find_contradictions() {
        let mut d = AtomicFactDecomposer::new();
        d.decompose("The learning rate should be high.", "paper_a", 0.9);
        d.decompose("The learning rate should be low.", "paper_b", 0.9);
        let cnt = d.find_contradictions(0.5);
        assert!(cnt > 0);
        let stats = d.stats();
        assert!(stats.contradictions > 0);
    }

    #[test]
    fn test_synthesize_from_multiple_sources() {
        let mut d = AtomicFactDecomposer::new();
        d.decompose(
            "Transformer models use attention mechanisms.",
            "paper_a",
            0.9,
        );
        d.decompose(
            "Attention computes weighted sums of values.",
            "paper_b",
            0.85,
        );
        d.decompose("Transformers excel at sequence modeling.", "paper_c", 0.8);
        assert_eq!(d.all_facts().len(), 3);
        assert_eq!(d.stats().unique_sources, 3);
    }

    #[test]
    fn test_empty_input() {
        let mut d = AtomicFactDecomposer::new();
        let facts = d.decompose("", "source", 0.9);
        assert!(facts.is_empty());
        let stats = d.stats();
        assert_eq!(stats.total_facts, 0);
    }

    #[test]
    fn test_confidence_filtering() {
        let mut d = AtomicFactDecomposer::new();
        d.min_confidence = 0.8;
        let facts = d.decompose("Low confidence claim.", "source", 0.5);
        assert!(facts.is_empty());
        let facts2 = d.decompose("High confidence claim.", "source", 0.9);
        assert_eq!(facts2.len(), 1);
    }

    #[test]
    fn test_redundancy_filter() {
        let mut d = AtomicFactDecomposer::new();
        d.redundancy_threshold = 0.7;
        d.decompose("Neural networks are powerful.", "src", 0.9);
        d.facts.push(AtomicFact {
            id: d.next_id,
            text: "Neural nets are powerful.".to_string(),
            vsa_hash: AtomicFactDecomposer::hash_text("Neural nets are powerful."),
            source: "src".to_string(),
            confidence: 0.9,
            contradictions: Vec::new(),
            supporting_sources: vec!["src".to_string()],
            created_at: 0,
        });
        d.next_id += 1;
        let removed = d.redundancy_filter();
        assert_eq!(removed, 1);
        assert_eq!(d.all_facts().len(), 1);
    }

    #[test]
    fn test_facts_for_source() {
        let mut d = AtomicFactDecomposer::new();
        d.decompose("Claim A.", "src_a", 0.9);
        d.decompose("Claim B.", "src_b", 0.9);
        d.decompose("Claim C.", "src_a", 0.9);
        let src_a_facts = d.facts_for_source("src_a");
        assert_eq!(src_a_facts.len(), 2);
    }

    #[test]
    fn test_stats() {
        let mut d = AtomicFactDecomposer::new();
        d.decompose("First claim.", "a", 0.9);
        d.decompose("Second claim.", "b", 0.8);
        d.decompose("Third claim.", "a", 0.7);
        let stats = d.stats();
        assert_eq!(stats.total_facts, 3);
        assert_eq!(stats.unique_sources, 2);
        assert!((stats.average_confidence - 0.8).abs() < 0.01);
    }
}

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::context_manager::ContextBudget;

const VSA_DIM: usize = 512;
const WORKING_CAP: usize = 100;
const EPISODIC_CAP: usize = 2000;
const SEMANTIC_CAP: usize = 5000;
const SESSION_TRACE_CAP: usize = 100;
const CONSOLIDATE_INTERVAL: Duration = Duration::from_secs(300);
const VSA_SIM_THRESHOLD: f64 = 0.72;

#[derive(Clone)]
pub struct MemoryChunk {
    pub vsa: Vec<u8>,
    pub text: String,
    pub importance: f64,
    pub timestamp: Instant,
    pub tags: Vec<String>,
}

impl MemoryChunk {
    pub fn token_estimate(&self) -> usize {
        (self.text.len() / 4).max(1)
    }
}

#[derive(Clone)]
pub struct EpisodicChunk {
    pub vsa: Vec<u8>,
    pub summaries: Vec<String>,
    pub importance: f64,
    pub timestamp: i64,
    pub session_id: u64,
    pub access_count: u64,
}

impl EpisodicChunk {
    pub fn token_estimate(&self) -> usize {
        self.summaries.iter().map(|s| (s.len() / 4).max(1)).sum()
    }

    pub fn relevance(&self, query_vsa: &[u8]) -> f64 {
        QuantizedVSA::similarity(query_vsa, &self.vsa)
    }
}

#[derive(Clone)]
pub struct SemanticChunk {
    pub vsa: Vec<u8>,
    pub pattern: String,
    pub confidence: f64,
    pub source_count: u32,
    pub last_accessed: Instant,
}

#[derive(Clone)]
pub struct SessionFingerprint {
    pub vsa: Vec<u8>,
    pub summary: String,
    pub chunk_count: usize,
    pub timestamp: Instant,
    pub token_cost: usize,
}

pub struct ContextCoherentMemory {
    pub working: VecDeque<MemoryChunk>,
    pub episodic: Vec<EpisodicChunk>,
    pub semantic: Vec<SemanticChunk>,
    pub session_trace: VecDeque<SessionFingerprint>,

    #[allow(dead_code)]
    budget: ContextBudget,
    last_consolidation: Instant,
    session_token_total: usize,
    session_id: u64,
    stats: MemoryStats,
}

#[derive(Default)]
pub struct MemoryStats {
    pub working_writes: u64,
    pub episodic_writes: u64,
    pub semantic_writes: u64,
    pub retrieval_calls: u64,
    pub tokens_saved_by_dedup: u64,
    pub tokens_saved_by_budget: u64,
}

impl ContextCoherentMemory {
    pub fn new(budget_cap: usize) -> Self {
        Self {
            working: VecDeque::with_capacity(WORKING_CAP),
            episodic: Vec::with_capacity(EPISODIC_CAP),
            semantic: Vec::with_capacity(SEMANTIC_CAP),
            session_trace: VecDeque::with_capacity(SESSION_TRACE_CAP),
            budget: ContextBudget::new(budget_cap),
            last_consolidation: Instant::now(),
            session_token_total: 0,
            session_id: rand_id(),
            stats: MemoryStats::default(),
        }
    }

    pub fn store(&mut self, text: &str, importance: f64, tags: Vec<String>) -> bool {
        let vsa = text_to_vsa(text);
        let chunk = MemoryChunk {
            vsa,
            text: text.to_string(),
            importance,
            timestamp: Instant::now(),
            tags,
        };

        if self.is_duplicate(&chunk) {
            self.stats.tokens_saved_by_dedup += chunk.token_estimate() as u64;
            return false;
        }

        self.session_token_total += chunk.token_estimate();
        self.working.push_back(chunk);
        self.stats.working_writes += 1;

        if self.working.len() > WORKING_CAP {
            self.consolidate_working();
        }

        if self.last_consolidation.elapsed() > CONSOLIDATE_INTERVAL {
            self.consolidate_episodic();
            self.last_consolidation = Instant::now();
        }

        true
    }

    pub fn store_batch(&mut self, chunks: Vec<(String, f64, Vec<String>)>) -> (usize, usize) {
        let mut stored = 0;
        let mut deduped = 0;
        for (text, imp, tags) in chunks {
            if self.store(&text, imp, tags) {
                stored += 1;
            } else {
                deduped += 1;
            }
        }
        (stored, deduped)
    }

    pub fn recall(&mut self, query: &str, max_tokens: usize) -> Vec<String> {
        self.stats.retrieval_calls += 1;
        let query_vsa = text_to_vsa(query);

        let mut scored: Vec<(f64, String, usize)> = Vec::new();

        for chunk in &self.working {
            let sim = QuantizedVSA::similarity(&query_vsa, &chunk.vsa);
            let effective = sim * chunk.importance * recency_weight(chunk.timestamp);
            scored.push((effective, chunk.text.clone(), chunk.token_estimate()));
        }

        for chunk in &self.episodic {
            let sim = chunk.relevance(&query_vsa);
            let effective = sim * chunk.importance * 0.8;
            let text = chunk.summaries.first().cloned().unwrap_or_default();
            scored.push((effective, text, chunk.token_estimate()));
        }

        for chunk in &self.semantic {
            let sim = QuantizedVSA::similarity(&query_vsa, &chunk.vsa);
            let effective = sim * chunk.confidence * 0.6;
            scored.push((
                effective,
                chunk.pattern.clone(),
                (chunk.pattern.len() / 4).max(1),
            ));
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut result = Vec::new();
        let mut tokens = 0;
        for (score, text, t) in scored {
            if score < VSA_SIM_THRESHOLD {
                break;
            }
            if tokens + t > max_tokens {
                self.stats.tokens_saved_by_budget += t as u64;
                continue;
            }
            result.push(text);
            tokens += t;
        }

        result
    }

    pub fn end_session(&mut self, summary: &str) -> SessionFingerprint {
        let session_vsa = self.compute_session_fingerprint();
        let fp = SessionFingerprint {
            vsa: session_vsa.clone(),
            summary: summary.to_string(),
            chunk_count: self.working.len(),
            timestamp: Instant::now(),
            token_cost: self.session_token_total,
        };

        let ep = EpisodicChunk {
            vsa: session_vsa,
            summaries: vec![summary.to_string()],
            importance: self.working.iter().map(|c| c.importance).sum::<f64>()
                / (self.working.len() as f64).max(1.0),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            session_id: self.session_id,
            access_count: 0,
        };

        self.consolidate_working();
        self.episodic.push(ep);
        self.stats.episodic_writes += 1;
        if self.episodic.len() > EPISODIC_CAP {
            self.episodic.sort_by(|a, b| {
                b.importance
                    .partial_cmp(&a.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.episodic.truncate(EPISODIC_CAP);
        }

        self.session_trace.push_back(fp.clone());
        if self.session_trace.len() > SESSION_TRACE_CAP {
            self.session_trace.pop_front();
        }

        self.working.clear();
        self.session_token_total = 0;
        self.session_id = rand_id();

        fp
    }

    pub fn find_related_sessions(&self, query: &str, k: usize) -> Vec<(f64, &SessionFingerprint)> {
        let query_vsa = text_to_vsa(query);
        let mut scored: Vec<(f64, &SessionFingerprint)> = self
            .session_trace
            .iter()
            .map(|fp| (QuantizedVSA::similarity(&query_vsa, &fp.vsa), fp))
            .filter(|(s, _)| *s > VSA_SIM_THRESHOLD)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    pub fn consolidate_working(&mut self) {
        if self.working.len() < 10 {
            return;
        }

        let mut clusters: Vec<(Vec<u8>, Vec<MemoryChunk>)> = Vec::new();

        let mut remaining: Vec<MemoryChunk> = self.working.drain(..).collect();

        while let Some(anchor) = remaining.pop() {
            let mut cluster = vec![anchor];
            let mut i = 0;
            while i < remaining.len() {
                let sim = QuantizedVSA::similarity(&cluster[0].vsa, &remaining[i].vsa);
                if sim > 0.85 {
                    cluster.push(remaining.remove(i));
                } else {
                    i += 1;
                }
            }

            if cluster.len() >= 3 {
                let bundled =
                    QuantizedVSA::bundle(&cluster.iter().map(|c| &c.vsa[..]).collect::<Vec<_>>());
                clusters.push((bundled, cluster));
            } else {
                for c in cluster {
                    self.working.push_back(c);
                }
            }
        }

        for (bundled_vsa, group) in clusters {
            let avg_imp = group.iter().map(|c| c.importance).sum::<f64>() / group.len() as f64;
            let summaries: Vec<String> = group.iter().map(|c| c.text.clone()).collect();

            self.episodic.push(EpisodicChunk {
                vsa: bundled_vsa,
                summaries,
                importance: avg_imp * 1.1,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
                session_id: self.session_id,
                access_count: 0,
            });
            self.stats.episodic_writes += 1;
        }

        if self.episodic.len() > EPISODIC_CAP {
            self.episodic.sort_by(|a, b| {
                b.importance
                    .partial_cmp(&a.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.episodic.truncate(EPISODIC_CAP);
        }
    }

    pub fn consolidate_episodic(&mut self) {
        let mut new_semantic: Vec<SemanticChunk> = Vec::new();

        let mut i = 0;
        while i < self.episodic.len() {
            let anchor_vsa = &self.episodic[i].vsa.clone();
            let mut group_indices = vec![i];
            let mut j = i + 1;
            while j < self.episodic.len() {
                if QuantizedVSA::similarity(anchor_vsa, &self.episodic[j].vsa) > 0.88 {
                    group_indices.push(j);
                }
                j += 1;
            }

            if group_indices.len() >= 3 {
                let bundled = QuantizedVSA::bundle(
                    &group_indices
                        .iter()
                        .map(|&idx| &self.episodic[idx].vsa[..])
                        .collect::<Vec<_>>(),
                );
                let avg_imp = group_indices
                    .iter()
                    .map(|&idx| self.episodic[idx].importance)
                    .sum::<f64>()
                    / group_indices.len() as f64;
                let pattern = format!("[pattern: {} episodes]", group_indices.len());

                new_semantic.push(SemanticChunk {
                    vsa: bundled,
                    pattern,
                    confidence: avg_imp.min(1.0),
                    source_count: group_indices.len() as u32,
                    last_accessed: Instant::now(),
                });

                for &idx in group_indices.iter().skip(1) {
                    self.episodic[idx].importance *= 0.5;
                }
                self.episodic[i].importance *= 1.2;
            }
            i += 1;
        }

        for sc in new_semantic {
            if !self
                .semantic
                .iter()
                .any(|s| QuantizedVSA::similarity(&s.vsa, &sc.vsa) > 0.92)
            {
                self.semantic.push(sc);
                self.stats.semantic_writes += 1;
            }
        }

        if self.semantic.len() > SEMANTIC_CAP {
            self.semantic.sort_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.semantic.truncate(SEMANTIC_CAP);
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "[Memory] {}W + {}E + {}S + {} sessions | {} retrievals | {}t saved dedup + {}t saved budget",
            self.working.len(),
            self.episodic.len(),
            self.semantic.len(),
            self.session_trace.len(),
            self.stats.retrieval_calls,
            self.stats.tokens_saved_by_dedup,
            self.stats.tokens_saved_by_budget,
        )
    }

    fn is_duplicate(&self, chunk: &MemoryChunk) -> bool {
        for existing in &self.working {
            if QuantizedVSA::similarity(&chunk.vsa, &existing.vsa) > 0.92 {
                return true;
            }
        }
        false
    }

    fn compute_session_fingerprint(&self) -> Vec<u8> {
        if self.working.is_empty() {
            return vec![0u8; VSA_DIM];
        }
        let vsas: Vec<&[u8]> = self.working.iter().map(|c| &c.vsa[..]).collect();
        QuantizedVSA::bundle(&vsas)
    }
}

fn text_to_vsa(text: &str) -> Vec<u8> {
    use std::hash::{Hash, Hasher};
    let mut bytes = vec![0u8; VSA_DIM];
    let seed: u64 = 0x9e3779b97f4a7c15;
    for (i, word) in text.split_whitespace().enumerate() {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        word.hash(&mut hasher);
        let h = hasher.finish();
        let pos = (h as usize) % VSA_DIM;
        let flip = (h.wrapping_mul(seed).wrapping_add(i as u64)) as u8;
        bytes[pos] ^= flip;
    }
    bytes
}

fn recency_weight(t: Instant) -> f64 {
    let age = t.elapsed();
    let secs = age.as_secs_f64();
    (-secs / 3600.0).exp()
}

fn rand_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chunk(text: &str, importance: f64) -> (String, f64, Vec<String>) {
        (text.to_string(), importance, vec![])
    }

    #[test]
    fn test_store_and_recall() {
        let mut mem = ContextCoherentMemory::new(10000);
        mem.store("hello world", 1.0, vec![]);
        mem.store("goodbye world", 0.8, vec![]);
        assert_eq!(mem.working.len(), 2);
    }

    #[test]
    fn test_dedup_on_store() {
        let mut mem = ContextCoherentMemory::new(10000);
        mem.store("duplicate text", 1.0, vec![]);
        mem.store("duplicate text", 1.0, vec![]);
        assert_eq!(mem.working.len(), 1);
    }

    #[test]
    fn test_recall_returns_relevant() {
        let mut mem = ContextCoherentMemory::new(10000);
        mem.store("rust programming", 1.0, vec![]);
        mem.store("cooking recipes", 1.0, vec![]);
        mem.store("tokio async runtime", 0.9, vec![]);

        let results = mem.recall("rust async", 5000);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_recall_respects_budget() {
        let mut mem = ContextCoherentMemory::new(100);
        for i in 0..20 {
            mem.store(&format!("item number {} in the list", i), 1.0, vec![]);
        }

        let results = mem.recall("item", 20);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_end_session_creates_fingerprint() {
        let mut mem = ContextCoherentMemory::new(10000);
        mem.store("session data point one", 1.0, vec![]);
        mem.store("session data point two", 0.8, vec![]);

        let fp = mem.end_session("test session");
        assert_eq!(fp.chunk_count, 2);
        assert_eq!(mem.session_trace.len(), 1);
        assert_eq!(mem.working.len(), 0);
    }

    #[test]
    fn test_find_related_sessions() {
        let mut mem = ContextCoherentMemory::new(10000);
        mem.store("tokio async event loop", 1.0, vec![]);
        mem.end_session("tokio session");
        mem.store("chicken soup recipe", 1.0, vec![]);
        mem.end_session("cooking session");

        let related = mem.find_related_sessions("tokio runtime", 5);
        assert_eq!(related.len(), 1);
    }

    #[test]
    fn test_store_batch() {
        let mut mem = ContextCoherentMemory::new(10000);
        let chunks = vec![
            make_chunk("a", 1.0),
            make_chunk("b", 1.0),
            make_chunk("a", 1.0),
        ];
        let (stored, deduped) = mem.store_batch(chunks);
        assert_eq!(stored, 2);
        assert_eq!(deduped, 1);
    }

    #[test]
    fn test_consolidate_working_to_episodic() {
        let mut mem = ContextCoherentMemory::new(10000);
        for i in 0..15 {
            mem.store(&format!("item {:02} in the list of things", i), 0.8, vec![]);
        }
        assert!(mem.working.len() < 15);
    }

    #[test]
    fn test_importance_affects_recall_rank() {
        let mut mem = ContextCoherentMemory::new(10000);
        mem.store("rust concurrency patterns", 0.3, vec![]);
        mem.store("rust async programming guide", 1.0, vec![]);

        let results = mem.recall("rust programming", 5000);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_session_continuity() {
        let mut mem = ContextCoherentMemory::new(10000);
        mem.store("discussed VSA memory architecture", 1.0, vec![]);
        let fp1 = mem.end_session("VSA discussion");

        mem.store("now implementing the solution", 1.0, vec![]);
        let fp2 = mem.end_session("implementation");

        let sim = QuantizedVSA::similarity(&fp1.vsa, &fp2.vsa);
        assert!(sim > 0.0);
    }

    #[test]
    fn test_empty_memory_recall() {
        let mut mem = ContextCoherentMemory::new(10000);
        let results = mem.recall("anything", 5000);
        assert!(results.is_empty());
    }

    #[test]
    fn test_low_importance_pruned() {
        let mut mem = ContextCoherentMemory::new(10000);
        mem.store("low importance noise", 0.1, vec![]);
        mem.store("high importance signal", 1.0, vec![]);

        let results = mem.recall("importance", 5000);
        assert!(results.iter().any(|r| r.contains("signal")));
    }

    #[test]
    fn test_episodic_consolidation_to_semantic() {
        let mut mem = ContextCoherentMemory::new(10000);
        for i in 0..5 {
            mem.store(&format!("pattern data version {}", i), 0.9, vec![]);
            mem.end_session(&format!("session {}", i));
        }
        mem.consolidate_episodic();
    }
}

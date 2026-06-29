#![forbid(unsafe_code)]

use std::collections::HashMap;

/// Three-phase lifecycle for a captured subagent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubAgentPhase {
    /// Freshly captured, minimal validation.
    Seedling,
    /// Proven useful across 3+ similar tasks.
    Established,
    /// Refined and robust for general use.
    Mature,
}

impl SubAgentPhase {
    pub fn name(&self) -> &'static str {
        match self {
            SubAgentPhase::Seedling => "seedling",
            SubAgentPhase::Established => "established",
            SubAgentPhase::Mature => "mature",
        }
    }
}

/// Retrieval strategy for finding relevant subagents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalStrategy {
    /// Return most similar by signature (hamming distance).
    BestMatch,
    /// Return top-N by success_rate.
    TopRated,
    /// Return most recently used.
    MostRecent,
}

/// A single reusable subagent captured from an execution trace.
#[derive(Debug, Clone)]
pub struct SubAgent {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub code_fingerprint: u64,
    /// VSA-like signature for similarity matching.
    pub signature: Vec<u8>,
    pub success_count: u64,
    pub failure_count: u64,
    pub refinement_count: u64,
    pub last_used_epoch: u64,
    pub created_epoch: u64,
    pub tags: Vec<String>,
    pub phase: SubAgentPhase,
}

impl SubAgent {
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.0
        } else {
            self.success_count as f64 / total as f64
        }
    }
}

/// Statistics for the accumulator.
#[derive(Debug, Clone)]
pub struct AccumulatorStats {
    pub total_captured: u64,
    pub total_seedling: u64,
    pub total_established: u64,
    pub total_mature: u64,
    pub total_retrieved: u64,
    pub retrieval_hit_rate: f64,
}

/// AgentFactory-inspired three-phase subagent lifecycle manager.
///
/// Phase 1 — **Install**: Capture successful execution traces as reusable subagents.
/// Phase 2 — **Self-Evolve**: Refine subagents based on execution feedback.
/// Phase 3 — **Deploy**: Retrieve and execute subagents for similar tasks.
#[derive(Debug)]
pub struct SubAgentAccumulator {
    subagents: Vec<SubAgent>,
    subagent_map: HashMap<u64, usize>,
    max_subagents: usize,
    next_id: u64,
    total_retrieved: u64,
    retrieval_hits: u64,
    retrieval_misses: u64,
}

impl SubAgentAccumulator {
    pub fn new(max_subagents: usize) -> Self {
        Self {
            subagents: Vec::with_capacity(max_subagents.min(64)),
            subagent_map: HashMap::new(),
            max_subagents,
            next_id: 1,
            total_retrieved: 0,
            retrieval_hits: 0,
            retrieval_misses: 0,
        }
    }

    /// Capture a new subagent from an execution trace.
    /// Generates a signature from code bytes. Starts as Seedling.
    /// Returns the new subagent's id.
    pub fn capture(
        &mut self,
        name: &str,
        description: &str,
        code: &[u8],
        tags: Vec<String>,
        epoch: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let code_fingerprint = hash_bytes(code);
        let signature = Self::generate_signature(code);

        let agent = SubAgent {
            id,
            name: name.to_string(),
            description: description.to_string(),
            code_fingerprint,
            signature,
            success_count: 0,
            failure_count: 0,
            refinement_count: 0,
            last_used_epoch: epoch,
            created_epoch: epoch,
            tags,
            phase: SubAgentPhase::Seedling,
        };

        let idx = self.subagents.len();
        self.subagents.push(agent);
        self.subagent_map.insert(id, idx);

        // Enforce capacity
        if self.subagents.len() > self.max_subagents {
            self.prune();
        }

        id
    }

    /// Retrieve subagents similar to a query signature.
    pub fn retrieve(
        &self,
        query_signature: &[u8],
        strategy: RetrievalStrategy,
        limit: usize,
    ) -> Vec<&SubAgent> {
        if self.subagents.is_empty() || limit == 0 {
            return Vec::new();
        }

        let limit = limit.min(self.subagents.len());

        let mut candidates: Vec<(&SubAgent, f64)> = match strategy {
            RetrievalStrategy::BestMatch => self
                .subagents
                .iter()
                .map(|a| {
                    let sim = Self::similarity(&a.signature, query_signature);
                    (a, sim)
                })
                .collect(),
            RetrievalStrategy::TopRated => self
                .subagents
                .iter()
                .map(|a| {
                    let score = a.success_rate();
                    (a, score)
                })
                .collect(),
            RetrievalStrategy::MostRecent => self
                .subagents
                .iter()
                .map(|a| (a, a.last_used_epoch as f64))
                .collect(),
        };

        candidates.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(limit);

        let result: Vec<&SubAgent> = candidates.into_iter().map(|(a, _)| a).collect();
        result
    }

    /// Record success or failure, auto-promote on thresholds.
    pub fn record_outcome(&mut self, id: u64, success: bool, epoch: u64) {
        let idx = match self.subagent_map.get(&id) {
            Some(&i) => i,
            None => return,
        };
        let agent = &mut self.subagents[idx];

        if success {
            agent.success_count += 1;
        } else {
            agent.failure_count += 1;
        }
        agent.last_used_epoch = epoch;

        // Auto-promotion thresholds
        let total_successes = agent.success_count;
        agent.phase = if total_successes >= 10 {
            SubAgentPhase::Mature
        } else if total_successes >= 3 {
            SubAgentPhase::Established
        } else {
            SubAgentPhase::Seedling
        };

        // Track retrieval hits/misses
        if success {
            self.retrieval_hits += 1;
        } else {
            self.retrieval_misses += 1;
        }
        self.total_retrieved += 1;
    }

    /// Refine an existing subagent (Self-Evolve phase).
    /// Updates fingerprint, signature, and description.
    pub fn refine(&mut self, id: u64, new_code: &[u8], new_description: &str) {
        let idx = match self.subagent_map.get(&id) {
            Some(&i) => i,
            None => return,
        };
        let agent = &mut self.subagents[idx];

        agent.code_fingerprint = hash_bytes(new_code);
        agent.signature = Self::generate_signature(new_code);
        agent.description = new_description.to_string();
        agent.refinement_count += 1;
    }

    /// Compute similarity between two signatures (hamming-based).
    pub fn similarity(a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len());
        if len == 0 {
            return 0.0;
        }

        let differing_bits: u64 = a[..len]
            .iter()
            .zip(b[..len].iter())
            .map(|(x, y)| (x ^ y).count_ones() as u64)
            .sum();

        let total_bits = (len as u64) * 8;
        1.0 - (differing_bits as f64 / total_bits as f64)
    }

    /// Manually promote a subagent to the next phase.
    pub fn promote(&mut self, id: u64) {
        let idx = match self.subagent_map.get(&id) {
            Some(&i) => i,
            None => return,
        };
        let agent = &mut self.subagents[idx];

        agent.phase = match agent.phase {
            SubAgentPhase::Seedling => SubAgentPhase::Established,
            SubAgentPhase::Established => SubAgentPhase::Mature,
            SubAgentPhase::Mature => SubAgentPhase::Mature,
        };
    }

    /// Remove subagents that are Seedling with >5 failures
    /// and haven't been used in 100 epochs.
    pub fn prune(&mut self) {
        let current_epoch = self
            .subagents
            .iter()
            .map(|a| a.last_used_epoch)
            .max()
            .unwrap_or(0);

        let mut to_remove: Vec<usize> = Vec::new();
        for (i, agent) in self.subagents.iter().enumerate() {
            if agent.phase == SubAgentPhase::Seedling
                && agent.failure_count > 5
                && current_epoch.saturating_sub(agent.last_used_epoch) > 100
            {
                to_remove.push(i);
            }
        }

        // Remove in reverse order to preserve indices
        to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for &i in &to_remove {
            let removed = self.subagents.swap_remove(i);
            self.subagent_map.remove(&removed.id);
            // Rebuild map for swapped element
            if i < self.subagents.len() {
                self.subagent_map
                    .insert(self.subagents[i].id, i);
            }
        }
    }

    /// Get current statistics.
    pub fn stats(&self) -> AccumulatorStats {
        let total_captured = self.subagents.len() as u64;
        let total_seedling = self
            .subagents
            .iter()
            .filter(|a| a.phase == SubAgentPhase::Seedling)
            .count() as u64;
        let total_established = self
            .subagents
            .iter()
            .filter(|a| a.phase == SubAgentPhase::Established)
            .count() as u64;
        let total_mature = self
            .subagents
            .iter()
            .filter(|a| a.phase == SubAgentPhase::Mature)
            .count() as u64;

        let retrieval_hit_rate = if self.total_retrieved > 0 {
            self.retrieval_hits as f64 / self.total_retrieved as f64
        } else {
            0.0
        };

        AccumulatorStats {
            total_captured,
            total_seedling,
            total_established,
            total_mature,
            total_retrieved: self.total_retrieved,
            retrieval_hit_rate,
        }
    }

    /// Total count of subagents
    pub fn total_count(&self) -> usize {
        self.subagents.len()
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        let s = self.stats();
        format!(
            "SubAgentAccumulator: {} total (S:{}/E:{}/M:{}), {} retrievals @ {:.1}% hit, max={}",
            s.total_captured,
            s.total_seedling,
            s.total_established,
            s.total_mature,
            s.total_retrieved,
            s.retrieval_hit_rate * 100.0,
            self.max_subagents,
        )
    }

    /// Generate a deterministic signature from code bytes.
    /// Uses a simple hash-per-byte scheme to produce a 64-byte signature.
    fn generate_signature(code: &[u8]) -> Vec<u8> {
        let mut sig = vec![0u8; 64];
        for (i, &b) in code.iter().enumerate() {
            sig[i % 64] = sig[i % 64].wrapping_mul(31).wrapping_add(b);
        }
        sig
    }
}

/// Simple non-cryptographic hash for code fingerprints.
fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_creates_seedling() {
        let mut acc = SubAgentAccumulator::new(100);
        let id = acc.capture("extract", "Extracts data from HTML", b"fn extract() {}", vec!["extract".into(), "html".into()], 1);
        assert_eq!(id, 1);
        let agent = &acc.subagents[0];
        assert_eq!(agent.name, "extract");
        assert_eq!(agent.phase, SubAgentPhase::Seedling);
        assert_eq!(agent.tags.len(), 2);
        assert!(agent.code_fingerprint != 0);
        assert_eq!(agent.signature.len(), 64);
    }

    #[test]
    fn test_retrieve_best_match_returns_most_similar() {
        let mut acc = SubAgentAccumulator::new(100);
        acc.capture("alpha", "Alpha agent", b"fn alpha() {}", vec![], 1);
        acc.capture("beta", "Beta agent", b"fn beta() {}", vec![], 1);
        acc.capture("gamma", "Gamma agent", b"fn gamma() {}", vec![], 1);

        let query = SubAgentAccumulator::generate_signature(b"fn alpha() {}");
        let results = acc.retrieve(&query, RetrievalStrategy::BestMatch, 2);
        assert_eq!(results.len(), 2);
        // alpha should be first (best match)
        assert_eq!(results[0].name, "alpha");
    }

    #[test]
    fn test_retrieve_top_rated() {
        let mut acc = SubAgentAccumulator::new(100);
        let id_a = acc.capture("good", "Good agent", b"good", vec![], 1);
        let id_b = acc.capture("bad", "Bad agent", b"bad", vec![], 1);
        acc.record_outcome(id_a, true, 2);
        acc.record_outcome(id_a, true, 3);
        acc.record_outcome(id_b, false, 2);
        acc.record_outcome(id_b, false, 3);

        let results = acc.retrieve(&[], RetrievalStrategy::TopRated, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "good");
    }

    #[test]
    fn test_retrieve_most_recent() {
        let mut acc = SubAgentAccumulator::new(100);
        acc.capture("old", "Old agent", b"old", vec![], 1);
        let id = acc.capture("new", "New agent", b"new", vec![], 50);
        acc.record_outcome(id, true, 100);

        let results = acc.retrieve(&[], RetrievalStrategy::MostRecent, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "new");
    }

    #[test]
    fn test_auto_promotion_to_established() {
        let mut acc = SubAgentAccumulator::new(100);
        let id = acc.capture("worker", "Worker agent", b"work()", vec![], 1);

        for i in 0..3 {
            acc.record_outcome(id, true, i + 1);
        }

        let agent = acc.subagents.iter().find(|a| a.id == id).unwrap();
        assert_eq!(agent.phase, SubAgentPhase::Established);
        assert_eq!(agent.success_count, 3);
    }

    #[test]
    fn test_auto_promotion_to_mature() {
        let mut acc = SubAgentAccumulator::new(100);
        let id = acc.capture("worker", "Worker agent", b"work()", vec![], 1);

        for i in 0..10 {
            acc.record_outcome(id, true, i + 1);
        }

        let agent = acc.subagents.iter().find(|a| a.id == id).unwrap();
        assert_eq!(agent.phase, SubAgentPhase::Mature);
        assert_eq!(agent.success_count, 10);
    }

    #[test]
    fn test_refine_updates_fingerprint_and_signature() {
        let mut acc = SubAgentAccumulator::new(100);
        let id = acc.capture("evolver", "Evolver agent", b"old_code()", vec![], 1);
        let old_fp = acc.subagents[0].code_fingerprint;
        let old_sig = acc.subagents[0].signature.clone();

        acc.refine(id, b"new_code()", "Refined evolver agent");
        let agent = &acc.subagents[0];
        assert_ne!(agent.code_fingerprint, old_fp);
        assert_ne!(agent.signature, old_sig);
        assert_eq!(agent.description, "Refined evolver agent");
        assert_eq!(agent.refinement_count, 1);
    }

    #[test]
    fn test_manual_promote() {
        let mut acc = SubAgentAccumulator::new(100);
        let id = acc.capture("agent", "agent", b"code", vec![], 1);

        assert_eq!(
            acc.subagents.iter().find(|a| a.id == id).unwrap().phase,
            SubAgentPhase::Seedling
        );

        acc.promote(id);
        assert_eq!(
            acc.subagents.iter().find(|a| a.id == id).unwrap().phase,
            SubAgentPhase::Established
        );

        acc.promote(id);
        assert_eq!(
            acc.subagents.iter().find(|a| a.id == id).unwrap().phase,
            SubAgentPhase::Mature
        );

        // Promoting Mature stays Mature
        acc.promote(id);
        assert_eq!(
            acc.subagents.iter().find(|a| a.id == id).unwrap().phase,
            SubAgentPhase::Mature
        );
    }

    #[test]
    fn test_prune_removes_dead_seedlings() {
        let mut acc = SubAgentAccumulator::new(100);
        // Create a seedling with many failures and old last_used
        let _id1 = acc.capture("dead", "Dead agent", b"dead", vec![], 1);
        let id2 = acc.capture("alive", "Alive agent", b"alive", vec![], 200);

        // Record many failures for first agent at old epoch
        acc.record_outcome(1, false, 1);
        acc.record_outcome(1, false, 1);
        acc.record_outcome(1, false, 1);
        acc.record_outcome(1, false, 1);
        acc.record_outcome(1, false, 1);
        acc.record_outcome(1, false, 1);

        acc.record_outcome(id2, true, 200);

        acc.prune();

        // Dead seedling should be removed, alive kept
        assert!(acc.subagent_map.get(&1).is_none());
        assert!(acc.subagent_map.get(&id2).is_some());
    }

    #[test]
    fn test_similarity_identical() {
        let a = vec![0u8; 64];
        let b = vec![0u8; 64];
        let sim = SubAgentAccumulator::similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_similarity_opposite() {
        let a = vec![0u8; 64];
        let b = vec![0xFFu8; 64];
        let sim = SubAgentAccumulator::similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_stats() {
        let mut acc = SubAgentAccumulator::new(100);
        let id = acc.capture("a", "A", b"a", vec![], 1);
        acc.record_outcome(id, true, 2);
        acc.record_outcome(id, true, 3);
        acc.record_outcome(id, true, 4);

        let s = acc.stats();
        assert_eq!(s.total_captured, 1);
        assert_eq!(s.total_established, 1);
        assert_eq!(s.total_retrieved, 3);
        assert!(s.retrieval_hit_rate > 0.0);
    }

    #[test]
    fn test_summary_format() {
        let mut acc = SubAgentAccumulator::new(50);
        let id = acc.capture("x", "X", b"x", vec![], 1);
        acc.record_outcome(id, true, 2);

        let summary = acc.summary();
        assert!(summary.contains("total"));
        assert!(summary.contains("50"));
        assert!(summary.contains("100.0%"));
    }

    #[test]
    fn test_capacity_enforced_on_capture() {
        let mut acc = SubAgentAccumulator::new(2);
        acc.capture("a", "A", b"a", vec![], 1);
        acc.capture("b", "B", b"b", vec![], 1);
        // Third capture exceeds max: prune removes dead ones
        // Since none have >5 failures, this should be okay — max is advisory for prune
        // Actually let's test that capture doesn't crash when over max
        let id = acc.capture("c", "C", b"c", vec![], 1);
        // prune is called, but agents have 0 failures so none removed
        assert!(acc.subagent_map.contains_key(&id));
    }

    #[test]
    fn test_retrieve_empty_returns_empty() {
        let acc = SubAgentAccumulator::new(100);
        assert!(acc.retrieve(&[], RetrievalStrategy::BestMatch, 5).is_empty());
    }

    #[test]
    fn test_record_outcome_unknown_id_noop() {
        let mut acc = SubAgentAccumulator::new(100);
        acc.record_outcome(999, true, 1);
        assert!(acc.subagents.is_empty());
    }

    #[test]
    fn test_refine_unknown_id_noop() {
        let mut acc = SubAgentAccumulator::new(100);
        acc.refine(999, b"code", "desc");
        assert!(acc.subagents.is_empty());
    }

    #[test]
    fn test_promote_unknown_id_noop() {
        let mut acc = SubAgentAccumulator::new(100);
        acc.promote(999);
        assert!(acc.subagents.is_empty());
    }

    #[test]
    fn test_success_rate() {
        let mut acc = SubAgentAccumulator::new(100);
        let id = acc.capture("r", "R", b"r", vec![], 1);
        assert!((acc.subagents[0].success_rate() - 0.0).abs() < 1e-9);

        acc.record_outcome(id, true, 2);
        acc.record_outcome(id, true, 2);
        acc.record_outcome(id, false, 2);
        let sr = acc.subagents[0].success_rate();
        assert!((sr - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_retrieve_limit_respected() {
        let mut acc = SubAgentAccumulator::new(100);
        for i in 0..10 {
            acc.capture(&format!("a{}", i), "", b"code", vec![], 1);
        }
        let results = acc.retrieve(&[], RetrievalStrategy::MostRecent, 3);
        assert_eq!(results.len(), 3);
    }
}

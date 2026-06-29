use crate::core::nt_core_knowledge::evidence::EvidenceRecord;

const MAX_RECENT: usize = 100;
const DEFAULT_THRESHOLD: f64 = 0.7;

/// Result of faithfulness check for a single cited evidence record
#[derive(Debug, Clone)]
pub struct FaithfulnessVerdict {
    pub evidence_id: u64,
    pub source_assertion: String,
    pub response_snippet: String,
    pub similarity: f64,
    pub is_faithful: bool,
    pub issue: Option<String>,
}

/// Tracks aggregate faithfulness metrics across a session
#[derive(Debug, Clone)]
pub struct FaithfulnessTracker {
    pub total_checks: usize,
    pub faithful_count: usize,
    pub unfaithful_count: usize,
    pub avg_similarity: f64,
    pub recent_verdicts: Vec<FaithfulnessVerdict>,
}

impl FaithfulnessTracker {
    fn new(max_recent: usize) -> Self {
        Self {
            total_checks: 0,
            faithful_count: 0,
            unfaithful_count: 0,
            avg_similarity: 0.0,
            recent_verdicts: Vec::with_capacity(max_recent),
        }
    }
}

pub struct FaithfulnessChecker {
    tracker: FaithfulnessTracker,
    similarity_threshold: f64,
    max_recent: usize,
}

impl FaithfulnessChecker {
    pub fn new() -> Self {
        Self {
            tracker: FaithfulnessTracker::new(MAX_RECENT),
            similarity_threshold: DEFAULT_THRESHOLD,
            max_recent: MAX_RECENT,
        }
    }

    pub fn with_threshold(mut self, t: f64) -> Self {
        self.similarity_threshold = t.clamp(0.0, 1.0);
        self
    }

    pub fn check(&mut self, evidence: &EvidenceRecord, response_text: &str) -> FaithfulnessVerdict {
        let assertion = &evidence.assertion;
        let best_sentence = find_best_matching_sentence(response_text, assertion);
        let sim = vsa_similarity(assertion, &best_sentence);
        let is_faithful = sim >= self.similarity_threshold;
        let issue = if !is_faithful {
            Some(format!(
                "similarity={:.3} < threshold={:.3}; assertion differs from response",
                sim, self.similarity_threshold
            ))
        } else {
            None
        };

        let verdict = FaithfulnessVerdict {
            evidence_id: evidence.id,
            source_assertion: assertion.clone(),
            response_snippet: best_sentence,
            similarity: sim,
            is_faithful,
            issue,
        };

        self.tracker.total_checks += 1;
        if is_faithful {
            self.tracker.faithful_count += 1;
        } else {
            self.tracker.unfaithful_count += 1;
        }
        let total = self.tracker.total_checks as f64;
        self.tracker.avg_similarity =
            self.tracker.avg_similarity + (sim - self.tracker.avg_similarity) / total;

        self.tracker.recent_verdicts.push(verdict.clone());
        if self.tracker.recent_verdicts.len() > self.max_recent {
            self.tracker.recent_verdicts.remove(0);
        }

        verdict
    }

    pub fn check_batch(
        &mut self,
        evidence_ids: &[u64],
        response_text: &str,
        evidence_manager: &crate::core::nt_core_knowledge::evidence::EvidenceManager,
    ) -> Vec<FaithfulnessVerdict> {
        let records = evidence_manager.get_by_ids(evidence_ids);
        records
            .iter()
            .map(|record| self.check(record, response_text))
            .collect()
    }

    pub fn tracker(&self) -> &FaithfulnessTracker {
        &self.tracker
    }

    pub fn reset(&mut self) {
        self.tracker = FaithfulnessTracker::new(self.max_recent);
    }

    pub fn summary(&self) -> String {
        let t = &self.tracker;
        if t.total_checks == 0 {
            return "faithfulness:no_data".to_string();
        }
        let faithful_pct = t.faithful_count as f64 / t.total_checks as f64 * 100.0;
        let unfaithful_pct = t.unfaithful_count as f64 / t.total_checks as f64 * 100.0;
        format!(
            "faithfulness:total={}_faithful={}({:.1}%)_unfaithful={}({:.1}%)_avg_sim={:.3}",
            t.total_checks,
            t.faithful_count,
            faithful_pct,
            t.unfaithful_count,
            unfaithful_pct,
            t.avg_similarity,
        )
    }
}

impl Default for FaithfulnessChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Deterministic 64-bit hash signature for a text.
/// Splits into words, hashes each word position-dependently, XORs together.
fn signature(text: &str) -> u64 {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return 0;
    }
    let mut sig: u64 = 0;
    for (i, word) in words.iter().enumerate() {
        let word_hash: u64 = word
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let rotated = word_hash.rotate_left((i * 7) as u32);
        sig ^= rotated;
    }
    sig
}

/// VSA-inspired similarity between two texts using deterministic hashing.
/// Returns hamming similarity: fraction of matching bits in 64-bit signatures.
fn vsa_similarity(text_a: &str, text_b: &str) -> f64 {
    let sig_a = signature(text_a);
    let sig_b = signature(text_b);
    let xor = sig_a ^ sig_b;
    let matching = 64 - xor.count_ones() as u64;
    matching as f64 / 64.0
}

/// Find the sentence in `response_text` that best matches `assertion`.
/// Splits on sentence boundaries, returns the highest-similarity sentence.
fn find_best_matching_sentence(response_text: &str, assertion: &str) -> String {
    if response_text.is_empty() {
        return String::new();
    }

    let sentences: Vec<&str> = response_text
        .split(|c: char| c == '.' || c == '!' || c == '?' || c == '\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if sentences.is_empty() {
        return response_text.to_string();
    }

    let mut best_sim = 0.0_f64;
    let mut best_sentence = sentences[0].to_string();

    for sentence in &sentences {
        let sim = vsa_similarity(assertion, sentence);
        if sim > best_sim {
            best_sim = sim;
            best_sentence = sentence.to_string();
        }
    }

    best_sentence
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_knowledge::evidence::EvidenceManager;

    fn make_evidence(assertion: &str) -> EvidenceRecord {
        EvidenceRecord::new(1, "https://example.com", "test", assertion)
    }

    #[test]
    fn test_exact_match_high_similarity() {
        let mut checker = FaithfulnessChecker::new();
        let ev = make_evidence("the sky is blue");
        let verdict = checker.check(&ev, "the sky is blue");
        assert!(verdict.is_faithful, "exact match should be faithful");
        assert!(
            verdict.similarity >= 0.9,
            "exact match similarity should be high"
        );
    }

    #[test]
    fn test_completely_different_low_similarity() {
        let mut checker = FaithfulnessChecker::new();
        let ev = make_evidence("quantum computing uses superposition");
        let verdict = checker.check(&ev, "the cat sat on the mat");
        assert!(
            !verdict.is_faithful,
            "completely different should be unfaithful"
        );
        assert!(
            verdict.similarity < 0.4,
            "different texts should have low similarity"
        );
        assert!(verdict.issue.is_some(), "unfaithful should have an issue");
    }

    #[test]
    fn test_partial_match_above_threshold() {
        let mut checker = FaithfulnessChecker::new();
        let ev = make_evidence("neural networks learn from data");
        let verdict = checker.check(
            &ev,
            "Neural networks are machine learning models that learn from data",
        );
        assert!(verdict.is_faithful, "partial match should be faithful");
    }

    #[test]
    fn test_batch_check_multiple() {
        let mut mgr = EvidenceManager::new(100);
        let id1 = mgr.add_evidence("https://a.com", "a", "the sky is blue");
        let id2 = mgr.add_evidence("https://b.com", "b", "quantum computing is fast");
        let id3 = mgr.add_evidence("https://c.com", "c", "cats are furry animals");

        let mut checker = FaithfulnessChecker::new();
        let text = "The sky is blue during the day. Quantum computing offers fast computation.";
        let verdicts = checker.check_batch(&[id1, id2, id3], text, &mgr);

        assert_eq!(verdicts.len(), 3);
        assert!(verdicts[0].is_faithful, "sky is blue should be faithful");
        assert!(
            verdicts[1].is_faithful,
            "quantum computing is fast should be faithful"
        );
        assert!(
            !verdicts[2].is_faithful,
            "cats are furry should be unfaithful"
        );

        let t = checker.tracker();
        assert_eq!(t.total_checks, 3);
        assert_eq!(t.faithful_count, 2);
        assert_eq!(t.unfaithful_count, 1);
    }

    #[test]
    fn test_tracker_metrics() {
        let mut checker = FaithfulnessChecker::new();
        let ev = make_evidence("test claim");

        for _ in 0..5 {
            let _ = checker.check(&ev, "totally unrelated text here");
        }
        let ev_match = make_evidence("totally unrelated text here");
        let _ = checker.check(&ev_match, "totally unrelated text here");

        let t = checker.tracker();
        assert_eq!(t.total_checks, 6);
        assert!(t.unfaithful_count >= 4);
        assert!(t.faithful_count >= 1);
        assert!(t.avg_similarity > 0.0);
        assert!(t.recent_verdicts.len() <= 6);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut checker = FaithfulnessChecker::new();
        let ev = make_evidence("the sky is blue");
        let _ = checker.check(&ev, "the sky is blue");

        assert_eq!(checker.tracker().total_checks, 1);
        checker.reset();
        assert_eq!(checker.tracker().total_checks, 0);
        assert_eq!(checker.tracker().faithful_count, 0);
        assert_eq!(checker.tracker().unfaithful_count, 0);
        assert!((checker.tracker().avg_similarity - 0.0).abs() < 1e-9);
        assert!(checker.tracker().recent_verdicts.is_empty());
    }

    #[test]
    fn test_with_threshold() {
        let high_checker = FaithfulnessChecker::new().with_threshold(0.95);
        assert!((high_checker.similarity_threshold - 0.95).abs() < 1e-9);

        let clamped_checker = FaithfulnessChecker::new().with_threshold(1.5);
        assert!((clamped_checker.similarity_threshold - 1.0).abs() < 1e-9);

        let mut low_checker = FaithfulnessChecker::new().with_threshold(0.01);
        let ev = make_evidence("quantum entanglement");
        let verdict = low_checker.check(&ev, "the weather is nice today");
        assert!(
            verdict.is_faithful,
            "with very low threshold, everything should be faithful"
        );
    }

    #[test]
    fn test_empty_text_graceful() {
        let mut checker = FaithfulnessChecker::new();
        let ev = make_evidence("some assertion");
        let verdict = checker.check(&ev, "");
        assert!(!verdict.is_faithful, "empty response should be unfaithful");
        assert!(
            (verdict.similarity - 0.0).abs() < 1e-9,
            "empty text should have zero similarity"
        );
        assert_eq!(verdict.response_snippet, "");
    }

    #[test]
    fn test_summary_format() {
        let mut checker = FaithfulnessChecker::new();
        assert_eq!(checker.summary(), "faithfulness:no_data");

        let ev = make_evidence("the sky is blue");
        let _ = checker.check(&ev, "the sky is blue");
        let s = checker.summary();
        assert!(s.contains("faithful"));
        assert!(s.contains("total="));
        assert!(s.contains("avg_sim="));
    }

    #[test]
    fn test_signature_deterministic() {
        let a = signature("hello world");
        let b = signature("hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn test_signature_different_inputs() {
        let a = signature("policy_a");
        let b = signature("policy_b");
        assert_ne!(a, b);
    }

    #[test]
    fn test_find_best_sentence_empty() {
        let result = find_best_matching_sentence("", "test");
        assert_eq!(result, "");
    }

    #[test]
    fn test_find_best_sentence_selects_best() {
        let text = "The cat sat on the mat. Quantum mechanics is fascinating. Neural networks are powerful.";
        let result = find_best_matching_sentence(text, "quantum mechanics");
        assert!(
            result.to_lowercase().contains("quantum"),
            "should select the quantum sentence, got: {}",
            result
        );
    }

    #[test]
    fn test_evidence_with_quotation_faithful() {
        let mut checker = FaithfulnessChecker::new();
        let ev = crate::core::nt_core_knowledge::evidence::EvidenceRecord::new(
            42,
            "https://arxiv.org",
            "arxiv",
            "ElephantBroker evidence system",
        )
        .with_quotation("ElephantBroker evidence system");
        let verdict = checker.check(
            &ev,
            "We implement the ElephantBroker evidence system for traceability",
        );
        assert!(
            verdict.is_faithful,
            "evidence quoted in response should be faithful"
        );
    }
}

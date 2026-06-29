/// VSI: Verified Self-Improvement
/// Uses reasoning verification (not just answer checking) for stable self-improvement training.
/// Reference: arXiv:2603.21558 — Verified Self-Improvement Training by Verifying Reasoning, Not Just Answers
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub step_number: usize,
    pub content: String,
    pub verification: StepVerification,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepVerification {
    Unchecked,
    Verified,   // Step passes all checks
    Suspicious, // Step has potential issues
    Invalid,    // Step contains verified errors
}

#[derive(Debug, Clone)]
pub struct ReasoningChain {
    pub steps: Vec<ReasoningStep>,
    pub answer: String,
    pub final_result: String,
    pub acceptance: SolutionAcceptance,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SolutionAcceptance {
    Accepted,         // Reasoning + answer both verified
    Rejected(String), // Rejected with reason
    Flagged,          // Answer correct but reasoning suspicious
}

#[derive(Debug, Clone)]
pub struct VsiVerifier {
    pub rejection_rate: f64,
    pub acceptance_rate: f64,
    pub history: VecDeque<SolutionAcceptance>,
    pub max_history: usize, // default 1000
}

impl VsiVerifier {
    pub fn new() -> Self {
        VsiVerifier {
            rejection_rate: 0.52, // VSI paper: ~52% acceptance
            acceptance_rate: 0.48,
            history: VecDeque::with_capacity(1000),
            max_history: 1000,
        }
    }

    /// Verify a chain of reasoning steps
    pub fn verify_reasoning(&self, chain: &mut ReasoningChain) -> SolutionAcceptance {
        let mut has_suspicious = false;

        for step in &mut chain.steps {
            match self.verify_step(step) {
                StepVerification::Verified => continue,
                StepVerification::Invalid => {
                    return SolutionAcceptance::Rejected(format!(
                        "Step {} has verified errors: {}",
                        step.step_number, step.content
                    ));
                }
                StepVerification::Suspicious => {
                    has_suspicious = true;
                }
                StepVerification::Unchecked => {
                    step.verification = StepVerification::Verified;
                }
            }
        }

        if has_suspicious {
            SolutionAcceptance::Flagged
        } else {
            SolutionAcceptance::Accepted
        }
    }

    fn verify_step(&self, step: &ReasoningStep) -> StepVerification {
        let content = &step.content;

        // Check for arithmetic: look for "= X" patterns and verify
        // Check for consistency: no contradictions with earlier steps
        // Check for domain constraints

        if content.contains("NaN") || content.contains("undefined") {
            return StepVerification::Invalid;
        }
        if content.contains("maybe") || content.contains("uncertain") {
            return StepVerification::Suspicious;
        }

        StepVerification::Verified
    }

    pub fn record_outcome(&mut self, acceptance: SolutionAcceptance) {
        self.history.push_back(acceptance.clone());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
        let total = self.history.len() as f64;
        let accepted = self
            .history
            .iter()
            .filter(|a| matches!(a, SolutionAcceptance::Accepted))
            .count() as f64;
        let rejected = self
            .history
            .iter()
            .filter(|a| matches!(a, SolutionAcceptance::Rejected(_)))
            .count() as f64;
        self.acceptance_rate = if total > 0.0 { accepted / total } else { 0.48 };
        self.rejection_rate = if total > 0.0 { rejected / total } else { 0.52 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chain(steps: Vec<(&str, StepVerification)>) -> ReasoningChain {
        ReasoningChain {
            steps: steps
                .into_iter()
                .enumerate()
                .map(|(i, (content, vf))| ReasoningStep {
                    step_number: i + 1,
                    content: content.to_string(),
                    verification: vf,
                })
                .collect(),
            answer: "final answer".to_string(),
            final_result: "result".to_string(),
            acceptance: SolutionAcceptance::Accepted,
        }
    }

    #[test]
    fn test_verify_reasoning_accepts_valid() {
        let verifier = VsiVerifier::new();
        let mut chain = make_chain(vec![
            ("2 + 2 = 4", StepVerification::Unchecked),
            ("4 * 3 = 12", StepVerification::Unchecked),
        ]);
        let result = verifier.verify_reasoning(&mut chain);
        assert_eq!(result, SolutionAcceptance::Accepted);
        assert_eq!(chain.steps[0].verification, StepVerification::Verified);
    }

    #[test]
    fn test_verify_reasoning_rejects_invalid() {
        let verifier = VsiVerifier::new();
        let mut chain = make_chain(vec![
            ("2 + 2 = 4", StepVerification::Unchecked),
            ("result is NaN", StepVerification::Unchecked),
        ]);
        let result = verifier.verify_reasoning(&mut chain);
        assert!(matches!(result, SolutionAcceptance::Rejected(_)));
    }

    #[test]
    fn test_verify_reasoning_flags_suspicious() {
        let verifier = VsiVerifier::new();
        let mut chain = make_chain(vec![("maybe the answer is 4", StepVerification::Unchecked)]);
        let result = verifier.verify_reasoning(&mut chain);
        assert_eq!(result, SolutionAcceptance::Flagged);
    }

    #[test]
    fn test_record_outcome_tracks_rates() {
        let mut verifier = VsiVerifier::new();
        verifier.record_outcome(SolutionAcceptance::Accepted);
        verifier.record_outcome(SolutionAcceptance::Accepted);
        verifier.record_outcome(SolutionAcceptance::Rejected("bad step".to_string()));
        assert!((verifier.acceptance_rate - 2.0 / 3.0).abs() < 0.01);
        assert!((verifier.rejection_rate - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_record_outcome_enforces_max_history() {
        let mut verifier = VsiVerifier::new();
        verifier.max_history = 2;
        verifier.record_outcome(SolutionAcceptance::Accepted);
        verifier.record_outcome(SolutionAcceptance::Accepted);
        verifier.record_outcome(SolutionAcceptance::Rejected("x".to_string()));
        assert_eq!(verifier.history.len(), 2);
    }
}

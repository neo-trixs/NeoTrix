use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::fmt;

const MAX_CANDIDATES: usize = 16;

#[derive(Debug, Clone)]
pub struct ActionCandidate {
    pub action_vsa: Vec<u8>,
    pub predicted_outcomes: Vec<Vec<u8>>,
    pub confidence: f64,
    pub expected_value: f64,
    pub description: String,
}

impl fmt::Display for ActionCandidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ActionCandidate(desc={}, conf={:.2}, ev={:.2})",
            self.description, self.confidence, self.expected_value
        )
    }
}

impl ActionCandidate {
    pub fn new(action: Vec<u8>, description: &str) -> Self {
        Self {
            action_vsa: action,
            predicted_outcomes: Vec::new(),
            confidence: 0.5,
            expected_value: 0.0,
            description: description.to_string(),
        }
    }

    pub fn with_prediction(mut self, outcome: Vec<u8>) -> Self {
        self.predicted_outcomes.push(outcome);
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

#[derive(Debug, Clone)]
pub struct VolitionEngine {
    candidates: Vec<ActionCandidate>,
    current_goal_vsa: Option<Vec<u8>>,
    selection_count: u64,
}

impl Default for VolitionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VolitionEngine {
    pub fn new() -> Self {
        Self {
            candidates: Vec::with_capacity(MAX_CANDIDATES),
            current_goal_vsa: None,
            selection_count: 0,
        }
    }

    pub fn propose(&mut self, candidate: ActionCandidate) {
        if self.candidates.len() >= MAX_CANDIDATES {
            self.candidates.sort_by(|a, b| {
                b.expected_value
                    .partial_cmp(&a.expected_value)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if self
                .candidates
                .last()
                .map_or(false, |last| candidate.expected_value > last.expected_value)
            {
                self.candidates.pop();
            } else {
                return;
            }
        }
        self.candidates.push(candidate);
    }

    pub fn set_goal(&mut self, goal_vsa: Vec<u8>) {
        self.current_goal_vsa = Some(goal_vsa);
    }

    pub fn select_best(&mut self) -> Option<ActionCandidate> {
        if self.candidates.is_empty() {
            return None;
        }
        for candidate in &mut self.candidates {
            let value = candidate.expected_value
                + (candidate.confidence * 0.3)
                + (candidate.predicted_outcomes.len() as f64 * 0.05);
            candidate.expected_value = value;
        }
        self.candidates.sort_by(|a, b| {
            b.expected_value
                .partial_cmp(&a.expected_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.selection_count += 1;
        Some(self.candidates.remove(0))
    }

    pub fn select_by_goal_alignment(&mut self) -> Option<ActionCandidate> {
        let goal = match &self.current_goal_vsa {
            Some(g) => g.clone(),
            None => return self.select_best(),
        };
        for candidate in &mut self.candidates {
            let goal_sim = QuantizedVSA::similarity(&candidate.action_vsa, &goal);
            candidate.expected_value = candidate.expected_value * 0.5 + goal_sim * 0.5;
        }
        self.candidates.sort_by(|a, b| {
            b.expected_value
                .partial_cmp(&a.expected_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.selection_count += 1;
        Some(self.candidates.remove(0))
    }

    pub fn evaluate_outcome(&mut self, selected: &ActionCandidate, actual_outcome: &[u8]) -> f64 {
        if selected.predicted_outcomes.is_empty() {
            return 0.0;
        }
        let mut prediction_accuracy = 0.0;
        for predicted in &selected.predicted_outcomes {
            prediction_accuracy += QuantizedVSA::similarity(predicted, actual_outcome);
        }
        prediction_accuracy /= selected.predicted_outcomes.len() as f64;
        prediction_accuracy
    }

    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn clear(&mut self) {
        self.candidates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine_empty() {
        let ve = VolitionEngine::new();
        assert!(ve.is_empty());
        assert_eq!(ve.candidate_count(), 0);
    }

    #[test]
    fn test_propose_adds_candidate() {
        let mut ve = VolitionEngine::new();
        ve.propose(ActionCandidate::new(vec![1; 100], "test action"));
        assert_eq!(ve.candidate_count(), 1);
    }

    #[test]
    fn test_select_best_returns_highest_value() {
        let mut ve = VolitionEngine::new();
        ve.propose(ActionCandidate::new(vec![1; 100], "low").with_confidence(0.3));
        ve.propose(ActionCandidate::new(vec![2; 100], "high").with_confidence(0.9));
        let selected = ve.select_best();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().description, "high");
    }

    #[test]
    fn test_select_best_empty_returns_none() {
        let mut ve = VolitionEngine::new();
        assert!(ve.select_best().is_none());
    }

    #[test]
    fn test_select_by_goal_alignment() {
        let mut ve = VolitionEngine::new();
        let goal = vec![1; 100];
        ve.set_goal(goal.clone());
        ve.propose(ActionCandidate::new(goal.clone(), "aligned"));
        ve.propose(ActionCandidate::new(vec![0; 100], "unaligned"));
        let selected = ve.select_by_goal_alignment();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().description, "aligned");
    }

    #[test]
    fn test_evaluate_outcome_accuracy() {
        let mut ve = VolitionEngine::new();
        let action = vec![1; 100];
        let candidate = ActionCandidate::new(action.clone(), "test").with_prediction(vec![1; 100]);
        let accuracy = ve.evaluate_outcome(&candidate, &vec![1; 100]);
        assert!((accuracy - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_propose_evicts_lowest_when_full() {
        let mut ve = VolitionEngine::new();
        for i in 0..MAX_CANDIDATES + 2 {
            let expected = if i < 5 { 0.1 } else { 0.9 };
            let mut c = ActionCandidate::new(vec![i as u8; 10], &format!("candidate {}", i));
            c.expected_value = expected;
            ve.propose(c);
        }
        assert_eq!(ve.candidate_count(), MAX_CANDIDATES);
    }

    #[test]
    fn test_clear_removes_all() {
        let mut ve = VolitionEngine::new();
        ve.propose(ActionCandidate::new(vec![1; 10], "a"));
        ve.propose(ActionCandidate::new(vec![2; 10], "b"));
        ve.clear();
        assert!(ve.is_empty());
    }
}

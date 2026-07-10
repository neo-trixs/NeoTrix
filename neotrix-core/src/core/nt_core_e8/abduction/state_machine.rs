#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbductiveState {
    Observation,
    HypothesisGeneration,
    Evaluation,
    Revision,
    Acceptance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbductiveStateMachine {
    pub state: AbductiveState,
    pub history: Vec<AbductiveState>,
    max_history: usize,
}

impl AbductiveStateMachine {
    pub fn new() -> Self {
        Self {
            state: AbductiveState::Observation,
            history: Vec::with_capacity(20),
            max_history: 20,
        }
    }

    pub fn current(&self) -> &AbductiveState {
        &self.state
    }

    pub fn transition(&mut self, target: AbductiveState) -> bool {
        if !self.can_transition(&self.state, &target) {
            return false;
        }
        self.history.push(self.state);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        self.state = target;
        true
    }

    pub fn can_transition(&self, from: &AbductiveState, to: &AbductiveState) -> bool {
        match (from, to) {
            (AbductiveState::Observation, AbductiveState::HypothesisGeneration) => true,
            (AbductiveState::HypothesisGeneration, AbductiveState::Evaluation) => true,
            (AbductiveState::Evaluation, AbductiveState::Revision) => true,
            (AbductiveState::Evaluation, AbductiveState::HypothesisGeneration) => true,
            (AbductiveState::Revision, AbductiveState::Acceptance) => true,
            (AbductiveState::Revision, AbductiveState::HypothesisGeneration) => true,
            _ => false,
        }
    }

    pub fn reset(&mut self) {
        self.state = AbductiveState::Observation;
        self.history.clear();
    }

    pub fn history(&self) -> &[AbductiveState] {
        &self.history
    }
}

impl Default for AbductiveStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let m = AbductiveStateMachine::new();
        assert_eq!(*m.current(), AbductiveState::Observation);
    }

    #[test]
    fn test_valid_transition_obs_to_hyp() {
        let mut m = AbductiveStateMachine::new();
        assert!(m.transition(AbductiveState::HypothesisGeneration));
        assert_eq!(*m.current(), AbductiveState::HypothesisGeneration);
    }

    #[test]
    fn test_full_chain_transition() {
        let mut m = AbductiveStateMachine::new();
        assert!(m.transition(AbductiveState::HypothesisGeneration));
        assert!(m.transition(AbductiveState::Evaluation));
        assert!(m.transition(AbductiveState::Revision));
        assert!(m.transition(AbductiveState::Acceptance));
        assert_eq!(*m.current(), AbductiveState::Acceptance);
    }

    #[test]
    fn test_invalid_transition() {
        let mut m = AbductiveStateMachine::new();
        assert!(!m.transition(AbductiveState::Acceptance));
        assert_eq!(*m.current(), AbductiveState::Observation);
    }

    #[test]
    fn test_invalid_from_acceptance() {
        let mut m = AbductiveStateMachine::new();
        m.transition(AbductiveState::HypothesisGeneration);
        m.transition(AbductiveState::Evaluation);
        m.transition(AbductiveState::Revision);
        m.transition(AbductiveState::Acceptance);
        assert!(!m.transition(AbductiveState::HypothesisGeneration));
        assert_eq!(*m.current(), AbductiveState::Acceptance);
    }

    #[test]
    fn test_reset() {
        let mut m = AbductiveStateMachine::new();
        m.transition(AbductiveState::HypothesisGeneration);
        m.transition(AbductiveState::Evaluation);
        m.reset();
        assert_eq!(*m.current(), AbductiveState::Observation);
        assert!(m.history().is_empty());
    }

    #[test]
    fn test_history_tracking() {
        let mut m = AbductiveStateMachine::new();
        m.transition(AbductiveState::HypothesisGeneration);
        m.transition(AbductiveState::Evaluation);
        assert_eq!(m.history().len(), 2);
        assert_eq!(m.history()[0], AbductiveState::Observation);
        assert_eq!(m.history()[1], AbductiveState::HypothesisGeneration);
    }

    #[test]
    fn test_revision_back_to_hypothesis() {
        let mut m = AbductiveStateMachine::new();
        m.transition(AbductiveState::HypothesisGeneration);
        m.transition(AbductiveState::Evaluation);
        m.transition(AbductiveState::Revision);
        assert!(m.transition(AbductiveState::HypothesisGeneration));
        assert_eq!(*m.current(), AbductiveState::HypothesisGeneration);
    }

    #[test]
    fn test_evaluation_back_to_hypothesis() {
        let mut m = AbductiveStateMachine::new();
        m.transition(AbductiveState::HypothesisGeneration);
        m.transition(AbductiveState::Evaluation);
        assert!(m.transition(AbductiveState::HypothesisGeneration));
    }
}

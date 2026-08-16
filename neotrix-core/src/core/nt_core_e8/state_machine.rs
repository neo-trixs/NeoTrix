use super::thinking_budget::DifficultyEstimator;
use crate::core::nt_core_hex::{FullReasoningState, ReasoningHexagram};
use crate::core::nt_core_ttc::{Allocation, TtcEngine};
use serde::{Deserialize, Serialize};

/// E₈ state machine wrapping FullReasoningState with TTC integration and multi-modal output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8StateMachine {
    pub current_state: FullReasoningState,
    pub state_trajectory: Vec<FullReasoningState>,
    #[serde(skip)]
    pub ttc_engine: Option<TtcEngine>,
    #[serde(skip)]
    pub budget: Option<Allocation>,
    pub scores: Vec<f64>,
    pub output_modality: String,
}

impl E8StateMachine {
    pub fn new(initial: FullReasoningState) -> Self {
        Self {
            current_state: initial,
            state_trajectory: Vec::new(),
            ttc_engine: None,
            budget: None,
            scores: Vec::new(),
            output_modality: "text".into(),
        }
    }

    pub fn set_output_modality(&mut self, modality: &str) {
        self.output_modality = modality.to_string();
    }

    pub fn set_ttc_engine(&mut self, engine: TtcEngine) {
        self.ttc_engine = Some(engine);
    }

    /// Transition to a new hexagram mode with optional TTC budget gating.
    /// If TTC is enabled and difficulty > 0.3, allocates budget and only
    /// allows the transition if the reasoning depth is within budget.
    pub fn transition(&mut self, target: ReasoningHexagram, task: &str) {
        self.state_trajectory.push(self.current_state);

        if let Some(ref ttc) = self.ttc_engine {
            if ttc.is_enabled() {
                let difficulty = DifficultyEstimator::heuristic_difficulty(task, "reasoning");
                if difficulty > 0.3 {
                    let remaining = 1.0 - (self.state_trajectory.len() as f64 * 0.1).min(0.9);
                    let allocation = ttc.allocate_budget(difficulty, remaining);
                    let current_depth = self.state_trajectory.len();
                    if current_depth >= allocation.budget.max_steps as usize {
                        return;
                    }
                    self.budget = Some(allocation);
                }
            }
        }

        self.current_state = self.current_state.transition_to(target);
    }

    /// Check whether to early-exit based on PRM score history.
    /// Returns true if the convergence signal indicates exit.
    pub fn check_early_exit(&mut self, score: f64) -> bool {
        self.scores.push(score);
        self.ttc_engine
            .as_ref()
            .is_some_and(|ttc| ttc.is_enabled() && ttc.check_early_exit(&self.scores).should_exit())
    }

    /// Reset TTC tracking (budget + scores) for a new reasoning session.
    pub fn reset_ttc(&mut self) {
        self.budget = None;
        self.scores.clear();
    }
}

impl From<FullReasoningState> for E8StateMachine {
    fn from(state: FullReasoningState) -> Self {
        Self {
            current_state: state,
            state_trajectory: Vec::new(),
            ttc_engine: None,
            budget: None,
            scores: Vec::new(),
            output_modality: "text".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hex::{MetaState, ReasoningHexagram};

    #[test]
    fn test_e8_state_machine_basic_transition() {
        let initial = FullReasoningState::new(ReasoningHexagram::new(0), MetaState::new(0));
        let mut sm = E8StateMachine::new(initial);
        let target = ReasoningHexagram::new(10);
        sm.transition(target, "simple query");
        assert_eq!(sm.current_state.mode, target);
        assert_eq!(sm.state_trajectory.len(), 1);
    }

    #[test]
    fn test_e8_state_machine_with_ttc_budget() {
        let initial = FullReasoningState::new(ReasoningHexagram::new(0), MetaState::new(0));
        let mut sm = E8StateMachine::new(initial);
        let ttc = TtcEngine::default();
        sm.set_ttc_engine(ttc);

        // High difficulty task should allocate budget
        let hard_task = "Implement a multi-threaded MapReduce framework with fault tolerance \
                         and exactly-once semantics in a distributed code system. Must guarantee \
                         data consistency under all conditions. Cannot tolerate data loss. Must \
                         recover from failures automatically. Need to ensure maximum throughput. \
                         Should minimize network overhead. Must not use blocking I/O.";
        let target = ReasoningHexagram::new(20);
        sm.transition(target, hard_task);
        assert!(sm.budget.is_some(), "Hard code task should allocate budget");
    }

    #[test]
    fn test_e8_state_machine_early_exit() {
        let initial = FullReasoningState::new(ReasoningHexagram::new(0), MetaState::new(0));
        let mut sm = E8StateMachine::new(initial);
        let ttc = TtcEngine::default();
        sm.set_ttc_engine(ttc);

        // First 2 calls: insufficient steps (< min_steps=3)
        assert!(!sm.check_early_exit(0.5));
        assert!(!sm.check_early_exit(0.5));

        // 3rd call: min_steps met, variance=0 → converged
        assert!(sm.check_early_exit(0.5));
    }

    #[test]
    fn test_e8_state_machine_reset_ttc() {
        let initial = FullReasoningState::new(ReasoningHexagram::new(0), MetaState::new(0));
        let mut sm = E8StateMachine::new(initial);
        let ttc = TtcEngine::default();
        sm.budget = Some(ttc.allocate_budget(0.9, 1.0));
        sm.set_ttc_engine(ttc);
        sm.scores.push(0.5);
        sm.reset_ttc();
        assert!(sm.budget.is_none());
        assert!(sm.scores.is_empty());
    }

    #[test]
    fn test_e8_state_machine_from_full_state() {
        let state = FullReasoningState::new(ReasoningHexagram::new(42), MetaState::new(2));
        let sm = E8StateMachine::from(state);
        assert_eq!(sm.current_state.mode.0, 42);
        assert_eq!(sm.current_state.meta.0, 2);
    }
}

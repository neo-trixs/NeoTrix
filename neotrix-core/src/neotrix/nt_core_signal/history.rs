//! 状态历史和意识层级
use crate::core::nt_core_ssm::SelectiveState;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub use crate::core::nt_core_ssm::ConsciousnessTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateHistory {
    max_size: usize,
    history: VecDeque<SelectiveState>,
}

impl StateHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            history: VecDeque::with_capacity(max_size),
        }
    }

    pub fn push(&mut self, state: SelectiveState) {
        if self.history.len() >= self.max_size {
            self.history.pop_front();
        }
        self.history.push_back(state);
    }

    pub fn get(&self, index: usize) -> Option<&SelectiveState> {
        self.history.get(index)
    }

    pub fn latest(&self) -> Option<&SelectiveState> {
        self.history.back()
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
}

impl Default for StateHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(data: Vec<f64>) -> SelectiveState {
        SelectiveState {
            data,
            hidden: vec![],
            importance: vec![],
            timestamp: 0,
        }
    }

    #[test]
    fn test_state_history_push_and_get() {
        let mut h = StateHistory::new(10);
        let s = make_state(vec![0.5]);
        h.push(s.clone());
        assert_eq!(h.get(0), Some(&s));
    }

    #[test]
    fn test_state_history_latest() {
        let mut h = StateHistory::new(10);
        let s1 = make_state(vec![0.3]);
        let s2 = make_state(vec![0.7]);
        h.push(s1);
        h.push(s2.clone());
        assert_eq!(h.latest(), Some(&s2));
    }

    #[test]
    fn test_state_history_max_size() {
        let mut h = StateHistory::new(3);
        for i in 0..5 {
            h.push(make_state(vec![i as f64 * 0.2]));
        }
        assert_eq!(h.len(), 3);
    }

    #[test]
    fn test_state_history_empty() {
        let h: StateHistory = StateHistory::new(10);
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
        assert_eq!(h.latest(), None);
    }

    #[test]
    fn test_consciousness_tier_from_score_mortal() {
        assert_eq!(
            ConsciousnessTier::from_score(0.1),
            ConsciousnessTier::Mortal
        );
    }

    #[test]
    fn test_consciousness_tier_from_score_awakened() {
        assert_eq!(
            ConsciousnessTier::from_score(0.4),
            ConsciousnessTier::Awakened
        );
    }

    #[test]
    fn test_consciousness_tier_from_score_enlightened() {
        assert_eq!(
            ConsciousnessTier::from_score(0.6),
            ConsciousnessTier::Enlightened
        );
    }

    #[test]
    fn test_consciousness_tier_from_score_ascended() {
        assert_eq!(
            ConsciousnessTier::from_score(0.8),
            ConsciousnessTier::Ascended
        );
    }

    #[test]
    fn test_consciousness_tier_from_score_transcendent() {
        assert_eq!(
            ConsciousnessTier::from_score(0.95),
            ConsciousnessTier::Transcendent
        );
    }

    #[test]
    fn test_consciousness_tier_from_score_boundary() {
        assert_eq!(
            ConsciousnessTier::from_score(0.3),
            ConsciousnessTier::Awakened
        );
        assert_eq!(
            ConsciousnessTier::from_score(0.5),
            ConsciousnessTier::Enlightened
        );
        assert_eq!(
            ConsciousnessTier::from_score(0.7),
            ConsciousnessTier::Ascended
        );
        assert_eq!(
            ConsciousnessTier::from_score(0.9),
            ConsciousnessTier::Transcendent
        );
    }

    #[test]
    fn test_consciousness_tier_threshold_mortal() {
        assert_eq!(ConsciousnessTier::Mortal.threshold(), (0.0, 0.3));
    }

    #[test]
    fn test_consciousness_tier_threshold_transcendent() {
        assert_eq!(ConsciousnessTier::Transcendent.threshold(), (0.9, 1.0));
    }
}

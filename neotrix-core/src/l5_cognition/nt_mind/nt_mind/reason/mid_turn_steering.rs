use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteeringAction {
    Continue,
    Redirect(String),
    Pause,
    Abort,
    Restart,
}

#[derive(Debug, Clone)]
pub struct SteeringDecision {
    pub action: SteeringAction,
    pub reason: String,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TurnState {
    pub turn_id: u32,
    pub progress: f64,
    pub tokens_used: u32,
    pub errors: u32,
    pub branches_taken: Vec<String>,
}

pub struct MidTurnSteerer {
    max_tokens: u32,
    max_errors: u32,
    min_progress_rate: f64,
}

impl MidTurnSteerer {
    pub fn new(max_tokens: u32, max_errors: u32, min_progress_rate: f64) -> Self {
        Self {
            max_tokens,
            max_errors,
            min_progress_rate,
        }
    }

    pub fn evaluate(&self, state: &TurnState) -> SteeringDecision {
        if state.errors >= self.max_errors {
            return SteeringDecision {
                action: SteeringAction::Abort,
                reason: format!("Too many errors: {} >= {}", state.errors, self.max_errors),
                confidence: 0.95,
                metadata: HashMap::new(),
            };
        }
        if state.tokens_used >= self.max_tokens {
            return SteeringDecision {
                action: SteeringAction::Pause,
                reason: format!("Token limit: {} >= {}", state.tokens_used, self.max_tokens),
                confidence: 0.9,
                metadata: HashMap::new(),
            };
        }
        if state.progress < self.min_progress_rate && state.turn_id > 5 {
            return SteeringDecision {
                action: SteeringAction::Redirect("Try a different approach".to_string()),
                reason: format!(
                    "Low progress: {:.2} < {:.2}",
                    state.progress, self.min_progress_rate
                ),
                confidence: 0.7,
                metadata: HashMap::new(),
            };
        }
        SteeringDecision {
            action: SteeringAction::Continue,
            reason: "On track".to_string(),
            confidence: 0.8,
            metadata: HashMap::new(),
        }
    }

    pub fn suggest_redirect(&self, state: &TurnState) -> Option<String> {
        if state.progress < self.min_progress_rate {
            Some("Consider simplifying the approach".to_string())
        } else {
            None
        }
    }

    pub fn should_restart(&self, state: &TurnState) -> bool {
        state.errors >= self.max_errors && state.progress < 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continue() {
        let s = MidTurnSteerer::new(10000, 3, 0.1);
        let state = TurnState {
            turn_id: 1,
            progress: 0.5,
            tokens_used: 100,
            errors: 0,
            branches_taken: vec![],
        };
        let d = s.evaluate(&state);
        assert_eq!(d.action, SteeringAction::Continue);
    }

    #[test]
    fn test_abort_on_errors() {
        let s = MidTurnSteerer::new(10000, 3, 0.1);
        let state = TurnState {
            turn_id: 5,
            progress: 0.2,
            tokens_used: 500,
            errors: 3,
            branches_taken: vec![],
        };
        let d = s.evaluate(&state);
        assert_eq!(d.action, SteeringAction::Abort);
    }

    #[test]
    fn test_pause_on_tokens() {
        let s = MidTurnSteerer::new(100, 3, 0.1);
        let state = TurnState {
            turn_id: 1,
            progress: 0.5,
            tokens_used: 100,
            errors: 0,
            branches_taken: vec![],
        };
        let d = s.evaluate(&state);
        assert_eq!(d.action, SteeringAction::Pause);
    }

    #[test]
    fn test_redirect_low_progress() {
        let s = MidTurnSteerer::new(10000, 3, 0.1);
        let state = TurnState {
            turn_id: 10,
            progress: 0.05,
            tokens_used: 500,
            errors: 1,
            branches_taken: vec![],
        };
        let d = s.evaluate(&state);
        assert!(matches!(d.action, SteeringAction::Redirect(_)));
    }
}

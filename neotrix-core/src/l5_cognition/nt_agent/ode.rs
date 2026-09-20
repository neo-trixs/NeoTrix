//! Observe-Decide-Execute loop -- inspired by Jev-cu.
//! AX parse -> candidate selection -> decision -> policy gate -> execute.

/// An observation from the environment
#[derive(Debug, Clone)]
pub struct Observation {
    pub content: String,
    pub source: String,
    pub timestamp: u64,
}

/// A candidate action
#[derive(Debug, Clone)]
pub struct ActionCandidate {
    pub id: String,
    pub description: String,
    pub risk_score: f64,
    pub goal_relevance: f64,
}

/// Decision output
#[derive(Debug, Clone)]
pub struct OdeDecision {
    pub selected_action: Option<ActionCandidate>,
    pub confidence: f64,
    pub done: bool,
    pub reasoning: String,
}

/// Policy gate result
#[derive(Debug, Clone)]
pub enum PolicyGateResult {
    Allow,
    Block { reason: String },
    Review { reason: String },
}

/// Policy gate configuration
#[derive(Debug, Clone)]
pub struct PolicyGate {
    pub blocked_actions: Vec<String>,
    pub max_risk_threshold: f64,
}

impl Default for PolicyGate {
    fn default() -> Self {
        Self {
            blocked_actions: Vec::new(),
            max_risk_threshold: 0.8,
        }
    }
}

impl PolicyGate {
    pub fn evaluate(&self, candidate: &ActionCandidate) -> PolicyGateResult {
        if self.blocked_actions.contains(&candidate.description) {
            PolicyGateResult::Block {
                reason: "Action is blocked by policy".into(),
            }
        } else if candidate.risk_score > self.max_risk_threshold {
            PolicyGateResult::Review {
                reason: "Risk exceeds threshold".into(),
            }
        } else {
            PolicyGateResult::Allow
        }
    }
}

/// The ODE loop orchestrator
pub struct ObserveDecideExecute {
    policy: PolicyGate,
    history: Vec<OdeDecision>,
}

impl ObserveDecideExecute {
    pub fn new(policy: PolicyGate) -> Self {
        Self {
            policy,
            history: Vec::new(),
        }
    }

    /// Observe the environment and return candidates
    pub fn observe(&self, _obs: &Observation) -> Vec<ActionCandidate> {
        // Placeholder -- will be wired to actual perception
        Vec::new()
    }

    /// Decide which action to take
    pub fn decide(&self, candidates: &[ActionCandidate]) -> OdeDecision {
        if candidates.is_empty() {
            return OdeDecision {
                selected_action: None,
                confidence: 1.0,
                done: true,
                reasoning: "No candidates".into(),
            };
        }
        let best = candidates
            .iter()
            .max_by(|a, b| a.goal_relevance.partial_cmp(&b.goal_relevance).unwrap())
            .unwrap();
        let gate_result = self.policy.evaluate(best);
        match gate_result {
            PolicyGateResult::Allow => OdeDecision {
                selected_action: Some(best.clone()),
                confidence: best.goal_relevance,
                done: false,
                reasoning: "Policy allows".into(),
            },
            PolicyGateResult::Block { reason } => OdeDecision {
                selected_action: None,
                confidence: 0.0,
                done: false,
                reasoning: reason,
            },
            PolicyGateResult::Review { reason } => OdeDecision {
                selected_action: Some(best.clone()),
                confidence: best.goal_relevance * 0.5,
                done: false,
                reasoning: reason,
            },
        }
    }

    /// Execute the decision (placeholder)
    pub fn execute(&mut self, decision: OdeDecision) -> Result<(), String> {
        self.history.push(decision);
        Ok(())
    }

    pub fn history(&self) -> &[OdeDecision] {
        &self.history
    }
}

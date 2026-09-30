//! Action loop pattern (from jev-browser).
//!
//! Each step: choose action from candidates → evaluate goal/stuck → execute → repeat.
//! Stop conditions: done chosen, goal achieved, stuck detected, budget exhausted.

use super::primitives::*;

/// An available action the agent can take.
#[derive(Debug, Clone)]
pub struct ActionCandidate {
    /// Unique action ID (e.g. "click_e2", "type_e1", "scroll_down").
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// Action type for categorization.
    pub action_type: ActionType,
}

/// Type of action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    /// Click an element.
    Click,
    /// Type text into a field.
    Type,
    /// Submit a form.
    Submit,
    /// Scroll the page.
    Scroll,
    /// Navigate back.
    Back,
    /// Done / stop.
    Done,
}

/// Result of executing a single step.
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Step number (1-indexed).
    pub step: usize,
    /// The proposed action.
    pub proposed: ActionCandidate,
    /// The action actually executed (may differ from proposed).
    pub executed: Option<ActionCandidate>,
    /// Detail string about what happened.
    pub detail: String,
    /// Choice confidence for this step.
    pub confidence: f64,
    /// Goal probability (0.0-1.0).
    pub goal_probability: f64,
    /// Stuck probability (0.0-1.0).
    pub stuck_probability: f64,
}

/// Stop condition that ended the loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopReason {
    /// Agent chose "done".
    Done,
    /// Goal achieved (probability > threshold).
    GoalAchieved,
    /// Agent detected stuck (probability > threshold).
    Stuck,
    /// Step budget exhausted.
    MaxSteps,
    /// Time budget exhausted.
    Timeout,
    /// An error occurred.
    Error(String),
}

/// Configuration for the action loop.
#[derive(Debug, Clone)]
pub struct ActionLoopConfig {
    /// Maximum steps before forced stop (default 24).
    pub max_steps: usize,
    /// Maximum seconds before timeout (default 180).
    pub max_seconds: u64,
    /// Goal probability threshold to stop (default 0.85).
    pub goal_threshold: f64,
    /// Stuck probability threshold to stop (default 0.85).
    pub stuck_threshold: f64,
}

impl Default for ActionLoopConfig {
    fn default() -> Self {
        Self {
            max_steps: 24,
            max_seconds: 180,
            goal_threshold: 0.85,
            stuck_threshold: 0.85,
        }
    }
}

/// The action loop state tracker.
pub struct ActionLoop {
    config: ActionLoopConfig,
    step_results: Vec<StepResult>,
    current_step: usize,
    start_time: std::time::Instant,
}

impl ActionLoop {
    /// Create a new action loop.
    pub fn new(config: ActionLoopConfig) -> Self {
        Self {
            config,
            step_results: Vec::new(),
            current_step: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// Check if the loop should stop, returning the reason if so.
    pub fn should_stop(
        &self,
        choice: &ChoiceAnswer,
        goal: &NoulAnswer,
        stuck: &NoulAnswer,
    ) -> Option<StopReason> {
        // Step budget
        if self.current_step >= self.config.max_steps {
            return Some(StopReason::MaxSteps);
        }

        // Time budget
        let elapsed = self.start_time.elapsed().as_secs();
        if elapsed >= self.config.max_seconds {
            return Some(StopReason::Timeout);
        }

        // Agent chose done
        if choice.choice == "done" {
            return Some(StopReason::Done);
        }

        // Goal achieved
        if goal.noul >= self.config.goal_threshold {
            return Some(StopReason::GoalAchieved);
        }

        // Stuck detected
        if stuck.noul >= self.config.stuck_threshold {
            return Some(StopReason::Stuck);
        }

        None
    }

    /// Record a step result.
    pub fn record_step(&mut self, result: StepResult) {
        self.step_results.push(result);
        self.current_step += 1;
    }

    /// Get all step results.
    pub fn steps(&self) -> &[StepResult] {
        &self.step_results
    }

    /// Get the current step number.
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Compute elapsed milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// Get the final summary.
    pub fn summary(&self) -> ActionLoopSummary {
        let total_confidence: f64 = self.step_results.iter().map(|s| s.confidence).sum();
        let avg_confidence = if self.step_results.is_empty() {
            0.0
        } else {
            total_confidence / self.step_results.len() as f64
        };

        ActionLoopSummary {
            total_steps: self.step_results.len(),
            elapsed_ms: self.elapsed_ms(),
            avg_confidence,
            steps: self.step_results.clone(),
        }
    }
}

/// Summary of a completed action loop.
#[derive(Debug, Clone)]
pub struct ActionLoopSummary {
    pub total_steps: usize,
    pub elapsed_ms: u64,
    pub avg_confidence: f64,
    pub steps: Vec<StepResult>,
}

/// Select the best action from candidates based on choice probabilities.
/// Returns the action with highest probability, or None if empty.
pub fn select_action<'a>(
    candidates: &'a [ActionCandidate],
    choice: &ChoiceAnswer,
) -> Option<&'a ActionCandidate> {
    if candidates.is_empty() {
        return None;
    }
    candidates.iter().find(|c| c.id == choice.choice)
}

/// Select next-best action when the proposed action fails.
/// Returns the candidate with the second-highest probability.
pub fn select_next_best<'a>(
    candidates: &'a [ActionCandidate],
    choice: &ChoiceAnswer,
    failed_id: &str,
) -> Option<&'a ActionCandidate> {
    // Sort probabilities descending
    let mut sorted: Vec<(&String, &f64)> = choice.probabilities.iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (id, _) in sorted {
        if id.as_str() != failed_id {
            if let Some(candidate) = candidates.iter().find(|c| c.id == *id) {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_candidates() -> Vec<ActionCandidate> {
        vec![
            ActionCandidate {
                id: "click_e1".to_string(),
                description: "Click search".to_string(),
                action_type: ActionType::Click,
            },
            ActionCandidate {
                id: "click_e2".to_string(),
                description: "Click link".to_string(),
                action_type: ActionType::Click,
            },
            ActionCandidate {
                id: "done".to_string(),
                description: "Stop".to_string(),
                action_type: ActionType::Done,
            },
        ]
    }

    fn make_choice(choice: &str, prob: f64) -> ChoiceAnswer {
        let mut probabilities = HashMap::new();
        probabilities.insert(choice.to_string(), prob);
        ChoiceAnswer {
            choice: choice.to_string(),
            probabilities,
            confidence: prob,
            margin: 0.0,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        }
    }

    #[test]
    fn test_should_stop_done() {
        let loop_ = ActionLoop::new(ActionLoopConfig::default());
        let choice = make_choice("done", 0.95);
        let goal = NoulAnswer {
            noul: 0.0,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        let stuck = NoulAnswer {
            noul: 0.0,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        assert_eq!(
            loop_.should_stop(&choice, &goal, &stuck),
            Some(StopReason::Done)
        );
    }

    #[test]
    fn test_should_stop_goal() {
        let loop_ = ActionLoop::new(ActionLoopConfig::default());
        let choice = make_choice("click_e1", 0.8);
        let goal = NoulAnswer {
            noul: 0.9,
            needs_review: false,
            reason: Some("goal met".to_string()),
            status: DecisionStatus::Selected,
        };
        let stuck = NoulAnswer {
            noul: 0.0,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        assert_eq!(
            loop_.should_stop(&choice, &goal, &stuck),
            Some(StopReason::GoalAchieved)
        );
    }

    #[test]
    fn test_should_stop_stuck() {
        let loop_ = ActionLoop::new(ActionLoopConfig::default());
        let choice = make_choice("click_e1", 0.8);
        let goal = NoulAnswer {
            noul: 0.0,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        let stuck = NoulAnswer {
            noul: 0.9,
            needs_review: false,
            reason: Some("stuck".to_string()),
            status: DecisionStatus::Selected,
        };
        assert_eq!(
            loop_.should_stop(&choice, &goal, &stuck),
            Some(StopReason::Stuck)
        );
    }

    #[test]
    fn test_should_not_stop() {
        let loop_ = ActionLoop::new(ActionLoopConfig::default());
        let choice = make_choice("click_e1", 0.8);
        let goal = NoulAnswer {
            noul: 0.0,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        let stuck = NoulAnswer {
            noul: 0.0,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        assert_eq!(loop_.should_stop(&choice, &goal, &stuck), None);
    }

    #[test]
    fn test_select_action() {
        let candidates = make_candidates();
        let choice = make_choice("click_e1", 0.9);
        let selected = select_action(&candidates, &choice);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "click_e1");
    }

    #[test]
    fn test_select_action_empty() {
        let candidates = vec![];
        let choice = make_choice("done", 0.9);
        assert!(select_action(&candidates, &choice).is_none());
    }

    #[test]
    fn test_action_loop_summary() {
        let mut loop_ = ActionLoop::new(ActionLoopConfig::default());
        loop_.record_step(StepResult {
            step: 1,
            proposed: make_candidates()[0].clone(),
            executed: Some(make_candidates()[0].clone()),
            detail: "clicked".to_string(),
            confidence: 0.9,
            goal_probability: 0.3,
            stuck_probability: 0.1,
        });
        let summary = loop_.summary();
        assert_eq!(summary.total_steps, 1);
        assert!((summary.avg_confidence - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_select_next_best() {
        let candidates = make_candidates();
        let mut probs = HashMap::new();
        probs.insert("click_e1".to_string(), 0.6);
        probs.insert("click_e2".to_string(), 0.3);
        probs.insert("done".to_string(), 0.1);
        let choice = ChoiceAnswer {
            choice: "click_e1".to_string(),
            probabilities: probs,
            confidence: 0.6,
            margin: 0.3,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        };
        let next = select_next_best(&candidates, &choice, "click_e1");
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, "click_e2");
    }
}

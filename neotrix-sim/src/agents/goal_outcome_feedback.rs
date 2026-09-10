use serde::{Deserialize, Serialize};

use crate::agents::planning::{Goal, GoalStatus, GoalType, PlanningStack};
use crate::agents::action_awareness::{ActionAwareness, ExpectedOutcome, ObservedOutcome};
use crate::agents::sim_agent::AgentAction;

/// Result of evaluating a goal-outcome pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalOutcomeRecord {
    pub goal_name: String,
    pub goal_type: GoalType,
    pub action_taken: AgentAction,
    pub expected: ExpectedOutcome,
    pub observed: ObservedOutcome,
    pub success_score: f32,
    pub tick: u64,
}

/// GoalOutcomeFeedback — Adapter 1
///
/// After each action execution, records expected vs actual outcomes and
/// updates PlanningStack goals based on success/failure. Implements the
/// "goal-outcome self-reflection" loop from Generative Agents (A6/P0-2).
///
/// Sources:
/// - Generative Agents (Park 2023): post-action reflection on goal achievement
/// - BDI architecture: intention evaluation after execution
pub struct GoalOutcomeFeedback {
    records: Vec<GoalOutcomeRecord>,
    max_records: usize,
    success_threshold: f32,
}

impl GoalOutcomeFeedback {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            max_records: 200,
            success_threshold: 0.3,
        }
    }

    /// Record a goal-outcome pair after action execution.
    /// Evaluates whether the action advanced the goal and updates goal status.
    pub fn record_outcome(
        &mut self,
        goal: &Goal,
        action: &AgentAction,
        awareness: &ActionAwareness,
        agent_energy: f32,
        agent_health: f32,
        tick: u64,
    ) -> Option<GoalOutcomeRecord> {
        let expected = awareness.last_expected.as_ref()?;
        let observed = ObservedOutcome {
            actual_position: crate::foundation::math_bridge::Vec2::new(0.0, 0.0),
            actual_energy: agent_energy,
            actual_health: agent_health,
            success: agent_energy > 0.0 && agent_health > 0.0,
        };

        let success_score = self.evaluate_goal_success(goal, action, &expected, &observed);

        let record = GoalOutcomeRecord {
            goal_name: goal.name.clone(),
            goal_type: goal.goal_type.clone(),
            action_taken: action.clone(),
            expected: expected.clone(),
            observed,
            success_score,
            tick,
        };

        self.records.push(record.clone());
        if self.records.len() > self.max_records {
            self.records.remove(0);
        }

        Some(record)
    }

    /// Update PlanningStack based on outcome success/failure.
    pub fn update_planning(
        &self,
        planning: &mut PlanningStack,
        record: &GoalOutcomeRecord,
    ) {
        if record.success_score >= self.success_threshold {
            // Success: advance the goal's action queue
            planning.advance_action();
        } else {
            // Failure: mark current goal as failed, let consolidate() clean up
            if let Some(goal) = planning.goals_mut().iter_mut().find(|g| {
                g.status == GoalStatus::Active && g.name == record.goal_name
            }) {
                goal.status = GoalStatus::Failed;
            }
        }
    }

    /// Evaluate how well the action advanced the goal (0.0 = total failure, 1.0 = perfect).
    fn evaluate_goal_success(
        &self,
        goal: &Goal,
        action: &AgentAction,
        expected: &ExpectedOutcome,
        observed: &ObservedOutcome,
    ) -> f32 {
        let mut score = 0.0f32;

        match goal.goal_type {
            GoalType::Survival => {
                match goal.name.as_str() {
                    "eat" => {
                        // Success if energy increased or hunger decreased
                        if observed.actual_energy > expected.expected_energy_delta.abs() {
                            score += 0.5;
                        }
                        if !observed.success {
                            score -= 0.5;
                        }
                    }
                    "rest" => {
                        if observed.actual_energy > 50.0 {
                            score += 0.5;
                        }
                    }
                    "flee" => {
                        // Flee success = still alive
                        if observed.success {
                            score += 0.5;
                        }
                    }
                    _ => {}
                }
            }
            GoalType::Social => {
                if matches!(action, AgentAction::Talk { .. } | AgentAction::Trade { .. }) {
                    score += 0.3;
                }
                if observed.success {
                    score += 0.2;
                }
            }
            GoalType::Exploration => {
                if matches!(action, AgentAction::Explore { .. }) {
                    score += 0.3;
                }
            }
            GoalType::Achievement | GoalType::Curiosity => {
                // Generic: survival + action taken = partial success
                if observed.success {
                    score += 0.2;
                }
            }
        }

        // Clamp to [0.0, 1.0]
        score.clamp(0.0, 1.0)
    }

    /// Get recent records for analysis
    pub fn recent(&self, n: usize) -> &[GoalOutcomeRecord] {
        let start = self.records.len().saturating_sub(n);
        &self.records[start..]
    }

    /// Compute overall success rate across recent records
    pub fn recent_success_rate(&self, n: usize) -> f32 {
        let recent = self.recent(n);
        if recent.is_empty() {
            return 0.5; // neutral default
        }
        let total: f32 = recent.iter().map(|r| r.success_score).sum();
        total / recent.len() as f32
    }

    pub fn records(&self) -> &[GoalOutcomeRecord] {
        &self.records
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for GoalOutcomeFeedback {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::action_awareness::{ExpectedOutcome, ObservedOutcome};
    use crate::agents::planning::{Goal, GoalStatus, GoalType};
    use crate::agents::sim_agent::AgentAction;
    use crate::foundation::math_bridge::Vec2;

    fn make_goal(name: &str, goal_type: GoalType) -> Goal {
        Goal {
            name: name.to_string(),
            priority: 0.8,
            goal_type,
            status: GoalStatus::Active,
            subgoals: vec![],
            created_tick: 0,
            deadline_tick: None,
            expected_actions: vec![AgentAction::Think],
        }
    }

    #[test]
    fn new_has_no_records() {
        let fb = GoalOutcomeFeedback::new();
        assert!(fb.is_empty());
        assert_eq!(fb.len(), 0);
    }

    #[test]
    fn record_outcome_returns_record() {
        let mut fb = GoalOutcomeFeedback::new();
        let mut awareness = ActionAwareness::new();
        let agent = crate::agents::sim_agent::SimAgent::new(1, Vec2::new(50.0, 50.0));
        awareness.predict(&AgentAction::Rest, &agent);

        let goal = make_goal("rest", GoalType::Survival);
        let record = fb.record_outcome(&goal, &AgentAction::Rest, &awareness, 80.0, 90.0, 10);
        assert!(record.is_some());
        assert_eq!(fb.len(), 1);
    }

    #[test]
    fn update_planning_advances_on_success() {
        let fb = GoalOutcomeFeedback::new();
        let mut planning = PlanningStack::new();
        planning.add_goal(Goal {
            name: "eat".to_string(),
            priority: 0.9,
            goal_type: GoalType::Survival,
            status: GoalStatus::Active,
            subgoals: vec![],
            created_tick: 0,
            deadline_tick: None,
            expected_actions: vec![AgentAction::Think, AgentAction::Rest],
        });

        let record = GoalOutcomeRecord {
            goal_name: "eat".to_string(),
            goal_type: GoalType::Survival,
            action_taken: AgentAction::Eat { resource_id: "r1".into() },
            expected: ExpectedOutcome {
                action: AgentAction::Eat { resource_id: "r1".into() },
                expected_position: None,
                expected_energy_delta: 10.0,
                expected_health_delta: 0.0,
                expected_resource_change: None,
            },
            observed: ObservedOutcome {
                actual_position: Vec2::new(50.0, 50.0),
                actual_energy: 60.0,
                actual_health: 80.0,
                success: true,
            },
            success_score: 0.8,
            tick: 10,
        };

        fb.update_planning(&mut planning, &record);
        // Action should have been advanced
        assert!(matches!(planning.next_action(), Some(AgentAction::Rest)));
    }

    #[test]
    fn update_planning_fails_on_low_score() {
        let fb = GoalOutcomeFeedback::new();
        let mut planning = PlanningStack::new();
        planning.add_goal(Goal {
            name: "eat".to_string(),
            priority: 0.9,
            goal_type: GoalType::Survival,
            status: GoalStatus::Active,
            subgoals: vec![],
            created_tick: 0,
            deadline_tick: None,
            expected_actions: vec![AgentAction::Think],
        });

        let record = GoalOutcomeRecord {
            goal_name: "eat".to_string(),
            goal_type: GoalType::Survival,
            action_taken: AgentAction::Eat { resource_id: "r1".into() },
            expected: ExpectedOutcome {
                action: AgentAction::Eat { resource_id: "r1".into() },
                expected_position: None,
                expected_energy_delta: 10.0,
                expected_health_delta: 0.0,
                expected_resource_change: None,
            },
            observed: ObservedOutcome {
                actual_position: Vec2::new(50.0, 50.0),
                actual_energy: 0.0,
                actual_health: 0.0,
                success: false,
            },
            success_score: 0.1,
            tick: 10,
        };

        fb.update_planning(&mut planning, &record);
        // Goal should be marked failed
        assert_eq!(planning.goals()[0].status, GoalStatus::Failed);
    }

    #[test]
    fn recent_success_rate_empty() {
        let fb = GoalOutcomeFeedback::new();
        assert_eq!(fb.recent_success_rate(10), 0.5);
    }

    #[test]
    fn recent_success_rate_computes() {
        let mut fb = GoalOutcomeFeedback::new();
        for i in 0..5 {
            fb.records.push(GoalOutcomeRecord {
                goal_name: "test".into(),
                goal_type: GoalType::Survival,
                action_taken: AgentAction::Rest,
                expected: ExpectedOutcome {
                    action: AgentAction::Rest,
                    expected_position: None,
                    expected_energy_delta: 5.0,
                    expected_health_delta: 1.0,
                    expected_resource_change: None,
                },
                observed: ObservedOutcome {
                    actual_position: Vec2::new(0.0, 0.0),
                    actual_energy: 80.0,
                    actual_health: 90.0,
                    success: true,
                },
                success_score: if i < 3 { 0.8 } else { 0.2 },
                tick: i,
            });
        }
        let rate = fb.recent_success_rate(5);
        assert!(rate > 0.0 && rate <= 1.0);
    }
}

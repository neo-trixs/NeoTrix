use std::collections::HashMap;

use crate::agents::planning::{GoalStatus, GoalType, PlanningStack};
use crate::agents::sim_agent::AgentAction;

/// Tracks a single committed intention for an agent
#[derive(Debug, Clone)]
pub struct CommittedIntention {
    pub goal_name: String,
    pub goal_type: GoalType,
    pub action: AgentAction,
    pub committed_at_tick: u64,
    pub commitment_duration: u64,
    pub streak: u32,
}

/// IntentionCommitment — Adapter 2
///
/// When PlanningStack selects a goal, lock the intention for N ticks.
/// Don't re-evaluate until N ticks pass or situation changes drastically.
/// Implements BDI "intention commitment" pattern (A5/P2-1).
///
/// Sources:
/// - BDI Architecture (JaKtA): commitment strategy — once agent commits to
///   an intention, it follows through until the goal is achieved or becomes
///   impossible.
/// - Project Sid (PIANO): agents maintain persistent goal states that
///   persist across ticks.
pub struct IntentionCommitment {
    commitments: HashMap<String, CommittedIntention>,
    default_duration: u64,
    max_streak: u32,
    critical_health_threshold: f32,
    critical_energy_threshold: f32,
}

impl IntentionCommitment {
    pub fn new() -> Self {
        Self {
            commitments: HashMap::new(),
            default_duration: 10,
            max_streak: 20,
            critical_health_threshold: 20.0,
            critical_energy_threshold: 15.0,
        }
    }

    /// Try to commit to the current goal's action.
    /// Returns the committed action if successful, or None if already committed.
    pub fn try_commit(
        &mut self,
        agent_id: &str,
        planning: &PlanningStack,
        tick: u64,
    ) -> Option<AgentAction> {
        let goal = planning.current_goal()?;
        let action = planning.next_action()?.clone();

        // Check if already committed
        if let Some(existing) = self.commitments.get(agent_id) {
            if tick < existing.committed_at_tick + existing.commitment_duration {
                return None; // Still in commitment period
            }
            // Commitment expired — check if same goal is still active
            if existing.goal_name == goal.name && goal.status == GoalStatus::Active {
                // Extend streak if same goal
                let streak = (existing.streak + 1).min(self.max_streak);
                self.commitments.insert(agent_id.to_string(), CommittedIntention {
                    goal_name: goal.name.clone(),
                    goal_type: goal.goal_type.clone(),
                    action: action.clone(),
                    committed_at_tick: tick,
                    commitment_duration: self.default_duration,
                    streak,
                });
                return Some(action);
            }
        }

        // New commitment
        self.commitments.insert(agent_id.to_string(), CommittedIntention {
            goal_name: goal.name.clone(),
            goal_type: goal.goal_type.clone(),
            action: action.clone(),
            committed_at_tick: tick,
            commitment_duration: self.default_duration,
            streak: 1,
        });

        Some(action)
    }

    /// Check if the agent is currently committed (should not re-evaluate).
    pub fn is_committed(&self, agent_id: &str, tick: u64) -> bool {
        if let Some(c) = self.commitments.get(agent_id) {
            tick < c.committed_at_tick + c.commitment_duration
        } else {
            false
        }
    }

    /// Check if a critical situation (low health/energy) should force commitment break.
    pub fn should_break_commitment(
        &self,
        agent_id: &str,
        health: f32,
        energy: f32,
    ) -> bool {
        if let Some(c) = self.commitments.get(agent_id) {
            // Survival goals should never be broken
            if matches!(c.goal_type, GoalType::Survival) {
                return false;
            }
            // Break if in critical condition
            health < self.critical_health_threshold || energy < self.critical_energy_threshold
        } else {
            false
        }
    }

    /// Force-break a commitment (critical situation override).
    pub fn break_commitment(&mut self, agent_id: &str) -> Option<CommittedIntention> {
        self.commitments.remove(agent_id)
    }

    /// Get current commitment info
    pub fn get_commitment(&self, agent_id: &str) -> Option<&CommittedIntention> {
        self.commitments.get(agent_id)
    }

    /// Get intention streak (how many consecutive ticks on same goal)
    pub fn streak(&self, agent_id: &str) -> u32 {
        self.commitments.get(agent_id).map(|c| c.streak).unwrap_or(0)
    }

    /// Remove commitment for dead agents
    pub fn cleanup(&mut self, alive_agent_ids: &[String]) {
        self.commitments.retain(|id, _| alive_agent_ids.contains(id));
    }

    pub fn len(&self) -> usize {
        self.commitments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commitments.is_empty()
    }
}

impl Default for IntentionCommitment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::planning::{Goal, GoalStatus, GoalType, PlanningStack};
    use crate::agents::sim_agent::AgentAction;

    fn make_planning_with_goal(name: &str, goal_type: GoalType) -> PlanningStack {
        let mut ps = PlanningStack::new();
        ps.add_goal(Goal {
            name: name.to_string(),
            priority: 0.8,
            goal_type,
            status: GoalStatus::Active,
            subgoals: vec![],
            created_tick: 0,
            deadline_tick: None,
            expected_actions: vec![AgentAction::Think],
        });
        ps
    }

    #[test]
    fn new_has_no_commitments() {
        let ic = IntentionCommitment::new();
        assert!(ic.is_empty());
        assert_eq!(ic.len(), 0);
    }

    #[test]
    fn try_commit_returns_action() {
        let mut ic = IntentionCommitment::new();
        let ps = make_planning_with_goal("explore", GoalType::Exploration);
        let result = ic.try_commit("agent_0", &ps, 0);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), AgentAction::Think));
    }

    #[test]
    fn try_commit_blocks_during_commitment_period() {
        let mut ic = IntentionCommitment::new();
        let ps = make_planning_with_goal("explore", GoalType::Exploration);
        ic.try_commit("agent_0", &ps, 0);
        // Within commitment period (default 10 ticks)
        let result = ic.try_commit("agent_0", &ps, 5);
        assert!(result.is_none());
    }

    #[test]
    fn try_commit_allows_after_expiry() {
        let mut ic = IntentionCommitment::new();
        let ps = make_planning_with_goal("explore", GoalType::Exploration);
        ic.try_commit("agent_0", &ps, 0);
        // After commitment period
        let result = ic.try_commit("agent_0", &ps, 15);
        assert!(result.is_some());
    }

    #[test]
    fn is_committed_returns_true_during_period() {
        let mut ic = IntentionCommitment::new();
        let ps = make_planning_with_goal("explore", GoalType::Exploration);
        ic.try_commit("agent_0", &ps, 0);
        assert!(ic.is_committed("agent_0", 5));
        assert!(!ic.is_committed("agent_0", 15));
    }

    #[test]
    fn streak_increments_on_same_goal() {
        let mut ic = IntentionCommitment::new();
        let ps = make_planning_with_goal("explore", GoalType::Exploration);
        ic.try_commit("agent_0", &ps, 0);
        ic.try_commit("agent_0", &ps, 15);
        ic.try_commit("agent_0", &ps, 30);
        assert_eq!(ic.streak("agent_0"), 3);
    }

    #[test]
    fn should_not_break_survival_goals() {
        let mut ic = IntentionCommitment::new();
        let ps = make_planning_with_goal("eat", GoalType::Survival);
        ic.try_commit("agent_0", &ps, 0);
        assert!(!ic.should_break_commitment("agent_0", 5.0, 5.0));
    }

    #[test]
    fn should_break_non_survival_in_critical() {
        let mut ic = IntentionCommitment::new();
        let ps = make_planning_with_goal("explore", GoalType::Exploration);
        ic.try_commit("agent_0", &ps, 0);
        assert!(ic.should_break_commitment("agent_0", 5.0, 5.0));
    }

    #[test]
    fn break_commitment_removes() {
        let mut ic = IntentionCommitment::new();
        let ps = make_planning_with_goal("explore", GoalType::Exploration);
        ic.try_commit("agent_0", &ps, 0);
        assert!(ic.break_commitment("agent_0").is_some());
        assert!(!ic.is_committed("agent_0", 5));
    }

    #[test]
    fn cleanup_removes_dead_agents() {
        let mut ic = IntentionCommitment::new();
        let ps = make_planning_with_goal("explore", GoalType::Exploration);
        ic.try_commit("agent_0", &ps, 0);
        ic.try_commit("agent_1", &ps, 0);
        ic.cleanup(&["agent_0".to_string()]);
        assert!(ic.is_committed("agent_0", 5));
        assert!(!ic.is_committed("agent_1", 5));
    }

    #[test]
    fn streak_capped_at_max() {
        let mut ic = IntentionCommitment::new();
        ic.max_streak = 5;
        let ps = make_planning_with_goal("explore", GoalType::Exploration);
        for i in 0..10 {
            ic.try_commit("agent_0", &ps, i * 15);
        }
        assert!(ic.streak("agent_0") <= 5);
    }
}

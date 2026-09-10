use serde::{Deserialize, Serialize};

use crate::agents::sim_agent::AgentAction;

/// A single goal with priority and subgoals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub name: String,
    pub priority: f32,
    pub goal_type: GoalType,
    pub status: GoalStatus,
    pub subgoals: Vec<Goal>,
    pub created_tick: u64,
    pub deadline_tick: Option<u64>,
    pub expected_actions: Vec<AgentAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalType {
    Survival,
    Social,
    Exploration,
    Achievement,
    Curiosity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalStatus {
    Active,
    Suspended,
    Completed,
    Failed,
}

/// The planning stack — manages goal hierarchy
pub struct PlanningStack {
    goals: Vec<Goal>,
    max_goals: usize,
    tick: u64,
}

impl PlanningStack {
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            max_goals: 8,
            tick: 0,
        }
    }

    pub fn add_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
        self.goals.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if self.goals.len() > self.max_goals {
            // Demote lowest-priority non-active goals
            for g in self.goals.iter_mut().skip(self.max_goals) {
                if g.status == GoalStatus::Active {
                    g.status = GoalStatus::Suspended;
                }
            }
        }
    }

    pub fn current_goal(&self) -> Option<&Goal> {
        self.goals.iter().find(|g| g.status == GoalStatus::Active)
    }

    pub fn next_action(&self) -> Option<&AgentAction> {
        self.current_goal()
            .and_then(|g| g.expected_actions.first())
    }

    pub fn advance_action(&mut self) {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.status == GoalStatus::Active) {
            if !goal.expected_actions.is_empty() {
                goal.expected_actions.remove(0);
            }
            if goal.expected_actions.is_empty() {
                goal.status = GoalStatus::Completed;
            }
        }
    }

    pub fn react(&mut self, observation: &str, priority_boost: f32) {
        if priority_boost > 0.7 {
            // Suspend current active goal
            if let Some(current) = self.goals.iter_mut().find(|g| g.status == GoalStatus::Active)
            {
                current.status = GoalStatus::Suspended;
            }
            // Add high-priority reactive goal
            self.add_goal(Goal {
                name: format!("react:{}", observation),
                priority: priority_boost,
                goal_type: GoalType::Survival,
                status: GoalStatus::Active,
                subgoals: vec![],
                created_tick: self.tick,
                deadline_tick: None,
                expected_actions: vec![AgentAction::Rest],
            });
        }
    }

    pub fn consolidate(&mut self) {
        // Promote subgoals of completed parents
        let completed: Vec<Vec<Goal>> = self
            .goals
            .iter()
            .filter(|g| g.status == GoalStatus::Completed)
            .map(|g| g.subgoals.clone())
            .collect();

        for subgoals in completed {
            for mut sg in subgoals {
                sg.status = GoalStatus::Active;
                self.goals.push(sg);
            }
        }

        // Remove Completed and Failed
        self.goals
            .retain(|g| g.status != GoalStatus::Completed && g.status != GoalStatus::Failed);

        // Re-sort
        self.goals.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn generate_survival_goals(&mut self, hunger: f32, energy: f32, health: f32, tick: u64) {
        self.tick = tick;

        if hunger > 60.0 && !self.has_active_goal_type(&GoalType::Survival, "eat") {
            self.add_goal(Goal {
                name: "eat".to_string(),
                priority: hunger / 100.0,
                goal_type: GoalType::Survival,
                status: GoalStatus::Active,
                subgoals: vec![],
                created_tick: tick,
                deadline_tick: None,
                expected_actions: vec![AgentAction::Explore {
                    direction: crate::foundation::math_bridge::Vec2::new(0.0, 0.0),
                }],
            });
        }

        if energy < 30.0 && !self.has_active_goal_type(&GoalType::Survival, "rest") {
            self.add_goal(Goal {
                name: "rest".to_string(),
                priority: (100.0 - energy) / 100.0,
                goal_type: GoalType::Survival,
                status: GoalStatus::Active,
                subgoals: vec![],
                created_tick: tick,
                deadline_tick: None,
                expected_actions: vec![AgentAction::Rest],
            });
        }

        if health < 40.0 && !self.has_active_goal_type(&GoalType::Survival, "flee") {
            self.add_goal(Goal {
                name: "flee".to_string(),
                priority: (100.0 - health) / 100.0,
                goal_type: GoalType::Survival,
                status: GoalStatus::Active,
                subgoals: vec![],
                created_tick: tick,
                deadline_tick: None,
                expected_actions: vec![AgentAction::Explore {
                    direction: crate::foundation::math_bridge::Vec2::new(1.0, 0.0),
                }],
            });
        }
    }

    pub fn generate_social_goals(&mut self, nearby_count: usize, has_relationships: bool, tick: u64) {
        self.tick = tick;

        if nearby_count > 0 && !has_relationships && !self.has_active_goal_type(&GoalType::Social, "")
        {
            self.add_goal(Goal {
                name: "socialize".to_string(),
                priority: 0.5,
                goal_type: GoalType::Social,
                status: GoalStatus::Active,
                subgoals: vec![],
                created_tick: tick,
                deadline_tick: None,
                expected_actions: vec![AgentAction::Talk {
                    target_id: String::new(),
                    message: "hello".to_string(),
                }],
            });
        }
    }

    pub fn generate_exploration_goals(&mut self, tick: u64) {
        self.tick = tick;

        if self.active_count() == 0 {
            self.add_goal(Goal {
                name: "explore".to_string(),
                priority: 0.3,
                goal_type: GoalType::Exploration,
                status: GoalStatus::Active,
                subgoals: vec![],
                created_tick: tick,
                deadline_tick: None,
                expected_actions: vec![AgentAction::Explore {
                    direction: crate::foundation::math_bridge::Vec2::new(1.0, 1.0),
                }],
            });
        }
    }

    pub fn goals(&self) -> &[Goal] {
        &self.goals
    }

    pub fn active_count(&self) -> usize {
        self.goals.iter().filter(|g| g.status == GoalStatus::Active).count()
    }

    fn has_active_goal_type(&self, goal_type: &GoalType, name_contains: &str) -> bool {
        self.goals.iter().any(|g| {
            g.status == GoalStatus::Active
                && std::mem::discriminant(&g.goal_type) == std::mem::discriminant(goal_type)
                && (name_contains.is_empty() || g.name.contains(name_contains))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_goal(name: &str, priority: f32) -> Goal {
        Goal {
            name: name.to_string(),
            priority,
            goal_type: GoalType::Achievement,
            status: GoalStatus::Active,
            subgoals: vec![],
            created_tick: 0,
            deadline_tick: None,
            expected_actions: vec![AgentAction::Think],
        }
    }

    #[test]
    fn test_empty_planning_stack() {
        let ps = PlanningStack::new();
        assert!(ps.current_goal().is_none());
        assert!(ps.next_action().is_none());
        assert_eq!(ps.active_count(), 0);
        assert!(ps.goals().is_empty());
    }

    #[test]
    fn test_goal_ordering_by_priority() {
        let mut ps = PlanningStack::new();
        ps.add_goal(make_goal("low", 0.2));
        ps.add_goal(make_goal("high", 0.9));
        ps.add_goal(make_goal("mid", 0.5));

        assert_eq!(ps.current_goal().unwrap().name, "high");
        assert_eq!(ps.goals()[0].name, "high");
        assert_eq!(ps.goals()[1].name, "mid");
        assert_eq!(ps.goals()[2].name, "low");
    }

    #[test]
    fn test_action_advancement() {
        let mut ps = PlanningStack::new();
        ps.add_goal(Goal {
            name: "multi".to_string(),
            priority: 1.0,
            goal_type: GoalType::Achievement,
            status: GoalStatus::Active,
            subgoals: vec![],
            created_tick: 0,
            deadline_tick: None,
            expected_actions: vec![AgentAction::Think, AgentAction::Rest],
        });

        assert!(matches!(ps.next_action(), Some(AgentAction::Think)));

        ps.advance_action();
        assert!(matches!(ps.next_action(), Some(AgentAction::Rest)));

        ps.advance_action();
        // Goal should be completed now
        assert!(ps.current_goal().is_none());
        assert_eq!(ps.goals().len(), 1);
        assert_eq!(ps.goals()[0].status, GoalStatus::Completed);
    }

    #[test]
    fn test_reactive_goal_suspends_current() {
        let mut ps = PlanningStack::new();
        ps.add_goal(make_goal("explore", 0.3));

        assert_eq!(ps.current_goal().unwrap().name, "explore");

        ps.react("danger", 0.9);

        // Current goal should be suspended
        assert!(ps.goals().iter().any(|g| g.name == "explore" && g.status == GoalStatus::Suspended));
        // Reactive goal should be active
        let reactive = ps.current_goal().unwrap();
        assert!(reactive.name.starts_with("react:"));
        assert_eq!(reactive.status, GoalStatus::Active);
    }

    #[test]
    fn test_low_priority_boost_does_not_suspend() {
        let mut ps = PlanningStack::new();
        ps.add_goal(make_goal("explore", 0.3));

        ps.react("minor", 0.5);

        // Original goal should still be active
        assert_eq!(ps.current_goal().unwrap().name, "explore");
    }

    #[test]
    fn test_survival_goal_generation_hungry() {
        let mut ps = PlanningStack::new();
        ps.generate_survival_goals(70.0, 80.0, 90.0, 1);

        assert_eq!(ps.active_count(), 1);
        assert_eq!(ps.current_goal().unwrap().name, "eat");
        assert!(matches!(ps.current_goal().unwrap().goal_type, GoalType::Survival));
    }

    #[test]
    fn test_survival_goal_generation_low_energy() {
        let mut ps = PlanningStack::new();
        ps.generate_survival_goals(10.0, 20.0, 90.0, 1);

        assert_eq!(ps.active_count(), 1);
        assert_eq!(ps.current_goal().unwrap().name, "rest");
    }

    #[test]
    fn test_survival_goal_generation_low_health() {
        let mut ps = PlanningStack::new();
        ps.generate_survival_goals(10.0, 80.0, 30.0, 1);

        assert_eq!(ps.active_count(), 1);
        assert_eq!(ps.current_goal().unwrap().name, "flee");
    }

    #[test]
    fn test_survival_goal_generation_no_duplicate() {
        let mut ps = PlanningStack::new();
        ps.generate_survival_goals(70.0, 80.0, 90.0, 1);
        ps.generate_survival_goals(70.0, 80.0, 90.0, 2);

        // Should not add a second "eat" goal
        let eat_goals: Vec<_> = ps.goals().iter().filter(|g| g.name == "eat").collect();
        assert_eq!(eat_goals.len(), 1);
    }

    #[test]
    fn test_social_goal_generation() {
        let mut ps = PlanningStack::new();
        ps.generate_social_goals(3, false, 1);

        assert_eq!(ps.active_count(), 1);
        assert_eq!(ps.current_goal().unwrap().name, "socialize");
        assert!(matches!(ps.current_goal().unwrap().goal_type, GoalType::Social));
    }

    #[test]
    fn test_social_goal_no_trigger_when_no_nearby() {
        let mut ps = PlanningStack::new();
        ps.generate_social_goals(0, false, 1);
        assert_eq!(ps.active_count(), 0);
    }

    #[test]
    fn test_social_goal_no_trigger_when_has_relationships() {
        let mut ps = PlanningStack::new();
        ps.generate_social_goals(3, true, 1);
        assert_eq!(ps.active_count(), 0);
    }

    #[test]
    fn test_exploration_when_idle() {
        let mut ps = PlanningStack::new();
        ps.generate_exploration_goals(1);

        assert_eq!(ps.active_count(), 1);
        assert_eq!(ps.current_goal().unwrap().name, "explore");
        assert!(matches!(ps.current_goal().unwrap().goal_type, GoalType::Exploration));
    }

    #[test]
    fn test_exploration_not_when_busy() {
        let mut ps = PlanningStack::new();
        ps.add_goal(make_goal("busy", 0.5));
        ps.generate_exploration_goals(1);

        // Should not add explore since there's an active goal
        let explore_goals: Vec<_> = ps.goals().iter().filter(|g| g.name == "explore").collect();
        assert!(explore_goals.is_empty());
    }

    #[test]
    fn test_consolidation_removes_completed() {
        let mut ps = PlanningStack::new();
        ps.add_goal(make_goal("done", 0.9));
        ps.add_goal(make_goal("active", 0.5));

        // Complete the first goal
        ps.goals[0].status = GoalStatus::Completed;

        ps.consolidate();

        assert_eq!(ps.goals().len(), 1);
        assert_eq!(ps.goals()[0].name, "active");
    }

    #[test]
    fn test_consolidation_promotes_subgoals() {
        let mut ps = PlanningStack::new();
        ps.add_goal(Goal {
            name: "parent".to_string(),
            priority: 0.9,
            goal_type: GoalType::Achievement,
            status: GoalStatus::Completed,
            subgoals: vec![make_goal("child1", 0.8), make_goal("child2", 0.6)],
            created_tick: 0,
            deadline_tick: None,
            expected_actions: vec![],
        });

        ps.consolidate();

        // Parent removed, subgoals promoted
        assert_eq!(ps.goals().len(), 2);
        assert!(ps.goals().iter().any(|g| g.name == "child1"));
        assert!(ps.goals().iter().any(|g| g.name == "child2"));
    }

    #[test]
    fn test_consolidation_removes_failed() {
        let mut ps = PlanningStack::new();
        ps.add_goal(make_goal("bad", 0.9));
        ps.goals[0].status = GoalStatus::Failed;
        ps.add_goal(make_goal("good", 0.5));

        ps.consolidate();

        assert_eq!(ps.goals().len(), 1);
        assert_eq!(ps.goals()[0].name, "good");
    }

    #[test]
    fn test_multiple_goals_priority_ordering() {
        let mut ps = PlanningStack::new();
        for i in 0..10 {
            ps.add_goal(make_goal(&format!("g{i}"), i as f32 / 10.0));
        }

        // Highest priority first
        assert_eq!(ps.current_goal().unwrap().name, "g9");
        assert_eq!(ps.goals().last().unwrap().name, "g0");
    }
}

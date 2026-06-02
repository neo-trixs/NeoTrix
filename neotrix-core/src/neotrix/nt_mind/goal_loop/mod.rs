pub mod types;
pub mod tracker;
pub mod priority;
pub mod fast_goal;
pub mod loop_impl;

pub use types::{RateLimiter, CircuitState, CircuitBreaker, GoalState, GoalConfig, GoalIterationRecord, GoalPriority, GoalScheduleStrategy, PlanLevel, PlanTemplate};
pub use tracker::GoalTracker;
pub use priority::{PriorityEngine, RICEScore, ICEScore, MoscowClass, PriorityFramework, PriorityDecision};
pub use fast_goal::FastGoal;
pub use loop_impl::GoalLoop;
// pub(crate) use loop_impl::truncate;

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::self_iterating::SelfIteratingBrain;

    #[test]
    fn test_goal_lifecycle() {
        let mut gl = GoalLoop::new();
        let mut brain = SelfIteratingBrain::new();

        gl.start_goal(&mut brain, "Test autonomous goal", None);
        assert_eq!(gl.active_goal.as_ref().expect("active_goal after start_goal").state, GoalState::Pursuing);

        gl.pause_goal();
        assert_eq!(gl.active_goal.as_ref().expect("active_goal after pause_goal").state, GoalState::Paused);

        gl.resume_goal();
        assert_eq!(gl.active_goal.as_ref().expect("active_goal after resume_goal").state, GoalState::Pursuing);

        gl.clear_goal();
        assert!(gl.active_goal.is_none());
        assert_eq!(gl.completed_goals.len(), 1);
    }

    #[test]
    fn test_goal_budget_exhaustion() {
        let mut config = GoalConfig::default();
        config.max_iterations = 1;

        let mut tracker = GoalTracker::new("test".into(), "test".into(), config);
        tracker.iterations_completed = 1;
        assert_eq!(tracker.budget_exhausted(), Some(GoalState::BudgetLimited));
    }

    #[test]
    fn test_goal_persistence() {
        let tmp = std::env::temp_dir().join("neotrix_test_goals.json");
        let _ = std::fs::remove_file(&tmp);

        let mut gl = GoalLoop::with_path(tmp.clone());
        let mut brain = SelfIteratingBrain::new();
        gl.start_goal(&mut brain, "persist test", None);
        gl.save().expect("save goal state to temp file");

        let mut gl2 = GoalLoop::with_path(tmp.clone());
        gl2.load();
        assert!(gl2.active_goal.is_some());
        assert_eq!(gl2.active_goal.expect("active_goal must be Some after load").description, "persist test");

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_pursue_iteration() {
        let mut gl = GoalLoop::new();
        let mut brain = SelfIteratingBrain::new();

        gl.start_goal(&mut brain, "test", None);
        let _continued = gl.pursue_iteration(&mut brain);
        assert!(gl.active_goal.as_ref().expect("active_goal after pursue_iteration").iterations_completed > 0);
    }

    #[test]
    fn test_status_no_goal() {
        let gl = GoalLoop::new();
        assert!(gl.status().contains("No active goal"));
    }

    #[test]
    fn test_state_labels() {
        assert_eq!(GoalState::Pursuing.label(), "pursuing");
        assert_eq!(GoalState::Achieved.label(), "achieved");
        assert_eq!(GoalState::Paused.label(), "paused");
        assert!(GoalState::Achieved.is_terminal());
        assert!(!GoalState::Pursuing.is_terminal());
    }

    #[test]
    fn test_continuation_prompt_generation() {
        let tracker = GoalTracker::new("t1".into(), "add unit tests".into(), GoalConfig::default());
        let prompt = tracker.continuation_prompt();
        assert!(prompt.contains("add unit tests"));
        assert!(prompt.contains("GOAL_COMPLETE"));
    }

    #[test]
    fn test_budget_prompt_generation() {
        let mut tracker = GoalTracker::new("t2".into(), "refactor module".into(), GoalConfig::default());
        tracker.iterations_completed = 5;
        tracker.total_cost_estimate = 0.25;
        tracker.tokens_consumed = 50000;
        let prompt = tracker.budget_prompt();
        assert!(prompt.contains("5/50"));
        assert!(prompt.contains("$0.2500"));
    }

    #[test]
    fn test_rate_limiter_allows_within_budget() {
        let mut rl = RateLimiter::new(5);
        for _ in 0..5 {
            assert!(rl.allow_call());
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let mut rl = RateLimiter::new(3);
        for _ in 0..3 {
            assert!(rl.allow_call());
        }
        assert!(!rl.allow_call());
    }

    #[test]
    fn test_circuit_breaker_trips_on_stall() {
        let mut cb = CircuitBreaker::new(5, 3, 3600);
        assert!(!cb.is_open());
        cb.record_stall();
        assert!(!cb.is_open());
        cb.record_stall();
        assert!(!cb.is_open());
        cb.record_stall();
        assert!(cb.is_open());
    }

    #[test]
    fn test_circuit_breaker_recovers() {
        let mut cb = CircuitBreaker::new(5, 3, 0);
        cb.record_stall();
        cb.record_stall();
        cb.record_stall();
        assert!(cb.is_open());
        cb.record_success();
        assert!(!cb.is_open());
    }

    #[test]
    fn test_integration_with_pursue_iteration() {
        let mut gl = GoalLoop::new();
        let mut brain = SelfIteratingBrain::new();

        gl.start_goal(&mut brain, "test rate limit", None);
        let _first = gl.pursue_iteration(&mut brain);
        assert!(gl.active_goal.as_ref().expect("active_goal after pursue_iteration in integration test").iterations_completed > 0);
        assert!(!gl.circuit_breaker.is_open());
        assert_eq!(gl.rate_limiter.call_timestamps.len(), 1);
    }

    #[test]
    fn test_complex_goal_detection() {
        let gl = GoalLoop::new();
        assert!(gl._is_complex_goal("设计一个响应式 UI 界面"));
        assert!(gl._is_complex_goal("analyze the performance"));
        assert!(gl._is_complex_goal("compare two approaches"));
        assert!(!gl._is_complex_goal("fix typo in readme"));
        assert!(!gl._is_complex_goal("update version number"));
    }

    #[test]
    fn test_complex_execution_fallsback_without_orchestrator() {
        let mut gl = GoalLoop::new();
        let mut brain = SelfIteratingBrain::new();
        gl.start_goal(&mut brain, "complex analysis task", None);
        let result = gl.pursue_iteration(&mut brain);
        assert!(result);
        assert!(gl.active_goal.as_ref().expect("active_goal after pursue_iteration in complex task").iterations_completed > 0);
    }

    #[test]
    fn test_with_orchestrator_does_not_panic() {
        let mut gl = GoalLoop::new();
        let mut brain = SelfIteratingBrain::new();
        let orch = crate::neotrix::orchestrator::Orchestrator::new();
        gl = gl.with_orchestrator(orch);
        gl.start_goal(&mut brain, "design a login page", None);
        let _ = gl.pursue_iteration(&mut brain);
    }

    #[test]
    fn test_with_agent_team_does_not_panic() {
        let mut gl = GoalLoop::new();
        let mut brain = SelfIteratingBrain::new();
        let team = crate::agent::AgentTeam::new("test", crate::agent::ProcessType::Sequential);
        gl = gl.with_agent_team(std::sync::Arc::new(std::sync::Mutex::new(team)));
        gl.start_goal(&mut brain, "research multiple topics", None);
        let _ = gl.pursue_iteration(&mut brain);
    }

    #[test]
    fn test_enqueue_dequeue() {
        let mut gl = GoalLoop::new();
        let mut brain = SelfIteratingBrain::new();

        gl.enqueue_goal(&mut brain, "low priority goal", None);
        gl.goal_queue.iter_mut().find(|g| g.description == "low priority goal").expect("low priority goal in queue").priority = GoalPriority::Low;

        gl.enqueue_goal(&mut brain, "high priority goal", None);
        gl.goal_queue.iter_mut().find(|g| g.description == "high priority goal").expect("high priority goal in queue").priority = GoalPriority::High;

        let dequeued = gl.dequeue_next().expect("dequeue_next with enqueued goals");
        assert_eq!(dequeued.description, "high priority goal");
        assert_eq!(gl.goal_queue.len(), 1);
    }

    #[test]
    fn test_rebalance_from_motivation_explore() {
        let mut gl = GoalLoop::new();
        let mut brain = SelfIteratingBrain::new();

        gl.enqueue_goal(&mut brain, "explore new knowledge", None);
        gl.enqueue_goal(&mut brain, "fix bug in parser", None);
        gl.goal_queue.iter_mut().for_each(|g| g.priority = GoalPriority::Medium);

        gl.set_motivation(crate::core::nt_core_self::MotivationState {
            intrinsic_reward: 0.8,
            confidence: 0.9,
            error_rate: 0.1,
            novelty_score: 0.9,
            should_explore: true,
            suggested_domains: vec![],
            suggested_strategies: vec![],
        });
        gl.rebalance_from_motivation();

        let explore_goal = gl.goal_queue.iter().find(|g| g.description.contains("explore"));
        assert!(explore_goal.is_some());
        assert_eq!(explore_goal.expect("explore_goal after rebalance").priority, GoalPriority::High);
    }

    #[test]
    fn test_rebalance_from_motivation_debug() {
        let mut gl = GoalLoop::new();
        let mut brain = SelfIteratingBrain::new();

        gl.enqueue_goal(&mut brain, "investigate reflection failure", None);
        gl.enqueue_goal(&mut brain, "add unit tests", None);
        gl.goal_queue.iter_mut().for_each(|g| g.priority = GoalPriority::Medium);

        gl.set_motivation(crate::core::nt_core_self::MotivationState {
            intrinsic_reward: 0.2,
            confidence: 0.3,
            error_rate: 0.5,
            novelty_score: 0.1,
            should_explore: false,
            suggested_domains: vec![],
            suggested_strategies: vec![],
        });
        gl.rebalance_from_motivation();

        let debug_goal = gl.goal_queue.iter().find(|g| g.description.contains("investigate"));
        assert!(debug_goal.is_some());
        assert_eq!(debug_goal.expect("debug_goal after rebalance").priority, GoalPriority::Critical);
    }

    #[test]
    fn test_goal_queue_deduplication() {
        let mut gl = GoalLoop::new();
        let mut brain = SelfIteratingBrain::new();

        gl.enqueue_goal(&mut brain, "duplicate goal", None);
        gl.enqueue_goal(&mut brain, "duplicate goal", None);
        assert_eq!(gl.goal_queue.len(), 1);

        gl.start_goal(&mut brain, "active goal", None);
        gl.enqueue_goal(&mut brain, "active goal", None);
        assert_eq!(gl.goal_queue.len(), 1);
    }

    #[test]
    fn test_multi_goal_auto_generate() {
        let brain = SelfIteratingBrain::new();
        let candidates = GoalLoop::auto_goal_candidates(&brain, 3);
        assert_eq!(candidates.len(), 3);
        assert!(!candidates[0].is_empty());
        assert!(!candidates[1].is_empty());
        assert!(!candidates[2].is_empty());
        // Verify diversity: at least two different descriptions
        let unique: std::collections::HashSet<&str> = candidates.iter().map(|s| s.as_str()).collect();
        assert!(unique.len() >= 2, "candidates should be diverse: {:?}", candidates);
    }

    // === Hierarchical Plan Tests ===

    #[test]
    fn test_create_macro_plan() {
        let gl = GoalLoop::new();
        let brain = SelfIteratingBrain::new();
        let plan = gl.create_macro_plan(&brain);
        assert_eq!(plan.level, PlanLevel::Macro);
        assert_eq!(plan.sub_plans.len(), 2);
        for meso in &plan.sub_plans {
            assert_eq!(meso.level, PlanLevel::Meso);
            assert!(meso.sub_plans.len() >= 2);
            for micro in &meso.sub_plans {
                assert_eq!(micro.level, PlanLevel::Micro);
            }
        }
    }

    #[test]
    fn test_drill_down() {
        let mut gl = GoalLoop::new();
        let brain = SelfIteratingBrain::new();
        gl.active_plan = Some(gl.create_macro_plan(&brain));

        let meso_level = gl.drill_down();
        assert!(meso_level.is_some());
        assert_eq!(meso_level.expect("meso_level after drill_down").level, PlanLevel::Meso);
        assert_eq!(gl.plan_stack.len(), 1);
        assert_eq!(gl.plan_stack[0].level, PlanLevel::Macro);
    }

    #[test]
    fn test_skip_condition_met() {
        let mut gl = GoalLoop::new();
        let brain = SelfIteratingBrain::new();
        gl.active_plan = Some(PlanTemplate {
            level: PlanLevel::Micro,
            name: "test".into(),
            description: "test skip".into(),
            sub_plans: vec![],
            skip_condition: Some("memory < 99999".into()),
            reflection_trigger: None,
            expected_duration_cycles: 1,
            completion_criteria: None,
        });
        // memory count is 0 (new brain) which is < 99999 → should skip
        assert!(gl.check_skip_condition(&brain));
    }

    #[test]
    fn test_check_skip_condition_not_met() {
        let mut gl = GoalLoop::new();
        let brain = SelfIteratingBrain::new();
        gl.active_plan = Some(PlanTemplate {
            level: PlanLevel::Micro,
            name: "test".into(),
            description: "no skip".into(),
            sub_plans: vec![],
            skip_condition: Some("capability > 100".into()),
            reflection_trigger: None,
            expected_duration_cycles: 1,
            completion_criteria: None,
        });
        // cap sum is roughly 0 (new brain), well below 100 → should NOT skip
        assert!(!gl.check_skip_condition(&brain));
    }

    #[test]
    fn test_reflection_trigger() {
        let mut gl = GoalLoop::new();
        gl.active_plan = Some(PlanTemplate {
            level: PlanLevel::Micro,
            name: "test".into(),
            description: "reflection test".into(),
            sub_plans: vec![],
            skip_condition: None,
            reflection_trigger: Some("after 5 iterations".into()),
            expected_duration_cycles: 1,
            completion_criteria: None,
        });
        assert!(gl.check_reflection_trigger(5));
        assert!(gl.check_reflection_trigger(10));
        assert!(!gl.check_reflection_trigger(3));
        assert!(!gl.check_reflection_trigger(0));
    }

    #[test]
    fn test_plan_summary_format() {
        let mut gl = GoalLoop::new();
        let brain = SelfIteratingBrain::new();
        gl.active_plan = Some(gl.create_macro_plan(&brain));
        let summary = gl.plan_summary();
        assert!(summary.contains("macro"), "summary should contain 'macro'");
        assert!(summary.contains("meso"), "summary should contain 'meso'");
        assert!(summary.contains("micro"), "summary should contain 'micro'");
        assert!(summary.contains("cycles est."), "summary should show cycles estimate");
    }

    #[test]
    fn test_auto_plan_creates_plan() {
        let mut gl = GoalLoop::new();
        let brain = SelfIteratingBrain::new();
        assert!(gl.active_plan.is_none());
        gl.auto_plan(&brain);
        assert!(gl.active_plan.is_some());
        assert_eq!(gl.active_plan.as_ref().expect("active_plan after auto_plan").level, PlanLevel::Macro);
    }
}

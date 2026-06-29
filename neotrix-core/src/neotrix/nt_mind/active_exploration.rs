use crate::neotrix::nt_mind::curiosity_drive::CuriositySignal;
use crate::neotrix::nt_mind::exploration_pipeline::ExploreDomain;
use std::time::{Duration, Instant};

const MAX_PLANS: usize = 50;
const MAX_STEPS_PER_PLAN: usize = 20;
const STALE_DURATION: Duration = Duration::from_secs(3600);
const SUCCESS_THRESHOLD: f64 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExploreStrategy {
    BroadFirst,
    DeepFirst,
    Random,
    GapDriven,
}

impl ExploreStrategy {
    pub fn name(&self) -> &'static str {
        match self {
            ExploreStrategy::BroadFirst => "Breadth-First",
            ExploreStrategy::DeepFirst => "Depth-First",
            ExploreStrategy::Random => "Random",
            ExploreStrategy::GapDriven => "Gap-Driven",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanStatus {
    Active,
    Completed,
    Stale,
    Abandoned,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub summary: String,
    pub gap_reduction: f64,
    pub new_questions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExplorationStep {
    pub step_id: u64,
    pub query: String,
    pub domain: ExploreDomain,
    pub status: StepStatus,
    pub result: Option<StepResult>,
}

#[derive(Debug, Clone)]
pub struct ExplorationPlan {
    pub id: u64,
    pub goal: String,
    pub strategy: ExploreStrategy,
    pub steps: Vec<ExplorationStep>,
    pub max_steps: usize,
    pub created_at: Instant,
    pub status: PlanStatus,
}

pub struct ActiveExploration {
    pub plans: Vec<ExplorationPlan>,
    pub max_plans: usize,
    next_id: u64,
    next_step_id: u64,
    pub total_explorations: u64,
    pub successful_explorations: u64,
}

impl ActiveExploration {
    pub fn new() -> Self {
        Self {
            plans: Vec::with_capacity(MAX_PLANS),
            max_plans: MAX_PLANS,
            next_id: 1,
            next_step_id: 1,
            total_explorations: 0,
            successful_explorations: 0,
        }
    }

    pub fn create_plan(
        &mut self,
        goal: &str,
        strategy: ExploreStrategy,
        queries: Vec<String>,
    ) -> u64 {
        let plan_id = self.next_id;
        self.next_id += 1;

        let domain = match &strategy {
            ExploreStrategy::BroadFirst => ExploreDomain::General,
            ExploreStrategy::DeepFirst => ExploreDomain::Papers,
            ExploreStrategy::Random => ExploreDomain::General,
            ExploreStrategy::GapDriven => ExploreDomain::General,
        };

        let steps: Vec<ExplorationStep> = queries
            .into_iter()
            .take(MAX_STEPS_PER_PLAN)
            .map(|query| {
                let step_id = self.next_step_id;
                self.next_step_id += 1;
                ExplorationStep {
                    step_id,
                    query,
                    domain,
                    status: StepStatus::Pending,
                    result: None,
                }
            })
            .collect();

        let plan = ExplorationPlan {
            id: plan_id,
            goal: goal.to_string(),
            strategy,
            steps,
            max_steps: 10,
            created_at: Instant::now(),
            status: PlanStatus::Active,
        };

        self.plans.push(plan);
        self.total_explorations += 1;

        if self.plans.len() > self.max_plans {
            self.plans.remove(0);
        }

        plan_id
    }

    pub fn create_plan_from_curiosity(&mut self, signal: &CuriositySignal) -> u64 {
        let queries = signal.suggested_search_terms.clone();
        let goal = signal.description.clone();
        self.create_plan(&goal, ExploreStrategy::GapDriven, queries)
    }

    pub fn step_completed(&mut self, plan_id: u64, step_id: u64, result: StepResult) {
        if let Some(plan) = self.plans.iter_mut().find(|p| p.id == plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.step_id == step_id) {
                step.status = StepStatus::Completed;
                step.result = Some(result);
            }

            let all_done = plan
                .steps
                .iter()
                .all(|s| matches!(s.status, StepStatus::Completed | StepStatus::Failed));
            if all_done {
                let avg_reduction = plan
                    .steps
                    .iter()
                    .filter_map(|s| s.result.as_ref().map(|r| r.gap_reduction))
                    .sum::<f64>()
                    / plan.steps.len().max(1) as f64;
                plan.status = PlanStatus::Completed;
                if avg_reduction > SUCCESS_THRESHOLD {
                    self.successful_explorations += 1;
                }
            }
        }
    }

    pub fn step_failed(&mut self, plan_id: u64, step_id: u64, _reason: &str) {
        if let Some(plan) = self.plans.iter_mut().find(|p| p.id == plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.step_id == step_id) {
                step.status = StepStatus::Failed;
            }
        }
    }

    pub fn active_plans(&self) -> Vec<&ExplorationPlan> {
        self.plans
            .iter()
            .filter(|p| p.status == PlanStatus::Active)
            .collect()
    }

    pub fn best_strategy(&self) -> ExploreStrategy {
        if self.total_explorations == 0 {
            return ExploreStrategy::BroadFirst;
        }
        let success_rate = self.successful_explorations as f64 / self.total_explorations as f64;
        if success_rate > 0.6 {
            ExploreStrategy::DeepFirst
        } else {
            ExploreStrategy::BroadFirst
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_explorations == 0 {
            0.0
        } else {
            self.successful_explorations as f64 / self.total_explorations as f64
        }
    }

    pub fn explore_summary(&self) -> String {
        let active_count = self.active_plans().len();
        let total_steps: usize = self.plans.iter().map(|p| p.steps.len()).sum();
        let completed_steps: usize = self
            .plans
            .iter()
            .flat_map(|p| &p.steps)
            .filter(|s| s.status == StepStatus::Completed)
            .count();
        let rate = self.success_rate() * 100.0;
        format!(
            "ActiveExploration: {} plans active | {} steps total / {} completed | {:.1}% success rate ({} successful / {} total)",
            active_count, total_steps, completed_steps, rate, self.successful_explorations, self.total_explorations,
        )
    }

    pub fn cleanup_stale_plans(&mut self) {
        let now = Instant::now();
        for plan in &mut self.plans {
            if plan.status == PlanStatus::Active {
                let elapsed = now.duration_since(plan.created_at);
                if elapsed > STALE_DURATION {
                    plan.status = PlanStatus::Stale;
                }
            }
        }
    }
}

impl Default for ActiveExploration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::gap::GapReport;
    use crate::neotrix::nt_mind::curiosity_drive::CuriosityLevel;

    #[test]
    fn test_create_plan() {
        let mut planner = ActiveExploration::new();
        let queries = vec![
            "rust async".to_string(),
            "tokio internals".to_string(),
            "async trait".to_string(),
        ];
        let plan_id = planner.create_plan("learn async rust", ExploreStrategy::BroadFirst, queries);

        let plan = planner.plans.iter().find(|p| p.id == plan_id).unwrap();
        assert_eq!(plan.steps.len(), 3);
        assert_eq!(plan.strategy, ExploreStrategy::BroadFirst);
        assert_eq!(plan.status, PlanStatus::Active);
        for step in &plan.steps {
            assert_eq!(step.status, StepStatus::Pending);
            assert!(step.result.is_none());
        }
    }

    #[test]
    fn test_step_completed_advances_plan() {
        let mut planner = ActiveExploration::new();
        let plan_id = planner.create_plan(
            "deep dive",
            ExploreStrategy::DeepFirst,
            vec!["query1".to_string(), "query2".to_string()],
        );

        planner.step_completed(
            plan_id,
            1,
            StepResult {
                summary: "found relevant paper".into(),
                gap_reduction: 0.5,
                new_questions: vec![],
            },
        );
        planner.step_completed(
            plan_id,
            2,
            StepResult {
                summary: "confirmed hypothesis".into(),
                gap_reduction: 0.7,
                new_questions: vec!["follow-up q".to_string()],
            },
        );

        let plan = planner.plans.iter().find(|p| p.id == plan_id).unwrap();
        assert_eq!(plan.status, PlanStatus::Completed);
        assert_eq!(
            plan.steps
                .iter()
                .filter(|s| s.status == StepStatus::Completed)
                .count(),
            2
        );
    }

    #[test]
    fn test_success_rate_tracking() {
        let mut planner = ActiveExploration::new();

        let p1 = planner.create_plan("p1", ExploreStrategy::BroadFirst, vec!["q1".to_string()]);
        planner.step_completed(
            p1,
            1,
            StepResult {
                summary: "good find".into(),
                gap_reduction: 0.8,
                new_questions: vec![],
            },
        );

        let p2 = planner.create_plan("p2", ExploreStrategy::BroadFirst, vec!["q1".to_string()]);
        planner.step_completed(
            p2,
            3,
            StepResult {
                summary: "poor result".into(),
                gap_reduction: 0.1,
                new_questions: vec![],
            },
        );

        assert_eq!(planner.total_explorations, 2);
        assert_eq!(planner.successful_explorations, 1);
        assert!((planner.success_rate() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_best_strategy_adaptive() {
        let mut planner = ActiveExploration::new();

        // No history → BroadFirst
        assert_eq!(planner.best_strategy(), ExploreStrategy::BroadFirst);

        // Low success rate (3/10 = 0.3) → BroadFirst
        planner.total_explorations = 10;
        planner.successful_explorations = 3;
        assert_eq!(planner.best_strategy(), ExploreStrategy::BroadFirst);

        // High success rate (7/10 = 0.7) → DeepFirst
        planner.successful_explorations = 7;
        assert_eq!(planner.best_strategy(), ExploreStrategy::DeepFirst);
    }

    #[test]
    fn test_stale_plan_cleanup() {
        let mut planner = ActiveExploration::new();
        planner.create_plan("fresh", ExploreStrategy::BroadFirst, vec!["q1".to_string()]);
        assert_eq!(planner.active_plans().len(), 1);

        // Freshly created plans should not be marked stale
        planner.cleanup_stale_plans();
        assert_eq!(planner.active_plans().len(), 1);

        // Manually mark as stale, then call cleanup again (should be no-op on status change)
        if let Some(plan) = planner.plans.first_mut() {
            plan.status = PlanStatus::Stale;
        }
        assert_eq!(planner.active_plans().len(), 0);
    }

    #[test]
    fn test_from_curiosity_signal() {
        let mut planner = ActiveExploration::new();
        let signal = CuriositySignal {
            domain: ExploreDomain::Papers,
            intensity: 0.8,
            description: "gap in tokio internals".to_string(),
            gap_report: None,
            suggested_search_terms: vec![
                "tokio internals".to_string(),
                "async rust scheduler".to_string(),
            ],
            vsa_signature: None,
        };

        let plan_id = planner.create_plan_from_curiosity(&signal);
        let plan = planner.plans.iter().find(|p| p.id == plan_id).unwrap();
        assert_eq!(plan.strategy, ExploreStrategy::GapDriven);
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].query, "tokio internals");
        assert_eq!(plan.steps[1].query, "async rust scheduler");
    }

    #[test]
    fn test_step_failed_does_not_complete() {
        let mut planner = ActiveExploration::new();
        let plan_id = planner.create_plan(
            "risky",
            ExploreStrategy::Random,
            vec!["q1".to_string(), "q2".to_string()],
        );

        planner.step_failed(plan_id, 1, "network error");
        planner.step_completed(
            plan_id,
            2,
            StepResult {
                summary: "ok".into(),
                gap_reduction: 0.9,
                new_questions: vec![],
            },
        );

        let plan = planner.plans.iter().find(|p| p.id == plan_id).unwrap();
        assert_eq!(plan.status, PlanStatus::Completed);
        assert_eq!(plan.steps[0].status, StepStatus::Failed);
        assert_eq!(plan.steps[1].status, StepStatus::Completed);
    }

    #[test]
    fn test_explore_summary_format() {
        let mut planner = ActiveExploration::new();
        let summary_empty = planner.explore_summary();
        assert!(summary_empty.contains("0 plans active"));

        planner.create_plan("test", ExploreStrategy::BroadFirst, vec!["q1".to_string()]);
        let summary_active = planner.explore_summary();
        assert!(summary_active.contains("1 plans active"));
        assert!(summary_active.contains("1 steps total"));
        assert!(summary_active.contains("0 completed"));
    }

    #[test]
    fn test_max_plans_enforced() {
        let mut planner = ActiveExploration::new();
        planner.max_plans = 3;

        for i in 0..5 {
            let queries = vec![format!("q{}", i)];
            planner.create_plan(&format!("plan {}", i), ExploreStrategy::BroadFirst, queries);
        }

        assert!(planner.plans.len() <= 3);
    }
}

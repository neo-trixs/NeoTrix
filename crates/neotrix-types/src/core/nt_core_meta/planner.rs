use std::collections::HashMap;
use super::self_model::{DebtSeverity, EvolutionEvent, EventKind, SelfModel};
use super::weakness::{Weakness, WeaknessReport};

/// Evolution planner that prioritizes weaknesses and constructs
/// an actionable evolution plan. Uses impact analysis to schedule
/// changes in dependency-aware order.
#[derive(Debug, Clone)]
pub struct EvolutionPlanner {
    pub queue: Vec<PlannedEvolution>,
    pub history: Vec<EvolutionAction>,
    pub max_concurrent: usize,
}

impl Default for EvolutionPlanner {
    fn default() -> Self {
        Self {
            queue: Vec::new(),
            history: Vec::new(),
            max_concurrent: 3,
        }
    }
}

impl EvolutionPlanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn plan_from_report(&mut self, report: &WeaknessReport) -> Vec<PlannedEvolution> {
        let mut plans: Vec<PlannedEvolution> = report.weaknesses.iter()
            .map(|w| self.weakness_to_plan(w))
            .collect();

        plans.sort_by_key(|p| {
            let severity_score = match p.weakness.severity {
                DebtSeverity::Critical => 0,
                DebtSeverity::Major => 1,
                DebtSeverity::Minor => 2,
                DebtSeverity::Cosmetic => 3,
            };
            (severity_score, p.estimated_impact.files_affected)
        });

        for (i, plan) in plans.iter_mut().enumerate() {
            plan.id = format!("EVO-{}", i + 1);
            plan.priority = (i / self.max_concurrent) as u8 + 1;
        }

        self.queue = plans.clone();
        plans
    }

    pub fn plan_from_weaknesses(&mut self, weaknesses: Vec<Weakness>) -> Vec<PlannedEvolution> {
        let mut plans: Vec<PlannedEvolution> = weaknesses.into_iter()
            .map(|w| self.weakness_to_plan(&w))
            .collect();

        plans.sort_by_key(|p| {
            let severity_score = match p.weakness.severity {
                DebtSeverity::Critical => 0,
                DebtSeverity::Major => 1,
                DebtSeverity::Minor => 2,
                DebtSeverity::Cosmetic => 3,
            };
            (severity_score, p.estimated_impact.risk.clone() as u8)
        });

        self.queue = plans.clone();
        plans
    }

    fn weakness_to_plan(&self, weakness: &Weakness) -> PlannedEvolution {
        let (files_affected, risk) = self.estimate_impact(weakness);
        PlannedEvolution {
            id: String::new(),
            priority: 1,
            weakness: weakness.clone(),
            target_module: weakness.target_module.clone(),
            action: weakness.suggestion.clone(),
            estimated_impact: ImpactEstimate { files_affected, risk },
            dependencies: Vec::new(),
        }
    }

    fn estimate_impact(&self, weakness: &Weakness) -> (usize, RiskLevel) {
        match weakness.pattern_id.as_str() {
            "CIRCULAR_DEP" => (5, RiskLevel::High),
            "EXCESS_UNSAFE" => (3, RiskLevel::High),
            "EXCESS_UNWRAP" => (10, RiskLevel::Medium),
            "MISSING_TESTS" => (2, RiskLevel::Low),
            "ORPHAN_MODULE" => (1, RiskLevel::Low),
            "LARGE_FILE" => (1, RiskLevel::Low),
            _ => (1, RiskLevel::Low),
        }
    }

    pub fn next_batch(&self) -> Vec<&PlannedEvolution> {
        let first_priority = self.queue.first().map(|p| p.priority).unwrap_or(1);
        self.queue.iter()
            .filter(|p| p.priority == first_priority)
            .take(self.max_concurrent)
            .collect()
    }

    pub fn plans_by_module(&self) -> HashMap<String, Vec<&PlannedEvolution>> {
        let mut map: HashMap<String, Vec<&PlannedEvolution>> = HashMap::new();
        for plan in &self.queue {
            let key = plan.target_module.clone().unwrap_or_else(|| "unknown".to_string());
            map.entry(key).or_default().push(plan);
        }
        map
    }

    pub fn weakest_modules(&self) -> Vec<(String, usize)> {
        let by_module = self.plans_by_module();
        let mut ranked: Vec<(String, usize)> = by_module.into_iter()
            .map(|(name, plans)| {
                let weighted = plans.iter().map(|p| match p.weakness.severity {
                    DebtSeverity::Critical => 5,
                    DebtSeverity::Major => 3,
                    DebtSeverity::Minor => 1,
                    DebtSeverity::Cosmetic => 0,
                }).sum();
                (name, weighted)
            })
            .collect();
        ranked.sort_by_key(|b| std::cmp::Reverse(b.1));
        ranked
    }

    pub fn module_roadmap(&self, module_name: &str) -> Vec<&PlannedEvolution> {
        self.queue.iter()
            .filter(|p| p.target_module.as_deref() == Some(module_name))
            .collect()
    }

    pub fn record_action(&mut self, plan_id: &str, status: ActionStatus, model: &mut SelfModel) {
        let action = EvolutionAction {
            timestamp: chrono::Utc::now(),
            plan_id: plan_id.to_string(),
            status: status.clone(),
        };
        self.history.push(action);

        if let Some(plan) = self.queue.iter().find(|p| p.id == plan_id) {
            model.register_evolution(EvolutionEvent {
                timestamp: chrono::Utc::now(),
                kind: match status {
                    ActionStatus::Completed => EventKind::TechDebtResolved,
                    ActionStatus::InProgress => EventKind::EvolutionPlanned,
                    ActionStatus::Blocked => EventKind::WeaknessDetected,
                },
                description: format!("{}: {} — {:?}", plan_id, plan.action, status),
                affected_modules: plan.weakness.file.clone()
                    .map(|f| vec![f])
                    .unwrap_or_default(),
            });
        }

        if matches!(status, ActionStatus::Completed) {
            self.queue.retain(|p| p.id != plan_id);
        }
    }

    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    pub fn completion_rate(&self) -> f64 {
        let total = self.queue.len() + self.history.len();
        if total == 0 {
            return 1.0;
        }
        let completed = self.history.iter()
            .filter(|a| matches!(a.status, ActionStatus::Completed))
            .count();
        completed as f64 / total as f64
    }
}

#[derive(Debug, Clone)]
pub struct PlannedEvolution {
    pub id: String,
    pub priority: u8,
    pub weakness: Weakness,
    pub target_module: Option<String>,
    pub action: String,
    pub estimated_impact: ImpactEstimate,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ImpactEstimate {
    pub files_affected: usize,
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct EvolutionAction {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub plan_id: String,
    pub status: ActionStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionStatus {
    InProgress,
    Completed,
    Blocked,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_meta::weakness::Weakness;

    fn make_weakness(severity: DebtSeverity, pattern: &str) -> Weakness {
        Weakness {
            pattern_id: pattern.to_string(),
            target_module: None,
            file: None,
            line: None,
            severity,
            description: "test weakness".into(),
            impact: "test impact".into(),
            suggestion: "fix it".into(),
        }
    }

    #[test]
    fn test_planner_prioritizes_critical_first() {
        let mut planner = EvolutionPlanner::new();
        let weaknesses = vec![
            make_weakness(DebtSeverity::Minor, "LARGE_FILE"),
            make_weakness(DebtSeverity::Critical, "CIRCULAR_DEP"),
            make_weakness(DebtSeverity::Major, "MISSING_TESTS"),
        ];
        let plans = planner.plan_from_weaknesses(weaknesses);
        assert!(plans[0].weakness.severity == DebtSeverity::Critical);
        assert!(plans[1].weakness.severity == DebtSeverity::Major);
        assert!(plans[2].weakness.severity == DebtSeverity::Minor);
    }

    #[test]
    fn test_planner_batches_by_priority() {
        let mut planner = EvolutionPlanner::new();
        planner.max_concurrent = 2;
        let weaknesses = vec![
            make_weakness(DebtSeverity::Critical, "CIRCULAR_DEP"),
            make_weakness(DebtSeverity::Critical, "EXCESS_UNSAFE"),
            make_weakness(DebtSeverity::Major, "MISSING_TESTS"),
        ];
        planner.plan_from_weaknesses(weaknesses);
        let batch = planner.next_batch();
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_planner_record_completion() {
        let mut planner = EvolutionPlanner::new();
        let mut model = SelfModel::new();
        planner.plan_from_weaknesses(vec![
            make_weakness(DebtSeverity::Critical, "CIRCULAR_DEP"),
        ]);
        planner.queue[0].id = "EVO-1".into();
        planner.record_action("EVO-1", ActionStatus::Completed, &mut model);
        assert_eq!(planner.pending_count(), 0);
        assert_eq!(planner.completion_rate(), 1.0);
    }

    #[test]
    fn test_planner_impact_estimation() {
        let planner = EvolutionPlanner::new();
        let dep = make_weakness(DebtSeverity::Critical, "CIRCULAR_DEP");
        let plan = planner.weakness_to_plan(&dep);
        assert_eq!(plan.estimated_impact.risk, RiskLevel::High);
        assert_eq!(plan.estimated_impact.files_affected, 5);

        let test = make_weakness(DebtSeverity::Major, "MISSING_TESTS");
        let plan = planner.weakness_to_plan(&test);
        assert_eq!(plan.estimated_impact.risk, RiskLevel::Low);
    }

    #[test]
    fn test_completion_rate_empty() {
        let planner = EvolutionPlanner::new();
        assert!((planner.completion_rate() - 1.0).abs() < 0.001);
    }
}

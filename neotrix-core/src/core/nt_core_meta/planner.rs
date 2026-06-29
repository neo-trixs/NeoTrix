use super::self_model::{DebtSeverity, EventKind, EvolutionEvent, SelfModel};
use super::weakness::{Weakness, WeaknessReport};
use std::collections::HashMap;

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
        let mut plans: Vec<PlannedEvolution> = report
            .weaknesses
            .iter()
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
        let mut plans: Vec<PlannedEvolution> = weaknesses
            .into_iter()
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
            estimated_impact: ImpactEstimate {
                files_affected,
                risk,
            },
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
        self.queue
            .iter()
            .filter(|p| p.priority == first_priority)
            .take(self.max_concurrent)
            .collect()
    }

    pub fn plans_by_module(&self) -> HashMap<String, Vec<&PlannedEvolution>> {
        let mut map: HashMap<String, Vec<&PlannedEvolution>> = HashMap::new();
        for plan in &self.queue {
            let key = plan
                .target_module
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            map.entry(key).or_default().push(plan);
        }
        map
    }

    pub fn weakest_modules(&self) -> Vec<(String, usize)> {
        let by_module = self.plans_by_module();
        let mut ranked: Vec<(String, usize)> = by_module
            .into_iter()
            .map(|(name, plans)| {
                let weighted = plans
                    .iter()
                    .map(|p| match p.weakness.severity {
                        DebtSeverity::Critical => 5,
                        DebtSeverity::Major => 3,
                        DebtSeverity::Minor => 1,
                        DebtSeverity::Cosmetic => 0,
                    })
                    .sum();
                (name, weighted)
            })
            .collect();
        ranked.sort_by_key(|b| std::cmp::Reverse(b.1));
        ranked
    }

    pub fn module_roadmap(&self, module_name: &str) -> Vec<&PlannedEvolution> {
        self.queue
            .iter()
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
        const MAX_HISTORY: usize = 10000;
        if self.history.len() > MAX_HISTORY {
            let excess = self.history.len() - MAX_HISTORY;
            self.history.drain(0..excess);
        }

        if let Some(plan) = self.queue.iter().find(|p| p.id == plan_id) {
            model.register_evolution(EvolutionEvent {
                timestamp: chrono::Utc::now(),
                kind: match status {
                    ActionStatus::Completed => EventKind::TechDebtResolved,
                    ActionStatus::InProgress => EventKind::EvolutionPlanned,
                    ActionStatus::Blocked => EventKind::WeaknessDetected,
                },
                description: format!("{}: {} — {:?}", plan_id, plan.action, status),
                affected_modules: plan
                    .weakness
                    .file
                    .clone()
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
        let completed = self
            .history
            .iter()
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

#[derive(Debug, Clone)]
pub struct MetaGoal {
    pub description: String,
    pub priority: f64,
    pub dedup_key: String,
}

fn severity_to_priority(severity: &DebtSeverity) -> f64 {
    match severity {
        DebtSeverity::Critical => 0.9,
        DebtSeverity::Major => 0.7,
        DebtSeverity::Minor => 0.4,
        DebtSeverity::Cosmetic => 0.2,
    }
}

pub fn weakness_to_goals(w: &Weakness) -> Vec<MetaGoal> {
    let priority = severity_to_priority(&w.severity);
    let dedup_key = format!(
        "{}:{}",
        w.pattern_id,
        w.target_module.as_deref().unwrap_or("global")
    );
    let description = format!("{} — {}", w.pattern_id, w.description);
    vec![MetaGoal {
        description,
        priority,
        dedup_key,
    }]
}

pub struct MetaGoalBridge {
    existing_keys: std::collections::HashSet<String>,
}

impl Default for MetaGoalBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaGoalBridge {
    pub fn new() -> Self {
        Self {
            existing_keys: std::collections::HashSet::new(),
        }
    }

    pub fn register_existing_goals(&mut self, keys: &[String]) {
        for k in keys {
            self.existing_keys.insert(k.clone());
        }
    }

    pub fn generate_actionable_tasks(&mut self, weaknesses: &[Weakness]) -> Vec<MetaGoal> {
        let mut goals: Vec<MetaGoal> = weaknesses
            .iter()
            .flat_map(weakness_to_goals)
            .filter(|g| !self.existing_keys.contains(&g.dedup_key))
            .collect();
        goals.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for g in &goals {
            self.existing_keys.insert(g.dedup_key.clone());
        }
        goals
    }

    pub fn bridge_size(&self) -> usize {
        self.existing_keys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_meta::weakness::{Weakness, WeaknessReport, WeaknessSummary};

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
        planner.plan_from_weaknesses(vec![make_weakness(DebtSeverity::Critical, "CIRCULAR_DEP")]);
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

    fn sample_weakness(pattern: &str, severity: DebtSeverity, module: Option<&str>) -> Weakness {
        Weakness {
            pattern_id: pattern.to_string(),
            target_module: module.map(|s| s.to_string()),
            file: None,
            line: None,
            severity,
            description: format!("test weakness: {}", pattern),
            impact: "test impact".to_string(),
            suggestion: "test suggestion".to_string(),
        }
    }

    #[test]
    fn test_weakness_to_goal_conversion() {
        let w = sample_weakness("LARGE_FILE", DebtSeverity::Minor, Some("core"));
        let goals = weakness_to_goals(&w);
        assert_eq!(goals.len(), 1);
        assert!(goals[0].description.contains("LARGE_FILE"));
        assert!((goals[0].priority - 0.4).abs() < 1e-6);
        assert_eq!(goals[0].dedup_key, "LARGE_FILE:core");
    }

    #[test]
    fn test_dedup_prevents_duplicates() {
        let w = sample_weakness("MISSING_TESTS", DebtSeverity::Major, Some("core"));
        let mut bridge = MetaGoalBridge::new();
        let goals = bridge.generate_actionable_tasks(&[w.clone()]);
        assert_eq!(goals.len(), 1);
        let goals2 = bridge.generate_actionable_tasks(&[w]);
        assert_eq!(goals2.len(), 0);
    }

    #[test]
    fn test_sorting_by_priority_descending() {
        let w1 = sample_weakness("LOW", DebtSeverity::Cosmetic, Some("a"));
        let w2 = sample_weakness("HIGH", DebtSeverity::Critical, Some("b"));
        let w3 = sample_weakness("MED", DebtSeverity::Major, Some("c"));
        let mut bridge = MetaGoalBridge::new();
        let goals = bridge.generate_actionable_tasks(&[w1, w2, w3]);
        assert_eq!(goals.len(), 3);
        assert_eq!(goals[0].dedup_key, "HIGH:b");
        assert_eq!(goals[1].dedup_key, "MED:c");
        assert_eq!(goals[2].dedup_key, "LOW:a");
    }

    #[test]
    fn test_empty_input() {
        let mut bridge = MetaGoalBridge::new();
        let goals = bridge.generate_actionable_tasks(&[]);
        assert!(goals.is_empty());
        assert_eq!(bridge.bridge_size(), 0);
    }

    #[test]
    fn test_register_existing_goals_blocks_duplicates() {
        let mut bridge = MetaGoalBridge::new();
        bridge.register_existing_goals(&["LARGE_FILE:core".to_string()]);
        let w = sample_weakness("LARGE_FILE", DebtSeverity::Minor, Some("core"));
        let goals = bridge.generate_actionable_tasks(&[w]);
        assert!(goals.is_empty());
    }

    #[test]
    fn test_critical_priority_mapping() {
        let w = sample_weakness("EXCESS_UNSAFE", DebtSeverity::Critical, None);
        let goals = weakness_to_goals(&w);
        assert!((goals[0].priority - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_key_format_no_module() {
        let w = sample_weakness("DEBT_ACCUMULATION", DebtSeverity::Critical, None);
        let goals = weakness_to_goals(&w);
        assert_eq!(goals[0].dedup_key, "DEBT_ACCUMULATION:global");
    }

    #[test]
    fn test_bridge_size_tracks_insertions() {
        let mut bridge = MetaGoalBridge::new();
        assert_eq!(bridge.bridge_size(), 0);
        let w1 = sample_weakness("A", DebtSeverity::Minor, Some("m1"));
        let w2 = sample_weakness("B", DebtSeverity::Minor, Some("m1"));
        bridge.generate_actionable_tasks(&[w1, w2]);
        assert_eq!(bridge.bridge_size(), 2);
    }

    #[test]
    fn test_meta_goal_bridge_creation() {
        let bridge = MetaGoalBridge::new();
        assert_eq!(bridge.bridge_size(), 0);
    }

    #[test]
    fn test_weakness_to_goals_empty_description() {
        let w = Weakness {
            pattern_id: "EMPTY_TEST".to_string(),
            target_module: None,
            file: None,
            line: None,
            severity: DebtSeverity::Cosmetic,
            description: String::new(),
            impact: String::new(),
            suggestion: String::new(),
        };
        let goals = weakness_to_goals(&w);
        assert_eq!(goals.len(), 1);
        assert!((goals[0].priority - 0.2).abs() < 1e-6);
        assert_eq!(goals[0].dedup_key, "EMPTY_TEST:global");
    }

    #[test]
    fn test_planner_plan_from_report_empty() {
        let mut planner = EvolutionPlanner::new();
        let report = WeaknessReport {
            timestamp: chrono::Utc::now(),
            weaknesses: vec![],
            summary: WeaknessSummary {
                total_count: 0,
                critical_count: 0,
                major_count: 0,
                minor_count: 0,
                cosmetic_count: 0,
            },
        };
        let plans = planner.plan_from_report(&report);
        assert!(plans.is_empty());
        assert!(planner.queue.is_empty());
    }

    #[test]
    fn test_planner_plans_by_module_grouping() {
        let mut planner = EvolutionPlanner::new();
        let w1 = sample_weakness("LARGE_FILE", DebtSeverity::Minor, Some("core"));
        let w2 = sample_weakness("MISSING_TESTS", DebtSeverity::Major, Some("core"));
        let w3 = sample_weakness("CIRCULAR_DEP", DebtSeverity::Critical, Some("agent"));
        planner.plan_from_weaknesses(vec![w1, w2, w3]);
        let by_module = planner.plans_by_module();
        assert_eq!(by_module.len(), 2);
        assert_eq!(by_module.get("core").unwrap().len(), 2);
        assert_eq!(by_module.get("agent").unwrap().len(), 1);
    }

    #[test]
    fn test_planner_module_roadmap_filter() {
        let mut planner = EvolutionPlanner::new();
        let w1 = sample_weakness("LARGE_FILE", DebtSeverity::Minor, Some("target_mod"));
        let w2 = sample_weakness("OTHER", DebtSeverity::Major, Some("other_mod"));
        planner.plan_from_weaknesses(vec![w1, w2]);
        let roadmap = planner.module_roadmap("target_mod");
        assert_eq!(roadmap.len(), 1);
        assert!(roadmap[0].action.contains("test suggestion"));
    }

    #[test]
    fn test_planner_module_roadmap_unknown() {
        let planner = EvolutionPlanner::new();
        let roadmap = planner.module_roadmap("nonexistent");
        assert!(roadmap.is_empty());
    }

    #[test]
    fn test_planner_completion_rate_partial() {
        let mut planner = EvolutionPlanner::new();
        let mut model = SelfModel::new();
        planner.plan_from_weaknesses(vec![
            sample_weakness("A", DebtSeverity::Critical, Some("m1")),
            sample_weakness("B", DebtSeverity::Major, Some("m2")),
        ]);
        planner.queue[0].id = "EVO-1".into();
        planner.queue[1].id = "EVO-2".into();
        planner.record_action("EVO-1", ActionStatus::Completed, &mut model);
        planner.record_action("EVO-2", ActionStatus::InProgress, &mut model);
        let rate = planner.completion_rate();
        assert!((rate - 1.0 / 3.0).abs() < 0.001, "rate={}", rate);
        assert_eq!(planner.pending_count(), 1);
    }

    #[test]
    fn test_planner_record_action_blocked_stays_in_queue() {
        let mut planner = EvolutionPlanner::new();
        let mut model = SelfModel::new();
        planner.plan_from_weaknesses(vec![sample_weakness(
            "CIRCULAR_DEP",
            DebtSeverity::Critical,
            Some("core"),
        )]);
        planner.queue[0].id = "EVO-1".into();
        planner.record_action("EVO-1", ActionStatus::Blocked, &mut model);
        assert_eq!(planner.pending_count(), 1);
        let rate = planner.completion_rate();
        assert!((rate).abs() < 0.001);
    }

    #[test]
    fn test_weakest_modules_ranking_by_severity() {
        let mut planner = EvolutionPlanner::new();
        let w1 = sample_weakness("CRIT", DebtSeverity::Critical, Some("mod1"));
        let w2 = sample_weakness("MAJ", DebtSeverity::Major, Some("mod2"));
        let w3 = sample_weakness("MIN", DebtSeverity::Minor, Some("mod1"));
        planner.plan_from_weaknesses(vec![w1, w2, w3]);
        let ranked = planner.weakest_modules();
        assert_eq!(ranked[0].0, "mod1");
        assert_eq!(ranked[0].1, 6);
        assert_eq!(ranked[1].0, "mod2");
        assert_eq!(ranked[1].1, 3);
    }

    #[test]
    fn test_weakness_to_goals_major_severity() {
        let w = sample_weakness("MISSING_TESTS", DebtSeverity::Major, Some("test_module"));
        let goals = weakness_to_goals(&w);
        assert!((goals[0].priority - 0.7).abs() < 1e-6);
        assert_eq!(goals[0].dedup_key, "MISSING_TESTS:test_module");
    }

    #[test]
    fn test_planner_default_max_concurrent() {
        let planner = EvolutionPlanner::new();
        assert_eq!(planner.max_concurrent, 3);
        assert!(planner.queue.is_empty());
        assert!(planner.history.is_empty());
    }

    #[test]
    fn test_planner_next_batch_empty_queue() {
        let planner = EvolutionPlanner::new();
        let batch = planner.next_batch();
        assert!(batch.is_empty());
    }
}

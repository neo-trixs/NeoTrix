use crate::neotrix::nt_act_goal::goal_generator::{EvolutionGoal, GoalCategory, GoalPriority};
use crate::neotrix::nt_mind_autonomy::trend_analyzer::{TrendDirection, TrendReport};

#[derive(Debug, Clone, PartialEq)]
pub enum MetaGoalCategory {
    ProcessImprovement,
    PerformanceOptimization,
    QualityGate,
    KnowledgeExpansion,
    TechnicalDebt,
}

#[derive(Debug, Clone)]
pub struct MetaGoal {
    pub id: String,
    pub category: MetaGoalCategory,
    pub description: String,
    pub priority: GoalPriority,
    pub target_metric: String,
    pub current_value: f64,
    pub target_value: f64,
    pub derived_from: String,
}

#[derive(Debug, Clone)]
pub struct MetaGoalGenerator {
    max_meta_goals: usize,
    declining_threshold: usize,
}

impl MetaGoalGenerator {
    pub fn new() -> Self {
        Self {
            max_meta_goals: 5,
            declining_threshold: 2,
        }
    }

    pub fn generate_from_trends(&self, report: &TrendReport) -> Vec<MetaGoal> {
        let mut goals = Vec::new();
        let declining_count = report.declining_count as usize;

        if declining_count >= self.declining_threshold {
            let labels: Vec<&str> = report
                .trends
                .iter()
                .filter(|t| t.direction == TrendDirection::Declining)
                .map(|t| t.label.as_str())
                .collect();
            let avg_slope: f64 = report
                .trends
                .iter()
                .filter(|t| t.direction == TrendDirection::Declining)
                .map(|t| t.slope)
                .sum::<f64>()
                / declining_count as f64;

            goals.push(MetaGoal {
                id: format!("META-PROCESS-IMPROVE-{}", goals.len() + 1),
                category: MetaGoalCategory::ProcessImprovement,
                description: format!(
                    "Improve {} — currently declining with slope {:.4}",
                    labels.join(", "),
                    avg_slope
                ),
                priority: GoalPriority::High,
                target_metric: "declining_trend_count".into(),
                current_value: declining_count as f64,
                target_value: (self.declining_threshold - 1) as f64,
                derived_from: "trend_analyzer.declining_count".into(),
            });
        }

        if report.overall_direction == TrendDirection::Declining {
            goals.push(MetaGoal {
                id: format!("META-QUALITY-GATE-{}", goals.len() + 1),
                category: MetaGoalCategory::QualityGate,
                description: "Overall evolution quality is declining — tighten quality gates"
                    .into(),
                priority: GoalPriority::Critical,
                target_metric: "overall_direction".into(),
                current_value: 0.0,
                target_value: 1.0,
                derived_from: "trend_analyzer.overall_direction".into(),
            });
        }

        if report.improving_count == 0 && !report.trends.is_empty() {
            goals.push(MetaGoal {
                id: format!("META-KNOWLEDGE-EXPAND-{}", goals.len() + 1),
                category: MetaGoalCategory::KnowledgeExpansion,
                description:
                    "No improving trends detected — expand knowledge domains to unlock improvements"
                        .into(),
                priority: GoalPriority::Medium,
                target_metric: "improving_count".into(),
                current_value: 0.0,
                target_value: 1.0,
                derived_from: "trend_analyzer.improving_count".into(),
            });
        }

        goals.truncate(self.max_meta_goals);
        goals
    }

    pub fn generate_initial_goals(&self) -> Vec<MetaGoal> {
        vec![
            MetaGoal {
                id: "META-AUTO-FIX-BASELINE".into(),
                category: MetaGoalCategory::QualityGate,
                description: "Establish auto-fix rate baseline".into(),
                priority: GoalPriority::High,
                target_metric: "auto_fix_rate".into(),
                current_value: 0.0,
                target_value: 0.5,
                derived_from: "initial".into(),
            },
            MetaGoal {
                id: "META-REDUCE-WARNINGS".into(),
                category: MetaGoalCategory::TechnicalDebt,
                description: "Reduce compilation warning count".into(),
                priority: GoalPriority::Medium,
                target_metric: "compile_warnings".into(),
                current_value: 0.0,
                target_value: 0.0,
                derived_from: "initial".into(),
            },
            MetaGoal {
                id: "META-TEST-COVERAGE-TREND".into(),
                category: MetaGoalCategory::ProcessImprovement,
                description: "Increase test coverage trend tracking".into(),
                priority: GoalPriority::Medium,
                target_metric: "test_coverage_trend".into(),
                current_value: 0.0,
                target_value: 1.0,
                derived_from: "initial".into(),
            },
        ]
    }

    pub fn to_evolution_goal(&self, meta: &MetaGoal) -> EvolutionGoal {
        let category = match meta.category {
            MetaGoalCategory::ProcessImprovement => GoalCategory::CodeHealth,
            MetaGoalCategory::PerformanceOptimization => GoalCategory::Performance,
            MetaGoalCategory::QualityGate => GoalCategory::TestCoverage,
            MetaGoalCategory::KnowledgeExpansion => GoalCategory::Knowledge,
            MetaGoalCategory::TechnicalDebt => GoalCategory::CodeHealth,
        };

        EvolutionGoal {
            id: meta.id.clone(),
            category,
            priority: meta.priority,
            description: meta.description.clone(),
            target_file: None,
            expected_impact: 0.5,
            effort_estimate: 0.3,
            dependencies: vec![],
        }
    }

    pub fn summarize(&self, goals: &[MetaGoal]) -> String {
        if goals.is_empty() {
            return "No meta-goals generated.".into();
        }
        let mut categories: Vec<String> = Vec::new();
        for g in goals {
            let cat = format!("{:?}", g.category);
            if !categories.contains(&cat) {
                categories.push(cat);
            }
        }
        format!(
            "{} meta-goal(s) across {} category(ies): {}",
            goals.len(),
            categories.len(),
            categories.join(", ")
        )
    }
}

impl Default for MetaGoalGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind_autonomy::trend_analyzer::Trend;

    fn declining_report() -> TrendReport {
        TrendReport {
            trends: vec![
                Trend {
                    label: "auto_fix_rate".into(),
                    direction: TrendDirection::Declining,
                    slope: -0.15,
                    intercept: 0.8,
                    confidence: 0.9,
                    data_points: 10,
                    prediction_next: 0.5,
                },
                Trend {
                    label: "test_pass_rate".into(),
                    direction: TrendDirection::Declining,
                    slope: -0.05,
                    intercept: 0.95,
                    confidence: 0.85,
                    data_points: 15,
                    prediction_next: 0.88,
                },
            ],
            overall_direction: TrendDirection::Declining,
            declining_count: 2,
            improving_count: 0,
            timestamp: 1000,
        }
    }

    fn stable_report() -> TrendReport {
        TrendReport {
            trends: vec![Trend {
                label: "auto_fix_rate".into(),
                direction: TrendDirection::Improving,
                slope: 0.1,
                intercept: 0.5,
                confidence: 0.8,
                data_points: 10,
                prediction_next: 0.7,
            }],
            overall_direction: TrendDirection::Improving,
            declining_count: 0,
            improving_count: 1,
            timestamp: 1000,
        }
    }

    fn below_threshold_report() -> TrendReport {
        TrendReport {
            trends: vec![Trend {
                label: "auto_fix_rate".into(),
                direction: TrendDirection::Declining,
                slope: -0.1,
                intercept: 0.6,
                confidence: 0.7,
                data_points: 5,
                prediction_next: 0.4,
            }],
            overall_direction: TrendDirection::Stable,
            declining_count: 1,
            improving_count: 1,
            timestamp: 1000,
        }
    }

    fn zero_improving_report() -> TrendReport {
        TrendReport {
            trends: vec![Trend {
                label: "compile_time".into(),
                direction: TrendDirection::Stable,
                slope: 0.0,
                intercept: 5.0,
                confidence: 0.5,
                data_points: 3,
                prediction_next: 5.0,
            }],
            overall_direction: TrendDirection::Stable,
            declining_count: 0,
            improving_count: 0,
            timestamp: 1000,
        }
    }

    #[test]
    fn test_generate_from_declining_trends() {
        let generator = MetaGoalGenerator::new();
        let goals = generator.generate_from_trends(&declining_report());
        assert!(!goals.is_empty());
        assert!(goals
            .iter()
            .any(|g| g.category == MetaGoalCategory::ProcessImprovement));
    }

    #[test]
    fn test_no_declining_trends_returns_empty() {
        let generator = MetaGoalGenerator::new();
        let goals = generator.generate_from_trends(&stable_report());
        assert!(goals.is_empty());
    }

    #[test]
    fn test_initial_goals_returns_three() {
        let generator = MetaGoalGenerator::new();
        let goals = generator.generate_initial_goals();
        assert_eq!(goals.len(), 3);
    }

    #[test]
    fn test_to_evolution_goal_mapping() {
        let generator = MetaGoalGenerator::new();
        let meta = MetaGoal {
            id: "META-TEST".into(),
            category: MetaGoalCategory::QualityGate,
            description: "Test mapping".into(),
            priority: GoalPriority::High,
            target_metric: "test_metric".into(),
            current_value: 0.0,
            target_value: 1.0,
            derived_from: "test".into(),
        };
        let evo = generator.to_evolution_goal(&meta);
        assert_eq!(evo.id, "META-TEST");
        assert_eq!(evo.category, GoalCategory::TestCoverage);
        assert_eq!(evo.priority, GoalPriority::High);
        assert_eq!(evo.description, "Test mapping");
        assert_eq!(evo.expected_impact, 0.5);
        assert_eq!(evo.effort_estimate, 0.3);
    }

    #[test]
    fn test_summary_format() {
        let generator = MetaGoalGenerator::new();
        let goals = generator.generate_initial_goals();
        let summary = generator.summarize(&goals);
        assert!(summary.contains("meta-goal(s)"));
        assert!(summary.contains("category(ies)"));
    }

    #[test]
    fn test_summary_empty() {
        let generator = MetaGoalGenerator::new();
        assert_eq!(generator.summarize(&[]), "No meta-goals generated.");
    }

    #[test]
    fn test_multiple_declining_triggers_process_improvement() {
        let generator = MetaGoalGenerator::new();
        let goals = generator.generate_from_trends(&declining_report());
        let pi: Vec<_> = goals
            .iter()
            .filter(|g| g.category == MetaGoalCategory::ProcessImprovement)
            .collect();
        assert_eq!(pi.len(), 1);
        assert!(pi[0].description.contains("auto_fix_rate"));
        assert!(pi[0].description.contains("test_pass_rate"));
    }

    #[test]
    fn test_below_threshold_returns_empty() {
        let generator = MetaGoalGenerator::new();
        let goals = generator.generate_from_trends(&below_threshold_report());
        assert!(goals.is_empty());
    }

    #[test]
    fn test_zero_improving_triggers_knowledge_expansion() {
        let generator = MetaGoalGenerator::new();
        let goals = generator.generate_from_trends(&zero_improving_report());
        assert!(goals
            .iter()
            .any(|g| g.category == MetaGoalCategory::KnowledgeExpansion));
    }

    #[test]
    fn test_overall_declining_triggers_quality_gate() {
        let generator = MetaGoalGenerator::new();
        let goals = generator.generate_from_trends(&declining_report());
        assert!(goals
            .iter()
            .any(|g| g.category == MetaGoalCategory::QualityGate));
    }
}

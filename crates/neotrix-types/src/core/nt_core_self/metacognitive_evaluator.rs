use std::collections::HashSet;
use super::silicon_self::SiliconSelfModel;
use super::reasoning_strategy::StrategyKind;

#[derive(Debug, Clone)]
pub struct CognitiveHealthReport {
    pub attention_health: f64,
    pub strategy_diversity: f64,
    pub trace_quality: f64,
    pub context_pressure: f64,
    pub stability_score: f64,
    pub flags: Vec<CognitiveFlag>,
    pub repair_suggestions: Vec<RepairSuggestion>,
    pub evaluation_id: usize,
}

#[derive(Debug, Clone)]
pub struct CognitiveFlag {
    pub severity: FlagSeverity,
    pub category: FlagCategory,
    pub message: String,
    pub metric_value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FlagSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagCategory {
    Attention,
    Strategy,
    Trace,
    Context,
    Identity,
}

#[derive(Debug, Clone)]
pub struct RepairSuggestion {
    pub target: RepairTarget,
    pub action: String,
    pub priority: usize,
    pub expected_impact: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairTarget {
    AttentionStimulus,
    StrategyBoost,
    ContextReset,
    IdentityUpdate,
    TracePrune,
}

#[derive(Debug, Clone)]
pub struct CognitiveEvaluator {
    pub evaluation_count: usize,
    pub history: Vec<CognitiveHealthReport>,
}

impl Default for CognitiveEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl CognitiveEvaluator {
    pub fn new() -> Self {
        Self {
            evaluation_count: 0,
            history: Vec::new(),
        }
    }

    pub fn evaluate(&mut self, model: &SiliconSelfModel) -> CognitiveHealthReport {
        let evaluation_id = self.evaluation_count;
        self.evaluation_count += 1;

        let attention_health = self.compute_attention_health(model);
        let strategy_diversity = self.compute_strategy_diversity(model);
        let trace_quality = self.compute_trace_quality(model);
        let context_pressure = self.compute_context_pressure(model);

        let flags = self.generate_flags(attention_health, strategy_diversity, trace_quality, context_pressure, model);

        let repair_suggestions = Self::generate_repair_suggestions(&flags);

        let stability_score = if attention_health > 0.0 || strategy_diversity > 0.0
            || trace_quality > 0.0
        {
            attention_health * 0.25
                + strategy_diversity * 0.25
                + trace_quality * 0.25
                + (1.0 - context_pressure) * 0.25
        } else {
            0.0
        };

        let report = CognitiveHealthReport {
            attention_health,
            strategy_diversity,
            trace_quality,
            context_pressure,
            stability_score,
            flags,
            repair_suggestions,
            evaluation_id,
        };

        self.history.push(report.clone());
        report
    }

    fn compute_attention_health(&self, model: &SiliconSelfModel) -> f64 {
        let total = model.attention_manager.heads.len();
        if total == 0 {
            return 0.0;
        }
        let active = model.attention_manager.heads.iter()
            .filter(|h| h.activation > 0.1)
            .count();
        active as f64 / total as f64
    }

    fn compute_strategy_diversity(&self, model: &SiliconSelfModel) -> f64 {
        let total_strategies = StrategyKind::all().len();
        if total_strategies == 0 {
            return 0.0;
        }
        let recent_traces = model.recent_traces(10);
        let mut unique_strategies: HashSet<StrategyKind> = HashSet::new();
        for trace in &recent_traces {
            for strategy in trace.strategies_used() {
                unique_strategies.insert(strategy);
            }
        }
        unique_strategies.len() as f64 / total_strategies as f64
    }

    fn compute_trace_quality(&self, model: &SiliconSelfModel) -> f64 {
        let recent_traces = model.recent_traces(5);
        if recent_traces.is_empty() {
            return 0.0;
        }
        let sum: f64 = recent_traces.iter().map(|t| t.grade.score()).sum();
        sum / recent_traces.len() as f64
    }

    fn compute_context_pressure(&self, model: &SiliconSelfModel) -> f64 {
        if model.context_window.capacity == 0 {
            return 1.0;
        }
        model.context_window.len() as f64 / model.context_window.capacity as f64
    }

    fn generate_flags(
        &self,
        attention_health: f64,
        strategy_diversity: f64,
        trace_quality: f64,
        context_pressure: f64,
        model: &SiliconSelfModel,
    ) -> Vec<CognitiveFlag> {
        let mut flags: Vec<CognitiveFlag> = Vec::new();

        if attention_health < 0.3 {
            flags.push(CognitiveFlag {
                severity: FlagSeverity::Critical,
                category: FlagCategory::Attention,
                message: format!("Attention health is critically low: {:.2}", attention_health),
                metric_value: attention_health,
            });
        }

        if strategy_diversity < 0.3 {
            flags.push(CognitiveFlag {
                severity: FlagSeverity::Warning,
                category: FlagCategory::Strategy,
                message: format!("Strategy diversity is low: {:.2}", strategy_diversity),
                metric_value: strategy_diversity,
            });
        }

        if trace_quality < 0.4 {
            flags.push(CognitiveFlag {
                severity: FlagSeverity::Critical,
                category: FlagCategory::Trace,
                message: format!("Trace quality is critically low: {:.2}", trace_quality),
                metric_value: trace_quality,
            });
        }

        if context_pressure > 0.85 {
            flags.push(CognitiveFlag {
                severity: FlagSeverity::Warning,
                category: FlagCategory::Context,
                message: format!("Context pressure is high: {:.2}", context_pressure),
                metric_value: context_pressure,
            });
        }

        let identity_avg = model.identity.capabilities.values().sum::<f64>()
            / model.identity.capabilities.len().max(1) as f64;
        if identity_avg < 0.3 {
            flags.push(CognitiveFlag {
                severity: FlagSeverity::Warning,
                category: FlagCategory::Identity,
                message: format!("Identity capability level is low: {:.2}", identity_avg),
                metric_value: identity_avg,
            });
        }

        flags
    }

    fn generate_repair_suggestions(flags: &[CognitiveFlag]) -> Vec<RepairSuggestion> {
        let mut suggestions: Vec<RepairSuggestion> = Vec::new();

        for flag in flags {
            match flag.category {
                FlagCategory::Attention => {
                    suggestions.push(RepairSuggestion {
                        target: RepairTarget::AttentionStimulus,
                        action: "Stimulate underactive attention domains to broaden coverage".into(),
                        priority: 1,
                        expected_impact: 0.6,
                    });
                }
                FlagCategory::Strategy => {
                    suggestions.push(RepairSuggestion {
                        target: RepairTarget::StrategyBoost,
                        action: "Introduce unused reasoning strategies to improve diversity".into(),
                        priority: 2,
                        expected_impact: 0.5,
                    });
                }
                FlagCategory::Trace => {
                    suggestions.push(RepairSuggestion {
                        target: RepairTarget::TracePrune,
                        action: "Prune low-quality traces and reinforce successful patterns".into(),
                        priority: 1,
                        expected_impact: 0.7,
                    });
                }
                FlagCategory::Context => {
                    suggestions.push(RepairSuggestion {
                        target: RepairTarget::ContextReset,
                        action: "Reset context window to reduce pressure and free capacity".into(),
                        priority: 2,
                        expected_impact: 0.4,
                    });
                }
                FlagCategory::Identity => {
                    suggestions.push(RepairSuggestion {
                        target: RepairTarget::IdentityUpdate,
                        action: "Update capability scores to reflect observed performance".into(),
                        priority: 3,
                        expected_impact: 0.3,
                    });
                }
            }
        }

        suggestions.sort_by_key(|s| s.priority);
        suggestions
    }

    pub fn latest_report(&self) -> Option<&CognitiveHealthReport> {
        self.history.last()
    }

    pub fn has_degraded(&self, threshold: f64) -> bool {
        if self.history.len() < 2 {
            return false;
        }
        let prev = self.history[self.history.len() - 2].stability_score;
        let current = self.history.last().expect("history.len() >= 2 checked above").stability_score;
        current <= prev - threshold
    }

    pub fn summary(&self) -> String {
        if self.history.is_empty() {
            return "CognitiveEvaluator: no evaluations performed".to_string();
        }
        let latest = self.history.last().expect("history.is_empty() checked above");
        let flag_summary = if latest.flags.is_empty() {
            "no flags".to_string()
        } else {
            let critical = latest.flags.iter().filter(|f| matches!(f.severity, FlagSeverity::Critical)).count();
            let warning = latest.flags.iter().filter(|f| matches!(f.severity, FlagSeverity::Warning)).count();
            let info = latest.flags.iter().filter(|f| matches!(f.severity, FlagSeverity::Info)).count();
            format!("{} critical, {} warning, {} info", critical, warning, info)
        };
        format!(
            "Evaluation #{} | stability={:.3} | attention={:.2} | diversity={:.2} | quality={:.2} | pressure={:.2} | {}",
            latest.evaluation_id,
            latest.stability_score,
            latest.attention_health,
            latest.strategy_diversity,
            latest.trace_quality,
            latest.context_pressure,
            flag_summary,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::silicon_self::SiliconSelfModel;
    use super::super::thinking_trace::{ThinkingTrace, ThinkingStep};
    use super::super::reasoning_strategy::StrategyKind;

    #[test]
    fn test_evaluator_new() {
        let evaluator = CognitiveEvaluator::new();
        assert_eq!(evaluator.evaluation_count, 0);
        assert!(evaluator.history.is_empty());
        assert!(evaluator.latest_report().is_none());
    }

    #[test]
    fn test_evaluate_initial() {
        let model = SiliconSelfModel::new();
        let mut evaluator = CognitiveEvaluator::new();
        let report = evaluator.evaluate(&model);

        assert_eq!(report.evaluation_id, 0);
        assert!((report.attention_health - 0.0).abs() < 1e-6);
        assert!((report.strategy_diversity - 0.0).abs() < 1e-6);
        assert!((report.trace_quality - 0.0).abs() < 1e-6);
        assert!((report.context_pressure - 0.0).abs() < 1e-6);

        assert_eq!(report.flags.len(), 3);
        let has_critical_attention = report.flags.iter().any(|f| {
            matches!(f.severity, FlagSeverity::Critical) && matches!(f.category, FlagCategory::Attention)
        });
        let has_warning_strategy = report.flags.iter().any(|f| {
            matches!(f.severity, FlagSeverity::Warning) && matches!(f.category, FlagCategory::Strategy)
        });
        let has_critical_trace = report.flags.iter().any(|f| {
            matches!(f.severity, FlagSeverity::Critical) && matches!(f.category, FlagCategory::Trace)
        });
        assert!(has_critical_attention);
        assert!(has_warning_strategy);
        assert!(has_critical_trace);

        assert!(evaluator.latest_report().is_some());
    }

    #[test]
    fn test_evaluate_after_good_traces() {
        let mut model = SiliconSelfModel::new();
        let mut evaluator = CognitiveEvaluator::new();

        let report_before = evaluator.evaluate(&model);
        assert!((report_before.trace_quality - 0.0).abs() < 1e-6);

        for i in 0..5 {
            let mut trace = ThinkingTrace::new(i, &format!("task {}", i));
            trace.add_step(ThinkingStep::new(1, "step", StrategyKind::Direct));
            trace.add_step(ThinkingStep::new(2, "step", StrategyKind::ChainOfThought));
            trace.set_grade_from_accuracy(0.95);
            model.add_thinking_trace(trace);
        }

        let report_after = evaluator.evaluate(&model);
        assert!((report_after.trace_quality - 1.0).abs() < 1e-6);
        assert!(report_after.trace_quality > report_before.trace_quality);
    }

    #[test]
    fn test_has_degraded() {
        let model = SiliconSelfModel::new();
        let mut evaluator = CognitiveEvaluator::new();

        assert!(!evaluator.has_degraded(0.1));
        evaluator.evaluate(&model);
        assert!(!evaluator.has_degraded(0.1));
        evaluator.evaluate(&model);
        assert!(!evaluator.has_degraded(0.1));
    }

    #[test]
    fn test_summary_format() {
        let model = SiliconSelfModel::new();
        let mut evaluator = CognitiveEvaluator::new();
        let _empty_summary = evaluator.summary();
        assert_eq!(_empty_summary, "CognitiveEvaluator: no evaluations performed");

        evaluator.evaluate(&model);
        let summary = evaluator.summary();
        assert!(summary.starts_with("Evaluation #"));
        assert!(summary.contains("stability="));
        assert!(summary.contains("attention="));
        assert!(summary.contains("diversity="));
        assert!(summary.contains("quality="));
        assert!(summary.contains("pressure="));
    }

    #[test]
    fn test_flag_severity_order() {
        assert!(FlagSeverity::Critical > FlagSeverity::Warning);
        assert!(FlagSeverity::Warning > FlagSeverity::Info);
        assert!(FlagSeverity::Critical > FlagSeverity::Info);
    }
}

use super::silicon_self::SiliconSelfModel;

#[derive(Debug, Clone)]
pub struct ThresholdAdjustment {
    pub parameter: String,
    pub old_value: f64,
    pub new_value: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct PlanRecord {
    pub timestamp_id: usize,
    pub plan_type: String,
    pub execution_quality: f64,
    pub threshold_adjustments: Vec<ThresholdAdjustment>,
}

const MAX_PLAN_HISTORY: usize = 200;

pub struct SelfReferentialMonitor {
    pub plan_history: Vec<PlanRecord>,
    pub threshold_attention_min: f64,
    pub threshold_strategy_diversity_min: f64,
    pub threshold_trace_quality_min: f64,
    pub threshold_context_max: f64,
    pub auto_tune_enabled: bool,
    pub tuning_count: usize,
}

impl Default for SelfReferentialMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfReferentialMonitor {
    pub fn new() -> Self {
        Self {
            plan_history: Vec::new(),
            threshold_attention_min: 0.3,
            threshold_strategy_diversity_min: 0.3,
            threshold_trace_quality_min: 0.4,
            threshold_context_max: 0.85,
            auto_tune_enabled: true,
            tuning_count: 0,
        }
    }

    pub fn evaluate_plan_quality(&mut self, model: &SiliconSelfModel) -> f64 {
        let recent = model.recent_traces(3);
        if recent.is_empty() {
            return 0.0;
        }

        let avg_grade: f64 =
            recent.iter().map(|t| t.grade.score()).sum::<f64>() / recent.len() as f64;

        let mut total_effectiveness = 0.0;
        let mut count = 0;
        for trace in &recent {
            for strat in trace.strategies_used() {
                if let Some(s) = model.strategy_registry.strategies.get(&strat) {
                    total_effectiveness += s.effectiveness;
                    count += 1;
                }
            }
        }
        let avg_effectiveness = if count > 0 {
            total_effectiveness / count as f64
        } else {
            0.5
        };

        0.7 * avg_grade + 0.3 * avg_effectiveness
    }

    pub fn record_plan(&mut self, plan_type: &str, quality: f64) {
        self.plan_history.push(PlanRecord {
            timestamp_id: self.plan_history.len(),
            plan_type: plan_type.to_string(),
            execution_quality: quality,
            threshold_adjustments: Vec::new(),
        });
        if self.plan_history.len() > MAX_PLAN_HISTORY * 2 {
            self.plan_history
                .drain(0..self.plan_history.len() - MAX_PLAN_HISTORY);
        }
    }

    pub fn auto_tune(&mut self, _model: &SiliconSelfModel) -> Vec<ThresholdAdjustment> {
        let mut adjustments = Vec::new();
        if !self.auto_tune_enabled || self.plan_history.len() < 3 {
            return adjustments;
        }

        let recent_quality: f64 = self
            .plan_history
            .iter()
            .rev()
            .take(3)
            .map(|r| r.execution_quality)
            .sum::<f64>()
            / 3.0;

        if recent_quality >= 0.5 {
            return adjustments;
        }

        let old_attention = self.threshold_attention_min;
        let new_attention = (old_attention - 0.05).max(0.1);
        if (new_attention - old_attention).abs() > 1e-9 {
            self.threshold_attention_min = new_attention;
            adjustments.push(ThresholdAdjustment {
                parameter: "threshold_attention_min".into(),
                old_value: old_attention,
                new_value: new_attention,
                reason: "Low plan quality, reducing attention threshold".into(),
            });
        }

        let old_trace = self.threshold_trace_quality_min;
        let new_trace = (old_trace - 0.05).max(0.2);
        if (new_trace - old_trace).abs() > 1e-9 {
            self.threshold_trace_quality_min = new_trace;
            adjustments.push(ThresholdAdjustment {
                parameter: "threshold_trace_quality_min".into(),
                old_value: old_trace,
                new_value: new_trace,
                reason: "Low plan quality, reducing trace quality threshold".into(),
            });
        }

        let old_context = self.threshold_context_max;
        let new_context = (old_context + 0.05).min(0.95);
        if (new_context - old_context).abs() > 1e-9 {
            self.threshold_context_max = new_context;
            adjustments.push(ThresholdAdjustment {
                parameter: "threshold_context_max".into(),
                old_value: old_context,
                new_value: new_context,
                reason: "Low plan quality, increasing context max threshold".into(),
            });
        }

        if !adjustments.is_empty() {
            self.tuning_count += 1;
            if let Some(last) = self.plan_history.last_mut() {
                last.threshold_adjustments = adjustments.clone();
            }
        }

        adjustments
    }

    pub fn plan_quality_trend(&self) -> f64 {
        let n = self.plan_history.len();
        if n < 5 {
            return 0.0;
        }

        let start = n.saturating_sub(10);
        let recent: Vec<&PlanRecord> = self.plan_history[start..].iter().collect();
        let m = recent.len() as f64;

        let sum_x: f64 = (0..recent.len()).map(|i| i as f64).sum();
        let sum_y: f64 = recent.iter().map(|r| r.execution_quality).sum();
        let sum_xy: f64 = recent
            .iter()
            .enumerate()
            .map(|(i, r)| i as f64 * r.execution_quality)
            .sum();
        let sum_x2: f64 = (0..recent.len()).map(|i| (i as f64).powi(2)).sum();

        let denom = m * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-12 {
            return 0.0;
        }

        (m * sum_xy - sum_x * sum_y) / denom
    }

    pub fn summary(&self) -> String {
        format!(
            "SelfReferentialMonitor | plans={} | tuning_count={} | attention_min={:.2} | strategy_diversity_min={:.2} | trace_quality_min={:.2} | context_max={:.2} | trend={:.4} | needs_intervention={}",
            self.plan_history.len(),
            self.tuning_count,
            self.threshold_attention_min,
            self.threshold_strategy_diversity_min,
            self.threshold_trace_quality_min,
            self.threshold_context_max,
            self.plan_quality_trend(),
            self.needs_intervention(),
        )
    }

    pub fn needs_intervention(&self) -> bool {
        self.plan_history.len() >= 5 && self.plan_quality_trend() < -0.05
    }
}

#[cfg(test)]
mod tests {
    use super::super::reasoning_strategy::StrategyKind;
    use super::super::silicon_self::SiliconSelfModel;
    use super::super::thinking_trace::{ReflectionGrade, ThinkingStep, ThinkingTrace};
    use super::*;

    fn make_model_with_grades(grades: &[ReflectionGrade]) -> SiliconSelfModel {
        let mut model = SiliconSelfModel::new();
        for (i, grade) in grades.iter().enumerate() {
            let mut trace = ThinkingTrace::new(i, &format!("task{}", i));
            trace.grade = grade.clone();
            let mut step = ThinkingStep::new(1, "step", StrategyKind::Direct);
            step.confidence = grade.score();
            trace.add_step(step);
            model.add_thinking_trace(trace);
        }
        model
    }

    #[test]
    fn test_monitor_new() {
        let m = SelfReferentialMonitor::new();
        assert!((m.threshold_attention_min - 0.3).abs() < 1e-6);
        assert!((m.threshold_strategy_diversity_min - 0.3).abs() < 1e-6);
        assert!((m.threshold_trace_quality_min - 0.4).abs() < 1e-6);
        assert!((m.threshold_context_max - 0.85).abs() < 1e-6);
        assert!(m.plan_history.is_empty());
        assert_eq!(m.tuning_count, 0);
    }

    #[test]
    fn test_evaluate_initial() {
        let mut m = SelfReferentialMonitor::new();
        let model = SiliconSelfModel::new();
        let quality = m.evaluate_plan_quality(&model);
        assert!((quality - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_after_good_traces() {
        let mut m = SelfReferentialMonitor::new();
        let model = make_model_with_grades(&[
            ReflectionGrade::Excellent,
            ReflectionGrade::Good,
            ReflectionGrade::Excellent,
        ]);
        let quality = m.evaluate_plan_quality(&model);
        assert!(quality > 0.5);
    }

    #[test]
    fn test_auto_tune_triggers() {
        let mut m = SelfReferentialMonitor::new();
        let model = SiliconSelfModel::new();

        for _i in 0..4 {
            m.record_plan("test", 0.3);
        }

        let adjustments = m.auto_tune(&model);
        assert!(!adjustments.is_empty());

        assert!((m.threshold_attention_min - 0.25).abs() < 1e-6);
        assert!((m.threshold_trace_quality_min - 0.35).abs() < 1e-6);
        assert!((m.threshold_context_max - 0.90).abs() < 1e-6);
        assert_eq!(m.tuning_count, 1);
    }

    #[test]
    fn test_needs_intervention() {
        let mut m = SelfReferentialMonitor::new();
        for i in 0..6 {
            m.record_plan("decline", 0.5 - (i as f64 * 0.06));
        }
        assert!(m.needs_intervention());
    }

    #[test]
    fn test_summary_format() {
        let m = SelfReferentialMonitor::new();
        let s = m.summary();
        assert!(s.contains("SelfReferentialMonitor"));
        assert!(s.contains("plans="));
        assert!(s.contains("trend="));
    }
}

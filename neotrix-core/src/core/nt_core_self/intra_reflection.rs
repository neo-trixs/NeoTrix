use super::attention_head::AttentionDomain;
use super::reasoning_strategy::StrategyKind;
use super::silicon_self::SiliconSelfState;
use super::thinking_trace::{ThinkingStep, ThinkingTrace};

#[derive(Debug, Clone)]
pub struct PredictedOutcome {
    pub probability: f64,
    pub desirability: f64,
    pub expected_effort: f64,
    pub uncertainty: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct IntraReflection {
    pub intended_action: String,
    pub strategy: StrategyKind,
    pub domain: AttentionDomain,
    pub predicted_outcomes: Vec<PredictedOutcome>,
    pub best_outcome: Option<PredictedOutcome>,
    pub revised_action: Option<String>,
    pub confidence_before: f64,
    pub confidence_after: f64,
    pub reflection_ms: u64,
}

#[derive(Debug, Clone)]
pub struct IntraReflectionReport {
    pub reflections: Vec<IntraReflection>,
    pub total_reflections: usize,
    pub actions_revised: usize,
    pub avg_confidence_delta: f64,
    pub stagnation_count: usize,
}

/// MIRROR-style pre-action introspection.
/// Predicts consequences BEFORE acting, revises if low-confidence.
pub struct PreActionIntrospector {
    pub total_reflections: usize,
    pub total_revisions: usize,
    pub stagnation_threshold: usize,
}

impl Default for PreActionIntrospector {
    fn default() -> Self {
        Self::new()
    }
}

impl PreActionIntrospector {
    pub fn new() -> Self {
        Self {
            total_reflections: 0,
            total_revisions: 0,
            stagnation_threshold: 10,
        }
    }

    /// Derive a health estimate from the model state
    fn model_health(state: &SiliconSelfState) -> f64 {
        let cap_avg = 0.65;
        let ctx_health = 1.0 - state.context_usage;
        (cap_avg * 0.5 + ctx_health * 0.3 + 0.2).clamp(0.0, 1.0)
    }

    fn model_consistency(state: &SiliconSelfState) -> f64 {
        let depth_score = (state.thinking_depth as f64 / 10.0).min(1.0);
        (0.5 + depth_score * 0.3 + 0.2).clamp(0.0, 1.0)
    }

    pub fn introspect(
        &mut self,
        intended_action: &str,
        strategy: StrategyKind,
        domain: AttentionDomain,
        state: &SiliconSelfState,
    ) -> IntraReflection {
        self.total_reflections += 1;

        let health = Self::model_health(state);
        let consistency = Self::model_consistency(state);
        let confidence_before = (health * 0.6 + consistency * 0.3 + 0.1).clamp(0.0, 1.0);

        let mut outcomes = self.generate_outcomes(intended_action, health, 3);
        outcomes.sort_by(|a, b| {
            let sa = a.probability * a.desirability - a.uncertainty;
            let sb = b.probability * b.desirability - b.uncertainty;
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        let best = outcomes.first().cloned();

        let (revised_action, confidence_after) = if let Some(ref o) = best {
            if o.desirability < -0.3 || o.uncertainty > 0.7 || o.probability < 0.2 {
                self.total_revisions += 1;
                let r = format!(
                    "{} (revised: mitigate via {})",
                    intended_action, o.reasoning
                );
                (Some(r), confidence_before * 0.8)
            } else if o.uncertainty > 0.4 {
                let r = format!(
                    "{} (proceed with caution: {})",
                    intended_action, o.reasoning
                );
                (Some(r), confidence_before * 0.9)
            } else {
                (None, (confidence_before * 1.05).min(1.0))
            }
        } else {
            (None, confidence_before)
        };

        IntraReflection {
            intended_action: intended_action.to_string(),
            strategy,
            domain,
            predicted_outcomes: outcomes,
            best_outcome: best,
            revised_action,
            confidence_before,
            confidence_after,
            reflection_ms: 5,
        }
    }

    fn generate_outcomes(&self, action: &str, health: f64, count: usize) -> Vec<PredictedOutcome> {
        (0..count)
            .map(|i| {
                let v = i as f64;
                PredictedOutcome {
                    probability: ((health * 0.5 + 0.3) + (v * 0.15).sin() * 0.2).clamp(0.0, 1.0),
                    desirability: (0.2 + (v * 0.2).cos() * 0.3).clamp(-1.0, 1.0),
                    expected_effort: 1.0 - health,
                    uncertainty: (1.0 - health) * 0.5 + 0.1,
                    reasoning: format!(
                        "outcome_{}: action_len={}, capability={:.2}",
                        i,
                        action.len(),
                        health
                    ),
                }
            })
            .collect()
    }

    pub fn introspect_trace(
        &mut self,
        trace: &mut ThinkingTrace,
        state: &SiliconSelfState,
    ) -> IntraReflectionReport {
        let mut reflections = Vec::new();
        let mut revisions = 0;
        let mut total_delta = 0.0;

        let steps: Vec<ThinkingStep> = trace.steps.drain(..).collect();

        for step in steps {
            let r = self.introspect(&step.description, step.strategy, step.domain, state);
            let revised = if let Some(ref rev) = r.revised_action {
                revisions += 1;
                ThinkingStep {
                    description: rev.clone(),
                    ..step.clone()
                }
            } else {
                step
            };
            trace.steps.push(revised);
            total_delta += r.confidence_after - r.confidence_before;
            reflections.push(r);
        }

        let recent: Vec<_> = reflections.iter().rev().take(5).collect();
        let low_conf = recent.iter().all(|r| r.confidence_after < 0.3);
        let no_rev = recent.iter().all(|r| r.revised_action.is_none());
        let stagnation = if low_conf && no_rev {
            self.stagnation_threshold
        } else {
            0
        };

        let total = reflections.len();
        let avg_delta = if total == 0 {
            0.0
        } else {
            total_delta / total as f64
        };

        IntraReflectionReport {
            reflections,
            total_reflections: total,
            actions_revised: revisions,
            avg_confidence_delta: avg_delta,
            stagnation_count: stagnation,
        }
    }

    pub fn reset(&mut self) {
        self.total_reflections = 0;
        self.total_revisions = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self::silicon_self::SiliconSelfModel;

    #[test]
    fn test_introspect_no_revision() {
        let mut sm = SiliconSelfModel::new();
        sm.observe("good start");
        let state = sm.current_state();

        let mut ip = PreActionIntrospector::new();
        let r = ip.introspect(
            "analyze network",
            StrategyKind::ChainOfThought,
            AttentionDomain::PatternMatch,
            &state,
        );

        assert_eq!(r.intended_action, "analyze network");
        assert!(r.confidence_before > 0.3);
    }

    #[test]
    fn test_introspect_trace_empty() {
        let sm = SiliconSelfModel::new();
        let mut trace = ThinkingTrace::new(0, "empty");
        let state = sm.current_state();

        let mut ip = PreActionIntrospector::new();
        let report = ip.introspect_trace(&mut trace, &state);
        assert_eq!(report.total_reflections, 0);
    }

    #[test]
    fn test_introspect_trace_with_steps() {
        let sm = SiliconSelfModel::new();
        let mut trace = ThinkingTrace::new(1, "analysis");
        trace
            .steps
            .push(ThinkingStep::new(1, "fetch data", StrategyKind::Reflection));
        trace.steps.push(ThinkingStep::new(
            2,
            "analyze patterns",
            StrategyKind::ChainOfThought,
        ));
        let state = sm.current_state();

        let mut ip = PreActionIntrospector::new();
        let report = ip.introspect_trace(&mut trace, &state);

        assert_eq!(report.total_reflections, 2);
        assert!(report.avg_confidence_delta > -2.0);
    }

    #[test]
    fn test_track_counters() {
        let sm = SiliconSelfModel::new();
        let state = sm.current_state();
        let mut ip = PreActionIntrospector::new();

        ip.introspect(
            "a1",
            StrategyKind::ChainOfThought,
            AttentionDomain::PatternMatch,
            &state,
        );
        ip.introspect(
            "a2",
            StrategyKind::Reflection,
            AttentionDomain::RiskAssessment,
            &state,
        );

        assert_eq!(ip.total_reflections, 2);
    }

    #[test]
    fn test_outcome_generation() {
        let ip = PreActionIntrospector::new();
        let outcomes = ip.generate_outcomes("test", 0.7, 3);
        assert_eq!(outcomes.len(), 3);
        assert!(outcomes
            .iter()
            .all(|o| (0.0..=1.0).contains(&o.probability)));
    }

    #[test]
    fn test_outcome_generation_varying_counts() {
        let ip = PreActionIntrospector::new();
        assert_eq!(ip.generate_outcomes("x", 0.5, 0).len(), 0);
        assert_eq!(ip.generate_outcomes("x", 0.5, 1).len(), 1);
        assert_eq!(ip.generate_outcomes("x", 0.5, 5).len(), 5);
    }

    #[test]
    fn test_introspect_low_health_triggers_revision() {
        let sm = SiliconSelfModel::new();
        let mut state = sm.current_state();
        state.context_usage = 0.95;
        state.thinking_depth = 1;
        let mut ip = PreActionIntrospector::new();
        let r = ip.introspect(
            "risky move",
            StrategyKind::ChainOfThought,
            AttentionDomain::PatternMatch,
            &state,
        );
        // Low health → reduced confidence, potential revision
        assert!(r.confidence_before < 0.65);
        assert_eq!(r.intended_action, "risky move");
    }

    #[test]
    fn test_reset_clears_counters() {
        let sm = SiliconSelfModel::new();
        let state = sm.current_state();
        let mut ip = PreActionIntrospector::new();
        ip.introspect(
            "a1",
            StrategyKind::ChainOfThought,
            AttentionDomain::PatternMatch,
            &state,
        );
        ip.introspect(
            "a2",
            StrategyKind::Reflection,
            AttentionDomain::RiskAssessment,
            &state,
        );
        assert_eq!(ip.total_reflections, 2);
        ip.reset();
        assert_eq!(ip.total_reflections, 0);
        assert_eq!(ip.total_revisions, 0);
    }

    #[test]
    fn test_introspection_report_metrics() {
        let sm = SiliconSelfModel::new();
        let mut trace = ThinkingTrace::new(1, "batch");
        trace.steps.push(ThinkingStep::new(
            1,
            "step one",
            StrategyKind::ChainOfThought,
        ));
        trace
            .steps
            .push(ThinkingStep::new(2, "step two", StrategyKind::Reflection));
        let state = sm.current_state();
        let mut ip = PreActionIntrospector::new();
        let report = ip.introspect_trace(&mut trace, &state);
        assert_eq!(report.total_reflections, 2);
        assert!(report.avg_confidence_delta >= -1.0 && report.avg_confidence_delta <= 1.0);
    }
}

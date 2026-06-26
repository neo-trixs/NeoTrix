use crate::core::nt_core_meta::meta_learning::{
    ConsolidationOutcome, MetaLearning, MetaLearningParams,
};
use crate::core::nt_core_meta::MetacognitiveState;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct BindingCycleReport {
    pub cycle: u64,
    pub msv_before: MetacognitiveState,
    pub msv_after: MetacognitiveState,
    pub meta_params_before: MetaLearningParams,
    pub meta_params_after: MetaLearningParams,
    pub narrative_quality: f64,
    pub curiosity_adjustment: f64,
    pub actions_taken: Vec<String>,
    pub cycle_duration_ms: u64,
}

pub struct CrossBindingLoop {
    pub meta_learning: MetaLearning,
    pub cycle_counter: u64,
    pub history: VecDeque<BindingCycleReport>,
    pub max_history: usize,
    pub loop_enabled: bool,
    pub narrative_coherence_history: VecDeque<f64>,
    pub curiosity_history: VecDeque<f64>,
}

pub enum LoopSignal {
    BoostExploration(f64),
    TriggerConsolidation,
    GenerateNarrative,
    AdjustReasoning {
        abstraction: f64,
        scope: f64,
        method: f64,
        depth: f64,
    },
    MetaCrisis,
}

impl CrossBindingLoop {
    pub fn new() -> Self {
        Self {
            meta_learning: MetaLearning::new(),
            cycle_counter: 0,
            history: VecDeque::with_capacity(100),
            max_history: 100,
            loop_enabled: true,
            narrative_coherence_history: VecDeque::with_capacity(100),
            curiosity_history: VecDeque::with_capacity(100),
        }
    }

    pub fn tick(
        &mut self,
        msv: &MetacognitiveState,
        outcome: Option<&ConsolidationOutcome>,
    ) -> BindingCycleReport {
        let msv_before = msv.clone();
        let params_before = self.meta_learning.params.clone();

        if let Some(outcome) = outcome {
            self.meta_learning.ingest_consolidation(outcome.clone());
        }

        let params_after = self.meta_learning.params.clone();

        let narrative_quality = msv.output_quality * (1.0 - msv.conflict_level);

        let mut curiosity_adjustment = 0.0;
        let mut new_curiosity = msv.curiosity_signal;
        if narrative_quality > 0.6 {
            let target_curiosity = 0.3;
            curiosity_adjustment = (target_curiosity - new_curiosity) * 0.2;
            new_curiosity = (new_curiosity + curiosity_adjustment).clamp(0.0, 1.0);
        } else if narrative_quality < 0.3 {
            let target_curiosity = 0.7;
            curiosity_adjustment = (target_curiosity - new_curiosity) * 0.2;
            new_curiosity = (new_curiosity + curiosity_adjustment).clamp(0.0, 1.0);
        }

        let mut msv_after = msv_before.clone();
        msv_after.curiosity_signal = new_curiosity;

        let mut actions_taken = Vec::new();
        if outcome.is_some() {
            actions_taken.push("ingest_consolidation".to_string());
        }
        if curiosity_adjustment.abs() > 0.01 {
            actions_taken.push("adjust_curiosity".to_string());
        }

        self.cycle_counter += 1;

        let report = BindingCycleReport {
            cycle: self.cycle_counter,
            msv_before,
            msv_after: msv_after.clone(),
            meta_params_before: params_before,
            meta_params_after: params_after,
            narrative_quality,
            curiosity_adjustment,
            actions_taken,
            cycle_duration_ms: 0,
        };

        self.narrative_coherence_history
            .push_back(narrative_quality);
        if self.narrative_coherence_history.len() > self.max_history {
            self.narrative_coherence_history.pop_front();
        }

        self.curiosity_history.push_back(msv_after.curiosity_signal);
        if self.curiosity_history.len() > self.max_history {
            self.curiosity_history.pop_front();
        }

        self.history.push_back(report.clone());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        report
    }

    pub fn detect_loop_signals(msv: &MetacognitiveState) -> Vec<LoopSignal> {
        let mut signals = Vec::new();

        if msv.curiosity_signal > 0.6 && msv.prediction_error > 0.3 {
            signals.push(LoopSignal::BoostExploration(0.2));
        }

        if msv.conflict_level > 0.5 {
            signals.push(LoopSignal::MetaCrisis);
        }

        if msv.epistemic_confidence > 0.7 && msv.cognitive_load < 0.3 {
            signals.push(LoopSignal::AdjustReasoning {
                abstraction: 0.8,
                scope: 0.6,
                method: 0.5,
                depth: 0.4,
            });
        }

        if msv.output_quality > 0.6 {
            signals.push(LoopSignal::GenerateNarrative);
        }

        signals
    }

    pub fn narrative_stability(&self) -> f64 {
        let n = self.narrative_coherence_history.len();
        if n < 2 {
            return 1.0;
        }
        let mean: f64 = self.narrative_coherence_history.iter().sum::<f64>() / n as f64;
        let variance: f64 = self
            .narrative_coherence_history
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / n as f64;
        1.0 - variance.sqrt().clamp(0.0, 1.0)
    }

    pub fn curiosity_drive_stability(&self) -> f64 {
        let n = self.curiosity_history.len();
        if n < 2 {
            return 1.0;
        }
        let mean: f64 = self.curiosity_history.iter().sum::<f64>() / n as f64;
        let variance: f64 = self
            .curiosity_history
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / n as f64;
        1.0 - variance.sqrt().clamp(0.0, 1.0)
    }

    pub fn loop_health(&self) -> String {
        let n = self.history.len();
        if n == 0 {
            return "no_cycles".to_string();
        }
        let last = self.history.back().unwrap();
        let avg_narrative: f64 = self
            .history
            .iter()
            .map(|r| r.narrative_quality)
            .sum::<f64>()
            / n as f64;
        let avg_curiosity_adj: f64 = self
            .history
            .iter()
            .map(|r| r.curiosity_adjustment.abs())
            .sum::<f64>()
            / n as f64;
        let nav_stab = self.narrative_stability();
        let cur_stab = self.curiosity_drive_stability();

        let cycle_count = self.cycle_counter;
        let binding = self.binding_strength();

        let mut issues = Vec::new();
        if avg_narrative < 0.3 {
            issues.push("low_narrative_coherence");
        }
        if avg_curiosity_adj > 0.2 {
            issues.push("high_curiosity_volatility");
        }
        if nav_stab < 0.5 {
            issues.push("unstable_narrative");
        }
        if cur_stab < 0.5 {
            issues.push("unstable_curiosity");
        }
        if binding < 0.3 {
            issues.push("weak_binding");
        }
        if last.msv_after.conflict_level > 0.5 {
            issues.push("active_conflict");
        }

        let health = if issues.is_empty() {
            "healthy"
        } else if issues.len() <= 2 {
            "degraded"
        } else {
            "critical"
        };

        format!(
            "health={} cycles={} avg_narrative={:.3} binding={:.3} issues=[{}]",
            health,
            cycle_count,
            avg_narrative,
            binding,
            issues.join(",")
        )
    }

    pub fn binding_strength(&self) -> f64 {
        let n = self.history.len();
        if n < 2 {
            return 0.5;
        }
        let mut narrative_vals: Vec<f64> = Vec::with_capacity(n);
        let mut curiosity_vals: Vec<f64> = Vec::with_capacity(n);

        for report in &self.history {
            narrative_vals.push(report.narrative_quality);
            curiosity_vals.push(report.curiosity_adjustment);
        }

        let n_mean: f64 = narrative_vals.iter().sum::<f64>() / n as f64;
        let c_mean: f64 = curiosity_vals.iter().sum::<f64>() / n as f64;

        let mut n_var = 0.0;
        let mut c_var = 0.0;
        let mut covar = 0.0;
        for i in 0..n {
            let nd = narrative_vals[i] - n_mean;
            let cd = curiosity_vals[i] - c_mean;
            n_var += nd * nd;
            c_var += cd * cd;
            covar += nd * cd;
        }
        n_var = (n_var / n as f64).sqrt();
        c_var = (c_var / n as f64).sqrt();
        covar /= n as f64;

        let denom = n_var * c_var;
        let raw_corr = if denom > 1e-12 { covar / denom } else { 0.0 };

        let recent_sync: f64 = if n >= 5 {
            let recent_n: f64 = narrative_vals[n - 5..].iter().copied().sum::<f64>() / 5.0;
            let recent_c: f64 = curiosity_vals[n - 5..].iter().copied().sum::<f64>() / 5.0;
            1.0 - (recent_n - recent_c).abs()
        } else {
            1.0 - (n_mean - c_mean).abs()
        };

        let correlation_score = (raw_corr.abs() * 0.5 + 0.5).clamp(0.0, 1.0);
        (correlation_score * 0.6 + recent_sync * 0.4).clamp(0.0, 1.0)
    }
}

impl Default for CrossBindingLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_meta::MetacognitiveState;

    fn sample_msv() -> MetacognitiveState {
        MetacognitiveState::new()
    }

    fn sample_outcome() -> ConsolidationOutcome {
        ConsolidationOutcome {
            sequences_replayed: 5,
            patterns_merged: 10,
            abstractions_formed: 2,
            novelty_score: 0.3,
            coherence_gain: 0.4,
            cross_session_consolidated: 1,
            buffer_eviction_count: 2,
            buffer_avg_utility: 0.6,
            hebbian_distillations: 1,
            scm_nrem_merged: 3,
            scm_rem_associations: 2,
            triggered_by_recurrence: false,
            learned_retrieval_rate: 0.5,
            compression: None,
            consolidation_duration_ms: 10,
        }
    }

    #[test]
    fn test_new_loop_empty() {
        let loop_ = CrossBindingLoop::new();
        assert_eq!(loop_.cycle_counter, 0);
        assert!(loop_.history.is_empty());
        assert!(loop_.loop_enabled);
        assert_eq!(loop_.max_history, 100);
    }

    #[test]
    fn test_tick_produces_report() {
        let mut loop_ = CrossBindingLoop::new();
        let msv = sample_msv();
        let report = loop_.tick(&msv, None);
        assert_eq!(report.cycle, 1);
        assert!((report.narrative_quality - 0.5).abs() < 1e-10);
        assert_eq!(loop_.cycle_counter, 1);
        assert_eq!(loop_.history.len(), 1);
    }

    #[test]
    fn test_tick_with_consolidation() {
        let mut loop_ = CrossBindingLoop::new();
        let msv = sample_msv();
        let outcome = sample_outcome();
        let report = loop_.tick(&msv, Some(&outcome));
        assert_eq!(report.cycle, 1);
        assert!(report
            .actions_taken
            .contains(&"ingest_consolidation".to_string()));
        assert_eq!(loop_.meta_learning.cycle, 1);
    }

    #[test]
    fn test_high_curiosity_triggers_exploration_signal() {
        let mut msv = sample_msv();
        msv.curiosity_signal = 0.7;
        msv.prediction_error = 0.5;
        let signals = CrossBindingLoop::detect_loop_signals(&msv);
        let has_exploration = signals
            .iter()
            .any(|s| matches!(s, LoopSignal::BoostExploration(_)));
        assert!(has_exploration);
    }

    #[test]
    fn test_high_conflict_triggers_meta_crisis() {
        let mut msv = sample_msv();
        msv.conflict_level = 0.7;
        let signals = CrossBindingLoop::detect_loop_signals(&msv);
        let has_crisis = signals.iter().any(|s| matches!(s, LoopSignal::MetaCrisis));
        assert!(has_crisis);
    }

    #[test]
    fn test_high_quality_triggers_narrative() {
        let mut msv = sample_msv();
        msv.output_quality = 0.8;
        let signals = CrossBindingLoop::detect_loop_signals(&msv);
        let has_narrative = signals
            .iter()
            .any(|s| matches!(s, LoopSignal::GenerateNarrative));
        assert!(has_narrative);
    }

    #[test]
    fn test_narrative_stability_after_ticks() {
        let mut loop_ = CrossBindingLoop::new();
        let msv = sample_msv();
        for _ in 0..10 {
            loop_.tick(&msv, None);
        }
        let stability = loop_.narrative_stability();
        assert!(stability >= 0.0);
        assert!(stability <= 1.0);
    }

    #[test]
    fn test_curiosity_drive_stability() {
        let mut loop_ = CrossBindingLoop::new();
        let msv = sample_msv();
        for _ in 0..10 {
            loop_.tick(&msv, None);
        }
        let stability = loop_.curiosity_drive_stability();
        assert!(stability >= 0.0);
        assert!(stability <= 1.0);
    }

    #[test]
    fn test_loop_health_format() {
        let mut loop_ = CrossBindingLoop::new();
        let msv = sample_msv();
        for _ in 0..5 {
            loop_.tick(&msv, None);
        }
        let health = loop_.loop_health();
        assert!(health.starts_with("health="));
        assert!(health.contains("cycles="));
        assert!(health.contains("binding="));
    }

    #[test]
    fn test_loop_health_no_cycles() {
        let loop_ = CrossBindingLoop::new();
        assert_eq!(loop_.loop_health(), "no_cycles");
    }

    #[test]
    fn test_binding_strength_returns_value() {
        let mut loop_ = CrossBindingLoop::new();
        let msv = sample_msv();
        for _ in 0..10 {
            loop_.tick(&msv, None);
        }
        let strength = loop_.binding_strength();
        assert!(strength >= 0.0);
        assert!(strength <= 1.0);
    }

    #[test]
    fn test_high_narrative_reduces_curiosity() {
        let mut loop_ = CrossBindingLoop::new();
        let mut msv = sample_msv();
        msv.output_quality = 1.0;
        msv.conflict_level = 0.0;
        let report = loop_.tick(&msv, None);
        assert!(
            report.msv_after.curiosity_signal < msv.curiosity_signal
                || (report.msv_after.curiosity_signal - msv.curiosity_signal).abs() < 0.001
        );
    }

    #[test]
    fn test_detect_loop_signals_adjust_reasoning() {
        let mut msv = sample_msv();
        msv.epistemic_confidence = 0.8;
        msv.cognitive_load = 0.2;
        let signals = CrossBindingLoop::detect_loop_signals(&msv);
        let has_adjust = signals
            .iter()
            .any(|s| matches!(s, LoopSignal::AdjustReasoning { .. }));
        assert!(has_adjust);
    }

    #[test]
    fn test_default_implemented() {
        let loop_ = CrossBindingLoop::default();
        assert_eq!(loop_.cycle_counter, 0);
    }
}

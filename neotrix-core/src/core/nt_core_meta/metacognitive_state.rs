use serde::{Deserialize, Serialize};

use crate::core::nt_core_meta::meta_learning::ConsolidationOutcome;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetacognitiveState {
    pub epistemic_confidence: f64,
    pub cognitive_load: f64,
    pub output_quality: f64,
    pub reasoning_mode_quality: f64,
    pub curiosity_signal: f64,
    pub conflict_level: f64,
    pub prediction_error: f64,
    pub previous_reasoning_quality: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictFlags {
    pub low_confidence_high_conflict: bool,
    pub high_load_low_quality: bool,
    pub high_curiosity_no_explore: bool,
    pub reasoning_degradation: bool,
}

impl MetacognitiveState {
    pub fn new() -> Self {
        Self {
            epistemic_confidence: 0.5,
            cognitive_load: 0.0,
            output_quality: 1.0,
            reasoning_mode_quality: 0.5,
            curiosity_signal: 0.0,
            conflict_level: 0.0,
            prediction_error: 0.0,
            previous_reasoning_quality: 0.5,
        }
    }

    pub fn composite_health(&self) -> f64 {
        (self.epistemic_confidence * 0.25
            + (1.0 - self.cognitive_load) * 0.20
            + self.output_quality * 0.20
            + self.reasoning_mode_quality * 0.15
            + (1.0 - self.conflict_level) * 0.10
            + (1.0 - self.prediction_error) * 0.10)
            .clamp(0.0, 1.0)
    }

    pub fn should_switch_reasoning_mode(&self) -> bool {
        self.reasoning_mode_quality < 0.3 && self.cognitive_load > 0.6
    }

    pub fn should_explore(&self) -> bool {
        self.curiosity_signal > 0.5 && self.cognitive_load < 0.7
    }

    pub fn should_defer(&self) -> bool {
        self.epistemic_confidence < 0.3 && self.conflict_level > 0.5
    }

    pub fn update_from_calibration(&mut self, calibration_error: f64, meta_d: f64) {
        self.epistemic_confidence = (1.0 - calibration_error).clamp(0.0, 1.0);
        let meta_norm = (meta_d / 3.0).clamp(0.0, 1.0);
        self.epistemic_confidence =
            (self.epistemic_confidence * 0.7 + meta_norm * 0.3).clamp(0.0, 1.0);
    }

    pub fn update_from_load(&mut self, avg_load: f64) {
        self.cognitive_load = avg_load.clamp(0.0, 1.0);
    }

    pub fn update_from_critic(&mut self, pass_rate: f64, overall_quality: f64) {
        self.output_quality = (pass_rate * 0.4 + overall_quality * 0.6).clamp(0.0, 1.0);
    }

    pub fn update_reasoning_quality(&mut self, hexagram_effectiveness: f64) {
        self.reasoning_mode_quality = hexagram_effectiveness.clamp(0.0, 1.0);
    }

    pub fn update_curiosity(&mut self, signal: f64) {
        self.curiosity_signal = signal.clamp(0.0, 1.0);
    }

    pub fn update_conflict(&mut self, level: f64) {
        self.conflict_level = level.clamp(0.0, 1.0);
    }

    pub fn update_prediction_error(&mut self, error: f64) {
        self.prediction_error = error.clamp(0.0, 1.0);
    }

    pub fn update_from_consolidation(&mut self, outcome: &ConsolidationOutcome) {
        self.previous_reasoning_quality = self.reasoning_mode_quality;
        self.reasoning_mode_quality =
            (self.reasoning_mode_quality * 0.7 + outcome.coherence_gain * 0.3).clamp(0.0, 1.0);
        self.prediction_error = (1.0 - outcome.novelty_score).clamp(0.0, 1.0);
        self.epistemic_confidence = (self.epistemic_confidence * 0.7
            + outcome.learned_retrieval_rate * 0.3)
            .clamp(0.0, 1.0);
    }

    pub fn detect_conflicts(&self) -> ConflictFlags {
        ConflictFlags {
            low_confidence_high_conflict: self.epistemic_confidence < 0.3
                && self.conflict_level > 0.5,
            high_load_low_quality: self.cognitive_load > 0.7 && self.output_quality < 0.3,
            high_curiosity_no_explore: self.curiosity_signal > 0.6 && self.prediction_error < 0.1,
            reasoning_degradation: self.reasoning_mode_quality
                < self.previous_reasoning_quality * 0.7,
        }
    }

    pub fn meta_awareness_trigger(&self) -> Option<&str> {
        let flags = self.detect_conflicts();
        if flags.low_confidence_high_conflict {
            Some("建议降低任务复杂度并验证已知事实")
        } else if flags.high_load_low_quality {
            Some("建议降低认知负载或切换到更简单的推理模式")
        } else if flags.high_curiosity_no_explore {
            Some("建议主动探索新领域以利用好奇心信号")
        } else if flags.reasoning_degradation {
            Some("建议检查推理模式是否匹配当前任务")
        } else {
            None
        }
    }
}

impl Default for MetacognitiveState {
    fn default() -> Self {
        Self::new()
    }
}

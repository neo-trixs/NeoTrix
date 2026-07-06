//! AntiDistillationStage — SEAL pipeline integration for anti-distillation.
//!
//! Lives in L8 (not L7) because it implements BrainStage and takes SelfIteratingBrain.
//! The underlying algorithms (watermark/tracer/detector/decompose) remain in L7's
//! nt_core_antidistil module.

use crate::core::nt_core_error::NeoTrixError;
use super::SelfIteratingBrain;
use super::pipeline::{BrainStage, StageDecision, BrainSnapshot};

/// SEAL pipeline stage that monitors anti-distillation health and
/// adaptively tunes watermark strength + task decomposition aggression.
pub struct AntiDistillationStage {
    _private: (),
}

impl Default for AntiDistillationStage {
    fn default() -> Self {
        Self::new()
    }
}

impl AntiDistillationStage {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl BrainStage for AntiDistillationStage {
    fn name(&self) -> &str {
        "anti_distillation"
    }

    fn frequency(&self) -> usize {
        5
    }

    fn process(
        &self,
        brain: &mut SelfIteratingBrain,
    ) -> Result<StageDecision, NeoTrixError> {
        let (owned_alerts, stats_summary) = match brain.reasoning_engine.as_ref() {
            Some(engine) => match engine.anti_distillation.as_ref() {
                Some(ads) => {
                    let recent: Vec<_> = ads.detector.recent_alerts(20);
                    let owned: Vec<_> = recent.into_iter().cloned().collect();
                    let s = format!(
                        "calls={} refuse={:.1}% water={:.2} decomplex={:.2} alerts={}",
                        ads.total_calls,
                        ads.refusal_rate() * 100.0,
                        ads.watermark_strength,
                        ads.decomplex_aggression,
                        ads.detector.alert_history.len(),
                    );
                    (owned, s)
                }
                None => (vec![], "anti_distillation: not configured".into()),
            },
            None => (vec![], "anti_distillation: no engine".into()),
        };

        let critical_threat = owned_alerts.iter().any(|a| a.confidence > 0.9);
        let alert_count = owned_alerts.len();

        if let Some(engine) = brain.reasoning_engine.as_mut() {
            if let Some(ads) = engine.anti_distillation.as_mut() {
                ads.adjust_from_alerts(&owned_alerts);

                if alert_count > 3 {
                    ads.rotate_scheme();
                    log::info!("[antidistil] rotated watermark scheme ({} alerts)", alert_count);
                }

                if critical_threat {
                    if let Some(ref mut gwt) = engine.gwt {
                        gwt.broadcast(&format!(
                            "[antidistil] CRITICAL: {} alerts, confidence={:.2} water={:.2}",
                            alert_count,
                            owned_alerts.first().map(|a| a.confidence).unwrap_or(0.0),
                            ads.watermark_strength,
                        ));
                    }
                }

                if let Some(ref kb) = brain._nt_memory_kb {
                    ads.persist_to_kb(kb);
                }
            }
        }

        log::info!("[antidistil] {}", stats_summary);

        if critical_threat {
            if let Some(engine) = brain.reasoning_engine.as_ref() {
                if let Some(ads) = engine.anti_distillation.as_ref() {
                    if ads.watermark_strength > 2.0 {
                        let task_type = brain._current_task_type();
                        let snap = BrainSnapshot::new(&brain.brain, &task_type);
                        log::warn!(
                            "[antidistil] CRITICAL: water={:.2} promoting",
                            ads.watermark_strength,
                        );
                        return Ok(StageDecision::Promote(snap));
                    }
                }
            }
        }

        Ok(StageDecision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anti_distillation_stage_name_and_frequency() {
        let stage = AntiDistillationStage::default();
        assert_eq!(stage.name(), "anti_distillation");
        assert_eq!(stage.frequency(), 5);
    }

    #[test]
    fn test_anti_distillation_stage_process_no_engine() {
        let mut brain = SelfIteratingBrain::new();
        let stage = AntiDistillationStage::new();
        let result = stage.process(&mut brain);
        assert!(result.is_ok());
    }
}

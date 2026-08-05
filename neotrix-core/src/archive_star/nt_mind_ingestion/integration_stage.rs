use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_mind::self_iterating::pipeline::{BrainStage, StageDecision};
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use super::{IngestionCore, SourceType, IngestionResult};

pub struct IngestionStage {
    core: IngestionCore,
    pending_inputs: Vec<(String, SourceType)>,
}

impl IngestionStage {
    pub fn new() -> Self {
        Self {
            core: IngestionCore::default(),
            pending_inputs: Vec::new(),
        }
    }

    pub fn queue(&mut self, input: &str, source_type: Option<SourceType>) {
        let st = source_type.unwrap_or_else(|| IngestionCore::auto_detect_type(input));
        self.pending_inputs.push((input.to_string(), st));
    }

    pub fn core(&self) -> &IngestionCore {
        &self.core
    }
}

impl Default for IngestionStage {
    fn default() -> Self {
        Self::new()
    }
}

impl BrainStage for IngestionStage {
    fn name(&self) -> &str {
        "nt_mind_ingestion"
    }

    fn frequency(&self) -> usize {
        3
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if self.pending_inputs.is_empty() {
            return Ok(StageDecision::Skip("no pending ingestion inputs".into()));
        }

        let mut processed = 0usize;
        let mut broadcasts: Vec<String> = Vec::new();

        for (input, source_type) in &self.pending_inputs {
            let result = self.core.process(input, *source_type);

            if let Some(ref mut router) = brain.attention_router {
                let wm: &mut GlobalWorkspace = router.wm();
                let report = format!(
                    "[ingestion] {} | {} | quality={:.3} | {} rounds | converged={} | entities={}",
                    result.title,
                    result.source_type.name(),
                    result.final_quality,
                    result.total_rounds,
                    result.converged,
                    result.entities.len(),
                );
                wm.broadcast(&report);
                broadcasts.push(report);
            }

            if let Some(ref kb) = brain._nt_memory_kb {
                let summary = format!("{} | steps: {} | quality: {:.3}",
                    result.summary, result.total_rounds, result.final_quality);
                let _ = kb.insert_or_get_node(
                    &result.title,
                    crate::neotrix::nt_memory_kb::NodeType::Insight,
                    Some(&summary),
                    None,
                    Some("ingestion"),
                );
            }

            processed += 1;
        }

        let reward_bonus = (processed as f64 * 0.05).min(0.3);
        brain._set_reward(brain._reward() + reward_bonus);

        log::info!(
            "[ingestion-stage] processed {} inputs, reward bonus: {:.3}, broadcasts: {}",
            processed, reward_bonus, broadcasts.len(),
        );

        Ok(StageDecision::Continue)
    }
}

pub fn auto_ingest(brain: &mut SelfIteratingBrain, input: &str, source_type: Option<SourceType>) -> IngestionResult {
    let core = IngestionCore::default();
    let st = source_type.unwrap_or_else(|| IngestionCore::auto_detect_type(input));
    let result = core.process(input, st);

    if let Some(ref mut router) = brain.attention_router {
        let wm: &mut GlobalWorkspace = router.wm();
        wm.broadcast(&format!(
            "[auto-ingest] {} ({:?}) | quality={:.3}",
            result.title, result.source_type, result.final_quality,
        ));
    }

    result
}

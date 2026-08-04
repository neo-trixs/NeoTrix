use super::pipeline::{BrainStage, StageDecision};
use super::SelfIteratingBrain;
use crate::neotrix::nt_core_error::NeoTrixError;

/// Runtime stage wrapper with recipe-level configuration.
pub struct RecipeStage {
    pub inner: Box<dyn BrainStage>,
    pub frequency_override: Option<usize>,
    pub enabled: bool,
}

impl RecipeStage {
    pub fn new(inner: Box<dyn BrainStage>) -> Self {
        Self { inner, frequency_override: None, enabled: true }
    }

    pub fn with_frequency(mut self, freq: usize) -> Self {
        self.frequency_override = Some(freq);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

impl BrainStage for RecipeStage {
    fn name(&self) -> &str { self.inner.name() }
    fn frequency(&self) -> usize { self.frequency_override.unwrap_or_else(|| self.inner.frequency()) }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if !self.enabled {
            return Ok(StageDecision::Skip("recipe stage disabled".to_string()));
        }
        self.inner.process(brain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_stage_frequency_override() {
        use crate::neotrix::nt_mind::self_iterating::pipeline::SnapshotStage;
        let stage = RecipeStage::new(Box::new(SnapshotStage::new()))
            .with_frequency(42);
        assert_eq!(stage.frequency(), 42);
    }

    #[test]
    fn test_recipe_stage_disabled() {
        use crate::neotrix::nt_mind::self_iterating::pipeline::SnapshotStage;
        let stage = RecipeStage::new(Box::new(SnapshotStage::new()))
            .disabled();
        assert!(!stage.enabled);
    }
}

use super::pipeline::{AutonomyLevel, BrainStage, PermissionLevel, StageDecision};
use super::SelfIteratingBrain;
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::neotrix::nt_world_model::TaskType;

/// A named, parameterized pipeline template.
#[derive(Debug, Clone)]
pub struct RecipeConfig {
    pub name: String,
    pub description: String,
    pub stage_names: Vec<String>,
    pub frequency_overrides: Vec<(String, usize)>,
    pub affinity: Vec<TaskType>,
    pub min_permission: PermissionLevel,
    pub min_autonomy: AutonomyLevel,
    pub priority: usize,
}

impl RecipeConfig {
    pub fn matches_task(&self, task: TaskType) -> bool {
        self.affinity.is_empty() || self.affinity.contains(&task)
    }
}

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

/// A complete pipeline recipe: ordered stages + metadata.
pub struct Recipe {
    pub config: RecipeConfig,
    pub stages: Vec<RecipeStage>,
}

impl Recipe {
    pub fn new(config: RecipeConfig, stages: Vec<RecipeStage>) -> Self {
        Self { config, stages }
    }

    pub fn execute(&self, brain: &mut SelfIteratingBrain) -> Result<(), NeoTrixError> {
        if self.config.min_permission != brain.permission {
            if (brain.permission as i32) < (self.config.min_permission as i32) {
                log::warn!("[recipe] '{}' skipped: permission {:?} < min {:?}",
                    self.config.name, brain.permission, self.config.min_permission);
                return Ok(());
            }
        }
        if self.config.min_autonomy != brain.autonomy {
            if (brain.autonomy as i32) < (self.config.min_autonomy as i32) {
                log::warn!("[recipe] '{}' skipped: autonomy {:?} < min {:?}",
                    self.config.name, brain.autonomy, self.config.min_autonomy);
                return Ok(());
            }
        }

        let iter = brain.iteration;
        for stage in &self.stages {
            if !stage.enabled { continue; }
            let freq = stage.frequency();
            if freq > 1 && !iter.is_multiple_of(freq as u64) { continue; }
            let decision = stage.process(brain)?;
            match decision {
                StageDecision::Continue => {}
                StageDecision::Skip(reason) => {
                    log::trace!("[recipe] stage '{}' skip: {}", stage.name(), reason);
                    return Ok(());
                }
                StageDecision::Promote(champ) => {
                    brain.champion = Some(champ);
                }
                StageDecision::Rollback(reason) => {
                    return Err(NeoTrixError::Brain(format!("[recipe] rollback: {}", reason)));
                }
            }
        }
        Ok(())
    }

    pub fn name(&self) -> &str { &self.config.name }
    pub fn affinity(&self) -> &[TaskType] { &self.config.affinity }
}

/// Registry of available recipes.
pub struct RecipeRegistry {
    recipes: Vec<Recipe>,
}

impl RecipeRegistry {
    pub fn new() -> Self { Self { recipes: Vec::new() } }

    pub fn register(&mut self, recipe: Recipe) { self.recipes.push(recipe); }

    pub fn select(&self, task: TaskType) -> Option<&Recipe> {
        let mut best: Option<&Recipe> = None;
        for r in &self.recipes {
            if r.affinity().contains(&task) {
                match best {
                    None => best = Some(r),
                    Some(current) => {
                        if r.config.priority > current.config.priority {
                            best = Some(r);
                        }
                    }
                }
            }
        }
        best
    }

    pub fn select_index(&self, task: TaskType) -> Option<usize> {
        let mut best_idx: Option<usize> = None;
        let mut best_priority: Option<usize> = None;
        for (i, r) in self.recipes.iter().enumerate() {
            if r.affinity().contains(&task) {
                match best_priority {
                    None => {
                        best_idx = Some(i);
                        best_priority = Some(r.config.priority);
                    }
                    Some(current_pri) => {
                        if r.config.priority > current_pri {
                            best_idx = Some(i);
                            best_priority = Some(r.config.priority);
                        }
                    }
                }
            }
        }
        best_idx
    }

    pub fn all(&self) -> &[Recipe] { &self.recipes }
    pub fn names(&self) -> Vec<String> { self.recipes.iter().map(|r| r.config.name.clone()).collect() }

    pub fn by_name(&self, name: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.config.name == name)
    }
}

/// Build standard preset recipes from the existing seal_pipeline config.
pub fn preset_standard() -> Recipe {
    let stages: Vec<RecipeStage> = vec![
        RecipeStage::new(Box::new(crate::neotrix::nt_mind_ingestion::pipeline_stages::StreamHygieneStage::new())),
        RecipeStage::new(Box::new(super::pipeline::SnapshotStage::new())),
        RecipeStage::new(Box::new(super::pipeline::MemoryRetrievalStage::new())),
        RecipeStage::new(Box::new(super::pipeline::GapAnalysisStage::new())),
        RecipeStage::new(Box::new(super::pipeline::SSMUpdateStage::new())),
        RecipeStage::new(Box::new(super::pipeline::SelfEditGenerationStage::new())),
        RecipeStage::new(Box::new(super::pipeline::ApplyEditsStage::new())),
        RecipeStage::new(Box::new(super::pipeline::RewardCalculationStage::new())),
        RecipeStage::new(Box::new(super::pipeline::GwtAbsorbStage::new())),
        RecipeStage::new(Box::new(super::pipeline::ChampionCompareStage::new())),
        RecipeStage::new(Box::new(super::pipeline::ReasoningBankStorageStage::new())),
        RecipeStage::new(Box::new(super::pipeline::HyperCubeOptimizeStage::new())).with_frequency(10),
        RecipeStage::new(Box::new(super::pipeline::SecurityStage::new())),
        RecipeStage::new(Box::new(super::pipeline::DistillationStage::new())).with_frequency(3),
        RecipeStage::new(Box::new(super::pipeline::MetaImprovementStage::new())).with_frequency(10),
        RecipeStage::new(Box::new(super::pipeline::SleepStage::new())).with_frequency(100),
        RecipeStage::new(Box::new(super::pipeline::UQCalibrationStage::new())).with_frequency(20),
    ];

    Recipe::new(RecipeConfig {
        name: "standard".into(),
        description: "Full cognition pipeline: reasoning, edit, absorb, consolidate".into(),
        stage_names: stages.iter().map(|s| s.name().to_string()).collect(),
        frequency_overrides: vec![],
        affinity: vec![TaskType::General, TaskType::CodeAnalysis, TaskType::CodeGeneration, TaskType::Research],
        min_permission: PermissionLevel::Suggest,
        min_autonomy: AutonomyLevel::Bounded,
        priority: 10,
    }, stages)
}

pub fn preset_kernel() -> Recipe {
    let stages: Vec<RecipeStage> = vec![
        RecipeStage::new(Box::new(super::pipeline::SnapshotStage::new())),
        RecipeStage::new(Box::new(super::pipeline::MemoryRetrievalStage::new())),
        RecipeStage::new(Box::new(super::pipeline::ChampionCompareStage::new())).with_frequency(2),
        RecipeStage::new(Box::new(super::pipeline::HyperCubeOptimizeStage::new())).with_frequency(20),
    ];

    Recipe::new(RecipeConfig {
        name: "kernel".into(),
        description: "Lightweight iteration: snapshot, retrieve, compare, consolidate".into(),
        stage_names: stages.iter().map(|s| s.name().to_string()).collect(),
        frequency_overrides: vec![],
        affinity: vec![TaskType::Reflection, TaskType::Learning],
        min_permission: PermissionLevel::Full,
        min_autonomy: AutonomyLevel::Full,
        priority: 5,
    }, stages)
}

pub fn preset_debug() -> Recipe {
    let stages: Vec<RecipeStage> = vec![
        RecipeStage::new(Box::new(super::pipeline::SnapshotStage::new())),
        RecipeStage::new(Box::new(super::pipeline::MemoryRetrievalStage::new())),
        RecipeStage::new(Box::new(super::pipeline::GapAnalysisStage::new())),
        RecipeStage::new(Box::new(super::pipeline::SelfEditGenerationStage::new())),
        RecipeStage::new(Box::new(super::pipeline::RewardCalculationStage::new())),
        RecipeStage::new(Box::new(super::pipeline::ChampionCompareStage::new())),
    ];

    Recipe::new(RecipeConfig {
        name: "debug".into(),
        description: "Minimal pipeline for debugging: snapshot, debug, fix, verify".into(),
        stage_names: stages.iter().map(|s| s.name().to_string()).collect(),
        frequency_overrides: vec![],
        affinity: vec![TaskType::Debugging],
        min_permission: PermissionLevel::Full,
        min_autonomy: AutonomyLevel::Full,
        priority: 20,
    }, stages)
}

pub fn preset_design() -> Recipe {
    let stages: Vec<RecipeStage> = vec![
        RecipeStage::new(Box::new(super::pipeline::SnapshotStage::new())),
        RecipeStage::new(Box::new(super::pipeline::MemoryRetrievalStage::new())),
        RecipeStage::new(Box::new(super::pipeline::OpenSourceCompareStage::new())).with_frequency(5),
        RecipeStage::new(Box::new(super::pipeline::GapAnalysisStage::new())),
        RecipeStage::new(Box::new(super::pipeline::SelfEditGenerationStage::new())),
        RecipeStage::new(Box::new(super::pipeline::ApplyEditsStage::new())),
        RecipeStage::new(Box::new(super::pipeline::RewardCalculationStage::new())),
        RecipeStage::new(Box::new(super::pipeline::GwtAbsorbStage::new())),
        RecipeStage::new(Box::new(super::pipeline::KnowledgeQualityStage::new())).with_frequency(3),
    ];

    Recipe::new(RecipeConfig {
        name: "design".into(),
        description: "Design-oriented pipeline: benchmark, gap analysis, edit, verify".into(),
        stage_names: stages.iter().map(|s| s.name().to_string()).collect(),
        frequency_overrides: vec![],
        affinity: vec![TaskType::Design, TaskType::UIDesign],
        min_permission: PermissionLevel::Suggest,
        min_autonomy: AutonomyLevel::Bounded,
        priority: 15,
    }, stages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_matches_task() {
        let r = RecipeConfig {
            name: "test".into(),
            description: "".into(),
            stage_names: vec![],
            frequency_overrides: vec![],
            affinity: vec![TaskType::CodeAnalysis, TaskType::Debugging],
            min_permission: PermissionLevel::Full,
            min_autonomy: AutonomyLevel::Full,
            priority: 1,
        };
        assert!(r.matches_task(TaskType::CodeAnalysis));
        assert!(r.matches_task(TaskType::Debugging));
        assert!(!r.matches_task(TaskType::Design));
    }

    #[test]
    fn test_registry_select_by_task() {
        let mut reg = RecipeRegistry::new();
        reg.register(preset_standard());
        reg.register(preset_debug());

        let selected = reg.select(TaskType::Debugging);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name(), "debug");

        let selected = reg.select(TaskType::CodeAnalysis);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name(), "standard");
    }

    #[test]
    fn test_registry_select_fallback() {
        let reg = RecipeRegistry::new();
        let selected = reg.select(TaskType::General);
        assert!(selected.is_none());
    }

    #[test]
    fn test_registry_by_name() {
        let mut reg = RecipeRegistry::new();
        reg.register(preset_kernel());
        assert!(reg.by_name("kernel").is_some());
        assert!(reg.by_name("nonexistent").is_none());
    }

    #[test]
    fn test_priority_respected() {
        let mut reg = RecipeRegistry::new();
        reg.register(preset_kernel());  // priority 5
        reg.register(preset_debug());    // priority 20

        // Debugging matches both kernel (priority 5) and debug (priority 20)
        let selected = reg.select(TaskType::Debugging);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name(), "debug");
    }

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

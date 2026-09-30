//! nt_types — 从 `pipeline.rs` 拆分 (types — snapshot/result/autonomy 核心类型 (行为零变更).)
//! 原文逐行搬运, 仅补可见性/导入, 行为零变更。

use super::super::super::core::CapabilityVector;
use super::super::SelfIteratingBrain;
use crate::l0_substrate::nt_core_error::NeoTrixError;
use crate::l2_perception::nt_world::nt_world_model::TaskType;

pub(crate) fn compute_capability_deltas(brain: &SelfIteratingBrain) -> Vec<(String, f64)> {
    let current = brain.brain.capability.arr().to_vec();
    let snap = brain._snapshot_capability().arr().to_vec();
    current
        .into_iter()
        .zip(snap)
        .enumerate()
        .map(|(i, (cur, snp))| (format!("cap_{}", i), cur - snp))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AutonomyLevel {
    Proposal,
    Bounded,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    Suggest,
    Full,
}

#[derive(Debug, Clone)]
pub struct BrainSnapshot {
    pub capability: CapabilityVector,
    pub learning_rate: f64,
    pub score: f64,
}

impl BrainSnapshot {
    pub fn new(brain: &super::super::brain_core::ReasoningBrain, task_type: &TaskType) -> Self {
        let _ = task_type;
        Self {
            capability: brain.capability.clone(),
            learning_rate: brain.learning_rate,
            score: 0.0,
        }
    }

    pub fn restore(&self, brain: &mut super::super::brain_core::ReasoningBrain) {
        brain.capability = self.capability.clone();
        brain.learning_rate = self.learning_rate;
    }
}

#[derive(Debug)]
pub enum StageDecision {
    Continue,
    Skip(String),
    Promote(BrainSnapshot),
    Rollback(String),
}

pub trait BrainStage: Send + Sync {
    fn name(&self) -> &str;
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError>;
}

pub struct StageResult {
    pub stage_name: String,
    pub efc: f64,
    pub efficiency: f64,
}

impl StageResult {
    pub fn new(stage_name: &str) -> Self {
        Self {
            stage_name: stage_name.to_string(),
            efc: 0.0,
            efficiency: 0.0,
        }
    }
}

#[derive(Default)]
pub struct BrainPipeline {
    pub stages: Vec<Box<dyn BrainStage>>,
}


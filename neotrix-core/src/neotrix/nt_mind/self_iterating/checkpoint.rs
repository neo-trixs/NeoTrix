use std::collections::VecDeque;
use std::time::Instant;

use super::pipeline::{AutonomyLevel, BrainSnapshot, BrainStage, PermissionLevel, StageDecision};
use super::SelfIteratingBrain;
use crate::make_stage;
use crate::neotrix::nt_core_error::NeoTrixError;

#[derive(Debug, Clone)]
pub struct BrainCheckpoint {
    pub id: String,
    pub timestamp: Instant,
    pub iteration: u64,
    pub brain_snapshot: BrainSnapshot,
    pub permission_level: PermissionLevel,
    pub autonomy_level: AutonomyLevel,
    pub reward: f64,
    pub stage_name: String,
}

pub struct CheckpointManager {
    checkpoints: VecDeque<BrainCheckpoint>,
    max_checkpoints: usize,
    next_id: u64,
}

impl CheckpointManager {
    pub fn new() -> Self {
        Self {
            checkpoints: VecDeque::with_capacity(5),
            max_checkpoints: 5,
            next_id: 0,
        }
    }

    pub fn with_max(max: usize) -> Self {
        Self {
            checkpoints: VecDeque::with_capacity(max),
            max_checkpoints: max,
            next_id: 0,
        }
    }

    pub fn push(
        &mut self,
        iteration: u64,
        snapshot: &BrainSnapshot,
        permission: PermissionLevel,
        autonomy: AutonomyLevel,
        reward: f64,
        stage_name: &str,
    ) {
        if self.checkpoints.len() >= self.max_checkpoints {
            self.checkpoints.pop_front();
        }
        self.checkpoints.push_back(BrainCheckpoint {
            id: format!("cp_{:04}", self.next_id),
            timestamp: Instant::now(),
            iteration,
            brain_snapshot: snapshot.clone(),
            permission_level: permission,
            autonomy_level: autonomy,
            reward,
            stage_name: stage_name.to_string(),
        });
        self.next_id += 1;
    }

    pub fn get_checkpoint(&self, id: &str) -> Option<BrainCheckpoint> {
        self.checkpoints.iter().find(|cp| cp.id == id).cloned()
    }

    pub fn restore(
        &self,
        brain: &mut super::brain_impl::ReasoningBrain,
        permission: &mut PermissionLevel,
        autonomy: &mut AutonomyLevel,
        reward: &mut f64,
        id: &str,
    ) -> Result<(), NeoTrixError> {
        let cp = self
            .checkpoints
            .iter()
            .find(|cp| cp.id == id)
            .ok_or_else(|| NeoTrixError::General {
                msg: format!("Checkpoint '{}' not found", id),
                backtrace: None,
            })?;
        cp.brain_snapshot.restore(brain);
        *permission = cp.permission_level;
        *autonomy = cp.autonomy_level;
        *reward = cp.reward;
        Ok(())
    }

    pub fn list(&self) -> &VecDeque<BrainCheckpoint> {
        &self.checkpoints
    }

    pub fn max_checkpoints(&self) -> usize {
        self.max_checkpoints
    }

    pub fn len(&self) -> usize {
        self.checkpoints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.checkpoints.is_empty()
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckpointManager {
    /// Convenience: push from a SelfIteratingBrain reference,
    /// creating the BrainSnapshot internally.
    /// Caller must have already released any conflicting borrows.
    pub fn push_from_brain(
        &mut self,
        iteration: u64,
        snapshot: &BrainSnapshot,
        permission: PermissionLevel,
        autonomy: AutonomyLevel,
        reward: f64,
        stage_name: &str,
    ) {
        self.push(iteration, snapshot, permission, autonomy, reward, stage_name);
    }

    /// Bridge from a StageCheckpoint (stage_contracts) into a BrainCheckpoint.
    /// Creates a BrainCheckpoint and pushes it into the manager.
    pub fn push_from_stage_checkpoint(
        &mut self,
        sc: &super::stage_contracts::StageCheckpoint,
        snapshot: &BrainSnapshot,
        permission: PermissionLevel,
        autonomy: AutonomyLevel,
        stage_name: &str,
    ) {
        self.push(sc.iteration, snapshot, permission, autonomy, sc.reward, stage_name);
    }

    /// Find the checkpoint with the highest reward.
    pub fn best_checkpoint(&self) -> Option<BrainCheckpoint> {
        self.checkpoints
            .iter()
            .max_by(|a, b| {
                a.reward
                    .partial_cmp(&b.reward)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }
}

make_stage!(CheckpointStage);
impl BrainStage for CheckpointStage {
    fn name(&self) -> &str {
        "checkpoint"
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task_type = brain._current_task_type();
        let snap = BrainSnapshot::new(&brain.brain, &task_type);
        let iteration = brain.iteration;
        let permission = brain.permission;
        let autonomy = brain.autonomy;
        let reward = brain._reward;
        brain
            .checkpoint_manager
            .push(iteration, &snap, permission, autonomy, reward, "checkpoint");
        Ok(StageDecision::Continue)
    }
}

make_stage!(RewindStage);
impl BrainStage for RewindStage {
    fn name(&self) -> &str {
        "rewind"
    }

    fn frequency(&self) -> usize {
        50
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain._reward < -0.3 {
            let best = {
                let mgr = &brain.checkpoint_manager;
                mgr.best_checkpoint()
            };
            if let Some(cp) = best {
                cp.brain_snapshot.restore(&mut brain.brain);
                brain.permission = cp.permission_level;
                brain.autonomy = cp.autonomy_level;
                brain._reward = cp.reward;
                log::info!(
                    "[rewind] restored to checkpoint {} (reward={:.4})",
                    cp.id,
                    cp.reward
                );
            }
        }
        Ok(StageDecision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

    fn make_snapshot(brain: &SelfIteratingBrain) -> BrainSnapshot {
        BrainSnapshot::new(&brain.brain, &brain._current_task_type())
    }

    #[test]
    fn test_checkpoint_push_and_list() {
        let mut brain = SelfIteratingBrain::new();
        let snap = make_snapshot(&brain);
        brain
            .checkpoint_manager
            .push(0, &snap, brain.permission, brain.autonomy, 0.5, "test");
        assert_eq!(brain.checkpoint_manager.len(), 1);
        assert!(!brain.checkpoint_manager.is_empty());
        let cp = &brain.checkpoint_manager.list()[0];
        assert_eq!(cp.iteration, 0);
        assert_eq!(cp.reward, 0.5);
        assert_eq!(cp.stage_name, "test");
    }

    #[test]
    fn test_checkpoint_ring_buffer() {
        let mut mgr = CheckpointManager::with_max(3);
        for i in 0..5 {
            let snap = BrainSnapshot {
                capability: Default::default(),
                learning_rate: 0.1,
                score: i as f64,
            };
            mgr.push(
                i as u64,
                &snap,
                PermissionLevel::Full,
                AutonomyLevel::Full,
                i as f64,
                "ring_test",
            );
        }
        assert_eq!(mgr.len(), 3);
        assert_eq!(mgr.list()[0].iteration, 2);
        assert_eq!(mgr.list()[2].iteration, 4);
    }

    #[test]
    fn test_checkpoint_get_and_restore() {
        let mut mgr = CheckpointManager::new();
        let snap = BrainSnapshot {
            capability: Default::default(),
            learning_rate: 0.42,
            score: 0.95,
        };
        mgr.push(
            1,
            &snap,
            PermissionLevel::Suggest,
            AutonomyLevel::Bounded,
            0.8,
            "test_restore",
        );

        let cp = mgr.get_checkpoint("cp_0000");
        assert!(cp.is_some());
        let cp = cp.unwrap();
        assert_eq!(cp.iteration, 1);
        assert_eq!(cp.brain_snapshot.learning_rate, 0.42);
        assert_eq!(cp.brain_snapshot.score, 0.95);
        assert_eq!(cp.permission_level, PermissionLevel::Suggest);
        assert_eq!(cp.autonomy_level, AutonomyLevel::Bounded);
        assert_eq!(cp.reward, 0.8);

        let not_found = mgr.get_checkpoint("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_checkpoint_stage_execution() {
        let mut brain = SelfIteratingBrain::new();
        let stage = CheckpointStage::new();
        let result = stage.process(&mut brain);
        assert!(result.is_ok());
        assert_eq!(
            brain.checkpoint_manager.len(),
            1,
            "CheckpointStage should create a checkpoint"
        );
        let cp = &brain.checkpoint_manager.list()[0];
        assert_eq!(cp.stage_name, "checkpoint");
    }

    #[test]
    fn test_rewind_stage_low_reward() {
        let mut brain = SelfIteratingBrain::new();
        // Set a high checkpoint first
        let snap = make_snapshot(&brain);
        brain._reward = 0.9;
        brain
            .checkpoint_manager
            .push(0, &snap, brain.permission, brain.autonomy, 0.9, "good");

        // Now set low reward and trigger rewind
        brain._reward = -0.5;
        let stage = RewindStage::new();
        let result = stage.process(&mut brain);
        assert!(result.is_ok());
        assert!(
            brain._reward >= 0.0,
            "RewindStage should restore reward to checkpoint value"
        );
    }

    #[test]
    fn test_best_checkpoint() {
        let mut mgr = CheckpointManager::new();
        for i in 0..4 {
            let snap = BrainSnapshot {
                capability: Default::default(),
                learning_rate: 0.1,
                score: i as f64 * 0.2,
            };
            mgr.push(
                i as u64,
                &snap,
                PermissionLevel::Full,
                AutonomyLevel::Full,
                i as f64 * 0.5,
                "best_test",
            );
        }
        let best = mgr.best_checkpoint().unwrap();
        assert_eq!(best.reward, 1.5);
        assert_eq!(best.iteration, 3);
    }
}

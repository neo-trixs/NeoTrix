use std::collections::VecDeque;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::pipeline::{AutonomyLevel, BrainSnapshot, BrainStage, PermissionLevel, StageDecision};
use super::SelfIteratingBrain;
use crate::core::CapabilityVector;
use crate::make_stage;
use crate::neotrix::nt_core_error::NeoTrixError;

/// ScienceFlow 持久化 re-anchor 桥接 (absorbed 2026-08-19, P3):
/// 内存环形 checkpoint (CheckpointManager) 之外, 把最高奖励锚点序列化写入
/// KB nt_core_state (`seal_checkpoint`)。进程重启后 build_full re-anchor:
/// 从上次锚点恢复 iteration/reward/brain capability, 而非零冷启动。
/// `BrainCheckpoint` 含 `Instant` 不可序列化, 故用此轻量 DTO 落盘。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedCheckpoint {
    pub iteration: u64,
    pub reward: f64,
    pub learning_rate: f64,
    pub score: f64,
    pub capability: CapabilityVector,
    pub permission: String,
    pub autonomy: String,
}

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
            .ok_or_else(|| NeoTrixError::Brain(format!("Checkpoint '{}' not found", id)))?;
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

/// ScienceFlow 持久化 re-anchor 桥接 (absorbed 2026-08-19, P3):
/// 内存环形 checkpoint 之外, 最新 checkpoint 序列化落盘 KB nt_core_state
/// (`seal_checkpoint`), 进程重启后可 re-anchor 恢复迭代/奖励/能力向量。
/// R-P79: 生产接线 → CheckpointStage::process 写, SelfIteratingBrain::re_anchor 读。
impl CheckpointManager {
    /// 把当前最高奖励 checkpoint 持久化到 KB (R-P79 生产路径直写)。
    pub fn persist_latest_to_kb(&self) -> Result<(), NeoTrixError> {
        self.persist_to_conn(None)
    }

    /// 注入连接变体 (测试用内存 conn, 避免污染生产 KB 全局连接)。
    pub fn persist_to_conn(&self, conn: Option<&rusqlite::Connection>) -> Result<(), NeoTrixError> {
        let cp = self
            .best_checkpoint()
            .ok_or_else(|| NeoTrixError::Brain("no checkpoint to persist".to_string()))?;
        let persisted = PersistedCheckpoint {
            iteration: cp.iteration,
            reward: cp.reward,
            learning_rate: cp.brain_snapshot.learning_rate,
            score: cp.brain_snapshot.score,
            capability: cp.brain_snapshot.capability.clone(),
            permission: format!("{:?}", cp.permission_level),
            autonomy: format!("{:?}", cp.autonomy_level),
        };
        let json = serde_json::to_string_pretty(&persisted)
            .map_err(|e| NeoTrixError::Serde(format!("checkpoint 序列化失败: {e}")))?;
        match conn {
            Some(c) => crate::core::nt_core_state::save_with(c, "seal_checkpoint", &json),
            None => crate::core::nt_core_state::save("seal_checkpoint", &json),
        }
        .map_err(NeoTrixError::Io)
    }

    /// 从 KB 加载持久化 checkpoint (re-anchor 锚点)。无则 None (首次运行)。
    pub fn load_from_kb() -> Option<PersistedCheckpoint> {
        Self::load_from_conn(None)
    }

    /// 注入连接变体。
    pub fn load_from_conn(conn: Option<&rusqlite::Connection>) -> Option<PersistedCheckpoint> {
        let json = match conn {
            Some(c) => crate::core::nt_core_state::load_with(c, "seal_checkpoint"),
            None => crate::core::nt_core_state::load("seal_checkpoint"),
        }?;
        serde_json::from_str(&json).ok()
    }

    /// 清除持久化 checkpoint (翻转期/测试清理)。
    pub fn clear_kb_persisted() -> Result<bool, String> {
        Self::clear_conn(None)
    }

    /// 注入连接变体。
    pub fn clear_conn(conn: Option<&rusqlite::Connection>) -> Result<bool, String> {
        match conn {
            Some(c) => crate::core::nt_core_state::delete_with(c, "seal_checkpoint"),
            None => crate::core::nt_core_state::delete("seal_checkpoint"),
        }
    }
}

impl SelfIteratingBrain {
    /// ScienceFlow re-anchor (absorbed 2026-08-19, P3): 进程重启后从 KB
    /// 恢复持久化 checkpoint — iteration/reward/brain capability/learning_rate。
    /// 与 ESTRA 的 "continue vs redirect" 对应: 恢复到最高奖励锚点继续进化,
    /// 而非每次零冷启动。skip_kb_io (单元测试) 时跳过, 避免污染生产 KB。
    pub fn re_anchor_from_kb(&mut self) {
        self.re_anchor_from_conn(None);
    }

    /// 注入连接变体 (测试用内存 conn)。
    pub fn re_anchor_from_conn(&mut self, conn: Option<&rusqlite::Connection>) {
        if self.skip_kb_io {
            return;
        }
        let Some(cp) = CheckpointManager::load_from_conn(conn) else {
            return;
        };
        // ESTRA re-anchoring 全量 (absorbed 2026-08-19, P6): 决策矩阵
        // 选择 continue (内存轨迹优于/持平 KB 锚点) 或 redirect (回退锚点)。
        // 内存内已有 checkpoints (进程内多轮) → 与锚点奖励对比;
        // 空环 (冷启动) → 无条件 re-anchor (P3 语义不变)。
        let decision = self._anchor_decision(&cp);
        self._last_anchor_decision = Some(decision.clone());
        if decision == AnchorDecision::Continue {
            log::info!(
                "[re-anchor] CONTINUE iter={} reward={:.4} (内存轨迹优于锚点, 保持)",
                cp.iteration,
                cp.reward
            );
            return;
        }
        self.iteration = cp.iteration;
        self._reward = cp.reward;
        self.brain.capability = cp.capability.clone();
        self.brain.learning_rate = cp.learning_rate;
        self.permission = match cp.permission.as_str() {
            "Full" => PermissionLevel::Full,
            _ => PermissionLevel::Suggest,
        };
        self.autonomy = match cp.autonomy.as_str() {
            "Full" => AutonomyLevel::Full,
            "Bounded" => AutonomyLevel::Bounded,
            _ => AutonomyLevel::Proposal,
        };
        log::info!(
            "[re-anchor] REDIRECT restored iter={} reward={:.4} lr={:.3}",
            cp.iteration,
            cp.reward,
            cp.learning_rate
        );
    }

    /// ESTRA continue/redirect 决策矩阵 (absorbed 2026-08-19, P6):
    /// - 内存 checkpoint 环为空 → 冷启动, 必须 re-anchor (Redirect)。
    /// - 内存环最近奖励 >= KB 锚点奖励 - ε → 当前轨迹不劣于锚点,
    ///   继续 (Continue) — 避免破坏进程内已跑出的更优状态。
    /// - 内存环最近奖励 < 锚点 → 轨迹退化, 回退到锚点 (Redirect)。
    fn _anchor_decision(&self, cp: &PersistedCheckpoint) -> AnchorDecision {
        const EPS: f64 = 1e-9;
        if self._checkpoint_manager.is_empty() {
            return AnchorDecision::Redirect;
        }
        let latest = self
            ._checkpoint_manager
            .list()
            .back()
            .map(|c| c.reward)
            .unwrap_or(f64::NEG_INFINITY);
        if latest + EPS >= cp.reward {
            AnchorDecision::Continue
        } else {
            AnchorDecision::Redirect
        }
    }

    /// 最近一次 re-anchor 的决策 (Continue/Redirect), 供测试与诊断。
    pub fn last_anchor_decision(&self) -> Option<AnchorDecision> {
        self._last_anchor_decision
    }
}

/// ESTRA continue-vs-redirect 决策 (absorbed 2026-08-19, P6)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorDecision {
    /// 当前内存轨迹不劣于 KB 锚点 → 继续进化, 不覆盖状态。
    Continue,
    /// 冷启动或轨迹退化 → 回退到 KB 持久化锚点。
    Redirect,
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
            ._checkpoint_manager
            .push(iteration, &snap, permission, autonomy, reward, "checkpoint");
        // ScienceFlow 持久化 re-anchor 接线 (absorbed 2026-08-19, P3, R-P79):
        // 每轮 checkpoint 落盘 KB, 进程重启后 build_full re-anchor 恢复。
        // 仅在非 skip_kb_io (生产路径) 时写, 单元测试不污染 ~/.neotrix。
        if !brain.skip_kb_io {
            if let Err(e) = brain._checkpoint_manager.persist_latest_to_kb() {
                log::warn!("[checkpoint] KB persist failed: {e}");
            }
        }
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
                let mgr = &brain._checkpoint_manager;
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
            ._checkpoint_manager
            .push(0, &snap, brain.permission, brain.autonomy, 0.5, "test");
        assert_eq!(brain._checkpoint_manager.len(), 1);
        assert!(!brain._checkpoint_manager.is_empty());
        let cp = &brain._checkpoint_manager.list()[0];
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
            brain._checkpoint_manager.len(),
            1,
            "CheckpointStage should create a checkpoint"
        );
        let cp = &brain._checkpoint_manager.list()[0];
        assert_eq!(cp.stage_name, "checkpoint");
    }

    #[test]
    fn test_rewind_stage_low_reward() {
        let mut brain = SelfIteratingBrain::new();
        // Set a high checkpoint first
        let snap = make_snapshot(&brain);
        brain._reward = 0.9;
        brain
            ._checkpoint_manager
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

    fn mem_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::core::nt_core_kb_primitives::schema_initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_persist_roundtrip_snapshot_fields() {
        let mut mgr = CheckpointManager::with_max(4);
        let snap = BrainSnapshot {
            capability: Default::default(),
            learning_rate: 0.42,
            score: 0.95,
        };
        mgr.push(
            7,
            &snap,
            PermissionLevel::Suggest,
            AutonomyLevel::Bounded,
            0.8,
            "persist_test",
        );
        let conn = mem_conn();
        mgr.persist_to_conn(Some(&conn)).ok();
        let loaded = CheckpointManager::load_from_conn(Some(&conn)).expect("roundtrip should load");
        assert_eq!(loaded.iteration, 7);
        assert!((loaded.reward - 0.8).abs() < 1e-9);
        assert!((loaded.learning_rate - 0.42).abs() < 1e-9);
        assert_eq!(loaded.permission, "Suggest");
        assert_eq!(loaded.autonomy, "Bounded");
    }

    #[test]
    fn test_re_anchor_restores_state() {
        let mut mgr = CheckpointManager::new();
        let snap = BrainSnapshot {
            capability: Default::default(),
            learning_rate: 0.33,
            score: 0.5,
        };
        mgr.push(
            11,
            &snap,
            PermissionLevel::Full,
            AutonomyLevel::Full,
            0.6,
            "anchor_test",
        );
        let conn = mem_conn();
        mgr.persist_to_conn(Some(&conn)).ok();

        // 全新 brain, skip_kb_io=false 模拟生产重启 → re-anchor 应恢复
        let mut fresh = SelfIteratingBrain::new_lightweight();
        assert_eq!(fresh.skip_kb_io, true, "lightweight 默认跳过 re-anchor");
        fresh.re_anchor_from_conn(Some(&conn)); // skip_kb_io=true → 无副作用
        assert_eq!(fresh.iteration, 0);

        // 强制以生产语义 re-anchor (仅测试: 直接翻转 skip_kb_io 走恢复路径)
        fresh.skip_kb_io = false;
        fresh.re_anchor_from_conn(Some(&conn));
        assert_eq!(fresh.iteration, 11);
        assert!((fresh._reward - 0.6).abs() < 1e-9);
        assert!((fresh.brain.learning_rate - 0.33).abs() < 1e-9);
        assert_eq!(fresh.permission, PermissionLevel::Full);
        assert_eq!(fresh.autonomy, AutonomyLevel::Full);
    }

    #[test]
    fn test_load_absent_returns_none() {
        let conn = mem_conn();
        assert!(CheckpointManager::load_from_conn(Some(&conn)).is_none(), "无持久化时应 None");
    }

    // ── P6 ESTRA continue/redirect 决策矩阵 ─────────────────────
    #[test]
    fn test_anchor_decision_cold_start_redirects() {
        // 内存环为空 (冷启动) + KB 有锚点 → 必须 Redirect。
        let mut mgr = CheckpointManager::new();
        let snap = BrainSnapshot {
            capability: Default::default(),
            learning_rate: 0.1,
            score: 0.5,
        };
        mgr.push(3, &snap, PermissionLevel::Full, AutonomyLevel::Full, 0.9, "cp");
        let conn = mem_conn();
        mgr.persist_to_conn(Some(&conn)).ok();

        let mut fresh = SelfIteratingBrain::new_lightweight();
        fresh.skip_kb_io = false;
        fresh.re_anchor_from_conn(Some(&conn));
        assert_eq!(
            fresh.last_anchor_decision(),
            Some(AnchorDecision::Redirect),
            "冷启动应回退锚点"
        );
        assert_eq!(fresh.iteration, 3);
        assert!((fresh._reward - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_anchor_decision_continue_when_memory_newer() {
        // 内存环已有更优 checkpoint (进程内多轮) → 不覆盖 (Continue)。
        let mut mgr = CheckpointManager::new();
        let snap = BrainSnapshot {
            capability: Default::default(),
            learning_rate: 0.1,
            score: 0.5,
        };
        mgr.push(5, &snap, PermissionLevel::Full, AutonomyLevel::Full, 0.5, "cp");
        let conn = mem_conn();
        mgr.persist_to_conn(Some(&conn)).ok();

        let mut fresh = SelfIteratingBrain::new_lightweight();
        fresh.skip_kb_io = false;
        // 内存环推入奖励更高 (0.9 > 0.5) 的 checkpoint → 决策 Continue。
        let better = BrainSnapshot {
            capability: Default::default(),
            learning_rate: 0.2,
            score: 0.8,
        };
        fresh._checkpoint_manager.push(
            6,
            &better,
            PermissionLevel::Full,
            AutonomyLevel::Full,
            0.9,
            "memory",
        );
        fresh.re_anchor_from_conn(Some(&conn));
        assert_eq!(
            fresh.last_anchor_decision(),
            Some(AnchorDecision::Continue),
            "内存轨迹更优应 Continue"
        );
        assert_eq!(fresh.iteration, 0, "Continue 不应覆盖内存状态");
    }

    #[test]
    fn test_anchor_decision_redirect_when_memory_worse() {
        // 内存环奖励低于 KB 锚点 → 轨迹退化, Redirect 回退锚点。
        let mut mgr = CheckpointManager::new();
        let snap = BrainSnapshot {
            capability: Default::default(),
            learning_rate: 0.1,
            score: 0.5,
        };
        mgr.push(7, &snap, PermissionLevel::Full, AutonomyLevel::Full, 0.9, "cp");
        let conn = mem_conn();
        mgr.persist_to_conn(Some(&conn)).ok();

        let mut fresh = SelfIteratingBrain::new_lightweight();
        fresh.skip_kb_io = false;
        let worse = BrainSnapshot {
            capability: Default::default(),
            learning_rate: 0.2,
            score: 0.4,
        };
        fresh._checkpoint_manager.push(
            8,
            &worse,
            PermissionLevel::Full,
            AutonomyLevel::Full,
            0.2,
            "memory",
        );
        fresh.re_anchor_from_conn(Some(&conn));
        assert_eq!(
            fresh.last_anchor_decision(),
            Some(AnchorDecision::Redirect),
            "内存退化应 Redirect"
        );
        assert_eq!(fresh.iteration, 7, "应回退到 KB 锚点迭代号");
        assert!((fresh._reward - 0.9).abs() < 1e-9);
    }
}

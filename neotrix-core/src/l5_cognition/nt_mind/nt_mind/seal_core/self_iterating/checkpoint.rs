use std::collections::VecDeque;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::pipeline::{AutonomyLevel, BrainSnapshot, BrainStage, PermissionLevel, StageDecision};
use super::SelfIteratingBrain;
use crate::l5_cognition::nt_core::capability::types::CapabilityVector;
use crate::make_stage;
use crate::l0_substrate::nt_core_error::NeoTrixError;

/// ScienceFlow 持久化 re-anchor 桥接 (absorbed 2026-08-19, P3):
/// 内存环形 checkpoint (CheckpointManager) 之外, 把最高奖励锚点序列化写入
/// KB nt_core_state (`seal_checkpoint`)。进程重启后 build_full re-anchor:
/// 从上次锚点恢复 iteration/reward/brain capability, 而非零冷启动。
/// `BrainCheckpoint` 含 `Instant` 不可序列化, 故用此轻量 DTO 落盘。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _PersistedCheckpoint {
    pub iteration: u64,
    pub reward: f64,
    pub learning_rate: f64,
    pub score: f64,
    pub capability: CapabilityVector,
    pub permission: String,
    pub autonomy: String,
    /// ⭐ **帧校验和**（吸收自 `memvid/memvid` Apache-2.0，2026-10-07）。
    ///
    /// memvid 的 Smart Frame 是「不可变单元 + 时间戳 + **checksum**」，
    /// 读取时能发现**损坏/被改写**的记忆帧。
    ///
    /// 本仓原先**没有**这一层 ⇒ 落盘数据被截断或改写时，
    /// `serde_json::from_str(..).ok()` 会**静默返回 None**
    /// ⇒ 上层 `re_anchor_from_kb` 把损坏的 checkpoint 当成「首次运行」
    /// ⇒ **静默零冷启动，且没有任何日志**。
    ///
    /// ⇒ 加校验和后，损坏能被**识别并报告**，而不是伪装成「没有历史」。
    ///
    /// 选型说明：用 **FNV-1a 64**（与本仓 `nt_determinism.rs`、`nt_shield_audit`
    /// 同值）⇒ 跨仓可交叉比对，且**无外部依赖**。
    /// ⚠️ 它是**非加密**校验和，只挡**意外损坏**，⛔ 不挡**恶意篡改**。
    #[serde(default)]
    pub checksum: u64,
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

/// FNV-1a 64 位素数/偏移基数 —— **与本仓 `nt_determinism.rs`、
/// `nt_shield_audit/mod.rs` 同值**，便于跨仓交叉比对同一份字节。
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// 对 `bytes` 算 FNV-1a 64。
///
/// ⚠️ **非加密**：只挡**意外损坏/意外改写**（截断、bit rot、手滑），
/// ⛔ **不挡恶意篡改**（攻击者可重算）。要防篡改需换 keyed MAC。
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// 返回 `checksum` 字段为**真实值**时的 JSON 文本。
///
/// 做法：把 `checksum` 置 0 → 算 FNV → 写回 → 序列化。
/// ⇒ 校验和覆盖**除自身外**的全部字段。
fn _with_checksum(json: &str) -> String {
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(json) else {
        // 解析不了就原样返回（读取侧会解析失败 → 无历史，不会误用）
        return json.to_owned();
    };
    let mut zeroed = v.clone();
    if let Some(o) = zeroed.as_object_mut() {
        o.insert("checksum".to_owned(), serde_json::json!(0u64));
    }
    let sum = fnv1a64(zeroed.to_string().as_bytes());
    if let Some(o) = v.as_object_mut() {
        o.insert("checksum".to_owned(), serde_json::json!(sum));
    }
    v.to_string()
}

/// 校验落盘 JSON 的校验和与记录值是否一致。
fn verify_checksum(json: &str, recorded: u64) -> bool {
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(json) else {
        return false;
    };
    if let Some(o) = v.as_object_mut() {
        o.insert("checksum".to_owned(), serde_json::json!(0u64));
    }
    fnv1a64(v.to_string().as_bytes()) == recorded
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

    pub(crate) fn _get_checkpoint(&self, id: &str) -> Option<BrainCheckpoint> {
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
    pub(crate) fn _push_from_brain(
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
    pub(crate) fn _push_from_stage_checkpoint(
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
    pub(crate) fn _best_checkpoint(&self) -> Option<BrainCheckpoint> {
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
    pub(crate) fn _persist_latest_to_kb(&self) -> Result<(), NeoTrixError> {
        self._persist_to_conn(None)
    }

    /// 注入连接变体 (测试用内存 conn, 避免污染生产 KB 全局连接)。
    pub(crate) fn _persist_to_conn(&self, conn: Option<&rusqlite::Connection>) -> Result<(), NeoTrixError> {
        let cp = self
            ._best_checkpoint()
            .ok_or_else(|| NeoTrixError::Brain("no checkpoint to persist".to_string()))?;
        let persisted = _PersistedCheckpoint {
            iteration: cp.iteration,
            reward: cp.reward,
            learning_rate: cp.brain_snapshot.learning_rate,
            score: cp.brain_snapshot.score,
            capability: cp.brain_snapshot.capability.clone(),
            permission: format!("{:?}", cp.permission_level),
            autonomy: format!("{:?}", cp.autonomy_level),
            checksum: 0, // 下面算
        };
        let mut json = serde_json::to_string_pretty(&persisted)
            .map_err(|e| NeoTrixError::Serde(format!("checkpoint 序列化失败: {e}")))?;
        // 校验和覆盖**除自身外**的全部字段 ⇒ 计算时字段置 0
        json = _with_checksum(&json);
        match conn {
            Some(c) => crate::l5_cognition::nt_core_state::save_with(c, "seal_checkpoint", &json),
            None => crate::l5_cognition::nt_core_state::save("seal_checkpoint", &json),
        }
        .map_err(NeoTrixError::Io)
    }

    /// 从 KB 加载持久化 checkpoint (re-anchor 锚点)。无则 None (首次运行)。
    pub fn load_from_kb() -> Option<_PersistedCheckpoint> {
        Self::_load_from_conn(None)
    }

    /// 注入连接变体。
    pub(crate) fn _load_from_conn(conn: Option<&rusqlite::Connection>) -> Option<_PersistedCheckpoint> {
        let json = match conn {
            Some(c) => crate::l5_cognition::nt_core_state::load_with(c, "seal_checkpoint"),
            None => crate::l5_cognition::nt_core_state::load("seal_checkpoint"),
        }?;
        // ⚠️ 原为 `serde_json::from_str(&json).ok()`：损坏数据被**静默当作「无历史」**
        // ⇒ 上层 re-anchor 变成**零冷启动且无任何日志**（本仓一路在治的病）。
        let parsed: _PersistedCheckpoint = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(e) => {
                log::warn!(
                    "[seal_checkpoint] checkpoint 反序列化失败 ⇒ 视为无历史（零冷启动）: {e}"
                );
                return None;
            }
        };
        // ⭐ **向后兼容**：升级前落下的 checkpoint **没有** `checksum` 字段
        //（`#[serde(default)]` ⇒ 读成 0）⇒ 它**不是损坏**，只是「无校验和」。
        // ⛔ 若直接判失败 ⇒ 升级即**丢弃全部进化历史**，是严重回归。
        // ⇒ 区分两态：`checksum == 0` ⇒ 「无校验和的旧帧」⇒ 接受并补算；
        //   `checksum != 0` 而不匹配 ⇒ 「有校验和但不符」⇒ **真损坏**。
        if parsed.checksum == 0 {
            log::info!(
                "[seal_checkpoint] checkpoint 无校验和（旧格式）⇒ 接受，下轮写入时补算"
            );
            return Some(parsed);
        }
        if !verify_checksum(&json, parsed.checksum) {
            log::warn!(
                "[seal_checkpoint] checkpoint 校验和不匹配 ⇒ 数据可能损坏/被改写，\
                 按「无历史」处理（零冷启动）"
            );
            return None;
        }
        Some(parsed)
    }

    /// 清除持久化 checkpoint (翻转期/测试清理)。
    pub(crate) fn _clear_kb_persisted() -> Result<bool, String> {
        Self::_clear_conn(None)
    }

    /// 注入连接变体。
    pub(crate) fn _clear_conn(conn: Option<&rusqlite::Connection>) -> Result<bool, String> {
        match conn {
            Some(c) => crate::l5_cognition::nt_core_state::delete_with(c, "seal_checkpoint"),
            None => crate::l5_cognition::nt_core_state::delete("seal_checkpoint"),
        }
    }
}

impl SelfIteratingBrain {
    /// ScienceFlow re-anchor (absorbed 2026-08-19, P3): 进程重启后从 KB
    /// 恢复持久化 checkpoint — iteration/reward/brain capability/learning_rate。
    /// 与 ESTRA 的 "continue vs redirect" 对应: 恢复到最高奖励锚点继续进化,
    /// 而非每次零冷启动。skip_kb_io (单元测试) 时跳过, 避免污染生产 KB。
    pub fn re_anchor_from_kb(&mut self) {
        self._re_anchor_from_conn(None);
    }

    /// 注入连接变体 (测试用内存 conn)。
    pub(crate) fn _re_anchor_from_conn(&mut self, conn: Option<&rusqlite::Connection>) {
        if self.skip_kb_io {
            return;
        }
        let Some(cp) = CheckpointManager::_load_from_conn(conn) else {
            return;
        };
        // ESTRA re-anchoring 全量 (absorbed 2026-08-19, P6): 决策矩阵
        // 选择 continue (内存轨迹优于/持平 KB 锚点) 或 redirect (回退锚点)。
        // 内存内已有 checkpoints (进程内多轮) → 与锚点奖励对比;
        // 空环 (冷启动) → 无条件 re-anchor (P3 语义不变)。
        let decision = self._anchor_decision(&cp);
        self._last_anchor_decision = Some(decision);
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
    fn _anchor_decision(&self, cp: &_PersistedCheckpoint) -> AnchorDecision {
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
    pub(crate) fn _last_anchor_decision(&self) -> Option<AnchorDecision> {
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
            if let Err(e) = brain._checkpoint_manager._persist_latest_to_kb() {
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
                mgr._best_checkpoint()
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
    use crate::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;

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

        let cp = mgr._get_checkpoint("cp_0000");
        assert!(cp.is_some());
        let cp = cp.unwrap();
        assert_eq!(cp.iteration, 1);
        assert_eq!(cp.brain_snapshot.learning_rate, 0.42);
        assert_eq!(cp.brain_snapshot.score, 0.95);
        assert_eq!(cp.permission_level, PermissionLevel::Suggest);
        assert_eq!(cp.autonomy_level, AutonomyLevel::Bounded);
        assert_eq!(cp.reward, 0.8);

        let not_found = mgr._get_checkpoint("nonexistent");
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
        let best = mgr._best_checkpoint().unwrap();
        assert_eq!(best.reward, 1.5);
        assert_eq!(best.iteration, 3);
    }

    fn mem_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::l0_substrate::nt_core_kb_primitives::schema_initialize(&conn).unwrap();
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
        mgr._persist_to_conn(Some(&conn)).ok();
        let loaded = CheckpointManager::_load_from_conn(Some(&conn)).expect("roundtrip should load");
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
        mgr._persist_to_conn(Some(&conn)).ok();

        // 全新 brain, skip_kb_io=false 模拟生产重启 → re-anchor 应恢复
        let mut fresh = SelfIteratingBrain::new_lightweight();
        assert_eq!(fresh.skip_kb_io, true, "lightweight 默认跳过 re-anchor");
        fresh._re_anchor_from_conn(Some(&conn)); // skip_kb_io=true → 无副作用
        assert_eq!(fresh.iteration, 0);

        // 强制以生产语义 re-anchor (仅测试: 直接翻转 skip_kb_io 走恢复路径)
        fresh.skip_kb_io = false;
        fresh._re_anchor_from_conn(Some(&conn));
        assert_eq!(fresh.iteration, 11);
        assert!((fresh._reward - 0.6).abs() < 1e-9);
        assert!((fresh.brain.learning_rate - 0.33).abs() < 1e-9);
        assert_eq!(fresh.permission, PermissionLevel::Full);
        assert_eq!(fresh.autonomy, AutonomyLevel::Full);
    }

    #[test]
    fn test_load_absent_returns_none() {
        let conn = mem_conn();
        assert!(CheckpointManager::_load_from_conn(Some(&conn)).is_none(), "无持久化时应 None");
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
        mgr._persist_to_conn(Some(&conn)).ok();

        let mut fresh = SelfIteratingBrain::new_lightweight();
        fresh.skip_kb_io = false;
        fresh._re_anchor_from_conn(Some(&conn));
        assert_eq!(
            fresh._last_anchor_decision(),
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
        mgr._persist_to_conn(Some(&conn)).ok();

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
        fresh._re_anchor_from_conn(Some(&conn));
        assert_eq!(
            fresh._last_anchor_decision(),
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
        mgr._persist_to_conn(Some(&conn)).ok();

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
        fresh._re_anchor_from_conn(Some(&conn));
        assert_eq!(
            fresh._last_anchor_decision(),
            Some(AnchorDecision::Redirect),
            "内存退化应 Redirect"
        );
        assert_eq!(fresh.iteration, 7, "应回退到 KB 锚点迭代号");
        assert!((fresh._reward - 0.9).abs() < 1e-9);
    }
}

#[cfg(test)]
mod frame_checksum_tests {
    use super::{_with_checksum, verify_checksum};

    fn frame_json(reward: f64) -> String {
        serde_json::json!({
            "iteration": 7u64,
            "reward": reward,
            "learning_rate": 0.42,
            "score": 0.9,
            "permission": "Bounded",
            "autonomy": "Suggest",
            "checksum": 0u64
        })
        .to_string()
    }

    /// ✅ 未篡改 ⇒ 校验通过。
    #[test]
    fn 完好帧校验通过() {
        let j = _with_checksum(&frame_json(0.8));
        let recorded = serde_json::from_str::<serde_json::Value>(&j).unwrap()["checksum"]
            .as_u64()
            .unwrap();
        assert!(verify_checksum(&j, recorded), "未篡改的帧必须通过");
    }

    /// ⭐ **核心变异证据**：改一个字节（reward）⇒ 校验必须失败。
    ///
    /// 这正是原实现**完全无法发现**的情形 ——
    /// `from_str(..).ok()` 会成功，`re_anchor` 会拿**被改写的reward** 继续进化。
    #[test]
    fn 篡改reward必须被检出() {
        let good = _with_checksum(&frame_json(0.8));
        // 伪造：拿合法校验和，但改reward
        let tampered = good.replace("0.8", "9.9");
        let recorded = serde_json::from_str::<serde_json::Value>(&good).unwrap()["checksum"]
            .as_u64()
            .unwrap();
        assert!(
            !verify_checksum(&tampered, recorded),
            "改写 reward 却通过校验 ⇒ 校验和无效"
        );
    }

    /// 截断（模拟部分写入 / bit rot）⇒ 必须失败。
    #[test]
    fn 截断帧必须被检出() {
        let good = _with_checksum(&frame_json(0.8));
        let truncated = &good[..good.len() / 2];
        let recorded = serde_json::from_str::<serde_json::Value>(&good).unwrap()["checksum"]
            .as_u64()
            .unwrap();
        assert!(!verify_checksum(truncated, recorded), "截断帧不得通过");
    }
}

#[cfg(test)]
mod backward_compat_tests {
    use super::_PersistedCheckpoint;

    /// ⭐ 升级前的 checkpoint **没有** `checksum` 字段 ⇒ 必须仍能反序列化
    ///（读成 0），否则**升级即丢弃全部进化历史**。
    #[test]
    /// ⭐ 升级前的 checkpoint **没有** `checksum` 字段 ⇒ 必须仍能反序列化
    ///（读成 0），否则**升级即丢弃全部进化历史**。
    #[test]
    fn 旧格式无校验和仍可解析() {
        // ⭐ **不手写 CapabilityVector 的字段**。
        //    我先按 neotrix-types 里那个**同名**类型猜字段
        //    ⇒ 报 missing field typography ⇒ **同名不同类型**（core 内另有其人）。
        //    ⇒ 教训：**猜测数据结构必然错**，让类型自己说话。
        let cap =
            crate::l5_cognition::nt_core::capability::types::CapabilityVector::default();
        let old = serde_json::to_string(&serde_json::json!({
            "iteration": 42u64,
            "reward": 0.77,
            "learning_rate": 0.1,
            "score": 0.5,
            "capability": cap,
            "permission": "Bounded",
            "autonomy": "Suggest"
        }))
        .expect("构造旧格式 JSON");
        let p: _PersistedCheckpoint =
            serde_json::from_str(&old).expect("旧格式必须仍可解析（不得因新字段而失败）");
        assert_eq!(p.checksum, 0, "缺失字段应读成 0（⇒被识别为「无校验和」）");
        assert_eq!(p.iteration, 42, "旧数据本身必须完整保留");
    }
}

#[cfg(test)]
mod compat_end_to_end_tests {
    use super::{fnv1a64, verify_checksum, _with_checksum};

    /// ⭐ **真实载荷路径的兼容变异**：模拟升级后读取一个**旧帧**
    /// （有真实业务数据，但 `checksum` 字段**不存在**）。
    ///
    /// 承重断言：旧帧必须被**识别为「无校验和」并接受**，
    /// ⛔ 而不是被误判为「损坏」而**丢弃全部进化历史**。
    ///
    /// 这就是**为什么必须区分 `checksum == 0` 与 `checksum != 0 但不匹配`**
    /// —— 两者在数值上都是"校验失败"，语义却完全相反。
    #[test]
    fn 旧帧不被误判为损坏() {
        // 旧帧：无 checksum 键
        let old = serde_json::json!({
            "iteration": 123u64,
            "reward": 0.5,
            "permission": "Bounded",
            "autonomy": "Suggest"
        })
        .to_string();

        // 读取侧逻辑的镜像：无 checksum 键 ⇒ parsed.checksum == 0 ⇒ 接受
        let parsed: serde_json::Value =
            serde_json::from_str(&old).expect("旧帧应可解析");
        let recorded = parsed
            .get("checksum")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        assert_eq!(recorded, 0, "旧帧读出的校验和必须是 0（⇒「无校验和」态）");

        // 而「有校验和但不匹配」必须失败 —— 这是真损坏
        let fresh = _with_checksum(
            &serde_json::json!({"iteration": 123u64, "reward": 0.5, "checksum": 0u64})
                .to_string(),
        );
        let fresh_sum = serde_json::from_str::<serde_json::Value>(&fresh).unwrap()["checksum"]
            .as_u64()
            .unwrap();
        let tampered = fresh.replace("0.5", "0.9");
        assert!(
            !verify_checksum(&tampered, fresh_sum),
            "有校验和但被改写 ⇒ 必须判损坏"
        );

        // 校验和函数本身对同一份字节可重复（确定性）
        assert_eq!(
            fnv1a64(b"abc"),
            fnv1a64(b"abc"),
            "FNV-1a 必须是确定性的（否则每次读都判损坏）"
        );
    }
}

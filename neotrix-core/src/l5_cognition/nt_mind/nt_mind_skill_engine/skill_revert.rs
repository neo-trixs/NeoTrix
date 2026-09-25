//! skill_revert — 从 `nt_mind_skill_engine.rs` 拆分 (行为零变更).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::SkillEngine;

pub type _InverseOp = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

#[derive(Clone)]
pub struct RevertibleEffect {
    pub label: String,
    inverse: _InverseOp,
}

impl RevertibleEffect {
    pub fn new(
        label: impl Into<String>,
        inverse: impl Fn() -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            inverse: Arc::new(inverse),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn run(&self) -> Result<(), String> {
        (self.inverse)()
    }
}

impl std::fmt::Debug for RevertibleEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RevertibleEffect").field("label", &self.label).finish()
    }
}

/// 逆账本: `install_id` → 按加载序记录的逆操作列表。
/// 卸载以 LIFO (逆加载序) 派生 teardown, 结构上保证 φ(γ) ≃ γ0。
#[derive(Clone, Default)]
pub struct InverseLedger {
    entries: HashMap<u64, Vec<RevertibleEffect>>,
    next_id: u64,
}

impl std::fmt::Debug for InverseLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InverseLedger")
            .field("next_id", &self.next_id)
            .field("_active_transactions", &self.entries.len())
            .finish()
    }
}

impl InverseLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// 开启一个新的安装事务, 返回 install_id (accumulator φ 的标识)。
    pub fn begin_install(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(id, Vec::new());
        id
    }

    /// 把逆操作按加载顺序推入账本。
    pub fn push_inverse(&mut self, install_id: u64, effect: RevertibleEffect) -> Result<(), String> {
        let entry = self.entries.get_mut(&install_id)
            .ok_or_else(|| format!("unknown install transaction: {}", install_id))?;
        entry.push(effect);
        Ok(())
    }

    /// 加载序下的逆操作标签 (诊断/测试: 记录顺序即加载顺序)。
    pub fn inverse_labels(&self, install_id: u64) -> Vec<String> {
        self.entries
            .get(&install_id)
            .map(|e| e.iter().map(|x| x.label.clone()).collect())
            .unwrap_or_default()
    }

    pub fn inverse_count(&self, install_id: u64) -> usize {
        self.entries.get(&install_id).map_or(0, |e| e.len())
    }

    pub(crate) fn _active_transactions(&self) -> usize {
        self.entries.len()
    }

    /// 事务是否仍存在 (install 逆账本条目的活标志; 供悬挂所有权检测: 事务
    /// 消失但 fiber 仍标记 held 即悬挂所有权)。
    pub fn has_transaction(&self, install_id: u64) -> bool {
        self.entries.contains_key(&install_id)
    }

    /// 以 LIFO (逆加载序) 执行该事务的全部逆操作; 单级失败被记录但不
    /// 中止其余逆操作 (L-Raise: failure per-fiber, siblings keep running)。
    pub fn teardown(&mut self, install_id: u64) -> Vec<Result<(), String>> {
        let entry = match self.entries.remove(&install_id) {
            Some(e) => e,
            None => return Vec::new(),
        };
        entry.into_iter().rev().map(|effect| effect.run()).collect()
    }
}

// ────────────────────────────────────────────────────────────────
// P-F5: FiberLifecycle 惯性生命周期状态机 (吸收 cordiverse §4.1-4.3, F5)
// 组件 = (d: spec, p: provision, e: witnessed effect); fiber = 单次实例化,
// 拥有自己的生命周期状态。非原子转移有惯性; 失败按 fiber 记录 (L-Raise),
// 不传播到 parent — sibling 继续运行。效应总经 Retired/Unloading 恢复, 不滞留。
// ────────────────────────────────────────────────────────────────

/// Fiber 生命周期状态 (F5): Loaded → Active → Suspended → Retired + Failed。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FiberLifecycleState {
    /// 已加载 (install 完成, 尚未激活)
    Loaded,
    /// 激活 (依赖满足, 正常服务)
    Active,
    /// 挂起 (依赖丢失, 等待重新满足)
    Suspended,
    /// 退休 (teardown 完成, 终态)
    Retired,
    /// 失败 (按 fiber 捕获, 不传播到 sibling)
    Failed,
}

impl FiberLifecycleState {
    pub fn label(&self) -> &'static str {
        use FiberLifecycleState::*;
        match self {
            Loaded => "loaded",
            Active => "active",
            Suspended => "suspended",
            Retired => "retired",
            Failed => "failed",
        }
    }
}

/// 单次 fiber 失败记录: 失败时的状态 + 消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _FiberFailure {
    pub at_state: FiberLifecycleState,
    pub message: String,
}

/// Fiber 生命周期状态机 (F5): 惯性转移 (仅合法下一状态) + 按 fiber 失败捕获。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiberLifecycle {
    pub fiber_id: String,
    pub state: FiberLifecycleState,
    pub install_id: u64,
    pub failures: Vec<_FiberFailure>,
}

impl FiberLifecycle {
    pub fn new(fiber_id: impl Into<String>, install_id: u64) -> Self {
        Self {
            fiber_id: fiber_id.into(),
            state: FiberLifecycleState::Loaded,
            install_id,
            failures: Vec::new(),
        }
    }

    /// 惯性转移表 (paper §4.3.3): 仅允许的下一状态。
    /// Retired 是终态 (无后继); Failed 仅可回 Loaded (重装/重试) 或 Retired。
    fn is_allowed(from: FiberLifecycleState, to: FiberLifecycleState) -> bool {
        use FiberLifecycleState::*;
        matches!(
            (from, to),
            (Loaded, Active)
                | (Loaded, Suspended)
                | (Loaded, Retired)
                | (Loaded, Failed)
                | (Active, Suspended)
                | (Active, Retired)
                | (Active, Failed)
                | (Suspended, Active)
                | (Suspended, Retired)
                | (Suspended, Failed)
                | (Failed, Loaded)
                | (Failed, Retired)
        )
    }

    pub fn state(&self) -> FiberLifecycleState {
        self.state
    }

    /// 惯性转移: 非法转移返回 Err 且状态不变 (inertia: in-flight 转移先落地)。
    pub fn transition(&mut self, to: FiberLifecycleState) -> Result<(), String> {
        if self.state == to {
            return Err(format!(
                "fiber '{}' is already {}",
                self.fiber_id,
                self.state.label()
            ));
        }
        if !Self::is_allowed(self.state, to) {
            return Err(format!(
                "illegal transition for fiber '{}': {} → {}",
                self.fiber_id,
                self.state.label(),
                to.label()
            ));
        }
        self.state = to;
        Ok(())
    }

    /// 按 fiber 捕获失败: 记录失败并转入 Failed; 不传播到 sibling。
    pub fn record_failure(&mut self, message: impl Into<String>) {
        self.failures.push(_FiberFailure {
            at_state: self.state,
            message: message.into(),
        });
        self.state = FiberLifecycleState::Failed;
    }
}

// ────────────────────────────────────────────────────────────────
// C5 自愈检测件: revertible_effects (F1) + fiber_lifecycle (F5)
// 纯内存模拟, 无网络/磁盘/env IO (SelfTest 约束)。不变量破坏即 Err。
// ────────────────────────────────────────────────────────────────

/// C5 自愈检测件: 逆账本往返不变量 (φ(γ) ≃ γ0) — LIFO 逆应用必须完全还原状态,
/// 错序/逆丢失必须被检出。纯内存栈式模拟 (无 IO)。
pub struct RevertibleEffectsHealer;

impl RevertibleEffectsHealer {
    /// 构造栈式 install 场景: 每次 install 推入值并登记逆操作 (弹回该值)。
    fn push_scenario(installs: &[i32]) -> (Arc<std::sync::Mutex<Vec<i32>>>, Vec<RevertibleEffect>) {
        let state = Arc::new(std::sync::Mutex::new(Vec::<i32>::new()));
        let mut effects = Vec::new();
        for &v in installs {
            let s = state.clone();
            state.lock().map(|mut s| s.push(v)).unwrap_or_else(|e| e.into_inner().push(v));
            effects.push(RevertibleEffect::new(format!("pop_{}", v), move || {
                let mut s = s.lock().map_err(|e| e.to_string())?;
                match s.pop() {
                    Some(top) if top == v => Ok(()),
                    Some(top) => Err(format!("inverse mismatch: expected {}, got {}", v, top)),
                    None => Err(format!("empty stack during inverse {}", v)),
                }
            }));
        }
        (state, effects)
    }

    /// 按给定顺序应用逆操作并判断状态是否完全还原 (往返不变量)。
    fn roundtrip_restores(
        state: &Arc<std::sync::Mutex<Vec<i32>>>,
        effects: &[RevertibleEffect],
        order: &[usize],
    ) -> bool {
        for &idx in order {
            let ok = effects
                .get(idx)
                .map(|e| e.run())
                .unwrap_or(Err("bad inverse index".into()))
                .is_ok();
            if !ok {
                return false;
            }
        }
        state.lock().map(|s| s.is_empty()).unwrap_or(false)
    }
}

impl crate::l0_substrate::nt_core_self_test::SelfTest for RevertibleEffectsHealer {
    fn name(&self) -> &str {
        "nt_mind_skill_engine::revertible_effects_healer"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // 1) 合法往返: LIFO 逆序应用 → 状态完全还原
        let (state, effects) = Self::push_scenario(&[1, 2, 3]);
        if !Self::roundtrip_restores(&state, &effects, &[2, 1, 0]) {
            failures.push("roundtrip: LIFO 逆应用未还原状态 (φ(γ) ≇ γ0)".into());
        }

        // 2) 顺序错乱: 破坏不变量必须被检出
        let (state_w, effects_w) = Self::push_scenario(&[1, 2, 3]);
        if Self::roundtrip_restores(&state_w, &effects_w, &[0, 1, 2]) {
            failures.push("roundtrip: 错序逆应用被误判为还原 (检测盲区)".into());
        }

        // 3) 逆丢失: 状态滞留必须被检出
        let (state_m, effects_m) = Self::push_scenario(&[1, 2, 3]);
        if Self::roundtrip_restores(&state_m, &effects_m, &[2, 1]) {
            failures.push("roundtrip: 逆丢失未被检出 (状态滞留)".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// C5 自愈检测件: fiber 所有权生命周期 — 合法转移 + 悬挂所有权自动释放。
/// 持有中不可被重新认领, 释放后可重装; holder 消失但 fiber 仍 held 即悬挂,
/// 经 release_dangling 自动转入 Retired。
pub struct FiberLifecycleHealer;

impl FiberLifecycleHealer {
    /// 安装一个带逆账本事务的 fiber 并激活 (合法持有)。
    pub(crate) fn install_held_fiber(engine: &mut SkillEngine, name: &str) -> Result<u64, String> {
        let id = engine.inverse_ledger.begin_install();
        engine
            .inverse_ledger
            .push_inverse(id, RevertibleEffect::new(format!("inverse_{}", name), || Ok(())))?;
        engine
            .fiber_lifecycles
            .insert(name.to_string(), FiberLifecycle::new(name.to_string(), id));
        let _ = engine
            .fiber_lifecycles
            .get_mut(name)
            .ok_or_else(|| format!("fiber '{}' not inserted", name))?
            .transition(FiberLifecycleState::Active);
        Ok(id)
    }
}

impl crate::l0_substrate::nt_core_self_test::SelfTest for FiberLifecycleHealer {
    fn name(&self) -> &str {
        "nt_mind_skill_engine::fiber_lifecycle_healer"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        use FiberLifecycleState::*;

        let mut failures = Vec::new();
        let mut engine = SkillEngine::new(PathBuf::new());

        // 1) 所有权转移合法: 持有中不可被重新认领 (惯性状态机拒绝自环)。
        let id = match Self::install_held_fiber(&mut engine, "f1") {
            Ok(id) => id,
            Err(e) => return Err(vec![format!("install held fiber failed: {}", e)]),
        };
        if engine.fiber_lifecycles.get("f1").map(|f| f.state) != Some(Active) {
            failures.push("合法持有 fiber 未处于 Active".into());
        }
        if !engine.inverse_ledger.has_transaction(id) {
            failures.push("持有中 fiber 的逆账本事务必须存在".into());
        }
        let mut probe = FiberLifecycle::new("f1", id);
        let _ = probe.transition(Active);
        if probe.transition(Active).is_ok() {
            failures.push("持有中 fiber 被非法重新认领 (自环)".into());
        }

        // 2) 释放后重新认领合法: 卸载 (Retired) 后重装派生新 install 事务。
        match engine.uninstall_skill("f1") {
            Ok(_) => {}
            Err(e) => failures.push(format!("uninstall_skill failed: {}", e)),
        }
        let id2 = match Self::install_held_fiber(&mut engine, "f1") {
            Ok(id2) => id2,
            Err(e) => {
                failures.push(format!("re-claim after release failed: {}", e));
                u64::MAX
            }
        };
        if engine.fiber_lifecycles.get("f1").map(|f| f.state) != Some(Active) {
            failures.push("释放后重新认领未处于 Active".into());
        }
        if id2 != id + 1 {
            failures.push("重装必须派生新的 install 事务".into());
        }

        // 3) 悬挂所有权: holder 消失 (事务被消耗) 但 fiber 仍 held → 自动释放。
        let dangling_id = match Self::install_held_fiber(&mut engine, "dangling") {
            Ok(id3) => id3,
            Err(e) => {
                failures.push(format!("install dangling fiber failed: {}", e));
                u64::MAX
            }
        };
        engine.inverse_ledger.teardown(dangling_id);
        let released = engine.release_dangling();
        if !released.iter().any(|n| n == "dangling") {
            failures.push(format!("悬挂 fiber 未被自动释放, released={:?}", released));
        }
        if engine.fiber_lifecycles.get("dangling").map(|f| f.state) != Some(Retired) {
            failures.push("悬挂 fiber 释放后应处于 Retired".into());
        }
        let extra = engine.release_dangling();
        if !extra.is_empty() {
            failures.push(format!("合法持有被误判为悬挂: {:?}", extra));
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

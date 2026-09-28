//! nt_tool_registry — 工具注册表 (构建期事实).
//! 从 `nt_core_gate/mod.rs` 纯搬移, 行为零变更.
//!
//! # ⛔⛔ 防误删：本文件是**活路径**，不是 stub ⛔⛔
//!
//! 2026-09-27 的 `EVOLUTION-ROADMAP-CODE-NODES-2026-09-27.md` §0.2 把它标为
//! 「45 行 stub，建议整文件删除」。**该结论已被实测证伪，照做会打断 shield
//! enforcer 的写操作可逆性判定**：
//!
//! | 证据 | 内容 |
//! |---|---|
//! | 活消费者 | `l3_embodiment/nt_shield_enforcer.rs:388-400`<br>`fn write_action_registry() -> &'static ...nt_core_gate::ToolRegistry`（`LazyLock` 静态） |
//! | 它承载什么 | `ToolSpec` 驱动的**可逆性事实源**：<br>`.register(ToolSpec::reversible("write_file", "undo_file"))`<br>`.register(ToolSpec::irreversible("git_force_push"))` … |
//! | 被谁读 | `nt_shield_enforcer.rs:411` 读 `spec.reversibility != ToolReversibility::ReadOnly` 来判定「是否写操作」 |
//!
//! **与 `nt_act/tool_registry.rs` 的 `ToolRegistry` 是「同名不同轴」，不是重复**：
//! - 本文件 = **构建期可逆性事实**（reversible / irreversible，单向声明）
//! - `nt_act` 那份 = **运行期计数**（`ToolStats`：调用次数、成功率）
//!
//! 二者**正交**，各管一轴，合并会丢一维语义。
//! 同类判例见 `docs/architecture/OWNERSHIP.md`（`agentic_browse::ToolRegistry`
//! 亦为正交、保留）。
//!
//! 判活/判死一律查**构造点**（`Type::new(` 的调用方），不查 `pub mod` 声明 ——
//! 「导出 ≠ 调用」已错过 4 次。

use serde::{Deserialize, Serialize};

use super::nt_types::ToolSpec;

/// 工具注册表 — 构建期事实, 非运行期猜测 (TianPan: risk class as versioned tool attribute)。
/// 同一注册表同时发 tool spec 与 gate config, 两者不能分歧。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolRegistry {
    specs: std::collections::HashMap<String, ToolSpec>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(mut self, spec: ToolSpec) -> Self {
        self.specs.insert(spec.name.clone(), spec);
        self
    }

    pub fn get(&self, name: &str) -> Option<&ToolSpec> {
        self.specs.get(name)
    }

    pub fn all_specs(&self) -> Vec<&ToolSpec> {
        self.specs.values().collect()
    }

    pub fn cloned_specs(&self) -> Vec<ToolSpec> {
        self.specs.values().cloned().collect()
    }

    /// 从工具名列表快速构建 (只读默认)。
    pub fn from_read_only(names: &[&str]) -> Self {
        let mut reg = Self::new();
        for n in names {
            reg = reg.register(ToolSpec::read_only(n));
        }
        reg
    }
}

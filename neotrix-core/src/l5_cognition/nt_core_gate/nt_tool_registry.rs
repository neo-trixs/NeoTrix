//! nt_tool_registry — 工具注册表 (构建期事实).
//! 从 `nt_core_gate/mod.rs` 纯搬移, 行为零变更.

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

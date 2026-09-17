//! # Unified SelfModel — 三重自我模型统一入口
//!
//! **职责**: 提供单一入口访问三个自我模型变体，消除命名歧义。
//!
//! ## 三个变体
//!
//! | 变体 | 别名 | 职责 | 模块路径 |
//! |------|------|------|----------|
//! | `StaticIdentity` | `StaticIdentityModel` | 结构身份快照 — "我是什么？" | `nt_meta::SelfModel` |
//! | `DynamicPerformance` | `DynamicPerformanceModel` | 性能估算 — "我表现如何？" | `nt_core_self::SelfModel` |
//! | `ValueFunction` | `ValueFunctionModel` | 价值函数 — "我重视什么？" | `nt_core_self_model::SelfModel` |
//!
//! ## 设计原则
//! - 三个模型各自独立演化，不强制合并结构
//! - `UnifiedSelfModel` 作为 facade 提供统一访问
//! - 向后兼容：旧路径的类型别名继续有效
//!
//! 铁律: `#![forbid(unsafe_code)]` (R-P1) — 零 unsafe。

pub use crate::l5_cognition::nt_core::nt_meta::self_model as static_identity;
pub use crate::l6_meta::nt_core_self::self_model as dynamic_performance;
pub use crate::l6_meta::nt_core_self_model as value_function;

/// 三重自我模型统一 facade。
///
/// 包含三个独立子模型，提供统一访问接口。
/// 各子模型保持独立演化，不强制结构对齐。
#[derive(Debug, Clone)]
pub struct UnifiedSelfModel {
    /// 静态结构身份 — "我是什么？"
    pub static_identity: static_identity::SelfModel,
    /// 动态性能估算 — "我表现如何？"
    pub dynamic_performance: dynamic_performance::SelfModel,
    /// 价值函数 — "我重视什么？"
    pub value_function: value_function::SelfModel,
}

impl Default for UnifiedSelfModel {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedSelfModel {
    /// 创建默认统一自我模型：三个子模型均取默认值。
    pub fn new() -> Self {
        Self {
            static_identity: static_identity::SelfModel::new(),
            dynamic_performance: dynamic_performance::SelfModel::new(),
            value_function: value_function::SelfModel::new(),
        }
    }

    /// 综合价值评估：结合价值函数与动态性能的加权分数。
    ///
    /// `action` — 待评估的动作/状态描述。
    /// 返回 `[0.0, 1.0]` 综合评分。
    pub fn evaluate(&self, action: &str) -> f64 {
        let vf = self.value_function.value_function(action);
        let perf = self.dynamic_performance.state.capability;
        // 60% 价值契合 + 40% 当前能力
        (vf * 0.6 + perf * 0.4).clamp(0.0, 1.0)
    }

    /// 推进动态性能模型一个 tick。
    pub fn tick(&mut self, workspace_signal: f64, load_delta: f64, meta_alarm: usize) {
        self.dynamic_performance.tick(workspace_signal, load_delta, meta_alarm);
    }

    /// 重置所有子模型到默认值。
    pub fn reset(&mut self) {
        self.static_identity = static_identity::SelfModel::new();
        self.dynamic_performance.reset();
        self.value_function = value_function::SelfModel::new();
    }
}

/// 统一自我模型别名，消除命名歧义
pub type SelfModel = UnifiedSelfModel;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_self_model_default() {
        let m = UnifiedSelfModel::new();
        assert_eq!(m.static_identity.module_count(), 0);
        assert_eq!(m.dynamic_performance.state.capability, 0.5);
        assert!((0.0..=1.0).contains(&m.value_function.value_function("test")));
    }

    #[test]
    fn unified_evaluate_bounds() {
        let m = UnifiedSelfModel::new();
        let score = m.evaluate("do something coherent and safe");
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn unified_tick_advances() {
        let mut m = UnifiedSelfModel::new();
        m.tick(0.9, 0.1, 0);
        assert!(m.dynamic_performance.updates > 0);
    }
}

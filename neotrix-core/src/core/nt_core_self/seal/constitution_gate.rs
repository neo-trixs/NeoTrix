//! S 门控的自编辑闭环 — 灵境引擎 L4「自指闭环」转译。
//!
//! # D≥4 回环证据链
//!
//! ```text
//! 意识循环 (nt_mind_background_loop::handlers_consciousness)
//!   └─ 写入 brain._last_consciousness_quality (Φ/IIT 派生信号, 0.0–1.0)
//!        └─ SEAL 自编辑应用点前 ConstitutionGate 裁决:
//!             · 尚无意识信号 (_consciousness_critique_count == 0) → None → 保守放行并计数
//!             · Some(q) < SELF_EDIT_MIN_CONSCIOUSNESS → 否决, 跳过本次应用 (行为改变)
//!                  └─ 被放行的编辑修改系统自身能力 (ReasoningBrain.apply_micro_edits)
//!                       └─ 下一轮意识质量由被修改后的系统重新产生
//!                            ⇒ 度量信号反过来控制"计算该度量的系统本身"的修改 ⇒ D≥4 自指闭环
//! ```
//!
//! 应用点接线 (R-P79 同 session 生产路径):
//! 1. [`BoundedEditStage`](super::super::neotrix::l8_autonomic_impl) — SEAL 双管线
//!    (seal_pipeline / kernel_iterate_pipeline) 中自编辑队列的唯一处理 stage,
//!    否决即清空待应用队列, 编辑不再流向下游采纳/归档。
//! 2. `SelfIteratingBrain::code_review_iterate` — 唯一直接调用
//!    `ReasoningBrain::apply_micro_edits` 的生产入口, 否决即跳过应用。
//!
//! 计数器暴露 getter (`allowed_count`/`denied_count`/`no_signal_count`),
//! 供 SelfTest 断言门控真实参与行为决策 (T3 行为接地)。
#![forbid(unsafe_code)]

/// 自编辑应用的最低意识质量阈值。
/// quality < 阈值 → 否决 (严格小于; 等于阈值视为可信, 放行)。
pub const SELF_EDIT_MIN_CONSCIOUSNESS: f64 = 0.5;

/// 宪法门控: 意识质量信号门控 SEAL 对自身的修改。
///
/// 单实例挂在 `SelfIteratingBrain::_constitution_gate` 上, 跨迭代累计裁决计数;
/// 纯函数判定逻辑见 [`ConstitutionGate::should_allow_self_edit`]。
#[derive(Debug, Clone)]
pub struct ConstitutionGate {
    threshold: f64,
    allowed_count: u64,
    denied_count: u64,
    no_signal_count: u64,
}

impl Default for ConstitutionGate {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstitutionGate {
    pub fn new() -> Self {
        Self {
            threshold: SELF_EDIT_MIN_CONSCIOUSNESS,
            allowed_count: 0,
            denied_count: 0,
            no_signal_count: 0,
        }
    }

    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            threshold,
            ..Self::new()
        }
    }

    /// 纯函数判定: 意识质量是否允许本轮自编辑应用。
    /// - `None` (尚无意识信号) → 保守放行 (不因信号缺失阻塞演化);
    /// - `Some(q)`, q < threshold → 否决;
    /// - `Some(q)`, q >= threshold → 放行。
    pub fn should_allow_self_edit(quality: Option<f64>, threshold: f64) -> bool {
        match quality {
            None => true,
            Some(q) => q >= threshold,
        }
    }

    /// 带计数的裁决入口 (应用点调用)。返回是否放行。
    pub fn judge(&mut self, quality: Option<f64>) -> bool {
        if quality.is_none() {
            self.no_signal_count += 1;
        }
        let allowed = Self::should_allow_self_edit(quality, self.threshold);
        if allowed {
            self.allowed_count += 1;
        } else {
            self.denied_count += 1;
        }
        allowed
    }

    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    pub fn allowed_count(&self) -> u64 {
        self.allowed_count
    }

    pub fn denied_count(&self) -> u64 {
        self.denied_count
    }

    /// 无信号保守放行的次数 (与 allowed_count 重叠计入)。
    pub fn no_signal_count(&self) -> u64 {
        self.no_signal_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_quality_allows_conservatively() {
        // 尚无意识信号 → 保守放行, 不阻塞演化
        assert!(ConstitutionGate::should_allow_self_edit(None, SELF_EDIT_MIN_CONSCIOUSNESS));
    }

    #[test]
    fn test_low_quality_denied() {
        assert!(!ConstitutionGate::should_allow_self_edit(Some(0.2), SELF_EDIT_MIN_CONSCIOUSNESS));
        assert!(!ConstitutionGate::should_allow_self_edit(Some(0.49), SELF_EDIT_MIN_CONSCIOUSNESS));
        assert!(!ConstitutionGate::should_allow_self_edit(Some(0.0), SELF_EDIT_MIN_CONSCIOUSNESS));
    }

    #[test]
    fn test_threshold_boundary_allows() {
        // 否决条件是严格小于: 等于阈值放行
        assert!(ConstitutionGate::should_allow_self_edit(Some(0.5), SELF_EDIT_MIN_CONSCIOUSNESS));
        assert!(ConstitutionGate::should_allow_self_edit(Some(0.51), SELF_EDIT_MIN_CONSCIOUSNESS));
        assert!(ConstitutionGate::should_allow_self_edit(Some(1.0), SELF_EDIT_MIN_CONSCIOUSNESS));
    }

    #[test]
    fn test_custom_threshold_respected() {
        assert!(ConstitutionGate::should_allow_self_edit(Some(0.7), 0.8).eq(&false));
        assert!(ConstitutionGate::should_allow_self_edit(Some(0.9), 0.8));
    }

    #[test]
    fn test_judge_counts_all_three_outcomes() {
        let mut gate = ConstitutionGate::new();
        assert!(gate.judge(None), "无信号放行");
        assert!(gate.judge(Some(0.9)), "高质量放行");
        assert!(!gate.judge(Some(0.1)), "低质量否决");
        assert_eq!(gate.allowed_count(), 2);
        assert_eq!(gate.denied_count(), 1);
        assert_eq!(gate.no_signal_count(), 1);
    }
}

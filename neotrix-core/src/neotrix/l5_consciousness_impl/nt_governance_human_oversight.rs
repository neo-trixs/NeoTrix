#![forbid(unsafe_code)]

//! NT-GOVERNANCE 人类监督治理 (Human Oversight Governance) — 外部吸收 (arXiv:2608.23642)
//!
//! 来源: "AI Agents Push Humans Out of the Loop" (Mitchell, Ghosh, Passi; 2026-08-24).
//! 与 l1 的 NT-SHIELD 监督退化检测件互补: 本节点站在治理层, 提供「保持人类在环」的
//! 设计级 affordance + 技能萎缩对策的组织协议, 而非运行期检测。
//!
//! 三支柱:
//! 1. OversightAffordance — 设计级监督 affordance 枚举 (让人类能/必须介入的接口形态)
//! 2. SkillAtrophyCountermeasure — 技能萎缩对策 (组织协议: 强制复训/知识刷新/角色轮转)
//! 3. HumanOversightPolicy — 治理策略单元: 组合二者并评测「有效人类在环」覆盖度
//!
//! 吸收裁决 (R-P42/R-P79): 强化 NT-GOVERNANCE Gov-衡 节点, 非平行适配器. Dark Forest: 接入即验证.

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// 监督 affordance — 系统主动暴露给人类监督者的介入接口形态。
/// 设计级 (论文 §design affordances): 把「人类在环」做成默认结构, 而非可选。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OversightAffordance {
    /// 审批闸门: 高风险动作需显式人类批准才执行 (action gating)。
    ApprovalGate,
    /// 批量复核: 多动作聚合后统一人类审阅 (batch review), 降低打扰。
    BatchReview,
    /// 战略摩擦: 注入时延/确认步骤, 打破 agent 自强化回路。
    StrategicFriction,
    /// 可解释回溯: 提供决策溯源/依据, 让监督者能低成本复核。
    ExplainableTrace,
    /// 降级接管: 人类可随时收回自治, 恢复手动控制。
    TakeoverFallback,
}

impl OversightAffordance {
    /// 是否直接缓解「监督者被边缘化」(认知需求支持维度)。
    pub fn supports_overseer_cognition(&self) -> bool {
        matches!(
            self,
            Self::ExplainableTrace | Self::BatchReview | Self::TakeoverFallback
        )
    }

    /// 是否约束 agent 自治边界 (bounded autonomy)。
    pub fn bounds_autonomy(&self) -> bool {
        matches!(
            self,
            Self::ApprovalGate | Self::StrategicFriction | Self::TakeoverFallback
        )
    }
}

/// 技能萎缩对策 — 组织协议, 对抗人类因长期脱离执行而技能退化。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillAtrophyCountermeasure {
    /// 强制复训: 定期由人类亲手执行原由 agent 承担的任务。
    MandatoryRetraining,
    /// 知识刷新: 监督者周期性重读 agent 决策依据/源证据。
    KnowledgeRefresh,
    /// 角色轮转: 人类与 agent 在执行/监督角色间轮换。
    RoleRotation,
    /// 协同执行: 高风险任务人类与 agent 并行操作 (co-active), 维持技能。
    CoActiveExecution,
}

impl SkillAtrophyCountermeasure {
    /// 该对策直接维持的技能维度 (论文: 执行技能 vs 监督技能)。
    pub fn sustains_execution_skill(&self) -> bool {
        matches!(
            self,
            Self::MandatoryRetraining | Self::CoActiveExecution | Self::RoleRotation
        )
    }
}

/// 监督治理策略 — 组合 affordance + 萎缩对策, 形成一个可注册的治理单元。
#[derive(Debug, Clone, Default)]
pub struct HumanOversightPolicy {
    pub affordances: Vec<OversightAffordance>,
    pub countermeasures: Vec<SkillAtrophyCountermeasure>,
}

impl HumanOversightPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    /// 策略完整性评分 ∈ [0,1]: 需同时覆盖 (a) 边界自治 affordance 与 (b) 萎缩对策。
    /// 缺口任一类 → 评分折半, 反映治理层对人类在环的覆盖度。
    pub fn coverage(&self) -> f64 {
        let has_bound = self.affordances.iter().any(|a| a.bounds_autonomy());
        let has_atrophy = !self.countermeasures.is_empty();
        match (has_bound, has_atrophy) {
            (true, true) => 1.0,
            (true, false) | (false, true) => 0.5,
            (false, false) => 0.0,
        }
    }

    /// 是否构成「有效人类在环」治理 (两类均覆盖且含认知支持 affordance)。
    pub fn is_effective(&self) -> bool {
        self.coverage() == 1.0
            && self
                .affordances
                .iter()
                .any(|a| a.supports_overseer_cognition())
    }
}

// ── SelfTest (T1 存在 + T2 注册) ───────────────────────────────

pub struct HumanOversightGovernance;
impl SelfTest for HumanOversightGovernance {
    fn name(&self) -> &str {
        "governance:human-oversight"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        // affordance 分类校验
        if !OversightAffordance::ApprovalGate.bounds_autonomy() {
            return Err(vec!["ApprovalGate 应约束自治".into()]);
        }
        if !OversightAffordance::BatchReview.supports_overseer_cognition() {
            return Err(vec!["BatchReview 应支持监督认知".into()]);
        }
        // 萎缩对策分类校验
        if !SkillAtrophyCountermeasure::MandatoryRetraining.sustains_execution_skill() {
            return Err(vec!["MandatoryRetraining 应维持执行技能".into()]);
        }
        // 完整策略应判有效
        let mut p = HumanOversightPolicy::new();
        p.affordances
            .push(OversightAffordance::ApprovalGate);
        p.affordances
            .push(OversightAffordance::ExplainableTrace);
        p.countermeasures
            .push(SkillAtrophyCountermeasure::MandatoryRetraining);
        if !p.is_effective() {
            return Err(vec!["覆盖 affordance+countermeasure 的策略应判有效".into()]);
        }
        // 缺口策略应判无效
        let mut gap = HumanOversightPolicy::new();
        gap.affordances.push(OversightAffordance::ExplainableTrace);
        if gap.is_effective() {
            return Err(vec!["仅 affordance 无对策应判无效".into()]);
        }
        Ok(())
    }
}

/// 注册进 SelfTestRegistry (T2) — 由 `register_absorbed_modules` 调用。
pub fn register_human_oversight_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(HumanOversightGovernance));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_self_test_passes() {
        assert!(
            HumanOversightGovernance.self_test().is_ok(),
            "人类监督治理 SelfTest 应通过"
        );
    }

    #[test]
    fn test_oversight_affordance_classification() {
        assert!(OversightAffordance::StrategicFriction.bounds_autonomy());
        assert!(!OversightAffordance::BatchReview.bounds_autonomy());
        assert!(OversightAffordance::TakeoverFallback.supports_overseer_cognition());
        assert!(!OversightAffordance::ApprovalGate.supports_overseer_cognition());
    }

    #[test]
    fn test_policy_coverage_and_effectiveness() {
        // 完整覆盖 = 1.0 且有效
        let mut full = HumanOversightPolicy::new();
        full.affordances.push(OversightAffordance::ApprovalGate);
        full.affordances.push(OversightAffordance::ExplainableTrace);
        full.countermeasures
            .push(SkillAtrophyCountermeasure::KnowledgeRefresh);
        assert_eq!(full.coverage(), 1.0);
        assert!(full.is_effective());

        // 仅 affordance 无对策 = 0.5 且无效
        let mut partial = HumanOversightPolicy::new();
        partial
            .affordances
            .push(OversightAffordance::ApprovalGate);
        assert_eq!(partial.coverage(), 0.5);
        assert!(!partial.is_effective());
    }
}

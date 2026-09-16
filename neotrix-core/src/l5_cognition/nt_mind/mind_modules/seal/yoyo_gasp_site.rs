//! NT-MIND — yoyo-gasp.yolog.dev 吸收 (C1 参考节点).
//!
//! yoyo-gasp: GASP 自进化智能体运行时参考。GASP 将 identity / skills / memory /
//! journal / lineage 全部入仓 (clone 即唤醒), 其 `patch.proposed → eval.finished →
//! decision.created → Promoted` 四阶段与 NeoTrix `make_stage!` 宏 (SEAL pipeline)
//! 高度同构。
//!
//! 本模块为 C1 参考节点: 定义 GASP 运行时五仓库字段 → NeoTrix 对应模块 trait stub
//! 映射, 实现字段级对照, 验证同构关系 (含 2-3 单测)。作为 NT-MIND 采纳 GASP schema
//! 的参考基线 (C2 集成详见 absorption-20260828-batch2.md 高价值标注)。

use crate::core::nt_core_self_test::SelfTest;

/// GASP 运行时五大入仓维度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _GaspVault {
    /// 智能体身份 (唤醒即加载)。
    Identity,
    /// 技能集 (可入仓/出仓的 skill 单元)。
    Skills,
    /// 经验记忆 (跨会话持久)。
    Memory,
    /// 演化日志 (可读审计轨迹)。
    Journal,
    /// 演化谱系 (父子代际传承链)。
    Lineage,
}

impl _GaspVault {
    /// GASP 五仓库 → NeoTrix 对应模块的字段映射说明。
    pub(crate) fn _maps_to_neotrix(&self) -> &'static str {
        match self {
            _GaspVault::Identity => "nt_core_self::Self (E8引导者 身份锚) / ConsciousnessTree 元认知身份",
            _GaspVault::Skills => "nt_mind_skill_engine (Disclosure Ladder / skill crystallization)",
            _GaspVault::Memory => "nt_memory KB (kv_store / experience namespace)",
            _GaspVault::Journal => "ConsciousnessTree 6-stage loop / experience-tree 吸收日志",
            _GaspVault::Lineage => "SEAL pipeline stage chain / make_stage! 阶段血缘",
        }
    }

    /// 全部五仓库维度。
    pub fn all() -> &'static [_GaspVault] {
        &[
            _GaspVault::Identity,
            _GaspVault::Skills,
            _GaspVault::Memory,
            _GaspVault::Journal,
            _GaspVault::Lineage,
        ]
    }
}

/// GASP 四阶段演化循环 (patch→eval→decision→promote) 与 make_stage! 同构映射。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _GaspStage {
    /// 提出修改 (patch.proposed)。
    PatchProposed,
    /// 评估完成 (eval.finished) — 对应 R-P42 验证门槛。
    EvalFinished,
    /// 决策生成 (decision.created)。
    DecisionCreated,
    /// 晋升落地 (Promoted)。
    Promoted,
}

impl _GaspStage {
    /// GASP 阶段 → NeoTrix SEAL stage 名称映射。
    pub(crate) fn _maps_to_seal_stage(&self) -> &'static str {
        match self {
            _GaspStage::PatchProposed => "SEAL explore/distill (propose mutation)",
            _GaspStage::EvalFinished => "SEAL self-test (R-P42 eval gate)",
            _GaspStage::DecisionCreated => "SEAL absorption gate (accept/reject)",
            _GaspStage::Promoted => "SEAL promote (wire to production path)",
        }
    }

    /// 阶段闭环顺序 (用于验证非空闭环)。
    pub fn lifecycle() -> &'static [_GaspStage] {
        &[
            _GaspStage::PatchProposed,
            _GaspStage::EvalFinished,
            _GaspStage::DecisionCreated,
            _GaspStage::Promoted,
        ]
    }
}

/// yoyo-gasp 运行时参考映射器。
pub struct _YoyoGaspRuntime;

impl _YoyoGaspRuntime {
    pub fn all_vaults_mapped(&self) -> bool {
        _GaspVault::all().iter().all(|v| !v._maps_to_neotrix().is_empty())
    }

    pub fn all_stages_mapped(&self) -> bool {
        _GaspStage::lifecycle()
            .iter()
            .all(|s| !s._maps_to_seal_stage().is_empty())
    }
}

impl SelfTest for _YoyoGaspRuntime {
    fn name(&self) -> &'static str {
        "_YoyoGaspRuntime"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();
        if !self.all_vaults_mapped() {
            errs.push("GASP vault→NeoTrix mapping incomplete".into());
        }
        if !self.all_stages_mapped() {
            errs.push("GASP stage→SEAL mapping incomplete".into());
        }
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_maps_to_neotrix_module() {
        // TODO(R-P79): Only checks hardcoded string constants — trivial assertion.
        // Replace with test that validates actual module resolution against real vault paths.
        let m = _YoyoGaspRuntime;
        assert!(m.all_vaults_mapped());
        assert_eq!(_GaspVault::Skills._maps_to_neotrix(),
            "nt_mind_skill_engine (Disclosure Ladder / skill crystallization)");
        assert!(m.self_test().is_ok());
    }

    #[test]
    fn test_stage_maps_to_seal_lifecycle() {
        let m = _YoyoGaspRuntime;
        assert!(m.all_stages_mapped());
        assert_eq!(_GaspStage::EvalFinished._maps_to_seal_stage(),
            "SEAL self-test (R-P42 eval gate)");
    }

    #[test]
    fn test_gasp_lifecycle_four_stages() {
        assert_eq!(_GaspStage::lifecycle().len(), 4);
        assert_eq!(_GaspStage::lifecycle()[3], _GaspStage::Promoted);
        assert_eq!(_GaspVault::all().len(), 5);
    }
}

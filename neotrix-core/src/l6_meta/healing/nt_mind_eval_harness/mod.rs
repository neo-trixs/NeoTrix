//! L9 Evaluation Harness — Model×Budget 质量-成本评测 (R2-Bench 式) (门面)。
//!
//! 参考: R2-Router (ICML 2026), R2-Bench dataset
//! 核心指标: AUDC (Area Under Deferral Curve), QNC (Query-Normalized Cost), Peak Quality
//! 预算执行: prompt 注入 "use at most K tokens" (Lee et al. 2025)
//!
//! 本模块为纯搬移门面：行为零变更，外部路径 `nt_mind_eval_harness::X` 不变。

pub mod nt_budget;
pub mod nt_compliance;
pub mod nt_harness;
pub mod nt_pareto;
pub mod nt_regression;
pub mod nt_types;
pub mod nt_verify_oracle;
#[cfg(test)]
mod tests;

pub use nt_budget::DEFAULT_BUDGET_GRID;
pub use nt_compliance::{
    ap_acc_score, ComplianceGate, InstructionPlane, PlaneConflictCase, WithholdingResult,
    AP_ACC_EPSILON,
};
pub use nt_harness::EvalHarness;
pub use nt_pareto::{
    hda_attribution, HdaAttribution, HdaAttributionReport, HdaAttributionSelfTest, HdaComponent,
};
pub use nt_regression::{
    HyperparamSensitivity, RegressionCase, RegressionResult, SmallScaleMethod, SmallScaleWarning,
};
pub use nt_types::{
    DatasetSpec, EvalError, EvalPoint, EvalQuery, EvalReport, GalaxyComparison, ModelQualityCurve,
    ModelSpec, ParetoPoint,
};
pub use nt_verify_oracle::{
    verify_constraint, verify_deterministic, verify_extractable, verify_unified,
    verify_unified_batch, LadderReport, OracleLadder, OracleLadderHealer, OracleRung, RewardSignal,
    RungOracle, RungResult, SelfVerifiableReward, SelfVerifiableRewardSelfTest, UnifiedVerifyRequest,
    UnifiedVerifyResult, VerificationChannel,
};

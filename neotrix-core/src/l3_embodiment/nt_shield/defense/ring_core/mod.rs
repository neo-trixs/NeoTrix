//! Ring Core - 核心环：零信任，最小权限
//!
//! 信任锚点 + 推理链保护 + ASI 合规
//! 信号流：所有输入必须经过此环验证

pub mod trust_anchor;
pub mod reasoning_shield;
pub mod asi_compliance;

pub use trust_anchor::TrustAnchor;
pub use reasoning_shield::ReasoningShield;
pub use asi_compliance::AsiComplianceChecker;

/// 核心环验证结果
///
/// ⭐ 审计裁定 2026-10-07（`dead-flag` 报字段「待人工判定」，核实后**部分成立**）：
///
/// | 字段 | 裁定 |
/// |---|---|
/// | `trust_level` | ✅ **真实计算** —— 来自 `calculate_trust()`（L160） |
/// | `signals` | ✅ **真实计算** —— 来自 `collect_signals()` |
/// | `reasoning_safe` | ⛔ **硬编码 `true`** —— `trust_anchor.rs:93` 是字面量，⛔ 非计算结果 |
/// | `asi_compliant` | ⛔ **硬编码 `true`** —— `trust_anchor.rs:94` 是字面量（注释说"由 asi_compliance 模块进一步验证"，但该验证**未接入**） |
///
/// ⇒ 后两个是**伪装成验证结果的常量**。它们比死字段更糟：
///    字段名 `reasoning_safe` / `asi_compliant` 读起来像**已验证的安全结论**，
///    而实际上恒为 `true` ⇒ 任何据此做的决策都在使用**未经检验的假设**。
///
/// ⚠️ 另注：`CoreVerification` **零外部消费者** —— 它由
///    `TrustAnchor::verify()` 构造并返回，但全仓无人调用该 `verify()`
///    （grep 命中的 `planner_executor.rs:329` 等是 `PlannerExecutor::verify`，
///     **同名异型**，返回类型带 `passed`/`depth`，与本类型无关）。
///    而 `OuterVerification` / `InnerVerification` / `BoundaryVerification`
///    三个同族类型**连构造点都没有** ⇒ 纯死类型。
#[derive(Debug, Clone)]
pub struct CoreVerification {
    pub trust_level: TrustLevel,
    /// ⛔⚠️ 硬编码 `true`，⛔ 非计算结果（见本 struct 的裁定文档）
    pub reasoning_safe: bool,
    /// ⛔⚠️ 硬编码 `true`，⛔ 非计算结果（见本 struct 的裁定文档）
    pub asi_compliant: bool,
    pub signals: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    Unknown = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Trusted = 4,
}

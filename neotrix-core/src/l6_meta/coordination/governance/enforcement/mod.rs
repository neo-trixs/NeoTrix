//! Governance enforcement engine (R-P141, R-P142).
//!
//! Provides policy-driven action evaluation with audit logging and
//! system-wide compliance reporting.
//!
//! NT-LAWS: Agent 行为不变量强制执行
//! - law_checker: Law trait 和 LawChecker，基于 Bend LAWS.bend 启发

pub mod audit;
pub mod compliance_checker;
pub mod enforcer;
pub mod law_checker;
pub mod policy;

pub use audit::{AuditEntry, AuditLog};
pub use compliance_checker::{ComplianceChecker, GovernanceComplianceStatus, PolicyCheckResult, SystemComplianceReport};
pub use enforcer::{EnforcementEngine, EnforcementResult, EnforcementStats, Violation};
pub use law_checker::{Law, LawCheckResult, LawChecker, LawSeverity, LawViolation};
pub use policy::{EnforcementLevel, Policy, PolicyRule, RuleSeverity};

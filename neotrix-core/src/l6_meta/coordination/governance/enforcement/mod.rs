//! Governance enforcement engine (R-P141, R-P142).
//!
//! Provides policy-driven action evaluation with audit logging and
//! system-wide compliance reporting.

pub mod audit;
pub mod compliance_checker;
pub mod enforcer;
pub mod policy;

pub use audit::{AuditEntry, AuditLog};
pub use compliance_checker::{ComplianceChecker, GovernanceComplianceStatus, PolicyCheckResult, SystemComplianceReport};
pub use enforcer::{EnforcementEngine, EnforcementResult, EnforcementStats, Violation};
pub use policy::{EnforcementLevel, Policy, PolicyRule, RuleSeverity};

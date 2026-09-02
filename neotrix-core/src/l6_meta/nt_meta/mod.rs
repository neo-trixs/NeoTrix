//! L6 Meta-Cognition Layer - Meta Modules
//! 
//! Meta-cognitive functionality including cross-domain awareness, goal analysis, routing.

// Local modules
pub mod nt_governance;
pub mod nt_core_intra_reflection;
pub mod nt_shield_approval;
pub mod nt_shield_audit;
pub mod nt_mind_repair;
pub mod nt_meta_sentrux;
pub mod nt_meta_build_watchdog;
pub mod nt_meta_integration_patterns;
pub mod nt_meta_integration_points;
pub mod nt_meta_concurrency_tester;
pub mod nt_meta_async_safety;
pub mod nt_meta_concurrency_detector;

// Re-export all
pub use nt_governance::*;
pub use nt_core_intra_reflection::*;
pub use nt_shield_approval::*;
pub use nt_shield_audit::*;
pub use nt_mind_repair::*;
pub use nt_meta_sentrux::{SentruxSensor, QualitySnapshot, SessionComparison};
pub use nt_meta_build_watchdog::{
    BuildWatchdog, WatchdogConfig, BuildMonitor, MonitorStatus, BuildStatus,
    CompilationResult, TestResult, CacheStatus, BuildAlert, AlertSeverity, WatchdogStats, FixAction,
};
pub use nt_meta_integration_patterns::{
    IntegrationPatternLibrary, IntegrationConfig, IntegrationPattern, PatternCategory,
    ActiveIntegration, IntegrationPlan, PlanStep, IntegrationCheck, RuleViolation,
};
pub use nt_meta_integration_points::{
    IntegrationPointManager, IntegrationPointConfig, IntegrationPoint, IntegrationType,
    IntegrationStatus, ModuleIntegration, ComplianceStatus, IntegrationPointStats,
    IntegrationAuditResult,
};
pub use nt_meta_concurrency_tester::{
    ConcurrencyIsolationTester, ConcurrencyConfig, TestSession, SessionStatus,
    IsolationConfig, ConcurrencyStats,
};
pub use nt_meta_async_safety::{
    AsyncSafetyWrapper, AsyncSafetyConfig, BlockingWrapper, WrapperType,
    AsyncSafetyStats, SafetyCheckResult, SafetyViolation, ViolationSeverity,
};
pub use nt_meta_concurrency_detector::*;

// 跨模块一致性检查 (动态漫技能吸收)
pub mod cross_module_audit;
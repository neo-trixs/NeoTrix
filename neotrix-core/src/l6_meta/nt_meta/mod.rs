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

// 模板复用标签系统
pub mod template_tag_registry;

// 质量控制审核
pub mod quality_gate;

// 通用能力模块 (从漫剧专用重构为通用)
pub mod quality_control; // 质量控制流水线 (原 quality_gate)

// 验证器引导模块
pub mod verifier_agent;

// 分层质量检查
pub mod layered_qa;

// 空值标准化器 (LLM 输出空值处理)
pub mod null_normalizer;

// 视频质量评分器 (自动化质量评分 + 时序一致性)
pub mod video_quality_scorer;

// 内容审核 (提示词过滤 + 输出分析 + 人工审核路由)
pub mod content_moderation;

// 视频审计追踪 (合规日志 + 溯源追踪)
pub mod video_audit_trail;

// 向后兼容别名
pub use quality_control::QualityGate;
# Governance Modules

> L6 Meta-Cognition — Quality Control, Audit, Compliance, Self-Improvement

---

## Overview

The governance subsystem provides quality control, cross-module auditing, and compliance frameworks for the NeoTrix system.

```text
coordination (nt_governance)
├── governance/              — Governance framework
├── nt_governance/           — Governance implementation
├── quality_control/         — Quality gates
├── quality_gate.rs          — Quality gate definition
├── cross_module_audit.rs    — Cross-module auditing
├── layered_qa.rs            — Layered QA system
├── verifier_agent.rs        — Verifier agent
├── null_normalizer.rs       — Null normalization
├── self_improvement.rs      — Self-improvement loop
├── template_tag_registry.rs — Template tag registry
├── nt_meta_sentrux.rs       — Sentrux sensor
├── nt_meta_build_watchdog.rs — Build watchdog
├── nt_meta_integration_patterns.rs — Integration patterns
├── nt_meta_integration_points.rs — Integration points
├── nt_meta_concurrency_tester.rs — Concurrency testing
├── nt_meta_async_safety.rs  — Async safety
├── nt_meta_concurrency_detector.rs — Concurrency detection
└── nt_task_orchestrator.rs  — Task orchestration
```

---

## Key Types

### `QualityGate`

Quality gate enforcement.

```rust
pub struct QualityGate {
    rules: Vec<QualityRule>,
    history: Vec<QualityCheck>,
}

pub enum QualityRule {
    MaxComplexity { threshold: usize },
    MinCoverage { threshold: f64 },
    MaxWarnings { threshold: usize },
    NoUnsafeCode,
    NoDeadCode,
    MaxFileSize { threshold: usize },
    Custom(String, Box<dyn Fn(&FileState) -> bool>),
}

pub struct FileState {
    pub path: String,
    pub lines: usize,
    pub complexity: usize,
    pub coverage: f64,
    pub warnings: usize,
    pub has_unsafe: bool,
    pub has_dead_code: bool,
}

pub struct QualityCheck {
    pub timestamp: DateTime<Utc>,
    pub passed: bool,
    pub violations: Vec<QualityViolation>,
    pub score: f64,
}

pub struct QualityViolation {
    pub rule: String,
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
    pub severity: ViolationSeverity,
}

pub enum ViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
```

**Public Methods:**

```rust
impl QualityGate {
    pub fn new() -> Self;
    pub fn add_rule(&mut self, rule: QualityRule);
    pub fn check(&self, state: &FileState) -> QualityCheck;
    pub fn check_all(&self, states: &[FileState]) -> Vec<QualityCheck>;
    pub fn history(&self) -> &[QualityCheck];
    pub fn pass_rate(&self) -> f64;
}
```

---

### `CrossModuleAuditor`

Cross-module auditing system.

```rust
pub struct CrossModuleAuditor {
    modules: Vec<ModuleInfo>,
    audit_history: Vec<AuditResult>,
}

pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    pub dependencies: Vec<String>,
    pub public_api: Vec<String>,
    pub tests: usize,
}

pub struct AuditResult {
    pub timestamp: DateTime<Utc>,
    pub modules_audited: usize,
    pub issues: Vec<AuditIssue>,
    pub score: f64,
}

pub struct AuditIssue {
    pub module: String,
    pub kind: IssueKind,
    pub description: String,
    pub severity: f64,
    pub recommendation: String,
}

pub enum IssueKind {
    DependencyCycle,
    UnusedExport,
    MissingTests,
    InconsistentStyle,
    DeadCode,
    SecurityRisk,
    PerformanceIssue,
}
```

**Public Methods:**

```rust
impl CrossModuleAuditor {
    pub fn new() -> Self;
    pub fn register_module(&mut self, module: ModuleInfo);
    pub fn audit(&mut self) -> AuditResult;
    pub fn audit_module(&self, name: &str) -> AuditResult;
    pub fn history(&self) -> &[AuditResult];
    pub fn issues(&self) -> Vec<&AuditIssue>;
}
```

---

### `SelfImprovementLoop`

Continuous self-improvement system.

```rust
pub struct SelfImprovementLoop {
    plans: Vec<ImprovementPlan>,
    history: Vec<CycleResult>,
    metrics: SystemMetrics,
}

pub struct ImprovementPlan {
    pub id: String,
    pub dimensions: Vec<ImprovementDimension>,
    pub actions: Vec<ImprovementAction>,
    pub status: PlanStatus,
    pub created_at: DateTime<Utc>,
}

pub enum ImprovementDimension {
    Performance,
    Reliability,
    Security,
    Maintainability,
    Documentation,
    Testing,
}

pub struct ImprovementAction {
    pub id: String,
    pub dimension: ImprovementDimension,
    pub description: String,
    pub impact: f64,
    pub effort: f64,
    pub status: ActionStatus,
}

pub enum PlanStatus {
    Draft,
    Active,
    Completed,
    Abandoned,
}

pub enum ActionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

pub struct SystemMetrics {
    pub performance_score: f64,
    pub reliability_score: f64,
    pub security_score: f64,
    pub maintainability_score: f64,
    pub overall_score: f64,
}

pub struct CycleResult {
    pub cycle: usize,
    pub timestamp: DateTime<Utc>,
    pub improvements: Vec<Improvement>,
    pub regressions: Vec<Regression>,
    pub net_score: f64,
    pub metrics: SystemMetrics,
}

pub struct Improvement {
    pub dimension: ImprovementDimension,
    pub before: f64,
    pub after: f64,
    pub delta: f64,
}

pub struct Regression {
    pub dimension: ImprovementDimension,
    pub before: f64,
    pub after: f64,
    pub delta: f64,
}
```

**Public Methods:**

```rust
impl SelfImprovementLoop {
    pub fn new() -> Self;
    pub fn create_plan(&mut self, plan: ImprovementPlan);
    pub fn run_cycle(&mut self) -> CycleResult;
    pub fn run_cycles(&mut self, n: usize) -> Vec<CycleResult>;
    pub fn plans(&self) -> &[ImprovementPlan];
    pub fn history(&self) -> &[CycleResult];
    pub fn current_metrics(&self) -> &SystemMetrics;
    pub fn trend(&self) -> TrendDirection;
}
```

---

### `BuildWatchdog`

Build monitoring and alerting.

```rust
pub struct BuildWatchdog {
    config: WatchdogConfig,
    monitor: BuildMonitor,
    alerts: Vec<BuildAlert>,
}

pub struct WatchdogConfig {
    pub check_interval: Duration,
    pub alert_threshold: u32,
    pub auto_fix: bool,
}

pub struct BuildMonitor {
    pub status: MonitorStatus,
    pub last_check: DateTime<Utc>,
    pub build_status: BuildStatus,
    pub compilation: CompilationResult,
    pub tests: TestResult,
    pub cache: CacheStatus,
}

pub enum MonitorStatus {
    Ok,
    Warning,
    Critical,
}

pub enum BuildStatus {
    Success,
    Failed,
    InProgress,
    Unknown,
}

pub struct CompilationResult {
    pub warnings: usize,
    pub errors: usize,
    pub duration: Duration,
}

pub struct TestResult {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration: Duration,
}

pub struct CacheStatus {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

pub struct BuildAlert {
    pub id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub action: Option<FixAction>,
}

pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

pub enum FixAction {
    Retry,
    Clean,
    Rebuild,
    Rollback,
    Notify,
}

pub struct WatchdogStats {
    pub total_checks: u64,
    pub total_alerts: u64,
    pub auto_fixes: u64,
    pub uptime: Duration,
}
```

**Public Methods:**

```rust
impl BuildWatchdog {
    pub fn new(config: WatchdogConfig) -> Self;
    pub fn check(&mut self) -> BuildStatus;
    pub fn alerts(&self) -> &[BuildAlert];
    pub fn acknowledge(&mut self, alert_id: &str);
    pub fn stats(&self) -> WatchdogStats;
    pub fn auto_fix(&mut self) -> Result<()>;
}
```

---

### `IntegrationPatternLibrary`

Integration pattern management.

```rust
pub struct IntegrationPatternLibrary {
    patterns: Vec<IntegrationPattern>,
    active: Vec<ActiveIntegration>,
    config: IntegrationConfig,
}

pub struct IntegrationConfig {
    pub auto_apply: bool,
    pub validation_level: ValidationLevel,
    pub rollback_on_failure: bool,
}

pub enum ValidationLevel {
    Strict,
    Normal,
    Lenient,
}

pub struct IntegrationPattern {
    pub id: String,
    pub name: String,
    pub category: PatternCategory,
    pub description: String,
    pub steps: Vec<PlanStep>,
    pub validation: Vec<IntegrationCheck>,
}

pub enum PatternCategory {
    ModuleIntegration,
    ApiMigration,
    DatabaseSchema,
    Configuration,
    Deployment,
    Testing,
}

pub struct PlanStep {
    pub order: usize,
    pub description: String,
    pub action: String,
    pub validation: Option<String>,
}

pub struct IntegrationCheck {
    pub name: String,
    pub kind: CheckKind,
    pub expected: String,
}

pub enum CheckKind {
    FileExists,
    CodeCompiles,
    TestsPass,
    NoWarnings,
    Custom(String),
}

pub struct ActiveIntegration {
    pub pattern_id: String,
    pub started_at: DateTime<Utc>,
    pub current_step: usize,
    pub status: IntegrationStatus,
    pub violations: Vec<RuleViolation>,
}

pub enum IntegrationStatus {
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

pub struct RuleViolation {
    pub rule: String,
    pub message: String,
    pub severity: ViolationSeverity,
}
```

**Public Methods:**

```rust
impl IntegrationPatternLibrary {
    pub fn new(config: IntegrationConfig) -> Self;
    pub fn add_pattern(&mut self, pattern: IntegrationPattern);
    pub fn apply_pattern(&mut self, pattern_id: &str) -> Result<ActiveIntegration>;
    pub fn validate(&self, integration: &ActiveIntegration) -> Vec<RuleViolation>;
    pub fn rollback(&mut self, integration_id: &str) -> Result<()>;
    pub fn patterns(&self) -> &[IntegrationPattern];
    pub fn active(&self) -> &[ActiveIntegration];
}
```

---

### `ConcurrencyIsolationTester`

Concurrency isolation testing.

```rust
pub struct ConcurrencyIsolationTester {
    config: ConcurrencyConfig,
    sessions: Vec<TestSession>,
    stats: ConcurrencyStats,
}

pub struct ConcurrencyConfig {
    pub max_concurrent: usize,
    pub timeout: Duration,
    pub isolation_level: IsolationConfig,
}

pub struct IsolationConfig {
    pub memory_isolation: bool,
    pub io_isolation: bool,
    pub state_isolation: bool,
}

pub struct TestSession {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub status: SessionStatus,
    pub metrics: SessionMetrics,
}

pub enum SessionStatus {
    Running,
    Completed,
    Failed,
    Timeout,
}

pub struct SessionMetrics {
    pub operations: u64,
    pub conflicts: u64,
    pub duration: Duration,
    pub memory_usage: usize,
}

pub struct ConcurrencyStats {
    pub total_sessions: u64,
    pub successful: u64,
    pub failed: u64,
    pub avg_duration: Duration,
    pub conflict_rate: f64,
}
```

**Public Methods:**

```rust
impl ConcurrencyIsolationTester {
    pub fn new(config: ConcurrencyConfig) -> Self;
    pub fn run_test(&mut self) -> TestSession;
    pub fn run_tests(&mut self, n: usize) -> Vec<TestSession>;
    pub fn sessions(&self) -> &[TestSession];
    pub fn stats(&self) -> &ConcurrencyStats;
    pub fn validate_isolation(&self) -> bool;
}
```

---

### `RecursiveController`

Task orchestration with recursive control.

```rust
pub struct RecursiveController {
    root: TaskNode,
    history: Vec<ControlEvent>,
}

pub struct TaskNode {
    pub id: String,
    pub name: String,
    pub kind: TaskKind,
    pub children: Vec<TaskNode>,
    pub status: TaskStatus,
    pub result: Option<Value>,
}

pub enum TaskKind {
    Sequential,
    Parallel,
    Conditional,
    Loop,
    Leaf,
}

pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

pub struct ControlEvent {
    pub timestamp: DateTime<Utc>,
    pub task_id: String,
    pub event: EventType,
    pub details: String,
}

pub enum EventType {
    Started,
    Completed,
    Failed,
    Skipped,
    Retried,
}
```

**Public Methods:**

```rust
impl RecursiveController {
    pub fn new(root: TaskNode) -> Self;
    pub fn execute(&mut self) -> Result<Value>;
    pub fn abort(&mut self);
    pub fn status(&self) -> &TaskStatus;
    pub fn history(&self) -> &[ControlEvent];
    pub fn task(&self, id: &str) -> Option<&TaskNode>;
}
```

---

## Related Modules

| Module | Description |
|--------|-------------|
| `nt_meta_sentrux` | Sentrux quality sensor |
| `nt_meta_integration_patterns` | Integration patterns |
| `nt_meta_integration_points` | Integration points |
| `nt_meta_concurrency_tester` | Concurrency testing |
| `nt_meta_async_safety` | Async safety |
| `nt_meta_concurrency_detector` | Concurrency detection |
| `nt_task_orchestrator` | Task orchestration |
| `null_normalizer` | Null normalization |
| `verifier_agent` | Verifier agent |

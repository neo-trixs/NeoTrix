# Meta Modules (nt_meta)

> L6 Meta-Cognition — Self-Model, Planning, Evolution, Monitoring

---

## Overview

NT-META provides meta-cognitive capabilities including self-modeling, evolution planning, code scanning, and knowledge gap detection.

```text
nt_meta
├── self_model          — Self-model (value function model)
├── weakness            — Weakness analysis
├── planner             — Evolution planning
├── scanner             — Code scanning
├── knowledge_gap_detector — Knowledge gap detection
├── monitor             — Health monitoring
├── arch_optimizer      — Architecture optimization
├── gwt_router          — GWT routing
├── meta_cognition      — Meta-cognition loop
├── metacognition_loop  — Meta-cognitive cycle
├── nt_core_meta_auditor — Meta auditor
├── nt_core_arch_lint   — Architecture linting
├── auto_inspector/     — Auto inspection
├── dream_replay/       — Dream replay
├── eval_engine/        — Evaluation engine
├── otel_bridge/        — OpenTelemetry bridge
└── session_replay/     — Session replay
```

---

## Key Types

### `SelfModel`

Value function model for self-awareness.

```rust
pub struct SelfModel {
    components: Vec<ComponentNode>,
    dependency_graph: DepGraph,
    tech_debt: TechDebtInventory,
    test_coverage: TestCoverage,
    compilation_health: CompilationHealth,
}

pub struct ComponentNode {
    pub id: String,
    pub name: String,
    pub kind: ComponentKind,
    pub dependencies: Vec<String>,
    pub health: f64,
}

pub enum ComponentKind {
    Module,
    Crate,
    Service,
    Database,
    External,
}

pub struct DepGraph {
    pub nodes: Vec<ComponentNode>,
    pub edges: Vec<DepEdge>,
}

pub struct DepEdge {
    pub from: String,
    pub to: String,
    pub kind: DepKind,
    pub weight: f64,
}

pub enum DepKind {
    Direct,
    Indirect,
    Cyclic,
    Optional,
}

pub struct TechDebtInventory {
    pub items: Vec<TechDebtItem>,
    pub total_severity: f64,
}

pub struct TechDebtItem {
    pub id: String,
    pub description: String,
    pub severity: DebtSeverity,
    pub location: String,
    pub created_at: DateTime<Utc>,
}

pub enum DebtSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct TestCoverage {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub percentage: f64,
    pub uncovered_files: Vec<String>,
}

pub struct CompilationHealth {
    pub warnings: usize,
    pub errors: usize,
    pub last_build: DateTime<Utc>,
    pub build_status: BuildStatus,
}

pub enum BuildStatus {
    Success,
    Warning,
    Error,
    Failed,
}
```

**Public Methods:**

```rust
impl SelfModel {
    pub fn new() -> Self;
    pub fn analyze(&mut self) -> Result<AnalysisReport>;
    pub fn health_score(&self) -> f64;
    pub fn component(&self, id: &str) -> Option<&ComponentNode>;
    pub fn dependencies(&self, id: &str) -> Vec<&DepEdge>;
    pub fn tech_debt(&self) -> &TechDebtInventory;
    pub fn test_coverage(&self) -> &TestCoverage;
    pub fn compilation_health(&self) -> &CompilationHealth;
    pub fn update(&mut self) -> Result<()>;
}
```

---

### `WeaknessAnalyzer`

Weakness detection and analysis.

```rust
pub struct WeaknessAnalyzer {
    weaknesses: Vec<Weakness>,
    history: Vec<WeaknessReport>,
}

pub struct Weakness {
    pub id: String,
    pub kind: WeaknessKind,
    pub severity: f64,
    pub description: String,
    pub location: String,
    pub detected_at: DateTime<Utc>,
}

pub enum WeaknessKind {
    Performance,
    Security,
    Maintainability,
    Correctness,
    Documentation,
    Testing,
}

pub struct WeaknessReport {
    pub timestamp: DateTime<Utc>,
    pub weaknesses: Vec<Weakness>,
    pub summary: WeaknessSummary,
}

pub struct WeaknessSummary {
    pub total: usize,
    pub by_kind: HashMap<WeaknessKind, usize>,
    pub avg_severity: f64,
    pub trend: TrendDirection,
}

pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}
```

**Public Methods:**

```rust
impl WeaknessAnalyzer {
    pub fn new() -> Self;
    pub fn analyze(&mut self) -> WeaknessReport;
    pub fn weaknesses(&self) -> &[Weakness];
    pub fn severity(&self, id: &str) -> Option<f64>;
    pub fn prioritize(&self) -> Vec<&Weakness>;
    pub fn history(&self) -> &[WeaknessReport];
}
```

---

### `EvolutionPlanner`

Evolution action planning.

```rust
pub struct EvolutionPlanner {
    goals: Vec<MetaGoal>,
    actions: Vec<EvolutionAction>,
    history: Vec<PlannedEvolution>,
}

pub struct MetaGoal {
    pub id: String,
    pub description: String,
    pub priority: f64,
    pub deadline: Option<DateTime<Utc>>,
    pub status: GoalStatus,
}

pub enum GoalStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

pub struct EvolutionAction {
    pub id: String,
    pub goal_id: String,
    pub kind: ActionKind,
    pub description: String,
    pub impact: ImpactEstimate,
    pub risk: RiskLevel,
    pub status: ActionStatus,
}

pub enum ActionKind {
    Refactor,
    Optimize,
    Test,
    Document,
    Migrate,
    Deprecate,
}

pub struct ImpactEstimate {
    pub files_affected: usize,
    pub lines_changed: usize,
    pub risk_score: f64,
    pub benefit_score: f64,
}

pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub enum ActionStatus {
    Planned,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

pub struct PlannedEvolution {
    pub id: String,
    pub goals: Vec<MetaGoal>,
    pub actions: Vec<EvolutionAction>,
    pub created_at: DateTime<Utc>,
    pub estimated_duration: Duration,
}
```

**Public Methods:**

```rust
impl EvolutionPlanner {
    pub fn new() -> Self;
    pub fn add_goal(&mut self, goal: MetaGoal);
    pub fn plan(&mut self) -> PlannedEvolution;
    pub fn execute(&mut self, plan: &PlannedEvolution) -> Result<()>;
    pub fn goals(&self) -> &[MetaGoal];
    pub fn actions(&self) -> &[EvolutionAction];
    pub fn history(&self) -> &[PlannedEvolution];
}
```

---

### `CodeScanner`

Codebase scanning and analysis.

```rust
pub struct CodeScanner {
    rules: Vec<ScanRule>,
    results: Vec<ScanResult>,
}

pub enum ScanRule {
    Complexity { max: usize },
    Duplication { threshold: f64 },
    DeadCode,
    UnsafeCode,
    WarningCount { max: usize },
}

pub struct ScanResult {
    pub file: String,
    pub line: usize,
    pub rule: String,
    pub severity: f64,
    pub message: String,
}
```

**Public Methods:**

```rust
impl CodeScanner {
    pub fn new() -> Self;
    pub fn add_rule(&mut self, rule: ScanRule);
    pub fn scan(&mut self, path: &Path) -> Vec<ScanResult>;
    pub fn scan_all(&mut self) -> Vec<ScanResult>;
    pub fn results(&self) -> &[ScanResult];
}
```

---

### `KnowledgeGapDetector`

Knowledge gap detection and analysis.

```rust
pub struct KnowledgeGapDetector {
    gaps: Vec<KnowledgeGap>,
    clusters: Vec<GapCluster>,
}

pub struct KnowledgeGap {
    pub id: String,
    pub category: GapCategory,
    pub description: String,
    pub severity: f64,
    pub suggested_actions: Vec<String>,
}

pub enum GapCategory {
    Documentation,
    Testing,
    Architecture,
    Security,
    Performance,
    Monitoring,
}

pub struct GapCluster {
    pub gaps: Vec<String>,
    pub pattern: String,
    pub recommendation: String,
}

pub struct GapReport {
    pub timestamp: DateTime<Utc>,
    pub gaps: Vec<KnowledgeGap>,
    pub clusters: Vec<GapCluster>,
    pub summary: GapSummary,
}

pub struct GapSummary {
    pub total_gaps: usize,
    pub by_category: HashMap<GapCategory, usize>,
    pub avg_severity: f64,
}
```

**Public Methods:**

```rust
impl KnowledgeGapDetector {
    pub fn new() -> Self;
    pub fn detect(&mut self) -> GapReport;
    pub fn gaps(&self) -> &[KnowledgeGap];
    pub fn clusters(&self) -> &[GapCluster];
    pub fn prioritize(&self) -> Vec<&KnowledgeGap>;
}
```

---

### `MetaMonitor`

System health monitoring.

```rust
pub struct MetaMonitor {
    health_checks: Vec<HealthCheck>,
    alerts: Vec<MetaAlert>,
    trends: Vec<HealthTrend>,
}

pub struct HealthCheck {
    pub name: String,
    pub status: CheckStatus,
    pub value: f64,
    pub threshold: f64,
    pub timestamp: DateTime<Utc>,
}

pub enum CheckStatus {
    Ok,
    Warning,
    Critical,
    Unknown,
}

pub struct MetaAlert {
    pub id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
}

pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

pub struct HealthTrend {
    pub metric: String,
    pub values: Vec<f64>,
    pub trend: TrendDirection,
    pub prediction: Option<f64>,
}
```

**Public Methods:**

```rust
impl MetaMonitor {
    pub fn new() -> Self;
    pub fn check_health(&mut self) -> Vec<HealthCheck>;
    pub fn alerts(&self) -> &[MetaAlert];
    pub fn acknowledge(&mut self, alert_id: &str);
    pub fn trends(&self) -> &[HealthTrend];
    pub fn add_check(&mut self, check: HealthCheck);
}
```

---

### `MetaCognitiveLoop`

Meta-cognitive cycle execution.

```rust
pub struct MetaCognitiveLoop {
    cycle_count: usize,
    results: Vec<MetaCycleResult>,
    config: LoopConfig,
}

pub struct LoopConfig {
    pub max_cycles: usize,
    pub cycle_timeout: Duration,
    pub auto_optimize: bool,
}

pub struct MetaCycleResult {
    pub cycle: usize,
    pub duration: Duration,
    pub improvements: Vec<Improvement>,
    pub regressions: Vec<Regression>,
    pub net_score: f64,
}

pub struct Improvement {
    pub kind: String,
    pub impact: f64,
    pub description: String,
}

pub struct Regression {
    pub kind: String,
    pub impact: f64,
    pub description: String,
}
```

**Public Methods:**

```rust
impl MetaCognitiveLoop {
    pub fn new(config: LoopConfig) -> Self;
    pub fn run_cycle(&mut self) -> MetaCycleResult;
    pub fn run_cycles(&mut self, n: usize) -> Vec<MetaCycleResult>;
    pub fn cycle_count(&self) -> usize;
    pub fn results(&self) -> &[MetaCycleResult];
    pub fn net_improvement(&self) -> f64;
}
```

---

### `RuntimeMonitor`

Real-time system monitoring.

```rust
pub struct RuntimeMonitor {
    metrics: HashMap<String, Metric>,
    history: Vec<MetricSnapshot>,
}

pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub updated_at: DateTime<Utc>,
}

pub struct MetricSnapshot {
    pub timestamp: DateTime<Utc>,
    pub metrics: HashMap<String, f64>,
}
```

**Public Methods:**

```rust
impl RuntimeMonitor {
    pub fn new() -> Self;
    pub fn record(&mut self, name: &str, value: f64, unit: &str);
    pub fn get(&self, name: &str) -> Option<&Metric>;
    pub fn history(&self, name: &str) -> Vec<&MetricSnapshot>;
    pub fn snapshot(&self) -> MetricSnapshot;
}
```

---

### `EvolvingEvaluator`

Continuous evaluation and scoring.

```rust
pub struct EvolvingEvaluator {
    evaluators: Vec<Box<dyn Evaluator>>,
    scores: Vec<EvaluationScore>,
}

pub trait Evaluator {
    fn evaluate(&self, system: &SystemState) -> f64;
    fn name(&self) -> &str;
}

pub struct EvaluationScore {
    pub timestamp: DateTime<Utc>,
    pub scores: HashMap<String, f64>,
    pub overall: f64,
}

pub struct SystemState {
    pub version: String,
    pub uptime: Duration,
    pub metrics: HashMap<String, f64>,
}
```

**Public Methods:**

```rust
impl EvolvingEvaluator {
    pub fn new() -> Self;
    pub fn add_evaluator(&mut self, evaluator: Box<dyn Evaluator>);
    pub fn evaluate(&self, state: &SystemState) -> EvaluationScore;
    pub fn history(&self) -> &[EvaluationScore];
    pub fn trend(&self) -> TrendDirection;
}
```

---

## Related Modules

| Module | Description |
|--------|-------------|
| `nt_core_self` | Self-awareness |
| `nt_core_self_constitution` | Self-constitution |
| `nt_core_aware` | Awareness system |
| `nt_core_observer` | Observer pattern |
| `nt_core_absorb` | Knowledge absorption |
| `nt_core_iter` | Iteration engine |
| `nt_core_scheduler` | Task scheduling |
| `nt_core_self_review` | Self-review |
| `nt_core_capability` | Capability management |
| `nt_agent_identity` | Agent identity |
| `nt_agent_gallery` | Agent gallery |
| `nt_safety_monitor` | Safety monitoring |
| `nt_emergence_detector` | Emergence detection |

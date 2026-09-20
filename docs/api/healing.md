# Healing Modules

> L6 Meta-Cognition — Self-Healing, Diagnostic, Repair, Predictive Maintenance

---

## Overview

The healing subsystem provides self-healing, diagnostic chain analysis, and predictive maintenance capabilities for the NeoTrix system.

```text
healing
├── diagnostic_chain/          — Diagnostic chain analysis
├── self_healing/              — Self-healing engine
├── predictive_maintenance/    — Predictive maintenance
├── nt_repair_facade           — Repair facade
├── nt_repair_self_heal        — Self-healing implementation
├── nt_repair_causal_trace     — Causal trace analysis
├── nt_repair_hanzi_video      — Hanzi video repair
├── nt_mind_consciousness_gold_standard — Consciousness gold standard
├── nt_mind_consciousness_monitor — Consciousness monitoring
├── nt_mind_eval_harness       — Evaluation harness
├── nt_core_self_test          — Self-test
└── nt_core_self_test_integration — Integration test
```

---

## Key Types

### `DiagnosticChain`

Chain analysis for root cause diagnosis.

```rust
pub struct DiagnosticChain {
    links: Vec<DiagnosticLink>,
    root_cause: Option<String>,
    confidence: f64,
}

pub struct DiagnosticLink {
    pub id: String,
    pub cause: String,
    pub effect: String,
    pub evidence: Vec<String>,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct DiagnosticReport {
    pub chain: DiagnosticChain,
    pub root_cause: String,
    pub recommendations: Vec<String>,
    pub severity: DiagnosticSeverity,
}

pub enum DiagnosticSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

**Public Methods:**

```rust
impl DiagnosticChain {
    pub fn new() -> Self;
    pub fn add_link(&mut self, link: DiagnosticLink);
    pub fn analyze(&mut self) -> DiagnosticReport;
    pub fn root_cause(&self) -> Option<&str>;
    pub fn confidence(&self) -> f64;
    pub fn links(&self) -> &[DiagnosticLink];
    pub fn recommendations(&self) -> Vec<String>;
}
```

---

### `SelfHealingEngine`

Automated self-healing capabilities.

```rust
pub struct SelfHealingEngine {
    rules: Vec<HealingRule>,
    history: Vec<HealingEvent>,
    config: HealingConfig,
}

pub struct HealingRule {
    pub id: String,
    pub trigger: HealingTrigger,
    pub action: HealingAction,
    pub priority: u32,
    pub enabled: bool,
}

pub enum HealingTrigger {
    ErrorRate { threshold: f64 },
    Latency { threshold_ms: u64 },
    MemoryUsage { threshold_percent: f64 },
    Custom(String),
}

pub enum HealingAction {
    Restart,
    Scale { replicas: u32 },
    Rollback,
    Alert,
    Custom(String),
}

pub struct HealingEvent {
    pub id: String,
    pub trigger: HealingTrigger,
    pub action: HealingAction,
    pub result: HealingResult,
    pub timestamp: DateTime<Utc>,
    pub duration: Duration,
}

pub enum HealingResult {
    Success,
    Failure(String),
    Partial(String),
}

pub struct HealingConfig {
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub cooldown: Duration,
    pub auto_heal: bool,
}
```

**Public Methods:**

```rust
impl SelfHealingEngine {
    pub fn new(config: HealingConfig) -> Self;
    pub fn add_rule(&mut self, rule: HealingRule);
    pub fn remove_rule(&mut self, id: &str);
    pub fn trigger(&mut self, trigger: &HealingTrigger) -> Result<HealingEvent>;
    pub fn history(&self) -> &[HealingEvent];
    pub fn stats(&self) -> HealingStats;
}
```

---

### `PredictiveMaintenance`

Predictive maintenance system.

```rust
pub struct PredictiveMaintenance {
    models: Vec<PredictionModel>,
    predictions: Vec<Prediction>,
    history: Vec<MaintenanceEvent>,
}

pub trait PredictionModel {
    fn predict(&self, metrics: &SystemMetrics) -> Prediction;
    fn name(&self) -> &str;
    fn accuracy(&self) -> f64;
}

pub struct Prediction {
    pub kind: PredictionKind,
    pub confidence: f64,
    pub time_to_event: Option<Duration>,
    pub recommendation: String,
}

pub enum PredictionKind {
    Failure,
    Degradation,
    CapacityLimit,
    SecurityBreach,
}

pub struct MaintenanceEvent {
    pub id: String,
    pub prediction: Prediction,
    pub action: MaintenanceAction,
    pub result: MaintenanceResult,
    pub timestamp: DateTime<Utc>,
}

pub enum MaintenanceAction {
    Preventive,
    Corrective,
    Predictive,
    Emergency,
}

pub enum MaintenanceResult {
    Success,
    Failure(String),
    Delayed,
}

pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: f64,
    pub error_rate: f64,
    pub latency_p99: f64,
}
```

**Public Methods:**

```rust
impl PredictiveMaintenance {
    pub fn new() -> Self;
    pub fn add_model(&mut self, model: Box<dyn PredictionModel>);
    pub fn predict(&self, metrics: &SystemMetrics) -> Vec<Prediction>;
    pub fn schedule_maintenance(&mut self, prediction: &Prediction) -> MaintenanceEvent;
    pub fn history(&self) -> &[MaintenanceEvent];
    pub fn accuracy(&self) -> f64;
}
```

---

### `CausalTrace`

Causal trace analysis for debugging.

```rust
pub struct CausalTrace {
    events: Vec<CausalEvent>,
    trace_id: String,
}

pub struct CausalEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub kind: EventKind,
    pub data: Value,
    pub causation: Option<String>,
}

pub enum EventKind {
    Input,
    Processing,
    Output,
    Error,
    StateChange,
    External,
}

pub struct TraceReport {
    pub trace_id: String,
    pub events: Vec<CausalEvent>,
    pub causation_chain: Vec<String>,
    pub anomalies: Vec<Anomaly>,
}

pub struct Anomaly {
    pub event_id: String,
    pub kind: AnomalyKind,
    pub description: String,
    pub severity: f64,
}

pub enum AnomalyKind {
    TimingAnomaly,
    ValueAnomaly,
    MissingEvent,
    UnexpectedEvent,
    CycleDetected,
}
```

**Public Methods:**

```rust
impl CausalTrace {
    pub fn new(trace_id: &str) -> Self;
    pub fn record(&mut self, event: CausalEvent);
    pub fn analyze(&self) -> TraceReport;
    pub fn causation_chain(&self, event_id: &str) -> Vec<&CausalEvent>;
    pub fn anomalies(&self) -> Vec<&Anomaly>;
    pub fn export(&self) -> String;
}
```

---

### `SelfTest`

System self-test capabilities.

```rust
pub struct SelfTest {
    tests: Vec<TestDefinition>,
    results: Vec<TestResult>,
}

pub struct TestDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: TestKind,
    pub timeout: Duration,
}

pub enum TestKind {
    Unit,
    Integration,
    System,
    Performance,
    Security,
}

pub struct TestResult {
    pub test_id: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub output: String,
    pub error: Option<String>,
}

pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Timeout,
    Error,
}

pub struct TestReport {
    pub timestamp: DateTime<Utc>,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration: Duration,
    pub results: Vec<TestResult>,
}
```

**Public Methods:**

```rust
impl SelfTest {
    pub fn new() -> Self;
    pub fn add_test(&mut self, test: TestDefinition);
    pub fn run(&mut self) -> TestReport;
    pub fn run_test(&mut self, test_id: &str) -> TestResult;
    pub fn results(&self) -> &[TestResult];
    pub fn report(&self) -> TestReport;
}
```

---

## Related Modules

| Module | Description |
|--------|-------------|
| `nt_repair_facade` | Repair facade interface |
| `nt_repair_self_heal` | Self-healing implementation |
| `nt_repair_causal_trace` | Causal trace analysis |
| `nt_repair_hanzi_video` | Hanzi video repair |
| `nt_mind_consciousness_gold_standard` | Consciousness gold standard |
| `nt_mind_consciousness_monitor` | Consciousness monitoring |
| `nt_mind_eval_harness` | Evaluation harness |
| `nt_core_self_test` | Self-test |
| `nt_core_self_test_integration` | Integration test |

---

## Usage Examples

### Diagnostic Chain

```rust
use neotrix_core::l6_meta::healing::diagnostic_chain::DiagnosticChain;

let mut chain = DiagnosticChain::new();
chain.add_link(DiagnosticLink {
    id: "link1".into(),
    cause: "High memory usage".into(),
    effect: "OOM error".into(),
    evidence: vec!["memory_usage > 90%".into()],
    confidence: 0.9,
    timestamp: Utc::now(),
});

let report = chain.analyze();
println!("Root cause: {}", report.root_cause);
```

### Self-Healing

```rust
use neotrix_core::l6_meta::healing::self_healing::SelfHealingEngine;

let mut engine = SelfHealingEngine::new(HealingConfig {
    max_retries: 3,
    retry_delay: Duration::from_secs(5),
    cooldown: Duration::from_secs(60),
    auto_heal: true,
});

engine.add_rule(HealingRule {
    id: "restart_on_error".into(),
    trigger: HealingTrigger::ErrorRate { threshold: 0.1 },
    action: HealingAction::Restart,
    priority: 1,
    enabled: true,
});

let event = engine.trigger(&HealingTrigger::ErrorRate { threshold: 0.15 })?;
```

### Predictive Maintenance

```rust
use neotrix_core::l6_meta::healing::predictive_maintenance::PredictiveMaintenance;

let mut pm = PredictiveMaintenance::new();
let predictions = pm.predict(&SystemMetrics {
    cpu_usage: 0.85,
    memory_usage: 0.92,
    disk_usage: 0.75,
    network_io: 1024.0,
    error_rate: 0.05,
    latency_p99: 150.0,
});

for prediction in &predictions {
    println!("{}: {} (confidence: {})", prediction.kind, prediction.recommendation, prediction.confidence);
}
```

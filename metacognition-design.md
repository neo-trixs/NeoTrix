# NeoTrix MetaCognition System — Self-Awareness Architecture

> **File**: `neotrix-core/src/core/metacognition/`
> **Layer**: Core (zero external runtime dependencies, `std`-only)
> **Status**: Implemented, tested, integrated into `core/mod.rs` re-exports

---

## 1. Why MetaCognition

A self-evolving code system cannot improve what it does not measure. MetaCognition is the system's capacity to observe its own structure, detect its own weaknesses, and plan its own evolution — without human intervention.

The NeoTrix MetaCognition module implements **computational self-awareness** as a first-class architectural primitive. It is the "self-image" layer: the system's answer to "what do I look like, what is wrong with me, and what should I do next?"

### Literature Grounding

The design is informed by five lines of metacognition research:

| Paper | Key Idea | Application in NeoTrix |
|-------|----------|----------------------|
| **CoT2-Meta** (arXiv:2603.28135) | Budgeted metacognitive control — LLMs allocate reasoning based on task difficulty | `MetaCognitiveLoop` caps iterations; `max_iterations` enforces budget |
| **Self-Improving Coding Agent** (arXiv:2504.15228) | Self-play scaffolding for code generation agents; self-critique as training signal | `WeaknessAnalyzer` as static self-critique; `EvolutionPlanner` as improvement loop |
| **Courchaine & Sethi** (TheWebConf 2026) | Metacognitive State Vector — explicit encoding of uncertainty and confidence | `SelfModel` as the metacognitive state; `HealthTrend` tracks state over time |
| **MC²** (arXiv:2604.17399) | Inner-outer loop consolidation — metacognitive reflection drives outer-loop policy update | `run_cycle()` = inner loop (scan-analyze); `EvolutionPlanner` = outer loop (plan-act-record) |
| **DGM-Hyperagents** (arXiv:2603.19461) | Metacognitive self-modification via hyperagent architecture | `SelfModel.modules` + `ComponentMap` = self-awareness; `WeaknessAnalyzer` triggers self-modification |
| **Introspection** (arXiv:2603.20276) | Linearly-decodable metacognitive states from neural activations | `MetaCycleResult` is a fully-decodable snapshot of system state at any cycle |
| **MARS** (arXiv:2601.11974) | Principle-based + procedural reflection for agentic systems | `TechDebtItem.suggested_action` = procedural fix; `Weakness.impact` = principle-level rationale |

---

## 2. Core Architecture

The metacognition system is composed of **6 modules** in a layered data-flow architecture:

```
┌──────────────────────────────────────────────────────────────────────┐
│                    MetaCognitiveLoop (orchestrator)                   │
│                                                                      │
│   ┌─────────┐    ┌─────────────┐    ┌──────────┐    ┌───────────┐   │
│   │ SCAN    │───▶│ ANALYZE     │───▶│ MONITOR  │───▶│ PLAN      │   │
│   │ Scanner │    │ Weakness    │    │ MetaMon  │    │ Evolution  │   │
│   │         │    │ Analyzer    │    │ itor     │    │ Planner   │   │
│   └────┬────┘    └──────┬──────┘    └────┬─────┘    └─────┬─────┘   │
│        │                │                │                │         │
│        ▼                ▼                ▼                ▼         │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │                    SelfModel (state)                         │   │
│   │  modules · files · dep_graph · component_map · test_cov     │   │
│   │  compilation · tech_debt · evolution_history                │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │               REPORT → MetaCycleResult                       │   │
│   │  health_check + weakness_report + alerts + plans + trend     │   │
│   └─────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **SCAN** → `CodeScanner` reads the filesystem, populates `SelfModel` with modules, files, dependency graph
2. **ANALYZE** → `WeaknessAnalyzer` applies 7+ pattern detectors to `SelfModel`, produces `WeaknessReport`
3. **MONITOR** → `MetaMonitor` converts weaknesses to `MetaAlert`s, runs health checks, computes `HealthTrend`
4. **PLAN** → `EvolutionPlanner` sorts weaknesses by severity, batches them by priority, estimates impact
5. **REPORT** → All data is packaged into `MetaCycleResult` for external consumers

### Dependency Flow (module-level)

```
metacognition.rs (mod.rs: re-exports)
├── self_model.rs (no intra-module deps)
├── scanner.rs (depends on: self_model)
├── weakness.rs (depends on: self_model)
├── monitor.rs (depends on: self_model, weakness)
├── planner.rs (depends on: self_model, weakness)
└── metacognition_loop.rs (depends on: self_model, weakness, monitor, planner)
```

---

## 3. SelfModel — The Self-Image

`SelfModel` is the central data structure — the system's complete answer to "what am I?"

```rust
pub struct SelfModel {
    pub timestamp: DateTime<Utc>,        // When this snapshot was taken
    pub modules: Vec<ModuleInfo>,        // Known modules
    pub files: Vec<FileInfo>,            // All .rs files, sorted by line count desc
    pub dep_graph: DepGraph,             // Module dependency edges
    pub component_map: ComponentMap,     // High-level component/layer architecture
    pub test_coverage: TestCoverage,     // Test statistics
    pub compilation: CompilationHealth,  // Compilation error/warning counts
    pub tech_debt: TechDebtInventory,    // Accumulated tech debt items
    pub evolution_history: Vec<EvolutionEvent>,  // Timeline of changes
}
```

### Sub-structures

**ModuleInfo** — Per-module metrics:
- `name`, `path`, `file_count`, `total_lines`, `test_count`, `has_tests`
- `unsafe_count`, `unwrap_count`, `todo_count`, `public_api_count`

**FileInfo** — Per-file detail:
- `path`, `module`, `lines`, `is_test_file`, `has_unsafe`, `has_todos`, `pub_fns`, `last_modified`

**DepGraph** — Directed dependency graph:
- `edges: Vec<DepEdge>` with `from`, `to`, `kind` (ModuleUse/TraitImpl/FunctionCall)
- `find_cycles()` — DFS-based cycle detection, returns all dependency cycles
- `orphans()` — Modules that nothing depends on (sink nodes only)

**ComponentMap** — Layered architecture view:
- `nodes: Vec<ComponentNode>` with `name`, `path`, `layer` (1–5), `file_count`, `lines`
- `find_orphan_components()` — Nodes with zero edges
- `find_hubs(threshold)` — Nodes with degree > threshold

**TestCoverage**:
- `total_tests`, `passing`, `failing`, `ignored`
- `modules_with_tests`, `modules_without_tests`

**CompilationHealth**:
- `errors`, `warnings`, `features_tested: Vec<String>`

**TechDebtInventory**:
- `items: Vec<TechDebtItem>` — each with `file`, `line`, `kind`, `description`, `severity`, `suggested_action`
- 9 `TechDebtKind` variants: UnwrapCall, LargeFile, MissingTests, UnsafeBlock, DeadCode, TodoComment, CircularDependency, OrphanModule, LargePublicApi
- 4 `DebtSeverity` levels: Critical(3) > Major(2) > Minor(1) > Cosmetic(0)

**EvolutionEvent** — Immutable audit trail:
- `timestamp`, `kind` (ModuleAdded/Refactored/BugFixed/FeatureAdded/TechDebtResolved/WeaknessDetected/EvolutionPlanned/MetaCognitionUpdated)
- `description`, `affected_modules`

---

## 4. Weakness Detection — 7 Pattern Detectors

`WeaknessAnalyzer` applies configurable static analysis patterns to the `SelfModel`:

| # | Detector | Pattern ID | Threshold | Severity | Trigger |
|---|----------|------------|-----------|----------|---------|
| 1 | Large Files | `LARGE_FILE` | >800 lines | Minor | Reduced maintainability |
| 2 | Missing Tests | `MISSING_TESTS` | >300 lines, no tests | Major | Unverified changes |
| 3 | Excess Unsafe | `EXCESS_UNSAFE` | >5 blocks | Critical | Memory safety risk |
| 4 | Excess Unwrap | `EXCESS_UNWRAP` | >20 calls | Major | Runtime panic risk |
| 5 | TODO Leftovers | `TODO_LEFTOVERS` | >3 markers | Minor | Incomplete features |
| 6 | Orphan Modules | `ORPHAN_MODULE` | Zero dependents | Major | Dead code |
| 7 | Circular Deps | `CIRCULAR_DEP` | Any cycle | Critical | Tight coupling |
| 8 | Test Gaps | `TEST_GAP` | pub APIs >0, tests =0 | Minor | Untested API surface |
| 9 | Debt Accumulation | `DEBT_ACCUMULATION` | Critical items >10 | Critical | Long-term maintainability |

Each detector produces a `Weakness` with:
- `pattern_id` — Machine-readable identifier for filtering/routing
- `file`, `line` — Optional location context
- `severity` — DebtSeverity level
- `description` — Human-readable explanation
- `impact` — Why this matters (risk narrative)
- `suggestion` — Actionable fix recommendation

### Tech Debt Bridge

`WeaknessAnalyzer::to_tech_debt_items()` converts any `WeaknessReport` into `TechDebtItem` records storable in `SelfModel.tech_debt`, enabling the audit trail to persist across cycles.

---

## 5. Evolution Planning

`EvolutionPlanner` converts weaknesses into actionable evolution plans:

```
WeaknessReport
    │
    ▼
plan_from_report() ───► PlannedEvolution[]
    │                        │
    │                   ┌────┴────┐
    │                   ▼         ▼
    │              sort by:   group by:
    │              severity   priority
    │              + impact   (batch_size)
    │                   │         │
    │                   ▼         ▼
    │              PlannedEvolution {
    │                  id: "EVO-1",
    │                  priority: 1,
    │                  weakness: ...,
    │                  action: "fix it",
    │                  estimated_impact: ImpactEstimate {
    │                      files_affected: 5,
    │                      risk: High,
    │                  },
    │                  dependencies: [],
    │              }
    │
    ▼
next_batch() ───► Vec<&PlannedEvolution>  (top priority, ≤ max_concurrent)

record_action() ───► EvolutionAction + EvolutionEvent
```

### Impact Estimation

Each weakness type has a predefined impact estimate:

| Pattern | Files Affected | Risk Level |
|---------|---------------|------------|
| `CIRCULAR_DEP` | 5 | High |
| `EXCESS_UNSAFE` | 3 | High |
| `EXCESS_UNWRAP` | 10 | Medium |
| `MISSING_TESTS` | 2 | Low |
| `ORPHAN_MODULE` | 1 | Low |
| `LARGE_FILE` | 1 | Low |

### Batch Scheduling

- `max_concurrent` (default: 3) limits parallel work items
- Plans are sorted by `(severity, files_affected)` — critical + high-impact first
- `priority` is computed as `index / max_concurrent`, grouping work into waves
- Completed plans are removed from the queue; `completion_rate()` tracks progress

---

## 6. Monitoring & Alerts

`MetaMonitor` provides continuous health surveillance:

### Alert Generation

`generate_alerts()` checks for 5 conditions:
1. **Large modules** (>1500 lines → Warning)
2. **Untested modules** (>300 lines + no tests → Warning)
3. **Excessive unsafe** (>5 blocks → Critical)
4. **Large files** (>800 lines → Info)
5. **Critical tech debt** (>10 items → Warning)
6. **Compilation failures** across feature combinations → Critical

### Health Check

`run_check()` records a timestamped `HealthCheck` with:
- `compilation_ok`, `test_count`, `weakness_count`, `alert_count`

### Trend Analysis

`trend_analysis()` compares the first and last health checks to classify overall trajectory:
- `"improving"` — compilation OK + test count not regressing
- `"stable"` — compilation OK, tests may have regressed
- `"regressing"` — compilation broken
- `"insufficient_data"` — fewer than 2 checks

### Weakness-to-Alert Bridge

`weaknesses_to_alerts()` converts any `WeaknessReport` into `MetaAlert` records, mapping severity levels:
- Critical → `AlertSeverity::Critical`
- Major → `AlertSeverity::Warning`
- Minor/Cosmetic → `AlertSeverity::Info`

---

## 7. MetaCognitive Loop — The 5-Phase Cycle

`MetaCognitiveLoop` orchestrates the complete metacognition workflow:

### Phase: SCAN
- `CodeScanner.scan()` walks the filesystem
- Populates `SelfModel` with modules, files, dep graph, component map
- Runtime: `std::fs` only — no async I/O in core layer

### Phase: ANALYZE
- `WeaknessAnalyzer.analyze(&self_model)` runs all 7+ detectors
- Updates `self_model.tech_debt` with found items
- Produces `WeaknessReport` with summary counts

### Phase: MONITOR
- `MetaMonitor.weaknesses_to_alerts()` converts weaknesses to alerts
- `MetaMonitor.run_check()` records health snapshot
- `MetaMonitor.trend_analysis()` computes trajectory

### Phase: PLAN
- `EvolutionPlanner.plan_from_report()` generates prioritized `PlannedEvolution` list
- Batch scheduling groups plans by priority wave

### Phase: REPORT
- Aggregates all data into `MetaCycleResult`
- Registers `MetaCognitionUpdated` evolution event
- Returns structured output for external consumers

### Cycle Control

```rust
// Run one cycle
let result = loop.run_cycle();

// Run N cycles
let results = loop.run_batch(5);

// Run until condition
let results = loop.run_until(|r| r.iteration >= 10);

// Reset with new model
loop.reset(new_model);

// Quick status
println!("{}", loop.status_summary());
// Output: "MetaCognition Cycle 3/100 | 12 weaknesses | 5 alerts | 3 plans pending | trend: improving"
```

---

## 8. MetaCycleResult — The Output Contract

Every cycle produces a complete, self-describing result:

```rust
pub struct MetaCycleResult {
    pub iteration: usize,             // Which cycle number
    pub health_check: HealthCheck,     // Point-in-time health snapshot
    pub report: WeaknessReport,        // All detected weaknesses
    pub alerts: Vec<MetaAlert>,        // Active alerts
    pub plans: Vec<PlannedEvolution>,  // Prioritized evolution plan
    pub trend: HealthTrend,            // Health trajectory
    pub model_snapshot: SelfModel,     // Full self-image at this moment
}
```

This is the **linearly-decodable metacognitive state** (cf. Introspection arXiv:2603.20276): any downstream system can read the full metacognitive state at any cycle without needing to recompute it.

---

## 9. LLM Distillation Bridge

The metacognition system is designed for LLM-driven distillation pipelines (ollama/llama.cpp):

### Pattern 1: Weakness → Improvement Proposal

```
WeaknessReport ──► LLM ──► struct {
    root_cause: String,
    proposed_fix: String,
    estimated_effort: String,
    cross_references: Vec<String>,
}
```

The `Weakness.pattern_id` acts as a classification head. An LLM can read the weakness report and generate natural-language improvement proposals that are more detailed than the static `suggestion` field.

### Pattern 2: SelfModel → Architecture Summary

```
SelfModel ──► LLM ──► ArchitectureDocument {
    layers: Vec<LayerSummary>,
    hotspots: Vec<Hotspot>,
    recommendations: Vec<Recommendation>,
}
```

The full `SelfModel` can be serialized and fed to an LLM for higher-level architectural analysis — identifying patterns the static detectors cannot see (e.g., "Module X and Y share an implicit protocol that should be formalized").

### Pattern 3: Cross-Project Learning

```
SelfModel(project A) + SelfModel(project B) ──► LLM ──► PatternTransfer {
    shared_anti_patterns: Vec<AntiPattern>,
    knowledge_gaps: Vec<Gap>,
    suggested_absorb_sources: Vec<KnowledgeSource>,
}
```

When NeoTrix is applied to multiple repositories, the metacognitive state of each can be compared to discover shared weaknesses and cross-project learning opportunities.

---

## 10. Integration Points

### AGENTS.md — Mandatory Pre-Action Check

Item 11 of the mandatory checks requires `core/metacognition/` self-check before any action:

> 启动或修改代码前，运行 `core/metacognition/` 的自检——当前项目有多少 modules？多少文件？哪些模块缺少测试？哪些文件过大？编译状态？技术债分布？

This ensures every code modification is preceded by metacognitive awareness — the system knows its state before it acts.

### core/consciousness/ — Attention Routing

`GlobalWorkspace` in `core/consciousness/workspace.rs` can register metacognitive findings as `SpecialistModule` instances. Weaknesses with high severity trigger broadcast events in the global workspace, routing system attention to critical areas:

```
WeaknessAnalyzer ──► MetaAlert(Critical) ──► GlobalWorkspace.broadcast()
                                                     │
                                                     ▼
                                            SpecialistModules react
                                            (planning, code gen, etc.)
```

### core/hypercube/ — Knowledge Encoding

`GapReport` in `core/hypercube/gap.rs` shares the gap-analysis paradigm with metacognition. The hypercube's `sparsity_score` and `empty_regions` can be cross-referenced with `WeaknessAnalyzer` findings to produce a unified gap matrix — combining knowledge gaps (hypercube) with structural weaknesses (metacognition):

```
MetaCognitiveLoop.report ──► GapReport.dim_index = weakness.severity
                                        .gap = weakness_count / threshold
                                        .sparsity_score = untested_modules / total_modules
```

### reasoning_brain/ — SEAL Loop Driver

The SEAL (Self-Editing Autotelic Loop) uses `MetaCognitiveLoop` as its planning input:

1. `MetaCognitiveLoop` runs a cycle → produces `PlannedEvolution` list
2. `SelfIteratingBrain` consumes the highest-priority plan
3. `generate_self_edit()` translates the plan into `MicroEdit` instructions
4. `absorb()` applies the edits, updates capability vectors
5. Next metacognition cycle measures whether the edit improved health

This creates a closed loop: awareness → planning → action → measurement → awareness.

### Orchestrator

The `Orchestrator` in `reasoning_brain/` can use `MetaCycleResult` as input to its `PlannerNode`:
- `PlannerNode` reads the weakness report to decompose work items
- `WorkerNode` executes the `planned_evolution.action`
- `CriticNode` evaluates the result using the next cycle's health check delta

---

## 11. Usage Examples

### Basic — Scan and Analyze

```rust
use neotrix_core::core::metacognition::*;

let scanner = CodeScanner::new("/path/to/project");
let model = scanner.scan();

let analyzer = WeaknessAnalyzer::new();
let report = analyzer.analyze(&model);

println!("Found {} weaknesses", report.summary.total_count);
println!("  Critical: {}", report.summary.critical_count);
println!("  Major:    {}", report.summary.major_count);
```

### Monitoring Cycle

```rust
let mut monitor = MetaMonitor::new(model);
let alerts = monitor.generate_alerts();
let check = monitor.run_check();
let trend = monitor.trend_analysis();

if trend.overall == "regressing" {
    eprintln!("⚠ Health regressing — intervention required");
}
```

### Full Metacognitive Loop

```rust
let scanner = CodeScanner::new("/path/to/project");
let model = scanner.scan();
let mut loop = MetaCognitiveLoop::new(model);
loop.max_iterations = 50;

let result = loop.run_cycle();
println!("{}", loop.status_summary());

// Automatic batch
let results = loop.run_batch(5);

// Condition-based
let results = loop.run_until(|r| {
    r.report.summary.critical_count == 0
});
```

### Planning and Evolution

```rust
let analyzer = WeaknessAnalyzer::new();
let report = analyzer.analyze(&model);

let mut planner = EvolutionPlanner::new();
planner.max_concurrent = 2;
let plans = planner.plan_from_report(&report);

let batch = planner.next_batch();
for plan in &batch {
    println!("{}: {} [risk={:?}]", plan.id, plan.action, plan.estimated_impact.risk);
    planner.record_action(&plan.id, ActionStatus::InProgress, &mut model);
    // ... execute evolution ...
    planner.record_action(&plan.id, ActionStatus::Completed, &mut model);
}
```

### Tech Debt Inventory

```rust
let critical_items = model.tech_debt_by_severity(DebtSeverity::Critical);
for item in &critical_items {
    println!("{:?} in {}: {}", item.kind, item.file, item.suggested_action);
}
```

---

## 12. Testing

The module has 30+ unit tests across all 6 files:

| Module | Tests | Coverage |
|--------|-------|----------|
| `self_model.rs` | 5 | DepGraph cycles/orphans, ComponentMap hubs, DebtSeverity ordering, evolution events |
| `scanner.rs` | 3 | Module path guessing, component map layers, nonexistent path handling |
| `monitor.rs` | 4 | Alert generation, trend analysis (insufficient + with history), severity ordering |
| `weakness.rs` | 7 | All 5 detectors + report summary + tech debt conversion |
| `planner.rs` | 5 | Prioritization, batching, completion recording, impact estimation, empty rate |
| `metacognition_loop.rs` | 6 | Single cycle, batch, run-until, status summary, reset, max iterations |

---

## 13. Future Directions

### Self-Modifying Metacognition

The module currently uses hardcoded thresholds (800 lines, 5 unsafe blocks, etc.). A future iteration could:
- Track which thresholds consistently produce false positives/negatives
- Adjust thresholds based on historical `EvolutionEvent` outcomes
- Allow the LLM bridge to generate new detector patterns at runtime

### Cross-Project Learning

When NeoTrix manages multiple codebases, the metacognition system can:
- Compare `SelfModel` snapshots across projects to identify shared anti-patterns
- Transfer `EvolutionPlanner` configurations (thresholds, impact estimates) between projects
- Build a meta-metacognition: "what do my weaknesses tell me about the kinds of codebases I tend to build?"

### Automated Tech Debt Fixing

The gap between `Weakness.suggestion` and actual code modification is currently manual. Future work:
- Generate `MicroEdit` sequences directly from `PlannedEvolution` items
- Apply edits, run `cargo check`, measure compilation health delta
- If health improves, `absorb()` the edit; if not, roll back and replan

### Real-Time Monitoring

The current `run_check()` is explicit. A future version could:
- Run in background (async, via `reasoning_brain/`)
- Emit `MetaAlert` on a channel/subscription system
- Reactively adjust `max_concurrent` based on `HealthTrend`

### Integration with MCP Tools

`WeaknessAnalyzer` could invoke MCP tools for deeper inspection:
- `react_doctor` for React module health
- `security_audit` for security-specific weakness detection
- `web_scrape` to compare project metrics against community standards

---

## Appendix: Type Index

| Type | File | Purpose |
|------|------|---------|
| `SelfModel` | `self_model.rs` | Complete project self-image |
| `ModuleInfo` | `self_model.rs` | Per-module metrics |
| `FileInfo` | `self_model.rs` | Per-file detail |
| `DepGraph` | `self_model.rs` | Dependency graph with cycle/orphan detection |
| `DepEdge` | `self_model.rs` | Single dependency edge |
| `DepKind` | `self_model.rs` | ModuleUse / TraitImpl / FunctionCall |
| `ComponentMap` | `self_model.rs` | Layered architecture model |
| `ComponentNode` | `self_model.rs` | Component node with layer/line count |
| `TestCoverage` | `self_model.rs` | Test statistics |
| `CompilationHealth` | `self_model.rs` | Compilation error/warning counts |
| `TechDebtInventory` | `self_model.rs` | Collection of tech debt items |
| `TechDebtItem` | `self_model.rs` | Single tech debt record |
| `TechDebtKind` | `self_model.rs` | 9 debt categories |
| `DebtSeverity` | `self_model.rs` | Critical/Major/Minor/Cosmetic |
| `EvolutionEvent` | `self_model.rs` | Immutable audit trail entry |
| `EventKind` | `self_model.rs` | 8 event types |
| `CodeScanner` | `scanner.rs` | Filesystem scanner (std::fs) |
| `MetaMonitor` | `monitor.rs` | Continuous health monitor |
| `MetaAlert` | `monitor.rs` | Health alert with severity/suggestion |
| `AlertSeverity` | `monitor.rs` | Critical/Warning/Info |
| `HealthCheck` | `monitor.rs` | Point-in-time health snapshot |
| `HealthTrend` | `monitor.rs` | Trajectory classification |
| `WeaknessAnalyzer` | `weakness.rs` | 7+ pattern detector engine |
| `Weakness` | `weakness.rs` | Single weakness finding |
| `WeaknessReport` | `weakness.rs` | Complete analysis output |
| `WeaknessSummary` | `weakness.rs` | Severity-level counts |
| `EvolutionPlanner` | `planner.rs` | Priority queue + batch scheduler |
| `PlannedEvolution` | `planner.rs` | One evolution action plan |
| `ImpactEstimate` | `planner.rs` | Files affected + risk level |
| `RiskLevel` | `planner.rs` | Low/Medium/High |
| `EvolutionAction` | `planner.rs` | Action record with status |
| `ActionStatus` | `planner.rs` | InProgress/Completed/Blocked |
| `MetaCognitiveLoop` | `metacognition_loop.rs` | 5-phase cycle orchestrator |
| `MetaCycleResult` | `metacognition_loop.rs` | Complete cycle output |

---

*Design document v1.0 — reflects implementation in `neotrix-core/src/core/metacognition/` as of 2026-05-24.*

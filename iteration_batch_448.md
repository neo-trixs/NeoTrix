# Iteration Batch 448 — Scheduling, Timetabling & Workflow Orchestration

**Date**: 2026-09-05
**Focus**: Job scheduling, constraint satisfaction / SAT solving, workflow orchestration / DAG scheduling

---

## Sources Cited

| # | Source | Date | Domain |
|---|--------|------|--------|
| S1 | Nature: "An intelligent job scheduling and real-time resource optimization for edge-cloud continuum" (s41598-025-25452-z) | 2025-11 | Job scheduling — EDF/EDZL/USG algorithms, Deadline Lookahead Module, Urgent Queue |
| S2 | Nature: "Quantum-Inspired Adaptive Meta-Heuristic–ML for multi-cloud orchestration" (s41598-026-43125-3) | 2026-03 | Quantum-inspired scheduling, hyper-distributed cross-platform orchestration |
| S3 | IEEE-RAS TASE 2026 Special Issue CFP | 2026-02 | AI-driven scheduling algorithms, multi-objective/multi-task scheduling, LLM-assisted scheduling |
| S4 | Applying AI: "AI-Driven Project Management in 2026" | 2026-04 | Autonomous replanning loops (RL agents), constraint-based optimization (CPLEX/OR-Tools), burnout risk scoring |
| S5 | Glean: "How AI can enhance scheduling and resource allocation" | 2026-03 | Predicted service time, no-show propensity, failure risk from maintenance histories |
| S6 | IJRASET 2026: "Configurable architecture for university timetabling" | 2026-05 | DSL-based constraint definition, CP-SAT solver, hybrid LLM + constraint satisfaction |
| S7 | SAT Competition 2026: "School Timetabling Benchmarks" (satres.kit.edu) | 2026 | Challenging SAT instances for timetabling, missing constraint types (workloads, split courses) |
| S8 | GitHub: e-cagan/csp-exam-timetabling (2026-03) | 2026-03 | CSP/CSOP formulation, CP-SAT with AC-3/MRV/LNS, multi-room splitting, dual-role conflict modeling |
| S9 | GitHub: well-documented-readme-timetabling-system (2026-04) | 2026-04 | Parallel CP-SAT with multi-config racing (ProcessPoolExecutor), pluggable constraint modules |
| S10 | GitHub: huguryildiz/course-timetabling (2026-06) | 2026-06 | Real university data, section+instructor+room assignment via CP-SAT |
| S11 | Zylos Research: "Agent Workflow Orchestration Patterns: DAG, Event-Driven, and Actor Models" | 2026-04 | Three orchestration schools: DAG, event-driven, actor model; durable execution; difficulty-aware routing |
| S12 | Airflow 3.3.1 Documentation (stable) | 2026 | DAG-based orchestration, Task Isolation, Event-Driven Workflows |
| S13 | Automation Atlas: "Apache Airflow Review 2026" | 2026-07 | Scheduler bottleneck at scale, no native streaming, 46K+ stars, multi-scheduler instances |
| S14 | AcmeMinds: "Airflow DAG Best Practices for 2026" | 2026-04 | Idempotency, retry strategies, inter-task XCom, performance optimization |
| S15 | OneUptime: "How to Fix DAG Scheduling Airflow Issues" | 2026-01 | DAG serialization, scheduler loop debugging, missed runs |
| S16 | Startupik: "Airflow Deep Dive 2026" | 2026-07 | KubernetesExecutor, event-aware patterns, orchestration vs compute separation |
| S17 | Nature: "Reinforcement learning based multi objective task scheduling for energy efficient cloud edge computing" | 2025-11 | RL multi-objective scheduling, energy efficiency |
| S18 | Springer: "Machine learning in project schedule creation" (J. Scheduling) | 2025-11 | ML for activity sequencing, resource determination, duration estimation |

---

## Defects Found

### DEFECT-448-1: Scheduling Algorithms Declared but Not Implemented

**Location**: `nt_act/parallel_task.rs:122-135`

The `SchedulingAlgorithm` enum declares five algorithms (FCFS, SJF, Priority, RoundRobin, MLFQ) but the `schedule_next()` method at line 218 only implements a naive linear scan with priority sort. No actual SJF, RoundRobin, or MLFQ logic exists.

**Evidence from research**: S3 (IEEE TASE 2026) highlights "AI-driven multi-objective scheduling" and "AI-guided meta-heuristics" as core 2026 advances. S1 (Nature) uses EDF/EDZL algorithms with deadline lookahead. S4 shows RL agents continuously recalibrating.

**Gap**: The enum creates a false interface — callers can select `SchedulingAlgorithm::MLFQ` but get priority-only behavior. This violates the "Dark Forest" axiom (compiled + tested + connected).

**Suggestion**: Either implement the algorithms (MLFQ requires per-queue aging; SJF needs duration estimation; RoundRobin needs time-slicing) or remove the enum and document the actual algorithm (priority-scan with GPU affinity).

---

### DEFECT-448-2: No Deadline-Aware Scheduling (EDF/EDZL)

**Location**: `nt_act/parallel_task.rs` (entire module), `nt_core_scheduler/engine.rs`

No task carries a deadline field. No earliest-deadline-first or deadline-zero-laxity logic exists. The `estimated_duration_secs` field (line 59) is declared but never consulted during scheduling.

**Evidence from research**: S1 demonstrates that EDF (Earliest Deadline First) and EDZL (Earliest Deadline first with Zero Laxity) are critical for real-time cloud workloads. The Deadline Lookahead Module predicts deadline violations and routes tasks to an Urgent Queue. S17 uses RL for multi-objective scheduling including latency.

**Gap**: NeoTrix's AI inference tasks (video generation, batch production) have implicit deadlines (SLA). Without deadline awareness, the scheduler cannot prioritize near-deadline tasks, leading to SLA violations.

**Suggestion**: Add `deadline_ts: Option<u64>` to `Task`/`SchedulingTask`. Implement EDF as a scheduling algorithm variant. Add a Deadline Lookahead Module that preempts tasks predicted to miss deadlines.

---

### DEFECT-448-3: No Constraint Satisfaction / SAT Solver Integration

**Location**: No file — missing entirely from the codebase

The timetabling domain (S6, S7, S8, S9, S10) shows that modern scheduling is increasingly driven by CP-SAT solvers (OR-Tools) and SAT/MaxSAT formulations. NeoTrix has no constraint solver abstraction.

**Evidence from research**: S6 describes a production-grade Timetable Optimizer combining CP-SAT with LLM intelligence. S7 introduces new SAT benchmarks with missing constraint types. S8 implements 6 hard + 5 soft constraints via CP-SAT. S9 demonstrates parallel CP-SAT with multi-config racing.

**Gap**: When NeoTrix needs to solve combinatorial scheduling (e.g., assigning agents to tasks across time slots with resource constraints), it falls back to ad-hoc heuristics. A CP-SAT integration would provide provably optimal or near-optimal solutions for structured scheduling problems.

**Suggestion**: Create `nt_core::constraint_solver` wrapping Google OR-Tools CP-SAT bindings (or a pure-Rust SAT solver like `picosat` bindings). Define a `ConstraintModel` trait with `add_variable`, `add_constraint`, `minimize/maximize`, `solve`.

---

### DEFECT-448-4: Production Orchestrator Checkpoint is a Stub

**Location**: `nt_act/production_orchestrator.rs:224-232`

```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    // TODO: 实际保存检查点到持久化存储
    let _ = workflow;
    Ok(())
}
```

And `restore_from_checkpoint` (line 235) just sets status to Paused without restoring step state.

**Evidence from research**: S11 identifies "durable execution" as the defining production requirement of 2026: "OpenAI's VP stated: Durable Execution is a core requirement for modern AI systems." Temporal.io's core insight is separating orchestration code from activity code, with replays from event history. S13 notes Airflow's scheduler bottleneck requires checkpoint serialization.

**Gap**: If a workflow crashes mid-execution, all progress is lost. The `enable_checkpoint` config flag (line 82) exists but does nothing. This directly violates the SEAL pipeline's resilience requirements.

**Suggestion**: Implement checkpoint serialization to KB (`kv_store` namespace `workflow_checkpoint`). On restore, replay completed steps from checkpoint, re-register remaining steps, and resume from the first incomplete step.

---

### DEFECT-448-5: No Event-Driven / Streaming Workflow Support

**Location**: `nt_act/production_orchestrator.rs` (entire module)

The orchestrator is purely batch-oriented: create workflow → start → complete steps sequentially → finish. No support for event-driven triggers, streaming data flows, or reactive patterns.

**Evidence from research**: S11 identifies three orchestration schools: DAG (batch), event-driven (reactive), actor model (isolated state). S12 (Airflow 3.0) added Event-Driven Workflows and Task Isolation. S13 notes Airflow's main limitation is "no native streaming support" and recommends Prefect/Temporal for event-driven patterns. S16 shows 2026 trend toward "event-aware patterns instead of pure cron scheduling."

**Gap**: NeoTrix's NT-WORLD (crawler events), NT-ACT (external triggers), and NT-MIND (evolution events) produce event streams that need reactive workflow triggering. The current orchestrator cannot express "when X event arrives, trigger workflow Y."

**Suggestion**: Add an `EventTrigger` variant to `WorkflowStatus` or create a parallel `EventDrivenOrchestrator` that subscribes to EventBus events and instantiates workflows reactively. Integrate with the existing `nt_core::eventbus`.

---

### DEFECT-448-6: Resource Budget Manager is Single-Dimensional

**Location**: `nt_act/resource_budget.rs`

The `BudgetConfig` holds a flat `Vec<ResourceQuota>` with no cross-resource constraint modeling. A task consuming GPU memory doesn't check if its CPU quota is also available. There's no concept of "this task needs 4GB GPU AND 2 CPU cores AND $0.50 budget" as an atomic allocation.

**Evidence from research**: S4 describes constraint-based optimization using CPLEX/OR-Tools that "distribute tasks such that no individual is underutilized or overburdened" while accounting for "time zones, resource cross-training opportunities, and personal work preferences." S5 shows a pipeline: signal preparation → learned estimates → optimization with hard/soft constraints. S18 identifies multi-resource determination as a core ML scheduling subprocess.

**Gap**: Tasks declare `gpu_memory_mb` but not CPU, network, or cost constraints atomically. The budget check is per-resource-type independent, missing compound resource allocation (the "knapsack problem" across multiple resource dimensions).

**Suggestion**: Add `ResourceRequirements` struct with `HashMap<ResourceType, f64>` for multi-dimensional resource claims. Implement a compound allocation check that validates all dimensions atomically before admitting a task.

---

### DEFECT-448-7: No Burnout / Fatigue-Aware Scheduling

**Location**: `nt_core_scheduler/engine.rs:108-114`

The `ContextGate` enum includes `LowCogLoad` and `SleepPressure` but the actual scheduling modules (`parallel_task.rs`, `task_scheduler.rs`) have no equivalent fatigue awareness.

**Evidence from research**: S4 describes "burnout risk scores—computed via sentiment analysis of chat logs and email metadata—to proactively rebalance assignments and prevent fatigue." S5 includes "no-show propensity" and "failure risk from maintenance histories" as scheduling signals.

**Gap**: NeoTrix tracks cognitive load in the GWT layer but doesn't propagate fatigue signals to task scheduling. High cognitive load should defer non-critical inference tasks, but the parallel task scheduler has no awareness of system-level fatigue.

**Suggestion**: Bridge the `ContextGate` from `SchedulerEngine` into `ParallelTaskManager` — expose `cognitive_load: f64` as a scheduling input. When load exceeds threshold, defer Low/Critical priority tasks.

---

### DEFECT-448-8: No Difficulty-Aware Dynamic Task Routing

**Location**: No file — missing capability

**Evidence from research**: S11 describes "Difficulty-Aware Dynamic Routing: Rather than routing all tasks through the same pipeline depth, a classifier estimates query difficulty and allocates compute proportionally. Simple queries get a shallow chain; complex queries get a deep multi-agent pipeline."

**Gap**: NeoTrix routes all tasks through the same pipeline depth regardless of complexity. A simple text completion and a complex multi-step reasoning task consume the same orchestration overhead.

**Suggestion**: Add a `difficulty_estimate: f64` field to tasks. Use it to route simple tasks to a fast/shallow execution path and complex tasks to a deep multi-agent path. This maps directly to GWT's salience-based attention routing.

---

### DEFECT-448-9: Workflow Step Completion Doesn't Validate Dependencies

**Location**: `nt_act/production_orchestrator.rs:193-221`

`complete_step()` marks any step as completed regardless of whether its dependencies have been completed. There's no dependency validation.

**Evidence from research**: S11: "The scheduler automatically parallelises independent branches and serialises dependent ones." S14: "By defining dependencies and execution rules explicitly, teams can ensure data integrity." S8: CSP formulation includes hard constraints like "no room double-booked" — dependency violations are constraint violations.

**Gap**: A step with `dependencies: vec!["step_1".to_string()]` can be completed before step_1 finishes. This breaks data flow integrity and can produce incorrect outputs.

**Suggestion**: In `complete_step()`, validate that all `dependencies` are marked `completed` before allowing the step to complete. Return `Err("dependency not satisfied")` if violated.

---

### DEFECT-448-10: Cron Parser Doesn't Support Ranges or Steps

**Location**: `nt_core_scheduler/engine.rs:349-366`

`parse_cron_field` only supports `*` (all values) and comma-separated integers. Standard cron supports ranges (`1-5`), steps (`*/2`, `1-10/3`), and named day/month values.

**Evidence from research**: S12 (Airflow 3.3.1) relies on cron expressions for DAG scheduling. S14 emphasizes cron expression flexibility. S15 shows DAG scheduling issues often stem from incorrect cron expressions.

**Gap**: NeoTrix's scheduler cannot express "every 2 hours" (`*/2`), "Monday through Friday" (`1-5`), or "March and September" (`3,9`). This limits scheduling expressiveness compared to standard cron implementations.

**Suggestion**: Extend `parse_cron_field` to handle range syntax (`min-max`), step syntax (`value/step` or `*/step`). Add named month/day parsing (`jan`, `mon`).

---

## Summary

| Category | Defects | Severity |
|----------|---------|----------|
| Scheduling Algorithms | 448-1, 448-2 | High |
| Constraint Solving | 448-3 | High |
| Workflow Orchestration | 448-4, 448-5, 448-9 | Critical |
| Resource Management | 448-6, 448-7 | Medium |
| Dynamic Routing | 448-8 | Medium |
| Cron/Time | 448-10 | Low |

**Critical path**: DEFECT-448-4 (checkpoint stub) + DEFECT-448-9 (no dependency validation) = production workflows can lose state and violate data integrity. These should be addressed first.

**Research alignment gap**: The 2026 literature converges on three themes absent from NeoTrix: (1) durable execution with replay, (2) constraint satisfaction solvers for combinatorial scheduling, (3) event-driven reactive workflows. NeoTrix's architecture is solid (event-driven claim pool is world-class) but these three gaps limit production readiness.

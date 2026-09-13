# Targeted Research #560 — Consciousness Core Internal Dispatch Routes

**Date**: 2026-09-13
**File**: `neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs`
**Scope**: Add 4 internal dispatch routes to consciousness core capability routing

## Routes Added

### 1. `temporal_continuity` — Video Temporal Continuity Checking
- **Capability tag**: `temporal_continuity`
- **Keywords**: 时序连续, 帧间连续, 镜头衔接, temporal_continuity, shot_continuity
- **Domain**: NT-ACT
- **Implementation**: `crate::l1_action::nt_act::temporal_continuity::TemporalContinuityChecker`
- **API**: `new()` → `statistics()` → `ContinuityStats { total_checks, passed, failed, total_issues, avg_pass_rate }`
- **Source file**: `neotrix-core/src/l1_action/nt_act/temporal_continuity.rs`

### 2. `resource_budget` — Resource Budget Management
- **Capability tag**: `resource_budget`
- **Keywords**: 资源预算, 成本控制, token预算, resource_budget, cost_manager
- **Domain**: NT-ACT
- **Implementation**: `crate::l1_action::nt_act::resource_budget::ResourceBudgetManager`
- **API**: `new()` → `statistics()` → `BudgetStats { total_cost_usd, total_tokens, total_tasks, avg_cost_per_task, avg_tokens_per_task }`
- **Source file**: `neotrix-core/src/l1_action/nt_act/resource_budget.rs`

### 3. `parallel_task` — Parallel Task Scheduling
- **Capability tag**: `parallel_task`
- **Keywords**: 并行任务, 任务调度, gpu调度, parallel_task, task_scheduler
- **Domain**: NT-ACT
- **Implementation**: `crate::l1_action::nt_act::parallel_task::ParallelTaskManager`
- **API**: `new()` → `register_device(GPUDevice)` → `statistics()` → `SchedulerStats { queued_tasks, running_tasks, completed_tasks, failed_tasks, gpu_utilization }` + `calculate_backoff_delay(retry_count)`
- **Source file**: `neotrix-core/src/l1_action/nt_act/parallel_task.rs`

### 4. `checkpoint_persistence` — Checkpoint Persistence
- **Capability tag**: `checkpoint_persistence`
- **Keywords**: 检查点, 断点续传, 状态快照, checkpoint_persistence, checkpoint
- **Domain**: NT-ACT
- **Implementation**: `crate::l1_action::nt_act::actions::core::checkpoint_persistence::CheckpointPersistence`
- **API**: `new(storage_path)` → `statistics()` → `CheckpointStats { total_checkpoints, total_size_bytes, unique_workflows }`
- **Source file**: `neotrix-core/src/l1_action/nt_act/actions/core/checkpoint_persistence.rs`

## CAPABILITY_ROUTES Entries

20 new keyword→capability mappings added (5 per route: Chinese keywords + English identifier + alias).

## Match Arms

4 new match arms in `dispatch_internal_capability()` — each instantiates the module's manager/checker, calls `statistics()`, and returns a formatted diagnostic summary.

## Build Status

All 11 compilation errors are pre-existing (`LlmProvider`/`LlmRequest` in `persona_routing.rs`, `nt_core_event_bus` in `streaming.rs`). No new errors introduced by this change.

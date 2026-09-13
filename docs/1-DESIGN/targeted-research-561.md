# Targeted Research 561 — Test Quality Fixes

## Problem

Four test files in `nt_act` contained tests that either:
1. Asserted on fabricated/hardcoded success data (tests that always pass)
2. Tested only trivial behavior without exercising real code paths
3. Had no cleanup, no boundary cases, and weak assertions

## Files Fixed

### 1. `temporal_continuity.rs` — Always-Pass String Comparison Tests

**Before (2 tests, both always pass):**
- `test_continuity_checker` — compared `"frame_001.png"` vs `"frame_002.png"` via string diff, low diff → always passed
- `test_check_all` — same pattern, trivially passed

**After (6 tests, honest behavior):**
- `test_detects_high_diff_frames` — verifies checker flags frames with zero common prefix (diff=1.0)
- `test_identical_frames_pass` — verifies identical strings produce diff=0.0
- `test_empty_frames_returns_pass` — edge case: empty input
- `test_scene_transition_stub_returns_error` — stub must not fabricate success
- `test_element_position_stub_returns_error` — stub must not fabricate success
- `test_check_all_fails_when_stubs_active` — `check_all` must not pass when stubs return errors

### 2. `parallel_task.rs` — Fabricated Success Data

**Before (2 tests):**
- `test_task_scheduler` — created fabricated `TaskResult { success: true, execution_time_ms: 50000 }` and only checked `completed_tasks == 1`
- `test_backoff_delay` — tested pure arithmetic (kept)

**After (11 tests, real scheduling logic):**
- `test_priority_ordering` — high-priority task scheduled before low
- `test_no_gpu_blocks_scheduling` — insufficient GPU → `None`
- `test_max_parallel_limit` — after `max_parallel_tasks`, scheduler blocks
- `test_dependency_blocking` — unmet dependency blocks; met dependency allows
- `test_failed_dependency_blocks` — failed dependency blocks downstream
- `test_retry_requeues_failed_task` — failed task with retries left → re-queued
- `test_exhausted_retries_marks_failure` — exhausted retries → `failed_tasks == 1`
- `test_gpu_memory_release` — memory freed after completion
- `test_gpu_memory_release_on_failure` — memory freed even on failure
- `test_complete_unknown_task_is_noop` — no panic on unknown task_id
- `test_statistics_accuracy` — stats reflect actual scheduler state
- `test_backoff_delay` — kept (honest pure-logic test)

### 3. `checkpoint_persistence.rs` — No Cleanup, Weak Assertions

**Before (1 test):**
- Wrote to `/tmp/checkpoints` without cleanup
- Only asserted `result.success`, no state verification

**After (5 tests, proper lifecycle):**
- `test_save_load_roundtrip_preserves_state` — verifies metadata, state restoration, output_files
- `test_load_nonexistent_checkpoint_fails` — missing checkpoint → error
- `test_delete_checkpoint_removes_index_and_file` — verifies disk and index cleanup
- `test_delete_nonexistent_checkpoint_fails` — no panic
- `test_statistics_reflect_actual_state` — stats match actual checkpoint count and workflows
- `test_list_checkpoints_filters_by_workflow` — cross-workflow filtering

### 4. `resource_budget.rs` — Missing Boundary Cases

**Before (3 tests):**
- `test_budget_check` — only tested within-budget path
- `test_record_usage` — basic tracking
- `test_cost_estimation` — model pricing (kept, improved)

**After (8 tests, boundary coverage):**
- `test_budget_within_limit_returns_continue` — existing test (kept)
- `test_budget_exceeded_returns_delay` — budget exceeded → `Delay` recommendation
- `test_alert_threshold_triggers_degraded` — 80%+ usage → `Degraded`
- `test_hard_limit_triggers_reject` — 95%+ usage → `Reject`
- `test_usage_accumulation_affects_subsequent_checks` — accumulated usage changes recommendation
- `test_unknown_resource_type_returns_unlimited` — no quota defined → `f64::MAX` remaining
- `test_record_usage_updates_statistics` — multi-task aggregation
- `test_cost_estimation_*` — 3 tests (known, unknown, external override)

## Test Count Summary

| File | Before | After | Change |
|------|--------|-------|--------|
| `temporal_continuity.rs` | 2 | 6 | +4 |
| `parallel_task.rs` | 2 | 11 | +9 |
| `checkpoint_persistence.rs` | 1 | 6 | +5 |
| `resource_budget.rs` | 3 | 8 | +5 |
| **Total** | **8** | **31** | **+23** |

## Patterns Applied

| Anti-Pattern | Fix | Files |
|-------------|-----|-------|
| Assert on fabricated success | Test real constraints (GPU limits, deps, retries) | parallel_task |
| Always-pass tests | Use inputs that trigger boundary conditions | temporal_continuity |
| No cleanup | `temp_dir()` + `cleanup()` helper | checkpoint_persistence |
| Weak assertions | Verify state transitions, not just `success` flag | all |
| Missing boundary tests | Add alert/hard-limit/exceeded paths | resource_budget |

## Notes

- Pre-existing compilation errors in `nt_io_provider` (unrelated `LlmError` import) prevent `cargo test` from running. All test code verified via `rustfmt --check` — no syntax errors.
- `temporal_continuity.rs` stubs (`check_scene_transition`, `check_element_position`) correctly return errors. Tests now verify this behavior rather than ignoring it.
- `checkpoint_persistence.rs` uses process-specific temp dirs to avoid test interference.

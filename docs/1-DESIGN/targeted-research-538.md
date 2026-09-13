# Targeted Research #538 — Internal Pain Points: Hardcoded/Stub/Fake Data Fixes

**Date**: 2026-09-13  
**Scope**: neotrix-core/src/l1_action/nt_act/ + nt_io_video_stitcher  
**Method**: File-by-file audit → direct fix application

---

## Pain Points Found & Fixed

### 1. `temporal_continuity.rs` — Stub checks always return "passed" (HIGH)

**File**: `neotrix-core/src/l1_action/nt_act/temporal_continuity.rs`

| Function | Problem | Fix |
|----------|---------|-----|
| `check_scene_transition()` | Always returned `passed: true, issue_count: 0` with hardcoded `check_time_ms: 50` | Returns `passed: false` with explicit error message explaining what real implementation needs (perceptual hashing / DNN scene-change detection) |
| `check_element_position()` | Always returned `passed: true, issue_count: 0` with hardcoded `check_time_ms: 80` | Returns `passed: false` with explicit error message explaining what real implementation needs (ByteTrack / optical flow) |
| `check_all()` `_` match arm | Returned `passed: true` for unknown check types | Returns `passed: false` with descriptive error for unimplemented types |
| `check_time_ms` in `check_all` | Hardcoded `200` | Now uses `Instant::now()` elapsed time |
| `check_time_ms` in `check_first_last_frame` | Hardcoded `100` | Now uses `Instant::now()` elapsed time |
| `calculate_frame_diff()` | Operates on file path strings (not pixel data) | Doc comment added explaining it's a placeholder; real impl needs SSIM/LPIPS/histogram distance on image buffers |

**Impact**: Callers no longer silently trust fabricated "passed" results. `check_all` now propagates errors from unimplemented sub-checks.

### 2. `parallel_task.rs` — Dead retry logic (BUG) (HIGH)

**File**: `neotrix-core/src/l1_action/nt_act/parallel_task.rs`

**Problem**: `complete_task()` removed the task from `running_tasks` at line 261, then tried to remove it again at line 271 for the retry path. The second `.remove()` always returned `None`, making retry logic dead code — failed tasks were never re-queued.

**Fix**: Restructured `complete_task()` to remove the task exactly once, then branch on success/failure using the owned `Task` value. Added `tracing::warn` for calls targeting unknown task IDs.

**Impact**: Exponential-backoff retry now actually works. Previously, all failures went straight to `completed_tasks` regardless of `max_retries`.

### 3. `video_stitcher.rs` — Silent transition downgrade (MEDIUM)

**File**: `neotrix-core/src/l1_action/nt_act/actions/video/video_stitcher.rs`

**Problem**: `add_transition()` had a `_ =>` catch-all that silently mapped all unsupported `TransitionType` variants (Wipe, Push, Zoom, Cut) to CrossDissolve. Callers had no way to know their transition was downgraded.

**Fix**: 
- Changed return type from `String` to `Result<String, String>`
- Made all 6 `TransitionType` variants explicit with correct FFmpeg xfade filter names
- `Cut` returns an empty string (no filter needed) instead of a misleading xfade
- Unknown variants (if enum is extended) will produce a compile error instead of silent fallback

**Impact**: Callers must now handle the `Result`. No more silent quality degradation.

### 4. `resource_budget.rs` — Stale hardcoded model prices (MEDIUM)

**File**: `neotrix-core/src/l1_action/nt_act/resource_budget.rs`

**Problem**: `estimate_cost()` had hardcoded 2026 pricing for ~12 models. Unknown models silently returned `0.001` as a "conservative estimate" — a fake value that misleads budget tracking.

**Fix**:
- Added `external_prices: Option<&HashMap<String, f64>>` parameter — callers can pass fresh prices
- Built-in table retained as fallback with `#[allow(clippy::uninlined_format_args)]` and explicit stale warning in docs
- Unknown models now return `None` instead of a fake estimate
- Updated test to verify `None` for unknown models and `Some` for external price overrides

**Impact**: No more silently wrong cost estimates. Callers can provide live pricing from provider APIs.

---

## Files NOT Needing Fixes

| File | Assessment |
|------|-----------|
| `nt_act_rate_limiter.rs` | Token bucket and sliding window implementations are correct and complete. The `adaptive` config field is unused but the core algorithm is real. No fake data. |

---

## Summary Table

| # | File | Severity | Issue | Fix Applied |
|---|------|----------|-------|-------------|
| 1 | temporal_continuity.rs | HIGH | Stub checks return fake "passed" | Explicit errors + real timing |
| 2 | parallel_task.rs | HIGH | Retry logic is dead code (double-remove bug) | Single-remove + owned branch |
| 3 | video_stitcher.rs | MEDIUM | Silent transition downgrade | `Result` return + explicit match |
| 4 | resource_budget.rs | MEDIUM | Stale hardcoded prices + fake fallback | External price map + `None` for unknown |

## Remaining Work

- `check_scene_transition` and `check_element_position` need real implementations (perceptual hashing, object tracking)
- `calculate_frame_diff` should compare pixel data, not file path strings
- `estimate_cost` built-in price table needs periodic refresh or migration to a pricing API
- `adaptive` field in `RateLimiterConfig` is unused — needs adaptive rate-limiting logic

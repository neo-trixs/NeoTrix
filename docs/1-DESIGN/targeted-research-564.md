# Targeted Research 564 — Error Handling Improvements

**Date**: 2026-09-13
**Scope**: Replace `unwrap()` in non-test code with safe error propagation
**Files**: 11 files across `nt_io_provider`, `nt_media`, `nt_mind`

## Summary

Eliminated **40+** `unwrap()` calls in production code across 11 files. Zero new compilation errors introduced (pre-existing `cacheable_prefix_hash` errors remain unrelated).

## Fixes Applied

### 1. `capability_router.rs` — Empty collection unwrap
| Line | Before | After |
|------|--------|-------|
| 77 | `sorted.first().unwrap().name.to_string()` | `sorted.into_iter().next().ok_or_else(\|\| LlmError::Unknown(...))?.name.to_string()` |

**Impact**: Router now returns `Err` instead of panicking when no providers match.

### 2. `learned_router.rs` — RwLock poisoning + NaN partial_cmp
| Lines | Pattern | Fix |
|-------|---------|-----|
| 285 | `find(...).unwrap()` | `.expect("best_model was set from candidates loop")` |
| 298 | `self.history.write().unwrap()` | `.unwrap_or_else(\|e\| e.into_inner())` |
| 387 | `partial_cmp(&a.1).unwrap()` | `.unwrap_or(std::cmp::Ordering::Equal)` |
| 487 | `find(...).unwrap()` | `.expect("anchored model exists (checked by any())")` |
| 513, 542, 552 | `max_by(partial_cmp().unwrap())` | `.unwrap_or(std::cmp::Ordering::Equal)` |
| 584, 587 | sort/max_by with `partial_cmp().unwrap()` | `.unwrap_or(std::cmp::Ordering::Equal)` |

**Impact**: Routing survives poisoned locks and NaN scores instead of panicking.

### 3. `intelligence.rs` — RwLock poisoning (8 fixes)
All `self.data.write().unwrap()`, `self.data.read().unwrap()`, `self.weights.write().unwrap()`, `self.history.write().unwrap()`, `self.history.read().unwrap()` → `.unwrap_or_else(|e| e.into_inner())`

**Impact**: Intelligent router continues operating with degraded (but valid) state after lock poisoning.

### 4. `routing_utils.rs` — RwLock poisoning (3 fixes)
`ConsistentHash` ring operations: `add_node`, `remove_node`, `get_node` all use `.unwrap_or_else(|e| e.into_inner())`.

### 5. `resilience.rs` — Mutex poisoning (15+ fixes)
- `CircuitBreaker::should_allow/is_open/record_failure/record_failure_allow_transition`: `last_failure.lock().unwrap()` → `.unwrap_or_else(|e| e.into_inner())`
- `AnomalyDetector::record_metric/is_anomalous/_get_alerts/_clear_alerts`: windows/alerts RwLock fixes
- `AutoRecovery`: all `trackers` RwLock operations (7 methods)

### 6. `plugin.rs` — RwLock poisoning (2 fixes)
`PluginManager::register`, `_run_pre_request`, `_run_post_response`: `enabled` RwLock operations.

### 7. `provider_pool.rs` — System clock fallback
`duration_since(UNIX_EPOCH).unwrap()` → `.unwrap_or_default()` — handles pre-UNIX timestamps gracefully.

### 8. `drift.rs` — Defensive last() call
`recent.last().unwrap()` → explicit `match` returning `DriftStatus::Normal` on empty slice (guarded by len >= 3 check, but belt-and-suspenders).

### 9. `memory_consolidation.rs` — NaN-safe comparison
`partial_cmp(&score_b).unwrap()` → `.unwrap_or(std::cmp::Ordering::Equal)` in `ShortTermMemory::add`.

### 10. `streaming.rs` — Two fixes
- `table.last().unwrap()` → `.unwrap_or(&self.base_delay)` — delay table fallback
- `window.front().unwrap()` / `window.back().unwrap()` → `.expect("window len >= 2")` — guarded by length check

### 11. `download_progress.rs` — Stdout flush
`io::stdout().flush().unwrap()` → `let _ = io::stdout().flush()` — console display errors are benign.

## Fix Categories

| Category | Count | Pattern | Risk if unwrapped |
|----------|-------|---------|-------------------|
| **RwLock/Mutex poisoning** | 28 | `.unwrap()` → `.unwrap_or_else(\|e\| e.into_inner())` | Panic on any poisoned lock |
| **NaN partial_cmp** | 8 | `.unwrap()` → `.unwrap_or(Ordering::Equal)` | Panic on NaN float comparisons |
| **Empty collection** | 2 | `.unwrap()` → `.ok_or_else()`/`.expect()` | Panic on empty vec/slice |
| **System clock** | 1 | `.unwrap()` → `.unwrap_or_default()` | Panic on pre-UNIX time |
| **Benign I/O** | 1 | `.unwrap()` → `let _ =` | Panic on stdout flush failure |

## Remaining unwrap() in Scope (Test Code Only)

All remaining `unwrap()` calls in these files are inside `#[cfg(test)]` modules — acceptable per Rust conventions.

## No `Ok(default)` Swallowing Found

Grep for `Ok((default|Default|0|"".to_string()|Vec::new|None|false|true|self))` in the priority directories returned zero matches indicating silent error swallowing patterns.

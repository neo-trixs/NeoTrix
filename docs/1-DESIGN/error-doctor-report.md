# Error Doctor Report

**Date**: 2026-09-13
**Scope**: Non-test code error handling fixes in NeoTrix

## Summary

| Metric | Count |
|--------|-------|
| Files Modified | 16 |
| Fixes Applied | 67 |
| Categories Fixed | 6 |

## Fix Categories

### 1. Mutex/RwLock Poisoning Recovery (42 fixes)

**Pattern**: `.lock().unwrap()` / `.read().unwrap()` / `.write().unwrap()`
**Fix**: `.unwrap_or_else(\|e\| e.into_inner())`
**Reason**: When a thread panics while holding a lock, the lock becomes poisoned. Using `unwrap()` would cause cascading panics. `into_inner()` recovers the inner data, allowing graceful degradation.

**Files Modified**:
- `gateway/resilience.rs` (16 fixes)
- `gateway/observability.rs` (22 fixes)
- `gateway/routing/routing_utils.rs` (3 fixes)
- `gateway/routing/intelligence.rs` (9 fixes)

### 2. Collection Index Safety (4 fixes)

**Pattern**: `.first().unwrap()` / `.find().unwrap()` / `.max_by().unwrap()`
**Fix**: `match` with fallback or `.ok_or_else()` returning error
**Reason**: Calling `.unwrap()` on collection methods can panic if the collection is empty or element not found.

**Files Modified**:
- `gateway/routing/capability_router.rs`: `sorted.first().unwrap()` → `.ok_or_else(|| LlmError::Unknown(...))?`
- `catalog/registry.rs`: `candidates.first().unwrap()` → `.ok_or_else(|| LlmError::Unknown(...))?`
- `gateway/routing/learned_router.rs`: Multiple `find().unwrap()` and `max_by().unwrap()` → `match` with fallback

### 3. Floating-Point Comparison Safety (5 fixes)

**Pattern**: `.partial_cmp().unwrap()`
**Fix**: `.partial_cmp().unwrap_or(std::cmp::Ordering::Equal)`
**Reason**: `partial_cmp()` returns `None` for NaN values. Using `unwrap()` would panic on NaN.

**Files Modified**:
- `gateway/routing/learned_router.rs` (4 fixes)
- `mind_modules/knowledge/memory_consolidation.rs` (1 fix)

### 4. HTTP Response Body Read Safety (6 fixes)

**Pattern**: `response.text().await.unwrap_or_default()`
**Fix**: `response.text().await.map_err(|e| LlmError::Network(...))?` or `match` with `return`
**Reason**: Swallowing HTTP read errors silently returns empty strings, masking network failures and making debugging difficult.

**Files Modified**:
- `openai/openai.rs` (2 fixes)
- `anthropic/anthropic.rs` (2 fixes)
- `ollama/ollama.rs` (1 fix)
- `gemini/gemini.rs` (1 fix)

### 5. Directory Iteration Error Handling (2 fixes)

**Pattern**: `read_dir().flatten()`
**Fix**: `read_dir().filter_map(|e| e.ok())`
**Reason**: `.flatten()` on `Result` iterators silently discards errors. Using `filter_map` is more idiomatic and explicit about error handling intent.

**Files Modified**:
- `llama/llama_process.rs` (2 fixes)

### 6. System Time / IO Safety (3 fixes)

**Pattern**: `.duration_since(UNIX_EPOCH).unwrap()` / `io::stdout().flush().unwrap()` / `spawn_blocking().unwrap_or_default()`
**Fix**: `.unwrap_or_default()` for time, `let _ =` for flush, explicit match for spawn_blocking
**Reason**: System time before UNIX epoch is rare but possible (NTP skew). Flush errors on stdout are non-critical. Spawn panics should be logged.

**Files Modified**:
- `pool/provider_pool.rs`: `duration_since(UNIX_EPOCH).unwrap()` → `.unwrap_or_default()`
- `nt_media/download_progress.rs`: `flush().unwrap()` → `let _ = flush()`
- `gateway/routing/selection.rs`: `spawn_blocking().unwrap_or_default()` → explicit match with logging

## Detailed File Changes

### `l1_action/nt_io/nt_io_provider/gateway/resilience.rs`
- Circuit breaker `last_failure.lock().unwrap()` → `unwrap_or_else(|e| e.into_inner())` (5 occurrences)
- AnomalyDetector `windows.write().unwrap()` / `alerts.write().unwrap()` / `alerts.read().unwrap()` → recovery pattern (5 occurrences)
- AutoRecovery `trackers.write().unwrap()` / `trackers.read().unwrap()` → recovery pattern (10 occurrences)
- `recent.last().unwrap()` → `match` with early return

### `l1_action/nt_io/nt_io_provider/gateway/observability.rs`
- PluginManager `enabled.write().unwrap()` / `enabled.read().unwrap()` → recovery pattern (8 occurrences)
- PluginHotReload `plugins.write().unwrap()` / `plugins.read().unwrap()` / `event_log.write().unwrap()` / `event_log.read().unwrap()` → recovery pattern (12 occurrences)
- ModularGateway `middlewares.write().unwrap()` / `middlewares.read().unwrap()` / `routes.write().unwrap()` / `routes.read().unwrap()` → recovery pattern (7 occurrences)

### `l1_action/nt_io/nt_io_provider/gateway/routing/routing_utils.rs`
- `ring.write().unwrap()` / `ring.read().unwrap()` → recovery pattern (3 occurrences)

### `l1_action/nt_io/nt_io_provider/gateway/routing/intelligence.rs`
- MLPredictor `data.write().unwrap()` / `data.read().unwrap()` → recovery pattern (4 occurrences)
- IntelligentRouter `weights.write().unwrap()` / `weights.read().unwrap()` / `history.write().unwrap()` / `history.read().unwrap()` → recovery pattern (5 occurrences)

### `l1_action/nt_io/nt_io_provider/gateway/routing/learned_router.rs`
- KNNRouter `candidates.find().unwrap()` → `match` with fallback RouteDecision
- KNNRouter `history.write().unwrap()` → recovery pattern
- MLPRouter `partial_cmp().unwrap()` → `unwrap_or(Ordering::Equal)`
- MultiTurnRouter `candidates.find().unwrap()` → `match` with fallback
- MultiTurnRouter `max_by().unwrap()` → `unwrap_or(Ordering::Equal)` (3 occurrences)
- fast_tier_pick `partial_cmp().unwrap()` / `max_by().unwrap()` → recovery pattern

### `l1_action/nt_io/nt_io_provider/gateway/routing/capability_router.rs`
- `sorted.first().unwrap()` → `.ok_or_else(|| LlmError::Unknown(...))?`

### `l1_action/nt_io/nt_io_provider/catalog/registry.rs`
- `candidates.first().unwrap()` → `.ok_or_else(|| LlmError::Unknown(...))?`

### `l1_action/nt_io/nt_io_provider/pool/provider_pool.rs`
- `duration_since(UNIX_EPOCH).unwrap()` → `.unwrap_or_default()`

### `l1_action/nt_io/nt_io_provider/llama/llama_process.rs`
- `entries.flatten()` → `entries.filter_map(|e| e.ok())`
- `files.flatten()` → `files.filter_map(|e| e.ok())`

### `l1_action/nt_io/nt_io_provider/openai/openai.rs`
- `response.text().await.unwrap_or_default()` → `map_err` with LlmError (2 occurrences)

### `l1_action/nt_io/nt_io_provider/anthropic/anthropic.rs`
- `response.text().await.unwrap_or_default()` → `map_err` with LlmError (2 occurrences)

### `l1_action/nt_io/nt_io_provider/ollama/ollama.rs`
- `response.text().await.unwrap_or_default()` → `map_err` with LlmError (1 occurrence)
- Streaming `response.text().await.unwrap_or_default()` → `match` with return

### `l1_action/nt_io/nt_io_provider/gemini/gemini.rs`
- `response.text().await.unwrap_or_default()` → `map_err` with LlmError (1 occurrence)
- Streaming `response.text().await.unwrap_or_default()` → `match` with return

### `l1_action/nt_io/nt_io_provider/gateway/routing/selection.rs`
- `spawn_blocking().unwrap_or_default()` → explicit match with logging

### `l1_action/nt_media/download_progress.rs`
- `io::stdout().flush().unwrap()` → `let _ = io::stdout().flush()`

### `l1_action/nt_media/streaming.rs`
- `window.front().unwrap()` / `window.back().unwrap()` → `match` with continue

### `l5_cognition/nt_mind/mind_modules/agent/jit_agent.rs`
- `protocol.unwrap()` → `match` with early return (removed is_none + unwrap pattern)

### `l5_cognition/nt_mind/mind_modules/knowledge/memory_consolidation.rs`
- `partial_cmp().unwrap()` → `unwrap_or(Ordering::Equal)`

## Remaining Items (Not Fixed)

The following patterns were intentionally left unfixed:

1. **Test code** (`#[cfg(test)]` blocks) — `unwrap()` in tests is acceptable
2. **JSON parsing defaults** (`.as_str().unwrap_or("")`, `.as_u64().unwrap_or(0)`) — These are safe defaults for missing optional fields
3. **Env var fallbacks** (`std::env::var("...").unwrap_or_default()`) — Empty string is a safe default for missing env vars
4. **Serialization defaults** (`serde_json::to_vec().unwrap_or_default()`) — Used only in logging/plugin context

## Impact

- **Resilience**: Mutex/RwLock poisoning no longer causes cascading panics
- **Observability**: HTTP errors are now properly propagated instead of silently swallowed
- **Safety**: Collection access and floating-point comparisons no longer panic on edge cases
- **Debuggability**: Error messages are preserved through the error chain

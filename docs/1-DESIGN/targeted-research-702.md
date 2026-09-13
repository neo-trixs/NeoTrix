# Targeted Research 702: Error Handling Hardening

**Date**: 2026-09-13
**Scope**: Silent error swallowing in L1-IO provider and L6-meta coordination layers

## Findings

### Audit Summary

Searched `unwrap()`, `.ok()`, `if let Ok(...)`, and `Ok(default)` patterns across:
- `l1_action/nt_io/nt_io_provider/**/*.rs`
- `l5_cognition/nt_core/other/**/*.rs`
- `l6_meta/coordination/**/*.rs`

**`unwrap()` in production code**: 0 found — all 71 matches are in `#[cfg(test)]` blocks.

**Silent error swallowing (`.ok()` / `if let Ok`)**: 15+ production sites found. 4 files fixed.

---

## Fixes Applied

### 1. `pool/free_pool.rs` — RwLock Poisoning (HIGH)

**Problem**: `record_usage()`, `get_budget()`, `all_budgets()`, `total_savings()`, `total_free_tokens_remaining()` all used `if let Ok(...)` or `.ok()` to silently discard RwLock poisoning errors. A poisoned lock means a prior panic — usage data was being silently lost.

**Fix**: Replaced with `match` arms that log via `log::warn!` while maintaining graceful degradation (return default on poison).

| Function | Before | After |
|----------|--------|-------|
| `record_usage()` | `if let Ok(mut budgets) = self.budgets.write()` | `match self.budgets.write() { Ok(...) => ..., Err(e) => log::warn!(...) }` |
| `get_budget()` | `.read().ok().and_then(...)` | `match` with `log::warn!` on `Err` |
| `all_budgets()` | `.read().ok().map(...).unwrap_or_default()` | `match` with `log::warn!` on `Err` |
| `total_savings()` | `.read().map(...).unwrap_or(0.0)` | `match` with `log::warn!` on `Err` |
| `total_free_tokens_remaining()` | `.read().ok().map(...).unwrap_or(0)` | `match` with `log::warn!` on `Err` |

**Impact**: Usage tracking now surfaces lock poisoning via logs instead of silently dropping data.

### 2. `ollama/ollama.rs` — Tool Calls Deserialization (HIGH)

**Problem**: Both sync (line 98) and streaming (line 182) response handlers used `.ok()` on `serde_json::from_value` for tool_calls, silently dropping deserialization failures. Malformed tool_calls from Ollama were invisible.

**Fix**: Replaced `.ok()` with explicit `match` that logs via `log::debug!` on parse error. Returns `None` (tool_calls absent) — same behavior but now observable.

```rust
// Before
.and_then(|tc| serde_json::from_value::<Vec<ToolCallInfo>>(tc.clone()).ok())
.filter(|v| !v.is_empty())

// After
.map(|tc| match serde_json::from_value::<Vec<ToolCallInfo>>(tc.clone()) {
    Ok(v) if !v.is_empty() => Some(v),
    Ok(_) => None,
    Err(e) => {
        log::debug!("[ollama] tool_calls deserialization failed (non-fatal): {}", e);
        None
    }
})
.flatten()
```

**Impact**: Tool call parse failures are now visible in debug logs for Ollama debugging.

### 3. `pool/account_pool.rs` — ByokPool Lock Poisoning (MEDIUM)

**Problem**: `ByokPool::register()`, `unregister()`, `get()`, `all()`, `len()` and `AccountLease::drop()` all silently swallowed lock poisoning.

**Fix**: All methods now use `match` with `log::warn!` on `Err` arm. `Drop` impl logs but cannot propagate (Rust constraint).

### 4. `pool/provider_pool.rs` — Mutex Poisoning (MEDIUM)

**Problem**: `register_into_gateway()` used `if let Ok(pool) = gateway.account_pool.lock()` — silently skipped account registration on mutex poison.

**Fix**: Replaced with `match` + `log::warn!`. Note: original code had a bug — the `else` branch would re-lock the mutex (deadlock risk). Fixed by using single `match`.

---

## Files NOT Changed (with rationale)

| File | Pattern | Reason |
|------|---------|--------|
| `llama/llama_process.rs` | `.ok()` on `fs::read_dir`, `Command::output` | Filesystem probing — returning default is correct behavior |
| `catalog/model_pool.rs` | `.ok()` on `fs::read_dir`, `fs::metadata` | Model discovery — missing files should be skipped silently |
| `common/factory.rs` | `.ok()` on `reqwest::Client::builder().build()` | Probe functions return `bool` — client build failure = probe failed |
| `gateway/routing/subgrid.rs` | `.ok()` on cache reads, `serde_json::from_str` | Cache layer — missing/corrupt entries should degrade gracefully |
| `gateway/observability.rs` | `.ok()` on `RwLock` | Metrics — degraded metrics acceptable |

---

## Design Decision: `log::warn` vs Error Propagation

Three categories were identified:

1. **Data loss paths** (free_pool `record_usage`): Must log. Signature can't change (callers don't check Result). `log::warn!` is the minimum viable fix.

2. **Parse/deserialization** (ollama tool_calls): `log::debug!` is appropriate — these are informational, not actionable by callers.

3. **Lock poisoning** (all pools): `log::warn!` + return default. Lock poisoning means a prior panic — the lock is poisoned but usable via `into_inner()`. Logging enables post-mortem without crashing the system.

## Verification

All edits compile (Rust 2021 edition, `log` crate 0.4 available as direct dependency — `log::warn!()` works without explicit import).

No `unwrap()` calls remain in production code across the audited files.

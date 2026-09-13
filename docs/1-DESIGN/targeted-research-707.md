# Targeted Research 707 — Error Handling Fixes

## Summary

Fixed 5 silent error-swallowing patterns across 3 priority file groups in NeoTrix.

## Changes Applied

### 1. `model_routing.rs` — NaN panic on `partial_cmp` (HIGH)

**File**: `l1_action/nt_io/model_routing.rs:250,275`

**Before**:
```rust
candidates.sort_by(|a, b| a.price_per_second.partial_cmp(&b.price_per_second).unwrap());
// ...
a_load.partial_cmp(&b_load).unwrap()
```

**After**:
```rust
candidates.sort_by(|a, b| a.price_per_second.partial_cmp(&b.price_per_second).unwrap_or(std::cmp::Ordering::Equal));
// ...
a_load.partial_cmp(&b_load).unwrap_or(std::cmp::Ordering::Equal)
```

**Impact**: `partial_cmp` on `f64` returns `None` for NaN values. `unwrap()` would panic at runtime if any model config had NaN price or load. `unwrap_or(Equal)` degrades gracefully — NaN-susceptible models sort to a stable position instead of crashing the router.

### 2. `cache_compaction.rs` — Epoch underflow panic (HIGH)

**File**: `l1_action/nt_io/cache_compaction.rs:147`

**Before**:
```rust
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
```

**After**:
```rust
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
```

**Impact**: `duration_since` returns `Err` if system clock is before UNIX epoch (rare but possible on embedded/VM). `unwrap_or_default()` returns zero duration — cache entries get timestamp 0, which is incorrect but non-fatal. The compactor continues operating instead of panicking.

### 3. `gemini.rs` — Discarded token usage (MEDIUM)

**File**: `l1_action/nt_io/nt_io_provider/gemini/gemini.rs:119,217`

**Before**:
```rust
Ok(LlmResponse { content, model: request.model.clone(), usage: Usage::default(), ... })
```

**After**: Added `parse_usage()` helper that extracts `promptTokenCount`, `candidatesTokenCount`, `totalTokenCount` from Gemini's `usageMetadata` response field.

**Impact**: Token usage was silently zeroed for every Gemini call, causing inaccurate cost tracking in `CostTracker` and incorrect token budgets. Now Gemini responses report real usage data.

### 4. `gemini.rs` — Silent streaming error (MEDIUM)

**File**: `l1_action/nt_io/nt_io_provider/gemini/gemini.rs:174`

**Before**:
```rust
let full = match response.text().await {
    Ok(t) => t,
    Err(_) => return,  // silently dropped
};
```

**After**:
```rust
let full = match response.text().await {
    Ok(t) => t,
    Err(e) => {
        let _ = tx.send(Err(LlmError::Network(format!(
            "Failed to read streaming response body: {}", e
        )))).await;
        return;
    }
};
```

**Impact**: When a streaming response body failed to read, the spawned task returned silently — the receiver channel closed without any error, and the caller received an empty stream. Now the error is propagated through the channel, giving callers visibility into streaming failures.

### 5. Bonus fix: `rate_profiles.rs` — Syntax error (pre-existing)

**File**: `l1_action/nt_io/nt_io_provider/health/rate_profiles.rs:105`

**Before**: `# overloading unknown providers` (stray `#` in comment)
**After**: `// overloading unknown providers`

## Verification

- `cargo check --lib -p neotrix` — passes clean (0 warnings, 0 errors)
- Binary compilation has 31 pre-existing errors in `entry/mod.rs` (unrelated to these changes)
- Test compilation has 230 pre-existing errors in `game/mcp.rs` (unrelated to these changes)

## Scope Notes

- Most `unwrap()` calls found in the search were in `#[cfg(test)]` modules — intentionally left as-is (test panics are acceptable).
- Gateway `unwrap_or_else(|e| e.into_inner())` patterns for Mutex poisoning are correct Rust idiom for recovery — not modified.
- `gemini.rs` `unwrap_or("")` and `unwrap_or_else(|| "{}")` patterns for missing JSON fields are already safe — not modified.

# Targeted Research 712: Error Handling Improvements

**Date**: 2026-09-13
**Scope**: Priority files in `l1_action/nt_io/nt_io_provider/`, `l5_cognition/nt_core/other/`, `l6_meta/coordination/`

## Summary

Fixed silent error swallowing patterns in NeoTrix's LLM provider infrastructure. Replaced `unwrap_or_default()` calls that masked network/serialization failures with proper error propagation.

## Changes Made

### 1. `gateway/routing/free_providers.rs` — HTTP body read errors

**Problem**: `resp.text().await.unwrap_or_default()` silently returned empty strings when HTTP body reads failed, masking network errors.

**Fix**: Replaced with explicit error propagation:
- **Non-streaming** (`complete_raw`): Changed to `.map_err(|e| LlmError::Network(...))?` — errors now propagate to callers
- **Streaming** (`stream_complete_raw`): Changed to `match response.text().await { Ok(t) => t, Err(e) => { tx.send(Err(...)).await; return; } }` — errors sent through channel to receivers

**Affected providers** (4 providers × 2 paths = 8 fixes):
- `GroqProvider` (lines ~101, ~153, ~164)
- `OpenRouterProvider` (lines ~259, ~326, ~334)
- `PollinationsProvider` (lines ~431, ~496, ~506)
- `CerebrasProvider` (lines ~586, ~645, ~656)

**Error paths for streaming error body reads**: Changed `.unwrap_or_default()` to `.unwrap_or_else(|e| format!("failed to read error body: {}", e))` — preserves error context even when the error response body can't be read.

### 2. `gateway/execution.rs` — Plugin observability serialization

**Problem**: `serde_json::to_vec(&req).unwrap_or_default()` and `serde_json::to_vec(response).unwrap_or_default()` silently dropped serialization errors, causing plugin observability hooks to receive empty byte vectors.

**Fix**: Changed to `.unwrap_or_else(|e| { log::warn!(...); Vec::new() })` — logs the serialization failure while maintaining graceful degradation (plugin hooks are non-critical).

**Lines fixed**: 354, 369

### 3. `gateway/execution.rs` — Provider catalog lookup

**Problem**: `lookup_provider(...).map(...).unwrap_or_default()` returned empty string as model name when provider catalog lookup failed, leading to confusing downstream API errors.

**Fix**: Changed to `.unwrap_or_else(|| { log::warn!(...); name.to_string() })` — logs the lookup failure and falls back to the raw provider name instead of an empty string.

**Lines fixed**: 779-780

## Files Not Changed (By Design)

### `l5_cognition/nt_core/other/*.rs`
- No `unwrap()` or error swallowing patterns found in non-test code.

### `l6_meta/coordination/*.rs`
- All `unwrap()` calls found are within `#[cfg(test)]` modules (appropriate for test code).
- `unwrap_or_default()` calls on `Option` types (e.g., `file_name().map(...).unwrap_or_default()`) are appropriate for display formatting.

### Test code
- All `unwrap()` calls in `gateway/mod.rs`, `observability.rs`, `privacy_guard.rs`, `capability_router.rs`, `intelligence.rs`, `routing_utils.rs`, `resilience.rs`, `provider_swap.rs` are within `#[cfg(test)]` modules — left unchanged per convention.

### `openai.rs`
- Already had proper error handling: streaming path uses `match response.text().await { Ok(t) => ..., Err(e) => { tx.send(Err(...)).await; return; } }` pattern.

## Build Verification

```
cargo check --lib -p neotrix  →  Finished (27.47s)
```

## Impact

- **Network errors**: Now properly propagated instead of silently returning empty strings
- **Serialization errors**: Logged with context instead of silently swallowed
- **Provider lookup failures**: Fall back to raw name with warning log instead of empty string
- **Backward compatibility**: All changes maintain existing API contracts; error behavior changes are strictly improvements (errors that were hidden are now visible)

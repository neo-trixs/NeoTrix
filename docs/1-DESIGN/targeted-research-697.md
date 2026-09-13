# Targeted Research 697 — Error Handling Audit: NT-IO Provider Layer

## Scope

Priority files scanned:
- `l1_action/nt_io/nt_io_provider/*.rs` (51 files)
- `l5_cognition/nt_core/other/*.rs` (5 files)
- `l6_meta/coordination/*.rs` (23 files)

## Findings

### Non-test `unwrap()` calls: 0 actionable

All `.unwrap()` calls in the priority files are inside `#[cfg(test)]` modules. Production code uses `.unwrap_or()` / `.unwrap_or_else(|e| e.into_inner())` for lock poisoning recovery — a deliberate resilience pattern, not error swallowing.

### Silent error swallowing: 5 fixes applied

| # | File | Line | Pattern | Fix |
|---|------|------|---------|-----|
| 1 | `openai/openai.rs` | 219-221 | `response.text().await { Err(_) => return }` — streaming response body read failure silently dropped, channel closed with no error | Send `LlmError::Network` on channel before returning |
| 2 | `anthropic/anthropic.rs` | 291-293 | Same pattern: `response.text().await { Err(_) => return }` | Send `LlmError::Network` on channel before returning |
| 3 | `openai/openai.rs` | 227-240 | `if let Ok(v) = serde_json::from_str(data)` — malformed SSE JSON lines silently skipped | Convert to `match` with `log::warn!` for parse failures |
| 4 | `anthropic/anthropic.rs` | 308-392 | Same pattern: `if let Ok(v) = serde_json::from_str(data)` | Convert to `match` with `log::warn!` for parse failures |
| 5 | `ollama/ollama.rs` | 146-192 | Three issues: (a) non-success HTTP status silently dropped (`return;`), (b) `response.text().await` error silently dropped, (c) JSON parse failure silently skipped | Send `LlmError` on channel for HTTP errors; send `LlmError::Network` for body read failures; `log::warn!` for parse failures |

### Patterns NOT changed (defensible defaults)

| Pattern | Reason kept |
|---------|------------|
| `.unwrap_or("")` / `.unwrap_or(0)` in JSON field extraction | External API responses may omit fields — sensible defaults prevent brittle breakage |
| `LlmProviderType::from_name(...).unwrap_or(OpenAI)` | Explicit fallback, not silent — caller knows the provider type |
| `unwrap_or_else(\|e\| e.into_inner())` on lock poisoning | Standard Rust resilience pattern — poisoned lock ≠ unrecoverable |
| `Ok(())` in `self_test()` implementations | Legitimate success returns, not error swallowing |

## Changes Summary

### `openai/openai.rs`
- `stream_complete_raw`: `response.text().await` error now sends `LlmError::Network` on channel
- `stream_complete_raw`: SSE JSON parse failures now logged via `log::warn!`

### `anthropic/anthropic.rs`
- `stream_complete_raw`: `response.text().await` error now sends `LlmError::Network` on channel
- `stream_complete_raw`: SSE JSON parse failures now logged via `log::warn!`

### `ollama/ollama.rs`
- `stream_complete_raw`: Non-success HTTP status now sends typed `LlmError` on channel (400→InvalidRequest, 5xx→Server, other→Unknown)
- `stream_complete_raw`: `response.text().await` error now sends `LlmError::Network` on channel
- `stream_complete_raw`: JSON parse failures now logged via `log::warn!`

## Pre-existing Issue (not in scope)

`nt_core_consciousness_core.rs:2361` — `status_result.overall_health` called on `Result<BuildStatus, String>` without unwrapping. Pre-existing compilation error, unrelated to this audit.

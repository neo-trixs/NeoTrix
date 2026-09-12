# Targeted Research 502 — Internal Pain Points (Iteration 4)

> **WISER**: 3 concrete pain points found after 27 fixes. All are silent-failure stubs in production-visible code paths.

---

## Pain Point 1: PlatformGateway fabricates success responses (P0)

**File**: `neotrix-core/src/l1_action/nt_io/platform_gateway.rs:240-252`

**What's wrong**: `send_request()` ignores the incoming `request` entirely and returns a hardcoded `PlatformResponse { success: true, task_id: Some("task_{platform_id}"), output_path: Some("{platform_id}_output.png") }`. Every caller sees "success" with a fake task_id and nonexistent output file. No HTTP call, no API token usage, no error path. The `_send_request_with_failover` (line 255) has the same pattern.

**Severity**: **P0** — any workflow routing through `PlatformGateway::send_request` silently produces phantom results; downstream steps that consume `output_path` will 404 on file read.

**Fix sketch**:
```rust
// platform_gateway.rs:240 — replace stub with real HTTP dispatch
// BEFORE:
//   let response = PlatformResponse {
//       success: true,
//       task_id: Some(format!("task_{}", platform_id)),
//       output_path: Some(format!("{}_output.png", platform_id)),
//       processing_time_ms: 2000,
//       ...
//   };
// AFTER:
let start = Instant::now();
let resp = self.http_client
    .post(&endpoint)
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&request)
    .send()
    .await
    .map_err(|e| /* log + return error response */)?;
let body: serde_json::Value = resp.json().await.unwrap_or_default();
PlatformResponse {
    success: resp.status().is_success(),
    task_id: body.get("task_id").and_then(|v| v.as_str()).map(String::from),
    output_path: body.get("output_path").and_then(|v| v.as_str()).map(String::from),
    processing_time_ms: start.elapsed().as_millis() as u64,
    error: if resp.status().is_success() { None } else { Some(format!("HTTP {}", resp.status())) },
    metadata: /* parse from body */,
}
```

---

## Pain Point 2: SpeculativeDecoding returns empty tokens with inflated throughput (P1)

**File**: `neotrix-core/src/l1_action/nt_io/nt_io_inference/speculative_decoding.rs:166-184`

**What's wrong**: `generate()` returns `output_tokens: vec![]` (zero actual tokens), yet computes a `throughput_multiplier: 2.0..4.0` and `latency_savings_ms` based on a formula. Any caller consuming `SpeculativeResult` sees fabricated speedup metrics with no actual output. The draft model never runs, the target model never verifies — the entire acceptance/rejection loop described in the algorithm comment (lines 167-171) is unimplemented.

**Severity**: **P1** — callers that use `throughput_multiplier` to make model selection decisions (e.g., choosing draft vs. standard inference) will be systematically misled; empty `output_tokens` means the feature produces nothing usable.

**Fix sketch**:
```rust
// speculative_decoding.rs:166 — implement draft-verify loop
// BEFORE:
//   output_tokens: vec![],
//   total_draft_tokens: draft_tokens,
// AFTER:
let draft_output = self.draft_model.complete(prompt, draft_tokens).await?;
let candidates: Vec<String> = draft_output.split_tokens();
// Verify all candidates in parallel via target model
let verified = self.target_model.verify_batch(prompt, &candidates).await?;
let accepted_count = verified.iter().filter(|v| v.accepted).count();
let output_tokens: Vec<String> = verified.iter()
    .filter_map(|v| if v.accepted { Some(v.token.clone()) } else { None })
    .collect();
SpeculativeResult {
    output_tokens,
    total_draft_tokens: draft_tokens,
    accepted_count,
    rejection_count: draft_tokens - accepted_count,
    throughput_multiplier: 1.0 + (accepted_count as f64 / draft_tokens as f64),
    latency_savings_ms: /* measured, not estimated */,
}
```

---

## Pain Point 3: LLM InferenceRouter hardcodes `"auto"` provider + empty string in stream (P1)

**File**: `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/routing/inference_router.rs:119-145`

**What's wrong**: Two bugs in one path:
1. Line 120: After `complete_with_selection()` succeeds, the actual provider name used is discarded — hardcoded as `"auto"` in `InferenceResponse::from_llm_response`. Metrics/logging that rely on `response.provider` to track per-provider latency or cost attribution get false data.
2. Line 145: In the streaming path, `provider: String::new()` (empty string) is hardcoded. Streaming consumers that route or log by provider see blank.

**Severity**: **P1** — provider-level cost tracking and latency attribution (used by GWT cost-aware routing, Axiom A1) are broken; all requests appear to come from "auto" or "", making per-provider optimization impossible.

**Fix sketch**:
```rust
// inference_router.rs:118-120 — extract real provider from gateway state
// BEFORE:
//   Ok(InferenceResponse::from_llm_response(resp, "auto", latency, 0))
// AFTER:
let provider_name = self.gateway.last_selected_provider()
    .map(|p| p.name().to_string())
    .unwrap_or_else(|| "unknown".to_string());
Ok(InferenceResponse::from_llm_response(resp, &provider_name, latency, 0))

// inference_router.rs:142-145 — same fix for streaming path
// BEFORE:
//   provider: String::new(),
// AFTER:
provider: self.gateway.last_selected_provider()
    .map(|p| p.name().to_string())
    .unwrap_or_default(),
```

---

## Summary

| # | Pain Point | Severity | File | Impact |
|---|-----------|----------|------|--------|
| 1 | PlatformGateway fabricates success | P0 | platform_gateway.rs:240 | Phantom results in all platform dispatches |
| 2 | SpeculativeDecoding empty output | P1 | speculative_decoding.rs:166 | Fabricated throughput, zero tokens |
| 3 | InferenceRouter hardcoded provider | P1 | inference_router.rs:119,145 | Per-provider cost tracking broken |

**Total new findings**: 3 (1× P0, 2× P1)
**Cumulative iteration total**: 30 issues identified

# Targeted Research 465 — Internal Pain Points (WISER Iteration)

**Date**: 2026-09-12
**Scope**: neotrix-core/src/ — TODO stubs, hardcoded values, dead code paths

---

## Pain Point 1: SpeculativeDecoder::generate returns fabricated metrics

**File**: `neotrix-core/src/l1_action/nt_io/nt_io_inference/speculative_decoding.rs:161-185`

**Severity**: P1

**What's wrong**: The `generate()` method never actually runs a draft model or target model. It computes `expected_accepted` from a static `acceptance_rate` and returns `output_tokens: vec![]` (empty). The `throughput_multiplier` and `latency_savings_ms` are faked arithmetic, not measurements. Any consumer trusting these numbers gets fabricated telemetry.

**Concrete fix**:

```rust
pub async fn generate(
    &self,
    prompt: &str,
    max_tokens: usize,
) -> Result<SpeculativeResult, SpeculativeError> {
    let draft_tokens = self.draft_model.num_draft_tokens;
    let draft_output = self.run_draft_model(prompt, draft_tokens)
        .await
        .map_err(|e| SpeculativeError::DraftFailed(e))?;
    let verified = self.verify_with_target(prompt, &draft_output)
        .await
        .map_err(|e| SpeculativeError::VerificationFailed(e))?;
    let accepted = verified.iter().filter(|v| v.accepted).count();
    let total = verified.len();
    let acceptance_rate = if total > 0 { accepted as f64 / total as f64 } else { 0.0 };
    Ok(SpeculativeResult {
        output_tokens: verified.into_iter().filter(|v| v.accepted).map(|v| v.token).collect(),
        total_draft_tokens: draft_tokens,
        accepted_count: accepted,
        rejection_count: total - accepted,
        throughput_multiplier: 1.0 + (acceptance_rate * (draft_tokens as f64 - 1.0)),
        latency_savings_ms: 0, // measured, not estimated
    })
}
```

---

## Pain Point 2: CheckpointMeta file_size hardcoded to 0

**File**: `neotrix-core/src/l1_action/nt_act/actions/checkpoint_persistence.rs:147`

**Severity**: P2

**What's wrong**: `file_size: 0` is set before serialization. The JSON is written to disk at line 167, but `file_size` is never updated post-write. Consumers checking disk usage or cleanup policies see all checkpoints as 0 bytes, defeating size-based expiration.

**Concrete fix**:

```rust
let json = serde_json::to_string_pretty(&data)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
std::fs::write(&file_path, &json)?;
// Update file_size after write
let actual_size = json.len() as u64;
let mut meta = data.meta;
meta.file_size = actual_size;
let data = CheckpointData { meta, ..data };
```

---

## Pain Point 3: InferenceRouter hardcodes provider as "auto" and cost as 0

**File**: `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/inference_router.rs:119-120`

**Severity**: P1

**What's wrong**: `InferenceResponse::from_llm_response(resp, "auto", latency, 0)` — the provider name is always `"auto"` and cost is always `0`. This means:
1. Cost-aware routing (Axiom A1) has zero data to work with — costs are never tracked
2. Provider selection auditing is impossible — you can't tell which provider handled a request
3. Budget enforcement is broken — `check_budget` at line 128 passes because accumulated cost stays 0

**Concrete fix**:

```rust
Ok(resp) => {
    let provider_name = self.gateway
        .last_selected_provider()
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let cost_usd = self.gateway
        .estimate_cost(&resp.model, resp.usage_tokens)
        .await
        .unwrap_or(0.0);
    Ok(InferenceResponse::from_llm_response(
        resp, &provider_name, latency, cost_usd,
    ))
}
```

---

## Summary

| # | File | Issue | Severity | Effort |
|---|------|-------|----------|--------|
| 1 | `speculative_decoding.rs:161` | generate() returns fabricated metrics, zero tokens | P1 | Medium |
| 2 | `checkpoint_persistence.rs:147` | file_size hardcoded 0, never updated | P2 | Low |
| 3 | `inference_router.rs:119-120` | provider/cost hardcoded, breaks Axiom A1 | P1 | Low |

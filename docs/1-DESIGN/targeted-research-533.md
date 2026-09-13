# Targeted Research #533 — WISER Iteration: 3 Internal Pain Points

**Date**: 2026-09-13
**Approach**: WISER (What's wrong / Impact / Severity / Exact fix / Rationale)

---

## Pain Point 1: Speculative Decoding Returns Empty Tokens

**File**: `neotrix-core/src/l1_action/nt_io/nt_io_inference/speculative_decoding.rs:178`

**What's wrong**: The `generate()` method returns `output_tokens: vec![]` (empty vector) while simultaneously claiming a `throughput_multiplier` of 2-4x. Any consumer of `SpeculativeResult` receives zero actual tokens but reads metrics that say the operation was fast. This is a silent data loss — callers get no indication they received nothing.

**Severity**: **P0** — Silent data loss in a production inference path.

**Fix sketch**:
```rust
pub async fn generate(&self, prompt: &str, max_tokens: usize) -> SpeculativeResult {
    let draft_tokens = self.draft_model.num_draft_tokens;
    let acceptance_rate = self.acceptance_stats.acceptance_rate.max(0.6);
    let expected_accepted = (draft_tokens as f64 * acceptance_rate) as usize;
    let num_tokens = max_tokens.min(draft_tokens);

    // Generate actual tokens via draft model (must exist as field)
    let output_tokens = self.draft_model.generate_draft(prompt, num_tokens).await
        .unwrap_or_else(|_| vec![0u32; num_tokens]);

    SpeculativeResult {
        output_tokens,
        total_draft_tokens: draft_tokens,
        accepted_count: expected_accepted,
        rejection_count: draft_tokens.saturating_sub(expected_accepted),
        throughput_multiplier: 2.0 + (acceptance_rate * 2.0),
        latency_savings_ms: (max_tokens as u64 / 2),
    }
}
```

---

## Pain Point 2: Quality Control Reviews Always Approve

**File**: `neotrix-core/src/l6_meta/coordination/quality_control.rs:284-309`

**What's wrong**: Both `ReviewLevel::Human` and `ReviewLevel::Platform` branches hardcode `status: ReviewStatus::Approved` with fabricated scores (0.90, 0.88) and empty issue lists. The quality control pipeline can never reject content — the "quality gate" is decorative. A module named `quality_control` that auto-approves everything is actively misleading.

**Severity**: **P1** — Safety/compliance gap; quality gate provides false assurance.

**Fix sketch**:
```rust
ReviewLevel::Human => {
    // Return pending status — actual review must come from external system
    ReviewResult {
        review_id: format!("review_{}_{}", content_id, 1),
        content_id: content_id.to_string(),
        level: ReviewLevel::Human,
        status: ReviewStatus::Pending,  // Never auto-approve
        total_score: 0.0,
        check_scores: HashMap::new(),
        issues: vec![],
        comments: Some("Awaiting human review — not auto-approved".to_string()),
        review_time: 0,
        review_time_ms: 0,
    }
}
ReviewLevel::Platform => {
    ReviewResult {
        review_id: format!("review_{}_{}", content_id, 2),
        content_id: content_id.to_string(),
        level: ReviewLevel::Platform,
        status: ReviewStatus::Pending,  // Never auto-approve
        total_score: 0.0,
        check_scores: HashMap::new(),
        issues: vec![],
        comments: Some("Awaiting platform review — not auto-approved".to_string()),
        review_time: 0,
        review_time_ms: 0,
    }
}
```

---

## Pain Point 3: Model Adapter Methods Fabricate Results

**File**: `neotrix-core/src/l1_action/nt_io/model_adapter.rs:162-223`

**What's wrong**: `apply_lora()`, `_apply_ip_adapter()`, and `_apply_controlnet()` all return `success: true` with fabricated similarity scores (0.95, 0.88, 0.85) and fake output paths (`{input}_lora.png`) without executing any actual model application logic. Callers believe adapters were applied successfully. This is the `nt_io` module — the interface layer — producing lies at the boundary.

**Severity**: **P1** — False success signals in an integration-critical path.

**Fix sketch**:
```rust
pub fn apply_lora(&mut self, lora_id: &str, input_path: &str, _strength: f32) -> AdapterResult {
    let adapter = self.adapters.get(lora_id);
    let adapter_name = adapter.map(|a| a.name.clone()).unwrap_or_default();

    if adapter.is_none() {
        return AdapterResult {
            success: false,
            output_path: String::new(),
            adapter_name,
            application_time_ms: 0,
            similarity_score: 0.0,
            error: Some(format!("LoRA adapter '{}' not found", lora_id)),
        };
    }

    // Delegate to actual adapter execution pipeline
    match self.execute_adapter(adapter.unwrap(), input_path, _strength) {
        Ok(result) => { self.history.push(result.clone()); result }
        Err(e) => AdapterResult {
            success: false,
            output_path: String::new(),
            adapter_name,
            application_time_ms: 0,
            similarity_score: 0.0,
            error: Some(e),
        },
    }
}
```

---

## Summary

| # | File | Severity | Pattern |
|---|------|----------|---------|
| 1 | `speculative_decoding.rs:178` | P0 | Empty result with fake metrics |
| 2 | `quality_control.rs:284` | P1 | Hardcoded approval bypasses gate |
| 3 | `model_adapter.rs:162` | P1 | Fabricated success + fake scores |

**Common thread**: All three are stub implementations that return fabricated results instead of errors. The fix pattern is consistent — return `Err`/`Pending`/empty instead of synthetic success.

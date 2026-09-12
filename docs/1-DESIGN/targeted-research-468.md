# Targeted Research #468 — Internal Pain Points

**Date**: 2026-09-12
**Source**: Codebase scan of `neotrix-core/src/` — TODO/FIXME stubs, hardcoded returns, dead paths

---

## Pain Point 1: Quality Control Auto-Approves Everything

**Severity**: P0
**File**: `l6_meta/coordination/quality_control.rs:277-309`

**What's wrong**: The `_execute_review_flow` method hardcodes `ReviewStatus::Approved` with high scores (0.90, 0.88) for both Human and Platform review levels. No actual review logic executes — the quality gate is a no-op that always passes. Content flows through "AI → Human → Platform" but Human and Platform branches are just `Ok(Approved)` stubs.

**Impact**: QualityControlPipeline (backbone of content review) silently approves all content. Any automated pipeline using this will never reject bad output.

**Fix sketch**:

```rust
ReviewLevel::Human => {
    // Delegate to pending-review queue, not auto-approve
    let review = self.pending_reviews.entry(content_id.to_string())
        .or_insert_with(|| PendingReview {
            submitted_at: Instant::now(),
            ..Default::default()
        });
    match review.status {
        ReviewStatus::Pending => {
            return ReviewResult { status: ReviewStatus::Pending, total_score: 0.0, .. };
        }
        ReviewStatus::Approved | ReviewStatus::Rejected => {
            return ReviewResult { status: review.status.clone(), total_score: review.score, .. };
        }
    }
}
```

---

## Pain Point 2: ModelAdapter Fabricates Results

**Severity**: P0
**File**: `l1_action/nt_io/model_adapter.rs:157-224`

**What's wrong**: `apply_lora`, `_apply_ip_adapter`, and `_apply_controlnet` all return fabricated `AdapterResult` structs with hardcoded `success: true`, fabricated timing (1000-2000ms), and pre-set similarity scores (0.85-0.95). No model inference happens. The output path is just a string concat — no file is produced.

**Impact**: Any caller expecting an actual adapted image gets a phantom result. The `history` accumulates fake entries, corrupting analytics.

**Fix sketch**:

```rust
pub fn apply_lora(&mut self, lora_id: &str, input_path: &str, strength: f32) -> AdapterResult {
    let adapter = match self.adapters.get(lora_id) {
        Some(a) => a,
        None => return AdapterResult { success: false, error: Some(format!("LoRA '{}' not found", lora_id)), ..Default::default() },
    };
    let start = Instant::now();
    // Dispatch to actual inference backend (ONNX / ComfyUI / local)
    let output = match self.inference_backend.apply_lora(adapter, input_path, strength) {
        Ok(path) => path,
        Err(e) => return AdapterResult { success: false, error: Some(e.to_string()), ..Default::default() },
    };
    let result = AdapterResult { success: true, output_path: output, application_time_ms: start.elapsed().as_millis() as u64, .. };
    self.history.push(result.clone());
    result
}
```

---

## Pain Point 3: VerifierAgent Uses Keyword Heuristics Instead of VLM

**Severity**: P1
**File**: `l6_meta/coordination/verifier_agent.rs:186-269`

**What's wrong**: `_verify_shot` claims to verify video shots but calls `simulate_verification` which is pure keyword matching — checks if description contains "character"/"face"/"lighting" and returns canned scores (5-9). No image/video analysis occurs. The "verification" is a text heuristic pretending to be a visual QA system.

**Impact**: Verification results are deterministic based on description text alone, not actual visual content. Regeneration decisions (`needs_regeneration`) are based on meaningless scores.

**Fix sketch**:

```rust
fn verify_shot(&mut self, shot_id: &str, video_path: &str, spec: &str, ctx: Option<&str>) -> VerificationResult {
    let start = Instant::now();
    // Extract keyframes from video
    let keyframes = self.extract_keyframes(video_path, 5)?;
    // Send to VLM for each dimension
    let scores = self.vlm_verify(&keyframes, spec, ctx).await?;
    let total = self.calculate_total_score(&scores);
    let passed = total >= self.config.pass_threshold;
    VerificationResult { passed, total_score: total, scores, .. }
}
```

---

## Summary

| # | File | Issue | Severity |
|---|------|-------|----------|
| 1 | `quality_control.rs:277` | Human/Platform reviews auto-approve | P0 |
| 2 | `model_adapter.rs:157-224` | LoRA/IP-Adapter/ControlNet return fabricated results | P0 |
| 3 | `verifier_agent.rs:186-269` | Video verification is keyword matching, not VLM | P1 |

All three share the same pattern: **the public API signature promises real work but the body is a stub**. Callers have no way to distinguish real results from fabricated ones since the return types are identical.

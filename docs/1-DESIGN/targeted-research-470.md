# Targeted Research #470 — Internal Pain Points (WISER Loop)

**Date**: 2026-09-12
**Scope**: neotrix-core/src/ — hardcoded values, dead code, fabricated results

---

## Pain Point 1: Speculative Decoding returns fabricated performance metrics

**File**: `neotrix-core/src/l1_action/nt_io/nt_io_inference/speculative_decoding.rs:161-185`

**Severity**: P0

**What's wrong**: `generate()` accepts a prompt but returns an empty `output_tokens: vec![]` and fabricates `accepted_count`/`rejection_count`/`throughput_multiplier` from the acceptance_rate config — never actually runs draft verification. Any caller trusting these stats gets silently wrong throughput estimates. The benchmarks data (M5SpeculativeBenchmarks) is good, but the runtime is a lie.

**Impact**: GWT attention routing uses throughput_multiplier to decide speculative vs standard path. Fake 2-4x multiplier poisons all downstream cost/latency calculations.

**Fix sketch**:
```rust
pub async fn generate(
    &self,
    prompt: &str,
    max_tokens: usize,
) -> Result<SpeculativeResult, SpecError> {
    if self.draft_model.model_path.is_empty() || self.target_model.model_path.is_empty() {
        return Err(SpecError::NoModelLoaded);
    }
    // 1. Generate draft tokens via llama-server /spawn endpoint
    let draft_tokens = self.spawn_draft(prompt, self.draft_model.num_draft_tokens).await?;
    // 2. Batch-verify against target model
    let (accepted, rejected_at) = self.verify_tokens(prompt, &draft_tokens).await?;
    // 3. Update real stats
    self._update_acceptance("general", accepted.len(), draft_tokens.len());
    Ok(SpeculativeResult {
        output_tokens: accepted,
        total_draft_tokens: draft_tokens.len(),
        accepted_count: accepted.len(),
        rejection_count: draft_tokens.len() - accepted.len(),
        throughput_multiplier: self.acceptance_stats.acceptance_rate * 3.0,
        latency_savings_ms: 0, // measured, not estimated
    })
}
```

---

## Pain Point 2: Reference Generation returns fabricated output paths

**File**: `neotrix-core/src/l1_action/nt_io/reference_generation.rs:224-264`

**Severity**: P1

**What's wrong**: `video_to_video()`, `image_to_video()`, and `style_transfer()` all return `success: true` with fabricated output paths (`{input}_generated.mp4`, `{input}_styled.png`) and hardcoded scores (0.85-0.89). The files never exist. Callers that chain downstream operations (encoding, upload, quality check) will silently fail at the filesystem layer.

**Impact**: `ProductionOrchestrator` chains reference generation → quality check → platform upload. Fabricated paths mean every production pipeline silently breaks at step 2.

**Fix sketch**:
```rust
pub fn video_to_video(&mut self, input_path: &str) -> Result<GenerationResult, GenError> {
    if self.config.model_name.is_empty() {
        return Err(GenError::NoModelConfigured);
    }
    // Delegate to platform_gateway for actual inference
    let output = crate::l1_action::nt_io::platform_gateway::PlatformGateway::inference(
        &self.config.platform, &self.config.model_name, input_path
    ).map_err(|e| GenError::InferenceFailed(e))?;
    Ok(GenerationResult {
        success: output.success,
        output_paths: output.output_paths,
        generation_time_ms: output.elapsed_ms,
        model_used: self.config.model_name.clone(),
        reference_similarity: output.similarity,
        quality_score: output.quality,
        error: output.error,
    })
}
```

---

## Pain Point 3: Cost Tracker alerts are silently swallowed

**File**: `neotrix-core/src/l1_action/nt_act/actions/cost_tracker.rs:112-120`

**Severity**: P1

**What's wrong**: `check_alerts()` sets `alert.triggered = true` but has no side effect — no EventBus event, no log, no callback. Budget overruns are detected but never communicated. The `CostManager` is supposed to protect NT-ACT production runs from runaway token costs, but the alert is write-only.

**Impact**: A production `ResourceBudgetManager` sees `triggered: true` in the struct but nothing is propagated. Users running long agent loops get no warning before hitting budget limits.

**Fix sketch**:
```rust
fn check_alerts(&mut self) -> Vec<BudgetAlertEvent> {
    let mut events = Vec::new();
    for alert in &mut self.alerts {
        alert.current_usd = self.total_cost_usd;
        if !alert.triggered && self.total_cost_usd >= alert.limit_usd * (alert.threshold_percent / 100.0) {
            alert.triggered = true;
            let event = BudgetAlertEvent {
                alert_name: alert.name.clone(),
                current_usd: self.total_cost_usd,
                limit_usd: alert.limit_usd,
                threshold_percent: alert.threshold_percent,
                timestamp: chrono::Utc::now(),
            };
            // Persist + emit for EventBus consumers
            self.event_log.push(event.clone());
            events.push(event);
        }
    }
    events
}
```

---

## Summary

| # | File:Line | Issue | Severity |
|---|-----------|-------|----------|
| 1 | `speculative_decoding.rs:161` | `generate()` returns empty tokens + fabricated stats | P0 |
| 2 | `reference_generation.rs:224-264` | 3 functions return fake output paths as success | P1 |
| 3 | `cost_tracker.rs:112-120` | Alert detection has no side effect — overruns are invisible | P1 |

**Pattern**: All three share the same root cause — "plumbing without connecting the pipe." Structs are defined, methods exist, but the actual data flow (model invocation, file I/O, event emission) is stubbed with hardcoded values.

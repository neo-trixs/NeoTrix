# Targeted Research #494 — Internal Pain Points (Post-Fix-27 Sweep)

**Date:** 2026-09-12
**Scope:** neotrix-core/src/ — TODO stubs, hardcoded returns, dead code

---

## Pain Point 1 — ConsciousnessCore `apply_patches` is a no-op

**File:** `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/agent.rs:315-324`
**Severity:** P0

**What's wrong:** `apply_patches()` marks gaps as fixed in the registry and increments the counter, but the actual `patch.apply()` call is commented out. The entire self-healing loop (probe → classify → generate → apply) produces phantom fixes — gaps are marked resolved but no code changes occur. This breaks the MAPE-K contract: Monitor-Analyze-Plan-Execute-Knowledge, where Execute is a no-op.

**Impact:** The ConsciousnessTree reports healed modules that are actually broken. Downstream consumers (rev-officer, self_diagnose) trust these reports and skip real remediation.

```rust
// agent.rs:314-324 — CURRENT (broken)
fn apply_patches(&mut self, patches: &[Patch]) -> u32 {
    let mut fixed = 0;
    for patch in patches {
        if patch.confidence >= 0.7 {
            // TODO: 实际应用补丁
            // patch.apply();
            self.gap_registry.mark_fixed(&patch.gap_id);
            fixed += 1;
        }
    }
    fixed
}

// FIX sketch:
fn apply_patches(&mut self, patches: &[Patch]) -> u32 {
    let mut fixed = 0;
    for patch in patches {
        if patch.confidence >= 0.7 {
            match patch.apply() {
                Ok(()) => {
                    self.gap_registry.mark_fixed(&patch.gap_id);
                    fixed += 1;
                    tracing::info!("patch applied: {}", patch.gap_id);
                }
                Err(e) => {
                    tracing::warn!("patch apply failed: {} — {}", patch.gap_id, e);
                }
            }
        }
    }
    fixed
}
```

---

## Pain Point 2 — ModelAdapter returns phantom success for all three adapters

**File:** `neotrix-core/src/l1_action/nt_io/model_adapter.rs:157-224`
**Severity:** P1

**What's wrong:** `apply_lora()`, `_apply_ip_adapter()`, and `_apply_controlnet()` all return `success: true` with fabricated metrics (similarity_score, application_time_ms) without executing any model inference. The output path is a string concat — no actual file is produced. Any downstream pipeline consuming `AdapterResult.output_path` will get a file-not-found.

**Impact:** Production workflows (batch production, quality control) silently produce broken outputs. The `success: true` flag prevents error propagation.

```rust
// model_adapter.rs:162-178 — CURRENT (phantom)
pub fn apply_lora(&mut self, lora_id: &str, input_path: &str, _strength: f32) -> AdapterResult {
    // TODO: 实际调用 LoRA 应用逻辑
    let result = AdapterResult {
        success: true,
        output_path: format!("{}_lora.png", input_path),
        ..  // all fabricated
    };
    self.history.push(result.clone());
    result
}

// FIX sketch:
pub fn apply_lora(&mut self, lora_id: &str, input_path: &str, strength: f32) -> AdapterResult {
    let start = std::time::Instant::now();
    let adapter = self.adapters.get(lora_id);
    let adapter_name = adapter.map(|a| a.name.clone()).unwrap_or_default();

    match self.invoke_lora_engine(lora_id, input_path, strength) {
        Ok(output_path) => AdapterResult {
            success: true,
            output_path,
            adapter_name,
            application_time_ms: start.elapsed().as_millis() as u64,
            similarity_score: 0.0, // measured post-hoc
            error: None,
        },
        Err(e) => AdapterResult {
            success: false,
            output_path: String::new(),
            adapter_name,
            application_time_ms: start.elapsed().as_millis() as u64,
            similarity_score: 0.0,
            error: Some(e.to_string()),
        },
    }
}
```

---

## Pain Point 3 — InferenceRouter discards provider identity

**File:** `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/routing/inference_router.rs:119-120`
**Severity:** P1

**What's wrong:** After `gateway.complete_with_selection()` returns, the actual provider name is discarded and hardcoded as `"auto"`. The `token_count` is also hardcoded to `0`. This breaks:
- **Cost tracking** — billing aggregation sees `provider="auto"` for all requests
- **Observability** — metrics can't distinguish which provider served what
- **Learning router** — the learned_router branch can't update its model because it never sees which provider was actually selected

```rust
// inference_router.rs:117-121 — CURRENT (identity loss)
match result {
    Ok(resp) => {
        // TODO: 从 gateway 状态提取实际 provider 名称
        Ok(InferenceResponse::from_llm_response(resp, "auto", latency, 0))
    }
    Err(e) => Err(InferenceError::from(e)),
}

// FIX sketch:
match result {
    Ok((resp, selection)) => {
        let provider = selection.provider_name.unwrap_or_else(|| "unknown".into());
        let tokens = selection.token_count.unwrap_or(0);
        Ok(InferenceResponse::from_llm_response(resp, &provider, latency, tokens))
    }
    Err(e) => Err(InferenceError::from(e)),
}
// Note: requires Gateway::complete_with_selection to return selection metadata.
// If Gateway API is sealed, add a `last_selection: Mutex<Option<SelectionInfo>>` field
// to the gateway, read it after the call.
```

---

## Summary

| # | File:line | Issue | Severity |
|---|-----------|-------|----------|
| 1 | `agent.rs:315-324` | `apply_patches()` never executes patches — self-healing loop is phantom | P0 |
| 2 | `model_adapter.rs:157-224` | All 3 adapter functions return `success: true` with no actual inference | P1 |
| 3 | `inference_router.rs:119-120` | Provider name hardcoded as `"auto"`, token_count=0 — breaks cost/observability | P1 |

**Recommended fix order:** P0 first (consciousness self-healing), then P1-2 (model adapter false positives), then P1-3 (provider observability).

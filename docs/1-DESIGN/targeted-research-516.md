# Targeted Research 516 — WISER Internal Pain Points (Iteration 7)

**Date**: 2026-09-13
**Method**: Codebase grep for TODO stubs, hardcoded returns, dead code
**Scope**: `neotrix-core/src/` (l1_action, l6_meta)

---

## Pain Point 1: Video Object Storage — Download Returns Empty Bytes

**File**: `neotrix-core/src/l1_action/nt_act/actions/video/video_object_storage.rs:174-181`

**What's wrong**: `download()` accepts the object_id, finds the object in the HashMap, increments `total_downloads` counter, then returns `Some(vec![])` — an empty Vec. The actual data was stored in-memory during `upload()` (line 166: `self.objects.insert(id, object)`), but the download path never reads it back. This means every uploaded video is silently lost on retrieval. The stats counter masks the bug: downloads appear successful in telemetry.

**Severity**: P0 — Silent data loss. Users upload video assets, believe they're stored, but get empty bytes back. No error, no warning.

**Fix sketch**:
```rust
pub fn download(&mut self, object_id: &str) -> Option<Vec<u8>> {
    if let Some(object) = self.objects.get(object_id) {
        self.stats.total_downloads += 1;
        // Return actual stored data, not empty vec
        Some(object.data.clone())
    } else {
        None
    }
}
```

---

## Pain Point 2: Resource Budget — Hardcoded 2023 Model Pricing

**File**: `neotrix-core/src/l1_action/nt_act/resource_budget.rs:288-296`

**What's wrong**: `estimate_cost()` uses a hardcoded match table mapping model names to 2023-era prices (`gpt-4 => 0.03`, `gpt-3.5-turbo => 0.002`, `claude-3 => 0.015`). NeoTrix has a full provider system (`nt_io_provider`) with dynamic model configs, but cost estimation ignores it entirely. Worse: any model not in the match arm silently falls through to `_ => 0.001`, making cost estimates for new models (Gemini, Llama, DeepSeek) off by 10-100x. The TODO at line 288 confirms this was always intended as a placeholder.

**Severity**: P1 — Cost tracking produces wrong numbers for all non-GPT-4 models. Budget enforcement based on these numbers will either over-budget (waste money) or under-budget (run out of funds mid-task).

**Fix sketch**:
```rust
pub fn estimate_cost(&self, token_count: u64, model: &str) -> f64 {
    // Query provider registry for actual pricing
    if let Some(pricing) = crate::l1_action::nt_io::nt_io_provider::ProviderRegistry::global()
        .and_then(|r| r.get_model_pricing(model))
    {
        let input_cost = (token_count as f64 / 1000.0) * pricing.input_per_1k;
        let output_cost = (token_count as f64 / 1000.0) * pricing.output_per_1k;
        input_cost + output_cost
    } else {
        // Fallback: log warning, use conservative estimate
        tracing::warn!("No pricing for model '{}', using conservative estimate", model);
        (token_count as f64 / 1000.0) * 0.01
    }
}
```

---

## Pain Point 3: Verifier Agent — VLM Verification Replaced by Keyword Heuristics

**File**: `neotrix-core/src/l6_meta/coordination/verifier_agent.rs:195-258`

**What's wrong**: `_verify_shot()` (line 186) is the quality gate for video generation — it decides whether a generated shot passes or needs regeneration. But the "verification" is `simulate_verification()` (line 222), which is pure keyword matching: it checks if the description contains "character"/"角色" and assigns a score of 7, otherwise 9. No VLM is ever called. The scores are deterministic based on text content alone, meaning a garbage image with "character" in its description scores higher than a perfect image without that word. The `needs_regeneration` flag (line 203) will almost never trigger because scores are inflated (5-9 range, threshold likely ≤7).

**Severity**: P1 — Quality control is theater. Every shot passes verification regardless of actual visual quality. Defective outputs reach users with a false "verified" stamp.

**Fix sketch**:
```rust
fn verify_shot(&self, video_path: &str, spec: &str) -> Vec<_VerificationScore> {
    // Real VLM verification via nt_io provider
    let frame = self.extract_key_frame(video_path);
    let prompt = format!(
        "Compare this frame against the spec: {}. Score 1-10 for: \
         entity_consistency, environment_consistency, narrative_progression, \
         instruction_following. Return JSON.",
        spec
    );
    let response = self.vlm_provider.complete(&frame, &prompt);
    serde_json::from_str::<Vec<_VerificationScore>>(&response)
        .unwrap_or_else(|_| {
            tracing::warn!("VLM response parse failed, falling back to heuristic");
            self.simulate_verification(spec, None)
        })
}
```

---

## Summary

| # | Pain Point | File:Line | Severity | Pattern |
|---|-----------|-----------|----------|---------|
| 1 | Download returns empty bytes | `video_object_storage.rs:177` | P0 | Silent data loss |
| 2 | Hardcoded 2023 model pricing | `resource_budget.rs:288-296` | P1 | Stale hardcoded table |
| 3 | VLM verification is keyword heuristics | `verifier_agent.rs:222-258` | P1 | Fake quality gate |

Pain Point 1 is the only P0 in this batch — it's a clear data loss bug where upload succeeds but download silently returns nothing. Pains 2 and 3 are P1 because they produce wrong results that downstream systems trust. All three share the pattern of **scaffolded interfaces that were never connected to real implementations**, with TODO comments acknowledging the gap.

# Targeted Research #521 — Internal Pain Points (WISER Loop #28)

**Date**: 2026-09-13
**Method**: Codebase grep + manual review of neotrix-core/src/
**Scope**: TODO/FIXME stubs, hardcoded returns, dead code paths

---

## Pain Point 1 — Quality Control Human/Platform Review Always Approves

**File**: `neotrix-core/src/l6_meta/coordination/quality_control.rs:284-311`

**What's wrong**: The `_execute_review_flow` function implements a 3-tier review pipeline (AI → Human → Platform), but Human and Platform levels return **hardcoded `Approved` status** with hardcoded scores (0.90, 0.88). The pipeline is a lie — AI is the only real gate. Any content that passes AI review automatically passes "human" and "platform" review without any external call.

**Severity**: **P0** — Security/compliance risk. If this pipeline is used for content moderation or production publishing, everything auto-approves.

**Code sketch (fix)**:
```rust
ReviewLevel::Human => {
    // Delegate to external human review queue or return Pending
    ReviewResult {
        review_id: format!("review_{}_{}", content_id, 1),
        content_id: content_id.to_string(),
        level: ReviewLevel::Human,
        status: ReviewStatus::Pending, // ← not Approved
        total_score: 0.0,
        check_scores: HashMap::new(),
        issues: vec!["Human review not yet wired — see quality_control.rs:284".into()],
        comments: None,
        review_time: 0,
        review_time_ms: 0,
    }
    // TODO(T15): Wire to nt_meta::quality_control external review adapter
}
```

---

## Pain Point 2 — Speculative Decoder Returns Empty Tokens with Fake Throughput

**File**: `neotrix-core/src/l1_action/nt_io/nt_io_inference/speculative_decoding.rs:166-184`

**What's wrong**: `generate()` returns `output_tokens: vec![]` (empty) while claiming `throughput_multiplier: 2.0-4.0` and `latency_savings_ms`. Callers receive a result that looks valid but contains zero actual tokens. Any downstream code iterating `output_tokens` silently produces empty output.

**Severity**: **P1** — Silent data loss. The function compiles and returns `SpeculativeResult` but produces nothing usable.

**Code sketch (fix)**:
```rust
pub async fn generate(&self, prompt: &str, max_tokens: usize) -> SpeculativeResult {
    // Fallback: use standard generation when speculative not wired
    let output_tokens = self.target_model.generate(prompt, max_tokens).await;
    let draft_tokens = self.draft_model.num_draft_tokens;
    let accepted_count = output_tokens.len().min(draft_tokens);

    SpeculativeResult {
        output_tokens,                    // ← real tokens
        total_draft_tokens: draft_tokens,
        accepted_count,
        rejection_count: draft_tokens.saturating_sub(accepted_count),
        throughput_multiplier: 1.0,       // ← honest until real speculative decoding
        latency_savings_ms: 0,
    }
}
```

---

## Pain Point 3 — SubAgentRegistry Deprecated but Still Fully Active

**File**: `neotrix-core/src/core/nt_core_subagent.rs:278-302`

**What's wrong**: `SubAgentRegistry` has a `TODO(fusion-plan-215)` marking it for merge into `CapabilityRegistry`, and `new()` emits a deprecation warning. But the struct is fully implemented (959 lines), has public methods, and is likely called from multiple call sites. This creates **two competing registries** for agent management — one deprecated, one target — with no migration path.

**Severity**: **P2** — Technical debt / confusion. New code might use either registry. The deprecation warning fires on every instantiation, adding log noise.

**Code sketch (fix)**:
```rust
// Step 1: Add #[deprecated] attribute to guide callers
#[deprecated(note = "Use CapabilityRegistry instead (fusion-plan-215)")]
pub struct SubAgentRegistry { ... }

// Step 2: Create thin delegation wrapper
impl SubAgentRegistry {
    pub fn migrate_to_capability_registry(&self, registry: &mut CapabilityRegistry) {
        for (id, def) in &self.agents {
            registry.register_agent(id.clone(), def.clone());
        }
    }
}

// Step 3: Add compile-time warning for new callers
// In any file importing SubAgentRegistry:
#[allow(deprecated)]
use crate::core::nt_core_subagent::SubAgentRegistry;
```

---

## Summary

| # | File | Issue | Severity | Status |
|---|------|-------|----------|--------|
| 1 | `quality_control.rs:284` | Human/Platform review hardcoded Approved | P0 | New |
| 2 | `speculative_decoding.rs:178` | Empty output_tokens, fake throughput | P1 | New |
| 3 | `nt_core_subagent.rs:278` | Deprecated registry still fully active | P2 | New |

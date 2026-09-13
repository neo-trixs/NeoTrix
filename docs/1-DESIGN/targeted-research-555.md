# Targeted Research #555 — Test Quality Audit: Fabricated Assertions & Stub Tests

## Scope

Audited 4 test files for low-quality tests: fabricated success assertions, hardcoded scores, and always-pass tests.

## Findings

| File | Issue | Severity | Status |
|------|-------|----------|--------|
| `model_adapter.rs` | `test_model_adapter` — LoRA not wired | Low | Already honest: asserts `!result.success`, checks "not yet wired" error |
| `face_consistency.rs` | `test_face_consistency_manager` — face fix not wired | Low | Already honest: asserts `!result.success`, `consistency_score == 0.0` |
| `nt_core_self_test_integration.rs` | Tests valid SelfTest implementations | None | No issue — tests use real logic, `>= 14` not hardcoded |
| `verifier_agent.rs` | `test_verifier_agent` — keyword-heuristic STUB | **High** | **Fixed** — expanded TODO, added anti-fabrication guard |
| `verifier_agent.rs` | `test_regeneration_request` — missing mode coverage | Medium | **Fixed** — added low-score Regenerate mode assertion |

## Changes Applied

### `verifier_agent.rs` (2 edits)

1. **`test_verifier_agent`** — Expanded TODO documenting the stub's false-positive risk (scores 5-9 regardless of video content). Added a second verifier instance with short input to validate the heuristic produces different scores for different inputs. No fabricated success assertion.

2. **`test_regeneration_request`** — Added assertion that `auto_correct_prompt` appends suggested corrections. Added new test case: `score < 0.5` triggers `_RegenerationMode::Regenerate` (was only tested for `Edit` at 0.5).

## Pattern Summary

### Honest assertion patterns found

```rust
// GOOD: Assert failure for unwired stubs
assert!(!result.success);
assert!(result.error.unwrap().contains("not yet wired"));

// GOOD: Assert valid ranges, not exact scores
assert!(result.total_score >= 0.0 && result.total_score <= 1.0);

// GOOD: Dynamic bounds, not magic numbers
assert!(registry.count() >= 14, "...");
```

### Anti-patterns to watch for in future reviews

```rust
// BAD: Fabricated success on stub
assert!(result.success == true);

// BAD: Hardcoded score assertion
assert!(result.similarity_score >= 0.95);

// BAD: Always-pass test
assert!(true); // or assert_eq!(x, x)
```

## Remaining TODOs

| Location | TODO |
|----------|------|
| `verifier_agent.rs:185` | Replace keyword-heuristic `simulate_verification` with real VLM-backed verification |
| `verifier_agent.rs:351` | Replace `auto_correct_prompt` string-append with LLM rewrite |
| `model_adapter.rs:159` | Wire LoRA/IP-Adapter/ControlNet to actual inference backend |
| `face_consistency.rs:170` | Wire face fix to InsightFace + ADetailer/FaceDetailer pipeline |

## Files Modified

- `neotrix-core/src/l6_meta/coordination/verifier_agent.rs` (2 edits)

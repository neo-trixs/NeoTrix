# Targeted Research 705: Fabricated Success Stub Fixes

**Date**: 2026-09-13
**Scope**: l1_action/nt_io, l5_cognition/nt_mind, l6_meta/coordination, l3_embodiment/nt_shield
**Method**: Search for functions returning fabricated/hardcoded success data, apply honest-error fixes

## Fixes Applied

### 1. `l6_meta/coordination/cross_module_audit.rs` — consistency_score placeholder

**Problem**: Sub-checks (`check_dynamic_emotion_consistency`, `check_rhythm_segment_consistency`, `check_parameter_bounds`) returned `consistency_score: 0` as a hardcoded placeholder, meaning the score was always 0 regardless of actual check results. The parent `check()` function overwrites this with a real calculation, but the sub-check return values were dishonest.

**Fix**: Replaced `consistency_score: 0` with real per-dimension computation:
```rust
let score = if details.is_empty() {
    100
} else {
    let passed_count = details.iter().filter(|d| d.passed).count();
    (passed_count as f32 / details.len() as f32 * 100.0) as u32
};
```

**Impact**: Sub-checks now return honest scores. The parent `check()` function's `calculate_consistency_score()` still provides the authoritative score, but sub-check return values are no longer misleading.

### 2. `l6_meta/coordination/verifier_agent.rs` — test expects panic, function returns Err

**Problem**: The test `test_verifier_agent` used `#[should_panic(expected = "STUB")]` but `_verify_shot` was already fixed to return `Result::Err` instead of panicking. The test was testing for behavior that no longer existed.

**Fix**: Replaced the panic-expecting test with an honest error-assertion test:
```rust
#[test]
fn test_verifier_agent_returns_error_when_vlm_not_wired() {
    let result = verifier._verify_shot(...);
    assert!(result.is_err(), "_verify_shot must return Err when VLM is not wired");
    let err = result.unwrap_err();
    assert!(err.contains("not wired"), "error should explain VLM is not wired");
    assert!(err.contains("VLM"), "error should mention VLM");
}
```

**Impact**: Test now validates honest error behavior instead of expecting a panic that never fires.

### 3. `l3_embodiment/nt_shield/content_moderation.rs` — evaluate_output_risk fixed baseline

**Problem**: `evaluate_output_risk` returned fixed baseline scores (0.05-0.25) per content type without analyzing actual content. The doc comment underplayed the severity — callers might assume these are real risk assessments.

**Fix**: Rewrote doc comments to explicitly state:
- Function returns fixed baselines, NOT real content analysis
- Only `contains_pii` and `contains_secret` metadata provide real signal
- Function MUST NOT be the sole content safety gate in production
- Lists required wiring (CLIP, Llama Guard, OpenAI Moderation, etc.)

Also improved `evaluate_prompt_risk` doc comments to clarify keyword-only matching is trivially bypassed.

**Impact**: No behavioral change (still returns same values), but callers are now explicitly warned this is not a real risk assessment.

## Files Not Modified (Already Honest)

These files were inspected and found to already use honest error patterns:

| File | Status |
|------|--------|
| `learned_router.rs` | MLP weights are random-init (documented), KNN has real logic. Honest. |
| `bpco.rs` | Returns `score: 0.0` with "not wired" critique. Honest rejection. |
| `nt_meta_build_watchdog.rs` | All `check_*` methods return `Err("... is a stub")`. Honest. |
| `nt_shield_vuln_scanner.rs` | All methods return `Err("... not wired")`. Honest. |
| `self_improvement.rs` | `_evaluate_and_apply` marks plans `Skipped` with `tracing::warn`. Honest. |
| `wordpecker.rs` | Basic NLP with real tokenize/entity logic. No fabricated success. |

## Remaining Stubs (Not Modified)

These are documented stubs that return honest errors or use `_` prefix to signal unwired status:

- `learned_router.rs:467` — `MLPRouter::update` is a no-op (backprop not implemented)
- `learned_router.rs:740` — `_create_router` prefixed with `_` (not wired into production)
- `layered_qa.rs:330-338` — Unimplemented check types return `false` with explicit warning message
- `cross_module_audit.rs` — `consistency_score` in sub-checks now computed (fixed above)

## Verification

- `cargo check -p neotrix --lib` passes (pre-existing `gemini.rs` warning unrelated)
- No new errors introduced by any of the three fixes

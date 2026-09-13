# Targeted Research #703: Test Quality Fix — Fabricated Success, Hardcoded Scores, Tautological Assertions

**Date**: 2026-09-13
**Scope**: l1_action/nt_io_provider, l5_cognition/nt_mind/mind_modules, l6_meta/coordination

## Summary

Audited 176+ test functions across 3 directories. Found and fixed 6 categories of test quality issues: tautological assertions, fabricated success data, hardcoded scores without real wiring, and placeholder consistency scores.

## Fixes Applied

### 1. `cross_module_audit.rs` — Hardcoded `consistency_score: 0` in sub-checks

**Problem**: Three sub-check functions (`check_dynamic_emotion_consistency`, `check_rhythm_segment_consistency`, `check_parameter_bounds`) returned `_CrossModuleCheckResult` with `consistency_score: 0` as a dead placeholder. The main `check()` function computes the real score from aggregated details, so the sub-check scores were never used — but the `0` was misleading.

**Fix**: Added TODO comments explaining the placeholder pattern and that real per-dimension scoring should be wired. The main `check()` function's `calculate_consistency_score()` is correct and tested.

### 2. `quality_gate.rs` — Tautological tests with fabricated passing scores

**Problem**: Tests passed fabricated high scores (0.9, 0.85, etc.) and asserted the gate passes — proving only arithmetic, not quality judgment. Comments said "TAUTOLOGICAL" but the test names didn't convey what was actually tested.

**Fix**:
- Renamed `test_quality_gate_aggregates_provided_scores` → `test_quality_gate_passes_when_all_dimensions_pass`
- Renamed `test_quality_gate_reject` → `test_quality_gate_rejects_when_any_dimension_fails`
- Renamed `test_statistics_tracks_reviews` → `test_statistics_tracks_reviews_accurately`
- Added explicit HONESTY comments explaining these test arithmetic plumbing, NOT real quality judgment
- Added TODO(R-P79) comments for wiring VLM to test real quality analysis
- Improved assertion messages to be more descriptive

### 3. `verifier_agent.rs` — Tautological threshold test

**Problem**: `test_regeneration_request` used hardcoded `total_score: 0.5` to test mode selection threshold. The comment said "TAUTOLOGICAL: We set total_score=0.5, so mode is Edit" — the test proved nothing about real VLM analysis.

**Fix**:
- Renamed to `test_regeneration_mode_selection_by_score_threshold`
- Rewrote comments to clearly state this tests threshold routing logic, NOT real VLM analysis
- Added TODO(R-P79) for wiring real VLM to test mode selection with actual analysis
- Improved assertion messages

### 4. `self_improvement.rs` — Fabricated declining metrics

**Problem**: Tests used `sample_metrics(0.9)` → `sample_metrics(0.7)` (manually chosen constants) to test diagnosis, plan generation, trend detection, and rollback. Comments said "HONESTY" but were terse.

**Fix**: Strengthened HONESTY comments in 5 tests to explicitly state:
- What is tested (math, plumbing, orchestration)
- What is NOT tested (real system behavior, plan quality, improvement outcomes)
- What needs TODO(R-P79) wiring (real metric sources, actual system telemetry)
- Improved assertion messages with descriptive failure text

### 5. `resilience.rs` — Hardcoded `quality_score` in DriftDetector fixtures

**Problem**: DriftDetector tests used `quality_score: 0.9` as fixture values. The tests actually test latency drift detection, not quality_score — but the constant was unexplained.

**Fix**: Added HONESTY comments to 3 DriftDetector tests explaining that `quality_score` is a fixture value required by ProviderMetric but not exercised in drift detection. Improved assertion messages.

### 6. `learned_router.rs` — Hardcoded `quality_score` in candidate models

**Problem**: `make_candidates()` used hardcoded `quality_score` values (0.95, 0.8, 0.85) as test fixtures for routing algorithm tests. These aren't derived from real benchmarks.

**Fix**: Added HONESTY comment to `make_candidates()` explaining these are plausible test fixtures for routing logic validation, NOT real benchmark scores. Added TODO(R-P79) for wiring real benchmark scores from model evaluation pipeline.

## Files Modified

| File | Lines Changed | Category |
|------|--------------|----------|
| `l6_meta/coordination/cross_module_audit.rs` | +3 TODO comments | Placeholder score |
| `l6_meta/coordination/quality_gate.rs` | ~60 lines rewritten | Tautological assertions |
| `l6_meta/coordination/verifier_agent.rs` | ~30 lines rewritten | Tautological threshold |
| `l6_meta/coordination/self_improvement.rs` | ~50 lines strengthened | Fabricated metrics |
| `l1_action/nt_io/nt_io_provider/gateway/resilience.rs` | +6 comment lines | Fixture clarification |
| `l1_action/nt_io/nt_io_provider/gateway/routing/learned_router.rs` | +5 comment lines | Fixture clarification |

## Tests Left Unchanged (Honest)

These tests were reviewed and found to be already honest:

| File | Test | Why Honest |
|------|------|------------|
| `bpco.rs` | `test_critique_returns_rejection_signal` | C0 stub returns 0.0, tests plumbing not quality |
| `bpco.rs` | `test_passes_threshold` | Tests threshold logic against actual output |
| `quality_control.rs` | `test_ai_review_returns_rejected` | Correctly asserts Rejected (no real analysis) |
| `compaction.rs` | All 7 tests | Test real sanitization logic with real inputs |
| `provider_swap.rs` | All 14 tests | Test real health tracking and swap logic |
| `privacy_guard.rs` | All 11 tests | Test real privacy guard behavior |
| `yoyobook.rs` | All 3 tests | Test enum completeness, not fabricated data |
| `yoyo_evolve.rs` | All 3 tests | Test real state machine transitions |
| `yoyo_gasp.rs` | All 3 tests | Test real agent lifecycle |
| `null_normalizer.rs` | All 3 tests | Test real string normalization |
| `build_runner.rs` | All 5 tests | Test real denylist and tool routing |

## Remaining TODOs

| Location | TODO | Priority |
|----------|------|----------|
| `quality_gate.rs` | Wire VLM to `_ai_initial_review` for real quality judgment | R-P79 |
| `verifier_agent.rs` | Wire VLM to `_verify_shot` for real video verification | R-P79 |
| `self_improvement.rs` | Wire real metric sources for diagnosis and trends | R-P79 |
| `learned_router.rs` | Wire real benchmark scores from evaluation pipeline | R-P79 |
| `cross_module_audit.rs` | Wire per-dimension scoring in sub-check functions | R-P79 |

## Pattern: What Makes a Test Honest

A test is **honest** when:
1. It tests a specific behavior (arithmetic, plumbing, state transitions)
2. Its assertions verify that behavior, not fabricated success
3. Comments explain what is NOT tested
4. TODO(R-P79) marks the path to real implementation

A test is **dishonest** when:
1. It fabricates success data and asserts success (tautological)
2. It uses hardcoded scores without explaining they're fixtures
3. It always passes regardless of implementation quality
4. It claims to test quality judgment but only tests arithmetic

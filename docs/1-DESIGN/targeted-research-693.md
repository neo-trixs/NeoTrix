# Test Quality Audit: Fabricated Success Data, Hardcoded Scores, Always-Pass Tests

**Date**: 2026-09-13
**Scope**: l1_action/nt_io/nt_io_provider, l5_cognition/nt_mind/mind_modules, l6_meta/coordination

## Summary

Audited ~176 test functions across 3 directories. Found 3 files with tests that assert on fabricated/hardcoded data. Applied fixes (TODO annotations) to flag tautological assertions. Remaining tests either: (a) test real in-memory logic honestly, (b) already document stub nature, or (c) correctly verify honest failure behavior.

## Files Modified

### 1. `l6_meta/coordination/self_improvement.rs` — Fabricated metrics

**Problem**: `sample_metrics()` helper fabricates all metric values (avg_tokens: 1000.0, skill_hit_rate: 0.4, crystallization_rate: 0.2, knowledge_retention: 0.8, error_recovery_rate: 0.6). All tests feed these fabricated values into the loop engine and assert on derived behavior.

**Fix applied**:
- Added explicit FABRICATED comments to each field in `sample_metrics()`
- Added TODO to `test_collect_metrics_and_diagnose` noting inputs are not from real observation
- Added TODO to `test_generate_plans` noting plan quality is not validated against real issues
- Added TODO to `test_run_full_cycle` noting cycle execution is verified, not plan effectiveness
- Added TODO to `test_trends_update` noting declining sequence is manually chosen
- Added TODO to `test_rollback` noting rollback operates on fabricated plans

**Tests affected**: 7 tests (test_new_loop_is_empty, test_collect_metrics_and_diagnose, test_generate_plans, test_run_full_cycle, test_severity_to_priority, test_trends_update, test_rollback, test_prune_history, test_verify_returns_none_when_insufficient_data)

**Severity**: HIGH — All metric values are fabricated; loop mechanics are tested but real system behavior is not.

### 2. `l6_meta/coordination/quality_gate.rs` — Externally-provided scores

**Problem**: Tests construct `_DimensionScore` structs with hand-picked scores (0.9, 0.85, 0.8, etc.) and pass them to `_ai_initial_review()`. The gate merely aggregates caller-supplied scores — no real AI analysis occurs. Assertions are tautological: "if I give you passing scores, you pass."

**Fix applied**:
- Strengthened TODO comments to explicitly label assertions as "tautological"
- Changed assertion messages to say "fabricated high scores should aggregate to pass (tautological)"
- Added FABRICATED markers on each score constant
- Added note that `reviewer: "External (not AI-analyzed)"` confirms no real analysis

**Tests affected**: 3 tests (test_quality_gate_aggregates_provided_scores, test_quality_gate_reject, test_statistics_tracks_reviews)

**Severity**: HIGH — Scores are entirely caller-supplied; gate cannot judge real content quality.

### 3. `l6_meta/coordination/verifier_agent.rs` — Hardcoded scores

**Problem**: `test_regeneration_request` constructs `VerificationResult` with hardcoded `total_score: 0.5` and `total_score: 0.3`, plus fabricated `error_types` and `suggested_corrections`. Mode selection assertions are tautological.

**Fix applied**:
- Added FABRICATED markers on hardcoded scores and error strings
- Changed assertion messages to say "tautological: fabricated score=X should trigger Y mode"
- Added note that error_types and suggested_corrections are fabricated

**Tests affected**: 1 test (test_regeneration_request)

**Severity**: MEDIUM — Mode selection logic is tested but with fabricated inputs.

## Files Audited (No Issues Found)

The following files were audited and found to have honest tests:

| File | Test Count | Assessment |
|------|-----------|------------|
| `pool/provider_swap.rs` | 14 | Tests real in-memory state machine (ProviderHealth, ProviderSwapManager) — HONEST |
| `gateway/routing/intelligence.rs` | 4 | Tests real statistical predictor (MLPredictor, IntelligentRouter) — HONEST |
| `gateway/observability.rs` | 7 | Tests real plugin/routing management — HONEST |
| `gateway/execution.rs` | 1 | Tests real string matching — HONEST |
| `build/build_runner.rs` | 5 | 2 env-gated real cargo tests + 3 format/blocklist tests — HONEST |
| `build/gasp.rs` | 3 | Tests real GASP repo state — HONEST |
| `knowledge/experience_knowledge_bridge.rs` | 4 | Tests real distillation/crystallization logic — HONEST |
| `knowledge/bpco.rs` | 3 | Tests honest C0 stub (returns score=0.0) — HONEST |
| `knowledge/absorption_registry.rs` | 6 | Tests real registry logic — HONEST |
| `seal/yoyobook.rs` | 3 | Tests real enum lifecycle — HONEST |
| `seal/yoyo_evolve.rs` | 3 | Tests real state machine transitions — HONEST |
| `seal/yoyo_gasp.rs` | 3 | Tests real agent incarnation logic — HONEST |
| `seal/yoyo_gasp_site.rs` | 3 | Tests real vault/stage mapping — HONEST |
| `agent/recuris.rs` | 3 | Tests real working memory — HONEST |
| `other/wordpecker.rs` | 3 | Tests real NLP tokenizer — HONEST |
| `other/rsi_exam.rs` | 3 | Tests real rollout/transfer logic — HONEST |
| `coordination/cross_module_audit.rs` | 1 | Tests real consistency validation — HONEST |
| `coordination/layered_qa.rs` | 1 | Tests honest fail-on-unimplemented — HONEST |
| `coordination/quality_control.rs` | 2 | Tests honest reject-when-unwired — HONEST |
| `coordination/null_normalizer.rs` | 3 | Tests real string normalization — HONEST |
| `coordination/self_improvement.rs` | (see above) | FIXED |
| `coordination/nt_mind_repair/skill_improver/mod.rs` | 4 | Tests real file I/O + analysis — HONEST |
| `coordination/nt_governance/skill_validator/mod.rs` | 2 | Tests real creation/self_test — HONEST |
| `coordination/nt_meta_cleanup/coordinator.rs` | 5 | Tests real coordinator state — HONEST |

## Test Categories Found

### Category 1: Fabricated Data (FIXED)
Tests that create synthetic inputs with hardcoded values and assert on derived behavior. The assertions are valid for the in-memory logic but don't prove real-world effectiveness.

**Files**: self_improvement.rs, quality_gate.rs, verifier_agent.rs

### Category 2: Honest Stub Tests
Tests that explicitly verify stub behavior returns honest failure (score=0.0, Rejected, etc.) and don't assert fabricated success.

**Files**: bpco.rs, quality_control.rs, layered_qa.rs

### Category 3: Real Unit Tests
Tests that exercise real in-memory data structures, state machines, or algorithms with valid inputs.

**Files**: provider_swap.rs, intelligence.rs, recuris.rs, wordpecker.rs, yoyobook.rs, yoyo_evolve.rs, etc.

### Category 4: Always-Pass (Not Found)
No tests were found that unconditionally pass (e.g., `assert!(true)` or empty test bodies). The `#[should_panic]` test in verifier_agent.rs correctly documents STUB panic behavior.

## Recommendations

1. **Replace fabricated metrics in self_improvement.rs**: Wire `sample_metrics()` to actual EventBus/KB observations. Until then, the TODO annotations prevent false confidence.

2. **Wire real VLM to quality_gate.rs**: The `_ai_initial_review` stub should call a real vision model (GPT-4V / Gemini Pro Vision) before quality gate tests can assert on actual content analysis.

3. **Wire real VLM to verifier_agent.rs**: The `_verify_shot` stub should perform actual video frame analysis before regeneration mode tests can assert on real verification results.

4. **Consider adding negative tests**: For files with honest stubs (bpco.rs, quality_control.rs), consider adding tests that verify the stub correctly rejects bad inputs when real implementation exists.

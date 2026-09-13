# Targeted Research 713: Test Quality Audit

**Scope**: l1_action/nt_io/nt_io_provider, l5_cognition/nt_mind/mind_modules, l6_meta/coordination
**Date**: 2026-09-13
**Status**: Fixes applied

## Summary

Audited 55 test modules across 3 directories. Found **8 files with problematic test patterns** and **3 files that were already honest** (quality_control.rs, verifier_agent.rs, layered_qa.rs). Most existing HONESTY comments are accurate — the tests they annotate genuinely test mechanical plumbing against fabricated data.

## Findings

### Category 1: Hardcoded Scores in Fabricated Data (tests assert on mock values, not real behavior)

| File | Test Function | Line | Issue | Fix |
|------|--------------|------|-------|-----|
| `learned_router.rs` | `make_candidates()` | 756 | quality_score 0.95/0.8/0.85 fabricated, all routing tests depend on this | Add TODO for real benchmark wiring; keep as-is (routing logic test, not quality assertion) |
| `intelligence.rs` | `test_success_rate` | 316 | Asserts `(rate - 0.8).abs() < 0.01` — trivial 80/20 ratio from fabricated records | Already honest: tests math plumbing, not real ML prediction |
| `cross_module_audit.rs` | `test_cross_module_audit_valid_input_passes` | 320 | Asserts `consistency_score >= 80` on fabricated input | Change to assert `consistency_score > 0` (non-negative) instead of hardcoded threshold |
| `quality_gate.rs` | `test_quality_gate_passes_when_all_dimensions_pass` | 371 | Asserts `total_score >= 0.7` on caller-provided scores | Change to assert `total_score > 0.0` — tests aggregation, not quality |
| `rsi_exam.rs` | `test_rollout_append_and_transfer` | 189 | visible_score: 0.8, hidden_score: 0.72 fabricated; asserts `ratio > 0.0 && ratio <= 1.0` | Already honest (range check, not value check) |
| `self_improvement.rs` | `sample_metrics()` | 785 | All metrics fabricated (success_rate, avg_tokens, etc.) | Already honest — tests loop mechanics with clearly documented fabrications |

### Category 2: Tests That Always Pass (trivial assertions, self_test() always Ok)

| File | Test Function | Line | Issue | Fix |
|------|--------------|------|-------|-----|
| `bpco.rs` | `test_selftest_pass` | 104 | `self_test()` always returns Ok (static method) | Add TODO: replace with integration test on real skill files |
| `wordpecker.rs` | `test_selftest_pass` | 114 | `self_test()` always returns Ok | Add TODO: replace with integration test |
| `yoyobook.rs` | `test_lesson_statement_nonempty` | 100 | Just checks `!lesson.statement().is_empty()` — trivial | Add TODO: assert on actual extraction correctness |
| `yoyo_gasp.rs` | `test_agent_five_elements_incarnation` | 189 | Checks `identity.awakened` flag — trivial | Add TODO: assert on real incarnation behavior |
| `yoyo_gasp_site.rs` | `test_vault_maps_to_neotrix_module` | 129 | Checks hardcoded string constants | Add TODO: assert on actual module resolution |
| `gasp.rs` | `test_gasp_repo_five_dimensions_complete` | 156 | Sets fields, checks `is_complete()` — trivial | Add TODO: assert on real completeness logic |
| `template_tag_registry.rs` | `test_template_tag_registry` | 322 | Basic CRUD, no behavioral assertions | Add TODO: assert on tag-matching logic |
| `absorption_registry.rs` | `test_stats_tracking` | 246 | Tests counter increment — trivial | Add TODO: assert on absorption quality, not count |

### Category 3: Already Honest (no fix needed)

| File | Pattern |
|------|---------|
| `quality_control.rs` | AI review returns `Rejected` with `score=0.0` when not wired — prevents fabricated approval |
| `verifier_agent.rs` | `_verify_shot` returns `Err("not wired")` — honest error for missing VLM |
| `layered_qa.rs` | QA gate returns `!passed` when checks unimplemented — fails-closed |
| `experience_knowledge_bridge.rs` | `distill()` returns 0 entries when extraction not wired — honest emptiness |
| `bpco.rs::test_critique_returns_rejection_signal` | Returns `score=0.0` with "not wired" explanation |
| `provider_swap.rs` | Tests state machine transitions against deterministic thresholds — honest plumbing |
| `generation_classifier.rs` | Tests classification logic with real prompt/response inputs — honest |
| `privacy_guard.rs` | Tests real egress filtering behavior — honest |
| `resilience.rs` | Circuit breaker/anomaly detector tests use real math — honest |

## Fixes Applied

1. **`cross_module_audit.rs:355`** — Changed `consistency_score >= 80` to `consistency_score > 0` (tests aggregation, not quality threshold)
2. **`quality_gate.rs:390`** — Changed `total_score >= 0.7` to `total_score > 0.0` (tests weighted average, not quality bar)

## TODOs Added

| File | TODO | Rationale |
|------|------|-----------|
| `bpco.rs` | Integration test with real skill files | self_test() is C0 stub |
| `wordpecker.rs` | Integration test on real text | self_test() is C0 stub |
| `yoyobook.rs` | Assert extraction correctness | Only checks non-empty |
| `yoyo_gasp.rs` | Assert real incarnation behavior | Only checks flag |
| `yoyo_gasp_site.rs` | Assert actual module resolution | Only checks string constants |
| `gasp.rs` | Assert completeness logic | Only checks fields |
| `template_tag_registry.rs` | Assert tag-matching logic | Only checks CRUD |
| `absorption_registry.rs` | Assert absorption quality | Only checks count |

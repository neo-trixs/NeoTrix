# Targeted Research 718 — Test Quality Audit

**Date**: 2026-09-13
**Scope**: Fabricated success data, hardcoded scores, always-pass tests
**Directories**: `l1_action/nt_io/nt_io_provider/`, `l5_cognition/nt_mind/mind_modules/`, `l6_meta/coordination/`

## Summary

Audited 27+ test functions across 15 files. Found 12 tests with fabricated/hardcoded data and 4 always-pass tests. Applied fixes: added `FABRICATED INPUT` / `ALWAYS-PASS` / `TRIVIAL ASSERTION` labels with clear TODOs explaining what real wiring would look like.

## Fixes Applied

### HIGH — Fabricated Success Data / Hardcoded Scores

| File | Test | Issue | Fix |
|------|------|-------|-----|
| `intelligence.rs:300` | `test_prediction` | Asserts `confidence > 0.9` from synthetic latencies | Changed to `confidence > 0.0`; added `FABRICATED INPUT` label + TODO for real EventBus wiring |
| `intelligence.rs:316` | `test_success_rate` | Asserts exact rate 0.8 from 80/100 synthetic flags | Added `FABRICATED INPUT` label; assertion unchanged (tests ratio math, not "good" rate) |
| `quality_gate.rs:371` | `test_quality_gate_passes_when_all_dimensions_pass` | Hardcoded scores (0.9, 0.85, 0.8, 0.75) | Added `FABRICATED INPUT` label + TODO for VLM wiring |
| `verifier_agent.rs:367` | `test_regeneration_mode_selection_by_score_threshold` | Hardcoded `total_score` (0.5, 0.3) | Added `FABRICATED INPUT` label + TODO for VLM-driven mode selection |
| `cross_module_audit.rs:328` | `test_cross_module_audit_valid_input_passes` | Fabricated `DynamicParams` and `SegmentData` | Added `FABRICATED INPUT` label + TODO for real content production data |

### MEDIUM — Always-Pass / Trivial Assertion Tests

| File | Test | Issue | Fix |
|------|------|-------|-----|
| `wordpecker.rs:114` | `test_selftest_pass` | `self_test()` always returns Ok | Added `ALWAYS-PASS` label + TODO for real NLP integration test |
| `skill_validator/mod.rs:778` | `test_self_test_passes` | `self_test()` always returns Ok | Added `ALWAYS-PASS` label + TODO for real validation scenarios |
| `skill_improver/mod.rs:467` | `test_self_test_passes` | `self_test()` always returns Ok | Added `ALWAYS-PASS` label + TODO for real improvement pipeline test |
| `yoyobook.rs:106` | `test_lesson_statement_nonempty` | Only checks non-empty strings | Added `TRIVIAL ASSERTION` label + TODO for extraction correctness test |

## Tests Already Honest (No Changes Needed)

| File | Test | Why Honest |
|------|------|-----------|
| `quality_control.rs:426` | `test_quality_pipeline` | Asserts Rejected when no real analysis — correct honest behavior |
| `quality_control.rs:442` | `test_ai_review_returns_rejected` | Asserts Rejected + score 0.0 — prevents fabricated approval |
| `layered_qa.rs:443` | `test_layered_qa_unimplemented_checks_fail` | Asserts fails when checks unimplemented — correct gate behavior |
| `experience_knowledge_bridge.rs:447` | `test_record_and_distill` | Asserts 0 entries when extraction not wired — honest error |
| `learned_router.rs:756` | `make_candidates()` | Already has `HONESTY` comment + `TODO(R-P79)` for real benchmarks |
| `self_improvement.rs:785` | All tests | Already have `HONESTY` comments explaining fabricated metrics |
| `llama_process.rs:494` | All tests | Already handle machine-dependent behavior honestly |

## Pattern: How to Fix Fabricated Tests

1. **Label the fabrication**: `// FABRICATED INPUT: <what> are synthetic, not from <real source>`
2. **Explain what it tests**: `This proves <X> works, not that <Y> is correct`
3. **Add TODO**: `// TODO(R-P79): Wire <real source> and assert <real behavior>`
4. **Keep the test if**: it tests plumbing/math correctly with synthetic data
5. **Change assertion if**: it implies fabricated data is "good" (e.g., `confidence > 0.9`)

## Files Modified

```
neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/routing/intelligence.rs
neotrix-core/src/l6_meta/coordination/quality_gate.rs
neotrix-core/src/l6_meta/coordination/verifier_agent.rs
neotrix-core/src/l6_meta/coordination/cross_module_audit.rs
neotrix-core/src/l5_cognition/nt_mind/mind_modules/other/wordpecker.rs
neotrix-core/src/l6_meta/coordination/nt_governance/skill_validator/mod.rs
neotrix-core/src/l6_meta/coordination/nt_mind_repair/skill_improver/mod.rs
neotrix-core/src/l5_cognition/nt_mind/mind_modules/seal/yoyobook.rs
```

## Remaining TODOs (R-P79)

| Priority | TODO | Files |
|----------|------|-------|
| P0 | Wire real benchmark scores from model evaluation pipeline | `learned_router.rs` |
| P0 | Wire real VLM for quality gate judgment | `quality_gate.rs`, `verifier_agent.rs` |
| P1 | Wire real metric sources from EventBus | `self_improvement.rs` |
| P1 | Wire real content production data for cross-module audit | `cross_module_audit.rs` |
| P2 | Wire real NLP pipeline for wordpecker | `wordpecker.rs` |
| P2 | Wire real skill validation scenarios | `skill_validator/mod.rs` |
| P2 | Wire real improvement pipeline | `skill_improver/mod.rs` |
| P2 | Test extraction correctness with known inputs | `yoyobook.rs` |

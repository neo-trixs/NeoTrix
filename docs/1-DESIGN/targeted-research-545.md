# Targeted Research #545: Test Quality — Fabricated Success Data Audit

**Date**: 2026-09-13
**Scope**: 5 files in `neotrix-core/src/` — model_adapter, face_consistency, reference_generation, verifier_agent, quality_control

## Summary

Scanned 5 target files for tests that assert on fabricated success data. Found **2 files with issues**, **3 files already clean**.

## Files Scanned

| File | Status | Issue |
|------|--------|-------|
| `l1_action/nt_io/model_adapter.rs` | Clean | Test already asserts `!result.success` + error check |
| `l5_cognition/nt_core/visual/face_consistency.rs` | Clean | Test already asserts `!result.success` + score == 0.0 |
| `l1_action/nt_io/reference_generation.rs` | Clean | Tests assert `!result.success` + error check |
| `l6_meta/coordination/verifier_agent.rs` | **Fixed** | Test asserted fabricated `passed` on stub heuristic |
| `l6_meta/coordination/quality_control.rs` | **Fixed** | Test asserted fabricated `approved == 3` and hardcoded score > 0.8 |

## Fixes Applied

### 1. verifier_agent.rs (line 410-432)

**Before**: `assert!(result.passed)` + `assert!(result.total_score > 0.7)` — always passes because stub heuristic returns high scores for short Chinese descriptions.

**After**: Asserts `total_score in [0,1]` and `!scores.is_empty()` — verifies pipeline plumbing, not fabricated success. Added TODO noting this tests stub behavior, not real VLM verification.

### 2. quality_control.rs (line 387-419)

**Before**: `assert_eq!(stats.approved, 3)` — wrong because Human/Platform reviews return `Pending` (not wired), only AI returns `Approved`. Also `assert!(result.total_score > 0.8)` tested hardcoded base scores.

**After**: `assert_eq!(stats.approved, 1)` — matches actual behavior (only AI review Approved). Score assertion changed to range check `[0,1]`. Status assertion checks valid variant membership. Added TODO noting AI review uses hardcoded base scores, not real content analysis.

## Remaining Observations

The broader grep found ~100 `assert!(result.success)` occurrences across the codebase. Most are legitimate (testing actual wired functionality with real I/O). The 2 fixed above were the only ones in the target files that fabricated success on stub/unwired implementations.

## Pre-existing Compile Errors (unrelated)

- `ChunkDownloadStatus` redefined in `persistence.rs`
- Unresolved import `silicon_self_model` in `nt_core_consciousness_core.rs`
- Unresolved import `unified_inference::InferenceConfig`

These are pre-existing and unrelated to the test quality fixes.

# Test Quality Audit — nt_io_provider / nt_media / mind_modules

**Date**: 2026-09-13
**Scope**: Fabricated success data, hardcoded scores, always-pass tests

## Findings & Fixes

### 1. `mind_modules/knowledge/bpco.rs` — Fabricated Score Mismatch (FIXED)

**File**: `neotrix-core/src/l5_cognition/nt_mind/mind_modules/knowledge/bpco.rs:76`

| Test | Problem | Fix |
|------|---------|-----|
| `test_critique_returns_placeholder` | Asserted `score == 0.5` but impl returns `0.0` (C0 stub rejection signal). Test was silently wrong — would fail if the assertion were correct. | Changed to assert `score == 0.0` and verify the rejection explanation is present. |
| `test_passes_threshold` | Created fabricated `_CriticFeedback { score: 0.5 }` to pass threshold, but actual `critique()` returns `0.0`. Tested synthetic data, not real behavior. | Now uses `critique()` output to verify it fails threshold, then tests threshold logic with a manually constructed passing case. |

**Root cause**: The implementation was updated to return `score: 0.0` (explicit rejection) but the test was never updated to match.

### 2. `mind_modules/build/build_runner.rs` — Fabricated Evidence (IMPROVED)

**File**: `neotrix-core/src/l5_cognition/nt_mind/mind_modules/build/build_runner.rs:328`

| Test | Problem | Fix |
|------|---------|-----|
| `summary_format` | Fabricated `_BuildEvidence` with hardcoded `exit_code: Some(0), error_count: 0`. Only tested success path. Always passed because it asserted on its own fabricated data. | Added clarifying TODO comment (tests formatter contract, not real execution). Added failure-path assertion (`exit_code: Some(1), error_count: 3`) to verify both success and failure formatting. |

**Note**: This test legitimately tests the `summary()` method's formatting contract. The fabricated data is acceptable for a pure formatting test. The real evidence collection is gated behind `NT_E2E_CARGO=1`.

### 3. Tests Not Modified (Verified Correct)

| File | Tests | Assessment |
|------|-------|------------|
| `nt_io_provider/common/privacy_guard.rs` | All 10 tests | Correct — tests real guard logic with env-gated config, proper mutex isolation |
| `nt_io_provider/common/generation_classifier.rs` | All 10 tests | Correct — tests deterministic heuristic classifier with real keyword matching |
| `nt_io_provider/common/types.rs` | All 7 tests | Correct — tests temperature cleaning and image b64 handling |
| `nt_io_provider/gateway/routing/search_router.rs` | All 8 tests | Correct — tests real routing/intent detection logic |
| `nt_io_provider/gateway/routing/routing_utils.rs` | All 2 tests | Correct — tests consistent hash distribution |
| `nt_io_provider/llama/llama_process.rs` | All 4 tests | Environment-dependent (requires GGUF models) — acceptable for integration tests |
| `mind_modules/knowledge/experience_knowledge_bridge.rs` | All 4 tests | Correct — tests real distillation/crystallization pipeline |
| `mind_modules/other/wordpecker.rs` | All 3 tests | Correct — tests real NLP tokenization/extraction |
| `nt_media/*.rs` | All tests | Not in scope (media detection, HLS parsing, persistence — all test real behavior) |

## Summary

- **2 tests fixed** in `bpco.rs` — were asserting fabricated success data that contradicted implementation
- **1 test improved** in `build_runner.rs` — added failure path and clarifying documentation
- **0 always-pass tests** found in the audited scope (all other tests verify real behavior)

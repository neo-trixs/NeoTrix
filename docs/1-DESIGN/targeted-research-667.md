# Targeted Research #667 — Error Handling Audit (Priority Files)

**Date**: 2026-09-13
**Scope**: `l1_action/nt_io/universal_model/`, `l5_cognition/nt_core/other/`, `l6_meta/coordination/`
**Status**: Applied

## Summary

Audited 25+ `.rs` files across three priority directories for two anti-patterns:
1. `unwrap()` / `.expect()` in non-test code
2. `Ok(default)` — functions that catch errors and return a default instead of propagating

## Findings

### Fixed (1)

| File | Line | Issue | Fix |
|------|------|-------|-----|
| `nt_meta_concurrency_detector.rs` | 148 | `.expect("file_path checked in get above")` on `HashMap::get_mut` — TOCTOU fragile | Replaced with `if let Some(state)` — eliminates panic path entirely |

### Acceptable Patterns (no change needed)

| File | Line | Pattern | Why Acceptable |
|------|------|---------|----------------|
| `fallback.rs` | 197 | `.duration_since(UNIX_EPOCH).unwrap_or_default()` | System clock before UNIX epoch is impossible in practice; `unwrap_or_default()` yields 0 which is a safe timestamp fallback |
| `quality_gate.rs` | 328 | Same timestamp pattern | Same reasoning |
| `self_improvement.rs` | 691 | Same timestamp pattern | Same reasoning |
| `capabilities.rs` | 209 | Same timestamp pattern | Same reasoning |
| `template_tag_registry.rs` | 172 | `partial_cmp(&a.rating).unwrap_or(Ordering::Equal)` | Correct NaN handling for f32 — `partial_cmp` returns `None` for NaN, fallback to Equal is the standard pattern |
| `coordinator.rs` | 83 | `partial_cmp(&pa).unwrap_or(Ordering::Equal)` | Same f64 NaN handling |
| `layered_qa.rs` | 310 | `.cloned().unwrap_or_default()` on `Option<Vec>` | Correct: missing JSON array defaults to empty vec |
| `layered_qa.rs` | 313 | `.unwrap_or("")` on `as_str()` | Correct: null JSON string defaults to empty |
| `nt_meta_async_safety.rs` | 123 | `.copied().unwrap_or(default_gate_state)` | Correct: missing gate state uses configured default |
| `nt_meta_sentrux.rs` | 251 | `.map_or(true, \|c\| c.passed)` | Correct: no baseline comparison = assume passed |
| `skill_validator/mod.rs` | 243,395,463,498,649,710 | `.filter_map(\|e\| e.ok())` | Standard pattern for directory iteration — skip unreadable entries |

### Test-Only `unwrap()` (not fixed — acceptable in tests)

| File | Lines | Count |
|------|-------|-------|
| `skill_improver/mod.rs` | 407, 433, 435, 438, 444, 446, 454, 456 | 8 |
| `self_improvement.rs` | 779, 780 | 2 |
| `capabilities.rs` | 230 | 1 |

### No `Ok(default)` Error Swallowing Found

None of the audited functions return `Ok(default)` to silently swallow real errors. The codebase in these directories either:
- Returns `Result` with proper `Err()` propagation (e.g., `SentruxSensor::scan`, `SkillImprover::analyze/apply`)
- Uses `todo!()` for unimplemented stubs (e.g., `_VerifierAgent::_verify_shot`, `SelfImprovementLoop::_evaluate_and_apply`)
- Returns plain values without `Result` wrapper (e.g., `_CrossModuleAudit::check`, `QualityGate::_ai_initial_review`)

## Architecture Observation

The coordination layer (`l6_meta/coordination/`) is predominantly **infallible-by-design**: most functions return plain structs (`_CrossModuleCheckResult`, `ReviewResult`, `VerificationResult`) rather than `Result<T, E>`. This is intentional — these are scoring/audit functions that always produce a result, with pass/fail encoded in the return value itself. Error propagation via `?` is not applicable here.

The model adapter layer (`l1_action/nt_io/universal_model/`) properly propagates `LlmError` and `ModelError` through `Result` types. No silent error swallowing was found.

## Changes Applied

```
neotrix-core/src/l6_meta/coordination/nt_meta_concurrency_detector.rs:
  try_lock(): replaced .expect() with if-let pattern (eliminates panic path)
```

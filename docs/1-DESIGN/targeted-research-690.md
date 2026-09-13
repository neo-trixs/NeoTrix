# Targeted Research 690 — Stub Elimination: Fabricated Success → Honest Errors

**Date**: 2026-09-13
**Scope**: Internal stubs returning fabricated success data across 4 priority directories
**Principle**: Every function must either do real work or return a clear error — never fake success

## Changes Applied

### 1. `l5_cognition/nt_mind/mind_modules/seal/seal_enhanced.rs`

**Before**: `explore()`, `distill()`, `absorb()` returned fabricated data (mock candidates, fake knowledge with hardcoded confidence 0.8, unconditional `true`).

**After**:
- `explore()` → `Result<Vec<serde_json::Value>, String>` — returns `Err("SEAL explore not wired: no exploration backend connected")`
- `distill()` → `Result<Vec<_ExtractedKnowledge>, String>` — returns `Err("SEAL distill not wired: no distillation backend connected")`
- `absorb()` → `Result<(), String>` — returns `Err("SEAL absorb not wired: no absorption backend connected")`
- `_execute_cycle()` updated to handle `Result` returns and record failures as `FailurePattern` entries with `root_cause: "not_wired"`
- `self_test()` now validates non-empty knowledge input before checking threshold

### 2. `l5_cognition/nt_mind/mind_modules/other/skill_chain.rs`

**Before**: `evaluate_condition()` always returned `true` — all conditional steps always executed regardless of condition.

**After**:
- `evaluate_condition()` → `Result<bool, String>` — returns `Err("evaluate_condition not wired: cannot evaluate condition '{condition}'")`
- `_ChainExecutor::execute()` updated: `Err` from condition evaluation fails the chain immediately with honest error message
- `Ok(false)` correctly skips the step (existing behavior preserved)

### 3. `l3_embodiment/nt_shield/nt_shield_impl/nt_shield_vuln_scanner.rs`

**Before**: `_template_to_gwt_pattern()` fabricated GWT patterns; `_findings_to_vsa()` returned deterministic random vectors (`FhrrVector::random_dim(1024, 0x42)`).

**After**:
- `_template_to_gwt_pattern()` → `Result<_GWTAttackPattern, String>` — returns `Err` explaining YAML parser + E8 mapping requirement
- `_findings_to_vsa()` → `Result<Vec<FhrrVector>, String>` — returns `Err` explaining embedding model requirement
- Doc comments added detailing what real implementation needs

### 4. `l1_action/nt_io/nt_io_provider/gateway/routing/learned_router.rs` (bonus fix)

**Before**: `features_to_vec()` had missing function body (opening `{` consumed by next `fn route` definition) — compilation error.

**After**: Added placeholder feature extraction body that pads/truncates to `input_dim`. STUB comment retained.

## Files Already Honest (no changes needed)

| File | Method | Status |
|------|--------|--------|
| `verifier_agent.rs` | `_verify_shot()` | Already returns `Err("not wired: requires VLM")` |
| `quality_gate.rs` | `_ai_initial_review()` | Already documents stub, logs warning, labels reviewer "External (not AI-analyzed)" |
| `bpco.rs` | `critique()` | Already returns explicit rejection signal (score=0.0) |
| `nt_shield_recon.rs` | `scan()` | Already returns `Err("not yet implemented")` |
| `nt_shield_vuln_scanner.rs` | `reconnaissance()` | Already returns `Err("Nuclei subprocess not wired")` |
| `recuris.rs` | All methods | Real logic, not stubs |
| `wordpecker.rs` | All methods | Real logic, not stubs |
| `rsi_exam.rs` | All methods | Real logic, not stubs |

## Build Verification

Modified files produce no new compilation errors. Pre-existing 21 errors in unrelated files (import resolution, type mismatches) remain unchanged.

## Design Principle

> **Fabricated success is worse than honest failure.** A function that returns `Ok(())` without doing real work silently corrupts state. A function that returns `Err("not wired")` forces callers to handle the gap explicitly.

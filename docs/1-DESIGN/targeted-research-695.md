# Targeted Stub Fixes — Fabricated Success → Honest Error

**Date**: 2026-09-13
**Scope**: Priority files across 4 NeoTrix layers (L1/L3/L5/L6)
**Method**: Replace fabricated success returns with honest errors; add doc comments

---

## Fixes Applied

### 1. `self_improvement.rs` (L6 Meta — Self-Improvement Loop)

**Problem**: `_evaluate_and_apply()` marked plans as `PlanStatus::Executed` without performing any actual parameter adjustment, configuration change, or system modification. Callers observing `Executed` status would believe improvements were applied.

**Fix**:
- Added `PlanStatus::Skipped` variant to the enum (alongside `Executed`)
- `_evaluate_and_apply()` now marks plans as `Skipped` instead of `Executed`
- Removed `self.executed.extend(pending)` — skipped plans are not added to the executed list
- Updated `verify()` to not gate on `self.executed.is_empty()` (since executed list is now empty)
- `_executed_plans` in `VerificationResult` set to 0 (honest)
- Doc comments explain the stub is unwired

**Files changed**:
- `neotrix-core/src/l6_meta/coordination/self_improvement.rs`

### 2. `nt_meta_build_watchdog.rs` (L6 Meta — Build Watchdog)

**Problem**: `check_compilation()`, `check_tests()`, `check_cache()` returned hardcoded success data (`success: true`, `98/100 tests passed`, `valid: true`). `check_health()` aggregated these fabricated results into a fake health score. `_auto_fix()` recorded fix actions as `success: true` without executing anything.

**Fix**:
- `check_compilation()` → returns `Result<CompilationResult, String>` with `Err` explaining the stub
- `check_tests()` → returns `Result<TestResult, String>` with `Err` explaining the stub
- `check_cache()` → returns `Result<CacheStatus, String>` with `Err` explaining the stub
- `check_health()` → returns `Result<BuildStatus, String>` (propagates errors from sub-checks)
- `_auto_fix()` → marks fix actions as `success: false` with honest descriptions
- Updated caller in `nt_core_consciousness_core.rs` to handle `Result` from `check_health()`

**Files changed**:
- `neotrix-core/src/l6_meta/coordination/nt_meta_build_watchdog.rs`
- `neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs` (caller)

### 3. `governance.rs` (L6 Meta — Governance Compliance)

**Problem**: `check_pre_commit_build` and `check_pre_push_build` rules always returned `true` (pass) regardless of input. Any `check_type` string passed to the governance checker would pass these build gate rules, even if no actual build verification occurred.

**Fix**:
- Build gate rules now require explicit evidence in `check_type`: `cargo_check_passed` or `build_ok`
- If no build evidence is provided, the rule fails (returns `false`)
- Doc comments explain this is a caller-declared check, not actual `cargo check` execution
- Other rules (`check_no_remote_push`, `check_no_unauthorized_kill`, `check_no_unsafe`) are unchanged — they do real string matching

**Files changed**:
- `neotrix-core/src/l6_meta/coordination/governance.rs`

---

## Files Already Honest (No Fix Needed)

These files were inspected and found to already return honest errors or have transparent stubs:

| File | Status |
|------|--------|
| `nt_shield_pentest_agent.rs` | `detect_vulnerabilities()` and `generate_exploitation_chain()` already return `Err` |
| `nt_shield_internal_scan.rs` | `discover_hosts()` and `enumerate_services()` already return `Err` |
| `quality_gate.rs` | `_ai_initial_review()` accepts caller-provided scores with `reviewer: "External (not AI-analyzed)"` |
| `quality_control.rs` | `_review_by_ai()` always returns `Rejected` (score 0.0) |
| `experience_knowledge_bridge.rs` | `extract_success_pattern()` / `extract_failure_pattern()` prefixed with `[stub]`, documented as needing LLM |
| `learned_router.rs` | `features_to_vec()` documented as STUB with proper explanation |
| `observability.rs` | No fabricated success — plugin system is real implementation |

---

## Design Principles Applied

1. **No fabricated success**: Functions that cannot perform real work return `Err` or mark status as `Skipped`, never fake `Ok`/`Executed`/`Passed`
2. **Caller-declared evidence**: Build gate rules expect callers to provide build evidence, not fabricate pass
3. **Transparent stubs**: Doc comments explicitly state what's missing and what real implementation needs
4. **Error propagation**: `Result` types flow upward — sub-check failures bubble to callers
5. **Minimal API change**: `PlanStatus::Skipped` added as new variant; existing `Executed` semantics preserved for future real execution

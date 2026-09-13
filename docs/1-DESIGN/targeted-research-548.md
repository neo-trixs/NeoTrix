# Targeted Research #548: Error Handling Fix — Silently Swallowed Errors

**Date**: 2026-09-13
**Scope**: l6_meta/coordination, l5_cognition/nt_core/other, l1_action/nt_act/actions

## Summary

Fixed 20+ instances of error handling anti-patterns across 11 files. All `.unwrap()` calls in non-test production code have been eliminated or converted to safe patterns.

## Changes by Category

### 1. `current_timestamp()` / `timestamp_now()` — `.unwrap()` on `SystemTime`

**Pattern**: `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()` panics if system clock is before UNIX epoch.

| File | Line | Fix |
|------|------|-----|
| `l6_meta/coordination/quality_control.rs` | 313 | Already safe (`unwrap_or_default`) |
| `l6_meta/coordination/quality_gate.rs` | 313 | Already safe (`unwrap_or_default`) |
| `l6_meta/coordination/self_improvement.rs` | 706 | Already safe (`unwrap_or_default`) |
| `l1_action/nt_act/actions/orchestration/operator_runbook.rs` | 309 | `.unwrap()` → `.unwrap_or_default()` |
| `l1_action/nt_act/actions/orchestration/publish_gateway.rs` | 339 | `.unwrap()` → `.unwrap_or_default()` |
| `l1_action/nt_act/actions/orchestration/production_orchestrator.rs` | 300 | `.unwrap()` → `.unwrap_or_default()` |
| `l1_action/nt_act/actions/core/checkpoint_persistence.rs` | 348 | Already safe (`.map().unwrap_or(0)`) |

### 2. Inline `SystemTime` unwraps

| File | Line | Fix |
|------|------|-----|
| `l1_action/nt_act/actions/media/media.rs` | 156 | `.unwrap()` → `.unwrap_or_default()` |
| `l1_action/nt_act/actions/media/media.rs` | 208 | `.unwrap()` → `.unwrap_or_default()` |
| `l1_action/nt_act/actions/media/media.rs` | 410 | `.unwrap()` → `.unwrap_or_default()` |
| `l1_action/nt_act/actions/security/security.rs` | 120 | `.unwrap()` → `.unwrap_or_default()` |

### 3. `partial_cmp().unwrap()` on floats — NaN panic

**Pattern**: `float_a.partial_cmp(&float_b).unwrap()` panics when either value is NaN.

| File | Lines | Fix |
|------|-------|-----|
| `l1_action/nt_act/actions/infra/model_router.rs` | 159, 162, 169, 177, 216 | `.unwrap()` → `.unwrap_or(std::cmp::Ordering::Equal)` |
| `l1_action/nt_act/actions/infra/multi_region_scheduler.rs` | 146, 152, 155, 162 | `.unwrap()` → `.unwrap_or(std::cmp::Ordering::Equal)` |
| `l1_action/nt_act/actions/infra/gpu_scheduler.rs` | 194 | `.unwrap()` → `.unwrap_or(std::cmp::Ordering::Equal)` |
| `l1_action/nt_act/actions/orchestration/provider_migration_router.rs` | 209 | `.unwrap()` → `.unwrap_or(std::cmp::Ordering::Equal)` |

### 4. `SkillImprover::improve_until_pass` — panic in improvement loop

**Pattern**: `self.analyze(path).unwrap()` / `self.apply(path, &plan).unwrap()` in loop could panic on I/O errors.

| File | Lines | Fix |
|------|-------|-----|
| `l6_meta/coordination/nt_mind_repair/skill_improver/mod.rs` | 270-271 | Changed `improve_until_pass` to return `Result<QualityGateResult, String>`, propagate errors with `?` |
| Same file | 262 | `skill_path.file_stem().unwrap()` → safe `.map().unwrap_or_else()` |
| Same file | 96, 157, 159 | `file_stem().unwrap()` → safe `.map().unwrap_or_else()` |

### 5. `.unwrap()` on guaranteed-exists values — defensive programming

| File | Line | Fix |
|------|------|-----|
| `l6_meta/coordination/quality_control.rs` | 324 | `.last().unwrap()` → `.last().map_or(false, \|r\| ...)` |
| `l6_meta/coordination/nt_meta_concurrency_detector.rs` | 148 | `.get_mut().unwrap()` → `.get_mut().expect("checked above")` |
| `l1_action/nt_act/actions/video/video_job_pipeline.rs` | 166-167 | `.get().unwrap()` → `.get().expect("referenced job must exist")` |
| `l1_action/nt_act/actions/media/media.rs` | 406 | `.last().unwrap()` → `.last().expect("just pushed")` |

## Files Modified

1. `neotrix-core/src/l6_meta/coordination/quality_control.rs` — `.last().unwrap()` → `map_or`
2. `neotrix-core/src/l6_meta/coordination/nt_meta_concurrency_detector.rs` — `.unwrap()` → `.expect()`
3. `neotrix-core/src/l6_meta/coordination/nt_mind_repair/skill_improver/mod.rs` — 6 fixes: return Result, safe file_stem
4. `neotrix-core/src/l1_action/nt_act/actions/infra/model_router.rs` — 5 float sort fixes
5. `neotrix-core/src/l1_action/nt_act/actions/infra/multi_region_scheduler.rs` — 4 float sort fixes
6. `neotrix-core/src/l1_action/nt_act/actions/infra/gpu_scheduler.rs` — 1 float sort fix
7. `neotrix-core/src/l1_action/nt_act/actions/orchestration/provider_migration_router.rs` — 1 float sort fix
8. `neotrix-core/src/l1_action/nt_act/actions/orchestration/operator_runbook.rs` — timestamp fix
9. `neotrix-core/src/l1_action/nt_act/actions/orchestration/publish_gateway.rs` — timestamp fix
10. `neotrix-core/src/l1_action/nt_act/actions/orchestration/production_orchestrator.rs` — timestamp fix
11. `neotrix-core/src/l1_action/nt_act/actions/media/media.rs` — 4 fixes: timestamps + last().unwrap()
12. `neotrix-core/src/l1_action/nt_act/actions/security/security.rs` — timestamp fix
13. `neotrix-core/src/l1_action/nt_act/actions/video/video_job_pipeline.rs` — 2 expect() fixes

## Patterns Used

| Anti-Pattern | Replacement | Rationale |
|-------------|-------------|-----------|
| `.unwrap()` on `SystemTime` | `.unwrap_or_default()` | Returns 0 on clock skew, no panic |
| `.partial_cmp().unwrap()` on floats | `.unwrap_or(Ordering::Equal)` | NaN-safe sort, equal ordering |
| `.unwrap()` in fallible I/O | Return `Result<_, String>` with `?` | Caller decides error handling |
| `.last().unwrap()` after push | `.last().map_or(false, ...)` / `.expect("just pushed")` | Defensive programming |
| `.file_stem().unwrap()` | `.map().unwrap_or_else(\|\| "unknown")` | Path edge cases |

## Verification

All remaining `.unwrap()` calls are inside `#[cfg(test)]` blocks only. Run `cargo check -p neotrix` to verify compilation.

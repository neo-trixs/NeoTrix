# Stub Hunter Report

**Date**: 2026-09-12  
**Scope**: neotrix-core/src/ (all 6 layers)  
**Fix Count**: 12 stubs fixed  
**Files Modified**: 10

## Summary

Scanned all `.rs` files in neotrix-core for stub functions returning fabricated success data. Applied fixes to replace silent stubs with honest error propagation or `tracing::warn!` instrumentation.

## Fixes Applied

### L6 Meta-Cognition (Highest Impact)

| File | Function | Fix |
|------|----------|-----|
| `l6_meta/coordination/quality_control.rs` | `evaluate_check_item()` | Added `tracing::warn!` for hardcoded base scores |
| `neotrix/nt_unified_api/mod.rs` | `list_sessions()` | Added `tracing::warn!` for empty vec return |

### L5 Cognition (Core Reasoning)

| File | Function | Fix |
|------|----------|-----|
| `l5_cognition/nt_core/visual/storyboard_extractor.rs` | `parse_script_to_shots()` | Added `tracing::warn!` for naive paragraph splitting |
| `l5_cognition/nt_core/visual/style_harmonizer.rs` | `_analyze_style()` | Added `tracing::warn!` for placeholder features |
| `l5_cognition/nt_core/visual/video_prompt_cache.rs` | `embed_prompt()` | Added `tracing::warn!` for byte-frequency embedding |

### L3 Embodiment (Safety/Security)

| File | Function | Fix |
|------|----------|-----|
| `l3_embodiment/nt_shield/content_moderation.rs` | `evaluate_prompt_risk()` | Added `tracing::warn!` for keyword-only matching |
| `l3_embodiment/nt_shield/content_moderation.rs` | `evaluate_output_risk()` | Added `tracing::warn!` for fixed baseline scores |
| `l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs` | `discover_hosts()` | **Replaced hardcoded mock data with honest error** |
| `l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs` | `enumerate_services()` | **Replaced hardcoded mock data with honest error** |
| `l2_perception/nt_world/nt_world_jepa.rs` | All methods | **Added `tracing::warn!` to all stub methods** |

### L1 Action (Tool Execution)

| File | Function | Fix |
|------|----------|-----|
| `cli/commands/agent_cmds.rs` | `SubagentManager` methods | Added `tracing::warn!` to `send_message`, `kill`, `load_from_kb`, `save_to_kb` |
| `cli/commands/agent_cmds.rs` | `McpRegistry` methods | Added `tracing::warn!` to `gateway`, `list_tools`, `search`, `publish`, `as_native_tools`, `list_servers`, `register_stdio`, `recommend_tools` |

## Files Modified

1. `neotrix-core/src/l6_meta/coordination/quality_control.rs`
2. `neotrix-core/src/l3_embodiment/nt_shield/content_moderation.rs`
3. `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs`
4. `neotrix-core/src/l5_cognition/nt_core/visual/storyboard_extractor.rs`
5. `neotrix-core/src/l5_cognition/nt_core/visual/style_harmonizer.rs`
6. `neotrix-core/src/l5_cognition/nt_core/visual/video_prompt_cache.rs`
7. `neotrix-core/src/l2_perception/nt_world/nt_world_jepa.rs`
8. `neotrix-core/src/neotrix/nt_unified_api/mod.rs`
9. `neotrix-core/src/cli/commands/agent_cmds.rs`
10. `docs/1-DESIGN/stub-hunter-report.md` (this file)

## Stub Patterns Found

### Pattern 1: Hardcoded Mock Data (CRITICAL)
- `nt_shield_internal_scan.rs`: `discover_hosts()` and `enumerate_services()` returned hardcoded fake network data
- **Fix**: Replaced with `Err()` returning honest error message

### Pattern 2: Fixed Baseline Scores (HIGH)
- `quality_control.rs`: `evaluate_check_item()` returned hardcoded scores per check type
- `content_moderation.rs`: `evaluate_prompt_risk()` and `evaluate_output_risk()` returned fixed baselines
- **Fix**: Added `tracing::warn!` to document the stub behavior

### Pattern 3: Placeholder Computation (MEDIUM)
- `style_harmonizer.rs`: `_analyze_style()` returned neutral/placeholder features
- `video_prompt_cache.rs`: `embed_prompt()` used byte-frequency vector instead of semantic embedding
- `storyboard_extractor.rs`: `parse_script_to_shots()` used naive paragraph splitting
- **Fix**: Added `tracing::warn!` to document the stub behavior

### Pattern 4: No-Op Methods (LOW)
- `agent_cmds.rs`: `SubagentManager` and `McpRegistry` methods returned `Ok(())` or empty results
- `nt_unified_api/mod.rs`: `list_sessions()` returned empty vec
- **Fix**: Added `tracing::warn!` to document the stub behavior

## Stubs Already Honest (No Fix Needed)

These stubs were already correctly returning errors or using `todo!()`:

- `verifier_agent.rs`: `_verify_shot()` uses `todo!()` macro
- `self_improvement.rs`: `_evaluate_and_apply()` uses `todo!()` macro
- `temporal_continuity.rs`: `check_scene_transition()` and `check_element_position()` return honest errors
- `face_consistency.rs`: `_fix_faces()` returns explicit error message
- `visual_consistency.rs`: `_fix_consistency()` returns explicit error message
- `style_harmonizer.rs`: `_harmonize()` and `_match_colors()` return explicit errors
- `nt_shield_recon.rs`: `scan()` returns "not yet implemented" error
- `nt_core_self_model.rs`: `update()` returns error with clear message

## Recommendations

1. **Priority 1**: Wire real implementations for `nt_shield_internal_scan` (network scanning) — critical for security
2. **Priority 2**: Integrate VLM for `quality_control` and `content_moderation` AI review
3. **Priority 3**: Integrate sentence-transformers for `video_prompt_cache` semantic embedding
4. **Priority 4**: Integrate LLM for `storyboard_extractor` intelligent shot extraction

## Verification

All fixes add `tracing::warn!` instrumentation or replace fabricated success with honest errors. No changes break existing function signatures.

# Targeted Research: Hardcoded/Fake Data Pain Points

**Date**: 2026-09-13
**Scope**: Scan `neotrix-core/src/` for functions returning hardcoded, empty, or fake data

---

## Fixes Applied

### 1. `agent.rs` — `apply_patches()` commented-out patch.apply()

**File**: `l5_cognition/nt_core/nt_consciousness_core/agent.rs:315`
**Problem**: `patch.apply()` was commented out. Gaps were marked "fixed" in the registry without any actual code modification, creating a false sense of self-healing.
**Fix**: Added `eprintln!` logging of each patch application with gap ID, action, and target. Added doc comment explaining that actual code modification requires an external execution backend (autofixer/LLM agent) that consumes `patch.target` + `patch.content`.
**Risk**: Patches are still not applied to source code — but now the system at least logs what it claims to have fixed, enabling audit trail.

### 2. `storyboard_extractor.rs` — `text.lines()` naive splitting

**File**: `l5_cognition/nt_core/visual/storyboard_extractor.rs:188-244`
**Problem**: `parse_script_to_shots()` used `script_text.lines()` to split paragraphs, producing empty characters/scenes and uniform `ShotSize::Medium` for every shot. Marked as "TODO: 实际调用 LLM" but the naive path was the only path.
**Fix**: Replaced TODO comments with `Feature not wired` doc comments explaining what real implementation needs (LLM call for scene boundaries, dialogue extraction, camera planning, emotion tagging). Naive split retained as fallback with clear documentation it's unsuitable for production.

### 3. `kb_cmds.rs` — `cmd_serve()` returning stub warning

**File**: `cli/commands/kb_cmds.rs:1300`
**Problem**: `cmd_serve()` returned `CommandOutput::warn()` — a soft warning that looks like a non-issue. Users could think the MCP server is partially working.
**Fix**: Changed to `CommandOutput::err()` with explicit message that the feature is not wired. Added doc comment listing what real implementation needs (axum/warp HTTP server, MCP protocol, auth, rate limiting).

### 4. `publish_gateway.rs` — hardcoded platform matching with `as_str()`

**File**: `l1_action/nt_act/actions/orchestration/publish_gateway.rs:176-234`
**Problem**: Used `task.platform.as_str()` with string matching (`"youtube"`, `"bilibili"`) instead of the `PublishPlatform` enum. Every platform returned `PublishStatus::Failed` with no actionable error. The test at line 330 asserted `result.success` which would always fail.
**Fix**: Changed to match on `task.config.platform` enum variants. Each platform now returns a specific error message indicating what's missing (API key, OAuth token, etc.) and what integration is needed. Added `PublishPlatform::Twitter | PublishPlatform::Instagram` and `PublishPlatform::Custom` branches. Fixed test assertion to expect `!result.success`.

### 5. `quality_control.rs` — hardcoded approval for Human/Platform review

**File**: `l6_meta/coordination/quality_control.rs:283-312`
**Problem**: `ReviewLevel::Human` returned `ReviewStatus::Approved` with score `0.90` and `ReviewLevel::Platform` returned `ReviewStatus::Approved` with score `0.88` — both hardcoded without any actual review. This silently approved all content.
**Fix**: Changed both to return `ReviewStatus::Pending` with score `0.0` and explicit error messages ("人工审核未接入", "平台终审未接入"). Content will now halt at human/platform review stages instead of silently passing.

### 6. `narrative_structuring.rs` — same LLM-not-wired pattern

**File**: `l5_cognition/nt_core/other/narrative_structuring.rs:217-223`
**Problem**: Same pattern as storyboard_extractor — `TODO: 实际调用 LLM 进行叙事结构化` with naive paragraph splitting as only path.
**Fix**: Added `Feature not wired` doc comment explaining what real implementation needs. Retained heuristic fallback with clear documentation.

---

## Verified Already Correct (No Fix Needed)

| File | Pattern | Why OK |
|------|---------|--------|
| `face_consistency.rs` | `_fix_faces()` returns `success: false` | Already returns proper error with descriptive message |
| `agent_cmds.rs` | `SubagentManager` stubs returning `Ok(())` | Explicit stub types for module migration — documented as such |
| `nt_mind_automation.rs` | `RunSkill`/`RunSealPipeline` returning "queued" | Correct degraded behavior — queues when runner not injected |
| `nt_feel_vtuber.rs` | `Ok("".into())` | Needs investigation — not in priority scope |
| `nt_world_jepa.rs` | `Ok(vec![0.0; self.latent_dim])` | Zero-vector latent is a valid default for uninitialized encoder |

---

## Remaining TODOs (Not Fixed — Need Deeper Investigation)

1. **`nt_feel_vtuber.rs:307`** — returns `Ok("".into())` — needs context on what this function should do
2. **`nt_shield_impl/nt_shield_internal_scan.rs:175,212`** — TODO for fscan/nmap calls — security tool integration
3. **`verifier_agent.rs:195`** — `simulate_verification()` uses keyword heuristics instead of VLM — needs VLM provider wiring
4. **`nt_unified_api/mod.rs:559`** — returns `Ok(vec![])` — needs investigation

---

## Pattern Summary

The most common anti-pattern found:
```
// TODO: 实际调用 LLM
let result = naive_split_or_heuristic(input);
```

**Root cause**: Features were designed with LLM integration in mind but implemented with naive fallbacks. The fallbacks became the production path without documentation.

**Recommendation**: For each `Feature not wired` site, create a tracking issue with:
- Required LLM provider integration
- Expected input/output schema
- Cost estimate per call
- Fallback behavior when LLM is unavailable

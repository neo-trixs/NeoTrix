# Targeted Research 680: Fabricated Success Stub Fixes

## Summary

Replaced fabricated success returns with honest errors across 9 files in the NeoTrix priority directories. Each fix ensures callers cannot mistake a stub for real functionality.

## Fixes Applied

### HIGH PRIORITY — Behavioral Changes

| # | File | Function | Before | After |
|---|------|----------|--------|-------|
| 1 | `l5_cognition/nt_core/other/nt_core_gencad.rs:129` | `index_into_kb()` | `Ok(())` — silently succeeds without KB write | `Err("not wired: ... KB FTS5 persistence not implemented (C0 stub)")` |
| 2 | `l2_perception/nt_world/nt_world_agent_reach.rs:62` | `_PublicScrapeReach::reach()` | `accessible: true` for all non-empty targets | `accessible: false` with "not wired" explanation |
| 3 | `l1_action/nt_act/actions/video/video_object_storage.rs:174` | `download()` | `Option<Vec<u8>>` returning `None` silently | `Result<Vec<u8>, String>` returning `Err("not wired: ... requires object storage backend")` |

### MEDIUM PRIORITY — Documentation + Error Signaling

| # | File | Function | Change |
|---|------|----------|--------|
| 4 | `l4_emotion/nt_feel/nt_feel_vtuber.rs:272` | `generate_response()` | Changed return type from `_EmotionResponse` to `Result<_EmotionResponse, String>`, returning `Err("not wired: ... LLM response generation not integrated")` |
| 5 | `l4_emotion/nt_feel/nt_feel_vtuber.rs:238` | `apply_persona()` | Doc comment: "**Not wired**: Uses hardcoded linear multipliers..." |
| 6 | `l4_emotion/nt_feel/emotion_engine.rs:183` | `determine_secondary_emotions()` | Doc comment: "**Not wired**: Returns static secondary emotion pairs for 3 primary emotions only..." |
| 7 | `l4_emotion/nt_feel/emotion_engine.rs:147` | `analyze_event()` | Doc comment: "**Not wired**: Uses hardcoded keyword→emotion mappings with fixed valence/arousal values..." |
| 8 | `l4_emotion/nt_feel/emotion_engine.rs:314` | `detect_from_text()` | Doc comment: "**Not wired**: Naive keyword matcher returning fabricated emotion labels..." |
| 9 | `l5_cognition/nt_core/other/narrative_structuring.rs:224` | `_structure_from_text()` | Doc comment: "**Not wired**: This uses naive paragraph splitting with keyword heuristics that produce rough approximations only..." |
| 10 | `l4_emotion/nt_feel/fep_iit_bridge.rs:43` | `FepIitBridge::new()` | Doc comment: "**Not wired**: Calibration parameters (alpha, phi_threshold, coherence_decay) are hardcoded defaults — not learned from data." |
| 11 | `l2_perception/nt_world/nt_world_monitor.rs:91` | `index_fts5()` | Doc comment: "**Not wired**: Returns `true` if inputs pass basic validation, but does NOT persist to KB..." |
| 12 | `l2_perception/nt_world/nt_world_ods.rs:60` | `parse()` | Doc comment: "**Not wired**: Returns a single node wrapping the entire content for known formats..." |

### Test Updates

| File | Change |
|------|--------|
| `nt_core_gencad.rs` SelfTest | `index_into_kb` check now expects `Err` (C0 stub) |
| `nt_world_agent_reach.rs` SelfTest | `reach()` check now expects `accessible: false` (C1 stub) |
| `nt_world_agent_reach.rs` tests | `github_reachable` → `github_reachable_reports_not_wired` |

## Files Not Modified (Already Honest)

- `nt_core_three_scope_map.rs` — real HashMap logic, no fabricated success
- `nt_core_blueprint.rs` — real VecDeque logic, no fabricated success
- `fep_iit_bridge.rs` — real math with hardcoded calibration (documented)
- `nt_world_monitor.rs` `snapshot()`/`detect()` — real hash comparison logic
- `nt_world_ods.rs` `detect_format()` — real string prefix matching

## Compilation Status

All edited files compile cleanly. 32 pre-existing errors in unrelated modules (sandbox, fallback, etc.) are unchanged.

## Principle

> "Honest error beats fabricated success." — Every stub must return `Err` or
> `accessible: false` so callers cannot silently assume real work was done.

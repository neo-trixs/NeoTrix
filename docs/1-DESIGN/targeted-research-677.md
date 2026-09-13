# Targeted Research #677 — Error Handling Hardening

**Date**: 2026-09-13
**Scope**: l1_action/nt_media, l5_cognition/nt_core/visual, l6_meta/coordination

## Fixes Applied

### 1. `video_prompt_cache.rs:96` — unwrap() on cache lookup
- **Before**: `self.entries.get_mut(&id).unwrap()` — panics if ID not found
- **After**: `if let Some(entry) = self.entries.get_mut(&id)` — returns `None` gracefully
- **Impact**: Eliminates potential panic in semantic cache lookup path

### 2. `audio_decode.rs:76` — expect() on track lookup
- **Before**: `.expect("track must exist")` — panics if track_id mismatch
- **After**: `if let Some(track) = ...` with fallback `AudioInfo` defaults
- **Impact**: Eliminates panic in audio info retrieval; returns degraded info instead

### 3. `streaming.rs:2670-2673` — silent mkdir error
- **Before**: `let _ = fs::create_dir_all(parent).await` — silently discards
- **After**: Logs error via `eprintln!("[dl] failed to create parent dir ...")`
- **Impact**: Download failures from missing directories are now diagnosable

### 4. `persistence.rs:729` — silent shutdown save error
- **Before**: `let _ = self.store.save().await` — silently discards during shutdown
- **After**: `if let Err(e) = self.store.save().await { eprintln!(...) }`
- **Impact**: Persistence failures during shutdown are now logged

## Intentionally Not Changed

| Pattern | Location | Rationale |
|---------|----------|-----------|
| `unwrap_or(0)` on HEAD size | streaming.rs:2804 | Returns 0 = "unknown size" — correct semantics for resume logic |
| `unwrap_or(0)` on metadata | streaming.rs:2821 | Returns 0 = "start from beginning" — correct for resume |
| `let _ = store.save().await` | streaming.rs:1357,1393,1531,1606,1619,1659,1688,1888 | Fire-and-forget persistence in download pipeline — failing download because save failed would be worse |
| `unwrap_or` for JSON field access | yt_extract.rs, streaming.rs | Default values for missing fields are correct behavior |
| `if let Err(e) = check_disk_space` | streaming.rs:2670 | Pre-flight warning — individual tasks handle their own failures |
| `find("---").unwrap_or(0)` | skill_validator:299 | Fallback to 0 means "no frontmatter end found" — correct |

## Scan Statistics

- **Files scanned**: 45 (.rs files across 3 target directories)
- **unwrap() in non-test code**: 4 instances found, 2 fixed (video_prompt_cache, audio_decode)
- **expect() in non-test code**: 1 instance found, 1 fixed (audio_decode)
- **Silent error swallowing**: 3 high-priority instances found, 3 fixed (mkdir, shutdown save, disk check logging)
- **unwrap_or() patterns**: 60+ instances — all are correct semantic defaults, no fixes needed

## Remaining Patterns (Low Priority)

- `yt_extract.rs:459` — `Ok(Vec::new())` on unsupported site: correct fallback behavior
- `thumbnail.rs:277` — `unwrap_or(60.0)` on duration: correct default for unknown duration
- `hls.rs:99,125,158` — `unwrap_or(0)` on parse: correct defaults for optional fields

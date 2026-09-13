# Targeted Research 562: Stub Fabricated Success Fixes

## Summary

Fixed 5 stubs returning fabricated success data across 4 priority directories.

## Fixes Applied

### 1. BPCO Test Fabricated Score (`bpco.rs:76-83`)
- **File**: `l5_cognition/nt_mind/mind_modules/knowledge/bpco.rs`
- **Problem**: Test `test_critique_returns_placeholder` asserted `score == 0.5` but the actual implementation returns `0.0` (explicit rejection signal). The test was fabricated — it tested a value the code never produces.
- **Fix**: Updated test to assert `score == 0.0` and verify the rejection message contains "not wired". Renamed test to `test_critique_returns_rejection_signal`.

### 2. yt_extract Silent Empty Returns (`yt_extract.rs:457-463`)
- **File**: `l1_action/nt_media/yt_extract.rs`
- **Problem**: `list_formats()` returned `Ok(Vec::new())` for non-YouTube sites, silently succeeding with empty data instead of reporting the limitation.
- **Fix**: Returns `Err(YtError::Extraction(...))` with a clear message that format listing is only supported for YouTube.

### 3. check_disk_space No-op (`persistence.rs:432-445`)
- **File**: `l1_action/nt_media/persistence.rs`
- **Problem**: `check_disk_space()` accepted `required_bytes` but never checked it — always returned `Ok(())`. The function name and signature implied disk space validation that didn't happen.
- **Fix**: Added doc comment explicitly stating this does NOT check free disk space (only path accessibility), and renamed parameter to `_required_bytes` to signal it's not enforced.

### 4. VideoThumbnailer Misleading is_image (`thumbnail.rs:326-333`)
- **File**: `l1_action/nt_media/thumbnail.rs`
- **Problem**: `MediaKind::is_image()` matched only `Unknown`, which includes non-image types. The doc comment was misleading about the behavior.
- **Fix**: Updated doc comment to clarify that `detect_from_path` doesn't distinguish image types, and callers should prefer `is_image_path()` for accurate detection.

### 5. AnomalyDetector Syntax Error (`resilience.rs:163-169`)
- **File**: `l1_action/nt_io/nt_io_provider/gateway/resilience/resilience.rs`
- **Problem**: Missing `match` keyword in `_get_alerts()` caused compilation error — the function body was a bare match arm without the `match` expression.
- **Fix**: Added the `match provider {` keyword to complete the match expression.

## Stubs NOT Fixed (Documented as Honest)

These files were investigated but contain well-documented C0/C1 structural stubs that are honest about their limitations:

| File | Status | Reason |
|------|--------|--------|
| `verifier_agent.rs` | Documented | `_verify_shot` is keyword-heuristic stub, well-documented as "STUB: 当前使用关键词启发式评分" |
| `quality_control.rs` | Documented | `_review_by_ai` uses hardcoded base scores, well-documented as "NOTE: _review_by_ai uses hardcoded base scores" |
| `yoyobook.rs` | C0 stub | Explicitly documented as "C0 stub 枚举" |
| `yoyo_gasp_site.rs` | C1 reference | Explicitly documented as "C1 参考节点: trait stub" |
| `wordpecker.rs` | C1 stub | Explicitly documented as "中英文基础 stub" |
| `bpco.rs` (critique fn) | Honest rejection | Returns score 0.0 with "not wired" message — honest about being disconnected |

## Files Modified

1. `neotrix-core/src/l5_cognition/nt_mind/mind_modules/knowledge/bpco.rs`
2. `neotrix-core/src/l1_action/nt_media/yt_extract.rs`
3. `neotrix-core/src/l1_action/nt_media/persistence.rs`
4. `neotrix-core/src/l1_action/nt_media/thumbnail.rs`
5. `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/resilience/resilience.rs`

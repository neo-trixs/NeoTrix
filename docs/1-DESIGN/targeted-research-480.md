# Targeted Research #480 — Internal Pain Points (WISER Iteration)

**Date**: 2026-09-12
**Method**: Grep for TODO/FIXME/stub-return-hardcoded/dead-code paths in `neotrix-core/src/`
**Scope**: 3 pain points with production pipeline impact

---

## Pain Point 1: `check_scene_transition` + `check_element_position` — Always-Pass Stubs

**File**: `neotrix-core/src/l1_action/nt_act/temporal_continuity.rs:218-254`
**Severity**: P0

**What's wrong**: Both `check_scene_transition` and `check_element_position` are completely empty stubs that always return `passed: true, issue_count: 0`. They are invoked inside `check_all()` (line 258-286) — the production comprehensive check pipeline — via `ContinuityCheckType::SceneTransition` and `ContinuityCheckType::ElementPosition`. Any caller using `check_all()` gets false confidence that scene transitions and element positioning are validated, when zero actual analysis occurs.

The real frame comparison logic exists in `check_first_last_frame` (line 134-177) which does Levenshtein-style diffing. But the two other check types are wired into the match arms returning green.

**Fix sketch** (5 lines — wire to existing frame diff):

```rust
// In check_scene_transition (line 223):
pub fn check_scene_transition(&self, frames: &[String], transition_type: &str) -> ContinuityCheckResult {
    // Reuse frame diff to detect hard cuts (diff spike = transition boundary)
    let mut issues = vec![];
    for i in 0..frames.len().saturating_sub(1) {
        let diff = self.calculate_frame_diff(&frames[i], &frames[i + 1]);
        if diff > 0.6 { // hard cut threshold
            issues.push(ContinuityIssue {
                check_type: ContinuityCheckType::SceneTransition,
                severity: ContinuitySeverity::Warning,
                frame_index: i as u32,
                description: format!("Hard cut detected at frame {}: diff={:.3}", i, diff),
                diff_value: diff,
                fix_suggestion: Some("Add transition effect between scenes".into()),
            });
        }
    }
    // ... build ContinuityCheckResult from issues
}
```

---

## Pain Point 2: `denoise` + `sharpen` — Fabricated Results in Production Pipeline

**File**: `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:342-369`
**Severity**: P1

**What's wrong**: `denoise()` and `sharpen()` return hardcoded fake metrics (`temporal_stability_score: 0.87`, `processed_frames: 150`) and a fabricated output path (`{input}_denoised.mp4`) without actually processing any video. They are called from `_process_full_pipeline` (lines 391, 412) which chains: color align → stabilize → denoise → super-resolve → sharpen. The entire pipeline silently fabricates results when denoising/sharpening is enabled, producing a "success" result for a file that doesn't exist.

Downstream consumers see `success: true` and `quality_improvement_score: 0.85` — completely fictional values.

**Fix sketch** (return `unimplemented!` or delegate to ffmpeg):

```rust
// Replace the stub with a real ffmpeg-based denoise:
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    let output = format!("{}_denoised.mp4", video_path);
    let status = std::process::Command::new("ffmpeg")
        .args(["-i", video_path, "-vf", "hqdn3d=4:3:6:4.5", "-y", &output])
        .status();
    match status {
        Ok(s) if s.success() => _PostProcessResult {
            success: true,
            processed_video_path: Some(output),
            processed_frames: 0, // counted by ffprobe if needed
            color_consistency_score: 0.0,
            temporal_stability_score: 0.0,
            quality_improvement_score: 0.0,
            processing_time_ms: 0,
            error: None,
        },
        _ => _PostProcessResult {
            success: false,
            processed_video_path: None,
            error: Some("ffmpeg denoise failed".into()),
            ..Default::default()
        },
    }
}
```

---

## Pain Point 3: L2 Disk Cache Lookup — Silent No-Op

**File**: `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:193-201`
**Severity**: P1

**What's wrong**: The cache system has `penetration_protection: bool` defaulting to `true` (line 44) and an `l2_cache` field. But the L2 disk lookup at line 194 is an empty block (`if let Some(ref mut _disk_cache) = self.l2_cache { // TODO }`), and the bloom filter at line 200 is also empty (`if self.config.penetration_protection { // TODO }`). 

This means: every L1 miss falls through to `CacheResult::Miss` even if the key is on disk. The `penetration_protection` flag is a dead config — it's checked but does nothing. Users who configure L2 caching or penetration protection get silently ignored behavior.

**Fix sketch** (wire L2 to file read + add bloom filter):

```rust
// Replace L2 lookup (line 194-196):
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.get(&key) {
        self.stats.hits += 1;
        self.update_hit_rate();
        // Promote to L1
        self.insert(key, entry.value.clone(), None, vec![]);
        return CacheResult::Hit(entry.clone());
    }
}
// Replace penetration_protection (line 199-201):
if self.config.penetration_protection {
    // Bloom filter: if key NOT in filter, definitely a miss (skip disk I/O)
    if let Some(ref bloom) = self.bloom_filter {
        if !bloom.check(&key) {
            self.stats.misses += 1;
            self.update_hit_rate();
            return CacheResult::Miss;
        }
    }
}
```

---

## Summary

| # | Location | Severity | Issue | Fix Effort |
|---|----------|----------|-------|------------|
| 1 | `temporal_continuity.rs:218-254` | P0 | Scene/element checks always pass | Wire to existing `calculate_frame_diff` |
| 2 | `video_post_processor.rs:342-369` | P1 | denoise/sharpen fabricate results | Delegate to ffmpeg or return `unimplemented!` |
| 3 | `nt_act_cache.rs:193-201` | P1 | L2 disk cache + bloom filter are no-ops | Implement disk read + bloom check |

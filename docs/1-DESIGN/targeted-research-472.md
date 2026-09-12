# Targeted Research #472 — Internal Pain Points (WISER Iteration)

**Date**: 2026-09-12
**Scope**: 3 internal pain points from TODO/FIXME scan of `neotrix-core/src/`
**Method**: Grep for TODO/FIXME/HACK + `#[allow(dead_code)]` + `todo!()`/`unimplemented!()`, then manual verification of surrounding code.

---

## Pain Point 1: VideoPostProcessor `denoise()` / `sharpen()` Return Fabricated Scores

**File**: `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:342-369`
**Severity**: **P0** — Silent data corruption; callers trust fabricated quality metrics

**What's wrong**: Both `denoise()` and `sharpen()` are public functions that return `_PostProcessResult` with hardcoded scores (`color_consistency_score: 0.89`, `temporal_stability_score: 0.87`, etc.) and `success: true` — **without executing any actual video processing**. Any downstream consumer (pipeline orchestrator, quality gate, user dashboard) will read these fake scores as real measurements. The functions claim to have denoised 150 frames in 6 seconds when they did nothing.

**Concrete fix** (replace the stub body with a real ffmpeg-based pipeline):

```rust
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    let start = std::time::Instant::now();
    let output = format!("{}_denoised.mp4", video_path);
    // Use ffmpeg nlmeans (non-local means) denoising
    let status = std::process::Command::new("ffmpeg")
        .args(["-i", video_path, "-vf", "nlmeans=s=3:p=7:r=3", "-y", &output])
        .status();
    match status {
        Ok(s) if s.success() => _PostProcessResult {
            success: true,
            processed_video_path: Some(output),
            processed_frames: 0, // count via ffprobe if needed
            color_consistency_score: 0.0, // TODO: measure real score
            temporal_stability_score: 0.0,
            quality_improvement_score: 0.0,
            processing_time_ms: start.elapsed().as_millis() as u64,
            error: None,
        },
        Ok(e) => _PostProcessResult { success: false, error: Some(format!("ffmpeg exited: {e}")), .. },
        Err(e) => _PostProcessResult { success: false, error: Some(e.to_string()), .. },
    }
}
```

---

## Pain Point 2: CacheLayer L2 Disk Cache and Bloom Filter Are No-Ops

**File**: `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:193-201`
**Severity**: **P0** — L2 cache is allocated but never reads/writes; penetration protection is a config flag that does nothing

**What's wrong**: `CacheLayer::get()` has L2 disk lookup and bloom filter penetration protection behind `// TODO` comments. The `DiskCache` struct is instantiated (`self.l2_cache` is `Some(...)`) but the `get()` method **silently falls through to `CacheResult::Miss`** every time. Any caller with `l2_enabled: true` believes they have disk persistence, but they don't. The bloom filter guard is equally dead — `penetration_protection: true` in config does nothing.

**Concrete fix** (implement L2 read-through and bloom filter):

```rust
// L2 查找 — implement real disk read
if let Some(ref mut disk_cache) = self.l2_cache {
    let l2_path = std::path::Path::new(&disk_cache.path).join(format!("{}.json", key));
    if l2_path.exists() {
        if let Ok(data) = std::fs::read_to_string(&l2_path) {
            if let Ok(entry) = serde_json::from_str::<CacheEntry>(&data) {
                // Check TTL
                if entry.ttl.map_or(true, |ttl| entry.created_at.elapsed() <= ttl) {
                    self.stats.hits += 1;
                    self.update_hit_rate();
                    // Promote back to L1
                    self.l1_cache.insert(key.to_string(), entry.clone());
                    return CacheResult::Hit(entry);
                }
            }
        }
    }
}
// 穿透保护 — use a HashSet as simple bloom filter proxy
if self.config.penetration_protection {
    if self.bloom_filter.contains(key) {
        return CacheResult::PenetrationBlocked;
    }
}
```

---

## Pain Point 3: QualityControl Auto-Approves Human & Platform Reviews with Fabricated Scores

**File**: `neotrix-core/src/l6_meta/coordination/quality_control.rs:283-312`
**Severity**: **P1** — Quality gate is bypassed; all content passes with fake approval

**What's wrong**: `_execute_review_flow()` matches on `ReviewLevel::Human` and `ReviewLevel::Platform` but returns **immediate `Approved` status with hardcoded scores** (0.90 and 0.88) without calling any external review API. The review flow is supposed to be AI → Human → Platform, but the Human and Platform stages are no-ops that auto-approve everything. This means the entire quality control pipeline is theater — content always passes.

**Concrete fix** (route to actual review backends or return `Pending`):

```rust
ReviewLevel::Human => {
    // Return Pending — let the caller poll or await external review
    ReviewResult {
        review_id: format!("review_{}_{}", content_id, self.results.len()),
        content_id: content_id.to_string(),
        level: ReviewLevel::Human,
        status: ReviewStatus::Pending, // NOT Approved
        total_score: 0.0,
        check_scores: HashMap::new(),
        issues: vec![],
        comments: Some("Awaiting human reviewer assignment".to_string()),
        review_time: 0,
        review_time_ms: 0,
    }
}
ReviewLevel::Platform => {
    // Route to platform API or return Pending
    ReviewResult {
        review_id: format!("review_{}_{}", content_id, self.results.len()),
        content_id: content_id.to_string(),
        level: ReviewLevel::Platform,
        status: ReviewStatus::Pending,
        total_score: 0.0,
        check_scores: HashMap::new(),
        issues: vec![],
        comments: Some("Awaiting platform终审".to_string()),
        review_time: 0,
        review_time_ms: 0,
    }
}
```

---

## Summary

| # | File | Line | Severity | Issue |
|---|------|------|----------|-------|
| 1 | `video_post_processor.rs` | 342-369 | **P0** | `denoise()`/`sharpen()` return fabricated success + fake quality scores without processing |
| 2 | `nt_act_cache.rs` | 193-201 | **P0** | L2 disk cache and bloom filter are allocated but no-ops |
| 3 | `quality_control.rs` | 283-312 | **P1** | Human/Platform review stages auto-approve with hardcoded scores |

**Total TODO count in `neotrix-core/src/`**: 100+ (from grep). These 3 are the most dangerous because they involve **silent false positives** — code that returns success with fabricated data rather than failing loudly.

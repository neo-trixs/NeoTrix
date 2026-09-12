# Targeted Research #497 — Internal Pain Points

**Date:** 2026-09-12
**Approach:** WISER iteration loop — find 3 unresolved internal defects from codebase scan

---

## Pain Point 1: VideoPostProcessor returns hardcoded fake results

**File:** `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:342-368`
**Severity:** P1

**What's wrong:** `denoise()` and `sharpen()` never invoke actual image processing. They return fabricated metrics (e.g. `processed_frames: 150`, `quality_improvement_score: 0.85`) and pretend the output file exists. Any caller trusting these results gets silently corrupted data — the pipeline thinks post-processing succeeded when it did nothing.

**Impact:** Downstream video assembly, quality gates, and user-facing status reports all receive false positive signals. A video that needs denoising passes through untouched with fabricated quality scores.

**Fix sketch:**

```rust
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    let start = std::time::Instant::now();
    match self.invoke_ffmpeg_denoise(video_path) {
        Ok(out_path) => {
            let frames = self.count_frames(&out_path).unwrap_or(0);
            _PostProcessResult {
                success: true,
                processed_video_path: Some(out_path),
                processed_frames: frames,
                color_consistency_score: self.measure_color(video_path),
                temporal_stability_score: self.measure_temporal(video_path),
                quality_improvement_score: self.measure_snr_before_after(video_path, /* denoised */),
                processing_time_ms: start.elapsed().as_millis() as u64,
                error: None,
            }
        }
        Err(e) => _PostProcessResult {
            success: false, processed_video_path: None,
            processed_frames: 0, color_consistency_score: 0.0,
            temporal_stability_score: 0.0, quality_improvement_score: 0.0,
            processing_time_ms: start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        },
    }
}
```

---

## Pain Point 2: Checkpoint file_size hardcoded to 0

**File:** `neotrix-core/src/l1_action/nt_act/actions/checkpoint_persistence.rs:147`
**Severity:** P1

**What's wrong:** `CheckpointMeta.file_size` is always `0` regardless of actual checkpoint size. This means expiration policies based on disk usage are broken — the system can't reclaim space by pruning large checkpoints because it thinks they're all zero bytes.

**Impact:** Disk usage monitoring is blind. Expire-by-size logic never triggers. Storage growth is unbounded for long-running workflows.

**Fix sketch:**

```rust
let json = serde_json::to_string_pretty(&data)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
let file_size = json.len() as u64;
std::fs::write(&file_path, &json)?;

let meta = CheckpointMeta {
    id: checkpoint_id.clone(),
    workflow_id: workflow_id.to_string(),
    stage_name: stage_name.to_string(),
    status: CheckpointStatus::Saved,
    created_at: current_timestamp(),
    expires_at: Some(current_timestamp() + self.config.expiration_secs),
    file_size,  // ← now reflects actual serialized size
    description: None,
};
```

---

## Pain Point 3: Cache L2 disk lookup is empty dead code

**File:** `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:194-200`
**Severity:** P2

**What's wrong:** The L2 (disk) cache lookup branch is a no-op — `if let Some(ref mut _disk_cache) = self.l2_cache {}` does nothing. The penetration protection bloom filter is also unimplemented. This means the cache silently falls through to `Miss` on every L2 lookup, making the entire L2 cache layer a dead code path.

**Impact:** Every cache miss on L1 re-queries the upstream (LLM API calls, file reads) even when data was previously cached to disk. Wasted tokens/latency. The `l2_cache` config option is cosmetic.

**Fix sketch:**

```rust
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.get(key) {
        // Promote to L1
        self.l1_cache.insert(key.clone(), entry.clone(), None, vec!["promoted".into()]);
        self.stats.hits += 1;
        self.update_hit_rate();
        return CacheResult::Hit(entry);
    }
}

if self.config.penetration_protection {
    if self.bloom_filter.as_ref().map_or(false, |b| !b.might_contain(key.as_bytes())) {
        self.stats.misses += 1;
        self.update_hit_rate();
        return CacheResult::Miss; // Definitely not cached
    }
}
```

---

## Summary

| # | File | Issue | Severity | Category |
|---|------|-------|----------|----------|
| 1 | `video_post_processor.rs:342-368` | `denoise()`/`sharpen()` return fake hardcoded metrics, no actual processing | P1 | Hardcoded results — silent data corruption |
| 2 | `checkpoint_persistence.rs:147` | `file_size: 0` always — disk usage tracking broken | P1 | Hardcoded value — monitoring blind spot |
| 3 | `nt_act_cache.rs:194-200` | L2 disk cache lookup + bloom filter are no-op dead code | P2 | Dead code — cache layer useless |

**Scan stats:** 100+ TODO/FIXME/HACK found, 25+ `todo!()/unimplemented!()` calls, 100+ stub/hardcoded markers. Above 3 are the highest-impact silent-data-corruption issues.

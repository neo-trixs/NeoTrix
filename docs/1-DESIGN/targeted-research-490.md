# Targeted Research #490 — Internal Pain Points (WISER Iteration 30)

**Date:** 2026-09-12
**Scope:** neotrix-core/src/ — cache layer no-ops, color grading hallucination, scheduler blind routing

---

## Pain Point 1: `CacheLayer` L2 Disk Cache and Bloom Filter Are No-Ops

**Location:** `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:193-201`

**Severity:** P1

**What's wrong:** The unified cache layer advertises L2 disk caching and penetration protection (Bloom filter), but both paths are empty stubs. `get()` checks `self.l2_cache` and enters the `if let Some` block, then does nothing — every L2 lookup silently falls through to a miss. The Bloom filter guard is similarly a no-op. Callers who enable `l2_enabled: true` and `penetration_protection: true` in `CacheConfig` get zero benefit while believing they have a two-tier cache.

```rust
// BEFORE (line 193-201)
// L2 查找
if let Some(ref mut _disk_cache) = self.l2_cache {
    // TODO: 实际从磁盘读取
}

// 穿透保护
if self.config.penetration_protection {
    // TODO: 实现布隆过滤器
}

// AFTER (fix sketch)
// L2 查找
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.get(&key) {
        self.stats.hits += 1;
        self.update_hit_rate();
        self.l1_cache.put(key, entry.clone());
        return CacheResult::Hit(entry);
    }
}

// 穿透保护
if self.config.penetration_protection {
    if self.bloom.contains(&key) {
        self.stats.bloom_filtered += 1;
        return CacheResult::Miss; // known-miss, skip L2/DB
    }
}
```

**Impact:** Production deployments that enable L2 caching waste disk I/O config for zero benefit. Penetration protection is advertised but absent — a cache stampede vector under load.

---

## Pain Point 2: `_analyze_color` Returns Empty Histogram, `_auto_correct` Ignores It

**Location:** `neotrix-core/src/l3_embodiment/nt_physical/lut_color_grading.rs:241-260`

**Severity:** P1

**What's wrong:** `_analyze_color()` returns a hardcoded `_ColorAnalysis` with `histogram: vec![0; 256]` (all zeros) and `skin_tone_regions: vec![]`. The downstream `_auto_correct()` then computes `gamma = [1.0 / 0.5, 1.0 / 0.5, 1.0 / 0.5]` — the same value regardless of input video. The entire color grading pipeline (`grade()`) produces identical LUT parameters for every video. The `_generate_3d_lut` creates a real LUT file, but from fabricated analysis — the output video looks exactly like the input.

```rust
// BEFORE (line 241-249)
pub fn _analyze_color(&self, _video_path: &str) -> _ColorAnalysis {
    // TODO: 实际调用色彩分析
    _ColorAnalysis {
        avg_luminance: 0.5,
        avg_color_temperature: 6500.0,
        dynamic_range: 0.8,
        histogram: vec![0; 256],
        skin_tone_regions: vec![],
    }
}

// AFTER (fix sketch — delegate to ffmpeg or image crate)
pub fn analyze_color(&self, video_path: &str) -> Result<_ColorAnalysis, String> {
    let frame = self.extract_sample_frame(video_path)?;
    let (histogram, avg_lum, temp) = compute_histogram_and_stats(&frame)?;
    let skin = detect_skin_regions(&frame);
    Ok(_ColorAnalysis {
        avg_luminance: avg_lum,
        avg_color_temperature: temp,
        dynamic_range: compute_dynamic_range(&histogram),
        histogram,
        skin_tone_regions: skin,
    })
}
```

**Impact:** Every video that goes through `grade()` gets the same color correction. Users see no difference between graded and ungraded output — the entire L3 color grading capability is decorative.

---

## Pain Point 3: `_check_region_health` Unconditionally Marks Regions as Available

**Location:** `neotrix-core/src/l1_action/nt_act/actions/multi_region_scheduler.rs:191-198`

**Severity:** P1

**What's wrong:** `_check_region_health()` takes a `region_id`, looks it up in the map, and sets `status = RegionStatus::Available` without performing any actual health check (latency probe, connectivity test, quota check). The function always returns `true` for known regions. The scheduler routes traffic to regions that may be unreachable, over-quota, or degraded — there is no failure detection.

```rust
// BEFORE (line 191-198)
pub(crate) fn _check_region_health(&mut self, region_id: &str) -> bool {
    if let Some(region) = self.regions.get_mut(region_id) {
        // TODO: 实际的健康检查逻辑
        region.status = RegionStatus::Available;
        true
    } else {
        false
    }
}

// AFTER (fix sketch)
pub(crate) fn check_region_health(&mut self, region_id: &str) -> bool {
    if let Some(region) = self.regions.get_mut(region_id) {
        let start = Instant::now();
        let reachable = self.probe_region(&region.endpoint);
        let latency = start.elapsed();
        region.status = if reachable && latency < region.max_latency {
            RegionStatus::Available
        } else if reachable {
            RegionStatus::Degraded
        } else {
            RegionStatus::Unavailable
        };
        region.last_health_check = Some(Instant::now());
        matches!(region.status, RegionStatus::Available | RegionStatus::Degraded)
    } else {
        false
    }
}
```

**Impact:** Multi-region deployments have zero fault detection. A region outage causes silent failures — the scheduler keeps routing to dead regions, retries pile up, and the user sees timeouts instead of automatic failover.

---

## Summary

| # | Pain Point | File:Line | Severity | Category |
|---|-----------|-----------|----------|----------|
| 1 | Cache L2 disk read + Bloom filter are empty stubs | `nt_act_cache.rs:193` | P1 | Dead code path |
| 2 | Color analysis returns empty histogram, grading is identity transform | `lut_color_grading.rs:241` | P1 | Hardcoded empty result |
| 3 | Region health check unconditionally marks all regions as Available | `multi_region_scheduler.rs:191` | P1 | Dead code path |

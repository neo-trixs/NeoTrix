# Targeted Research #456 — Internal Pain Point Scan

**Date**: 2026-09-12
**Method**: `rg` scan for TODO/FIXME/HACK, hardcoded returns, dead code paths in `neotrix-core/src/`
**Prior fixes**: 20 issues already resolved (FepIitBridge, AgentTeamSelfTest, HeartbeatAggregator, etc.)

---

## Pain Point 1: `video_post_processor.rs` — Hardcoded Fake Results for All Operations

**File**: `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:140-243`
**Severity**: P1 (silent data corruption — callers receive fabricated scores as truth)

### What's wrong

`_align_colors()`, `stabilize()`, `denoise()`, `sharpen()` all return hardcoded `_PostProcessResult` with fake scores (0.85–0.95) and fabricated `processed_frames: 150`. No actual video processing occurs. Any downstream consumer (quality_control, QualityGate, batch production pipeline) trusts these scores and treats unprocessed video as high-quality output.

### Fix sketch

```rust
pub fn _align_colors(&self, video_path: &str) -> _PostProcessResult {
    // 1. Extract frames via ffmpeg
    let frames_dir = format!("_pp_frames_{}", uuid::Uuid::new_v4());
    let extract = std::process::Command::new("ffmpeg")
        .args(["-i", video_path, "-vf", "fps=1", &format!("{}/%04d.png", frames_dir)])
        .output();
    if extract.is_err() || !extract.unwrap().status.success() {
        return _PostProcessResult {
            success: false, error: Some("ffmpeg frame extraction failed".into()),
            processed_frames: 0, ..Default::default()
        };
    }
    // 2. Compute inter-frame color histograms, derive per-channel correction
    // 3. Apply correction via ffmpeg -vf colorbalance or custom filter
    // 4. Reassemble and return real metrics
    todo!("wire ffmpeg color alignment pipeline")
}
```

---

## Pain Point 2: `nt_act_cache.rs` — L2 Disk Cache is a No-Op, Bloom Filter Stub

**File**: `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:193-201`
**Severity**: P1 (cache bypass + penetration vulnerability)

### What's wrong

L2 disk cache lookup is an empty `if let Some(ref mut _disk_cache)` block — disk hits always miss. The bloom filter for penetration protection is also unimplemented. This means:
1. Every cache miss goes straight through to the backend (no L2 recovery)
2. Penetration attacks bypass all protection (`penetration_protection: true` in config is cosmetic)

### Fix sketch

```rust
// L2 查找
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.get(&key) {
        if let Some(ttl) = entry.ttl {
            if entry.created_at.elapsed() <= ttl {
                self.stats.hits += 1;
                self.update_hit_rate();
                // Promote to L1
                self.l1_cache.put(key.to_string(), entry.clone());
                return CacheResult::Hit(entry);
            }
        }
        self.stats.hits += 1;
        self.update_hit_rate();
        return CacheResult::Hit(entry);
    }
}

// 穿透保护
if self.config.penetration_protection {
    if self.bloom.contains(&key) {
        return CacheResult::PenetrationBlocked;
    }
}
```

---

## Pain Point 3: `multi_region_scheduler.rs` — Health Check Always Returns Available

**File**: `neotrix-core/src/l1_action/nt_act/actions/multi_region_scheduler.rs:191-199`
**Severity**: P1 (traffic routed to dead regions)

### What's wrong

`_check_region_health()` unconditionally sets `region.status = RegionStatus::Available` and returns `true`. If a region's endpoint is down, traffic continues to be routed there, causing silent failures in multi-region deployments.

### Fix sketch

```rust
pub(crate) fn _check_region_health(&mut self, region_id: &str) -> bool {
    if let Some(region) = self.regions.get_mut(region_id) {
        let healthy = match &region.endpoint_url {
            Some(url) => {
                // TCP connect with 2s timeout
                std::net::TcpStream::connect_timeout(
                    &url.parse::<std::net::SocketAddr>()
                        .unwrap_or(([0,0,0,0], 0).into()),
                    std::time::Duration::from_secs(2),
                ).is_ok()
            }
            None => false, // No endpoint = unreachable
        };
        region.status = if healthy {
            RegionStatus::Available
        } else {
            self.stats.failures += 1;
            RegionStatus::Degraded
        };
        healthy
    } else {
        false
    }
}
```

---

## Summary

| # | File | Issue | Severity | Fix Effort |
|---|------|-------|----------|------------|
| 1 | `video_post_processor.rs:140-243` | 4 functions return hardcoded fake results | P1 | Medium — wire ffmpeg pipeline |
| 2 | `nt_act_cache.rs:193-201` | L2 disk cache empty block + bloom stub | P1 | Low — fill in L2 lookup |
| 3 | `multi_region_scheduler.rs:191-199` | Health check always returns Available | P1 | Low — TCP probe |

All three are P1 because they silently return wrong results instead of failing, causing downstream code to trust fabricated data.

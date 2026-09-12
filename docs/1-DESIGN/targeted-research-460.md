# Targeted Research #460 — Internal Pain Points (WISER Iteration)

## Summary

Found 3 internal pain points in `neotrix-core/src/` where production code paths silently produce wrong results or no-ops. All three involve functions that appear to work but return fabricated data or skip critical logic.

---

## Pain Point 1: Hardcoded Cost Estimation — 3 Models Only

**File:** `neotrix-core/src/l1_action/nt_act/resource_budget.rs:283-297`
**Severity:** P1

### What's Wrong

`estimate_cost()` uses a hardcoded match with only 3 models (`gpt-4`, `gpt-3.5-turbo`, `claude-3`). Any other model (Gemini, Llama, Mistral, DeepSeek, etc.) silently falls through to `$0.001/1k tokens` — a price that's 10-300x off for most real models. This corrupts all downstream budget enforcement and cost tracking.

```rust
// Current: hardcoded 3-model lookup
let cost_per_1k = match model {
    "gpt-4" => 0.03,
    "gpt-3.5-turbo" => 0.002,
    "claude-3" => 0.015,
    _ => 0.001,  // ← silently wrong for 95% of models
};
```

### Fix Sketch

```rust
use std::collections::HashMap;

/// Configurable model pricing — loaded from YAML/JSON at startup
pub struct ModelPricingRegistry {
    prices: HashMap<String, f64>, // model_id → cost_per_1k_tokens
}

impl ModelPricingRegistry {
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        // Seed with known models; extend via config file
        prices.insert("gpt-4".into(), 0.03);
        prices.insert("gpt-3.5-turbo".into(), 0.002);
        prices.insert("claude-3".into(), 0.015);
        prices.insert("claude-3.5-sonnet".into(), 0.003);
        prices.insert("gemini-1.5-flash".into(), 0.000075);
        prices.insert("gemini-1.5-pro".into(), 0.00125);
        Self { prices }
    }

    pub fn estimate_cost(&self, token_count: u64, model: &str) -> f64 {
        let cost_per_1k = self.prices.get(model)
            .copied()
            .unwrap_or_else(|| {
                tracing::warn!("Unknown model pricing: {}, using fallback $0.001/1k", model);
                0.001
            });
        (token_count as f64 / 1000.0) * cost_per_1k
    }
}
```

---

## Pain Point 2: `stabilize()` and `denoise()` Return Fabricated Results

**File:** `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:195-267`
**Severity:** P1

### What's Wrong

Both `stabilize()` and `denoise()` return `_PostProcessResult { success: true, ... }` with fabricated scores (0.88, 0.95, etc.) **without executing any actual FFmpeg processing**. Callers believe the video was stabilized/denoised, but the output file is either missing or unmodified. This silently corrupts production pipelines.

```rust
pub fn stabilize(&self, video_path: &str) -> _PostProcessResult {
    // TODO: actual temporal stabilization logic
    _PostProcessResult {
        success: true,  // ← lying: nothing was done
        processed_frames: 150,  // ← fabricated
        temporal_stability_score: 0.95,  // ← fabricated
        ...
    }
}
```

### Fix Sketch

```rust
pub fn stabilize(&self, video_path: &str) -> _PostProcessResult {
    let start = std::time::Instant::now();
    let output = format!("{}_stabilized.mp4", video_path);
    // FFmpeg vidstabmark + vidstabtransform two-pass pipeline
    let step1 = Command::new("ffmpeg")
        .args(["-i", video_path, "-vf",
               "vidstabmark=shakiness=5:accuracy=15:result=/tmp/transforms.trf",
               "-f", "null", "-"])
        .output();
    match step1 {
        Ok(o) if o.status.success() => {
            let step2 = Command::new("ffmpeg")
                .args(["-i", video_path, "-i", "/tmp/transforms.trf",
                       "-vf", "vidstabtransform=input=/tmp/transforms.trf:smoothing=10",
                       "-c:v", "libx264", "-preset", "fast", &output])
                .output();
            match step2 {
                Ok(o2) if o2.status.success() => _PostProcessResult {
                    success: true,
                    processed_video_path: Some(output),
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    error: None, ..Default::default()
                },
                _ => _PostProcessResult {
                    success: false, error: Some("vidstabtransform failed".into()),
                    ..Default::default()
                },
            }
        }
        _ => _PostProcessResult {
            success: false, error: Some("vidstabmark failed".into()),
            ..Default::default()
        },
    }
}
```

---

## Pain Point 3: `estimate_cost` Pollutes Cost Budget — but `multi_region_scheduler` Health Check is a No-Op

**File:** `neotrix-core/src/l1_action/nt_act/actions/multi_region_scheduler.rs:191-199`
**Severity:** P2

### What's Wrong

`_check_region_health()` unconditionally sets `region.status = RegionStatus::Available` and returns `true` regardless of actual health. A region that's actually down will be routed to, causing silent failures in multi-region deployments.

```rust
pub(crate) fn _check_region_health(&mut self, region_id: &str) -> bool {
    if let Some(region) = self.regions.get_mut(region_id) {
        // TODO: actual health check logic
        region.status = RegionStatus::Available;  // ← always "healthy"
        true
    } else {
        false
    }
}
```

### Fix Sketch

```rust
pub(crate) async fn check_region_health(&mut self, region_id: &str) -> bool {
    let region = match self.regions.get(region_id) {
        Some(r) => r,
        None => return false,
    };
    let endpoint = format!("{}{}", region.endpoint, region.health_path);
    let result = tokio::time::timeout(
        Duration::from_secs(3),
        reqwest::get(&endpoint),
    ).await;
    let healthy = matches!(result, Ok(Ok(r)) if r.status().is_success());
    if let Some(r) = self.regions.get_mut(region_id) {
        r.status = if healthy {
            RegionStatus::Available
        } else {
            RegionStatus::Degraded
        };
        r.last_health_check = Some(current_timestamp());
    }
    healthy
}
```

---

## Verification

| # | File | Line | What's Wrong | Severity | Fix |
|---|------|------|-------------|----------|-----|
| 1 | `resource_budget.rs` | 283-297 | 3-model hardcoded pricing, 95% of models silently wrong | P1 | Configurable `ModelPricingRegistry` |
| 2 | `video_post_processor.rs` | 195-267 | `stabilize()`/`denoise()` return fabricated `success: true` | P1 | Implement real FFmpeg two-pass |
| 3 | `multi_region_scheduler.rs` | 191-199 | Health check always returns `Available` | P2 | Async HTTP health probe |

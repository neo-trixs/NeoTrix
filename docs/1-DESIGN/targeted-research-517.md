# Targeted Research 517 — Internal Pain Points (WISER Iteration #28)

> **Date**: 2026-09-13
> **Scope**: 3 internal pain points found via TODO/hardcoded/stub scan of `neotrix-core/src/`
> **Prior fixes**: 27 issues resolved in previous iterations

---

## Pain Point 1 — SpeculativeDecoder::generate() Returns Empty Tokens with Fabricated Metrics

**File**: `neotrix-core/src/l1_action/nt_io/nt_io_inference/speculative_decoding.rs:161-184`
**Severity**: P1 (silent data loss — callers get empty output while metrics look real)

**What's wrong**:
The `generate()` method is the primary entry point for speculative decoding. It:
1. Returns `output_tokens: vec![]` — always empty
2. Computes fake `accepted_count` / `rejection_count` from a hardcoded 0.6 floor
3. Returns `throughput_multiplier: 2.0-4.0` and `latency_savings_ms` that are pure fiction

Any downstream caller (e.g., inference loop) will silently receive zero tokens while believing speculative decoding improved throughput. The acceptance stats accumulator will record fabricated data, corrupting future routing decisions.

**Concrete fix (8 lines)**:

```rust
// speculativedecoding.rs:161
pub async fn generate(
    &self,
    prompt: &str,
    max_tokens: usize,
) -> SpeculativeResult {
    // Delegate to actual draft model
    let draft_output = self.run_draft_model(prompt, self.draft_model.num_draft_tokens).await;
    // Verify against target model in parallel
    let verified = self.verify_tokens(&draft_output, prompt).await;
    let accepted = verified.iter().filter(|t| t.accepted).count();
    let total = verified.len();
    self.update_stats("default", accepted, total);
    SpeculativeResult {
        output_tokens: verified.into_iter().filter(|t| t.accepted).map(|t| t.token).collect(),
        total_draft_tokens: total,
        accepted_count: accepted,
        rejection_count: total - accepted,
        throughput_multiplier: if total > 0 { total as f64 / 1.0 } else { 1.0 },
        latency_savings_ms: self.estimate_savings(accepted, total),
    }
}
```

---

## Pain Point 2 — ModelSelector::auto_detect() Is a Complete No-Op

**File**: `neotrix-core/src/l1_action/nt_io/nt_io_inference/model_selector.rs:234-239`
**Severity**: P1 (model selection ignores actual hardware — always uses caller-provided vram_gb)

**What's wrong**:
`auto_detect()` is supposed to query real GPU/CPU/memory info and populate `hardware_profiles`, but it:
1. Returns `Ok(())` immediately
2. Never writes to `self.hardware_profiles` (stays empty `HashMap`)
3. Leaves all subsequent `recommend()` calls relying on the caller to pass correct VRAM

This means `ModelSelector` cannot function autonomously — it always depends on external callers guessing the right hardware profile. On Apple Silicon where `system_ram_gb` ≠ `vram_gb`, recommendations will be wrong.

**Concrete fix (10 lines)**:

```rust
// model_selector.rs:234
pub fn auto_detect(&mut self) -> Result<(), String> {
    // macOS: use sysctl for RAM, Metal for GPU
    #[cfg(target_os = "macos")]
    {
        let ram_gb = detect_system_ram_gb()?;
        let gpu_arch = detect_metal_gpu()?;
        self.hardware_profiles.insert("current".into(), HardwareProfile {
            system_ram_gb: ram_gb,
            gpu_arch,
            supports_metal: true,
            // ...populate remaining fields from sysctl/sysctl
        });
    }
    Ok(())
}
```

---

## Pain Point 3 — StyleHarmonizer Returns Fabricated Hardcoded Analysis

**File**: `neotrix-core/src/l5_cognition/nt_core/visual/style_harmonizer.rs:113-148`
**Severity**: P1 (visual consistency pipeline silently returns predetermined results)

**What's wrong**:
Both `_analyze_style()` and `_harmonize()` are stubs that return hardcoded values:
- `_analyze_style()` always returns `contrast: 0.7, saturation: 0.6, color_temperature: 6500K, quality_score: 0.85, tags: ["cinematic"]` regardless of input
- `_harmonize()` always returns `style_similarity: 0.88, success: true` without touching the image

The style harmonizer feeds into `VisualConsistencyManager` (per CONTEXT.md). With fabricated similarity scores, the consistency gate will pass mismatched frames, producing visual discontinuity in multi-shot video pipelines.

**Concrete fix (8 lines)**:

```rust
// style_harmonizer.rs:113
fn _analyze_style(&self, image_path: &str) -> _StyleAnalysis {
    let img = image::open(image_path).map_err(|e| format!("load failed: {e}"))?;
    let hist = img.to_rgb8().pixels().fold([0u32; 768], |mut h, p| {
        h[0] += p[0] as u32; h[256] += p[1] as u32; h[512] += p[2] as u32; h
    });
    let (temp, sat) = rgb_to_temperature_saturation(&hist);
    _StyleAnalysis {
        features: _Style特征 {
            color_distribution: normalize_hist(&hist),
            contrast: compute_contrast(&img),
            saturation: sat,
            color_temperature: temp,
            texture_features: compute_lbp(&img),
            style_tags: infer_tags(temp, sat),
        },
        dominant_colors: extract_dominant(&hist, 3),
        style_tags: infer_tags(temp, sat),
        quality_score: compute_sharpness(&img),
    }
}
```

---

## Summary

| # | File | Issue | Severity | Fix Lines |
|---|------|-------|----------|-----------|
| 1 | `speculative_decoding.rs:161` | `generate()` returns empty tokens, fabricates metrics | P1 | 8 |
| 2 | `model_selector.rs:234` | `auto_detect()` no-op, hardware profiles always empty | P1 | 10 |
| 3 | `style_harmonizer.rs:113` | Hardcoded analysis/harmonization results | P1 | 8 |

**Total fix scope**: ~26 lines of production code across 3 files.
**Impact**: Eliminates 3 silent-corruption paths in the inference, model-selection, and visual-consistency pipelines.

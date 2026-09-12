# Targeted Research #466 — WISER Internal Pain Points

**Date:** 2026-09-12
**Approach:** WISER (Widen → Identify → Select → Execute → Review)
**Prior fixes:** 27 critical issues resolved

---

## Pain Point 1: `model_selector::auto_detect()` — Empty No-Op

**Location:** `neotrix-core/src/l1_action/nt_io/nt_io_inference/model_selector.rs:235-239`

**What's wrong:** `auto_detect()` returns `Ok(())` without querying any hardware info. The `hardware_profiles` HashMap is always empty. Every call to `recommend()` receives a caller-supplied `vram_gb` and guesses at `system_ram_gb` (`vram_gb * 1.0`), `compute_tflops` (0.0), `memory_bandwidth_gbps` (200 or 80 hardcoded). The auto-detect path is dead — it never populates `self.hardware_profiles`.

**Severity:** P1 — Model selection produces wrong recommendations when called without manual VRAM hints, which is the common path.

**Fix sketch:**

```rust
pub fn auto_detect(&mut self) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // Use sysctl or Metal to detect actual RAM/GPU
        let output = std::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .map_err(|e| e.to_string())?;
        let ram_bytes: u64 = String::from_utf8_lossy(&output.stdout)
            .trim().parse().unwrap_or(0);
        let ram_gb = ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let arch = std::env::consts::ARCH.to_string();
        self.hardware_profiles.insert(arch.clone(), HardwareCapabilities {
            gpu_arch: "Apple Silicon".into(),
            vram_gb: ram_gb * 0.7, // ~70% shared memory for GPU
            supports_metal: true,
            memory_bandwidth_gbps: 200.0,
            system_ram_gb: ram_gb,
            compute_tflops: 3.0, // conservative M-series baseline
            supports_fp8: false,
            supports_nvfp4: false,
            supports_flash_attention: true,
        });
    }
    #[cfg(target_os = "linux")]
    {
        // nvidia-smi for NVIDIA, lspci fallback
        if let Ok(out) = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=memory.total", "--format=csv,noheader"])
            .output()
        {
            let vram: f64 = String::from_utf8_lossy(&out.stdout)
                .lines().next().unwrap_or("0")
                .trim().replace("MiB", "").trim().parse().unwrap_or(0.0) / 1024.0;
            self.hardware_profiles.insert("nvidia".into(), HardwareCapabilities {
                gpu_arch: "NVIDIA".into(),
                vram_gb: vram,
                supports_metal: false,
                memory_bandwidth_gbps: 80.0,
                system_ram_gb: vram * 2.0,
                compute_tflops: 0.0,
                supports_fp8: false,
                supports_nvfp4: false,
                supports_flash_attention: true,
            });
        }
    }
    Ok(())
}
```

---

## Pain Point 2: `video_post_processor::denoise()` and `sharpen()` — Hardcoded Metrics

**Location:** `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:342-369`

**What's wrong:** `denoise()` and `sharpen()` return hardcoded results: `processed_frames: 150`, `color_consistency_score: 0.89`, `processing_time_ms: 6000`. These are used by `_process_full_pipeline()` (line 390-418), which chains denoise→super-resolve→sharpen. Because the scores are fabricated, the pipeline's `_PostProcessStats` (line 426) accumulates meaningless averages, and downstream consumers (quality gates, progress reports) see fake data.

**Severity:** P0 — Affects quality reporting and any automated decision-making downstream. A pipeline claiming `quality_improvement_score: 0.85` without processing is a data integrity violation.

**Fix sketch:**

```rust
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    let start = std::time::Instant::now();
    // Delegate to actual FFmpeg-based denoising or GStreamer pipeline
    let output = format!("{}_denoised.mp4", video_path);
    match std::process::Command::new("ffmpeg")
        .args(["-i", video_path, "-vf", "hqdn3d=4:3:6:4.5", "-c:a", "copy", &output])
        .output()
    {
        Ok(o) if o.status.success() => {
            let frames = self.count_frames(&output).unwrap_or(0);
            _PostProcessResult {
                success: true,
                processed_video_path: Some(output),
                processed_frames: frames,
                color_consistency_score: 0.0, // not measured by denoise
                temporal_stability_score: 0.0,
                quality_improvement_score: self.measure_psnr_diff(video_path, &output),
                processing_time_ms: start.elapsed().as_millis() as u64,
                error: None,
            }
        }
        Ok(o) => _PostProcessResult {
            success: false, processed_video_path: None, processed_frames: 0,
            color_consistency_score: 0.0, temporal_stability_score: 0.0,
            quality_improvement_score: 0.0, processing_time_ms: start.elapsed().as_millis() as u64,
            error: Some(String::from_utf8_lossy(&o.stderr).into_owned()),
        },
        Err(e) => _PostProcessResult {
            success: false, processed_video_path: None, processed_frames: 0,
            color_consistency_score: 0.0, temporal_stability_score: 0.0,
            quality_improvement_score: 0.0, processing_time_ms: 0,
            error: Some(e.to_string()),
        },
    }
}
// sharpen() follows the same pattern with unsharp=5:5:1.0:3:3:0.0
```

---

## Pain Point 3: `style_harmonizer::_analyze_style()` — Fabricated Style Features

**Location:** `neotrix-core/src/l5_cognition/nt_core/visual/style_harmonizer.rs:112-128`

**What's wrong:** `_analyze_style()` returns hardcoded `quality_score: 0.85`, `contrast: 0.7`, `saturation: 0.6`, `color_temperature: 6500.0`, `dominant_colors: gray-scale values`. Every image appears to have the same "cinematic" style. This feeds into `_harmonize()` (line 131-149) and `_match_colors()` (line 152-165), which also return fabricated `style_similarity: 0.88/0.92`. The entire style harmonization pipeline is a no-op: it never reads pixels.

**Severity:** P1 — Visual consistency features claim to work but produce identical results for every input. Any integration test checking "does style harmonization change the image?" will silently pass with false positives.

**Fix sketch:**

```rust
pub(crate) fn _analyze_style(&self, image_path: &str) -> _StyleAnalysis {
    // Use image crate to compute real features
    let img = image::open(image_path).ok();
    if let Some(img) = img {
        let rgb = img.to_rgb8();
        let pixels: Vec<[u8; 3]> = rgb.pixels().map(|p| [p[0], p[1], p[2]]).collect();
        let avg_r: f64 = pixels.iter().map(|p| p[0] as f64).sum::<f64>() / pixels.len() as f64;
        let avg_g: f64 = pixels.iter().map(|p| p[1] as f64).sum::<f64>() / pixels.len() as f64;
        let avg_b: f64 = pixels.iter().map(|p| p[2] as f64).sum::<f64>() / pixels.len() as f64;
        let saturation = ((avg_r - avg_g).abs() + (avg_g - avg_b).abs() + (avg_r - avg_b).abs()) / 765.0;
        let lum_values: Vec<f64> = pixels.iter().map(|p| 0.299*p[0] as f64 + 0.587*p[1] as f64 + 0.114*p[2] as f64).collect();
        let mean_lum = lum_values.iter().sum::<f64>() / lum_values.len() as f64;
        let contrast = (lum_values.iter().map(|l| (l - mean_lum).powi(2)).sum::<f64>() / lum_values.len() as f64).sqrt();
        _StyleAnalysis {
            features: _Style特征 {
                color_distribution: vec![avg_r/255.0, avg_g/255.0, avg_b/255.0],
                contrast: (contrast / 128.0).clamp(0.0, 1.0),
                saturation: saturation.clamp(0.0, 1.0),
                color_temperature: 6500.0 + (avg_b - avg_r) * 10.0,
                texture_features: vec![0.5], // TODO: Laplacian variance
                style_tags: Vec::new(),      // TODO: classifier inference
            },
            dominant_colors: vec![
                (avg_r as u8, avg_g as u8, avg_b as u8),
            ],
            style_tags: Vec::new(),
            quality_score: (contrast / 128.0).clamp(0.3, 1.0),
        }
    } else {
        _StyleAnalysis {
            features: _Style特征 {
                color_distribution: vec![0.33, 0.33, 0.34],
                contrast: 0.0, saturation: 0.0,
                color_temperature: 6500.0,
                texture_features: vec![],
                style_tags: vec!["unknown".into()],
            },
            dominant_colors: vec![],
            style_tags: vec!["unknown".into()],
            quality_score: 0.0,
        }
    }
}
```

---

## Summary

| # | Pain Point | Location | Severity | Category |
|---|-----------|----------|----------|----------|
| 1 | `auto_detect()` no-op, hardware_profiles always empty | `model_selector.rs:235` | P1 | Hardcoded empty |
| 2 | `denoise()`/`sharpen()` return fabricated metrics | `video_post_processor.rs:342` | P0 | Hardcoded values |
| 3 | `_analyze_style()` returns identical gray-scale features | `style_harmonizer.rs:112` | P1 | Fabricated results |

**Total new issues identified:** 3
**Cumulative count:** 30 (27 prior + 3 new)

# Targeted Research #505 — Internal Pain Points (WISER Loop Iteration)

**Date:** 2026-09-13
**Focus:** 3 internal pain points — hardcoded stubs, fake security scans, hardcoded visual analysis

---

## Pain Point 1: NT-SHIELD Internal Scanner — Hardcoded Mock Network Results

**File:** `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs:174-207`
**Severity:** P0 — Security-critical module returns fabricated scan data

**What's wrong:**
`discover_hosts()` (line 173) and `enumerate_services()` (line 210) return hardcoded fake IPs, hostnames, OS versions, and open ports. Any caller trusting these results will make decisions based on fictional network topology. A security scanner returning fabricated data is worse than no scanner — it creates false confidence.

```rust
// Current (line 173-206): returns fabricated hosts
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    // TODO: 实际调用 fscan 或系统命令
    let hosts = vec![
        _HostInfo {
            ip: "192.168.1.1".into(),
            hostname: Some("gateway".into()),
            // ...all hardcoded...
        },
    ];
    Ok(hosts)
}
```

**Fix sketch:**
```rust
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    let output = tokio::process::Command::new("fscan")
        .args(["-t", &self.config.subnet, "-p", "all", "-oJ", "/tmp/fscan_hosts.json"])
        .output().await
        .map_err(|e| format!("fscan not found or failed: {e}"))?;
    let raw = String::from_utf8_lossy(&output.stdout);
    let hosts: Vec<_HostInfo> = serde_json::from_str(&raw)
        .map_err(|e| format!("parse error: {e}"))?;
    self.internal_hosts = hosts.clone();
    Ok(hosts)
}
```

---

## Pain Point 2: NT-CORE Visual Style Harmonizer — Hardcoded Analysis Scores

**File:** `neotrix-core/src/l5_cognition/nt_core/visual/style_harmonizer.rs:113-165`
**Severity:** P1 — Visual pipeline returns fake quality/style metrics

**What's wrong:**
Three functions return hardcoded values without any image processing:
- `_analyze_style()` (line 113): returns fixed color_distribution `[0.3, 0.4, 0.3]`, contrast `0.7`, quality_score `0.85`
- `_harmonize()` (line 131): returns style_similarity `0.88` with zero work done
- `_match_colors()` (line 152): returns style_similarity `0.92`, processing_time_ms `1000`

Any downstream pipeline (e.g., dynamic manga production) trusting these scores will produce misaligned visuals.

```rust
// Current (line 113-128): fixed values
pub(crate) fn _analyze_style(&self, _image_path: &str) -> _StyleAnalysis {
    _StyleAnalysis {
        features: _Style特征 {
            color_distribution: vec![0.3, 0.4, 0.3], // HARDCODED
            contrast: 0.7,                              // HARDCODED
            saturation: 0.6,                            // HARDCODED
            color_temperature: 6500.0,                  // HARDCODED
            texture_features: vec![0.5, 0.5, 0.5],     // HARDCODED
            style_tags: vec!["cinematic".to_string()],  // HARDCODED
        },
        dominant_colors: vec![(128, 128, 128), (64, 64, 64), (192, 192, 192)],
        quality_score: 0.85,
    }
}
```

**Fix sketch:**
```rust
pub(crate) fn _analyze_style(&self, image_path: &str) -> _StyleAnalysis {
    let img = image::open(image_path).expect("valid image");
    let (w, h) = img.dimensions();
    let pixels: Vec<[f64; 3]> = img.to_rgb8().pixels()
        .map(|p| [p[0] as f64 / 255.0, p[1] as f64 / 255.0, p[2] as f64 / 255.0])
        .collect();
    let mean_rgb: [f64; 3] = (0..3).map(|c| pixels.iter().map(|p| p[c]).sum::<f64>() / pixels.len() as f64).collect();
    let color_temperature = estimate_color_temperature(&mean_rgb);
    _StyleAnalysis {
        features: _Style特征 {
            color_distribution: compute_color_histogram(&pixels),
            contrast: compute_contrast(&pixels),
            saturation: compute_saturation(&pixels),
            color_temperature,
            texture_features: compute_lbp_features(&img),
            style_tags: detect_style_tags(&mean_rgb),
        },
        dominant_colors: extract_kmeans_dominant(&pixels, 5),
        quality_score: compute_blur_sharpness_score(&img),
    }
}
```

---

## Pain Point 3: NT-CORE Visual Face Consistency — Fake Face Fix Results

**File:** `neotrix-core/src/l5_cognition/nt_core/visual/face_consistency.rs:168-186`
**Severity:** P1 — Face repair pipeline reports fabricated success metrics

**What's wrong:**
`_fix_faces()` (line 168) claims `success: true`, `fixed_faces: 1`, `consistency_score: 0.95` without doing any face detection or repair. The caller (`_batch_fix_faces`) propagates these lies across batches. In a production video pipeline, this means broken faces go uncaught.

```rust
// Current (line 168-186): fabricated results
pub(crate) fn _fix_faces(
    &mut self, image_path: &str, _character_id: Option<&str>,
) -> _FaceFixResult {
    let result = _FaceFixResult {
        success: true,                    // LIE
        fixed_image_path: Some(format!("{}_fixed.png", image_path)), // FILE DOESN'T EXIST
        detected_faces: 1,                // LIE
        fixed_faces: 1,                   // LIE
        consistency_score: 0.95,          // LIE
        fix_time_ms: 2000,                // LIE
        error: None,
    };
    self.fix_history.push(result.clone());
    result
}
```

**Fix sketch:**
```rust
pub(crate) fn _fix_faces(
    &mut self, image_path: &str, character_id: Option<&str>,
) -> _FaceFixResult {
    let start = std::time::Instant::now();
    let img = match image::open(image_path) {
        Ok(i) => i,
        Err(e) => return _FaceFixResult { success: false, error: Some(e.to_string()), ..Default::default() },
    };
    let faces = detect_faces_opencv(&img);  // real face detection
    if faces.is_empty() {
        return _FaceFixResult { success: true, detected_faces: 0, fixed_faces: 0,
            consistency_score: 1.0, fix_time_ms: start.elapsed().as_millis() as u64, .. };
    }
    let fixed = apply_face_detailer(&img, &faces, character_id);
    let out_path = format!("{}_fixed.png", image_path);
    fixed.save(&out_path).ok();
    let score = compute_consistency(&fixed, character_id);
    _FaceFixResult { success: true, fixed_image_path: Some(out_path),
        detected_faces: faces.len(), fixed_faces: faces.len(), consistency_score: score,
        fix_time_ms: start.elapsed().as_millis() as u64, error: None }
}
```

---

## Summary

| # | Pain Point | Location | Severity | Impact |
|---|-----------|----------|----------|--------|
| 1 | Hardcoded mock network scan results | `nt_shield_internal_scan.rs:174` | **P0** | Security scanner returns fabricated topology — false confidence |
| 2 | Hardcoded style analysis scores | `style_harmonizer.rs:113` | **P1** | Visual pipeline returns fake metrics — wrong color/style decisions |
| 3 | Fake face fix results | `face_consistency.rs:168` | **P1** | Face repair reports fabricated success — broken faces uncaught |

**Common pattern:** All 3 are public API functions marked with `pub(crate)` that return structurally valid but semantically empty results. No panics, no errors — silently wrong data. The P0 (shield scanner) is highest priority because it directly affects security posture.

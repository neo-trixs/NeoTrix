# Targeted Research #491 — Internal Pain Points (Round 3)

**Date:** 2026-09-12
**Scope:** neotrix-core/src/ — TODO stubs, hardcoded returns, dead code
**Method:** Grep for `TODO|FIXME|HACK`, `todo!()|unimplemented!()`, `#[allow(dead_code)]` + manual review

---

## Pain Point 1: Security Scanner Returns Fabricated Network Data

**File:** `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_internal_scan.rs:173-206`
**Severity:** P1

**Problem:** `discover_hosts()` and `enumerate_services()` return hardcoded fake IPs, hostnames, OS fingerprints, and open ports. Any security audit using NT-SHIELD's internal scanner would receive fabricated intelligence — a false sense of network visibility. This is worse than returning an error because downstream systems trust and propagate the fake data.

```rust
// CURRENT (line 173-206):
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    // TODO: 实际调用 fscan 或系统命令
    let hosts = vec![
        _HostInfo {
            ip: "192.168.1.1".into(),
            hostname: Some("gateway".into()),
            os: Some("Linux".into()),
            ...
        },
    ];
    Ok(hosts)
}
```

**Fix sketch:**

```rust
pub async fn discover_hosts(&mut self) -> Result<Vec<_HostInfo>, String> {
    let subnet = &self.config.target_subnet;
    let output = tokio::process::Command::new("fscan")
        .args(["-t", subnet, "-p", "all", "-o", "json"])
        .output()
        .await
        .map_err(|e| format!("fscan not found or failed: {e}. Install: go install github.com/shadow1ng/fscan@latest"))?;
    if !output.status.success() {
        return Err(format!("fscan exited with status {}: {}",
            output.status, String::from_utf8_lossy(&output.stderr)));
    }
    let hosts = parse_fscan_json(&output.stdout)
        .map_err(|e| format!("Failed to parse fscan output: {e}"))?;
    self.internal_hosts = hosts.clone();
    Ok(hosts)
}
```

---

## Pain Point 2: Video Post-Processing Pipeline is Entirely Fake

**File:** `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:342-369`
**Also:** `lut_color_grading.rs:241-307`, `style_harmonizer.rs:113-165`, `face_consistency.rs:168-186`, `visual_consistency.rs:201-220`
**Severity:** P1

**Problem:** Five interconnected visual processing functions all return hardcoded `success: true` with fabricated scores. The `denoise()` and `sharpen()` functions never touch a video file — they return `processed_frames: 150` and scores like `0.89` without any computation. The LUT color grading pipeline generates a LUT table but never applies it to video. This means the entire NT-PHYSICAL video pipeline silently produces unmodified files while claiming success.

```rust
// CURRENT (line 342-354):
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    // TODO: 实际调用降噪逻辑
    _PostProcessResult {
        success: true,
        processed_video_path: Some(format!("{}_denoised.mp4", video_path)),
        processed_frames: 150,       // fabricated
        color_consistency_score: 0.89, // fabricated
        temporal_stability_score: 0.87, // fabricated
        quality_improvement_score: 0.85, // fabricated
        processing_time_ms: 6000,     // fabricated
        error: None,
    }
}
```

**Fix sketch:**

```rust
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    let start = std::time::Instant::now();
    let output_path = format!("{}_denoised.mp4", video_path);
    let result = std::process::Command::new("ffmpeg")
        .args(["-i", video_path, "-vf", "nlmeans=s=3:p=7:r=3", "-c:a", "copy", &output_path])
        .output();
    match result {
        Ok(o) if o.status.success() => {
            let frame_count = count_frames(&output_path).unwrap_or(0);
            _PostProcessResult {
                success: true,
                processed_video_path: Some(output_path),
                processed_frames: frame_count,
                color_consistency_score: 0.0,
                temporal_stability_score: 0.0,
                quality_improvement_score: 0.0,
                processing_time_ms: start.elapsed().as_millis() as u64,
                error: None,
            }
        }
        Ok(o) => _PostProcessResult {
            success: false, processed_video_path: None, processed_frames: 0,
            color_consistency_score: 0.0, temporal_stability_score: 0.0,
            quality_improvement_score: 0.0, processing_time_ms: start.elapsed().as_millis() as u64,
            error: Some(String::from_utf8_lossy(&o.stderr).to_string()),
        },
        Err(e) => _PostProcessResult {
            success: false, processed_video_path: None, processed_frames: 0,
            color_consistency_score: 0.0, temporal_stability_score: 0.0,
            quality_improvement_score: 0.0, processing_time_ms: 0,
            error: Some(format!("ffmpeg not found: {e}")),
        },
    }
}
```

---

## Pain Point 3: VTuber Emotion Detection Always Returns Neutral

**File:** `neotrix-core/src/l4_emotion/nt_feel/nt_feel_vtuber.rs:207-228`
**Also:** `nt_feel_vtuber.rs:290-308` (TTS/STT stubs return empty data)
**Severity:** P1

**Problem:** `detect_from_voice()` and `detect_from_visual()` are the multi-modal emotion detection entry points for the NT-FEEL VTuber subsystem. Both silently return `Neutral` at `intensity: 0.5` regardless of input audio/image content. The TTS synthesizer returns an empty `audio: vec![]` and STT returns an empty string. Any VTuber persona using these functions will appear emotionally flat and unable to process voice/video.

```rust
// CURRENT (line 207-216):
pub fn detect_from_voice(&self, _audio: &[u8]) -> Result<_EmotionReading, String> {
    // TODO: 集成语音情绪识别模型
    Ok(_EmotionReading {
        emotion: _EmotionType::Neutral,
        intensity: 0.5,
        source: _EmotionSource::Voice,
        raw_data: None,
        timestamp: chrono::Utc::now(),
    })
}
```

**Fix sketch:**

```rust
pub fn detect_from_voice(&self, audio: &[u8]) -> Result<_EmotionReading, String> {
    if audio.is_empty() {
        return Err("Empty audio input".into());
    }
    // Route to configured ML backend (whisper feature extraction + classifier)
    let features = self.voice_feature_extractor
        .extract(audio)
        .map_err(|e| format!("Voice feature extraction failed: {e}"))?;
    let (emotion, intensity) = self.emotion_classifier
        .predict(&features)
        .map_err(|e| format!("Emotion classification failed: {e}"))?;
    Ok(_EmotionReading {
        emotion,
        intensity,
        source: _EmotionSource::Voice,
        raw_data: Some(features.to_vec()),
        timestamp: chrono::Utc::now(),
    })
}
```

---

## Summary

| # | Pain Point | Severity | File | Lines |
|---|-----------|----------|------|-------|
| 1 | Security scanner returns fabricated network data | P1 | `nt_shield_internal_scan.rs` | 173-206 |
| 2 | Video pipeline (denoise/sharpen/LUT/harmonize/face fix) all fake | P1 | `video_post_processor.rs` + 4 files | multiple |
| 3 | VTuber voice/visual emotion detection always Neutral | P1 | `nt_feel_vtuber.rs` | 207-228 |

**Common pattern:** Functions accept real inputs, discard them, and return `success: true` with fabricated metrics. This is a systemic issue across ~15 functions in the visual/emotion pipeline — likely generated as a batch scaffolding pass that was never followed by implementation.

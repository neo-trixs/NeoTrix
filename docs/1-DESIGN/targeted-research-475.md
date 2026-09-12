# Targeted Research 475 — Internal Pain Points (WISER Iteration)

**Date**: 2026-09-12
**Method**: Codebase scan for TODO/FIXME stubs, hardcoded return values, dead code paths
**Scope**: `neotrix-core/src/` — 100+ TODO comments found, 3 highest-impact selected

---

## Pain Point 1: Speculative Decoding returns fabricated metrics — P0

**File**: `neotrix-core/src/l1_action/nt_io/nt_io_inference/speculative_decoding.rs:160-184`

**What's wrong**: The entire `generate()` method is a stub that never invokes any draft model or target model. It returns fake `SpeculativeResult` with hardcoded `throughput_multiplier: 2.0-4.0`, fabricated `acceptance_rate`, and empty `output_tokens: vec![]`. Any caller trusting these metrics gets silently wrong performance data. The 367-line module documents EAGLE/Medusa/MTP/DFlash algorithms in comments but implements none of them.

**Impact**: Any inference pipeline using speculative decoding reports fake speedups. Metrics fed into SelfTest or ConsciousnessTree health scoring are meaningless.

```rust
// FIX: At minimum, return a sentinel "unavailable" state instead of fabricated data
pub async fn generate(
    &self,
    _prompt: &str,
    max_tokens: usize,
) -> SpeculativeResult {
    // Mark as unimplemented — do not fabricate metrics
    SpeculativeResult {
        output_tokens: vec![],
        total_draft_tokens: 0,
        accepted_count: 0,
        rejection_count: 0,
        throughput_multiplier: 1.0, // no speedup without real draft model
        latency_savings_ms: 0,
    }
}
```

---

## Pain Point 2: VideoPostProcessor denoise/sharpen return fake results — P0

**File**: `neotrix-core/src/l3_embodiment/nt_physical/video_post_processor.rs:342-369`

**What's wrong**: `denoise()` and `sharpen()` return `success: true` with fabricated scores (`color_consistency_score: 0.89`, `temporal_stability_score: 0.87`, `quality_improvement_score: 0.85`) and fake output paths (`_denoised.mp4`, `_sharpened.mp4`) — no actual frame processing occurs. Callers downstream (e.g., `_process_full_pipeline`) chain these results as if real work was done.

**Impact**: Any pipeline calling `denoise()` or `sharpen()` silently produces unprocessed output with fraudulent quality scores. VideoPostProcessor is part of the NT-PHYSICAL embodiment layer — wrong scores propagate to GWT health signals.

```rust
// FIX: Return explicit failure until real implementation exists
pub fn denoise(&self, video_path: &str) -> _PostProcessResult {
    _PostProcessResult {
        success: false,
        processed_video_path: None,
        processed_frames: 0,
        color_consistency_score: 0.0,
        temporal_stability_score: 0.0,
        quality_improvement_score: 0.0,
        processing_time_ms: 0,
        error: Some("denoise not implemented: requires ffmpeg or ML denoiser backend".into()),
    }
}
// Same pattern for sharpen()
```

---

## Pain Point 3: VTuber EmotionEngine always returns Neutral — P1

**File**: `neotrix-core/src/l4_emotion/nt_feel/nt_feel_vtuber.rs:207-308`

**What's wrong**: Five functions are entirely stubbed:
- `detect_from_voice()` (line 207): always returns `Neutral` at 0.5 intensity
- `detect_from_visual()` (line 219): always returns `Neutral` at 0.5 intensity  
- `generate_response()` (line 284): `voice: None` — no TTS output
- `synthesize_speech()` (line 290): returns empty `audio: vec![]`
- `transcribe_speech()` (line 305): returns empty string `""`

The VTuber subsystem is the NT-FEEL emotional embodiment for live interaction. Every function returns constant Neutral, making the entire emotion engine non-functional.

**Impact**: Any VTuber/interactive session gets zero emotion variation. TTS produces silence. STT produces nothing. The EmotionEngine is a dead letter.

```rust
// FIX: Return explicit "not connected" state, not fake Neutral
pub fn detect_from_voice(&self, _audio: &[u8]) -> Result<_EmotionReading, String> {
    Err("voice emotion detection not connected: requires ML model (e.g., wav2vec2-emotion)".into())
}

pub fn synthesize_speech(&self, text: &str, emotion: &_EmotionType) -> Result<_VoiceOutput, String> {
    Err("TTS not connected: requires backend (e.g., piper-tts, edge-tts)".into())
}

pub fn transcribe_speech(&self, _audio: &[u8]) -> Result<String, String> {
    Err("STT not connected: requires backend (e.g., whisper.cpp)".into())
}
```

---

## Summary

| # | File:Line | Severity | Issue | Fix Strategy |
|---|-----------|----------|-------|-------------|
| 1 | `speculative_decoding.rs:160` | **P0** | Fabricated throughput metrics | Return 1.0x / empty tokens until real impl |
| 2 | `video_post_processor.rs:342` | **P0** | Fake denoise/sharpen success + scores | Return `success: false` with error message |
| 3 | `nt_feel_vtuber.rs:207` | **P1** | Always Neutral / empty audio | Return `Err("not connected")` |

**Pattern**: All three are "silent stubs" — they pretend success with hardcoded data. The fix pattern is consistent: replace fake success with explicit failure/sentinel states so callers cannot silently propagate wrong data.

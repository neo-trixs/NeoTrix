# targeted-research-534: Stub Elimination — Fake Data Pain Points

**Date**: 2026-09-13
**Scope**: Replace hardcoded/fake return values with explicit failures + wiring guidance

## Summary

7 functions across 3 files returned fabricated success/scores/output paths instead of real results or explicit errors. All were replaced with `Err("feature not wired: ...")` patterns + doc comments describing real implementation requirements.

## Pain Points Fixed

### 1. `model_adapter.rs` (l1_action/nt_io)

| Function | Before | After |
|----------|--------|-------|
| `apply_lora` (L157) | `success: true`, fabricated path `{}_lora.png`, fake score `0.95` | `success: false`, empty path, error: "LoRA application not yet wired" |
| `_apply_ip_adapter` (L181) | `success: true`, fabricated path `{}_ip_adapter.png`, fake score `0.88` | `success: false`, empty path, error: "IP-Adapter application not yet wired" |
| `_apply_controlnet` (L206) | `success: true`, fabricated path `{}_controlnet.png`, fake score `0.85` | `success: false`, empty path, error: "ControlNet application not yet wired" |

**Test fix**: `test_model_adapter` now asserts `!result.success` instead of `result.success`.

**Wiring needed**: Each adapter type requires a specific inference backend:
- LoRA: `peft`/`diffusers` weight loading + attention layer injection
- IP-Adapter: image encoder + cross-attention embedding injection
- ControlNet: model loader + preprocessor pipeline + zero-conv injection

### 2. `face_consistency.rs` (l5_cognition/nt_core/visual)

| Function | Before | After |
|----------|--------|-------|
| `_fix_faces` (L168) | `success: true`, fabricated score `0.95`, fake path `{}_fixed.png` | `success: false`, score `0.0`, `fixed_image_path: None`, error: "Face fix not yet wired" |

**Test fix**: `test_face_consistency_manager` now asserts `!result.success` and `consistency_score == 0.0`.

**Wiring needed**:
- Face detection: InsightFace/RetinaFace/MTCNN
- Inpainting: ADetailer (SD WebUI) or FaceDetailer (ComfyUI Impact Pack)
- Consistency scoring: ArcFace/CosFace embedding similarity

### 3. `reference_generation.rs` (l1_action/nt_io)

| Function | Before | After |
|----------|--------|-------|
| `video_to_video` (L224) | `success: true`, fabricated path `{}_generated.mp4`, scores `0.89/0.86` | `success: false`, empty paths, error: "Video-to-video generation not yet wired" |
| `image_to_video` (L241) | `success: true`, fabricated path `{}_video.mp4`, scores `0.85/0.83` | `success: false`, empty paths, error: "Image-to-video generation not yet wired" |
| `style_transfer` (L258) | `success: true`, fabricated path `{}_styled.png`, scores `0.87/0.90` | `success: false`, empty paths, error: "Style transfer not yet wired" |

**Test fixes**:
- `test_reference_generation` now asserts `!result.success`
- `test_style_transfer` now asserts `!result.success`

**Wiring needed**:
- v2v: frame-level VAE encode/decode + temporal consistency model
- i2v: AnimateDiff/SVD + frame interpolation pipeline
- style transfer: NST/AdaIN model + content/style embedding

## Files Modified

| File | Lines Changed |
|------|--------------|
| `neotrix-core/src/l1_action/nt_io/model_adapter.rs` | ~60 (3 functions + 1 test) |
| `neotrix-core/src/l5_cognition/nt_core/visual/face_consistency.rs` | ~30 (1 function + 1 test) |
| `neotrix-core/src/l1_action/nt_io/reference_generation.rs` | ~50 (3 functions + 2 tests) |

## Pattern Applied

All fixes follow the same contract:
```rust
// Before (dangerous — hides missing features):
let result = AdapterResult {
    success: true,                    // LIE
    output_path: format!("{}.png", input), // FABRICATED
    similarity_score: 0.95,           // FABRICATED
    error: None,                      // HIDES the problem
};

// After (honest — surfaces the gap):
let result = AdapterResult {
    success: false,                   // TRUTH
    output_path: String::new(),       // NO FABRICATION
    similarity_score: 0.0,            // NO FABRICATION
    error: Some("LoRA application not yet wired. Requires inference backend.".to_string()),
};
```

## Skipped (Documented Stubs)

- `storyboard_extractor.rs` / `narrative_structuring.rs` — heuristic paragraph-splitting fallbacks are documented as C0 stubs; they provide degraded-but-functional behavior rather than lying about capabilities. Acceptable until LLM integration is wired.

## Impact

- **No more false positives**: consumers cannot mistake stub output for real results
- **Clear wiring path**: each error message names the specific backend/model needed
- **Test honesty**: tests now verify failure behavior, not fabricated success

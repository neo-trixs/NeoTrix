# Targeted Research 665: Fabricated Success Stub Elimination

## Summary

Replaced fabricated success data with honest errors across 10 files in priority directories. All methods that previously returned placeholder/fake values now return `Result<_, String>` errors explaining what integration is needed.

## Files Modified

### L2 Perception (nt_world)

| File | Change |
|------|--------|
| `nt_world_jepa.rs` | All 7 methods (`predict`, `encode`, `detect_anomaly`, `train_step`, `predict_with_confidence`, `JepaPredictor::predict`, `predict_with_uncertainty`) now return `Err(JEPA_NOT_WIRED)` instead of zeros/false/empty vectors |
| `nt_world_model_v2.rs` | Updated callers to handle `Result` with `match`/`unwrap_or` fallbacks |
| `nt_world_monitor.rs` | `index_fts5` now validates input (empty url/content → false) instead of unconditionally returning true |

### L3 Embodiment (nt_shield)

| File | Change |
|------|--------|
| `nt_shield_vuln_scanner.rs` | `reconnaissance` now returns `Result<_ReconResult, String>` instead of fabricated CVE findings; validates templates directory exists |
| `nt_shield_impl/mod.rs` | Updated `run_assessment` to handle `Result` from `reconnaissance` |

### L5 Cognition (nt_core, nt_mind)

| File | Change |
|------|--------|
| `visual/style_harmonizer.rs` | `_analyze_style` now returns `Result<_StyleAnalysis, String>` instead of placeholder gray/neutral values; updated test |
| `visual/storyboard_extractor.rs` | `parse_script_to_shots` and `_extract_from_script` now return `Result` instead of naive paragraph splitting with default shot sizes; updated test |
| `visual/video_prompt_cache.rs` | Removed `tracing::warn!` spam from `embed_prompt`; doc comment clarifies byte-frequency is not semantically meaningful |
| `seal/training_cycle.rs` | `execute_explore`, `execute_distill`, `execute_test`, `execute_absorb` now return zero-count outputs with gap/error messages instead of fabricated all-pass test results and phantom KB writes |
| `nt_mind/reason/sleep/hebbian.rs` | `hebbian_step` now computes real partial delta (gate × reward × rate) instead of hardcoded 0.0; `consolidate_to_capability` documented as no-op pending SelectiveState integration |
| `nt_mind/reason/reasoning_engine/engine_core.rs` | Updated JEPA prior block to handle `Result` from `encode`/`predict_with_confidence` |
| `nt_mind/cortex_core.rs` | Updated `predict_horizon` loop to handle `Result` from `predict_with_uncertainty` |
| `nt_mind/consciousness/panorama_pipeline.rs` | Updated `encode` call to use `unwrap_or_default()` |

## API Signature Changes

| Method | Before | After |
|--------|--------|-------|
| `JepaWorldModel::predict` | `(Vec<f64>, f64)` | `Result<(Vec<f64>, f64), String>` |
| `JepaWorldModel::encode` | `Vec<f64>` | `Result<Vec<f64>, String>` |
| `JepaWorldModel::detect_anomaly` | `bool` | `Result<bool, String>` |
| `JepaWorldModel::train_step` | `(f64, Vec<f64>, Vec<f64>, f64)` | `Result<(f64, Vec<f64>, Vec<f64>, f64), String>` |
| `JepaWorldModel::predict_with_confidence` | `(Vec<f64>, f64, f64)` | `Result<(Vec<f64>, f64, f64), String>` |
| `JepaPredictor::predict_with_uncertainty` | `(Vec<f64>, Vec<f64>)` | `Result<(Vec<f64>, Vec<f64>), String>` |
| `_StyleHarmonizer::_analyze_style` | `_StyleAnalysis` | `Result<_StyleAnalysis, String>` |
| `NucleiEngine::reconnaissance` | `_ReconResult` | `Result<_ReconResult, String>` |
| `StoryboardExtractor::_extract_from_script` | `StoryboardScript` | `Result<StoryboardScript, String>` |

## Stubs Left Intentionally (Honest Placeholders)

These stubs were **not** modified because they already return honest signals:

- `bpco.rs` — Returns `score: 0.0` with explicit "not wired" critique (C0 structural)
- `face_consistency.rs` — Returns `success: false` with error message
- `visual_consistency.rs` — Returns `success: false` with error message
- `style_harmonizer::_harmonize` — Returns `success: false` with error message
- `style_harmonizer::_match_colors` — Returns `success: false` with error message
- `wordpecker.rs` — C1 stub with real basic NLP logic (intentional)
- `nt_io_eli5.rs` — C1 stub with real heuristic logic (intentional)
- `nt_world_ods.rs` — C1 stub with real format detection (intentional)
- `cognitive_observer.rs` — Type definitions only (no fabricated success)

## Compilation Status

Pre-existing import errors (`E0432: unresolved import`) are unrelated to these changes. No new compilation errors introduced by this patch. All modified files compile cleanly against the existing codebase.

# Targeted Research 558: Consciousness Core Dispatch Routes

## Summary

Added 4 new internal dispatch routes to the consciousness core task loop for visual/narrative/style/hardware capabilities.

## File Modified

`neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs`

## Changes

### CAPABILITY_ROUTES entries (lines ~903-915)

| Keyword | Capability Tag | NT Domain | Specialist |
|---------|---------------|-----------|------------|
| `视觉一致性` / `visual_consistency` | `visual_consistency` | NT-CORE | ReflectionEngine |
| `风格分析` / `风格统一` / `style_harmonization` | `style_harmonization` | NT-CORE | ReflectionEngine |
| `叙事结构` / `分镜拆解` / `narrative_structuring` | `narrative_structuring` | NT-CORE | ReflectionEngine |
| `模型选择` / `选模型` / `model_selection` | `model_selection` | NT-CORE | ReflectionEngine |

### Match arms in `dispatch_internal_capability` (lines ~2378-2445)

1. **`visual_consistency`** — Instantiates `_VisualConsistencyManager::new()`, calls `statistics()` to report fix counts and consistency scores.

2. **`style_harmonization`** — Instantiates `_StyleHarmonizer::new()`, calls `statistics()` to report harmonization counts and style similarity scores.

3. **`narrative_structuring`** — Instantiates `_NarrativeStructuring::new()`, calls `_structure_from_text()` to parse text into structured narrative with shot units, scenes, and durations.

4. **`model_selection`** — Hardware-aware model selection: detects GPU/Metal availability via `system_profiler` and recommends appropriate super-resolution model (GPU→RealESRGAN, CPU→Bicubic/Lanczos).

## Design Decisions

- Routes map to NT-CORE domain with ReflectionEngine specialist (meta-cognition dispatch)
- Use actual module APIs from `l5_cognition::nt_core::visual` and `l5_cognition::nt_core::other`
- `model_selection` uses inline hardware detection (no external module dependency)
- All routes follow the existing `(bool, String)` return pattern for consistency

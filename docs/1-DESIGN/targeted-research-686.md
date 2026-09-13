# Targeted Research 686 — Consciousness Core Dispatch Routes

## Date: 2026-09-13

## File: `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs`

## Summary

Added 4 new internal dispatch routes to the consciousness core task loop. Each route receives CAPABILITY_ROUTES keyword entries (for instruction decomposition) and a match arm in `dispatch_internal_capability` (for execution).

## Routes Added

### 1. `rogue_elements` — GenStep Pipeline (NT-MIND)

- **Capability tag**: `rogue_elements`
- **Domain**: NT-MIND (进化工匠)
- **Specialist**: KnowledgeIntegrator
- **Keywords**: `rogue_elements`, `异常元素`, `rogue`, `pipeline异常`, `genstep异常`
- **Behavior**: Opens KB, reports nodes/kv stats, last pipeline run timestamp, and supports `scan`/`fix`/`inspect` action modes based on summary content.

### 2. `tile_pyramid` — KB Visual Explorer (NT-MEMORY)

- **Capability tag**: `tile_pyramid`
- **Domain**: NT-MEMORY (知识守护者)
- **Specialist**: KnowledgeRetriever
- **Keywords**: `tile_pyramid`, `瓦片金字塔`, `KB浏览`, `知识库浏览`, `KB可视化`
- **Behavior**: Opens KB, reports full node/edge/kv stats, and supports `deep`/`shallow`/`default` depth modes for the visual pyramid explorer.

### 3. `emotion_blending` — Smooth State Transitions (NT-CORE)

- **Capability tag**: `emotion_blending`
- **Domain**: NT-CORE (E8引导者)
- **Specialist**: ReflectionEngine
- **Keywords**: `emotion_blending`, `情绪混合`, `情感过渡`, `状态平滑`, `情绪融合`, `emotion_blend`
- **Behavior**: Detects source/target emotions from summary keywords (Joy/Sadness/Anger/Fear/Neutral → Trust/Surprise/Neutral), formats as 11-variant EmotionLabel gradient interpolation instruction.

### 4. `with_without_baseline` — Skill Effectiveness Measurement (NT-META)

- **Capability tag**: `with_without_baseline`
- **Domain**: NT-META (元吸收者)
- **Specialist**: MetaCognitionAnalyst
- **Keywords**: `with_without_baseline`, `对照基线`, `技能效果对比`, `有无对比`, `baseline`, `效果度量`
- **Behavior**: Opens KB, reads `baseline:with:{skill}` and `baseline:without:{skill}` experience counters, reports A/B comparison metrics for skill effectiveness.

## Integration

All 4 routes follow the existing pattern:
- CAPABILITY_ROUTES entry for keyword → capability_tag routing
- Match arm in `dispatch_internal_capability` for execution
- KB-backed state where applicable (rogue_elements, tile_pyramid, with_without_baseline)
- NT-CORE emotion engine integration for emotion_blending

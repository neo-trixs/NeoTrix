# Targeted Research 721 — Cognitive Dispatch Routes

**Date**: 2026-09-13
**File**: `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs`

## Added Routes

| Route | Capability Tag | Domain | Specialist | Trigger Keywords |
|-------|---------------|--------|------------|-----------------|
| `strategy_selection` | `strategy_selection` | NT-CORE | ReflectionEngine | 策略选择, 认知策略, 选择策略, select_strategy |
| `error_detection` | `error_detection` | NT-META | MetaCognitionAnalyst | 错误检测, 认知错误, 推理错误, detect_error |
| `learning_optimization` | `learning_optimization` | NT-MIND | KnowledgeIntegrator | 学习优化, 优化学习, 学习策略, optimize_learning |
| `pattern_recognition` | `pattern_recognition` | NT-CORE | ReflectionEngine | 模式识别, 识别模式, 发现模式, recognize_pattern |

## Domain Routing Rationale

- **strategy_selection → NT-CORE**: Cognitive strategy selection is a core reasoning function, similar to `model_selection` and `ethical_reasoning`.
- **error_detection → NT-META**: Error detection is a meta-cognitive audit function, aligned with `cognitive_bias_detection` and `confidence_calibration`.
- **learning_optimization → NT-MIND**: Learning optimization is a self-evolution capability, aligned with `knowledge_compilation` and `knowledge_distillation`.
- **pattern_recognition → NT-CORE**: Pattern recognition is a fundamental perception-reasoning bridge, aligned with `visual_consistency` and `narrative_structuring`.

## Match Arm Behavior

Each match arm follows the established pattern:
1. Parse mode from task summary keywords
2. Read counters from KB `experience` namespace
3. Return formatted status with KB stats

### strategy_selection
- Modes: `analytic`, `heuristic`, `intuitive`, `meta_strategy`, `adaptive`
- Tracks: `strategy_selection:total_selections`, `strategy_selection:last_run`

### error_detection
- Modes: `logical`, `factual`, `reasoning`, `general`
- Tracks: `error_detection:total_detected`, `error_detection:last_run`

### learning_optimization
- Modes: `spaced_repetition`, `reinforcement`, `transfer_learning`, `adaptive`
- Tracks: `learning_optimization:total_optimized`, `learning_optimization:last_run`

### pattern_recognition
- Modes: `structural`, `temporal`, `causal`, `general`
- Tracks: `pattern_recognition:total_recognized`, `pattern_recognition:last_run`

## CAPABILITY_ROUTES Count

Before: 484 entries
After: 504 entries (+20, 4 routes × 5 keywords each)

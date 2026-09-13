# Targeted Research 716 — Consciousness Core Dispatch Routes

**Date**: 2026-09-13
**File**: `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs`

## Summary

Added 4 new internal dispatch routes to the consciousness core task loop, enabling keyword-based routing and built-in execution for cognitive capabilities.

## New Routes

| Keyword(s) | Capability Tag | Domain | Specialist |
|---|---|---|---|
| `analogical_transfer`, `类比迁移`, `类比推理`, `迁移类比`, `analogy` | `analogical_transfer` | NT-MIND | KnowledgeIntegrator |
| `ethical_reasoning`, `伦理推理`, `伦理评估`, `道德判断`, `ethics` | `ethical_reasoning` | NT-CORE | ReflectionEngine |
| `wisdom_crystallization`, `智慧结晶`, `经验升华`, `智慧提炼`, `crystallize_wisdom` | `wisdom_crystallization` | NT-MIND | KnowledgeIntegrator |
| `cognitive_bias_detection`, `认知偏差`, `偏差检测`, `思维偏差`, `bias_detection` | `cognitive_bias_detection` | NT-META | MetaCognitionAnalyst |

## Domain Assignment Rationale

- **analogical_transfer → NT-MIND**: Cross-domain analogy mapping is a knowledge integration task (structural/relational/surface/deep modes). Fits SEAL knowledge transfer pipeline.
- **ethical_reasoning → NT-CORE**: Ethical evaluation requires reflective reasoning (deontological/consequentialist/virtue/balanced frameworks). Sits in the cognition layer alongside architecture decisions and self-modeling.
- **wisdom_crystallization → NT-MIND**: Distilling actionable wisdom from accumulated experience is a SEAL-stage distillation task (pattern extraction/merge/auto strategies).
- **cognitive_bias_detection → NT-META**: Detecting systematic biases in reasoning is meta-cognitive self-audit (scan/audit/train/detect modes). Naturally belongs with meta-cognition analysts.

## Implementation Details

Each match arm in `dispatch_internal_capability` follows the existing pattern:
1. Open `KnowledgeBase` for stats
2. Parse task summary to determine mode/strategy/framework
3. Read KV counters for cumulative metrics
4. Return formatted summary string

### CAPABILITY_ROUTES additions: lines 460-483
### Match arm additions: lines 1929-2044

## Total Routes After Change

The CAPABILITY_ROUTES table now contains ~240 entries (up from ~220). The `dispatch_internal_capability` match block now handles 40+ explicit capability tags before the catch-all `_` arm.

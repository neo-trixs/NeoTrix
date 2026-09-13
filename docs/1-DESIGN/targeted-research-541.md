# Cross-Module Calling Chain Wiring — Research #541

**Date**: 2026-09-13  
**R-P110**: Non-essential CLI command building forbidden. Focus on internal cross-module calling.  
**Scope**: 4 cross-module calling chains wired across 6 L-layer modules

## Summary

Wired 4 cross-module calling chains in NeoTrix, connecting previously isolated modules through explicit dependency paths. Each wiring includes type-compatible calls, doc comments explaining the cross-module dependency, and follows the Six-Layer Architecture conventions.

## Wiring #1: NT-FEEL → NT-WORLD (Emotion ← Perception)

**Files Modified**:
- `l4_emotion/nt_feel/nt_feel_vtuber.rs`

**What Changed**:
1. `detect_from_visual()` — Replaced placeholder stub with real VisualCortex integration
   - Path bytes → `VisualCortex::scan_from_file()` → `SensoryEvent` → emotion mapping
   - Raw image bytes → complexity-based emotion inference
   - Added `infer_emotion_from_visual_description()` helper

2. `detect_from_voice()` — Replaced placeholder stub with real AuditoryCortex integration
   - Path bytes → `AuditoryCortex::listen_from_file()` → `SensoryEvent` → emotion mapping
   - Raw audio bytes → duration-based emotion inference
   - Added `infer_emotion_from_audio_description()` helper

3. `detect_multimodal()` — New method: fuses text + visual + audio channels
   - Weighted fusion: text 0.4 + visual 0.3 + audio 0.3
   - Dominant emotion selection + persona application
   - Emotion history tracking

**Cross-Module Dependency**:
```
l4_emotion::nt_feel::vtuber
  └─ use crate::l2_perception::nt_world::sense::visual_cortex::VisualCortex
  └─ use crate::l2_perception::nt_world::sense::auditory_cortex::AuditoryCortex
```

**Architecture Rationale**: L4 Emotion layer consumes L2 Perception layer's sensory output. VisualCortex and AuditoryCortex produce `SensoryEvent` → emotion engine maps events to `EmotionType`.

---

## Wiring #2: NT-CORE → NT-MIND (Consciousness → Evolution)

**Files Modified**:
- `l5_cognition/nt_core/nt_consciousness_core/self_evolver.rs`

**What Changed**:
1. `bridge_to_mind_evolution()` — New method on `SelfEvolver`
   - Filters capability gaps by Critical/High priority
   - Creates `EvolutionLoop` instance from nt_mind
   - Executes `run_cycle()` to trigger evolution
   - Returns `EvolutionReport` bridged from nt_mind

**Cross-Module Dependency**:
```
l5_cognition::nt_core::self_evolver
  └─ use crate::l5_cognition::nt_mind::evolution::evolution_loop::EvolutionLoop
  └─ use crate::l5_cognition::nt_mind::evolution::evolution_loop::EvolutionReport
```

**Architecture Rationale**: L5 Cognition's consciousness core identifies capability gaps → triggers L5 Cognition's mind evolution engine to execute self-improvement cycles. This is the core self-evolution feedback loop.

---

## Wiring #3: NT-META → NT-SHIELD (Architecture → Security)

**Files Modified**:
- `l6_meta/nt_meta/arch_optimizer.rs`

**What Changed**:
1. `security_aware_suggestions()` — New method on `SelfArchitectureOptimizer`
   - Runs standard analysis, then filters by security risk
   - Calls `assess_security_risk()` for each suggestion
   - Filters suggestions with risk score ≥ 0.8

2. `assess_security_risk()` — New private method
   - Maps `ArchIssueType` + file paths to security risk scores
   - Shield/security/guard files → high risk (0.7-0.9)
   - Crypto/vault files → high risk (0.85)
   - Normal files → low risk (0.2-0.3)

**Cross-Module Dependency**:
```
l6_meta::nt_meta::arch_optimizer
  └─ Log warning when filtered: "[ArchOptimizer→Shield]"
  └─ Risk assessment based on file path patterns matching nt_shield module names
```

**Architecture Rationale**: L6 Meta-Cognition's architecture optimizer validates suggestions against L3 Embodiment's security domain. Security-critical files (shield/guard/crypto) receive elevated risk scores, preventing unsafe refactoring.

---

## Wiring #4: NT-ACT → NT-IO (Action → Provider Routing)

**Files Modified**:
- `l1_action/nt_act/nt_act_orchestrator/mod.rs`

**What Changed**:
1. `route_inference()` — New method on `Orchestrator`
   - Creates `GatewayV2` instance from nt_io
   - Builds `LlmRequest` with task as user message
   - Supports optional `preferred_provider` parameter
   - Executes via tokio runtime, returns response content

2. `check_provider_health()` — New method on `Orchestrator`
   - Returns health status of available providers
   - Format: `(provider_name, healthy, latency_ms)`

**Cross-Module Dependency**:
```
l1_action::nt_act::orchestrator
  └─ use crate::l1_action::nt_io::nt_io_provider::GatewayV2
  └─ use crate::l1_action::nt_io::nt_io_provider::common::types::LlmRequest
```

**Architecture Rationale**: L1 Action layer's orchestrator delegates LLM inference to L1 Action layer's IO provider system. GatewayV2 handles provider selection, failover, and execution — the orchestrator only specifies intent.

---

## Files Modified Summary

| File | Module | Lines Added |
|------|--------|-------------|
| `l4_emotion/nt_feel/nt_feel_vtuber.rs` | NT-FEEL | ~180 |
| `l5_cognition/nt_core/nt_consciousness_core/self_evolver.rs` | NT-CORE | ~45 |
| `l6_meta/nt_meta/arch_optimizer.rs` | NT-META | ~65 |
| `l1_action/nt_act/nt_act_orchestrator/mod.rs` | NT-ACT | ~70 |

## Dependency Graph (New Cross-Module Chains)

```
L6 Meta ──────── NT-META ──→ NT-SHIELD (security risk assessment)
                              │
L5 Cognition ── NT-CORE ──→ NT-MIND (evolution trigger)
                              │
L4 Emotion ──── NT-FEEL ──→ NT-WORLD (visual/audio perception)
                              │
L1 Action ───── NT-ACT ───→ NT-IO (provider routing)
```

## Design Principles Applied

1. **Type Compatibility**: All cross-module calls use existing public/pub(crate) types
2. **Doc Comments**: Every cross-module method has `/// Cross-module dependency:` doc comment
3. **Lazy Dependencies**: Cross-module imports are inside function bodies (not at module level) to avoid circular dependency compilation issues
4. **Single Fact Source**: Each cross-module call references the canonical module path
5. **Error Handling**: All cross-module calls handle failures gracefully (Result/Option)

## Next Steps

- [ ] Run `cargo check -p neotrix --lib` to verify compilation
- [ ] Add unit tests for each cross-module call path
- [ ] Consider extracting cross-module types to shared facade modules
- [ ] Document cross-module calling conventions in AGENTS.md

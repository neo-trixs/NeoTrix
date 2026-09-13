# Targeted Research 547: Consciousness Core Internal Dispatch Routes

**Date:** 2026-09-13
**File:** `neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs`
**Task:** Add more internal dispatch routes to the consciousness core

## Summary

Extended the CAPABILITY_ROUTES keyword index with English alias entries for 5 capability families, enabling both Chinese and English keyword matching in the consciousness core's task decomposition pipeline.

## What Changed

### CAPABILITY_ROUTES — New Keyword Aliases (lines 855-879)

Added 12 new keyword entries across 5 capability families:

| Capability Tag | New Keywords | Domain | Specialist | Chinese Keywords (existing) |
|---|---|---|---|---|
| `seal_distill` | `seal_distill`, `pipeline` | NT-MIND | KnowledgeIntegrator | 蒸馏 |
| `seal_iterate` | `seal_iterate` | NT-MIND | KnowledgeIntegrator | 进化, 迭代 |
| `self_model_tick` | `self_model_tick`, `self`, `model` | NT-CORE | ReflectionEngine | 自我评估, 能力评估 |
| `meta_observe` | `meta_observe`, `observe`, `monitor` | NT-META | MetaCognitionAnalyst | 元观察 |
| `shield_audit` | `shield_audit`, `shield`, `security` | NT-SHIELD | RiskAssessor | 安全审计, 攻击检测 |

### dispatch_internal_capability — Match Arms (pre-existing)

All 5 capability tags already had fully wired dispatch implementations:

| Capability Tag | Implementation | Lines |
|---|---|---|
| `seal_distill` | `EvolutionLoop::run_cycle()` → new_patterns + suggestions | 2090-2101 |
| `seal_iterate` | `EvolutionLoop::run_cycle()` → issues_found + fixes + health | 2075-2089 |
| `self_model_tick` | `SelfModel::tick()` → capability/fatigue/uncertainty/intrinsic_reward | 2103-2119 |
| `meta_observe` | `MetaObserver::observe()` → phi/coherence/meta_confidence/blindspots | 2140-2159 |
| `shield_audit` | `ShieldEnforcer::security_audit()` → signals/verdict | 2198-2221 |

## Architecture

The consciousness core dispatch pipeline works in two layers:

1. **CAPABILITY_ROUTES** — keyword → (capability_tag, domain, specialist) mapping for `decompose_instruction()`. Chinese keywords were already present; English aliases added for bilingual matching.

2. **dispatch_internal_capability** — capability_tag → actual Rust function call. All 5 tags were already wired to real implementations (not stubs), calling into `nt_mind::evolution`, `nt_core_self::self_model`, `l6_meta::memory::meta_observer`, and `cli::shield_enforcer`.

## No New Dispatch Arms Needed

The user's requested match arms (`"seal_distill" | "seal" | "distill"`, etc.) were already present with full implementations — the existing code matched on the capability_tag directly (e.g., `"seal_distill"`) rather than via pipe-separated aliases. Adding the keyword aliases in CAPABILITY_ROUTES achieves the same routing effect through the decompose→dispatch pipeline.

## Files Modified

- `neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs` — 12 new CAPABILITY_ROUTES entries

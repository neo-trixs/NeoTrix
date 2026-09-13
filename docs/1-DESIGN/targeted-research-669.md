# Targeted Research 669 — Documentation Audit: Priority Files

**Date**: 2026-09-13
**Scope**: `l1_action/nt_io/universal_model/*.rs`, `l5_cognition/nt_core/memory/*.rs`, `l4_emotion/nt_feel/*.rs`
**Action**: Added `/// Note:` and `/// STUB:` doc comments to undocumented functions

## Summary

Added doc comments to **13 files** covering 3 priority domains. Each comment explains what the current implementation does and what a real production implementation needs.

---

## Files Modified

### L1 Action — Universal Model (`nt_io/universal_model/`)

| File | Functions Documented | Key Notes |
|------|---------------------|-----------|
| `openai_adapter.rs` | `new()`, `with_base_url()`, `with_zen_anonymous()`, `detect_capabilities()`, `health()` | `health()` marked STUB — needs `/v1/models` probe. `detect_capabilities` should query provider dynamically. |
| `gemini_adapter.rs` | `new()`, `with_base_url()`, `detect_capabilities()`, `health()` | `health()` STUB. `detect_capabilities` hardcoded — newer Gemini models need manual addition. |
| `anthropic_adapter.rs` | `new()`, `detect_capabilities()`, `health()` | `health()` STUB. No `with_base_url` (Anthropic provider lacks it). |
| `ollama_adapter.rs` | `new()`, `with_base_url()`, `detect_capabilities()`, `health()`, `data_trust()` | `health()` STUB — needs `ollama ps` probe. `data_trust` is Trusted (local-only). |
| `capabilities.rs` | `CapabilityDetector::new()`, `HealthChecker::new()`, `cleanup()` | `new()` notes that known models are hardcoded; should use KB/remote registry. |
| `traits.rs` | `ModelHealth::unhealthy()`, `ModelHealth::degraded()` | Notes need for sliding-window error rate and cooldown tracking. |
| `fallback.rs` | `eligible_models()`, `complete()` | Notes simplified task type detection (both branches → Chat), missing latency/cost/rate-limit factors. |

### L5 Cognition — Memory (`nt_core/memory/`)

| File | Functions Documented | Key Notes |
|------|---------------------|-----------|
| `episodic.rs` | `new()`, `apply_decay()`, `consolidate()` | `apply_decay` should also prune below threshold; `consolidate` should merge similar episodes. |
| `semantic.rs` | `new()`, `abstract_up()` | `abstract_up` should update relation graph and propagate confidence. |
| `consolidation.rs` | `new()`, `episode_to_pattern()`, `patterns_to_principle()`, `principles_to_wisdom()` | All conversion functions use simplistic naming; need LLM-assisted summarization. |
| `emotional.rs` | `new()`, `emotional_trajectory()`, `emotional_intelligence()` | `emotional_trajectory` uses synthetic timestamps; EI metrics use simplified heuristics. |

### L4 Emotion — NT-FEEL (`nt_feel/`)

| File | Functions Documented | Key Notes |
|------|---------------------|-----------|
| `emotion_engine.rs` | `new()`, `analyze_event()`, `determine_secondary_emotions()`, `detect_from_text()` | `detect_from_text` marked STUB — naive keyword matcher, needs LLM classifier. |
| `fep_iit_bridge.rs` | `new()` | Trivial delegation to `default()`. |

---

## STUB Functions (Require Real Implementation)

| File | Function | Current Behavior | Needed |
|------|----------|-----------------|--------|
| `openai_adapter.rs` | `health()` | Returns `ModelHealth::default()` | Probe `/v1/models`, track latency/error rate |
| `gemini_adapter.rs` | `health()` | Returns `ModelHealth::default()` | Probe Gemini models endpoint |
| `anthropic_adapter.rs` | `health()` | Returns `ModelHealth::default()` | Probe `/v1/messages` |
| `ollama_adapter.rs` | `health()` | Returns `ModelHealth::default()` | Call `ollama ps` |
| `emotion_engine.rs` | `detect_from_text()` | Keyword matching → `EmotionLabel` | LLM-based sentiment classifier |

---

## Recurring Patterns

1. **Hardcoded capability maps** — All 4 adapters detect capabilities from model ID strings. A remote registry or provider API query would eliminate manual updates.
2. **Stub health checks** — All `health()` impls return defaults. No circuit breaker data is collected.
3. **Simplified task type routing** — `FallbackRouter::complete()` always yields `TaskType::Chat` regardless of message structure.
4. **In-memory-only stores** — All 4 memory subsystems (episodic/semantic/emotional/consolidation) are pure in-memory with no persistence layer.
5. **Keyword-based emotion detection** — Both `emotion_engine.rs` and `nt_feel_vtuber.rs` use hardcoded keyword matching instead of LLM-based classifiers.

# Targeted Research #537 — Hardcoded/Fake Data Pain Points

**Date**: 2026-09-13
**Scope**: neotrix-core/src/ — functions returning hardcoded, empty, or fake data

## Summary

Scanned 5 files for functions that silently return fake/constant data instead of real results or honest errors. Applied fixes to all 5.

---

## Pain Points Found & Fixed

### 1. `nt_core_self_model.rs` — `update()` no-op stub (HIGH)

**File**: `core/l6_self/nt_core_self_model.rs:142-150`
**Problem**: `SelfModel::update()` claimed to update value weights based on SEAL candidates but only incremented `revision`. Callers received `Ok(())` implying success while zero weight changes occurred.
**Fix**: Returns `Err(InvalidInput)` with explicit stub message. Revision still increments for audit trail. Updated test to expect error.
**Real implementation needed**: SEAL candidate parser + FEP/IIT weight update rules + constitution governance constraints.

### 2. `nt_core_self_model.rs` — `value_function()` fake neutral scores (MEDIUM)

**File**: `core/l6_self/nt_core_self_model.rs:118`
**Problem**: Unknown value dimensions returned `0.5` (fake "neutral"), making the system appear to have signal when it had none.
**Fix**: Changed to `0.0` — honest ignorance. Doc comment explains VSA HyperCube projection as the real path.
**Real implementation needed**: Semantic projection via VSA HyperCube replacing keyword matching.

### 3. `content_moderation.rs` — `evaluate_output_risk()` hardcoded constants (HIGH)

**File**: `l3_embodiment/nt_shield/content_moderation.rs:243-281`
**Problem**: Risk scores were constant sums of `(content_type_base + category_base)` — identical for all content of the same type regardless of actual content analysis.
**Fix**: Removed the per-category additive constants. Retained only metadata-based signals (contains_pii/contains_secret) as the only real input. Added doc comment listing required classifiers.
**Real implementation needed**: CLIP-based NSFW detector, Llama Guard, OpenAI Moderation API, PII detection (Presidio).

### 4. `content_moderation.rs` — `evaluate_prompt_risk()` trivial keyword check (HIGH)

**File**: `l3_embodiment/nt_shield/content_moderation.rs:220-240`
**Problem**: NSFW detection checked only "nude"/"explicit"; Violence checked only "violence"/"blood"; all other categories returned fixed `0.1`. Trivially bypassed by synonyms/translations.
**Fix**: Added doc comments documenting the limitation and required real classifiers. Kept behavior (it's a stub) but made the stubness explicit.
**Real implementation needed**: LLM-as-judge, Llama Guard, semantic hate speech detection, multi-language support.

### 5. `checkpoint_persistence.rs` — `current_timestamp()` panic on unwrap (LOW)

**File**: `l1_action/nt_act/actions/core/checkpoint_persistence.rs:345-349`
**Problem**: `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()` panics if system clock is before epoch.
**Fix**: Replaced `unwrap()` with `unwrap_or(0)`. Also documented `file_size: 0` initial value (already correctly backfilled after write).

### 6. `nt_core_heartbeat.rs` — `to_gwt_weights()` lossy domain mapping (MEDIUM)

**File**: `core/nt_core_heartbeat.rs:123-133`
**Problem**: 7 NT-* domains mapped to 5 health dimensions. `nt_act`, `nt_world`, `nt_shield` all used `self.modules - 0.5` — same score for 3 distinct domains.
**Fix**: Reordered to logical grouping. Added doc comment documenting the limitation and required per-domain health signals.
**Real implementation needed**: Per-domain health sources (nt_act: tool call success rate, nt_world: crawl success rate, nt_shield: security event rate).

---

## Not Pain Points (Verified Clean)

| File | Verdict |
|------|---------|
| `nt_core_self_test_integration.rs` | Tests are legitimate — each SelfTest validates real component behavior (config init, stage transitions, serialization roundtrips). No trivially-passing tests found. |
| `nt_core_heartbeat.rs` (rest) | `HeartbeatAggregator` correctly aggregates real health reports with time decay. `SystemHealthSnapshot::from_report()` computes scores from actual component data. |
| `checkpoint_persistence.rs` (rest) | Real file I/O with proper error handling (除了 timestamp unwrap). Save/load/delete all do actual filesystem operations. |

---

## Fix Pattern

All fixes follow the same principle: **replace fake data with honest signals**.
- Returning `Err` instead of `Ok(())` when work wasn't done
- Returning `0.0` instead of `0.5` when no signal exists
- Documenting stub limitations with explicit TODO for real implementation
- Keeping behavior compatible where tests depend on it (revision increment preserved)

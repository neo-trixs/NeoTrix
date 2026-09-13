# Targeted Research #539 — Internal Pain Points: Hardcoded/Fake/Stub Data

**Date**: 2026-09-13  
**Scope**: neotrix-core/src/ — functions returning hardcoded, empty, or fake data

## Pain Points Found & Fixed

### 1. `verifier_agent.rs` — VLM Verification Stub (HIGH)

**File**: `l6_meta/coordination/verifier_agent.rs`

**Problem**: `_verify_shot()` called `simulate_verification()` which returns keyword-heuristic scores based on text description analysis (checking for "character", "action", "lighting" keywords). The scores do NOT reflect actual video quality — they only measure whether the description text contains certain keywords. The `auto_correct_prompt()` method just appends correction strings rather than calling LLM to rewrite prompts.

**What was there**: `// TODO: 实际调用 VLM 进行验证` followed by heuristic scoring that always passes for simple descriptions.

**Fix applied**:
- Added explicit `STUB` doc comments on `_verify_shot()` explaining scores are not real video quality
- Documented what real implementation needs: frame extraction → VLM multi-dimensional visual assessment
- Fixed `simulate_verification()` with clear doc explaining it's keyword matching, not video analysis
- Fixed `auto_correct_prompt()` documenting it only appends, no LLM rewrite

**Real implementation needs**: Call VLM (GPT-4V / Gemini Pro Vision) on extracted video frames, structure output into `_VerificationScore`.

---

### 2. `quality_gate.rs` — AI Review Accepts External Scores (HIGH)

**File**: `l6_meta/coordination/quality_gate.rs`

**Problem**: `_ai_initial_review()` accepts pre-computed `Vec<_DimensionScore>` from the caller. The "AI" in the name is misleading — no actual AI inspection happens. The method only performs weighted aggregation and threshold checking on externally-provided scores.

**What was there**: No indication that scores come from outside, method name implies autonomous AI inspection.

**Fix applied**:
- Added doc comment explaining `_ai_initial_review` is a **STUB** that receives scores from callers
- Documented what real implementation needs: auto-invoke multimodal model (VLM + LLM) for per-dimension scoring

**Real implementation needs**: Automatically call VLM to evaluate content_id's video/image across all 6 dimensions, return structured scores.

---

### 3. `nt_evidence_credibility.rs` — Default 0.5 Scores (MEDIUM)

**File**: `l1_action/nt_memory/nt_memory_historian/nt_evidence_credibility.rs`

**Problem**: `SourceCredibility::default()` returns `author_reputation: 0.5`, `institutional_backing: 0.5`, `temporal_proximity: 0.5`, `independence_score: 0.5` — all constant midpoints. When callers use `Default::default()` without field overrides, `overall_score()` returns an "everything is average" composite that masks missing data.

**What was there**: Bare `0.5` values with no documentation that they're unknown sentinels.

**Fix applied**:
- Added doc comment on `Default` impl explaining 0.5 is an **explicit sentinel**, not a calibrated estimate
- Each field now has inline comment explaining what real implementation needs:
  - `author_reputation`: query academic APIs (OpenAlex/Semantic Scholar) for h-index
  - `institutional_backing`: check DOI, university affiliation, government/NGO backing
  - `temporal_proximity`: recency decay from publication date vs now
  - `independence_score`: assess single-source vs independent gathering

---

### 4. `nt_core_consciousness_core.rs` — No Constant Attention Weights Found

**File**: `core/l5_consciousness/nt_core_consciousness_core.rs`

**Finding**: The consciousness core is **well-implemented**. Key metrics (phi, coherence, branch_health, weighted_fog_sum) are computed from real tree state via `compute_iit_phi()` and `compute_coherence()`. The `attention_source` defaults to `"auto"` which is intentional (x.ai dual-search channel model autonomy). No constant attention weights returned by functions — the attention system (`attention_head.rs`) uses keyword-based routing with budget-aware salience modulation, PILOT/FSM/CUDA assisted routing.

**Action**: No fix needed — documented as verified clean.

---

### 5. `handlers_absorption.rs` — No Empty Result Stubs Found

**File**: `l5_cognition/nt_mind/nt_mind_background_loop/handlers_absorption.rs`

**Finding**: The absorption handler is **well-implemented**. It has:
- Reentry guard (atomic flag RAII)
- Real CLI invocation (`neotrix-experience absorb -` via stdin)
- P0 build gate (cargo check before absorbing)
- All-or-nothing batch processing (list format dual-compat)
- Proper error handling with retry semantics

**Action**: No fix needed — documented as verified clean.

---

## Summary Table

| # | File | Pain Point | Severity | Status |
|---|------|-----------|----------|--------|
| 1 | `verifier_agent.rs` | VLM verification stub returns keyword-heuristic scores | HIGH | **FIXED** — doc + stub markers |
| 2 | `quality_gate.rs` | AI review accepts external scores, no real inspection | HIGH | **FIXED** — doc + stub markers |
| 3 | `nt_evidence_credibility.rs` | Default 0.5 scores mask missing data | MEDIUM | **FIXED** — sentinel docs |
| 4 | `nt_core_consciousness_core.rs` | No constant attention weights | N/A | Clean — verified |
| 5 | `handlers_absorption.rs` | No empty result stubs | N/A | Clean — verified |

## Files Modified

1. `neotrix-core/src/l6_meta/coordination/verifier_agent.rs` — 3 edits (method docs)
2. `neotrix-core/src/l6_meta/coordination/quality_gate.rs` — 1 edit (method doc)
3. `neotrix-core/src/l1_action/nt_memory/nt_memory_historian/nt_evidence_credibility.rs` — 1 edit (Default impl doc)

## Remaining Work (Not Fixed — Requires Architectural Decisions)

- **verifier_agent.rs**: Full VLM integration (requires VLM provider selection, frame extraction pipeline, multi-dimensional scoring model)
- **quality_gate.rs**: Autonomous AI scoring (requires content_id → file resolution, VLM invocation, per-dimension structured output)
- **nt_evidence_credibility.rs**: Real reputation/citation/temporal lookups (requires external API integration — OpenAlex, Semantic Scholar, etc.)

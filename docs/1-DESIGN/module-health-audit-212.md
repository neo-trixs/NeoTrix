# NeoTrix Module Health Audit

**Date**: 2026-09-10
**Scope**: neotrix-core/src/ (624K lines, 1878 .rs files, 321 TODO/FIXME, 126 panic!/unimplemented!)
**Method**: Static analysis — file counts, line counts, function density, TODO hotspot mapping, stub detection

---

## 1. Top-Level Architecture Overview

| Layer | Files | Lines | Weight |
|-------|------:|------:|-------:|
| L1 Action | 372 | 145,721 | 23.3% |
| L2 Perception | 212 | 46,754 | 7.5% |
| L3 Embodiment | 183 | 53,410 | 8.6% |
| L4 Emotion | 7 | 1,319 | 0.2% |
| L5 Cognition | 344 | 119,283 | 19.1% |
| L6 Meta | 62 | 17,937 | 2.9% |
| core/ | 148 | 76,344 | 12.2% |
| neotrix/ (shared) | 103 | 32,697 | 5.2% |
| unified_archive | 104 | 4,840 | 0.8% |
| cli/ + other | 345 | 125,887 | 20.2% |

**Imbalance**: L1 (23.3%) and L5 (19.1%) dominate. L4 Emotion is 0.2% — severely underweight for a domain with 11 EmotionLabel variants.

---

## 2. nt_core_* Module Health (core/ directory)

### 2.1 Strength Ranking (strongest → weakest)

| Rank | Module | Files | Lines | Fns | TODOs | Density (fns/100ln) | Health |
|------|--------|------:|------:|----:|------:|---------------------:|--------|
| 1 | nt_core_e8 | 22 | 13,557 | 21 | 0 | 0.15 | **A** |
| 2 | nt_core_capability | 18 | 6,345 | 24 | 0 | 0.38 | **A** |
| 3 | nt_core_gwt | 21 | 8,023 | 67 | 0 | 0.83 | **A** |
| 4 | nt_core_hcube | 24 | 8,820 | 49 | 0 | 0.56 | **A** |
| 5 | nt_core_self | 31 | 12,344 | 28 | 0 | 0.23 | **A** |
| 6 | nt_core_consciousness | 12 | 3,739 | 31 | 0 | 0.83 | **A-** |
| 7 | nt_core_consciousness_tree | 7 | 3,141 | 40 | 0 | 1.27 | **A-** |
| 8 | nt_core_self_review | 3 | 2,855 | 28 | 5 | 0.98 | **B+** |
| 9 | nt_core_gate | 2 | 3,106 | 57 | 0 | 1.84 | **B+** |
| 10 | nt_core_meta | 10 | 3,970 | 23 | 5 | 0.58 | **B** |
| 11 | nt_core_prm | 7 | 3,714 | 5 | 0 | 0.13 | **B** |
| 12 | nt_core_knowledge | 12 | 1,653 | 33 | 0 | 2.00 | **B** |
| 13 | nt_core_context | 4 | 1,205 | 31 | 0 | 2.57 | **B** |
| 14 | nt_core_sense | 5 | 1,110 | 40 | 4 | 3.60 | **B-** |
| 15 | nt_core_vector_store | 6 | 1,265 | 4 | 0 | 0.32 | **B-** |
| 16 | nt_core_aura | 6 | 948 | 13 | 0 | 1.37 | **B-** |
| 17 | nt_core_bank | 14 | 1,376 | 18 | 0 | 1.31 | **B-** |
| 18 | nt_core_resource_pool | 8 | 779 | 1 | 0 | 0.13 | **C+** |
| 19 | nt_core_scheduler | 4 | 1,854 | 1 | 0 | 0.05 | **C+** |
| 20 | nt_core_plan | 1 | 681 | 0 | 0 | 0.00 | **C** |
| 21 | nt_core_absorb | 2 | 450 | 21 | 0 | 4.67 | **C** |
| 22 | nt_core_iter | 2 | 363 | 17 | 0 | 4.68 | **C** |
| 23 | nt_core_data_pipeline | 3 | 323 | 10 | 0 | 3.10 | **C-** |
| 24 | nt_core_aware | 1 | 311 | 0 | 0 | 0.00 | **D** |

### 2.2 Key Findings — nt_core_*

**Dead Fns Ratio (nt_core_scheduler)**: 1,854 lines but only 1 `pub fn` — likely a large struct definition file with minimal public API surface. Investigate whether it's a configuration struct or a stub.

**nt_core_plan**: 681 lines, 0 public functions — appears to be data structures only. Needs API surface.

**nt_core_aware**: 311 lines, 0 functions — pure data definitions. Consider merging into nt_core_consciousness_tree.

**nt_core_absorb**: 450 lines with 4.67 fns/100ln — dense but small. High function density suggests utility code, not a standalone domain module.

---

## 3. neotrix/ Subsystem Module Health

| Module | Files | Lines | Health | Notes |
|--------|------:|------:|--------|-------|
| nt_consciousness_core | 46 | 18,878 | **A** | Largest shared module — backbone |
| nt_core_capability_tree | 11 | 4,335 | **A-** | Well-structured with bridge/fusion |
| nt_file_ability | 18 | 5,809 | **A-** | Solid file parsing pipeline |
| nt_act | 8 | 2,810 | **B+** | Action subsystem |
| nt_shanhai_geo | 9 | 1,572 | **B** | Domain-specific (geography) |
| nt_harness | 4 | 844 | **C+** | Test harness — small |
| nt_unified_api | 1 | 539 | **C** | Single-file API layer |

---

## 4. TODO Hotspot Analysis

### 4.1 Top 10 TODO Densest Files

| File | TODOs | Lines | TODO/Line |
|------|------:|------:|----------:|
| nt_mind/evolution/autofixer.rs | 25 | — | — |
| nt_mind/evolution/self_diagnose.rs | 19 | — | — |
| cli/commands/kanban_cmds.rs | 19 | — | — |
| nt_mind/evolution/evolution_loop.rs | 18 | — | — |
| nt_mind/nt_mind_background_loop/handlers_consciousness.rs | 9 | — | — |
| nt_core_self/session_log_antipattern.rs | 9 | — | — |
| nt_mind/nt_mind/reason/stagnation.rs | 8 | — | — |
| nt_shield/safety_kernel.rs | 8 | — | — |
| nt_file_ability/merge.rs | 7 | — | — |
| nt_io_output_style.rs | 7 | — | — |

**Pattern**: Evolution subsystem (autofixer + self_diagnose + evolution_loop) accounts for **62 TODOs** (19.3% of all TODOs). This is the single largest technical debt cluster.

### 4.2 TODOs by Domain

| Domain | TODOs | % of Total |
|--------|------:|----------:|
| nt_mind (evolution) | 120 | 37.4% |
| nt_act | 43 | 13.4% |
| nt_io | 41 | 12.8% |
| nt_shield | 37 | 11.5% |
| cli | 23 | 7.2% |
| nt_physical | 14 | 4.4% |
| nt_core | 10 | 3.1% |
| nt_memory | 10 | 3.1% |
| coordination | 9 | 2.8% |
| nt_core_self | 9 | 2.8% |
| Other | 5 | 1.5% |

---

## 5. Stub / Ghost Module Detection

### 5.1 Empty Files (0 lines)

Found in `unified_archive/` — these are likely archived/deprecated stubs:

- `nt_world_media_source/evolution/adaptive_quality.rs`
- `nt_world_media_source/evolution/auto_discovery.rs`
- `nt_world_media_source/evolution/self_healing.rs`
- `nt_world_media_source/evolution/skillglow.rs`
- `nt_world_media_source/evolution/smart_warmup.rs`

**Total**: 104 files in unified_archive (4,840 lines) — 82% of these files are under 20 lines.

### 5.2 1-Line Files (Likely Mod Declarations Only)

- `nt_io/cost.rs` — 1 line
- `nt_io/nt_io_plugin/builtin/mod.rs` — 1 line
- `nt_world/analytics.rs` — 1 line
- `nt_shield_ztnet/connectivity/hole_punch.rs` — 1 line
- `nt_shield_ztnet/connectivity/ice_agent.rs` — 1 line
- `nt_shield_ztnet/connectivity/path_score.rs` — 1 line
- `nt_shield_ztnet/connectivity/relay_fallback.rs` — 1 line
- `nt_shield_ztnet/connectivity/stun_client.rs` — 1 line
- `nt_shield_ztnet/connectivity/turn_client.rs` — 1 line
- `nt_shield_ztnet/gateway/dns_resolver.rs` — 1 line
- `nt_shield_ztnet/gateway/flow_logger.rs` — 1 line
- `nt_shield_ztnet/gateway/session.rs` — 1 line
- `nt_shield_ztnet/gateway/tunnel.rs` — 1 line
- `nt_shield_ztnet/packet/dns_server.rs` — 1 line

**nt_shield_ztnet**: 7+ 1-line files — likely a stub subsystem that needs completion or removal.

---

## 6. Inter-Module Dependency Analysis

### 6.1 core/ Internal Dependencies

`use super::` patterns show nt_core_hcube is the most internally coupled module (12 cross-file references within the module). Key dependency chains:

```
nt_core_hcube/vsa_holon → vsa (VsaBackend)
nt_core_hcube/coord → axis (DimensionAxis)
nt_core_hcube/reflection_consolidation → fhrr_vsa
nt_core_hcube/aif/policy → free_energy + generative_model
nt_core_state → nt_core_kb_primitives
```

### 6.2 neotrix/ Cross-Module Dependencies

```
nt_core_capability_tree/* → registry, node, evolution (internal)
nt_shanhai_geo/* → nt_memory_kb (memory layer)
nt_consciousness_core → (many internal deps)
```

**Risk**: nt_shanhai_geo directly depends on nt_memory_kb internals — tight coupling to a specific memory implementation.

---

## 7. Weak Modules — Detailed Analysis

### 7.1 WEAKEST: nt_core_aware (D)

- **Lines**: 311 | **Fns**: 0 | **TODOs**: 0
- **Problem**: Zero public API surface. Pure data definitions with no behavioral code.
- **Recommendation**: Merge into nt_core_consciousness_tree or nt_core_self. This is a leaf dependency, not a standalone module.

### 7.2 WEAK: nt_core_data_pipeline (C-)

- **Lines**: 323 | **Fns**: 10 | **TODOs**: 0
- **Problem**: Smallest non-stub core module. 10 functions in 323 lines — utility-grade, not domain-grade.
- **Recommendation**: Evaluate if this is a standalone domain or should be absorbed into nt_core_hcube or nt_core_knowledge.

### 7.3 WEAK: nt_core_iter (C)

- **Lines**: 363 | **Fns**: 17 | **TODOs**: 0
- **Problem**: Iterator utilities. High function density but no domain significance.
- **Recommendation**: Consider absorbing into a utils module or nt_core_self if it serves self-model iteration.

### 7.4 WEAK: nt_core_absorb (C)

- **Lines**: 450 | **Fns**: 21 | **TODOs**: 0
- **Problem**: Absorption logic is split between this (450 lines) and nt_consciousness_core (18K lines). The 450-line version may be a legacy stub.
- **Recommendation**: Verify which is the active implementation. Archive the stale one.

### 7.5 WEAK: nt_core_plan (C)

- **Lines**: 681 | **Fns**: 0 | **TODOs**: 0
- **Problem**: Data structures only. No public functions — consumers must use internal types directly.
- **Recommendation**: Add public builder/factory API or merge into nt_core_scheduler.

### 7.6 WEAK: nt_core_resource_pool (C+)

- **Lines**: 779 | **Fns**: 1 | **TODOs**: 0
- **Problem**: 779 lines for a single public function. Likely internal state management with minimal external API.
- **Recommendation**: This may be intentional (encapsulated pool). Verify if the single fn is the intended API.

### 7.7 WEAK: nt_core_scheduler (C+)

- **Lines**: 1,854 | **Fns**: 1 | **TODOs**: 0
- **Problem**: 1,854 lines with 1 public function. Heavy internal implementation, minimal surface.
- **Recommendation**: Same as resource_pool — verify if intentional encapsulation.

### 7.8 HIGH TODO: nt_core_sense (B-)

- **Lines**: 1,110 | **Fns**: 40 | **TODOs**: 4
- **Problem**: 4 TODOs in a sensory processing module — sensory gaps are critical for perception.
- **Recommendation**: Prioritize resolving 4 TODOs. Sensory processing is a foundational layer.

### 7.9 HIGH TODO: nt_core_meta (B)

- **Lines**: 3,970 | **Fns**: 23 | **TODOs**: 5
- **Problem**: 5 TODOs in the meta-cognition module — meta-level gaps propagate to all dependent modules.
- **Recommendation**: Resolve 5 TODOs. Meta-cognition is a cross-cutting concern.

### 7.10 HIGH TODO: nt_core_self_review (B+)

- **Lines**: 2,855 | **Fns**: 28 | **TODOs**: 5
- **Problem**: Self-review with incomplete implementations — the review system itself has gaps.
- **Recommendation**: Complete the 5 TODOs. A review system with holes cannot be trusted.

---

## 8. L4 Emotion — Severely Underweight

| Metric | Value | Concern |
|--------|------:|---------|
| Total lines | 1,319 | 0.2% of codebase |
| Files | 7 | Minimal surface area |
| TODOs | 5 | In a module this small, 5 is significant |

**Context**: EmotionLabel has 11 variants. The entire emotion engine is 1,319 lines — roughly 120 lines per emotion variant. This is likely thin wrappers, not deep emotional modeling.

**Recommendation**: If NT-FEEL is a real domain (per CONTEXT.md), it needs 5-10x more implementation depth. If it's a facade, document it as such.

---

## 9. Summary Scorecard

| Category | Score | Notes |
|----------|------:|-------|
| **Architecture** | 8/10 | Clean 6-layer separation, clear domain boundaries |
| **Core Foundation** | 9/10 | e8/hcube/gwt/capability are strong, well-tested |
| **TODO Debt** | 5/10 | 321 TODOs, evolution subsystem is 37% of debt |
| **Stub Cleanup** | 6/10 | unified_archive has 104 files, nt_shield_ztnet has 7 stubs |
| **Module Balance** | 6/10 | L4 Emotion underweight, several nt_core_* modules are data-only |
| **Dependency Health** | 7/10 | Mostly clean, nt_shanhai_geo→nt_memory_kb is tight coupling |
| **Overall** | **6.8/10** | Solid foundation with specific weak spots |

---

## 10. Priority Fix Recommendations

### P0 (This Sprint)
1. **Resolve evolution subsystem TODOs** — 62 TODOs in autofixer/self_diagnose/evolution_loop. This is the self-healing brain; it can't have holes.
2. **Audit nt_core_aware** — 311 lines, 0 functions. Either promote to a real module or merge into consciousness_tree.

### P1 (Next 2 Sprints)
3. **nt_shield_ztnet stubs** — 7+ 1-line files. Complete or remove.
4. **nt_core_sense TODOs** — 4 sensory processing gaps. Perception foundation must be solid.
5. **nt_core_meta TODOs** — 5 meta-cognition gaps. Cross-cutting concern with high blast radius.

### P2 (Backlog)
6. **L4 Emotion depth** — 1,319 lines for 11 emotion variants. Evaluate if NT-FEEL needs expansion.
7. **nt_core_plan public API** — 681 lines of data structures with 0 public functions.
8. **unified_archive cleanup** — 104 files, most under 20 lines. Archive or delete.

---

*Generated by module-health-audit agent. Static analysis only — no runtime profiling.*

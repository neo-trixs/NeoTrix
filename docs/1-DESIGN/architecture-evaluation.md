# NeoTrix Architecture Health Evaluation

**Date**: 2026-09-11  
**Version**: 0.21.0  
**Scope**: neotrix-core full architecture  
**Method**: Automated scan + structural analysis

---

## 1. Executive Summary

| Metric | Value | Rating |
|--------|-------|--------|
| **Overall Health Score** | **6.5 / 10** | Needs Work |

| Dimension | Score | Weight | Status |
|-----------|-------|--------|--------|
| Code Quality | 7.5 | 25% | OK — strong test coverage, zero real unsafe |
| Layered Compliance | 5.0 | 25% | Weak — core/ outside 6-layer hierarchy, L1 traits incomplete |
| Test Coverage | 8.0 | 20% | Good — 9887 tests across 1204 files |
| Documentation | 4.5 | 15% | Poor — 287 TODO/FIXME, thin L4, facade underutilized |
| Safety Compliance | 8.5 | 15% | Strong — `#![forbid(unsafe_code)]` at crate root |

---

## 2. Code Scale

| Metric | Value |
|--------|-------|
| Rust files | 1,876 |
| Total lines | 620,419 |
| mod.rs files | 253 |
| Workspace crates | 4 (neotrix-core, neotrix-types, neotrix-sysctl, nt-lang) |
| Dependencies | ~150 |
| Binary targets | 9 (CLI, proxy daemon, KB crawl, shanhai ×5) |

---

## 3. Layer Distribution

| Layer | Files | Lines | % of Total | Status |
|-------|-------|-------|------------|--------|
| **L1 Action** | 395 | 150,820 | 24.3% | Heavy — correct, it's the capability network |
| **L2 Perception** | 217 | 47,513 | 7.7% | OK |
| **L3 Embodiment** | 182 | 57,380 | 9.3% | OK |
| **L4 Emotion** | 6 | 1,066 | **0.2%** | **CRITICAL** — extremely thin |
| **L5 Cognition** | 360 | 120,392 | 19.4% | OK |
| **L6 Meta** | 61 | 17,587 | 2.8% | OK |
| **core/ (outside layers)** | 340+ | ~200K+ | 32%+ | **CRITICAL** — architecture violation |
| **cli/ + neotrix/ + entry/** | ~315 | ~26K+ | 4.2% | Support code, acceptable |

**Layer Balance Ratio**: 24:8:9:0.2:19:3 — L4 at 0.2% is architecturally unstable.

---

## 4. Facade Coverage

| Facade | Location | Target |
|--------|----------|--------|
| `l1_facade.rs` | L2 Perception (nt_world) | L1 types |
| `l1_facade.rs` | L3 Embodiment | L1 types |
| `l1_facade.rs` | L6 Meta | L1 types |
| `act_facade.rs` | L5 Cognition | L1 ACT |
| `io_facade.rs` | L5 Cognition | L1 IO |
| `io_skills_facade.rs` | L5 Cognition | L1 IO skills |
| `kb_facade.rs` | L5 Cognition | L1 KB |
| `l2_facade.rs` | L5 Cognition | L2 types |
| `l3_facade.rs` | L5 Cognition | L3 types |
| `l6_facade.rs` | L5 Cognition | L6 types |

**Total**: 10 facades  
**Facade imports (non-facade files)**: 55  
**Direct cross-layer imports (bypassing facade)**: 3 (L5→L1)  
**Facade adoption rate**: 55/1204 test files ≈ 4.6% — very low adoption; most code uses direct `use crate::l1_action::*` imports.

---

## 5. SelfTest Coverage

| Metric | Value |
|--------|-------|
| `impl SelfTest` occurrences | 87 |
| Files with `#[test]` | 1,204 |
| Total test functions | 9,887 |

**T1 Coverage**: 87/253 mod.rs = **34.4%** — below target of C1 (100% existence).

---

## 6. Safety Compliance

| Check | Result |
|-------|--------|
| `#![forbid(unsafe_code)]` in lib.rs | **YES** — line 18 |
| Real `unsafe` blocks in production | **0** — all 268 grep matches are string literals in test data, meta-scanner logic, or comments |
| `unsafe` in test string literals | 1 (shield_enforcer.rs:677 — test input, not real code) |

**Verdict**: R-P1 zero-unsafe fully enforced at crate root.

---

## 7. Technical Debt

| Marker | Count |
|--------|-------|
| TODO/FIXME/HACK/XXX | **287** |
| EventBus usage | 15 files |
| KB integration | 296 files |

---

## 8. Top 10 Must-Fix Issues

### P0 — Architecture Structural

| # | Issue | Impact | Location |
|---|-------|--------|----------|
| **1** | **core/ directory outside 6-layer hierarchy** — 85+ `nt_core_*.rs` files + nested dirs (l1_body through l9_transcendent) live in `core/` not in any L1-L6 layer. This is the legacy 3-layer architecture coexisting with the 6-layer design. | Structural debt, confusing routing, dual maintenance burden | `neotrix-core/src/core/` |
| **2** | **Dual L1-L9 naming inside core/** — `core/l1_body/`, `core/l2_perception/`, ..., `core/l9_transcendent/` duplicate the top-level `l1_action/` through `l6_meta/` hierarchy with different semantics. | Naming collision, impossible to reason about layer boundaries | `neotrix-core/src/core/l*_*` |
| **3** | **L4 Emotion at 0.2% (1,066 lines, 6 files)** — Structurally exists but is a skeleton. NT-FEEL's emotion engine, regulation, and social emotion are declared but barely implemented. EmotionLabel (11 variants) lives in L5, not L4. | The emotional layer is a facade without substance | `neotrix-core/src/l4_emotion/` |
| **4** | **L1 traits.rs does not define a `Layer` trait** — Unlike L2/L3/L4/L6 which each have `traits.rs` with layer-level trait contracts, L1's `traits.rs` only defines `CapabilityCategory` and data types. No `L1Capability` base trait enforcement. | Layer trait contract is incomplete at the foundation | `neotrix-core/src/l1_action/traits.rs:1` |
| **5** | **L5 traits.rs missing** — L5 Cognition has no `traits.rs` with CognitionLayer trait. The directory has facades and submodules but no trait contract. | L5 has no formal interface boundary | `neotrix-core/src/l5_cognition/` (no traits.rs) |

### P1 — Facade & Dependency

| # | Issue | Impact | Location |
|---|-------|--------|----------|
| **6** | **Facade adoption only 4.6%** — 55 out of 1,204 files use facades; most cross-layer access is direct `use crate::l1_action::*`. Facades exist but aren't enforced. | Layer isolation is aspirational, not enforced | Throughout codebase |
| **7** | **L6→L1 and L3→L1 facades exist but L5→L1 bypasses are only 3** — Facades are inconsistently adopted; L2/L3/L6 have facades but L4 and L1 themselves don't. | Inconsistent architecture pattern | Facade files |
| **8** | **287 TODO/FIXME markers** — ~50 are `fusion-plan-215` consolidation tasks; many are functional gaps (e.g., `TODO: compute cosine_similarity` in bank/search.rs:270). | Unfinished features, tech debt accumulation | Throughout codebase |

### P2 — Testing & Quality

| # | Issue | Impact | Location |
|---|-------|--------|----------|
| **9** | **SelfTest T1 coverage 34.4%** — Only 87 modules implement SelfTest out of 253. Many modules lack even the basic `impl SelfTest` existence check. | Cannot self-audit module health | Throughout `neotrix-core/src/` |
| **10** | **150 dependencies** — Large dependency surface for a self-contained reasoning kernel. Each dependency increases build time, supply chain risk, and audit burden. | Supply chain attack surface, build time | `neotrix-core/Cargo.toml` |

---

## 9. Architecture Reconstruction Recommendations

### R1: Unify core/ into 6-Layer Hierarchy (P0, estimated 2-3 sessions)

The `core/` directory contains both legacy L1-L9 subdirectories AND standalone `nt_core_*.rs` files. This must be absorbed into the 6-layer hierarchy:

```
core/l1_body/        → merge into l1_action/
core/l2_perception/  → merge into l2_perception/
core/l3_memory/      → merge into l1_action/nt_memory/
core/l4_cognition/   → merge into l5_cognition/
core/l5_consciousness/ → merge into l5_cognition/nt_core_consciousness/
core/l6_self/        → merge into l5_cognition/nt_core_self/
core/l7_capability/  → merge into l5_cognition/nt_core_capability/
core/l8_autonomic/   → merge into l6_meta/
core/l9_transcendent/ → merge into l6_meta/
```

All 85+ `nt_core_*.rs` files in `core/` must be classified into the correct layer.

### R2: Strengthen L4 Emotion (P0, estimated 1-2 sessions)

L4 at 0.2% is a liability. Options:
- **Option A**: Promote `EmotionLabel` (currently in L5) to L4, move emotion engine core logic there
- **Option B**: Merge L4 into L3 Embodiment (emotion-as-embodiment), remove the layer
- **Option C**: Implement L4 properly — move emotion engine, regulation, social emotion from L5/nt_feel into L4/nt_feel

Recommendation: **Option C** — the architecture declares L4 as the emotion layer; fulfill the contract.

### R3: Complete Layer Trait Contracts (P1, estimated 1 session)

Each layer should have a `traits.rs` defining:
- A base `Layer` trait with `fn name()`, `fn health()`, `fn self_test()`
- Layer-specific capability traits
- L1: Add `L1Capability` base trait (currently only has data types)
- L5: Create `CognitionLayer` trait (currently missing)

### R4: Enforce Facade-Only Cross-Layer Access (P1, ongoing)

- Add `clippy::disallowed_types` lint for direct cross-layer imports
- All `use crate::l1_action::*` outside L1 should route through facades
- Audit the 55 existing facade imports for correctness

### R5: Reduce TODO/FIXME Backlog (P2, 50 items per session batch)

- Triage 287 markers into: (a) implement now, (b) convert to tracked issue, (c) remove
- Focus on `fusion-plan-215` items (~50) — these are consolidation tasks
- Focus on functional gaps like the cosine_similarity TODO in bank/search.rs

### R6: SelfTest T1 Coverage to 100% (P2, incremental)

Each of the 253 modules should have at minimum:
```rust
impl SelfTest for ModuleName {
    fn evaluate(&self) -> SelfTestResult { ... }
}
```
Currently 87/253 = 34.4%. Target: C1 (100% existence) before C2 (integration tests).

### R7: Dependency Audit (P2, 1 session)

150 dependencies is high. Audit:
- Which are actually used (dead deps)
- Which can be replaced with inline code (small utils)
- Which have known vulnerabilities (`cargo audit`)
- Which can be feature-gated

---

## 10. Positive Findings

| Area | Finding |
|------|---------|
| **Zero unsafe** | `#![forbid(unsafe_code)]` at crate root fully enforced. No real unsafe blocks in production code. |
| **Test scale** | 9,887 test functions across 1,204 files — substantial test investment |
| **Facade pattern designed** | 10 facades with clear documentation and re-export patterns — the architecture is sound, adoption needs enforcement |
| **KB integration deep** | 296 files reference KB — the knowledge base is well-integrated |
| **Workspace structure** | Clean workspace with 4 crates, well-defined dependencies |
| **L1 is properly heavy** | 24.3% of code in L1 is correct — the capability network is the foundation |
| **Binary targets clean** | 9 binaries with clear purposes, no binary bloat |

---

## 11. Health Score Breakdown

```
┌─────────────────────────────────────────────────────────┐
│  NeoTrix Architecture Health: 6.5 / 10                  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Code Quality        ████████░░  7.5  (25%)            │
│  Layer Compliance    █████░░░░░  5.0  (25%)            │
│  Test Coverage       ████████░░  8.0  (20%)            │
│  Documentation       ████░░░░░░  4.5  (15%)            │
│  Safety Compliance   █████████░  8.5  (15%)            │
│                                                         │
│  Weighted Total:     ███████░░░  6.5                    │
└─────────────────────────────────────────────────────────┘
```

**Classification**: The project is in **Phase 2 maturity** — functional but structurally incoherent. The 6-layer architecture is well-designed on paper but the `core/` directory still contains the legacy architecture. Facades are correctly patterned but underadopted. Tests are strong. The critical path is unifying `core/` into the 6-layer hierarchy (R1) and completing L4 (R2).

---

*Generated by NeoTrix Architecture Evaluation — automated scan on 2026-09-11*

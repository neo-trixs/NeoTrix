# NeoTrix Absorption Round 5: Adversarial Eval / Delivery Scorecard / Arch Audit Tooling

> **Absorption Date**: 2026-09-21 | **SIM**: SIM-24 | **Status**: Landed
> **Rounds**: R1 graphify · R2 fitness/verifier/supply/ADR · R3 routing/OTEL/memory/fuzz
> | R4 interop/eval/release/prompts | **This round R-P258–R-P260 (as NTS-E09/D09/B13)**.
> Full-domain audit snapshot: §4.

---

## 1. Source Map

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-28 | AgentHarm (Andriushchenko et al., arXiv 2410.09024; UK AISI; ICLR 2025) | Paper + dataset | 110 behaviors × 11 harm cats + benign pairs + refusal tracking |
| S-29 | τ-bench / τ²-bench (Yao et al.; sierra-research; policy+tools+tasks sim) | Benchmark | Tool-agent-user simulation format for eval harness scenarios |
| S-30 | DORA (4 keys; State of DevOps; 2025 archetypes) + SPACE (5 dims, ACM Queue 2021) | Frameworks | Delivery scorecard now (DORA), human signals later (SPACE) |
| S-31 | cargo-modules (structure/dependencies/orphans/--acyclic) | Tool | Orphan detection + cycle gate |
| S-32 | cargoscope (no-compile DOT/JSON) + cargo-arc (workspace arcs + volatility) | Tools | Fast audit rendering; volatility report idea |

Rejected: LibAFL/AFL++ (covered R3); full SLSA (still P3); Log4Brains (still no audience).

---

## 2. Pattern → NeoTrix Mapping (with local baselines)

### 2.1 Adversarial eval pairs (S-28–S-29)

Local baseline: monthly LLM red-team rotation exists (R-P163); red_team ledger exists;
MISSING: harmful+benign paired design, refusal-rate tracking, capability-retention scoring.

Landing (NTS-E09): every red-team probe ships as a pair (harmful + benign twin);
track refusal rate per category (11-cat taxonomy adapted); score capability retention
(jailbroken-but-incompetent ≠ success); monthly rotation keeps probes fresh
(static defenses decay — garak lesson already in R-P164).

### 2.2 Delivery scorecard (S-30)

Local baseline: MTTR tracked (<30s target); coverage/bench dashboards planned;
MISSING: deployment frequency, lead time, change failure rate as first-class cells.

Landing (NTS-D09): DORA four keys join the §9 dashboard now (all system-derived,
no surveys); SPACE dims deferred until DORA stable (consensus sequencing);
metrics at team/system level, never individual; AI-amplifier note (2025 report):
strong foundations get stronger — measure before spreading agent tooling.

### 2.3 Audit tooling (S-31–S-32)

Local baseline: rg-based layer script (12 violation groups), doc-drift script (111),
 hand-drawn D-charts; MISSING: orphan detection, machine-checked acyclicity,
 volatility (frequently-changed modules) signal.

Landing (NTS-B13): adopt `orphans` (unlinked files) + `--acyclic` as G4 gates
(tool-agnostic: cargo-modules primary, cargoscope fast path);
volatility report feeds P2 split decisions (change hotspots first).

---

## 3. Sanctioned Exception Record (audit finding, SIM-24)

`crates/neotrix-sysctl`: REAL `unsafe { libc::… }` (macOS sysctl FFI), crate-level
`#![allow(unsafe_code)]` WITH written justification header (FFI isolation layer,
safe interface outward, neotrix-core keeps forbid). Verdict: SANCTIONED, permanent
(platform FFI cannot expire) with annual re-ratification. Recorded in NT-STD
Annex B (first SANCTIONED row). Rest of first-party tree: zero real unsafe
(all other hits are test fixtures / scanner patterns / format strings) — R-P1 holds.

---

## 4. Full-Domain Audit Snapshot (2026-09-21, machine-measured)

| Layer | Files | Lines | Note |
|-------|-------|-------|------|
| L0 substrate | 46 | 19,211 | smallest, zero-dep |
| L1 action | 445 | 144,573 | |
| L2 perception | 339 | 91,180 | |
| L3 embodiment | 247→259 | 72,774 | shield-heavy |
| L4 emotion | 218 | 83,538 | |
| L5 cognition | 660 | 233,245 | largest module tree |
| L6 meta | 224 | 74,409 | |
| **neotrix-core/src total** | **2,356 .rs** | **~719k** | +9 workspace crates, 26 integration test files |
| Layer violations | 12 groups | | check-layer-deps.sh (ratchet → 0) |
| Doc drift | 111 files | | unchanged since baseline — no progress, flagged |
| unsafe (first-party) | 1 sanctioned crate | | sysctl FFI (this §3); core 0 |
| TODO markers | scattered singles | | agent.rs 5, main.rs 1, misc 2s — no action |

---

*End of Absorption Round 5*

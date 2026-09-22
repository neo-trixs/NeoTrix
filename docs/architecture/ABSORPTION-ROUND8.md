# NeoTrix Absorption Round 8: Policy / Durability / WASM

> **Absorption Date**: 2026-09-21 | **SIM**: SIM-38 | **Status**: Landed
> **Rounds**: R1 graphify · R2 fitness/verifier/supply/ADR · R3 routing/OTEL/memory/fuzz
> | R4 interop/eval/release/prompts · R5 adversarial/DORA/arch-tooling |
> R6 API/Diátaxis/judge/chaos · R7 flags/signing/behavior |
> **This round: NTS-F10 (policy-as-code) + NTS-E11 (wasm hardening)**.

---

## 1. Source Map

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-44 | Cedar (AWS/CNCF; Rust crates; MongoDB Atlas 2025; 42–60x vs Rego; formal verification) | Language + engine | principal/action/resource/context; explicit permit/forbid; schema-first |
| S-45 | Temporal/DBOS durable execution (established knowledge — live search transport-failed, noted) | Pattern | Event-sourced durable steps; crash-resume for long agent loops |
| S-46 | WASM Component Model (WIT IDL; Bytecode Alliance) + noorle production patterns | Standard + examples | capability sandbox; language-agnostic plugin interfaces; cargo-component flow |
| S-47 | Safeguard WASM plugin checklist (2026-08) + tartanllama native-plugin post | Checklists | deny-default store; fuel/epoch traps; provenance; hostile-plugin tests |

Rejected: managed policy service (no new vendor); native `.so` plugins as default path
(deny-by-default until proven); full SLSA retained at P3 (R2 decision stands).

---

## 2. Pattern → NeoTrix Mapping (with local baselines)

### 2.1 Policy-as-code (S-44)

Local baseline: 935-line `nt_core_policy.rs` (L5) — single file, mechanism TBD by reading;
no declarative policy artifact found.

Landing (NTS-F10): authorization logic migrates to declarative
principal/action/resource/context policies, separate from application code;
default deny; schema-validated; git-versioned and reviewed. Applies first to the
SDB Verifier column (policies ARE the deterministic checks) and to governance
compliance checks. Cedar-the-crate is an adoption candidate (Rust-native), not
a decision — decision needs its own SIM with dependency review (R-P248 lens).

### 2.2 Durability (S-45)

Local baseline: outbox pattern exists in hive paths (SDB C-column); no crash-resume
for long loops; `run_bounded` precedent exists (behavioral_verifier) for timeout-kill.

Landing (bake note, no clause): durable steps = event-sourced outbox + replay on
restart; every irreversibility checkpointed before execution (already NTS-B03 spirit);
long agent loops get resume tokens. Full Temporal/DBOS-shaped runtime is P3+;
the pattern lands now, the platform later.

### 2.3 WASM hardening (S-46–S-47)

Local baseline (measured): wasmtime `Engine::default()` in `nt_io_plugin/wasm.rs`;
registry loads `.wasm` AND native `.so`/`.dll`/`.dylib`; **zero** hardening found
(no StoreLimits/fuel/WasiCtx/provenance/cosign).

Landing (NTS-E11): deny-by-default store; fuel/epoch limits + explicit trap test
(the classic misconfiguration is enabling `consume_fuel` without `set_fuel`);
signed bundles + content-hash pinning; host-call logging; capability diff review;
hostile-plugin suite in CI. Native `.so` path: deny-by-default until a dedicated
SIM proves otherwise — this is the single highest-leverage line in the clause.

---

## 3. Landings (this round)

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R8-1 | NTS-F10/E11 in NT-STD 1.0.6 | clauses present with Verify |
| L-R8-2 | SIM-38 record | row + §37 present |
| L-R8-3 | BLUEPRINT v1.6.3 changelog | line present |

Deferred (recorded): Cedar crate adoption (needs dependency-review SIM);
durable runtime platform (P3+); WASM hardening code (P-task, L1/L3 owners);
native-plugin verdict (dedicated SIM or it stays denied).

---

*End of Absorption Round 8*

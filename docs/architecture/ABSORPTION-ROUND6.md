# NeoTrix Absorption Round 6: API Governance / Docs Architecture / Judge Debias / Chaos

> **Absorption Date**: 2026-09-21 | **SIM**: SIM-25 | **Status**: Landed
> **Rounds**: R1 graphify · R2 fitness/verifier/supply/ADR · R3 routing/OTEL/memory/fuzz
> | R4 interop/eval/release/prompts · R5 adversarial/DORA/arch-tooling |
> **This round: NTS-D10 (API), NTS-F09 (judge), NTS-E10 (fault injection)** + Diátaxis MAP.

---

## 1. Source Map

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-33 | oasdiff (CLI + Action + Pro) + BREAKING-CHANGES taxonomy | Tool + docs | ERR/WARN/INFO gates, fail-on, label bypass, severity config |
| S-34 | Schemathesis (Hypothesis property-based; 4 checks; stateful links; JUnit) | Tool + guide | not_a_server_error, schema/content-type/status conformance |
| S-35 | Symfony/OpenAPI pipeline (spec export → Schemathesis → oasdiff → Prism mock) | Industry guide | The 4-stage reference pipeline; empty-baseline guard |
| S-36 | Diátaxis (map/compass/workflow; Canonical; Gatsby precedent) | Framework | 4 quadrants; compass test; never build empty quadrants |
| S-37 | LLM-judge bias literature (established knowledge — live search 429, noted) | Knowledge | Position/length/self-preference biases + mitigations |
| S-38 | Toxiproxy (Shopify, 2014–; toxics; /populate + /reset discipline) | Tool + guides | latency/timeout/reset_peer; per-test teardown; CI service job |

Rejected: LibAFL/AFL++ (standing rejection); full SLSA (still P3); Pro-only oasdiff
approve-gates (OSS `fail-on` + label bypass suffices — no new vendor token).

---

## 2. Pattern → NeoTrix Mapping (with local baselines)

### 2.1 API contract governance (S-33–S-35)

Local baseline (measured): 19 Tauri commands + 267 Axum `.route(` hits (ceiling),
**0 OpenAPI spec**. Linting one document cannot catch wire breaks; diffing two can —
but there is no first document yet.

Landing (NTS-D10, bake plan, P-task): export spec (step 0) → Schemathesis 4 checks
against test server (JUnit gate) → oasdiff `breaking --fail-on ERR` vs base branch
(empty-baseline guard for brand-new specs) → `breaking-change-approved` label bypass
with report-only mode (human decision stays in audit trail). Spec drift without a
gate is how silent consumer outages are born (api-contract-testing 2026).

### 2.2 Docs architecture (S-36)

Local baseline: 70 files under docs/, no quadrant structure; DOCUMENTATION-MAP owns layout.

Landing (MAP §Diátaxis, this round): classify by compass (action/cognition ×
acquisition/application), never build empty quadrants (Diátaxis workflow warning);
tutorials hold the hand, how-tos assume competence, reference mirrors the machinery,
explanation answers why. Existing files mapped in place — structure grows from inside.

### 2.3 Judge debias (S-37)

Local baseline: `llm_judge.rs` Criterion/weight framework, zero bias handling (measured).

Landing (NTS-F09): position bias → swap order + average both runs; length bias →
length-normalize or cap verbosity scoring; self-preference → blind model identity
where feasible; all mitigations + fixed judge model (R-P255 carryover); scores stay
relative signals, human-spot-checked before thresholds bite.

### 2.4 Fault injection (S-38)

Local baseline: zero chaos refs; breakers exist unproven under black-hole sockets.

Landing (NTS-E10): Toxiproxy per-test pattern (populate once, toxic per test,
`/reset` unconditional teardown); first faults: `timeout=0` black-hole (finds missing
read timeouts) + `latency`+jitter (finds pool exhaustion) + disabled-proxy down
(refused vs silent-hang are different bugs — test both); CI service-container job
in P3. Mocks prove logic; toxics prove resilience — run both, confuse neither.

---

## 3. Landings (this round)

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R6-1 | scripts/check-api-surface.sh (baseline 19/267/0) | script runs, numbers match probe |
| L-R6-2 | DOCUMENTATION-MAP Diátaxis section | section present, files mapped |
| L-R6-3 | NTS-D10/F09/E10 in dev NT-STD 1.0.4 | clauses present with Verify |
| L-R6-4 | SIM-25 record | row + §24 present |
| L-R6-5 | BLUEPRINT v1.5.0 changelog | line present |

Deferred (recorded): spec export + oasdiff/schemathesis CI (P-task, needs the spec);
judge bias code (P2 with eval gates); toxiproxy suite (P3).

---

*End of Absorption Round 6*

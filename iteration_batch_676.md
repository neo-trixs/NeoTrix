# Iteration Batch 676 — Chaos Engineering, Fault Injection & Resilience Testing Survey

**Date**: 2026-09-06
**Previous Batch**: 675 (profiling blind spots: no profiling profile, no allocation profiling, no CI benchmark regression, async profiling blind spot, no flame graph CI regression)
**Sources**: 15+ sources from 2026 chaos engineering/fault injection/resilience testing literature

---

## 1. CHAOS ENGINEERING — 2026 Landscape

### Sources
- Pi Stack: "Best Self-Hosted Chaos Engineering Platforms: Litmus vs Chaos Mesh vs Chaos Toolkit 2026" (2026-04-16)
- CNCF Blog: "LitmusChaos Q1-Q2 2026 update" (2026-08-06)
- Youngju Kim: "Practical Guide to Chaos Engineering: Fault Injection and Resilience Validation in Kubernetes with Litmus and Chaos Mesh" (2026-03-09)
- Reintech: "LitmusChaos vs Chaos Mesh: Which Kubernetes Chaos Tool to Choose in 2026" (2026-04-13)
- wg/all: "Chaos Engineering 2026: Building Production Resilience with Litmus, Gremlin & Chaos Mesh" (2026-03-15)
- core.cz: "Chaos Engineering in Practice: How to Test System Resilience" (2026)

### Key 2026 Developments
1. **Chaos Mesh v2.7.0** — Added PhysicalMachineChaos for bare-metal/VM faults, HTTPChaos for service-mesh level abort/delay/replace, DNSChaos for DNS error simulation
2. **LitmusChaos v3.0.0** — CNCF graduated, ChaosHub with 50+ experiments, multi-cluster support via Litmus Portal, Prometheus integration for metrics scraping during experiments
3. **Chaos Toolkit** — API-first, platform-agnostic, pip-installable, no Kubernetes requirement, Python-native
4. **SLO-Based Steady State Validation** — Moving from ad-hoc "is the system up?" to formal SLO metrics as steady-state hypothesis with LitmusProbes and Chaos Mesh StatusCheck
5. **GameDay Maturity** — Formalized 3-phase process: Preparation (1-2 weeks) → Execution (day-of) → Post-analysis (within 1 week)
6. **Recovery Validation** — Industry now recognizes that "surviving a failure but never recovering" is worse than the failure itself; recovery validation is now a mandatory test phase

---

## 2. FAULT INJECTION — 2026 Patterns

### Sources
- TotalShiftLeft: "Fault Injection Testing Explained: Build Resilience (2026)" (2026-02-14)
- OneUptime: "How to Create Fault Injection Testing" (2026-01-30)
- Logiciel.io: "Fault Injection Testing for Technology & SaaS" (2026-08-05)
- ITU Online: "Mastering Fault Injection Testing in AWS Cloud" (2026-07-14)
- Microsoft Engineering Playbook: Fault Injection Testing (2024, still authoritative)

### Key 2026 Patterns
1. **Three Failure Modes Per Dependency** — Unavailable, high latency (10x normal), error responses — minimum viable fault injection coverage
2. **Recovery Validation as First-Class** — Connection pool refill, circuit breaker close, queue drain, steady-state return — all must be verified post-fault
3. **Fault-Error-Failure Cycle** — Formal model: fault → error state → propagation → observable failure; each new error acts as a fault (cascade)
4. **Load + Fault Injection Combination** — Circuit breakers working at 10 RPS may fail at 10,000 RPS; combined testing is now standard
5. **AI-Assisted Fault Scenario Generation** — AI generates checks, identifies scenarios, summarizes evidence; humans still verify
6. **Observability Requirements** — Every fault injection test requires distributed traces, service-level metrics, application logs, and alert verification
7. **40-60% Fewer Resilience Incidents** — Teams with mature fault injection practices see 40-60% reduction in production resilience incidents

---

## 3. RESILIENCE TESTING — GameDay & Chaos Monkey 2026

### Sources
- Youngju Kim: "Chaos Engineering Complete Guide 2025: Resilience Testing, Chaos Monkey, Litmus, Game Day" (2026-04-14)
- techinterview.org: "System Design: Chaos Engineering — Netflix Chaos Monkey, Fault Injection, Game Days" (2026-04-20)
- Reintech: "Chaos Engineering Game Days: Planning and Execution Guide" (2026-05-01)
- OneUptime: "How to Implement Chaos Engineering Game Days" (2026-01-28)
- Zylos Research: "Chaos Engineering: Building Resilience Through Controlled Failure" (2026-02-12)
- Virtualization Review: "Cloud Resiliency Expert Dives Deep into Chaos Engineering" (2026-05-28)

### Key 2026 Patterns
1. **4-Level Maturity Progression** — Level 1: manual experiments → Level 2: planned game days → Level 3: automated CI/CD chaos → Level 4: continuous production chaos (Netflix model)
2. **Red Team vs Blue Team Game Days** — Red team injects failures secretly, blue team responds; tests monitoring and detection, not just resilience
3. **Cross-Team Game Days** — Backend, frontend, infrastructure, support teams coordinate during failure; tests organizational, not just technical, resilience
4. **Adaptive Blast Radius** — ML models predict safe experiment scope based on historical outcomes
5. **Automated Hypothesis Generation** — AI analyzes system topology and suggests relevant experiments
6. **Experiment Telemetry Tagging** — Alerts from chaos tests must be distinguishable from unrelated anomalies
7. **Kill Switch & Incident Command Authority** — Someone must have explicit power to terminate experiments and redirect responders if real incidents occur
8. **"Never inject failure into a system you can't observe"** — Observability (centralized logging, tracing, metrics, dependency mapping, alert correlation) is a hard prerequisite

---

## 4. NEW DEFECTS & IMPROVEMENTS FOR NEOTRIX

### Defect D676.1: No Chaos Engineering Integration — NT-REPAIR Cannot Inject Controlled Faults
**Severity**: HIGH
**Source**: Pi Stack 2026, LitmusChaos 2026, Chaos Mesh 2026
**Finding**: NT-REPAIR (self-healer) only detects and repairs degradation. It never injects controlled faults to verify its own repair logic works. The entire repair pathway is untested against realistic failure modes.
**Impact**: NT-REPAIR's repair patterns are theoretical — never validated against actual faults. A repair that works in unit tests may fail under real cascade conditions.
**Fix**: Implement `nt_repair::chaos_engine` — a controlled fault injection module that:
- Injects network latency between modules (simulating degraded IPC)
- Triggers memory pressure (simulating OOM conditions)
- Simulates KB write failures (testing repair from persistence failure)
- Validates repair completes and system returns to steady state
**Constellation**: C0 → target C2

### Defect D676.2: No Steady-State Hypothesis for ConsciousnessTree Growth Cycles
**Severity**: HIGH
**Source**: Youngju Kim 2026 (SLO-based steady state validation), Principles of Chaos
**Finding**: `neotrix-core_consciousness_tick` runs growth cycles (Soil→Roots→Trunk→Branches→Fruits→Core) but has no formal "steady state hypothesis" — no definition of what "healthy" looks like during a cycle. No SLO metrics define whether a cycle succeeded.
**Impact**: Growth cycles may produce degraded outputs (low phi, poor coherence, fragmented branches) without any mechanism detecting the regression.
**Fix**: Define `SteadyStateHypothesis` per growth cycle phase:
- Soil: KB write latency < 50ms, no corruption errors
- Roots: Module registration completes, no orphan modules
- Trunk: Cross-domain health scores within bounds
- Branches: Skill tree mutations within blast radius (no more than 2 nodes per cycle)
- Fruits: Evolution果实 quality score > threshold
- Core: Phi value stable or improving
**Constellation**: C0 → target C1

### Defect D676.3: No Recovery Validation After SEAL Pipeline Faults
**Severity**: HIGH
**Source**: TotalShiftLeft 2026 (recovery validation as first-class), Logiciel.io 2026
**Finding**: SEAL pipeline (exploration→distillation→self-test→absorption) has no recovery validation. If a SEAL stage fails mid-pipeline, there's no verification that the system returns to a valid state before the next cycle.
**Impact**: A failed distillation stage may leave partial artifacts that corrupt subsequent absorption stages, causing silent knowledge degradation.
**Fix**: Implement `seal::recovery_validator` — after each SEAL phase failure:
1. Verify no partial artifacts persist in KB
2. Verify pipeline state machine can reset to Phase-0
3. Verify module health scores return to pre-pipeline values
4. Log recovery time (MTTR metric for SEAL pipeline)
**Constellation**: C0 → target C1

### Defect D676.4: No GameDay Simulation for Cross-Domain Cascade Failures
**Severity**: MEDIUM
**Source**: techinterview.org 2026 (GameDay structure), Reintech 2026 (cross-team game days)
**Finding**: NeoTrix has 9+ domains (NT-CORE through NT-FEEL) with complex interdependencies, but no GameDay-style simulation tests cross-domain cascade failures. Each domain's SelfTest runs in isolation.
**Impact**: A failure in NT-WORLD (perception) cascading to NT-CORE (consciousness) and then NT-ACT (action) is never tested. Silent cascade paths remain undiscovered.
**Fix**: Implement `nt_meta::gameday_simulator`:
- Define cascade failure scenarios (e.g., NT-WORLD fetch timeout → NT-CORE phi degradation → NT-ACT decision paralysis)
- Inject fault in source domain, verify downstream domains handle gracefully
- Track cascade depth (how many domains affected before containment)
- Run as part of CI/nightly build
**Constellation**: C0 → target C1

### Defect D676.5: No Chaos Experiments in CI/CD Pipeline
**Severity**: MEDIUM
**Source**: wg/all 2026 (automating chaos engineering), OneUptime 2026 (Level 3 maturity)
**Finding**: NeoTrix CI pipeline runs `cargo test` and `cargo check` but never injects controlled faults. There's no Level 3 maturity (automated chaos in CI/CD). All resilience testing is manual or non-existent.
**Impact**: Fault handling code paths are only exercised by happy-path tests. Regression in error handling goes undetected until production incident.
**Fix**: Add chaos experiments to CI:
- `cargo test --features chaos` — inject faults during unit/integration tests
- Kill random module threads during test execution
- Inject KB write failures during test runs
- Verify all tests still pass (or fail gracefully) under fault conditions
- Gate PR merge on chaos experiment pass rate
**Constellation**: C0 → target C1

### Defect D676.6: No Observability Prerequisite Validation for Chaos
**Severity**: MEDIUM
**Source**: Virtualization Review 2026 ("never inject failure into a system you can't observe")
**Finding**: Before any chaos injection, NeoTrix should verify observability infrastructure is operational. There's no check that logging, tracing, metrics, and alerting are functional before running experiments.
**Impact**: Chaos experiments run without observability produce no useful signal — you can't learn from failures you can't see.
**Fix**: Implement `nt_shield::chaos_prereq_check`:
- Verify EventBus is receiving events
- Verify KB metrics are being recorded
- Verify HeartbeatAggregator is producing snapshots
- Verify log output is being captured
- Block chaos experiments if any observability channel is down
**Constellation**: C0 → target C1

### Defect D676.7: No Blast Radius Control for Module-Level Fault Injection
**Severity**: MEDIUM
**Source**: Zylos Research 2026 (adaptive blast radius), core.cz 2026 (minimize blast radius)
**Finding**: If fault injection is implemented, there's no mechanism to limit blast radius — no kill switch, no abort criteria, no incident command authority for terminating experiments.
**Impact**: A poorly scoped fault injection could cascade into real system degradation with no way to stop it.
**Fix**: Implement `nt_shield::chaos_safety`:
- Blast radius limit: max N modules affected per experiment
- Abort criteria: auto-terminate if system health drops below threshold
- Kill switch: manual override to immediately halt all chaos experiments
- Cooldown: minimum time between experiments on same module
- Audit log: every injection and its outcome recorded
**Constellation**: C0 → target C1

### Defect D676.8: No MTTR Tracking for Self-Healing Repair Cycles
**Severity**: LOW
**Source**: TotalShiftLeft 2026 (MTTR metric), OneUptime 2026 (MTTR measurement)
**Finding**: NT-REPAIR performs self-healing but never measures Mean Time To Recovery (MTTR). Without MTTR, there's no way to track whether self-healing is improving over time.
**Impact**: Self-healing may be getting slower or less effective with no detection mechanism.
**Fix**: Add MTTR measurement to NT-REPAIR:
- Record timestamp when degradation is detected
- Record timestamp when repair completes and system returns to healthy
- Store MTTR per module per repair type
- Track MTTR trend over time (should be decreasing)
- Alert if MTTR exceeds SLA
**Constellation**: C0 → target C1

---

## 5. SUMMARY — What's NEW

| # | Defect | Severity | Domain | Fix Module |
|---|--------|----------|--------|------------|
| D676.1 | No chaos engineering integration — NT-REPAIR can't inject faults | HIGH | NT-REPAIR | `chaos_engine` |
| D676.2 | No steady-state hypothesis for growth cycles | HIGH | NT-CORE | `SteadyStateHypothesis` |
| D676.3 | No recovery validation after SEAL pipeline faults | HIGH | NT-MIND | `recovery_validator` |
| D676.4 | No GameDay simulation for cross-domain cascades | MEDIUM | NT-META | `gameday_simulator` |
| D676.5 | No chaos experiments in CI/CD pipeline | MEDIUM | CI/CD | `chaos` feature flag |
| D676.6 | No observability prerequisite validation | MEDIUM | NT-SHIELD | `chaos_prereq_check` |
| D676.7 | No blast radius control for fault injection | MEDIUM | NT-SHIELD | `chaos_safety` |
| D676.8 | No MTTR tracking for self-healing | LOW | NT-REPAIR | MTTR measurement |

**Total new defects**: 8
**HIGH**: 3 | **MEDIUM**: 4 | **LOW**: 1
**Domains affected**: NT-REPAIR, NT-CORE, NT-MIND, NT-META, NT-SHIELD, CI/CD

---

## 6. CROSS-BATCH CONTINUITY

**From Batch 675** (profiling blind spots):
- D675.1: No profiling profile → D676.5 adds chaos experiments to CI (complementary: profiling finds performance regressions, chaos finds resilience regressions)
- D675.2: No allocation profiling → D676.1 fault injection tests memory pressure scenarios
- D675.3: No CI benchmark regression → D676.5 chaos in CI/CD is the resilience analog of benchmark regression
- D675.4: Async profiling blind spot → D676.3 SEAL pipeline recovery validation covers async failure modes
- D675.5: No flame graph CI regression → D676.4 GameDay cascade simulation visualizes failure propagation paths

**Cumulative defect count (batches 671-676)**: 48 defects across 15+ research iterations

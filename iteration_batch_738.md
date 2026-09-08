# Iteration Batch 738 — Production/Operational/Launch Readiness Defects

**Date**: 2026-09-07
**Prior Batch**: 737 (WCAG conformance, accessibility bridge, AT integration, WCAG 3.0 scoring)
**Research Domain**: Production readiness, operational readiness, launch checklists, AI agent deployment governance

---

## Sources Consulted

| # | Source | Date | Key Insight |
|---|--------|------|-------------|
| 1 | Cortex — Production Readiness Review Checklist & Best Practices | 2026-01-14 | PRR as formalized gate; 98% of leaders report major fallout from unprepared launches; 66% cite inconsistent standards across teams |
| 2 | AI Agent Production Go-Live Checklist 2026 (RockB) | 2026-06-20 | 45 checks across 6 domains; 88% of AI agent pilots never make production; three-tier scoring (Blocking/Risk-Reducing/Maturity-Building) |
| 3 | Stop Shipping AI Agents on Faith (arXiv:2607.27677) | 2026-07-30 | ProofAgent Index (PAI); capability ≠ readiness; four dimensions (Eval/Context/Compliance/Governance); AUC=0.98 for held-out failure prediction |
| 4 | Kuberstar — Production Readiness Checklist 2026 | 2026-06-13 | "Service is production-ready when an on-call engineer who did not build it can tell that it broke, find out why, and fix it — without calling the author at 2 a.m." |
| 5 | eDarpan — Production Readiness Checklist 2026 | 2026-05-26 | Rollback in ≤2 commands; DB migration forward-and-backward safe; pre-launch dry run of error/alert/backup/rollback |
| 6 | LLM Guardrails Deployment 2026 (Future AGI) | 2026-05-14 | Four placement points for guardrails; input-only defenders lose; prompt-level guardrails trivially bypassed |
| 7 | Tauri Release Checklist (techxcelerate) | 2026 | Every PE file must be signed; unsigned installers are not production artifacts; versioned immutable URLs required |
| 8 | AI Agent Safety Checklist (eSecurity Planet) | 2026-03-12 | Feature flags for controlled rollout; kill switch with version tracking; use-case guardrails for approved vs. restricted |
| 9 | Kanerika — Software Product Launch Checklist | 2026-07-29 | Launch = controlled production change, not marketing milestone; pre-agreed rollback triggers; named incident commander |
| 10 | CheckOff.ai — Complete Production Readiness Checklist | 2026-05-07 | Named owner + defined SLO + monitoring + runbook + on-call rotation + security posture = minimum viable production readiness |

---

## Defects Found

### DEFECT 738.1: No Formal Production Readiness Review (PRR) Process

**Severity**: Blocking
**Domain**: NT-CORE / NT-META

**Evidence**: Cortex (2026-01-14) reports 98% of engineering leaders experienced major fallout from launching unprepared services. Google SRE's PRR model defines a structured 4-stage gate: Engagement → Analysis → Training → Handoff. Kuberstar (2026-06-13) defines production readiness as: "the service can be operated — observed, scaled, and recovered — by an on-call engineer who did not write it."

**NeoTrix Gap**: NeoTrix has no PRR process for its 9+ domains. The Constellation maturity model (C0-C6) measures compilation/test/benchmark/integration status but does NOT validate operational readiness (monitoring, on-call, runbooks, rollback, security scans). A module can be C4 (integrated into pipeline) and still lack operational readiness.

**Defect**: C0-C6 constellation model conflates technical maturity with operational readiness. Missing PRR gate that validates observability, on-call coverage, security scans, deployment pipeline testing, and documentation completeness before any module reaches C4+.

**Fix Required**: Define a PRR gate as a prerequisite for C4 constellation promotion. PRR must validate: (1) monitoring/alerting in place, (2) on-call rotation assigned, (3) security scans passing, (4) deployment pipeline tested, (5) runbook exists, (6) rollback tested.

---

### DEFECT 738.2: No Agent Decision Trajectory Logging

**Severity**: Blocking
**Domain**: NT-MEMORY / NT-ACT

**Evidence**: RockB (2026-06-20) defines "decision trajectory logging" as the single highest-impact observability pattern for production agents: "You need to know not just what the agent output, but what path it took to get there — which tools it called in which order, why it chose each one (the model's chain-of-thought), and where it deviated from the expected path." Check #9 in the 45-check checklist. Every production incident traces to missing trajectory logging or missing input guardrails.

**NeoTrix Gap**: NT-ACT's EventBus logs events but not decision trajectories. The consciousness core (`nt_core_consciousness`) processes tasks but does not emit structured trajectory events (tool calls, reasoning chains, deviation markers). When an agent sub-task fails, there is no structured record of the decision path that led to the failure.

**Defect**: No structured decision trajectory events in EventBus. Agent task processing does not emit per-step tool-call sequences, reasoning chain snapshots, or deviation markers. Debugging agent failures requires re-running the task, not replaying the trajectory.

**Fix Required**: Define a `DecisionTrajectory` event type in EventBus containing: (1) step sequence, (2) tool-call records (timestamp, tool_id, input_hash, output_summary, latency), (3) reasoning snapshot per step, (4) deviation markers when trajectory diverges from expected path. Wire into `nt_core_consciousness::consciousness_task`.

---

### DEFECT 738.3: No Three-Layer Guardrail Architecture (Input → Output → Action)

**Severity**: Blocking
**Domain**: NT-SHIELD / NT-CORE

**Evidence**: RockB (2026-06-20) defines three guardrail layers as the "single highest-impact pattern for production safety": (1) Input guardrail — scans for prompt injection and PII leakage, (2) Output guardrail — validates against schema and business rules, (3) Action guardrail — requires confirmation before irreversible writes. Must be implemented as middleware, not prompt instructions. Future AGI (2026-05-14) confirms four placement points; input-only defenders lose. arXiv:2607.27677 (PAI) shows missing compliance evidence triggers hard blocks.

**NeoTrix Gap**: NeoTrix has `egress_privacy_guard` (outbound filtering only) and `nt_shield_sandbox` (egress policy). There is NO inbound input guardrail (prompt injection detection, PII scanning), NO output guardrail (schema validation, business rule enforcement), and NO action guardrail (confirmation before irreversible writes). The egress-only model protects against data leakage but not against prompt injection attacks on the agent itself.

**Defect**: Guardrail architecture is egress-only. Missing: (1) Input layer — prompt injection scanner + PII leakage detector, (2) Output layer — schema validator + business rule enforcer, (3) Action layer — irreversible write confirmation gate. Agent is vulnerable to prompt injection that bypasses all existing protections.

**Fix Required**: Implement three-layer guardrail middleware stack: `nt_shield_guardrails` crate with InputGuardrail, OutputGuardrail, ActionGuardrail traits. Wire into agent task processing pipeline between EventBus receipt and tool execution. Each layer must be configurable per-tool and emit structured audit events.

---

### DEFECT 738.4: No Kill Switch with Sub-60-Second Execution

**Severity**: Blocking
**Domain**: NT-ACT / NT-SHIELD

**Evidence**: RockB (2026-06-20) Check #36: "Kill switch that pauses all agent execution in under 60 seconds." Scored 0-2 in Risk-Reducing tier. Practical implementation requires: (1) infrastructure rollback — 30 seconds, (2) configuration rollback via feature flag — 10 seconds, (3) full stop kill switch — 5 seconds. Must be "a single button in your monitoring dashboard with a pre-authenticated session." eSecurity Planet (2026-03-12): "Organizations need the ability to shut down AI capabilities quickly if unexpected behavior occurs."

**NeoTrix Gap**: No documented kill switch. NT-ACT can be configured to disable tool permissions via capability-based security, but there is no single-command full-stop mechanism. Stopping all agent execution requires manually revoking capabilities across multiple modules — no pre-authenticated single-command path exists.

**Defect**: No kill switch. Stopping agent execution requires manual multi-step capability revocation. No dashboard-level emergency stop. No pre-authenticated path. Time to full stop is unmeasured and likely exceeds 60 seconds.

**Fix Required**: Implement `nt_shield::KillSwitch` trait with: (1) single-command full-stop via EventBus broadcast, (2) feature-flag-based graceful degradation, (3) pre-authenticated dashboard button, (4) measured execution time < 60 seconds. Test under timer before production readiness.

---

### DEFECT 738.5: No Token Budget / Cost Control Circuit Breaker

**Severity**: High
**Domain**: NT-ACT / NT-MIND

**Evidence**: RockB (2026-06-20) Domain 4: "The same agentic coding task can vary 30× in token consumption with zero correlation to output quality. Cost controls are not an optimization — they are a safety mechanism." Three-level token bucket required: per-request, per-user, global. Salim et al. "Tokenomics" (arXiv:2601.14470): coordinator agents consume 40-60% of total tokens. Auto-rollback trigger on per-request cost spike > 20%.

**NeoTrix Gap**: NT-ACT has `ResourceBudgetManager` (formerly CostManager) for cost estimation, but no runtime token bucket enforcement. No per-request hard stop. No per-user daily budget. No global daily budget with hard stop. No loop detection (error after N identical tool calls). No auto-rollback on cost spike. The Egress Privacy Guard does not track token consumption.

**Defect**: `ResourceBudgetManager` is estimation-only, not enforcement. Missing: (1) per-request token budget with hard stop, (2) per-user daily budget, (3) global daily budget, (4) loop detection (3+ identical tool calls = error), (5) real-time cost dashboard, (6) auto-rollback on >20% cost spike.

**Fix Required**: Upgrade `ResourceBudgetManager` to enforce budgets at three levels. Add loop detection to tool-call middleware. Wire cost metrics to EventBus for real-time dashboard. Add auto-rollback trigger on cost anomaly.

---

### DEFECT 738.6: No Staged Autonomy Model for Agent Deployment

**Severity**: High
**Domain**: NT-ACT / NT-META

**Evidence**: RockB (2026-06-20) Domain 5: "You do not start with full autonomy. You start with 100% human review, measure the agent's false-positive rate and the human reviewer's override rate, and remove human review category by category over 2-4 weeks." Stack Archive research: staged autonomy achieves production deployment in 6-8 weeks vs. 14+ weeks with 60%+ revert rate for full-autonomy attempts.

**NeoTrix Gap**: NeoTrix's agent system (`nt_core_consciousness::consciousness_task`) runs with full autonomy from the start. No staged autonomy model. No human review tier for initial deployment. No per-category removal schedule. No false-positive/override rate tracking. The SEAL pipeline self-evolution loop does not differentiate between pilot and production autonomy levels.

**Defect**: Agent system deploys with full autonomy immediately. No staged autonomy phases defined. No human review tier for initial deployment. No metrics to measure when human review can be safely removed per category.

**Fix Required**: Define staged autonomy model with three phases: (1) Phase 1 — 100% human review for all irreversible actions, (2) Phase 2 — async approval for medium-risk actions, (3) Phase 3 — full autonomy for low-risk actions. Track false-positive and override rates to determine when to advance phases. Wire into SEAL pipeline as a deployment gate.

---

### DEFECT 738.7: No Module-Level Runbooks

**Severity**: High
**Domain**: All Domains

**Evidence**: Kuberstar (2026-06-13): "A service is production-ready when an on-call engineer who did not build it can tell that it broke, find out why, and fix it — without calling the author at 2 a.m." Cortex (2026-01-14): runbooks must include common failure scenarios, step-by-step remediation, debugging tips, and service context. CheckOff.ai (2026-05-07): runbook is a minimum viable production readiness requirement alongside SLO, on-call, and monitoring.

**NeoTrix Gap**: No per-module runbooks exist. The `AGENTS.md` contains architectural guidance but no operational runbooks. When a domain module (e.g., NT-WORLD crawler, NT-MEMORY KB) fails in production, there is no documented remediation path. Knowledge exists only in developer memory.

**Defect**: Zero runbooks for any module. No documented failure scenarios, remediation steps, debugging commands, or service context. On-call (which also doesn't exist — see 738.8) would require reverse-engineering the module from source code during an incident.

**Fix Required**: Create runbook template with sections: (1) Service context and dependencies, (2) Common failure scenarios with symptoms, (3) Step-by-step remediation, (4) Debugging commands and dashboard links, (5) Blast radius description. Create runbooks for all C3+ constellation modules. Store in `docs/runbooks/` alongside module code.

---

### DEFECT 738.8: No On-Call Rotation Defined

**Severity**: High
**Domain**: NT-META / Operations

**Evidence**: eDarpan (2026-05-26): "On-call rotation defined. Even if it is just two founders alternating weeks, write it down. Without a defined on-call, alerts go to nobody." Cortex (2026-01-14): on-call coverage with clear escalation paths is a prerequisite for production readiness. CheckOff.ai (2026-05-07): on-call rotation that can actually answer the page is minimum viable.

**NeoTrix Gap**: No on-call rotation defined. No escalation path. No alert routing to specific people. The HeartbeatAggregator collects system health signals but does not route alerts to humans. Alerts go to the system, not to people.

**Defect**: No on-call rotation. No escalation policy. HeartbeatAggregator health signals are collected but not routed to any human responder. During an incident, there is no defined path to reach the right person.

**Fix Required**: Define on-call rotation (even if just two people alternating). Create escalation policy. Wire HeartbeatAggregator alerts to human notification channel (email/Slack/PagerDuty). Document escalation paths per module severity level.

---

### DEFECT 738.9: No Defined SLOs for Any Module

**Severity**: High
**Domain**: All Domains

**Evidence**: Cortex (2026-01-14): "SLOs are a non-negotiable part of readiness. Without them, teams have no shared definition of acceptable performance, and incidents become subjective debates." CheckOff.ai (2026-05-07): defined SLO is a minimum viability requirement alongside monitoring, runbook, on-call, and security. Google SRE: error budgets calculated from SLOs guide deployment decisions.

**NeoTrix Gap**: No Service Level Objectives defined for any module. No latency SLOs, no error rate SLOs, no availability SLOs. No error budgets calculated. No subjective performance measurement. The HeartbeatAggregator collects metrics but has no target thresholds to compare against.

**Defect**: Zero SLOs defined across all 9+ domains. HeartbeatAggregator metrics are collected but have no target thresholds. No error budgets. No objective definition of "working" vs. "degraded" vs. "failed." Incident response is ad-hoc with no quantitative basis.

**Fix Required**: Define SLOs for each C3+ module: (1) availability target (e.g., 99.9%), (2) latency target (e.g., p99 < 500ms for NT-ACT tool calls), (3) error rate target (e.g., < 0.1% for NT-MEMORY KB queries). Calculate error budgets. Wire SLO thresholds into HeartbeatAggregator for automated degradation detection.

---

### DEFECT 738.10: Capability ≠ Readiness — Constellation Model Misclassification

**Severity**: High
**Domain**: NT-CORE / NT-META

**Evidence**: arXiv:2607.27677 (PAI paper, 2026-07-30): "Capability measures what an agent can do under test. Readiness measures whether that agent can be safely, lawfully, and accountably deployed." Ablation analysis: Evaluation dimension alone achieves AUC=0.80 for failure prediction; full PAI with Context/Compliance/Governance achieves AUC=0.98. Key finding: "Context engineering strongly changes agent reliability, capability improves behavior but does not determine readiness by itself."

**NeoTrix Gap**: NeoTrix's C0-C6 constellation model measures technical maturity (compiles → unit tests → integration tests → benchmarked → integrated → self-healing). It does NOT separate capability from readiness. A C4 module (integrated into pipeline) could still lack: monitoring, on-call, runbooks, security scans, SLOs, guardrails, cost controls, or governance evidence. The constellation model conflates "the code works" with "the service is ready for production."

**Defect**: C0-C6 constellation model measures capability but not readiness. A module at C4 can be technically integrated yet operationally unready (no monitoring, no runbook, no SLO, no on-call, no guardrails). This creates a false sense of production readiness.

**Fix Required**: Extend constellation model with a parallel readiness dimension. Define readiness tiers separate from maturity tiers: R0 (no operational readiness), R1 (SLOs + monitoring defined), R2 (on-call + runbooks), R3 (guardrails + cost controls + PRR passed). A module must achieve both C4 AND R3 before production deployment. Update CONTEXT.md with new notation.

---

### DEFECT 738.11: No Pre-Launch Dry Run Protocol

**Severity**: Medium
**Domain**: NT-META / Operations

**Evidence**: eDarpan (2026-05-26): "The night before launch, run through the alarms-go-off scenarios: (1) Trigger an error and confirm it lands in your error tracker, (2) Trigger an alert and confirm someone actually gets paged, (3) Restore yesterday's backup to a clean environment, (4) Roll back the last deploy and verify the previous version comes up cleanly. If any of these fail, you are not ready to launch."

**NeoTrix Gap**: No pre-launch dry run protocol. No script to trigger test errors and verify alert delivery. No backup restoration test. No rollback verification test. The SEAL pipeline convergence_check validates structural integrity but not operational scenarios.

**Defect**: No pre-launch dry run. No automated test of: (1) error → alert pipeline, (2) backup restoration, (3) rollback execution, (4) alert delivery to humans. Launch readiness is assumed, not verified through scenario testing.

**Fix Required**: Create `nt_meta::dry_run` module implementing four pre-launch scenarios: (1) trigger deliberate error → verify error tracker receives it, (2) trigger alert → verify human gets paged, (3) restore KB backup → verify data integrity, (4) rollback last deploy → verify previous version comes up. Run as SEAL Phase-0 gate before any C4+ promotion.

---

### DEFECT 738.12: No Database Migration Safety Validation

**Severity**: Medium
**Domain**: NT-MEMORY

**Evidence**: eDarpan (2026-05-26): "Database migrations forward-and-backward safe. No migration that drops a column the previous version still reads. Multi-step migrations for non-trivial changes." This is a production readiness requirement, not a development convenience.

**NeoTrix Gap**: NT-MEMORY uses SQLite KB. Schema migrations exist (via KB versioning) but are not validated for forward-and-backward safety. A migration that drops a column could break the previous version on rollback. No migration testing in CI. No multi-step migration protocol for non-trivial changes.

**Defect**: KB schema migrations are not validated for forward-and-backward safety. Rollback could break on dropped columns. No CI validation of migration safety. No multi-step migration protocol.

**Fix Required**: Add migration safety validation to CI: (1) test that migration applies cleanly, (2) test that rollback reverts cleanly without data loss, (3) enforce multi-step migrations for column drops (add new → copy data → drop old in separate migration), (4) add to convergence_check as a structural gate.

---

### DEFECT 738.13: No Data Deletion Endpoint (GDPR/DPDP Compliance)

**Severity**: Medium
**Domain**: NT-MEMORY / NT-SHIELD

**Evidence**: eDarpan (2026-05-26): "Data deletion path. An actual implemented endpoint or process for 'delete my account and data'. DPDP and GDPR both expect this." This is a compliance readiness requirement.

**NeoTrix Gap**: NT-MEMORY stores user data in KB. No documented or implemented data deletion endpoint. No "delete my account and data" capability. KB has versioning but no user-scoped data purge mechanism. EU AI Act and GDPR compliance requires this.

**Defect**: No data deletion endpoint. User data in KB cannot be purged on request. Non-compliant with GDPR/DPDP right-to-erasure requirements. No audit trail for deletion requests.

**Fix Required**: Implement `nt_memory::data_deletion` endpoint: (1) accept user_id, (2) purge all KB nodes/edges/embeddings scoped to user, (3) log deletion event for audit, (4) confirm completion. Add to NT-SHIELD audit dimensions.

---

### DEFECT 738.14: No Canary Deployment with Auto-Rollback on Anomaly

**Severity**: Medium
**Domain**: NT-ACT / Operations

**Evidence**: RockB (2026-06-20) Domain 6: "Canary deployment strategy with graduated rollout (5% → 25% → 100%)." Auto-rollback trigger configured and tested. Dual monitoring: traditional metrics (latency, error rate, cost) AND agent-specific metrics (task success rate, tool-call accuracy, trajectory divergence). Kanerika (2026-07-29): "Pre-agreed rollback triggers (error-rate threshold, latency threshold, specific customer-impact reports) that don't require a fresh debate to invoke."

**NeoTrix Gap**: No canary deployment strategy. No graduated rollout mechanism. No auto-rollback on anomaly detection. The SEAL pipeline handles code evolution but not deployment traffic management. Feature flags exist via Tauri capability system but are not wired to traffic routing or auto-rollback.

**Defect**: No canary deployment. No graduated rollout. No auto-rollback on anomaly. Deployment is binary (all-or-nothing). No pre-agreed rollback triggers. No dual monitoring (traditional + agent-specific metrics).

**Fix Required**: Define canary deployment strategy for Tauri releases: (1) feature-flag-based gradual rollout, (2) auto-rollback triggers on error rate / latency / cost thresholds, (3) dual monitoring dashboards (operational + agent-specific), (4) pre-agreed rollback triggers documented in runbooks.

---

### DEFECT 738.15: Tauri Release Lacks PE File Signing Validation

**Severity**: Medium
**Domain**: NT-IO (Tauri Desktop)

**Evidence**: techxcelerate (2026): "Every Portable Executable file in the shipped payload must be signed. Unsigned installers are not production artifacts." Release checklist requires: main exe signed, all DLLs signed, all helper EXEs and sidecars signed, uninstaller signed, outer installer signed last. Signature verification must pass on every shipped PE file.

**NeoTrix Gap**: `neotrix-tauri` build process does not include automated PE file signing validation. No CI step to verify all DLLs, sidecars, and helper binaries are signed. No signature verification test on build artifacts. The Tauri build produces installers but does not validate that every PE in the payload is signed.

**Defect**: No automated PE file signing validation in Tauri build pipeline. Unsigned DLLs or sidecars could be shipped. No CI gate on signature verification. No clean-VM install test.

**Fix Required**: Add to Tauri CI pipeline: (1) enumerate all PE files in build output, (2) verify signature on each PE, (3) fail build if any unsigned PE detected, (4) test silent install on clean VM, (5) verify versioned immutable URL for installer.

---

### DEFECT 738.16: No Incident Commander Role Defined

**Severity**: Medium
**Domain**: NT-META / Operations

**Evidence**: Kanerika (2026-07-29): "A named incident commander with the authority to pause or roll back the rollout without waiting for a committee." Kanerika also defines: "A customer communication plan ready to go if the launch needs to be paused publicly, not drafted after the fact."

**NeoTrix Gap**: No incident commander role defined. No authority matrix for emergency actions. No customer communication plan. During an incident, decision-making is ad-hoc with no pre-assigned authority.

**Defect**: No incident commander role. No authority matrix. No customer communication plan. Emergency decisions require ad-hoc coordination with no pre-assigned decision-maker.

**Fix Required**: Define incident commander role with: (1) named person(s) with authority to pause/rollback, (2) authority matrix per action type, (3) customer communication template, (4) escalation timeline. Document in runbooks.

---

### DEFECT 738.17: No Continuous Readiness Monitoring (Post-Launch Drift Detection)

**Severity**: Medium
**Domain**: NT-META

**Evidence**: Cortex (2026-01-14): "32% of organizations lack a continuous readiness process, which means they're blind to drift until an incident forces them to notice. A service that was production ready three months ago might no longer be ready if ownership changed, dependencies were upgraded, or monitoring was accidentally removed during a refactor." Continuous readiness = periodic reviews + automated validation + trend tracking.

**NeoTrix Gap**: Constellation maturity (C0-C6) is set once and does not drift-detect. A module promoted to C4 is not re-evaluated for readiness unless manually reviewed. No continuous readiness monitoring. No automated re-validation. No trend tracking. No periodic review schedule.

**Defect**: Constellation maturity is static, not continuous. No automated drift detection. No periodic re-validation of readiness. No readiness trend tracking. A C4 module that loses monitoring or gains a dependency with a critical CVE is not detected until the next manual review (which may never happen).

**Fix Required**: Add continuous readiness monitoring: (1) periodic automated re-validation of readiness criteria, (2) drift detection on monitoring coverage, dependency freshness, on-call status, (3) readiness trend dashboard, (4) automated demotion from C4 if readiness criteria regresses.

---

### DEFECT 738.18: No Eval Suite for Agent Behavior Validation

**Severity**: Medium
**Domain**: NT-MIND / NT-ACT

**Evidence**: RockB (2026-06-20) Domain 3: "Minimum 20-30 representative eval test cases covering happy path, edge cases, and failure modes." "Offline eval runs against golden dataset before every deployment." "Regression gates block deploys when scores drop below threshold." LangChain 2026 survey: only 52% run any offline evaluations before shipping. arXiv:2607.27677: PAI Evaluation dimension measures task success, hallucination resistance, safety, instruction following, manipulation resistance, and tool use.

**NeoTrix Gap**: No formal eval suite for agent behavior. No golden dataset. No offline eval gate in CI. No regression gate on behavior scores. The SEAL pipeline runs self-tests (SelfTest trait) but these validate structural properties, not agent behavior under adversarial conditions.

**Defect**: No agent behavior eval suite. No golden dataset. No adversarial test cases. No offline eval gate. SelfTest validates structure, not behavior. Agent behavior quality is assessed informally, not through systematic evaluation.

**Fix Required**: Create `nt_mind::agent_eval` module: (1) define 25-35 eval test cases (golden path, edge cases, failure modes, adversarial), (2) golden dataset stored in KB, (3) offline eval gate in CI pipeline, (4) regression gate blocking deploys on score drop, (5) trajectory-level evaluation (not just output-level).

---

## Summary

| Severity | Count | Defects |
|----------|-------|---------|
| Blocking | 4 | 738.1 (No PRR), 738.2 (No Trajectory Logging), 738.3 (No 3-Layer Guardrails), 738.4 (No Kill Switch) |
| High | 6 | 738.5 (No Token Budgets), 738.6 (No Staged Autonomy), 738.7 (No Runbooks), 738.8 (No On-Call), 738.9 (No SLOs), 738.10 (Capability≠Readiness) |
| Medium | 8 | 738.11 (No Dry Run), 738.12 (Migration Safety), 738.13 (Data Deletion), 738.14 (No Canary), 738.15 (PE Signing), 738.16 (No Incident Commander), 738.17 (No Continuous Readiness), 738.18 (No Eval Suite) |
| **Total** | **18** | |

## What's NEW vs. Batch 737

Batch 737 focused on WCAG/accessibility gaps. Batch 738 shifts to **production and operational readiness** — the entire domain of "can this system be safely operated in production by someone who didn't build it." Every defect in this batch is orthogonal to the accessibility findings:

- 737.1-737.5: WCAG conformance, accessibility bridge, AI output audit, AT integration, WCAG 3.0 scoring
- 738.1-738.18: PRR process, trajectory logging, guardrail architecture, kill switch, token budgets, staged autonomy, runbooks, on-call, SLOs, capability≠readiness, dry runs, migration safety, data deletion, canary deployment, PE signing, incident commander, continuous readiness, eval suites

The two batches together cover the two critical production gaps: (1) the system is not accessible to users with disabilities, and (2) the system cannot be safely operated in production. Both must be resolved before any production deployment.

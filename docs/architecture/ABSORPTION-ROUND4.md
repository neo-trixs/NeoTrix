# NeoTrix Absorption Round 4: Interop / Eval / Release / Prompts

> **Absorption Date**: 2026-09-21 | **SIM**: SIM-12 | **Status**: Landed
> **Rounds**: R1 graphify (R-P230–240) | R2 fitness/verifier/supply/ADR (R-P246–249)
> | R3 routing/OTEL/memory/fuzz (R-P250–253) | **This round R-P254–257**.

---

## 1. Source Map

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-21 | A2A Protocol spec (Linux Foundation; a2aproject/A2A + google/A2A llms.txt) | Standard | AgentCard, Task lifecycle, streaming/push, bindings, security |
| S-22 | Google Developers Blog: Announcing A2A (2025-04, 50+ partners) | Announcement | A2A complements MCP (agents talk ↔ tools/context); HTTP/SSE/JSON-RPC stack |
| S-23 | DeepEval docs (metrics intro, RAGAS, answer-relevancy) + AgentsCamp/QASkills comparisons (2026) | Framework docs | Metric quartet, gate-vs-diagnose posture, trajectory metrics, calibration discipline |
| S-24 | cargo-dist book + repo (axodotdev, plan/build/host/publish) | Tool docs | Tag-driven releases, generated CI, dist-manifest, PR plan mode |
| S-25 | Orhun automated Rust releases (git-cliff + release-plz + dist + Dependabot) | Industry guide | Full pipeline wiring order, Unreleased-heading contract, role split |
| S-26 | git-cliff Cargo integration docs | Tool docs | `[package.metadata.git-cliff]` / workspace metadata config |
| S-27 | promptfoo (established knowledge — live search rate-limited 429, noted honestly) | Tool (unverified live) | Prompt test YAMLs in CI; superseded here by DeepEval-native gating as primary |

---

## 2. Pattern → NeoTrix Mapping (with local baselines)

### 2.1 A2A conformance ladder (S-21–S-22)

Local baseline (measured): `nt_core/capability/a2a.rs` HAS AgentCard struct + registry
(register/get/find_by_capability/find_by_skill) + A2ATask map + TaskState
(Completed/Failed/Canceled + transition + terminal check); MCP endpoints exist
(mcp_server, mcp_bridge, mcp_protocol/, shield MCP security).

Gap ladder (spec → status):

| Rung | Spec Requirement | Status |
|------|-----------------|--------|
| 1 AgentCard | identity/skills/endpoint/auth | PRESENT (extend: securitySchemes, supportedInterfaces, MIME modes) |
| 2 Task lifecycle | SUBMITTED/WORKING/COMPLETED/FAILED/CANCELED/**REJECTED**/INPUT_REQUIRED/AUTH_REQUIRED | PARTIAL (missing REJECTED + interrupted states) |
| 3 Identity rules | server-generated taskIds; no client-created IDs; TaskNotFoundError; contextId/taskId mismatch rejection | TO VERIFY (P-task) |
| 4 Async | streaming (message-only vs task-lifecycle), push webhooks, Get/List/Cancel/Subscribe ops | MISSING (P-task) |
| 5 Bindings | JSON-RPC + gRPC + REST identical semantics; multi-protocol fallback | MISSING (P-task) |
| 6 Posture | A2A = agent↔agent, MCP = agent↔tools (complementary, not rival) | ADOPT as doctrine |

### 2.2 Eval: quartet + gate posture (S-23)

Local baseline (measured): `nt_mind_eval_harness.rs` HAS golden_answers map +
DatasetSpec + ap_acc_score; `llm_judge.rs` HAS Criterion/weight/JudgeResult
(G-Eval-shaped); gateway `gate.rs` HAS FaithfulnessReport::audit(claims, evidence_ids).

Gap (DeepEval/RAGAS vocabulary → status):

| Metric | Evaluates | Needs Truth? | Status |
|--------|-----------|-------------|--------|
| Faithfulness | claims ⊆ context? | No | PRESENT (claims-vs-evidence audit) |
| Answer relevancy | output addresses input? | No | MISSING |
| Context precision/recall | retrieval right + ranked? | Recall: yes | MISSING |
| TaskCompletion/trajectory | whole-trace quality | Golden trace | MISSING |
| Threshold-as-gate | `assert_test` fails build | — | MISSING (scores exist, gates don't) |

Doctrine (AgentsCamp 2026): RAGAS-style metrics for retrieval tuning (diagnose),
DeepEval posture (`assert_test` in CI) for gating; reference-free on prod traffic,
reference-based on golden; judge model fixed across comparisons; scores are relative
signals, spot-checked vs human labels before trusting a threshold.

### 2.3 Release automation ladder (S-24–S-26)

Local baseline (measured): `release.yml` tag-driven + git-cliff changelog job +
build matrix exist; **defect found**: cliff `config:` pointed at repo-root
`cliff.toml`, real file lives at `config/cliff.toml` → changelog job would fail
on next tag push (fixed this round: L-R4-1).

Ladder (Orhun pipeline → status):

| Step | Tool | Status |
|------|------|--------|
| Changelog | git-cliff (+ `config/cliff.toml` path fix) | PRESENT (fixed) |
| Crate versions + crates.io | release-plz (Release PR, Unreleased-heading contract) | MISSING (P3 bake) |
| Binaries + installers + manifest | cargo-dist (`dist init` generates CI; PR plan mode) | MISSING (P3 bake) |
| Deps | Dependabot (cargo + actions) | TO VERIFY (P-task: check .github/dependabot.yml) |
| Provenance | SLSA (deferred R2) | P3 |

### 2.4 Prompt-as-code (S-23 gate posture + S-27 noted)

Local baseline (measured): `prompt_manager/` HAS registry (register/get_latest/
get_version/list_versions) + PromptVersion + prompt_eval module + prompt_guardian.
Missing: per-version eval-threshold gate (scores without gates, same disease as §2.2).

Doctrine: prompts change only via registry versions (never inline edits);
a version promotes to latest only on gate pass; eval thresholds live next to the
prompt, not in a distant test file.

---

## 3. Landings (this round)

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R4-1 | release.yml cliff path fix (`config/cliff.toml`) | file exists at referenced path; workflow YAML unchanged otherwise |
| L-R4-2 | R-P254–R-P257 in dev-rules v1.7.0 | rules present with source+implementation |
| L-R4-3 | SIM-12 record in SIM-PROTOCOL.md | row + §10 present |
| L-R4-4 | BLUEPRINT v1.2.3 changelog | line present |

Deferred (recorded): A2A rungs 3–5 (P-task, needs conformance tests);
eval quartet completion + gates (P2 with coverage gate); dist/plz init (P3, needs
secrets + tag discipline); Dependabot check (fast P-task).

---

*End of Absorption Round 4*

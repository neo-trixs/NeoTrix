# NeoTrix Absorption Round 9: Skills / Deliberation / Defense / Routing

> **Absorption Date**: 2026-09-22 | **SIM**: SIM-41 | **Status**: Landed
> **Rounds**: R1 graphify · R2 fitness/verifier/supply/ADR · R3 routing/OTEL/memory/fuzz
> | R4 interop/eval/release/prompts · R5 adversarial/DORA/arch-tooling |
> R6 API/Diátaxis/judge/chaos · R7 flags/signing/behavior · R8 policy/durability/WASM |
> **This round: NTS-F11 (provider ledger) + NTS-E12 (evidence-only handoff) + NTS-E13 (saturation stop) + NTS-G10 (self-evolution gate)**.

---

## 1. Triage — full batch (~95 unique after dedupe)

Deduped repeats: council-of-high-intelligence ×4, Auto-Empirical-Research-Skills ×2,
Agent-Reach ×2, jev-ultrafast ×3, fast-jev-compaction ×2, laya-mlx ×2, minimind/atlas/
cutter/shimmy/needle/KLPO/repowise/codebase-memory-mcp/arxiv-pdfs ×2 each.

| Pri | Items | Rationale |
|-----|-------|-----------|
| P0 read (18) | S-48–S-65 below | direct L5/L6/E/F/G resonance, read this round |
| P1 next | jev-codex-router, fast-jev-compaction, jev-review, jev-security-scan, jev-mcp/mc, jev-hu, jev-cu, jev-trader, KaLM-Jev, jev-skill, hermes-jev-skills, awesome-hermes-agent, super-hermes, Agent-Reach, ARES, astra-flash-orchestrator, Raven, typesafe-mcp, winnow, agent-desktop, PI-Desktop, Orchard, Open-LLM-VTuber, openhuman, multica, first-tree, meetily, recurse, Scrapling, patchright-enhanced, awesome-ai-security-tools, Exegol, ai-ctf, classifier-dev, maka-cu, arxiv-complete, brainapi2, osiris, termgram, Crucix, cumora, deskport, OpenJev-Vision, beautifului, tirith, Cairn, orca, openhermit, takt, dopbase, pond, mira, user-scanner, killmyidea, typesafe-mario, mr-boxington, map3d | skill-ecosystem / router / desktop / security candidates for R10 |
| P2 parked | VoiceStudio, MiroFish, splash, laya, laya-mlx, upscayl, Infographic, coolapk-desktop, wx-cli-again, jynew, rish-app, HowToLiveBetter, hehe-industry-pack, computer-repair-skill, guizang-video-skill, hyalite-liquid-glass, SoL-Pi, SemIf, RuView, ASC, pi, personal-ai, jev-skill (dup), jynew | domain annex material (media/desktop/vertical skills), not core scope |

---

## 2. Source Map

Prior batch (read earlier, landed here): S-48–S-55. arXiv abstracts: S-56–S-57.
New P0 (this session): S-58–S-65.

| # | Source | Type | Used For |
|---|--------|------|----------|
| S-48 | Shimmy (local inference hub; 3-box compat) | Engine | provider certification ledger → NTS-F11 |
| S-49 | MiniMind (tiny LLM training; Agentic RL note) | Method | SEAL reward-design tripwire |
| S-50 | Needle (tiny triage models; per-response confidence) | Pattern | SDB scoring + Triage Gate note |
| S-51 | codebase-memory-mcp (confidence-stamped edges; Scout/Verify/Auditor tiers) | Pattern | validates Confidence enum + triage tiers |
| S-52 | Repowise (errors-first command compression; change-risk 0-10) | Convention | `distill` practice + D-05 G6 note |
| S-53 | KLPO (terminal verifier + KL regularization) | Method | SEAL reward-design tripwire |
| S-54 | Cutter/rizin (RE platform; plugin arch) | Engine | R-P170 binary-analysis candidate (P-task) |
| S-55 | Atlas (Tauri; commit+session checkpoint bundles; handoff packs) | Pattern | validates sessions/handoff practice |
| S-56 | arXiv 2609.10883 Story Imprinting (affinity effect; <2% stories flip conditional behavior) | Paper abs | safety note: fine-tune corpora need trait screening |
| S-57 | arXiv 2609.20800 JEPA-Anything (orthogonal predictive factorization) | Paper abs | parked: world-model pattern, no landing |
| S-58 | council-of-high-intelligence 4.4k★ (blind analysis→cross-exam→stance→synthesis; FACT/INFERENCE/ASSUMPTION/UNKNOWN labels; verdict leads with unresolved + kill criteria + prediction/owner/review-date; polarity-separated multi-provider routing) | Protocol | ADR verdict-format note (no clause) |
| S-59 | Auto-Empirical-Research-Skills 4.0k★ (1096 skills; catalog/skills.json as validated source of truth; 9-stage pipeline + Paper-WorkFlow meta-orchestrator; trust surface w/ pass-fail fixtures; external scoreboard recomputed, not self-reported; artifact-idempotent stages) | System | validates SDB-registry + scoreboard practice |
| S-60 | defending-code-reference-harness 7.5k★ (recon→find→verify→report→patch; **PoC-only crossover** find→grader; dedupe judge; patch grader: builds + PoC dead + tests pass + fresh-find can't bypass; gVisor + egress allowlist; threat model cuts FPs) | Pipeline | NTS-E12 |
| S-61 | SkillOpt 17.3k★ (skill doc = trainable state of frozen agent; trajectory-driven bounded edits; **held-out validation gate**, strict-improvement accept; rejected-edit buffer; textual LR budget; best_skill.md artifact; Sleep: harvest→mine→replay→consolidate) | Optimizer | NTS-G10 |
| S-62 | deepteam 2.9k★ (50+ vulns incl. Agentic: goal theft, recursive hijacking, excessive agency, inter-agent comm compromise, drift; 20+ attacks; OWASP LLM 2025 / Agents 2026 / NIST / MITRE ATLAS mapping; 7 production guardrails) | Framework | SEAL threat-model P-task |
| S-63 | codex-security 10.8k★ (**stopAfterNoNew=3 / maxDiscoveryRuns=10 / maxTimeHours** saturation; findings service + embedding dedupe + independent review; classify-severity under own rubric w/ SQLite checkpoint reuse; sensitive docs stay outside repo; Trusted Access approval) | CLI+SDK | NTS-E13 |
| S-64 | LLMRouter 3.0k★ (16+ routers, 5 categories incl. agentic/personalized; routing as sequential decision; xRouteBench joint quality+cost w/ pre-recorded zero-cost replay; custom-router plugin discovery; routing memory query→model; Pareto alpha*perf − beta*cost) | Library | EQ-08 dispatcher note |
| S-65 | OpenResearch 5.5k★ (local-first; **isolated git worktree per direction**; git-native experiment tree + immutable archive per run; evidence tied to work; autoresearch loop; local SQLite; opt-out telemetry) | Workspace | validates .worktrees/ + evidence-tied-to-work |

Rejected: managed policy/security SaaS (no new vendor); native-plugin default (R8 stands);
full SLSA now (P3); JEPA runtime (parked); sleep-style auto-mutation without gate (G10 forbids).

---

## 3. Pattern → NeoTrix Mapping

### 3.1 Provider ledger (S-48) → NTS-F11
Multi-provider seats without a compat record = silent skew (council S-58 separates
polarities across families for the same reason). Ledger file per provider/model:
endpoint + version + eval snapshot + expiry; CI fails on expired entries.

### 3.2 Evidence-only handoff (S-60) → NTS-E12
Finder and grader MUST be separated; only the artifact (PoC/finding) crosses, never
the finder's narrative. Patch acceptance = builds + PoC dead + suite green +
fresh-find can't bypass. Mirrors SDB Propose→Verify→Commit.

### 3.3 Saturation stop (S-63) → NTS-E13
Every unbounded search loop (fuzz, red-team, discovery scans) declares
no-new-N / max-runs / max-time before starting; results report which bound fired.

### 3.4 Self-evolution gate (S-61) → NTS-G10
Any self-modifying artifact (skill, prompt, policy) changes only via bounded edits
accepted on strict held-out improvement; rejected edits buffered, never silently dropped.
Sleep-style offline consolidation runs behind the same gate. (Resonance: harvest→mine→
replay→consolidate mirrors our 快照→蒸馏→分类→落盘→反馈.)

### 3.5 Bake notes (no clause)
- Council verdict format (S-58): ADR verdicts lead with unresolved + kill criteria +
  prediction/owner/review-date; evidence labeled FACT/INFERENCE/ASSUMPTION/UNKNOWN.
- AERS trust (S-59): scoreboards recomputed with one scorer, never self-reported;
  catalog file is the validated source of truth (our SDB-registry already does this).
- Agentic threat taxonomy (S-62): feeds SEAL threat model (P-task, L5).
- Routing memory + Pareto cost (S-64): feeds EQ-08 dispatcher decision (P-task, L1).
- Worktree-per-direction + immutable run archives (S-65): our `.worktrees/` + sessions/
  practice validated; run-archive immutability is a P-task.
- Story imprinting (S-56): fine-tune/eval corpora get trait screening (safety note).
- Distill convention (S-52): errors-first command compression with markers, adopted as
  practice; confidence-per-response (S-50) + checkpoint bundles (S-55, S-51) validate
  existing Confidence/SDB/session designs.

---

## 4. Landings (this round)

| ID | Artifact | Verifies As |
|----|----------|-------------|
| L-R9-1 | NTS-F11/E12/E13/G10 in NT-STD 1.0.7 | clauses present with Verify |
| L-R9-2 | SIM-41 record | row + §40 present |
| L-R9-3 | BLUEPRINT v1.6.6 changelog | line present |

Deferred (recorded): Cedar adoption (SIM-gated); WASM code (P-task L1/L3); rizin engine
(P-task); SEAL threat model + reward design (P-task L5); EQ-08 dispatcher (P-task L1);
run-archive immutability (P-task); R10 P1-batch triage.

---

*End of Absorption Round 9*

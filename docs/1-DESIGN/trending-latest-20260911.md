# Trending Research Report — September 11, 2026

## Executive Summary

**Key signal**: The "Harness as a First-Class Cost Lever" paradigm is now mainstream. NVIDIA's SoL-Pi (Sep 10) proved harness optimization yields 45-64% token savings at 94% quality. The academic pipeline has exploded with 10+ harness evolution papers in Sep 2026 alone. Context management is now treated as a lifecycle problem, not just storage.

**New paradigm shift**: "Harness Scaling Law" — harness quality is orthogonal to model scaling, and compound improvement is possible through automated research loops.

---

## SECTION 1: NVlabs/SoL-Pi — BREAKING (Sep 10, 2026)

**URL**: https://github.com/NVlabs/SoL-Pi
**Stars**: New (just released)
**Source**: NVIDIA Efficient AI Team

### Architecture
SoL-Pi is an efficiency enhancement layer built on top of Pi (the minimal coding agent harness). It uses an automated "Agent researching Agent" pipeline: AI researchers analyze agent workflows, identify token wastage, propose architectural changes, and test modifications. From 152 candidate ideas, only 4 mechanisms survived three rounds of filtering.

### 4 Core Mechanisms (all avoid modifying the main program)

| Mechanism | What It Does | Token Savings |
|-----------|-------------|---------------|
| **Action Fusion** | Merges sequential edit+test into single calls, eliminating intermediate model turns | Eliminates redundant model decisions |
| **Online Context Compact** | Dynamically compresses context when future savings outweigh rewrite cost; breaks tasks into sub-tasks, re-evaluates after each step | ~45% vs base Pi |
| **ObservationPack** | Archives large tool outputs to disk, replaces with indexed handles + excerpts | Eliminates repeated output billing |
| **Evidence-Preserving Reducer** | Small model pre-screens logs, strict line-by-line evidence verification against archived originals | 50-54% cost reduction |

### Performance
- 45-49% fewer tokens vs base Pi
- 35-64% fewer tokens vs Codex/Claude Code harnesses
- 50-54% API cost reduction
- ~94% task performance retained
- Install: `pi install git:github.com/NVlabs/SoL-Pi`

### NeoTrix Mapping
- **Domain**: NT-CORE (self-optimization engine)
- **Module to create**: `nt_core_harness_optimizer` — automated harness efficiency research pipeline
- **Absorption priority**: **P0** — direct implementation of Action Fusion, ObservationPack, Evidence-Preserving Reducer into NT-ACT tool execution
- **Key pattern**: "Harness Scaling Law" — self-sustaining research flywheel where AI discovers more efficient mechanisms

### Related: pi-agent-space
- **URL**: https://github.com/soulstrop/pi-agent-space
- Systematic exploration of multi-dimensional agent configuration space using 5D Pareto frontier (tokens, dollars, scaling slope, quality, subjective)
- Hexagonal (ports-and-ads) architecture for harness optimization
- Maps to: NT-MIND optimization engine

---

## SECTION 2: GitHub Trending — NEW Projects (Sep 2026)

### 2.1 headroomlabs-ai/headroom
- **Stars**: 1.9k+ (weekly trending)
- **URL**: https://github.com/headroomlabs-ai/headroom
- **Architecture**: External `/v1/compress` proxy for context token compression before provider routing. Works as middleware between agent and LLM provider.
- **Key pattern**: Context compression as a service — decouples compression from agent logic
- **NeoTrix Mapping**: NT-IO context pipeline integration; maps to `nt_io::context_compressor`
- **Absorption priority**: P1

### 2.2 mksglu/context-mode
- **Stars**: 525 (trending today)
- **URL**: https://github.com/mksglu/context-mode
- **Architecture**: Context mode management for coding agents — likely implements structured context switching between different agent modes (exploration, editing, debugging)
- **Key pattern**: Mode-aware context assembly
- **NeoTrix Mapping**: NT-CORE attention routing enhancement
- **Absorption priority**: P2

### 2.3 Tencent/teamai-cli
- **Stars**: 878 (trending today)
- **URL**: https://github.com/Tencent/teamai-cli
- **Architecture**: Team-oriented AI CLI — multi-agent coordination with team management semantics
- **Key pattern**: Multi-agent team orchestration with role-based access
- **NeoTrix Mapping**: NT-ACT orchestration extension
- **Absorption priority**: P2

### 2.4 THU-MAIC/OpenMAIC
- **Stars**: 32,821 (9,193 this week)
- **URL**: https://github.com/THU-MAIC/OpenMAIC
- **Architecture**: Open-source multi-agent intelligence framework from Tsinghua. TypeScript-based.
- **Key pattern**: Multi-agent collaboration platform
- **NeoTrix Mapping**: NT-ACT multi-agent orchestration
- **Absorption priority**: P1

### 2.5 huggingface/funes
- **Stars**: Trending Sep 8 (Rust)
- **URL**: https://github.com/huggingface/funes
- **Architecture**: Rust-based inference runtime from HuggingFace
- **Key pattern**: High-performance Rust inference
- **NeoTrix Mapping**: NT-IO provider optimization
- **Absorption priority**: P2

### 2.6 NousResearch/hermes-agent
- **Stars**: 32.9k+ (monthly trending)
- **URL**: https://github.com/NousResearch/hermes-agent
- **Architecture**: Self-growing agent with periodic post-execution evolution
- **Key pattern**: Periodic harness evolution (slower than real-time)
- **NeoTrix Mapping**: NT-MIND evolution pipeline comparison
- **Absorption priority**: P1

### 2.7 Gitlawb/openclaude
- **Stars**: 32,889 (1,944 this week)
- **URL**: https://github.com/Gitlawb/openclaude
- **Architecture**: Open-source Claude-like agent — likely implements similar harness patterns
- **Key pattern**: Harness transparency for production-grade coding agents
- **NeoTrix Mapping**: NT-ACT harness reference
- **Absorption priority**: P2

### 2.8 YeQing17-2026/OmniAgent
- **Stars**: 2,557
- **URL**: https://github.com/YeQing17-2026/OmniAgent
- **Architecture**: Full-dimensional self-evolution (OmniEvolve) + Hyper-Harness + Deep Reflexion
- **Key patterns**:
  - Real-time skill evolution during execution (vs periodic)
  - Four-layer dynamic security scanning (LLM review → Policy engine → Interactive approval → Execution sandbox)
  - Dynamic concurrent tool execution with dependency resolution
  - Inner-outer dual-layer reflective architecture
- **NeoTrix Mapping**: NT-MIND real-time evolution + NT-SHIELD security layers
- **Absorption priority**: **P0** — Hyper-Harness design directly applicable to NT-ACT

### 2.9 PrimeIntellect-ai/prime-agent
- **Stars**: 1,456
- **URL**: https://github.com/PrimeIntellect-ai/prime-agent
- **Architecture**: Recursive Language Model (RLM) treating context as variables, tools as recursive subagents in persistent REPL. Continual Harness stores durable state (supplemental prompts, memories, skills) that agent refines through evidence-backed updates.
- **Key patterns**:
  - `/refine` command for evidence-backed harness state updates
  - Daemon-backed sessions survive terminal disconnect
  - Agent-to-agent direct communication
  - Skills as executable Python packages
- **NeoTrix Mapping**: NT-MIND harness refinement + NT-NEXUS cross-session state
- **Absorption priority**: **P0** — Continual Harness pattern directly applicable

### 2.10 HKUDS/nanobot
- **Stars**: 47,663
- **URL**: https://github.com/HKUDS/nanobot
- **Architecture**: Ultra-lightweight self-hosted agent with WebUI, tools, long-term memory (Dream), MCP, multi-agent delegation, scheduled automation. OpenAI-compatible API.
- **Key pattern**: Lightweight agent gateway with chat app integrations (Telegram, Discord, Slack, WeChat, Email)
- **NeoTrix Mapping**: NT-IO integration reference
- **Absorption priority**: P2

### 2.11 CodeSoul-co/Hypha
- **Stars**: 165
- **URL**: https://github.com/CodeSoul-co/Hypha
- **Architecture**: Two-layer design: Agent Core (reasoning/ReAct, planning, tool selection, memory) + Production Harness (FSM control, policy/approval, checkpoints, recovery, replay, audit)
- **Key pattern**: Clean separation of reasoning vs execution harness
- **NeoTrix Mapping**: NT-CORE (reasoning) + NT-ACT (execution) separation
- **Absorption priority**: P1

### 2.12 vercel/eve
- **Stars**: 4,957
- **URL**: https://github.com/vercel/eve
- **Architecture**: Filesystem-first framework for durable AI agents. Core capabilities live in conventional filesystem locations for inspectability.
- **Key pattern**: Filesystem as the single source of truth for agent state
- **NeoTrix Mapping**: NT-MEMORY filesystem integration
- **Absorption priority**: P2

### 2.13 withastro/flue
- **Stars**: 8,056
- **URL**: https://github.com/withastro/flue
- **Architecture**: Agent Harness Framework with TypeScript runtime, built-in sessions/tools/skills/instructions/filesystem/sandbox
- **Key pattern**: Harness as a runtime primitive
- **NeoTrix Mapping**: NT-ACT harness runtime
- **Absorption priority**: P1

### 2.14 decolua/9router
- **Stars**: 23,774
- **URL**: https://github.com/decolua/9router
- **Architecture**: AI router with 40+ providers, RTK token saver (20-40% savings), auto-fallback (subscription→cheap→free), multi-account load balancing
- **Key patterns**:
  - RTK (from rtk-ai/rtk ⭐40K) — compress tool outputs before sending to LLM
  - Caveman Mode — inject prompt for terse replies (65% output token savings)
  - Ponytail — "lazy senior dev" prompt for YAGNI-first code
  - Headroom integration — external compression proxy
- **NeoTrix Mapping**: NT-IO provider routing + token optimization
- **Absorption priority**: **P0** — RTK pattern directly applicable to NT-IO context pipeline

### 2.15 DietrichGebert/ponytail
- **Stars**: 12.9k (monthly trending)
- **URL**: https://github.com/DietrichGebert/ponytail
- **Architecture**: "Lazy senior dev" prompt injection — LLM writes minimal, YAGNI-first code with Lite/Full/Ultra modes
- **Key pattern**: Prompt-level code quality optimization
- **NeoTrix Mapping**: NT-MIND code quality heuristics
- **Absorption priority**: P2

### 2.16 humanlayer/skills
- **Stars**: 2.8k (trending Sep 3)
- **URL**: https://github.com/humanlayer/skills
- **Architecture**: Battle-tested agent skills library for coding agents
- **Key pattern**: Curated skill marketplace
- **NeoTrix Mapping**: NT-ACT skill nodes reference
- **Absorption priority**: P2

### 2.17 open-gitagent/gitagent
- **Stars**: 670
- **URL**: https://github.com/open-gitagent/gitagent
- **Architecture**: Git-native agent — identity, rules, memory, tools, skills all version-controlled in git repos. "Agents as repos" paradigm.
- **Key pattern**: Version-controlled agent state
- **NeoTrix Mapping**: NT-MEMORY versioning
- **Absorption priority**: P2

### 2.18 EvoMap/AutoResearch
- **Stars**: Trending (daily)
- **URL**: https://github.com/EvoMap/AutoResearch
- **Architecture**: Autonomous research automation
- **Key pattern**: Automated research pipeline
- **NeoTrix Mapping**: NT-MIND research automation
- **Absorption priority**: P2

---

## SECTION 3: Grok Build (xAI) — Latest Updates

### Grok Build Open Source (Jul 15, 2026)
- **Stars**: 23,000+ (growing fast)
- **URL**: https://github.com/xai-org/grok-build
- **License**: Apache 2.0 (read-only mirror from internal monorepo)
- **Language**: Rust

### Architecture
```
xai-grok-shell      → Agent runtime (leader/stdio/headless)
xai-grok-tools      → Tool implementations (terminal, file edit, search)
xai-grok-workspace  → Host filesystem, VCS, execution checkpoints
xai-grok-pager      → TUI (scrollback, prompts, modals, rendering)
```

### Key Patterns
- Extension system: Skills / Plugins / Hooks / MCP Servers / Subagents
- Local-first: compile yourself, point `base_url` to local inference
- ACP (Agent Client Protocol) for IDE embedding
- Model-agnostic via `config.toml`
- Read-only mirror — no external PRs accepted

### Grok Bot (Aug 11, 2026)
- Always-on cloud computer per user account
- Multiple Bots coordinate (sales, recruiting, finance, dev)
- Persistent learning — observe → remember → automate
- Access/network/audit controls for enterprise

### NeoTrix Mapping
- NT-ACT harness reference implementation (Rust)
- NT-IO ACP integration pattern
- NT-SHIELD audit trail design
- Absorption priority: P1 (harness patterns), P2 (TUI)

---

## SECTION 4: New arXiv Papers — Harness Evolution (Sep 2026)

### 4.1 RobustSGPO: Search-Space Control for Agent Harness Evolution
- **URL**: https://arxiv.org/abs/2609.09646
- **Date**: Sep 9, 2026
- **Key finding**: Periodic permission scheduling exceeds fixed max permission by +0.28 test-score. Category retention reduces source-task degradation after domain shift.
- **NeoTrix mapping**: NT-MIND harness evolution search-space control

### 4.2 HarnessDev: Can LLMs Create and Evolve Their Own Agent Harness?
- **URL**: https://arxiv.org/abs/2609.01437
- **Date**: Sep 1, 2026
- **Key finding**: Models match human reference on writing and ML experimentation, but remain far behind on code and search/research harnesses. Evolution gains are unstable and transfer only partially.
- **NeoTrix mapping**: NT-MIND self-evolution calibration

### 4.3 HarnessEvolve: Learning from Reference Trajectories
- **URL**: https://arxiv.org/abs/2609.00829
- **Date**: Sep 1, 2026
- **Key finding**: Decouples execution from evolution. Reference trajectories solve credit assignment. Quality gate + performance gate prevent shortcut learning and catastrophic forgetting.
- **NeoTrix mapping**: NT-MIND evolution gating mechanism

### 4.4 Harness-of-Harness (HoH): Multi-Day Autonomous Development
- **URL**: https://arxiv.org/abs/2609.01481
- **Date**: Sep 1, 2026
- **Key finding**: 52.25% average relative gain, 82.86% max gain after 3 iterations. Tested on Codex/GPT-5.5, OpenCode/DeepSeek-V4-Pro, Pi/MiniMax-M3. 70+ iteration autonomous game development demo.
- **NeoTrix mapping**: NT-MIND iterative harness improvement

### 4.5 ContextPipe: Database-Inspired Context Assembly
- **URL**: https://arxiv.org/abs/2609.00749
- **Date**: Sep 1, 2026
- **Key finding**: Context assembly is structurally isomorphic to database query execution. 5-phase pipeline (Plan→Bind→Optimize→Execute→Feedback). 31% token reduction, 23% fewer LLM calls, 9% faster.
- **NeoTrix mapping**: **P0** — directly applicable to NT-MEMORY context pipeline. EXPLAIN ANALYZE trace pattern for context auditing.

### 4.6 ContextPilot: Proactive Context Management via Fine-grained RL
- **URL**: https://arxiv.org/abs/2608.28476
- **Date**: Aug 21, 2026
- **Key finding**: Context-aware partial rollout + fine-grained credit assignment. Tools: memorize, readMemory, summarizeContext, compressContext, foldHistory.
- **NeoTrix mapping**: NT-MEMORY proactive context management

### 4.7 ACM: Agentic Context Management
- **URL**: https://arxiv.org/abs/2607.23809
- **Date**: Jul 26, 2026
- **Key finding**: Agent-initiated lossless context management. 27% improvement on BrowseComp-Plus, 16% on DeepSearchQA, 8% on SWE-Bench Verified. 20% peak token reduction.
- **NeoTrix mapping**: NT-MEMORY agent-initiated compaction

### 4.8 Context as an Environment (Scroll)
- **URL**: https://arxiv.org/abs/2608.21690
- **Date**: Aug 21, 2026
- **Key finding**: Session as executable environment with append-only Event Log + persistent Python kernel. 94.8% on LongMemEval_S, 73.1% on BEAM_10M, 86.7% on LOCA_256K.
- **NeoTrix mapping**: NT-NEXUS session environment design

### 4.9 Compile, Don't Memorize (CCA)
- **URL**: https://arxiv.org/abs/2609.00759
- **Date**: Sep 1, 2026 (EMNLP 2026 Findings)
- **Key finding**: Typed IR with fixed slots (rules.must_do, rules.must_not, conditional, output_spec, available_tools, data_profile). Lifts Kimi K2.5 from 15.4% to 21.4%.
- **NeoTrix mapping**: NT-CORE context compilation pipeline

### 4.10 What Does Multi-Harness RL Learn?
- **URL**: https://arxiv.org/abs/2609.04518
- **Date**: Sep 3, 2026
- **Key finding**: Evaluation harness is the dominant variable (4.3x factor), not training recipe (1.16x). Cross-harness credit yields configuration adaptation, not portable capability.
- **NeoTrix mapping**: NT-MIND evaluation harness importance

### 4.11 Building the Harness Automatically (Self-Play)
- **URL**: https://arxiv.org/abs/2609.09468
- **Date**: Sep 8, 2026
- **Key finding**: Agent writes/evaluates optimizer programs, distills into 197-word text harness. 48% regret reduction. Transfers across model families (Gemini→Claude).
- **NeoTrix mapping**: NT-MIND executable practice → text harness distillation

### 4.12 Context Privilege Escalation Attacks
- **URL**: https://arxiv.org/abs/2609.01222
- **Date**: Sep 1, 2026
- **Key finding**: Two novel attack categories: M-CPE (role escalation) and X-CPE (cross-scope persistence). Tested on 12 real harnesses including Claude Code and Codex. Full agent compromise, RCE, DoS possible.
- **NeoTrix mapping**: **P0** — NT-SHIELD must implement context privilege boundaries

### 4.13 Co-Evolving Harnesses and Models
- **URL**: https://arxiv.org/abs/2609.09134
- **Date**: Sep 9, 2026
- **Key finding**: Imitation of expert under evolved harness causes -14.9 avg regression (planning style mismatch). On-policy correction (fix single failing turn) enables harness+model co-evolution. +1.7 gain.
- **NeoTrix mapping**: NT-MIND on-policy correction for model adaptation

---

## SECTION 5: Priority Absorption Map

### P0 — Immediate Implementation

| Project/Paper | Pattern | NT Module |
|---------------|---------|-----------|
| NVlabs/SoL-Pi | Action Fusion, ObservationPack, Evidence-Preserving Reducer | `nt_core_harness_optimizer` |
| OmniAgent | Hyper-Harness, dynamic concurrent tool execution, four-layer security | `nt_act::hyper_harness` |
| PrimeIntellect/prime-agent | Continual Harness, `/refine` evidence-backed updates | `nt_mind::continual_harness` |
| 9router/RTK | Tool output compression before LLM | `nt_io::context_compressor` |
| ContextPipe | 5-phase database-inspired context assembly | `nt_memory::context_pipeline` |
| Context Privilege Escalation | M-CPE and X-CPE attack defense | `nt_shield::context_boundary` |

### P1 — High Priority

| Project/Paper | Pattern | NT Module |
|---------------|---------|-----------|
| headroom | External compression proxy | `nt_io::compression_proxy` |
| OpenMAIC | Multi-agent intelligence framework | `nt_act::multi_agent` |
| hermes-agent | Periodic harness evolution | `nt_mind::periodic_evolution` |
| Hypha | Two-layer Core+Harness separation | `nt_core::reasoning_harness_split` |
| flue | Harness as runtime primitive | `nt_act::harness_runtime` |
| ContextPilot | Proactive context management with RL | `nt_memory::proactive_context` |
| ACM | Agent-initiated lossless compaction | `nt_memory::agent_compaction` |
| HoH | Iterative multi-day harness improvement | `nt_mind::iterative_harness` |
| Scroll | Session environment with Event Log | `nt_nexus::session_environment` |

### P2 — Watch

| Project | Pattern |
|---------|---------|
| context-mode | Mode-aware context assembly |
| teamai-cli | Team-oriented multi-agent CLI |
| funes | Rust inference runtime |
| openclaude | Harness transparency |
| nanobot | Lightweight agent gateway |
| eve | Filesystem-first agent state |
| ponytail | Prompt-level code quality |
| skills | Curated skill marketplace |
| gitagent | Version-controlled agent state |
| AutoResearch | Automated research pipeline |
| CCA | Context compilation with typed IR |
| Self-Play Harness | Executable practice → text harness |

---

## SECTION 6: Key Trends Identified

### 1. Harness as First-Class Cost Lever
- Databricks benchmark: 2x cost variance at identical quality across harnesses
- SoL-Pi: 45-64% token savings, 50-54% cost reduction
- Implication: Harness choice is now a measurable dollar-per-task decision

### 2. Context as Lifecycle (Not Storage)
- 7 papers in Sep 2026 treat context as lifecycle: architecting → ingesting → scoping → anticipating → compacting
- Database-inspired context assembly (ContextPipe)
- Agent-initiated lossless compaction (ACM)
- Implication: NT-MEMORY needs lifecycle-aware context pipeline

### 3. Harness Scaling Law (Emerging)
- NVIDIA's hypothesis: harnesses exhibit their own scaling law
- Compound efficiency: cheaper harness → larger research cycles → cheaper harness
- Implication: NT-MIND needs automated harness research loop

### 4. Security in Context Assembly
- First systematic analysis of context assembly security (12 harnesses tested)
- Two novel attack categories: M-CPE and X-CPE
- Implication: NT-SHIELD must implement context privilege boundaries

### 5. On-Policy Correction > Imitation
- Expert imitation under evolved harness causes regression
- On-policy correction (fix single failing turn) enables co-evolution
- Implication: NT-MIND model adaptation must preserve harness fit

### 6. Filesystem as Agent State
- Eve, GitAgent, Grok Build all use filesystem as single source of truth
- Implication: NT-MEMORY should adopt filesystem-first state management

---

## SECTION 7: Previous Research Cross-Reference

Projects from prior research cycles (for deduplication):
- OpenClaw (210k+ stars) — already tracked
- Claude Code — already tracked
- Codex CLI — already tracked
- Hermes Agent — already tracked
- Browser-use — already tracked
- CrewAI, MetaGPT, AutoGen — already tracked
- LangChain, Langflow, Dify — already tracked

**All projects in this report are NEW since last research cycle.**

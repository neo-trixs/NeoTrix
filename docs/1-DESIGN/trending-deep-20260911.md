# Trending Deep Research — 2026-09-11

## Executive Summary

Three dominant architectural patterns emerged across all sources this week:
1. **Hook-First Harness Architecture** — lifecycle hooks as first-class citizens (Grok Build, harness-pi, OpenHarness)
2. **Self-Evolving Harness** — harness that modifies itself from execution traces (SoL-Pi, OmniAgent, GenericAgent, HarnessX paper)
3. **Context-as-First-Class** — treating context management as a programmable substrate, not an afterthought (ACM, ContextPilot, ContextPipe, Scroll)

---

## 1. NVIDIA SoL-Pi

| Field | Value |
|-------|-------|
| **URL** | https://github.com/NVlabs/SoL-Pi |
| **Stars** | New (just open-sourced Sep 2026) |
| **Language** | Python (built on Pi) |
| **Core Architecture** | Self-evolving agent harness built on top of Pi coding agent. "Agent researches Agent" pipeline: 535 verifiable environments → 152 optimization directions → 3-round screening. |
| **Key Pattern** | **Automated harness evolution** — systematically explores optimization directions through verifiable environments. Each optimization is validated against concrete benchmarks before adoption. |
| **NeoTrix Domain** | NT-MIND (self-evolution pipeline) + NT-CORE (reasoning engine) |
| **Absorption Priority** | **P0** |
| **NeoTrix Target** | `nt_mind/src/seal/` — extend SEAL pipeline with automated environment-based optimization. Create `nt_mind/src/seal/harness_evolution.rs` for systematic harness self-improvement via verifiable environments. |

**Absorbable Details:**
- Verifiable environment pattern: 535 environments as ground truth for evaluating harness mutations
- Optimization direction proposal + 3-round screening (propose → filter → validate)
- Pi install integration: `pi install git:github.com/NVlabs/SoL-Pi` — harness as a package
- Maps directly to NT-MIND's SEAL pipeline as a new Phase between Distillation and Absorption

---

## 2. Grok Build (xAI / SpaceXAI)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/xai-org/grok-build |
| **Stars** | 20k+ (open-sourced Jul 2026) |
| **Language** | Rust |
| **Core Architecture** | Full-screen TUI coding agent with hook-first architecture. Agent runtime in `xai-grok-shell`, tools in `xai-grok-tools`, workspace in `xai-grok-workspace`. Supports headless, TUI, and ACP agent modes. |
| **Key Pattern** | **3-cadence hook system** (session/turn/tool-level) + **ACP (Agent Client Protocol)** for IDE integration + **Landlock sandbox** for security + **Subagent worktrees** for parallel execution. |
| **NeoTrix Domain** | NT-IO (ACP protocol, TUI) + NT-ACT (tool system) + NT-SHIELD (sandbox) + NT-CORE (agent loop) |
| **Absorption Priority** | **P0** |
| **NeoTrix Target** | `nt_io/src/acp/` — implement ACP protocol for IDE integration. `nt_shield/src/sandbox/` — landlock-based sandboxing. `nt_act/src/hooks/` — 3-cadence hook system. |

**Absorbable Details:**
- Hook events: `SessionStart/End`, `Stop/SubagentStop`, `PreToolUse/PostToolUse`, `PreCompact/PostCompact`, `PermissionDenied`
- Blocking hooks can return `HookDecision` (Deny/Block) — agent receives denial as feedback
- ACP protocol: JSON-RPC over stdin/stdout, used by Zed, Neovim, Emacs
- Plugin hooks auto-namespaced to prevent collisions
- SSRF protection on HTTP hooks, `${VAR}` env expansion with safety
- Subagent worktrees: each child gets isolated context window
- Context assembly transparency: every step inspectable

---

## 3. NVIDIA NOOA (Object-Oriented Agents)

| Field | Value |
|-------|-------|
| **URL** | https://developer.nvidia.com/blog/six-agent-harness-capabilities-for-higher-model-performance/ |
| **Stars** | N/A (research preview) |
| **Language** | Python |
| **Core Architecture** | Agent = single Python class. Methods = capabilities, fields = state, docstrings = prompts, type annotations = enforced contracts. Ellipsis methods completed by LLM at runtime. |
| **Key Pattern** | **6 model-facing interface ideas**: typed I/O, pass-by reference (live objects, not serialization), code as action, programmable loops, explicit object state, model-callable harness APIs. |
| **NeoTrix Domain** | NT-CORE (agent architecture) + NT-MEMORY (knowledge graph memory) |
| **Absorption Priority** | **P0** |
| **NeoTrix Target** | `nt_core/src/agent/` — implement typed I/O contracts for tool calls. `nt_memory/src/knowledge_graph.rs` — model-callable memory with typed relationships (supports/contradicts/derived-from). |

**Absorbable Details:**
- Pass-by reference: model operates on live Python objects, sees bounded previews instead of serialized dumps
- Token savings: peak 2x efficiency from avoiding serialization
- Memory as curated store (not automatic summarization): agent deliberately writes/queries/corrects records
- Knowledge graph with typed relationships + background reflection pass (merge duplicates, link records, distill insights)
- SQLite-backed, human-readable, inspectable
- Model-callable harness APIs: context blocks and event history are APIs the model can inspect/manage

---

## 4. HarnessX (Darwin-Agent)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/Darwin-Agent/HarnessX |
| **Stars** | 454 |
| **Language** | Python |
| **Core Architecture** | Harness foundry: typed harness primitives composed via substitution algebra. AEGIS evolution engine (Digester→Planner→Evolver→Critic) with deterministic gating. |
| **Key Pattern** | **Operational mirror** mapping harness adaptation onto RL: configurations=states, edits=actions, traces+verifier scores=feedback. **Harness-model co-evolution** via shared replay buffer + cross-harness GRPO. |
| **NeoTrix Domain** | NT-MIND (harness evolution) + NT-CORE (typed primitives) |
| **Absorption Priority** | **P1** |
| **NeoTrix Target** | `nt_mind/src/seal/harness_evolution.rs` — AEGIS 4-stage pipeline for harness self-improvement. `nt_core/src/harness/primitives.rs` — typed, composable harness primitives. |

**Absorbable Details:**
- AEGIS 4-stage: Digester (compress traces) → Planner (adaptation landscape) → Evolver (typed edits) → Critic (reject unsupported claims)
- Deterministic gating: no edit ships without Critic + gate approval
- Variant-isolation strategy prevents cross-task interference
- Cross-harness GRPO: model internalizes strategies from successive harness versions
- +14.5% average gain across 5 benchmarks (up to +44%)

---

## 5. ContextMode (mksglu)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/mksglu/context-mode |
| **Stars** | 19.9k |
| **Language** | TypeScript |
| **Core Architecture** | Context-window optimization layer that sandboxes tool output before it reaches the model. Progressive disclosure applied to tool results (not instructions). Cross-platform session memory via MCP + hooks. |
| **Key Pattern** | **Tool output sandboxing** — 98% reduction in context consumption by progressive disclosure of tool results. Session memory persisted across 17 agent platforms. |
| **NeoTrix Domain** | NT-CORE (context management) + NT-MEMORY (cross-session memory) |
| **Absorption Priority** | **P1** |
| **NeoTrix Target** | `nt_core/src/context/sandbox.rs` — tool output sandboxing with progressive disclosure. Extend `kv_cache_optimizer.rs` with tool-result tiered disclosure. |

**Absorbable Details:**
- Tool output sandboxing: intercept tool results before they enter context
- Progressive disclosure: show only summary initially, expand on demand
- Cross-platform: 17 agent platforms supported via MCP + hooks
- Claims 98% context reduction — worth validating against NeoTrix patterns

---

## 6. harness-pi (chasey-myagi)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/chasey-myagi/harness-pi |
| **Stars** | New (0.6.0 RC) |
| **Language** | TypeScript |
| **Core Architecture** | Hook-first service runtime: kernel does 2 things (LLM-tool loop + hook dispatch). 22 plugin factories + 14 controllers. Everything else is plugins. |
| **Key Pattern** | **Cache-aware compaction projection** — `compaction_boundary` as live first-class citizen, prefix byte stability for provider prompt-cache. Branded Slot API, LeaseQueue with exponential backoff. |
| **NeoTrix Domain** | NT-CORE (agent loop) + NT-IO (LLM cache optimization) |
| **Absorption Priority** | **P1** |
| **NeoTrix Target** | `nt_core/src/agent/loop.rs` — kernel should fire hooks at 3 cadences, not embed logic. `nt_io/src/llm/cache.rs` — cache-aware compaction boundaries. |

**Absorbable Details:**
- Kernel = 2 responsibilities only: run LLM-tool loop + dispatch hooks
- All strategies (watchdog, metrics, compaction, tool buffer, log, permission gate) = hook instances
- `compaction_boundary` as live projection: prefix bytes stable across compaction, cache-friendly
- LeaseQueue with exponential backoff for session management
- 0.5.0 flagship: cache-aware compaction — verified with real provider cache A/B tests
- OTel sink for observability

---

## 7. pi-agent-harness (baryonlabs)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/baryonlabs/pi-agent-harness |
| **Stars** | 1 |
| **Language** | TypeScript |
| **Core Architecture** | Team-architecture factory: domain sentence → multi-agent team. 6 patterns (Pipeline, Fan-out/Fan-in, Expert Pool, Producer-Reviewer, Supervisor, Hierarchical Delegation). |
| **Key Pattern** | **L3 Meta-Factory** — generates other harnesses rather than being one. Model tiering per agent (cheap for recon, expensive for reasoning). Progressive disclosure for skill context. |
| **NeoTrix Domain** | NT-ACT (orchestration) + NT-MIND (skill generation) |
| **Absorption Priority** | **P2** |
| **NeoTrix Target** | `nt_act/src/orchestrator/team_factory.rs` — domain decomposition into coordinated agent teams. |

**Absorbable Details:**
- 6 team-architectural patterns mapped to delegation modes (single/parallel/chain)
- Model tiering: per-agent model selection based on task complexity
- Progressive Disclosure for skill loading
- `_workspace/` file handoff between agents
- Validation: trigger verification + dry-run + with/without skill comparison

---

## 8. OmniAgent

| Field | Value |
|-------|-------|
| **URL** | https://github.com/YeQing17-2026/OmniAgent |
| **Stars** | 2,557 |
| **Language** | Python |
| **Core Architecture** | Full-dimensional self-evolution (OmniEvolve): Skill + Context + BrainModel evolution. Hyper-Harness with dynamic multi-agent + concurrent tool execution. Deep Reflexion inner-outer dual-layer loop. |
| **Key Pattern** | **4-layer dynamic security scanning** (LLM review → Policy engine → Interactive approval → Execution sandbox). **Dynamic concurrent tool execution** — auto-resolves inter-tool dependencies for async parallel invocation. |
| **NeoTrix Domain** | NT-MIND (self-evolution) + NT-SHIELD (security) + NT-ACT (parallel execution) |
| **Absorption Priority** | **P1** |
| **NeoTrix Target** | `nt_shield/src/security/dynamic_scanner.rs` — 4-layer progressive security. `nt_mind/src/seal/omnievolve.rs` — real-time skill evolution during execution. |

**Absorbable Details:**
- Skill injection via User Message saves 90% token cost vs alternatives
- Progressive Context Loading inspired by Anthropic Claude Skills
- Dynamic Multi-Agent: Sentinel (planning) + Guardian (safety) agents activate on-demand
- Inner-Outer Failure Experience Conversion: LLM-driven RCA + heuristic strategy extraction
- Three-layer failure prevention: trajectory repetition, error action repetition, loop pseudo-termination

---

## 9. GenericAgent

| Field | Value |
|-------|-------|
| **URL** | https://github.com/lsdefine/GenericAgent |
| **Stars** | 14,114 |
| **Language** | Python |
| **Core Architecture** | ~3K lines core, 9 atomic tools, ~100-line Agent Loop. Self-evolving skill tree from task execution. Token-efficient (<30K context window). |
| **Key Pattern** | **Contextual Information Density Maximization** — compact context with high information density. Skill tree grows with every use. Morphling mode for project-level capability absorption. Goal Hive for multi-worker parallel objectives. |
| **NeoTrix Domain** | NT-MIND (skill tree growth) + NT-CORE (agent loop) |
| **Absorption Priority** | **P1** |
| **NeoTrix Target** | `nt_mind/src/skill_tree/` — self-evolving skill tree with token efficiency. Study token-efficient patterns for context compression. |

**Absorbable Details:**
- <30K context window vs 200K-1M for competitors
- Morphling mode: extract goals + tests from external repos, decide per component: call/rewrite/discard
- Goal Hive: BBS-coordinated master/workers for long-horizon parallel objectives
- Conductor sub-agent orchestration with auto-cleanup
- L4 session archive memory + scheduler cron integration

---

## 10. OpenHarness (HKUDS)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/hkuds/openharness |
| **Stars** | 15,539 |
| **Language** | Python |
| **Core Architecture** | 10-subsystem agent harness: engine, tools (43), skills, plugins, permissions, hooks, commands (54), MCP, memory, tasks, coordinator, prompts, config, UI. 44x lighter than Claude Code. |
| **Key Pattern** | **Auto-Compact + MEMORY.md persistent memory** — context compression triggered by agent. Swarm coordination with subagent spawning + team registry. PreToolUse/PostToolUse hooks. |
| **NeoTrix Domain** | NT-CORE (harness) + NT-MEMORY (persistent memory) + NT-ACT (coordination) |
| **Absorption Priority** | **P1** |
| **NeoTrix Target** | `nt_core/src/harness/subsystems.rs` — 10-subsystem harness pattern. `nt_memory/src/persistent_memory.rs` — MEMORY.md style cross-session persistence. |

**Absorbable Details:**
- 43 tools across 12 categories (file, search, notebook, agent, task, MCP, mode, schedule, meta)
- Compatible with anthropics/skills & plugins ecosystems
- Multi-channel: Feishu, Slack, Telegram, Discord
- Path-level & command rules for governance
- Interactive approval dialogs for high-risk operations

---

## 11. Prime Agent (PrimeIntellect)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/PrimeIntellect-ai/prime-agent |
| **Stars** | 1,456 |
| **Language** | TypeScript |
| **Core Architecture** | Recursive Language Model (RLM) + Continual Harness. Prompt-as-variable + programmatic tool/sub-agent calling in persistent REPL. `/refine` for evidence-backed harness updates. |
| **Key Pattern** | **Continual Harness** — durable state (supplemental prompts, memories, skill descriptions, subagent specs) that agent refines through small, evidence-backed updates. Never rewrites immutable base system prompt. |
| **NeoTrix Domain** | NT-MIND (harness refinement) + NT-CORE (RLM loop) |
| **Absorption Priority** | **P1** |
| **NeoTrix Target** | `nt_mind/src/harness/refinement.rs` — `/refine` pattern for evidence-backed harness state updates with rollback support. |

**Absorbable Details:**
- `rlm(...)` spawns real child agents for parallel/background work
- `/refine` reviews trajectory, applies small evidence-backed updates to harness state
- Never rewrites immutable base prompt — only supplements
- Recorded refinement history for rollback
- Daemon-backed: sessions survive terminal disconnect
- Direct agent-to-agent communication without user routing
- Bounded autonomous mode with turn/token/time budgets + quality gates

---

## 12. Jig — Pre-Execution Agent Firewall

| Field | Value |
|-------|-------|
| **URL** | https://github.com/luyi14-bits/agent-harness |
| **Stars** | 187 |
| **Language** | Python |
| **Core Architecture** | First pre-execution agent firewall. Intercepts tool calls before execution with whitelist+denylist hard constraints. 11 preset agents. CostAwareRouter. |
| **Key Pattern** | **Pre-execution intercept** — code-level firewall between model decision and tool execution. DeepSeek cache optimization via SHA-256 prefix. GraphOrch for orchestration. |
| **NeoTrix Domain** | NT-SHIELD (firewall) + NT-ACT (tool governance) |
| **Absorption Priority** | **P2** |
| **NeoTrix Target** | `nt_shield/src/firewall/pre_execution.rs` — intercept tool calls before execution. |

**Absorbable Details:**
- Pre-execution intercept: unique among all harnesses surveyed
- Whitelist+Denylist for hard constraints (not prompt-based soft rules)
- 4-layer memory system
- Meta-Harness for external agent governance
- LoopEngine for orchestration

---

## 13. DeepSeek Harness

| Field | Value |
|-------|-------|
| **URL** | https://github.com/deepseek-ai/deepseek-harness |
| **Stars** | 56.8k |
| **Language** | Not specified (trending Aug-Sep 2026) |
| **Core Architecture** | DeepSeek's official agent harness. Trending #1 on GitHub for multiple weeks. |
| **Key Pattern** | **To investigate further** — massive adoption signal (56.8k stars in weeks). |
| **NeoTrix Domain** | NT-IO (provider integration) |
| **Absorption Priority** | **P1** |
| **NeoTrix Target** | Study architecture patterns when source is available. |

---

## Arxiv Papers — Key Architectural Patterns

### Paper 1: Architectural Design Decisions in AI Agent Harnesses (2604.18071)
- **Source**: Survey of 70 agent-system projects
- **5 Recurring Dimensions**: subagent architecture, context management, tool systems, safety mechanisms, orchestration
- **Key Finding**: File-persistent, hybrid, and hierarchical context strategies dominate. Registry-oriented tool systems dominant, MCP/plugin emerging. Deeper coordination → more explicit context services. Stronger execution → more structured governance.
- **NeoTrix Mapping**: Validates NT-CORE's 6-layer architecture. Suggests NT-MEMORY should implement hierarchical context (not just flat KV store).

### Paper 2: Harness Engineering as Categorical Architecture (2605.12239)
- **Source**: ArchAgents framework formalization
- **Key Insight**: Architecture triple (G, Know, Φ) = syntactic wiring + structural knowledge + deployment maps. Structural guarantees are Know-level certificates preserved across framework compilation.
- **NeoTrix Mapping**: Maps to NT-CORE's trait-based architecture. Structural replay = our const-monomorphization guarantees. Certificate preservation = compile-time verification.

### Paper 3: From Model Scaling to System Scaling (2605.26112)
- **Source**: CheetahClaws 2 reference harness
- **6 Components**: Reasoning substrate (R), Memory store (M), Context constructor (C), Skill-routing layer (S), Orchestration loop (O), Verification/governance (G)
- **3 Bottlenecks**: context governance, trustworthy memory, dynamic skill routing
- **Key Insight**: "Future progress depends as much on system design as on stronger foundation models"
- **NeoTrix Mapping**: Direct alignment with NT-CORE (R,O,G) + NT-MEMORY (M,C) + NT-ACT (S)

### Paper 4: Natural-Language Agent Harnesses (2603.25723)
- **Source**: NLAH + IHR framework
- **Key Insight**: Harness policy as editable natural language documents, executed by shared runtime. Division of labor: NL for policy, code for mechanisms.
- **NeoTrix Mapping**: AGENTS.md IS our NLAH. Validate that our approach is research-backed.

### Paper 5: HarnessDev — LLMs Creating Their Own Harness (2609.01437)
- **Source**: Sep 2026 benchmark
- **Key Finding**: Generated harnesses match/exceed references on writing and ML experimentation, but lag on code and search. Evolution gains unstable and partially transfer.
- **NeoTrix Mapping**: Supports SEAL pipeline's iterative approach. Validates that harness evolution needs verifiable environments (SoL-Pi pattern).

### Paper 6: HEART — Agent-Native Reusable Tool Primitives (2609.01736)
- **Source**: ToolPrimitives + ToolFace + HEART framework
- **Key Pattern**: Natural language as tool interface (not schema-based). ToolFace: 25,519 functions with dynamic retrieval. HEART = Planner + Router + Verifier.
- **NeoTrix Mapping**: Tool interface design for NT-ACT. Dynamic tool retrieval for large tool catalogs.

### Paper 7: Diverse Skill Routing via DPP (2609.05824)
- **Source**: Determinantal Point Process for skill routing
- **Key Pattern**: Diversity-aware reranking balances relevance + non-redundancy. Query-residual diversity kernel penalizes redundant skill overlap.
- **NeoTrix Mapping**: NT-CORE's GWT attention routing — apply DPP for skill/tool selection to avoid redundant tool calls.

### Paper 8: TRIAGE — Trajectory-as-a-Skill (2609.01428)
- **Source**: 3-level routing (Direct Reuse → Skill Substitution → Full ReAct)
- **Key Pattern**: TaaS abstracts historical trajectories into reusable skills. 62.3% token savings. 56% of queries at Level 2 (0 tokens via parameter substitution).
- **NeoTrix Mapping**: NT-MEMORY experience storage — store trajectories as skills, not just summaries.

### Paper 9: CacheRouter (2608.22708)
- **Source**: Dual-path tool routing with cache preservation
- **Key Pattern**: Fixed core tool set (stable prefix) + dynamic long-tail via INTERNAL_ROUTER pseudo-tool. 90.99% token-level cache hit rate. Tool growth doesn't linearly grow context.
- **NeoTrix Mapping**: NT-IO LLM cache optimization — stable core tools + on-demand long-tail routing.

### Paper 10: ACM — Agentic Context Management (2607.23809)
- **Source**: Manage_context + query_memory tools
- **Key Pattern**: Agent-initiated compression (not heuristic triggers). Lossless: originals saved to external storage, summaries with IDs for retrieval. 27% improvement on BrowseComp-Plus.
- **NeoTrix Mapping**: NT-CORE context management — agent decides when to compress, not the framework.

### Paper 11: ContextPilot (2608.28476)
- **Source**: Fine-grained RL for context management
- **Key Pattern**: Context-aware partial rollout + action-level credit assignment. Extended toolset: memorize, readMemory, summarizeContext, compressContext, foldHistory.
- **NeoTrix Mapping**: NT-CORE — context management as RL problem, not heuristic.

### Paper 12: Scroll — Context as Environment (2608.21690)
- **Source**: Append-only Event Log + sandboxed persistent Python kernel
- **Key Pattern**: Typed namespace across model calls, variable binding instead of serialization. Eviction index with compact landmarks for recovery. 94.8% on LongMemEval_S.
- **NeoTrix Mapping**: NT-MEMORY — event log + variable binding pattern for session state.

### Paper 13: ContextPipe (2609.00749)
- **Source**: Database-inspired context assembly
- **Key Pattern**: 5-phase pipeline (Plan→Bind→Optimize→Execute→Feedback) with structured data-source catalog, deterministic cache-aware optimizer, EXPLAIN ANALYZE trace. 31% token reduction.
- **NeoTrix Mapping**: NT-CORE — treat context assembly as query execution problem.

---

## Cross-Source Synthesis: Top 5 Absorbable Patterns

| # | Pattern | Source | NeoTrix Mapping | Priority |
|---|---------|--------|-----------------|----------|
| 1 | **3-Cadence Hook System** | Grok Build, harness-pi, OpenHarness | NT-CORE agent loop — hooks at session/turn/tool level | P0 |
| 2 | **Pass-by-Reference Tool Results** | NOOA | NT-ACT — live object previews instead of serialized dumps | P0 |
| 3 | **Cache-Aware Compaction Boundaries** | harness-pi, CacheRouter | NT-IO LLM cache — stable prefix across compaction | P0 |
| 4 | **Agent-Initiated Context Management** | ACM, ContextPilot | NT-CORE — agent decides when to compress, not framework | P1 |
| 5 | **Trajectory-as-a-Skill** | TRIAGE, GenericAgent | NT-MEMORY — store trajectories as reusable skills | P1 |

---

## Trendshift Top 5 AI Agent Repos This Week (Sep 2026)

| # | Repo | Stars | Language | Key Pattern |
|---|------|-------|----------|-------------|
| 1 | deepseek-ai/deepseek-harness | 56.8k | — | DeepSeek official harness |
| 2 | mattpocock/skills | 14.5k+ | Shell | Skills for real engineers |
| 3 | affaan-m/ECC | 242k+ | JS | Agent harness optimization system |
| 4 | openai/skills | — | — | OpenAI skills |
| 5 | humanlayer/skills | — | — | Human-in-the-loop skills |

## GitHub Trending (Sep 2026)

| # | Repo | Stars | Language | Key Pattern |
|---|------|-------|----------|-------------|
| 1 | openai/codex | 113.9k | Rust | Sandboxed tool-call loop |
| 2 | mattpocock/skills | 232.5k | Shell | Skill ecosystem |
| 3 | affaan-m/ECC | 242.2k | JS | Harness performance optimization |
| 4 | obra/superpowers | 276.3k | Shell | Agentic skills framework |
| 5 | anthropics/claude-code | 142.6k | Python | Reference agent harness |

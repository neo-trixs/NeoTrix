# Trending Rankings — Cycle 352 (2026-09-12)

## Search Scope

GitHub Trending (weekly), ProductHunt (Sep 6-12 2026), autonomous-agents topics, research papers. Excluded projects present in cycles 318–351.

---

## 10 New Projects

### 1. causaldynamics-research/reasonara

| Field | Value |
|-------|-------|
| **URL** | https://github.com/causaldynamics-research/reasonara |
| **Stars** | New (trending in agent-memory) |
| **Language** | Python |
| **Category** | Graph-Structured Causal Memory |

**What it does**: Graph-structured causal memory system for agentic environments. Constructs persistent causal graphs from heterogeneous enterprise data (code, docs, conversations, traces). Retrieval is modeled as an MDP with three signals: semantic search, BM25 keyword, and graph traversal. Trained via GRPO on downstream answer quality. Introduces "cognitive anchors" that anticipate future query contexts for memories semantically distant from content.

**Novel Pattern**: Causal memory as MDP — retrieval is not lookup but sequential decision-making. Cognitive anchors solve "contextual isolation" where the right answer depends on latent constraints, not direct factual recall. Deferred memory construction decouples ingest latency from construction quality.

**NeoTrix Mapping**: NT-MEMORY (KB retrieval), NT-CORE (reasoning). Maps to experience-tree — cognitive anchors could replace heuristic route-table matching with learned retrieval policies. MDP-based retrieval aligns with GWT attention routing (sequential evidence gathering). Deferred construction validates lazy branch loading.

---

### 2. EverOS OSS (evermind.ai)

| Field | Value |
|-------|-------|
| **URL** | https://docs.evermind.ai/open-source/overview |
| **Stars** | New (Sep 2026 launch) |
| **Language** | Python |
| **Category** | Memory Framework (Markdown-First) |

**What it does**: Lightweight, markdown-first, edge-cloud isomorphic memory framework for AI agents. Core design: Markdown files are source of truth, SQLite for state, LanceDB for index. Three memory tiers: User Memory, Agent Memory, Wiki Memory. Zero external services, human-readable files, offline-capable edge deployment, cloud migration without code changes.

**Novel Pattern**: Markdown-as-truth — memory is human-readable, version-controllable, and portable. Edge-to-cloud isomorphism: same code runs on a laptop or in the cloud with zero migration effort. mRAG (multi-granularity RAG) selects the right memory at the right granularity automatically.

**NeoTrix Mapping**: NT-MEMORY (KB storage), NT-PHYSICAL (edge). Validates markdown-as-memory approach — AGENTS.md and CONTEXT.md already follow this pattern. Edge-cloud isomorphism maps to NT-PHYSICAL body schema (local inference → cloud fallback). mRAG aligns with GWT multi-granularity attention.

---

### 3. AIDC-AI/Pixelle-Video

| Field | Value |
|-------|-------|
| **URL** | https://github.com/AIDC-AI/Pixelle-Video |
| **Stars** | ~5K+ (trending in video generation) |
| **Language** | Python |
| **Category** | End-to-End Video Pipeline |

**What it does**: Topic-in, video-out pipeline. Type a topic → get a finished video. Orchestrates: script writing (LLM), AI-generated visuals (image/video models), voice synthesis (TTS), background music, final composition. Zero editing experience needed. The whole pipeline, not just generation.

**Novel Pattern**: Pipeline-as-product — not a generator but an orchestrator. Script → Visuals → Voice → Music → Composition as a single atomic operation. The differentiator from "Sora but worse" is the orchestration layer, not the generative model.

**NeoTrix Mapping**: NT-ACT (orchestration), NT-WORLD (content extraction). Maps to ProductionOrchestrator — shows that multi-stage AI pipelines can be a single user-facing product. Validates "The Spice Must Flow" axiom: every module has clear input→transform→output. Pipeline composition aligns with SEAL stage architecture.

---

### 4. Definable (definable.ai)

| Field | Value |
|-------|-------|
| **URL** | https://docs.definable.ai/introduction |
| **Stars** | New (Sep 2026) |
| **Language** | Python |
| **Category** | Agent Framework (Production-Grade) |

**What it does**: Production-grade Python framework for building AI agents. Composable primitives: models, tools, knowledge, memory, guardrails. Execution engine orchestrates them into reliable agentic systems. 10 LLM providers, async-first, token counting, cost tracking, native tracing, full type annotations. Supports Teams (multi-agent coordination) and Workflows (sequential/parallel/conditional steps).

**Novel Pattern**: Composable primitive architecture — agents are assembled from typed building blocks, not monolithic configs. Guardrails and security are first-class, not afterthoughts. Workflow engine supports conditional/iterative steps, not just linear chains.

**NeoTrix Mapping**: NT-ACT (orchestration), NT-IO (provider routing). Validates composable architecture approach. 10-provider support + automatic failover extends Ordered Backend Router pattern. Guardrails-as-primitives aligns with NT-SHIELD egress privacy guard. Workflow engine maps to SEAL pipeline conditional branching.

---

### 5. Mastra Factory (mastra.ai)

| Field | Value |
|-------|-------|
| **URL** | https://mastra.ai |
| **Stars** | #1 ProductHunt Sep 9, 2026 (491 points) |
| **Language** | TypeScript |
| **Category** | AI Agent Framework |

**What it does**: From the team behind Gatsby. Framework for building AI-powered apps and agents with workflows, memory, streaming, evals, tracing, and Studio (interactive UI for dev and testing). `npm create mastra@latest` to start. Backed by Gatsby engineering heritage.

**Novel Pattern**: Studio-as-development-surface — interactive UI for building, testing, and debugging agents. Not just a CLI framework but a visual development environment. Evals and tracing built-in from day one, not bolted on post-hoc.

**NeoTrix Mapping**: NT-IO (development interface), NT-MIND (evaluation). Studio pattern validates interactive development for complex agent systems. Built-in evals aligns with SelfTest T3 (production wiring). Tracing maps to ConvergenceCheck provenance tracking.

---

### 6. GoModel (gomodel.enterpilot.io)

| Field | Value |
|-------|-------|
| **URL** | https://gomodel.enterpilot.io |
| **Stars** | 126 points on ProductHunt (Sep 9, 2026) |
| **Language** | Go |
| **Category** | AI Gateway / Model Router |

**What it does**: Open-source OpenRouter alternative in Go. One OpenAI-compatible API for every provider, with budgets, caching, guardrails, load balancing, and failover. Single binary, ~20MB Docker image, MIT license. Self-hosted alternative to OpenRouter and LiteLLM.

**Novel Pattern**: Single-binary gateway — no dependencies, no infrastructure. Budget management, caching, and guardrails at the gateway level, not per-application. Load balancing with failover as a built-in primitive, not an add-on.

**NeoTrix Mapping**: NT-IO (provider routing). Direct analog to Ordered Backend Router. Go single-binary validates NeoTrix's Rust single-binary approach. Budget + caching + guardrails at gateway level maps to Cost-Aware Routing (A1) + NT-SHIELD egress guard. Extends Switchyard (cycle 351) with production hardening.

---

### 7. modeinspect

| Field | Value |
|-------|-------|
| **URL** | https://modeinspect.com |
| **Stars** | 127 points on ProductHunt (Sep 10, 2026) |
| **Language** | TypeScript |
| **Category** | AI Design Canvas |

**What it does**: AI design canvas for working on the real product. Connect codebase, open existing screen, design with components, tokens, live data, states, and breakpoints. Explore with AI. Publish changes live or send to engineering for review. The canvas is not the destination — the product is.

**Novel Pattern**: Codebase-connected design — not a mockup tool but a live design surface connected to real components, tokens, and data. Design-in-context eliminates the "design says one thing, code does another" gap. Breakpoints in design (not just code).

**NeoTrix Mapping**: NT-IO (visualization), NT-CORE (reasoning). Design-as-code paradigm maps to ConvergenceCheck — design intent is traceable to implementation. Live data in design surface validates "The Spice Must Flow" (no disconnects between design and runtime).

---

### 8. Hyperprobe (hyperprobe.dev)

| Field | Value |
|-------|-------|
| **URL** | https://hyperprobe.dev |
| **Stars** | 208 points on ProductHunt (Sep 5, 2026) |
| **Language** | — |
| **Category** | Agent Debugging Tool |

**What it does**: Lets AI agents debug production without redeploying. Drop read-only probes into running services, capture variable state never recorded. Agents debug like they have a local repro, closing bugs in one sitting. Works with Claude Code, Codex, Cursor.

**Novel Pattern**: Agent-native observability — probes are read-only, safe, and agent-consumable. Not log-based debugging but variable-state capture at the agent's request. "Debug like local repro" without actual local repro.

**NeoTrix Mapping**: NT-SHIELD (security), NT-CORE (debugging). Read-only probes align with NT-SHIELD safety kernel (no mutation in production). Agent-consumable debug data maps to ConvergenceCheck — structured evidence from production, not ad-hoc log parsing. Extends HeartbeatAggregator with targeted diagnostic probes.

---

### 9. Browzer (browzer.com)

| Field | Value |
|-------|-------|
| **URL** | https://browzer.com |
| **Stars** | ProductHunt Sep 2026 |
| **Language** | — |
| **Category** | Technical Content Automation |

**What it does**: Put your technical content on autopilot. Automated technical content pipeline — from codebase analysis to published documentation. Agent-driven content generation from source code and architecture.

**Novel Pattern**: Codebase-to-content pipeline — technical documentation generated from actual code, not manually maintained. Agent-driven means content stays current as code evolves, not stale docs that drift.

**NeoTrix Mapping**: NT-WORLD (content extraction), NT-IO (documentation). Maps to NT-WORLD crawl pipeline — technical content as a crawl target. Validates "The Spice Must Flow" for documentation: clear input (code) → transform (analysis) → output (docs). Could integrate with experience-tree for auto-generated session reports.

---

### 10. PrimeIntellect-ai/prime-agent

| Field | Value |
|-------|-------|
| **URL** | https://github.com/PrimeIntellect-ai/prime-agent |
| **Stars** | 1,456 (new, trending) |
| **Language** | Python |
| **Category** | Self-Improving RLM Agent |

**What it does**: Self-improving RLM (Reinforcement Learning from Model feedback) agent for coding workflows. Two core abstractions: Recursive Language Model (context as variables, tools as recursive sub-agents) and Continual Harness (durable state for prompts, memories, skills). `/refine` applies small, evidence-backed updates to harness state. Daemon-backed sessions survive terminal disconnect.

**Novel Pattern**: Prompt-as-a-variable — context is programmable, not static. Continual Harness stores durable state that improves through small, reviewed updates (never rewrites base prompt). Skills are executable Python packages, not just prompts.

**NeoTrix Mapping**: NT-MIND (self-evolution), NT-MEMORY (persistent state). Continual Harness maps to experience-tree — durable state that improves through reviewed updates. `/refine` as evidence-backed updates validates AGENTS.md指针守恒 (pointer conservation). Daemon-backed sessions map to NT-NEXUS cross-session memory. Self-improvement via execution traces extends SEAL pipeline with online learning.

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Implication |
|---------|----------|---------------------|
| **Causal/MDP Memory** | Reasonara, EverOS | Memory retrieval as sequential decision-making — extend GWT with learned retrieval policies |
| **Pipeline-as-Product** | Pixelle-Video, Mastra | Multi-stage AI pipelines as single user-facing products — validates SEAL stage architecture |
| **Gateway Hardening** | GoModel, Switchyard (351) | Budget + caching + guardrails at gateway level — Cost-Aware Routing (A1) + egress guard |
| **Codebase-Connected** | modeinspect, Browzer | Design and docs connected to live code — "Spice Must Flow" for all artifacts |
| **Self-Improvement** | prime-agent, Reasonara | RL-learned + evidence-backed updates — SEAL could adopt learned reward signals |
| **Agent Debugging** | Hyperprobe | Read-only production probes for agents — extends NT-SHIELD safety kernel |

---

## Signal Strength

- **Strongest trend**: Causal/MDP memory (Reasonara, EverOS) — retrieval as decision-making, not lookup
- **Fastest growing**: Mastra Factory (#1 ProductHunt Sep 9) — Gatsby team's agent framework
- **Most relevant to NeoTrix**: GoModel — single-binary gateway with budget/caching/guardrails validates Rust-first approach
- **Highest risk/reward**: Reasonara — MDP-based retrieval could revolutionize KB but needs training infrastructure
- **Most production-ready**: Definable — composable primitives with guardrails, 10 providers, async-first

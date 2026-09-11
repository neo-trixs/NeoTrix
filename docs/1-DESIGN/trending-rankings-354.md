# Trending Rankings — Cycle 354

**Date**: 2026-09-12  
**Focus**: AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns  
**Sources**: GitHub Trending, ProductHunt, arXiv, SWEN.ai, Firecrawl, GitTrends AI

---

## Top 10 New Projects (Not in Cycles 318-353)

### 1. DeepSeek Harness — Plugin-Based Agent Runtime
- **GitHub**: [deepseek-ai/harness](https://github.com/deepseek-ai/harness) — ★ 152.1K+ (Aug 2026 growth), Apache-2.0
- **What**: Agent harness — the runtime layer around models, tools, skills, sessions, interfaces, storage, and execution. Not a foundation model but the infrastructure that makes agents operational. Supports plugin architecture for extensible agent capabilities.
- **Key Pattern**: Harness-as-platform — separates agent logic from agent runtime. Plugins for tools, memory, routing, and governance are swappable. Sessions, storage, and execution are first-class runtime concerns, not application code.
- **NeoTrix Relevance**: Validates NT-ACT as a harness layer (not just a tool registry). The plugin architecture maps to Rune Socketing (Crimson/Indigo/Obsidian/Golden/Alabaster slots). Session management aligns with NT-NEXUS cross-session memory. The separation of "runtime" from "agent logic" is the architectural insight — NeoTrix should treat its runtime (KB, EventBus, GWT) as a harness that plugins slot into, not monolithic core code.
- **Stars**: 152.1K+ | **License**: Apache-2.0 | **URL**: deepseek-ai/harness

### 2. Mastra Factory — AI Agent Framework with Workflows, Memory, Evals
- **ProductHunt**: #1 Product of the Day (Sep 9, 2026) — Score 491
- **What**: From the Gatsby team. Framework for building AI-powered apps and agents with workflows, memory, streaming, evals, tracing, and Studio (interactive UI for dev and testing). `npm create mastra@latest`.
- **Key Pattern**: Full-lifecycle agent development — workflow builder + memory + evals + tracing in one framework. Studio UI for interactive debugging. The "evals + tracing" combination means agents are observable from day one, not bolted on later.
- **NeoTrix Relevance**: The evals-first philosophy validates SEAL pipeline's self-test tiers (T1/T2/T3). The Studio UI pattern maps to NT-IO agent transparency — agents should have a visual debugging surface. The workflow + memory + evals trifecta is the minimal viable agent stack; NeoTrix should ensure these three are always coupled. Tracing integration aligns with EventBus event_log (R-P84).
- **URL**: mastra.ai | **License**: Open Source

### 3. Harden AIF — Security Layer for AI Coding Agents
- **ProductHunt**: #2 Product of the Day (Sep 9, 2026) — Score 405
- **What**: Free, local security tool for AI coding agents. Post-trained model checks tool calls before they run, using request and session context. Beat frontier models on agent-security benchmarks. Keeps repo and tool output on your machine.
- **Key Pattern**: Pre-execution tool call validation — not sandboxing, not post-hoc audit, but a trained model that evaluates each tool call against session context BEFORE execution. Local-first (no data leaves machine).
- **NeoTrix Relevance**: Directly maps to NT-SHIELD egress guard. The "post-trained security model" pattern extends beyond policy rules — a small model learned from attack patterns could replace hand-written policies. Session-context-aware validation (not just static rules) is the key insight for NT-SHIELD: the same tool call might be safe in one session context and dangerous in another. Local-first aligns with NT-SHIELD trust tiers (Trusted/Contracted/Untrusted).
- **URL**: harden.run | **License**: Free, local

### 4. OmniRoute — AI Routing Gateway with Fallback and Load Balancing
- **GitHub**: [omniroute/omniroute](https://github.com/omniroute/omniroute) — ★ 16.8K+ (Aug 2026 growth)
- **What**: Single AI gateway in front of many model providers. Hundreds of providers, 1000+ model endpoints. Routing strategies, automatic fallback, quota management, token compression, MCP, A2A, desktop interfaces, and developer-tool integrations.
- **Key Pattern**: Gateway-as-infrastructure — routing, fallback, quota, compression all in one layer. The "ordered backend fallback" pattern (castor, Better-Fullstack) validated at scale. Token compression at the gateway level (not per-app).
- **NeoTrix Relevance**: Extends NT-IO Ordered Backend Router pattern (P4 from absorbed terminology). The token compression at gateway level is a new pattern — NeoTrix could add compression as a gateway concern before requests hit providers. Quota management maps to ResourceBudgetManager. A2A (Agent-to-Agent) protocol support validates multi-agent coordination needs. The "1000+ endpoints" scale validates the need for a proper router, not manual provider selection.
- **Stars**: 16.8K+ | **License**: Open Source | **URL**: omniroute.io

### 5. PlugMem — Plug-and-Play Long-Term Memory for LLM Agents
- **GitHub**: [TIMAN-group/PlugMem](https://github.com/TIMAN-group/PlugMem) — ICML 2026
- **What**: Task-agnostic long-term memory system. Organizes experience into compact, reusable knowledge units (not raw interaction histories). Three memory types: Semantic (facts), Procedural (workflows), Episodic (interactions). Graph structure with LLM-enhanced retrieval. Ships as plugins for OpenClaw and Claude Code.
- **Key Pattern**: Knowledge units over raw histories — distills interactions into reusable semantic/procedural/episodic nodes. Memory Inspector UI for visualizing the memory graph. 90.2 Acc on LongMemEval, 79.1 F1 on HotpotQA (SOTA).
- **NeoTrix Relevance**: The three memory types (Semantic/Procedural/Episodic) map directly to NT-MEMORY's knowledge representation: KB nodes (semantic), SEAL skill crystallization (procedural), experience-tree sessions (episodic). The "compact knowledge units over raw history" principle validates NeoTrix's experience distillation (snapshot→distill→classify→persist→feedback). The Memory Inspector UI is a template for NT-IO KB visualization. ICML 2026 validation means this is peer-reviewed, not just star-count popular.
- **Stars**: Growing (ICML 2026) | **License**: Apache-2.0 | **URL**: github.com/TIMAN-group/PlugMem

### 6. Cortex — Neuroscience-Based Persistent Memory for Claude Code
- **GitHub**: [cdeust/Cortex](https://github.com/cdeust/Cortex) — ★ Growing, AGPL-3.0
- **What**: Persistent memory engine built on 36 neuroscience mechanisms. Consolidates what matters, keeps it current, reconstructs right context at right time. 52 memory tools, 9 lifecycle hooks. SQLite/PostgreSQL backend. Hippocampal Replay for compaction survival.
- **Key Pattern**: Neuroscience-grounded memory — encoding→consolidation→retrieval→ forgetting with a cited mechanism at every stage. BEAM benchmark validated (10 conversations × 10M tokens). "I don't know" as a first-class response. Temporal day-level partitioning outperforms topic labels.
- **NeoTrix Relevance**: The 36-mechanism neuroscience model is a reference architecture for NT-MEMORY. Hippocampal Replay (compaction checkpoint + reconstruction) validates experience-tree's lazy branch loading. "I don't know" as a first-class response maps to uncertainty signaling in SelfModel. Temporal partitioning over topic partitioning is an actionable insight for KB indexing — time-based clustering may outperform semantic clustering for session memory. The five-signal fusion (vector, FTS, trigram, heat, recency) is a retrieval template.
- **Stars**: Growing | **License**: AGPL-3.0 | **URL**: github.com/cdeust/Cortex

### 7. M-Flow — Cognitive Memory Engine with Cone Graph Architecture
- **GitHub**: [LiuYihey/m_flow](https://github.com/LiuYihey/m_flow) — ★ 2, Apache-2.0
- **What**: Cognitive memory engine that reasons through relationships, not surface similarity. Four-level Cone Graph (abstract summaries → atomic facts). Graph-routed retrieval: cast wide net, project into KG, propagate cost, score by tightest evidence chain. 50+ file formats, 5 retrieval modes, MCP server.
- **Key Pattern**: "One strong path is enough" — retrieval scores by the tightest chain of evidence, not aggregate similarity. Cone Graph hierarchy: Episode → Summary → Fact → Detail. Five retrieval modes: Episodic, Procedural, Triplet Completion, Lexical, Cypher.
- **NeoTrix Relevance**: The "tightest evidence chain" scoring is a retrieval paradigm shift — instead of returning top-K similar chunks, find the single strongest reasoning path. This maps to GWT's salience routing (broadcast the most relevant path, not all candidates). The four-level Cone Graph mirrors NeoTrix's experience hierarchy (hub index → branch summary → full experience → raw session). The multi-backend support (LanceDB, Neo4j, PostgreSQL, ChromaDB, KuzuDB, Pinecone) validates the adapter pattern in NT-MEMORY.
- **Stars**: 2 | **License**: Apache-2.0 | **URL**: m-flow.ai

### 8. Noodle Seed — Governed Runtime for AI Agent Workflows
- **ProductHunt**: #4 Product of the Day (Sep 9, 2026) — Score 254
- **What**: Helps software teams make products ready for AI agents. Build workflows in TypeScript, expose through secure branded assistant inside product, make capabilities available to external agents. Governed runtime for identity, permissions, secrets, audit, and operations.
- **Key Pattern**: Governance-as-runtime — identity, permissions, secrets, audit are baked into the runtime, not application code. "Branded assistant" inside your product + external agent access through the same interface.
- **NeoTrix Relevance**: The "governance as runtime" pattern maps to NT-GOVERNANCE + NT-SHIELD integration. Instead of bolting governance onto agents, it should be the substrate they run on. The dual-mode interface (internal branded assistant + external agent access) is a template for NT-IO. Secrets management (runtime-injected, never seen by model) aligns with NT-SHIELD egress guard secret scrubbing.
- **URL**: noodleseed.com | **License**: SaaS

### 9. MagiCrew — Multi-Agent Research and Analysis Platform
- **ProductHunt**: #3 Product of the Day (Sep 3, 2026) — Score 269
- **What**: Open-source platform for assembling specialized AI agents for research, analysis, reporting, and presentations. Six agents in marketplace. Apache 2.0 for self-hosting.
- **Key Pattern**: Agent marketplace with role-specialized agents. Self-hostable. The "assemble from marketplace" pattern — don't build agents, compose them from pre-built specialists.
- **NeoTrix Relevance**: The agent marketplace pattern maps to NT-ACT capability registry + CapabilityRegistry. Role-specialized agents validate the "Ascending dual specialization" pattern (two Weapon Sets per session). The marketplace composition model is a UX template for NT-IO — users should browse and compose agent capabilities, not configure them from scratch. Apache 2.0 licensing aligns with open-source philosophy.
- **URL**: ProductHunt | **License**: Apache 2.0

### 10. Archify — Codebase-to-Diagram Agent Skill
- **GitHub**: [archify/archify](https://github.com/archify/archify) — ★ 28.7K+ (Aug 2026 growth)
- **What**: Agent skill that turns codebases and systems into verifiable technical maps. Architecture, workflow, sequence, data flow, and lifecycle views. Not just visualization — diagrams are verified against actual code structure.
- **Key Pattern**: Verified diagrams — not LLM-generated guesses, but diagrams validated against code structure. The "agent skill as a product" pattern — a single, well-defined capability packaged for reuse.
- **NeoTrix Relevance**: Verified diagrams map to SelfTest T3 (production wiring) — architecture diagrams should be validated against actual module structure, not hand-drawn. The "skill as product" pattern validates SKILL-SPEC.md contract (<200 lines). The diagram types (architecture, workflow, sequence, data flow, lifecycle) are the same views NeoTrix's ConsciousnessTree should expose. This could be integrated as an NT-CORE diagnostic tool — auto-generate architecture diagrams and diff against expected structure.
- **Stars**: 28.7K+ | **License**: Open Source | **URL**: archify.dev

---

## Pattern Summary

| Pattern | Frequency | NeoTrix Domain |
|---------|-----------|----------------|
| **Agent harness/runtime** | 2/10 (DeepSeek Harness, Mastra) | NT-ACT + NT-IO |
| **Memory as first-class product** | 3/10 (PlugMem, Cortex, M-Flow) | NT-MEMORY |
| **Pre-execution security** | 2/10 (Harden AIF, Noodle Seed) | NT-SHIELD + NT-GOVERNANCE |
| **Routing/gateway infrastructure** | 1/10 (OmniRoute) | NT-IO |
| **Verified architecture** | 1/10 (Archify) | NT-CORE + NT-REPAIR |
| **Agent marketplace/composition** | 2/10 (MagiCrew, Mastra) | NT-ACT |
| **Neuroscience-grounded design** | 1/10 (Cortex) | NT-MEMORY + NT-FEEL |

---

## Key Insight

**Memory is eating the agent stack.** Three of the top ten projects (PlugMem, Cortex, M-Flow) are dedicated memory systems — not memory features bolted onto agents, but memory-first architectures where the agent is a client of the memory layer. This validates NeoTrix's KB-first architecture but suggests the market is moving faster than NeoTrix's current memory implementation.

The second major pattern is **"harness-as-platform"** — DeepSeek Harness (+152K stars) and Mastra Factory (#1 PH) both treat the agent runtime as a separate product from the agent itself. This is the architectural insight: the runtime (KB, EventBus, GWT, routing) should be a standalone platform that agents plug into, not embedded in agent code.

**Actionable for NeoTrix**: (1) Accelerate NT-MEMORY — the market is converging on "memory as infrastructure" and NeoTrix's KB is the right foundation but needs the knowledge-unit distillation layer (PlugMem) and neuroscience-grounded consolidation (Cortex). (2) Consider extracting the runtime into a standalone harness — KB + EventBus + GWT + routing as a platform that NT-ACT agents and external agents both consume.

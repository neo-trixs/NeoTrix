# Trending Rankings — Cycle 416 (2026-09-12)

**Sources**: GitHub Trending, ProductHunt LLM Developer Tools, arXiv, web search
**Focus**: AI agents, LLM tools, reasoning frameworks, novel patterns for memory/attention/routing
**Filter**: Projects NOT in cycles 318–415

---

## Top 10 New Projects

### 1. Kilo Code — All-in-one Agentic Engineering Platform
- **URL**: https://github.com/kilo-org/kilocode
- **Stars**: 27,195 | **Lang**: TypeScript | **License**: MIT
- **Description**: Open-source coding agent for VS Code, JetBrains, and CLI. 500+ models, zero markup, self-checking, terminal/browser control, MCP marketplace.
- **Key Pattern**: Multi-interface sync (VS Code ↔ JetBrains ↔ CLI ↔ Cloud), session persistence across devices, custom agent modes (Code/Plan/Ask/Debug/Orchestrator).
- **NeoTrix Mapping**: NT-ACT (agent execution), NT-IO (multi-interface), NT-MEMORY (session state).
- **Why Notable**: #1 Open Source Product of Month (ProductHunt), 3M+ users, 40T+ tokens processed. Forked OpenCode CLI into Kilo CLI. Marketplace for Skills/MCP servers.

### 2. Mozaik — TypeScript Runtime for Concurrent AI Agents
- **URL**: https://github.com/jigjoy-ai/mozaik
- **Stars**: 127 | **Lang**: TypeScript | **License**: MIT
- **Description**: Event-driven runtime where agents work independently, communicate via semantic events, coordinate without predefined workflows.
- **Key Pattern**: Typed event handlers (onMessage/onFunctionCall/onReasoning/onModelMessage), fire-and-forget inference, agent awareness via participant registry, adaptive runtime coordination.
- **NeoTrix Mapping**: NT-CORE (event bus), NT-ACT (agent coordination), NT-MEMORY (runtime state).
- **Why Notable**: Eliminates "agentic waterfall" — agents don't need predefined handoff graphs. Collaboration emerges at runtime. Supports agent swarms.

### 3. oMLX — Mac LLM Server with SSD KV Cache
- **URL**: https://github.com/jundot/omlx
- **Stars**: ~20,000 | **Lang**: Python | **License**: Apache 2.0
- **Description**: Apple Silicon-native LLM inference server. Two-tier KV cache (RAM hot + SSD cold) persists across restarts. Cuts Claude Code wait from 90s to 5s.
- **Key Pattern**: Persistent KV cache on SSD, continuous batching (4.14x speedup at 8x concurrency), multi-model serving (LLM+VLM+embedding+reranker), native macOS menu bar app.
- **NeoTrix Mapping**: NT-IO (local inference), NT-MEMORY (KV persistence), NT-PHYSICAL (Apple Silicon optimization).
- **Why Notable**: Solves the critical problem of local coding agent KV cache invalidation. Distributed mode splits model across 2 Macs via Thunderbolt.

### 4. IQ Routing — Trajectory-Aware LLM Routing
- **URL**: https://iq-routing.com
- **ProductHunt**: Aug 27, 2026 | **Category**: Developer Tools
- **Description**: Drop-in AI gateway that classifies each request within an agent trajectory, serves from cache, routes to cheapest quality-sufficient model.
- **Key Pattern**: Session-aware agent governance (tracks cumulative cost across agent loop), per-step routing with trajectory awareness, spending caps on thinking tokens.
- **NeoTrix Mapping**: NT-CORE (GWT routing), NT-ACT (cost-aware routing), NT-SHIELD (governance).
- **Why Notable**: Based on AgentRouter paper (ICML 2026) — 72% cost reduction with <3% quality loss. Single-turn routers (RouteLLM/FrugalGPT) fail on trajectories.

### 5. Agnost AI — Agent Failure Detection from Production
- **URL**: https://github.com/AgnostAI/agnost
- **Stars**: YC Summer 2026 | **License**: Proprietary + open skill
- **Description**: Reads production conversations to find silent failures (hallucinations, broken promises, policy violations) your evals miss.
- **Key Pattern**: Auto-clustering conversations into failure categories, traces linked to exact conversations, opens reviewed PRs to fix agents, trains specialist models from production traces.
- **NeoTrix Mapping**: NT-REPAIR (failure detection), NT-MEMORY (conversation analysis), NT-MIND (self-improvement from traces).
- **Why Notable**: Training specialist models from production traces: agnost-0.1 shows 87.9% task success vs 71.5% for Opus 4.8, 94.5% cost reduction on specific workloads.

### 6. Vercel Zero — Agent-First Programming Language
- **URL**: https://github.com/vercel-labs/zerolang
- **Stars**: 5,346 | **Lang**: Native | **License**: Apache 2.0
- **Description**: Graph-native language where semantic graph IS the program database. Agents query/patch the graph; humans review readable projections.
- **Key Pattern**: ProgramGraph as compiler-owned structure, `zero patch` for checked graph edits, graph hashes for stale/invalid edit rejection, `zero query/inspect/check/test/run` exposed as agent commands.
- **NeoTrix Mapping**: NT-CORE (semantic graph), NT-ACT (agent-native tooling), NT-MEMORY (program database).
- **Why Notable**: Flips the paradigm: source text is a human-readable projection of the graph, not the source of truth. Every subcommand shares `--json` flag with stable error codes.

### 7. Flare — Graph-First IDE for Agentic Coding
- **URL**: https://github.com/AlgoNoRhythm/Flare
- **ProductHunt**: Aug 28, 2026 | **License**: MIT
- **Description**: Desktop IDE where codebase is a live dependency graph (files=nodes, imports=edges). Terminal for Claude/Codex/OpenCode below, review cockpit above.
- **Key Pattern**: Burst-based review (grouped file changes by author), blast-radius file tiering (read carefully/read/skim), intent recording via MCP, hidden git repo auto-commits every change burst.
- **NeoTrix Mapping**: NT-CORE (dependency graph), NT-REPAIR (blast radius analysis), NT-ACT (agent oversight).
- **Why Notable**: Review cockpit tells you what was tested vs what was edited-again-without-retest. "Oh no" button for agentic coding via worktree isolation.

### 8. Reflexio — Behavioral Learning Layer for AI Agents
- **URL**: https://github.com/ReflexioAI/reflexio
- **Stars**: 363 | **Lang**: Python | **License**: Apache 2.0
- **Description**: Turns user corrections and successful outcomes into persisted behavioral improvements. Agent playbook extraction and aggregation.
- **Key Pattern**: User-scoped learning (corrections stay per-user), cross-user playbook aggregation with approval workflow, shadow deployment testing, domain-specific signal extraction.
- **NeoTrix Mapping**: NT-MIND (self-improvement), NT-MEMORY (behavioral playbooks), NT-REPAIR (correction loop).
- **Why Notable**: On GDPVal benchmark: -81% planning steps, -72% tokens on warm baseline. Memory ≠ learning — Reflexio builds actionable behavioral rules, not just context retrieval.

### 9. Cortex (SKYNETLAB) — Memory Layer for AI Agents
- **URL**: https://cortexmemory.dev
- **ProductHunt**: Aug 24, 2026 | **License**: Open source
- **Description**: 4-layer architecture: ACID conversations + vector index + facts extraction + graph database. Single `cortex.memory.*` API orchestrates all layers.
- **Key Pattern**: Memory Spaces for multi-tenant isolation, Hive Mode for agent coordination, user profiles, A2A communication with importance scoring and delivery confirmation.
- **NeoTrix Mapping**: NT-MEMORY (multi-layer memory), NT-CORE (knowledge graph), NT-ACT (agent-to-agent).
- **Why Notable**: Facts layer extracts structured knowledge with 60-90% storage savings. One API coordinates ACID, vector, facts, and graph simultaneously.

### 10. Mastra — TypeScript AI Agent Framework
- **URL**: https://mastra.ai | **GitHub**: mastra-ai/mastra
- **Stars**: YC-backed | **Lang**: TypeScript | **License**: Apache 2.0
- **Description**: All-in-one framework: agents, workflows, memory (observational + semantic), workspaces, observability. Built by Gatsby team.
- **Key Pattern**: Observational memory (background agents maintain dense observation log replacing raw message history), Harness for multi-mode agents (build/plan modes), graph-based workflow engine, human-in-the-loop suspension.
- **NeoTrix Mapping**: NT-ACT (agent orchestration), NT-MEMORY (observational memory), NT-IO (model router).
- **Why Notable**: "SoTA observational memory that learns about your users automatically." Used by Brex ($5.1B), MongoDB, Marsh. Graph workflows with `.then()/.branch()/.parallel()` syntax.

---

## Meta-Patterns (Cross-Project Synthesis)

| Pattern | Projects | NeoTrix Implication |
|---------|----------|-------------------|
| **Trajectory-aware routing** | IQ Routing, AgentRouter paper | GWT salience must model inter-step dependencies, not per-call routing |
| **Persistent KV cache** | oMLX, KVMem | NT-MEMORY needs SSD-tier KV for long sessions |
| **Agent-native languages** | Vercel Zero | Semantic graph as program DB — aligns with HyperCube knowledge representation |
| **Behavioral learning** | Reflexio, Agnost AI | NT-MIND needs correction→playbook loop, not just memory retrieval |
| **Graph-first visualization** | Flare, Zero | Live dependency graph as primary interface — aligns with NT-CORE topology |
| **Event-driven concurrency** | Mozaik | Eliminate workflow graphs; let coordination emerge from typed events |
| **Multi-layer memory** | Cortex, Mastra | ACID + vector + facts + graph — aligns with NT-MEMORY 4-tier architecture |
| **Burst-based review** | Flare | Group changes by author, track test coverage per burst — NT-REPAIR alignment |
| **Session-aware governance** | IQ Routing, Zero | Cost/quality tracking across full agent trajectory, not per-call |

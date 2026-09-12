# Trending Rankings — Cycle 443

**Date**: 2026-09-12
**Source**: GitHub Trending, ProductHunt, arXiv, ByteByteGo
**Focus**: AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns

---

## Tier 1 — High Signal (★ ★ ★ ★ ★)

### 1. Graphify
- **URL**: https://github.com/Graphify-Labs/graphify
- **Stars**: 107,831 | **License**: Apache 2.0
- **Description**: Turns any codebase (code, docs, SQL, configs, PDFs) into a queryable knowledge graph via deterministic AST parsing. No vector store needed. Works in Claude Code, Cursor, Codex, Gemini CLI, OpenCode, and 15+ platforms. Leiden community detection for graph clustering.
- **Key Pattern**: AST-based knowledge graph extraction with zero LLM calls for edge creation; hybrid query (graph traversal + vector + keyword)
- **NeoTrix Relevance**: NT-MEMORY could adopt deterministic AST-to-graph for KB node extraction from codebases; NT-NEXUS could use graph topology for cross-session knowledge weaving; maps to VSA HyperCube concept relations
- **Cycle Gap**: Not in 318-442

### 2. nanobot
- **URL**: https://github.com/HKUDS/nanobot
- **Stars**: 47,663 | **License**: MIT
- **Description**: Ultra-lightweight self-hosted personal AI agent framework (Python). WebUI + terminal + chat apps. Tools, long-term memory (Dream), MCP integrations, model routing, multi-agent delegation, scheduled automation. OpenAI-compatible API.
- **Key Pattern**: "Dream" persistent memory that auto-consolidates session history into long-term memory; model routing with fallback chains; cron-based scheduled automation
- **NeoTrix Relevance**: NT-MEMORY experience-tree absorption pattern matches Dream memory consolidation; NT-IO provider routing could use nanobot's model switching pattern; NT-ACT scheduled automation maps to cron-based evolution cycles
- **Cycle Gap**: Not in 318-442

### 3. ponytail
- **URL**: https://github.com/DietrichGebert/ponytail
- **Stars**: 91,866 | **License**: MIT
- **Description**: "Makes your AI agent think like the laziest developer." -54% code (up to -94%), -20% cost, -27% faster, 100% safe. Skill-based approach that forces minimal necessary code. Validated on real agent sessions editing FastAPI+React repos.
- **Key Pattern**: Anti-over-engineering skill: write only what's needed, never cut validation/error-handling/security. Intensity levels (lite/full/ultra) for gradual adoption.
- **NeoTrix Relevance**: NT-ACT tool calling could adopt anti-over-engineering discipline; NT-MIND SEAL pipeline could use intensity-graded skill application; validates Axiom A1 (cost-aware routing) — simpler code = fewer tokens
- **Cycle Gap**: Not in 318-442

### 4. Webwright
- **URL**: https://github.com/microsoft/Webwright
- **Stars**: 5,961 | **License**: MIT
- **Description**: Microsoft's SWE-style browser agent. Terminal-based web browsing with code-as-action (no coordinate prediction). Skill Factory distills solved tasks into reusable zero-token scripts. 86.7% on Online-Mind2Web, 60.1% on Odysseys long-horizon tasks.
- **Key Pattern**: Code-as-browser-action (not xy-coordinates) + Skill Factory (solved tasks → parameterized CLI tools that run in ~40s with zero tokens)
- **NeoTrix Relevance**: NT-WORLD crawl pipeline could use code-as-action pattern for web extraction; NT-MIND skill crystallization could adopt Skill Factory's solve→distill→reuse loop; NT-ACT tool execution benefits from parameterized skill templates
- **Cycle Gap**: Not in 318-442

### 5. GBrain
- **URL**: https://github.com/garrytan/gbrain
- **Stars**: 26,952 | **License**: MIT
- **Description**: "Brain layer for AI agents" by YC CEO Garry Tan. Self-wiring knowledge graph (zero LLM calls for edge creation). 146K pages, 24K people, 5K companies. 30+ MCP tools. Hybrid search (vector + BM25 + RRF + graph signals). 43 curated skills.
- **Key Pattern**: Zero-LLM entity extraction + typed edges (attended/works_at/invested_in); graph signals (adjacency boost, cross-source corroboration, session demotion); hybrid search with intent-aware query rewriting
- **NeoTrix Relevance**: NT-MEMORY KB could adopt zero-LLM graph extraction for node/edge creation; GWT attention could use graph signals for salience modulation; NT-NEXUS cross-session memory could use typed edges for relationship tracking
- **Cycle Gap**: Not in 318-442

---

## Tier 2 — Strong Signal (★ ★ ★ ★)

### 6. vercel/eve
- **URL**: https://github.com/vercel/eve
- **Stars**: 4,957 | **License**: Apache 2.0
- **Description**: Filesystem-first framework for durable AI agents. Core agent capabilities live in conventional file locations (agents/, tools/, memory/). Projects are inspectable, extensible, operable. By Vercel.
- **Key Pattern**: Filesystem-as-configuration: agent identity, tools, memory, rules all live as versioned files in conventional paths. No hidden state.
- **NeoTrix Relevance**: NT-MEMORY knowledge could adopt filesystem-first layout for KB inspection; NT-ACT skills could use conventional file locations for discoverability; aligns with NeoTrix's modular architecture philosophy
- **Cycle Gap**: Not in 318-442

### 7. gitagent
- **URL**: https://github.com/open-gitagent/gitagent
- **Stars**: 670 | **License**: MIT
- **Description**: Agent IS a git repository. Identity (agent.yaml), personality (SOUL.md), rules (RULES.md), memory (git-committed with full history), tools (YAML), skills, hooks all version-controlled. MCP client auto-discovers server tools.
- **Key Pattern**: Git-native agent lifecycle: memory is git commits (full history/rollback), skills are versioned directories, agent identity is a tracked file
- **NeoTrix Relevance**: NT-MEMORY experience-tree could adopt git-commit pattern for versioned experience snapshots; NT-META self-audit could use git history for change tracking; NT-SHIELD could use version-controlled rules for audit trail
- **Cycle Gap**: Not in 318-442

### 8. OpenAI Agents API
- **URL**: https://openai.com/index/introducing-the-agents-api/
- **Stars**: N/A (API product, Sep 2026)
- **Description**: Production agent harness powering Codex, now exposed as API. Managed sandboxes, context compaction across windows, tool search (load relevant tools on-demand), programmatic tool calling (parallel), subagent delegation. No additional fees — pay for tokens/tools only.
- **Key Pattern**: Context compaction (auto-compress earlier context approaching limits); tool search (load tool defs on-demand to reduce token cost); subagent parallelization with isolated contexts
- **NeoTrix Relevance**: NT-CORE kv_cache_optimizer could adopt context compaction pattern; Axiom A2 (context as scarce resource) validated by production deployment; NT-ACT tool orchestration could use tool search for token-efficient tool loading
- **Cycle Gap**: Not in 318-442

### 9. Mastra
- **URL**: https://mastra.ai
- **Stars**: ProductHunt #1 Sep 9, 2026 | **License**: Open Source
- **Description**: TypeScript agent framework by the Gatsby team. Workflows, memory, streaming, evals, tracing, and Studio (interactive UI). From issue to production, run by agents. `npm create mastra@latest`.
- **Key Pattern**: Full agent lifecycle in TypeScript: workflows + memory + evals + tracing + visual Studio. "From issue to production" closed loop.
- **NeoTrix Relevance**: NT-ACT agent workflows could adopt Mastra's eval+trace+workflow triple; NT-IO TypeScript integration could use Mastra as reference for tool calling patterns; validates Axiom A3 (skill as production template)
- **Cycle Gap**: Not in 318-442

### 10. Harden AIF
- **URL**: https://harden.run
- **Stars**: ProductHout #2 Sep 9, 2026 | **License**: Free, Local
- **Description**: Security layer for AI coding agents. Post-trained model checks tool calls before they run, using request + session context. Beats frontier models on agent-security benchmarks. Runs locally — repo and tool output never leave your machine.
- **Key Pattern**: Pre-execution interception: model validates tool calls against session context before execution. Local-only, no data egress.
- **NeoTrix Relevance**: NT-SHIELD sandbox egress policy could adopt pre-execution tool call validation; NT-SHIELD input firewall pattern validated by Harden's approach; aligns with NeoTrix's privacy-first architecture
- **Cycle Gap**: Not in 318-442

---

## Honorable Mentions

| Project | Stars | Why Notable |
|---------|-------|-------------|
| **TanStack/ai** | 3,042 | Type-safe provider-agnostic TypeScript AI SDK. Streaming chat, tool calling, agents across OpenAI/Anthropic/Gemini. Code Mode agents write+execute TypeScript in sandbox. Maps to NT-IO provider abstraction. |
| **AlphaApollo** | 450+ | Agentic reasoning framework: tool-integrated reasoning + agentic post-training (multi-turn SFT/RL) + self-evolution. GRPO/PPO/DAPO support. Maps to NT-MIND SEAL pipeline training loop. |
| **Mastra Factory** | N/A | "From issue to production, run by agents." Closed-loop: GitHub issue → agent writes code → PR → deploy. Maps to NT-ACT autonomous workflow. |
| **Devin Voice** | N/A | Voice-driven agent: "You say it, Devin ships it." Natural language → production code via voice. Maps to NT-IO multimodal input. |
| **Jackalope** | N/A | Codex, Claude Code, Grok, OpenCode in one shared workspace. Multi-model agent coordination. Maps to GWT multi-domain attention. |
| **Computable GPU Index** | N/A | First open-source price index for GPU compute. Maps to A1 cost-aware routing for compute pricing. |

---

## Trend Analysis

### 1. Knowledge Graphs Replace Vector Search (VSA HyperCube Validation)
Graphify (107K★), GBrain (27K★), and Semantica (cycle 441) all converge on deterministic knowledge graphs as the primary agent memory layer. Zero-LLM entity extraction is the common pattern — edges created by AST parsing, not inference. This validates NeoTrix's VSA HyperCube approach: symbolic knowledge representation outperforms raw vector search for structured reasoning.

### 2. Code-as-Action Dominates Browser Agents (NT-WORLD Pattern)
Webwright proves code-as-browser-action beats xy-coordinate prediction by +26.6 points on Odysseys. The Skill Factory pattern (solved task → parameterized CLI tool → zero-token reuse) is the most transferable pattern for NT-WORLD crawl automation: solve once, crystallize as skill, execute without LLM.

### 3. Anti-Over-Engineering as Agent Discipline (Axiom A1 Deepening)
ponytail's -54% code / -20% cost / -27% speed results validate that "write only what's needed" is a learnable agent skill. Combined with OpenAI Agents API's tool search (load tool defs on-demand), the cost-aware routing axiom gains a code-generation dimension: not just which model to use, but how much code to generate.

### 4. Git-Native Agent Identity (Version-Controlled Self)
gitagent (agent IS a repo) and eve (filesystem-first) both treat agent configuration as versioned files. This aligns with NeoTrix's modular architecture: identity, rules, memory, skills all inspectable and rollback-able. The git-commit-as-memory-snapshot pattern is directly transferable to experience-tree absorption.

### 5. Pre-Execution Security Gains Traction (NT-SHIELD Validation)
Harden AIF (post-trained model checking tool calls) + NanoBot's sandbox mode + Webwright's end-to-end scripts all validate pre-execution interception as the security paradigm. NT-SHIELD's sandbox egress policy + input firewall pattern is production-ready.

---

## NeoTrix Integration Opportunities

| Pattern | Source | NT Mapping | Priority |
|---------|--------|-----------|----------|
| Deterministic Graph Extraction | Graphify | NT-MEMORY AST-to-KB node extraction | P0 |
| Zero-LLM Entity Edges | GBrain | NT-MEMORY graph edge creation without inference | P0 |
| Code-as-Action Browser Skills | Webwright | NT-WORLD crawl pipeline + skill crystallization | P0 |
| Anti-Over-Engineering Skill | ponytail | NT-ACT tool calling discipline | P1 |
| Context Compaction | OpenAI Agents API | NT-CORE kv_cache_optimizer | P1 |
| Git-Commit Memory Snapshots | gitagent | NT-MEMORY experience-tree versioning | P1 |
| Filesystem-First Agent Layout | eve | NT-ACT/MEMORY conventional file paths | P1 |
| Pre-Execution Tool Validation | Harden AIF | NT-SHIELD sandbox tool call interception | P1 |
| Tool Search (On-Demand Loading) | OpenAI Agents API | NT-IO token-efficient tool loading | P2 |
| Multi-Model Workspace | Jackalope | GWT multi-domain attention | P2 |

---

## Cross-Cycle Pattern Summary (Cycles 441-443)

| Pattern | Frequency | NeoTrix Status |
|---------|-----------|---------------|
| Knowledge graph / structured memory | 5 projects + 3 papers | VSA HyperCube (active) |
| Code-as-action / skill crystallization | 3 projects | NT-MIND SEAL (active) |
| Context compaction / token efficiency | 4 projects + 2 papers | Axiom A2 (active) |
| Git-native / filesystem-first agents | 3 projects | NT-MEMORY KB (active) |
| Pre-execution security | 2 projects | NT-SHIELD (active) |
| Multi-agent fleet coordination | 3 projects | GWT broadcast (active) |
| Anti-over-engineering discipline | 2 projects | Axiom A1 (active) |
| Type-safe agent SDKs | 2 projects | NT-IO provider (active) |

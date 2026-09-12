# Trending Rankings — Cycle 433

**Date:** 2026-09-12
**Focus:** AI agents, LLM tools, reasoning frameworks, memory, attention, routing

---

## 1. Headroom

- **GitHub:** ~7.7K stars (1,200+ this week)
- **Category:** Context Compression
- **What:** Compresses tool outputs, logs, files, and RAG chunks before LLM ingestion — 60–95% fewer tokens with same answer quality. Ships as library, proxy, and MCP server.
- **NeoTrix mapping:** Direct competitor/complement to NT-MEMORY token efficiency. Aligns with Axiom A2 (Context as Scarce Resource). Could replace or enhance `kv_cache_optimizer.rs` compression paths.
- **Pattern:** Pre-ingestion compression as infrastructure primitive. Not post-hoc summarization but proactive token reduction before context window enters.
- **Novel:** Distribution-aware compression that preserves retrieval quality rather than naive truncation.

## 2. Scrapling

- **GitHub:** ~59.5K stars (1,100+ this week)
- **Category:** Adaptive Web Scraping
- **What:** Adaptive web scraping framework that handles anti-bot defenses, CAPTCHAs, and dynamic page layouts automatically. Self-healing selectors that adapt when sites change.
- **NeoTrix mapping:** NT-WORLD crawl pipeline. Current `UnifiedCrawler` could absorb Scrapling's adaptive selector pattern for production-grade web acquisition.
- **Pattern:** Self-healing data acquisition. Scrapers that maintain themselves across site redesigns without manual selector updates.
- **Novel:** LLM-assisted DOM adaptation — uses vision models to understand page layout when CSS selectors break.

## 3. Open-LLM-VTuber

- **GitHub:** ~8.6K stars
- **Category:** Voice AI / Local LLM Interface
- **What:** Hands-free voice interaction with any local LLM, voice interruption, Live2D avatar — all running locally. Closest thing to "Jarvis-style" assistant.
- **NeoTrix mapping:** NT-IO interface domain. Pattern for voice-first agent interaction that maps to NT-PHYSICAL audio sync and NT-FEEL emotional expression.
- **Pattern:** Voice interruption as first-class input. Agents that handle mid-speech corrections and emotional tone, not just transcript processing.
- **Novel:** Real-time voice emotion detection feeding back into agent behavior adaptation.

## 4. GenericAgent

- **GitHub:** ~14K stars
- **Category:** Self-Evolving Agent Framework
- **What:** ~3K LOC core, 9 atomic tools, 100-line Agent Loop. Self-evolving skill crystallization — every task becomes a reusable skill. Token-efficient (<30K context vs 200K–1M typical).
- **NeoTrix mapping:** SEAL pipeline skill crystallization (NT-MIND). Goal Hive mode maps to NT-ACT multi-worker orchestration. Morphling mode (project-level capability absorption) maps to R-P79/R-P42.
- **Pattern:** Information Density Maximization — agent context is curated not accumulated. Skills crystallize from execution traces, not prompts.
- **Novel:** Goal Hive: BBS-coordinated master/workers for long-horizon parallel objectives. Conductor sub-agent orchestration.

## 5. Webwright (Microsoft)

- **GitHub:** ~6K stars
- **Category:** Browser Agent Framework
- **What:** Terminal-to-browser agent framework. SOTA on Online-Mind2Web (86.7%) and Odysseys (60.1%, +15.6pp over prior SOTA). Skill Factory — every solved task leaves a reusable code skill (40s execution, zero tokens).
- **NeoTrix mapping:** NT-WORLD web interaction capability. Skill Factory pattern directly maps to SEAL skill crystallization. Code-as-action beats coordinate prediction.
- **Pattern:** Solved tasks become parameterized CLI tools. Skills are code, not prompts — rerunnable without LLM inference cost.
- **Novel:** Cross-harness skill reuse (Claude Code, Codex, OpenClaw, Hermes all load same skill folder).

## 6. AgentMemory (rohitg00)

- **GitHub:** ~28K stars
- **Category:** Persistent Agent Memory
- **What:** BM25 + Vector + Graph (RRF fusion) memory engine. 95.2% R@5 on LongMemEval-S. 4-tier consolidation + decay + auto-forget. Works across all MCP agents. ~1,900 tokens/session (92% less than CLAUDE.md).
- **NeoTrix mapping:** NT-MEMORY knowledge domain. Hybrid retrieval pattern (BM25+Vector+Graph) validates NT-MEMORY's multi-signal retrieval architecture. 4-tier consolidation maps to experience-tree absorption.
- **Pattern:** Memory as infrastructure service (MCP server), not agent-specific plugin. Cross-agent memory portability.
- **Novel:** Lease-based memory coordination for multi-agent teams. Real-time viewer on port 3113.

## 7. Eve (Vercel)

- **GitHub:** ~5K stars
- **Category:** Filesystem-First Agent Framework
- **What:** Agents as filesystem conventions. Core capabilities live in conventional locations (`skills/`, `tools/`, `memory/`), making projects inspectable, extendable, and operable. Beta.
- **NeoTrix mapping:** NT-MEMORY + NT-ACT architecture. Filesystem-as-contract pattern maps to NeoTrix's domain directory structure. Convention-over-configuration for agent capabilities.
- **Pattern:** Agent state as filesystem artifacts. Debugging via file inspection, not log parsing.
- **Novel:** Agents ship with their full documentation in `node_modules/eve/docs` — coding agents can read docs locally without API calls.

## 8. GitTrends AI v5.0

- **GitHub:** Open-source registry
- **Category:** AI Tool Discovery / MCP Registry
- **What:** Real-time GitHub velocity tracker + 1-click MCP discovery for coding agents. 4 editorial leaderboards: Agent Skills, MCP Servers, Ecosystem Marketplaces, Star Velocity Radar. Native MCP server.
- **NeoTrix mapping:** NT-IO discovery layer. Pattern for agent-native tool discovery. Could feed NT-CORE GWT attention with real-time ecosystem signals.
- **Pattern:** Agent-native discovery — MCP server exposes trending data directly to coding agents mid-task, no browser required.
- **Novel:** Velocity-weighted ranking (50 stars gaining 40/day > 100K stars gaining 5/day). Off-peak GitHub Actions scheduling.

## 9. Agnost AI

- **ProductHunt:** YC 2026
- **Category:** Agent Failure Detection
- **What:** Analyzes production AI agent conversations to discover silent failures, behavior drift, hallucinations, user frustration, hidden feature requests. Groups into recurring patterns, shows exact conversations, turns into evals and fixes.
- **NeoTrix mapping:** NT-REPAIR + NT-SHIELD audit. Silent failure detection maps to SelfTest T3 (production wiring). Pattern mining maps to ConsciousnessTree health chain.
- **Pattern:** Agent observability that converts production failures into automated evals. Not just monitoring but feedback loop closure.
- **Novel:** Trains small SLMs from agent failure patterns — more accurate, faster, cheaper than frontier models for failure detection.

## 10. n8n (AI-Native)

- **GitHub:** 400+ integrations, massive user base
- **Category:** Workflow Automation + AI Agents
- **What:** Visual no-code interface with native AI capabilities. LangChain integration, custom AI agent automations alongside traditional API calls. Self-hosted, fair-code license.
- **NeoTrix mapping:** NT-ACT orchestration. Visual agent pipeline builder pattern. Integration breadth (400+ connectors) as model for NT-ACT tool ecosystem.
- **Pattern:** Hybrid automation — AI agents coexist with traditional workflows in same pipeline. No "AI-only" silos.
- **Novel:** Enterprise-grade self-hosted AI automation with data governance. Bridges business automation and AI agent workflows.

---

## Cross-Cutting Patterns (Cycle 433)

| Pattern | Prevalence | NeoTrix Mapping |
|---------|-----------|-----------------|
| **Skill Crystallization** (tasks → reusable code) | GenericAgent, Webwright, ECC | SEAL pipeline, skill nodes |
| **Memory as Infrastructure** (MCP, not plugin) | AgentMemory, TencentDB, Memoria | NT-MEMORY as MCP server |
| **Token Compression** (pre-ingestion, not post-hoc) | Headroom, 9Router RTK, Ponytail | Axiom A2, kv_cache_optimizer |
| **Agent-Native Discovery** (MCP-for-tools) | GitTrends AI, n8n AI | NT-IO discovery layer |
| **Self-Healing Selectors** (adaptive scraping) | Scrapling | NT-WORLD crawl pipeline |
| **Code-as-Action** (solved tasks → CLI tools) | Webwright, GenericAgent | SEAL skill nodes |
| **Voice Interruption** (mid-speech correction) | Open-LLM-VTuber | NT-FEEL emotional feedback |
| **Failure → Eval Loop** (production feedback) | Agnost AI | NT-REPAIR SelfTest T3 |

## Priority Absorption Targets

1. **Headroom** — Token compression is immediate ROI for NT-MEMORY context management
2. **Webwright Skill Factory** — Code-as-skill pattern validates SEAL crystallization direction
3. **AgentMemory RRF Fusion** — BM25+Vector+Graph retrieval pattern for NT-MEMORY
4. **GenericAgent Goal Hive** — Multi-worker orchestration for NT-ACT long-horizon tasks
5. **Scrapling Adaptive Selectors** — Self-healing crawl for NT-WORLD production robustness

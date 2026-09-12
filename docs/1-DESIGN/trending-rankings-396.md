# Trending Rankings — Cycle 396

**Date:** 2026-09-12
**Focus:** AI agents, LLM tools, reasoning frameworks, novel patterns for memory/attention/routing

---

## Top 10 New Projects (Not in Cycles 318-395)

| # | Project | Stars | Category | Key Innovation |
|---|---------|-------|----------|----------------|
| 1 | **OpenClaw 2.0** | 210K+ | Personal AI Agent Platform | Multiplayer shared cloud sessions, credential isolation (masked prompts), self-learning Skill Workshop, SQLite-backed durable memory, 575ms Gateway startup |
| 2 | **Graphify** | 107K+ | Knowledge Graph for Codebases | AST-based deterministic knowledge graph from code/docs/PDFs. No vector store. Query codebase as graph instead of grepping. Cross-framework plugin (Claude Code, Cursor, Codex, Gemini CLI, OpenClaw) |
| 3 | **Ponytail** | 91K+ | Agent Code Efficiency | Skill that makes agents write 54% less code (up to 94%) while staying 100% safe. YAGNI-driven: writes only what's needed. Benchmarked on real agentic sessions (Haiku 4.5, n=4). ~20% cheaper, ~27% faster |
| 4 | **Kilo Code** | 27K+ | Agentic Engineering Platform | All-in-one IDE/CLI/Cloud agent. 500+ models at cost (zero markup). Auto Model routing. Parallel isolated worktrees. Agent Manager for multi-agent orchestration. Code Reviewer. Acquired by Anaconda |
| 5 | **Nanobot** | 47K+ | Ultra-Lightweight Agent Framework | Self-hosted personal AI agent in Python. Dream memory, MCP, model routing, multi-agent delegation, scheduled automation. Runs in WebUI/terminal/Telegram/Discord/Slack/WeChat. OpenAI-compatible API |
| 6 | **Mastra** | N/A | TypeScript AI Agent Framework | Graph-based workflow engine, observational memory, human-in-the-loop workflows, 90+ model providers, Mastra Studio for interactive testing. Used by Replit, SoftBank, PayPal, PLAID |
| 7 | **Switch** (SandboxAQ) | N/A | Agent-Collaboration Bridge | Open-source: bring any AI agent into Slack/Teams/Discord/Telegram/Mattermost. Rooms hold context (not agents). One agent, many rooms. MCP-native. Apache 2.0. Team of 5 devs + 40 agents |
| 8 | **Harden AIF** | N/A | Agent Security Layer | Local cybersecurity LLM checks every tool call before execution. Rewrites unsafe commands, blocks exfiltration. Beats GPT-5.5 on AgentHazard (83.7% vs 81.4%). Free for individual devs. Works with Claude Code, Codex, Cursor, Gemini CLI |
| 9 | **Webwright** | 5.9K | Browser Agent Framework | Microsoft: code-as-action beats coordinate prediction. 86.7% on Mind2Web, 60.1% on Odysseys (+15.6pp over SOTA). Skill Factory: solved tasks become reusable CLI tools (~40s, zero tokens). Plugin for Claude Code/Codex/OpenClaw |
| 10 | **GBrain** | 26K+ | Personal Knowledge Brain | Self-wiring knowledge graph (zero LLM calls for edges). Hybrid search (vector + BM25 + RRF + source-tier boost). 43 curated skills. 146K pages, 24K people, 5K companies indexed. +31.4 P@5 over vector-only RAG |

---

## Key Patterns Observed

### 1. Agent-as-Repo Paradigm
- **GitAgent**: Agent IS a git repository — identity, rules, memory, tools, skills all version-controlled files
- **OpenClaw 2.0**: Session-bound workspaces, conversation branches, durable progress cards
- **Graphify**: Codebase-as-knowledge-graph with deterministic AST parsing

### 2. Security as First-Class Concern
- **Harden AIF**: Pre-execution tool-call filtering with trained cybersecurity LLM
- **OpenClaw 2.0**: Session permission modes, masked credential prompts, plugin trust review
- **Switch**: Room-level permission policies for agent participation

### 3. Efficiency-Driven Agent Design
- **Ponytail**: YAGNI discipline — 54% less code, 20% cheaper, 27% faster
- **Webwright**: Code-as-action (scripts) vs coordinate prediction — 15.6pp improvement
- **Kilo Code**: Auto Model routing picks cheapest capable model per task

### 4. Memory Architecture Maturation
- **GBrain**: Self-wiring graph + hybrid search + gap analysis (not just retrieval)
- **OpenClaw 2.0**: Background memory consolidation with Dream Diary
- **Nanobot**: Dream memory with long-term persistence

### 5. Multi-Agent Collaboration Infrastructure
- **Switch**: Room-based agent collaboration (context per room, not per agent)
- **Mastra**: Graph-based workflow engine with human-in-the-loop suspension
- **OpenClaw 2.0**: Shared cloud sessions for team collaboration

---

## NeoTrix Domain Mapping

| Project | Primary Domain | Secondary Domain |
|---------|---------------|-----------------|
| OpenClaw 2.0 | NT-IO (platform runtime) | NT-SHIELD (security) |
| Graphify | NT-MEMORY (knowledge graph) | NT-WORLD (code perception) |
| Ponytail | NT-ACT (code efficiency) | NT-MIND (skill optimization) |
| Kilo Code | NT-IO (IDE integration) | NT-ACT (multi-agent orchestration) |
| Nanobot | NT-IO (multi-channel) | NT-MEMORY (Dream memory) |
| Mastra | NT-IO (framework) | NT-ACT (workflow engine) |
| Switch | NT-IO (collaboration bridge) | NT-SHIELD (permission policies) |
| Harden AIF | NT-SHIELD (pre-execution security) | NT-CORE (intent verification) |
| Webwright | NT-WORLD (browser perception) | NT-ACT (skill factory) |
| GBrain | NT-MEMORY (hybrid search+graph) | NT-CORE (gap analysis) |

---

## Absorption Candidates (High NeoTrix Relevance)

1. **Graphify's deterministic AST→knowledge-graph pipeline** → Enhance NT-MEMORY code-understanding capabilities
2. **Harden's pre-execution tool-call filtering** → Strengthen NT-SHIELD sandbox security model
3. **Ponytail's YAGNI discipline** → Inform SEAL pipeline skill efficiency metrics
4. **Switch's room-based context model** → Extend NT-IO multi-agent collaboration patterns
5. **Webwright's Skill Factory** → Adapt for NeoTrix skill crystallization (solved tasks → reusable code)
6. **GBrain's gap analysis** → Enhance NT-CORE reasoning with self-knowledge about unknowns

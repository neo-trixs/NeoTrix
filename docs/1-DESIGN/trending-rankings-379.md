# Trending Rankings — Cycle 379

**Date:** 2026-09-12
**Sources:** GitHub Trending, Product Hunt, arXiv, ACL 2026, TuringPost

---

## Top 10 New Projects (Not in Cycles 318–378)

| # | Project | Stars | Category | Key Pattern | NeoTrix Domain |
|---|---------|-------|----------|-------------|----------------|
| 1 | **archify** (tt-a1i) | 28.2K | Architecture diagram agent | Self-contained HTML diagrams with motion — agent skill for verifiable architecture visualization | NT-IO + NT-WORLD |
| 2 | **K-Dense-AI/scientific-agent-skills** | 36.9K | Turn any agent into an AI Scientist | Skill-as-methodology: agent skill files encode scientific workflows as composable templates | NT-MIND (skill crystallization) |
| 3 | **OpenMontage** (calesthio) | 53.5K | Agentic video production system | 12 production pipelines, 100+ tools, 700+ agent skill files — full video production studio from code assistant | NT-ACT + NT-IO |
| 4 | **FreeLLMAPI** (tashfeenahmed) | 21.7K | Unified free LLM gateway | 34 providers, 635 endpoints, one /v1 endpoint. Smart routing + automatic failover + encrypted keys | NT-IO (provider routing) |
| 5 | **Chrome DevTools MCP** (ChromeDevTools) | 50.0K | Chrome DevTools for coding agents | MCP-native browser introspection — agents read DOM, network, performance directly | NT-WORLD + NT-IO |
| 6 | **Antigravity Panel** (n2ns) | 10K+ | Community toolkit for Google Antigravity IDE | Quota dashboard + usage trends + runway prediction + cache manager + auto-accept mode | NT-IO + NT-SHIELD |
| 7 | **SkillDock** (wanghuan9) | 5K+ | AI skill manager desktop app | Real-directory scanning + Git-aware Diff previews for skills/MCP/plugins across 6+ coding tools | NT-MIND + NT-IO |
| 8 | **SAFEdep/gryph** | 4K+ | Security layer for AI coding agents | Works across Claude Code, Cursor, Windsurf, Gemini CLI, OpenCode — universal agent security | NT-SHIELD |
| 9 | **Mozaik** (jigjoy-ai) | 3K+ | TypeScript runtime for interoperable AI agents | Type-safe agent runtime — agents communicate via typed message contracts | NT-IO + NT-ACT |
| 10 | **GitNexus** | 46.2K | Zero-server code intelligence engine | Client-side knowledge graph creator + built-in Graph RAG agent for code exploration | NT-WORLD + NT-MEMORY |

---

## Pattern Analysis

### 1. Agent-as-Production-System (OpenMontage, scientific-agent-skills, SkillDock)
- **OpenMontage**: 700+ agent skill files form a complete video production knowledge base. Agent skill is the production unit, not prompts.
- **scientific-agent-skills**: Encodes scientific methodology as agent skill files — turn any agent into a domain specialist.
- **SkillDock**: Manages skill lifecycle (install/organize/edit/sync/update) across multiple coding tools. Git-aware diffing for skill evolution.
- **NeoTrix mapping**: Validates SEAL pipeline skill crystallization. OpenMontage's 700+ skills demonstrate C4→C5 constellation progression at scale. SkillDock's cross-tool management maps to NT-MIND's skill governance.

### 2. Universal Gateway & Routing (FreeLLMAPI, Antigravity Panel, archify)
- **FreeLLMAPI**: 34 providers behind one endpoint. Smart routing + failover. Token cost optimization.
- **Antigravity Panel**: Quota dashboard with runway prediction — resource-aware agent operation.
- **NeoTrix mapping**: Direct alignment with NT-IO provider management. Ordered Backend Router (P4) validated. Antigravity's runway prediction maps to ResourceBudgetManager.

### 3. Browser-Native Agent Introspection (Chrome DevTools MCP, archify)
- **Chrome DevTools MCP**: Agents read DOM, network, performance via MCP protocol — browser as first-class agent workspace.
- **archify**: Agent produces self-contained HTML with motion — output is inspectable, interactive, verifiable.
- **NeoTrix mapping**: NT-WORLD perception could integrate DevTools MCP for web tasks. archify's verifiable-output pattern aligns with quality gate requirements.

### 4. Cross-Tool Agent Security (Gryph, Antigravity Panel)
- **Gryph**: Universal security layer across 7+ coding tools. Same security posture regardless of harness.
- **Antigravity Panel**: Cache management + auto-accept mode control — agent autonomy boundaries.
- **NeoTrix mapping**: NT-SHIELD universal agent security. Gryph validates cross-harness security as a product category. Complements Egress Privacy Guard (outbound) with inbound agent protection.

### 5. Interoperable Agent Runtimes (Mozaik, GitNexus)
- **Mozaik**: Type-safe TypeScript runtime for agent interoperability. Agents as typed services.
- **GitNexus**: Client-side knowledge graph + Graph RAG. No server needed — all processing local.
- **NeoTrix mapping**: Mozaik's typed contracts map to NT-IO's interface contracts. GitNexus's client-side KB aligns with NeoTrix's local-first philosophy.

---

## Actionable Absorption Candidates

| Project | Absorption Target | Priority |
|---------|-------------------|----------|
| OpenMontage 700+ skill files | SEAL skill crystallization — large-scale skill knowledge base as production artifact | P0 |
| FreeLLMAPI smart routing | NT-IO Ordered Backend Router — 34-provider chain with failover + cost tracking | P0 |
| Gryph cross-tool security | NT-SHIELD universal agent security layer — inbound tool-call protection | P1 |
| SkillDock Git-aware diffs | NT-MIND skill versioning — Git-aware skill evolution tracking | P1 |
| Chrome DevTools MCP | NT-WORLD web perception — browser introspection via MCP | P1 |
| Antigravity runway prediction | ResourceBudgetManager — usage forecasting + quota management | P2 |
| Mozaik typed agent runtime | NT-IO typed message contracts for agent interoperability | P2 |
| GitNexus client-side KB | NT-MEMORY local-first knowledge graph with Graph RAG | P2 |
| archify verifiable diagrams | NT-IO output verification — agent produces inspectable artifacts | P3 |
| scientific-agent-skills methodology | NT-MIND domain methodology encoding as skill templates | P3 |

# Trending Rankings — Cycle 421 (2026-09-12)

## 10 New Projects (Not in Cycles 318-420)

| # | Project | Stars | Focus | NeoTrix Relevance |
|---|---------|-------|-------|-------------------|
| 1 | **DeepSeek Harness** | +152.1K Aug | Plugin-based agent runtime — tools, skills, sessions, interfaces, storage, execution. Not a model but the software layer surrounding models. | NT-ACT: Agent runtime architecture. Plugin-based extensibility maps to capability node composition. NT-IO: skill/tool registry pattern for runtime skill discovery. |
| 2 | **mattpocock/skills** | +33.9K | Portable engineering workflows for coding agents. Composable skills from `.agents/` directory, cross-model compatible. Claude Code + OpenCode native. | NT-MEMORY + NT-ACT: Skill-as-artifact pattern — each skill is a portable knowledge unit. Aligns with experience-tree branch loading + SKILL-SPEC.md contract. |
| 3 | **Archify** | +28.7K | Turns codebases/systems into verifiable technical maps (architecture, workflow, sequence, data flow, lifecycle views). Agent skill for diagramming. | NT-CORE: E8 engine visualization — architecture diagrams as reasoning output. Maps to CapabilityBridge tree↔registry cross-view. |
| 4 | **Superpowers (obra)** | ~1.5K | Agentic skills framework + software development methodology. Composable skills + behavioral anchors for coding agents. Structured SDLC for machines. | NT-ACT: Methodology-as-skill pattern. Aligns with SEAL pipeline stage definitions — structured lifecycle for autonomous agents. |
| 5 | **Offsite** | ~800 | Hybrid human+agent teams in shared workspace. Org chart visualization, agent-to-agent communication, human-in-the-loop approval. MCP-compatible. | NT-GOVERNANCE + NT-ACT: Multi-agent coordination with human oversight. Maps to AttentionManager dual-weapon routing with governance gating. |
| 6 | **BetterClaw** | ~2.5K | No-code AI agent builder. 60-second deploy, 95+ OAuth integrations, trust levels (Intern→Specialist→Lead), BYOK zero markup. | NT-SHIELD: Trust escalation model (Intern→Lead) mirrors Constellation maturity C0→C5. Security guardrails + secrets auto-purge. |
| 7 | **Unabyss** | ~1.2K | MCP-native self-updating context layer. Set up once, auto-extract/structure/update context from daily apps. Granular per-tool visibility. | NT-MEMORY + NT-NEXUS: Persistent cross-session context that auto-updates. Maps to experience-tree hub index with lazy branch loading. |
| 8 | **Switch** | ~900 | Bring any AI agent into Slack, Teams & Discord. Universal agent connector across communication platforms. | NT-IO: Multi-platform agent interface. Aligns with PlatformGateway pattern — single agent, multiple channel surfaces. |
| 9 | **Mastra** | ~3.5K | Modern TypeScript stack for building AI agents. First-class TypeScript, streaming, tool calling, memory, evals. Production-ready agent framework. | NT-IO + NT-ACT: TypeScript agent runtime. Maps to web-server ACP interface for agent deployment. Framework-as-infrastructure pattern. |
| 10 | **Apache Maka** | ~300 | Local-first AI agent workspace. Append-only log of messages, tool calls, tool results, permission decisions, termination events. Full audit trail. | NT-SHIELD + NT-MEMORY: Append-only audit log for agent actions. Maps to CleanupEvent logging + EventBus traceability. |

## Category Breakdown

### Agent Runtime & Orchestration
- **DeepSeek Harness** — plugin-based agent runtime
- **Offsite** — hybrid human+agent coordination
- **Superpowers** — methodology-as-skill framework

### Agent Skills & Knowledge
- **mattpocock/skills** — portable engineering workflows
- **Archify** — codebase→technical diagrams
- **Unabyss** — auto-updating context layer

### Platform & Infrastructure
- **Switch** — cross-platform agent connector
- **Mastra** — TypeScript agent stack
- **BetterClaw** — no-code agent builder

### Audit & Governance
- **Apache Maka** — append-only agent workspace log

## Top 3 Actionable Insights for NeoTrix

1. **Skill-as-artifact distribution** (mattpocock/skills + Superpowers) — Skills are becoming portable, cross-model units. NeoTrix should adopt SKILL-SPEC.md as the universal skill contract format, enabling skills to move between NT-* domains like packages move between repositories.

2. **Trust escalation as architecture** (BetterClaw Intern→Lead model) — Agent trust should be a first-class architectural concern, not an afterthought. Map to Constellation maturity (C0→C5) and Shield permission tiers. Each trust level unlocks more capabilities.

3. **Append-only agent audit** (Apache Maka) — Every agent action, tool call, permission decision, and termination event should be logged as an immutable append-only stream. This enables post-hoc analysis, self-healing, and governance compliance without runtime overhead.

## Sources

- GitHub Trending: August 2026 monthly dataset (GitTrend)
- ProductHunt: September 2026 top products
- SWEN.AI GitHub Radar (2026-09-12)
- Firecrawl Best Trending Repos 2026 (2026-08-27)
- ByteByteGo Top AI GitHub Repos 2026 (2026-03-09)

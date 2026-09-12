# Trending Rankings — Cycle 404 (2026-09-12)

## 10 New Projects (Not in Cycles 318-403)

### 1. OpenClaw
- **GitHub**: https://github.com/openclaw/openclaw
- **Stars**: 387k+ | **Forks**: 81k+
- **Category**: Personal AI Agent / Multi-Channel Gateway
- **What**: Open-source AI assistant that runs on your devices and meets you in 20+ messaging platforms (Telegram, Discord, Slack, Signal, iMessage). Gateway architecture connects models, tools, and channels. MIT licensed, maintained by OpenClaw Foundation.
- **Key Pattern**: Gateway-as-architecture — single Node.js gateway routes between messaging apps and LLM providers. Skills/plugins extend capabilities. ClawHub registry for skill discovery.
- **NeoTrix Relevance**: NT-IO (multi-platform gateway), NT-ACT (skill registry, plugin SDK), NT-MEMORY (session persistence across channels). Validates NeoTrix's domain-routed architecture pattern.
- **URL**: https://openclaw.ai

### 2. MagiCrew
- **GitHub**: https://github.com/dtyq/magic
- **Stars**: 5k+ | **Forks**: 558
- **Category**: Enterprise AI Agent Platform
- **What**: Open-source all-in-one AI productivity platform. Multi-agent orchestration with orchestrator dispatching specialist agents in parallel. Built-in rendering framework transforms AI output into deliverables (PPT, dashboards, reports, Excel). Enterprise controls: budget caps, approval gates, sandbox isolation.
- **Key Pattern**: Deliverable-first agents — output transforms directly into production artifacts, not plain text. Three-tier budget control (department/user/task). Skill ecosystem compatible with OpenClaw/Anthropic skills.
- **NeoTrix Relevance**: NT-ACT (production orchestration), NT-SHIELD (sandbox isolation, approval gates), NT-MEMORY (enterprise knowledge preservation). Validates NeoTrix's Dark Forest + safety-first patterns.

### 3. screenpipe (YC S26)
- **GitHub**: https://github.com/screenpipe/screenpipe
- **Stars**: 20k+ | **Category**: Agent Memory / Desktop Context
- **What**: Open-source local-first screen + audio recorder that creates searchable AI-powered memory from computer activity. Gives agents context about what users actually did. Event-driven capture (app switches, clicks, typing pauses). Local PII redaction. SOC 2 compliant. YC S26 batch.
- **Key Pattern**: Passive context acquisition — agents get real workflow context without explicit prompting. Event-driven capture reduces noise. Local-first privacy with optional team sync. "Pipes" for autonomous agent actions triggered on device events.
- **NeoTrix Relevance**: NT-WORLD (perception/data acquisition), NT-MEMORY (searchable activity memory), NT-SHIELD (local PII redaction). Validates NeoTrix's PerceptionBridge pattern for attention-gated sensory flow.

### 4. Flare
- **GitHub**: https://github.com/AlgoNoRhythm/Flare
- **Stars**: 120+ (new, Aug 2026) | **Category**: Agentic IDE / Code Visualization
- **What**: Graph-first IDE for agentic coding. Live dependency graph (files=nodes, imports=edges) with integrated terminal for Claude Code/Codex/OpenCode. Burst-based review cockpit groups changes by author. File tiering (read carefully/read/skim) based on blast radius, coverage, complexity. Hidden git repo auto-commits every change burst. MIT licensed.
- **Key Pattern**: Blast-radius-aware code review — agent changes are tiered by risk. Intent recording via MCP tool before editing. "Oh no" button for reverting agent changes without touching real repo.
- **NeoTrix Relevance**: NT-REPAIR (blast-radius analysis for self-healing), NT-META (change attribution, intent tracking), NT-CORE (dependency graph as reasoning substrate). Validates NeoTrix's module survival (Dark Forest) pattern.

### 5. Dropstone
- **Website**: https://dropstone.io | **Category**: AI Runtime / Persistent Memory
- **What**: Self-hosted AI coding agent runtime with cross-surface persistent memory ("Continuity"). One memory shared by CLI, chat, SDK, and phone. Standing rules vs facts distinction. 1M token context window. 4x usage-per-dollar vs competitors. Model-agnostic (open weights, self-hosted, Ollama). D3 Engine: recursive swarm architecture for parallel exploration.
- **Key Pattern**: Memory-as-identity — corrections become standing rules that persist across sessions and surfaces. "Continuity" = account-level memory shared across all interfaces. D3 Engine: 10,000 agents explore divergent paths, propagate "negative knowledge" (failure modes) across swarm via vector-space dedup.
- **NeoTrix Relevance**: NT-MEMORY (cross-session memory), NT-MIND (recursive self-improvement via swarm), NT-CORE (GWT-style attention routing across agents). Validates NeoTrix's ConsciousnessTree cycle pattern.

### 6. Kilo Code
- **GitHub**: https://github.com/Kilo-Org/kilocode
- **Stars**: 5M+ users | **Category**: Open-Source Agentic Coding Platform
- **What**: All-in-one agentic coding platform. VS Code + JetBrains + CLI + Cloud. 500+ models, zero markup. Five agent modes: Orchestrator (task decomposition), Architect (planning), Code (implementation), Debug (troubleshooting), Ask. Parallel isolated worktrees. Auto model routing. Acquired by Anaconda.
- **Key Pattern**: Mode-specialized agents — different personas for different phases of development. Orchestrator breaks complex tasks into subtasks delegated to specialized modes. "Kilo Speed" = agents own features end-to-end.
- **NeoTrix Relevance**: NT-ACT (orchestration, tool calling), NT-MIND (mode specialization like Ascendancy dual-weapon sets), NT-CORE (GWT salience routing between modes). Validates NeoTrix's Dual Specialization pattern.

### 7. Hermes Agent
- **Website**: https://hermes-agent.ai | **Category**: Multi-Agent Framework / Terminal Agent
- **What**: Open-source AI agent framework by Nous Research. Runs in terminal, desktop app, messaging platforms, IDEs. Provider-agnostic (20+ providers). Multi-platform gateway (Telegram, Discord, Slack, WhatsApp, Signal). Multi-agent orchestration v0.6.0: spawn specialist workers with tailored context. Kanban swarm for durable multi-agent workflows. Atropos RL pipeline for self-improvement.
- **Key Pattern**: Hierarchical task decomposition — orchestrator analyzes task, identifies work breakdown structure, spawns worker agents with task-relevant context subset. "Dynamic Workflow" moves plan+intermediate results OUT of context window INTO script.
- **NeoTrix Relevance**: NT-ACT (agent orchestration), NT-MIND (RL self-improvement via Atropos), NT-MEMORY (persistent memory, session resume). Validates NeoTrix's SEAL pipeline pattern for self-evolution.

### 8. Hermes Studio
- **GitHub**: https://github.com/JPeetz/Hermes-Studio
- **Stars**: 347 | **Category**: Agent Dashboard / Orchestration UI
- **What**: Self-hosted web dashboard for Hermes Agent. Multi-agent crews with live SSE activity feed. Visual workflow builder (DAG editor) for sequential/parallel agent pipelines. Conductor V2 with animated SVG office layouts. Cron job scheduling. MCP server management. Profile-scoped workspaces. 30+ features.
- **Key Pattern**: Visual orchestration cockpit — DAG-based workflow editor with live execution status. Crew-based agent groups with real-time monitoring. "Conductor" metaphor for agent supervision.
- **NeoTrix Relevance**: NT-IO (agent dashboard, visual monitoring), NT-GOVERNANCE (approval workflows, permission controls), NT-META (cross-agent coordination visualization). Validates NeoTrix's ConsciousnessTree visual monitoring pattern.

### 9. MemQL
- **Website**: https://memql.io | **Category**: Agent Runtime / Memory Substrate
- **What**: Open-source Go-native AI agent runtime and memory substrate. Append-only time-series graph for memory (blending recency + relevance). Cost and safety spine with rate ceilings, per-plan budgets, loop breakers. Compiles into secured, traced node mesh. Multi-node observable operations.
- **Key Pattern**: Time-series graph memory — append-only graph that blends recency and relevance for recall. Loop breakers prevent infinite agent loops. Budget-aware routing at infrastructure level.
- **NeoTrix Relevance**: NT-MEMORY (graph-based memory), NT-SHIELD (loop breakers, safety enforcement), NT-CORE (budget-aware routing aligns with Cost-Aware Routing axiom A1). Validates NeoTrix's A1 axiom.

### 10. Agent Crew
- **Website**: https://agentcrew.sh | **Category**: Multi-Agent Orchestration Platform
- **What**: Open-source platform for orchestrating collaborative AI agent teams. Agents defined by Markdown files (instructions, roles, skills) — no code required. Supports engineering, marketing, finance, operations domains. Scheduled tasks and webhooks. MCP integration for external tools. Self-hosted (Docker/Kubernetes).
- **Key Pattern**: Markdown-as-agent-definition — agent configuration in plain Markdown, no code needed. Domain-specialized teams (engineering, marketing, finance). Webhook-triggered autonomous workflows.
- **NeoTrix Relevance**: NT-ACT (agent team orchestration), NT-GOVERNANCE (Markdown-based policy), NT-IO (MCP integration, webhook triggers). Validates NeoTrix's SKILL-SPEC.md contract pattern.

## Trend Analysis

| Signal | Count | Implication |
|--------|-------|-------------|
| Multi-agent orchestration | 6/10 | Industry converging on orchestrator-worker patterns |
| Persistent memory | 5/10 | Cross-session memory becoming table stakes |
| Open-source agent platforms | 7/10 | Open-source winning in agent infrastructure |
| Enterprise security | 4/10 | Budget controls + approval gates emerging |
| Graph-based visualization | 2/10 | Dependency graphs for code review (Flare) + memory (MemQL) |
| Local-first privacy | 3/10 | PII redaction + local processing (screenpipe, Dropstone) |

## Absorption Candidates for NeoTrix

| Pattern | Source | NeoTrix Domain | Priority |
|---------|--------|----------------|----------|
| Burst-based blast-radius review | Flare | NT-REPAIR | High |
| Standing rules vs facts memory | Dropstone | NT-MEMORY | High |
| Dynamic Workflow (plan-out-of-context) | Hermes Agent | NT-MIND | High |
| Time-series graph memory | MemQL | NT-MEMORY | Medium |
| Markdown-as-agent-definition | Agent Crew | NT-ACT | Medium |
| Crew-based agent monitoring | Hermes Studio | NT-GOVERNANCE | Medium |
| Event-driven passive capture | screenpipe | NT-WORLD | Medium |
| Mode-specialized agent personas | Kilo Code | NT-CORE | Medium |

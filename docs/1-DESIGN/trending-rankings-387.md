# Trending Rankings — Cycle 387

**Date**: 2026-09-12
**Sources**: GitHub Trending, ProductHunt, GitTrend, ossinsight

## Top 10 New Projects (Not in Cycles 318-386)

### 1. OpenClaw (250K+ ⭐)
- **URL**: https://github.com/openclaw/openclaw
- **Category**: Personal AI Agent / Local Runtime
- **What**: Open-source AI agent that runs on your machine, manages tasks, automates workflows, persistent memory. Integrates with WhatsApp, Telegram, Discord, iMessage. Self-generates skills.
- **Key Pattern**: Local-first autonomous agent with plugin marketplace + skill auto-generation. Peter Steinberger (ex-OpenAI) led 60K stars in 72h.
- **NeoTrix Mapping**: NT-ACT (orchestration), NT-MEMORY (persistent memory), NT-IO (multi-channel)
- **Novel**: Self-healing skill marketplace + "liver" coding agent integration (Claude Code/Codex/OpenCode as background workers)

### 2. screenpipe (21K ⭐, YC S26)
- **URL**: https://github.com/screenpipe/screenpipe
- **Category**: Agent Memory / Screen Intelligence
- **What**: Continuously captures screen + audio locally, creates searchable AI-powered memory. Pipes = scheduled AI agents triggered by work activity.
- **Key Pattern**: Event-driven agent activation from screen/audio signals. Three-layer permission enforcement (skill gating, agent interception, server middleware).
- **NeoTrix Mapping**: NT-MEMORY (perception memory), NT-WORLD (sensory capture), NT-SHIELD (3-layer permission)
- **Novel**: Screen events as agent triggers; event-driven instead of query-driven memory

### 3. ToolRank
- **URL**: https://toolrank.dev
- **Category**: Agent Tool Discovery
- **What**: Platform for optimizing AI agent tool discovery. Scores tool definitions across findability, clarity, precision, efficiency. LLM selection tournaments + runtime reliability testing.
- **Key Pattern**: Tool-as-rankable-entity with 4-dimension scoring + tournament-based selection
- **NeoTrix Mapping**: NT-ACT (tool registry optimization), NT-CORE (ranking engine)
- **Novel**: Treats tools as first-class rankable entities with continuous quality scoring

### 4. Revolte
- **URL**: ProductHunt #4 of day (May 28, 2026)
- **Category**: AI Software Engineering
- **What**: AI for software engineering with interactive sessions. Workflow automation + engineering platform.
- **Key Pattern**: Interactive AI sessions for engineering workflows (not just code gen)
- **NeoTrix Mapping**: NT-ACT (engineering automation), NT-IO (interactive sessions)
- **Novel**: Session-based engineering AI (vs one-shot code gen)

### 5. GitNexus (Akon Labs)
- **URL**: ProductHunt launch Aug 2026
- **Category**: Coding Agent Infrastructure
- **What**: Open-source kernel for coding agents. Foundation layer for building coding agents.
- **Key Pattern**: Kernel-level abstraction for coding agents (like OS kernel for processes)
- **NeoTrix Mapping**: NT-ACT (agent runtime kernel), NT-CORE (foundation layer)
- **Novel**: Coding agent as OS-level primitive

### 6. Dropstone
- **URL**: ProductHunt launch Aug 2026 (5.0 rating, 4 reviews)
- **Category**: Persistent Agent Runtime
- **What**: AI runtime that remembers, learns, and acts everywhere. Cross-tool persistence.
- **Key Pattern**: Cross-tool memory persistence + learning loop
- **NeoTrix Mapping**: NT-MEMORY (cross-session persistence), NT-NEXUS (cross-tool bridge)
- **Novel**: Runtime memory that survives across different tools/environments

### 7. Flare
- **URL**: ProductHunt launch Aug 2026
- **Category**: Agentic Coding IDE
- **What**: Graph-first IDE and interactive map for agentic coding. Visual graph of code relationships.
- **Key Pattern**: Graph-based code visualization as IDE primitive for agent workflows
- **NeoTrix Mapping**: NT-CORE (graph topology), NT-ACT (agent coding)
- **Novel**: Code-as-graph for agent navigation (vs file-tree)

### 8. Construct Computer
- **URL**: ProductHunt (Aug 23, 2026)
- **Category**: AI Agent Desktop Environment
- **What**: "Your AI coworker gets a computer. You get your day back." Full desktop environment for AI agents.
- **Key Pattern**: Agent-as-co-worker with dedicated computing environment
- **NeoTrix Mapping**: NT-ACT (autonomous computing), NT-PHYSICAL (virtual embodiment)
- **Novel**: Agent gets its own computer (sandboxed environment)

### 9. HarnessRouter Community Edition
- **URL**: ProductHunt (Aug 16, 2026)
- **Category**: Agent Harness Abstraction
- **What**: Open-source unified interface for agent harnesses. Single interface across Claude Code, Codex, etc.
- **Key Pattern**: Harness-agnostic routing layer for coding agents
- **NeoTrix Mapping**: NT-IO (harness abstraction), NT-ACT (multi-harness routing)
- **Novel**: Unified interface across incompatible agent harnesses

### 10. ClawCompactor (open-compress)
- **URL**: https://github.com/open-compress/claw-compactor
- **Category**: LLM Token Compression
- **What**: 14-stage Fusion Pipeline for LLM token compression. Reversible compression, AST-aware code analysis, intelligent content routing. Zero LLM inference cost.
- **Key Pattern**: Multi-stage reversible compression with AST awareness
- **NeoTrix Mapping**: NT-CORE (compression pipeline), NT-MEMORY (context optimization)
- **Novel**: Reversible compression preserving code structure; 14-stage pipeline

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Domain |
|---------|----------|----------------|
| **Local-first autonomy** | OpenClaw, screenpipe, Construct Computer | NT-SHIELD, NT-PHYSICAL |
| **Event-driven agents** | screenpipe (screen events as triggers) | NT-WORLD, NT-ACT |
| **Tool-as-entity** | ToolRank, HarnessRouter | NT-ACT |
| **Persistent agent memory** | OpenClaw, Dropstone, screenpipe | NT-MEMORY, NT-NEXUS |
| **Agent-as-OS-primitive** | GitNexus, Construct Computer | NT-CORE, NT-ACT |
| **Reversible compression** | ClawCompactor | NT-CORE, NT-MEMORY |
| **Graph-first coding** | Flare | NT-CORE |
| **Harness abstraction** | HarnessRouter, GitNexus | NT-IO, NT-ACT |

## Signal: The Agent Runtime Wars

The ecosystem is converging on **agent runtime as the new platform**. Three competing abstractions:
1. **OpenClaw model**: Local agent with plugin marketplace + messaging integration
2. **screenpipe model**: Event-driven agent triggered by work activity
3. **GitNexus model**: Kernel-level coding agent infrastructure

NeoTrix advantage: Already has NT-ACT (orchestration) + NT-MEMORY (KB) + NT-SHIELD (sandbox). Missing: **event-driven activation** (screenpipe pattern) and **harness abstraction** (HarnessRouter pattern).

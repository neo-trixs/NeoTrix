# Trending Rankings — Cycle 397

**Date**: 2026-09-12  
**Focus**: AI agents, LLM tools, reasoning frameworks, novel memory/attention/routing patterns  
**Exclusions**: Projects from cycles 318-396

---

## Top 10 New Projects

### 1. Screenpipe — AI Agent Memory (Local-First)
- **URL**: https://github.com/screenpipe/screenpipe
- **Stars**: 21,410+ | **YC**: S26
- **Innovation**: 24/7 screen + audio capture for AI agent memory. Event-driven capture via accessibility APIs with OCR fallback. Pipes system: markdown-defined scheduled agents that act on screen data. MCP server for Claude/Cursor/Codex integration. All processing local-first.
- **Pattern**: Persistent episodic memory via continuous observation → structured extraction → MCP-accessible search API
- **NeoTrix Relevance**: NT-MEMORY (episodic memory layer), NT-WORLD (perception pipeline), NT-SHIELD (on-device PII scrubbing)

### 2. Agno — Agent Platform Runtime
- **URL**: https://github.com/agno-agi/agno
- **Stars**: 42,053 | **License**: Apache-2.0
- **Innovation**: 3-layer architecture: SDK (build) → AgentOS (runtime) → Control Plane (operate). JWT-based RBAC, OpenTelemetry tracing, 100+ integrations. Microsecond agent instantiation. Coding-agent-first workflow: prompt your coding agent to deploy entire agent platform. v3.0 released Aug 2026.
- **Pattern**: Agent-as-a-Service with built-in observability, human approval gates, and self-improvement loop via usage data
- **NeoTrix Relevance**: NT-ACT (orchestration patterns), NT-IO (multi-interface exposure: Slack/Telegram/MCP/A2A), NT-MIND (self-improvement via evals)

### 3. Mem0 — Universal Memory Layer for AI Agents
- **URL**: https://github.com/mem0ai/mem0
- **Stars**: 60,074+ | **YC**: S24
- **Innovation**: 4-layer memory hierarchy (conversation/session/user/organizational). Single-pass ADD-only extraction (no UPDATE/DELETE). Entity linking across memories. LoCoMo benchmark: 91.6 (+20pts over prior). Under 7K tokens per retrieval call vs 25K+ for full-context. Graph memory for relational structures.
- **Pattern**: Token-efficient memory via single-pass fact extraction + entity linking + hierarchical scoping
- **NeoTrix Relevance**: NT-MEMORY (multi-level memory architecture), NT-CORE (entity linking → HyperCube association), NT-MIND (memory consolidation)

### 4. Experiential Labs — Open Source AI Gateway
- **URL**: https://experientiallabs.ai
- **Stars**: 880+ (launched Sep 2026)
- **Innovation**: Zero-markup gateway for 1000+ models. Learns from traffic to cut costs, recommend models, and train specialized models you own. One trace format across all requests → catch cache misses, wasted tokens, async-batch opportunities. Embedded guardrails for PII stripping. 10B+ tokens processed daily since HN launch.
- **Pattern**: Traffic-driven model routing optimization with owned-model training from production traces
- **NeoTrix Relevance**: NT-IO (cost-aware model routing), NT-SHIELD (egress PII guardrails), NT-MEMORY (trace-to-model training loop)

### 5. Composio — Agent Tool Orchestration
- **URL**: https://composio.dev
- **Rating**: 5.0 (8 reviews, 329 followers)
- **Innovation**: 1000+ production-ready tools with managed OAuth, smart tool resolution, sandboxed execution. Agent calls tool → Composio handles auth, permissions, reliability. Used by Zoom, Glean, HubSpot. UTCP protocol (scalable, secure alternative to MCP).
- **Pattern**: Tool-as-a-Service with managed auth lifecycle and deterministic execution sandboxing
- **NeoTrix Relevance**: NT-ACT (tool orchestration), NT-SHIELD (sandboxed execution), NT-IO (MCP/UTCP protocol bridge)

### 6. HarnessRouter — Unified Agent Harness Interface
- **URL**: ProductHunt launch Aug 2026
- **Innovation**: Open-source unified interface for agent harnesses. Single configuration layer across multiple coding agents (Claude Code, Codex, Gemini CLI, etc.). Standardizes agent interaction protocols.
- **Pattern**: Protocol-agnostic agent harness abstraction layer
- **NeoTrix Relevance**: NT-IO (multi-agent protocol unification), NT-ACT (harness abstraction for agent deployment)

### 7. Entroly — Reversible Context Compression
- **URL**: https://github.com/juyterman1000/entroly
- **Innovation**: Cut AI context cost without trusting the compressor. Every reduction is reversible, byte-exact recoverable, with auditable receipt. Local-first. Works through proxy, MCP, SDK, or agent wrapper.
- **Pattern**: Lossless reversible compression with cryptographic audit trail for context optimization
- **NeoTrix Relevance**: NT-MEMORY (context compression with verification), NT-SHIELD (auditable receipts), NT-CORE (cost-aware context management)

### 8. Orkas — Multi-Agent Desktop Client
- **URL**: https://github.com/Orkas-AI/Orkas
- **Innovation**: Commander LLM dispatches sub-agents in parallel/series. Agents self-evolve via reflection and skill crystallization. Local-first, BYO LLM keys. macOS/Windows/Linux.
- **Pattern**: Commander-worker swarm with self-evolving agent skills
- **NeoTrix Relevance**: NT-ACT (multi-agent dispatch), NT-MIND (skill crystallization), NT-CORE (GWT-inspired attention routing)

### 9. Nixis — Agent Firewall
- **URL**: https://github.com/mayankjain0141/nixis
- **Innovation**: Intercepts tool calls (file, shell, network) and enforces deterministic policies at sub-microsecond latency. CEL (Common Expression Language), IFC (Information Flow Control), secret scanning, audit logging.
- **Pattern**: Deterministic policy enforcement at tool-call granularity with formal verification
- **NeoTrix Relevance**: NT-SHIELD (agent safety kernel), NT-ACT (tool call interception), NT-GOVERNANCE (policy enforcement)

### 10. Astromesh — Multi-Model Agent Runtime
- **URL**: https://github.com/monaccode/astromesh
- **Innovation**: Define agents in YAML, route each role to a model. 7 orchestration patterns: ReAct, Plan & Execute, Fan-Out, Pipeline, Supervisor, Swarm, Glyph. REST/WebSocket API with RAG, memory, MCP tools, guardrails, OpenTelemetry.
- **Pattern**: Role-to-model routing with pluggable orchestration patterns and built-in observability
- **NeoTrix Relevance**: NT-ACT (orchestration pattern library), NT-IO (multi-model routing), NT-MEMORY (RAG integration)

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Domain |
|---------|----------|----------------|
| **Local-first privacy** | Screenpipe, Entroly | NT-SHIELD, NT-MEMORY |
| **Token-efficient memory** | Mem0, Entroly, Screenpipe | NT-MEMORY |
| **Agent self-evolution** | Orkas, Agno, Mem0 | NT-MIND |
| **Multi-agent orchestration** | Orkas, Astromesh, Agno | NT-ACT |
| **Tool-call security** | Nixis, Composio | NT-SHIELD |
| **Cost-aware routing** | Experiential, Astromesh | NT-IO |
| **Audit trails** | Entroly, Nixis, Agno | NT-GOVERNANCE |

## Signal Strength Assessment

| Signal | Strength | Source |
|--------|----------|--------|
| Screen memory as first-class agent capability | 🔴 Critical | Screenpipe 21K stars, YC S26 |
| Memory hierarchy (4-layer) becoming standard | 🔴 Critical | Mem0 60K stars, production benchmarks |
| Swarm coordination ≈ model scaling | 🟡 Strong | SwarmSys paper, AgentPSO |
| Agent firewall / deterministic policy | 🟡 Emerging | Nixis, Composio UTCP |
| Traffic-driven model optimization | 🟢 Early | Experiential Labs, 10B tokens/day |

# Trending Rankings — Cycle 413

> **Date**: 2026-09-12
> **Sources**: GitHub Trending, ProductHunt, arXiv, Hacker News, dev.to
> **Prior baseline**: cycles 318-412 (no duplicates)

---

## Top 10 New Projects

### 1. Harden AIF — Agent Security Firewall

| Field | Value |
|-------|-------|
| **GitHub** | https://github.com/hardenrun/aif |
| **Stars** | 2,500+ (new, launched Sep 9 2026) |
| **Language** | Rust + Python |
| **Category** | Agent Security / Guardrails |

**What it does**: On-device AI firewall for coding agents. An 8B post-trained cybersecurity LLM runs locally and inspects every tool call before execution. Beats frontier models (GPT-5.5) on agent-security benchmarks (AgentHazard 83.7% first-harm catch, SABER 48% harmful-action recall). Supports Claude Code, Codex, Cursor, Gemini CLI, Hermes, OpenClaw.

**Novel patterns**:
- **Pre-execution interception**: Checks actions before they run, not after
- **Intent-aware decisions**: Matches tool calls against original developer intent + session context
- **Graduated responses**: Allow → Ask → Redact → Block → Record (agent keeps working on blocked calls)
- **Privacy-by-design**: 8B model runs on-device, repo/tool output never leaves machine

**NeoTrix mapping**: NT-SHIELD (pre-execution tool auditing), NT-CORE (GWT salience for action filtering), NT-ACT (safe tool execution guard)

---

### 2. Mastra — TypeScript Agent Framework

| Field | Value |
|-------|-------|
| **GitHub** | https://github.com/mastra-ai/mastra |
| **Stars** | Trending #1 on ProductHunt Sep 9 (491 upvotes) |
| **Language** | TypeScript |
| **Category** | Agent Framework / Orchestration |

**What it does**: Production TypeScript framework for AI agents with typed agents, graph-based workflows, observational memory, human-in-the-loop, multi-agent supervisor pattern, and Mastra Studio for dev/testing. Integrates with React, Next.js, Node.js. Apache 2.0.

**Novel patterns**:
- **Observational Memory**: Background agent distills message history into dense observation logs as context grows
- **Typed Workflow Engine**: `.then()`, `.branch()`, `.parallel()` with Zod schemas, suspend/resume for HITL
- **Studio**: Interactive UI for testing agents, tracing execution, editing prompts without code access
- **Multi-agent Supervisor**: Coordinator routes tasks to specialized subagents with shared state

**NeoTrix mapping**: NT-ACT (agent orchestration), NT-MEMORY (observational memory pattern), NT-IO (typed workflow contracts)

---

### 3. DeerFlow 2.0 — ByteDance SuperAgent Harness

| Field | Value |
|-------|-------|
| **GitHub** | https://github.com/bytedance/deer-flow |
| **Stars** | 56,000+ (v2.0 ground-up rewrite) |
| **Language** | Python + TypeScript |
| **Category** | SuperAgent Harness / Long-horizon Tasks |

**What it does**: Long-horizon agent harness that orchestrates sub-agents, persistent memory, sandboxed execution, and extensible skills. Built on LangGraph/LangChain. 9-middleware pipeline, per-thread isolation, Docker/E2B sandboxes, progressive skill loading.

**Novel patterns**:
- **9-Middleware Pipeline**: ThreadData → Uploads → Sandbox → Summarization → TodoList → Title → Memory → ViewImage → Clarification
- **Progressive Skill Loading**: Skills (Markdown-defined) loaded on-demand, saving context/tokens
- **Sandbox per Thread**: Each conversation thread gets isolated filesystem + execution environment
- **Subagent Concurrency**: Max 3 subagents per turn, 15-min timeout, background thread pools

**NeoTrix mapping**: NT-ACT (sub-agent delegation), NT-WORLD (sandbox execution), NT-MEMORY (progressive context loading)

---

### 4. Noodle Seed — Governed MCP Runtime

| Field | Value |
|-------|-------|
| **ProductHunt** | Sep 9 2026 (254 upvotes, #4) |
| **Website** | https://noodleseed.com |
| **Language** | TypeScript |
| **Category** | MCP Infrastructure / Agent Governance |

**What it does**: Platform to build, host, and ship MCP servers. Author one `server.ts`, get a governed multi-tenant runtime serving ChatGPT, Claude, Codex, and any MCP client. Credential broker, policy gates, OAuth/OIDC, machine-to-machine auth.

**Novel patterns**:
- **Thin Language, Fat Runtime**: Declarative `server.ts` → compiled artifact → shared stateless runtime serves it
- **Credential Broker**: Never forwards inbound tokens to backends; exchanges validated identity for scoped credentials
- **Multi-tenant Governance**: Policy, audit, rate limits, redaction applied in runtime before connector side effects
- **Machine-to-Machine MCP**: Service principals with least-privilege grants for CI/scheduled jobs

**NeoTrix mapping**: NT-IO (MCP server governance), NT-SHIELD (credential isolation), NT-ACT (agent-to-backend bridge)

---

### 5. OmniAgent — Self-Evolving Agent Framework

| Field | Value |
|-------|-------|
| **GitHub** | https://github.com/YeQing17-2026/OmniAgent |
| **Stars** | 2,557 |
| **Language** | Python |
| **Category** | Self-Evolving Agent / Security |

**What it does**: Agent with full-dimensional self-evolution (Skill, Context, BrainModel) via OmniEvolve. Hyper-Harness for safe execution, Deep Reflexion for dual-layer reflection (real-time risk interception + failure-to-insight conversion). Online RL (GRPO + PRM) for BrainModel evolution.

**Novel patterns**:
- **OmniEvolve**: Skills evolve in real-time during execution (not periodic post-execution)
- **Deep Reflexion**: Dual-layer loop — real-time risk interception + offline failure analysis
- **Hyper-Harness**: Dynamic multi-agent + concurrent tool execution, progressive context loading
- **BrainModel Self-Evolution**: Online reinforcement learning feedback loop during interactive use

**NeoTrix mapping**: NT-MIND (SEAL pipeline real-time evolution), NT-SHIELD (Hyper-Harness safety), NT-CORE (Deep Reflexion meta-cognition)

---

### 6. Atomic Agent — Local-First AI Agent

| Field | Value |
|-------|-------|
| **GitHub** | https://github.com/AtomicBot-ai/atomic-agent |
| **Stars** | 2,432 |
| **Language** | TypeScript + Rust |
| **Category** | Local-First Agent / Browser Automation |

**What it does**: Local-first agent running on consumer hardware. Browser automation (Playwright), file/shell ops, memory with hybrid recall, MCP tools, TUI. TurboQuant llama.cpp fork (+30-50% throughput). Beats Hermes on GAIA L1 (69.8% vs 58.5%). Supports Claude Code + Codex subscriptions via signed-in CLIs.

**Novel patterns**:
- **TurboQuant llama.cpp**: Quantized model optimization for consumer GPUs
- **Hybrid Recall Memory**: Profile facts, notes, links, lessons with voting + reflection
- **Approval-gated Execution**: MCP tools default to approval-gated; trusted servers batch with reads
- **Dual Interface**: CLI for automation, TUI for interactive control (approvals, logs, skills, memory)

**NeoTrix mapping**: NT-PHYSICAL (local-first inference), NT-MEMORY (hybrid recall), NT-ACT (browser automation)

---

### 7. GitAgent — Git-Native Agent Framework

| Field | Value |
|-------|-------|
| **GitHub** | https://github.com/open-gitagent/gitagent |
| **Stars** | 670 |
| **Language** | TypeScript |
| **Category** | Agent-as-Repo / Version-Controlled Agents |

**What it does**: Agent IS a git repository. Identity (`SOUL.md`), rules (`RULES.md`), memory, tools, skills — all version-controlled files. Fork agents, branch personalities, `git log` memory history. MCP client. OpenTelemetry tracing with cost attribution.

**Novel patterns**:
- **Agents as Repos**: Fork/branch/diff agent configurations like code
- **Version-Controlled Memory**: `git log` your agent's memory, diff its rules
- **OpenTelemetry Native**: `gen_ai.chat` spans + `gitagent.tool.execute` spans with cost attribution
- **Plugin System**: Reusable extensions providing tools, hooks, skills, prompts, memory layers

**NeoTrix mapping**: NT-MEMORY (version-controlled experience), NT-SHIELD (trace-based audit), NT-ACT (MCP client integration)

---

### 8. GoModel — Open-Source OpenRouter

| Field | Value |
|-------|-------|
| **ProductHunt** | Sep 9 2026 (126 upvotes) |
| **Website** | https://gomodel.enterpilot.io |
| **Language** | Go |
| **Category** | AI Gateway / Model Routing |

**What it does**: Self-hosted OpenAI-compatible API gateway. One endpoint for every provider with budgets, caching, guardrails, load balancing, and failover. Single binary, ~20MB Docker image. MIT license.

**Novel patterns**:
- **Ordered Backend Fallback**: Single interface with ordered provider fallback chain
- **Budget-aware Routing**: Cost tracking + budget limits per model/provider
- **Guardrails at Gateway Level**: Safety checks before model inference, not just after
- **Zero-Config Failover**: Automatic provider switching on failure

**NeoTrix mapping**: NT-IO (ordered backend router), NT-ACT (cost-aware model routing), NT-SHIELD (gateway-level guardrails)

---

### 9. Headroom — Token Compression for LLM Context

| Field | Value |
|-------|-------|
| **GitHub** | https://github.com/headroom-ai/headroom |
| **Stars** | 7,700+ |
| **Language** | Python |
| **Category** | Context Compression / Token Optimization |

**What it does**: Compresses tool outputs, logs, files, and RAG chunks before they reach an LLM, achieving 60-95% fewer tokens with same answer quality. Ships as library, proxy, and MCP server.

**Novel patterns**:
- **Pre-LLM Compression**: Compress inputs before they reach the model, not after
- **Multi-format**: Works on tool outputs, logs, files, RAG chunks
- **MCP Server Mode**: Drop-in compression for any MCP workflow
- **Quality-Preserving**: Maintains answer quality while reducing tokens 60-95%

**NeoTrix mapping**: NT-MEMORY (context compaction), NT-IO (token-efficient routing), NT-CORE (attention budget optimization)

---

### 10. OmniAgent Deep Reflexion — Dual-Layer Reflection

| Field | Value |
|-------|-------|
| **GitHub** | https://github.com/YeQing17-2026/OmniAgent |
| **Stars** | 2,557 (same repo, new module) |
| **Language** | Python |
| **Category** | Meta-Cognition / Self-Reflection |

**What it does**: Dual-layer reflective architecture — real-time risk interception at the action level and failure-to-insight conversion at the session level. Inspired by Reflexion (Shinn 2023) but adds production safety.

**Novel patterns**:
- **Real-Time Risk Interception**: Checks every action before execution for safety violations
- **Failure-to-Insight**: Converts failed actions into reusable lessons stored in memory
- **Dual-Layer Loop**: Layer 1 (action-level safety) + Layer 2 (session-level learning)
- **Production Safety**: Unlike pure research Reflexion, integrates with execution harness

**NeoTrix mapping**: NT-CORE (ConsciousnessTree feedback loop), NT-REPAIR (failure-to-insight), NT-SHIELD (pre-execution safety)

---

## Summary: New Patterns for NeoTrix

| Pattern | Source | NeoTrix Integration |
|---------|--------|---------------------|
| Pre-execution tool firewall | Harden AIF | NT-SHIELD guard on every tool call |
| Observational memory | Mastra | NT-MEMORY background distillation |
| Progressive skill loading | DeerFlow 2.0 | NT-ACT on-demand skill injection |
| Credential broker isolation | Noodle Seed | NT-SHIELD token-to-credential swap |
| Real-time skill evolution | OmniAgent | NT-MIND SEAL pipeline continuous evolution |
| TurboQuant local inference | Atomic Agent | NT-PHYSICAL quantized model optimization |
| Agents-as-repos | GitAgent | NT-MEMORY version-controlled experience |
| Gateway-level guardrails | GoModel | NT-IO ordered backend + cost routing |
| Pre-LLM token compression | Headroom | NT-MEMORY context compaction |
| Dual-layer reflection | OmniAgent Reflexion | NT-CORE action + session meta-cognition |

---

*Generated: 2026-09-12 | Cycle 413 | 10 projects | 10 patterns*

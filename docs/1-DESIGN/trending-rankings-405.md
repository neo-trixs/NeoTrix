# Trending Rankings — Cycle 405 (2026-09-12)

## 10 New Projects (Not in Cycles 318-404)

### 1. ToolRank
- **GitHub**: https://github.com/imhiroki/toolrank
- **Stars**: 1 (early stage, Mar 2026) | **Category**: Agent Tool Optimization (ATO)
- **What**: The "PageRank for AI agent tools." Scores MCP tool definitions across 4 dimensions: Findability (25%), Clarity (35%), Precision (25%), Efficiency (15%). Scanned 4,162 MCP servers — 97.1% have quality defects. Optimized tools get selected 3.6x more often by AI agents. Includes CLI, Python SDK, ecosystem scanner, and daily blog generation.
- **Key Pattern**: ATO (Agent Tool Optimization) — next evolution after SEO→LLMO. Optimizes for autonomous agent execution, not human mentions. Four-dimension scoring with maturity levels (Dominant/Preferred/Selectable/Visible/Absent).
- **NeoTrix Relevance**: NT-ACT (tool discovery optimization, MCP tool scoring), NT-IO (tool ecosystem scanning), NT-SHIELD (tool quality validation). Validates NeoTrix's CapabilityRegistry pattern — tools need optimization to be discovered by agents.

### 2. Claw Compactor
- **GitHub**: https://github.com/open-compress/claw-compactor
- **Stars**: Growing | **Category**: LLM Token Compression
- **What**: 14-stage Fusion Pipeline for LLM token compression. 15–82% reduction depending on content, zero LLM inference cost, reversible compression. Stages: QuantumLock→Cortex→Photon→RLE→SemanticDedup→Ionizer→Neurosyntax→Nexus→TokenOpt→Abbrev. Each stage understands content structure (AST-aware code analysis, JSON statistical sampling, simhash deduplication). 1,600+ tests.
- **Key Pattern**: Content-type-aware compression — unlike LLMLingua-2 which drops tokens by perplexity (destroying code identifiers), Claw Compactor uses 14 specialized stages that understand structure. Reversible via marker IDs. ROUGE-L @0.5 = 0.723 vs 0.570 for LLMLingua-2.
- **NeoTrix Relevance**: NT-MEMORY (context compression for long sessions), NT-CORE (Axiom A2: Context as Scarce Resource — KVMem-style optimization), NT-ACT (token cost reduction). Validates NeoTrix's compaction strategy for <256K sessions.

### 3. HarnessRouter
- **GitHub**: https://github.com/HarnessRouter/harnessrouter
- **Stars**: 647 | **Forks**: 62 | **Category**: Agent Harness Unified Interface
- **What**: YC-backed open-source unified interface for agent harnesses. Runs Codex, Claude Code, Hermes, PI, DSH through one API. Implements Unified Harness Protocol (UHP) — open standard with 10 normative chapters. Sessions, streaming, files, cancellation, failure handling. Single Docker container. Your keys, your infrastructure. Apache-2.0.
- **Key Pattern**: Harness-as-a-Service (HaaS) with open protocol. UHP abstracts agent runtime differences — write once, execute across Codex/Claude Code/Hermes. Conformance suite ensures interoperability. Starter kits for end-to-end integration examples.
- **NeoTrix Relevance**: NT-ACT (harness abstraction, unified execution), NT-IO (multi-provider gateway), NT-GOVERNANCE (UHP standardization). Validates NeoTrix's CapabilityRouter pattern — single interface routing across agent backends.

### 4. IBM MCP ContextForge
- **GitHub**: https://github.com/IBM/mcp-context-forge
- **Stars**: 4,423 | **Forks**: 852 | **Category**: AI Gateway / MCP Proxy
- **What**: Open-source registry and proxy federating MCP, A2A, and REST/gRPC APIs into one clean endpoint. Tools Gateway (MCP, gRPC-to-MCP translation, TOON compression), Agent Gateway (A2A protocol, OpenAI/Anthropic routing), API Gateway (rate limiting, auth, retries). 40+ plugins. Built-in 30+ safety guardrails. OpenTelemetry observability. Apache-2.0.
- **Key Pattern**: Federated tool governance — single endpoint with centralized discovery, guardrails, observability across all agent-tool interactions. gRPC-to-MCP translation enables legacy API virtualization as MCP-compliant tools.
- **NeoTrix Relevance**: NT-IO (unified gateway), NT-SHIELD (30+ safety guardrails, rate limiting), NT-ACT (tool federation, A2A agent routing). Validates NeoTrix's Egress Privacy Guard pattern — centralized governance for outbound agent calls.

### 5. Eko (FellouAI)
- **GitHub**: https://github.com/FellouAI/eko
- **Stars**: 4,954 | **Forks**: 442 | **Category**: Agentic Workflow Framework
- **What**: Production-ready JavaScript framework for building reliable agents from natural language. One sentence → multi-step workflow. Dependency-aware parallel agent execution. Pause/resume/interrupt controls with task_snapshot workflow recovery. Native MCP + browser extension support. Human-in-the-loop with intervention points. Works in browser and Node.js.
- **Key Pattern**: Workflow-as-code with recovery — dependency-aware DAG execution, snapshot-based resume, human intervention gates. "One sentence to multi-step" translation with observable chain execution.
- **NeoTrix Relevance**: NT-ACT (workflow orchestration, MCP integration), NT-MIND (self-improving loops), NT-SHIELD (human-in-the-loop approval gates). Validates NeoTrix's SEAL pipeline pattern for production-grade agentic workflows.

### 6. Agent of Empires (AoE)
- **GitHub**: https://github.com/agent-of-empires/agent-of-empires
- **Stars**: 3,191 | **Forks**: 340 | **Category**: Multi-Agent Session Manager
- **What**: Rust-based session manager for AI coding agents on Linux/macOS. Manages Claude Code, OpenCode, Codex CLI, Gemini CLI, Mistral Vibe, Copilot CLI, Factory Droid, and 12+ more. TUI + Web dashboard + CLI + HTTP API. Git worktrees for parallel agents across branches. Docker/Podman/Apple Containers sandboxing. Persistent tmux sessions survive disconnects. Remote phone access via Tailscale/Cloudflare tunnels. Mozilla.ai backed.
- **Key Pattern**: Session-as-infrastructure — each agent runs in isolated tmux session with optional container sandboxing. Multi-surface access (TUI/web/phone) to the same agent sessions. Worktree isolation for parallel branch work.
- **NeoTrix Relevance**: NT-ACT (multi-agent session management), NT-SHIELD (container sandboxing, isolation), NT-IO (TUI/web/phone surfaces). Validates NeoTrix's worktree isolation pattern (Axiom P2: Isolation-per-Task).

### 7. stereOS
- **GitHub**: https://github.com/papercomputeco/stereOS
- **Stars**: 485 | **Category**: AI Agent Operating System
- **What**: NixOS-based Linux hardened and purpose-built for AI agents. "Mixtapes" bundle hardened minimal Linux with specific agent harnesses (opencode-mixtape, claude-code-mixtape). Two user model: admin (operations) + agent (restricted workspace). stereosd + agentd daemons for lifecycle management. Lambda MicroVM packaging for AWS deployment. Direct-kernel boot bypassing UEFI/GRUB.
- **Key Pattern**: Agent-as-OS-resident — hardened Linux with restricted agent user, daemon-managed lifecycle. Mixtape model = immutable agent images. Two-tier trust model (admin vs agent user separation).
- **NeoTrix Relevance**: NT-SHIELD (hardened agent OS, user separation), NT-PHYSICAL (hardware-level agent isolation), NT-GOVERNANCE (admin/agent privilege boundary). Validates NeoTrix's safety kernel pattern for physical embodiment.

### 8. Watt-Mind Factory
- **GitHub**: https://github.com/watt-mind/factory
- **Stars**: 12 | **Category**: Self-Improving Agentic Runtime
- **What**: "The factory that builds software — and itself." Runtime for self-improving agentic loops: tracker is control plane, git is truth, CI is gate. Production-grade loop self-editing with observability. Every self-edit ships with falsifiable prediction verified against outcomes. Closed-loop harness self-improvement.
- **Key Pattern**: Verifiable self-improvement — every agent self-edit includes a falsifiable prediction that is checked against outcomes. Git as source of truth, CI as quality gate. Bounded self-refinement (not open-ended recursive improvement).
- **NeoTrix Relevance**: NT-MIND (self-improvement loops), NT-GOVERNANCE (CI-as-gate, falsifiable predictions), NT-REPAIR (self-healing with verification). Validates NeoTrix's SEAL pipeline pattern with bounded refinement ceiling.

### 9. Flare IDE
- **GitHub**: https://github.com/AlgoNoRhythm/Flare
- **Stars**: Growing (Aug 2026) | **Category**: Graph-First Agentic IDE
- **What**: Desktop IDE where the main surface is a live dependency graph (files=nodes, imports=edges). Integrated terminal for Claude Code/Codex/OpenCode. Burst-based review cockpit groups changes by author. File tiering: read carefully/read/skim based on blast radius, coverage, complexity. Intent recording via MCP tool. Hidden git repo auto-commits every change burst. MIT licensed.
- **Key Pattern**: Blast-radius-aware code review — agent changes tiered by structural risk. "Oh no" button reverts agent changes without touching real repo. Intent recording enables post-hoc reasoning reconstruction.
- **NeoTrix Relevance**: NT-REPAIR (blast-radius analysis for self-healing), NT-META (change attribution, intent tracking), NT-CORE (dependency graph as reasoning substrate). Validates NeoTrix's Dark Forest module survival pattern.

### 10. Watt-Mind Coach
- **GitHub**: https://github.com/watt-mind/coach
- **Stars**: 82 | **Category**: AI Fitness/Health Dashboard
- **What**: Open-source AI coach and unified fitness dashboard. Native iOS/Android companion app (Watts Mobile). Integrates wearable data, workout tracking, nutrition, recovery metrics. AI-powered coaching recommendations. Raycast extension for quick access. TypeScript-based, Apache-2.0.
- **Key Pattern**: Domain-specialized AI agent — focused coaching agent with wearable integration, not general-purpose. Cross-platform (web + mobile + desktop extension) with unified data model.
- **NeoTrix Relevance**: NT-PHYSICAL (wearable sensor integration, health metrics), NT-FEEL (coaching feedback loop, motivational patterns), NT-IO (cross-platform surfaces). Validates NeoTrix's NT-PHYSICAL domain for embodied AI patterns.

## Trend Analysis

| Signal | Count | Implication |
|--------|-------|-------------|
| Agent harness unification | 4/10 | UHP + ContextForge + AoE = industry standardizing agent runtime interfaces |
| Token compression/optimization | 3/10 | Context as scarce resource (Axiom A2) driving aggressive optimization |
| Tool discovery optimization | 2/10 | ATO emerging as post-SEO/LLMO discipline for agent ecosystems |
| Hardened agent OS | 2/10 | Agent security shifting from app-level to OS-level isolation |
| Self-improving loops | 2/10 | Bounded refinement with verification becoming consensus practice |
| Graph-based code understanding | 2/10 | Dependency graphs as primary IDE surface for agent oversight |
| Open-source agent infrastructure | 8/10 | Open-source dominant in agent tooling (HarnessRouter, ContextForge, AoE, stereOS, etc.) |

## Cross-Cycle Absorption Candidates

| Project | Absorption Target | Pattern |
|---------|------------------|---------|
| ToolRank ATO scoring | NT-ACT capability scoring | 4-dimension tool readiness metrics |
| Claw Compactor 14-stage pipeline | NT-MEMORY context compaction | Content-type-aware reversible compression |
| HarnessRouter UHP | NT-ACT harness abstraction | Open protocol for multi-harness execution |
| ContextForge federated gateway | NT-IO unified gateway | MCP/A2A/REST federation with guardrails |
| AoE session isolation | NT-SHIELD agent sandboxing | tmux + container dual isolation model |
| stereOS two-tier trust | NT-SHIELD admin/agent boundary | OS-level privilege separation for agents |
| Factory verifiable self-edit | NT-MIND bounded refinement | Falsifiable prediction + CI gate pattern |
| Flare blast-radius review | NT-REPAIR change risk analysis | Dependency graph tiered review cockpit |

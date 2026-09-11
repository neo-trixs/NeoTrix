# Trending Rankings — Cycle 357 (2026-09-12)

## Search Scope
GitHub Trending, ProductHunt, arXiv, GitTrend — AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns.

---

## 10 New Projects (Not in Cycles 318–356)

### 1. Nanobot — Ultra-Lightweight Agent Framework
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/HKUDS/nanobot |
| **Stars** | New (2026) |
| **Language** | Python |
| **Category** | Personal AI Agent Framework |
| **What it does** | Ultra-lightweight, self-hosted personal AI agent with WebUI, tools, memory, MCP, multi-agent workflows, automation, and chat apps. Zero external dependencies beyond core LLM. |
| **NeoTrix mapping** | NT-IO (interface) + NT-ACT (action execution). The "ultra-lightweight self-hosted" pattern maps to NeoTrix's local-first architecture. Memory + MCP integration aligns with NT-MEMORY KB layer. |
| **Key pattern** | **Minimal agent substrate** — prove that a useful agent can run with <500 LOC core + MCP plugins, challenging the bloated-framework assumption. |

### 2. claw-compactor — 14-Stage Fusion Token Compression
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/open-compress/claw-compactor |
| **Stars** | New (2026) |
| **Language** | Python |
| **Category** | LLM Token Compression |
| **What it does** | 14-stage fusion pipeline for reversible token compression. AST-aware code analysis, intelligent content routing. Zero LLM inference cost for compression. |
| **NeoTrix mapping** | NT-MIND (distillation/compression) + NT-CORE (context management). Directly maps to AgentCompress in AgentInfer — asynchronous context summarization without disrupting reasoning. |
| **Key pattern** | **Reversible fusion compression** — compress tokens while preserving the ability to reconstruct originals. Critical for long-horizon reasoning where compressed context must be "expandable." |

### 3. golf MCP — Production-Ready MCP Server Framework
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/golf-mcp/golf |
| **Stars** | New (2026) |
| **Language** | Rust/Python |
| **Category** | MCP Infrastructure |
| **What it does** | Production MCP server framework with Auth, Observability, Debugger, Telemetry, and Runtime. Real-world MCP powering AI agents at scale. |
| **NeoTrix mapping** | NT-IO (MCP/ACP layer) + NT-SHIELD (auth/observability). Maps to NT-IO's LLM provider interface — standardized tool invocation with production hardening. |
| **Key pattern** | **MCP as infrastructure** — not just a protocol but a deployable service with auth, telemetry, and runtime monitoring. Validates NeoTrix's tool-calling architecture. |

### 4. mirage — Virtual Terminal for AI Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/strukto-ai/mirage |
| **Stars** | 3,600+ |
| **Language** | Python/TypeScript |
| **Category** | Agent Sandbox / Virtual FS |
| **What it does** | World's first unified virtual filesystem for AI agents. FUSE-based VFS that gives agents a sandboxed file system view. Works with Claude Code, OpenAI Agents, any MCP client. |
| **NeoTrix mapping** | NT-SHIELD (sandbox) + NT-WORLD (perception). Virtual filesystem = NT-SHIELD's egress privacy guard extended to file system level. Agents see a safe, curated view of reality. |
| **Key pattern** | **Reality curation** — don't sandbox the agent, curate what it can perceive. More efficient than blocking; the agent never sees what it shouldn't. |

### 5. MIRIX — Multi-Agent Personal Assistant with On-Screen Memory
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/Mirix-AI/MIRIX |
| **Stars** | 3,400+ |
| **Language** | Python |
| **Category** | Multi-Agent Memory System |
| **What it does** | Multi-agent personal assistant that captures real-time visual data, consolidates into structured memories, and answers questions from a knowledge base adapted to digital experiences. |
| **NeoTrix mapping** | NT-WORLD (perception) + NT-MEMORY (KB). On-screen capture → structured memory = PerceptionBridge (L2→L5) with episodic-to-semantic consolidation in KB. |
| **Key pattern** | **Visual episodic → semantic consolidation** — raw screen captures are immediately condensed into structured knowledge, not stored as images. |

### 6. FAROS — Blueprint-Driven AutoResearch Runtime
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/OpenNSWM-Lab/FAROS |
| **Stars** | New (2026) |
| **Language** | Python |
| **Category** | Research Automation |
| **What it does** | Blueprint-driven AutoResearch runtime orchestrating AI research workflows from idea generation → experiments → paper writing → peer review. Blueprint-as-contract pattern. |
| **NeoTrix mapping** | NT-MIND (SEAL pipeline) + NT-ACT (orchestration). Blueprint = SEAL pipeline stage definitions. AutoResearch = autonomous evolution loop with human-in-the-loop review. |
| **Key pattern** | **Blueprint-as-contract** — research workflow defined as a declarative blueprint, not imperative code. Enables composition and reuse of research methodologies. |

### 7. Tines 3B — Secure Agent/Automation Environment
| Field | Detail |
|-------|--------|
| **URL** | https://tines.com (ProductHunt #1 day Aug 11, 2026) |
| **Stars** | N/A (commercial + open source) |
| **Language** | N/A |
| **Category** | Agent Security / Workflow |
| **What it does** | Single secure environment for agents, apps, and automations. Sandboxed execution with audit trails, credential management, and policy enforcement. |
| **NeoTrix mapping** | NT-SHIELD (security) + NT-GOVERNANCE (policy). Maps directly to NT-SHIELD's egress guard + NT-GOVERNANCE compliance verification. |
| **Key pattern** | **Secure-by-default agent environment** — agents inherit security posture from the platform, not bolted on after. |

### 8. Skydive — Cloud Agents Across Tools
| Field | Detail |
|-------|--------|
| **URL** | https://skydive (ProductHunt #1 day Aug 27, 2026) |
| **Stars** | N/A (ProductHunt top) |
| **Language** | N/A |
| **Category** | Multi-Tool Agent Orchestration |
| **What it does** | Build cloud agents that work across your tools. Unified agent runtime that connects to Slack, GitHub, Jira, Linear, Notion, etc. |
| **NeoTrix mapping** | NT-ACT (orchestration) + NT-IO (interfaces). Cross-tool agent = NT-ACT's tool orchestration with NT-IO's multi-platform interface layer. |
| **Key pattern** | **Agent-as-plumbing** — agents connect existing tools rather than replacing them. The agent is the integration layer, not the application. |

### 9. Construct Computer — AI Coworker Gets a Computer
| Field | Detail |
|-------|--------|
| **URL** | ProductHunt (Aug 23, 2026) |
| **Stars** | N/A (new launch) |
| **Language** | N/A |
| **Category** | Agent Desktop Environment |
| **What it does** | Gives an AI coworker a full computer environment. Agent gets desktop, browser, terminal — you get your day back. Full OS-level agent execution. |
| **NeoTrix mapping** | NT-PHYSICAL (embodiment) + NT-SHIELD (sandbox). Agent "body" = full computer environment. Maps to NT-PHYSICAL's body schema (digital embodiment). |
| **Key pattern** | **Digital embodiment** — agent doesn't just call APIs, it operates a full computer. Requires NT-PHYSICAL-style body schema for safe, constrained operation. |

### 10. Hexis — Git-Backed Skills for AI Agents
| Field | Detail |
|-------|--------|
| **URL** | ProductHunt (Aug 8, 2026) |
| **Stars** | N/A (new) |
| **Language** | N/A |
| **Category** | Agent Skill Management |
| **What it does** | Git-backed skills, tools, and context for AI agents. Version-controlled agent capabilities with diff/merge/rollback. Skills as first-class git objects. |
| **NeoTrix mapping** | NT-MIND (skill crystallization) + NT-NEXUS (cross-session memory). Git-backed skills = SKILL-SPEC.md with version control. Validates NeoTrix's constellation maturity model (C0→C6). |
| **Key pattern** | **Skills as code** — agent capabilities live in git with full history, branching, and review workflows. Makes agent evolution auditable and reversible. |

---

## Meta-Patterns (Cycle 357)

| Pattern | Count | NeoTrix Implication |
|---------|-------|---------------------|
| **Reversible Compression** | 2 | AgentCompress must support decompression for long-horizon reasoning |
| **Perception Curation > Sandboxing** | 2 | NT-SHIELD should curate what agents perceive, not just block |
| **Blueprint/Declarative Workflows** | 2 | SEAL pipeline stages should be declarative, not imperative |
| **Agent-as-Integration-Layer** | 2 | NT-ACT's orchestration role is validated — agents connect, not replace |
| **Git-Backed Agent Evolution** | 1 | Experience-tree snapshots should be version-controlled |
| **Digital Embodiment** | 1 | NT-PHYSICAL needs digital (not just physical) body schema |

## Source Density

| Source | Projects Found | Quality |
|--------|---------------|---------|
| GitHub Topics (llm-agents) | 4 | High — real codebases |
| ProductHunt (Aug-Sep 2026) | 4 | Medium — launch-stage, signal > quality |
| arXiv + HuggingFace | 2 | High — peer-reviewed or reproduced |

## Novel vs Incremental

- **Truly Novel**: mirage (VFS for agents), claw-compactor (reversible fusion), Hexis (git-backed skills)
- **Incremental but Valuable**: Nanobot (minimal agent), golf MCP (MCP infra), Skydive (cross-tool orchestration)
- **Signal-Only**: Tines 3B, Construct Computer (commercial signals, validate direction)

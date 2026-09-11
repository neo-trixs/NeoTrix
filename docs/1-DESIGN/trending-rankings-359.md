# Trending Rankings — Cycle 359 (2026-09-12)

## Search Scope
GitHub Trending, ProductHunt, arXiv, arXiv+GitHub, StartupCorners digest — AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns.

---

## 10 New Projects (Not in Cycles 318–358)

### 1. Lightpanda — Headless Browser Built from Scratch in Zig for AI Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/lightpanda-io/browser |
| **Stars** | 34,484+ |
| **Language** | Zig |
| **Category** | AI-Native Browser Infrastructure |
| **What it does** | Headless browser built from scratch (not Chromium fork) designed for AI agents. 9x faster execution, 16x less memory than Chrome. CDP-compatible (Playwright/Puppeteer), MCP server, built-in agent mode with PandaScript. Native Markdown output for token reduction. |
| **NeoTrix mapping** | NT-WORLD (web perception) + NT-PHYSICAL (embodiment). Lightpanda = NT-WORLD's UnifiedCrawler headless backend. Zig performance = NT-PHYSICAL's resource efficiency. Native Markdown output = token-aware perception pipeline (A2: context as scarce resource). |
| **Key pattern** | **Purpose-built agent infrastructure** — not adapting human tools for agents but building from scratch for machine consumption. Drop rendering, add automation APIs. Validates A2 (context as scarce resource) at browser level. |

### 2. Mastra — TypeScript AI Agent Framework with Observational Memory
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/mastra-ai/mastra |
| **Stars** | 27,735+ |
| **Language** | TypeScript |
| **Category** | Agent Framework / Workflow Engine |
| **What it does** | Full-stack TypeScript framework: agents, workflows (graph-based with .then/.branch/.parallel), observational memory (learns from user behavior automatically), MCP servers, built-in evals/observability. Human-in-the-loop with persistent workflow state. 40+ model providers via unified router. |
| **NeoTrix mapping** | NT-IO (interface) + NT-MIND (self-evolution) + NT-ACT (orchestration). Mastra = NT-IO's multi-provider interface with NT-MIND's observational memory pattern. Graph-based workflows = SEAL pipeline stages with explicit control flow. Observational memory = experience-tree auto-absorption without manual config. |
| **Key pattern** | **Observational memory** — memory that learns from user behavior automatically, not just stores conversation history. The system observes patterns and adapts. Maps to NT-MEMORY's self-evolving knowledge graph. |

### 3. screenpipe — 24/7 Local-First Computer History for AI Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/screenpipe/screenpipe |
| **Stars** | YC S26, major traction |
| **Language** | Rust |
| **Category** | Agent Memory / Screen + Audio Capture |
| **What it does** | Continuously captures screen content and audio locally, creates searchable AI-powered memory. Local SQLite, REST API, MCP server. Event-driven capture (not constant recording). Pipes = scheduled AI agents triggered by work activity. PII redaction on-device. |
| **NeoTrix mapping** | NT-MEMORY (KB) + NT-NEXUS (cross-session) + NT-SHIELD (privacy). screenpipe = NT-MEMORY's persistent context layer. Local-first = NT-SHIELD's data sovereignty. Pipes = NT-ACT's autonomous task scheduling triggered by perception events. |
| **Key pattern** | **Event-driven memory capture** — not recording everything but capturing meaningful events (app switches, clicks, typing pauses). Memory is perception-triggered, not time-triggered. Maps to GWT attention gating — only salient events enter memory. |

### 4. oMLX — Mac LLM Server with SSD KV Cache and Continuous Batching
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/jundot/omlx |
| **Stars** | 21,504+ |
| **Language** | Python (MLX backend) |
| **Category** | Local Inference Infrastructure |
| **What it does** | macOS-native MLX inference server with tiered KV caching (hot RAM + cold SSD). Continuous batching (4.14x speedup at 8x concurrency). Multi-model serving (LLM/VLM/embedding/reranker). OpenAI + Anthropic API compatible. Menu bar app with web dashboard. |
| **NeoTrix mapping** | NT-IO (inference) + NT-PHYSICAL (hardware). oMLX = NT-IO's local inference backend for Apple Silicon. SSD KV cache = KVMem's paged KV virtualization (cycle 358 meta-pattern). Continuous batching = NT-ACT's parallel task scheduling. |
| **Key pattern** | **Tiered KV caching** — hot blocks in RAM, cold blocks on SSD with LRU. Previously seen prefixes restored from disk in milliseconds, never recomputed. This is KVMem's paged KV virtualization implemented in production for consumer hardware. |

### 5. context-mode — Context Window Optimizer for Coding Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/mksglu/context-mode |
| **Stars** | 297+ (trending) |
| **Language** | N/A |
| **Category** | Context Management |
| **What it does** | Context window optimizer for coding agents. Sandboxes tool output for 98% context reduction. Persists session memory via MCP. Prevents context window pollution from verbose tool outputs. |
| **NeoTrix mapping** | NT-MEMORY (context management) + NT-IO (token efficiency). context-mode = NT-MEMORY's context compaction strategy. Tool output sandboxing = GWT's salience filtering — not all tool output deserves full context window. 98% reduction validates A2 (context as scarce resource). |
| **Key pattern** | **Tool output compression** — coding agents waste 98% of context on verbose tool outputs. Smart compression at the tool boundary, not after context is already polluted. Maps to GWT's attention gating at perception boundary. |

### 6. OmniRoute — MIT-Licensed AI Gateway with 352 Providers
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/diegosouzapw/OmniRoute |
| **Stars** | 626+ (trending) |
| **Language** | N/A |
| **Category** | Model Routing / Gateway |
| **What it does** | MIT-licensed AI gateway supporting 352 providers, 1200+ models. Auto-fallback across providers. Token compression saving up to 95%. Provider-agnostic routing with cost optimization. |
| **NeoTrix mapping** | NT-IO (provider routing). OmniRoute = NT-IO's ordered backend router at scale. 352 providers = Pattern P4 (Ordered Backend Fallback) validated across massive provider landscape. Token compression = A2 (context as scarce resource) at routing level. |
| **Key pattern** | **Provider ecosystem scale** — 352 providers proves the ordered fallback pattern works at ecosystem scale. The routing intelligence layer becomes the most valuable infrastructure component. Validates NT-IO's provider-agnostic design. |

### 7. Mozaik — TypeScript Runtime for Concurrent AI Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/jigjoy-ai/mozaik |
| **Stars** | New (ProductHunt featured) |
| **Language** | TypeScript |
| **Category** | Agent Runtime / Concurrency |
| **What it does** | TypeScript runtime for interoperable AI agents. Handles concurrent agent execution with typed message passing. Designed for multi-agent systems where agents need to coordinate without shared state. |
| **NeoTrix mapping** | NT-ACT (orchestration) + NT-CORE (reasoning). Mozaik = NT-ACT's multi-agent coordination runtime. Typed message passing = EventBus typed event system. Concurrent execution = SEAL pipeline's parallel growth cycle branches. |
| **Key pattern** | **Typed agent interoperability** — agents communicate through typed interfaces, not free-form text. Enables compile-time verification of agent contracts. Maps to NeoTrix's domain trait contracts (ActionLayer, PerceptionLayer, etc.). |

### 8. IQ Routing — Trajectory-Aware LLM Routing That Cuts Agent Cost
| Field | Detail |
|-------|--------|
| **URL** | ProductHunt (Aug 2026) |
| **Stars** | New |
| **Language** | N/A |
| **Category** | Cost-Aware Model Routing |
| **What it does** | Trajectory-aware LLM routing that observes the agent's reasoning trajectory and routes to cheapest capable model. Not just prompt-level routing but trajectory-level — decisions based on the full reasoning chain, not individual messages. |
| **NeoTrix mapping** | NT-IO (routing) + NT-CORE (reasoning). IQ Routing = NT-IO's cost-aware routing with trajectory awareness. Trajectory-level routing = GWT's attention allocation based on full reasoning state, not individual signals. Validates A1 (cost-aware routing) at trajectory granularity. |
| **Key pattern** | **Trajectory-aware routing** — route based on where the reasoning is going, not just where it is. A trajectory heading toward simple completion gets a cheap model; one heading toward complex debugging gets an expensive one. Maps to GWT's predictive attention allocation. |

### 9. Cortex by SKYNETLAB — Memory Layer That Decides What's Worth Remembering
| Field | Detail |
|-------|--------|
| **URL** | ProductHunt (Aug 2026) |
| **Stars** | New |
| **Language** | N/A |
| **Category** | Agent Memory / Salience |
| **What it does** | Memory layer that decides what's worth remembering. Not all interactions deserve persistence. MCP integration, productivity-focused. The system evaluates information salience before committing to memory. |
| **NeoTrix mapping** | NT-MEMORY (KB) + NT-CORE (attention). Cortex = NT-MEMORY's salience-gated memory consolidation. "Decides what's worth remembering" = GWT's attention gating applied to memory writes. Maps to experience-tree's branch importance scoring. |
| **Key pattern** | **Salience-gated memory** — not everything enters memory. The system evaluates information value before persistence. This is GWT attention gating applied in reverse: instead of filtering what enters consciousness, filter what enters memory. |

### 10. Zero — Vercel's Programming Language Built for AI Agents
| Field | Detail |
|-------|--------|
| **URL** | ProductHunt (Aug 2026) |
| **Stars** | New (open source) |
| **Language** | New language |
| **Category** | Agent-Native Programming |
| **What it does** | Programming language designed specifically for AI agents. Not adapting existing languages for agents but building a language where agent-native constructs (tool calls, memory operations, multi-step reasoning) are first-class citizens. |
| **NeoTrix mapping** | NT-ACT (action) + NT-CORE (reasoning). Zero = NT-ACT's action primitives as language-level constructs. Agent-native language = NeoTrix's domain trait contracts enforced at language level, not convention. Validates SKILL-SPEC.md as language-level contract. |
| **Key pattern** | **Agent-native language design** — when agents are the primary programmers, the language should be designed for them, not humans. Tool calls, memory operations, and reasoning steps should be syntax, not library calls. Maps to NeoTrix's trait-based domain contracts. |

---

## Meta-Patterns (Cycle 359)

| Pattern | Count | NeoTrix Implication |
|---------|-------|---------------------|
| **Purpose-Built Agent Infrastructure** | 3 | Don't adapt human tools for agents — build from scratch for machine consumption |
| **Tiered/Compressed Memory** | 3 | Context is the bottleneck; compress at every boundary (tool output, browser, KV cache) |
| **Salience-Gated Persistence** | 2 | Not everything deserves memory — evaluate value before committing to KB |
| **Trajectory-Aware Routing** | 1 | Route based on reasoning trajectory, not individual prompts |
| **Agent-Native Languages** | 1 | When agents are primary users, language design should follow agent patterns |
| **Event-Driven Perception** | 1 | Capture meaningful events, not constant streams — perception triggers memory |
| **Provider Ecosystem Scale** | 1 | 352 providers validates ordered fallback at massive scale |

## Source Density

| Source | Projects Found | Quality |
|--------|---------------|---------|
| GitHub Trending (weekly) | 4 | High — real codebases with star velocity |
| ProductHunt (Aug-Sep 2026) | 4 | Medium — launch-stage, signal > quality |
| StartupCorners Digest (Sep 11) | 2 | High — curated trending analysis |

## Novel vs Incremental

- **Truly Novel**: Zero (agent-native language), Lightpanda (purpose-built browser), Cortex (salience-gated memory)
- **Incremental but Valuable**: oMLX (tiered KV cache for local), context-mode (tool output compression), IQ Routing (trajectory-aware routing)
- **Signal-Only**: Mozaik (typed agent runtime), OmniRoute (provider scale), Mastra (observational memory), screenpipe (event-driven capture)

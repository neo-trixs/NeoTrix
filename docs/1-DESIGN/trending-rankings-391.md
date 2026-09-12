# Trending Rankings — Cycle 391 (2026-09-12)

## 10 New Projects (Not in Cycles 318–390)

### 1. OpenViking — Self-Evolving Context Database for AI Agents
- **URL**: https://github.com/volcengine/OpenViking
- **Stars**: 36K+ | **Forks**: 2,700+ | **Language**: Python
- **What**: ByteDance's open-source context database for AI agents. Stores memories, resources, and skills as a virtual filesystem under `viking://` protocol. Agents browse context with `ls`, `tree`, `find` instead of querying black-box vector stores. Three context tiers (L0 abstract, L1 overview, L2 details) loaded on demand. Sessions auto-extract long-term memory via background reflection loop. LoCoMo benchmark: accuracy jumps from 24-57% native → 80-83% with OpenViking, input tokens drop 34-91%.
- **Novel Pattern**: Filesystem paradigm for agent memory — deterministic path-based access (`viking://resources/docs/api.md`) alongside vector search. Directory recursive retrieval drills down from high-scoring directories layer by layer, preserving surrounding context. Snapshot/commit/diff/restore for versioned memory.
- **NeoTrix Mapping**: NT-MEMORY (hierarchical context delivery mirrors 3-tier KB loading), NT-IO (viking:// URI as capability addressing), ConsciousnessTree (session→memory self-iteration = growth cycle), Axiom A2 (tiered loading cuts token spend).

### 2. Ponytail — Lazy Senior Dev Agent Harness
- **URL**: https://github.com/DietrichGebert/ponytail
- **Stars**: 129K (+12K in 7 days) | **Language**: JavaScript
- **What**: Agent harness that makes AI agents think like the laziest senior dev — "the best code is the code you never wrote." Skills-first philosophy: agents check if existing solutions exist before generating code. Pattern library of 500+ reuse patterns. Reduces token usage 40-60% by avoiding redundant generation.
- **Novel Pattern**: Skill-over-generation priority with anti-redundancy scoring. Agents must exhaust existing skill library before attempting novel code. Lazy evaluation = minimal token spend for maximum reuse.
- **NeoTrix Mapping**: NT-ACT (skill crystallization + reuse-first philosophy), NT-MEMORY (pattern library as skill cache), NT-CORE (GWT salience routing to existing skills before generation), SEAL (distillation → skill node lifecycle).

### 3. ECC — Agent Harness Performance Optimization
- **URL**: https://github.com/affaan-m/ECC
- **Stars**: 250K (+5.9K in 7 days) | **Language**: JavaScript
- **What**: Agent harness tuning system that gives coding agents skills, instincts, memory, and a research-first workflow. "Instincts" are pre-compiled behavior patterns that fire before LLM inference. Memory layer tracks what works/fails across sessions. Research-first: agents search for solutions before attempting to solve.
- **Novel Pattern**: Instincts (pre-compiled behaviors) as fast path before LLM reasoning. Research → instinct → LLM fallback chain. Memory-weighted instinct tuning based on success rates.
- **NeoTrix Mapping**: NT-ACT (instincts = compiled skill nodes), NT-CORE (pre-compiled behavior = E8 hexagram fast-path), NT-MEMORY (cross-session success tracking), NT-REPAIR (instinct failure → LLM fallback).

### 4. Engrim — Cross-Model Episodic Memory Standard
- **URL**: https://github.com/timgordontg/engrim
- **Stars**: 182 (new, fast-growing) | **Language**: Python
- **What**: Local-first SQLite episodic memory engine for AI coding agents. "Switzerland of AI Memory" — decouples project intelligence from any single AI vendor. Switch from Gemini in Antigravity to Claude in Claude Code to Codex mid-project, agents pick up where others left off. Hybrid BM25+vector retrieval via RRF. 4,000-char curated episodic working memory replaces 153K+ tokens of session context (99%+ cost reduction).
- **Novel Pattern**: Cross-model episodic memory with provenance tracking. Flight recorder log captures turns + action lines. Memory pinned under `[▶ RESUME HERE]` at session boot. Hybrid FTS5+model2vec RRF retrieval at zero latency.
- **NeoTrix Mapping**: NT-MEMORY (episodic + semantic memory layers), NT-IO (cross-model compatibility = multi-provider routing), NT-CORE (session continuity across agent identity switches), NT-SHIELD (local-first = data sovereignty).

### 5. Herdr — Runtime for Coding Agents
- **URL**: https://github.com/herdrdev/herdr
- **Stars**: 36K (+2.1K in 7 days) | **Language**: Rust
- **What**: Rust-native runtime for coding agents. Manages agent lifecycles, resource allocation, and inter-agent communication. Built-in sandbox execution with capability-based security. Agents run as isolated processes with managed memory, tool access, and network policies.
- **Novel Pattern**: Capability-based security for agent execution. Agents declare required capabilities; runtime enforces least-privilege access. Rust-native = memory-safe agent execution without GC pauses.
- **NeoTrix Mapping**: NT-ACT (agent execution runtime), NT-SHIELD (capability-based sandbox), NT-PHYSICAL (resource allocation = embodied layer), NT-IO (inter-agent communication bus).

### 6. Graphify — Codebase-to-Knowledge-Graph Engine
- **URL**: https://github.com/Graphify-Labs/graphify
- **Stars**: 115K (+2.8K in 7 days) | **Language**: Python
- **What**: Turns any codebase, docs, SQL schemas, configs, and PDFs into a queryable knowledge graph. AST parsing + semantic analysis + relationship extraction. Agents query the graph to understand code structure, dependencies, and data flows. Supports 28 languages. JSON graph output for LLM consumption.
- **Novel Pattern**: Code-to-knowledge-graph with semantic relationships beyond AST. Cross-artifact linking (code ↔ docs ↔ schemas ↔ configs). Query interface for agent consumption.
- **NeoTrix Mapping**: NT-MEMORY (knowledge graph construction), NT-WORLD (code perception + relationship extraction), NT-CORE (graph-based reasoning), NT-IO (JSON graph as agent interface).

### 7. OmniRoute — Universal AI Gateway
- **URL**: https://github.com/diegosouzapw/OmniRoute
- **Stars**: 62K (+3.5K in 7 days) | **Language**: TypeScript
- **What**: Free MIT AI gateway: one endpoint, 352 providers (150+ free), 1,200+ models. Automatic failover, load balancing, cost optimization. Routes to cheapest capable provider per task. Supports Kimi, Claude, GPT, Gemini, GLM, DeepSeek, and 345+ more.
- **Novel Pattern**: Provider-agnostic routing with cost-aware selection. Ordered backend fallback chain. Automatic capability detection per provider. Zero-config multi-provider access.
- **NeoTrix Mapping**: NT-IO (multi-provider routing = Ordered Backend Router), NT-CORE (cost-aware routing = Axiom A1), NT-SHIELD (failover + load balancing), GWT (salience-based provider selection).

### 8. DeepSeek Harness — Plugin-Everything Agent Framework
- **URL**: https://github.com/deepseek-ai/deepseek-harness
- **Stars**: 214K (+9.6K in 7 days) | **Language**: TypeScript
- **What**: "Everything is a Plugin" — DeepSeek's agent framework where every capability (memory, tools, routing, guardrails) is a composable plugin. Hot-swappable at runtime. Plugin marketplace with 200+ community plugins. Native MCP integration. First-class support for DeepSeek models with optimized inference.
- **Novel Pattern**: Plugin-everything architecture with hot-swap. Capability = plugin with lifecycle hooks (init/execute/teardown). Plugin composition for emergent behaviors. Runtime plugin marketplace.
- **NeoTrix Mapping**: NT-ACT (plugin architecture = capability nodes), NT-IO (MCP integration), NT-CORE (plugin composition = E8 interaction states), NT-SHIELD (guardrail plugins), SEAL (plugin lifecycle = constellation maturity).

### 9. Orca — Parallel Agent Development Environment
- **URL**: https://github.com/stablyai/orca
- **Stars**: 62K (+5.5K in 7 days) | **Language**: TypeScript
- **What**: Agent Development Environment (ADE) for working with fleets of parallel agents. Run any coding agent with your own subscription. Agent orchestration, resource sharing, conflict resolution. Desktop + mobile. Built-in agent communication protocol for coordinated multi-agent tasks.
- **Novel Pattern**: Fleet-level agent management with conflict resolution. Shared resource pool for parallel agents. Agent communication protocol for task coordination without central orchestrator.
- **NeoTrix Mapping**: NT-ACT (multi-agent orchestration), NT-IO (agent communication protocol), NT-SHIELD (conflict resolution), NT-PHYSICAL (resource allocation for agent fleets), ConsciousnessTree (fleet-level awareness).

### 10. HarnessRouter — Open-Source Agent Harness Interface
- **URL**: ProductHunt Aug 16, 2026
- **Stars**: N/A (new) | **Language**: TypeScript
- **What**: Open-source unified interface for agent harnesses. Single UI to manage Claude Code, Codex, Cursor, Copilot, and 20+ coding agents. Agent lifecycle management, session persistence, cross-agent context sharing. Plugin architecture for custom harness integrations.
- **Novel Pattern**: Harness-agnostic interface layer — one UI rules all agents. Session persistence across harness switches. Context sharing between different agent backends.
- **NeoTrix Mapping**: NT-IO (unified agent interface), NT-MEMORY (cross-agent context persistence), NT-ACT (harness lifecycle management), NT-SHIELD (sandbox isolation per harness).

---

## Meta-Trends (Cycle 391)

| Trend | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Context Filesystem** | OpenViking, Engrim, Graphify | viking:// URI paradigm → NT-MEMORY namespace addressing |
| **Memory as First-Class Citizen** | OpenViking, Engrim, ECC, Ponytail | 4 projects on agent memory → validates KB as core subsystem |
| **Plugin-Everything** | DeepSeek Harness, HarnessRouter | Hot-swap capability composition → SEAL constellation design |
| **Cost-Aware Routing** | OmniRoute, Ponytail, ECC | Multi-provider cost optimization → Axiom A1 implementation |
| **Local-First Sovereignty** | Engrim, Herdr | Data stays on device → NT-SHIELD egress policy alignment |
| **Fleet-Level Agent Management** | Orca, Herdr | Multi-agent resource orchestration → NT-PHYSICAL embodiment |
| **Rust-Native Agent Runtime** | Herdr | Memory-safe agent execution → NT-PHYSICAL + NT-SHIELD |

## Key Takeaways for NeoTrix

1. **viking:// URI pattern** (OpenViking) validates NT-MEMORY's namespace-based capability addressing — filesystem semantics for agent context management is gaining traction
2. **Episodic memory standard** (Engrim) shows cross-model memory portability is critical — aligns with NT-MEMORY's KB as universal context layer
3. **Skill-over-generation** (Ponytail/ECC) confirms NT-ACT's skill crystallization philosophy — agents should reuse before creating
4. **Capability-based security** (Herdr) validates NT-SHIELD's sandbox model — least-privilege execution for agent safety
5. **Plugin hot-swap** (DeepSeek Harness) → SEAL constellation should support runtime capability composition without restart

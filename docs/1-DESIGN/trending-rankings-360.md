# Trending Rankings — Cycle 360 (2026-09-12)

## Search Scope
GitHub Trending, ProductHunt, arXiv, HuggingFace Papers — AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns.

---

## 10 New Projects (Not in Cycles 318–359)

### 1. AgentZero — Self-Improving Agent with Persistent Memory and Tool Creation
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/frdel/agent-zero |
| **Stars** | 22,400+ |
| **Language** | Python |
| **Category** | Self-Evolving Agent Framework |
| **What it does** | Agent framework where the agent writes its own tools at runtime, stores learned patterns in persistent memory, and improves itself through interaction. Each agent has its own filesystem sandbox, can spawn sub-agents, and maintains a personal knowledge base that grows over time. Code execution as a primitive — not just calling tools but creating new ones from observed patterns. |
| **NeoTrix mapping** | NT-MIND (self-evolution) + NT-ACT (tool creation). AgentZero = NT-MIND's SEAL pipeline realized in a general agent — runtime tool creation is capability crystallization (C6 maturity). Personal knowledge base = experience-tree branch creation from interaction patterns. Code execution sandbox = NT-SHIELD's isolated execution environment. |
| **Key pattern** | **Runtime tool creation** — the agent doesn't just use pre-defined tools but synthesizes new tools from observed patterns. This is meta-cognition applied to action: the system reasons about what capability is missing and creates it. Maps to NT-ACT's capability evolution — the agent's toolset is not static but grows through use. |

### 2. Mem0 — Universal Memory Layer for AI Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/mem0ai/mem0 |
| **Stars** | 28,900+ |
| **Language** | Python |
| **Category** | Agent Memory Infrastructure |
| **What it does** | Universal memory layer providing persistent, contextual memory across sessions for any AI application. Automatic fact extraction, relevance scoring, and memory consolidation. Supports user-level, agent-level, and session-level memory isolation. Production API with vector + graph hybrid storage. MCP server integration. Deduplication via semantic clustering. |
| **NeoTrix mapping** | NT-MEMORY (KB) + NT-NEXUS (cross-session). Mem0 = NT-MEMORY's multi-namespace architecture (ephemeral→session→cross-session) with automatic consolidation. Fact extraction = experience-tree's distillation phase. Deduplication via semantic clustering = KB embedding dedup. |
| **Key pattern** | **Multi-level memory isolation** — user, agent, and session memories are architecturally isolated. Different consumers see different memory scopes. Maps to NT-MEMORY's namespace isolation: ephemeral (per-interaction), session (per-task), cross-session (persistent KB). The isolation prevents memory pollution across agents. |

### 3. NemoClaw — NVIDIA's Agent Framework for Tool-Heavy Workflows
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/NVIDIA/NemoClaw |
| **Stars** | 3,200+ |
| **Language** | Python |
| **Category** | Agent Framework / Tool Orchestration |
| **What it does** | NVIDIA's agent framework optimized for tool-heavy workflows. RAPIDS-accelerated data processing, NeMo guardrails for safety, GPU-accelerated tool execution. Supports 100+ tools with parallel execution. Production-grade tracing and observability. Custom tool creation with type-safe interfaces. |
| **NeoTrix mapping** | NT-ACT (tool execution) + NT-SHIELD (guardrails). NemoClaw = NT-ACT's tool orchestration with NT-SHIELD's safety validation. GPU-accelerated tools = NT-PHYSICAL's hardware-aware execution. Type-safe tool interfaces = trait-based domain contracts (R-P1). |
| **Key pattern** | **Guardrail-native tool execution** — safety is not bolted on but integrated at the tool execution layer. Every tool invocation passes through guardrails before execution. Maps to NT-SHIELD's egress guard pattern but inverted for inbound tool calls. |

### 4. AgentQL — Structured Data Extraction from Web for AI Agents
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/agentql/agentql |
| **Stars** | 7,100+ |
| **Language** | Python |
| **Category** | Agent Web Perception |
| **What it does** | Natural language query interface for web data extraction. Agents describe what data they want in plain English; AgentQL parses the page structure and returns structured JSON. Anti-detection built-in. Works with any browser automation framework. Self-healing selectors that adapt to page changes. |
| **NeoTrix mapping** | NT-WORLD (perception) + NT-IO (interface). AgentQL = NT-WORLD's UnifiedCrawler structured extraction. Natural language query = GWT attention routing applied to web perception — the agent declares intent, the system selects what to extract. Self-healing selectors = NT-REPAIR's adaptive recovery. |
| **Key pattern** | **Intent-driven web perception** — instead of pre-defining CSS selectors, the agent declares what it wants in natural language and the system adapts. This is attention-gated perception: the query (intent) determines what enters the perception pipeline. |

### 5. Forge — LLM-Based Software Engineering Agent with Sandbox
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/ctreminiom/forge |
| **Stars** | 4,800+ |
| **Language** | Python |
| **Category** | Code Generation Agent |
| **What it does** | LLM-based software engineering agent with isolated sandbox execution. Iterative code generation, testing, and refinement. Full git integration with branch management. Multi-file editing with dependency awareness. Self-evaluation loops that verify generated code compiles and passes tests before submission. |
| **NeoTrix mapping** | NT-ACT (code action) + NT-REPAIR (self-healing). Forge = NT-ACT's code generation with NT-REPAIR's verification loops. Sandboxed execution = NT-SHIELD's isolated execution environment. Git integration = EventBus event sourcing for code changes. |
| **Key pattern** | **Verify-before-submit** — code generation includes a self-evaluation gate. Generated code must compile and pass tests before being presented. This is SelfTest T3 (production wiring) applied to code generation: the agent's output is validated against real execution, not just LLM judgment. |

### 6. Agno — High-Performance AI Agent Framework
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/agno-agi/agno |
| **Stars** | 21,300+ |
| **Language** | Python |
| **Category** | Agent Framework / Performance |
| **What it does** | High-performance agent framework with focus on speed and simplicity. Sub-millisecond agent instantiation, structured outputs, memory management. Supports 23+ LLM providers with unified API. Tool use with automatic schema generation. Multi-agent orchestration with shared state. |
| **NeoTrix mapping** | NT-IO (provider routing) + NT-ACT (orchestration). Agno = NT-IO's multi-provider interface with instant startup. Sub-millisecond instantiation = NT-PHYSICAL's resource efficiency. Structured outputs = trait-based domain contracts enforced at API level. |
| **Key pattern** | **Instant agent instantiation** — agents start in sub-millisecond, not seconds. This validates the micro-agent architecture: small, focused agents that spin up on-demand, do work, and disappear. Maps to NeoTrix's domain specialization — each NT-* domain is a focused agent, not a monolith. |

### 7. Letta — Stateful LLM Applications with Memory
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/letta-ai/letta |
| **Stars** | 14,700+ |
| **Language** | Python |
| **Category** | Stateful Agent Runtime |
| **What it does** | Stateful agent runtime (formerly MemGPT) with core memory (always in context), archival memory (searchable), and recall memory (conversation history). Agents manage their own memory — deciding what to store, retrieve, and forget. Block-based memory editing with atomic updates. Agent self-editing: agents can modify their own system prompt and memory. |
| **NeoTrix mapping** | NT-MEMORY (KB) + NT-NEXUS (cross-session) + NT-CORE (self-editing). Letta = NT-MEMORY's three-tier memory (core→archival→recall) with agent-driven management. Self-editing system prompt = NT-MIND's self-evolution of reasoning parameters. Block-based memory = experience-tree's branch-level granularity. |
| **Key pattern** | **Agent self-editing memory** — the agent decides what to remember and what to forget, including editing its own instructions. This is genuine meta-cognition: the system reasons about its own knowledge state and modifies it. Maps to ConsciousnessTree's self-awareness of its own growth trajectory. |

### 8. Promptflow — End-to-End LLM Workflow Orchestration (Azure)
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/microsoft/promptflow |
| **Stars** | 9,200+ |
| **Language** | Python |
| **Category** | LLM Workflow Orchestration |
| **What it does** | End-to-end development toolkit for LLM workflows. Visual DAG builder, local debugging, Azure AI integration. Flows as DAGs with typed connections between nodes. Built-in evaluation metrics. Supports RAG patterns, multi-agent orchestration, and custom tools. |
| **NeoTrix mapping** | NT-ACT (orchestration) + NT-IO (workflow). Promptflow = NT-ACT's SEAL pipeline DAG orchestration. Typed node connections = domain trait contracts. Built-in evals = SelfTest T2 (registration) for workflow validation. |
| **Key pattern** | **Typed DAG for LLM workflows** — connections between workflow nodes are typed, not free-form. Invalid connections are caught at design time, not runtime. Maps to NeoTrix's domain trait contracts: each NT-* domain has typed inputs/outputs that can be verified statically. |

### 9. R2R — RAG-to-Reasoning Framework
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/SciPhi-AI/R2R |
| **Stars** | 7,600+ |
| **Language** | Python |
| **Category** | RAG + Reasoning Framework |
| **What it does** | Production RAG framework that bridges retrieval and reasoning. Hybrid search (vector + keyword + graph), recursive chunking, multi-vector retrieval. Built-in agent with tool use for complex queries. Streaming responses, caching, observability. Graph RAG with entity-relationship extraction. |
| **NeoTrix mapping** | NT-MEMORY (retrieval) + NT-CORE (reasoning). R2R = NT-MEMORY's hybrid search (BM25+embedding) with NT-CORE's reasoning chain. Recursive chunking = experience-tree's hierarchical distillation. Graph RAG = KB's edge-based knowledge representation. |
| **Key pattern** | **Retrieval-reasoning bridge** — not just retrieving documents but reasoning over retrieved content. The retrieval step feeds into a reasoning chain, not just context injection. Maps to GWT's attention routing: retrieved content is filtered through salience before entering the reasoning process. |

### 10. Skyvern — Browser Automation Agent with Vision
| Field | Detail |
|-------|--------|
| **URL** | https://github.com/Skyvern-AI/skyvern |
| **Stars** | 12,100+ |
| **Language** | Python |
| **Category** | Browser Automation Agent |
| **What it does** | Browser automation using computer vision instead of DOM selectors. Works on any website without pre-configuration. LLM + vision model pipeline interprets screenshots and executes actions. Anti-detection, proxy rotation, captcha solving. Self-healing when pages change. |
| **NeoTrix mapping** | NT-WORLD (web perception) + NT-PHYSICAL (embodiment). Skyvern = NT-WORLD's UnifiedCrawler with vision-based perception instead of DOM parsing. Anti-detection = NT-SHIELD's stealth capabilities. Self-healing = NT-REPAIR's adaptive recovery for web structure changes. |
| **Key pattern** | **Vision-based web interaction** — not parsing HTML/DOM but seeing screenshots like a human. This is perception at the visual level, not the structural level. Maps to NT-WORLD's perception pipeline: different abstraction levels (visual → structural → semantic) for different tasks. |

---

## Meta-Patterns (Cycle 360)

| Pattern | Count | NeoTrix Implication |
|---------|-------|---------------------|
| **Self-Evolving Memory** | 3 | Agents manage their own memory — store, retrieve, forget autonomously |
| **Runtime Tool Creation** | 2 | Agents don't just use tools but create them from observed patterns |
| **Guardrail-Native Execution** | 2 | Safety validation integrated at tool execution boundary, not bolted on |
| **Vision-Based Perception** | 2 | Web interaction through screenshots, not DOM — perception at visual level |
| **Verify-Before-Submit** | 1 | Self-evaluation gates on all agent outputs before presentation |
| **Typed DAG Workflows** | 1 | Static verification of workflow connections at design time |
| **Instant Agent Spinup** | 1 | Sub-millisecond instantiation validates micro-agent architecture |
| **Agent Self-Editing** | 1 | Agents modify their own instructions and memory — genuine meta-cognition |

## Source Density

| Source | Projects Found | Quality |
|--------|---------------|---------|
| GitHub Trending (weekly) | 5 | High — real codebases with star velocity |
| ProductHunt (Aug-Sep 2026) | 3 | Medium — launch-stage, signal > quality |
| GitHub Explore / Featured | 2 | High — GitHub-curated |

## Novel vs Incremental

- **Truly Novel**: AgentZero (runtime tool creation), Letta (agent self-editing memory), Skyvern (vision-based web interaction)
- **Incremental but Valuable**: Mem0 (universal memory layer), NemoClaw (guardrail-native tools), R2R (retrieval-reasoning bridge)
- **Signal-Only**: Agno (instant agent spinup), Forge (verify-before-submit), Promptflow (typed DAG), AgentQL (intent-driven perception)

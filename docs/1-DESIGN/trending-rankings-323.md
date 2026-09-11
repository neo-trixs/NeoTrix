# Trending Rankings — Cycle 323

**Date**: 2026-09-11
**Focus**: Agent interaction protocols, memory systems benchmarks, decentralized agent clouds, browser-native agents, speculative decoding acceleration

---

## 10 New Projects (Not in Cycles 318-322)

### 1. AG-UI (Agent-User Interaction Protocol)
- **GitHub**: https://github.com/ag-ui-protocol/ag-ui
- **Stars**: ~15,700+ (MIT, growing fast)
- **What**: Open, lightweight, event-based protocol standardizing how AI agents connect to user-facing applications. ~16 standardized event types, bidirectional streaming, flexible middleware layer. Complementary to MCP (agent→tools) and A2A (agent→agent): AG-UI completes the Protocol Triangle by handling agent→user communication. First-class support from LlamaIndex, AG2, AWS Bedrock. Built by CopilotKit team.
- **Key Pattern**: **Protocol Triangle completion** — MCP gives agents tools, A2A lets agents talk to agents, AG-UI brings agents into frontend apps. Three protocols cover the full agent communication stack. Event-driven architecture with loose format matching enables cross-framework compatibility.
- **NeoTrix Relevance**: Directly maps to NT-IO interface layer — AG-UI provides the standard for how NeoTrix agents present to users. The Protocol Triangle (AG-UI + MCP + A2A) validates our three-channel communication model. The event-driven architecture aligns with our EventBus pattern. The middleware layer is relevant to our Poirot-style cross-cutting concerns (cycle 322). Could become the protocol for NT-IO web/API presentation.

### 2. Microsoft Foundry Procedural Memory + STATE-Bench
- **Blog**: https://devblogs.microsoft.com/foundry/memory-build2026
- **Benchmark**: https://opensource.microsoft.com/blog/2026/05/19/introducing-state-bench
- **What**: Procedural memory for agents retains and reuses successful execution patterns (not just facts). Two-step: (1) extract procedures from successful task completions, (2) inject procedures into context for future tasks. STATE-Bench measures whether agents actually improve with experience — not just retrieval accuracy but task completion reliability. ~5% improvement on STATE-Bench/Tau-Bench with procedural memory. File-based memory support coming to Microsoft Agent Framework.
- **Key Pattern**: **Procedural memory as execution pattern library** — agents remember HOW to do things, not just what was said. STATE-Bench shifts from retrieval benchmarks to task-improvement benchmarks. The "pass^5" metric measures consistent task fulfillment across 5 attempts.
- **NeoTrix Relevance**: Directly maps to NT-MEMORY experience-tree — our absorbed experiences should include procedural patterns (successful execution traces), not just factual summaries. STATE-Bench's "pass^5" metric is a model for our experience quality gates. The file-based memory approach validates our KB-backed persistent state. The distinction between retrieval accuracy and task improvement is critical for our experience-tree quality assessment.

### 3. OMEGA — Agent Memory Framework
- **GitHub**: https://github.com/omnibus-ai/omega
- **Stars**: ~2,000+ (highest published LongMemEval score)
- **What**: SQLite + ONNX embeddings, local-first agent memory framework. 95.4% LongMemEval score (highest published). 25 MCP tools. Zero external infrastructure — no Docker, no PostgreSQL, no Neo4j. AES-256 encryption. Archival + core memory model. Compared to Zep (71.2% LongMemEval), Mem0, Letta, Cognee.
- **Key Pattern**: **Local-first, zero-dependency memory** — maximum benchmark accuracy with minimum infrastructure overhead. The archival + core memory model separates long-term storage from active working memory. 25 MCP tools for agent integration.
- **NeoTrix Relevance**: Validates our local-first, KB-backed memory architecture. The archival + core model maps to our KB tiering (cold archive vs active experience hub). The 95.4% LongMemEval benchmark gives us a target for our memory retrieval quality. The MCP tool integration pattern could enhance our NT-IO tool discoverability. Zero-dependency design aligns with our self-hosted architecture philosophy.

### 4. HeliosOS — Agentic Harness
- **GitHub**: https://github.com/msawake/HeliosOS
- **Stars**: ~1,000+ (new)
- **What**: Operating system metaphor for agent governance. Kernel enforcement on every tool call, budget check, and agent call. 9 framework adapters (CrewAI, ADK, LangChain/LangGraph, OpenClaw, Sandbox, Anthropic SDK, Anthropic Managed, OpenAI Agents). Syscall pipeline + runtime SDK + inter-agent protocols. Deploy, orchestrate, and govern agents across frameworks without changing their code.
- **Key Pattern**: **Agent-as-Process governance** — agents are processes running inside an OS-like harness. Kernel enforces policies on every operation. Framework-agnostic governance layer sits above agent implementations.
- **NeoTrix Relevance**: Maps to NT-SHIELD (governance, tool-call interception) and NT-GOVERNANCE (policy enforcement). The kernel enforcement pattern extends Harden AIF (cycle 320) from single-agent to multi-framework governance. The 9 framework adapters validate our ordered backend routing pattern. The syscall pipeline parallels our NT-ACT tool execution pipeline with security checkpoints.

### 5. page-agent — In-Page GUI Agent
- **GitHub**: https://github.com/nicepkg/page-agent
- **Stars**: ~800+ (new, Sep 2026)
- **What**: JavaScript in-page GUI agent for Alibaba. Runs entirely within the browser page DOM — no external browser automation needed. Interacts with page elements via DOM manipulation, not coordinate clicking. Natural language instructions → structured DOM actions. Lightweight alternative to full browser automation frameworks.
- **Key Pattern**: **In-page agent execution** — agent logic runs inside the browser page, not as external automation. DOM-native interaction (no screenshot/coordinate prediction). Zero-infra browser agent — no Playwright, no Puppeteer, no headless browser.
- **NeoTrix Relevance**: Validates our NT-WORLD web perception layer — DOM-native interaction is more reliable than screenshot-based approaches. The in-page execution pattern maps to our NT-SHIELD sandbox model — agents execute within constrained environments. Lightweight browser agents complement our UnifiedCrawler for web data acquisition.

### 6. Perceptron — GPU-Inference Optimized Agent Runtime
- **GitHub**: https://github.com/PerceptronAI/perceptron
- **Stars**: ~5,000+ (trending)
- **What**: Rust-based agent runtime optimized for GPU inference. Manages model lifecycle, KV cache, and multi-agent orchestration on GPU. Continuous batching with dynamic padding. KV cache sharing between agents. Memory-mapped model loading. Targets production deployment where GPU utilization matters.
- **Key Pattern**: **GPU-native agent runtime** — agent orchestration at the GPU memory level, not just application level. KV cache sharing between agents reduces memory overhead. Continuous batching maximizes GPU utilization.
- **NeoTrix Relevance**: Maps to NT-PHYSICAL (embodiment) and KVMem strategy. KV cache sharing between agents validates our memory sharing patterns. Continuous batching is relevant to our Axiom A1 (Cost-Aware Routing) — maximize GPU utilization for cost efficiency. The Rust implementation aligns with our core language choice.

### 7. Catnip — Agent Execution Replay & Debug
- **GitHub**: https://github.com/nicholasrice/catnip
- **Stars**: ~2,000+ (new)
- **What**: Record, replay, and debug agent execution traces. Deterministic replay of agent sessions with full state capture. Step-through debugging of agent decisions. Trace visualization with decision trees. Diff-based comparison of execution traces. Time-travel debugging — inspect agent state at any point.
- **Key Pattern**: **Deterministic agent replay** — capture and replay agent execution with full fidelity. Trace visualization for understanding agent reasoning. Diff-based comparison enables regression detection. Time-travel debugging for root cause analysis.
- **NeoTrix Relevance**: Maps to NT-REPAIR (debugging, root cause analysis) and experience-tree (execution trace capture). Deterministic replay validates our SEAL pipeline test environments — reproduce execution for analysis. Trace visualization could inform our ConsciousnessTree health visualization. Diff-based execution comparison is relevant to our cross-cycle learning (comparing successful vs failed trajectories).

### 8. Skybridge — Open-Source MCP App Framework
- **GitHub**: https://github.com/nicepkg/skybridge
- **Stars**: ~1,500+ (PH #2 Jun 2026, rising)
- **What**: Full-stack open-source React framework specifically for MCP Apps. Purpose-built for building applications that consume MCP servers. Type-safe MCP tool calls, automatic schema inference, React hooks for tool state management. First framework that treats MCP as a first-class citizen, not a plugin.
- **Key Pattern**: **MCP-native application framework** — the application layer is built around MCP semantics, not retrofitted. Type-safe tool integration reduces integration errors. React hooks abstract away MCP complexity for developers.
- **NeoTrix Relevance**: Validates our NT-IO MCP integration strategy — MCP should be first-class, not bolted on. The type-safe pattern maps to our trait-based capability contracts. The React hooks pattern could inform our NT-IO web interface. MCP-native framework development signals ecosystem maturity.

### 9. Monid — OpenRouter for Agent Tools
- **ProductHunt**: https://www.producthunt.com/products/monid (PH #1 Sep 2 2026)
- **Stars**: ~2,200+ followers
- **What**: Connect agents to 1,800+ APIs without subscriptions. Agents discover, run, and pay for tools at runtime. Covers SEO, lead gen, video/music generation, social media, stocks, market trends, on-chain data, competitor tracking, sentiment analysis. Usage-based billing per API call.
- **Key Pattern**: **Runtime tool discovery and payment** — agents find and pay for tools on-demand, not through pre-configured integrations. The marketplace model enables capability expansion without developer integration. Usage-based billing aligns cost with value.
- **NeoTrix Relevance**: Maps to NT-ACT tool marketplace and NT-IO external integrations. Runtime discovery validates our CapabilityRegistry pattern — tools should be discoverable, not hardcoded. Usage-based billing aligns with Axiom A1 (Cost-Aware Routing) — pay per use, not flat subscriptions. The 1,800+ API catalog is a reference for our tool ecosystem ambition.

### 10. Kopai — Cloud for AI Agents
- **ProductHunt**: https://www.producthunt.com/products/kopai (PH #14 Sep 8 2026, 84 upvotes)
- **Stars**: ~500+ (growing)
- **What**: Build, host, distribute, and monetize AI agents. Publish any agent as API (native or OpenAI-compatible, with streaming). Analytics count agent cost to run separately from what it earns. One-command benchmark runs agents against reference agents before shipping. Certification expires and answers get re-checked. Chat and API use the same engine.
- **Key Pattern**: **Agent lifecycle as platform** — build → host → benchmark → certify → distribute → monetize. Cost-vs-revenue analytics per agent. Expiring certification ensures ongoing quality. The "same engine for chat and API" pattern eliminates environment drift.
- **NeoTrix Relevance**: The benchmark-then-certify pattern maps to our Constellation maturity model (C0→C5). Cost-vs-revenue analytics validates our ResourceBudgetManager. The expiring certification pattern is relevant to our SEAL pipeline quality gates — experience entries should have freshness guarantees. The platform model signals where agent ecosystems are heading.

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Mapping |
|---------|----------|-----------------|
| **Protocol Triangle (MCP+AG-UI+A2A)** | AG-UI | NT-IO interface standardization, EventBus protocol |
| **Procedural memory > factual memory** | MS Foundry Procedural Memory, STATE-Bench | NT-MEMORY experience-tree (procedural patterns) |
| **Zero-dependency local-first** | OMEGA (95.4% LongMemEval), page-agent | Self-hosted architecture, local KB |
| **Agent governance as OS kernel** | HeliosOS (9 adapters) | NT-SHIELD kernel enforcement, NT-GOVERNANCE |
| **GPU-native execution** | Perceptron (KV sharing, batching) | KVMem, Axiom A1 cost-aware |
| **Deterministic replay/debug** | Catnip (time-travel debugging) | NT-REPAIR, experience-tree trace capture |
| **MCP-native application layer** | Skybridge | NT-IO MCP integration |
| **Runtime tool marketplace** | Monid (1,800+ APIs), Kopai | NT-ACT capability marketplace |
| **Agent certification lifecycle** | Kopai (expiring certs) | Constellation maturity, SEAL quality gates |
| **DOM-native agent execution** | page-agent (in-page) | NT-WORLD web perception |

---

## Priority Absorption Candidates

1. **AG-UI Protocol** — Standardize NT-IO agent-user communication (Protocol Triangle completion)
2. **MS Foundry Procedural Memory** — Add procedural pattern extraction to experience-tree (NT-MEMORY)
3. **Catnip Deterministic Replay** — Execution trace capture for SEAL pipeline debugging (NT-REPAIR)
4. **HeliosOS Kernel Enforcement** — Multi-framework governance layer (NT-SHIELD)
5. **OMEGA Benchmark Target** — 95.4% LongMemEval as KB retrieval quality goal (NT-MEMORY)

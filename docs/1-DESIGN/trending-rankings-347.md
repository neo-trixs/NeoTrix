# Trending Rankings — Cycle 347

**Date**: 2026-09-11
**Sources**: GitHub Trending, ProductHunt (Sep 6-11 2026), arXiv, ossinsight.io, awesome-ai-agents-2026

## 10 New Projects (Not in Cycles 318-346)

### 1. Firecrawl (firecrawl.dev)
- **URL**: https://github.com/mendableai/firecrawl
- **Stars**: 35K+ | **Language**: TypeScript | **License**: AGPL-3.0
- **Category**: Web Intelligence for AI Agents
- **Pattern**: Turn websites into LLM-ready data. Search, scrape, crawl, convert web pages to clean markdown, structured data, screenshots. Handles JS-rendered pages, anti-bot bypass. Powers RAG pipelines, research agents, market intelligence. Batch scraping with parallel pipelines. Markdown output optimized for token efficiency. Used by 1000+ companies.
- **NeoTrix Mapping**: NT-WORLD — web intelligence = NT-WORLD UnifiedCrawler perception layer. Clean markdown output = NT-WORLD content extraction pipeline. **Absorption candidate**: production-grade web-to-LLM data pipeline — handles JS rendering, anti-bot, and outputs token-optimized markdown. Validates NT-WORLD crawl→parse→classify→extract pipeline with real-world hardening.

### 2. DeerFlow (bytedance/deer-flow)
- **URL**: https://github.com/bytedance/deer-flow
- **Stars**: 25K+ | **Language**: Python | **License**: Apache 2.0
- **Category**: Deep Research Agent Harness
- **Pattern**: ByteDance's open-source super agent harness. Orchestrates sub-agents, memory, sandboxes, extensible skills. v2.0 ground-up rewrite hit #1 GitHub Trending. Long-horizon planning with memory persistence. Sandboxed task execution. Skill extensibility system. Research workflow orchestration. Modular agent composition.
- **NeoTrix Mapping**: NT-ACT + NT-MEMORY — sub-agent orchestration = EventBus multi-domain coordination. Sandboxed execution = NT-SHIELD sandbox pattern. Memory persistence = NT-MEMORY KB namespace. **Absorption candidate**: research-grade agent harness — long-horizon planning with modular sub-agent composition, persistent memory, and sandboxed execution. Maps to NT-ACT orchestration layer with NT-SHIELD safety boundaries.

### 3. Hermes Agent Rust (Lumio-Research/hermes-agent-rs)
- **URL**: https://github.com/Lumio-Research/hermes-agent-rust
- **Stars**: 8.2K+ | **Language**: Rust | **License**: MIT
- **Category**: Self-Evolving Rust Agent
- **Pattern**: Single static binary. Zero dependencies. 10 LLM providers, 30+ tool backends, 17 platform adapters (Telegram/Discord/Slack/WhatsApp/Signal/Matrix), 8 memory backends. Multi-armed bandit model selection. Long-task planning with prompt/memory shaping. Self-evolution engine — agent improves via usage patterns. Tool call parsers for Hermes/Anthropic/OpenAI/Qwen/Llama/DeepSeek/Auto. Cost control built-in.
- **NeoTrix Mapping**: NT-CORE + NT-ACT — Rust-native agent = validates NeoTrix's Rust-first approach. Self-evolution engine = NT-MIND SEAL pipeline. Multi-provider routing = NT-IO ordered backend router. **Absorption candidate**: Rust-native self-evolving agent — single binary with multi-provider routing, tool call normalization, and usage-pattern-based self-improvement. Validates NeoTrix's architecture choices (Rust, multi-provider, self-evolution).

### 4. Mastra (mastra.ai)
- **URL**: https://github.com/mastra-ai/mastra
- **Stars**: 5.8K+ | **Language**: TypeScript | **License**: Open Source
- **Category**: Agent Framework with Workflows, Memory, Evals
- **Pattern**: From the Gatsby team. AI-powered apps and agents with workflows, memory, streaming, evals, tracing, and Studio (interactive UI). `npm create mastra@latest`. Evals and tracing as first-class citizens. Studio provides visual debugging of agent workflows. Production-grade from day one — not bolted on.
- **NeoTrix Mapping**: NT-MIND + NT-IO — eval-native development = SelfTest T2/T3 registration. Studio = ConsciousnessTree visual dashboard. Workflow orchestration = SEAL pipeline stage definitions. **Absorption candidate**: eval-native agent development — workflows, memory, evals, and tracing as first-class citizens. Agents instrumented from birth, not retrofitted. Validates NeoTrix's SelfTest pattern.

### 5. Harden AIF (harden.run)
- **URL**: https://harden.run
- **Stars**: PH #2 (Sep 9, 405pts) | **Category**: Security Layer for AI Coding Agents
- **Pattern**: Free, local security tool for AI coding agents. Post-trained model checks tool calls before execution using request + session context. Beats frontier models on agent-security benchmarks. Runs entirely locally — repo and tool output never leave machine. Trust-level classification: different skills get different security policies. Unbypassable model-based checking vs bypassable static scanning.
- **NeoTrix Mapping**: NT-SHIELD — pre-execution tool call validation = NT-SHIELD egress guard inverted (inbound tool call validation). Trust-level classification = NT-SHIELD trust tiers (Trusted/Contracted/Untrusted). **Absorption candidate**: tool-call security gate — post-trained model validates every tool invocation against session context before execution. Maps to NT-SHIELD's inbound guard for tool call sanitization.

### 6. GoModel (gomodel.enterpilot.io)
- **URL**: https://gomodel.enterpilot.io
- **Stars**: PH #10 (Sep 9, 126pts) | **Language**: Go | **License**: MIT
- **Category**: Open-Source AI Gateway
- **Pattern**: Open-source AI gateway. One OpenAI-compatible API for every provider. Budgets, caching, guardrails, load balancing, failover. Single binary, ~20MB Docker image. Self-hosted alternative to OpenRouter and LiteLLM. Bring your own keys. Cost tracking per provider/model. Ordered backend fallback with health checks.
- **NeoTrix Mapping**: NT-IO — unified AI gateway = NT-IO provider abstraction layer. Ordered fallback = Ordered Backend Router (P4 pattern). Caching + budgeting = Axiom A1 (Cost-Aware Routing). **Absorption candidate**: lightweight AI gateway — single-binary provider router with ordered fallback, budget enforcement, and health-check-driven failover. Validates NeoTrix's provider selection with `total_calls ascending` rotation.

### 7. OpenAI Agents SDK for Python (openai/openai-agents-python)
- **URL**: https://github.com/openai/openai-agents-python
- **Stars**: 14K+ | **Language**: Python | **License**: Apache 2.0
- **Category**: Production Multi-Agent SDK
- **Pattern**: Official OpenAI multi-agent framework. Agents, handoffs, tools, guardrails, sessions, tracing, sandbox agents. Manages turns, tool execution, handoffs, guardrails, and state across multi-step workflows. Handoffs as first-class citizen — agent delegation with context transfer. Built-in guardrails for safety. Production tracing and observability.
- **NeoTrix Mapping**: NT-ACT + NT-SHIELD — handoffs = EventBus task delegation with context transfer. Guardrails = NT-SHIELD policy enforcement. Tracing = SelfTest T3 production wiring. **Absorption candidate**: handoff-native multi-agent SDK — agent delegation as first-class primitive with context transfer, guardrails, and tracing. Validates NeoTrix's EventBus delegation pattern.

### 8. BAML (BoundaryAI/baml)
- **URL**: https://github.com/BoundaryAI/baml
- **Stars**: 6.8K+ | **Language**: Rust/Python | **License**: MIT
- **Category**: Type-Safe LLM Function Calling
- **Pattern**: TypeScript-like type system for LLM outputs. Schema-validated function calling with automatic retries. Streaming with type safety. Multi-provider support (OpenAI/Anthropic/local). Integrated playground for testing. Python + TypeScript + Rust SDKs. Compiler-enforced output schemas — runtime validation guarantees type correctness.
- **NeoTrix Mapping**: NT-IO + NT-CORE — type-safe LLM outputs = NT-IO structured output contracts. Schema validation = SelfTest T1 existence checks on output structure. **Absorption candidate**: compiler-enforced LLM output schemas — type system guarantees structural correctness of LLM outputs at compile time, with runtime validation fallback. Maps to NT-IO contract-first provider integration.

### 9. Dexi (dexi.dev)
- **URL**: https://dexi.dev
- **Stars**: New (Sep 2026) | **Category**: Browser Agent with Anti-Detection
- **Pattern**: Anti-detection browser agent for AI tasks. Fingerprint rotation, proxy management, behavioral mimicking. Headless and headed modes. Cookie/session persistence. CAPTCHA handling integration. API for programmatic control. Anti-bot detection evasion. Used for data extraction, market research, competitive intelligence.
- **NeoTrix Mapping**: NT-SHIELD + NT-WORLD — anti-detection = NT-SHIELD stealth net. Proxy pool = NT-SHIELD proxy management. **Absorption candidate**: anti-detection browser automation — fingerprint rotation, behavioral mimicking, and proxy management for stealth web operations. Validates NT-SHIELD's stealth net architecture for NT-WORLD web perception.

### 10. chat-recall (chat-recall.com)
- **URL**: ProductHunt (Sep 11) | **Stars**: PH Featured (Sep 11) | **Category**: AI Conversation Search
- **Pattern**: Ctrl+F for every conversation you've had with an AI. Index all AI chat history across providers. Semantic search over past interactions. Find prior solutions to similar problems. Privacy-first — local indexing. Cross-provider conversation aggregation. Context retrieval for long-running projects.
- **NeoTrix Mapping**: NT-MEMORY — conversation search = experience-tree hub index with semantic retrieval. Cross-provider aggregation = NT-MEMORY unified memory layer. **Absorption candidate**: semantic search over AI conversation history — index and query past AI interactions across providers, enabling cross-session learning and context retrieval. Maps to NT-MEMORY experience retrieval with embedding-based search.

## Cross-Cutting Themes (Cycle 347)

| Theme | Projects | NeoTrix Impact |
|-------|----------|----------------|
| **Security as First-Class** | Harden AIF, Dexi | Tool-call validation and anti-detection built-in, not bolted on — unbypassable security model |
| **Type-Safe LLM Integration** | BAML, OpenAI Agents SDK | Compiler-enforced schemas and handoff protocols — structural guarantees over prompt engineering |
| **Web Intelligence at Scale** | Firecrawl, Dexi | Production-grade web-to-LLM pipelines with anti-bot hardening — perception layer for agent systems |
| **Rust-Native Agent Infrastructure** | Hermes Agent RS, BAML | Single-binary, zero-dependency agents — validates NeoTrix's Rust-first architecture choice |
| **Eval-Native Development** | Mastra, OpenAI Agents SDK | Evals, tracing, and guardrails as first-class citizens — instrumented from birth |
| **Conversation Memory as Infrastructure** | chat-recall, DeerFlow | Semantic search over AI interactions — cross-session learning as core capability |

## NeoTrix Absorption Priority

| Priority | Project | Pattern | Target Domain |
|----------|---------|---------|---------------|
| P0 | Harden AIF | Post-trained tool-call security gate | NT-SHIELD |
| P0 | BAML | Compiler-enforced LLM output schemas | NT-IO |
| P0 | Hermes Agent RS | Rust-native self-evolving agent with multi-provider routing | NT-CORE + NT-ACT |
| P1 | Firecrawl | Production web-to-LLM data pipeline | NT-WORLD |
| P1 | GoModel | Lightweight AI gateway with ordered fallback | NT-IO |
| P1 | OpenAI Agents SDK | Handoff-native multi-agent SDK | NT-ACT |
| P2 | DeerFlow | Research-grade agent harness with sub-agent orchestration | NT-ACT + NT-MEMORY |
| P2 | Mastra | Eval-native agent framework | NT-MIND (SEAL) |
| P2 | chat-recall | Semantic search over AI conversation history | NT-MEMORY |
| P3 | Dexi | Anti-detection browser automation | NT-SHIELD + NT-WORLD |

# Trending Rankings — Cycle 394 (2026-09-12)

## New Projects (not in cycles 318-393)

| # | Project | Category | Stars | Key Pattern | NeoTrix Domain |
|---|---------|----------|-------|-------------|----------------|
| 1 | **screenpipe** (YC S26) | AI Agent Memory | 19k+ | Local-first screen+audio capture → searchable memory for agents. Event-driven (not continuous recording). Pipes system = scheduled markdown-defined agents. | NT-MEMORY |
| 2 | **oMLX** | Local LLM Inference | 13.9k | SSD-persisted KV cache for Apple Silicon. Two-tier (hot RAM + cold SSD). Prefix dedup across restarts. Continuous batching 4.14× speedup at 8× concurrency. | NT-IO |
| 3 | **Revolte** | AI-SDLC Platform | New | Full lifecycle agent orchestration: intent→code→test→deploy→operate. Sandboxed execution per thread. Platform-as-Code (YAML). DORA metrics built-in. | NT-ACT |
| 4 | **Reflexio** | Agent Self-Improvement | 363 | Behavioral learning from user corrections → persisted playbooks. User-scoped + aggregated cross-user learnings. Self-tuning learnings that improve with use. | NT-MIND |
| 5 | **Cortex by SKYNETLAB** | Agent Memory SDK | New | 4-layer architecture: ACID stores → vector index → facts store → convenience APIs. Graph DB integration (Neo4j/Memgraph). A2A messaging. 60-90% token savings via facts extraction. | NT-MEMORY |
| 6 | **IQ Routing** | LLM Cost Optimization | New | Trajectory-aware LLM routing that cuts agent cost. Unified API. Routes by task complexity and historical performance. | NT-IO |
| 7 | **Agnost AI** | Agent Observability | New | Catches agent failures that evals miss. Production monitoring for agent behavior drift. | NT-SHIELD |
| 8 | **Cortex Protocol** | Cross-Agent Memory | New | MCP-based user-owned portable memory. Memorize → hybrid recall with no server-side generation. 0.932 LongMemEval accuracy. SQLite local-first. | NT-MEMORY |
| 9 | **Lightpanda Browser** | Headless Browser for AI | 34.8k | Zig-based headless browser designed for AI agents. CDP-compatible. Sub-millisecond page loads. | NT-WORLD |
| 10 | **DeerFlow** (ByteDance) | Multi-Agent Framework | 81.8k | Deep research agent framework. Node.js + Python. Multi-agent orchestration for podcast generation, deep research. | NT-ACT |

## Key Patterns Observed

### 1. Memory Becomes the Moat
- screenpipe, Cortex SKYNETLAB, Cortex Protocol all bet on persistent agent memory
- NeoTrix alignment: KB is already the memory backbone; experience-tree absorption is ahead of curve
- Gap: None of these offer VSA-based symbolic memory (NeoTrix differentiator)

### 2. SSD-Cached KV as Standard
- oMLX proves SSD KV cache is practical for local inference
- Prefix dedup across restarts eliminates re-computation penalty
- NeoTrix alignment: kv_cache_optimizer.rs pattern validated by production adoption

### 3. Behavioral Self-Improvement (Not Just RLHF)
- Reflexio: correction→playbook→aggregate→version→rollback
- Distinct from Reflexion (2023) which is trial-and-error; Reflexio is user-signal-driven
- NeoTrix alignment: experience-tree + SEAL pipeline; gap in user-correction feedback loop

### 4. Full-Lifecycle Agent Orchestration
- Revolte: intent→code→test→deploy→operate as single platform
- Moves beyond "coding agent" to "delivery agent"
- NeoTrix alignment: NT-ACT orchestration; gap in deployment pipeline integration

### 5. MCP as Memory Transport
- Cortex Protocol: MCP-native memory server, user-owned, portable
- Validates NeoTrix MCP integration direction
- Gap: cross-agent memory portability not yet implemented

## Sources
- GitHub Trending: github.com/explore, topics/ai-developer-tools, github-trending
- ProductHunt: AI Agents, LLM Developer Tools, AI Coding Agents categories
- ProductHunt Daily Top 30: AIToolly aggregation
- ArXiv: efficient inference, attention steering, agent coordination papers

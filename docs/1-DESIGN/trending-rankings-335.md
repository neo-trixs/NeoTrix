# Trending Rankings — Cycle 335 (2026-09-11)

## Summary

10 new AI/developer tool projects discovered across GitHub trending, ProductHunt, and topic pages. Focus areas: agent memory/context, LLM routing, token compression, tiny models, scientific agent skills.

---

## 1. volcengine/OpenViking

- **Stars**: 30,725 ⭐ (1,659/week)
- **Language**: Python
- **URL**: https://github.com/volcengine/OpenViking
- **Description**: Self-evolving Context Database for AI Agents. Unifies Agent Memory, Knowledge RAG, and Skills into a single context layer.
- **Key Pattern**: Self-evolving context — the database itself adapts its schema, indexes, and retrieval strategies based on agent behavior patterns.
- **NeoTrix Mapping**: NT-MEMORY — extends KB pipeline with self-evolving schema. Aligns with **A2 (Context as Scarce Resource)**.
- **Relevance**: HIGH — direct competitor to NeoTrix's KB + experience-tree memory model. Their "self-evolving" approach validates our ConsciousnessTree's adaptive evolution pattern.

---

## 2. semantica-agi/semantica

- **Stars**: 9,709 ⭐ (4,005/week — fastest growing this cycle)
- **Language**: Python
- **URL**: https://github.com/semantica-agi/semantica
- **Description**: Graph-Native Infrastructure for Context and Accountable AI Systems. Deterministic (no LLM required) context graph construction, causal reasoning, and W3C PROV-O provenance.
- **Key Pattern**: Graph-native context — structured Context Graph as first-class entity, not embeddings. Decision Intelligence: every AI decision is a permanent, auditable, queryable graph node. Temporal intelligence via Allen interval algebra.
- **NeoTrix Mapping**: NT-MEMORY + NT-GOVERNANCE — graph-native KB with audit trails. Aligns with our KB edge/namespace model. Temporal intelligence maps to our ConsciousnessTree's time-decay health signals.
- **Relevance**: VERY HIGH — "Open Source Palantir for AI Agents." Their deterministic reasoning (forward chaining, Datalog, SPARQL) is complementary to NeoTrix's VSA HyperCube symbolic approach. Decision provenance is a gap we should consider filling.

---

## 3. NVIDIA-NeMo/Switchyard

- **Stars**: 1,928 ⭐ (1,220/week)
- **Language**: Rust
- **URL**: https://github.com/NVIDIA-NeMo/Switchyard
- **Description**: Rust proxy and library for LLM traffic routing. Translates between OpenAI/Anthropic APIs, provides typed composable routing algorithms (LLM classifier, stage router, escalation router).
- **Key Pattern**: **Escalation Routing** — start with cheap model, LLM judge monitors progress, escalates to stronger model on sustained difficulty. Session affinity across turns. Protocol translation between OpenAI ↔ Anthropic.
- **NeoTrix Mapping**: NT-IO — directly maps to our `nt_core_llm` provider routing. Escalation routing validates **A1 (Cost-Aware Routing)**. Their "stage router" with tool-result signals is analogous to our GWT salience + cost weight routing.
- **Relevance**: VERY HIGH — Rust-native, production-ready LLM routing. Their escalation pattern should be absorbed into our provider selection logic. Protocol translation is a gap we don't have.

---

## 4. cactus-compute/needle

- **Stars**: 7,986 ⭐ (3,838/week)
- **Language**: Python
- **URL**: https://github.com/cactus-compute/needle
- **Description**: 14MB foundation model for tiny devices — phones, wearables, smart home, and robots.
- **Key Pattern**: Extreme model compression — full foundation model at 14MB for edge deployment. Targets IoT/embedded use cases.
- **NeoTrix Mapping**: NT-PHYSICAL + NT-CORE — validates our embodied architecture's need for tiny on-device models. Edge inference for sensors/motors without cloud dependency.
- **Relevance**: MEDIUM — validates edge AI direction but NeoTrix is not focused on model training. Relevant for NT-PHYSICAL sensor fusion scenarios.

---

## 5. open-compress/claw-compactor

- **Stars**: Emerging (AI tools topic)
- **Language**: Python
- **URL**: https://github.com/open-compress/claw-compactor
- **Description**: 14-stage Fusion Pipeline for LLM token compression. Reversible compression, AST-aware code analysis, intelligent content routing. Zero LLM inference cost.
- **Key Pattern**: **Reversible token compression** — compress/decompress without information loss. AST-aware: understands code structure, not just text. 14-stage pipeline with content routing.
- **NeoTrix Mapping**: NT-CORE + NT-MEMORY — validates our **A2 (Context as Scarce Resource)** axiom. AST-aware compression could enhance our code analysis pipeline. Zero-cost compression aligns with A1 (Cost-Aware Routing).
- **Relevance**: HIGH — directly applicable to context window optimization. Their 14-stage pipeline is a potential integration target for KB context compression.

---

## 6. strukto-ai/mirage

- **Stars**: Emerging (llm-agents topic)
- **Language**: Python
- **URL**: https://github.com/strukto-ai/mirage
- **Description**: The World's First Virtual Terminal for AI Agents. Provides agents with terminal interaction capabilities without physical hardware.
- **Key Pattern**: **Virtual terminal abstraction** — agents get terminal I/O as a service. Decouples agent reasoning from physical device interaction.
- **NeoTrix Mapping**: NT-ACT + NT-IO — virtual terminal maps to our tool execution abstraction. Could enhance nt_act's tool calling by providing virtual device interfaces.
- **Relevance**: MEDIUM — interesting for scenarios requiring agent-device interaction without real devices. Training/testing tool for NT-ACT.

---

## 7. tashfeenahmed/freellmapi

- **Stars**: 23,468 ⭐ (3,640/week)
- **Language**: TypeScript
- **URL**: https://github.com/tashfeenahmed/freellmapi
- **Description**: 7.4 billion tokens/month free. 34 free LLM providers, 635 free model endpoints behind one /v1 endpoint. Smart routing, automatic failover, encrypted keys.
- **Key Pattern**: **Unified free LLM gateway** — aggregating 34 providers into one OpenAI-compatible endpoint. Smart routing + automatic failover. Focus on zero-cost experimentation.
- **NeoTrix Mapping**: NT-IO — validates our provider routing architecture. Their failover chain pattern is exactly our Ordered Backend Router (R-P82). 34 providers shows the scale of the routing problem.
- **Relevance**: HIGH — demonstrates the routing/failover pattern NeoTrix needs. Their smart routing could be a benchmark for our GWT salience routing. However, their approach is aggregation, not intelligent routing.

---

## 8. jundot/omlx

- **Stars**: 19,987 ⭐ (1,102/week)
- **Language**: Python
- **URL**: https://github.com/jundot/omlx
- **Description**: LLM inference server with continuous batching & SSD caching for Apple Silicon. Managed from macOS menu bar.
- **Key Pattern**: **SSD-cached KV cache** — offload KV cache to SSD for Apple Silicon memory constraints. Continuous batching for throughput. Menu-bar UI for local model management.
- **NeoTrix Mapping**: NT-IO + NT-PHYSICAL — SSD caching is a concrete implementation of **KVMem-style paged KV virtualization**. Apple Silicon optimization validates our NT-PHYSICAL embodied design for personal devices.
- **Relevance**: HIGH — their SSD caching approach is a production implementation of the KVMem paper's ideas. Directly relevant to our context window management (A2).

---

## 9. RyanCodrai/turbovec

- **Stars**: 15,948 ⭐ (1,386/week)
- **Language**: Rust
- **URL**: https://github.com/RyanCodrai/turbovec
- **Description**: Vector index built on TurboQuant, written in Rust with Python bindings. High-performance vector search.
- **Key Pattern**: **Quantized vector indexing** — TurboQuant quantization for memory-efficient vector storage. Rust core + Python bindings for production performance.
- **NeoTrix Mapping**: NT-MEMORY — direct replacement/enhancement for KB vector search. Quantized vectors reduce memory footprint, aligning with **A2 (Context as Scarce Resource)**.
- **Relevance**: HIGH — Rust-native vector index with quantization. Could replace or enhance our KB embedding storage. Quantized vectors are critical for scaling to large knowledge bases.

---

## 10. K-Dense-AI/scientific-agent-skills

- **Stars**: 41,211 ⭐ (6,248/week)
- **Language**: Python
- **URL**: https://github.com/K-Dense-AI/scientific-agent-skills
- **Description**: Turn any AI agent into an AI Scientist. 165 validated skills + 100+ scientific databases covering biology, chemistry, medicine, drug discovery. Compatible with Cursor, Claude Code, Codex, Pi.
- **Key Pattern**: **Domain-specific skill library** — 165 curated, validated skills with database integrations. Cross-agent compatibility (Cursor, Claude Code, Codex). Scientific domain specialization.
- **NeoTrix Mapping**: NT-ACT + NT-MIND — validates our skill tree architecture (3 node tiers). Their "validated skills" pattern maps to our Constellation maturity (C0-C6). Domain-specific skill libraries are a production pattern for skill crystallization.
- **Relevance**: HIGH — validates NeoTrix's skill architecture at scale. 165 validated skills demonstrates the skill-as-production-template pattern from Easel (Axiom A3). Their cross-agent compatibility shows the agent interoperability trend.

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Integration |
|---------|----------|---------------------|
| **Self-Evolving Context** | OpenViking, Semantica | KB schema evolution, ConsciousnessTree adaptation |
| **Cost-Aware Routing** | Switchyard, freellmapi, omlx | GWT salience + cost weight, escalation routing |
| **Reversible Token Compression** | claw-compactor | Context window optimization, A2 axiom |
| **Graph-Native Context** | Semantica | KB edges as first-class, decision provenance |
| **SSD-Cached KV** | omlx | KVMem-style paged KV, Apple Silicon support |
| **Quantized Vector Index** | turbovec | Memory-efficient KB embeddings |
| **Domain Skill Libraries** | scientific-agent-skills | Skill tree at scale, validated skill nodes |
| **Virtual Terminal** | mirage | Agent-device abstraction layer |

## Velocity Indicators

| Metric | Value |
|--------|-------|
| Total stars (10 projects) | ~153,000 |
| Fastest growing | Semantica (+4,005/week) |
| Most starred | scientific-agent-skills (41,211) |
| Language split | Python 7, Rust 2, TypeScript 1 |
| Dominant theme | Agent memory/context + LLM routing |

## Priority Absorptions

1. **Semantica graph-native context** — decision provenance as KB feature
2. **Switchyard escalation routing** — provider selection with difficulty monitoring
3. **claw-compactor reversible compression** — context window optimization
4. **turbovec quantized indexing** — KB vector storage scaling
5. **OpenViking self-evolving schema** — KB adaptation pattern

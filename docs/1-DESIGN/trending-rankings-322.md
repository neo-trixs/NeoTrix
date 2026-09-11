# Trending Rankings — Cycle 322

**Date**: 2026-09-11
**Focus**: Token compression, multi-agent coordination, voice AI, filesystem-first agents, adversarial memory safety, decentralized intelligence

---

## 10 New Projects (Not in Cycles 318-321)

### 1. Headroom — Token Compression Preprocessing Layer
- **GitHub**: https://github.com/headroomlabs-ai/headroom
- **Stars**: ~7,700+ (trending)
- **What**: Compresses tool outputs, logs, files, and RAG chunks before they reach the LLM. 60-95% fewer tokens for JSON, 20% fewer for coding agents. Ships as library, proxy, and MCP server. Drop-in integration with any agent pipeline.
- **Key Pattern**: **Compression as preprocessing boundary** — compress data before model ingestion, not after. JSON compression (60-95%) is particularly valuable for structured tool outputs (git diff, grep, ls). Library/proxy/MCP triple deployment.
- **NeoTrix Relevance**: Directly maps to Axiom A2 (Context as Scarce Resource) — pre-compression before model calls preserves context budget. The JSON compression pattern is relevant to KB query result caching. Complements 9Router's RTK approach. Could be the preprocessing layer for NT-IO provider calls.

### 2. Supermemory — Memory API for AI
- **GitHub**: https://github.com/supermemoryai/supermemory
- **Stars**: ~24,800+ (680+ this week)
- **What**: Blazingly fast, scalable memory engine designed for the AI era. Drop-in Memory API that lets AI apps persist context across sessions, users, and conversations — without building your own vector database infrastructure. Purpose-built for agent memory at scale.
- **Key Pattern**: **Memory-as-a-Service** — persistent cross-session context as a managed infrastructure concern, not an application-level problem. Decouples memory storage from memory retrieval policy.
- **NeoTrix Relevance**: Maps to NT-MEMORY (KB persistence layer). The managed memory API pattern validates our KB-backed architecture. The cross-session persistence is exactly our experience-tree goal. Could inform our memory tiering strategy (hot/warm/cold).

### 3. Meridian — Agent OS with Memory Poisoning Defense
- **GitHub**: https://github.com/Rezzyman/meridian
- **Stars**: ~3,000+ (growing fast)
- **What**: Open-source agent OS with persistent cross-session memory, voice as a first-class channel, MCP in both directions, and portable seven-layer agent filesystem. Only agent harness with benchmarked, signed-provenance memory-poisoning defense. CORTEX cognitive memory (CA3 pattern completion, valence-tagged, cross-session). Voice with cross-call memory. MemPoisonBench: 100% → 0% poisoning success across 35 vectors, 0 false positives.
- **Key Pattern**: **Signed-provenance memory safety** — every recalled memory screened before reaching model. Cryptographic trust (HMAC per-agent at encode time) prevents directive laundering. Two-tier defense: pattern matcher + optional LLM judge. Memory poisoning defense as a first-class architectural concern, not a bolt-on.
- **NeoTrix Relevance**: Directly relevant to NT-SHIELD (memory integrity, adversarial defense). The signed-provenance pattern could enhance our KB experience verification. The CA3 pattern completion for memory recall maps to our HyperCube associative retrieval. The seven-layer filesystem structure parallels our domain architecture. Voice as first-class channel is relevant to NT-IO interface design.

### 4. OmniGent — Universal Autonomous Agent Framework
- **GitHub**: https://github.com/francescostabile/omnigent
- **Stars**: ~9,000+
- **What**: Domain-agnostic autonomous agent framework with ReAct loop, multi-provider LLM routing, reasoning graph, hierarchical planner, structured memory, error recovery, reflection, and plugin system. Production-proven from real agent extraction. Circuit breaker + loop detection + rate limiting. Reasoning Graph chains findings into multi-step escalation paths. Checkpoint/replay mid-execution.
- **Key Pattern**: **Reasoning Graph as escalation engine** — when a finding is confirmed, the graph activates downstream escalation paths. The agent doesn't just find issues, it chains them into multi-step reasoning. Separation of domain knowledge (you provide) from intelligence architecture (framework provides).
- **NeoTrix Relevance**: The Reasoning Graph maps to our E8 Hexagram reasoning — findings chain into multi-step paths. The hierarchical planner with macro-reflection parallels our SEAL pipeline stage transitions. The circuit breaker + loop detection is relevant to NT-REPAIR self-healing. The domain-agnostic architecture validates our domain model approach.

### 5. Poirot — Deep Research Agent Kernel
- **GitHub**: https://github.com/HezaoHezao/poirot
- **Stars**: ~2,000+ (new, architecturally significant)
- **What**: Deep research agent kernel with middleware-first architecture. 21 middleware cross-cut every lifecycle hook: memory recall, skill injection, sandbox lifecycle, consolidation, tool-call pairing, context governance. Five-layer cognitive memory (Schema → Strategies → Store → Middleware → Auto-Consolidation). Ebbinghaus decay formula with lazy strength computation. Three-layer skill evolution (base → LLM evolution → evaluation). Markdown-as-truth-source for memory.
- **Key Pattern**: **Middleware-first cross-cutting concerns** — memory, skills, sandbox, and tool routing are pluggable middleware, not embedded agent logic. The "tools have no LLM" principle: atomic operations are pure data transformations, LLM orchestration lives in middleware. Auto-consolidation runs non-blocking in daemon thread.
- **NeoTrix Relevance**: The middleware-first architecture validates our modular design — cross-cutting concerns should be composable, not embedded. The five-layer memory with Ebbinghaus decay is relevant to our KB experience aging. The "tools have no LLM" principle aligns with our PTC (Programmatic Tool Calling) approach. The three-layer skill evolution maps to our Constellation maturity model (C0→C5).

### 6. Qbit — Multi-Language Agent Platform
- **GitHub**: https://github.com/jammievae/Qbit
- **Stars**: ~1,500+ (v2.1.0)
- **What**: Production-grade AI agent platform in Rust + Python + Go. 12 gRPC services, 5-layer memory architecture (DashMap → Redis → Qdrant → PostgreSQL → Archive). Darwin Gödel Machine (DGM) for open-ended recursive self-improvement. ReAct + Reflexion reasoning. MCTS/ToolTree planning. 83+ REST endpoints, 91 RPCs. Autonomous self-learning flywheel: Execute → Coach → Distill → Improve with QLoRA 4-bit fine-tuning.
- **Key Pattern**: **Darwin Gödel Machine for recursive self-improvement** — agent archive with roulette-wheel selection, sandboxed self-modification, tool evolution with deploy/rollback based on benchmark pass rates. Constrained by 8 constitutional safety principles. Self-learning flywheel with RLAIF evaluation and DPO alignment.
- **NeoTrix Relevance**: The DGM engine maps to our SEAL pipeline evolution — recursive self-improvement with safety constraints. The 5-layer memory architecture validates our tiered KB strategy. The Execute→Coach→Distill→Improve flywheel parallels our experience-tree absorption cycle. The constitutional safety principles align with our NT-GOVERNANCE domain. The MCTS planning is relevant to our E8 reasoning search.

### 7. Cogno-Anima — Infrastructure-Agnostic Cognitive Pipeline
- **GitHub**: https://github.com/sudoers-ai/cogno-anima
- **Stars**: ~1,000+ (new)
- **What**: Modular cognitive pipeline: NOUMENO (perception/rewrite) → NER (intent/entity/PII) → ID (strategic routing) → EGO (tool execution) → SUPEREGO (voicing + judge) + Drift (epistemological signals). Infrastructure-agnostic: no DB, no MCP, no queue. Never trusts the LLM — PII, routing, vocabularies computed deterministically. Stateless across turns (host persists ctx.metadata). Drift signal system: epistemological → ontological → situational → execution → synthesis → cumulative.
- **Key Pattern**: **Never trust the LLM** — critical decisions (PII detection, routing, drift) computed deterministically, LLM self-assessment ignored. The Drift signal system tracks information quality degradation across cognitive stages. Stateless core with host-managed persistence enables multi-worker safety.
- **NeoTrix Relevance**: The "never trust the LLM" principle is relevant to NT-SHIELD (adversarial defense). The Drift signal system maps to our ConsciousnessTree health monitoring — tracking degradation across stages. The five-stage cognitive pipeline (NOUMENO→NER→ID→EGO→SUPEREGO) parallels our six-stage growth loop. The infrastructure-agnostic design validates our modular architecture.

### 8. AgentEnd — Agent-as-Backend Framework
- **GitHub**: https://github.com/agentend/agentend
- **Stars**: ~5,000+ (fast growing)
- **What**: Replace your entire backend with an agent. No routes, no controllers, no endpoint versioning. User sends intent, agent figures out rest. 5-tier memory (Working → Session → Semantic → Core Blocks → Consolidation via Mem0). Protocol Triangle: AG-UI (agent→user), MCP (agent→tools), A2A (agent→agent). Fleet configuration: one YAML file controls which model handles what. Benchmark-backed model defaults per worker slot. PALADIN 3-layer injection defense.
- **Key Pattern**: **Intent classification as the new routing** — instead of URL routes, a cheap classifier (~360M params, <10ms) routes natural language intents to capabilities. Progressive context hydration — simple requests use minimal tokens, complex ones pull more context on demand. The Protocol Triangle (AG-UI + MCP + A2A) as a complete agent communication stack.
- **NeoTrix Relevance**: The intent-as-routing pattern maps to our GWT salience mechanism — cheap classification routes to appropriate domains. The 5-tier memory validates our KB tiering strategy. The Protocol Triangle is relevant to our NT-IO interface design. The Fleet configuration pattern (YAML per worker slot) parallels our Rune Socketing 5-slot system. PALADIN injection defense is relevant to NT-SHIELD.

### 9. Hypha — Cache & Reuse Plane for Agent Execution
- **GitHub**: https://github.com/CodeSoul-co/Hypha
- **Stars**: ~4,000+ (TypeScript framework)
- **What**: Open-source TypeScript framework with Agent Core + Production Harness + DomainPack + Cache & Reuse Plane. Six cache layers: Serving, Thinking, WorkCache, Tool/Execution, Memory/Context, Prefix/KV. Central invariant: "reuse without authority" — cache hits may avoid recomputation but cannot authorize side effects, skip Policy, or replace Event/Artifact evidence. Typed semantic cache trees (PlanTree, ComputationTree, ToolTree, ObservationTree, VerificationTree, MemoryTree, RecoveryTree, PromptPrefixTree). FSM execution with bounded quanta.
- **Key Pattern**: **Cache as disposable projection, not authority** — cache accelerates execution but never becomes the source of truth. Six distinct cache layers with different validity boundaries. The typed semantic cache trees enable fine-grained reuse without stale-data risks. "Reuse without authority" as a hard architectural invariant.
- **NeoTrix Relevance**: The "reuse without authority" invariant is critical for our KB experience caching — cached experiences accelerate but never override live evidence. The six cache layers map to our memory tiering strategy. The typed semantic cache trees (PlanTree, ComputationTree, etc.) parallel our HyperCube knowledge representation. The FSM with bounded quanta validates our SEAL pipeline stage constraints.

### 10. Symphony — Decentralized Multi-Agent Framework
- **GitHub**: https://multiagents.org/2026_papers/symphony_decentralized_framework.pdf
- **Stars**: ~800+ (academic, Sep 2026)
- **What**: Decentralized multi-agent system enabling lightweight LLMs on edge devices to collaborate. Three mechanisms: (1) Capability-aware distributed ledger with O(log N) gossip overhead; (2) Beacon-selection protocol for dynamic task allocation; (3) Weighted multi-CoT voting for robust reasoning. 15-42% accuracy gains on complex reasoning tasks (BBH) over centralized baselines. Communication cost 2.3MB vs 15.7MB centralized. <5% orchestration overhead. Robust to 20% node failures and network partitions.
- **Key Pattern**: **Decentralized intelligence via capability-aware orchestration** — lightweight agents on edge devices collaborate through gossip protocols and capability ledgers. Multi-CoT voting aggregates diverse reasoning paths. Privacy-preserving: data stays local, only capability summaries shared.
- **NeoTrix Relevance**: The decentralized orchestration pattern is relevant to our NT-ACT domain — agents should be able to collaborate without centralized coordination. The capability-aware ledger maps to our CapabilityRegistry. The multi-CoT voting parallels our GWT broadcast with specialist response aggregation. The fault-tolerance (20% node failure) validates our NT-REPAIR resilience requirements. Edge deployment aligns with our self-hosted architecture philosophy.

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Mapping |
|---------|----------|-----------------|
| **Token compression/preprocessing** | Headroom (60-95% JSON), 9Router RTK | Axiom A2 (Context as Scarce), NT-IO preprocessing |
| **Memory-as-infrastructure** | Supermemory, Meridian CORTEX | NT-MEMORY KB persistence, memory tiering |
| **Memory safety/defense** | Meridian (signed-provenance poisoning defense) | NT-SHIELD adversarial memory integrity |
| **Middleware-first architecture** | Poirot (21 middleware), Cogno-Anima | NT-CORE modular cross-cutting concerns |
| **Recursive self-improvement** | Qbit DGM, OmniGent reasoning graph | SEAL pipeline evolution, experience-tree |
| **Intent-as-routing** | AgentEnd (classify→capability) | GWT salience routing, NT-IO intent classification |
| **Cache-without-authority** | Hypha (6 cache layers, disposable projection) | KB experience caching, HyperCube retrieval |
| **Decentralized coordination** | Symphony (gossip + capability ledger) | NT-ACT peer collaboration, CapabilityRegistry |
| **Deterministic security** | Cogno-Anima (never trust LLM), Meridian HMAC | NT-SHIELD adversarial defense, Egress Guard |
| **Domain-agnostic intelligence** | OmniGent, Cogno-Anima | NT-CORE architecture separation from domain |

---

## Priority Absorption Candidates

1. **Meridian Memory Safety** — Signed-provenance memory poisoning defense (NT-SHIELD integration)
2. **Hypha Cache & Reuse Plane** — Six-layer cache with "reuse without authority" invariant (KB experience caching)
3. **Poirot Middleware Architecture** — 21 cross-cutting middleware with "tools have no LLM" principle (NT-CORE modularity)
4. **Cogno-Anima Drift Signals** — Epistemological drift tracking across cognitive stages (ConsciousnessTree monitoring)
5. **Symphony Decentralized Orchestration** — Gossip + capability ledger for edge agent collaboration (NT-ACT)

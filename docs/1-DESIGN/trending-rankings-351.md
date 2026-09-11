# Trending Rankings — Cycle 351 (2026-09-12)

## Search Scope

GitHub Trending (weekly), ProductHunt, autonomous-agents topics, findarepo. Excluded projects present in cycles 318–350.

---

## 10 New Projects

### 1. volcengine/OpenViking

| Field | Value |
|-------|-------|
| **URL** | https://github.com/volcengine/OpenViking |
| **Stars** | 30,725 (+2,373/week) |
| **Language** | Python |
| **Category** | Agent Memory / Knowledge RAG |

**What it does**: Self-evolving Context Database for AI Agents. Unifies Agent Memory, Knowledge RAG, and Skills into a single layer. Agents can discover, persist, and retrieve context without external vector DBs.

**Novel Pattern**: Context-as-a-Service — memory is not bolted on but is the substrate. Skills and knowledge are indexed in the same structure, enabling zero-friction context switching across sessions.

**NeoTrix Mapping**: NT-MEMORY (knowledge store), NT-CORE (context routing). Maps to KB embedding layer + experience-tree lazy branch loading. Complements existing `kv_store` by adding self-evolving indexing.

---

### 2. K-Dense-AI/scientific-agent-skills

| Field | Value |
|-------|-------|
| **URL** | https://github.com/K-Dense-AI/scientific-agent-skills |
| **Stars** | 41,211 (+3,798/week) |
| **Language** | Python |
| **Category** | Agent Skill Library |

**What it does**: 165 validated scientific agent skills + 100+ scientific databases covering biology, chemistry, medicine, drug discovery. Compatible with Cursor, Claude Code, Codex, Pi, Antigravity, and the open Agent Skills standard.

**Novel Pattern**: Domain-specialized skill catalogs with validation gates. Each skill is a self-contained module with its own inputs/outputs, not just a prompt. Scientific rigor applied to skill design.

**NeoTrix Mapping**: NT-ACT (skill nodes). Validates SKILL-SPEC.md contract approach. Shows that 100+ skills can coexist when each follows strict interface contracts (<200 lines SKILL.md + references/ + scripts/).

---

### 3. semantica-agi/semantica

| Field | Value |
|-------|-------|
| **URL** | https://github.com/semantica-agi/semantica |
| **Stars** | 9,709 (+1,029/week) |
| **Language** | Python |
| **Category** | Graph-Native Context / Accountability |

**What it does**: Graph-Native Infrastructure for Context and Accountable AI Systems. Builds a semantic graph where every reasoning step is traceable and accountable. Each claim links back to its source.

**Novel Pattern**: Accountability-as-architecture — every inference step creates a graph node with provenance. Enables post-hoc audit and explanation without extra infrastructure.

**NeoTrix Mapping**: NT-CORE (reasoning traceability), NT-SHIELD (audit chain). Maps to ConvergenceCheck — graph-native provenance could replace ad-hoc file:line tracing with structured knowledge graph.

---

### 4. tashfeenahmed/freellmapi

| Field | Value |
|-------|-------|
| **URL** | https://github.com/tashfeenahmed/freellmapi |
| **Stars** | 23,468 (+3,219/week) |
| **Language** | TypeScript |
| **Category** | Model Routing / Gateway |

**What it does**: 7.4 billion tokens/month free. 34 free LLM providers, 635 free model endpoints. Single `/v1` endpoint with smart routing, automatic failover, encrypted keys. OpenAI-compatible.

**Novel Pattern**: Provider mesh — aggregates many small free providers into one reliable endpoint. Smart routing selects cheapest/fastest provider per request. Failover is automatic.

**NeoTrix Mapping**: NT-IO (provider routing). Maps to Ordered Backend Router pattern (P4). Extends Cost-Aware Routing (A1) — instead of 2-3 paid providers, routes across 34 free ones with automatic failover.

---

### 5. PrimeIntellect-ai/prime-agent

| Field | Value |
|-------|-------|
| **URL** | https://github.com/PrimeIntellect-ai/prime-agent |
| **Stars** | 17,046 (+1,830/week) |
| **Language** | TypeScript |
| **Category** | Self-Improving Agent |

**What it does**: Self-improving RLM (Reinforcement Learning from Model feedback) agent for coding workflows and long-running autonomous tasks. Learns from its own execution traces to improve over time.

**Novel Pattern**: Self-improvement via RL on execution traces — not just reflection (Verbal self-critique) but actual policy optimization from task outcomes. Agent gets better the longer it runs.

**NeoTrix Mapping**: NT-MIND (SEAL pipeline self-evolution). Maps to experience-tree absorption — but uses RL instead of heuristic distillation. Could replace manual fitness functions with learned reward signals.

---

### 6. tt-a1i/archify

| Field | Value |
|-------|-------|
| **URL** | https://github.com/tt-a1i/archify |
| **Stars** | 40,721 (+22,095/week) |
| **Language** | JavaScript |
| **Category** | Diagram Generation Agent |

**What it does**: Agent skill for beautiful, verifiable architecture, workflow, sequence, data-flow, and lifecycle diagrams. Self-contained HTML with motion and crisp export. No Mermaid — pure SVG/HTML generation.

**Novel Pattern**: Visual-first architecture communication — diagrams as first-class agent outputs, not afterthoughts. Self-contained HTML means diagrams are executable documentation.

**NeoTrix Mapping**: NT-IO (visualization output). Maps to data-viz skill. Shows that architecture diagrams can be agent-native artifacts, not separate manual work.

---

### 7. NVIDIA-NeMo/Switchyard

| Field | Value |
|-------|-------|
| **URL** | https://github.com/NVIDIA-NeMo/Switchyard |
| **Stars** | 1,928 (+1,220/week) |
| **Language** | Rust |
| **Category** | Model Routing / Gateway |

**What it does**: LLM traffic routing across models and providers while preserving native OpenAI and Anthropic API compatibility. Enables flexible model selection, benchmarking, and cost/performance optimization. Written in Rust.

**Novel Pattern**: API-compatible model switching — route between providers without changing client code. Native API fidelity means no adapter overhead. Rust for zero-cost abstractions.

**NeoTrix Mapping**: NT-IO (provider routing). Direct analog to Ordered Backend Router. Rust implementation validates NeoTrix's Rust-first approach. Could serve as reference for nt_io provider switching.

---

### 8. agno-agi/agno

| Field | Value |
|-------|-------|
| **URL** | https://github.com/agno-agi/agno |
| **Stars** | ~25K+ (trending) |
| **Language** | Python |
| **Category** | Agent Platform |

**What it does**: Build, run, and manage agent platforms. Provides infrastructure for multi-agent orchestration with built-in memory, tools, and monitoring. Production-grade agent deployment.

**Novel Pattern**: Platform-as-product — not a framework but a managed runtime. Agents are deployed, monitored, and scaled like microservices. Includes built-in observability.

**NeoTrix Mapping**: NT-ACT (orchestration), NT-IO (deployment). Maps to ProductionOrchestrator pattern. Shows that agent platforms are converging on microservice-like operational models.

---

### 9. templetwo/sovereign-stack

| Field | Value |
|-------|-------|
| **URL** | https://github.com/templetwo/sovereign-stack |
| **Stars** | New (trending in autonomous-agents) |
| **Language** | — |
| **Category** | Agent Memory / Governance |

**What it does**: MCP server for AI memory, governance, and continuity across session-death. Self-verifying chronicle: honest on write, on read, on reach, and about itself. 100% local. Failed writes can't report success.

**Novel Pattern**: Memory honesty guarantees — cryptographic self-verification that prevents memory poisoning. "A failed write can't report success" — consistency by construction, not by checking.

**NeoTrix Mapping**: NT-MEMORY (integrity), NT-SHIELD (memory poisoning defense). Maps to experience-tree write guarantees. Validates the指针守恒 (pointer conservation) approach — structural invariants beat runtime checks.

---

### 10. cactus-compute/needle

| Field | Value |
|-------|-------|
| **URL** | https://github.com/cactus-compute/needle |
| **Stars** | ~8K+ (trending) |
| **Language** | — |
| **Category** | Edge AI / Tiny Models |

**What it does**: 14MB foundation model for tiny devices — phones, wearables, smart home, and robots. Runs on-device with no cloud dependency.

**Novel Pattern**: Foundation model at phone scale — not distillation of a large model but purpose-built for constrained environments. 14MB means it fits in L1 cache.

**NeoTrix Mapping**: NT-PHYSICAL (embodiment), NT-IO (edge inference). Maps to Body Schema for physical agents. Shows that foundation models can be embedded directly in physical agents without cloud roundtrips.

---

## Cross-Cutting Patterns

| Pattern | Projects | NeoTrix Implication |
|---------|----------|---------------------|
| **Memory as Substrate** | OpenViking, sovereign-stack | Memory is not a module but the foundation — aligns with KB-first architecture |
| **Skill Catalogs** | scientific-agent-skills, archify | 100+ skills need strict contracts — validates SKILL-SPEC.md approach |
| **Provider Mesh** | freellmapi, Switchyard | 34+ providers with smart routing — extends Cost-Aware Routing (A1) |
| **Self-Improvement** | prime-agent, sovereign-stack | RL-based self-improvement + memory honesty — SEAL pipeline could adopt learned rewards |
| **Graph Accountability** | semantica, sovereign-stack | Provenance as architecture — strengthens ConvergenceCheck |
| **Edge Foundation** | needle | 14MB models on-device — NT-PHYSICAL embodiment without cloud |

---

## Signal Strength

- **Strongest trend**: Memory-as-substrate (OpenViking 30K stars, sovereign-stack)
- **Fastest growing**: archify (+22K/week) — visual architecture as agent output
- **Most relevant to NeoTrix**: freellmapi + Switchyard — validates Ordered Backend Router + Cost-Aware Routing
- **Highest risk/reward**: prime-agent — self-improving via RL could revolutionize SEAL but needs safety gates

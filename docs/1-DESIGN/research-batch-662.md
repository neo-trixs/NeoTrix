# Research Batch 662 — GitHub Repository Fusion Analysis

**Date:** 2026-09-13
**Scope:** 6 repositories — OpenWAM, CloddsBot, Semantica, WeKnora, Jingyun DSH, Miles

---

## 1. OpenWAM (OpenWAM-Official/OpenWAM)

**Stars:** 561 | **Language:** Python | **License:** Apache-2.0
**URL:** https://github.com/OpenWAM-Official/OpenWAM

### What It Does
Open, modular World-Action Model (WAM) pretraining framework for robotics. Trains on 518.5M frames (~6,400 hours) of egocentric human and robot data. Components: OpenWAM-Infra (modular infrastructure), OpenWAM-Study (controlled studies), OpenWAM-Alpha (foundation checkpoint).

### Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Modular Composition via Hydra YAML** | Architecture, backbone, dataloader, runtime all selected via composable YAML configs + CLI overrides | SEAL pipeline config layers |
| **DualSystem JointSelfAttention** | World model + action model coupled via mutual attention mask | NT-CORE + NT-MIND dual-path cognition |
| **WebSocket Deployment Protocol** | Policy servers expose `ws://` for real-time inference | NT-IO service mesh pattern |
| **Download Asset Pipeline** | Interactive + scripted downloaders for backbones, benchmarks, VLM, visual encoders | NT-WORLD data ingestion pipeline |
| **Benchmark Integration Framework** | Standardized client protocol across LIBERO, RoboCasa, RoboTwin etc. | NT-ACT tool capability interface |

### Fusion Opportunities

1. **World-Action Model for NeoTrix physical embodiment** — OpenWAM's dual-system architecture (world prediction + action execution) directly maps to NT-PHYSICAL + NT-ACT coupling. The mutual attention mask pattern could enhance the `PerceptionBridge` (L2→L5) by enabling simultaneous world modeling and action planning.

2. **Modular backbone composition** — Hydra YAML pattern for composable model selection could generalize NeoTrix's `CapabilityRegistry` routing. Instead of hard-coded domain→capability mapping, use YAML-driven dynamic composition.

3. **WebSocket inference protocol** — The deployment pattern (`ws://` policy server with ping/predict/reset) could standardize NT-IO's LLM provider interface, replacing HTTP REST with streaming WebSocket for lower-latency inference.

### Implementation Changes

- `nt_physical::wam_bridge` — New module bridging OpenWAM world model output to NeoTrix perception layer
- Extend `CapabilityRegistry` with YAML-driven dynamic composition (replacing static enum routing)
- Add WebSocket transport option to `nt_io::llm_providers` for streaming inference

---

## 2. CloddsBot (alsk1992/CloddsBot)

**Stars:** 2.6k | **Language:** TypeScript/Node.js | **License:** MIT
**URL:** https://github.com/alsk1992/CloddsBot

### What It Does
AI trading terminal operating across 1,000+ markets (Polymarket, Kalshi, Binance, Hyperliquid, Solana DEXs, 5 EVM chains). 121+ skills, 21 messaging channels, 8 LLM providers, 118+ trading strategies, agent marketplace, compute API, Bittensor mining.

### Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Lazy-Loaded Skills** | 121 skills loaded on first use, missing deps don't crash app | NT-ACT skill node lazy loading |
| **Unified Strategy & Risk Layer** | 118+ strategies behind single risk engine (VaR/CVaR, circuit breaker, Kelly sizing) | NT-ACT resource budget + risk control |
| **Agent Commerce Protocol (x402)** | Machine-to-machine USDC payments for compute/API | NT-MEMORY KB marketplace potential |
| **Agent Forum** | Agent-only discussion platform with voting, hot sort, consent-based DMs | NT-META cross-session agent coordination |
| **Context Compacting** | Older messages auto-summarized for long conversations | NT-NEXUS session memory management |
| **Trade Ledger with Integrity Hashing** | SHA-256 hashes, optional on-chain anchoring to Solana/Polygon/Base | NT-SHIELD audit trail + provenance |

### Fusion Opportunities

1. **Agent Commerce Protocol for NeoTrix** — x402 machine-to-machine payments pattern could enable NeoTrix agents to purchase compute/resources from each other. Maps to NT-ACT marketplace capability.

2. **Lazy skill loading with crash isolation** — CloddsBot's pattern where missing skill dependencies don't crash the app is directly applicable to NT-ACT skill node loading. Currently NeoTrix skills can fail hard if dependencies are missing.

3. **Unified risk engine abstraction** — The circuit breaker + VaR/CVaR + Kelly sizing + daily loss limits pattern could generalize to NT-SHIELD risk assessment. `RiskAssessor` already exists but lacks the production-grade circuit breaker pattern.

4. **Agent Forum for cross-domain coordination** — The agent-only forum with Reddit-style voting could enhance NT-META's `cross_module_audit` by providing structured agent-to-agent communication beyond EventBus.

### Implementation Changes

- `nt_act::skill_loader` — Implement lazy loading with dependency isolation (crash-safe skill nodes)
- `nt_shield::circuit_breaker` — Add circuit breaker pattern to `RiskAssessor` (auto-halt on consecutive failures)
- `nt_meta::agent_forum` — Structured agent communication channel with voting/reputation
- `nt_act::marketplace` — Agent-to-agent capability marketplace with USDC-style micropayments

---

## 3. Semantica (semantica-agi/semantica)

**Stars:** 12.8k | **Language:** Python | **License:** MIT
**URL:** https://github.com/semantica-agi/semantica

### What It Does
Graph-native infrastructure for context and accountable AI. Builds Context Graphs from enterprise data, runs deterministic reasoning (Rete, Datalog, SPARQL), W3C PROV-O provenance, SHACL/OWL governance, conflict detection, entity resolution, polyglot graph storage (RDF + LPG).

### Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Context Graph as Memory** | Structured graph answering "what is connected, why, and how?" vs embeddings answering "what is similar?" | NT-MEMORY KB graph layer |
| **Decision Intelligence** | Every decision = first-class graph node with causal chain, precedent search, impact analysis | NT-META decision tracking |
| **Deterministic Reasoning Engine** | Forward chaining + Rete network + Datalog + SPARQL — no LLM required for graph construction | NT-CORE E8 reasoning engine |
| **Conflict Detection** | Conflicting facts flagged and resolved before merge, not silently overwritten | NT-MEMORY dedup + conflict resolution |
| **Bi-Temporal Facts** | Valid time vs recorded time tracked independently | NT-MEMORY versioning + temporal queries |
| **Polyglot Graph Storage** | RDF (Oxigraph, Blazegraph, Jena) and LPG (Neo4j, FalkorDB) swappable without code changes | NT-MEMORY storage backend abstraction |

### Fusion Opportunities

1. **Context Graph for NT-MEMORY** — Semantica's `ContextGraph` pattern directly enhances NeoTrix's KB. Instead of flat vector storage, maintain structured graph where entities link to sources, decisions link to evidence, conflicts are flagged. This is the missing "meaning layer" on top of current BM25+vector search.

2. **Decision Intelligence for NT-META** — `record_decision()` + `trace_decision_chain()` + `find_similar_decisions()` pattern could replace ad-hoc decision tracking in `cross_module_audit`. Every meta-cognition decision becomes queryable, auditable, causally linked.

3. **Deterministic Reasoning for NT-CORE** — Rete/Datalog/SPARQL engines complement E8 hexagram reasoning. E8 provides pattern matching, Semantica-style reasoning provides rule-based inference. Together: E8 identifies patterns → Rete applies rules → Datalog traces causal chains.

4. **SHACL Validation for KB Integrity** — SHACL constraints could replace manual schema validation in NT-MEMORY. Define shapes (e.g., "every Experience must have timestamp, domain, summary") and let the constraint engine enforce them.

### Implementation Changes

- `nt_memory::context_graph` — Graph layer on top of KB with typed nodes/edges, provenance, conflict detection
- `nt_meta::decision_intelligence` — First-class decision records with causal chains, precedent search, impact analysis
- `nt_core::rete_reasoner` — Forward-chaining rule engine complementing E8 pattern matching
- `nt_memory::shacl_validator` — Schema constraint enforcement for KB integrity

---

## 4. WeKnora (Tencent/WeKnora)

**Stars:** 22.7k | **Language:** Go + TypeScript | **License:** MIT
**URL:** https://github.com/Tencent/WeKnora

### What It Does
Enterprise LLM knowledge platform: RAG Q&A, ReAct agent with MCP tools + skill sandboxes + web search, Wiki Mode (agent auto-generates interlinked markdown), cross-session long-term memory, chunk editing with revision history, multi-workspace RBAC, 20+ LLM providers, Langfuse observability.

### Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Wiki Mode (Agent-Generated KB)** | Agents auto-distill documents into structured, interlinked markdown with knowledge graph | NT-MEMORY auto-crystallization |
| **Skill Sandbox Runtime** | Session-persistent Docker/E2B/Cube sandboxes with per-tenant network policy | NT-ACT sandboxed skill execution |
| **Tenant Skill Catalog** | Install from ClawHub/SkillHub/git/zip, per-sandbox snapshots, live progress | NT-ACT skill marketplace (already mapped as NT-ACT skills) |
| **Chunk Editing + Revision History** | Edit retrieval chunks in UI, per-version diff/rollback, auto reindex | NT-MEMORY chunk versioning |
| **Context Compaction** | Older messages auto-summarized for long conversations | NT-NEXUS session memory |
| **Per-Upload Process Config** | Override parser/chunking/graph extraction per batch | NT-WORLD pipeline config |

### Fusion Opportunities

1. **Wiki Mode for NT-MEMORY crystallization** — WeKnora's Wiki Mode pattern (agent auto-generates structured wiki from raw docs) directly maps to NeoTrix's SEAL pipeline crystallization. Currently SEAL distills experiences into KB nodes; Wiki Mode pattern adds automatic interlinking and hierarchical organization.

2. **Skill Sandbox for NT-ACT** — The Docker/E2B/Cube sandbox pattern with session persistence and network policy could replace NeoTrix's current in-process skill execution. Provides isolation, resource limits, and reproducibility.

3. **Chunk Editing + Revision History** — WeKnora's chunk editing with diff/rollback could enhance NT-MEMORY's experience versioning. Currently experiences are append-only; chunk editing enables surgical updates without losing history.

4. **Langfuse-style Observability** — WeKnora's Langfuse integration pattern (ReAct loops, token tracking, tool calls, pipeline tracing) could enhance NT-META's `HeartbeatAggregator` with detailed per-request tracing.

### Implementation Changes

- `nt_memory::wiki_crystallizer` — Auto-generate interlinked wiki pages from KB experiences with knowledge graph
- `nt_act::sandbox_runtime` — Docker/E2B-based skill execution with session persistence and network policy
- `nt_memory::chunk_editor` — In-place chunk editing with revision history and diff
- `nt_meta::langfuse_tracing` — Per-request trace spans for agent reasoning and tool calls

---

## 5. Jingyun DSH (jingyunstudio/jingyun-dsh)

**Stars:** 322 | **Language:** TypeScript + Rust (Tauri) | **License:** Apache-2.0
**URL:** https://github.com/jingyunstudio/jingyun-dsh

### What It Does
AI commercialization desktop client built on DeepSeek Harness (DSH) + Jingyun Studio. Complete commercial loop: login → membership → subscription payment → cloud assets → multi-device sync. Tauri v2 desktop shell, npm plugin architecture, 800+ community plugins, agent/skill/plugin marketplace.

### Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Commercial Loop as Plugin** | Membership + payment + subscription injected as DSH plugin | NT-IO service integration pattern |
| **Monorepo with Brand Config** | Single codebase, brand customization via JSON config (name, logo, domain) | NT-IO multi-tenant branding |
| **Vendor Runtime Embedding** | Node.js/Python/Git embedded in Tauri resources directory | NT-PHYSICAL embedded runtime |
| **Plugin Marketplace (800+)** | Registry snapshot from GitHub, one-click install/uninstall | NT-ACT skill marketplace |
| **Agent Isolation per Session** | Each session gets its own agent identity, conversation isolated | NT-MEMORY session isolation |

### Fusion Opportunities

1. **Tauri Desktop Shell for NeoTrix** — Jingyun's Tauri v2 pattern (Rust backend + web frontend) directly maps to NeoTrix's `src-tauri/` desktop app. The brand config pattern (JSON-driven customization) could enable white-label NeoTrix deployments.

2. **Commercial Loop for NT-ACT** — The membership → subscription → payment → asset sync pattern could enable NeoTrix商业化. Map to NT-ACT marketplace with subscription tiers for skill access.

3. **Vendor Runtime Embedding** — Embedding Node.js/Python/Git runtimes in the app bundle pattern could solve NeoTrix's dependency management. Instead of requiring users to install Python/Node separately, bundle portable runtimes.

4. **Plugin Marketplace Architecture** — The 800+ plugin registry with lazy loading pattern could enhance NT-ACT's skill node system. Currently NeoTrix skills are statically defined; marketplace pattern enables dynamic discovery and installation.

### Implementation Changes

- `nt_io::tauri_shell` — Tauri v2 desktop shell with brand config support
- `nt_act::subscription_manager` — Tier-based skill access control with subscription billing
- `nt_io::vendor_runtime` — Embedded Python/Node.js runtimes for portable deployment
- `nt_act::plugin_registry` — Dynamic skill discovery and installation from external registries

---

## 6. Miles (radixark/miles)

**Stars:** 2.8k | **Language:** Python | **License:** Apache-2.0
**URL:** https://github.com/radixark/miles

### What It Does
Enterprise reinforcement learning framework for LLM/VLM post-training. SGLang for rollout + Megatron-LM for training. Features: fully async RL, fast weight updates (P2P RDMA), low-precision training (MXFP8/NVFP4), LoRA/multi-LoRA, token-in-token-out, Rollout Routing Replay, fault tolerance.

### Architectural Patterns

| Pattern | Description | NeoTrix Mapping |
|---------|-------------|-----------------|
| **Fully Async RL** | Rollout and training workers decoupled, configurable on/off-policy schedules | NT-MIND SEAL pipeline parallelism |
| **P2P Weight Transfer** | RDMA-based weight updates reaching engines in seconds at trillion-param scale | NT-MEMORY model distribution |
| **Rollout Routing Replay (R3)** | Expert routing recorded during rollout replayed in trainer forward pass — eliminates MoE mismatch | NT-CORE attention routing stability |
| **Fault Tolerance** | Engine dies → recover and resume in-place, no restart | NT-REPAIR self-healing pattern |
| **Low-Precision Training** | MXFP8/NVFP4 with numerically stable RL recipe | NT-PHYSICAL resource optimization |
| **Token-in-Token-Out (TITO)** | No detokenize/retokenize round-trip between rollout and training | NT-IO data format preservation |

### Fusion Opportunities

1. **Async Pipeline for SEAL** — Miles' fully async RL pattern (decoupled rollout + training workers) could parallelize SEAL pipeline stages. Currently SEAL runs sequentially; async decoupling would enable overlap of exploration, distillation, and absorption.

2. **P2P Weight Transfer for Model Distribution** — RDMA-based weight update pattern could enable NeoTrix to hot-swap model weights across distributed NT-IO nodes without full model reload. Applicable when switching between LLM providers.

3. **R3 for Attention Routing Stability** — Rollout Routing Replay pattern (record routing during inference, replay during training) could stabilize GWT attention routing. Currently GWT re-computes salience each cycle; recording + replaying could reduce variance.

4. **Fault Tolerance for NT-REPAIR** — Miles' in-place engine recovery pattern (detect failure → recover → resume without restart) is the production-grade version of NT-REPAIR's self-healing. Currently NT-REPAIR detects degradation; Miles pattern adds automatic recovery.

### Implementation Changes

- `nt_mind::async_seal` — Decouple SEAL pipeline stages with async worker pools
- `nt_io::p2p_weight_transfer` — RDMA-based model weight hot-swap across distributed nodes
- `nt_core::routing_replay` — Record GWT routing decisions for training-time replay
- `nt_repair::fault_tolerance` — In-place engine recovery with automatic resume

---

## Cross-Repository Synthesis

### Pattern Convergence Matrix

| Pattern | OpenWAM | CloddsBot | Semantica | WeKnora | Jingyun | Miles | NeoTrix Target |
|---------|---------|-----------|-----------|---------|---------|-------|----------------|
| Lazy Loading | - | ✅ 121 skills | - | - | ✅ 800+ plugins | - | NT-ACT skill nodes |
| Decision Tracking | - | ✅ Trade Ledger | ✅ Decision Intelligence | - | - | - | NT-META decision records |
| Fault Tolerance | - | - | - | - | - | ✅ In-place recovery | NT-REPAIR self-healing |
| Context Compaction | - | ✅ WebChat | - | ✅ Chat | - | - | NT-NEXUS session memory |
| Graph Memory | - | - | ✅ Context Graph | ✅ Wiki KG | - | - | NT-MEMORY knowledge graph |
| Async Pipeline | ✅ Hydra composition | - | - | ✅ MQ tasks | - | ✅ Fully async RL | SEAL pipeline parallelism |
| Commercial Loop | - | ✅ Agent commerce | - | - | ✅ Full commercial | - | NT-ACT marketplace |
| Deterministic Reasoning | - | - | ✅ Rete/Datalog/SPARQL | - | - | - | NT-CORE reasoning engine |
| Sandbox Execution | - | - | - | ✅ Docker/E2B/Cube | - | - | NT-ACT skill isolation |
| Provenance/Audit | - | ✅ SHA-256 + on-chain | ✅ W3C PROV-O | ✅ Langfuse | - | - | NT-SHIELD audit trail |

### Top 5 Fusion Priorities

| Priority | Pattern | Source | Target Module | Effort |
|----------|---------|--------|---------------|--------|
| P0 | **Context Graph for KB** | Semantica | NT-MEMORY | High — new graph layer on SQLite |
| P0 | **Lazy Skill Loading** | CloddsBot/Jingyun | NT-ACT | Medium — isolate skill dependencies |
| P1 | **Decision Intelligence** | Semantica | NT-META | Medium — structured decision records |
| P1 | **Async SEAL Pipeline** | Miles | NT-MIND | High — decouple pipeline stages |
| P2 | **Fault Tolerance** | Miles | NT-REPAIR | Medium — automatic recovery pattern |

### Implementation Roadmap

**Phase 1 (Immediate):**
- Lazy skill loading with crash isolation (CloddsBot pattern → NT-ACT)
- Circuit breaker for RiskAssessor (CloddsBot pattern → NT-SHIELD)

**Phase 2 (Next Cycle):**
- Context Graph layer on NT-MEMORY (Semantica pattern)
- Decision Intelligence for NT-META (Semantica pattern)
- Chunk editing with revision history (WeKnora pattern → NT-MEMORY)

**Phase 3 (Future):**
- Async SEAL pipeline (Miles pattern → NT-MIND)
- Fault tolerance for NT-REPAIR (Miles pattern)
- Agent Commerce Protocol (CloddsBot pattern → NT-ACT marketplace)
- Wiki Mode crystallization (WeKnora pattern → NT-MEMORY)

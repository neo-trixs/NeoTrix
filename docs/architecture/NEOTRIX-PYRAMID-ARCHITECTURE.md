# NeoTrix Pyramid Architecture Design Document

> **Version**: 1.0.0 | **Date**: 2026-09-21 | **Status**: Architecture Audit Design | **Classification**: Internal

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture Philosophy & Axioms](#2-architecture-philosophy--axioms)
3. [Pyramid Architecture Overview](#3-pyramid-architecture-overview)
4. [Layer Detailed Design (L0-L6)](#4-layer-detailed-design)
5. [Dependency Governance](#5-dependency-governance)
6. [Full Architecture Audit Design](#6-full-architecture-audit-design)
7. [Quality Gate System](#7-quality-gate-system)
8. [Anti-Fragile & Evolution Roadmap](#8-anti-fragile--evolution-roadmap)
9. [Appendix](#9-appendix)

---

## 1. Executive Summary

### 1.1 Project Positioning

NeoTrix is a **self-evolving AI agent system** (v0.18.0) implemented in Rust. Core formula:

```
Psi(t+1) = Select(O, x) x Select(M, x) x Psi(t)
```

The system follows the **Consciousness-Embodiment-Capability (CEC) six-layer pyramid architecture**, implementing a complete capability stack from bottom-level infrastructure to top-level meta-cognition. Each layer depends only on the layer below it, forming a strict unidirectional dependency topology.

### 1.2 Architecture Scale

| Metric | Value |
|--------|-------|
| Source Files | 2,426  |
| Lines of Code | ~798,707 |
| Architecture Layers | 6 (L0-L6) |
| Sub-modules | 360 |
| Independent Crates | 7 |
| External Absorption Modules | 5 (sim, guard_core, cumora, munder-difflin, colibri) |

### 1.3 Design Goals

| Goal | Description | Metric |
|------|-------------|--------|
| **Clear Layers** | Strict unidirectional dependency, no reverse references |  reverse dependency detection |
| **Engineering** | Facade/Registry/Adapter pattern unification | Public interfaces <= 3 per layer |
| **Auditable** | Complete audit points and quality gates per layer | 100% audit coverage |
| **Evolvable** | Module absorption mechanism, self-evolution pipeline | Absorption cycle <= 2 weeks |
| **Anti-Fragile** | Circuit breaker, degradation, self-healing | Fault recovery time <= 30s |

---

## 2. Architecture Philosophy & Axioms

### 2.1 Core Axioms

| ID | Axiom | Description | Violation Consequence |
|----|-------|-------------|----------------------|
| **R-P1** | Zero Unsafe |  globally prohibits unsafe code | Compilation failure |
| **R-P2** | Pointer Conservation | Ownership is explicit, no dangling pointers | Compilation failure (Rust ownership system) |
| **R-P3** | Dark Forest | Principle of least exposure, Facade limits public interfaces | Compilation warning + architecture review |
| **R-P4** | Unidirectional Dependency | Higher layers depend on lower layers, reverse references prohibited |  rejection |
| **R-P5** | Module Autonomy | Every module can be compiled and tested independently | CI blockage |
| **R-P6** | Type Safety | Zero type confusion, Newtype pattern enforces semantics | Compilation failure |

### 2.2 Design Pattern Matrix

| Pattern | Implementation | Scope |
|---------|---------------|-------|
| **Facade** | Single public interface per layer:  (L1),  (L3/L4/L5/L6),  (L5) | L1, L3, L5 |
| **Registry** | Extensible registries:  (L2, 15+ sources),  (L5),  (L5, 30+ providers) | L2, L5 |
| **Engine Adapter** | LLM Provider abstraction with Tier-based routing: Fast / Standard / Think | L1 |
| **Triage Gate** | Small models/rules as gatekeepers, decide whether to wake full agent (saves 80%+ compute) | L0-L2 |
| **Self-Evolution** | SEAL pipeline, experience trees, evolving evaluators | L5-L6 |
| **Module Absorption** | External system patterns absorbed into layered architecture | All layers |

---

## 3. Pyramid Architecture Overview

### 3.1 Pyramid Diagram (Top-Down)

```
                              +---------------------------+
                              |         L6 Meta          |  Meta-Cognition Layer
                              |  Meta-model/Governance    |  Smallest area, highest abstraction
                              |  Evolution / Self-Heal    |
                              +-------------+-------------+
                                            |
                              +-------------v-------------+
                              |       L5 Cognition        |  Cognition Layer
                              |  Consciousness/Decision    |  Core intelligence
                              |  GWT / SEAL Pipeline      |
                              +-------------+-------------+
                                            |
                              +-------------v-------------+
                              |        L4 Emotion         |  Emotion Layer
                              |  Emotion/Memory/Sense     |  Contextual understanding
                              |  Coverage Ledger          |
                              +-------------+-------------+
                                            |
                              +-------------v-------------+
                              |       L3 Embodiment       |  Embodiment Layer
                              |  Security/Compute/Pathfind |  Physical expression
                              |  Shield / Guard Chain     |
                              +-------------+-------------+
                                            |
                              +-------------v-------------+
                              |       L2 Perception       |  Perception Layer
                              |  World Model/E8/Data      |  Data perception
                              |  Vector Store/Web         |
                              +-------------+-------------+
                                            |
                              +-------------v-------------+
                              |        L1 Action          |  Action Layer
                              |  LLM Providers/IO/Dispatch |  Operation execution
                              |  Media / Infrastructure   |
                              +-------------+-------------+
                                            |
                              +-------------v-------------+
                              |       L0 Substrate        |  Foundation Layer
                              |  Core Types/Events/ECS    |  Largest area, zero dependencies
                              |  Telemetry/Errors/Platform|
                              +---------------------------+
```

### 3.2 Scale Distribution (Top-Down)

```
L6 Meta        [####]                          ~11K lines (1.4%)
L5 Cognition   [#####################]         ~182K lines (22.8%)
L4 Emotion     [##########]                   ~83K lines (10.4%)
L3 Embodiment  [#########]                    ~72K lines (9.0%)
L2 Perception  [##########]                   ~89K lines (11.1%)
L1 Action      [##################]           ~141K lines (17.7%)
L0 Substrate   [####]                         ~19K lines (2.4%)
External Crates[#####################]         ~281K lines (35.2%)
```

### 3.3 Dependency Flow Diagram

```
  L6 --> L5 --> L4 --> L3 --> L2 --> L1 --> L0   (ALLOWED)

  Prohibited directions (all reversed arrows are forbidden):
    L0 --> L1  X
    L1 --> L2  X
    L2 --> L3  X
    L3 --> L4  X
    L4 --> L5  X
    L5 --> L6  X

  Special channels (L0 reverse re-exports):
    nt_core_cross_layer    -> L5<->L6 abstraction via L0
    nt_core_kb_primitives  -> L6 types re-exported to L0
    nt_core_memory_asset   -> L6 types re-exported to L0
```

### 3.4 Layer Responsibility Matrix

| Layer | Primary Responsibility | Key Abstractions | Public Facade |
|-------|----------------------|------------------|---------------|
| L0 | Foundation infrastructure | Types, Events, ECS, Telemetry | None (internal) |
| L1 | Execution & IO | LLM Providers, Task Dispatch, Media |  |
| L2 | Data Perception | World Model, E8 Reasoning, Vector Store | Module-level pub |
| L3 | Physical Expression | Security Shield, Computer, Guard Chain |  (re-export) |
| L4 | Emotion & Memory | Emotion Engine, Coverage Ledger, Memory | Module-level pub |
| L5 | Cognition & Decision | Consciousness, GWT, SEAL, Planning |  |
| L6 | Meta-Cognition | Self-Model, Governance, Evolution, Healing | Module-level pub |

---

## 4. Layer Detailed Design

### 4.1 L0 Substrate -- Foundation Layer

**Responsibility**: Pure Rust foundation, zero external dependencies, provides core types, event system, ECS framework, telemetry, and error handling.

#### Module Inventory

| Module | Files | Lines | Responsibility | Audit Level |
|--------|-------|-------|----------------|-------------|
| `nt_core_shared_types` | 1 | 306 | Shared types (Modality, E8VsaEmbedding) | P0 |
| `nt_core_event` | 1 | 471 | Event system (EventBus, Event, EventHandler) | P0 |
| `nt_ecs` | 1 | 1,362 | ECS framework (Archetype-based + parallel scheduler) | P0 |
| `nt_core_telemetry` | 1 | 1,570 | Telemetry store (tracing, metrics, logs, spans) | P1 |
| `nt_core_error/` | 3 | 1,695 | Unified error types + recovery mechanism | P0 |
| `nt_core_platform/` | 12 | 1,594 | Platform init, pipeline registry | P0 |
| `nt_core_di` | 1 | 175 | Dependency injection container | P1 |
| `nt_core_traits` | 1 | 380 | Core traits (AbsorbTextScanner) | P0 |
| `nt_core_hex` | 1 | 1,530 | E8 x 64 state-space reasoning model | P0 |
| `nt_core_math` | 1 | 226 | Math utilities (cosine, Hamming, URL norm) | P1 |
| `nt_core_cache` | 1 | 580 | Semantic cache | P1 |
| `nt_core_hot_data` | 1 | 506 | Hot data handling | P1 |
| `nt_core_cross_layer` | 1 | 915 | Cross-layer types (L5<->L6 abstraction) | P0 |
| `nt_core_state` | 1 | 162 | Internal state unified KB constructs | P1 |
| `nt_tick_schedule` | 1 | 257 | Multi-timescale tick schedule (5 levels) | P1 |
| `nt_core_prompt_cache` | 1 | 345 | Prompt cache | P2 |
| `nt_core_schema_watchdog` | 1 | 366 | Schema watchdog | P2 |
| `nt_core_speculative_decoding` | 1 | 510 | Speculative decoding | P2 |
| `nt_core_ws` | 1 | 248 | WebSocket foundation | P2 |
| `nt_core_substrate_types` | 1 | 340 | Substrate type definitions | P0 |
| `nt_core_axiom_tree` | 1 | 328 | Axiom tree | P2 |
| `nt_core_awareness_monitor` | 1 | 283 | Awareness monitoring | P2 |
| `nt_core_kb_primitives` | 1 | 520 | KB storage primitives (re-exported from L6) | P0 |
| `nt_core_memory_asset` | 1 | 202 | Memory asset types (re-exported from L6) | P1 |

**Total**: 45 files, ~18,893 lines

#### L0 Audit Checkpoints

| Check | Tool | Criteria | Severity |
|-------|------|----------|----------|
| Zero external deps | `cargo deny` | No external crate imports in L0 | BLOCKER |
| Zero unsafe | `cargo audit` + `forbid(unsafe_code)` | 0 unsafe blocks | BLOCKER |
| Event system completeness | Code review | EventBus 100% coverage | HIGH |
| ECS performance | Criterion bench | archetype query < 1us | HIGH |
| Error recovery coverage | Code review | All Error types implement Recovery | HIGH |
| Telemetry integrity | Integration test | traces + metrics + logs triple | MEDIUM |
| Cross-layer correctness | `cargo deny` | L5<->L6 via L0 does not violate direction | BLOCKER |
| Tick scheduler precision | Unit test | 5-level tick error < 5% | MEDIUM |

---

### 4.2 L1 Action -- Execution Layer

**Responsibility**: Executes LLM calls, IO operations, memory management, media processing, and task dispatch. Depends only on L0.

#### Module Inventory

| Module | Files | Lines | Responsibility | Audit Level |
|--------|-------|-------|----------------|-------------|
| `nt_io/` | 163 | 53,127 | IO operations (the largest L1 module) | P0 |
| -- `nt_io_provider/` | 30+ | -- | LLM provider implementations | P0 |
| -- `nt_io_llm/` | 1 | 168 | Unified LLM interface | P0 |
| -- `nt_io_web/` | -- | -- | Web operations | P1 |
| -- `nt_io_inference/` | -- | -- | Speculative decoding | P2 |
| `nt_act/` | 214 | 76,308 | Action orchestration (largest in codebase) | P0 |
| `nt_media/` | 13 | 7,864 | Unified media capability | P1 |
| `nt_memory_spatial/` | 4 | 822 | Spatial memory storage | P1 |
| `nt_core_graph_memory` | 1 | 1,208 | Graph memory with PageRank | P1 |
| `nt_core_bank/` | 15 | 3,444 | Resource bank with iteration | P1 |
| `nt_core_resource_pool/` | 8 | 779 | Resource pool + selection strategies | P1 |
| `nt_core_task_dispatcher` | 1 | 1,473 | Task decomposition and dispatch | P0 |
| `nt_infra_ai/` | 6 | 1,082 | AI infrastructure: isolated environments | P1 |
| `nt_infra_breaker` | 1 | 242 | Circuit breaker | P0 |
| `nt_infra_persistence` | 1 | 131 | Persistence layer | P1 |
| `nt_infra_tracing` | 1 | 169 | Infrastructure tracing | P1 |
| `nt_infra_unified_search` | 1 | 485 | Unified search across providers | P1 |
| `nt_action_facade` | 1 | 385 | **Sole L1 facade** (Qingjian pattern) | P0 |
| `nt_core_llm/` | 1 | 168 | LLM unified interface | P0 |

**Total**: ~450+ files, ~147K lines

#### L1 Audit Checkpoints

| Check | Tool | Criteria | Severity |
|-------|------|----------|----------|
| Provider coverage | Code review | 30+ LLM providers registered | HIGH |
| Facade isolation | `cargo deny` | All L1 external access via `nt_action_facade` | BLOCKER |
| Circuit breaker | Unit test | All providers wrapped in breaker | HIGH |
| Resource pool | Unit test | Selection strategy correctness | MEDIUM |
| Task dispatch | Integration test | Decomposition + parallel execution | HIGH |
| Persistence | Integration test | Write/read round-trip | MEDIUM |

---

### 4.3 L2 Perception -- Sensing Layer

**Responsibility**: World perception, data collection, content understanding. Depends on L1.

#### Module Inventory

| Module | Files | Lines | Responsibility | Audit Level |
|--------|-------|-------|----------------|-------------|
| `nt_world/` | 259 | 60,668 | World model (second-largest module) | P0 |
| -- `source/` | -- | -- | DataSource/MediaSource/IntelSource traits | P0 |
| -- `data_source/` | 15+ | -- | IntelSource impls (GDELT, EDGAR, USGS, etc.) | P1 |
| -- `osint/` | 20+ | -- | OsintSource impls (DNS, Shodan, Censys, etc.) | P1 |
| -- `crawl/` | -- | -- | Web crawling capability | P1 |
| `nt_core_e8/` | 22 | 14,761 | E8 abduction/hypothesis engine | P0 |
| `nt_core_hcube/` | 24 | 9,813 | Hypercube/E8 lattice | P0 |
| `nt_core_knowledge/` | 12 | 2,271 | Knowledge types | P0 |
| `nt_core_sense/` | 5 | 1,102 | Sensor traits | P1 |
| `nt_core_vector_store/` | 7 | 1,285 | Vector store factory | P1 |
| `nt_core_code_search` | 1 | 671 | Code search capability | P1 |
| `nt_core_e8_predictor` | 1 | 356 | E8 state prediction | P1 |
| `nt_core_e8_vsa` | 1 | 201 | E8 VSA integration | P1 |
| `nt_judgment/` | 4 | 313 | Judgment primitives | P1 |
| `nt_routing/` | 1 | 127 | Perception routing | P1 |
| `nt_web_perception/` | 3 | 93 | Web perception with CSS selectors | P2 |

**Total**: ~360+ files, ~91K lines

#### L2 Audit Checkpoints

| Check | Tool | Criteria | Severity |
|-------|------|----------|----------|
| DataSource registry | Unit test | All 15+ sources register correctly | HIGH |
| E8 reasoning | Unit test | Abduction cycle produces valid hypotheses | HIGH |
| World model | Integration test | Query/response latency < 50ms | MEDIUM |
| Vector store | Unit test | Embedding + search round-trip | HIGH |
| OSINT sources | Integration test | 20+ sources accessible | MEDIUM |

---

### 4.4 L3 Embodiment -- Physical Expression Layer

**Responsibility**: Security protection, computational clusters, and guard chains. Depends on L2.

#### Module Inventory

| Module | Files | Lines | Responsibility | Audit Level |
|--------|-------|-------|----------------|-------------|
| `nt_shield/` | 247 | 70,186 | Security protection system (P0 critical) | P0 |
| -- Guard chain | -- | -- | Guard chain, sandbox, circuit breaker | P0 |
| -- Stealth network | -- | -- | Feature-gated stealth capability | P0 |
| -- Traffic intercept | -- | -- | Traffic interception | P0 |
| -- Egress control | -- | -- | Egress control + audit trail | P0 |
| -- AI security | -- | -- | AI security, pentest swarm | P0 |
| -- Anti-distillation | -- | -- | Anti-distillation defense | P0 |
| -- Safety alignment | -- | -- | Safety alignment engine | P0 |
| -- Proxy detection | -- | -- | Proxy detection + account clustering | P1 |
| `nt_computer` | 1 | 477 | Computer abstraction (first-class) | P1 |
| `nt_computer_fleet` | 1 | 262 | Cross-OS computer fleet management | P1 |
| `nt_core_guard_chain` | 1 | 234 | Guard chain primitives | P0 |
| `nt_security/` | 3 | 149 | Security policies | P0 |
| `nt_astar` | 1 | 295 | A* pathfinding (absorbed from neotrix-sim) | P2 |

**Total**: ~255+ files, ~71K lines

#### L3 Audit Checkpoints

| Check | Tool | Criteria | Severity |
|-------|------|----------|----------|
| Shield coverage | Security audit | All egress/ingress guarded | BLOCKER |
| Guard chain | Unit test | Chain execution order correct | HIGH |
| Safety alignment | Integration test | Alignment engine prevents harmful actions | BLOCKER |
| Anti-distillation | Red team test | Distillation attack resistance | HIGH |
| Computer abstraction | Unit test | Cross-OS execution consistency | MEDIUM |

---

### 4.5 L4 Emotion -- Emotion & Memory Layer

**Responsibility**: Emotion engine, affect modeling, empathy, and memory with coverage ledger. Depends on L3.

#### Module Inventory

| Module | Files | Lines | Responsibility | Audit Level |
|--------|-------|-------|----------------|-------------|
| `nt_feel/` | 15 | 5,492 | Digital human emotions | P1 |
| `nt_memory/` | 199 | 77,513 | Memory system (one of largest modules) | P0 |
| -- KB pipeline | -- | -- | Knowledge base pipeline | P0 |
| -- Experience trees | -- | -- | Experience tree management | P0 |
| -- Coverage ledger | -- | -- | Coverage tracking | P1 |
| -- Wikiskill | -- | -- | Wiki-skill integration | P1 |
| `nt_emotion_facade` | 1 | 196 | Emotion facade | P1 |
| `nt_emotion_reasoning_bridge` | 1 | 324 | Emotion-to-reasoning bridge (PAD -> GWT/E8/CT) | P0 |
| `nt_feel_facade` | 1 | 15 | Feel facade | P1 |

**Total**: ~218+ files, ~84K lines

#### L4 Audit Checkpoints

| Check | Tool | Criteria | Severity |
|-------|------|----------|----------|
| Memory KB pipeline | Integration test | Write/query/delete round-trip | HIGH |
| Experience tree | Unit test | Tree traversal + pruning correctness | MEDIUM |
| Coverage ledger | Unit test | Coverage tracking accuracy | MEDIUM |
| Emotion bridge | Integration test | PAD -> GWT/E8/CT conversion fidelity | HIGH |
| Memory persistence | Integration test | Cross-session memory recovery | HIGH |

---

### 4.6 L5 Cognition -- Cognition & Decision Layer

**Responsibility**: Consciousness, decision-making, reasoning, planning, context management. Depends on L4.

#### Module Inventory

| Module | Files | Lines | Responsibility | Audit Level |
|--------|-------|-------|----------------|-------------|
| `nt_mind/` | 397 | 137,432 | **The reasoning brain** (largest module in entire codebase) | P0 |
| -- SEAL pipeline | -- | -- | Self-iterating brain, self-evolver | P0 |
| -- Evolution experiments | -- | -- | A/B test design, experiment runner | P1 |
| -- Skill engine | -- | -- | Skill creation, registration, execution | P0 |
| `nt_core/` | 151 | 44,486 | Core cognition | P0 |
| -- Information absorber | -- | -- | Information absorption engine | P0 |
| -- Abstract engine | -- | -- | Abstraction layer | P1 |
| -- Memory (semantic/episodic) | -- | -- | Cognitive memory systems | P0 |
| -- Consciousness core | -- | -- | Core consciousness | P0 |
| `nt_core_consciousness_core` | 1 | 4,665 | Consciousness core module | P0 |
| `nt_core_consciousness_tree/` | 8 | 3,514 | Consciousness tree + review | P0 |
| `nt_core_gwt/` | 25 | 9,310 | Global Workspace Theory attention | P0 |
| `nt_core_gate/` | 2 | 3,108 | Gating mechanism | P0 |
| `nt_core_prm/` | 7 | 3,715 | PRM collector | P1 |
| `nt_core_plan/` | 1 | 681 | Planning system | P0 |
| `nt_core_policy/` | 1 | 935 | Policy engine | P0 |
| `nt_core_rule_memory` | 1 | 1,130 | Rule-based memory | P1 |
| `nt_core_context/` | 4 | 1,211 | Context budget management | P1 |
| `nt_goal/` | 8 | 2,073 | Goal management + behavioral verifier | P0 |
| `nt_council/` | 1 | 110 | Multi-perspective deliberation | P1 |
| `nt_decision_engine` | 1 | 686 | Multi-layer decision engine | P0 |
| `nt_resonator_network` | 1 | 1,037 | VSA resonator network | P1 |
| `nt_core_model_gateway` | 1 | 604 | Unified model gateway (cost-aware) | P0 |
| `nt_core_model_router` | 1 | 630 | Real-time model router | P0 |
| `nt_core_byoa` | 1 | 847 | Bring Your Own Agent | P1 |
| `nt_cognition_facade` | 1 | 327 | **Sole L5 facade** | P0 |

**Crate Re-exports from L5**:
| Crate | Key Modules |
|-------|------------|
| `neotrix-consciousness` | consciousness_core, GWT, context, second_bubble_wall, IIT phi |
| `neotrix-reasoning` | reasoning_core, quantum_fusion, resonator, aura, scoring |
| `neotrix-gateway` | model_gateway, skill_registry, semantic_router, hybrid_search, BYOA |
| `neotrix-multi-agent` | multi_agent, parallel, coordinator, hive, experience_tree |

**Total**: ~600+ files, ~220K lines (including crate re-exports)

#### L5 Audit Checkpoints

| Check | Tool | Criteria | Severity |
|-------|------|----------|----------|
| SEAL pipeline | Integration test | Self-iteration produces measurable improvement | HIGH |
| GWT attention | Unit test | Attention routing correctness | HIGH |
| Consciousness tree | Unit test | Tree traversal + review cycle | MEDIUM |
| Goal management | Integration test | Goal lifecycle (create/pause/resume/cancel) | HIGH |
| Model gateway | Integration test | Cost-aware routing + fallback | HIGH |
| Facade isolation | `cargo deny` | All L5 external access via `nt_cognition_facade` | BLOCKER |
| Policy engine | Unit test | Policy enforcement correctness | HIGH |
| Planning system | Integration test | Plan create/step/complete lifecycle | MEDIUM |

---

### 4.7 L6 Meta -- Meta-Cognition Layer

**Responsibility**: Meta-cognition, self-modeling, evolution, governance. Depends on L5. This is the apex of the pyramid.

#### Module Inventory

| Module | Files | Lines | Responsibility | Audit Level |
|--------|-------|-------|----------------|-------------|
| `coordination/` | -- | -- | Coordination subsystem | P0 |
| -- `nt_governance/` | 5 | 907 | Governance system | P0 |
| -- `nt_meta_cleanup/` | 5 | 411 | Cleanup coordinator | P1 |
| -- `nt_meta_async_safety` | 1 | 186 | Async safety | P0 |
| -- `nt_meta_build_watchdog` | 1 | 375 | Build watchdog | P1 |
| -- `nt_meta_concurrency_detector` | 1 | 241 | Concurrency detection | P1 |
| `memory/` | -- | -- | Meta-memory | P0 |
| -- Experience tree | 1 | 234 | Experience tree management | P0 |
| -- Knowledge pipeline | 1 | 228 | Knowledge pipeline | P1 |
| -- Wikiskill | 1 | 298 | Wiki-skill integration | P1 |
| `healing/` | -- | -- | Self-healing subsystem | P0 |
| -- Self-test | 2 | 514 | System self-testing | P0 |
| -- Causal trace | 1 | 637 | Causal trace for repair | P0 |
| -- Self-heal | 1 | 301 | Self-healing execution | P0 |
| -- Consciousness monitor | 1 | 679 | Consciousness health monitoring | P0 |
| -- Consciousness gold standard | 1 | 558 | Gold standard benchmark | P1 |
| -- Repair facade | 1 | 36 | Repair facade | P1 |
| -- Repair causal trace | 1 | 637 | Repair causal trace | P1 |
| `evolution/` | -- | -- | Evolution subsystem | P0 |
| -- Background loop manager | 1 | 207 | Background loop management | P0 |
| -- Human approval | 1 | 219 | Human-in-the-loop approval | P0 |
| `nt_core_self/` | 29 | 13,026 | Self model + skill crystals | P0 |
| `nt_core_self_review/` | 3 | 2,855 | Self review | P0 |
| `nt_core_capability/` | 17 | 5,579 | Capability system | P0 |
| `nt_core_self_constitution` | 1 | 956 | Self constitution | P0 |
| `nt_core_self_model` | 1 | 201 | Self model value function | P0 |
| `nt_core_observer` | 1 | 1,012 | Observer pattern | P0 |
| `nt_core_scheduler/` | 4 | 1,855 | Scheduler system | P1 |
| `nt_core_absorb/` | 2 | 526 | Information absorption | P1 |
| `nt_meta/` | 44 | 11,689 | Meta system (full) | P0 |
| `nt_nexus/` | 8 | 2,147 | Cross-session checkpoint | P0 |
| `nt_agent_identity` | 1 | 631 | Agent identity system | P0 |
| `nt_agent_gallery` | 1 | 316 | Agent gallery | P1 |
| `nt_emergence_detector` | 1 | 662 | Emergence detection | P1 |
| `nt_safety_monitor` | 1 | 494 | Safety monitor + anomaly detection | P0 |

**Total**: ~130+ files, ~47K lines

#### L6 Audit Checkpoints

| Check | Tool | Criteria | Severity |
|-------|------|----------|----------|
| Governance | Unit test | Policy enforcement + compliance | BLOCKER |
| Self-healing | Integration test | Fault detection + repair cycle < 30s | HIGH |
| Self-model | Unit test | Value function accuracy | HIGH |
| Evolution | Integration test | Background loop produces measurable improvement | MEDIUM |
| Safety monitor | Unit test | Anomaly detection + alerting | BLOCKER |
| Cross-session | Integration test | Checkpoint save/load round-trip | HIGH |
| Agent identity | Unit test | Identity persistence across sessions | MEDIUM |
| Capability system | Unit test | Capability registration + lookup | HIGH |

---

## 5. Dependency Governance

### 5.1 Dependency Rules Matrix

| Rule ID | Description | Enforcement | Tool |
|---------|-------------|-------------|------|
| **D-1** | Each layer may only depend on layers below it | Compile-time + CI | `cargo deny` |
| **D-2** | L0 provides reverse re-export channels for L5<->L6 types | Architecture review | Manual audit |
| **D-3** | External crates are isolated in `/crates/` directory | Build system | Cargo workspace |
| **D-4** | Facade is the sole public interface per layer | `pub` visibility audit | `cargo doc` + custom lint |
| **D-5** | No cross-layer `use` statements bypassing facades | Compile-time | `cargo deny` + custom lint |

### 5.2 Reverse Re-export Channels (L0)

These channels allow L6 types to be used in L0 without violating the dependency direction:

```
L6 (defines types)
    |
    v
nt_core_kb_primitives (L0) ---- re-exports L6 KB types
nt_core_memory_asset (L0)  ---- re-exports L6 memory asset types
nt_core_cross_layer (L0)   ---- provides L5<->L6 abstraction bridge
    |
    v
L0 (consumes re-exported types)
```

**Audit**: These re-exports must be reviewed quarterly to ensure they do not accumulate unnecessary type leaks.

### 5.3 External Crate Dependency Map

| Crate | Depends On | Used By | Size |
|-------|-----------|---------|------|
| `neotrix-consciousness` | L0 | L5 (re-export) | ~20K lines |
| `neotrix-reasoning` | L0 | L5 (re-export) | ~15K lines |
| `neotrix-gateway` | L0, L1 | L5 (re-export) | ~10K lines |
| `neotrix-multi-agent` | L0 | L5 (re-export) | ~12K lines |
| `neotrix-types` | L0 | All layers | ~5K lines |
| `neotrix-sysctl` | L0 | System control | ~1K lines |
| `nt-lang` | L0 | DSL parsing | ~3K lines |

### 5.4 Module Absorption Registry

External modules are absorbed into the layered architecture through a structured process:

| Source System | Target Layer | Target Module | Absorption Date | Status |
|---------------|-------------|---------------|-----------------|--------|
| nt-world-sim | L2 | nt_world | 2026-08 | COMPLETE |
| guard_core | L3 | nt_shield | 2026-07 | COMPLETE |
| cumora | L5 | nt_core_byoa | 2026-08 | COMPLETE |
| munder-difflin | L6 | nt_agent_gallery | 2026-08 | COMPLETE |
| neotrix-sim | L3/L5 | nt_astar, nt_decision_engine | 2026-07 | COMPLETE |

---

## 6. Full Architecture Audit Design

### 6.1 Audit Framework

The audit framework follows a **three-tier model**: Continuous (automated), Periodic (scheduled), and Deep (ad-hoc).

```
+-------------------------------------------------------------------+
|                    Architecture Audit Framework                    |
+-------------------------------------------------------------------+
|                                                                    |
|  Tier 1: CONTINUOUS (Every Build)                                 |
|  +--------------------------------------------------------------+ |
|  | - cargo check --all-targets -p neotrix                       | |
|  | - cargo test -p neotrix --lib                                | |
|  | - cargo deny (dependency audit)                              | |
|  | - forbid(unsafe_code) verification                           | |
|  | - Facade boundary enforcement                                | |
|  +--------------------------------------------------------------+ |
|                                                                    |
|  Tier 2: PERIODIC (Weekly/Sprint)                                 |
|  +--------------------------------------------------------------+ |
|  | - Architecture fitness functions (nt_core_arch_fitness)      | |
|  | - Dependency direction validation                            | |
|  | - Public API surface analysis                                | |
|  | - Performance regression testing                             | |
|  | - Security vulnerability scanning                            | |
|  +--------------------------------------------------------------+ |
|                                                                    |
|  Tier 3: DEEP (Quarterly / On-Demand)                             |
|  +--------------------------------------------------------------+ |
|  | - Full architecture review (rev-officer skill)               | |
|  | - Cross-layer coupling analysis                              | |
|  | - Module absorption review                                   | |
|  | - Anti-fragility stress testing                              | |
|  | - Evolution effectiveness assessment                         | |
|  +--------------------------------------------------------------+ |
|                                                                    |
+-------------------------------------------------------------------+
```

### 6.2 Layer-Specific Audit Specifications

#### L0 Audit Protocol

| Audit Item | Method | Frequency | Owner | Criteria |
|-----------|--------|-----------|-------|----------|
| Zero unsafe | Automated scan | Every build | CI | 0 unsafe blocks |
| Zero external deps | `cargo deny` | Every build | CI | No external imports in L0 |
| ECS correctness | Property test | Weekly | Arch Fitness | All operations idempotent |
| Event system | Fuzz test | Weekly | Arch Fitness | No panics, no UB |
| Error recovery | Code review | Sprint | Dev team | All Error types implement Recovery |
| Cross-layer types | Manual review | Quarterly | Architect | No dependency violations |
| Tick scheduler | Performance test | Weekly | Arch Fitness | Precision within 5% |

#### L1 Audit Protocol

| Audit Item | Method | Frequency | Owner | Criteria |
|-----------|--------|-----------|-------|----------|
| Provider correctness | Integration test | Every build | CI | All 30+ providers respond |
| Facade isolation | `cargo deny` | Every build | CI | All external access via facade |
| Circuit breaker | Chaos test | Weekly | SRE | Breaker trips within 3 failures |
| Resource pool | Load test | Sprint | Perf team | Pool exhaustion handled gracefully |
| Task dispatch | Integration test | Every build | CI | Decomposition + parallel execution |
| Media processing | E2E test | Sprint | QA | All media types handled |
| Persistence | Durability test | Weekly | SRE | Data survives restart |

#### L2 Audit Protocol

| Audit Item | Method | Frequency | Owner | Criteria |
|-----------|--------|-----------|-------|----------|
| DataSource registry | Unit test | Every build | CI | All sources register correctly |
| E8 reasoning | Property test | Weekly | Arch Fitness | Hypotheses are internally consistent |
| World model | Integration test | Every build | CI | Query latency < 50ms (p99) |
| Vector store | Accuracy test | Weekly | ML team | Recall@10 > 90% |
| OSINT sources | Connectivity test | Daily | SRE | 20+ sources accessible |
| Web perception | E2E test | Sprint | QA | CSS selector extraction correct |
| Knowledge types | Code review | Sprint | Dev team | Type hierarchy consistent |

#### L3 Audit Protocol

| Audit Item | Method | Frequency | Owner | Criteria |
|-----------|--------|-----------|-------|----------|
| Shield coverage | Penetration test | Monthly | Security | 0 unguarded egress/ingress |
| Guard chain | Unit test | Every build | CI | Execution order correct |
| Safety alignment | Red team test | Monthly | Security | Alignment prevents harmful actions |
| Anti-distillation | Adversarial test | Monthly | Security | Distillation attack resistance > 95% |
| Computer abstraction | Cross-OS test | Weekly | Dev team | Behavior consistent across OS |
| Security policies | Compliance audit | Quarterly | Governance | All policies enforceable |

#### L4 Audit Protocol

| Audit Item | Method | Frequency | Owner | Criteria |
|-----------|--------|-----------|-------|----------|
| Memory KB pipeline | Integration test | Every build | CI | CRUD round-trip succeeds |
| Experience tree | Unit test | Every build | CI | Tree traversal + pruning correct |
| Coverage ledger | Accuracy test | Weekly | ML team | Coverage tracking within 5% |
| Emotion bridge | Fidelity test | Sprint | Dev team | PAD -> GWT/E8/CT conversion loss < 10% |
| Memory persistence | Durability test | Weekly | SRE | Cross-session recovery succeeds |
| Memory budget | Load test | Sprint | Perf team | Budget enforcement prevents OOM |

#### L5 Audit Protocol

| Audit Item | Method | Frequency | Owner | Criteria |
|-----------|--------|-----------|-------|----------|
| SEAL pipeline | Evolution test | Weekly | ML team | Measurable improvement per iteration |
| GATT attention | Unit test | Every build | CI | Attention routing correct |
| Consciousness tree | Unit test | Every build | CI | Tree review cycle completes |
| Goal management | Lifecycle test | Every build | CI | Create/pause/resume/cancel works |
| Model gateway | Integration test | Every build | CI | Cost-aware routing + fallback |
| Facade isolation | `cargo deny` | Every build | CI | All external access via facade |
| Policy engine | Compliance test | Sprint | Governance | Policy enforcement 100% |
| Planning system | Integration test | Every build | CI | Plan lifecycle completes |
| Multi-agent | Concurrency test | Weekly | Dev team | No deadlocks, no race conditions |
| Skill engine | E2E test | Sprint | QA | Skill create/register/execute works |

#### L6 Audit Protocol

| Audit Item | Method | Frequency | Owner | Criteria |
|-----------|--------|-----------|-------|----------|
| Governance | Compliance test | Every build | CI | Policy enforcement 100% |
| Self-healing | Chaos test | Weekly | SRE | Fault detection + repair < 30s |
| Self-model | Accuracy test | Monthly | ML team | Value function accuracy > 85% |
| Evolution | Effectiveness test | Monthly | ML team | Background loop measurable improvement |
| Safety monitor | Injection test | Weekly | Security | Anomaly detection + alerting |
| Cross-session | Durability test | Weekly | SRE | Checkpoint save/load round-trip |
| Agent identity | Persistence test | Sprint | Dev team | Identity persists across sessions |
| Capability system | Registry test | Every build | CI | Registration + lookup correct |
| Emergence detection | Sensitivity test | Monthly | ML team | False positive rate < 5% |

### 6.3 Audit Severity Levels

| Severity | Description | Response Time | Escalation |
|----------|-------------|---------------|------------|
| **BLOCKER** | Compilation failure, security vulnerability, data loss risk | Immediate | Auto-merge block |
| **HIGH** | Performance regression, functionality broken, compliance violation | 24 hours | Sprint backlog |
| **MEDIUM** | Code quality issue, minor regression, documentation gap | 1 week | Team review |
| **LOW** | Style inconsistency, optimization opportunity | Next sprint | Backlog |

### 6.4 Audit Report Template

```markdown
# Architecture Audit Report

**Date**: YYYY-MM-DD
**Scope**: [L0-L6 / Specific Layer / Specific Module]
**Auditor**: [Tool / Person]
**Severity Distribution**: [BLOCKER: N | HIGH: N | MEDIUM: N | LOW: N]

## Findings

### [BLOCKER] Finding Title
- **Location**: `file:line`
- **Description**: ...
- **Impact**: ...
- **Recommendation**: ...
- **Status**: OPEN / IN_PROGRESS / RESOLVED

## Summary

| Metric | Value |
|--------|-------|
| Total findings | N |
| BLOCKER | N |
| HIGH | N |
| MEDIUM | N |
| LOW | N |
| Resolution rate | N% |
| Mean time to resolution | N days |
```

---

## 7. Quality Gate System

### 7.1 Build Pipeline Gates

```
PR Submitted
    |
    v
[Gate 1: Compile Check] ---------> cargo check --all-targets
    |                                  |
    | Pass                             | Fail
    v                                  v
[Gate 2: Unit Tests] ------------> cargo test -p neotrix --lib
    |                                  |
    | Pass                             | Fail
    v                                  v
[Gate 3: Dependency Audit] ------> cargo deny
    |                                  |
    | Pass                             | Fail
    v                                  v
[Gate 4: Architecture Fitness] --> nt_core_arch_fitness
    |                                  |
    | Pass                             | Fail
    v                                  v
[Gate 5: Security Scan] ---------> cargo audit + custom lint
    |                                  |
    | Pass                             | Fail
    v                                  v
[Gate 6: Integration Tests] -----> cargo test --test '*'
    |                                  |
    | Pass                             | Fail
    v                                  v
[Gate 7: Performance Bench] -----> cargo bench (regression < 5%)
    |                                  |
    | Pass                             | Fail
    v                                  v
MERGE APPROVED
```

### 7.2 Architecture Fitness Functions

| Function | Layer | Metric | Threshold | Enforcement |
|----------|-------|--------|-----------|-------------|
| AF-1 | All | `unsafe` block count | 0 | Compile fail |
| AF-2 | All | Reverse dependency count | 0 | `cargo deny` |
| AF-3 | L0 | External dependency count | 0 | `cargo deny` |
| AF-4 | L1, L5 | Public interface count (facade) | <= 3 | Custom lint |
| AF-5 | All | Maximum function length | 200 lines | Custom lint |
| AF-6 | All | Maximum module coupling | <= 5 deps | Custom lint |
| AF-7 | All | Test coverage (critical path) | >= 80% | CI gate |
| AF-8 | L5 | SEAL iteration improvement | > 0% | Eval harness |
| AF-9 | L3 | Security policy coverage | 100% | Compliance audit |
| AF-10 | L6 | Self-healing MTTR | < 30s | Chaos test |

### 7.3 Documentation Quality Gates

| Gate | Criteria | Tool |
|------|----------|------|
| API documentation | All public items have `///` doc comments | `cargo doc` |
| Architecture decision records | All design decisions have ADR | Manual review |
| Module README | Each module has README.md | Automated check |
| Changelog | All changes have CHANGELOG entry | PR template |

---

## 8. Anti-Fragile & Evolution Roadmap

### 8.1 Anti-Fragility Design

| Mechanism | Layer | Description | Recovery Time |
|-----------|-------|-------------|---------------|
| Circuit Breaker | L1 | Prevents cascading failures from LLM providers | < 5s |
| Guard Chain | L3 | Multi-layer security defense | < 1s |
| Self-Healing | L6 | Automatic fault detection + repair | < 30s |
| Degradation | L1-L5 | Graceful degradation under load | < 3s |
| Evolution Loop | L6 | Continuous improvement via SEAL pipeline | Per iteration |
| Experience Trees | L4-L5 | Cross-session knowledge preservation | N/A (persistent) |
| Safety Monitor | L6 | Anomaly detection + emergency stop | < 1s |

### 8.2 Evolution Roadmap

```
Phase 1 (Current - v0.18.0)
+-------------------------------------------+
| - Six-layer CEC architecture complete     |
| - 30+ LLM providers integrated           |
| - SEAL self-evolution pipeline active     |
| - 20+ OSINT sources registered            |
| - Security shield operational             |
+-------------------------------------------+
           |
           v
Phase 2 (v0.19.0 - Q4 2026)
+-------------------------------------------+
| - Cross-session memory consolidation      |
| - Multi-agent coordination hardening      |
| - Performance optimization (2x latency)   |
| - Architecture fitness automation         |
| - Module absorption: 3 new sources        |
+-------------------------------------------+
           |
           v
Phase 3 (v0.20.0 - Q1 2027)
+-------------------------------------------+
| - Full anti-fragility (chaos engineering) |
| - Real-time evolution metrics dashboard   |
| - Architecture self-governance            |
| - External plugin ecosystem               |
| - Production SLA: 99.9% uptime            |
+-------------------------------------------+
```

### 8.3 Key Metrics Dashboard

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Lines of Code | ~798K | < 1M | ON_TRACK |
| Module Count | 360 | < 400 | ON_TRACK |
| Test Coverage | ~75% | > 80% | AT_RISK |
| Unsafe Blocks | 0 | 0 | PASS |
| Reverse Dependencies | 0 | 0 | PASS |
| LLM Providers | 30+ | 40+ | ON_TRACK |
| OSINT Sources | 20+ | 30+ | ON_TRACK |
| Self-Healing MTTR | ~25s | < 30s | PASS |
| Architecture Fitness Score | 85/100 | > 90 | AT_RISK |

---

## 9. Appendix

### 9.1 Glossary

| Term | Definition |
|------|-----------|
| **CEC** | Consciousness-Embodiment-Capability architecture pattern |
| **SEAL** | Self-Evolving Autonomous Learning pipeline |
| **GWT** | Global Workspace Theory (attention routing mechanism) |
| **E8** | 8-dimensional lattice reasoning model |
| **VSA** | Vector Symbolic Architecture |
| **PRM** | Process Reward Model |
| **IIT** | Integrated Information Theory (phi metric) |
| **BYOA** | Bring Your Own Agent |
| **OSINT** | Open Source Intelligence |
| **MTTR** | Mean Time To Recovery |
| **KB** | Knowledge Base |
| **ADR** | Architecture Decision Record |

### 9.2 Related Documents

| Document | Location | Description |
|----------|----------|-------------|
| ARCHITECTURE.md | `docs/architecture/` | High-level architecture overview |
| DATAFLOW.md | `docs/architecture/` | Data flow documentation |
| NEOTRIX-FULL-ARCHITECTURE.md | `docs/architecture/` | Full architecture reference |
| DESIGN-FUSION-ANALYSIS.md | `docs/architecture/` | Design fusion analysis |
| ALL-CAPABILITY-ANALYSIS.md | `docs/architecture/` | Capability analysis |
| CAPABILITY-TOPOLOGY-MAP.md | `docs/architecture/` | Capability topology |
| RUST-STANDARDS.md | Root | Rust coding standards |
| CONTRIBUTING.md | Root | Contribution guidelines |
| AGENTS.md | Root | Agent instructions |

### 9.3 Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-09-21 | NeoTrix Agent | Initial architecture audit design document |

---

*End of Document*

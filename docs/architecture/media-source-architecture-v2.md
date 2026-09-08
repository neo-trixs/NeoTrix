# NeoTrix Media Source Module — Architecture V2

## 1. Executive Summary

The `nt_world_media_source` module is NeoTrix's unified media ingestion engine, handling audio, video, image, document, book, lyrics, social, and feed data. It has grown to **223 Rust source files** across **42 subdirectories** — a 47-phase expansion that now requires structural consolidation.

This document defines the target architecture: a clean 6-layer structure that eliminates redundancy, resolves cross-domain misalignment, and enforces clear module boundaries.

---

## 2. Current State Analysis

### 2.1 Scale

```
Total Rust source files (neotrix-core/src):        3,201
Media source files (nt_world_media_source):          223
Media source subdirectories:                          42
Largest file (core/nt_core_consciousness_core.rs):  3,214 lines
Largest file (l1_action/nt_memory_kb/mod.rs):       2,929 lines
Unified directory files (near-duplicate):           1,603
```

### 2.2 Directory Map (Current)

```
nt_world_media_source/
├── ai/                    # Embedding, NER, sentiment, RAG, summarizer
├── analytics/             # A/B testing, funnel, recommendation
├── api_gateway/           # REST, GraphQL, gRPC
├── arch/                  # Architecture markers
├── audio/                 # 12 providers (Spotify, NetEase, QQ, etc.)
├── book/                  # Anna's Archive, OpenLibrary
├── cloud/                 # CDN, routing, abstraction
├── compliance/            # GDPR, HIPAA, SOC2, audit reports
├── cost/                  # API tracking, cache optimization, resource reports
├── deployment/            # Docker, config, health checks
├── disaster_recovery/     # Backup, failover, recovery
├── document/              # ArXiv, Semantic Scholar
├── ecosystem/             # E8 integration, evolution signals, KB sync, shield
├── edge/                  # Device adapter, lightweight inference, offline
├── engine/                # Core engine (cache, pool, ranking, health, Magika)
├── evolution/             # Adaptive quality, self-healing, SkillGlow
├── feed/                  # Dedup, scoring, alerting, parser
├── governance/            # Audit trail, classification, GDPR, retention
├── graph/                 # Entity graph, backlinks, timeline
├── i18n/                  # Locale, RTL, localization
├── image/                 # Pexels, Pixabay, Unsplash, Wikimedia
├── integration/           # Evolution feedback, KB bridge, security audit
├── lyrics/                # Genius, LRCLib, timed, multi-source
├── media_cache/           # Multi-level, consistency, warmup
├── ml/                    # A/B framework, embedding manager, vector index
├── multitenancy/          # Billing, tenant config, isolation
├── observability/         # Logging, metrics, tracing, alerting
├── performance/           # Concurrency, memory, pool, query optimization
├── pipeline/              # ETL, lineage, quality
├── plugin/                # Loader, MusicFree compat
├── realtime/              # SSE, WebSocket, live monitor
├── reliability/           # Retry, circuit breaker, bulkhead, degradation
├── security/              # Audit, input validation, rate limit, zero trust
├── serverless/            # Cold start, deployment, orchestration
├── simulation/            # Capacity planning, load test, system simulator
├── social/                # Instagram, TikTok, Twitter, Telegram, RSS, yt-dlp
├── testing/               # Chaos, E2E, perf regression
├── tests/                 # Benchmark, fuzz, integration
├── video/                 # Bilibili, Vimeo, YouTube, metadata, stream, offline
└── workflow/              # Engine, event-driven, scheduler, visualization
```

### 2.3 Redundancy Inventory

| Pattern | Locations | Count |
|---------|-----------|-------|
| Circuit breaker | `nt_act_circuit_breaker.rs`, `nt_infra_breaker.rs`, `nt_io_provider/circuit_breaker.rs`, `nt_repair/circuit_breaker.rs`, `media_source/reliability/circuit_breaker.rs` | 5 |
| Retry logic | `media_source/reliability/retry.rs`, `nt_world_crawl/resilient.rs`, `nt_io_provider` retry wrappers | 3+ |
| Caching | `media_cache/`, `engine/cache.rs`, `cost/cache_optimization.rs`, `nt_core_cache.rs`, `nt_core_deploy_cache.rs` | 5 |
| Rate limiting | `security/rate_limit.rs`, `engine/ratelimit.rs`, `nt_act_rate_limiter.rs` | 3 |
| Security/auth | `media_source/security/`, `integration/security_audit.rs`, `governance/`, `l3_embodiment/nt_shield/` | 4 domains |
| Observability | `media_source/observability/`, `l1_action/nt_infra_tracing.rs`, `nt_act/observability_stack.rs` | 3 |
| Health checks | `engine/health.rs`, `deployment/health_check.rs`, `core/nt_core_heartbeat.rs` | 3 |

### 2.4 Duplication: `unified/` vs Layer Directories

The `unified/layers/` directory is a **near-complete mirror** of the layer directories:

```
unified/layers/action/nt_act/   ←→  l1_action/nt_act/        (both ~160+ files)
unified/layers/action/nt_io/    ←→  l1_action/nt_io/
unified/layers/action/nt_memory/←→  l1_action/nt_memory/
unified/layers/cognition/       ←→  l5_cognition/
unified/layers/embodiment/      ←→  l3_embodiment/
unified/layers/emotion/         ←→  l4_emotion/
unified/layers/perception/      ←→  l2_perception/
unified/layers/meta/            ←→  l6_meta/
unified/core/                   ←→  core/
```

This duplication doubles maintenance cost and creates confusion about which is the source of truth.

---

## 3. Identified Defects

### 3.1 Flat Hierarchy (47 Top-Level Directories)

The `core/` directory contains **122 entries** (files + directories) at the top level. Many are `nt_core_*` prefixed but have no clear grouping:

```
core/
├── nt_core_absorb/
├── nt_core_accessor.rs
├── nt_core_agent_patterns.rs
├── nt_core_answer_engine.rs
├── ... (90+ more nt_core_* items)
├── l0_substrate/
├── l1_body/
├── ... (10 l* directories)
├── nt_game/
└── energy_core/
```

**Impact**: Developer cognitive load, unclear ownership, difficult navigation.

### 3.2 Cross-Domain Misalignment

Security concerns are scattered across 4 domains:

```
l1_action/nt_act/nt_act_security.rs          # Action domain
l3_embodiment/nt_shield/                      # Embodiment domain
media_source/security/                        # Perception domain
media_source/governance/                      # Perception domain (governance)
media_source/compliance/                      # Perception domain (compliance)
unified/layers/meta/nt_repair/circuit_breaker.rs  # Meta domain
```

The `governance` and `compliance` directories inside a perception module (`nt_world`) are conceptually wrong — governance is a meta-cognition concern, not a perception concern.

### 3.3 Circular Dependency Risk

```
nt_world_media_source/ecosystem/shield_integration.rs
  → imports from nt_shield (L3 Embodiment)
    → nt_shield imports from nt_world (L2 Perception)
      → nt_world contains nt_world_media_source
        → nt_world_media_source/ecosystem/shield_integration.rs  ← CYCLE
```

### 3.4 God Modules

| File | Lines | Responsibilities |
|------|-------|-----------------|
| `nt_core_consciousness_core.rs` | 3,214 | Consciousness orchestration, state management, cycle execution |
| `nt_memory_kb/mod.rs` | 2,929 | KB operations, graph operations, search, crawl, unify |
| `nt_core_mcp.rs` | 2,178 | MCP protocol, tool registration, execution |
| `nt_core_gate/mod.rs` | 2,149 | Gate logic, policy enforcement, routing |
| `nt_io_neocodex/agent.rs` | 1,899 | Agent loop, tool execution, state management |

---

## 4. Proposed 6-Layer Architecture

### 4.1 Target Layer Model

Aligns with NeoTrix's Six-Layer Architecture (L1-L6):

```
┌─────────────────────────────────────────────────────────────┐
│  L6 Meta-Cognition (元认知层)                                │
│  nt_meta | nt_repair | nt_nexus | governance | compliance   │
│  Cross-session memory, self-healing, policy enforcement     │
├─────────────────────────────────────────────────────────────┤
│  L5 Cognition (认知层)                                      │
│  nt_core | nt_mind                                          │
│  Reasoning, SEAL pipeline, knowledge distillation           │
├─────────────────────────────────────────────────────────────┤
│  L4 Emotion (情感层)                                        │
│  nt_feel                                                    │
│  Emotion engine, regulation, social emotion                 │
├─────────────────────────────────────────────────────────────┤
│  L3 Embodiment (具身层)                                     │
│  nt_physical | nt_shield | nt_feel(embodiment)              │
│  Safety kernel, security, power management                  │
├─────────────────────────────────────────────────────────────┤
│  L2 Perception (感知层)                                     │
│  nt_world | nt_sense                                        │
│  Crawling, media ingestion, sensory integration             │
├─────────────────────────────────────────────────────────────┤
│  L1 Action (行动层)                                         │
│  nt_act | nt_io | nt_memory                                 │
│  Tool execution, LLM providers, KB operations               │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Media Source Module Restructure

**Current** (flat, 42 dirs):
```
nt_world_media_source/
├── ai/ analytics/ api_gateway/ arch/ audio/ book/ cloud/
├── compliance/ cost/ deployment/ disaster_recovery/ document/
├── ecosystem/ edge/ engine/ evolution/ feed/ governance/
├── graph/ i18n/ image/ integration/ lyrics/ media_cache/
├── ml/ multitenancy/ observability/ performance/ pipeline/
├── plugin/ realtime/ reliability/ security/ serverless/
├── simulation/ social/ testing/ tests/ video/ workflow/
```

**Target** (layered, 8 domains):
```
nt_world_media_source/
├── domain/                    # Media type providers
│   ├── audio/                 # 12 providers
│   ├── video/                 # 6 providers
│   ├── image/                 # 4 providers
│   ├── document/              # 2 providers
│   ├── book/                  # 2 providers
│   ├── lyrics/                # 3 providers
│   ├── social/                # 6 providers
│   └── feed/                  # Feed engine
├── engine/                    # Core engine (slimmed)
│   ├── core.rs                # Routing, pool, health
│   ├── magika/                # Content type detection
│   └── types.rs               # Shared types
├── intelligence/              # AI/ML capabilities
│   ├── ai/                    # Embedding, NER, sentiment, RAG
│   ├── analytics/             # A/B, funnel, recommendation
│   ├── ml/                    # Vector index, online learning
│   └── graph/                 # Entity graph, timeline
├── pipeline/                  # Data flow
│   ├── etl.rs                 # Extract-transform-load
│   ├── lineage.rs             # Data lineage tracking
│   ├── quality.rs             # Quality gates
│   └── workflow/              # Event-driven, scheduler
├── platform/                  # Deployment & infrastructure
│   ├── api_gateway/           # REST, GraphQL, gRPC
│   ├── cloud/                 # CDN, routing
│   ├── deployment/            # Docker, config, health
│   ├── edge/                  # Device adapter, offline
│   ├── serverless/            # Cold start, orchestration
│   └── realtime/              # SSE, WebSocket
├── reliability/               # Fault tolerance (canonical)
│   ├── retry.rs               # Single retry implementation
│   ├── circuit_breaker.rs     # Single circuit breaker
│   ├── bulkhead.rs            # Bulkhead isolation
│   ├── degradation.rs         # Graceful degradation
│   └── resilience.rs          # Composed resilience patterns
├── intelligence_ops/          # Observability & operations
│   ├── observability/         # Logging, metrics, tracing
│   ├── performance/           # Optimization
│   ├── media_cache/           # Caching (canonical)
│   ├── cost/                  # Cost tracking
│   ├── multitenancy/          # Tenant isolation
│   └── simulation/            # Load testing, capacity
└── governance/                # Policy & compliance
    ├── security/              # Input validation, rate limit
    ├── compliance/            # GDPR, HIPAA, SOC2
    ├── audit.rs               # Audit trail
    └── integration/           # External system bridges
```

### 4.3 Module Ownership

| Domain | Owner | Layers | Modules |
|--------|-------|--------|---------|
| **domain/** | NT-WORLD | L2 | Audio, video, image, document, book, lyrics, social, feed providers |
| **engine/** | NT-WORLD | L2 | Core routing, pool, health, content detection |
| **intelligence/** | NT-MIND | L5 | AI/ML, analytics, graph, vector index |
| **pipeline/** | NT-ACT | L1 | ETL, lineage, quality, workflow |
| **platform/** | NT-IO | L1 | API gateway, cloud, deployment, edge, serverless |
| **reliability/** | NT-SHIELD | L3 | Retry, circuit breaker, bulkhead, degradation |
| **intelligence_ops/** | NT-META | L6 | Observability, performance, cache, cost, simulation |
| **governance/** | NT-GOVERNANCE | L6 | Security, compliance, audit, integration |

### 4.4 Cross-Domain Interfaces

```
                    ┌──────────────┐
                    │  L6 Meta     │
                    │  governance/ │
                    │  ops/        │
                    └──────┬───────┘
                           │ policy enforcement
                           ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  L5 Cognition│    │  L2 Percept. │    │  L3 Embodim. │
│  intelligence│◄───│  domain/     │───►│  reliability/ │
│  (AI/ML)     │    │  engine/     │    │  (fault tol) │
└──────┬───────┘    └──────┬───────┘    └──────────────┘
       │                   │
       ▼                   ▼
┌──────────────┐    ┌──────────────┐
│  L1 Action   │    │  L1 IO       │
│  pipeline/   │    │  platform/   │
│  (ETL/work)  │    │  (gateway)   │
└──────────────┘    └──────────────┘
```

**Dependency rules**:
- L2 (perception) may import from L1 (action/io) for data transport
- L3 (embodiment/reliability) is imported BY L2, not the reverse
- L5 (cognition/intelligence) imports from L2 via trait interfaces
- L6 (meta/governance) imports from all layers but exposes only traits
- No upward dependencies (L1 never imports L2, L2 never imports L3, etc.)

---

## 5. Key Design Decisions

### 5.1 Eliminate `unified/` Duplication

The `unified/layers/` directory is a full mirror of the layer structure. Decision: **delete `unified/`** and consolidate into the canonical `l1_action/` through `l6_meta/` directories. The `unified/` tree appears to be a migration artifact that was never completed.

### 5.2 Canonical Reliability Module

All retry, circuit breaker, and bulkhead implementations consolidate into:
```
l3_embodiment/nt_shield/reliability/
├── retry.rs          # Canonical retry
├── circuit_breaker.rs # Canonical circuit breaker
├── bulkhead.rs       # Canonical bulkhead
├── degradation.rs    # Graceful degradation
└── mod.rs            # Re-exports
```

Media source, crawl, and provider modules import from this single location.

### 5.3 Canonical Caching Module

All cache implementations consolidate into:
```
l1_action/nt_memory/cache/
├── multi_level.rs    # L1/L2/L3 cache hierarchy
├── consistency.rs    # Cache invalidation
├── warmup.rs         # Cache warming
└── mod.rs
```

### 5.4 Security Module Consolidation

Security concerns consolidate into `l3_embodiment/nt_shield/`:
- `nt_shield/security/` — input validation, rate limiting, threat detection
- `nt_shield/compliance/` — GDPR, HIPAA, SOC2
- `nt_shield/audit.rs` — audit trail

Remove `media_source/security/`, `media_source/governance/`, `media_source/compliance/`.

---

## 6. Migration Impact

| Metric | Current | Target | Delta |
|--------|---------|--------|-------|
| Media source subdirs | 42 | 8 domains × ~5 = 40 | -2 (net) |
| Circuit breaker files | 5 | 1 | -4 |
| Caching files | 5 | 1 | -4 |
| `unified/` files | 1,603 | 0 | -1,603 |
| Cross-domain security | 4 locations | 1 | -3 |
| Largest file | 3,214 lines | <1,000 lines | -69% |
| Top-level core entries | 122 | ~30 | -75% |

---

## 7. Constellation Maturity Target

| Phase | Constellation | Target |
|-------|--------------|--------|
| Phase 1: Consolidate reliability | C1 → C2 | Unit + integration tests for canonical reliability module |
| Phase 2: Flatten hierarchy | C0 → C1 | Compiles, no test regression |
| Phase 3: Fix cross-domain | C1 → C2 | Integration tests pass with new module boundaries |
| Phase 4: Extract cross-cutting | C1 → C2 | Observability/caching have own test suites |
| Phase 5: Domain boundaries | C2 → C3 | Benchmarked, documented interfaces |
| Final state | C4 | Integrated into SEAL pipeline, self-healing |

---

## 8. Risk Mitigation

1. **Build cache invalidation**: After structural changes, run `cargo clean` before `cargo check` (R-P9/R-P17/R-P29).
2. **Re-read verification**: After each edit, re-read the file to confirm persistence (R-P16).
3. **Zero unsafe**: All consolidated modules maintain `#![forbid(unsafe_code)]` (R-P1).
4. **Dark Forest rule**: Any module that doesn't compile + test + have consumers gets deleted.
5. **No dead code**: Every consolidated module must connect to a production path in the same session (R-P79).

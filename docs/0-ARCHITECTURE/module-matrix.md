# NeoTrix Media Source — Module Dependency Matrix

## 1. Module Classification

### 1.1 Core Modules (Foundation Layer)

These modules define types, traits, and the core engine. All other modules depend on them.

| Module | Path | Lines | Purpose | Depended On By |
|--------|------|-------|---------|----------------|
| `types` | `nt_world_media_source/types.rs` | ~200 | `MediaType`, `MediaQuery`, `MediaResult`, `ProviderConfig` | All media modules |
| `engine::core` | `nt_world_media_source/engine/mod.rs` | ~400 | `MediaEngine` trait, routing, pool management | All providers |
| `engine::magika` | `nt_world_media_source/engine/magika/` | ~600 | Content type detection (Magika integration) | Pipeline, providers |
| `api` | `nt_world_media_source/api.rs` | ~300 | Unified API surface (`search`, `resolve`, `metadata`) | CLI, web, MCP |
| `mod` | `nt_world_media_source/mod.rs` | ~150 | Module root, re-exports | All consumers |

### 1.2 Domain Modules (Media Providers)

Each provider implements `MediaProvider` trait from core.

| Module | Path | Providers | External APIs |
|--------|------|-----------|---------------|
| **audio** | `domain/audio/` | Spotify, NetEase, QQ Music, Kugou, Kuwo, Migu, SoundCloud, Bandcamp, Deezer, JioSaavn, Piped | 11 APIs |
| **video** | `domain/video/` | YouTube, Bilibili, Vimeo, yt-dlp, offline | 5 APIs |
| **image** | `domain/image/` | Unsplash, Pexels, Pixabay, Wikimedia | 4 APIs |
| **document** | `domain/document/` | ArXiv, Semantic Scholar | 2 APIs |
| **book** | `domain/book/` | Anna's Archive, OpenLibrary | 2 APIs |
| **lyrics** | `domain/lyrics/` | Genius, LRCLib, multi-source | 3 APIs |
| **social** | `domain/social/` | Twitter/X, Instagram, TikTok, Telegram, RSS bridge | 5 APIs |
| **feed** | `domain/feed/` | RSS/Atom feed engine, dedup, scoring | Self-contained |

### 1.3 Infrastructure Modules

| Module | Path | Purpose | Consumers |
|--------|------|---------|-----------|
| **reliability** | `l3_embodiment/nt_shield/reliability/` | Retry, circuit breaker, bulkhead, degradation | All providers, crawl, engine |
| **observability** | `l1_action/nt_act/observability/` | Logging, metrics, tracing, alerting | All modules |
| **security** | `l3_embodiment/nt_shield/security/` | Input validation, rate limiting, threat detection | API gateway, providers |
| **caching** | `l1_action/nt_memory/cache/` | Multi-level cache, invalidation, warming | Engine, providers, pipeline |

### 1.4 Integration Modules

| Module | Path | Purpose | Dependencies |
|--------|------|---------|--------------|
| **ecosystem** | `intelligence/media_ecosystem/` | E8 integration, KB sync, evolution signals | nt_core, nt_shield |
| **evolution** | `intelligence/media_evolution/` | Adaptive quality, self-healing, SkillGlow | nt_mind, nt_core_self |
| **ml** | `intelligence/ml/` | Vector index, embedding manager, A/B framework | nt_core_knowledge |
| **edge** | `platform/edge/` | Device adapter, lightweight inference, offline | nt_physical |
| **serverless** | `platform/serverless/` | Cold start, deployment, orchestration | nt_io |
| **cloud** | `platform/cloud/` | CDN, routing, abstraction | nt_io |

### 1.5 Cross-Cutting Concerns

| Module | Path | Purpose | Applied To |
|--------|------|---------|------------|
| **i18n** | `governance/i18n/` | Locale, RTL, localization | All UI-facing modules |
| **graph** | `intelligence/graph/` | Entity graph, backlinks, timeline | Feed, social, document |
| **analytics** | `intelligence/analytics/` | A/B testing, funnel, recommendation | Feed, social |
| **cost** | `intelligence_ops/cost/` | API tracking, resource reports | All providers |
| **multitenancy** | `intelligence_ops/multitenancy/` | Tenant isolation, billing | API gateway |
| **compliance** | `governance/compliance/` | GDPR, HIPAA, SOC2 | All data-handling modules |

---

## 2. Dependency Matrix

### 2.1 Read Direction

Rows depend on columns. `●` = direct dependency, `○` = indirect (via trait), `空` = no dependency.

```
                    │ types │ engine │ reliability │ observability │ security │ caching │ nt_core │ nt_shield │ nt_io │ nt_memory │
────────────────────┼───────┼────────┼─────────────┼───────────────┼──────────┼─────────┼─────────┼───────────┼───────┼───────────┤
domain/audio        │   ●   │   ●    │      ●      │       ○       │          │    ●    │         │           │       │           │
domain/video        │   ●   │   ●    │      ●      │       ○       │          │    ●    │         │           │       │           │
domain/image        │   ●   │   ●    │      ●      │       ○       │          │    ●    │         │           │       │           │
domain/document     │   ●   │   ●    │      ●      │       ○       │          │         │         │           │       │           │
domain/book         │   ●   │   ●    │      ●      │       ○       │          │         │         │           │       │           │
domain/lyrics       │   ●   │   ●    │      ●      │       ○       │          │    ●    │         │           │       │           │
domain/social       │   ●   │   ●    │      ●      │       ○       │     ●    │    ●    │         │           │       │           │
domain/feed         │   ●   │   ●    │      ●      │       ○       │          │    ●    │         │           │       │           │
intelligence/ai     │   ●   │        │             │       ○       │          │         │    ●    │           │       │           │
intelligence/ml     │   ●   │        │             │       ○       │          │         │    ●    │           │       │     ●     │
intelligence/graph  │   ●   │        │             │       ○       │          │         │    ●    │           │       │     ●     │
intelligence/analytics│ ●   │        │             │       ○       │          │         │    ●    │           │       │           │
pipeline            │   ●   │   ●    │      ●      │       ○       │          │         │         │           │       │     ●     │
platform/api_gateway│   ●   │   ●    │      ●      │       ○       │     ●    │    ●    │         │           │  ●    │           │
platform/cloud      │   ●   │   ●    │      ●      │       ○       │          │         │         │           │  ●    │           │
platform/edge       │   ●   │   ●    │      ●      │       ○       │          │         │         │     ●     │       │           │
platform/serverless │   ●   │   ●    │      ●      │       ○       │          │         │         │           │  ●    │           │
platform/realtime   │   ●   │   ●    │      ●      │       ○       │          │         │         │           │  ●    │           │
intelligence_ops/obs│   ●   │        │      ●      │       ●       │          │         │         │           │       │           │
intelligence_ops/perf│  ●   │   ●    │      ●      │       ○       │          │    ●    │         │           │       │           │
intelligence_ops/cost│  ●   │        │             │       ○       │          │         │    ●    │           │       │           │
intelligence_ops/mt │   ●   │        │             │       ○       │     ●    │         │         │           │       │           │
intelligence_ops/sim│   ●   │   ●    │      ●      │       ○       │          │         │    ●    │           │       │           │
governance/security │   ●   │        │      ●      │       ○       │     ●    │         │         │     ●     │       │           │
governance/compliance│  ●   │        │             │       ○       │     ●    │         │         │           │       │           │
governance/i18n     │   ●   │        │             │               │          │         │         │           │       │           │
ecosystem           │   ●   │        │      ●      │       ○       │          │         │    ●    │     ●     │       │           │
evolution           │   ●   │        │             │       ○       │          │         │    ●    │           │       │           │
```

### 2.2 Dependency Direction Summary

```
                  ┌─────────────────────────────────────────────────┐
                  │              L6 Meta-Cognition                  │
                  │  governance/ | intelligence_ops/ | evolution    │
                  └───────────────────────┬─────────────────────────┘
                                          │ imports (via traits)
                  ┌───────────────────────▼─────────────────────────┐
                  │              L5 Cognition                       │
                  │  intelligence/ai | intelligence/ml |            │
                  │  intelligence/graph | intelligence/analytics    │
                  └───────────────────────┬─────────────────────────┘
                                          │ imports (via traits)
          ┌───────────────────────────────▼─────────────────────────┐
          │              L2 Perception                               │
          │  domain/* | engine/ | pipeline/                         │
          └───┬───────────────────────────────────────────┬─────────┘
              │ imports                                   │ imports
              ▼                                           ▼
┌─────────────────────┐                   ┌─────────────────────────┐
│   L3 Embodiment     │                   │   L1 Action / IO        │
│   reliability/      │◄──────────────────│   platform/*            │
│   security/         │                   │   caching/              │
└─────────────────────┘                   │   observability/        │
                                          └─────────────────────────┘
```

---

## 3. Circular Dependency Audit

### 3.1 Known Cycles

| Cycle | Path | Resolution |
|-------|------|------------|
| **shield ↔ world** | `nt_shield` → `nt_world` (import) → `media_source/ecosystem/shield_integration` → `nt_shield` | Event-based decoupling (Phase 5) |
| **mind ↔ world** | `nt_mind` → `nt_world` (import) → `media_source/evolution` → `nt_mind` | Move evolution to `nt_mind` (Phase 3) |
| **core ↔ act** | `nt_core` → `nt_act` (some types) → `nt_core` (via traits) | Acceptable (trait-based) |

### 3.2 Cycle-Free Modules

These modules have no known circular dependencies:
- `types.rs` — pure data definitions
- `engine/magika/` — content detection, no cross-module imports
- `governance/i18n/` — locale utilities
- `domain/book/` — standalone provider implementations
- `domain/document/` — standalone provider implementations

---

## 4. Module Size Distribution

### 4.1 By File Count

| Category | Files | % of Total |
|----------|-------|-----------|
| domain/ (providers) | 48 | 21.5% |
| engine/ | 12 | 5.4% |
| intelligence/ (AI/ML) | 14 | 6.3% |
| pipeline/ | 4 | 1.8% |
| platform/ (deployment) | 18 | 8.1% |
| reliability/ | 5 | 2.2% |
| intelligence_ops/ | 16 | 7.2% |
| governance/ | 14 | 6.3% |
| ecosystem/ | 5 | 2.2% |
| evolution/ | 8 | 3.6% |
| tests/ | 4 | 1.8% |
| other (root files) | 75 | 33.6% |
| **Total** | **223** | **100%** |

### 4.2 By Line Count (Top 15)

| File | Lines | Category |
|------|-------|----------|
| `engine/media_engine.rs` | ~800 | Engine |
| `audio/spotify.rs` | ~450 | Domain |
| `api.rs` | ~300 | Core |
| `feed/engine.rs` | ~350 | Domain |
| `social/twitter.rs` | ~400 | Domain |
| `observability/metrics.rs` | ~250 | Cross-cutting |
| `reliability/circuit_breaker.rs` | ~200 | Infrastructure |
| `pipeline/etl.rs` | ~300 | Pipeline |
| `ml/vector_index.rs` | ~350 | Intelligence |
| `graph/entity_graph.rs` | ~280 | Intelligence |
| `security/input_validation.rs` | ~200 | Governance |
| `compliance/gdpr.rs` | ~250 | Governance |
| `cost/api_tracking.rs` | ~180 | Intelligence ops |
| `multitenancy/tenant_isolation.rs` | ~220 | Intelligence ops |
| `edge/device_adapter.rs` | ~200 | Platform |

---

## 5. Import Heatmap

### 5.1 Most-Imported Modules

```
1. types.rs              ████████████████████ 100% of modules
2. engine/mod.rs         ███████████████░░░░░  78% of modules
3. reliability/retry.rs  ██████████████░░░░░░  72% of modules
4. caching/mod.rs        ████████████░░░░░░░░  62% of modules
5. observability/metrics ██████████░░░░░░░░░░  54% of modules
6. security/rate_limit.rs████████░░░░░░░░░░░░  42% of modules
7. api.rs                ███████░░░░░░░░░░░░░  38% of modules
8. engine/magika/         █████░░░░░░░░░░░░░░  28% of modules
```

### 5.2 Most-Depended-On External Modules

```
1. nt_core (reasoning)   ████████████████░░░░  82% of intelligence modules
2. nt_shield (security)  ████████████░░░░░░░░  58% of platform modules
3. nt_io (providers)     ██████████░░░░░░░░░░  48% of platform modules
4. nt_memory (KB)        ████████░░░░░░░░░░░░  42% of intelligence modules
5. nt_mind (evolution)   ██████░░░░░░░░░░░░░░  32% of evolution modules
```

---

## 6. Interface Contracts

### 6.1 Core Trait: `MediaProvider`

```rust
// Required by: all domain/* modules
pub trait MediaProvider: Send + Sync + Debug {
    fn name(&self) -> &str;
    fn supported_types(&self) -> &[MediaType];
    async fn search(&self, query: &MediaQuery) -> Result<Vec<MediaResult>>;
    async fn resolve(&self, id: &str) -> Result<MediaDetail>;
    async fn metadata(&self, url: &str) -> Result<MediaMetadata>;
}
```

### 6.2 Infrastructure Trait: `ReliabilityPolicy`

```rust
// Required by: all modules that make external calls
pub trait ReliabilityPolicy: Send + Sync {
    fn retry_policy(&self) -> &RetryPolicy;
    fn circuit_breaker(&self) -> &CircuitBreaker;
    fn bulkhead(&self) -> Option<&Bulkhead>;
}
```

### 6.3 Intelligence Trait: `Analyzer`

```rust
// Required by: intelligence/* modules
pub trait Analyzer: Send + Sync {
    fn analyze(&self, content: &MediaContent) -> Result<AnalysisResult>;
    fn confidence(&self) -> f64;
}
```

### 6.4 Platform Trait: `DeploymentTarget`

```rust
// Required by: platform/* modules
pub trait DeploymentTarget: Send + Sync {
    fn deploy(&self, config: &DeployConfig) -> Result<DeployResult>;
    fn health_check(&self) -> HealthStatus;
    fn scale(&self, replicas: u32) -> Result<()>;
}
```

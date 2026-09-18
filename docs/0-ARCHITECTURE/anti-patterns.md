# NeoTrix Media Source — Anti-Patterns Analysis

## 1. God Module

### 1.1 `nt_core_consciousness_core.rs` (3,214 lines)

**Symptoms**: Single file handles consciousness orchestration, state management, cycle execution, and cross-domain health tracking.

**Responsibilities Found**:
- `ConsciousnessState` management (CRUD, serialization)
- Growth cycle execution (6-stage feedback loop)
- Cross-domain health aggregation
- GWT attention modulation
- Phi (IIT) calculation triggers
- Self-test integration
- Event bus coordination
- Telemetry emission

**Impact**:
- Change in cycle logic requires recompiling 3,214 lines
- No independent testability of sub-concerns
- Merge conflicts guaranteed on any consciousness-related work

**Fix**: Split into:
```
consciousness/
├── state.rs              # ConsciousnessState (CRUD, ser/de)        ~300 lines
├── cycle.rs              # Growth cycle execution                   ~400 lines
├── health.rs             # Cross-domain health aggregation          ~300 lines
├── attention.rs          # GWT attention modulation                 ~250 lines
├── phi.rs                # IIT phi calculation                      ~200 lines
├── self_test.rs          # Self-test integration                    ~200 lines
└── mod.rs                # Re-exports, orchestration                ~150 lines
```

### 1.2 `nt_memory_kb/mod.rs` (2,929 lines)

**Symptoms**: KB operations, graph operations, search, crawl, and unify all in one file.

**Responsibilities Found**:
- KB CRUD operations
- Graph traversal (entity/relation)
- FTS5 search
- Embedding management
- Crawl ingestion pipeline
- Knowledge unification
- Confidence scoring
- Community ingestion
- Resource ingestion
- Geo-spatial queries

**Fix**: Split into:
```
kb/
├── operations.rs         # CRUD operations                          ~400 lines
├── graph.rs              # Entity/relation graph                    ~350 lines
├── search.rs             # FTS5 search                              ~300 lines
├── embedding.rs          # Vector embeddings                        ~250 lines
├── ingest.rs             # Crawl/resource ingestion                 ~400 lines
├── unify.rs              # Knowledge unification                    ~300 lines
├── confidence.rs         # Confidence scoring                       ~250 lines
├── geo.rs                # Geo-spatial queries                      ~200 lines
└── mod.rs                # Re-exports                               ~100 lines
```

### 1.3 `nt_core_mcp.rs` (2,178 lines)

**Symptoms**: MCP protocol handling, tool registration, execution, and state management in one file.

**Fix**: Split into:
```
mcp/
├── protocol.rs           # MCP protocol types                       ~300 lines
├── registry.rs           # Tool registration                        ~400 lines
├── execution.rs          # Tool execution engine                    ~500 lines
├── state.rs              # Session state management                 ~300 lines
└── mod.rs                # Re-exports, routing                      ~200 lines
```

### 1.4 `nt_core_gate/mod.rs` (2,149 lines)

**Symptoms**: Gate logic, policy enforcement, and routing all in one file.

**Fix**: Split into:
```
gate/
├── policy.rs             # Policy definitions                       ~400 lines
├── enforcement.rs        # Policy enforcement engine                ~500 lines
├── routing.rs            # Gate routing logic                       ~300 lines
└── mod.rs                # Re-exports                               ~200 lines
```

---

## 2. Circular Dependencies

### 2.1 Shield ↔ World Cycle

**Path**:
```
nt_shield (l3_embodiment)
    → nt_world (l2_perception)        [imports types]
        → nt_world_media_source
            → ecosystem/shield_integration.rs
                → nt_shield           [imports shield types]  ← CYCLE
```

**Evidence**: `media_source/ecosystem/shield_integration.rs` imports from `nt_shield`, while `nt_shield` imports from `nt_world` (which contains `media_source`).

**Impact**:
- Rust compiler cannot determine compilation order
- Forces `#[cfg(test)]` hacks or `lazy_static` workarounds
- Prevents independent module testing

**Fix**: Introduce event-based decoupling:
```rust
// nt_world_media_source publishes event (no import from nt_shield)
EventBus::publish(MediaIngested { url, content_type, risk_score });

// nt_shield subscribes (no import from nt_world)
impl EventHandler<MediaIngested> for ShieldAuditor {
    fn handle(&self, event: &MediaIngested) {
        if event.risk_score > self.threshold {
            self.block(&event.url);
        }
    }
}
```

### 2.2 Mind ↔ World Cycle

**Path**:
```
nt_mind (l5_cognition)
    → nt_world (l2_perception)
        → nt_world_media_source
            → evolution/
                → nt_mind            ← CYCLE
```

**Fix**: Move `media_source/evolution/` into `l5_cognition/nt_mind/media_evolution/`.

### 2.3 Core ↔ Act Soft Cycle

**Path**:
```
nt_core (l5_cognition)
    → nt_act (l1_action)             [some type imports]
        → nt_core                    [via trait imports]  ← SOFT CYCLE
```

**Severity**: Low (trait-based, Rust allows this via orphan rules). Monitor but do not refactor unless it causes compilation issues.

---

## 3. Feature Envy

### 3.1 Media Engine Accessing Provider Internals

**Location**: `engine/media_engine.rs`

**Symptoms**:
```rust
impl MediaEngine {
    async fn search(&self, query: &MediaQuery) -> Result<Vec<MediaResult>> {
        // Accesses provider.rate_limiter directly
        // Accesses provider.cache_backend directly
        // Accesses provider.circuit_breaker directly
        // Should only call provider.search()
    }
}
```

**Evidence**: `engine/media_engine.rs` imports from `reliability/`, `caching/`, and `observability/` directly, rather than letting providers manage their own infrastructure.

**Fix**: Providers should own their infrastructure:
```rust
struct AudioProvider {
    reliability: ReliabilityPolicy,  // Provider owns this
    cache: CacheBackend,             // Provider owns this
    client: HttpClient,
}

impl MediaProvider for AudioProvider {
    async fn search(&self, query: &MediaQuery) -> Result<Vec<MediaResult>> {
        self.reliability.execute(|| self.client.search(query)).await
    }
}
```

### 3.2 Feed Engine Accessing Dedup Internals

**Location**: `feed/engine.rs`

**Symptoms**:
```rust
impl FeedEngine {
    async fn process(&mut self, item: FeedItem) -> Result<()> {
        // Directly accesses dedup.bloom_filter.bits
        // Directly accesses dedup.seen_hashes
        // Should call dedup.is_duplicate()
    }
}
```

**Fix**: Encapsulate dedup behind trait:
```rust
trait Deduplicator {
    fn is_duplicate(&self, item: &FeedItem) -> bool;
    fn mark_seen(&mut self, item: &FeedItem);
}
```

### 3.3 Pipeline Accessing Provider Config

**Location**: `pipeline/etl.rs`

**Symptoms**:
```rust
impl EtlPipeline {
    async fn extract(&self, source: &str) -> Result<Data> {
        // Reads provider.config.api_key directly
        // Reads provider.config.rate_limit directly
        // Should call provider.extract()
    }
}
```

---

## 4. Data Clumps

### 4.1 Query Parameters Clump

**Repeated combination**: `(query: &str, media_type: MediaType, limit: u32, offset: u32)`

**Found in**:
- `domain/audio/spotify.rs` — `search()`
- `domain/audio/netease.rs` — `search()`
- `domain/video/youtube.rs` — `search()`
- `domain/image/unsplash.rs` — `search()`
- `domain/social/twitter.rs` — `search()`
- `engine/media_engine.rs` — `search()`
- `pipeline/etl.rs` — `extract()`

**Count**: 7+ locations

**Fix**: Extract into `MediaQuery` type (already exists in `types.rs` but not used everywhere):
```rust
pub struct MediaQuery {
    pub query: String,
    pub media_type: MediaType,
    pub limit: u32,
    pub offset: u32,
    pub filters: Option<QueryFilters>,
}
```

Ensure all providers accept `MediaQuery` rather than individual parameters.

### 4.2 Auth Credentials Clump

**Repeated combination**: `(api_key: String, api_secret: Option<String>, token: Option<String>)`

**Found in**:
- `domain/audio/spotify.rs`
- `domain/audio/netease.rs`
- `domain/video/youtube.rs`
- `domain/social/twitter.rs`
- `domain/social/instagram.rs`
- `platform/api_gateway/rest.rs`

**Fix**: Extract into `Credentials` type:
```rust
pub enum Credentials {
    ApiKey(String),
    OAuth2 { token: String, refresh: Option<String> },
    Basic { username: String, password: String },
    None,
}
```

### 4.3 Rate Limit Config Clump

**Repeated combination**: `(requests_per_second: u32, burst: u32, cooldown: Duration)`

**Found in**:
- `reliability/retry.rs`
- `security/rate_limit.rs`
- `engine/ratelimit.rs`
- 5+ provider config structs

**Fix**: Extract into `RateLimitConfig`:
```rust
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst: u32,
    pub cooldown: Duration,
}
```

### 4.4 Error Context Clump

**Repeated combination**: `(module: &str, operation: &str, details: String)`

**Found in**: Every provider's error handling.

**Fix**: Use `thiserror` derive or extract `ErrorContext`:
```rust
pub struct ErrorContext {
    pub module: &'static str,
    pub operation: &'static str,
    pub details: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

---

## 5. Primitive Obsession

### 5.1 String-Based Media Types

**Current** (in `types.rs`):
```rust
pub struct MediaResult {
    pub url: String,           // Should be Url type
    pub media_type: String,    // Should be MediaType enum
    pub provider: String,      // Should be ProviderId type
    pub title: Option<String>, // Acceptable
}
```

**Problems**:
- `media_type: String` allows invalid values like `"audoi"` (typo)
- `url: String` doesn't validate URL format
- `provider: String` allows any string, no compile-time safety

**Fix**:
```rust
pub struct MediaResult {
    pub url: url::Url,           // Type-safe URL
    pub media_type: MediaType,   // Enum: Audio, Video, Image, etc.
    pub provider: ProviderId,    // Newtype: ProviderId(String)
    pub title: Option<String>,
}

pub enum MediaType {
    Audio,
    Video,
    Image,
    Document,
    Book,
    Lyrics,
    Social,
    Feed,
}

pub struct ProviderId(String);  // Newtype with validation
```

### 5.2 String-Based Provider Names

**Current**:
```rust
fn get_provider(name: &str) -> Box<dyn MediaProvider> {
    match name {
        "spotify" => Box::new(SpotifyProvider::new()),
        "netease" => Box::new(NeteaseProvider::new()),
        // 15+ more string matches
    }
}
```

**Fix**: Use enum:
```rust
pub enum ProviderKind {
    Spotify,
    Netease,
    QQMusic,
    // ...
}

impl ProviderKind {
    fn from_str(s: &str) -> Result<Self> { /* ... */ }
    fn create(&self, config: &ProviderConfig) -> Box<dyn MediaProvider> { /* ... */ }
}
```

### 5.3 String-Based Error Codes

**Current**:
```rust
return Err(format!("{}: {} failed: {}", module, operation, details));
```

**Fix**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("Provider {0} not found")]
    ProviderNotFound(String),
    #[error("Rate limited by {0}, retry after {1:?}")]
    RateLimited(String, Duration),
    #[error("Content detection failed for {0}")]
    DetectionFailed(String),
    // ...
}
```

### 5.4 Raw String URLs

**Current** (in provider configs):
```rust
struct ProviderConfig {
    search_url: String,      // "https://api.spotify.com/v1/search"
    detail_url: String,      // "https://api.spotify.com/v1/tracks/{id}"
    auth_url: String,        // "https://accounts.spotify.com/api/token"
}
```

**Fix**: Use typed URL templates:
```rust
struct ProviderConfig {
    search_url: UrlTemplate,  // "https://api.spotify.com/v1/search"
    detail_url: UrlTemplate,  // "https://api.spotify.com/v1/tracks/{id}"
    auth_url: UrlTemplate,    // "https://accounts.spotify.com/api/token"
}

impl UrlTemplate {
    pub fn fill(&self, params: &[(&str, &str)]) -> Result<Url> { /* ... */ }
}
```

---

## 6. Additional Anti-Patterns

### 6.1 Shotgun Surgery

**Symptom**: Adding a new provider requires changes in 5+ files:
1. Create provider file in `domain/`
2. Add to `engine/mod.rs` provider registry
3. Add to `api.rs` search routing
4. Add to `types.rs` MediaType handling
5. Add to `pipeline/etl.rs` extraction
6. Add tests in `tests/`

**Fix**: Use registry pattern:
```rust
// Provider self-registration
inventory::submit! {
    ProviderRegistration {
        kind: ProviderKind::Spotify,
        factory: || Box::new(SpotifyProvider::new()),
        supported_types: &[MediaType::Audio],
    }
}

// Auto-discovery
fn all_providers() -> Vec<Box<dyn MediaProvider>> {
    inventory::iter::<ProviderRegistration>
        .map(|reg| (reg.factory)())
        .collect()
}
```

### 6.2 Divergent Change

**Symptom**: `engine/media_engine.rs` changes for 3 unrelated reasons:
1. Adding new provider support
2. Changing retry logic
3. Modifying cache behavior

**Fix**: Separate concerns:
- Provider routing → `engine/router.rs`
- Retry logic → `reliability/` (external)
- Cache behavior → `caching/` (external)

### 6.3 Parallel Inheritance

**Symptom**: Multiple provider hierarchies with parallel structures:
```
AudioProvider → SpotifyProvider, NeteaseProvider
VideoProvider → YouTubeProvider, BilibiliProvider
ImageProvider → UnsplashProvider, PexelsProvider
```

Each has identical retry/cache/auth logic.

**Fix**: Use composition over inheritance:
```rust
struct Provider {
    inner: Box<dyn MediaProvider>,
    reliability: ReliabilityPolicy,
    cache: CacheBackend,
    auth: AuthManager,
}
```

### 6.4 Speculative Generality

**Symptom**: Modules exist for features not yet implemented:
- `serverless/` — no actual serverless deployment in use
- `edge/` — no actual edge devices connected
- `multitenancy/` — single-tenant deployment only
- `ml/online_learning.rs` — no online learning pipeline active

**Fix**: Apply Dark Forest rule — modules without consumers get deleted. Re-implement when needed.

### 6.5 Lazy Class

**Symptom**: Thin wrapper modules that add no value:
- `arch/mod.rs` — empty architecture markers
- `now_playing.rs` — 10-line wrapper
- `playback.rs` — 15-line wrapper
- `resource_store.rs` — 20-line wrapper

**Fix**: Inline into consumers or delete.

---

## 7. Summary Matrix

| Anti-Pattern | Severity | Instances | Fix Phase |
|-------------|----------|-----------|-----------|
| God Module | **Critical** | 4 files (3,214; 2,929; 2,178; 2,149 lines) | Phase 2 |
| Circular Dependency | **High** | 2 cycles (shield↔world, mind↔world) | Phase 3, 5 |
| Feature Envy | **Medium** | 3 locations | Phase 1 |
| Data Clumps | **Medium** | 4 clumps (7+ locations each) | Phase 1, 4 |
| Primitive Obsession | **Medium** | 4 categories | Phase 1, 5 |
| Shotgun Surgery | **Low** | 1 (provider registration) | Phase 5 |
| Divergent Change | **Low** | 1 (media_engine.rs) | Phase 2 |
| Parallel Inheritance | **Low** | 3 provider hierarchies | Phase 1 |
| Speculative Generality | **Low** | 5 modules | Phase 2 |
| Lazy Class | **Low** | 4 modules | Phase 2 |

---

## 8. Prioritized Remediation

### Priority 1 (Phase 1): Quick Wins
- [ ] Extract `MediaQuery` data clump into consistent usage
- [ ] Extract `Credentials` data clump
- [ ] Replace string-based `MediaType` with enum
- [ ] Replace string-based `ProviderId` with newtype

### Priority 2 (Phase 2): Structural
- [ ] Split `nt_core_consciousness_core.rs` (3,214 lines)
- [ ] Split `nt_memory_kb/mod.rs` (2,929 lines)
- [ ] Split `nt_core_mcp.rs` (2,178 lines)
- [ ] Split `nt_core_gate/mod.rs` (2,149 lines)
- [ ] Delete speculative generality modules

### Priority 3 (Phase 3): Dependencies
- [ ] Break shield↔world cycle via events
- [ ] Break mind↔world cycle via relocation
- [ ] Move governance to L6

### Priority 4 (Phase 5): Long-Term
- [ ] Implement provider self-registration
- [ ] Enforce layer dependency rules
- [ ] Delete `unified/` directory

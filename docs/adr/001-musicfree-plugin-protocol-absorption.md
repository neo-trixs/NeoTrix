# ADR-001: Absorb MusicFree Plugin Protocol as MediaSource Adapters

- **Status**: Accepted
- **Date**: 2026-09-07
- **Decisioners**: NeoTrix Core Team
- **Relates to**: NT-WORLD domain, L2 Perception Layer

## Context

MusicFree (26.7K stars, `maotoumao/MusicFree`) is a plugin-based, ad-free music player where **all music sources come from third-party CommonJS plugins**. The player itself contains zero music source code — it defines a plugin protocol (`IPluginDefine`) and delegates all source-specific logic to community plugins.

**Why absorb this protocol into NeoTrix?**

1. **Multi-source music perception**: NT-WORLD already crawls the web (NT-WORLD-CRAWL), searches (NT-WORLD-SEARCH), and processes social signals (NT-WORLD-SOCIAL-INTEL). Adding structured music source access completes the media perception layer.

2. **Plugin ecosystem leverage**: MusicFree's plugin registry (`plugins.json`) lists 20+ community-maintained plugins covering Bilibili, YouTube, Audiomack, and others. Absorbing the protocol gives NeoTrix instant access to this ecosystem.

3. **Pattern alignment**: The adapter pattern (`SocialSource` trait in `nt_world_social_intel`) is the proven NT-WORLD integration model. MusicFree's plugin protocol maps cleanly to this pattern.

4. **Data pipeline enrichment**: Music metadata (lyrics, album info, artist works) feeds into NT-MEMORY knowledge graphs and NT-MIND evolution cycles (e.g., trend analysis, content recommendation).

### MusicFree Plugin Protocol Summary

Each plugin is a CommonJS module implementing `IPluginDefine`:

| Method | Signature | Purpose |
|--------|-----------|---------|
| `search` | `(query, page, type) → {isEnd, data[]}` | Search music/albums/artists/sheets |
| `getMediaSource` | `(musicItem, quality) → {url, headers, userAgent}` | Get playable URL |
| `getMusicInfo` | `(musicBase) → Partial<IMusicItem>` | Fetch full track metadata |
| `getLyric` | `(musicItem) → {lrc, rawLrc}` | Fetch lyrics (LRC format) |
| `getAlbumInfo` | `(albumItem, page) → {data[], isEnd}` | Album track listing |
| `getArtistWorks` | `(artistItem, page, type) → {data[], isEnd}` | Artist discography |
| `importMusicSheet` | `(urlLike) → IMusicItem[]` | Import playlist from URL |
| `getTopLists` | `() → IMusicTopListGroupItem[]` | Platform charts |
| `getTopListDetail` | `(topListItem) → WithMusicList` | Chart detail + tracks |

**Key data types** (from `types/plugin.d.ts`):
- `IMusicItem`: `{id, platform, title, artist, album, artwork, duration, [custom]}`
- `IAlbumItem`: `{id, platform, title, artist, artwork, description}`
- `IArtistItem`: `{id, platform, name, avatar, description}`
- `IMusicSheetItem`: `{id, title, coverImg, description}`

### Existing NT-WORLD Patterns

| Pattern | Module | Key Trait/Type |
|---------|--------|----------------|
| SocialSource adapter | `nt_world_social_intel/source.rs` | `trait SocialSource: Send + Sync` |
| SocialIntelEngine | `nt_world_social_intel/engine.rs` | `SocialIntelEngine { sources: Vec<Box<dyn SocialSource>> }` |
| EventBus bridge | `nt_world_social_intel/event_bridge.rs` | `SocialSignalEvent` → EventBus topic |
| OrderedBackendRouter | `nt_world_crawl/ordered_backend_router/mod.rs` | `OrderedBackendRouter` with health cache + fallback chain |
| Module registration | `nt_world/mod.rs` | `pub mod nt_world_*;` declarations |

## Decision

### Create `nt_world_media_source/` module

New module at `neotrix-core/src/unified/layers/perception/nt_world/nt_world_media_source/` following the SocialSource adapter pattern.

### MediaSource trait design

```rust
// nt_world_media_source/source.rs

/// Music source platform identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MusicPlatform {
    Bilibili,
    YouTube,
    Audiomack,
    WebDAV,
    MaoerFM,
    Yinyuetai,
    Custom(String),  // extensible for community plugins
}

/// Source health (mirrors SocialSource::SourceHealth)
#[derive(Debug, Clone)]
pub struct MediaSourceHealth {
    pub healthy: bool,
    pub message: Option<String>,
    pub last_query: Option<i64>,
    pub total_queries: u64,
    pub error_rate: f64,
}

/// Source capabilities declaration
#[derive(Debug, Clone, Default)]
pub struct MediaSourceCapabilities {
    pub can_search: bool,
    pub can_get_media_source: bool,
    pub can_get_lyrics: bool,
    pub can_get_album_info: bool,
    pub can_get_artist_works: bool,
    pub can_import_sheet: bool,
    pub can_get_top_lists: bool,
    pub supports_quality: Vec<QualityLevel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLevel {
    Low,
    Standard,
    High,
    Super,
}

/// Music data types (MusicFree-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicItem {
    pub id: String,
    pub platform: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub artwork: Option<String>,
    pub duration: Option<u32>,
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumItem {
    pub id: String,
    pub platform: String,
    pub title: String,
    pub artist: Option<String>,
    pub artwork: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistItem {
    pub id: String,
    pub platform: String,
    pub name: String,
    pub avatar: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSourceResult {
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub user_agent: Option<String>,
    pub quality: Option<QualityLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricSource {
    pub lrc: Option<String>,
    pub raw_lrc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub is_end: bool,
    pub data: Vec<T>,
}

/// MusicSource trait — platform adapter interface
///
/// Mirrors SocialSource pattern: each music platform implements this trait.
/// Plugins are loaded as CommonJS modules and wrapped in this trait.
pub trait MusicSource: Send + Sync {
    fn platform(&self) -> MusicPlatform;
    fn name(&self) -> &str;
    fn health_check(&self) -> MediaSourceHealth;
    fn capabilities(&self) -> MediaSourceCapabilities;

    // Core methods (all return Result for error handling)
    fn search(
        &self,
        query: &str,
        page: u32,
        media_type: &str,  // "music" | "album" | "artist" | "sheet"
    ) -> Result<SearchResult<MusicItem>, String>;

    fn get_media_source(
        &self,
        music_item: &MusicItem,
        quality: QualityLevel,
    ) -> Result<MediaSourceResult, String>;

    // Optional methods (default no-op implementations)
    fn get_music_info(&self, _music_base: &MusicItem) -> Result<Option<MusicItem>, String> {
        Ok(None)
    }

    fn get_lyric(&self, _music_item: &MusicItem) -> Result<Option<LyricSource>, String> {
        Ok(None)
    }

    fn get_album_info(
        &self,
        _album_item: &AlbumItem,
        _page: u32,
    ) -> Result<SearchResult<MusicItem>, String> {
        Ok(SearchResult { is_end: true, data: vec![] })
    }

    fn get_artist_works(
        &self,
        _artist_item: &ArtistItem,
        _page: u32,
        _media_type: &str,
    ) -> Result<SearchResult<MusicItem>, String> {
        Ok(SearchResult { is_end: true, data: vec![] })
    }

    fn import_music_sheet(&self, _url_like: &str) -> Result<Vec<MusicItem>, String> {
        Ok(vec![])
    }

    fn get_top_lists(&self) -> Result<Vec<MusicItem>, String> {
        Ok(vec![])
    }
}
```

### MediaSourceEngine — orchestration layer

```rust
// nt_world_media_source/engine.rs

/// Media source engine configuration
#[derive(Debug, Clone)]
pub struct MediaSourceConfig {
    /// Default quality preference
    pub default_quality: QualityLevel,
    /// Max search results per query
    pub max_results: u32,
    /// Cache TTL for search results (seconds)
    pub cache_ttl: u64,
    /// Enable EventBus integration
    pub publish_events: bool,
}

/// Media source engine — orchestrates multiple MusicSource adapters
///
/// Mirrors SocialIntelEngine pattern:
/// - Add sources via add_source()
/// - Query via unified search/get_media_source
/// - Publish events to EventBus for downstream consumption
pub struct MediaSourceEngine {
    sources: Vec<Box<dyn MusicSource>>,
    config: MediaSourceConfig,
}

impl MediaSourceEngine {
    pub fn new(config: MediaSourceConfig) -> Self { ... }

    /// Register a music source adapter
    pub fn add_source(&mut self, source: Box<dyn MusicSource>) { ... }

    /// List registered platforms
    pub fn platforms(&self) -> Vec<MusicPlatform> { ... }

    /// Health check all sources
    pub fn health_check(&self) -> Vec<(MusicPlatform, MediaSourceHealth)> { ... }

    /// Unified search across all sources (aggregated + deduplicated)
    pub fn search(
        &self,
        query: &str,
        page: u32,
        media_type: &str,
    ) -> Result<Vec<SearchResult<MusicItem>>, String> { ... }

    /// Get playable media source (tries sources in priority order)
    pub fn get_media_source(
        &self,
        music_item: &MusicItem,
        quality: QualityLevel,
    ) -> Result<MediaSourceResult, String> { ... }

    /// Get lyrics (aggregated from all sources)
    pub fn get_lyric(&self, music_item: &MusicItem) -> Result<Option<LyricSource>, String> { ... }

    /// Get album info (aggregated)
    pub fn get_album_info(
        &self,
        album_item: &AlbumItem,
        page: u32,
    ) -> Result<SearchResult<MusicItem>, String> { ... }

    /// Get top lists (aggregated)
    pub fn get_top_lists(&self) -> Result<Vec<MusicItem>, String> { ... }
}
```

### EventBus integration

```rust
// nt_world_media_source/event_bridge.rs

/// Music event type constant
pub const MUSIC_EVENT_TOPIC: &str = "media.music";

/// Music events published to EventBus
#[derive(Debug, Clone, Serialize)]
pub enum MusicEvent {
    /// Search completed — top results available
    SearchCompleted {
        platform: MusicPlatform,
        query: String,
        result_count: usize,
    },
    /// Media source resolved — playable URL found
    MediaSourceResolved {
        platform: MusicPlatform,
        music_id: String,
        quality: QualityLevel,
    },
    /// Lyrics fetched
    LyricsFetched {
        platform: MusicPlatform,
        music_id: String,
        has_lrc: bool,
    },
    /// Source error
    SourceError {
        platform: MusicPlatform,
        error: String,
    },
}

/// EventBus consumer trait for music events
pub trait MusicEventConsumer: Send + Sync {
    fn on_music_event(&self, event: &MusicEvent);
}
```

### Plugin loader — bridge to MusicFree CommonJS plugins

```rust
// nt_world_media_source/plugin_loader.rs

/// Load a MusicFree CommonJS plugin and wrap it as MusicSource
///
/// Uses deno_core (or similar JS runtime) to evaluate the plugin module,
/// then bridges the IPluginDefine methods to the MusicSource trait.
pub struct MusicFreePluginLoader {
    runtime: deno_core::JsRuntime,  // or equivalent
}

impl MusicFreePluginLoader {
    /// Load a plugin from a .js file path
    pub fn load_from_file(&mut self, path: &Path) -> Result<Box<dyn MusicSource>, String> { ... }

    /// Load a plugin from a URL (fetch + evaluate)
    pub fn load_from_url(&mut self, url: &str) -> Result<Box<dyn MusicSource>, String> { ... }

    /// Load all plugins from a registry JSON (plugins.json format)
    pub fn load_from_registry(
        &mut self,
        registry_url: &str,
    ) -> Result<Vec<Box<dyn MusicSource>>, String> { ... }
}
```

### File structure

```
neotrix-core/src/unified/layers/perception/nt_world/nt_world_media_source/
├── mod.rs                    # Module declaration + re-exports
├── source.rs                 # MusicSource trait + data types
├── engine.rs                 # MediaSourceEngine orchestration
├── event_bridge.rs           # EventBus integration
├── plugin_loader.rs          # MusicFree CommonJS → MusicSource bridge
├── adapters/                 # Built-in native adapters
│   ├── mod.rs
│   ├── http_direct.rs        # Direct HTTP source (no plugin needed)
│   └── mock.rs               # Test mock
└── types.rs                  # Shared type definitions
```

### Integration with existing modules

| Integration Point | Module | How |
|-------------------|--------|-----|
| **NT-WORLD mod.rs** | `nt_world/mod.rs` | Add `pub mod nt_world_media_source;` |
| **OrderedBackendRouter** | `nt_world_crawl/` | MediaSource engine uses router for HTTP fallback when no plugin available |
| **EventBus** | Global event bus | `MusicEvent` published under `media.music` topic, consumed by NT-ACT (player) and NT-MIND (evolution) |
| **KB embedding** | NT-MEMORY | Music metadata indexed into KB for cross-domain queries (e.g., "songs matching this mood") |
| **SocialIntelEngine** | `nt_world_social_intel/` | Parallel pattern — same adapter + engine + bridge structure |
| **UnifiedCrawler** | `nt_world_crawl/` | Plugins that wrap HTTP APIs can reuse crawl pipeline components |

## Alternatives Considered

### 1. Extend SocialSource directly

**Rejected.** SocialSource is designed for social messaging platforms (WeChat, Telegram, Slack). Its methods (`list_sessions`, `read_messages`, `search_messages`) don't map to music operations (`search`, `get_media_source`, `get_lyric`). Forcing music into this interface would violate Interface Segregation Principle and create confusing abstractions.

### 2. Use JS runtime directly (deno_core) without trait abstraction

**Rejected.** While deno_core can execute MusicFree plugins, skipping the `MusicSource` trait would:
- Lose type safety at the Rust boundary
- Prevent native Rust adapters (direct HTTP, mock tests)
- Make testing impossible without a JS runtime
- Violate R-P1 (zero unsafe) if FFI boundaries aren't carefully managed

The trait abstraction allows both JS plugins AND native Rust adapters to coexist.

### 3. Direct HTTP scraping without plugin protocol

**Rejected.** This would:
- Require maintaining per-platform scrapers (fragile, breaks on API changes)
- Miss the community-maintained plugin ecosystem
- Duplicate effort that MusicFree plugins already solve
- Violate R-P42 (absorption strengthens existing nodes, no parallel adapter modules)

### 4. Fork MusicFree's plugin runtime wholesale

**Rejected.** MusicFree's runtime includes React Native dependencies and Android-specific code. NeoTrix only needs the plugin protocol (CommonJS module → IPluginDefine), not the full player runtime. A minimal JS runtime (deno_core) with protocol-specific bridging is sufficient.

## Consequences

### Positive

- **Multi-source music perception**: NT-WORLD gains structured access to 20+ music platforms via community plugins
- **Plugin ecosystem compatibility**: Existing MusicFree plugins work with zero modification
- **Pattern consistency**: Follows proven SocialSource adapter + engine + bridge architecture
- **Testability**: `MusicSource` trait enables mock adapters for unit tests without JS runtime
- **EventBus integration**: Music events flow to NT-ACT (player control) and NT-MIND (trend evolution)
- **KB enrichment**: Music metadata feeds knowledge graph for cross-domain reasoning
- **R-P79 compliance**: Absorbed protocol immediately wired to production path (EventBus → downstream consumers)

### Negative / Risks

| Risk | Mitigation |
|------|-----------|
| JS runtime dependency (deno_core) | Feature-gated: `#[cfg(feature = "media-plugins")]`. Core compiles without it. |
| Plugin quality varies (community-maintained) | Health check + error rate tracking per source (mirrors SocialSource pattern) |
| Legal considerations (copyright data from plugins) | NeoTrix only provides the protocol bridge, not the plugins. Plugin data is user-managed. Follow MusicFree's own disclaimer pattern. |
| Maintenance burden (protocol drift) | Pin to MusicFree protocol version. `appVersion` field in plugins provides compatibility signals. |
| Performance (JS evaluation overhead) | Cache plugin module state. Only evaluate plugin methods, not full module reload per call. |

### Constellation Maturity Target

| Phase | Constellation | Criteria |
|-------|--------------|----------|
| Phase 1 | C0 → C1 | Module compiles + unit tests (mock adapter) |
| Phase 2 | C1 → C2 | Integration test with real MusicFree plugin (Bilibili) |
| Phase 3 | C2 → C3 | Benchmark: search latency < 500ms, media source resolve < 2s |
| Phase 4 | C3 → C4 | Integrated into NT-ACT player pipeline |
| Phase 5 | C4 → C5 | Self-healing: plugin failure → fallback to next source → health report |

## Implementation Plan

| Phase | Tasks | Files | Dependencies |
|-------|-------|-------|-------------|
| **Phase 1: Trait + Types** | Define `MusicSource` trait, data types, `MediaSourceEngine` skeleton | `source.rs`, `types.rs`, `engine.rs`, `mod.rs` | None |
| **Phase 2: EventBus Bridge** | `MusicEvent` enum, `MusicEventConsumer` trait, topic registration | `event_bridge.rs` | EventBus infrastructure (existing) |
| **Phase 3: Mock + Tests** | `MockMusicSource` adapter, unit tests for engine aggregation | `adapters/mock.rs`, engine tests | Phase 1 |
| **Phase 4: Plugin Loader** | deno_core integration, `MusicFreePluginLoader`, JS↔Rust bridge | `plugin_loader.rs` | deno_core dependency |
| **Phase 5: First Plugin** | Load Bilibili plugin as proof-of-concept, integration test | Test fixture + Bilibili plugin.js | Phase 4 |
| **Phase 6: Production Wiring** | Register in NT-WORLD mod.rs, NT-ACT player consumes MusicEvent, KB indexing | `nt_world/mod.rs`, NT-ACT modules | Phases 1-5 |

**Phase 1-3 can proceed immediately** (no external dependencies). Phase 4-5 require deno_core evaluation.

## Related Documentation

- MusicFree plugin protocol: `types/plugin.d.ts` in `maotoumao/MusicFreePlugins`
- SocialSource pattern: `nt_world_social_intel/source.rs:33`
- SocialIntelEngine pattern: `nt_world_social_intel/engine.rs:44`
- OrderedBackendRouter: `nt_world_crawl/ordered_backend_router/mod.rs:69`
- EventBus bridge pattern: `nt_world_social_intel/event_bridge.rs:1`
- ADR-001 (collective awareness): `docs/adr/ADR-001-collective-awareness.md`

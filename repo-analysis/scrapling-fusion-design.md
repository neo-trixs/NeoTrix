# Scrapling → NeoTrix 融合设计文档

> **Source**: D4Vinci/Scrapling (v0.4.8, 54.7k ⭐, BSD-3-Clause, Python 99.9%)
> **Target**: NeoTrix (Rust, reasoning-driven crawler at `neotrix-core/src/neotrix/crawler/`)
> **Date**: 2026-05-29
> **Status**: 融合设计 · 0/7 features absorbed

---

## 1. Panoramic Gap Matrix

### 1.1 Scrapling Has → NeoTrix Lacks

| Feature | Scrapling Approach | NeoTrix Gap | Impact | Priority |
|---------|--------------------|-------------|--------|----------|
| **Adaptive Element Tracking** | `adaptive=True` on CSS selectors auto-relocates via similarity; `auto_save=True` saves element context; `find_similar()` finds similar elements | No element-level tracking; crawler works at URL/page level only. `scraper.rs` is raw HTTP fetch → text dump | H | P0 (G-01) |
| **Checkpoint Spider** | Scrapy-like `start_urls` + `parse`; `crawldir="./crawl_data"` with full checkpoint/persist/resume; Ctrl+C graceful shutdown; resume on restart | P0-W01 exists in `pipeline.rs` as basic checkpoint, but lacks Scrapy-like callback architecture, auto-resume, streaming | H | P0 (G-02) |
| **Multi-Session Router** | `Fetcher` (HTTP/3, TLS fingerprint), `StealthyFetcher` (Cloudflare bypass), `DynamicFetcher` (Playwright); `configure_sessions()` routes URLs by type; session pool with stats | `fetcher.rs` has `FetcherPool` with HTTP + Browser modes but no stealthy/dynamic split, no URL-based routing, no pool stats | H | P1 (G-03) |
| **Proxy Rotator** | `ProxyRotator` with cyclic/custom strategies; per-request override; DNS-over-HTTPS leak prevention | `proxy_chain/` already has `DynamicProxyChain` with multi-hop + health probes + failover. Stronger than Scrapling here. But lacks simple cyclic rotation API for non-Tor users | M | P1 (G-04) |
| **MCP Scraping Server** | Built-in MCP server for AI-assisted scraping; pre-extracts content before passing to Claude/Cursor | `mcp_tools.rs` has `web_scrape` tool but no dedicated scraping MCP server with auto-selector generation | M | P2 (G-05) |
| **CLI Scraping Shell** | `scrapling shell` with IPython; `scrapling extract` commands | No equivalent CLI interactive mode | L | P2 (G-06) |
| **robots.txt Compliance** | `robots_txt_obey` flag | `CrawlerConfig.respect_robots_txt` exists but unused (no parser) | L | P2 (G-07) |

### 1.2 NeoTrix Has → Scrapling Lacks

| Feature | NeoTrix | Scrapling Status |
|---------|---------|-----------------|
| **SEAL Self-Iteration Loop** | `self_iterating/` — generate_self_edit → absorb → RL reward cycle | ❌ Python framework, no self-modification |
| **Reasoning Engine** | `reasoning_engine.rs` — LLM provider + ReasoningBank + CapabilityVector | ❌ |
| **Knowledge HyperCube** | `core/hypercube/` — VSA-based 4096-dim knowledge store | ❌ |
| **Agent Protocol** | `agent_protocol/` — UDP discovery + capability routing | ❌ |
| **Orchestrator** | PlannerNode → WorkerNode → CriticNode recursive loop | ❌ |
| **Goal Loop** | `goal_loop.rs` — GoalState lifecycle + RateLimiter + CircuitBreaker | ❌ |
| **Stealth Network (Tor)** | `stealth_net/` — Tor integration + proxy chain | ❌ |
| **Stagnation Detector** | `stagnation.rs` — rolling window detection | ❌ |
| **Security Audit Tools** | `security_audit.rs` — OWASP checklist | ❌ |
| **KnowledgeSource System** | 40+ registered external knowledge sources | ❌ |
| **MCP Tool Bridge** | rmcp 0.5 with 3 built-in tools (web_scrape, security_audit, react_doctor) | Has MCP scraping server but no general tool bridge |
| **Background Loop** | `background_loop.rs` — ticker-based periodic tasks | ❌ |
| **CamoFox Browser** | Anti-detection browser automation | Has `DynamicFetcher` (Playwright) but no fingerprint spoofing |

### 1.3 Both Lack

| Feature | Why Missing | Potential Source |
|---------|-------------|-----------------|
| **Distributed Crawling** | Neither has leader-worker / message queue based crawl distribution | Apache Nutch, Frontera |
| **JS Rendering Diff Comparison** | Neither captures visual diffs between crawl iterations | Playwright screenshot diff |
| **Form Auto-Fill Testing** | Neither has automated form interaction for testing | Playwright form fill |
| **Sitemap Auto-Discovery** | Neither auto-discovers sitemap.xml/robusts.txt sitemaps | Standard crawler feature |

---

## 2. Priority Classification (Impact × Urgency)

```
Impact: H(3) M(2) L(1)
Urgency: H(3) M(2) L(1)
Priority Score = Impact × Urgency

P0: Score ≥ 6 — must absorb
P1: Score 4-5 — high value
P2: Score ≤ 3 — nice to have
```

| ID | Feature | Impact | Urgency | Score | Priority |
|----|---------|--------|---------|-------|----------|
| G-01 | Adaptive Element Tracking | 3 | 3 | **9** | **P0** |
| G-02 | Checkpoint Spider | 3 | 3 | **9** | **P0** |
| G-03 | Multi-Session Router | 3 | 2 | **6** | **P1** |
| G-04 | Proxy Rotator (simple cyclic) | 2 | 2 | **4** | **P1** |
| G-05 | MCP Scraping Server | 2 | 1 | **2** | **P2** |
| G-06 | CLI Scraping Shell | 1 | 1 | **1** | **P2** |
| G-07 | robots.txt Compliance | 1 | 1 | **1** | **P2** |

### Priority Rationale

**P0 (G-01, G-02)**: Adaptive element tracking is the single most differentiated Scrapling feature — it's what makes Scrapling 5.2x faster than AutoScraper. The checkpoint spider with Ctrl+C resume is essential for production crawling. Both are blocking needs for the crawler module.

**P1 (G-03, G-04)**: Multi-session routing unlocks anti-bot bypass (Cloudflare, fingerprinting). NeoTrix already has strong proxy chain infrastructure — G-04 is just API wrapping. G-03 has high impact but requires more design work.

**P2 (G-05, G-06, G-07)**: These are UX/compliance features. The MCP scraping server is interesting but NeoTrix already has `web_scrape` tool. The CLI shell is low priority since NeoTrix is a library, not a CLI-first tool.

---

## 3. Integration Points

### 3.1 G-01: Adaptive Element Tracking — File: `crawler/adaptive.rs`

**Design**: Port Scrapling's similarity-based element relocation to Rust, leveraging `scraper` crate (Rust's equivalent of BeautifulSoup).

```rust
// crawler/adaptive.rs  — new module
pub struct AdaptiveTracker {
    saved_elements: Vec<SavedElement>,
    similarity_threshold: f64,  // default 0.75
}

pub struct SavedElement {
    pub css_selector: String,
    pub tag: String,
    pub text_content: String,
    pub class_patterns: Vec<String>,
    pub structural_hash: u64,  // DOM path fingerprint
    pub attribute_signature: HashMap<String, String>,
}

impl AdaptiveTracker {
    // Step 1: auto_save — when user selects an element, save its context
    pub fn auto_save(&mut self, html: &str, selector: &str) -> Result<SavedElement>;

    // Step 2: adaptive_find — when selector fails, relocate using similarity
    pub fn adaptive_find(&self, html: &str, saved: &SavedElement) -> Option<String>;

    // Step 3: find_similar — find elements with similar structure/text
    pub fn find_similar(&self, html: &str, saved: &SavedElement, limit: usize) -> Vec<LocatedElement>;
}
```

**Integration Points**:
- `crawler/mod.rs` → add `pub mod adaptive;`
- `unified.rs` → `UnifiedCrawler` gains optional `AdaptiveTracker` field
- `classifier.rs` → can use adaptive tracking for repeated element extraction across pages
- `KnowledgeSource::ScraplingAdaptive` — register as new source

**Similarity Algorithm** (ported from Scrapling's Python):
1. Extract DOM path signature (tag hierarchy + index)
2. Extract attribute fingerprint (class, id, data-* attributes)
3. Text content cosine similarity
4. Weighted combination → score
5. Return highest-scoring element above threshold

**Dependencies**: `scraper` crate (already likely in tree), `html5ever`, `select.rs`

### 3.2 G-02: Checkpoint Spider — File: `crawler/spider.rs`

**Design**: Scrapy-like async spider with checkpoint persistence and auto-resume.

```rust
// crawler/spider.rs  — new module
pub trait SpiderParse {
    async fn parse(&self, response: Response) -> Result<Vec<SpiderItem>>;
}

pub struct Spider {
    pub start_urls: Vec<String>,
    pub parse_handler: Box<dyn SpiderParse>,
    pub crawldir: PathBuf,
    pub concurrent_requests: usize,
    pub domain_throttle: HashMap<String, Duration>,
    pub download_delay: Duration,
    pub robots_txt_obey: bool,
    checkpoint: CheckpointStore,
}

pub struct CheckpointStore {
    pub path: PathBuf,
    pub visited: HashSet<String>,
    pub queued: VecDeque<UrlEntry>,
    pub in_progress: HashSet<String>,
    pub completed_count: u64,
    pub last_save: Instant,
}

impl CheckpointStore {
    pub fn load_or_create(path: &Path) -> Self;
    pub fn save(&self);
    pub fn mark_done(&mut self, url: &str);
    pub fn uncompleted(&self) -> Vec<UrlEntry>;
}

pub struct Response {
    pub url: String,
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub elapsed: Duration,
}
```

**Integration Points**:
- `crawler/mod.rs` → add `pub mod spider;`
- Reuses `DualQueueFrontier` from `frontier.rs` for URL queuing
- Reuses `FetcherPool` from `fetcher.rs` for HTTP fetching
- Checkpoint store serializes to JSON/MessagePack at `crawldir/checkpoint.json`
- Signal handler (SIGINT/SIGTERM) → `checkpoint.save()` then graceful exit
- `UnifiedCrawler.run_cycle()` can delegate to Spider for complex crawling
- `KnowledgeSource::ScraplingSpider` — register

**Streaming Mode** (bonus):
```rust
impl Spider {
    pub fn stream(&mut self) -> impl Stream<Item = SpiderItem> {
        async_stream! {
            // yields items as they're parsed
        }
    }
}
```

### 3.3 G-03: Multi-Session Router — Extension: `crawler/fetcher.rs`

**Design**: Route URLs to different fetcher types based on domain/URL pattern rules.

```rust
// Add to crawler/fetcher.rs or new file crawler/session_router.rs

pub struct SessionRouter {
    pub rules: Vec<RouteRule>,
    pub http_pool: FetcherPool,
    pub stealthy_pool: StealthyFetcher,
    pub dynamic_pool: DynamicFetcher,
    pub pool_stats: SessionPoolStats,
}

pub enum FetcherKind {
    Http,       // reqwest — fast, minimal
    Stealthy,   // CamoFox — anti-bot, fingerprint spoofing
    Dynamic,    // Playwright — full JS rendering
}

pub struct RouteRule {
    pub domain_pattern: String,  // glob or regex
    pub fetcher: FetcherKind,
    pub config: RouteConfig,
}

pub struct SessionPoolStats {
    pub busy_tabs: usize,
    pub free_tabs: usize,
    pub error_tabs: usize,
    pub total_sessions: usize,
}
```

**Integration Points**:
- `fetcher.rs` → already has `FetcherPool` and `FetcherProtocol` enum — extend with `Stealthy` variant
- `config.rs` → `CrawlerConfig` gains `session_routes: Vec<RouteRule>`
- `unified.rs` → `run_cycle()` uses router to select fetcher per URL
- `browser_automation/camofox.rs` → wraps as `StealthyFetcher`
- `KnowledgeSource::ScraplingSession`

### 3.4 G-04: Proxy Rotator (Simple Cyclic) — Extension: `crawler/config.rs`

**Design**: Add a simple cyclic proxy rotator to complement the existing multi-hop `DynamicProxyChain`.

```rust
// Add to crawler/config.rs or a new file

pub struct CyclicProxyRotator {
    proxies: Vec<String>,
    index: AtomicUsize,
    dns_over_https: Option<String>,  // e.g. "https://dns.google/dns-query"
}

impl CyclicProxyRotator {
    pub fn new(proxies: Vec<String>) -> Self;
    pub fn next(&self) -> Option<String>;           // round-robin
    pub fn next_for_url(&self, url: &str) -> Option<String>;  // domain-hash sticky
}
```

**Integration**:
- `config.rs` → `CrawlerConfig.proxy_pool: Vec<String>` already exists — use it to seed rotator
- `fetcher.rs` → `fetch()` uses rotator if configured
- For Tor/proxy-chain users: delegate to existing `DynamicProxyChain` (already superior)
- `KnowledgeSource::ScraplingProxy`

### 3.5 G-05: MCP Scraping Server — File: `mcp_tools.rs` extension

**Design**: Add a dedicated MCP scraping tool with auto-selector generation and content pre-extraction.

```rust
// In mcp_tools.rs

pub fn handle_scrapling_scrape(args: &serde_json::Value) -> Result<String, String> {
    // 1. Parse URL + optional CSS selector
    // 2. Fetch with SessionRouter (auto-selects fetcher type)
    // 3. If selector provided, extract specific elements
    // 4. If no selector, auto-generate selectors for likely targets
    // 5. Return structured JSON with extracted content + generated selectors
}
```

**Integration**:
- `mcp_tools.rs` → add `McpToolDef { name: "scrapling_scrape", ... }`
- Uses `SessionRouter` (G-03) for fetcher selection
- Uses `AdaptiveTracker` (G-01) for auto-selector generation
- `KnowledgeSource::ScraplingMcp`

### 3.6 KnowledgeSource Registration

New sources to register in `core/knowledge/sources.rs`:

```rust
KnowledgeSource::ScraplingAdaptive => "D4Vinci/Scrapling/adaptive-parsing",
KnowledgeSource::ScraplingSpider => "D4Vinci/Scrapling/checkpoint-spider",
KnowledgeSource::ScraplingSession => "D4Vinci/Scrapling/multi-session",
KnowledgeSource::ScraplingProxy => "D4Vinci/Scrapling/proxy-rotation",
KnowledgeSource::ScraplingMcp => "D4Vinci/Scrapling/mcp-scraping",
```

CapabilityVector dimensions for Scrapling knowledge:

| Source | Key Dimensions | Values |
|--------|---------------|--------|
| ScraplingAdaptive | `element_tracking`, `similarity_matching`, `dom_analysis` | 0.92, 0.88, 0.85 |
| ScraplingSpider | `crawl_architecture`, `checkpoint_persistence`, `streaming` | 0.95, 0.90, 0.85 |
| ScraplingSession | `anti_bot`, `session_management`, `fingerprint_rotation` | 0.88, 0.85, 0.82 |
| ScraplingProxy | `proxy_rotation`, `dns_leak_prevention` | 0.85, 0.80 |

---

## 4. Phase Plan

### Phase 1: Adaptive Parser (`crawler/adaptive.rs`) — ~200 lines

**Files to create/modify**:
- `neotrix-core/src/neotrix/crawler/adaptive.rs` — NEW (200 lines)
- `neotrix-core/src/neotrix/crawler/mod.rs` — add `pub mod adaptive;`
- `neotrix-core/src/neotrix/crawler/classifier.rs` — integrate adaptive tracking
- `core/knowledge/sources.rs` — add `ScraplingAdaptive`
- `core/knowledge/sources.rs` — `capability_vector()` implementation
- `core/knowledge/sources.rs` — `source_weight()` definition

**Key types**: `AdaptiveTracker`, `SavedElement`, `LocatedElement`, `SimilarityScorer`

**Tests**: 8 (see §5)

**Integration**: `UnifiedCrawler.run_cycle()` after fetch, runs adaptive extraction on each page. Results feed into `ContentClassifier` for higher-quality classification.

### Phase 2: Checkpoint Spider (`crawler/spider.rs`) — ~350 lines

**Files to create/modify**:
- `neotrix-core/src/neotrix/crawler/spider.rs` — NEW (350 lines)
- `neotrix-core/src/neotrix/crawler/mod.rs` — add `pub mod spider;`
- `core/knowledge/sources.rs` — add `ScraplingSpider`

**Key types**: `Spider`, `SpiderParse` trait, `CheckpointStore`, `Response`, `SpiderItem`

**Tests**: 10 (see §5)

**Integration**: Spider wraps `DualQueueFrontier` + `FetcherPool`. Optionally replaces `UnifiedCrawler.run_cycle()` for Scrapy-like use cases. Checkpoint serializes to `crawldir/checkpoint.json` via serde.

### Phase 3: Multi-Session Router (`crawler/session_router.rs`) — ~250 lines

**Files to create/modify**:
- `neotrix-core/src/neotrix/crawler/session_router.rs` — NEW (250 lines)
- `neotrix-core/src/neotrix/crawler/mod.rs` — add `pub mod session_router;`
- `neotrix-core/src/neotrix/crawler/fetcher.rs` — extend `FetcherProtocol::Stealthy`
- `neotrix-core/src/neotrix/crawler/config.rs` — add `session_routes`
- `neotrix-core/src/neotrix/browser_automation/camofox.rs` — wrap as `StealthyFetcher` impl
- `core/knowledge/sources.rs` — add `ScraplingSession`

**Key types**: `SessionRouter`, `RouteRule`, `FetcherKind`, `SessionPoolStats`

**Tests**: 7 (see §5)

**Integration**: `UnifiedCrawler.run_cycle()` calls `SessionRouter.select_fetcher(url)` before fetching. Pool stats exposed via `FetcherSummary`.

### Phase 4: Proxy Rotator Enhancement — ~100 lines

**Files to modify**:
- `neotrix-core/src/neotrix/stealth_net/proxy_chain/mod.rs` — add `CyclicProxyRotator`
- `neotrix-core/src/neotrix/crawler/fetcher.rs` — integrate simple rotation
- `neotrix-core/src/neotrix/stealth_net/proxy_chain/types.rs` — add cyclic selection strategy
- `core/knowledge/sources.rs` — add `ScraplingProxy`

**Key types**: `CyclicProxyRotator` (wraps existing `DynamicProxyChain`)

**Tests**: 4 (see §5)

**Integration**: `FetcherPool.fetch()` checks `config.proxy_pool` and uses rotator if non-empty. DNS-over-HTTPS as optional feature.

### Phase 5 (P2): MCP Scraping Server + CLI + robots.txt — ~200 lines

**Files to modify**:
- `neotrix-core/src/neotrix/mcp_tools.rs` — add `scrapling_scrape` MCP tool
- `neotrix-core/src/neotrix/crawler/spider.rs` — add robots.txt parser
- `neotrix-cli/` — add `neotrix shell` command
- `core/knowledge/sources.rs` — add `ScraplingMcp`

**Tests**: 6 (see §5)

---

## 5. Test Strategy

### Per-Feature Test Matrix

| Feature | Unit Tests | Integration Tests | Total | Key Test Cases |
|---------|-----------|-------------------|-------|----------------|
| **G-01 Adaptive** | 6 | 2 | **8** | `auto_save_saves_context`, `adaptive_find_relocates_on_selector_failure`, `adaptive_find_returns_none_on_no_match`, `find_similar_returns_multiple`, `similarity_scorer_text_match`, `similarity_scorer_attribute_match`, `structural_hash_identical_for_same_dom_path`, `adaptive_find_tolerates_minor_html_changes` |
| **G-02 Spider** | 7 | 3 | **10** | `checkpoint_save_load_roundtrip`, `spider_visits_start_urls`, `spider_resumes_from_checkpoint`, `checkpoint_mark_done`, `spider_stream_yields_items`, `spider_respects_concurrent_requests`, `spider_domain_throttle`, `ctrl_c_graceful_shutdown`, `spider_empty_queue_returns`, `spider_honors_max_depth` |
| **G-03 Session Router** | 5 | 2 | **7** | `router_selects_http_for_default`, `router_selects_stealthy_for_cloudflare_domain`, `router_selects_dynamic_for_js_heavy`, `route_rule_glob_matches`, `pool_stats_tracks_busy_free`, `router_falls_back_on_empty_rules`, `session_reuses_existing_fetcher` |
| **G-04 Proxy Rotator** | 3 | 1 | **4** | `cyclic_rotator_round_robins`, `sticky_routing_same_domain_gets_same_proxy`, `rotator_skips_dead_proxies`, `rotator_integration_with_fetcher` |
| **G-05 MCP Scraping** | 3 | 1 | **4** | `scrapling_scrape_returns_json`, `auto_selector_generates_valid_css`, `scrapling_scrape_handles_errors`, `scrapling_scrape_respects_session_routing` |
| **G-06 CLI Shell** | 1 | 1 | **2** | `shell_parses_command`, `shell_extract_runs_fetch` |
| **G-07 robots.txt** | 2 | 0 | **2** | `parses_robots_txt`, `allows_disallowed_url_returns_false` |
| **Total** | **27** | **10** | **37** | — |

### Test Architecture

- **Unit tests**: Inline `#[cfg(test)] mod tests { use super::*; }` per module (no `use crate::` imports)
- **Integration tests**: Use mock HTTP server (`mockito` or `wiremock`) for fetcher tests
- **Checkpoint tests**: Temp directories via `tempfile` crate
- **Adaptive tests**: Static HTML fixtures stored as `&str` constants

### Quality Gates

| Gate | Command | Target |
|------|---------|--------|
| Compile | `cargo check --lib` | 0 errors |
| Full features | `cargo check --features full --lib` | 0 errors |
| Tests | `cargo test --lib adaptive` | all pass |
| Warning | `cargo check --lib 2>&1 | grep warning | wc -l` | 0 new warnings |

---

## 6. Dependency Impact

| Depedency | Phase | Notes |
|-----------|-------|-------|
| `scraper` crate | G-01 | HTML parsing + CSS selector (likely already transitive) |
| `select.rs` / `ego-tree` | G-01 | DOM traversal for structural hash |
| `serde` + `serde_json` | G-02 | Checkpoint serialization (already in tree) |
| `signal-hook` | G-02 | Ctrl+C handler for graceful shutdown |
| `async-stream` | G-02 | Streaming mode support |
| `mockito` / `wiremock` | Test | HTTP mock for fetcher tests |
| `tempfile` | Test | Temp directories for checkpoint tests |

No new major dependencies — everything is available in the Rust ecosystem and likely already in `Cargo.toml`.

---

## 7. Feature Mapping to Existing Modules

```
Scrapling Feature              NeoTrix Module
─────────────────              ──────────────
Adaptive Parser     ────────→  crawler/adaptive.rs        (NEW)
Checkpoint Spider   ────────→  crawler/spider.rs           (NEW)
Multi-Session       ────────→  crawler/session_router.rs   (NEW)
  ├─ Fetcher        ────────→  crawler/fetcher.rs          (extend)
  ├─ Stealthy       ────────→  browser_automation/camofox.rs(wrap)
  └─ Dynamic        ────────→  (Playwright bridge)         (future)
Proxy Rotator       ────────→  stealth_net/proxy_chain/    (extend)
MCP Server          ────────→  mcp_tools.rs                (extend)
CLI Shell           ────────→  neotrix-cli/                (extend)
robots.txt          ────────→  crawler/spider.rs           (extend)
Knowledge           ────────→  core/knowledge/sources.rs   (register)
```

---

## 8. Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Similarity algorithm port from Python to Rust loses accuracy | Medium | Implement equivalent structural/attribute/text scoring; benchmark against Scrapling test fixtures |
| Checkpoint serialization perf at scale (100k+ URLs) | Low | Use MessagePack instead of JSON for large checkpoints; lazy loading for visited set |
| CamoFox integration as StealthyFetcher is complex | Medium | Start with simple wrapper; full integration deferred to Phase 5 |
| `scraper` crate DOM API differs from BeautifulSoup | Low | Scrapling uses lxml; Rust's `scraper` + `ego-tree` has equivalent traversal — map operations 1:1 |
| Ctrl+C handler conflicts with existing signal handlers | Low | Use `signal-hook` registry pattern, not raw `libc` signals |

---

## 9. Success Criteria

| Metric | Current State | Post-Absorption Target |
|--------|--------------|----------------------|
| Element extraction robustness | None (page-level only) | Adaptive relocation works with ≥80% success on changed pages |
| Crawl resume capability | P0-W01 basic checkpoint | Full checkpoint spider with Ctrl+C → resume in <1s |
| Fetcher protocols available | 2 (HTTP, Browser) | 3 (HTTP, Stealthy, Dynamic) |
| Proxy rotation strategies | 5 (Random, Weighted, LowestLatency, HighestSuccess, GeoRoundRobin) | 6 (+ Cyclic) |
| MCP scraping tools | 1 (web_scrape raw) | 2 (+ scrapling_scrape with auto-selectors) |
| Tests added | 0 (new code) | 37 new tests |
| Knowledge sources registered | 40+ | 45+ (+5 Scrapling sources) |

---

*End of fusion design document. Phase 0 complete — ready for user approval before Phase 1 implementation.*

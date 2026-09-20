# OSINT & World Modules (nt_world)

> L2 Perception — OSINT, Crawling, Exploration, Media Sources

---

## Overview

NT-WORLD provides comprehensive world perception capabilities including OSINT intelligence gathering, web crawling, exploration engines, and media source management.

```text
nt_world
├── crawl/              — Web crawling & browsing
│   ├── nt_world_crawl
│   ├── browse
│   └── browse_auto
├── osint/              — Intelligence gathering
│   ├── nt_world_osint
│   └── absorber
├── sense/              — Perception & sensing
│   ├── nt_world_sense
│   ├── jepa
│   └── model
├── explore/            — Exploration & mapping
│   ├── nt_world_map
│   └── cleanup
├── source/             — Media sources
│   ├── audio
│   ├── video
│   ├── text
│   ├── social
│   └── playlist
├── data_source/        — Intelligence data sources (12)
│   ├── edgar
│   ├── gdelt
│   ├── usgs
│   ├── gdacs
│   ├── ucdp
│   ├── urlhaus
│   ├── ofac
│   ├── polymarket
│   ├── aoi
│   ├── adsb
│   ├── bgpview
│   └── opencorporates
└── Specialized Modules
    ├── nt_world_search
    ├── nt_world_code_search
    ├── nt_world_scrape
    ├── nt_world_prefetch
    └── nt_world_video_pipeline
```

---

## Key Types

### `OsimpEngine`

OSINT intelligence collection engine.

```rust
pub struct OsintEngine {
    sources: Vec<Box<dyn IntelSource>>,
    config: OsintConfig,
    cache: IntelCache,
}

pub struct OsintConfig {
    pub max_concurrent: usize,
    pub timeout: Duration,
    pub cache_ttl: Duration,
    pub rate_limit: u32,
}

pub struct IntelReport {
    pub target: String,
    pub findings: Vec<IntelFinding>,
    pub metadata: HashMap<String, Value>,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
}

pub struct IntelFinding {
    pub category: IntelCategory,
    pub data: Value,
    pub source: String,
    pub confidence: f64,
}

pub enum IntelCategory {
    Network,
    Domain,
    Whois,
    Dns,
    Certificate,
    Technology,
    Personnel,
    Financial,
    Legal,
    Social,
}
```

**Public Methods:**

```rust
impl OsintEngine {
    pub fn new(config: OsintConfig) -> Self;
    pub fn collect_intel(&self, target: &str) -> Result<IntelReport>;
    pub fn add_source(&mut self, source: Box<dyn IntelSource>);
    pub fn query(&self, query: &str) -> Vec<IntelFinding>;
    pub fn cache_stats(&self) -> CacheStats;
}
```

---

### `WebCrawler`

Web crawling and content extraction.

```rust
pub struct WebCrawler {
    client: HttpClient,
    config: CrawlConfig,
    visited: HashSet<String>,
}

pub struct CrawlConfig {
    pub max_depth: usize,
    pub max_pages: usize,
    pub delay: Duration,
    pub respect_robots: bool,
    pub user_agent: String,
}

pub struct CrawlResult {
    pub url: String,
    pub content: String,
    pub links: Vec<String>,
    pub metadata: CrawlMetadata,
    pub duration: Duration,
}

pub struct CrawlMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub content_type: String,
    pub size: u64,
}
```

**Public Methods:**

```rust
impl WebCrawler {
    pub fn new(config: CrawlConfig) -> Self;
    pub fn crawl(&mut self, url: &str) -> Result<CrawlResult>;
    pub fn crawl_batch(&mut self, urls: &[String]) -> Vec<Result<CrawlResult>>;
    pub fn visited(&self) -> &HashSet<String>;
    pub fn clear_visited(&mut self);
}
```

---

### `ExplorationEngine`

Autonomous exploration and discovery.

```rust
pub struct ExplorationEngine {
    knowledge_base: KnowledgeBase,
    frontier: Vec<ExplorationTarget>,
    history: Vec<ExplorationResult>,
}

pub struct ExplorationTarget {
    pub url: String,
    pub priority: f64,
    pub discovered_at: DateTime<Utc>,
    pub source: String,
}

pub struct ExplorationResult {
    pub target: ExplorationTarget,
    pub findings: Vec<Finding>,
    pub new_targets: Vec<ExplorationTarget>,
    pub duration: Duration,
}

pub struct Finding {
    pub kind: FindingKind,
    pub data: Value,
    pub confidence: f64,
}

pub enum FindingKind {
    Entity,
    Relation,
    Fact,
    Pattern,
    Anomaly,
}
```

**Public Methods:**

```rust
impl ExplorationEngine {
    pub fn new(knowledge_base: KnowledgeBase) -> Self;
    pub fn explore(&mut self, target: &str) -> Result<ExplorationResult>;
    pub fn frontier(&self) -> &[ExplorationTarget];
    pub fn history(&self) -> &[ExplorationResult];
    pub fn prioritize(&mut self);
}
```

---

### `MediaAssetRegistry`

Media asset management and processing.

```rust
pub struct MediaAssetRegistry {
    assets: HashMap<String, MediaAsset>,
    processors: Vec<Box<dyn MediaProcessor>>,
}

pub struct MediaAsset {
    pub id: String,
    pub url: String,
    pub kind: MediaKind,
    pub metadata: MediaMetadata,
    pub processed: bool,
}

pub enum MediaKind {
    Image,
    Video,
    Audio,
    Document,
    Archive,
}

pub struct MediaMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub duration: Option<Duration>,
    pub size: u64,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
}
```

**Public Methods:**

```rust
impl MediaAssetRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, asset: MediaAsset) -> String;
    pub fn get(&self, id: &str) -> Option<&MediaAsset>;
    pub fn process(&mut self, id: &str) -> Result<()>;
    pub fn search(&self, query: &str) -> Vec<&MediaAsset>;
    pub fn stats(&self) -> RegistryStats;
}
```

---

### `DynamicMemoryBank`

Dynamic memory for world model state.

```rust
pub struct DynamicMemoryBank {
    entries: HashMap<String, MemoryEntry>,
    decay_rate: f64,
}

pub struct MemoryEntry {
    pub key: String,
    pub value: Value,
    pub strength: f64,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
}
```

**Public Methods:**

```rust
impl DynamicMemoryBank {
    pub fn new(decay_rate: f64) -> Self;
    pub fn store(&mut self, key: &str, value: Value);
    pub fn retrieve(&self, key: &str) -> Option<&Value>;
    pub fn decay(&mut self);
    pub fn strongest(&self, n: usize) -> Vec<(&str, &Value)>;
}
```

---

## Intelligence Data Sources (12)

| Source | Module | Description |
|--------|--------|-------------|
| SEC EDAR | `nt_world_edgar` | SEC filings & corporate data |
| GDELT | `nt_world_gdelt` | Global event database |
| USGS | `nt_world_usgs` | Earthquake & seismic data |
| GDACS | `nt_world_gdacs` | Disaster alert system |
| UCDP | `nt_world_ucdp` | Conflict data program |
| URLHaus | `nt_world_urlhaus` | Malware URL database |
| OFAC | `nt_world_ofac` | Sanctions database |
| Polymarket | `nt_world_polymarket` | Prediction market data |
| AOI | `nt_world_aoi` | Area of interest data |
| ADS-B | `nt_world_adsb` | Aircraft tracking |
| BGPView | `nt_world_bgpview` | BGP prefix data |
| OpenCorporates | `nt_world_opencorporates` | Corporate registry |

---

## Specialized Modules

### `nt_world_search`

Web search integration.

### `nt_world_code_search`

Code search across repositories.

### `nt_world_scrape`

Advanced scraping with anti-detection.

### `nt_world_prefetch`

Predictive content prefetching.

### `nt_world_video_pipeline`

Video processing and analysis pipeline.

### `nt_world_jepa`

Joint Embedding Predictive Architecture for world models.

### `nt_world_model_v2`

World model v2 with temporal reasoning.

### `nt_world_novel`

Novelty detection in world observations.

---

## Related Modules

| Module | Description |
|--------|-------------|
| `nt_nlp_capability` | NLP processing |
| `ocr` | Optical character recognition |
| `asset_map` | Asset mapping |
| `rag_pipeline` | Local RAG pipeline |
| `function_recovery` | Function recovery & disassembly |
| `cfg_builder` | Control flow graph construction |

---

## Usage Examples

### OSINT Collection

```rust
use neotrix_core::l2_perception::nt_world::osint::OsimpEngine;

let engine = OsintEngine::new(OsintConfig::default());
let report = engine.collect_intel("example.com")?;

for finding in &report.findings {
    println!("{:?}: {:?}", finding.category, finding.data);
}
```

### Web Crawling

```rust
use neotrix_core::l2_perception::nt_world::crawl::WebCrawler;

let mut crawler = WebCrawler::new(CrawlConfig {
    max_depth: 3,
    max_pages: 100,
    delay: Duration::from_secs(1),
    respect_robots: true,
    user_agent: "NeoTrix/0.18.0".into(),
});

let result = crawler.crawl("https://example.com")?;
println!("Found {} links", result.links.len());
```

### Exploration

```rust
use neotrix_core::l2_perception::nt_world::explore::ExplorationEngine;

let mut engine = ExplorationEngine::new(knowledge_base);
let result = engine.explore("https://news.ycombinator.com")?;

for finding in &result.findings {
    println!("{:?}: {:?}", finding.kind, finding.data);
}
```

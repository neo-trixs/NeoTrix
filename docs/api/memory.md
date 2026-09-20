# Memory Modules

> L5 Memory & Knowledge Management — ReasoningBank, CortexMemory, KnowledgeEngine

---

## Overview

The memory subsystem manages structured knowledge storage, retrieval, and knowledge chain operations. It provides a multi-layered memory architecture inspired by human cognition.

```text
KnowledgeEngine ←→ ReasoningBank ←→ CortexMemory
       ↓                ↓                ↓
  LiteratureSearch   MemoryLayers    DimensionTags
       ↓                ↓                ↓
  KnowledgeChain    MemoryTrace     Modality
```

---

## Key Types

### `ReasoningBank`

Multi-dimensional memory bank for reasoning context.

```rust
pub struct ReasoningBank {
    entries: Vec<ReasoningMemory>,
    stats: ReasoningBankStats,
}

pub struct ReasoningMemory {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
    pub access_count: u64,
}

pub struct ReasoningBankStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub avg_access_frequency: f64,
}
```

**Public Methods:**

```rust
impl ReasoningBank {
    pub fn new(config: MemoryConfig) -> Self;
    pub fn store(&mut self, entry: ReasoningMemory) -> Result<()>;
    pub fn retrieve(&self, query: &str, top_k: usize) -> Vec<ReasoningMemory>;
    pub fn update(&mut self, id: &str, content: &str) -> Result<()>;
    pub fn delete(&mut self, id: &str) -> Result<()>;
    pub fn stats(&self) -> ReasoningBankStats;
    pub fn iterate(&mut self) -> MemoryIterationResult;
}
```

---

### `CortexMemory`

Human-brain-inspired multi-dimensional memory architecture.

```rust
pub struct CortexMemory {
    layers: Vec<MemoryLayer>,
    config: CmsConfig,
    stats: CortexStats,
}

pub struct MemoryLayer {
    pub name: String,
    pub capacity: usize,
    pub entries: Vec<MemoryTrace>,
}

pub struct MemoryTrace {
    pub id: String,
    pub content: String,
    pub dimensions: Vec<DimensionTag>,
    pub modality: Modality,
    pub strength: f64,
    pub last_accessed: DateTime<Utc>,
}

pub enum Modality {
    Semantic,
    Episodic,
    Procedural,
    Emotional,
}

pub struct DimensionTag {
    pub name: String,
    pub weight: f64,
}
```

**Public Methods:**

```rust
impl CortexMemory {
    pub fn new(config: CmsConfig) -> Self;
    pub fn encode(&self, trace: &MemoryTrace) -> Vec<f32>;
    pub fn decode(&self, embedding: &[f32]) -> MemoryTrace;
    pub fn consolidate(&mut self) -> CmsResult;
    pub fn recall(&self, query: &str, modality: Option<Modality>) -> Vec<MemoryTrace>;
    pub fn sleep_cycle(&mut self) -> SleepResult;
    pub fn stats(&self) -> CortexStats;
}
```

---

### `KnowledgeEngine`

Structured knowledge engine with literature search, persistence, and relation networks.

```rust
pub struct KnowledgeEngine {
    entries: Vec<KnowledgeEntry>,
    relations: Vec<KnowledgeRelation>,
    searcher: LiteratureSearcher,
    stats: KnowledgeEngineStats,
}

pub struct KnowledgeEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source: SourceType,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

pub struct KnowledgeRelation {
    pub from: String,
    pub to: String,
    pub relation: RelationType,
    pub weight: f64,
}

pub enum RelationType {
    DependsOn,
    ConflictsWith,
    Supports,
    Extends,
    Replaces,
}

pub enum SourceType {
    Literature,
    Code,
    Web,
    Manual,
    Inferred,
}
```

**Public Methods:**

```rust
impl KnowledgeEngine {
    pub fn new() -> Self;
    pub fn add_entry(&mut self, entry: KnowledgeEntry) -> Result<String>;
    pub fn get_entry(&self, id: &str) -> Option<&KnowledgeEntry>;
    pub fn add_relation(&mut self, relation: KnowledgeRelation) -> Result<()>;
    pub fn query(&self, query: &str) -> Vec<KnowledgeEntry>;
    pub fn traverse(&self, start: &str, depth: usize) -> Vec<KnowledgeEntry>;
    pub fn search_literature(&self, query: &str) -> Vec<KnowledgeEntry>;
    pub fn export_graph(&self) -> KnowledgeGraph;
    pub fn stats(&self) -> KnowledgeEngineStats;
}
```

---

### `KnowledgeChain`

Knowledge chain connecting mining → verification → absorption → storage.

```rust
pub struct KnowledgeChain {
    phases: Vec<KnowledgeChainPhase>,
    status: KnowledgeChainStatus,
}

pub enum KnowledgeChainPhase {
    Mining,
    Verification,
    Absorption,
    Storage,
    crystallization,
}

pub enum KnowledgeChainStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Failed(String),
}

pub struct ChainRunResult {
    pub entries_mined: usize,
    pub entries_verified: usize,
    pub entries_absorbed: usize,
    pub entries_stored: usize,
    pub duration: Duration,
    pub errors: Vec<String>,
}
```

**Public Methods:**

```rust
impl KnowledgeChain {
    pub fn new() -> Self;
    pub fn run(&mut self) -> ChainRunResult;
    pub fn pause(&mut self);
    pub fn resume(&mut self);
    pub fn status(&self) -> &KnowledgeChainStatus;
    pub fn phase(&self) -> Option<&KnowledgeChainPhase>;
}
```

---

### `KnowledgeMiner`

Automated knowledge mining from external sources.

```rust
pub struct KnowledgeMiner {
    sources: Vec<Box<dyn KnowledgeSource>>,
    mined: Vec<MinedKnowledge>,
}

pub struct MinedKnowledge {
    pub content: String,
    pub source: String,
    pub confidence: f64,
    pub metadata: HashMap<String, Value>,
}

pub struct MinedRoundResult {
    pub total_mined: usize,
    pub accepted: usize,
    pub rejected: usize,
    pub duration: Duration,
}
```

**Public Methods:**

```rust
impl KnowledgeMiner {
    pub fn new() -> Self;
    pub fn add_source(&mut self, source: Box<dyn KnowledgeSource>);
    pub fn mine_round(&mut self) -> MinedRoundResult;
    pub fn mined_knowledge(&self) -> &[MinedKnowledge];
    pub fn clear(&mut self);
}
```

---

## Related Modules

| Module | Description |
|--------|-------------|
| `knowledge::web_miner` | Web knowledge mining (wiki/arXiv/GitHub) |
| `knowledge::knowledge_aging` | Knowledge freshness tracking |
| `knowledge::impact_matrix` | Capability dimension impact weights |
| `knowledge::seal_algebra` | SEAL convergence verification |
| `knowledge::exploration_pipeline` | External knowledge absorption |
| `knowledge::change_archive` | Delta spec + conflict detection |
| `nt_core_second_brain` | Second brain integration |

---

## Usage Examples

### Store and Retrieve

```rust
let mut bank = ReasoningBank::new(MemoryConfig::default());
bank.store(ReasoningMemory {
    id: "doc1".into(),
    content: "NeoTrix uses selective state-space operations".into(),
    embedding: vec![0.1, 0.2, 0.3],
    metadata: HashMap::new(),
    created_at: Utc::now(),
    access_count: 0,
})?;

let results = bank.retrieve("state-space", 5);
```

### Knowledge Chain Pipeline

```rust
let mut chain = KnowledgeChain::new();
let result = chain.run();
println!("Mined: {}, Stored: {}", result.entries_mined, result.entries_stored);
```

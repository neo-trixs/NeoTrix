# NeoTrix Architecture Redesign v2

## Principles
1. Single responsibility per module directory
2. Clean separation: core (pure VSA/consciousness) vs runtime (I/O, network, persistence)
3. Unified storage layer: all persistence through one service
4. Encryption at rest for all persistent data

## Proposed Directory Structure

```
neotrix-core/src/
├── consciousness/       # Consciousness pipeline (loop engine, integration)
│   ├── mod.rs
│   ├── core.rs          # Pipeline orchestration
│   ├── backpressure/    # Stage isolation
│   ├── handlers/        # Individual handler implementations
│   └── types/           # Shared types
│
├── knowledge/           # Knowledge & memory
│   ├── mod.rs
│   ├── engine/          # KnowledgeEngine, graph ops
│   ├── evidence/        # Evidence tracking, scoring
│   ├── hypergraph/      # N-ary hypergraph
│   ├── spread/          # Spreading activation
│   ├── versioning/      # Knowledge version management
│   └── retrieval/       # RAG, entity resolution
│
├── hcube/               # VSA HyperCube operations
│   ├── mod.rs
│   ├── core/            # QuantizedVSA, bind/unbind/bundle
│   ├── encoder/         # Adaptive, cross-modal, ngram
│   ├── resonator/       # Multi-head, resonator networks
│   └── transform/       # FFT, cleanup, permutation
│
├── agent/               # Multi-agent system
│   ├── mod.rs
│   ├── bus/             # AgentCommunicationBus
│   ├── a2a/             # A2A protocol
│   ├── consensus/       # BFT consensus
│   ├── mcp/             # MCP client/server
│   └── identity/        # Agent identity
│
├── reasoning/           # Reasoning engines
│   ├── mod.rs
│   ├── e8/              # E8 64-state reasoning
│   ├── godel/           # Gödel self-reference
│   └── ne/              # Ne language runtime
│
├── perception/          # Input processing
│   ├── mod.rs
│   ├── vsa_input/       # VSAInputPipeline
│   ├── vision/          # Image pipeline
│   ├── audio/           # Speech-to-text
│   └── nlp/             # Text processing
│
├── experience/          # Learning & adaptation
│   ├── mod.rs
│   ├── curriculum/       # Curriculum generation
│   ├── calibration/      # Calibration engine
│   ├── loss/             # Composite loss
│   ├── arena/            # Adversarial co-evolution
│   └── skill/            # Skill accumulation
│
├── meta/                # Self-reflection & metadata
│   ├── mod.rs
│   ├── timer/            # TimerRegistry
│   ├── audit/            # Audit engine
│   ├── profiler/         # HandlerProfiler
│   └── monitor/          # Health patrol
│
├── storage/             # Unified persistence layer — NEW
│   ├── mod.rs
│   ├── ntsseg/           # NTSSEG segment format
│   ├── encrypted/        # AES-256-GCM wrapper
│   ├── keychain/         # Key management (derived from system seed)
│   ├── index/            # IVF similarity index
│   └── migrate/          # Migration from raw files
│
├── shield/              # Safety & security
│   ├── mod.rs
│   ├── safety_gate/      # Edit safety
│   ├── sandbox/          # Code sandbox
│   ├── stealth_net/      # Proxy, anti-detection
│   └── audit/            # SecurityAuditor
│
├── world/               # External world interaction — consolidated from nt_world_*
│   ├── mod.rs
│   ├── crawl/            # Web crawling
│   ├── browse/           # Browser automation
│   ├── search/           # Web search
│   └── model/            # World model
│
├── entry/               # Runtime entry points (keep as-is)
├── bin/                 # Binary targets (keep as-is)
├── lib.rs               # Crate root
└── neotrix/             # Legacy — frozen, no new code
```

## Encrypted Unified Storage Layer

### Design
- All persistent data goes through a single `StorageService` trait
- Encryption: AES-256-GCM with per-segment nonce
- Key derivation: HKDF from system seed (stored in OS keychain or env NEOTRIX_SEED)
- Segment format: NTSSEG magic + IV (12 bytes) + ciphertext + HMAC (16 bytes)
- Index: IVF over VSA vectors, encrypted at rest

### StorageService trait
```rust
pub trait StorageService {
    fn store(&mut self, namespace: &str, key: &str, data: &[u8]) -> Result<(), StorageError>;
    fn load(&self, namespace: &str, key: &str) -> Result<Vec<u8>, StorageError>;
    fn delete(&mut self, namespace: &str, key: &str) -> Result<(), StorageError>;
    fn list(&self, namespace: &str) -> Result<Vec<String>, StorageError>;
    fn vsa_search(&self, query: &[u8], top_k: usize) -> Result<Vec<(String, f64)>, StorageError>;
}
```

### Namespace mapping
| Namespace | Content | Encryption |
|-----------|---------|------------|
| `brain` | SelfIteratingBrain state | Yes |
| `knowledge` | KnowledgeEntries | Yes |
| `lexicon` | Translation lexicons | Yes |
| `config` | BackgroundConfig | No (plain JSON) |
| `cache` | DNS cache, temp data | No |
| `session` | Conversation history | Yes |
| `agent` | Agent identities | Yes |
| `logs` | Log files | No |

### Migration plan
1. Implement StorageService with NTSSEG backend
2. Add encrypt/decrypt wrapper
3. One-by-one migrate subsystems from raw files
4. Keep backward compat: old files still readable
5. Remove old read/write paths once migrated

## Migration Strategy

### Phase 1: Structure (this session)
- Create new module stubs (empty mod.rs files)
- Update lib.rs to expose new module tree
- Old modules remain accessible for backward compat

### Phase 2: Storage layer (next session)
- Implement StorageService
- Wire into consciousness pipeline
- No existing code changes needed

### Phase 3: Module migration (ongoing)
- Move code from neotrix/ to new structure
- One subsystem per PR
- Old path re-exports from new path for backward compat

### Phase 4: Legacy removal (future)
- Remove neotrix/ re-exports
- Delete old directories
- Full structure consolidation

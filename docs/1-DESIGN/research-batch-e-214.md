# Research Batch E-214: Framework & Technology Landscape

**Date**: 2026-09-10
**Purpose**: Absorb latest ecosystem state for NeoTrix architecture decisions
**Sources**: 45+ web sources across 15 topics, Q3-Q4 2026

---

## 1. Rust Web Frameworks: Axum vs Actix-Web vs Rocket

### Core Advantages

| Framework | Core Advantage | Key Metric (2026) |
|-----------|---------------|-------------------|
| **Axum 0.8.x** | Type-safe extractors, Tower ecosystem, Tokio-native. Minimal macro magic, pure async functions as handlers. Best middleware composability via `tower::Layer`. | 120K req/s, 11MB memory, p99 9ms |
| **Actix-Web 4.x/5.0** | Raw throughput champion. Actor-inspired share-nothing worker model. Multi-threaded, per-worker state isolation. | 142K req/s, 12MB memory, p99 8ms |
| **Rocket 0.5.x/0.6** | Developer ergonomics leader. Heavy macro-driven routing, built-in request guards, form handling. Rails of Rust. | 104K req/s, 14MB memory, p99 12ms |

### Applicable Scenarios
- **Axum**: API gateways, microservices, complex middleware chains, modern Rust APIs. Recommended default for 80% of new projects.
- **Actix-Web**: High-frequency trading, real-time data, maximum throughput services (>100K req/s).
- **Rocket**: Rapid prototyping, internal tools, junior Rust teams, side projects.

### NeoTrix Mapping
- **Primary**: Axum — aligns with Tokio ecosystem (nt-io LLM providers, nt-core HTTP). Tower middleware maps to NT-SHIELD egress guard layers.
- **Secondary**: Actix-Web — for ultra-hot paths (KB embedding search, vector similarity scoring).
- **Not recommended**: Rocket — slower release cadence, less Tower integration, macro-heavy for NeoTrix's compositional style.

### Priority: **P0** — Web server foundation for NT-IO and NT-WORLD HTTP interfaces

---

## 2. Rust Async Runtimes: Tokio vs async-std vs smol

### Core Advantages

| Runtime | Core Advantage | Key Metric |
|---------|---------------|------------|
| **Tokio** | Industry standard. Work-stealing scheduler. 90%+ of Rust async crates target Tokio. Battle-tested at Discord, AWS, Cloudflare. | 1.2M req/s, p99 4.2ms, 85MB |
| **async-std** | API mirrors `std`. Lower learning curve. Simple dependency tree. **Discontinued March 2025** — replaced by smol. | 900K req/s, p99 6.8ms |
| **smol** | Minimal (~1500 LOC). Composable. No runtime coupling. Best for libraries and CLI tools. | 1M req/s, p99 5.1ms, 45MB |
| **glommio** | Thread-per-core, io_uring. Maximum throughput for I/O-bound workloads. Linux only. | 1.8M req/s, p99 1.2ms |

### Applicable Scenarios
- **Tokio**: Production web services, database clients, anything touching network. Default choice.
- **smol**: Small CLI tools, libraries that shouldn't impose runtime. Embedded.
- **glommio**: Database engines, proxies, brokers on Linux with io_uring.
- **async-std**: Avoid for new projects (discontinued).

### NeoTrix Mapping
- **Mandatory**: Tokio — entire NT ecosystem (axum, sqlx, reqwest, tonic, redis-rs) is Tokio-native. Already embedded in neotrix-core.
- **Optional**: smol — for lightweight NT-ACT CLI tools where Tokio's dependency weight is excessive.

### Priority: **P0** — Already adopted. Tokio is non-negotiable.

---

## 3. Rust Serialization: Serde + Bincode vs MessagePack

### Core Advantages

| Format | Core Advantage | Key Metric |
|--------|---------------|------------|
| **Serde (framework)** | Format-agnostic derive macros. 340M+ downloads. Zero-copy deserialization. | Framework, not format |
| **Bincode** | Fastest binary serialization. Rust-to-Rust communication. Minimal overhead. | 43ns serialize, 127ns deserialize, 58 bytes |
| **MessagePack (rmp)** | Smallest serialized size. Cross-language binary. JSON drop-in replacement. | 93ns serialize, 197ns deserialize, 24 bytes |
| **Postcard** | Embedded-optimized. Good size/speed compromise. | 55ns serialize, 210ns deserialize, 41 bytes |
| **Protobuf** | Schema-enforced. Best for gRPC. Strong cross-language support. | 1.8μs serialize, 4.3μs deserialize |

### Applicable Scenarios
- **Bincode**: Internal Rust-to-Rust protocols, KB embedding storage, caching, event sourcing events.
- **MessagePack**: Cross-language APIs, JSON migration, compact wire format without schema.
- **Protobuf/gRPC**: External service APIs, inter-service communication (tonic).
- **JSON (serde_json)**: Human-readable config, API responses, debugging.

### NeoTrix Mapping
- **Internal persistence**: Bincode — KB embedding vectors, VSA HyperCube state, SEAL pipeline intermediate data.
- **External APIs**: Protobuf via tonic — NT-IO LLM provider communication.
- **Config/docs**: serde_json — human-readable settings, CLI output.
- **Cross-language**: MessagePack — if Python tool integration needed (NT-ACT scripts).

### Priority: **P1** — Bincode for internal, Protobuf for external, JSON for config

---

## 4. Rust Concurrency: Rayon vs Crossbeam

### Core Advantages

| Crate | Core Advantage | Key Feature |
|-------|---------------|-------------|
| **Rayon** | Drop-in parallel iterators. `par_iter()` replaces `iter()`. Work-stealing thread pool. Data-race free by construction. | Parallel iterators, fork-join, scoped tasks |
| **Crossbeam** | Scoped threads, work-stealing deques, lock-free data structures, epoch-based GC, MPMC channels. | `scope()`, `deque`, `channel`, `AtomicCell` |

### Applicable Scenarios
- **Rayon**: CPU-bound data parallelism — vector similarity search, KB embedding batch computation, graph algorithms.
- **Crossbeam**: Concurrent data structures — lock-free queues for EventBus, work-stealing for SEAL pipeline task scheduling, scoped threads for NT-MEMORY parallel ingestion.

### NeoTrix Mapping
- **GWT attention routing**: Rayon — parallel salience scoring across specialist modules.
- **KB embedding batch ops**: Rayon — parallel vector computation for HyperCube.
- **EventBus**: Crossbeam — `SegQueue` for lock-free event dispatch, `channel` for cross-domain messaging.
- **SEAL pipeline**: Crossbeam `scope` — parallel distillation stages with borrowable state.

### Priority: **P1** — Both essential. Rayon for data parallelism, Crossbeam for concurrency primitives.

---

## 5. Rust Numerical Computing: nalgebra vs ndarray

### Core Advantages

| Crate | Core Advantage | Best For |
|-------|---------------|----------|
| **ndarray** | N-dimensional arrays (NumPy equivalent). Flexible slicing, striding. BLAS/OpenBLAS/MKL backend. | General numerical computing, high-dimensional data, tensor ops |
| **nalgebra** | Linear algebra focused. Compile-time shape checking. Column-major. Geometry utilities. | Matrix math, coordinate transforms, small fixed-size operations |

### Applicable Scenarios
- **ndarray**: VSA HyperCube vector operations, embedding similarity computation, Bayesian experiment design (VoI calculations).
- **nalgebra**: E8 Hexagram geometric transformations, perception layer coordinate math, small matrix operations.

### NeoTrix Mapping
- **VSA HyperCube**: ndarray — high-dimensional vector operations, bulk similarity scoring.
- **E8 Hexagram**: nalgebra — geometric reasoning, 8D space transformations.
- **nt_core_self dynamic params**: ndarray — vector math for amplitude/speed/frequency calculations.
- **Both**: Different use cases, not competing. ndarray for bulk, nalgebra for geometric/linear algebra.

### Priority: **P2** — ndarray for VSA, nalgebra for E8 geometry. Both complement.

---

## 6. AI Frameworks: PyTorch vs TensorFlow vs JAX

### Core Advantages

| Framework | Core Advantage | Key Metric (2026) |
|-----------|---------------|-------------------|
| **PyTorch 2.x** | Research default (~75% papers). Dynamic graphs. torch.compile 34% speedup. HuggingFace-first. | 82K stars, 4.21h ResNet-50 100 epochs |
| **TensorFlow 2.x/3.x** | Production deployment. TFLite edge inference. TFX MLOps pipeline. Mature serving. | 189K stars, 5.12h training, 116ms edge inference |
| **JAX 0.5.x** | Functional programming. Native TPU. vmap/pmap. Fastest training (39% JIT speedup). | 32K stars, 3.79h training, AlphaFold-native |

### Applicable Scenarios
- **PyTorch**: Research, NLP, LLM fine-tuning, HuggingFace model integration.
- **TensorFlow**: Mobile/edge deployment, production MLOps, GCP/TPU workloads.
- **JAX**: Custom research, TPU-scale training, scientific computing, functional programming.

### NeoTrix Mapping
- **NT-MIND model evaluation**: PyTorch — HuggingFace integration for SEAL pipeline model testing.
- **NT-IO model adapters**: TensorFlow Lite / ONNX — edge inference for NT-PHYSICAL.
- **NT-CORE VSA**: JAX-style functional approach — immutable VSA vectors, pure transform functions.
- **Note**: NeoTrix is Rust-native; Python AI frameworks accessed via NT-ACT bridge (PyO3/FFI), not embedded.

### Priority: **P2** — Bridge via NT-ACT for model evaluation, not core dependency

---

## 7. LLM Frameworks: LangChain vs LlamaIndex

### Core Advantages

| Framework | Core Advantage | Key Metric |
|-----------|---------------|------------|
| **LangChain (1.0 + LangGraph)** | General-purpose LLM orchestration. 1000+ integrations. LangGraph for stateful agents with checkpointing. | 119K stars, ~10ms overhead/step |
| **LlamaIndex (0.14.x)** | Retrieval-first. Best data ingestion (130+ formats via LlamaParse). Hybrid search, sub-question decomposition. | 44K stars, ~6ms overhead/step, 33% fewer tokens |

### Applicable Scenarios
- **LangChain/LangGraph**: Multi-step agent workflows, tool orchestration, human-in-the-loop, durable state.
- **LlamaIndex**: Document Q&A, knowledge bases, RAG with complex PDFs, retrieval-heavy pipelines.

### NeoTrix Mapping
- **NOT directly adopted** — NeoTrix has native KB (SQLite + FTS5 + embeddings) and SEAL pipeline.
- **Pattern absorption**: LangGraph's checkpoint pattern → NT-MEMORY experience persistence. LlamaIndex's hybrid search → NT-WORLD UnifiedCrawler search.
- **Reference architecture**: When users build LLM apps on NeoTrix, provide LangGraph/LlamaIndex as optional tools via NT-ACT.

### Priority: **P3** — Pattern reference, not direct integration

---

## 8. Multi-Agent Frameworks: AutoGen vs CrewAI vs MetaGPT

### Core Advantages

| Framework | Core Advantage | Best For |
|-----------|---------------|----------|
| **AutoGen/AG2** | Conversational multi-agent. Peer-to-peer NL negotiation. Azure integration. | Research automation, collaborative workflows |
| **CrewAI** | Role-based agent teams. YAML config. Gentle learning curve. MCP protocol support. | Business process automation, content generation |
| **MetaGPT** | SOP-based software company simulation. PM/Architect/Engineer roles. | Code generation, dev pipeline automation |
| **LangGraph** | Graph-based orchestration. Checkpointing, human-in-the-loop. | Production stateful agents |

### Applicable Scenarios
- **AutoGen**: Multi-agent research, dynamic conversation patterns.
- **CrewAI**: Rapid team-based task automation, role specialization.
- **MetaGPT**: Software engineering automation, code review pipelines.
- **LangGraph**: Production agents requiring durable execution and audit trails.

### NeoTrix Mapping
- **NT-ACT orchestration**: Absorb CrewAI's role-based pattern → NT-ACT capability routing.
- **NT-MIND SEAL pipeline**: Absorb LangGraph's checkpoint pattern → SEAL phase persistence.
- **NOT adopted wholesale**: NeoTrix has native GWT attention routing which is more sophisticated than any agent framework's orchestration.
- **Cross-reference**: Agent protocols (A2A, ANP, ACP) inform NT-IO inter-agent communication design.

### Priority: **P3** — Pattern absorption for NT-ACT and NT-MIND, not direct adoption

---

## 9. Vector Databases: Qdrant vs Weaviate vs Milvus

### Core Advantages

| Database | Core Advantage | Key Metric (2026) |
|----------|---------------|-------------------|
| **Qdrant** (Rust) | Fastest single-node. In-graph filtered HNSW. Payload index fused into traversal. Simple setup (5min Docker). | 14,200 QPS, 14GB RAM@5M vectors, p99 42ms |
| **Weaviate** (Go) | Best hybrid search (BM25+vector). Built-in vectorizer modules. ACORN filtered traversal. | 8,100 QPS, 18GB RAM, native multi-tenancy |
| **Milvus** (Go/C++) | Billion-scale. Columnar compression (37% storage savings). GPU indexing. Distributed by design. | 11,600 QPS, 22GB RAM, partition keys |

### Applicable Scenarios
- **Qdrant**: Low-latency filtered RAG, small-mid teams, self-hosted simplicity.
- **Weaviate**: Hybrid search (vector+keyword), complex document pipelines, managed cloud.
- **Milvus**: Billion-scale corpora, multi-tenant enterprise, distributed indexing.

### NeoTrix Mapping
- **KB embedding store**: **Qdrant** — Rust-native (aligns with NeoTrix stack), fastest filtered search, simplest ops. For NT-MEMORY vector similarity.
- **NT-WORLD crawl index**: Qdrant for crawled content embeddings.
- **Skip if <10K docs**: SQLite FTS5 (already in KB) handles small corpora.
- **NOT adopted**: Milvus (overkill for NeoTrix scale), Weaviate (Go, heavier ops).
- **Note**: NeoTrix KB already has SQLite + FTS5; Qdrant would be an *optional* high-performance tier.

### Priority: **P2** — Qdrant as optional KB vector backend when >50K embeddings

---

## 10. Graph Databases: Neo4j vs ArangoDB

### Core Advantages

| Database | Core Advantage | Key Metric (2026) |
|----------|---------------|-------------------|
| **Neo4j 5.x** | Native graph storage. Cypher query language. Rich ecosystem (APOC, Bloom, GDS). 3.2x faster deep traversals. | 8,200 QPS@8-hop, 12.4ms p99 |
| **ArangoDB 3.12** | Multi-model (graph+document+KV). AQL query language. Free clustering in OSS. 40% less RAM for mixed workloads. | 11,300 QPS mixed, SmartGraphs sharding |

### Applicable Scenarios
- **Neo4j**: Pure graph workloads, deep traversals, fraud detection, knowledge graphs, Cypher ecosystem.
- **ArangoDB**: Mixed graph+document workloads, cost-sensitive teams, multi-model flexibility.

### NeoTrix Mapping
- **ConsciousnessTree health graph**: Neo4j — deep cross-domain dependency traversals, module health propagation.
- **NOT adopted**: NeoTrix KB (SQLite) handles graph queries via recursive CTEs for domain relationships. Dedicated graph DB only needed if ConsciousnessTree scales beyond SQLite capability.
- **If needed**: Neo4j for NT-CORE E8 reasoning graph (hexagram relationship traversal).

### Priority: **P3** — Reference only. SQLite recursive CTEs sufficient for current scale.

---

## 11. Architecture Patterns: Clean Architecture vs Hexagonal

### Core Advantages

| Pattern | Core Advantage | When to Use |
|---------|---------------|-------------|
| **Clean Architecture** | 4 concentric rings (Entities→Use Cases→Adapters→Frameworks). Dependency Rule: deps point inward only. Strong domain isolation. | Domain-rich apps, complex business rules, regulated systems, 2-5yr horizon |
| **Hexagonal (Ports & Adapters)** | One cut: application vs adapters. Driving ports (inbound) vs driven ports (outbound). Symmetric treatment of I/O. | Microservices, event-driven systems, frequent infrastructure swaps |

### Key Overlap
Both share: domain at center, deps inward-only, DIP, testability, framework independence.

### Key Differences
- **Depth**: Clean makes 2 cuts (Entities vs Use Cases). Hexagonal makes 1 cut (app vs adapters).
- **Prescriptive vs Descriptive**: Clean is prescriptive (4 named rings). Hexagonal is descriptive (shape metaphor).
- **DTOs**: Clean mandates Input/Output DTOs at boundaries. Hexagonal passes domain types directly.

### NeoTrix Mapping
- **Hybrid**: Clean's internal layering (Entities→Use Cases→Adapters→Frameworks) maps to NeoTrix's L1-L6 architecture.
- **Hexagonal's ports/adapters**: Map to NT-* domain trait interfaces (`ActionLayer`, `PerceptionLayer`, `EmbodimentLayer`).
- **Dependency Rule**: Already enforced — `#![forbid(unsafe_code)]` + domain traits + capability registry.
- **Domain isolation**: Each NT-* domain defines trait interfaces; implementations are adapters.

### Priority: **P0** — Foundational. Already implicit in NeoTrix architecture. Formalize via CONTEXT.md.

---

## 12. CQRS + Event Sourcing

### Core Advantages

| Pattern | Core Advantage | When Worth It |
|---------|---------------|---------------|
| **CQRS** | Separate read/write models. Independent scaling. Multiple read views from same data. | Read/write patterns differ significantly, high read load |
| **Event Sourcing** | Append-only event log as source of truth. Complete audit trail. Temporal queries. Replay to any point. | Regulatory compliance, complex state machines, debugging production issues |

### Key Production Insights (2026)
- Events must be immutable. Never modify stored events — use upcasting.
- Projections must be idempotent (`ON CONFLICT DO UPDATE`).
- Snapshot every N events to avoid replaying 10K+ events.
- `correlationId` + `causationId` in event metadata are mandatory for distributed debugging.
- EventStoreDB rebranded to Kurrent (2025).

### NeoTrix Mapping
- **SEAL Pipeline**: CQRS — write commands (explore/distill/absorb) separate from read queries (health metrics, evolution velocity).
- **Experience Tree**: Event Sourcing — experience entries are immutable events. Replay to reconstruct session state. KB `experience` namespace as event store.
- **ConsciousnessTree**: CQRS — write health signals separate from read dashboards.
- **NOT over-applied**: Use only for SEAL pipeline (complex state machines) and experience persistence (audit trail). Simple CRUD (NT-WORLD fetch config) stays traditional.

### Priority: **P1** — For SEAL pipeline and experience persistence

---

## 13. Microservices vs Modular Monolith

### Core Advantages

| Architecture | Core Advantage | Key Insight (2026) |
|-------------|---------------|-------------------|
| **Modular Monolith** | ~80% of microservices benefits at ~20% cost. Internal module boundaries enable future extraction. Shopify: 284M req/min. | 42% of orgs considering return to monolith. "Boring tech" movement. |
| **Microservices** | Independent deployment, scaling, tech diversity. Best for 50+ developers, orgs with platform engineering. | Only 9% report complete success. 2-3x higher infra cost. |
| **Strangler Fig** | Gradual migration: build alongside, route traffic, retire old code. Most successful migration pattern. | Netflix full migration took 7 years + 30 teams. |

### Decision Thresholds
- **<20 developers**: Modular monolith.
- **20-50 developers**: Modular monolith with selective extraction.
- **50+ developers**: Microservices (with platform engineering investment).
- **Break-even**: $10M+ revenue, 50+ developers for positive microservices ROI.

### NeoTrix Mapping
- **Current**: Modular monolith (neotrix-core crate with domain modules). Correct architecture for current team size.
- **Future**: If NeoTrix scales to multi-team, extract NT-WORLD (crawl pipeline) and NT-IO (LLM providers) as separate services.
- **Pattern**: Each NT-* domain module already has trait boundaries that could become service boundaries.
- **NOT premature**: Keep modular monolith until genuine scaling pressure demands extraction.

### Priority: **P1** — Validates current architecture. Plan for future extraction points.

---

## 14. Actor Model (Rust)

### Core Advantages

| Library | Core Advantage | Key Feature |
|---------|---------------|-------------|
| **Actix** | Mature, fastest messaging. Own runtime (Tokio-based). Robust supervision. | 22K stars, fastest spawn+message |
| **Kameo** | Balanced. Built-in distributed actors. Tokio-native. Good supervision. | Distributed + local, easy API |
| **Ractor** | Lightweight. Async Std + Tokio. Fault tolerance. | Simpler design, single message type |
| **Xtra** | Multi-runtime (Tokio/Async Std/Smol/Wasm). Minimal framework. | Runtime flexibility |
| **Coerce** | Distributed-first on Tokio. | Distribution focus |

### Applicable Scenarios
- **Actix**: High-throughput local actors, message-heavy systems, web server backends.
- **Kameo/Ractor**: Distributed actor systems, cross-node communication.
- **Tokio actor pattern (manual)**: Simple actor-with-handle pattern using `mpsc::channel` — no framework needed for basic cases.

### NeoTrix Mapping
- **GWT specialist modules**: Actor pattern — each NT-* domain as actor processing messages from Global Workspace.
- **NOT Actix actors**: NeoTrix uses Tokio tasks + channels (lighter than full actor framework).
- **Pattern absorption**: Actor supervision → NT-SHIELD fault detection. Mailbox backpressure → EventBus flow control.
- **Existing implementation**: `nt_core_self::AttentionManager` already uses channel-based actor-like pattern.

### Priority: **P2** — Pattern absorbed. No framework dependency needed; Tokio channels sufficient.

---

## 15. Typestate Pattern (Rust)

### Core Advantages

| Aspect | Description |
|--------|-------------|
| **Compile-time safety** | Encode state machine transitions in the type system. Illegal transitions are compile errors. |
| **Zero-cost abstraction** | No runtime overhead — state checks happen at compile time. |
| **Self-documenting** | Type signatures encode valid state transitions. Code reads as documentation. |

### Example Pattern
```rust
struct Locked;
struct Unlocked;

struct Door<S> { _state: PhantomData<S> }

impl Door<Locked> {
    fn unlock(self) -> Door<Unlocked> { Door { _state: PhantomData } }
}

impl Door<Unlocked> {
    fn lock(self) -> Door<Locked> { Door { _state: PhantomData } }
}

// Door<Locked>.lock() → COMPILE ERROR (already locked)
// Door<Unlocked>.unlock() → COMPILE ERROR (already unlocked)
```

### NeoTrix Mapping
- **SEAL pipeline phases**: Typestate for phase transitions (Soil→Roots→Trunk→Branches→Fruits→Core). Prevent skipping phases.
- **Constellation maturity (C0-C6)**: Typestate for compile-time enforcement of maturity progression.
- **SelfTest wiring (T1→T2→T3)**: Typestate for ensuring tests are registered before being called.
- **KB transaction safety**: Typestate for read/write/transaction mode enforcement.
- **Already partially used**: NeoTrix's trait-based architecture (ActionLayer, PerceptionLayer) is a soft form of typestate.

### Priority: **P1** — Adopt for SEAL pipeline phases and SelfTest wiring progression

---

## Cross-Cutting Summary

### Priority Distribution

| Priority | Topics | NeoTrix Components |
|----------|--------|-------------------|
| **P0** | Axum, Tokio, Clean/Hexagonal Architecture | NT-IO web server, NT-WORLD HTTP, core architecture |
| **P1** | Serde/Bincode, Rayon/Crossbeam, CQRS/ES, Modular Monolith, Typestate | KB persistence, EventBus, SEAL pipeline, experience tree |
| **P2** | ndarray/nalgebra, Qdrant, Actor Model, AI framework bridge | VSA HyperCube, vector search, GWT attention, model evaluation |
| **P3** | LangChain/LlamaIndex, Agent frameworks, Neo4j/ArangoDB | Pattern references, not direct adoption |

### Technology Stack Confirmation

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Async Runtime** | Tokio | Non-negotiable ecosystem standard |
| **Web Framework** | Axum | Tower ecosystem, type-safe, Tokio-native |
| **Serialization** | Serde + Bincode (internal) / Protobuf (external) | Performance + cross-language |
| **Concurrency** | Rayon + Crossbeam | Data parallelism + concurrency primitives |
| **Numerical** | ndarray (bulk) + nalgebra (geometric) | Complementary, not competing |
| **Vector Search** | Qdrant (optional tier) + SQLite FTS5 (default) | Scale-appropriate |
| **Architecture** | Clean Architecture + Hexagonal hybrid | Domain isolation + port/adapter interfaces |
| **State Management** | CQRS + Event Sourcing (SEAL/experience only) | Complexity justifies for state machines |
| **Deployment** | Modular monolith | Correct for current scale |
| **Type Safety** | Typestate pattern | Compile-time state machine enforcement |

---

*Generated by Research Batch E-214 | NeoTrix Knowledge Absorption*

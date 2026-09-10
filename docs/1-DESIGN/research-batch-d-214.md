# Research Batch D (214) — 8-Topic External Research

**Generated**: 2026-09-10 | **Sources**: 40+ high-quality references across 8 advanced topics

---

## 1. Rust Error Handling (thiserror / anyhow)

| # | Title | URL | Core Contribution | NeoTrix Mapping | Priority |
|---|-------|-----|-------------------|-----------------|----------|
| 1 | Rust Error Handling Patterns for Production Applications | https://andrewodendaal.com/rust-error-handling-patterns-production/ | Canonical split: `thiserror` for libraries (typed enums, caller can match), `anyhow` for applications (context chaining). `?` operator as composable conversion pipeline via `From`. `.with_context()` only allocates on error path. Layer-per-error-enum with `From` propagation. | NT-CORE error type design — each layer defines its own `thiserror` enum, application wraps with `anyhow`. Aligns with R-P1 zero-unsafe. | P0 |
| 2 | Rust Error Handling in 2026: Result, Option, thiserror and anyhow | https://sharpskill.dev/en/blog/rust/rust-error-handling-result-option-thiserror-anyhow | `Result` + `Option` sum types enforced at compile time. `thiserror` derives `Display`/`Error`/`From` via proc macro — zero runtime cost. `anyhow::Context` trait adds `.context()`/`.with_context()`. `bail!`/`ensure!` macros for early exit. Async `?` works identically. | SEAL pipeline error handling — `thiserror` for domain errors, `anyhow` in CLI/binary layer. | P0 |
| 3 | Rust for TS/JS Developers — anyhow & thiserror | https://rs4ts.dev/08-error-handling/06-anyhow-thiserror/ | Decision rule: does any caller need to match? Yes → `thiserror`. No → `anyhow`. `#[non_exhaustive]` for library error enums (semver safety). `#[error(transparent)]` for pass-through variants. `{:#}` format shows full chain. `Send + Sync + 'static` required by async runtimes — `thiserror` derives satisfy automatically. | NT-IO / NT-ACT public APIs: expose `thiserror` enums; internal logic uses `anyhow`. | P1 |
| 4 | Rust Error Handling: anyhow vs thiserror Guide | https://krun.pro/rust-anyhow-vs-thiserror/ | `Box<dyn Error>` as simplest type erasure. `thiserror` vs `anyhow` = caller needs to match vs doesn't. `.context()` wraps error preserving original as `source()`. Heap allocation only on error path — irrelevant cost. Community convention confirmed: libraries → `thiserror`, applications → `anyhow`. | Consistent with existing NeoTrix crate error patterns. Audit: ensure no `anyhow` in public lib APIs. | P1 |
| 5 | Rust for TS/JS Developers — Best Practices | https://rs4ts.dev/08-error-handling/08-best-practices/ | Match granularity to caller behavior (not message text). Write error messages like log lines (lowercase, no period). `main()` returns `anyhow::Result<()>` for free handler. Panic for bugs, `Result` for conditions. Clippy lint flags `.unwrap()` in non-test code. | CI policy: enforce no `.unwrap()` outside test modules (R-P1 alignment). | P2 |

**Synthesis**: The Rust ecosystem has fully converged on `thiserror` (libraries) + `anyhow` (applications) as the standard pattern. Zero runtime cost on success path. Key discipline: `.with_context()` at trust boundaries, `#[non_exhaustive]` for public enums, clippy lint for unwrap-free production code.

---

## 2. AI Agent Tool Use Architecture

| # | Title | URL | Core Contribution | NeoTrix Mapping | Priority |
|---|-------|-----|-------------------|-----------------|----------|
| 1 | Harness Engineering via Agent-Native Reusable Tool Primitives (HEART) | https://arxiv.org/abs/2609.01736 | Tool Primitives: natural language interface replaces rigid API schemas. Each tool wrapped with LLM interface handling schema resolution internally. ToolFace: centralized repo of 25,519 functions, LLM retrieves relevant tools at inference (no full enumeration in context). HEART framework: Planner + Router + Verifier. | NT-ACT tool registry: dynamic retrieval instead of enumerating all tools in context. GWT salience for tool selection. | P0 |
| 2 | HyperAgent: Tool-Schema Hypergraph for Tool-Use LLM Agents | https://arxiv.org/html/2608.02650 | Tool-Schema Hypergraph (TSH): tools as hyperedges from input-schema nodes to output-schema nodes. Deficit-Oriented Expansion: beam search to find minimal tool support subgraph. Reduces redundant API calls, LLM interactions, token consumption. | CapabilityBridge alignment: schema-level tool dependency modeling. Tool composition as hypergraph traversal. | P1 |
| 3 | STEM Agent: Self-adapting, Tool-enabled, Extensible, Multi-agent | https://arxiv.org/pdf/2603.22359 | 5-layer architecture: Caller → Standard Interface (5 protocols: A2A, AG-UI, A2UI, UCP, AP2) → Agent Core → Memory → MCP Integration. MCP-native: all external capabilities acquired at runtime via MCP. Biologically-inspired skills: cell differentiation model for skill crystallization (progenitor → committed → mature → apoptosis). | NT-IO MCP integration, NT-MIND skill crystallization (SEAL pipeline), ConsciousnessTree growth lifecycle. | P0 |
| 4 | AI Agent Architecture in 2026: The Practical Reference | https://arahi.ai/blog/ai-agent-architecture | 6 architectural layers: perception, reasoning, planning, memory, tool use, oversight. 5 canonical architectures: ReAct, Plan-Execute, Reflexion, Tree-of-Thoughts, Multi-Agent. MCP as universal tool interface in 2026. Clean boundaries > clever patterns. | Six-Layer Architecture validation. Tool architecture: single-responsibility tools, structured results, high-stakes confirmation. | P1 |
| 5 | Procedural Graphs: Self-Evolving Execution Structures for LLM Agents | https://arxiv.org/abs/2609.09153 | Procedural Graph: organizes procedural knowledge into (procedure, relation, procedure) triplets. Self-evolving: LLM refiner edits graph topology from failed/successful trajectories. Guidance model translates subgraph into step-level situational guidance. | ConsciousnessTree as procedural graph. Experience-tree absorption: failed trajectories inform graph edits. | P2 |

**Synthesis**: 2026 agent tool architecture converges on: (1) MCP as universal tool protocol, (2) schema-aware tool composition (not flat enumeration), (3) natural language tool interfaces over rigid schemas, (4) self-evolving tool graphs from execution traces. NeoTrix's GWT + CapabilityBridge aligns with TSH/HEART patterns.

---

## 3. Knowledge Graph Query Optimization

| # | Title | URL | Core Contribution | NeoTrix Mapping | Priority |
|---|-------|-----|-------------------|-----------------|----------|
| 1 | LogosKG: Hardware-Optimized Scalable KG Retrieval | https://aclanthology.org/2026.acl-long.1966/ | Hardware-aligned k-hop retrieval: degree-aware partitioning, cross-graph routing, on-demand caching. Billion-edge scale. KG-LLM interaction: KG topology shapes LLM diagnostic reasoning alignment. | NT-MEMORY KB query optimization. HyperCube traversal could adopt degree-aware partitioning for large node graphs. | P1 |
| 2 | GOpt: Modular Graph-Native Query Optimization Framework | https://doi.org/10.1145/3722212.3724425 | Decoupled optimization: GraphIrBuilder converts multi-language queries to unified GIR (Graph Intermediate Representation). PhysicalConverter bridges to backends. Heuristic rules + cost-based optimization for Complex Graph Patterns. Neo4j 9.2x, GraphScope 33.4x speedup. | KB query layer: GIR pattern for unified query planning across different storage backends (SQLite, vector, graph). | P1 |
| 3 | KGCache: Amortized Subgraph Retrieval for KG Reasoning | https://arxiv.org/html/2608.07954 | Entity-level caching for KGQA: one-hop neighborhoods cached at entity granularity. Semantic-context caching for similar queries. LRU/LFU policies. 1.91x KG speedup, 38% backend query reduction. Complementary to iterative (ToG) and one-shot (RoG) paradigms. | NT-MEMORY KB caching: entity-level cache for frequently accessed KB nodes. Semantic caching for repeated query patterns. | P1 |
| 4 | UniQGen: Constraint-guided LLM Agents for Graph Query Generation | https://www.alphaxiv.org/abs/2605.00845 | Chase & Backchase algorithm extended with LLM agents for dynamic query constraint refinement. Works across SPARQL/Cypher/property graphs. F1 gains: 31.6% on GraphQ, 4.9% on GrailQA. No fine-tuning for schema matching. | KB query generation: LLM-assisted query construction for NeoTrix's SQLite KB. | P2 |
| 5 | R2O: Dual-Layer Framework for Distributed Property Graph Query Optimization | https://doi.org/10.1145/3786685 | Partition-aware query rewriting + GNN/reinforcement learning for join ordering. 1-2 orders of magnitude speedup on billion-scale graphs. Compatible with different partitioning methods. | KB scaling: if NeoTrix KB grows to billion-scale, partition-aware rewriting pattern applies. | P3 |

**Synthesis**: KG optimization in 2026 focuses on (1) hardware-aware partitioning, (2) entity-level caching, (3) LLM-assisted query generation, (4) decoupled optimization layers. NeoTrix's KB benefits from entity caching and GIR-style unified query planning.

---

## 4. Distributed Systems Event-Driven

| # | Title | URL | Core Contribution | NeoTrix Mapping | Priority |
|---|-------|-----|-------------------|-----------------|----------|
| 1 | Event-Driven Architecture in 2026: Kafka, Streaming SQL, and the AI Layer | https://risingwave.com/blog/event-driven-architecture-2026/ | 2026 EDA standard: Kafka (events) → RisingWave (streaming SQL materialized views) → AI Agents (query via MCP). Three event patterns: point-to-point, stream processing, agent queries. Streaming database as queryable live state for agents. | NT-WORLD data pipeline: event-driven crawl pipeline with materialized view layer. Agent queries via MCP. | P0 |
| 2 | Event-Driven Architecture in 2026: Patterns, Tools, and When to Use | https://encore.dev/articles/event-driven-architecture | Comprehensive EDA guide: Pub/Sub as primitive, event sourcing, CQRS, saga patterns. Broker selection: Kafka (high-throughput), Redpanda (simpler ops), NATS (edge/IoT), SNS+SQS (AWS default). Idempotency + DLQ from day one. Schema registry with CI gating. | EventBus architecture validation. Event catalog + schema registry for NT-* inter-domain events. | P1 |
| 3 | Event-Driven Architecture in 2026: CloudEvents, Kafka, Async APIs | https://kawaldeepsingh.medium.com/event-driven-architecture-in-2026-cloudevents-kafka-async-apis-and-practical-patterns-for-52a122b64171 | CloudEvents standard envelope. Outbox pattern for exactly-once. Consumer contract tests in CI. 6-step adoption roadmap. Choreography vs orchestration hybrid: choreography for fan-out, orchestration (Temporal) for multi-step transactions. | NT-ACT orchestration: Temporal-style workflow for complex cross-domain tasks. CloudEvents for inter-service events. | P1 |
| 4 | Distributed Complex Event Processing for Intelligent Environments | https://link.springer.com/article/10.1007/s00607-026-01713-1 | Edge-fog-cloud DCEP: 3-layer hierarchical event processing. Kafka as messaging backbone. Esper CEP engine. 68K events/sec throughput. Domain decomposition across heterogeneous nodes. | NT-WORLD edge processing: hierarchical event processing for IoT/sensor data ingestion. | P2 |
| 5 | Event-Driven Platform Engineering: Always-On Consumer Systems | https://jisem-journal.com/index.php/journal/article/view/14374 | Exactly-once semantics via transactional outbox + CDC. Sub-second latency with strict ordering. Schema evolution strategies. Late-arriving data handling. Reference architecture for production EDA. | NT-ACT exactly-once guarantees for critical workflows (experience-tree absorption, KB writes). | P2 |

**Synthesis**: 2026 EDA = events + streaming SQL + agent queries. Key patterns: outbox for exactly-once, CloudEvents envelope, schema registry in CI, DLQ from day one. NeoTrix EventBus should adopt materialized view pattern for GWT attention routing.

---

## 5. Type-Safe API Design (Rust)

| # | Title | URL | Core Contribution | NeoTrix Mapping | Priority |
|---|-------|-----|-------------------|-----------------|----------|
| 1 | Type-Driven API Design in Rust (Book) | https://willcrichton.net/rust-api-type-patterns/introduction.html | Central theme: replace dynamically-typed abstractions with statically-typed ones. Enums over strings (compiler verifies inputs). "Parse, don't validate" — separate parsing from API contract. Guards pattern: witness + mediated access (MutexGuard). | NT-ACT tool interfaces: typed tool parameters (enums over stringly-typed). State machine guards for capability transitions. | P0 |
| 2 | Rust API Guidelines: Type Safety | https://rust-lang.github.io/api-guidelines/type-safety.html | C-NEWTYPE: `Kilometers` vs `Miles` at compile time. C-CUSTOM-TYPE: `Small`/`Round` over `bool`. C-BITFLAG: `bitflags` crate for flag sets. C-BUILDER: non-consuming (preferred) vs consuming builders. | NT-* domain type design: newtypes for domain values, bitflags for capability sets, builder pattern for complex configurations. | P0 |
| 3 | Type-Driven API Design in Rust — Guards | https://willcrichton.net/rust-api-type-patterns/guards.html | Guard = witness + mediated access + RAII cleanup. MutexGuard enforces: access control (borrow tied to guard lifetime), cleanup (Drop unlocks). Closure guards for scoped access. | NT-SHIELD security guards, NT-PHYSICAL resource guards. RAII pattern for capability lifecycle. | P1 |
| 4 | Typeway: Type-Level Web Framework | https://github.com/joshburgess/typeway | API as single Rust type. REST server, REST client, OpenAPI, gRPC server, gRPC client, .proto all derived from one type definition. Compile-time handler completeness. Type-level API versioning (V2 as typed delta of V1). | NT-IO API design: type-derived endpoints. API evolution as typed deltas. gRPC + REST co-serving from single definition. | P2 |
| 5 | Rust API Guidelines: Builders | https://docs.rs/api-guidelines/latest/api_guidelines/enum.TypeSafety.html | Builder pattern for complex data structures. Non-consuming builders preferred (terminal method takes `&self`). Consuming builders: all methods take/return owned `self` for one-liner ergonomics. | NT-MEMORY KB query builders, NT-WORLD crawl configuration builders. | P2 |

**Synthesis**: Type-driven design in Rust = newtypes, enums, guards, builders. Compile-time enforcement eliminates runtime validation. NeoTrix should use typed tool parameters (not strings), RAII guards for capabilities, and builder pattern for complex configurations.

---

## 6. AI Memory Architecture Survey

| # | Title | URL | Core Contribution | NeoTrix Mapping | Priority |
|---|-------|-----|-------------------|-----------------|----------|
| 1 | Memory Architectures for AI Agents in 2026: A Survey, Synthesis, and Evaluation Framework | https://doi.org/10.5281/zenodo.20618995 | Survey of 19 memory systems. Unified five-layer architecture. Seven-dimension design-time evaluation framework. Covers academic, industrial, and PKM communities. | NT-MEMORY / NT-NEXUS: five-layer memory architecture alignment. Evaluation framework for memory subsystem quality. | P0 |
| 2 | From Storage to Experience: Evolution of LLM Agent Memory (ACL 2026) | https://aclanthology.org/2026.findings-acl.2069.pdf | Three evolutionary stages: Storage (trajectory preservation) → Reflection (trajectory refinement) → Experience (trajectory abstraction). Active exploration + cross-trajectory abstraction as Experience-stage mechanisms. Memory = substrate for agent self-evolution. | SEAL pipeline: Storage → Distillation → Experience stages map directly. Experience-tree = cross-trajectory abstraction. | P0 |
| 3 | Anatomy of Agentic Memory: Taxonomy and Empirical Analysis | https://arxiv.org/abs/2602.19320 | 4 taxonomy categories of MAG systems. Context Saturation Gap (Δ) metric: external memory must provide structural advantage. LLM-as-judge evaluation misalignment with lexical metrics. Memory backbone sensitivity: open-weight models show "silent failure". | NT-MEMORY evaluation: Δ metric for KB usefulness. Backbone-aware memory operations. | P1 |
| 4 | Foundation Agent Memory Survey (3-axis taxonomy) | https://arxiv.org/pdf/2602.06052 | Three dimensions: substrate (internal/external), mechanism (sensory/working/episodic/semantic/procedural), subject (user-centric/agent-centric). Memory as self-evolution substrate. Working memory gates perception; long-term memory accumulates skills. Three architecture patterns: (A) context-only, (B) context + retrieval store, (C) tiered with learned control. | NT-* architecture: Pattern C (tiered memory) = HyperCube (semantic) + KB (episodic) + context window (working). | P0 |
| 5 | Memory Survey: Write-Manage-Read Loop (2022-2026) | https://arxiv.org/pdf/2603.07670 | Formalized as write–manage–read loop in POMDP cycle. Three-dimensional taxonomy: temporal scope, representational substrate, control policy. Five mechanism families: context compression, RAG stores, reflective self-improvement, hierarchical virtual context, policy-learned management. Learned memory control (AgeMem): RL-trained store/retrieve/update/summarize/discard. | NT-MEMORY operations: write (experience-tree), manage (KB consolidation), read (GWT routing). AgeMem pattern for learned memory management. | P1 |

**Synthesis**: 2026 AI memory = Storage → Reflection → Experience evolution. Pattern C (tiered with learned control) is the frontier. Key insight: memory is not passive storage but the substrate for agent self-evolution. NeoTrix's experience-tree + KB + HyperCube directly maps to this architecture.

---

## 7. Plugin System Architecture

| # | Title | URL | Core Contribution | NeoTrix Mapping | Priority |
|---|-------|-----|-------------------|-----------------|----------|
| 1 | Rust Plugin System Architecture (corvid-agent) | https://github.com/CorvidLabs/corvid-agent/blob/v0.63.0/PLUGIN_SYSTEM_DESIGN.md | Trait-based plugin interface + WASM sandbox. Capability-based security (PLUGIN.yaml declarations). Isolated DB tables per plugin. 3 crate structure: plugin-api, plugin-runtime, server integration. External plugin repos with version management. | NT-ACT tool/plugin architecture: trait-based plugins, capability declarations, DB isolation per domain. | P0 |
| 2 | How to Build a Plugin System in Rust (Arroyo) | https://www.arroyo.dev/blog/rust-plugin-systems/ | FFI-based dynamic loading via `dlopen2`. C ABI for cross-compiler compatibility. `#[repr(C)]` data types. `cdylib` crate type. `catch_unwind` across FFI boundary. Metadata + entrypoint contract. | NT-ACT dynamic tool loading: C ABI FFI for cross-language plugins. Panic isolation at boundaries. | P1 |
| 3 | CPEX Rust: Core Plugin Runtime | https://github.com/contextforge-org/contextforge-plugins-framework/blob/dev/docs/specs/cpex-rust-spec.md | 5-phase pipeline: Sequential → Transform → Audit → Concurrent → FireAndForget. Typed dispatch as default (compile-time payload/hook compatibility). Capability-gated writes with WriteToken. Async-by-default handlers (zero-cost when no await). | NT-ACT tool execution pipeline: phase-based execution modes. Typed tool invocations. WriteToken for KB mutations. | P0 |
| 4 | Plugin and Extension System Design (AI VOID SSG) | https://aivoid.dev/stellar-gen-guide/chapter-12-plugin-system/ | Plugin trait with lifecycle hooks: on_init, on_content_processed, on_page_rendered, on_site_finished. PluginRegistry of `Box<dyn Plugin>`. Sequential processing for mutable plugin correctness. | NT-MIND SEAL pipeline hooks: plugin-like lifecycle for evolution stages. | P2 |
| 5 | pluggable: Async Plugin System for Rust | https://github.com/tomerlichtash/pluggable | Async-first plugin execution. Dependency management with topological sorting. Parallel execution of independent plugins. Capability-based permissions + sandboxing. Event system for inter-plugin communication. | NT-ACT dependency-ordered tool execution. Parallel tool invocation where dependencies allow. | P1 |

**Synthesis**: 2026 Rust plugin patterns: (1) trait-based with lifecycle hooks, (2) WASM sandbox for untrusted, (3) capability-based security declarations, (4) typed dispatch (compile-time safety), (5) 5-phase execution pipeline. NeoTrix tool system aligns with CPEX 5-phase model.

---

## 8. Rust Performance Optimization

| # | Title | URL | Core Contribution | NeoTrix Mapping | Priority |
|---|-------|-----|-------------------|-----------------|----------|
| 1 | Rust Performance Optimization: A Systems Engineer's Guide | https://krun.pro/rust-performance-optimization/ | Debug builds 10-100x slower. Heap alloc in hot loop = suicide. `Box<dyn Trait>` vtable kills branch prediction; generics faster. Cache locality > algorithmic complexity at small N. jemalloc/mimalloc cuts alloc latency 30-60%. `cargo-flamegraph` + Criterion.rs as standard toolchain. | All NT-* hot paths: flamegraph profiling mandatory before optimization. allocator audit for high-churn paths. | P0 |
| 2 | Performance — Rust for TS/JS Developers | https://rs4ts.dev/21-performance/ | Profiling-first: samply/perf/Instruments. `black_box` prevents compiler elision in benchmarks. Memory layout: struct field ordering, niche optimization (`Option<NonZero*>` = same size as base). SoA vs AoS for cache efficiency. Iterators compile to same code as hand-written loops (zero-cost confirmed). | NT-* data structures: SoA for entity processing, niche optimization for enum-heavy code, iterators over index loops. | P0 |
| 3 | Rust Performance Optimization: Complete Guide 2026 | https://reintech.io/blog/rust-performance-optimization-complete-guide-2026 | `Vec::with_capacity()` + `.clear()` pattern. FxHashMap for integer keys. `#[inline]` hints for cross-crate hot functions. PGO: 10-20% improvement. Profile-guided: `opt-level = "z"` for binary size, LTO for release. Rayon for CPU-parallel, async for I/O-parallel only. | Cargo.toml profile tuning for NeoTrix releases. PGO for CLI binary. FxHashMap for ID-keyed lookups. | P1 |
| 4 | How to Profile and Optimize Rust Code | https://oneuptime.com/blog/post/2026-02-01-rust-profiling-optimization/view | Workflow: benchmark → profile → flamegraph → memory profile → optimize → verify. DHAT for allocation profiling. Clone-hotspot detection. Arrays over vectors for fixed sizes. Release mode + LTO mandatory. | Mandatory workflow before any NeoTrix performance optimization. | P0 |
| 5 | Performance and Compiler Optimization (DeepWiki) | https://deepwiki.com/jason931225/rust-skills/6.2-performance-and-compiler-optimization | SIMD tiers: autovectorization → portable SIMD (`wide`/`std::simd`) → intrinsics. `target-cpu=native` for known deployments, fleet baseline (x86-64-v3) for distributed. Hot/cold field splitting. `std::hint::cold_path()` (stable 1.95). `#[cold]` for error constructors. | NT-CORE hot paths: cold_path for error paths, hot/cold splitting for Entity-like structs, SIMD for HyperCube operations. | P1 |

**Synthesis**: 2026 Rust perf = profile-first (never guess), allocator audit (jemalloc default), cache-friendly layouts (SoA), zero-cost iterators confirmed, SIMD tiers progressive. Key rule: `--release` + `target-cpu=native` + flamegraph before any optimization attempt.

---

## Cross-Topic Synthesis: NeoTrix Priority Actions

### P0 — Immediate (Next Sprint)

1. **Error handling standard**: Enforce `thiserror` (libs) + `anyhow` (apps) across all crates. CI clippy lint for unwrap-free non-test code.
2. **Tool architecture**: Adopt MCP as universal tool interface. Dynamic tool retrieval (HEART/ToolFace pattern) instead of flat enumeration.
3. **Memory architecture**: Implement 3-tier memory (context + retrieval store + cold archive). Experience-tree = cross-trajectory abstraction layer.
4. **Plugin system**: CPEX 5-phase pipeline (Sequential → Transform → Audit → Concurrent → FireAndForget). Typed tool dispatch.
5. **Performance baseline**: mandatory flamegraph profiling before optimization. jemalloc as default allocator.

### P1 — Near-Term

1. **KG optimization**: Entity-level caching for frequently accessed KB nodes. GIR-style unified query planning.
2. **Event-driven**: Outbox pattern for exactly-once KB writes. CloudEvents envelope for inter-domain events.
3. **Type safety**: Newtypes for domain values, typed tool parameters, RAII guards for capabilities.
4. **Plugin security**: Capability declarations per tool/domain, DB isolation per plugin.

### P2 — Future

1. **Learned memory control**: AgeMem-style RL-trained memory operations.
2. **Procedural graphs**: Self-evolving execution structures from success/failure traces.
3. **SIMD optimization**: Portable SIMD for HyperCube operations after autovectorization confirmed insufficient.
4. **API-as-type**: Typeway-style type-derived API endpoints for NT-IO.

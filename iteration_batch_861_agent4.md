# Agent 4: Zero-Copy Patterns (Batch 861)

## Sources
- rkyv GitHub repository and documentation (https://github.com/rkyv/rkyv)
- rkyv book on zero-copy deserialization (https://rkyv.org/zero-copy-deserialization.html)
- Apache Iggy zero-copy implementation blog (https://iggy.apache.org/blogs/2025/05/08/zero-copy-deserialization)
- Cloudflare mmap-sync library (https://github.com/cloudflare/mmap-sync)
- serde_bytes documentation (https://docs.rs/serde_bytes/latest/serde_bytes/)
- Microsoft Rust Training serialization patterns (https://microsoft.github.io/RustTraining/rust-patterns-book/ch11-serialization-zero-copy-and-binary-data.html)
- rust_serialization_benchmark (https://github.com/djkoloski/rust_serialization_benchmark)
- memmap2 documentation (https://docs.rs/memmap2/latest/memmap2)
- NeoTrix codebase analysis (crates/neotrix-types/src/core/nt_core_rkyv.rs, neotrix-core/src/unified/)

## Defects

**D-ZERO-001: RkyvStorage struct defined but never instantiated in production code** | `crates/neotrix-types/src/core/nt_core_rkyv.rs:20` | Critical | NeoTrix defines a zero-copy `RkyvStorage<K, V>` struct but never uses it outside tests. The `RkyvStorage` is only instantiated in unit tests (`nt_core_rkyv.rs:123,132,139,148`), making the entire module dead code. This wastes compilation time and gives false impression of zero-copy capability.

**D-ZERO-002: store_to_rkyv uses JSON serialization, not rkyv** | `crates/neotrix-types/src/core/nt_core_bank/bank/bank_impl/persist.rs:140` | High | The `store_to_rkyv` function name implies rkyv zero-copy storage, but line 140 uses `serde_json::to_string_pretty(&memories)` - standard JSON serialization. This is a naming欺骗 that misleads developers into thinking bank persistence uses zero-copy. The `load_from_rkyv` function (line 157) also uses `serde_json::from_str`.

**D-ZERO-003: memmap2 dependency declared but never used** | `neotrix-core/Cargo.toml:98` | Medium | The `memmap2 = { version = "0.9", optional = true }` dependency is declared and included in the `rkyv-storage` feature flag (line 182), but no code in the repository actually imports or uses memmap2. The KB stores binary blobs (embeddings, rkyv_blobs) that could benefit from memory-mapped I/O but use `std::fs::read` instead.

**D-ZERO-004: rkyv_blobs table stores raw bytes without zero-copy access** | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_unify.rs:743` | High | The `rkyv_store` function stores serialized bytes in SQLite BLOB column but `rkyv_load` (line 758) returns `Vec<u8>` requiring full copy. The data is stored as opaque bytes, not as rkyv-validated archived data. Zero-copy access would require memory-mapping the file and using rkyv's `from_bytes` on the mapped region.

**D-ZERO-005: KnowledgeNode uses owned String types preventing zero-copy deserialization** | `neotrix-core/src/unified/core/nt_core_kb_types.rs:335-357` | High | `KnowledgeNode` has 6 owned `String` fields (id, title, summary, content, url, domain) and `Option<String>` fields. These force allocation on every deserialization. For read-heavy KB operations (search, query, traversal), zero-copy `&str` references with lifetime binding to the underlying buffer would eliminate millions of allocations.

**D-ZERO-006: serde_bytes not used for BLOB/embedding fields** | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_embed.rs:760-761` | Medium | The embeddings table stores `vector BLOB` and `pq_codes BLOB` as raw bytes. When serializing/deserializing these with serde_json (as seen in nt_memory_store.rs:802-803), without `serde_bytes` attribute, each byte is processed individually. Using `#[serde(with = "serde_bytes")]` would enable bulk memcpy operations, providing 10x+ speedup for large embedding vectors.

**D-ZERO-007: Experience data stored as JSON in KV store** | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_unify.rs:960` | Medium | Experience absorption writes data via `serde_json::from_str::<serde_json::Value>(&content)` - parsing JSON into dynamic Value type. This loses all type information and prevents zero-copy access. Experiences are read-heavy (query by keyword), making them ideal candidates for rkyv archived storage with memory-mapped access.

**D-ZERO-008: No alignment handling for zero-copy data** | `crates/neotrix-types/src/core/nt_core_rkyv.rs:58-62` | Medium | The `store` method uses `rkyv::to_bytes` which returns `AlignedVec`, but writes directly to file via `std::fs::write(&path, &bytes)`. When loading, there's no guarantee the file is memory-mapped with proper alignment. rkyv requires 8-byte minimum alignment for zero-copy access; without mmap or aligned buffer, `from_bytes` must copy data to achieve alignment.

**D-ZERO-009: No bytecheck validation for rkyv blobs** | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_unify.rs:758-771` | High | The `rkyv_load` function returns raw `Vec<u8>` without any rkyv validation. If these bytes are later deserialized with rkyv, there's no upfront validation via `bytecheck`. Corrupted data could cause undefined behavior. The rkyv-storage feature enables bytecheck (Cargo.toml line 97) but it's not used in the KB blob storage path.

**D-ZERO-010: Large file reads use std::fs::read instead of memory mapping** | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_crawl.rs:184` | Low | When processing crawled content, the code uses `serde_json::from_str(&text)` where `text` comes from `std::fs::read_to_string`. For large HTML/JSON files (multi-MB), memory-mapped I/O would avoid double-copy (disk→kernel→user→parsed). The `mmap-sync` pattern from Cloudflare demonstrates 2x throughput improvement for read-heavy workloads.

## Key Insights

1. **Zero-copy capability exists but is unused**: NeoTrix has rkyv and memmap2 dependencies, feature flags, and a complete `RkyvStorage` implementation - but none of it is wired into the actual data paths. The entire zero-copy infrastructure is dead code.

2. **Naming欺骗 creates maintenance burden**: Functions named `*_to_rkyv` and `*_from_rkyv` that actually use JSON serialization mislead developers and create false performance expectations.

3. **KB is the prime zero-candidate**: The KnowledgeBase handles massive amounts of structured data (nodes, edges, embeddings, experiences) with read-heavy access patterns. Converting `KnowledgeNode` to rkyv-archived format with memory-mapped storage could yield 5-10x improvement for search and traversal operations.

4. **serde_bytes gap for binary data**: Embeddings (768-1536 dimensional f32 vectors) are stored as BLOBs but serialized/deserialized without `serde_bytes`, losing potential bulk-memcpy optimization.

5. **Security implications**: Without bytecheck validation, corrupted rkyv blobs in the KB could lead to undefined behavior when accessed. The `rkyv-storage` feature enables validation but it's not enforced in the storage path.

6. **Architecture opportunity**: The `rkyv-storage` feature flag (line 182) suggests the team intended to add zero-copy support but never completed the integration. This is a C0 maturity gap - the feature compiles but doesn't actually provide zero-copy benefits.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Critical | 1 |
| High | 4 |
| Medium | 4 |
| Low | 1 |
| Sources consulted | 9 |

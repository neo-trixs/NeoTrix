# Agent 3: Database Patterns (Batch 863)

## Sources
1. SQLx.dev - The Guide to SQLx in Rust (2026)
2. rustify.rs - SQLx vs Diesel vs SeaORM Comparison 2026
3. byteiota.com - Rust ORMs 2026 Comparison
4. reintech.io - Diesel vs SQLx vs SeaORM Comparison 2026
5. docs.rs/sqlx - Pool and PoolOptions documentation
6. oneuptime.com - Database Connection Pooling in Rust with SQLx
7. leapcell.io - Efficient Database Connection Management with sqlx and bb8/deadpool
8. rs4ts.dev - Rust Databases: SQLx, Diesel & SeaORM
9. sea-ql.org - SeaORM Documentation
10. GitHub launchbadge/sqlx - Migration patterns and best practices

## Defects

D-DB-001: NeoTrix KB uses synchronous `rusqlite::Connection` directly without connection pooling, violating async database best practices. All 70+ database operations in `nt_memory_kb/` block the Tokio async runtime. | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_store.rs:4` | HIGH | industry-patterns

D-DB-002: No connection pool configuration exists. Industry standard requires `PgPoolOptions`/pool sizing formula (`(CPU cores * 2) + 1`), idle timeout, max lifetime, and acquire timeout. NeoTrix opens raw `Connection::open()` per operation with zero pooling. | `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:158-163` | HIGH | sqlx-docs

D-DB-003: Schema initialization uses `CREATE TABLE IF NOT EXISTS` without versioned migration tracking. While `SCHEMA_VERSION=8` exists, there is no up/down migration system. Schema changes require manual DDL modification rather than auditable migration files. | `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:154-168` | MEDIUM | github-sqlx-migrations

D-DB-004: `schema_initialize` is called repeatedly across 20+ call sites (consciousness_core, e8_predictor, rule_memory, state, etc.) without connection reuse or OnceLock guard. Each call executes `PRAGMA journal_mode=WAL` + full DDL batch, causing redundant I/O. | `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:168-169` | MEDIUM | sqlx-best-practices

D-DB-005: FTS5 virtual table `nodes_fts` uses `porter unicode61` tokenizer without language-aware configuration. NeoTrix stores CJK content (Chinese/JP) but FTS5 porter stemmer cannot handle CJK tokenization, causing missed search hits for Chinese queries. | `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:243-246` | MEDIUM | sqlite-fts5-docs

D-DB-006: `insert_node_rows` uses `last_insert_rowid()` for FTS rowid mapping, which is fragile when FTS5 content-sync is disabled. If FTS table gets out of sync with nodes table, orphaned FTS rows accumulate silently. No integrity check exists. | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_store.rs:67-71` | MEDIUM | sqlite-patterns

D-DB-007: `KnowledgeStorage` (knowledge_storage.rs) implements custom JSONL journal + snapshot compaction instead of using SQLite WAL mode + standard migration. This duplicates SQLite's own crash recovery and compaction, adding complexity without benefit. | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/knowledge_storage.rs:28-37` | MEDIUM | level-db-patterns

D-DB-008: Graph BFS in `shortest_path` performs N individual SQL queries (one per node in path) instead of a single batch query or CTE. For deep graphs this becomes O(N) round-trips to SQLite. | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_graph.rs:84-110` | MEDIUM | sqlx-batch-patterns

D-DB-009: `embed_text` uses `reqwest::blocking::Client` in async context (OnceLock + blocking HTTP). This blocks the Tokio worker thread during embedding API calls (up to 120s timeout), violating async database best practices. | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_embed.rs:135-145` | HIGH | tokio-blocking-guide

D-DB-010: No prepared statement caching or statement reuse. Each `conn.prepare()` call re-parses SQL. Industry pattern (SQLx) uses `statement_cache_capacity(256)` for frequently-run queries like FTS search. | `neotrix-core/src/unified/layers/action/nt_memory/nt_memory_kb/nt_memory_search.rs:15-28` | LOW | sqlx-pool-config

D-DB-011: `kv_set` uses `ON CONFLICT ... DO UPDATE` without transaction wrapping. Concurrent writes from multiple Tokio tasks (via `open_raw_conn` creating new connections) can cause WAL contention without proper serialization. | `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:74-83` | MEDIUM | sqlite-concurrency-guide

D-DB-012: `embeddings` table stores vectors as BLOB without index for cosine similarity search. Full table scan required for every vector search query. No HNSW/IVF/PQ index exists despite `pq_codebook` and `embeddings_pq` tables being defined but unused. | `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:216-233` | MEDIUM | vector-search-patterns

## Key Insights

1. **Architecture Gap**: NeoTrix uses raw `rusqlite` (synchronous, no pooling) while industry standard for Rust async services is SQLx with connection pooling. The 70+ `nt_memory_kb` modules all block the Tokio runtime.

2. **No Migration System**: Despite `SCHEMA_VERSION=8`, there are no actual migration files. Schema changes are applied via `CREATE TABLE IF NOT EXISTS` + manual DDL, making rollback impossible and audit trails absent.

3. **Connection Anti-Pattern**: `open_raw_conn()` creates a new `Connection` per call without pooling or reuse. For a consciousness system running background loops every 60 seconds, this creates connection churn.

4. **FTS5 Tokenization Mismatch**: Porter stemmer cannot tokenize Chinese/JP content, yet NeoTrix stores大量中文知识. This creates a silent search quality degradation for CJK queries.

5. **Vector Search Without Index**: Embeddings are stored but searched via brute-force cosine similarity. The `pq_codebook`/`embeddings_pq` tables suggest intent to implement Product Quantization but it remains unimplemented.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Sources analyzed | 10 |
| Database patterns reviewed | SQLx, Diesel, SeaORM, rusqlite, connection pooling, migrations |

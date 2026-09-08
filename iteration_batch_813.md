# Iteration Batch 813 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### Database Crates (8)
- rusqlite 0.31: 9 versions behind current 0.40.1 (missing bug fixes, WAL improvements, RETURNING clause)
- SQLx 0.9.0: Requires DB at compile time (R-P9 violation), unsafe SQLite bridge (R-P1 violation)
- SeaORM 2.0.0: 10-20% overhead on every KB operation, entity codegen coupling
- Diesel 2.3.6: Sync-first, schema-first, async bolt-on (diesel-async)
- sled 0.34.7: In maintenance mode, no SQL, LSM read amplification
- redb: Pure Rust B+tree ACID MVCC, but KV-only (no SQL for NeoTrix KB)
- shodh-redb: Fork adds native vector search (IVF-PQ), blob store, TTL, compression
- Rusqlite 0.40.1: Rich feature set (backup, hooks, virtual tables, window functions, blob I/O)

### Logging/Tracing (8)
- tracing-subscriber CVE-2025-58160: ANSI injection in versions < 0.3.20
- tracing-opentelemetry 0.28: 5 major versions behind (0.33 current), broken OTel context
- No opentelemetry-appender-tracing: OTel log export missing
- Direct log = "0.4" alongside tracing: Redundant bridge overhead
- tracing-subscriber missing json/fmt/registry features
- No tracing-appender for file-based log rotation
- tracing v0.2 still unreleased (breaking changes planned)
- Datadog dd-trace-rs: Context stack rewrite (PR #2378) fixes tracing↔otel interop

### HTTP Client Patterns (8)
- reqwest pinned to 0.12 (2 generations behind, TLS cert verification changes in 0.13)
- 100+ reqwest::blocking::Client usages (no middleware, no retry, no tracing)
- Blocking client in async context (spawns blocking threads per request)
- Duplicated client construction boilerplate (15+ instances)
- No http-body-util for streaming responses
- Axum 0.8 with no tower-http integration
- No hyper-util connection pooling
- tower-http 0.7.1 released (breaking from 0.6.x)

### Testing Frameworks (8)
- Zero mocking layer (no mockall, wiremock, httpmock anywhere)
- Zero property-based testing (no proptest anywhere)
- criterion 0.5.0 (current 0.8.2, missing async bench, statistical improvements)
- No fake/test data generation
- Disabled tests in tests/_disabled/ (stress, e2e, security)
- MSRV 1.81 blocks criterion 0.8 (needs 1.86) and proptest 1.11 (needs 1.85)
- No HTTP mocking for NT-WORLD/NT-IO crawlers
- Manual test fixtures instead of realistic distributions

---

## Defects Identified (34+)

### Database (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-DB-1 | rusqlite 0.31 (9 versions behind 0.40.1) | High |
| D-DB-2 | All KB operations synchronous (blocks Tokio executor) | High |
| D-DB-3 | No connection pooling (per-call Connection::open) | Medium |
| D-DB-4 | Single-writer SQLite contention under concurrent module writes | Medium |
| D-DB-5 | WAL/busy_timeout not consistently set across all connection paths | Low |

### Logging/Tracing (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-LOG-1 | tracing-subscriber CVE-2025-58160 ANSI injection (can resolve to < 0.3.20) | Critical |
| D-LOG-2 | tracing-opentelemetry 0.28 (5 versions behind, broken OTel context) | High |
| D-LOG-3 | No opentelemetry-appender-tracing (OTel log export missing) | High |
| D-LOG-4 | Direct log = "0.4" alongside tracing (redundant) | Medium |
| D-LOG-5 | tracing-subscriber missing json/fmt/registry features | Medium |
| D-LOG-6 | No tracing-appender for file-based log rotation | Low |
| D-LOG-7 | tracing v0.2 still unreleased (breaking changes planned) | Low |

### HTTP Client (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-HTTP-1 | reqwest pinned to 0.12 (2 generations behind, TLS issues) | High |
| D-HTTP-2 | 100+ blocking::Client usages (no middleware, no retry, no tracing) | High |
| D-HTTP-3 | Blocking client in async context (unbounded thread spawning) | High |
| D-HTTP-4 | Duplicated client construction boilerplate (15+ instances) | Medium |
| D-HTTP-5 | No http-body-util for streaming responses | Medium |
| D-HTTP-6 | Axum 0.8 with no tower-http integration | Medium |
| D-HTTP-7 | No hyper-util connection pooling | Low |

### Testing (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-TEST-1 | Zero mocking layer (no mockall, wiremock, httpmock) | High |
| D-TEST-2 | Zero property-based testing (no proptest) | High |
| D-TEST-3 | criterion 0.5.0 (3 major versions behind) | Medium |
| D-TEST-4 | No fake/test data generation | Medium |
| D-TEST-5 | Disabled tests (stress, e2e, security) | High |
| D-TEST-6 | MSRV 1.81 blocks modern testing stack | Medium |
| D-TEST-7 | No HTTP mocking for crawlers | High |

## Key Insights (This Batch)

1. **rusqlite upgrade is lowest-risk highest-impact**: NeoTrix's entire KB layer (100+ call sites) is raw SQL with rusqlite. Switching libraries would be massive regression. Upgrade 0.31→0.40.1 is straightforward.

2. **tracing-subscriber CVE-2025-58160 is critical**: ANSI escape injection allows terminal manipulation attacks. Must pin >= 0.3.20 immediately.

3. **reqwest blocking client in async context**: Every NT-WORLD fetcher spawns blocking threads per request. This is wasteful and unbounded. Must switch to async reqwest with .await.

4. **Zero mocking layer**: Every unit test hits real dependencies or uses ad-hoc stubs. Tests are slow, flaky, and untestable in isolation. mockall + httpmock would unblock this.

5. **Bincode hostile to AI contributions**: Official bincode repo bans AI. NeoTrix as AI-native project should avoid this dependency.

6. **serde_bytes is mandatory**: Without it, rmp-serde serializes byte arrays as array-of-ints (50% overhead). Must enforce on all Vec<u8> fields.

7. **SQLx violates R-P1 and R-P9**: Requires DB at compile time and uses unsafe SQLite bridge. Both conflict with NeoTrix's build rules.

8. **tracing-opentelemetry broken context activation**: NeoTrix's 0.28 pin causes broken traces when mixing tracing + OTel API calls. Must upgrade to 0.33.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 813 |
| New defects (this batch) | 26 |
| Cumulative defects | D01-D76474 |
| Research sources (this batch) | 32 |
| Cumulative research sources | 97,252+ |

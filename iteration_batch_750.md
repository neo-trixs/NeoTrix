# Iteration Batch 750 — Database Migration & Schema Evolution

**Date**: 2026-09-07  
**Source**: Web research (24 sources across 3 queries)  
**Prior Context**: Batch 749 confirmed (1) Microsoft SMTP AUTH dead, (2) no OAuth2 for email, (3) no channel routing/fallback, (4) no user preference store, (5) no E2EE.

---

## NEW FINDINGS

### Finding 1: NeoTrix KB Has Zero Migration Infrastructure
**Severity**: CRITICAL  
**Evidence**: Search of codebase returned 0 hits for `migration`, `schema_migrations`, `sqlx_migrations`, `embed_migrations`, or `migrate!`. No `.sql` files exist. No expand-contract or dual-write patterns found. The KB at `~/.neotrix/knowledge.db` uses raw SQLite with `kv_store` table but has no versioned migration system.

**Industry Standard (2026)**: Both SQLx and Diesel provide embedded migration infrastructure:
- SQLx: `sqlx::migrate!` macro embeds SQL into binary, checksum validation, `sqlx_migrations` metadata table tracking version/checksum/execution-time
- Diesel: `embed_migrations!` macro, `diesel_schema_migrations` table, `schema.rs` auto-regeneration on migration
- Both enforce: never edit applied migrations, checksum mismatch = refusal to run, metadata table = source of truth for schema state

**Defect**: NeoTrix KB schema is implicit — table structure lives in code but is never formally versioned. A schema change (e.g., adding column to `kv_store`) silently breaks existing readers. No migration checksums, no metadata table, no rollback capability.

**Sources**: [rs4ts.dev/17-database/09-migrations](https://rs4ts.dev/17-database/09-migrations/), [rustfaq.org/en/how-to-run-database-migrations-in-rust](https://www.rustfaq.org/en/how-to-run-database-migrations-in-rust), [abrarqasim.com/blog/sqlx-vs-diesel-2026](https://abrarqasim.com/blog/sqlx-vs-diesel-2026-how-i-pick-my-rust-database-library)

---

### Finding 2: No Schema Drift Detection for KB
**Severity**: HIGH  
**Evidence**: Atlas (Ariga) introduced "Schema Monitoring" — periodic production snapshots compared against code schema to detect drift, with Slack/email alerts when someone runs `ALTER TABLE` directly. Flyway, Liquibase, Prisma Migrate lack this capability. NeoTrix has no drift detection at all.

**Industry Standard (2026)**: Atlas's drift detection runs hourly snapshots:
```
atlas schema monitor --url "postgres://prod.../app" --token "$ATLAS_TOKEN" --interval 1h
```
Drift triggers alerts. Bytebase, Skeema also offer schema comparison tools.

**Defect**: NeoTrix KB (`knowledge.db`) is modified by multiple code paths: `experience.rs` writes experience entries, `kb_cmds.rs` manages nodes/edges, `cortex_cmds.rs` reads consciousness state, `nt_shield_local_inference.rs` writes agent profiles, `bandit.rs` writes RL state. Any of these could silently alter the schema without detection. No schema version table exists to detect drift between expected and actual state.

**Sources**: [bytebase.com/blog/top-database-schema-change-tool-evolution](https://www.bytebase.com/blog/top-database-schema-change-tool-evolution/), [youngju.dev/blog/2026-05-14-database-migration-tools](https://www.youngju.dev/blog/culture/2026-05-14-database-migration-tools-2026-atlas-bytebase-skeema-liquibase-flyway-pg-online-comparison-deep-dive.en)

---

### Finding 3: No Expand-Contract Pattern for KB Schema Evolution
**Severity**: HIGH  
**Evidence**: The expand-contract (parallel change) pattern is the 2026 standard for zero-downtime schema changes:
1. **Expand**: Add new column/table, old code ignores it, new code writes both
2. **Migrate**: Backfill data from old to new structure
3. **Contract**: Remove old column/table after all instances use new code

NeoTrix has zero implementation of this pattern for KB changes. The `experience` namespace `kv_store` has been modified across iterations (iteration 604 identified this gap) but never through formal expand-contract.

**Industry Standard (2026)**: [knowledgelib.io](https://knowledgelib.io/software/patterns/database-migration-strategies/2026) provides a decision tree: tables >10M rows use `gh-ost`/`pt-online-schema-change` (MySQL) or `pg_repack`/`pgroll` (PostgreSQL). Rename = expand-contract 3-phase. Type change = expand-contract. Column drop = deploy app first, then drop in separate migration.

**Defect**: NeoTrix KB migrations (managed by `nt_memory`) have no formal expand-contract protocol. Schema changes to `kv_store` namespaces (`experience`, `consciousness`, `domain_nt_*`, `state.*`) could silently break existing readers/writers. No phased rollout of schema changes.

**Sources**: [hostmycode.com/blog/database-migration-strategies-high-traffic-applications](https://www.hostmycode.com/blog/database-migration-strategies-high-traffic-applications-zero-downtime-schema-changes-data-movement-2026), [appscale.blog/en/blog/zero-downtime-database-migration-architecture](https://appscale.blog/en/blog/zero-downtime-database-migration-architecture-expand-contract-backfill-2026), [knowledgelib.io/software/patterns/database-migration-strategies/2026](https://knowledgelib.io/software/patterns/database-migration-strategies/2026)

---

### Finding 4: No Backfill Strategy for KB Data Migrations
**Severity**: HIGH  
**Evidence**: The expand-contract pattern requires a backfill phase — moving historical data from old structure to new. NeoTrix has no backfill infrastructure. When KB schema changes (e.g., adding fields to experience entries), existing records are left with default/null values rather than being backfilled from source data.

**Industry Standard (2026)**: [appscale.blog](https://appscale.blog/en/blog/zero-downtime-database-migration-architecture-expand-contract-backfill-2026) defines backfill as the critical middle phase:
1. Backfill in batches (not full table scan)
2. Rate-limited to avoid overwhelming production
3. Checkpointable/resumable
4. Verified before switching reads

**Defect**: NeoTrix experience entries in `kv_store` have been modified across 700+ iterations. Each schema change (adding confidence fields, feedback tracking, branch metadata) leaves older entries incomplete. No batch backfill mechanism exists to update historical entries to current schema.

**Sources**: [appscale.blog/en/blog/zero-downtime-database-migration-architecture](https://appscale.blog/en/blog/zero-downtime-database-migration-architecture-expand-contract-backfill-2026)

---

### Finding 5: No Online Schema Change for Large KB Tables
**Severity**: MEDIUM  
**Evidence**: For tables >10M rows, online schema change tools prevent lock contention:
- MySQL: `gh-ost` (GitHub's Online Schema Migration) or `pt-online-schema-change` (Percona)
- PostgreSQL: `pg_repack`, `pgroll`, `CREATE INDEX CONCURRENTLY`
- `CREATE INDEX CONCURRENTLY` cannot run inside a transaction block — migration tools must be configured to run this outside transactions

**Defect**: NeoTrix `kv_store` table grows monotonically (iteration 362: "no explicit forgetting mechanism"). As experience entries accumulate (1957+ entries per iteration 707 calibration), schema changes to `kv_store` will increasingly cause lock contention. No online schema change tool is configured for SQLite.

**Sources**: [knowledgelib.io/software/patterns/database-migration-strategies/2026](https://knowledgelib.io/software/patterns/database-migration-strategies/2026), [hostmycode.com/blog/database-migration-strategies-high-traffic-applications](https://www.hostmycode.com/blog/database-migration-strategies-high-traffic-applications-zero-downtime-schema-changes-data-movement-2026)

---

### Finding 6: No Migration Testing in Production-Like Environments
**Severity**: MEDIUM  
**Evidence**: [hostmycode.com](https://www.hostmycode.com/blog/database-migration-strategies-high-traffic-applications-zero-downtime-schema-changes-data-movement-2026): "Staging databases with toy datasets don't reveal migration problems. Create staging environments that mirror production data volume and access patterns. Use data masking tools to sanitize production data for staging use."

**Defect**: NeoTrix has no migration testing framework. Schema changes are tested only against the local `knowledge.db` which may have different data volume than production deployments. No load testing during migrations. No production-like staging environment for migration validation.

**Sources**: [hostmycode.com/blog/database-migration-strategies-high-traffic-applications](https://www.hostmycode.com/blog/database-migration-strategies-high-traffic-applications-zero-downtime-schema-changes-data-movement-2026)

---

### Finding 7: No Migration Rollback Infrastructure
**Severity**: HIGH  
**Evidence**: [gitscrum.com](https://docs.gitscrum.com/en/best-practices/database-migration-management): "Never run DDL migrations without a tested rollback script or compensating migration." Both SQLx and Diesel support reversible migrations (`up.sql`/`down.sql` pairs). SQLx supports optional reversibles; Diesel always generates both.

**Defect**: NeoTrix KB schema changes are forward-only. No rollback scripts exist. If a migration breaks existing readers/writers, the only recovery is manual SQLite intervention or restoring from backup (which also doesn't exist per iteration 725). The experience-tree absorption protocol writes to KB but has no compensating migration if the write corrupts the experience hub.

**Sources**: [docs.gitscrum.com/en/best-practices/database-migration-management](https://docs.gitscrum.com/en/best-practices/database-migration-management), [rs4ts.dev/17-database/09-migrations](https://rs4ts.dev/17-database/09-migrations/)

---

### Finding 8: No Feature Flag Decoupling for Schema Changes
**Severity**: MEDIUM  
**Evidence**: [hostmycode.com](https://www.hostmycode.com/blog/database-migration-strategies-high-traffic-applications-zero-downtime-schema-changes-data-movement-2026): "Feature flags help decouple application changes from schema changes." [codelit.io](https://codelit.io/blog/migration-strategies-zero-downtime): Feature flags enable gradual schema rollout.

**Defect**: NeoTrix's KB schema changes are tightly coupled to code changes. When `experience.rs` adds a new field, the code that reads/writes that field is deployed simultaneously with the schema change. No feature flag infrastructure exists to decouple these, meaning a schema change that breaks old code cannot be rolled back without code rollback.

**Sources**: [hostmycode.com/blog/database-migration-strategies-high-traffic-applications](https://www.hostmycode.com/blog/database-migration-strategies-high-traffic-applications-zero-downtime-schema-changes-data-movement-2026), [codelit.io/blog/migration-strategies-zero-downtime](https://codelit.io/blog/migration-strategies-zero-downtime)

---

### Finding 9: No Encryption-in-Transit for KB During Migration
**Severity**: MEDIUM  
**Evidence**: [tessell.com](https://www.tessell.com/blog/zero-downtime-migration-strategies): "TLS encryption for replication streams, VPN or private peering for cross-cloud transfers, and audit logging of all migration activity are baseline requirements for any regulated workload."

**Defect**: NeoTrix KB (`knowledge.db`) is a local SQLite file. While in-transit encryption is less relevant for local files, the broader concern is: when KB data is migrated between devices or synced (future multi-device feature), no encryption-in-transit protocol exists. The `knowledge.db` file itself is unencrypted at rest (iteration 725).

**Sources**: [tessell.com/blog/zero-downtime-migration-strategies](https://www.tessell.com/blog/zero-downtime-migration-strategies)

---

### Finding 10: No Post-Migration Validation Framework
**Severity**: HIGH  
**Evidence**: [tessell.com](https://www.tessell.com/blog/zero-downtime-migration-strategies): "Run full rehearsals before production cutover. Every migration should be rehearsed on a production-equivalent non-production environment at least twice before touching live data." [techment.com](https://www.techment.com/blogs/data-migration-trends-best-practices-2026/): "Parallel run strategies, automated reconciliation, rollback and contingency planning."

**Defect**: NeoTrix has no post-migration validation. After a KB schema change, there's no automated check that:
1. All existing data is accessible in new schema
2. All read/write paths still function
3. No data corruption occurred during migration
4. Performance is within acceptable bounds
The `PRAGMA integrity_check` exists but is manual only.

**Sources**: [tessell.com/blog/zero-downtime-migration-strategies](https://www.tessell.com/blog/zero-downtime-migration-strategies), [techment.com/blogs/data-migration-trends-best-practices-2026](https://www.techment.com/blogs/data-migration-trends-best-practices-2026/)

---

## DEFECT SUMMARY

| ID | Defect | Severity | Category |
|----|--------|----------|----------|
| B-750-1 | Zero migration infrastructure (no SQLx/Diesel macros, no .sql files, no metadata table) | CRITICAL | Infrastructure |
| B-750-2 | No schema drift detection (no Atlas-style monitoring, no hourly snapshots) | HIGH | Monitoring |
| B-750-3 | No expand-contract pattern for KB schema evolution | HIGH | Pattern |
| B-750-4 | No backfill strategy for data migrations | HIGH | Data |
| B-750-5 | No online schema change tooling for large tables | MEDIUM | Performance |
| B-750-6 | No migration testing in production-like environments | MEDIUM | Testing |
| B-750-7 | No migration rollback infrastructure (forward-only, no compensating migrations) | HIGH | Safety |
| B-750-8 | No feature flag decoupling for schema changes | MEDIUM | Architecture |
| B-750-9 | No encryption-in-transit for KB during multi-device migration | MEDIUM | Security |
| B-750-10 | No post-migration validation framework | HIGH | Quality |

## RELATIONSHIP TO PRIOR BATCHES

- **Batch 604** identified expand-contract gap → Batch 750 confirms zero implementation + adds backfill/rollback findings
- **Batch 725** identified no backup strategy → Batch 750 adds no migration rollback (complementary safety gap)
- **Batch 672** identified SQLite B-tree vs LSM mismatch → Batch 750 adds that schema changes compound this by lacking online schema change
- **Batch 423** identified `kv_store` namespace scan without index optimization → Batch 750 adds no migration testing for index changes

## SOURCES CITED

1. rs4ts.dev — SQLx and Diesel migration guide (2026)
2. rustfaq.org — Database migrations in Rust (April 2026)
3. abrarqasim.com — SQLx vs Diesel in 2026 (June 2026)
4. bytebase.com — Top database schema change tools (Jan 2026)
5. datastackhub.com — 9 best open source schema evolution tools (Aug 2026)
6. gitscrum.com — Database migrations best practices
7. blog.vibecoder.me — Schema evolution handling long term (April 2026)
8. knowledgelib.io — Database migration strategies (Feb 2026)
9. youngju.dev — DB migration tools 2026 comparison (May 2026)
10. hostmycode.com — Zero-downtime migration strategies (April 2026)
11. dbschema.com — Schema migration tools compared (Aug 2026)
12. medium.com — 15 schema evolution tools (Feb 2026)
13. eidosoft.co — Zero-downtime cloud data migration (Feb 2026)
14. tessell.com — Zero downtime migration strategies (April 2026)
15. ceoworld.biz — Top data migration solutions for 2026 (Feb 2026)
16. appscale.blog — Zero-downtime database migration architecture (June 2026)
17. techment.com — Data migration trends best practices (May 2026)
18. codingcops.com — Zero-downtime data migration guide (Jan 2026)
19. latentview.com — Enterprise data migration (May 2026)
20. codelit.io — Zero downtime migration strategies (March 2026)
21. quinnox.com — Data migration plan 2026 (May 2026)
22. integrate.io — Top 15 ETL consultants (Jan 2026)
23. cloud.toolsinfo.com — Schema migration selection guide 2026
24. docs.rs/sqlx — Migrator and Migration structs

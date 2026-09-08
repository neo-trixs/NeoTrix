# Iteration Batch 442 — External Research: Time Series DB / OLAP / Analytical Query Optimization (2026-09-06)

## Sources Cited

### Time Series Databases
1. **TimescaleDB vs InfluxDB 2026** (pdpspectra, May 2026) — InfluxDB 3.0 FDAP architecture (Flight+DataFusion+Arrow+Parquet), TimescaleDB Hypercore (combined row+columnstore), VictoriaMetrics/IoTDB differentiation. InfluxDB 3.0: 4.2x faster aggregate queries than TimescaleDB 2.14 on 1M-point workloads, 820 qps vs 195 qps. TimescaleDB: 5.2x compression ratio (18MB vs 48MB for 1M points). Dual-tier architecture (InfluxDB hot + TimescaleDB cold) 19% cheaper than either alone.
   - https://pdpspectra.com/blog/timescaledb-vs-influxdb-2026/
   - https://pdpspectra.com/blog/time-series-influx-vs-timescale-vs-quest/
2. **InfluxDB 3.0 vs TimescaleDB 2.14** (johal.in, May 2026) — Benchmark on 1M data points: InfluxDB 3.0 p50=12ms, TimescaleDB p50=51ms. Ingestion: InfluxDB 12K pts/sec, TimescaleDB 8.5K pts/sec. TimescaleDB has PostgreSQL ACID transactions; InfluxDB does not. InfluxDB 3.0 native Parquet export via Flight SQL.
   - https://johal.in/comparison-influxdb-30-vs-timescaledb-214-time-series
3. **InfluxDB 3 vs TimescaleDB vs ClickHouse for IoT** (IoT Digital Twin PLM, Apr 2026) — InfluxDB 3 ingestion 10-100M rows/sec (single node), unlimited cardinality. TimescaleDB ACID + JOINs, 100K-1M rows/sec. ClickHouse 100K-5M rows/sec, no transactions. Decision: InfluxDB for cardinality/throughput, TimescaleDB for relational+ACID, ClickHouse for analytics.
   - https://iotdigitaltwinplm.com/influxdb-vs-timescaledb-vs-clickhouse-iot-time-series-2026/
4. **Time-Series Databases: InfluxDB vs TimescaleDB** (topperblog, Feb 2026) — TSM engine (append-only, 90%+ compression), hypertables (auto-partitioned time chunks), continuous aggregates (materialized rollups), compression engine (row→columnar after threshold, 10-40x compression).
   - https://topperblog.hashnode.dev/time-series-databases-influxdb-vs-timescaledb

### OLAP Databases
5. **ClickHouse vs DuckDB vs StarRocks** (SumGuy, Jul 2026) — ClickHouse: 0.05s COUNT(*), 0.9s 3-key GROUP BY, 2.1s 3-table JOIN. DuckDB: 0.3s COUNT(*), 0.9s 3-table JOIN (in-process, zero-ops). StarRocks: 0.6s 3-table JOIN (MPP, MySQL-protocol). ClickHouse wins aggregation; DuckDB wins JOINs under 100GB; StarRocks wins multi-source JOINs at scale.
   - https://sumguy.com/clickhouse-vs-duckdb-vs-starrocks/
6. **Top 5 OLAP 2026** (top-5-solutions, Apr 2026) — ClickHouse 9.2/10, DuckDB 8.9/10, StarRocks 8.3/10. ClickHouse for warehouse-class SQL + portability. DuckDB for analytical SQL on local files (<100GB). StarRocks for MPP + MySQL dialect + Iceberg/Hudi/Delta Lake.
   - https://top-5-solutions.com/top-5-olap-database-solutions/2026/
7. **DuckDB vs ClickHouse Benchmark 2026** (duckdblab, Jun 2026) — DuckDB wins 7/10 categories (simple agg, window functions, JSON, geospatial, ML, data export, simple queries). ClickHouse wins 3/10 (concurrency, full-text search, time-series). DuckDB: 1.2GB RAM for 5GB dataset vs ClickHouse 3.8GB. DuckDB first query 2.1s vs ClickHouse 8.3s.
   - https://duckdblab.org/en/post/duckdb-vs-clickhouse-benchmark-2026/
8. **Alternatives to ClickHouse 2026** (creacosas, Aug 2026) — DuckDB for <50GB + single-process. ClickHouse for shared service + concurrent writes + multi-machine. StarRocks for normalized schema + heavy JOINs + upsert. Druid/Pinot for real-time sub-second at high concurrency (expensive ops).
   - https://www.creacosas.com/en/blogs/technology/alternativas-clickhouse-comparativa

### Analytical Query Optimization
9. **Presto MV Rewrite — Expression GROUP BY + Conditional Aggregates** (prestodb, 2026) — Extends materialized view query rewriting to support expression GROUP BY keys (`CAST`, `DATE_TRUNC`), conditional aggregates (`SUM(IF(...))`, `SUM(CASE WHEN ...)`), and `SET_UNION` rollups. No new config needed; fallback to base tables on mismatch.
   - https://github.com/prestodb/presto/pull/27996
10. **Workload Acceleration by MV Selection via Local Search** (arXiv 2606.03772) — Incorporates incremental view maintenance cost into ILP objective. Local search with containment-based neighbors. Top-3 initial solutions: frequency, utility, utility-per-storage. Outperforms BIGSUBS on RedBench.
    - https://arxiv.org/html/2606.03772
11. **Cost-Based MV Candidate Selection** (prestodb, Feb 2026) — Instead of picking first compatible MV, collects all candidates and lets cost-based optimizer choose cheapest. `MVRewriteCandidatesNode` plan node holds original + all MV candidates. Gated by `materialized_view_query_rewrite_cost_based_selection_enabled`.
    - https://github.com/prestodb/presto/pull/27222
12. **Join-Aware MV Query Rewrite** (Eric Sun, Jun 2026) — SPJG containment: join graph subsumption, grouping rollup, measure derivability, filter compensation. StarRocks and BigQuery support join-containing MVs with transparent rewrite. Snowflake split: MVs = single-table rewrite, Dynamic Tables = joins but no transparent rewrite. Redshift supports it.
    - https://eric-sun.medium.com/the-join-aware-materialized-view-query-rewrite-gap-bdcb248dbd18
13. **BigQuery MVs + BI Engine** (DEV Community, Jun 2026) — Four-layer acceleration: partitioning → clustering → MV → BI Engine (RAM). Smart MVs support JOINs (since 2024), `max_staleness` for stale MV merge. MV is declarative; planner transparently rewrites base-table queries. Sub-50ms on 12TB fact table.
    - https://dev.to/gowthampotureddi/bigquery-materialized-views-bi-engine-sub-second-dashboards-on-petabytes-4bb2

---

## Defects Found in NeoTrix Design

### DEF-442-01: No Continuous Aggregates / Incremental Pre-Aggregation in KB
**Severity**: High | **Domain**: NT-MEMORY
**Finding**: TimescaleDB continuous aggregates incrementally update materialized rollups as new data arrives — the single most impactful feature for analytical dashboards. BigQuery smart MVs transparently rewrite base-table queries into MV scans. Presto now supports expression GROUP BY keys + conditional aggregates in MV rewriting. NeoTrix KB has a flat `kv_store` table with no pre-aggregation layer. All analytical queries (experience aggregation, domain health scoring, heartbeat signal computation) scan full tables every time.
**Impact**: Every `/stats`, `/doctor`, or consciousness tick must re-aggregate all KB data from raw rows. As KB grows with crawl data and experiences, these queries degrade linearly. No incremental update path exists.
**Suggestion**: Add a `kb_aggregates` materialized table layer in NT-MEMORY. Implement incremental refresh: when nodes/edges are inserted or updated, update affected aggregate rows. Cover the 3 most common aggregates: (1) domain node counts + last-updated timestamps, (2) experience density by domain × staleness, (3) heartbeat signal rollups (compilation/test/KB health). Gate behind `Rune Socketing` (Obsidian rune = cache/pre-aggregation). This eliminates full-scan aggregation for dashboard queries.

### DEF-442-02: No Columnar Analytical Storage Path for KB Metrics
**Severity**: High | **Domain**: NT-MEMORY + NT-CORE (HeartbeatAggregator)
**Finding**: InfluxDB 3.0 achieves 4.2x faster aggregate queries than TimescaleDB via columnar Parquet storage. ClickHouse achieves sub-second aggregations over 100M rows via columnar MergeTree. TimescaleDB Hypercore combines row+columnstore. NeoTrix KB stores everything in SQLite row-based format. HeartbeatAggregator, E8Predictor, and consciousness metrics are time-series data stored in flat `kv_store` JSON blobs.
**Impact**: HeartbeatAggregator must deserialize JSON blobs and scan all entries for each aggregation. As heartbeat history accumulates (60s tick × 24h = 1,440 entries/day × 365 = 525K entries/year), aggregation latency degrades linearly. No columnar scan capability means analytics on time-series metrics are always O(n) over full history.
**Suggestion**: Add a `MetricsColumnar` path for time-series KB data. Two options: (a) Export to DuckDB in-process for analytical queries over metrics (DuckDB 1.x: zero-ops, in-process, wins 7/10 analytical categories over ClickHouse under 100GB), or (b) Add TimescaleDB-style continuous aggregates for heartbeat/metrics data. For NT-PHYSICAL edge deployments, DuckDB in-process is ideal (8.9/10 rating, 1.2GB RAM for 5GB dataset). Gate behind Obsidian rune for analytics caching.

### DEF-442-03: No TTL / Automatic Data Retention Policies
**Severity**: Medium | **Domain**: NT-MEMORY + NT-SHIELD
**Finding**: TimescaleDB: `add_retention_policy()` auto-expires old chunks. InfluxDB: retention policies at bucket creation. VictoriaMetrics: per-tenant retention with streaming downsampling. QuestDB: SAMPLE BY + partition-based retention. NeoTrix has `nt_core_knowledge::versioning` with `StalenessLevel` multipliers (Fresh=1.0 → Obsolete=0.1) but no actual deletion/expiration mechanism. Data accumulates indefinitely.
**Impact**: KB grows unbounded. Session logs, crawl results, and experience entries from months ago occupy storage and slow queries. NT-SHIELD audit trail has no compliance expiry. No data lifecycle management exists.
**Suggestion**: Implement TTL-aware KB compaction in SEAL pipeline cycles. Three tiers: (1) Hot (last 7 days) — full fidelity, (2) Warm (7-90 days) — continuous aggregates replace raw data, (3) Cold (>90 days) — export to Parquet + delete from SQLite. Align with DEF-441-08 (entity TTL). Use the existing `StalenessLevel` system as the policy engine: when staleness reaches `Obsolete`, auto-expire after grace period. This is the TimescaleDB retention pattern adapted for NeoTrix.

### DEF-442-04: No Join-Aware Analytical Query Capability
**Severity**: Medium | **Domain**: NT-MEMORY + NT-CORE
**Finding**: StarRocks' core differentiator is cost-based optimizer for multi-table JOINs (0.6s for 3-table JOIN vs ClickHouse 2.1s). BigQuery smart MVs now support JOINs with transparent rewrite. Snowflake explicitly lacks join-aware MV rewrite — it's the #1 cited gap. NeoTrix KB queries are single-table: `kv_store` lookups by namespace+key, FTS5 searches, or simple `SELECT` on `nodes`/`edges`. No analytical JOIN capability exists.
**Impact**: Cross-domain analysis (e.g., "how does NT-WORLD crawl quality correlate with NT-MEMORY KB coverage?") requires application-level JOIN logic. No way to express relational queries across KB entities. The VSA HyperCube concept graph exists as edges but cannot be queried relationally.
**Suggestion**: For analytical workloads, add a DuckDB analytical view layer that can attach to SQLite and run analytical JOINs across KB tables. DuckDB's native SQLite attachment (`ATTACH 'kb.db' AS kb (TYPE SQLITE)`) enables SQL JOINs across SQLite tables without migration. This gives StarRocks-like JOIN capability with DuckDB's zero-ops advantage. For production dashboards, materialize common cross-domain JOINs as views.

### DEF-442-05: No Pre-Computed Materialized Views for Dashboard Queries
**Severity**: Medium | **Domain**: NT-MEMORY + NT-IO (CLI/dashboard)
**Finding**: BigQuery's 4-layer stack (partitioning → clustering → MV → BI Engine) delivers sub-50ms on 12TB fact tables. Presto's cost-based MV selection picks the cheapest MV candidate automatically. NeoTrix `/stats`, `/doctor`, `/explore stats`, and TUI dashboard all compute metrics on-the-fly from raw data. No pre-computed views exist.
**Impact**: Dashboard latency scales linearly with KB size. On large KBs (>100K nodes), stats commands take seconds. No way to serve sub-second dashboards for consciousness monitoring or domain health.
**Suggestion**: Implement a `DashboardCache` layer in NT-IO that maintains pre-computed views for the 5 most common dashboard queries: (1) Domain node counts, (2) Experience index (cycle count + last cycle pointer), (3) Heartbeat health snapshot, (4) Skill constellation maturity matrix, (5) Build baseline status. Refresh strategy: event-driven on KB writes + periodic background refresh (60s tick). Store as compressed JSON in `kv_store` namespace `dashboard_cache`. This is the TimescaleDB continuous aggregate pattern simplified for SQLite.

### DEF-442-06: No Downsampling / Rollup for Time-Series Metrics
**Severity**: Medium | **Domain**: NT-MEMORY + NT-CORE (HeartbeatAggregator/E8Predictor)
**Finding**: TimescaleDB continuous aggregates = materialized rollups that update incrementally. VictoriaMetrics recording rules + streaming downsampling. InfluxDB downsampling tasks. QuestDB SAMPLE BY. All mature TSDBs provide automatic downsampling of high-frequency metrics into lower-resolution summaries. NeoTrix heartbeat ticks at 60s intervals. E8Predictor samples at variable rates. All raw data stored without downsampling.
**Impact**: Heartbeat history at 60s resolution = 525K entries/year. E8Predictor samples accumulate similarly. Querying "average phi over last month" requires scanning ~43K entries. No rollup to hourly/daily summaries exists.
**Suggestion**: Implement a `RollupScheduler` that runs during SEAL pipeline cycles: (1) Raw (last 24h) — 60s resolution, (2) Hourly rollup (24h-7d) — avg/min/max per hour, (3) Daily rollup (7d-90d) — avg/min/max per day, (4) Monthly rollup (>90d) — avg/min/max per month. Store rollups in `kv_store` namespace `metrics_rollup`. Query router automatically selects appropriate resolution based on time range. This is the TimescaleDB continuous aggregate pattern with explicit rollup tiers.

### DEF-442-07: No DuckDB Integration for Embedded Analytical Engine
**Severity**: Medium | **Domain**: NT-MEMORY + NT-PHYSICAL
**Finding**: DuckDB 1.x is the 2026 consensus default for embedded analytical SQL (8.9/10 rating, MIT license, 7/10 categories won vs ClickHouse). It natively reads Parquet, CSV, JSON, Arrow, Iceberg, and can `ATTACH` SQLite databases. It runs in-process with zero operations. NeoTrix has no embedded analytical engine — all analytics go through SQLite's limited query capabilities.
**Impact**: Analytical workloads (aggregation, window functions, complex JOINs, geospatial queries) fall back to application-level code or fail entirely. DuckDB's window function performance is 2.6x faster than ClickHouse. NeoTrix cannot leverage this without integration.
**Suggestion**: Add DuckDB as an optional analytical backend in NT-MEMORY. Use `duckdb-rs` crate to embed DuckDB in-process. Two integration points: (1) `ATTACH` the NeoTrix SQLite KB for analytical queries across KB tables, (2) Query Parquet exports from NT-WORLD crawl data directly. Gate behind Obsidian rune (analytics). For NT-PHYSICAL edge: DuckDB runs on 4GB RAM. For cloud: ClickHouse integration for distributed analytics.

### DEF-442-08: No Partitioning Strategy for Time-Based KB Data
**Severity**: Low-Medium | **Domain**: NT-MEMORY
**Finding**: TimescaleDB hypertables automatically partition by time intervals (chunks). InfluxDB TSM files organized by time ranges. ClickHouse MergeTree partitions by key (typically time). All production time-series systems use time-based partitioning to enable efficient range scans and data lifecycle management. NeoTrix `kv_store` has no partitioning — all data in one table. Nodes/edges have `created_at`/`updated_at` columns but no partition pruning.
**Impact**: Time-range queries ("experiences from last week", "heartbeat data for today") scan the entire table. As KB grows to millions of rows, time-range queries degrade. No efficient data lifecycle operations (archive, delete by age) possible without partitioning.
**Suggestion**: Implement logical partitioning in NT-MEMORY: (1) Add `partition_key` column to `kv_store` derived from `updated_at` (e.g., `YYYY-MM`), (2) Create partition-specific indexes, (3) Enable partition-level operations (archive, compact, drop). For DuckDB integration, use DuckDB's native partitioned Parquet export. This enables the TimescaleDB chunk-based lifecycle without requiring a different database engine.

### DEF-442-09: No Cost-Based Query Optimization for KB Queries
**Severity**: Low-Medium | **Domain**: NT-MEMORY + NT-CORE
**Finding**: Presto's cost-based MV selection (PR #27222) lets the optimizer compare all candidate plans and pick the cheapest. StarRocks' cost-based optimizer automatically selects broadcast vs shuffle vs colocate joins. NeoTrix KB queries are statically dispatched: `kv_get` → point lookup, `kb_search` → FTS5, `kb_get_related` → edge traversal. No cost estimation or adaptive query planning exists.
**Impact**: As KB grows and query patterns diversify, static dispatch cannot adapt. A query that was fast at 1K nodes may be slow at 100K nodes. No mechanism to switch strategies (e.g., from FTS5 to embedding search) based on data characteristics.
**Suggestion**: Add a thin `QueryPlanner` in NT-MEMORY that: (1) Estimates result set size from query predicates, (2) Selects between FTS5 vs embedding vs graph traversal based on estimate, (3) Falls back gracefully when primary strategy is slow. This is the Presto cost-based pattern simplified for single-node SQLite. Not a full optimizer — just adaptive strategy selection for the 3 KB query paths.

### DEF-442-10: No Iceberg/Delta Lake Integration for Analytical Data
**Severity**: Low | **Domain**: NT-MEMORY + NT-WORLD
**Finding**: StarRocks 3.x has first-class Iceberg/Hudi/Delta Lake support with external catalogs. DuckDB natively reads Iceberg tables. ClickHouse supports Iceberg via table functions. The 2026 analytical ecosystem standardizes on open table formats (Iceberg/Delta Lake) for cross-engine data sharing. NeoTrix exports crawl data as raw files with no standard table format.
**Impact**: NT-WORLD crawl results cannot be consumed by external analytical tools (Grafana, Superset, Jupyter). No standard format for sharing crawl data across NeoTrix instances or with external pipelines. Data is locked in NeoTrix's proprietary format.
**Suggestion**: Add Iceberg export capability in NT-WORLD ingest path. When crawl results are finalized, export to Iceberg format on local filesystem or S3. This enables: (1) DuckDB/ClickHouse/StarRocks direct query of NeoTrix crawl data, (2) Cross-instance data sharing via shared Iceberg catalogs, (3) Integration with the broader data lake ecosystem. Low priority because NT-WORLD data volume is typically manageable in SQLite.

---

## Summary

| # | Defect | Severity | Sources |
|---|--------|----------|---------|
| 442-01 | No continuous aggregates / incremental pre-aggregation | High | TimescaleDB, BigQuery MVs, Presto MV rewrite |
| 442-02 | No columnar analytical storage for KB metrics | High | InfluxDB 3.0, ClickHouse, DuckDB benchmark |
| 442-03 | No TTL / automatic data retention | Medium | TimescaleDB, InfluxDB, VictoriaMetrics retention |
| 442-04 | No join-aware analytical query capability | Medium | StarRocks, BigQuery smart MVs, Snowflake gap |
| 442-05 | No pre-computed materialized views for dashboards | Medium | BigQuery 4-layer stack, Presto cost-based MV |
| 442-06 | No downsampling / rollup for time-series metrics | Medium | TimescaleDB, VictoriaMetrics, QuestDB SAMPLE BY |
| 442-07 | No DuckDB integration for embedded analytics | Medium | DuckDB 1.x (8.9/10, MIT, 7/10 categories) |
| 442-08 | No partitioning strategy for time-based data | Low-Med | TimescaleDB hypertables, InfluxDB TSM, ClickHouse |
| 442-09 | No cost-based query optimization | Low-Med | Presto cost-based MV, StarRocks CBO |
| 442-10 | No Iceberg/Delta Lake integration | Low | StarRocks 3.x, DuckDB Iceberg, ClickHouse |

**Top Priority**: DEF-442-01 (continuous aggregates) and DEF-442-07 (DuckDB integration) — these have the highest impact-to-effort ratio. Continuous aggregates directly eliminate the #1 KB performance bottleneck (full-scan aggregation). DuckDB integration provides an immediate analytical engine with zero operations overhead, covering JOINs, window functions, and Parquet I/O that SQLite cannot handle.

### Cross-Cutting Themes
1. **Pre-computation over re-computation**: Every 2026 TSDB/OLAP system emphasizes materialized views, continuous aggregates, and pre-aggregation. NeoTrix's "compute everything from raw data" approach is the anti-pattern.
2. **Embedded analytical engines are production-ready**: DuckDB (MIT, in-process, 8.9/10) has reached the point where embedding an analytical engine is strictly better than hand-rolling analytics in application code. NeoTrix should adopt this.
3. **Data lifecycle management is non-negotiable**: TTL, retention policies, downsampling, and partitioning are standard in every 2026 data system. NeoTrix's "store everything forever" approach will hit wall at scale.
4. **Columnar for analytics, row-based for transactions**: The 2026 consensus is hybrid: row-store for writes/transactions (TimescaleDB Hypercore), columnar for reads/analytics (InfluxDB 3.0 Parquet, DuckDB). NeoTrix is 100% row-based.
5. **Open table formats are the interop layer**: Iceberg/Delta Lake are the 2026 standard for cross-engine data sharing. NeoTrix's proprietary format creates lock-in without benefit.

# Data Engineering for AI Systems — Research Brief (2025–2026)

**Research scope**: Streaming data processing, data versioning/lineage, feature stores, data quality validation
**Sources**: 30+ sources including InfoQ Trends 2025, Conduktor Streaming Report, DEW Year in Review, RisingWave, Databricks, Confluent, academic papers (EDBT 2025), ISO/IEC 5259 series, open-source projects (Aegis DQ, ADRI, ODVS, MLineage, PrismaDV)

---

## Finding 1: The Streaming-Lakehouse Merger — Stream-Table Duality Becomes Reality

**Key Insight**: The fifteen-year-old Lambda Architecture (maintain separate batch + streaming paths) is dead. In 2025, Apache Paimon and Apache Fluss emerged as lake formats designed for real-time updates, offering the high-throughput ingestion of a stream with the query capabilities of a lakehouse table. The barrier between "Stream" and "Table" dissolved — you ingest data once and it's immediately available for both real-time operational lookups and historical analytical queries.

- **Source**: DEW Year in Review 2025 (Ananth Packkildurai), RisingWave Real-Time Data Stack 2026, Databricks Lakehouse//RT announcement
- **Breakthrough**: Apache Iceberg standardized lakehouse storage with incremental scan support (every commit = snapshot). Kafka tiered storage by default reduces retention costs. Spark Structured Streaming real-time mode achieves p99 latencies in single-digit milliseconds.

**NeoTrix Integration**:
- NeoTrix's KB pipeline currently separates ingestion from serving. The streaming-lakehouse merger could unify NT-MEMORY's storage layer: ingest once via CDC, serve immediately for both real-time (GWT attention routing) and historical (experience querying).
- Directly addresses the "pipeline duality" gap — NT-WORLD's UnifiedCrawler could write to Iceberg once and serve both streaming aggregation (for real-time consciousness monitoring) and batch analytics (for SEAL pipeline evolution analysis).
- **Complexity**: Medium. Requires adopting Iceberg as the unified table format across NT-MEMORY storage, but eliminates the need for separate streaming/batch paths.

---

## Finding 2: End-to-End ML Provenance — MLflow2PROV and W3C PROV Compliance

**Key Insight**: MLflow's provenance capabilities remain rudimentary — focused on model training, ignoring data preprocessing and feature transformation steps. MLflow2PROV introduces a W3C PROV-compliant provenance model that captures end-to-end traces by combining MLflow tracking events with Git repository activities. The tool continuously extracts provenance graphs, enabling querying via graph databases (Neo4j) and answering questions like "which commit, which data, whose hands produced the model running in production?"

- **Source**: MLflow2PROV (Schlegel & Sattler, EDBT 2025), PROLIT (LLM-guided provenance, Springer 2025)
- **Breakthrough**: Three granularity levels for provenance (Full, Sketch, Only Columns) — users select precision vs. overhead tradeoff. LLM automatically rewrites user scripts to capture provenance transparently, without requiring code changes.

**NeoTrix Integration**:
- NeoTrix's SEAL pipeline runs exploration→distillation→self-test→absorption cycles but lacks formal provenance tracking between cycle outputs. MLflow2PROV's model could instrument each SEAL phase transition, capturing which KB snapshots produced which evolution artifacts.
- The three-granularity approach maps to NeoTrix's SelfTest tiers (T1 existence, T2 registration, T3 production wiring) — different provenance depths for different maturity levels.
- **Complexity**: Low-Medium. NeoTrix already has cycle tracking via experience-tree; adding W3C PROV-compliant edges between cycle phases and KB artifacts is a metadata layer addition.

---

## Finding 3: Streaming SQL Databases Replace Traditional Feature Stores

**Key Insight**: Traditional feature stores (Feast, Tecton) were designed for batch-era ML pipelines with hour-level freshness. In 2026, streaming SQL databases (RisingWave, Apache Fluss) eliminate the offline/online split entirely: define features in SQL, the engine maintains materialized views continuously, serving layer reads via PostgreSQL protocol. No separate Redis cluster, no sync pipeline, no training-serving skew.

- **Source**: RisingWave Real-Time Feature Store 2026, Feature Store Comparison (Tacnode), Lyft Feature Store Architecture (2026)
- **Breakthrough**: RisingWave achieves single-digit millisecond feature lookups by maintaining pre-computed materialized views. Apache Fluss adds columnar streaming (Arrow IPC) with 10× read throughput improvement via column pruning. Tecton acquired by Databricks (2025) → commercial landscape shifting.

**NeoTrix Integration**:
- NeoTrix's NT-MIND SEAL pipeline and NT-ACT action execution both consume features (self-test metrics, capability health scores, emotion states). Currently these are computed ad-hoc. A streaming SQL feature store would maintain continuously-updated feature views for the ConsciousnessTree's 11 branches — enabling GWT attention routing to query fresh health metrics instead of stale snapshots.
- Specifically: feature views for `system_health_snapshot`, `module_maturity_score`, `evolution_velocity`, `phi_coherence` could be materialized in real-time from EventBus events.
- **Complexity**: Medium-High. Requires integrating a streaming SQL engine (RisingWave or Fluss) as a new subsystem, but replaces scattered feature computation across NT-MEMORY and NT-CORE.

---

## Finding 4: Agentic Data Quality — LLM-Powered Root Cause Analysis and Auto-Fix

**Key Insight**: Data quality validation has shifted from static rule-checking to agentic, LLM-powered diagnosis. Aegis DQ introduces a 5-node agentic pipeline (plan→execute→reconcile→remediate→report) that generates rules from business docs, validates against warehouses, and diagnoses every failure with LLM root-cause analysis plus SQL fix proposals. ADRI (Agent Data Readiness Index) takes a lighter approach: executable YAML data contracts enforced as decorators before data reaches agent steps.

- **Source**: Aegis DQ (aegis-dq.dev), ADRI (adri-standard), DQuaG (GNN-based validation, EDBT 2025), PrismaDV (task-aware validation, 2026), ISO/IEC 5259 series (2025)
- **Breakthrough**: 31 rule types covering completeness, uniqueness, validity, referential integrity, timeliness, volume, and ML anomaly detection. ISO/IEC 5259-5:2025 establishes the first international data quality governance framework for analytics and ML.

**NeoTrix Integration**:
- NeoTrix's KB data flows through multiple pipelines (crawl→parse→classify→embed→store) but lacks systematic quality gates. Aegis DQ's agentic pipeline could instrument each NT-WORLD ingestion stage: validate crawled content against contracts before it enters KB, preventing "garbage in" that degrades VSA HyperCube representations.
- ADRI's decorator pattern maps directly to NeoTrix's trait-based architecture: `#[data_quality_guard]` on ingestion functions, with YAML contracts defined per content type.
- The ISO/IEC 5259 governance framework provides the compliance basis for NT-SHIELD's audit dimensions (D1-D50).
- **Complexity**: Medium. Aegis DQ is Apache 2.0 licensed with DuckDB adapter (no external warehouse needed for local development). Integration via MCP protocol aligns with NeoTrix's tool architecture.

---

## Finding 5: Single-Engine Streaming SQL — The Architecture Simplification Fork

**Key Insight**: The Kafka+Flink+ClickHouse stack (2018–2024) has fragmented into three competing architectures. The winning fork for most teams: single-engine streaming SQL (RisingWave, Materialize, Apache Fluss) that collapses source→materialized view→query into one system. Three teams become one. Apache Fluss 0.9 (March 2026) introduces columnar real-time analytics storage with Arrow IPC, server-side column pruning (10× read throughput), and Aggregation Merge Engine that pushes real-time aggregation into storage.

- **Source**: Streaming OLAP: The Post-Kafka Stack (The AI Vibe, June 2026), RisingWave blog, Apache Fluss GitHub
- **Breakthrough**: Kafka isn't dying — it's becoming plumbing. For single-team real-time analytics, one of the three forks is better than the full Kafka+Flink+OLAP stack. Decision axis: complexity (streaming SQL wins), cost (table-format-as-stream wins), latency (OLAP-eats-streaming wins).

**NeoTrix Integration**:
- NeoTrix's current architecture has NT-IO handling multiple data sources (LLM providers, web APIs, file systems). A single-engine streaming SQL layer could replace scattered event processing: EventBus events → RisingWave materialized views → queryable by any NT-* domain.
- This directly reduces operational complexity for NT-MEMORY's KB update pipeline — instead of separate crawl→process→store→query paths, a single streaming SQL engine handles ingestion, transformation, and serving.
- **Complexity**: Low-Medium for new subsystems, but requires evaluating whether NeoTrix's existing EventBus already covers this use case (it may, at a lower fidelity).

---

## Finding 6: Dataset Versioning on Iceberg — ODVS and Content-Addressed Lineage

**Key Insight**: Production-grade dataset versioning now runs on Apache Iceberg + Spark + S3. ODVS provides immutable versioned snapshots, time-travel reads, schema evolution, hash-based deduplication, and full lineage tracking from source URI → transforms → Iceberg snapshot. MLineage adds temporal evolution tracking ("git blame for ML models") with blame() to identify which update caused a metric regression.

- **Source**: ODVS (GitHub), MLineage (GitHub), pipeline-lineage-tracker (GitHub), Metaxy (multimodal lineage)
- **Breakthrough**: Content-addressed model resolution (SHA-256 of model file itself) ensures lineage travels with the bytes. Metaxy introduces prunable updates for multimodal pipelines — detecting which downstream paths are unaffected by upstream changes.

**NeoTrix Integration**:
- NeoTrix's KB already uses SQLite with versioning, but lacks formal dataset versioning for training/fine-tuning data used by NT-MIND's distillation pipeline. Iceberg-based versioning would enable time-travel queries on KB snapshots — "what did the knowledge base look like 3 SEAL cycles ago?"
- Metaxy's prunable updates align with NeoTrix's Dark Forest axiom — if a module's input data hasn't changed, don't trigger downstream re-evaluation.
- **Complexity**: Medium. Requires Iceberg integration for KB storage, but ODVS provides a reference implementation.

---

## Finding 7: Real-Time Spark — Millisecond Latency Without Replatforming

**Key Insight**: Apache Spark Structured Streaming introduces real-time mode (August 2025) — a new trigger type that processes events as they arrive with p99 latencies in single-digit milliseconds. Enabled via a single config change (`RealTimeTrigger.apply()`), no code rewrites required. Uses streaming shuffle for inter-stage data transfer and concurrent stage scheduling.

- **Source**: Databricks Blog (August 2025), Data Engineering Guide 2025
- **Breakthrough**: Eliminates the choice between "use Spark (familiar, ecosystem) but accept micro-batch latency" vs. "use Flink (lower latency) but rewrite everything." Same APIs, same code, millisecond latency.

**NeoTrix Integration**:
- NeoTrix's NT-WORLD crawl pipelines currently use batch processing. Spark real-time mode could enable streaming ingestion with millisecond freshness for real-time knowledge updates — critical for NT-CORE's GWT attention routing which needs fresh signals.
- **Complexity**: Low. Single config change on existing Spark jobs. No architectural restructuring.

---

## Integration Priority Matrix

| Priority | Finding | Effort | Impact | NeoTrix Domain |
|----------|---------|--------|--------|----------------|
| P0 | Agentic Data Quality (Aegis DQ) | Medium | Critical — prevents KB data degradation | NT-SHIELD + NT-WORLD |
| P0 | Single-Engine Streaming SQL | Low-Med | High — simplifies pipeline architecture | NT-MEMORY + NT-IO |
| P1 | Streaming-Lakehouse Merger | Medium | High — unifies storage model | NT-MEMORY |
| P1 | End-to-End ML Provenance | Low-Med | Medium — instruments SEAL pipeline | NT-MIND |
| P2 | Streaming Feature Stores | Med-High | High but complex — new subsystem | NT-CORE + NT-MIND |
| P2 | Dataset Versioning on Iceberg | Medium | Medium — formalizes KB versioning | NT-MEMORY |
| P3 | Spark Real-Time Mode | Low | Medium — faster ingestion | NT-WORLD |

---

## Defects/Gaps Addressed by These Findings

1. **KB data quality vacuum**: NeoTrix lacks systematic validation before data enters the KB. Aegis DQ fills this with contract-based validation + LLM diagnosis.
2. **Pipeline dualism**: Separate batch/streaming paths create maintenance burden. Streaming-lakehouse merger eliminates this.
3. **SEAL provenance blindness**: No formal trace from KB snapshot → evolution artifact → absorbed capability. MLflow2PROV's W3C PROV model provides the missing graph.
4. **Feature staleness in consciousness monitoring**: GWT attention routing queries stale health metrics. Streaming SQL materialized views keep them fresh.
5. **No dataset versioning**: KB snapshots lack Iceberg-style time-travel and content-addressed deduplication.

---

*Generated: 2026-09-05 | Sources verified via web search*

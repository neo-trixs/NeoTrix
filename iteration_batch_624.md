# Iteration Batch 624 — Data Governance, Quality & Lifecycle Research

**Date**: 2026-09-06  
**Research Loop**: 624/10000+  
**Previous Batch (623)**: Testing pyramid→architecture-shaped hybrid, Pact V4 bidirectional contracts, can-i-deploy mandatory CI gate, SelfTest lacks resource constraints, no consumer-driven contracts for NT boundaries.

---

## 1. DATA GOVERNANCE — NEW FINDINGS

### 1.1 Lineage-Driven Metadata Propagation (Google Cloud Governance Agent)
**Source**: https://cloud.google.com/blog/products/data-analytics/governance-on-autopilot-automate-data-governance-with-lineage (Aug 2026)

**What's NEW**: Google's Governance Agent propagates metadata (descriptions, PII tags, glossary terms, trust scores) automatically downstream via column-level lineage. When a column passes through transformations (SUM, CASE WHEN, COALESCE), it reads the SQL to generate accurate descriptions rather than copying stale upstream ones.

**Defect Found — NT-MEMORY Has No Column-Level Lineage**:
- NeoTrix KB stores nodes (entities) and edges (relations) but has zero column-level lineage between KB fields.
- When an `experience` entry in one namespace references a `domain_nt_*` entity, there is no lineage trail showing which fields were derived vs. passed through.
- Impact: If a KB schema evolves, there's no automated way to propagate the change to downstream consumers (e.g., `neotrix-experience query` results depend on field structures that may have shifted).

**Action**: Add a `lineage_edges` table to KB with `(source_namespace, source_field, target_namespace, target_field, transformation_type)` — required for any cross-domain schema evolution.

### 1.2 MCP Servers as Governance Delivery Mechanism
**Source**: https://atlan.com/know/data-catalog-for-ai/ (Mar 2026)

**What's NEW**: Modern data catalogs expose governed context to AI tools via MCP (Model Context Protocol) servers. Bidirectional sync ensures governance decisions propagate to consumers and context flows back. Single fact source for what AI agents can access.

**Defect Found — NT-IO Has No MCP Governance Layer**:
- NeoTrix's LLM provider integration (`nt_io`) has no MCP-based governance for what context agents can consume.
- Egress Privacy Guard blocks outbound leaks, but there's no inbound context governance — agents can consume unvetted KB data without lineage/quality signals.
- A hallucinated response from an LLM could reference stale or unvalidated KB entries with no audit trail.

**Action**: Create a `ContextGatekeeper` in NT-IO that wraps KB queries with lineage freshness + quality score checks before returning context to LLM agents.

### 1.3 AI Gateway for Agent Behavior Governance
**Source**: https://www.databricks.com/blog/whats-new-unity-catalog-data-ai-summit-2026 (Jun 2026)

**What's NEW**: Unity Catalog's AI Gateway governs what AI agents **do**, not just what they **access** — models, agents, tools, and MCPs under one runtime governance layer. Tracks agent behavior lineage (which agent called which tool with which data).

**Defect Found — NT-ACT Has No Agent Behavior Lineage**:
- NeoTrix tools (MCP tools in `nt_act`) have no runtime behavior tracking.
- When `nt_core_consciousness_task` delegates to subagents, there's no lineage graph showing which subagent used which tool with which data.
- If a subagent produces a corrupted output, root cause analysis is impossible without tracing the tool→data→output chain.

**Action**: Add an `agent_behavior_log` to EventBus or KB: `(agent_id, tool_id, input_hash, output_hash, timestamp, lineage_parent_id)`.

---

## 2. DATA QUALITY — NEW FINDINGS

### 2.1 GX Cloud Shutdown + OSS GX Core as Standard
**Source**: https://www.twicedata.com/labs/data-validation-with-great-expectations (May 2026), https://github.com/great-expectations/great_expectations (v1.17.2)

**What's NEW**: Great Expectations Cloud was acquired and shut down ~June 6, 2026. GX Core (OSS, Apache 2.0) is the only maintained path. v1.17.2 shipped May 14, 2026. The "multi-tool validation" pattern is now standard: GX for contract-shape assertions, Soda for SQL-native observability, Elementary for dbt anomaly detection.

**Defect Found — NeoTrix Has No Data Validation Layer**:
- The SEAL pipeline (`SEAL Pipeline`) processes knowledge through Soil→Roots→Trunk→Branches→Fruits→Core stages.
- Zero validation checkpoints exist between stages. No schema validation, no distribution checks, no row-count bounds.
- If `nt_memory` ingests corrupted experience data, it flows through all 6 stages undetected.

**Action**: Define validation checkpoints at each SEAL stage transition: Stage 0→1 (schema contract), Stage 2→3 (distribution check), Stage 4→5 (output quality gate).

### 2.2 dbt-expectations Package Deprecated
**Source**: https://www.twicedata.com/labs/data-validation-with-great-expectations (May 2026)

**What's NEW**: The `dbt-expectations` package by Calogica is officially unmaintained since late 2024 (README banner: "This package is no longer actively supported"). Last release Sept 2024. Teams using it need to migrate to Elementary for dbt-native anomaly detection.

**Defect Found — NEO-TRIX DEPENDENCY HYGIENE GAPS**:
- Batch 623 identified no resource constraints on SelfTest. This adds a new dimension: **unmaintained dependencies are invisible risk**.
- NeoTrix's `Cargo.toml` dependencies should be periodically audited for maintenance status, not just version compatibility.
- No mechanism exists to flag dependencies that haven't had a commit in N months.

**Action**: Add a `DependencyHealthCheck` to ConsciousnessTree that scans `Cargo.lock` and flags crates without commits in 90 days.

### 2.3 Three-Checkpoint Validation Pattern
**Source**: https://www.twicedata.com/labs/data-validation-with-great-expectations (May 2026)

**What's NEW**: Production-grade data quality uses three distinct checkpoints:
- **Checkpoint A (Ingest gate)**: Validates contract with source before writing to storage
- **Checkpoint B (Gold gate)**: Validates after transformation, before consumers read
- **Checkpoint C (Drift watch)**: Runs on schedule, compares current snapshot to rolling 30-day baseline

**Defect Found — NT-MEMORY Has No Read-Side Quality Gate**:
- NT-MEMORY serves data to NT-CORE, NT-MIND, and NT-ACT via KB queries.
- No validation exists at the read boundary — consumers receive whatever the KB returns without quality signals.
- A query to `neotrix-experience query --kw` could return stale or partially-written experience records.

**Action**: Add a `QueryQualityGate` to NT-MEMORY read path that attaches lineage freshness + completeness scores to query results.

### 2.4 Proactive Validation at Source
**Source**: https://medium.com/towards-data-engineering/proactive-data-quality-checks-with-great-expectations-4096b2163fcf (Jan 2026)

**What's NEW**: Best practice is to validate data as close to the source as possible — before it enters ingestion layers. Proactive validation catches issues 85% faster than reactive monitoring.

**Defect Found — NT-WORLD Has No Source-Side Validation**:
- NT-WORLD's `UnifiedCrawler` fetches external data but has no validation at the fetch boundary.
- Raw data enters the pipeline without schema contracts or quality assertions.
- If an external API changes its response schema, NeoTrix silently ingests malformed data.

**Action**: Add a `SourceContractValidator` to NT-WORLD fetchers that validates API responses against declared schemas before passing to parsers.

---

## 3. DATA LIFECYCLE — NEW FINDINGS

### 3.1 Five-Stage ILM with Automated Disposition
**Source**: https://www.archondatastore.com/blog/information-lifecycle-management/ (May 2026)

**What's NEW**: Modern ILM defines 5 stages: Creation/Capture → Active Use → Archive/Retention → Legal Hold → Secure Disposition. The critical insight: **the deletion guarantee is what auditors ask for, not the delete statement.** NIST SP 800-88 requires verified sanitization, not just `DELETE FROM`.

**Defect Found — NT-MEMORY Has No Data Lifecycle Stages**:
- NeoTrix KB entries have no lifecycle metadata: no `created_at`, no `last_accessed`, no `retention_period`, no `disposal_policy`.
- KB entries accumulate forever — no mechanism to age, archive, or dispose of stale experience records.
- Over time, the KB grows unbounded with stale experiences that degrade query quality.

**Action**: Add lifecycle fields to KB nodes: `created_at: DateTime`, `last_accessed: DateTime`, `retention_days: Option<u32>`, `disposal_policy: enum {Keep, Archive, Delete, CryptoShred}`. Implement a `LifecycleManager` that runs as SEAL Phase-0.

### 3.2 Hot/Warm/Cold Tiering with Declarative Lifecycle Rules
**Source**: https://dev.to/gowthampotureddi/data-retention-archival-tiered-lifecycle-hotwarmcold-legal-hold-cost-3fho (Aug 2026)

**What's NEW**: Storage tiering is mandatory for cost control. S3 Standard→IA→Glacier→Deep Archive maps to access frequency. Declarative lifecycle rules (no cron) evaluate on the platform's schedule. ~70% of lake ages into IA/Glacier automatically, cutting storage bills ~60%.

**Defect Found — NT-MEMORY Has No Tiering Strategy**:
- All KB data lives in the same SQLite store regardless of age or access frequency.
- Frequently accessed experience records compete with rarely-accessed historical records for I/O.
- No mechanism to promote hot data or demote cold data.

**Action**: Define a 3-tier KB strategy: Hot (current session experiences, <7 days), Warm (recent experiences, 7-90 days), Cold (historical, >90 days). Implement with SQLite virtual tables or separate databases with transparent access.

### 3.3 Legal Hold Must Override TTL
**Source**: https://infinisynapse.com/en/blog/data-retention-policy (Jul 2026)

**What's NEW**: Legal holds suspend deletion when litigation or investigation requires preservation. The operating model must balance GDPR's right-to-erasure against compliance retention floors. Crypto-shredding (destroying encryption keys) satisfies both constraints — encrypted bytes remain to honor the hold while becoming unreadable for erasure.

**Defect Found — NT-MEMORY Has No Hold/Override Mechanism**:
- No concept of "legal hold" or "compliance freeze" exists in NeoTrix.
- If a historical experience is under review (e.g., for a security audit), there's no way to freeze it against deletion.
- Conversely, if a user requests data erasure (GDPR Article 17), there's no mechanism to propagate deletion across all KB copies (production, backups, exports).

**Action**: Add a `HoldManager` to NT-MEMORY with: `freeze(namespace, key, reason)`, `unfreeze(namespace, key)`, `is_held(namespace, key) -> bool`. All lifecycle operations must check holds before executing.

### 3.4 AI-Adjacent Data Needs Explicit Retention Rules
**Source**: https://infinisynapse.com/en/blog/data-retention-policy (Jul 2026)

**What's NEW**: NIST AI Risk Management Framework requires explicit retention rules for AI-adjacent data: training sets, prompt/response logs, derived datasets. Whatever an agent can query falls under the same schedules.

**Defect Found — NT-CORE Has No Retention Policy for AI Artifacts**:
- NeoTrix stores LLM prompt/response logs, experience records, and derived reasoning chains in KB.
- None of these have retention policies defined.
- A prompt log from 6 months ago that contains sensitive data persists indefinitely.

**Action**: Define retention categories for NeoTrix AI artifacts:
- Experience records: 365 days (configurable)
- LLM prompt/response logs: 90 days
- Reasoning chains: 180 days
- Heartbeat snapshots: 30 days

---

## 4. CROSS-DOMAIN DEFECTS (Governance + Quality + Lifecycle)

### 4.1 No Unified Data Governance Framework
**Source**: https://www.prnewswire.com/news-releases/nucleus-research-releases-2026-data-governance-technology-value-matrix-302807878.html (Jun 2026)

**Finding**: Leaders (Alation, Atlan, Collibra, Oracle, Salesforce) all converge on: metadata management + lineage + quality + governance workflows as a unified stack. Governance is no longer a separate concern — it's the runtime backbone for AI systems.

**Defect — NeoTrix Has No Cross-Domain Data Governance**:
- Each NT-* domain manages its own data independently.
- No shared governance framework enforces consistency across domains.
- If NT-WORLD ingests data that NT-MEMORY stores that NT-CORE reasons over that NT-ACT acts on — there's no single lineage graph spanning the full chain.

**Action**: Create a `DomainGovernance` trait in NT-META that all domains implement, providing: `lineage_query()`, `quality_score()`, `retention_check()`, `hold_status()`.

### 4.2 No Bidirectional Metadata Sync
**Source**: https://atlan.com/know/data-catalog-for-ai/ (Mar 2026)

**Finding**: One-way ingestion produces a catalog that drifts from reality. Bidirectional sync keeps the context layer current.

**Defect — NeoTrix KB Has No Sync Protocol**:
- When NT-WORLD discovers new information about a topic that NT-MEMORY has indexed, there's no mechanism to update the KB entry's metadata.
- Knowledge becomes stale without any feedback loop.

**Action**: Implement a `MetadataSync` protocol: domain modules publish metadata updates to EventBus, a coordinator merges changes back to KB, and version conflicts are resolved by last-writer-wins or explicit merge rules.

---

## 5. SUMMARY OF DEFECTS FOUND

| # | Defect | Domain | Severity | Source Pattern |
|---|--------|--------|----------|----------------|
| 1 | No column-level lineage in KB | NT-MEMORY | HIGH | Google Governance Agent |
| 2 | No MCP governance layer for inbound context | NT-IO | MEDIUM | Atlan MCP Server |
| 3 | No agent behavior lineage | NT-ACT | HIGH | Unity AI Gateway |
| 4 | No SEAL pipeline validation checkpoints | NT-MEMORY | HIGH | GX Three-Checkpoint Pattern |
| 5 | No dependency health monitoring | NT-CORE | MEDIUM | dbt-expectations deprecation |
| 6 | No read-side quality gate | NT-MEMORY | HIGH | GX Checkpoint B pattern |
| 7 | No source-side validation | NT-WORLD | HIGH | Proactive validation pattern |
| 8 | No KB lifecycle metadata | NT-MEMORY | HIGH | Five-Stage ILM |
| 9 | No tiering strategy | NT-MEMORY | MEDIUM | Hot/Warm/Cold pattern |
| 10 | No hold/override mechanism | NT-MEMORY | HIGH | Legal Hold requirement |
| 11 | No AI artifact retention policy | NT-CORE | MEDIUM | NIST AI RMF |
| 12 | No cross-domain governance framework | NT-META | HIGH | Unified governance pattern |
| 13 | No bidirectional metadata sync | NT-MEMORY | MEDIUM | Bidirectional sync pattern |

---

## 6. SOURCES CITED

1. Google Cloud Blog — "Governance on Autopilot: Automate Data Governance with Lineage" (Aug 2026)
2. Data Workers — "Data Lineage: Complete Guide to Tracking Data Flows in 2026" (Apr 2026)
3. Everest Group — "Modern Data Catalog Products PEAK Matrix 2026" (May 2026)
4. The Data Governor — "What Is a Data Catalog" (Mar 2026)
5. Nucleus Research — "2026 Data Governance Technology Value Matrix" (Jun 2026)
6. Atlan — "Data Catalog for AI: Capabilities, Uses & Tooling in 2026" (Mar 2026)
7. Databricks — "What's new with Unity Catalog at Data + AI Summit 2026" (Jun 2026)
8. Google Cloud Blog — "Introducing the Google Cloud Knowledge Catalog" (Apr 2026)
9. Great Expectations — GX Core (v1.17.2, May 2026)
10. Modern DataTools — "Great Expectations Review (2026)" (Mar 2026)
11. TD Labs — "Data validation in the lakehouse" (May 2026)
12. ADHDecode — "MLOps Data Quality: Validate Datasets with Great Expectations" (Apr 2026)
13. StackPractices — "Data Quality Guide: Validation, Profiling, Great Expectations" (Jul 2026)
14. Medium/TDE — "Proactive Data Quality Checks with Great Expectations" (Jan 2026)
15. Archon Data Store — "Information Lifecycle Management Explained" (May 2026)
16. Cloudian — "Data Archiving Strategy in 2026" (May 2025/2026)
17. DEV Community — "Data Retention, Archival & Tiered Lifecycle" (Aug 2026)
18. ShareArchiver — "Enterprise Data Archiving in 2026" (Feb 2026)
19. Multishoring — "Data Lifecycle Management: Retention and Compliance" (Jun 2026)
20. InfiniSynapse — "Data Retention Policy: How to Build One in 2026" (Jul 2026)
21. InfiniSynapse — "What Is a Data Retention Policy? Definition and 2026 Guide" (Jul 2026)

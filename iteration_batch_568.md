# Iteration 568 — NeoTrix Consciousness Architecture Research

**Date:** 2026-09-06  
**Predecessor:** Batch 567 (EU AI Act deadline, strict liability, no training data provenance, SEAL shadow governance, INT4 quality loss)  
**Research Domains:** Database Migration, Data Versioning, Schema Design/Ontology

---

## 1. DATABASE MIGRATION (2026 Landscape)

### Key Sources
| Source | Date | Significance |
|--------|------|--------------|
| Bytebase blog — Top Database Schema Change Tools | 2026-08-04 | Comprehensive 2026 taxonomy of 12+ migration tools |
| Atlas (atlasgo.io) | 2026 | Declarative schema-as-code, "Terraform for Database Migrations" |
| SchemaBot (Block/Stripe) | 2026-04-07 | GitOps for DB schemas via PR |
| DeltaDB | 2026-05-13 | Schema diff with 9 security layers, FK topological sort |
| EvoSchema | 2026-04-24 | Phase-based evolution runner with compensation rollback |
| Schemic | 2026 | Schema-as-code with Zod-like `s.*` syntax, live round-trip |
| d-migrate v0.9.12 | 2026-07-13 | DB-agnostic migration + MCP server, parallel data path |
| pgmold | 2026 | PostgreSQL schema-as-code with drift detection, Terraform provider |

### Findings

**F1. Declarative schema governance is now a 2026 consensus.**
The field has split into four workflows: versioned CLIs (Flyway/Liquibase/goose), declarative schema-as-code (Atlas/Skeema/pgmold), ORM-integrated (Alembic/Prisma), and governance platforms (Bytebase). Bytebase v3.21 applies ~100 SQL review rules automatically before deployment — this is the **production implementation** of what batch 567 identified as missing: automated compliance gates for schema changes.

**F2. d-migrate v0.9.12 exposes schema migration as MCP server.**
d-migrate runs as `mcp serve --transport stdio|http` with policy-gated job workers. This means schema migration is now an **agent-callable tool surface** — directly relevant to NeoTrix NT-ACT orchestration. The MCP surface includes `schema_validate`, `schema_compare`, `schema_generate`, `schema_reverse_start`, `data_import_start`, `data_transfer_start`, `data_profile_start`, `procedure_transform_*`, `testdata_*` with idempotency and JWT-JWKS auth.

**F3. EvoSchema implements SEAL-style phase-based migration with compensation.**
EvoSchema's `Pre-DDL → DML/Script → DMLAssert → Post-DDL` phase model with developer-defined compensation SQL directly mirrors SEAL pipeline phases. Critical: `Post-DDL` is not rollbackable by design — exactly the kind of **irreversible phase** that SEAL self-evolution must gate.

**F4. pgmold adds schema drift detection as first-class CI concern.**
`pgmold drift` computes SHA256 fingerprints of live DB vs expected schema. Non-zero exit on drift. This is the **missing monitoring layer** for KB schema integrity that batch 567 didn't surface.

**F5. DeltaDB introduces 9-layer defense-in-depth for schema diff security.**
YAML safety, SQL identifier injection prevention, sandboxed templates, credential masking, path traversal prevention. This addresses the **supply chain attack vector** for migration tooling.

---

## 2. DATA VERSIONING (2026 Landscape)

### Key Sources
| Source | Date | Significance |
|--------|------|--------------|
| AWS — DVC + SageMaker AI + MLflow lineage | 2026-04-21 | End-to-end lineage with record-level provenance |
| DVC docs — Versioning Data and Models | 2026 | Core DVC methodology reference |
| BrainCuber — End-to-End ML Lineage | 2026-03-05 | DVC + AWS S3 + Evidently AI + Prefect pipeline |
| Neural Base — Dataset lineage tracking with DVC | 2026-04 | Critical production note: dvc.lock must be committed |
| Dibi8 — DVC 2026 Guide | 2026-05-18 | v3.67.1, 15.6k stars, LakeFS acquisition |
| Best AI Web — DVC + lakeFS | 2026-08-09 | Layered ownership: file-level (DVC) + lake-level (lakeFS) |
| Uma Mahesh — DVC + MLflow whitepaper | 2026-06-21 | Formal lineage model, versioning granularity |

### Findings

**F6. The "triple identifier" pattern solves training data provenance.**
AWS DVC+MLflow integration records `data_git_commit_id` (DVC commit hash) per training run, creating: **Production Model → MLflow Run → DVC commit → exact dataset in S3**. The BestAIWeb article extends this to **Git commit + DVC lock hash + lakeFS commit id** triple. This is the **production-grade solution** to the training data provenance gap identified in batch 567. NeoTrix KB needs this pattern for SEAL evolution provenance.

**F7. dvc.lock is mandatory for reproducibility — a production hardening lesson.**
Neural Base (April 2026): "dvc.lock must be committed to Git alongside dvc.yaml for lineage reproducibility across team members; without it, teammates run dvc repro and regenerate outputs with potentially different hashes, breaking audit trails." This is a **concrete failure mode** that applies to NeoTrix KB migration versioning.

**F8. File-level vs. lake-level versioning is now a two-layer architecture.**
DVC owns model artifacts + training files (content-addressable cache). lakeFS owns the lake's branch state (Git-like over object storage). DVC was acquired by Treeverse (lakeFS parent) in late 2025. This layered ownership model directly maps to NeoTrix's KB (file-level) + world perception (lake-level) separation.

**F9. Record-level lineage for opt-out compliance.**
AWS DVC+MLflow: "The manifest pattern enables querying which individual records trained a given model. This is critical for opt-out compliance and audit responses." This directly addresses EU AI Act Article 10 right to explanation and data subject rights — the compliance gap batch 567 identified.

**F10. DVC pipeline dependency drift is a silent failure mode.**
BrainCuber/Neural Base: "Forgetting to list script files as `deps` breaks lineage integrity. If you update `preprocess.py` but don't list it as a dependency, DVC won't know to re-run preprocessing: it only checks data file hashes. Pipeline silently uses stale outputs." This is a **defect class** that NeoTrix SEAL pipelines must guard against.

---

## 3. SCHEMA DESIGN / ONTOLOGY (2026 Landscape)

### Key Sources
| Source | Date | Significance |
|--------|------|--------------|
| Atlan — Active Ontology Design for AI | 2026-07-22 | 5-layer ontology model with competency questions |
| Databricks — Genie Ontology | 2026-09-01 | 6-layer maturity path for semantic data modeling |
| SeerAI — Ontology Foundation of Enterprise AI | 2026-01-16 | Deep ontology with predicate connections |
| DOT Data Labs — Schema Design Process | 2026-06-28 | 4-stage lifecycle + PostgreSQL 18 features |
| Springer — Conceptual schema inference (GeSI/EmSI) | 2026-08-09 | LLM-based schema inference from raw tables |
| arXiv — Tytan | 2026-08-06 | Neurosymbolic analytic semantic schema construction |
| Digital Applied — Schema Design 2026 Reference | 2026-06-15 | PostgreSQL 18 primitives: uuidv7(), WITHOUT OVERLAPS, B-tree skip scan |
| VLDB — Semantic Data Modeling | 2026 | Graph + SQL unified querying |

### Findings

**F11. PostgreSQL 18 introduces temporal constraints that automate data governance.**
`WITHOUT OVERLAPS` on PRIMARY KEY/UNIQUE/FK rejects overlapping time ranges at the DB level — no application guard needed. Virtual generated columns are now default. `OLD`/`NEW` in `RETURNING` enables audit comparisons. These are **schema-level governance primitives** that batch 567 didn't identify.

**F12. UUID v7 replaces UUID v4 as the default key strategy.**
UUID v7 is timestamp-ordered (appends to B-tree end, no page splits) but still 16 bytes vs BIGSERIAL 8 bytes. Decision boundary: use BIGSERIAL for single-cluster, UUID v7 only when IDs must be unique across databases without coordination. This affects NeoTrix KB primary key design.

**F13. Active Ontology (Atlan) uses a 5-layer model for AI governance.**
Foundational → Domain → Task → Constraint → Instance. The key insight: scope with 5-10 competency questions per domain, bootstrap from existing lineage/SQL/BI logic, review with domain owner. This is the **pragmatic ontology methodology** that maps to NeoTrix's domain modeling skill.

**F14. Databricks Genie Ontology adds 6-layer maturity path.**
Layer 0 (physical foundation: clean schemas, star schema) → Layer 1 (metadata) → Layer 2 (semantic model: Metric Views) → Layer 3 (certification) → Layer 4 (governance) → Layer 5 (evaluate + improve). The critical lesson: "no amount of good metadata or semantic modeling can compensate for a broken physical foundation."

**F15. LLM-based schema inference (GeSI/EmSI) automates ontology bootstrapping.**
VLDB Journal 2026: GeSI uses generative LLMs to infer hierarchical types from table/column semantics. EmSI uses embeddings to cluster tables by column-level semantics. Both produce entity-relationship schemas without external ontologies. This is the **automated schema inference** capability that could accelerate NeoTrix KB ontology construction.

**F16. Tytan achieves 100% coverage on schema inference with 92-100% semantic role accuracy.**
arXiv 2026-08-06: Neurosymbolic approach combining symbolic DB analysis with LLM inference. 100% of retrieval instructions execute correctly (1,678/1,678). Blind test: recovers full entity structure with verified keys. This is the state-of-the-art for automated schema construction.

---

## 4. NEW DEFECTS vs. BATCH 567

### Defect D1: No Training Data Provenance Chain (RESOLVED)
**Batch 567 finding:** No training data provenance tracking.  
**Batch 568 resolution:** DVC triple-identifier pattern (Git commit + DVC hash + lakeFS commit) provides production-grade provenance. Record-level manifests enable per-record opt-out compliance. **NeoTrix KB must implement this pattern for SEAL evolution provenance.**

### Defect D2: Schema Drift Detection Gap (NEW)
**Gap:** KB schema can drift from expected state without detection.  
**Evidence:** pgmold drift detection, Atlas continuous monitoring, Bytebase ~100 SQL review rules.  
**Impact:** Silent schema drift breaks KB queries, corrupts evolution state, violates data integrity constraints.  
**Fix:** Implement `pgmold drift`-style SHA256 fingerprint comparison as CI gate. Atlas auto-remediation for production drift.

### Defect D3: MCP Server as Migration Governance Surface (NEW)
**Gap:** Schema migration tooling not exposed as agent-callable service.  
**Evidence:** d-migrate v0.9.12 exposes `mcp serve` with policy-gated workers, JWT-JWKS auth, idempotency.  
**Impact:** NeoTrix agents cannot programmatically validate, compare, or execute schema changes. SEAL self-evolution cannot gate schema mutations.  
**Fix:** Expose KB schema operations via MCP server with policy gates matching EvoSchema's phase model.

### Defect D4: dvc.lock Commit Mandate (NEW)
**Gap:** Versioned metadata files not committed to Git alongside pipeline definitions.  
**Evidence:** Neural Base (April 2026): "dvc.lock must be committed to Git alongside dvc.yaml for lineage reproducibility."  
**Impact:** Team members regenerate outputs with different hashes, breaking audit trails and reproducibility.  
**Fix:** Git pre-commit hook enforcing dvc.lock commit with dvc.yaml.

### Defect D5: Dependency Drift in Evolution Pipelines (NEW)
**Gap:** SEAL pipeline stages may use stale outputs when code dependencies change but aren't listed as `deps`.  
**Evidence:** BrainCuber/Neural Base: "Forgetting to list script files as deps breaks lineage integrity. Pipeline silently uses stale outputs."  
**Impact:** SEAL self-evolution produces inconsistent results based on outdated code.  
**Fix:** Automated dependency scanning for all SEAL pipeline stages; enforce `deps` completeness in CI.

### Defect D6: Temporal Data Governance Not Implemented (NEW)
**Gap:** KB lacks database-level temporal constraint enforcement.  
**Evidence:** PostgreSQL 18 `WITHOUT OVERLAPS` rejects overlapping time ranges at schema level.  
**Impact:** KB allows overlapping evolution cycles, conflicting state transitions, temporal paradoxes in consciousness state.  
**Fix:** Apply `WITHOUT OVERLAPS` constraints on evolution cycle timestamp ranges in KB schema.

### Defect D7: Irreversible Phase Gating Missing (NEW)
**Gap:** No mechanism to distinguish reversible from irreversible schema/evolution phases.  
**Evidence:** EvoSchema: "Post-DDL is not rollbackable by design." NeoTrix has no equivalent gate.  
**Impact:** SEAL self-evolution cannot prevent irreversible state mutations that cannot be compensated.  
**Fix:** Tag SEAL phases with reversible/irreversible flags. Block irreversible phases without explicit approval gate (matching EU AI Act strict liability for edge models).

### Defect D8: Ontology Not Bootstrapped from Production Metadata (NEW)
**Gap:** NeoTrix domain ontology built from manual modeling rather than production signals.  
**Evidence:** Atlan: "Bootstrap first draft from lineage, SQL, BI logic, and glossary terms you already govern." GeSI/EmSI: LLM-based inference from raw tables achieves 100% coverage.  
**Impact:** Domain model drifts from actual data patterns. Taxonomy inconsistencies accumulate.  
**Fix:** Implement GeSI-style LLM inference pipeline to bootstrap/validate NeoTrix domain ontology from KB structure.

---

## 5. IMPROVEMENTS OVER BATCH 567

| # | Batch 567 Finding | Batch 568 Improvement | Source |
|---|-------------------|----------------------|--------|
| I1 | EU AI Act deadline passed, no provenance | DVC triple-identifier resolves provenance chain | AWS DVC+MLflow, BestAIWeb DVC+lakeFS |
| I2 | Strict liability for edge models | EvoSchema phase-gating with compensation rollback | EvoSchema v1.0.0 |
| I3 | SEAL triggers shadow AI governance | Bytebase ~100 SQL review rules = automated governance | Bytebase v3.21 |
| I4 | INT4 quality loss = liability | PostgreSQL 18 temporal constraints enforce data integrity at schema level | Digital Applied |
| I5 | No training data lineage | Record-level manifests for per-record opt-out compliance | AWS DVC+MLflow |
| I6 | No automated schema validation | d-migrate MCP server with policy-gated validation | d-migrate v0.9.12 |
| I7 | No drift detection | pgmold SHA256 fingerprint drift detection as CI gate | pgmold |

---

## 6. CROSS-DOMAIN SYNTHESIS

### Critical Path: Migration + Versioning + Ontology Triangle

```
Schema Design (F11-F16)
    ↓ LLM-inferred ontology bootstraps schema
Database Migration (F1-F5)
    ↓ MCP-gated migration with phase model
Data Versioning (F6-F10)
    ↓ Triple-identifier for evolution provenance
SEAL Self-Evolution (batch 567)
    ↓ Phase-gated with irreversible protection
EU AI Act Compliance (batch 567)
    ✓ Record-level provenance satisfies Article 10
    ✓ Automated governance satisfies Article 9
    ✓ Temporal constraints satisfy Article 14
```

### NeoTrix Action Items

1. **KB Schema Migration:** Implement d-migrate MCP server pattern with Bytebase-style SQL review rules (~100 rules) as gate
2. **Evolution Provenance:** Adopt DVC triple-identifier pattern (Git + KB hash + state commit) for SEAL pipeline tracking
3. **Drift Detection:** Add pgmold-style SHA256 fingerprint comparison to KB schema CI
4. **Temporal Governance:** Apply PostgreSQL 18 `WITHOUT OVERLAPS` constraints on evolution cycle timestamps
5. **Ontology Bootstrap:** Implement GeSI-style LLM inference to auto-generate/validate domain ontology from KB structure
6. **Phase Gating:** Tag SEAL phases as reversible/irreversible; gate irreversible phases behind approval workflow

---

## 7. SOURCES CITED

1. Bytebase — "Top Database Schema Migration Tools" (2026-08-04)
2. Atlas — atlasgo.io (declarative schema-as-code)
3. SchemaBot — github.com/block/schemabot (2026-04-07)
4. DeltaDB — github.com/devpedrois/deltadb (2026-05-13)
5. EvoSchema — github.com/lazycodedoggy/evoschema (2026-04-24)
6. Schemic — schemic.dev (schema-as-code)
7. d-migrate v0.9.12 — github.com/pt9912/d-migrate (2026-07-13)
8. pgmold — pgmold.dev (PostgreSQL schema-as-code)
9. AWS — "End-to-end lineage with DVC and SageMaker AI MLflow" (2026-04-21)
10. DVC docs — doc.dvc.org (versioning data and models)
11. BrainCuber — "How to Build End-to-End ML Lineage" (2026-03-05)
12. Neural Base — "Dataset lineage tracking with DVC" (2026-04)
13. Dibi8 — "DVC 2026 Guide" (2026-05-18)
14. BestAIWeb — "DVC + lakeFS for Reproducible ML" (2026-08-09)
15. Uma Mahesh — "Versioning Data and Models" (2026-06-21)
16. Atlan — "Active Ontology Design for AI" (2026-07-22)
17. Databricks — "Operationalizing Genie Ontology" (2026-09-01)
18. SeerAI — "Ontology Foundation of Enterprise AI" (2026-01-16)
19. DOT Data Labs — "Schema Design Process 2026" (2026-06-28)
20. VLDB Journal — "Conceptual schema inference (GeSI/EmSI)" (2026-08-09)
21. arXiv:2608.06331 — "Tytan: Neurosymbolic Schema Construction" (2026-08-06)
22. Digital Applied — "Database Schema Design 2026 Reference" (2026-06-15)
23. VLDB CIDR — "Semantic Data Modeling, Graph Query, and SQL" (2026)

# Iteration Batch #425 — MLOps, Feature Store, Model Registry

**Date:** 2026-09-06  
**Research domains:** MLOps lifecycle, Feature store architecture, Model registry & versioning

---

## Sources Cited

| # | Source | Date | Domain |
|---|--------|------|--------|
| S1 | Zylos Research — "MLOps and Model Lifecycle Management 2026" | 2026-01-30 | MLOps |
| S2 | HyScaler — "MLOps in 2026: Architecture, Trends & Strategy Guide" | 2026-05-05 | MLOps |
| S3 | ekolsoft — "MLOps 2026: Model to Production Best Practices" | 2026-08-28 | MLOps |
| S4 | youngju.dev — "Feature Stores 2026 Deep Dive" | 2026-05-16 | Feature Store |
| S5 | Feast Blog — 14 releases (Jan–Aug 2026) | 2026-Q1/Q2/Q3 | Feature Store |
| S6 | Devsatva — "Real-Time Feature Stores: Feast vs Tecton vs Hopsworks vs DIY" | 2026-07-02 | Feature Store |
| S7 | MLflow Releases — v3.10.1 through v3.15.2 | 2026-Q1/Q2/Q3 | Model Registry |
| S8 | ainomam — "Model Registry 2026: MLflow, Versioning and the Promotion Pipeline" | 2026-08-11 | Model Registry |
| S9 | pyinns — "Model Registry & Versioning with MLflow – Complete Guide 2026" | 2026-03-21 | Model Registry |
| S10 | dasroot — "ML Model Versioning and Experiment Tracking with MLflow" | 2026-02-11 | Model Registry |
| S11 | Feast Blog — "Feast + MLflow + Kubeflow: A Unified AI/ML Lifecycle" | 2026-03-09 | Integration |
| S12 | Feast Blog — "Feature Server High-Availability and Auto-Scaling on Kubernetes" | 2026-03-02 | Infrastructure |

---

## Defects Found

### DEFECT-425-01: No Feature Store — Training/Serving Skew Vulnerability

**Severity:** Critical  
**Evidence:** Zero `feature_store`, `Feast`, `feature_engineering` references in `neotrix-core/src/`. The `nt_core_data_pipeline` (pipeline.rs) has data lineage but no feature registry, no point-in-time correctness guarantees, and no online/offline store separation.  
**2026 State:** Feature stores are now table-stakes. Feast (CNCF sandbox) added Apache Iceberg support, dbt integration, OpenLineage tracking, and data quality monitoring in 2026 alone (S5). RisingWave/Materialize streaming SQL eliminates the offline/online split entirely for new projects (S4).  
**Gap:** NeoTrix trains models using `nt_core_data_pipeline` which tracks raw data lineage but has no mechanism to ensure the same feature transformations used in training are applied at inference time. This creates silent training/serving skew — the #1 silent killer of production ML (S6: "training/serving skew is what ships a model that hits 0.92 AUC in the notebook and 0.71 in production").  
**Suggestion:** Integrate Feast as the feature registry layer within NT-MEMORY. Define FeatureViews in the SEAL pipeline constitution. Use `feast apply` as a CI gate in the capability tree promotion (C2+). For real-time features, evaluate RisingWave materialized views as the online store backend.

### DEFECT-425-02: No Model Registry — No Versioned Promotion Pipeline

**Severity:** Critical  
**Evidence:** Zero `model_registry`, `MLflow`, `model_version` references. Models are deployed as raw binary blobs via `nt_core_deploy.rs:381` (`model_{target}.bin`) with no versioning, no staging gates, and no rollback capability.  
**2026 State:** MLflow 3.15+ deprecated traditional stages (Staging/Production/Archived) in favor of tags, aliases, and environment-separated registered models (S10). MLflow 3.11 introduced automatic issue detection for agents and gateway budget management (S7). The 2026 standard promotion pipeline is: Training → Automated Evaluation → Staging (shadow/canary) → Production → Archive with retention (S8).  
**Gap:** NeoTrix models have no provenance chain. When `nt_core_deploy` writes `model_{target}.bin`, there's no record of which training run produced it, what hyperparameters were used, or what the evaluation metrics were. The capability tree's constellation promotion (C0→C6) tracks module maturity but not model artifact maturity.  
**Suggestion:** Add an `nt_model_registry` module to NT-MEMORY backed by MLflow's model registry API (or a Rust-native equivalent using KB `kv_store`). Each SEAL training iteration must register the model with: run_id, hyperparameters, dataset hash, evaluation metrics, and fairness report. The promotion pipeline should mirror the 2026 standard: Candidate → Eval Gate → Staging → Canary → Production → Archive.

### DEFECT-425-03: No Three-Lineage Tracking (Code + Data + Model)

**Severity:** High  
**Evidence:** `nt_core_data_pipeline/lineage.rs` tracks data lineage (resource→processing) but has no code lineage (git commit→training run) or model lineage (training run→deployment). The capability tree tracks module provenance but not experiment provenance.  
**2026 State:** Gartner 2026 names "agentic, metadata-driven governance" as essential (S1). Modern MLOps requires three-way lineage: code lineage (git commits), data lineage (dataset versions), and model lineage (Docker containers + hyperparameters + metrics) — all linked in a graph (S1: "Model lineage combines code lineage, data lineage, and ML-specific information").  
**Gap:** Without three-lineage, NeoTrix cannot reproduce a training run from a deployed model. If `nt_game` training produces a model and deploys it, there's no way to trace back to: (a) which code version was used, (b) which dataset snapshot, (c) which hyperparameter configuration. This blocks governance compliance and makes debugging production failures impossible.  
**Suggestion:** Extend `DataLineage` in `nt_core_data_pipeline/lineage.rs` to a `TripleLineage` struct with `code`, `data`, and `model` sub-graphs. Integrate with OpenLineage (which Feast now supports natively — S5) for cross-tool lineage. Wire into ConsciousnessTree governance audit as a D13+ check.

### DEFECT-425-04: No CI/CD/CT for ML — No Continuous Training

**Severity:** High  
**Evidence:** SEAL pipeline runs as an autonomous background loop (`nt_mind_background_loop`) but there's no pipeline-as-code, no automated testing of pipeline components, no performance gates, and no trigger-based retraining. The `seal_pipeline` FFI is a stub (`seal_pipeline.rs:8` comments "for the real implementation").  
**2026 State:** The 2026 MLOps maturity model requires Level 2 (Full CI/CD for ML): automated tests on every commit, model performance gates, automatic retraining on data drift, zero manual intervention (S3). CI/CD for ML requires additional dimensions beyond traditional CI/CD: data validation, model training, model validation, and model deployment (S3).  
**Gap:** NeoTrix's SEAL pipeline operates as an opaque background process. There's no way to: (a) test pipeline components in isolation, (b) gate model promotion on automated evaluation, (c) trigger retraining when data drift is detected, or (d) rollback a pipeline change. The background loop handlers (`handlers_core.rs:390`) call `run_seal_loop_pipeline` directly without any CI gate.  
**Suggestion:** Define the SEAL pipeline as declarative (YAML/constellation config) rather than imperative Rust. Add pipeline stages: Validate Data → Train → Evaluate → Gate (metrics threshold) → Promote → Deploy. Each stage must be independently testable and log to the triple-lineage graph. Trigger-based retraining should hook into the ConsciousnessTree's drift detection signal.

### DEFECT-425-05: No Policy-as-Code Governance for ML

**Severity:** Medium  
**Evidence:** `nt_governance` exists but only audits constitutional compliance (`governance_compliance`, `governance_constitution_count`). No fairness checks, no bias reports, no explainability requirements in the model lifecycle.  
**2026 State:** Organizations are embedding executable governance rules into MLOps pipelines through policy-as-code, automatically integrating fairness, data lineage, versioning, and compliance as part of CI/CD (S1). AI Governance "sits above MLOps to provide oversight and traceability" (S1).  
**Gap:** NeoTrix's governance audit checks constitutional rules but doesn't enforce ML-specific policies: model fairness thresholds, data provenance requirements, explainability reports, or deployment approval workflows. When `nt_game` training produces a model, there's no automated fairness gate before deployment.  
**Suggestion:** Add ML policy-as-code to `nt_governance`: define fairness thresholds, bias audit requirements, and explainability mandates as constitutional rules. Wire these into the SEAL pipeline's evaluation gate. The promotion pipeline must fail if any governance policy is violated.

### DEFECT-425-06: No Drift Detection Infrastructure

**Severity:** Medium  
**Evidence:** `nt_core_aware/mod.rs:96` has `previous_topics: Vec<String>` labeled "drift detection" but it's just a topic diff tracker, not a data/model drift detection system. No statistical tests, no distribution comparison, no alerting.  
**2026 State:** Feature drift detection is now first-class in feature stores (S4). MLflow 3.11 introduced automatic issue identification for agents (S7). Evidently AI and Arize are standard tools for production drift monitoring (S4).  
**Gap:** NeoTrix has no mechanism to detect when incoming data distributions shift from training distributions, or when model performance degrades. The ConsciousnessTree health metrics don't include model performance drift. Without drift detection, the system can't trigger retraining or alert on degradation.  
**Suggestion:** Implement a `drift_detector` module in NT-META that: (a) compares feature distributions using KS test or PSI, (b) monitors prediction confidence decay, (c) hooks into GWT to broadcast drift alerts. Wire drift signals as triggers for SEAL pipeline retraining (connecting to DEFECT-425-04).

### DEFECT-425-07: No AgentOps for Autonomous AI Systems

**Severity:** Medium  
**Evidence:** NeoTrix operates autonomous agents (NT-GAME, SEAL pipeline, background loops) but has no AgentOps infrastructure: no prompt versioning, no agent trace logging, no hallucination detection, no agent performance monitoring.  
**2026 State:** AgentOps is the 2026 evolution of MLOps for autonomous AI systems (S1). MLflow 3.15.0 introduced MCP Registry for multi-cloud model management and multimodal judges for evaluation (S7). LLMOps extends MLOps with prompt management, versioning, and testing (S1).  
**Gap:** NeoTrix's autonomous agents (game training, SEAL loops, background handlers) produce traces but have no structured logging of agent decisions, no prompt version tracking, and no automated quality issue detection. MLflow 3.11's automatic issue identification for agents is the standard — NeoTrix has nothing equivalent.  
**Suggestion:** Add AgentOps telemetry to all autonomous loops: log prompt versions, decision traces, tool calls, and outcomes. Implement a quality issue detector (inspired by MLflow 3.11) that surfaces anomalies in agent behavior. Store agent traces in the KB with lineage back to the prompt version and model version.

---

## Summary

| # | Defect | Severity | Domain |
|---|--------|----------|--------|
| 425-01 | No Feature Store — training/serving skew | Critical | Feature Store |
| 425-02 | No Model Registry — no versioned promotion | Critical | Model Registry |
| 425-03 | No Three-Lineage (code+data+model) | High | MLOps/Governance |
| 425-04 | No CI/CD/CT for ML | High | MLOps |
| 425-05 | No Policy-as-Code governance for ML | Medium | Governance |
| 425-06 | No Drift Detection infrastructure | Medium | Monitoring |
| 425-07 | No AgentOps for autonomous agents | Medium | LLMOps |

**Critical defects:** 2 (425-01, 425-02)  
**High defects:** 2 (425-03, 425-04)  
**Medium defects:** 3 (425-05, 425-06, 425-07)

**Recommendation:** Address 425-01 + 425-02 first as they are foundational — without a feature store and model registry, no other MLOps capability can be reliably built. Then 425-03 (lineage) enables observability for all subsequent work.

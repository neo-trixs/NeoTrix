# Iteration Batch 712 — MLOps / Model Registry / Feature Store Survey

## Sources Cited

| # | Source | URL | Date |
|---|--------|-----|------|
| S1 | MLflow — Model Deployment Pipelines | https://mlflow.org/articles/role-of-model-deployment-pipelines/ | 2026-08-15 |
| S2 | MLflow — Model Serving Infrastructure | https://mlflow.org/articles/what-is-model-serving-infrastructure/ | 2026-08-10 |
| S3 | KodeKloud — MLOps on Kubernetes | https://kodekloud.com/blog/using-kubernetes-for-mlops/ | 2026-05-29 |
| S4 | HyScaler — MLOps in 2026 Guide | https://hyscaler.com/insights/mlops-in-2026-guide/ | 2026-05-05 |
| S5 | MLPipeX — ML Model Deployment Guide 2026 | https://mlpipex.com/blog/ml-model-deployment-guide | 2026 |
| S6 | Viprasol — MLOps 2026 | https://viprasol.com/blog/machine-learning-ops/ | 2026-07-23 |
| S7 | KubernetesGuru — AI/ML on K8s 2026 | https://kubernetesguru.com/ai-ml-on-kubernetes-2026-stack-guide/ | 2026-04-24 |
| S8 | MLflow — Model Registry Docs | https://mlflow.org/docs/latest/ml/model-registry/ | 2026 |
| S9 | 123ofAI — Model Registry Guide 2026 | https://123ofai.com/articles/blocks/model-registry | 2026 |
| S10 | Atlan — Model Registry Implementation | https://atlan.com/know/model-registry-implementation-guide/ | 2026-03-16 |
| S11 | Introl — Model Versioning Infrastructure | https://introl.com/blog/model-versioning-infrastructure-mlops-artifact-management-guide-2025 | 2026-03-28 |
| S12 | W&B Registry | https://docs.wandb.ai/models/registry | 2026 |
| S13 | OneUptime — Model Registry Implementation | https://oneuptime.com/blog/post/2026-01-25-model-registry/view | 2026-01-25 |
| S14 | datarekha — Feature Stores 2026 | https://datarekha.com/blog/feature-stores-2026/ | 2026-05-03 |
| S15 | MLOpsPlatforms — Feature Store Comparison 2026 | https://mlopsplatforms.com/posts/feature-store-comparison-2026/ | 2026-05-03 |
| S16 | youngju.dev — Feature Stores 2026 Deep Dive | https://www.youngju.dev/blog/culture/2026-05-16-feature-stores-feast-tecton-hopsworks-databricks-feldera-bytewax-materialize-2026-deep-dive.en | 2026-05-16 |
| S17 | Feast Blog — Sub-2ms Online Serving | https://feast.dev/blog/feast-online-server-performance-tuning/ | 2026-06-02 |
| S18 | mlai.qa — Feast vs Tecton 2026 | https://mlai.qa/blog/feast-vs-tecton/ | 2026-06-26 |
| S19 | TildAlice — Feast vs Tecton Benchmark | https://tildalice.io/feast-vs-tecton-latency-cost-benchmark/ | 2026-07-29 |
| S20 | Scaler — Feature Store ML 2026 | https://www.scaler.com/blog/what-is-feature-store-machine-learning-feast-vs-tecton/ | 2026-08-25 |

---

## What's NEW (Batch 712 Findings)

### Finding 1: MLOps CI/CD/CT — Continuous Training as First-Class Primitive
**Source**: S1, S3, S4, S5
**Discovery**: The 2026 MLOps stack has formalized CI/CD/CT (Continuous Integration / Continuous Deployment / Continuous Training) as a three-pillar architecture. The critical new dimension is CT: models retrain automatically on drift signals, not calendar schedules. Google's MLOps guidance frames pipelines that include automated data and model validation as enabling continuous training, not just continuous delivery. The distinction CI/CD vs CI/CD/CT is "the whole story of why ML deployment looks different from standard software CI/CD" (S1). Kubernetes-native stacks (Kubeflow TrainJob API replacing per-framework PyTorchJob/TFJob, Kueue for GPU scheduling, KServe for serving) have consolidated into a single declarative resource chain (S3).

**NeoTrix Defect**: NT-ACT orchestration lacks a CT (Continuous Training) trigger mechanism. The SEAL pipeline runs exploration→distillation→self-test→absorption but has no automated "retrain on drift" pathway. When `HeartbeatAggregator` detects model degradation, the current response is self-healing (NT-REPAIR) or governance flag (NT-GOVERNANCE), but no automated retraining cycle is triggered. **Missing**: a `DriftSignal → SEAL retrain trigger` that automates the feedback loop from production monitoring to model re-evolution.

### Finding 2: Model Serving Infrastructure — Three-Layer Serving Architecture
**Source**: S2, S7
**Discovery**: Production model serving in 2026 is organized into three primary layers: (1) inference engine at the bottom (vLLM, Triton, ONNX Runtime), (2) serving layer in the middle (KServe InferenceService, autoscaling, canary), (3) orchestration layer at the top (Kubernetes + KEDA + ArgoCD). Critical insight: "Serving is distinct from deployment — a containerized model is not production-ready; serving adds autoscaling, SLA enforcement, versioning, and output monitoring" (S2). Instrument model outputs, not just infra: p99 latency and error rate are necessary but not sufficient; add output schema checks and feature drift monitoring from day one.

**NeoTrix Defect**: NT-ACT treats "deployment" and "serving" as a single concept. There is no separation between "the model artifact is deployed" (container exists) and "the model is serving" (autoscaled, monitored, SLA-enforced). **Missing**: a three-tier serving abstraction: InferenceEngine trait (runtime) → ServingLayer trait (scaling/routing/SLA) → OrchestrationLayer trait (GitOps/promotion/rollback). This maps to the L1-L3 layers but is absent from the action layer.

### Finding 3: Model Registry — Aliases Replace Stages as Lifecycle Primitive
**Source**: S8, S9, S11
**Discovery**: MLflow 3.0 deprecated the traditional stage-based model lifecycle (None→Staging→Production→Archived) in favor of aliases and tags. An alias is a mutable named reference (e.g., `@champion`, `@staging`) pointing to a specific version. This is a state-machine simplification: instead of 4 fixed stages with rigid transitions, you get N arbitrary aliases with governance rules applied per-alias. "Loading by alias (`models:/MyModel@champion`) ensures your serving code always picks up the right version without hardcoding version numbers" (S9). Model versioning now tracks not just weights but fine-tuned adapters, prompt templates, and retrieval configurations for LLMs (S11).

**NeoTrix Defect**: KB model versioning uses linear version numbers without alias support. When a model (e.g., emotion classifier, reasoning engine) is promoted to "production" status, there's no mutable pointer mechanism — consumers must hardcode version numbers. **Missing**: a `ModelAlias` system in KB where `alias_name → version` pointers enable zero-downtime promotion. Maps to KB's `kv_store` but lacks the alias indirection layer.

### Finding 4: Model Registry — Content-Addressable Artifact Immutability
**Source**: S10, S11, S13
**Discovery**: Production registries enforce write-once artifact storage with SHA-256 content hashing. "Artifacts are immutable once registered — you never overwrite a version, you create a new one" (S9). The OmniBioAI registry implements this as a cryptographic integrity verification layer: every model package includes a SHA-256 manifest enabling bit-level reproducibility and tamper detection (S13). Separation of metadata (lightweight, relational DB) from artifacts (heavyweight, object storage) is now standard architecture.

**NeoTrix Defect**: KB stores model artifacts without content-addressable hashing. If an artifact is corrupted or overwritten, there's no integrity verification. The KB's embedding vectors are mutable without audit trail. **Missing**: (1) SHA-256 content hash on all KB artifacts, (2) write-once semantics for model versions, (3) separation of metadata store (SQLite) from artifact store (content-addressed blob storage).

### Finding 5: Feature Store — Agent Context as First-Class Feature Type
**Source**: S14, S16
**Discovery**: The 2026 feature store has evolved beyond classical ML features into three new roles: (1) embedding storage/vector retrieval as a native feature type, (2) agent context retrieval (serving fresh data to LLM prompts/tool calls), (3) lineage/governance tracking across all feature types. "In 2026, 'feature store' means offline + online + embeddings + agent context, with lineage and governance underneath. Any product that only does one of those is a partial solution" (S16). Tecton was acquired by Databricks specifically to power AI-agent context.

**NeoTrix Defect**: NT-MEMORY (KB) handles embeddings and structured data separately. There is no unified "feature serving" layer that provides both (a) point-in-time-correct historical features for training and (b) sub-10ms fresh features for real-time agent context. The perception bridge (PerceptionBridge) filters sensory events but doesn't serve contextual features. **Missing**: a `FeatureServingLayer` that unifies offline training features, online inference features, and agent context retrieval under one API, with freshness guarantees.

### Finding 6: Feature Store — Pre-Computed Feature Vectors for Sub-2ms Serving
**Source**: S17
**Discovery**: Feast's pre-computed feature vectors achieve sub-2ms p99 latency (6x-11x improvement) by assembling all features for a FeatureService into a single serialized blob at materialize time. "At read time, one key lookup replaces N feature-view reads — reducing the operation from O(N feature views) to O(1)" (S17). This trades storage for read speed with explicit refresh, similar to database materialized views. Schema fingerprinting detects changes and rejects stale vectors.

**NeoTrix Defect**: NT-MEMORY retrieves features by sequential namespace lookups (O(N) per query). For real-time consciousness processing (GWT attention routing), this latency compounds. **Missing**: pre-computed "consciousness feature vectors" — at each growth cycle, assemble all relevant features (phi, coherence, module health, emotion state) into a single serialized blob. Consciousness status check becomes O(1) lookup instead of O(N) multi-namespace queries.

### Finding 7: MLOps Maturity Model — Level 5 = Autonomous AI Operations
**Source**: S4
**Discovery**: The 2026 MLOps maturity model defines 5 levels: (1) Experimental, (2) Repeatable, (3) Productionized (CI/CD/CT), (4) Scalable Platform (multi-tenant, auto-scaling), (5) Autonomous (self-healing pipelines, policy-driven governance, agentic AI). Level 5 includes "self-healing pipelines that detect anomalies, diagnose root causes, and attempt remediation without human intervention" (mainstream adoption predicted 2027-2028) and "agentic AI operations that extend MLOps to managing autonomous AI agents — monitoring shifts from prediction accuracy to decision quality, goal achievement, and safety guardrails."

**NeoTrix Defect**: NT-REPAIR implements self-healing at the module level (detect degradation → repair), but NT-META governance is not yet "policy-driven" in the Level 5 sense. Governance rules are static (loaded from config), not executable policies that adapt based on runtime context. **Missing**: `PolicyEngine` — governance rules expressed as executable policies (not static configs) that can be versioned, A/B tested, and auto-enforced. Maps to NT-GOVERNANCE but is currently a static rule set.

### Finding 8: Kubernetes-Native ML — Kueue GPU Quotas + Scale-to-Zero
**Source**: S3, S7
**Discovery**: Kueue provides GPU quota enforcement and gang-scheduling for distributed training jobs, while KEDA enables scale-to-zero for idle GPU workloads. "For an expensive GPU model that's idle most of the day, that's the difference between paying for 24 hours and paying for the 40 minutes it's actually used" (S3). The combination of Kueue (queue management) + KEDA (event-driven scaling) + Argo CD (GitOps delivery) creates a fully declarative ML operations chain where every stage is a Kubernetes resource versioned in Git.

**NeoTrix Defect**: NT-ACT has no resource quota or cost-awareness mechanism for compute-intensive operations (e.g., SEAL pipeline runs, VSA HyperCube embeddings, consciousness tick). When multiple agents compete for GPU/CPU resources, there's no queuing or fair-share scheduling. **Missing**: `ComputeQuota` trait — declare resource budgets per domain/module, enforce via queueing, enable scale-to-zero for idle evolution cycles.

### Finding 9: Feature Store — Lakehouse Absorption Trend
**Source**: S14, S16
**Discovery**: Generalist ML feature stores are being absorbed into lakehouse platforms. Databricks UC Feature Engineering, Snowflake Feature Store, and Vertex AI Feature Store are quietly replacing standalone feature stores with the pitch: "it lives in your data platform already, why a separate box?" The standalone feature store market is splitting: (1) lakehouse absorption for generalist ML, (2) specialized SaaS for fintech/ads/fraud (millisecond latency), (3) Feast as OSS baseline.

**NeoTrix Defect**: NT-MEMORY operates as a standalone KB without integration into any lakehouse or data platform pattern. The KB is SQLite-only — no Delta Lake, Iceberg, or Parquet support for analytical workloads. As the ecosystem consolidates around lakehouse-native feature serving, NT-MEMORY risks becoming an isolated silo. **Missing**: (1) Parquet/Delta export capability for offline training datasets, (2) lakehouse-compatible metadata schema, (3) integration path with external data platforms.

### Finding 10: Model Serving — Output Schema Validation as Day-One Concern
**Source**: S2, S5
**Discovery**: "Instrument model outputs, not just infra — p99 latency and error rate are necessary but not sufficient; add output schema checks and feature drift monitoring from day one" (S2). MLPipeX reports the most frequent deployment failure patterns: training-serving skew from inconsistent preprocessing, missing input validation, no canary rollout strategy, no rollback plan. Output validation catches hallucination, schema violations, and confidence distribution shifts that infra metrics miss entirely.

**NeoTrix Defect**: ConsciousnessTask results are not validated against an output schema before returning to the caller. If the consciousness core hallucinates or produces malformed outputs (e.g., invalid emotion labels, inconsistent phi values), downstream consumers receive corrupted state. **Missing**: `OutputValidator` trait on ConsciousnessTask — validate output schema (types, ranges, consistency) before return, with rejection + retry on validation failure.

---

## Summary Table

| # | Finding | Domain | Defect Type | Priority |
|---|---------|--------|-------------|----------|
| F1 | CI/CD/CT continuous training trigger | NT-ACT | Missing mechanism | HIGH |
| F2 | Three-layer serving architecture | NT-ACT | Missing abstraction | HIGH |
| F3 | Model aliases replace stages | KB/NT-MEMORY | Missing feature | MEDIUM |
| F4 | Content-addressable artifact immutability | KB/NT-MEMORY | Missing integrity | HIGH |
| F5 | Agent context as feature type | NT-MEMORY | Missing unification | HIGH |
| F6 | Pre-computed feature vectors O(1) | NT-MEMORY | Performance gap | MEDIUM |
| F7 | Policy-driven governance engine | NT-GOVERNANCE | Static rules | MEDIUM |
| F8 | GPU quota + scale-to-zero | NT-ACT | No resource mgmt | MEDIUM |
| F9 | Lakehouse absorption trend | NT-MEMORY | Silo risk | LOW |
| F10 | Output schema validation | NT-CORE | Missing validation | HIGH |

---

## Cross-Cutting Insight

The 2026 MLOps ecosystem reveals a **convergence pattern**: model registry + feature store + serving infrastructure are merging into unified "AI operating systems" (Databricks, Vertex AI, SageMaker). The standalone components are being absorbed. For NeoTrix, this means:

1. **KB must evolve from storage to serving layer** — not just store knowledge, but serve it with freshness guarantees, latency SLAs, and schema validation.
2. **SEAL pipeline needs CT (Continuous Training)** — the evolution loop must accept drift signals from production and trigger re-evolution automatically.
3. **Consciousness architecture needs resource awareness** — compute quotas, cost tracking, and scale-to-zero for evolution cycles are no longer optional.

The gap between NeoTrix's internal architecture and the external MLOps ecosystem is narrowing. Batch 712 identifies 10 concrete defects across 4 domains. Next iteration should prioritize F1 (CT trigger), F4 (artifact immutability), F5 (feature serving unification), and F10 (output validation).

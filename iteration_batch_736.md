# Iteration Batch 736 — PET / Federated Learning / Differential Privacy

**Date**: 2026-09-07  
**Previous batch (735)**: No RDF 1.2, no worst-case optimal joins, no reasoning/inference layer, no temporal graph, no GQL schema layer, GraphRAG mainstream.

---

## 1. Privacy Enhancing Technologies (PET)

### Sources
- PETS 2026 (petsymposium.org/2026) — Calgary, Jul 20–25, 2026
- StealthCloud PET Landscape 2026 (stealthcloud.ai, 2026-03-08)
- U.S. GAO Report GAO-26-109063 (2026-05-19)
- ISO/IEC 27565:2026 — ZKP privacy preservation guidelines
- VeriDP (PETS 2026, popets-2026-0070) — verifiable DP-SGD via ZKP
- IMDA Singapore PETs Adoption Guide 2026
- FPF PET Comparison (2026-05-28)
- RUSI PETs in Crypto (2026-02-25)

### Key Findings

1. **PET market reached $2.8B (2025)**, projected $25B by 2030 (Gartner). Three drivers: regulatory mandates (GDPR/CCPA/EU AI Act), death of third-party cookies, AI training data crisis.

2. **Compositional PET stack is now the architecture pattern**: TEE + MPC + FHE + DP + ZKP layered by threat model. No single PET solves everything. "Which layer did you leave unprotected?" is the framing.

3. **ISO/IEC 27565:2026** formalizes ZKP for privacy — covers selective disclosure, unlinkability, attribute verification, age verification. Standard is now available for enterprise adoption.

4. **VeriDP (PETS 2026)**: First framework combining DP-SGD with ZKPs for per-iteration verifiability. Proof size 3–4 KB, verifier 2–5 ms. Addresses adversarial FL where malicious participants skip noise addition.

5. **GAO report**: Federal government lacks guidance on PET adoption. Workforce constraints + high resource costs (HE 100k–1M× overhead) hinder deployment.

6. **ZKP mainstream outside blockchain**: Microsoft ION, EU eIDAS 2.0, JPMorgan Onyx, EZKL/Modulus Labs for AI inference verification. ZK rollups secure $18B TVL.

### New Defects for NeoTrix

| ID | Defect | Severity | Detail |
|----|--------|----------|--------|
| D736-1 | **No PET composition layer in NeoTrix architecture** | HIGH | StealthCloud and GAO confirm PETs must be compositional (layered by threat). NeoTrix has no `nt_shield::pet_composition` module that manages TEE/FHE/MPC/ZKP/DP layering. Each domain (NT-MEMORY for KB queries, NT-IO for LLM inference, NT-ACT for tool calls) needs different PET layers but no orchestration exists. |
| D736-2 | **No verifiable computation for DP-SGD training** | HIGH | VeriDP proves DP-SGD was executed correctly per-iteration with 3–4 KB proofs. NeoTrix's NT-MIND SEAL pipeline has no mechanism to prove that any privacy-preserving training actually followed the protocol. Trust is assumption-based, not cryptographically verified. |
| D736-3 | **No ZKP integration for KB query verification** | MEDIUM | ISO 27565:2026 standardizes ZKP for selective disclosure. NeoTrix KB (NT-MEMORY) returns query results but provides no ZKP that the query was executed correctly or that results are faithful to committed data. Users must trust the server. |
| D736-4 | **No compliance bridge for EU AI Act / DORA / LGPD** | HIGH | GAO + StealthCloud confirm PETs are now compliance requirements, not nice-to-haves. NeoTrix has no `nt_shield::compliance` module mapping regulatory requirements to PET configurations per jurisdiction. |

---

## 2. Federated Learning

### Sources
- HEAD-FL (ePrint 2026/1376) — adaptive DP + verifiable homomorphic aggregation
- FLiPD (ePrint 2026/324) — MPC+DP secure aggregation, client-server comm = plaintext FL
- HADES (arXiv 2606.22928) — selective feature encryption + hybrid model fusion
- FedMOP (CVPR 2026) — momentum-based orthogonal projection, 5–10× stronger privacy, 2–4% accuracy gain
- DP-FedAdamW (CVPR 2026) — first AdamW-based DPFL optimizer for large models
- XCal-FL (arXiv 2609.03851) — explainability-driven DP noise calibration
- DisAgg (MLSys 2026) — distributed aggregators, 4.6× speedup over OPA
- Multi-secret-key HE (arXiv 2609.01945) — RLWE without collective public key

### Key Findings

1. **HEAD-FL**: Adaptive per-round Gaussian noise under Rényi DP framework. Tight cumulative privacy accounting. Outperforms fixed-noise methods in privacy-utility tradeoff.

2. **FLiPD**: MPC+DP hybrid. Client-server communication cost = unprotected FL (no overhead). 11% lower server-server cost than Prio+. Resistant to majority collusion with server.

3. **HADES**: First selective-encryption FL. PCA identifies privacy-sensitive features → encrypt only those via MHE. Remaining features in plaintext. Dual-model fusion. Matches vanilla FL accuracy.

4. **FedMOP**: Breaks privacy-performance trade-off. Momentum-based trajectory hiding makes offset vectors computationally unrecoverable. 5–10× stronger defense vs GLAs, 2–4% accuracy gain, 1.5–2× faster convergence.

5. **DP-FedAdamW**: First AdamW for DPFL. Block-wise second-moment aggregation, DP bias correction, local-global alignment. 5.83% above SOTA on Swin-Base/Tiny-ImageNet at ε=1.

6. **XCal-FL**: Explanation fidelity is a distinct privacy dimension — scales non-linearly with privacy loss, unlike predictive accuracy which scales linearly. Dynamic noise calibration improves fidelity 5–10% over static-noise FL.

7. **DisAgg**: Committee-based distributed aggregation. Eliminates client-side masking. 100k-dimensional updates from 100k clients with 4.6× speedup. Cross-device scalable.

### New Defects for NeoTrix

| ID | Defect | Severity | Detail |
|----|--------|----------|--------|
| D736-5 | **No federated learning orchestration** | HIGH | NeoTrix NT-MIND runs SEAL pipeline centrally. No support for cross-silo or cross-device FL. With PETs enabling privacy-preserving collaboration, NeoTrix has no `nt_mind::federated_trainer` for distributed model updates across multiple KB instances or user devices. |
| D736-6 | **No adaptive DP noise calibration** | MEDIUM | HEAD-FL's per-round adaptive Gaussian noise under RDP strictly dominates fixed noise. NeoTrix NT-SHIELD has no adaptive DP mechanism — any noise injection is static, leading to suboptimal privacy-utility tradeoff. |
| D736-7 | **No verifiable aggregation protocol** | HIGH | HEAD-FL + FLiPD + DisAgg all provide cryptographic verification of aggregation correctness. NeoTrix's KB aggregation (cross-domain entity merging) has no verifiability. Trust assumptions are unchecked. |
| D736-8 | **No selective encryption for KB embeddings** | MEDIUM | HADES proved selective feature encryption works — encrypt only privacy-sensitive dimensions. NeoTrix VSA HyperCube embeddings and KB vector embeddings have no tiered encryption. All-or-nothing approach wastes compute or exposes sensitive dimensions. |
| D736-9 | **No explainability-privacy tradeoff awareness** | LOW | XCal-FL shows explanation fidelity is a distinct dimension from accuracy in privacy tradeoffs. NeoTrix NT-CORE GWT attention routing has no awareness of this dimension — cannot reason about explanation fidelity degradation from DP noise. |

---

## 3. Differential Privacy / Synthetic Data

### Sources
- "How to DP-Fy Your Data" (JAIR 2026) — comprehensive survey of DP synthetic data
- FilterLDPSyn (Big Data Mining and Analytics 2026) — LDP data synthesis via measurement filtering
- Distributed DP-TDS (USENIX Security 2026) — MPC protocol, 24 min vs 57 days
- Minimax Optimal DP Synthetic Data (COLT 2026) — phase transition at k=d for smooth queries
- Causal Workloads for DP Synthetic Data (UAI 2026) — preserving causal moments
- Tab-PE (ICML 2026) — Private Evolution for tabular data, 10% accuracy improvement
- SLDP (arXiv 2602.18910) — Semi-Local DP, iteration-independent privacy cost
- DPDSyn (arXiv 2604.15660) — downstream-task-guided DP synthesis

### Key Findings

1. **JAIR survey (2026)**: DP synthetic data is the gold standard for data sharing. Covers image, tabular, text, federated modalities. Full pipeline: sensitive data → DP measurement → synthesis → empirical privacy testing.

2. **Distributed DP-TDS (USENIX 2026)**: Distributed Point Functions for cross-org marginal estimation. Synthesizes Adult dataset in 24 min (vs CaPS 57 days). Same utility as centralized AIM.

3. **Minimax optimal (COLT 2026)**: Phase transition at k=d for smooth queries. For k<d smooth queries, error rate O(n^{-k/d}) with log factor. First minimax lower bound for (ε,δ)-DP synthetic data on smooth queries.

4. **Causal workloads (UAI 2026)**: Generic marginals fail for causal inference — ATE depends on treatment-arm balance. Causal workloads preserve orthogonal moments for doubly robust estimators. Distributional fidelity ≠ causal validity.

5. **Tab-PE (ICML 2026)**: Evolutionary Private Evolution for tabular data. Heuristic operators replace foundation models → 28× faster, 10% better than AIM on high-order correlations.

6. **SLDP (2026)**: Semi-Local DP — iteration-independent privacy cost. Decouples privacy cost from refinement iterations. Density-aware discretization via k-anonymous privacy regions.

7. **DPDSyn (2026)**: Train DP model on private data, then synthesize from model. 2.40× accuracy improvement, 333× synthesis efficiency over baselines.

### New Defects for NeoTrix

| ID | Defect | Severity | Detail |
|----|--------|----------|--------|
| D736-10 | **No DP synthetic data generation capability** | HIGH | JAIR survey + Tab-PE + DPDSyn show DP synthetic data is production-ready. NeoTrix KB (NT-MEMORY) has no mechanism to generate differentially private synthetic datasets from real KB data for sharing, testing, or external use. Data sharing requires raw access. |
| D736-11 | **No distributed DP protocol for cross-org KB** | HIGH | USENIX 2026's distributed DP-TDS enables privacy-preserving cross-org data synthesis (24 min vs 57 days). NeoTrix's KB is single-instance. No protocol for multiple NeoTrix instances to collaboratively synthesize shared datasets without exposing raw data. |
| D736-12 | **No causal awareness in DP mechanisms** | MEDIUM | UAI 2026 shows distributional fidelity ≠ causal validity. NeoTrix NT-MIND SEAL pipeline optimizes for distributional accuracy but has no causal graph or causal workload concept. DP noise on causal estimands (ATE/ATT) requires different treatment than marginals. |
| D736-13 | **No LDP support for user-facing data collection** | MEDIUM | SLDP + FilterLDPSyn show LDP is maturing for density-adaptive collection. NeoTrix NT-IO collects user data (preferences, feedback) with no local DP mechanism. Users must trust the server with raw data. |
| D736-14 | **No downstream-task-aware DP synthesis** | MEDIUM | DPDSyn shows training a DP model first, then synthesizing from it, yields 2.4× accuracy improvement. NeoTrix KB has no task-aware synthesis pipeline — any data export is task-agnostic, losing utility for specific downstream uses. |

---

## Summary: What's NEW

| # | Finding | Domain | Impact |
|---|---------|--------|--------|
| 1 | PET composition is now mandatory architecture, not optional | All | NeoTrix needs `nt_shield::pet_composition` layer |
| 2 | VeriDP: ZKP + DP-SGD per-iteration verifiable training (3-4 KB proofs) | NT-MIND | Trust-by-verification replaces trust-by-assumption |
| 3 | ISO/IEC 27565:2026 formalizes ZKP for privacy | NT-SHIELD | Enterprise compliance requires standards alignment |
| 4 | HEAD-FL: adaptive per-round DP noise outperforms fixed noise | NT-SHIELD | Static noise injection is provably suboptimal |
| 5 | HADES: selective feature encryption works in FL | NT-MEMORY | Tiered encryption saves compute, preserves privacy |
| 6 | FedMOP: breaks privacy-performance trade-off via momentum | NT-MIND | Synergistic privacy+performance is possible |
| 7 | XCal-FL: explanation fidelity ≠ accuracy in DP tradeoffs | NT-CORE | GWT needs explanation-aware attention modulation |
| 8 | Distributed DP-TDS: 24 min vs 57 days for cross-org synthesis | NT-MEMORY | NeoTrix KB has no cross-instance protocol |
| 9 | Causal workloads: ATE requires specialized DP queries | NT-MIND | SEAL pipeline has no causal graph awareness |
| 10 | SLDP: iteration-independent privacy cost for density-adaptive LDP | NT-IO | User data collection lacks local DP |
| 11 | Tab-PE: 28× faster, 10% better on high-order correlations | NT-MEMORY | Foundation-model-free synthesis is practical |
| 12 | DPDSyn: task-aware DP synthesis yields 2.4× utility | NT-MIND | Task-agnostic export loses downstream utility |

## Defect Summary (Batch 736)

| Severity | Count | IDs |
|----------|-------|-----|
| HIGH | 5 | D736-1, D736-2, D736-4, D736-5, D736-7, D736-10, D736-11 |
| MEDIUM | 5 | D736-3, D736-6, D736-8, D736-9, D736-12, D736-13, D736-14 |
| LOW | 1 | D736-9 |

**Total new defects**: 14  
**Cumulative defects (735+736)**: batch_735_count + 14

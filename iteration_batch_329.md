# Iteration Batch 329 — Cross-Domain External Research: Aerospace × Autonomous Vehicles × Maritime

**Date:** 2026-09-06  
**Domains:** Aerospace, Autonomous Vehicles, Maritime  
**Sources:** 27 external sources (research papers, industry reports, conference proceedings)  
**Method:** 9 parallel deep-web searches across 3 domains, 2026 timeframe

---

## 1. Sources Cited

### Aerospace
| # | Source | Date | URL |
|---|--------|------|-----|
| S1 | Axis Intelligence — AI Air Traffic Management Systems Guide | 2025-11 | https://axis-intelligence.com/2025/11/20/ |
| S2 | Landing Aero — AI in Air Traffic Control | 2026-01 | https://landing.aero/articles/en/ai-air-traffic-control-applications |
| S3 | Skycrumbs — AI in Aviation 2026 | 2026-06 | https://skycrumbs.com/blog/ai-aviation-2026 |
| S4 | Yenra — AI ATC Optimization: 20 Operational Improvements | 2026-03 | https://yenra.com/ai20/air-traffic-control-optimization |
| S5 | SmallSat 2026 — Satellite Constellations: Survey, Trends, Economic Viability | 2026-08 | https://digitalcommons.usu.edu/smallsat/2026/all2026/53/ |
| S6 | ORBilu — Satellite Constellation Optimization for 6G | 2026 | https://orbilu.uni.lu/handle/10993/68098 |
| S7 | Williams Rogers et al. — Optimal Satellite Constellation Configuration Design (MILP) | 2025-07/2026-03 | https://arxiv.org/abs/2507.09855 |
| S8 | ScienceDirect — UAV Path Planning in Complex Environments (RGO-RRT*) | 2026-01 | https://www.sciencedirect.com/science/article/pii/S0305054825003259 |
| S9 | Springer — UAV Path Planning and Trajectory Optimization Survey | 2025-12/2026 | https://link.springer.com/article/10.1007/s13369-025-10971-8 |
| S10 | ACM — Comprehensive Review of Path Planning Techniques for UAVs | 2025-05 | https://dl.acm.org/doi/10.1145/3737280 |

### Autonomous Vehicles
| # | Source | Date | URL |
|---|--------|------|-----|
| S11 | DriveX Workshop @ ECCV 2026 — Foundation Models for Autonomous Driving | 2026-09 | https://drivex-workshop.github.io/eccv2026/ |
| S12 | VeteranAD (AAAI 2026) — Perception in Plan: Coupled Perception and Planning | 2025-08 | https://github.com/LogosRoboticsGroup/VeteranAD |
| S13 | Springer — Perception in Autonomous Vehicles: Catalyzing Evolution | 2026-07 | https://link.springer.com/article/10.1007/s41315-026-00551-w |
| S14 | ScienceDirect — Dynamic Prompting for AD Perception via Large-Model Optimization | 2026-05 | https://www.sciencedirect.com/science/article/pii/S0968090X26001609 |
| S15 | UniDriveVLA — Unifying Understanding, Perception, and Action Planning | 2026-04 | https://arxiv.org/abs/2604.02190 |
| S16 | Fishsoup0/AD-Perception — Comprehensive Review with 2025-2026 Highlights | 2026-07 | https://github.com/Fishsoup0/Autonomous-Driving-Perception |
| S17 | PatSnap — Sensor Fusion Patents for ADAS Accuracy in 2026 | 2026-04 | https://www.patsnap.com/resources/blog/articles/sensor-fusion-patents-for-adas-accuracy-in-2026/ |
| S18 | Embien — ADAS Sensor Fusion ECU Architecture Design Guide | 2026-06 | https://www.embien.com/automotive-insights/adas-sensor-fusion-ecu-architecture-design |
| S19 | Promwad — Sensor Fusion for Autonomous Transport 2026 | 2026-02 | https://promwad.com/news/sensor-fusion-autonomous-transport-safety-2026 |
| S20 | Dennemeyer — C-V2X April 2026 Report | 2026-04 | https://www.dennemeyer.com/resources/reports/2026/cellular-vehicle-to-everything/ |
| S21 | GMInsights — Automotive V2X Market Size 2026-2035 | 2026-02 | https://www.gminsights.com/industry-analysis/automotive-vehicle-to-everything-market |

### Maritime
| # | Source | Date | URL |
|---|--------|------|-----|
| S22 | EONSR — Autonomous Cargo Ships Navigating with AI in 2026 | 2026-01 | https://eonsr.com/en/autonomous-cargo-ships-navigating-global-waters-with-ai-in-2026/ |
| S23 | Maritime Hub — Rise of Autonomous Vessels 2026 | 2026-02 | https://maritime-hub.com/the-rise-of-autonomous-vessels/ |
| S24 | Yenra — AI Autonomous Ship Navigation: 20 Updated Directions | 2026-03 | https://yenra.com/ai20/autonomous-ship-navigation |
| S25 | ShipUniverse — Autonomous Vessel Tech: Where Are We Really in 2026 | 2026-06 | https://www.shipuniverse.com/tech/autonomous-vessel-tech-where-are-we-really-in-2026/ |
| S26 | ShipServiceHub — MASS Code 2026: AI, Remote Pilotage, Digital Twins | 2026-09 | https://www.shipservicehub.com/blog/mass-code-2026 |
| S27 | ShipUniverse — Port Call Optimization 2026 Guide | 2026-01 | https://www.shipuniverse.com/tech/port-call-optimization-2026-guide |
| S28 | Innovez One — PCO Guide Takes Effect April 2026 | 2026-04 | https://www.innovez-one.com/post/port-call-optimisation-the-pco-guide |
| S29 | Hashmeta AI — Generative AI in Shipping: 2026 Guide | 2026-01 | https://www.hashmeta.ai/en/generative-ai/generative-ai-shipping |

---

## 2. Key External Advances (2026)

### A. Aerospace
- **ATM Digital Twins:** NATS Project Bluebird deploying AI "digital controllers" in full airspace digital twin, shadow-mode comparison vs humans. Live trial planned 2026 in London airspace (S2, S4).
- **MILP Constellation Optimization:** Mixed-Integer Linear Programming frameworks for satellite constellation design — coverage as objective/constraint, provably optimal configs for percent coverage, revisit times, dynamic targets (S7).
- **Heterogeneous Multi-Layer Constellations (HMLC):** Transition from single-layer LEO to multi-layer constellations with different orbital altitudes, cross-phase stitching reducing satellite count ~20.5% while maintaining dual coverage (S5, ScienceDirect LEO study).
- **UAV Swarm Cooperative Path Planning:** Multi-UAV collaborative planning for urban logistics in dense building environments, risk-aware A* with real-time MPC for detection-risk-aware routing (S8, S10).
- **Energy-Aware UAV Planning:** AoI-aware UAV-assisted data collection with multi-agent DRL trajectory optimization (S9).

### B. Autonomous Vehicles
- **Perception-in-Plan (VeteranAD, AAAI 2026):** Integrates perception INTO the planning process rather than sequentially — "perception-in-plan" framework where perception is guided by evolving planning objectives. Achieves 91.1 PDMS score (S12).
- **VLA (Vision-Language-Action) Models:** NVIDIA 500+ TOPS compute with VLA architecture — unifies visual perception, contextual reasoning, and action planning. OpenDriveVLA, OccLLaMA, EMMA (Waymo) represent the frontier (S15, S16, NextMsc).
- **World Models for Driving:** ResWorld (2026), DriveMamba (2026), Infrastructure-Centric World Models — temporal residual world models for E2E driving (S16).
- **Graph-Based Sensor Fusion (BMW EP 2025):** Graph neural networks for sensor data fusion — structural relationship modeling between sensor nodes (S17).
- **Bi-Directional Feedback Fusion (Qualcomm US 2025):** Perception ↔ object tracking bidirectional feedback loop (S17).
- **Modular Multi-Engine Fusion Platforms:** Digital Global Systems' sensor-agnostic platforms with built-in inference + validation stages, enabling continuous improvement without full system requalification (S17).
- **C-V2X Sidelink Architectures:** Moving toward decentralized, sidelink-based V2X for lower latency. Qualcomm/Toyota strong engagement on NR V2X sidelink architecture (S20).
- **Cooperative Perception Going Heterogeneous:** UniV2X (AAAI 2025), STAMP (ICLR 2025), SparseAlign (CVPR 2025) — end-to-end cooperative perception across heterogeneous agents (S16).

### C. Maritime
- **IMO MASS Code 2026:** First unified international regulatory framework for autonomous vessels (Degree 0-4). Remote pilotage operational in Norway, Estonia Baltic routes. Kongsberg K-Bridge autonomous navigation in Degree 2 mode (S26).
- **Maritime LLMs:** UKHO/Marine AI project teaching autonomous vessels to read and act on official navigational data via bespoke maritime LLM — world-first, spring 2026 demo (S22).
- **Diffusion AI for Ship Navigation:** Osaka Metropolitan University using diffusion AI instead of imitation learning — generates whole trajectories based on range of actions experienced humans might take, tested in congested Seto Inland Sea (Ocean News).
- **Digital Twin Architecture for MASS:** IoT sensor layer → satellite comms → shore ROC data processing → AI analysis layer. Latency tiers: monitoring (seconds OK) vs remote control (sub-second required) (S26).
- **PCO Guide V1.0 (April 2026):** First global standard for port call data sharing — mandatory April 1 2026, endorsed by 40+ maritime organizations. Standardized event timestamps enabling JIT arrivals, up to 20% fuel savings (S28).
- **Generative AI in Maritime:** $4.8B maritime AI market (2025), projected $7.2B (2026). 78% of top 100 shipping companies deploying AI. Generative AI for documentation, customs, cargo stowage (S29).
- **Supervised Autonomy Continuum:** Maritime industry framing autonomy as decision support → supervised autonomy → remote operations → bounded automated functions, not "crewless ships" (S25).

---

## 3. Defects Found in NeoTrix Design

### DEFECT-A1: No Cross-Domain Spatial Reasoning Subsystem
**Severity: HIGH** | **Domain:** Aerospace × AV × Maritime  
**Evidence:** All three domains now use digital twins for spatial reasoning — airspace digital twins (S2), driving world models (S16), maritime MASS digital twins (S26). NeoTrix has no unified spatial reasoning substrate.  
**Gap:** L5 Cognition (nt_core + nt_mind) lacks a `spatial_twin` or `world_model` component that all domains could share. The PerceptionBridge only handles sensory→consciousness flow, not persistent spatial state modeling.  
**Suggestion:** Introduce `nt_core::WorldModel` — a persistent spatial state representation with temporal layering, shared across domains. Satellite constellation, vehicle, and ship domains could project their state into this model.

### DEFECT-A2: No Adaptive Fusion Strategy Selection
**Severity: HIGH** | **Domain:** AV (ADAS) × Aerospace  
**Evidence:** 2026 ADAS fusion landscape shows three distinct strategies (early/mid/late fusion) with Elektrobit's dynamic selection based on accuracy-vs-compute tradeoff (S17, S18). BMW's graph-based fusion (S17). Qualcomm's bi-directional feedback (S17).  
**Gap:** NeoTrix PerceptionBridge uses a single flow (SensoryIntegrationHub → SelectiveState). No mechanism to dynamically select fusion depth based on compute budget, latency requirements, or sensor health.  
**Suggestion:** Add `FusionStrategySelector` to PerceptionBridge — a policy-driven module that selects early/mid/deep fusion per task based on `compute_budget`, `latency_slack`, and `sensor_health_score`. Register strategies as constellation nodes.

### DEFECT-A3: No VLA/World-Model Integration Pattern
**Severity: HIGH** | **Domain:** AV  
**Evidence:** 2026 autonomous driving has shifted to VLA models (OpenDriveVLA, UniDriveVLA, EMMA) and world models (ResWorld, DriveMamba) that unify perception-language-action in a single framework (S15, S16).  
**Gap:** NeoTrix separates perception (L2), cognition (L5), and action (L1) into distinct layers with trait-based interfaces. No provision for models that fuse all three in a single forward pass. The SEAL pipeline processes stages sequentially, not as unified inference.  
**Suggestion:** Define `UnifiedInferenceTrait` at L5 that can span L1-L5 in a single call — analogous to how VLA models collapse perception→language→action. Map to a new constellation tier for "foundation model" subsystems.

### DEFECT-A4: No Cooperative Multi-Agent Perception Protocol
**Severity: MEDIUM** | **Domain:** AV (V2X) × Maritime  
**Evidence:** Heterogeneous cooperative perception (UniV2X, STAMP, CoopTrack) enables vehicles to share partial perception across agents (S16). Maritime MASS Degree 2 remote pilotage requires ship↔shore perception sharing (S26). C-V2X sidelink for sub-second latency (S20).  
**Gap:** GWT broadcasts salience across internal specialist modules but has no protocol for broadcasting partial perception state to EXTERNAL agents (other vehicles, ships, shore centers). EventBus is internal only.  
**Suggestion:** Define `ExternalPerceptionBus` — a trait-based interface for sharing partial perception state with external agents, with latency tiers (real-time sidelink vs store-and-forward satellite). Register as NT-IO capability.

### DEFECT-A5: No Regulatory Compliance Knowledge Layer
**Severity: MEDIUM** | **Domain:** Maritime × Aerospace  
**Evidence:** IMO MASS Code 2026 is now mandatory (S26). FAA NextGen/SESAR certification requirements threaten timeline delays beyond 2028 (S1). PCO Guide V1.0 mandatory April 2026 (S28). EASA AI Roadmap for aviation (S1).  
**Gap:** NeoTrix's governance domain (NT-GOVERNANCE) handles constitution compliance and policy enforcement internally. No mechanism to ingest, track, and enforce EXTERNAL regulatory standards (IMO MASS, FAA, EASA, PCO). SelfTest tiers (T1-T3) have no compliance dimension.  
**Suggestion:** Add `RegulatoryCompliance` trait to NT-GOVERNANCE — an external-standard ingestion pipeline that maps regulatory requirements to module-level compliance checks. Feed into SelfTest T3 (production wiring) as a mandatory gate.

### DEFECT-A6: No Maritime-Specific Domain Module
**Severity: MEDIUM** | **Domain:** Maritime  
**Evidence:** Maritime has unique constraints: COLREGs compliance, S-100/S-102 data standards, SOLAS requirements, port call event chains, draft/trim optimization, weather routing with hydrodynamic limits (S22, S24, S26).  
**Gap:** NT-WORLD covers "perception domain: UnifiedCrawler, fetchers, parsers, classifiers" but has no maritime-specific module. Ship navigation involves fundamentally different physics (hydrodynamics, currents, wind) and regulations (COLREGs) than road or air.  
**Suggestion:** Introduce `nt_world_maritime` subdomain — COLREGs-aware routing, S-100 data parsing, port call event chain management. Map as NT-WORLD constellation node.

### DEFECT-A7: No Satellite Constellation Optimization in KB
**Severity: MEDIUM** | **Domain:** Aerospace  
**Evidence:** 2026 shows MILP frameworks for constellation design (S7), heterogeneous multi-layer optimization reducing satellite count 20.5%, unit economics as primary constraint (S5). The problem is fundamentally a KB-graph optimization: satellites as nodes, coverage as edge constraints.  
**Gap:** KB stores nodes/edges/embeddings but has no native optimization solver for constrained graph problems. Constellation design requires mixed-integer programming, not just semantic search.  
**Suggestion:** Add `nt_memory::ConstraintSolver` — a pluggable optimization backend to KB that supports MILP/LP for graph-constrained problems. First use case: satellite constellation coverage optimization.

### DEFECT-A8: No Diffusion/Generative Decision-Making Pattern
**Severity: MEDIUM** | **Domain:** Maritime × AV  
**Evidence:** Osaka Metropolitan University's diffusion AI for ship navigation generates whole trajectories based on range of human actions rather than predicting single best action (Ocean News, S24). DiffusionDrive (CVPR 2025) for autonomous driving (S16).  
**Gap:** SEAL pipeline uses deterministic stage progression. E8 reasoning uses hexagram state transitions. No generative/diffusion pattern for producing diverse action proposals and selecting among them.  
**Suggestion:** Introduce `DiffusionSampler` trait at L5 — generates N candidate action trajectories from a learned distribution, then selects via VoI or other criteria. Complements deterministic reasoning with stochastic exploration.

### DEFECT-A9: No Latency-Tiered Communication Architecture
**Severity: MEDIUM** | **Domain:** Maritime × AV × Aerospace  
**Evidence:** Maritime MASS requires tiered latency: monitoring (seconds OK) vs remote control (sub-second required) (S26). C-V2X sidelink for sub-second vehicle communication (S20). ATM requires trajectory updates within seconds (S4).  
**Gap:** NT-IO handles "LLM providers, CLI, web server, ACP, LSP" — all request-response patterns. No latency-tiered communication substrate that handles sub-second (sidelink), second-scale (satellite), and minute-scale (store-and-forward) differently.  
**Suggestion:** Define `LatencyTieredBus` trait — registers communication channels with explicit latency guarantees. Auto-selects channel based on message urgency and available infrastructure.

### DEFECT-A10: No Predictive Maintenance / Self-Healing for Physical Systems
**Severity: LOW** | **Domain:** All three  
**Evidence:** ATM predictive maintenance for infrastructure (S1). Maritime predictive maintenance via Wartsila Expert Insight (S24). ADAS calibration tracking for multi-sensor systems (S18).  
**Gap:** NT-REPAIR handles software self-healing. NT-PHYSICAL has sensors/motors/safety but no predictive maintenance pipeline. HeartbeatAggregator collects health signals but doesn't project degradation timelines.  
**Suggestion:** Extend HeartbeatAggregator with `DegradationProjector` — uses historical health signal trends to predict time-to-failure for physical components, triggering proactive maintenance scheduling.

---

## 4. Summary Table

| ID | Defect | Severity | Domains | NeoTrix Component Affected |
|----|--------|----------|---------|---------------------------|
| A1 | No cross-domain spatial reasoning | HIGH | All | L2 PerceptionBridge, L5 Cognition |
| A2 | No adaptive fusion strategy | HIGH | AV, Aerospace | L2 PerceptionBridge |
| A3 | No VLA/world-model integration | HIGH | AV | L1-L5 layer separation |
| A4 | No cooperative multi-agent perception | MEDIUM | AV, Maritime | GWT, NT-IO |
| A5 | No regulatory compliance layer | MEDIUM | Maritime, Aerospace | NT-GOVERNANCE, SelfTest |
| A6 | No maritime domain module | MEDIUM | Maritime | NT-WORLD |
| A7 | No constellation optimization in KB | MEDIUM | Aerospace | NT-MEMORY |
| A8 | No diffusion/generative decisions | MEDIUM | Maritime, AV | SEAL, E8 |
| A9 | No latency-tiered communication | MEDIUM | All | NT-IO |
| A10 | No predictive maintenance for physical | LOW | All | NT-REPAIR, NT-PHYSICAL |

---

## 5. Recommendations (Priority Order)

1. **DEFECT-A1 (WorldModel)** and **DEFECT-A2 (AdaptiveFusion)** are the highest-impact — they represent foundational substrate missing that all three external domains rely on. Implement as L5 Cognition extension with L2 PerceptionBridge integration.

2. **DEFECT-A3 (VLA/UnifiedInference)** represents the 2026 paradigm shift in autonomous systems. The layer separation model needs an "escape hatch" for foundation-model-class subsystems that fuse L1-L5 in a single pass.

3. **DEFECT-A9 (LatencyTieredBus)** is the communication backbone needed by A4 (cooperative perception) and by any real-time physical system deployment.

4. **DEFECT-A5 (RegulatoryCompliance)** should be added to NT-GOVERNANCE before any production deployment in regulated domains.

5. **DEFECT-A8 (DiffusionSampler)** and **DEFECT-A7 (ConstraintSolver)** represent capability gaps that can be added as pluggable backends without architectural restructuring.

---

*Iteration 329 complete. Next: focus on A1+A2+A3 substrate design, prototype AdaptiveFusion with existing PerceptionBridge.*

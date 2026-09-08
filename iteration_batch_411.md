# Iteration Batch 411 — Autonomous Driving Perception, Planning & Safety Architecture

**Date**: 2026-09-06
**Domain**: NT-WORLD (L2 Perception) + NT-ACT (L1 Action) + NT-CORE (L5 Cognition) + NT-PHYSICAL (L3 Embodiment)

---

## Sources Cited

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| S1 | Waymo: "10 AI Lessons from Driving 200+ Million Fully Autonomous Miles" | theverge.com/transportation/985503/waymo-tesla-lidar-camera-robotaxi | 2026-08-27 | Cameras alone are insufficient for L4; multi-sensor (camera+LiDAR+radar) redundancy is required; HD maps as "prior memory"; 500K driverless rides/week |
| S2 | Waymo 6th-Gen Driver Blog | waymo.com/blog/2026/02/ro-on-6th-gen-waymo-driver | 2026-02-12 | 13 cameras (17MP) + 4 LiDAR + 6 imaging radar + EARs; ~42% fewer sensors than 5th-gen; fully driverless since Feb 2026; modular perception+fusion+planner |
| S3 | Tesla FSD v14.3.2 / MotiveGrid Analysis | motivegrid.com/insights/tesla-fsd-vs-waymo | 2026-05-23 | End-to-end multimodal MoE architecture; MLIR runtime ~20% faster reactions; camera-only $1-2k BOM; Waymo sub-$20k; convergence risk: cheap LiDAR ($500 class) may close BOM gap |
| S4 | Waymo Expands to 14 Cities (Denver/San Diego/Tampa) | electrek.co/2026/09/01/waymo-public-robotaxi | 2026-09-01 | 14 US cities; Zeekr-built vehicles; targeting 1M weekly rides by end of year; 94% fewer serious-injury crashes vs humans |
| S5 | Tesla Cybercab Launch | theverge.com/transportation/987901/tesla-cybercab-launch | 2026-09-02 | Camera-only Cybercab entering operation; no steering wheel/pedals; remote teleoperation as failsafe; Starlink connectivity for backup comms |
| S6 | Spe-BEVHead (CVPR 2026) | openaccess.thecvf.com/content/CVPR2026/papers/Zhang_Spe-BEVHead | 2026 | BEV-specific detection head: Rotated Box Kernel (replaces Gaussian), Local Response Refinement Module (NMS-free), dual-branch supervision; +1.7% NDS on FastBEV |
| S7 | Dr.Occ (CVPR 2026) | openaccess.thecvf.com/content/CVPR2026/papers/Zhu_Dr.Occ | 2026 | Depth-guided 2D-to-3D view transformer using MoGe-2 priors; Region-guided Expert Transformer (R²-EFormer) for spatial semantic imbalance; +7.43% mIoU on BEVDet4D |
| S8 | Collaborative Perceiver / CoP (arXiv:2507.21358v2) | arxiv.org/html/2507.21358v2 | 2026 | Multi-task learning: 3D detection + occupancy prediction; local density-aware occupancy (LDO); voxel-height-guided sampling; 49.5% mAP, 59.2% NDS on nuScenes |
| S9 | TT-Occ: Test-Time 3D Occupancy (CVPR 2026) | openaccess.thecvf.com/content/CVPR2026/papers/Zhang_Test-Time | 2026 | Zero-training occupancy via VFMs (VGGT, REX-Omni); time-aware 3D Gaussians; arbitrary voxel resolution; open-vocabulary recognition at test time |
| S10 | BePo: Dual Representation (CVPR 2026 Workshop) | openaccess.thecvf.com/content/CVPR2026W/WAD/papers/Shi_BePo | 2026 | BEV + sparse 3D points dual representation; cross-attention between branches; SOTA on Occ3D-nuScenes (37.27 mIoU); captures small objects missed by BEV-only |
| S11 | OccAny: Generalized Urban 3D Occupancy (CVPR 2026) | openaccess.thecvf.com/content/CVPR2026/html/Cao_OccAny | 2026 | First unconstrained urban 3D occupancy model; works on out-of-domain uncalibrated scenes; novel view rendering for test-time augmentation |
| S12 | GuideFlow: Constraint-Guided Flow Matching (CVPR 2026) | openaccess.thecvf.com/content/CVPR2026/papers/Liu_GuideFlow | 2026 | Flow matching for E2E planning; explicit safety constraints embedded in generation; EBM for autonomous constraint satisfaction; SOTA 43.0 EPDMS on NavSim hard |
| S13 | NDPNet: Near-to-Distant Trajectory Prediction (CVPR 2026) | openaccess.thecvf.com/content/CVPR2026/papers/Hu_Perceiving_the_Near | 2026 | Dual-stage: near-future (dynamics-driven) + distant-future (semantics-driven); motion-aware coherence loss; first sub-1.75 minFDE6 at 8s on WOMD |
| S14 | CogDrive: Cognition-Driven Prediction-Planning Fusion | sciopen.com/article/10.26599/COMMTR.2026.9640016 | 2026-06-30 | Topological motion semantics for sparse behaviors; trajectory tree planner for multimodal uncertainty; SOTA on Argoverse 2 + INTERACTION |
| S15 | CDJMP: Adaptive Diffusion for Joint Motion Prediction | link.springer.com/article/10.1007/s43684-026-00138-z | 2026-09-01 | Two-stage diffusion: proposal + adaptive denoising steps; group-aware conditional encoder; 9-10% minFDE/minADE reduction on INTERACTION |
| S16 | GuideFlow / Flow Matching for E2E Planning | openaccess.thecvf.com (same as S12) | 2026 | Constrain-Velocity-Field + Constrain-Flow-States + Refine-by-EBM; driving aggressiveness as control signal |
| S17 | Ranging from Prediction to Planning: Survey (Springer) | link.springer.com/article/10.1007/s10462-026-11604-8 | 2026-08-06 | Comprehensive survey: interactive prediction+planning; LLMs, world models, distillation, RLHF for AD; long-tail learning challenges |
| S18 | Active Interaction-Aware MPPI (arXiv:2608.21400) | arxiv.org/abs/2608.21400 | 2026-08-06 | Ego-conditioned generative prediction within MPPI control; "what-if" reasoning; nested sampling for tractable cost evaluation |
| S19 | GPOcc: Visual Geometry Priors for Sparse Gaussian Occupancy (CVPR 2026) | openaccess.thecvf.com/content/CVPR2026/papers/Zhou_GPOcc | 2026 | Ray-based volumetric sampling from surface priors; opacity-based pruning; +9.99 mIoU over prior SOTA; training-free incremental update for streaming |
| S20 | Learning-Based Behavior Planning: Real-World Deployment (arXiv:2608.12198) | arxiv.org/html/2608.12198 | 2026 | Hybrid architecture: DNN behavior planner + optimization-based trajectory supervision; MLOps workflow for continuous refinement; open-source in OpenADS |

---

## Defects Found

### DEFECT-411.1: No 3D Occupancy Prediction Pipeline (CRITICAL)

**Sources**: S6, S7, S8, S9, S10, S11, S19
**NeoTrix gap**: NT-WORLD's `perception_bridge.rs` connects sensory input to consciousness via `awareness_score()`, but has no volumetric scene representation. The 2026 perception frontier has converged on **3D semantic occupancy** as the core scene representation for autonomous systems:
- Spe-BEVHead (S6) proves BEV detection heads need BEV-specific redesign, not 2D head transplantation
- Dr.Occ (S7) achieves +7.43% mIoU via depth-guided geometric alignment + region-specific experts for spatial semantic imbalance
- BePo (S10) demonstrates BEV alone fails for small objects; dual BEV+points representation is necessary
- TT-Occ (S9) achieves test-time occupancy without any training via Vision Foundation Models
- OccAny (S11) enables unconstrained urban occupancy from uncalibrated scenes
- GPOcc (S19) shows ray-based volumetric sampling from surface priors outperforms dense volume methods

**Impact**: NeoTrix's `nt_world_sense` perceives in 2D image space. The VSA HyperCube has no volumetric geometric grounding. The PerceptionBridge (L2→L5) transmits flat sensory events, not 3D spatial occupancy. This means:
- No free-space vs occupied classification for navigation
- No dense geometric understanding for collision avoidance
- No open-vocabulary object recognition in 3D (TT-Occ achieves this at test time)
- GWT attention routing has no spatial occupancy signal to weight

### DEFECT-411.2: No Sensor Fusion Redundancy Architecture

**Sources**: S1, S2, S3, S4, S5
**NeoTrix gap**: The Waymo vs Tesla debate crystallizes a fundamental architecture question. Waymo's 6th-gen Driver uses 13 cameras + 4 LiDAR + 6 imaging radar + EARs as a **unified redundant system** where removing any single sensor measurably degrades performance (S1). Tesla bets camera-only suffices. NeoTrix has neither approach:
- `nt_world_sense/visual_cortex.rs` processes camera images only
- No LiDAR point cloud processing path
- No radar signal processing
- No audio event localization (Waymo uses EARs for emergency siren detection)
- No sensor redundancy voting or graceful degradation mechanism

**Impact**: NT-WORLD cannot operate in degraded sensing conditions (glare, fog, rain, occlusion). The system has no redundancy — a single sensor failure (camera blinded) causes total perception loss. Waymo's explicit lesson: "safe, fully autonomous operations at scale require more" than cameras. NeoTrix's L2 perception layer has no multi-modal fusion, no sensor health monitoring, and no graceful degradation path.

### DEFECT-411.3: No BEV Feature Representation Layer

**Sources**: S6, S7, S8, S10
**NeoTrix gap**: Bird's-Eye-View has become the **dominant intermediate representation** for autonomous perception in 2026. Every paper above uses BEV as the canonical feature space. NeoTrix's perception pipeline operates in perspective camera space:
- No Lift-Splat-Shoot (LSS) or transformer-based view transformation
- No BEV feature grid construction
- No temporal BEV fusion (BEVDet4D-style 4D features)
- The `perception_bridge.rs` passes raw sensory events, not BEV-structured features

**Impact**: Without BEV representation, NeoTrix cannot:
- Achieve view-invariant spatial reasoning (BEV unifies multi-camera into single coordinate frame)
- Perform 3D detection with proper depth reasoning
- Support downstream occupancy prediction (all occupancy methods start from BEV features)
- Leverage the massive BEV research ecosystem (BEVDepth, BEVFormer, BEVStereo, etc.)

### DEFECT-411.4: No Trajectory Prediction Module

**Sources**: S13, S14, S15, S17, S18
**NeoTrix gap**: NT-ACT's `production_orchestrator.rs` manages task scheduling but has zero trajectory prediction capability. The 2026 frontier shows trajectory prediction is the critical bridge between perception and planning:
- NDPNet (S13) decouples near-future (dynamics) from distant-future (semantics) modeling, achieving first sub-1.75 minFDE6 at 8s
- CogDrive (S14) introduces topological motion semantics for sparse safety-critical behaviors
- CDJMP (S15) uses adaptive diffusion steps for diverse multi-agent joint prediction
- The survey (S17) identifies interactive prediction+planning as the paradigm shift

**Impact**: NeoTrix cannot predict future states of dynamic agents. The GWT attention system has no predictive signal — it can only react to current observations, not anticipate future states. The SEAL pipeline cannot evolve planning strategies because there is no prediction module to plan against. The system is perpetually reactive.

### DEFECT-411.5: No Behavior Planning / Motion Planning Module

**Sources**: S12, S14, S17, S18, S20
**NeoTrix gap**: NeoTrix has no motion planning capability. The 2026 landscape shows planning has evolved to:
- **Hybrid architectures** (S20): DNN behavior planner + optimization-based trajectory supervision with safety fallback
- **Constraint-guided generation** (S12): GuideFlow embeds hard safety constraints directly in flow matching generation, achieving SOTA 43.0 EPDMS
- **Interactive reasoning** (S18): Ego-conditioned generative prediction within MPPI for "what-if" reasoning
- **Cognition-driven fusion** (S14): Topological motion semantics + trajectory tree planning

**Impact**: NT-ACT has action execution (`nt_act`) but no trajectory planning, no behavior prediction, no safety-constrained motion generation. The system can execute predefined actions but cannot generate safe trajectories in dynamic environments. The "行动执行者" (Action Executor) can act but cannot plan.

### DEFECT-411.6: No Uncertainty-Aware or Multi-Modal Prediction

**Sources**: S13, S14, S15, S16
**NeoTrix gap**: Modern prediction must handle **multimodal futures** — a car at an intersection could go straight, turn left, or turn right. NeoTrix's latent-space JEPA prediction is single-modal:
- NDPNet (S13) explicitly models the difference between dynamics-driven near-term and semantics-driven long-term uncertainty
- CogDrive (S14) uses trajectory trees for multi-branch planning under uncertainty
- CDJMP (S15) generates diverse trajectories via adaptive diffusion steps
- GuideFlow (S16) mitigates mode collapse via flow matching with diverse conditioning

**Impact**: NeoTrix's JEPA world model predicts a single latent future trajectory. It cannot represent the branching, stochastic nature of real-world dynamics. This means:
- GWT attention cannot weight multiple hypotheses
- The SEAL pipeline cannot evaluate branching strategies
- No risk-aware decision making (all predictions treated as deterministic)

### DEFECT-411.7: No Test-Time Adaptation / Open-Vocabulary Perception

**Sources**: S9, S11, S17
**NeoTrix gap**: TT-Occ (S9) achieves zero-training occupancy using Vision Foundation Models at test time. OccAny (S11) handles unconstrained out-of-domain scenes. NeoTrix's perception requires pre-trained models with fixed vocabularies:
- No VFM integration (VGGT, REX-Omni, DepthAnything)
- No open-vocabulary recognition (cannot recognize novel object categories)
- No test-time geometric completion
- No streaming adaptation without retraining

**Impact**: NeoTrix cannot handle novel objects, unseen environments, or domain shifts without retraining. The SEAL evolution loop requires full retraining cycles rather than online adaptation. Real-world deployment demands handling the unexpected — NeoTrix's perception is brittle to distribution shift.

### DEFECT-411.8: No Sensor-Conditioned Planning (Perception-Planning Bridge Missing)

**Sources**: S12, S17, S20
**NeoTrix gap**: The 2026 survey (S17) identifies the key paradigm: **interactive prediction-planning** where perception informs planning and planning shapes perception attention. NeoTrix's architecture has a hard disconnect:
- NT-WORLD (L2) → perception events → GWT → NT-ACT (L1) → actions
- No closed-loop perception-planning cycle
- No planning-conditioned perception (e.g., "look for pedestrians because I plan to turn")
- No perception-conditioned planning (e.g., "I see a pedestrian, plan to yield")

**Impact**: The PerceptionBridge sends sensory events upward but receives no top-down attention guidance from planning. GWT broadcasts salience but has no planning context to modulate what is salient. The system operates open-loop: perceive → broadcast → act, without the closed-loop feedback that characterizes safe autonomous systems.

---

## Suggestions

### SUGGESTION-411.1: Implement `nt_world_occ` — 3D Occupancy Prediction Module

**Priority**: P0 (CRITICAL)
**Effort**: Large (new module)
**Implementation**: Create `neotrix-core/src/l2_perception/nt_world_occ/` implementing:
- BEV feature construction via LSS or transformer-based view transformation
- 3D semantic occupancy prediction (start with BePo dual-representation for efficiency)
- Integration with existing `perception_bridge.rs` to provide volumetric context to GWT
- Open-vocabulary recognition path using TT-Occ's VFM approach for zero-shot capability

### SUGGESTION-411.2: Add Multi-Modal Sensor Fusion to `nt_world_sense`

**Priority**: P0 (CRITICAL)
**Effort**: Medium
**Implementation**: Extend `visual_cortex.rs` to:
- Accept LiDAR point clouds, radar returns, and audio events as additional input streams
- Implement sensor health monitoring and graceful degradation (voting, confidence weighting)
- Add temporal fusion across sensor modalities
- Reference Waymo's lesson: redundancy is not optional for safety-critical systems

### SUGGESTION-411.3: Create `nt_act_trajectory` — Prediction + Planning Module

**Priority**: P0 (CRITICAL)
**Effort**: Large (new module)
**Implementation**: Create `neotrix-core/src/l1_action/nt_act/trajectory/` implementing:
- Multi-modal trajectory prediction (NDPNet-style near/distant decoupling)
- Constraint-guided trajectory generation (GuideFlow-style safety constraints in generation)
- Behavior planning with optimization-based safety supervision (S20 hybrid architecture)
- Integration with GWT for planning-conditioned attention

### SUGGESTION-411.4: Implement Closed-Loop Perception-Planning Feedback

**Priority**: P1 (HIGH)
**Effort**: Medium
**Implementation**: Modify `perception_bridge.rs` to:
- Accept top-down attention modulation from planning module
- Enable planning-conditioned perception ("I plan to turn → attend to pedestrians on right")
- Create bidirectional information flow between L2 perception and L1 action
- Align with GWT's resonance-based routing for planning-aware attention

### SUGGESTION-411.5: Add VFM Integration Layer for Test-Time Adaptation

**Priority**: P1 (HIGH)
**Effort**: Medium
**Implementation**: Create a VFM adapter in `nt_world_sense` that:
- Integrates geometry foundation models (VGGT, DepthAnything) for test-time depth
- Supports open-vocabulary semantic recognition (REX-Omni, GroundingSAM)
- Enables zero-shot handling of novel objects without retraining
- Feeds into occupancy prediction for online scene understanding

### SUGGESTION-411.6: Extend JEPA World Model with Multi-Modal Futures

**Priority**: P1 (HIGH)
**Effort**: Medium
**Implementation**: Modify the JEPA prediction in `nt_mind` to:
- Output multi-modal latent trajectory distributions (not single trajectory)
- Support branching temporal predictions (trajectory tree)
- Enable risk-weighted evaluation of future branches
- Integrate with GWT for uncertainty-aware attention allocation

---

## Summary

| Metric | Value |
|--------|-------|
| Sources cited | 20 |
| Defects found | 8 (3 CRITICAL, 3 HIGH, 2 MEDIUM) |
| Suggestions | 6 |
| Domain coverage | NT-WORLD, NT-ACT, NT-CORE, NT-PHYSICAL |
| Key 2026 paradigm shifts identified | 3D occupancy as core representation; sensor fusion redundancy; constraint-guided generative planning; test-time adaptation via VFMs; interactive prediction-planning loop |

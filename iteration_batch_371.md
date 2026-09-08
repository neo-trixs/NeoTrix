# Iteration Batch 371 — Spatial Perception & SLAM Research

**Date:** 2026-09-06
**Research Domains:** Visual SLAM, LiDAR SLAM, Neural SLAM, Sensor Fusion, IMU Fusion, 3D Mapping, Semantic Mapping

---

## Sources Cited

### Visual SLAM
1. Zhong et al., "Dynamic Visual SLAM using a General 3D Prior" (CVPR 2026) — feed-forward reconstruction + patch-based BA for dynamic scenes
2. VIGS-SLAM (ECCV 2026) — visual-inertial 3D Gaussian Splatting SLAM, tightly coupled IMU+visual optimization
3. VGGT-SLAM++ (CVPR 2026W) — transformer-based SLAM with DEM + DINOv2 embeddings, bounded memory
4. UniSim-SLAM (ECCV 2026) — unified Sim(3) factor graph for multi-level optimization, 38.5–45.9% error reduction
5. SCE-SLAM (CVPR 2026) — scale-consistent monocular SLAM via scene coordinate embeddings, 36 FPS, 8.36m ATE improvement on KITTI
6. DROID-W (CVPR 2026) — dynamic-aware SLAM with per-pixel uncertainty via differentiable BA, 10 FPS
7. SLAM-MER (CVPR 2026) — 80+ FPS real-time monocular SLAM with spatio-temporal modeling, modular C++ framework

### LiDAR SLAM
8. Tak et al., "Real-Time LiDAR Gaussian Splatting SLAM via Geometry-Aware Covariance Coupling" (ECCV 2026) — covariance reuse between tracking/mapping, 20+ FPS, F-score 86.78%
9. Wang et al., "SFC-SLAM: Spatial Feature Change-aware LiDAR SLAM" (Meas. Sci. Technol. 2026) — SFC probability model, adaptive IEKF, 17 Hz
10. HP²-SLAM (arXiv 2608.14996, Aug 2026) — adaptive hybrid ICP, planarity-aware threshold, real-time on commodity hardware
11. ICF-SLAM (Meas. Sci. Technol. 2026) — instance-guided loop closure + asynchronous historical correction, F1 0.998
12. LXD-SLAM (arXiv 2606.27811, Jun 2026) — 32 configurable sensor combinations, unified IESKF, GP sub-meshes

### Neural SLAM
13. MSN-SLAM (arXiv 2608.09146, Aug 2026) — multi-submap neural SLAM with foundation model descriptors, city-scale (500m×400m)
14. Unblur-SLAM (CVPR 2026) — handles motion + defocus blur in SLAM, adaptive computation based on blur level
15. RoSe-SLAM (IROS 2026) — semantic-aware Gaussian Splatting SLAM from dynamic monocular videos, 2D foundation model features
16. CGS-SLAM (arXiv 2608.26868, Aug 2026) — collaborative multi-agent 3DGS SLAM using RGB+IMU only
17. FeatureSLAM (arXiv 2601.05738, Jan 2026) — feature-enriched 3DGS SLAM with open-set segmentation, real-time at 5 FPS

### Sensor Fusion
18. CRUISE (arXiv 2608.09202, Aug 2026) — VLM-guided uncertainty-aware cross-modal fusion, +4.87% 3D detection
19. FusionBridge (SenSys 2026) — lightweight cross-modal edge fusion with calibration-free transformer adapter
20. DAP-Pose (arXiv 2607.23755, Jul 2026) — bi-level cross-modal fusion + deep temporal alignment for Visual-Inertial-GNSS
21. ADM-Fusion (arXiv 2606.25111, Jun 2026) — adaptive mixture-of-experts for sensor routing, sim-to-real transfer
22. NeuroFusion-SLAM (Sensors 2026) — depthwise separable conv, 30.4ms/frame, 40% fewer parameters
23. FusionCore (arXiv 2605.25239, May 2026) — 23-state UKF fusing IMU/wheel/GPS/VSLAM at 100 Hz

### IMU Fusion
24. MARIO (CVPR 2026F) — pose-prior grounded inertial odometry, multi-sensor fusion (barometer+magnetometer+dual IMU), 42% drift reduction
25. KISS-IMU (arXiv 2603.06205, Mar 2026) — self-supervised IO via LiDAR ICP pseudo-labels, no ground truth needed
26. IMU-GNSS (Sensors 2026) — PINN+EKF for seamless indoor/outdoor navigation, 59% RMSE reduction
27. RISAF (arXiv 2606.29271, Jun 2026) — robust EKF for massive MEMS IMU arrays with dynamic percentile gating
28. IMU Parameter Setting (arXiv 2608.21433, Aug 2026) — Allan variance-based IMU parameter tuning in Kalman filtering

### 3D Mapping / Reconstruction
29. SLARM (CVPR 2026) — streaming language-aligned 4D Gaussian reconstruction, +21% motion accuracy, +1.6dB PSNR
30. ZipMap (CVPR 2026) — linear-time 3D reconstruction via TTT layers, 700+ frames in <10s, 20× faster than VGGT
31. OVGGT (ECCV 2026) — O(1) constant-cost streaming, Self-Selective Caching + Dynamic Anchor Protection
32. Online3R (CVPR 2026) — online learning with local-global self-supervised consistency for sequential reconstruction
33. MoRGS (CVPR 2026) — per-Gaussian motion reasoning for streamable dynamic scenes
34. RayMap3R (arXiv 2603.20588, Mar 2026) — training-free dynamic-aware streaming via RayMap static bias

### Semantic Mapping
35. OVI-MAP (CVPR 2026) — open-vocabulary instance-semantic mapping, decoupled instance reconstruction from semantic inference
36. SuperMap (RSS 2026) — 4D spatio-temporal mapping for VLN, existence-and-label confidence updates
37. M2-SMap (arXiv 2608.07074, Aug 2026) — memory-efficient hierarchical multi-model semantic mapping, 29–98 Hz
38. GLMap (CVPR 2026) — multi-scale Gaussian-Language map with dual-modality interface for zero-shot navigation
39. HSGM (CVPR 2026) — hierarchical semantic-geometric map bridging VLMs with 3D world for VLN

---

## Defects Found in NeoTrix Design

### DEFECT-371.1: No Spatial Memory / SLAM Module in NT-PHYSICAL

**Severity:** High (architectural gap)
**Context:** NT-PHYSICAL (L3 embodiment) is defined as "sensors, motors, safety kernel, power management, body schema" but contains no spatial memory, SLAM, or persistent 3D map representation.

**Evidence from research:**
- VIGS-SLAM (ECCV 2026) demonstrates that visual-inertial Gaussian Splatting SLAM achieves robust real-time tracking with high-fidelity reconstruction by tightly coupling IMU pre-integration with visual features. NeoTrix has no equivalent pipeline for maintaining spatial awareness.
- LXD-SLAM (Jun 2026) shows 32 configurable sensor combinations with unified IESKF — demonstrating that modular sensor fusion with plug-and-play architecture is now standard practice.
- FusionCore (May 2026) provides a 23-state UKF fusing IMU/wheel/GPS/VSLAM at 100 Hz with online bias estimation — a mature reference architecture that NeoTrix's NT-PHYSICAL lacks.

**Impact:** An embodied NeoTrix agent cannot build, maintain, or query a persistent spatial map. The "body schema" is static, not spatially grounded.

**Suggestion:** Add `nt_physical::spatial_memory` module implementing a sparse keyframe-based 3D map with Gaussian Splatting or point cloud representation. Use a lightweight Sim(3) factor graph (inspired by UniSim-SLAM) for long-horizon consistency. Define a `SpatialMemory` trait with: `insert_keyframe()`, `query_nearby()`, `loop_detect()`, `get_trajectory()`.

---

### DEFECT-371.2: No Dynamic Scene Handling in PerceptionBridge

**Severity:** High (operational failure mode)
**Context:** `PerceptionBridge` (L2→L5 attention-gated bridge) connects SensoryIntegrationHub with SelectiveState via `awareness_score()` but has no mechanism to detect or suppress dynamic objects that corrupt perceptual input.

**Evidence from research:**
- π³Mos (CVPR 2026) shows feed-forward reconstruction models can simultaneously predict depth AND segment moving objects, providing clean static-scene input to SLAM — a paradigm shift from "detect then track" to "reconstruct, dynamics emerge as by-products."
- DROID-W (CVPR 2026) introduces per-pixel uncertainty via multi-view feature similarity in differentiable BA, achieving robust tracking at 10 FPS in cluttered dynamic scenes without predefined dynamic priors.
- RoSe-SLAM (IROS 2026) uses 2D foundation model semantic features to identify dynamic distractors, with spatial-temporal motion mask generation.

**Impact:** NT-WORLD's crawlers/perception pipeline assumes static input. When deployed in embodied scenarios (robotics, AR), dynamic objects will corrupt spatial reasoning and attention routing.

**Suggestion:** Extend `PerceptionBridge` with a `DynamicFilter` stage that computes per-pixel/mesh uncertainty from multi-view consistency (using DINOv2 features as in DROID-W). Add a `staticness_score` to each sensory event before it reaches `SelectiveState`. This is not about removing dynamics — it's about labeling them so GWT attention routing can reason about them explicitly.

---

### DEFECT-371.3: No Online / Streaming 3D Reconstruction Capability

**Severity:** Medium-High
**Context:** NT-WORLD handles "UnifiedCrawler, fetchers, parsers, classifiers, content extraction" — all web/data extraction. There is no streaming 3D reconstruction capability for processing sequential visual data.

**Evidence from research:**
- ZipMap (CVPR 2026) achieves linear-time 3D reconstruction (700+ frames in <10s) using TTT layers that compress scene into queryable hidden state — 20× faster than quadratic-time methods. This is now the efficiency frontier.
- OVGGT (ECCV 2026) achieves O(1) constant-cost streaming via Self-Selective Caching + Dynamic Anchor Protection — meaning arbitrarily long video can be processed with fixed VRAM.
- Online3R (CVPR 2026) introduces online learning with local-global self-supervised consistency for sequential reconstruction from a frozen geometry foundation model.
- SLARM (CVPR 2026) unifies dynamic scene reconstruction, semantic understanding, and streaming inference in a single feed-forward model.

**Impact:** NeoTrix cannot process sequential visual input (camera streams, video) into persistent 3D representations. This blocks embodied agent scenarios and limits NT-WORLD to text/web data.

**Suggestion:** Add `nt_world::streaming_reconstruction` module implementing a TTT-based (Test-Time Training) feed-forward 3D reconstruction pipeline. Key interfaces: `FeedForwardReconstructor` trait with `process_frame(image, pose_hint) -> (depth, gaussian_map, scene_state)`. Support both bidirectional (batch) and streaming (incremental) modes. Use OVGGT's O(1) caching strategy for bounded memory.

---

### DEFECT-371.4: No Uncertainty-Aware Sensor Fusion Architecture

**Severity:** Medium-High
**Context:** NT-PHYSICAL defines "sensors" and "power management" but has no unified uncertainty modeling for cross-modal sensor fusion. The architecture lacks a principled way to handle sensor degradation or conflicting measurements.

**Evidence from research:**
- CRUISE (Aug 2026) demonstrates VLM-guided fine-grained pixel-level uncertainty estimation outperforms modality-level UQ by 4.87% in 3D detection — showing that uncertainty must be spatially resolved, not just modality-level.
- ADM-Fusion (Jun 2026) uses adaptive mixture-of-experts with content-aware routing to dynamically re-weight sensor contributions in real-time — demonstrating that fixed fusion weights are obsolete.
- DAP-Pose (Jul 2026) introduces deep temporal alignment in latent space to handle asynchronous sensors without hardware synchronization — solving a practical deployment problem.
- FusionBridge (SenSys 2026) achieves calibration-free cross-modal fusion with 15% latency overhead and 0.4KB/frame — proving lightweight fusion is feasible on edge devices.

**Impact:** NeoTrix's multi-sensor integration (when extended to embodied agents) will fail under sensor degradation, adversarial conditions, or asynchronous sampling — the three most common real-world failure modes.

**Suggestion:** Define `SensorFusionLayer` trait in NT-PHYSICAL with: (1) per-pixel uncertainty estimation from multi-view consistency, (2) adaptive MoE-style sensor routing (as in ADM-Fusion), (3) latent temporal alignment for asynchronous streams (as in DAP-Pose). Model sensor trust as a dynamic variable, not a static config.

---

### DEFECT-371.5: No Semantic Spatial Grounding for Language Models

**Severity:** Medium
**Context:** NT-IO handles "LLM providers, CLI, web server" but there is no mechanism to ground LLM/VLM reasoning in 3D spatial representations. The semantic-geometric gap identified in HSGM (CVPR 2026) directly applies.

**Evidence from research:**
- HSGM (CVPR 2026) explicitly identifies the "Semantic-Geometric Gap" — VLMs excel at 2D visual understanding but are geometrically naive, unable to bridge language reasoning with 3D spatial layout.
- GLMap (CVPR 2026) proposes dual-modality interface where each semantic unit jointly stores natural language description + 3D Gaussian representation — the first native bridge between VLMs and 3D maps.
- SuperMap (RSS 2026) demonstrates 4D spatio-temporal mapping with existence-and-label confidence updates for long-term stability in dynamic environments.
- OVI-MAP (CVPR 2026) decouples instance reconstruction from semantic inference, enabling zero-shot semantic labeling from a small set of selected views.

**Impact:** NeoTrix's LLM integration (NT-IO) cannot reason about spatial layouts, cannot ground language instructions in 3D environments, and cannot support Vision-Language Navigation — a critical capability gap for embodied agents.

**Suggestion:** Add `nt_io::spatial_grounding` module implementing a GLMap-style dual-modality interface. Each spatial entity stores: (1) 3D Gaussian for rendering/querying, (2) language-aligned semantic embedding for VLM/LLM reasoning. Define `SpatialGrounding` trait: `query_by_text(description) -> spatial_entities`, `render_view(pose) -> image`, `update_semantics(new_observations)`.

---

### DEFECT-371.6: No Loop Closure or Global Consistency Mechanism

**Severity:** Medium
**Context:** No component in the architecture addresses accumulated drift or provides global consistency across temporal/spatial windows. The SEAL pipeline operates on knowledge evolution, not spatial consistency.

**Evidence from research:**
- ICF-SLAM (2026) achieves F1 0.998 for loop detection via instance-guided loop closure, reducing ATE by 38.6% vs FAST-LIO2 — demonstrating that loop closure is essential for long-horizon spatial consistency.
- MSN-SLAM (Aug 2026) uses foundation model descriptors (SALAD) for hierarchical local-to-global loop closure, handling city-scale scenes up to 500m×400m.
- UniSim-SLAM (ECCV 2026) formulates multi-level Sim(3) factor graph with submap-to-submap tie constraints — the state of the art for long-horizon scale consistency.
- SCE-SLAM (CVPR 2026) achieves scale consistency through learned scene coordinate embeddings that propagate scale information across temporal windows.

**Impact:** Any spatial memory in NT-PHYSICAL will accumulate drift over time without loop closure. The system cannot maintain globally consistent maps across sessions or long trajectories.

**Suggestion:** Add `SpatialConsistency` trait with: `detect_loop(keyframe) -> Option<LoopConstraint>`, `optimize_graph(constraints) -> ConsistentTrajectory`, `get_scale_consistency() -> ScaleFactor`. Use a lightweight pose graph optimization (GTSAM-style) with submap management. Foundation model descriptors (DINOv2/SALAD) for place recognition.

---

### DEFECT-371.7: No Collaborative / Multi-Agent Spatial Awareness

**Severity:** Medium-Low (future capability)
**Context:** The architecture assumes a single agent. No provision for collaborative mapping or multi-agent spatial awareness.

**Evidence from research:**
- CGS-SLAM (Aug 2026) demonstrates collaborative multi-agent 3DGS SLAM using only RGB+IMU, with decentralized tracking + centralized submap alignment — proving multi-agent spatial mapping is practical.
- FusionBridge (SenSys 2026) enables calibration-free cross-node fusion with minimal overhead — the foundation for distributed spatial perception.

**Impact:** NeoTrix cannot support multi-agent scenarios (robot swarms, distributed sensors, collaborative AR) where spatial maps must be shared, merged, and kept consistent across agents.

**Suggestion:** Define `CollaborativeMap` trait in NT-PHYSICAL with: `share_keyframes(agent_id, keyframes)`, `merge_submap(remote_submap) -> MergedResult`, `resolve_conflicts(overlapping_regions)`. Use keyframe encoding exchange (as in CGS-SLAM) for low-bandwidth communication.

---

## Summary

| # | Defect | Severity | Primary Research Source |
|---|--------|----------|----------------------|
| 371.1 | No spatial memory / SLAM in NT-PHYSICAL | High | VIGS-SLAM, LXD-SLAM, FusionCore |
| 371.2 | No dynamic scene handling in PerceptionBridge | High | π³Mos, DROID-W, RoSe-SLAM |
| 371.3 | No streaming 3D reconstruction | Med-High | ZipMap, OVGGT, SLARM, Online3R |
| 371.4 | No uncertainty-aware sensor fusion | Med-High | CRUISE, ADM-Fusion, DAP-Pose |
| 371.5 | No semantic spatial grounding for LLMs | Medium | HSGM, GLMap, SuperMap, OVI-MAP |
| 371.6 | No loop closure / global consistency | Medium | ICF-SLAM, MSN-SLAM, UniSim-SLAM |
| 371.7 | No collaborative multi-agent spatial awareness | Med-Low | CGS-SLAM, FusionBridge |

**Key insight from this batch:** The 2026 SLAM/fusion research has converged on three paradigm shifts that NeoTrix has not absorbed:
1. **Feed-forward foundation models replacing handcrafted pipelines** — VGGT, DUSt3R, π³ now provide geometric priors that eliminate failure-prone multi-view geometry stages
2. **Uncertainty as a first-class citizen** — per-pixel uncertainty from multi-view consistency (not predefined motion priors) is the new standard for dynamic scene handling
3. **Linear-time / O(1) streaming** — ZipMap (TTT layers) and OVGGT (self-selective caching) prove that quadratic complexity is no longer acceptable for real-time 3D reconstruction

NeoTrix's NT-PHYSICAL and NT-WORLD layers must absorb these paradigm shifts to maintain relevance for embodied agent scenarios.

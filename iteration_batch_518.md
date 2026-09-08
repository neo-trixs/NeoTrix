# Iteration Batch 518 — Research Loop: SLAM, Motion Planning, Robot Perception

**Date**: 2026-09-06
**Phase**: External Research → Defect Identification → Design Optimization

---

## 1. Sources Cited

### SLAM (2026)
1. **Visual SLAM vs LiDAR SLAM Explained** — Smashing Robotics, 2026-09-03. Covers sensor fusion as 2026 standard practice, neural edge SLAM (SuperPoint+LightGlue), edge AI silicon acceleration. `https://www.smashingrobotics.com/visual-slam-vs-lidar-slam-explained/`
2. **Hilti-Trimble SLAM Challenge 2026** — 360° Visual-Inertial benchmark with floor plan priors on active construction sites. Open dataset on HuggingFace. `https://hilti-challenge.com/dataset-2026`
3. **Visual SLAM Guide 2026: Algorithms, Research, and Deep Learning Trends** — AppliedKaos/Substack. Covers SELM-SLAM3 with SuperPoint+LightGlue replacing handcrafted features. `https://appliedkaos.substack.com/p/visual-slam-guide-2026-algorithms`
4. **Robot SLAM Technology Landscape 2026** — PatSnap Eureka. 70+ patent records, 4 innovation phases, VLA-SLAM (Vision-Language-Action) as the AI-native frontier. `https://www.patsnap.com/resources/blog/rd-blog/robot-slam-technology-landscape-2026-patsnap-eureka/`
5. **AELVI-SLAM: LiDAR–visual–inertial SLAM** — ScienceDirect, 2026. Multi-sensor fusion for agricultural robots. `https://www.sciencedirect.com/science/article/pii/S258972172600036X`
6. **Lidar-inertial SLAM with visual QR codes** — Nature Scientific Reports 16, 4020 (2026). Indoor mobile robot localization via QR landmarks. `https://www.nature.com/articles/s41598-025-34165-2`
7. **Degeneracy-Resistant LiDAR-SLAM** — Wiley, 2025. Multi-feature-modality SLAM fusing visual features from LiDAR-generated images with geometric features. `https://onlinelibrary.wiley.com/doi/full/10.1002/rob.70026`

### Motion Planning (2026)
8. **General Introduction to Recent Advancements in Path and Trajectory Planning** — IJARS 2026. Comprehensive survey: sampling-based, APF, optimal control, graph methods. `https://journals.sagepub.com/doi/10.1177/17298806261446021`
9. **Optimal Motion Planning for Autonomous Robotic Systems** — JAIA 2026. Categorizes into graph-based, tree-based, trajectory optimization. Challenges in non-convex problems. `https://www.sciltp.com/journals/jaia/articles/2602003141`
10. **AI-driven Path Planning for Autonomous Vehicles** — ScienceDirect 2026. Hybrid frameworks combining graph, sampling, optimization, and learning-based methods. `https://www.sciencedirect.com/science/article/pii/S2590123026022735`
11. **Coverage Path Planning: Classical Foundations, Recent Advances** — arXiv 2026. Multi-robot CPP, 3D CPP, learning-based CPP, visual CPP. 125 works surveyed. `https://arxiv.org/pdf/2607.10649`
12. **ProxDDP: Proximal Constrained Trajectory Optimization** — IEEE TRO 2025. DDP-type method for fast constrained trajectory optimization with warm-starting for MPC. `https://dl.acm.org/doi/10.1109/TRO.2025.3554437`
13. **Motion Planning in Dynamic Environments** — arXiv 2026. Survey from classical to modern methods. `https://arxiv.org/pdf/2606.02677`

### Robot Perception (2026)
14. **Computer Vision Breakthroughs of 2026** — Robotocist. 3D scene understanding, video generation, real-time visual reasoning. `https://robotocist.com/articles/computer-vision-2026`
15. **Industrial Robot Perception Technology Landscape 2026** — PatSnap. 44% patent activity surge in 2025. 3D vision, sensor fusion, AI object recognition. `https://www.patsnap.com/resources/blog/articles/industrial-robot-perception-technology-landscape-2026/`
16. **Articulated 3D Scene Graphs for Open-World Mobile Manipulation** — arXiv 2026 (MoMa-SG). Semantic-kinematic scene graphs with articulation models. `https://arxiv.org/abs/2602.16356`
17. **CVPR 2026 Workshop on 3D Scene Understanding** — 3D foundational models, zero-shot generalization, real-to-sim-to-real transfer, VLA models. `https://scene-understanding.com/`
18. **Deep Learning for Humanoid Robot Vision** — Springer 2026. Real-time object detection, 3D scene understanding, multimodal sensor fusion. `https://link.springer.com/chapter/10.1007/978-981-95-7087-4_31`
19. **VLM-driven Scene Understanding for Robotic Manipulation** — IEEE 2026. Vision-Language-Model approach for unknown environment manipulation. `https://ieeexplore.ieee.org/document/10711845`
20. **Active Perception for 3D Scene Representations** — UPenn PhD 2025. 3D Gaussian Splatting + Fisher Information Matrix for active SLAM. `https://repository.upenn.edu/entities/publication/46461780-adcc-4aae-9b67-3c31e5087d56`

---

## 2. Defects Identified

### DEFECT-001: No SLAM Module in NT-PHYSICAL or NT-WORLD
**Severity**: CRITICAL
**Evidence**: Grep for `SLAM|slam|VisualOdometry|LiDAR|lidar` across all `.rs` files returned zero relevant matches. The `nt_physical::mod.rs` defines `SensorType` with `Camera, Microphone, IMU, GPS, Temperature, Proximity, Force, Custom` — missing `LiDAR`, `DepthCamera`, `RGBD`, `UWB`, `Radar`, `EventCamera`.
**Gap**: The 2026 landscape shows sensor fusion (Camera+LiDAR+IMU) is now standard practice. NeoTrix's physical embodiment layer has no SLAM capability, no occupancy grid mapping, and no pose estimation pipeline. This means NT-PHYSICAL cannot perform spatial localization or mapping — a fundamental capability for any embodied system.
**Source**: [1], [4], [5], [7]

### DEFECT-002: SensorType Enum Incomplete for 2026 Multi-Modal Fusion
**Severity**: HIGH
**Evidence**: `SensorType` at `nt_physical/mod.rs:32-41` only has 6 concrete types + Custom. The 2026 PatSnap landscape (70+ patents) shows production robots use Camera+LiDAR+IMU+UWB+WiFi+Event Cameras as standard sensor stacks. Missing types: `LiDAR`, `DepthCamera`, `RGBD`, `EventCamera`, `UltraWideband`, `WiFiFingerprint`, `Radar`, `Ultrasonic`, `ThermalCamera`.
**Gap**: Without these sensor types, the perception pipeline cannot model the multi-modal sensor fusion that defines 2026 production robots.
**Source**: [1], [4], [15]

### DEFECT-003: No Neural Feature Extraction in Visual Cortex
**Severity**: HIGH
**Evidence**: `VisualCortex` at `visual_cortex.rs:4-49` is a minimal stub — it reads a file path and creates a `SensoryEvent` with metadata only. No actual image processing, no feature extraction, no keypoint detection. The 2026 trend is SELM-SLAM3 with SuperPoint+LightGlue (learned features) replacing handcrafted ORB/SIFT.
**Gap**: VisualCortex cannot perform any real visual perception. It should integrate learned feature extractors (SuperPoint) and transformer-based matchers (LightGlue) for robust keypoint detection in dynamic/low-light environments.
**Source**: [3], [14], [18]

### DEFECT-004: No Occupancy Grid or Spatial Mapping
**Severity**: HIGH
**Evidence**: `nt_world_map` module (types, projection, spatial, geocode) handles geographic coordinates and spatial indexing, but has no occupancy grid, no volumetric mapping, no octree representation. The 2026 SLAM Challenge benchmark evaluates trajectory completeness and position accuracy on construction sites with floor plan priors.
**Gap**: The map module is designed for flat geographic data (GeoPoint, BoundingBox, WebMercator) — not for 3D spatial mapping required by embodied agents navigating physical spaces.
**Source**: [2], [16]

### DEFECT-005: No Motion Planning Pipeline
**Severity**: CRITICAL
**Evidence**: Grep for `motion_planning|path_planning|trajectory|MPC|RRT|DWA|PID` returned zero motion planning code. The `BodySchema` in `nt_physical/mod.rs:137-171` defines joints and parts but has no forward/inverse kinematics, no collision checking, no trajectory generation.
**Gap**: The 2026 state-of-the-art shows motion planning decomposed into path planning + trajectory planning, with optimization-based methods (MPC, ProxDDP) as the preferred approach for real-time control. NeoTrix has no equivalent.
**Source**: [8], [9], [10], [12]

### DEFECT-006: PerceptionBridge Gate Logic Commented Out
**Severity**: MEDIUM
**Evidence**: `perception_bridge.rs:80-102` — the core `gate_event` and `gate_events` methods are entirely commented out. Tests reference undefined `SelectiveState` type. The attention-gated perception flow described in CONTEXT.md (`PerceptionBridge` using `awareness_score()`) is non-functional.
**Gap**: The PerceptionBridge is supposed to connect L2 SensoryIntegrationHub with L5 consciousness via attention gating. Without this, sensory events flood consciousness without filtering — no salience routing.
**Source**: Code inspection

### DEFECT-007: No Event-Based / Neuromorphic Sensor Support
**Severity**: MEDIUM
**Evidence**: 2026 Visual SLAM Guide highlights event-based sensors as transforming how machines perceive space. The `SensorType` enum has no `EventCamera` variant. Event cameras offer microsecond temporal resolution and high dynamic range — critical for fast-moving robots and extreme lighting.
**Gap**: Missing foundational sensor type for next-generation visual SLAM.
**Source**: [3], [14]

### DEFECT-008: No VLA (Vision-Language-Action) Integration in Perception
**Severity**: MEDIUM
**Evidence**: PatSnap 2026 reports VLA-SLAM as the AI-native frontier — integrating Vision-Language-Action large models into SLAM pipelines for dynamic filtering, keyframe selection, and semantic-conceptual alignment. Hefei Keda's 2026 CN patents are the most advanced VLA-SLAM integrations. China Telecom 2026 introduces natural language intent as SLAM exploration driver.
**Gap**: NeoTrix's VSA HyperCube and GWT attention routing are conceptual analogs but have no concrete VLA-SLAM bridge — language-guided spatial exploration is missing.
**Source**: [4]

### DEFECT-009: No Loop Closure or Map Consistency Verification
**Severity**: MEDIUM
**Evidence**: SLAM systems require loop closure detection to correct accumulated drift. The `WorldConsciousness` module tracks conversation turns but has no spatial loop closure. The Hilti Challenge 2026 evaluates on exponential accuracy models that heavily penalize drift.
**Gap**: Without loop closure, long-duration spatial exploration will accumulate unbounded pose error.
**Source**: [2], [6]

### DEFECT-010: No 3D Scene Graph Representation
**Severity**: MEDIUM
**Evidence**: MoMa-SG (arXiv 2026) introduces semantic-kinematic 3D scene graphs with articulation models for open-world mobile manipulation. CVPR 2026 Workshop focuses on 3D foundational models for embodied AI. NeoTrix's `OmniscientView` is conversation-focused, not spatial.
**Gap**: No structured spatial-semantic representation (scene graph) that connects geometry, semantics, and kinematics for manipulation tasks.
**Source**: [16], [17]

### DEFECT-011: Safety Kernel is Rule-String-Based, Not Constraint-Based
**Severity**: LOW
**Evidence**: `SafetyKernel` at `nt_physical/mod.rs:88-114` uses string-typed `rule_type` and `condition` fields. Modern safety kernels use formal constraint specifications (timelimit constraints, energy bounds, velocity limits) that can be verified at compile time or runtime via convex optimization.
**Gap**: String-based rules cannot be formally verified and are error-prone.
**Source**: [9], [12]

### DEFECT-012: No Coverage Path Planning for Task Execution
**Severity**: LOW
**Evidence**: CPP survey (arXiv 2026) covers 125 works on multi-robot coverage, 3D coverage, learning-based coverage. NeoTrix's `ProductionOrchestrator` manages task parallelism but has no spatial coverage planning for physical inspection, cleaning, or surveillance tasks.
**Gap**: When NT-PHYSICAL agents need to systematically cover a physical area, there is no algorithmic support.
**Source**: [11]

---

## 3. Optimization Suggestions

### SUGGEST-001: Add `nt_physical_slam` Module (P0 — Foundation)
- Implement a modular SLAM trait `trait SlamBackend` with pluggable implementations:
  - `VisualSlam` (ORB-SLAM3 / SELM-SLAM3 wrapper)
  - `LidarSlam` (Cartographer / Hector SLAM wrapper)
  - `FusedSlam` (multi-sensor fusion via factor graph)
- Extend `SensorType` enum with `LiDAR`, `DepthCamera`, `RGBD`, `EventCamera`, `UltraWideband`, `Radar`
- Add `OccupancyGrid` and `OctreeMap` types to `nt_world_map`

### SUGGEST-002: Reactivate and Extend PerceptionBridge (P0)
- Uncomment `gate_event` / `gate_events` in `perception_bridge.rs`
- Add `SpatialEvent` variant to `SensoryEventKind` for SLAM-related sensory data
- Connect PerceptionBridge output to ConsciousnessTree's GWT attention routing

### SUGGEST-003: Add Motion Planning Trait Hierarchy (P1)
- Define `trait MotionPlanner` with:
  - `fn plan_path(&self, start: &Config, goal: &Config, obstacles: &ObstacleMap) -> Path`
  - `fn optimize_trajectory(&self, path: &Path, constraints: &DynamicsConstraints) -> Trajectory`
- Implement `RRTStar`, `BIT*` for global planning
- Implement `MPC` / `ProxDDP` wrapper for local trajectory optimization
- Add `CollisionChecker` with GJK/SAT support

### SUGGEST-004: Add Semantic-Spatial Scene Graph (P1)
- Implement `SpatialSceneGraph` type with nodes (objects, rooms, surfaces) and edges (spatial relations, containment, kinematics)
- Support articulation models (revolute/prismatic joints) per MoMa-SG 2026
- Bridge to VSA HyperCube for semantic embedding of spatial concepts

### SUGGEST-005: Add VLA-SLAM Bridge (P2)
- Define `trait VlaSlamBridge` that connects LLM/VLM inference with SLAM keyframe selection
- Use NeoTrix's existing LLM infrastructure for language-grounded exploration planning
- Implement intent-driven frontier selection (China Telecom 2026 pattern)

### SUGGEST-006: Loop Closure Detection (P2)
- Add `LoopClosureDetector` to SLAM pipeline using:
  - Bag-of-words (DBoW2) for visual loop closure
  - Scan context for LiDAR loop closure
- Connect to KB for long-term spatial memory persistence

### SUGGEST-007: Event Camera Integration (P2)
- Add `EventCamera` sensor type and event stream processing
- Integrate with visual SLAM for high-speed/low-light scenarios
- Event cameras produce sparse asynchronous data — natural fit for GWT attention gating

### SUGGEST-008: Formal Safety Constraints (P3)
- Replace string-based `SafetyRule` with typed constraint structs:
  ```rust
  enum SafetyConstraint {
      VelocityLimit { joint: JointId, max_vel: f64 },
      TorqueLimit { joint: JointId, max_torque: f64 },
      WorkspaceBoundary { aabb: Aabb },
      EnergyBudget { max_watts: f64, window: Duration },
  }
  ```
- Add runtime verification via convex optimization feasibility check

---

## 4. Summary

| Metric | Count |
|--------|-------|
| Sources cited | 20 |
| Defects found | 12 |
| Suggestions | 8 |
| CRITICAL defects | 2 (no SLAM module, no motion planning) |
| HIGH defects | 3 (incomplete SensorType, no neural features, no spatial mapping) |
| MEDIUM defects | 5 (PerceptionBridge disabled, no event camera, no VLA, no loop closure, no scene graph) |
| LOW defects | 2 (string safety rules, no coverage planning) |

**Priority actions**: DEFECT-001 and DEFECT-005 are foundational — NT-PHYSICAL cannot function as an embodied agent without SLAM and motion planning. SUGGEST-001 and SUGGEST-003 should be implemented first.

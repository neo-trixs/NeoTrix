# Iteration Batch 345 — Robotics Research Defect Scan

**Date**: 2026-09-06
**Domains**: Manipulation, Locomotion, Motion Planning
**Research Depth**: 2026 ICRA/CoRL/ICLR/IEEE + arXiv (50+ sources scanned)

---

## 1. SOURCES CITED

### Manipulation (Dexterous Grasping & Tactile)
| # | Source | Date | Key Advance |
|---|--------|------|-------------|
| S1 | ICRA 2026 Dexterous Hands Survey (Robots Daily) | 2026-06 | Wuji Hand 2: 20-DOF, <570g, 12kg grip, full tactile arrays; hands now serve as embodied AI interface, not end-effectors |
| S2 | IEEE RAM "Developments Toward Dexterous Embodied Manipulation" (Li et al.) | 2026-04 | Survey mapping evolution from mechanical programming → embodied intelligence; defines open challenges in force-closure, contact-rich tasks |
| S3 | DexSim2Real (Zeng et al., arXiv:2605.05241) | 2026-05 | VLM-guided domain randomization + Tactile-Visual Cross-Attention Policy; 78.2% sim-to-real success, only 8.3% sim-real gap |
| S4 | VTLA (Zhang et al., Biomim. Intell. Robot.) | 2026-04 | Vision-Tactile-Language-Action model; DPO preference learning for sim-to-real peg-in-hole; cross-modal temporal reasoning |
| S5 | TacVLA (arXiv:2603.12665) | 2026-03 | Contact-aware tactile fusion for VLA models; tactile modality closes vision-only failure modes |
| S6 | Science Robotics "Visual-tactile pretraining for humanlike dexterity" | 2026-01 | Cross-attention IPL token pretraining → humanlike grasp; attention maps show tactile is binding constraint |
| S7 | Tactile Survey 2020-2026 (Huang) | 2026 | UniVTAC (sim platform), FlexiTac (open hardware), CGP (contact-grounded policy); data scarcity + sensor standardization flagged as critical bottlenecks |
| S8 | DexRobot Automate 2026 | 2026-06 | Integrated hardware+data+toolchain ecosystem; DexTele teleoperation for high-fidelity data acquisition |
| S9 | DO AS I DO (arXiv:2606.19333) | 2026-06 | Retarget monocular human video → multi-fingered robot hand; in-the-wild data scaling |
| S10 | 1X Technologies NEO | 2026 | 25-DOF tendon-driven hand with force transparency; forearm-motor, lightweight fingers |

### Locomotion (Quadruped/Bipedal)
| # | Source | Date | Key Advance |
|---|--------|------|-------------|
| S11 | He et al., Frontiers Robot. AI "Adaptive multi-mode locomotion via sparse MoE-DRL" | 2026-02 | Mixture-of-Experts for wheel-legged bipedal; resolves gradient conflict in multi-modal locomotion; phased curriculum |
| S12 | Harvard/SEAS "Flexible Locomotion Learning with Diffusion-MPC" (Huang et al., ICLR 2026) | 2026-04 | Diffusion model as learned MPC prior; zero-shot adaptation to new constraints/terrains at test time on real Unitree Go2 |
| S13 | Zhang et al., Biomim. Intell. Robot. "Phase-aware iLQR for quadruped jumping" | 2026-03 | Phase-aware trajectory optimization; validated on Deeprobotics Lite3 for dynamic jumping |
| S14 | Neuromorphic RL for Quadruped Locomotion (arXiv:2605.09595) | 2026 | CPG-RL integration with neuromorphic compute; event-driven sensory processing for uneven terrain |
| S15 | Nature Sci. Reports "Adaptive motion planning for legged robots via deep RL" | 2026-01 | Deep RL for terrain-aware locomotion; generalizes across rough/slippery/mixed terrain |
| S16 | Figure AI System 0 | 2026 | 10M-parameter neural net replacing 100K lines of code; full-body control at 1kHz for climbing ladders |
| S17 | Google DeepMind Gemini Robotics 2 | 2026 | Whole-body control dual architecture; <200 training samples to adapt to new hardware |
| S18 | NVIDIA SONIC | 2026 | Open-source motion foundation model; universal control policy from MoCap data; generalizes to unseen behaviors |
| S19 | Boston Dynamics Atlas (electric) | 2026 | Sim-to-real in 1 hour from millions of sim hours; full electric transition |
| S20 | HOVER (CMU/NVIDIA, 2025) | 2025 | 15+ control modes distilled into single locomotion controller |

### Motion Planning (Path Planning & Obstacle Avoidance)
| # | Source | Date | Key Advance |
|---|--------|------|-------------|
| S21 | HMP-DRL (Kolomeytsev et al., arXiv:2512.24651) | 2025-12 | Hybrid graph+DRL planner; entity-aware collision avoidance; socially compliant navigation |
| S22 | ADRL+RPCO (ScienceDirect) | 2025-09 | Adaptive DRL with meta-optimized exploration for indoor path planning |
| S23 | DRL+MPC+Noise Layers (MDPI Sensors) | 2025-01 | Hybrid DRL-MPC with noise injection for dynamic obstacle safety guarantees |
| S24 | DDQN+Dual-Branch CNN for AUV (Ocean Eng.) | 2026 | Multimodal perception (sonar+current); zero prior info needed; kinematic smoothing |
| S25 | NVIDIA Isaac Lab / GPU-accelerated planning | 2026 | Massively parallel sim for thousands of planning scenarios simultaneously |
| S26 | Skild Brain (Skild AI) | 2025 | Omni-bodied foundation model: humanoids, quadrupeds, arms, mobile manipulators unified |
| S27 | "Grounding Sim-to-Real in VLA Models" (Jin et al., arXiv:2603.22876) | 2026-03 | Empirical study of 4 sim-to-real dimensions; domain randomization still dominant lever |

---

## 2. DEFECTS FOUND IN NEOTRIX DESIGN

### DEFECT M-1: No Vision-Tactile Fusion Pipeline (Manipulation)
**Severity**: HIGH | **Layer**: L3 Embodiment / L2 Perception
**Evidence**: `nt_physical/mod.rs` defines `SensorType::Force` but has no `Tactile` variant. No tactile sensor type, no tactile data model, no tactile-policy integration path. The entire tactile sim-to-real pipeline (UniVTAC, Tac2Real, VTLA) is absent.
**2026 Gap**: S3/S4/S5/S7/S8 all demonstrate that tactile-visual cross-attention is the critical enabler for contact-rich manipulation. Without tactile fusion, NT-PHYSICAL cannot support dexterous grasping tasks. Force sensors alone are insufficient — tactile arrays provide contact geometry, slip detection, and force distribution that force/torque sensors cannot.
**Suggestion**: Add `SensorType::Tactile` with subtypes (GelSight, magnetic, capacitive). Define a `TactileProcessor` in `nt_physical` that outputs contact-geometry maps. Create a cross-attention fusion module in `nt_sense` (L2) that combines visual + tactile streams for manipulation policies.

### DEFECT M-2: No Vision-Language-Action (VLA) Model Integration
**Severity**: HIGH | **Layer**: L5 Cognition / L1 Action
**Evidence**: No VLA model in the codebase. `nt_core` has no foundation model interface for robot action generation. The trend toward dual-system VLA architectures (fast control + slow reasoning) is entirely unrepresented.
**2026 Gap**: S3/S4/S5/S6/S10 demonstrate that VLA models (VTLA, TacVLA, π0, GR00T N1) are the dominant paradigm for manipulation. The dual-system split (Helix/GR00T/Fast-in-Slow) maps directly to NT-CORE's E8 + GWT attention architecture, but no bridge exists.
**Suggestion**: Define a `VLAProvider` trait in `nt_io` that bridges VLM backbones to action heads. Leverage GWT's saliency routing to implement the fast-slow split: GWT → fast control loop, E8 → reasoning/planning. This is a natural fit for the consciousness architecture.

### DEFECT L-1: No Diffusion-Based Locomotion Controller
**Severity**: HIGH | **Layer**: L3 Embodiment / L5 Cognition
**Evidence**: `nt_physical/mod.rs` models motors as simple position/velocity targets. No diffusion model, no learned dynamics prior, no test-time adaptation. The `PhysicalEmbodiment::control_motor` is a trivial pass-through with no dynamics model.
**2026 Gap**: S12 (Diffusion-MPC) demonstrates that diffusion models as learned priors enable flexible locomotion adaptation at test time — changing joint constraints, heights, terrains without retraining. This outperforms fixed RL policies. S18 (SONIC) shows universal motion foundation models generalizing to unseen behaviors.
**Suggestion**: Add a `LocomotionPolicy` trait in `nt_physical` with implementations for: (a) Diffusion-MPC (learned prior + MPC optimization at test time), (b) Foundation model backbone (SONIC/HOVER-style). Connect to E8 for gait selection reasoning. The trait should accept real-time constraint overrides (joint limits, velocity bounds, terrain type).

### DEFECT L-2: No Gait Foundation Model or Multi-Mode Control
**Severity**: MEDIUM | **Layer**: L3 Embodiment
**Evidence**: `BodySchema` has no gait representation. The `Joint` type models single-joint limits but has no coordination model across joints. No concept of gait patterns, phase synchronization, or multi-mode locomotion (walk/run/climb/jump).
**2026 Gap**: S11 (MoE-DRL) shows that multi-mode locomotion requires explicit mode-switching with specialized expert subnetworks. S20 (HOVER) distills 15+ modes into one controller. S16 (Figure System 0) demonstrates whole-body coordination at 1kHz.
**Suggestion**: Add `GaitPattern` enum (Walk, Trot, Run, Crawl, Jump, Climb) and `GaitController` that coordinates joint phase relationships. Integrate with MoE architecture: one expert per gait mode, gating network for smooth transitions.

### DEFECT P-1: No Hybrid Global-Local Motion Planner
**Severity**: HIGH | **Layer**: L1 Action / L2 Perception
**Evidence**: NT-ACT has no motion planning module. NT-WORLD has `UnifiedCrawler` for web crawling but no path planning. The perception layer has no obstacle representation or spatial awareness.
**2026 Gap**: S21 (HMP-DRL) demonstrates that the global-graph + local-DRL hybrid is the state-of-the-art for navigation. Pure DRL fails at long-range goals; pure graph planning fails at dynamic obstacles. The combination is essential.
**Suggestion**: Create `nt_act::motion_planner` module with: (a) Global layer (A*/Dijkstra over occupancy grid), (b) Local layer (DRL policy for dynamic obstacle avoidance), (c) Entity-aware collision cost (different safety margins for humans vs objects vs walls). Bridge with NT-WORLD's spatial perception for real-time obstacle updates.

### DEFECT P-2: No Sim-to-Real Transfer Infrastructure
**Severity**: HIGH | **Layer**: L3 Embodiment / Cross-cutting
**Evidence**: No domain randomization, no sim environment abstraction, no sim-real gap measurement. `PhysicalEmbodiment` is purely a runtime abstraction with no simulation counterpart.
**2026 Gap**: S3 (DexSim2Real) achieves 78.2% real-world success with only 8.3% sim-real gap using VLM-guided DR. S19 (Atlas) achieves 1-hour sim-to-real. S27 (Jin et al.) empirically identifies the four critical dimensions of sim-to-real transfer. Without sim infrastructure, NT-PHYSICAL cannot iterate on manipulation/locomotion policies.
**Suggestion**: Define a `SimulationBackend` trait (Isaac Gym / MuJoCo / custom) with domain randomization parameters. Create a `SimRealGap` metric in `nt_meta` that tracks policy transfer performance. Integrate with SEAL pipeline for automated sim-real evaluation cycles.

### DEFECT L-3: No Neuromorphic / Event-Driven Sensing Path
**Severity**: MEDIUM | **Layer**: L2 Perception / L3 Embodiment
**Evidence**: Sensor polling model is synchronous (`read_sensor` returns `Option<&Sensor>`). No event-driven or neuromorphic processing. No support for DVS cameras or event-based tactile sensors.
**2026 Gap**: S14 (Neuromorphic RL) demonstrates that event-driven sensing dramatically reduces latency for legged locomotion on uneven terrain. The robotics field is converging on neuromorphic processing for low-latency, low-power perception.
**Suggestion**: Add `SensorMode::EventDriven` variant. Define `EventBuffer` that accumulates sparse sensor events between control ticks. This enables sub-millisecond reactive responses without continuous polling, critical for dynamic locomotion and contact-rich manipulation.

### DEFECT M-3: No Teleoperation / Demonstration Data Pipeline
**Severity**: MEDIUM | **Layer**: L1 Action / NT-ACT
**Evidence**: No teleoperation interface. No demonstration recording. No kinesthetic teaching. The only data collection path is through manual coding.
**2026 Gap**: S8 (DexTele) and S9 (DO AS I DO) demonstrate that teleoperation and video-based demonstration retargeting are the primary paths for scalable data collection. Without this, NT-ACT cannot bootstrap manipulation policies.
**Suggestion**: Define `TeleoperationInterface` trait supporting: (a) VR/controller teleoperation, (b) human video retargeting (DO AS I DO style), (c) demonstration recording to KB for imitation learning. Connect to NT-MEMORY for experience storage.

### DEFECT P-3: No Force-Closure / Grasp Planning Module
**Severity**: MEDIUM | **Layer**: L1 Action / L5 Cognition
**Evidence**: No grasp planner. No force-closure computation. No object pose estimation integration. The motor control is purely position/velocity without contact-force awareness.
**2026 Gap**: S1/S2 identify that grasp planning with force-closure guarantees is a prerequisite for reliable manipulation. Current systems integrate vision-based pose estimation with analytical grasp planning (S2) or learned policies (S1).
**Suggestion**: Add `GraspPlanner` in `nt_act` that combines: (a) point cloud → object pose estimation (from NT-WORLD), (b) force-closure analysis, (c) candidate grasp scoring. Output feeds into `PhysicalEmbodiment::control_motor` as target finger positions/forces.

---

## 3. CROSS-CUTTING SUGGESTIONS

### Suggestion CC-1: Map VLA Dual-System to Consciousness Architecture
The 2026 VLA landscape converges on dual-system architectures (fast control + slow reasoning). This maps directly to NeoTrix's existing GWT + E8 design:
- **GWT (fast path)**: Broadcasts salient perceptual signals → immediate motor commands (sub-100ms)
- **E8 (slow path)**: Hexagram-based reasoning for task planning, grasp strategy, gait selection
- **Bridge**: `PerceptionBridge` already exists but needs to connect to action generation, not just attention modulation

### Suggestion CC-2: Foundation Model Abstraction in NT-IO
The proliferation of foundation models (SONIC, HOVER, GR00T, π0, Cosmos Policy) demands a unified abstraction. NT-IO should define a `RobotFoundationModel` trait that abstracts across:
- Motion foundation models (SONIC, HOVER)
- Manipulation foundation models (π0, RDT-1B, FP3)
- World models (Cosmos, DreamZero, 1X WM)
This allows the consciousness architecture to reason about and select among available models.

### Suggestion CC-3: Contact-Rich Task Pipeline
Several 2026 papers converge on a pipeline: Perception → Contact Prediction → Grasp/Manipulate → Tactile Feedback → Force Adjustment. NeoTrix should define this as a first-class pipeline in the SEAL stages, not ad-hoc per task.

---

## 4. SEVERITY SUMMARY

| Severity | Count | Defects |
|----------|-------|---------|
| HIGH | 5 | M-1, M-2, L-1, P-1, P-2 |
| MEDIUM | 4 | L-2, L-3, M-3, P-3 |

**Critical Path**: M-1 (tactile) → M-2 (VLA) → P-1 (motion planning) → P-2 (sim-to-real)
These four form a dependency chain: tactile fusion feeds VLA, VLA generates plans, motion planner executes, sim-to-real validates.

---

## 5. RECOMMENDED NEXT ITERATION FOCUS

1. **Highest ROI**: DEFECT M-1 + M-2 — tactile fusion + VLA integration. This unlocks the entire manipulation pipeline and leverages existing GWT/E8 architecture.
2. **Infrastructure**: DEFECT P-2 — sim-to-real. Without this, all manipulation/locomotion work cannot validate on real hardware.
3. **Quick Win**: DEFECT P-1 — motion planning. The HMP-DRL hybrid approach is well-understood and can be implemented as an NT-ACT module with existing graph search libraries.

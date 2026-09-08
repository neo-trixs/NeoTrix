# Iteration Batch 479 — Robotic Manipulation & Sim-to-Real Gap Analysis

**Date**: 2026-09-06
**Focus**: Robot Grasping, Dexterous Manipulation, Sim-to-Real Transfer — 2026 Advances
**Target**: NT-PHYSICAL domain defect identification against state-of-the-art

---

## Sources Cited

| # | Source | Year | Key Insight |
|---|--------|------|-------------|
| S1 | [geoSuctionBot / ICCAS 2026](https://arxiv.org/abs/2608.28246) | 2026-08 | Training-free suction grasping using VLM (Gemini Robotics-ER) + SAM2 + geometric surface scoring (KNN-PCA/Sobel/RANSAC). 88.2% single-object, 72.6% cluttered retrieval. No retraining needed — prompt-driven retargeting. |
| S2 | [AdaRoboVLG / arXiv 2609.04096](https://arxiv.org/abs/2609.04096) | 2026-09 | Composable foundation priors for task-adaptive Vision-Language-Grasp. Decouples physical grasp synthesis (kinematic + force-closure base policy) from task understanding (pluggable foundation-model priors). Zero cross-hand generalization without retraining. |
| S3 | [GOAG / arXiv 2608.19759](https://arxiv.org/abs/2608.19759) | 2026-08 | Object-agnostic gripper-centric grasp planner. CVAE learns gripper contact-surface distribution at training time (no object data), retrieves compatible contact areas at inference via BPS encoding. 86.93% avg across Barrett/Allegro/Shadow. |
| S4 | [GraspGen-X / CVPR 2026](https://openaccess.thecvf.com/content/CVPR2026/papers/Han_GraspGen-X_Cross-Embodiment_6-DOF_Diffusion-based_Grasping_CVPR_2026_paper.pdf) | 2026 | Cross-embodiment 6-DOF diffusion grasping. Swept-volume gripper encoding (12-dim), trained on 395M grasps across 25 procedural grippers. Zero-shot generalization to novel real-world grippers. |
| S5 | [UNOGrasp / CVPR 2026](https://openaccess.thecvf.com/content/CVPR2026/html/Jiao_Obstruction_Reasoning_for_Robotic_Grasping_CVPR_2026_paper.html) | 2026 | Vision-language obstruction reasoning for grasping. Multi-step reasoning about objects that must be cleared before grasping target. Supervised + RL finetuning with verifiable reasoning rewards. 100K+ human-annotated obstruction paths. |
| S6 | [CoToGrasp / ECCV 2026](https://arxiv.org/abs/2608.19776) | 2026-08 | Contact-topology-conditioned dexterous grasp synthesis. Trained object-agnostically in canonical workspace. Zero-shot generalization to unseen objects. State-of-the-art on DexGraspNet. |
| S7 | [VTAP Gripper / Nature 2026](https://www.nature.com/articles/s44182-026-00079-y) | 2026-02 | Visuo-Tactile Active Palm — 13-DOF gripper with actuated bi-modal (vision+tactile) palm + piezoresistive fingertip arrays. 93.3% YCB success, peg-in-hole at 1mm tolerance. Demonstrates finger-palm synergy with only 7 DOF. |
| S8 | [VISTA-Policy / arXiv 2608.25872](https://arxiv.org/abs/2608.25872) | 2026-08 | Visual Deformation Field (VDF) as visuo-physical feedback for contact-rich manipulation. Imitation learning without tactile hardware — compliant gripper deformation as proxy. Outperforms pure-vision and tactile baselines. |
| S9 | [SAT / CVPR 2026](https://openaccess.thecvf.com/content/CVPR2026/papers/Lei_Structural_Action_Transformer_for_3D_Dexterous_Manipulation_CVPR_2026_paper.pdf) | 2026 | Structural Action Transformer — reframes actions as variable-length joint-wise trajectory sequences (Da, T) instead of temporal chunks. Embodied Joint Codebook for cross-embodiment transfer. 19.36M params. |
| S10 | [FibTac / npj Robotics 2026](https://www.nature.com/articles/s44182-026-00112-0) | 2026-08 | Fiber-based pneumatic gripper with embedded tactile sensing. Carbon fibers in silicone, pneumatic actuation, internal camera captures fiber-tip motion. Works underwater. 186g payload. |
| S11 | [Closing Reality Gap / alphaXiv 2026-01](https://www.alphaxiv.org/abs/2601.02778) | 2026-01 | Zero-shot sim-to-real for dexterous force-based grasping. Fast tactile sim (600 virtual tactile units, parallel FK), current-to-torque calibration (no torque sensors needed), actuator dynamics randomization. First controllable force-based grasping on multi-finger hand. |
| S12 | [Blind Dexterous Grasping / arXiv 2606.11767](https://arxiv.org/abs/2606.11767) | 2026-06 | Real2Sim tactile calibration pipeline + layout-aware tactile encoder + diffusion policy aggregation. 27% blind grasp success on real LEAP Hand using only tactile feedback, no vision. |
| S13 | [Facet-0 / arXiv 2609.01596](https://arxiv.org/abs/2609.01596) | 2026-09 | Robotic foundation model for contact-rich precise manipulation. Joint action-wrench proposal (flow matching generates action + predicted wrench). 82% success on sub-millimeter assembly, 50ms latency. |
| S14 | [TwinRL / arXiv 2602.09023](https://arxiv.org/pdf/2602.09023) | 2026 | Digital twin-real-world collaborative RL. Twin as exploration amplifier (not just data aug). Twin RL warm-up initializes real replay buffer. Human-in-the-loop guided by twin-identified failure-prone states. |
| S15 | [HyperSim / arXiv 2605.26638](https://arxiv.org/html/2605.26638) | 2026 | Holistic sim-to-real: geometry-aware 3DGS rendering + adversarial trajectory synthesis + sim-and-real co-training. 80-95% real-world success rates (ACT/π₀). 35% higher completion under perturbations. |
| S16 | [VIRAL / CVPR 2026](https://openaccess.thecvf.com/content/CVPR2026/html/He_VIRAL_Visual_Sim-to-Real_at_Scale_for_Humanoid_Loco-Manipulation_CVPR_2026_paper.html) | 2026 | Visual sim-to-real at scale for humanoid loco-manipulation. Teacher-student: privileged RL teacher → vision student via tiled rendering (up to 64 GPUs). Zero-shot to Unitree G1, 54 continuous cycles. |
| S17 | [Video2Sim2Real / arXiv 2606.08828](https://arxiv.org/abs/2606.08828) | 2026-06 | Full-stack autonomous dexterous skill acquisition from single human video. Digital-twin reconstruction → object-centric keyframe optimization → decoupled IL (geometry gap) + residual RL (physics gap). 95% real-world success. |
| S18 | [Sim2Real Practitioner Guide 2026](https://www.roboticscenter.ai/blog/sim-to-real-tips-practitioners) | 2026-04 | Domain randomization best practices: camera ±10cm/±10deg, brightness ±30%, friction ±50%, mass ±30%. System ID before training reduces error 30-50%. Contact parameter randomization critical. Highest-performing pipeline = sim training + small real fine-tuning. |

---

## Defects Found

### DEFECT-001: NT-PHYSICAL Is a Game Animation Engine, Not a Robotic Manipulation Platform

**Source**: S1-S18 (all 2026 sources)
**Current Code**: `nt_physical/mod.rs:9-17` — `PhysicalEmbodiment` struct contains `Vec<Sensor>`, `Vec<Motor>`, `SafetyKernel`, `PowerManager`, `BodySchema` — a generic hardware abstraction
**Current Code**: `nt_physical/embodied_physics.rs:1-531` — `Skeleton`, `RagdollPhysics`, `PhysicsDrivenAnimation`, `BalanceController` — a humanoid ragdoll physics engine with PD-controlled joints
**Gap**: The 2026 state-of-the-art in robotic manipulation has evolved far beyond generic sensor/motor abstractions and humanoid ragdoll physics. The entire field has converged on: (1) grasp planning via foundation models or learning-based policies, (2) tactile-reactive manipulation with dense contact feedback, (3) cross-embodiment transfer via structural action representations, and (4) sim-to-real pipelines with digital twins. NT-PHYSICAL has none of these capabilities. It is architecturally positioned as a "physical embodiment skeleton" but implements only game-engine physics (PBD ragdoll, PD controllers, forward kinematics) with no robotic manipulation primitives.
**Impact**: NT-PHYSICAL cannot serve as the embodiment layer for any robotic manipulation task. The `Sensor` enum (Camera/Microphone/IMU/GPS/Temperature/Proximity/Force/Custom) has no tactile sensing variant. The `Motor` enum (Servo/Stepper/DC/Brushless/Pneumatic/Hydraulic) has no gripper/finger actuator type. The `BodySchema` is rigidly humanoid with no support for arbitrary robot morphologies.
**Suggestion**: Refactor NT-PHYSICAL into two sub-domains: (1) `nt_physical::game_physics` (existing ragdoll/animation, for digital content), (2) `nt_physical::robotics` (new: grasp planning, tactile sensing, manipulation policy, cross-embodiment adapter). Add `SensorType::Tactile` (with `TaxelGrid` resolution), `MotorType::Gripper`, `MotorType::Finger`, and `BodySchema::from_urdf()` for arbitrary robot morphology loading.

### DEFECT-002: No Tactile Sensing Model — Blind to Contact-Rich Tasks

**Source**: S7 (VTAP), S8 (VISTA), S10 (FibTac), S11 (Closing Reality Gap), S12 (Blind Dexterous)
**Current Code**: `nt_physical/mod.rs:32-41` — `SensorType` enum: `Camera, Microphone, IMU, GPS, Temperature, Proximity, Force, Custom(String)`
**Gap**: The 2026 tactile sensing landscape has converged on: (1) vision-based tactile sensors (VTAP's bi-modal palm, GelSight-style), (2) piezoresistive fingertip arrays (VTAP's FlexiTac 32×12, 2mm resolution), (3) fiber-based distributed sensing (FibTac's carbon fiber in silicone), (4) dense virtual tactile units for sim (600 units per hand in S11). The current `SensorType::Force` is a scalar — it captures a single force magnitude, not a contact distribution. There is no `Tactile` sensor type, no taxel grid representation, no contact geometry, no slip detection, no force-closure evaluation.
**Impact**: Any manipulation task requiring contact feedback (grasping fragile objects, in-hand reorientation, peg-in-hole insertion, blind grasping) is impossible. The system cannot detect contact geometry, estimate surface normals, detect incipient slip, or evaluate grasp stability from tactile signals. S7 (VTAP) shows 93.3% grasp success using only tactile feedback; S12 shows 27% blind grasp success on real hardware. NT-PHYSICAL has zero tactile capability.
**Suggestion**: Add `SensorType::Tactile` with `TactileSensor` struct: `resolution: (usize, usize)` (taxel grid dimensions), `modality: TactileModality` (VisionBased | Piezoresistive | FiberBased | Capacitive), `contact_distribution: Vec<ContactPoint>` where `ContactPoint { position: (f32,f32,f32), force: f32, normal: (f32,f32,f32), slip_detected: bool }`. Add `GraspStability::evaluate(tactile: &TactileReading) -> StabilityScore` with force-closure check.

### DEFECT-003: No Grasp Planning Pipeline

**Source**: S1 (geoSuctionBot), S2 (AdaRoboVLG), S3 (GOAG), S4 (GraspGen-X), S5 (UNOGrasp), S6 (CoToGrasp)
**Current Code**: No grasp planning module exists in NT-PHYSICAL or anywhere in the codebase
**Gap**: 2026 grasp planning has evolved along multiple parallel tracks: (1) VLM-driven grasp detection with no retraining (S1: Gemini+SAM2+geometric scoring), (2) cross-embodiment diffusion models with swept-volume gripper encoding trained on 395M grasps (S4), (3) object-agnostic gripper-centric planners that learn contact-surface distributions (S3, S6), (4) obstruction reasoning for multi-step grasp sequences in clutter (S5), (5) composable foundation priors for task-adaptive grasping (S2). NT-PHYSICAL has zero grasp planning — no grasp pose generation, no contact point selection, no stability evaluation, no gripper morphology encoding.
**Impact**: The system cannot plan, evaluate, or execute any grasping task. It has no concept of a "grasp pose," "contact point," "force closure," or "grasp success metric." This is the single largest functional gap in NT-PHYSICAL.
**Suggestion**: Add `nt_physical::grasp_planning` module with: `GraspPlanner` trait (`plan_grasp(scene: &Scene, target: &Object, gripper: &GripperModel) -> Vec<GraspCandidate>`), `GraspCandidate { pose: Pose6D, confidence: f64, stability: StabilityScore, approach_direction: (f32,f32,f32) }`, `GripperModel` enum (ParallelJaw | Suction | MultiFinger { fingers: usize, dof_per_finger: usize }). Start with geometric scoring (S1 approach) as baseline, then add learned planners.

### DEFECT-004: No Sim-to-Real Transfer Pipeline

**Source**: S11 (Closing Reality Gap), S14 (TwinRL), S15 (HyperSim), S16 (VIRAL), S17 (Video2Sim2Real), S18 (Practitioner Guide)
**Current Code**: No sim-to-real, digital twin, or domain randomization code exists
**Gap**: Sim-to-real transfer is now the dominant paradigm for robotic manipulation deployment. 2026 approaches converge on: (1) digital twins as exploration amplifiers (S14: TwinRL), (2) adversarial trajectory generation + sim-and-real co-training (S15: HyperSim, 80-95% success), (3) teacher-student distillation with privileged state (S16: VIRAL, 64-GPU scale), (4) Real2Sim2Real pipelines with decoupled IL+RL (S17: Video2Sim2Real), (5) systematic domain randomization with pre-training system ID (S18: reduces error 30-50%). NT-PHYSICAL has no sim-to-real infrastructure: no domain randomization, no digital twin, no co-training, no Real2Sim calibration.
**Impact**: Any manipulation policy trained in simulation cannot be deployed to real hardware. The system cannot bridge the reality gap, cannot calibrate simulation parameters, cannot generate diverse training data, and cannot adapt policies to real-world dynamics.
**Suggestion**: Add `nt_physical::sim_to_real` module with: `DomainRandomizer` (camera offset ±10cm, friction ±50%, mass ±30%, lighting ±30% brightness), `DigitalTwin` trait (reconstruct from URDF + point cloud, align via ICP+differentiable rendering), `SimRealCoTrainer` (joint optimization on mixed sim/real batches), `Real2SimCalibrator` (trajectory matching + genetic algorithm parameter optimization per S11).

### DEFECT-005: No Cross-Embodiment Transfer Mechanism

**Source**: S3 (GOAG), S4 (GraspGen-X), S9 (SAT), S6 (CoToGrasp)
**Current Code**: `nt_physical/embodied_physics.rs:55-140` — `Skeleton::humanoid()` hardcoded to single humanoid morphology
**Gap**: 2026 cross-embodiment transfer has converged on: (1) gripper-centric training (object-agnostic, S3: GOAG), (2) swept-volume gripper encoding (12-dim, S4: GraspGen-X), (3) structural action representation with variable-length joint sequences + Embodied Joint Codebook (S9: SAT), (4) canonical workspace projection for morphology-agnostic grasp synthesis (S6: CoToGrasp). NT-PHYSICAL hardcodes a single humanoid skeleton — cannot represent or transfer skills across different robot morphologies (grippers, multi-finger hands, suction cups, soft robots).
**Impact**: Each new robot requires complete rewrite of all manipulation code. No skill sharing between a 2-finger gripper and a 5-finger hand. No adaptation to novel gripper morphologies at inference time.
**Suggestion**: Add `nt_physical::cross_embodiment` module with: `EmbodimentEncoder` trait (encode any robot morphology into fixed-dim latent, e.g., swept-volume heuristic), `ActionAdapter` (remap joint-space actions between embodiments via joint codebook), `MorphologyAgnosticPolicy` (policy conditioned on embodiment encoding, not hardcoded joint count). Add `BodySchema::from_urdf(path)` for arbitrary robot loading.

### DEFECT-006: No Contact-Rich Manipulation Policy

**Source**: S8 (VISTA), S9 (SAT), S13 (Facet-0), S17 (Video2Sim2Real)
**Current Code**: `nt_physical/embodied_physics.rs:368-394` — PD controller with target joint angles (animation control, not manipulation)
**Gap**: 2026 manipulation policies operate on: (1) visual deformation fields as visuo-physical feedback without tactile hardware (S8: VISTA), (2) structural action representations with time-flow matching (S9: SAT), (3) joint action-wrench proposals with flow matching that predict both action and contact forces (S13: Facet-0, 82% sub-mm assembly), (4) decoupled IL (geometry) + residual RL (physics) for real-world adaptation (S17). NT-PHYSICAL's `PhysicsDrivenAnimation` uses PD control for animation blending — it has no concept of manipulation goals, contact-force optimization, or policy-driven action generation.
**Impact**: The system can animate a humanoid ragdoll but cannot perform any manipulation task (pick, place, insert, twist, pour, reorient). The PD controller targets animation poses, not task objectives.
**Suggestion**: Add `nt_physical::manipulation_policy` module with: `ManipulationPolicy` trait (`observe(state: &Observation) -> Action`), `Action` enum (JointSpace(Vec<f64>) | CartesianSpace(Pose6D, Wrench6D)), `Observation` (point_cloud: PointCloud, tactile: Option<TactileReading>, proprioception: JointState, language: Option<String>). Implement `DiffusionPolicy` (flow-matching based) and `StructuralActionTransformer` (S9 approach) as concrete policies.

### DEFECT-007: No Force/Torque Control — Position-Only Actuation

**Source**: S11 (Closing Reality Gap), S13 (Facet-0), S7 (VTAP)
**Current Code**: `nt_physical/mod.rs:225-247` — `control_motor(motor_id, position: Option<f64>, velocity: Option<f64>)` — position/velocity control only
**Current Code**: `nt_physical/embodied_physics.rs:370-378` — PD controller with `stiffness * error - damping * velocity` (position-domain)
**Gap**: 2026 manipulation requires force/torque control: (1) S11 demonstrates current-to-torque calibration enables controllable grasp force tracking without torque sensors, (2) S13 (Facet-0) proposes joint action-wrench proposals — the policy predicts both action and resulting wrench, enabling contact-aware planning, (3) S7 (VTAP) uses MPC for tactile-reactive manipulation requiring force feedback. NT-PHYSICAL's motor control is purely kinematic — `control_motor` sets position/velocity directly with no torque command, no force control loop, no impedance/admittance control.
**Impact**: Cannot perform force-sensitive tasks (insertion, assembly, handling fragile objects). Cannot regulate grasp force (will crush soft objects or drop heavy ones). Cannot detect contact forces for tactile feedback loops.
**Suggestion**: Add `MotorControlMode` enum (Position | Velocity | Torque | Impedance { stiffness: f64, damping: f64 }) to `Motor`. Add `control_motor_torque(motor_id, torque: f64)` and `control_motor_impedance(motor_id, target_force: f64, stiffness: f64)` methods. Add `Wrench6D { force: (f32,f32,f32), torque: (f32,f32,f32) }` for 6-axis force/torque representation.

### DEFECT-008: No URDF/Robot Description Language Support

**Source**: S4 (GraspGen-X procedural grippers), S9 (SAT heterogeneous embodiments), S16 (VIRAL Unitree G1)
**Current Code**: `nt_physical/embodied_physics.rs:55-140` — `Skeleton::humanoid()` hardcoded with 20 bone definitions and 14 joint definitions
**Gap**: Every 2026 robotics framework loads robot descriptions from URDF, MJCF, or USD files: (1) S4 trains on 25 procedurally generated gripper URDFs, (2) S9 handles heterogeneous embodiments by treating joint count as variable sequence length, (3) S16 deploys to Unitree G1 humanoid loaded from its URDF. NT-PHYSICAL hardcodes a single humanoid skeleton as Rust struct literals — cannot load external robot descriptions, cannot represent non-humanoid morphologies, cannot serialize/deserialize robot models.
**Impact**: Each new robot requires manual Rust code to define its skeleton. No interoperability with standard robotics tooling (ROS, MuJoCo, IsaacGym). Cannot load commercial robot models (UR5, Franka, Allegro, Shadow).
**Suggestion**: Add `nt_physical::robot_description` module with: `RobotModel::from_urdf(xml: &str) -> Result<Self>`, `RobotModel::from_mjcf(xml: &str) -> Result<Self>`. Parse `<joint>`, `<link>`, `<visual>`, `<collision>`, `<dynamics>` elements into existing `BodyPart`/`Joint`/`JointLimits` structs. Add `to_urdf()` for export. Use `urdf-rs` crate as dependency.

### DEFECT-009: No Foundation Model Integration for Manipulation

**Source**: S1 (VLM grasping), S5 (VL obstruction reasoning), S13 (Facet-0 contact-aware VLA), S8 (VISTA visuo-physical)
**Current Code**: No foundation model integration in NT-PHYSICAL
**Gap**: 2026 manipulation has converged on foundation model integration: (1) VLMs for grasp target identification from text prompts (S1: Gemini), (2) VL models for multi-step obstruction reasoning (S5: custom VLM), (3) VLA models for contact-rich manipulation with wrench prediction (S13: Facet-0), (4) visual deformation fields as policy input without tactile hardware (S8: VISTA). NT-PHYSICAL has zero LLM/VLM/VLA integration — it is a pure physics engine with no semantic understanding.
**Impact**: Cannot interpret natural language manipulation instructions, cannot reason about task geometry, cannot leverage pre-trained visual features for grasp planning.
**Suggestion**: Add `nt_physical::foundation_bridge` module with: `ManipulationVLM` trait (wraps external VLM for grasp target grounding), `GraspLanguageGrounding` (text prompt → object mask → grasp candidates, per S1 pipeline), `ContactAwareVLA` (action-wrench prediction conditioned on language, per S13). Integrate with existing `nt_core_llm` provider system for model routing.

### DEFECT-010: No Digital Twin Infrastructure

**Source**: S14 (TwinRL), S17 (Video2Sim2Real), S15 (HyperSim), S16 (VIRAL)
**Current Code**: `nt_physical/mod.rs:296-315` — `integrate_3d_tool()` returns `"status": "simulated"` placeholder
**Gap**: Digital twins are now the core infrastructure for sim-to-real transfer: (1) S14 uses digital twin as exploration amplifier with twin RL warm-up, (2) S17 reconstructs digital twins from single human videos using foundation models (Gemini+SAM3+HaMeR), (3) S15 uses 3DGS-based photorealistic digital twins with physics, (4) S16 uses tiled rendering across 64 GPUs for large-scale twin simulation. NT-PHYSICAL's `integrate_3d_tool()` is a stub that returns `"status": "simulated"`.
**Impact**: Cannot create simulation environments for policy training. Cannot run parallel rollouts. Cannot bridge sim-to-real gap. The 3D tool integration (Blender-MCP/Unity-MCP) absorbed in CONTEXT.md terminology has no actual implementation.
**Suggestion**: Add `nt_physical::digital_twin` module with: `DigitalTwin` struct (scene_graph: SceneGraph, physics_engine: PhysicsEngine, renderer: Renderer), `TwinBuilder::from_point_cloud(pc: &PointCloud) -> DigitalTwin`, `TwinBuilder::from_urdf(urdf: &str) -> DigitalTwin`, `parallel_rollout(policy: &impl Policy, n: usize) -> Vec<Trajectory>`. Use existing `nt_shield_sandbox` Docker infrastructure for isolated simulation containers.

### DEFECT-011: Safety Kernel Lacks Contact-Aware Safety Rules

**Source**: S7 (VTAP force limits), S11 (force-based grasp constraints), S13 (Facet-0 wrench prediction), S18 (practitioner guide)
**Current Code**: `nt_physical/mod.rs:87-114` — `SafetyKernel` with `SafetyRule { condition: String, action: String }` — rule conditions are arbitrary strings with no typed safety model
**Current Code**: `nt_physical/mod.rs:264-293` — `check_safety()` only checks battery level and motor overheat
**Gap**: 2026 contact-aware safety requires: (1) force/torque limits per joint and end-effector (S11: grasp failure constraints for multi-suction), (2) contact force thresholds for fragile objects (S7: tactile-reactive force regulation), (3) wrench-based safety margins (S13: predicted wrench vs. joint limits). NT-PHYSICAL's safety kernel has only two hardcoded checks (battery, motor overheat) with string-based rules that cannot express contact-force safety constraints.
**Impact**: No force-limiting safety. Robot can exert unlimited force on objects/environment. Cannot enforce grasp failure constraints from trajectory planning. No protection against contact-related damage.
**Suggestion**: Add `ContactSafetyRule` struct: `max_contact_force: f64`, `max_joint_torque: Vec<f64>`, `force_closure_required: bool`, `max_slip_rate: f64`. Add `GraspFailureConstraint` (per S11) that models suction cup load distribution and integrates with trajectory planning. Replace string-based `condition` with typed `SafetyCondition` enum (LowBattery | MotorOverheat | ExcessiveForce { threshold: f64 } | JointTorqueLimit | GraspInstability).

### DEFECT-012: No Proprioceptive State Estimation for Manipulation

**Source**: S11 (current-to-torque calibration), S12 (layout-aware tactile encoder), S17 (IL from point clouds)
**Current Code**: `nt_physical/embodied_physics.rs:211-214` — `get_joint_angles()` returns raw joint angles only
**Current Code**: No state estimation, no Kalman filtering, no contact state estimation
**Gap**: 2026 manipulation requires rich proprioceptive state: (1) S11 maps motor current → joint torque without torque sensors, (2) S12 uses layout-aware tactile encoding with sensor geometry priors, (3) S17 estimates contact state from point clouds. NT-PHYSICAL's proprioception is raw joint angles — no velocity estimation, no torque estimation, no contact state, no object pose estimation in hand, no state filtering.
**Impact**: Policies receive minimal state information. Cannot estimate contact forces from motor currents. Cannot track object pose during in-hand manipulation. Cannot filter noisy sensor readings.
**Suggestion**: Add `ProprioceptiveState` struct: `joint_positions: Vec<f64>`, `joint_velocities: Vec<f64>`, `joint_torques: Vec<f64>`, `contact_states: Vec<ContactState>`, `object_pose_in_hand: Option<Pose6D>`. Add `StateEstimator` with Kalman filter for velocity/torque estimation, `CurrentToTorqueMapper` for motor-current-based torque estimation (per S11), `ContactStateEstimator` for contact detection from tactile/force data.

---

## Summary

| Metric | Count |
|--------|-------|
| **Sources Analyzed** | 18 (2026 publications) |
| **Defects Found** | 12 |
| **Critical** | 4 (DEFECT-001, DEFECT-003, DEFECT-004, DEFECT-006) |
| **High** | 5 (DEFECT-002, DEFECT-005, DEFECT-007, DEFECT-008, DEFECT-010) |
| **Medium** | 3 (DEFECT-009, DEFECT-011, DEFECT-012) |

### Top 3 Priority Fixes

1. **DEFECT-001 (NT-PHYSICAL Identity Crisis)** — The domain is architecturally a game animation engine but labeled as "physical embodiment skeleton." Refactor into game-physics and robotics sub-domains. All other defects are downstream of this identity misalignment.

2. **DEFECT-003 (No Grasp Planning)** — The most impactful missing capability. A single `GraspPlanner` trait with geometric scoring baseline (S1 approach) would unlock basic manipulation. Cross-embodiment support (S3/S4) should follow.

3. **DEFECT-004 (No Sim-to-Real)** — Without sim-to-real infrastructure, no manipulation policy can be deployed to real hardware. Start with domain randomization (S18 best practices) and digital twin (S14 approach) as foundational infrastructure.

### Cross-Cutting Theme: Foundation Model Integration

DEFECT-009 (foundation model integration) connects across all other defects: VLMs for grasp target grounding (DEFECT-003), VLAs for contact-rich manipulation (DEFECT-006), digital twin reconstruction from videos (DEFECT-010). NeoTrix's existing `nt_core_llm` provider system and `Egress Privacy Guard` provide the infrastructure for secure foundation model calls — the manipulation layer needs to plug into this rather than building independent model integrations.

### Architecture Implication

The NT-PHYSICAL domain as currently designed is unsuitable for robotic manipulation. The 2026 research landscape shows that manipulation requires tight integration of perception (tactile+visual), planning (grasp+motion), control (force/torque), and learning (sim-to-real+foundation models). A partial fix is possible by adding modules alongside the existing code, but a clean sub-domain split (game-physics vs. robotics) would prevent architectural debt accumulation.

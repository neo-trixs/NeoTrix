# Iteration Batch 426 — Robotics Middleware & Software Architecture Research

**Date**: 2026-09-06
**Research Areas**: ROS 2 2026, Robot Middleware, Robot Software Architecture

---

## Sources Cited

| # | Source | URL | Date | Relevance |
|---|--------|-----|------|-----------|
| S1 | ROS 2 Releases (GitHub) | https://github.com/ros2/ros2/releases | 2026-04-30 | Lyrical Luth LTS released May 2026, Jazzy Patch 8 |
| S2 | ROS 2 Release Schedule | https://docs.ros.org/en/rolling/The-ROS2-Project/Release-Schedule.html | 2026 | Non-LTS in odd years, LTS aligned to Ubuntu LTS |
| S3 | ROS 2 Distribution Matrix | https://docs.ros.org/en/jazzy/Releases.html | 2026-05-22 | Iron EOL Dec 2024; Kilted EOL Dec 2026; Lyrical LTS to 2031 |
| S4 | ROS 2 News: Latest Releases & EOL | https://posts.terabox.com/hub/ros-2-news-today-latest-releases-migration-deadlines-and-the-future-of-robotics | 2026-08 | Lyrical targets Ubuntu 26.04, "Embodied AI" native hooks for LLMs/VLA |
| S5 | ROS 2 & Robotics Stack 2026 Deep Dive | https://www.youngju.dev/blog/culture/2026-05-16-ros2-robotics-stack-2026-jazzy-kilted-gazebo-moveit-isaac-sim-drake-foxglove-nav2-deep-dive.en | 2026-05-16 | Zenoh as 3rd middleware; BT.cpp v4 + Groot2 standard; Nav2 recovery BT |
| S6 | micro-ROS Official | https://micro.vulcanexus.org/ | 2026 | Multi-RTOS (Zephyr/FreeRTOS/NuttX); XRCE-DDS client-server; zero dynamic memory client |
| S7 | Micro XRCE-DDS | https://micro.vulcanexus.org/docs/concepts/middleware/Micro_XRCE-DDS | 2026-04-29 | DDS-XRCE protocol; multi-transport (UDP/TCP/Serial); full QoS inheritance |
| S8 | ROS 2 for Humanoid Robots 2026 | https://sachinsharma.dev/blogs/ros-2-for-humanoid-robots-the-middleware-powering-the-2026-boom-2026 | 2026-08-01 | DDS zero-copy transport; micro-ROS for embedded joint controllers; Zenoh wireless |
| S9 | Mobile Robot Middleware Market 2026 | https://www.researchandmarkets.com/reports/6245066/mobile-robot-middleware-market-report | 2026 | Market: simulation/testing, device integration, distributed coordination |
| S10 | Harness Engineering for Physical AI (arXiv) | https://arxiv.org/pdf/2606.09416 | 2026-06-09 | **KEY**: Middleware as harness layer for AI model governance (Projection/Isolation/Transfer) |
| S11 | Behavior Trees Reference Architecture 2026 | https://iotdigitaltwinplm.com/behavior-trees-robot-task-planning-architecture-2026/ | 2026-07-28 | BT.cpp v4 standard; FSM→BT shift; recovery Fallback chains; blackboard ports |
| S12 | CABTO: Context-Aware BT Grounding (arXiv) | https://arxiv.org/abs/2603.16809 | 2026-03-17 | **KEY**: LLM-driven BT generation from context + observations |
| S13 | BT vs State Machines in Robotics (ScienceDirect) | https://www.sciencedirect.com/science/article/pii/S2590118425000164 | 2025-11 | Empirical study: BT reactivity advantage; FlexBE vs Groot DSL comparison |
| S14 | Behavior Trees for Robot Autonomy (RobotForge) | https://robotforge.org/tutorials/planning/behavior-trees | 2026-04-26 | BT.cpp production standard; Nav2 navigate_to_pose.xml canonical example |
| S15 | BTGenBot (GitHub) | https://github.com/AIRLab-POLIMI/BTGenBot | 2026 | LLM→BT generation system with GUI + remote robot deployment |

---

## Defects Found

### DEFECT-426-1: No Middleware Abstraction Layer (DDS/Zenoh/XRCE)

**Severity**: HIGH
**Location**: NT-PHYSICAL (L3 Embodiment), NT-ACT (L1 Action)
**Evidence**: S5, S7, S8, S10

**Gap**: NeoTrix has no equivalent to the ROS 2 middleware layer (DDS pub/sub, Zenoh routing, XRCE-DDS for microcontrollers). The architecture assumes direct function calls between modules, but real robotic systems require a publish/subscribe or request/reply transport substrate with QoS semantics.

**Impact**: NT-PHYSICAL (sensors/motors) cannot communicate with NT-ACT (orchestration) in a decoupled, fault-tolerant manner. No support for:
- Zero-copy shared memory transport (DDS intra-process)
- Wireless telemetry over unreliable links (Zenoh)
- Microcontroller-embedded joint controllers (micro-ROS XRCE-DDS)

**Suggestion**: Add an **L0 Transport Layer** or define a `nt_physical::transport` module implementing:
- `trait Transport { fn publish<T: Serialize>(&self, topic: &str, msg: T); fn subscribe<T: DeserializeOwned>(&self, topic: &str) -> Receiver<T>; }`
- QoS profiles: Reliable/BestEffort, TransientLocal/Volatile, history depth
- Transport backends: in-process (channel::bounded), TCP (tokio-tungstenite), DDS (via rmw binding), Zenoh

---

### DEFECT-426-2: No Behavior Tree Integration for Task Orchestration

**Severity**: HIGH
**Location**: NT-ACT (L1 Action), SEAL Pipeline (NT-MIND)
**Evidence**: S11, S12, S13, S14, S15

**Gap**: NeoTrix's task execution is pipeline-stage-based (SEAL make_stage! macro), which is sequential and lacks the reactive recovery semantics that BehaviorTree.CPP v4 provides. The 2026 standard for robot task orchestration is BT with Fallback/Sequence/ReactiveFallback control nodes.

**Impact**:
- No composable recovery chains (every failure mode requires manual FSM transitions)
- No tick-based reactivity to changing world state
- No visual inspection/debugging of task execution flow
- Cannot compose multi-robot fleet allocation (who) with per-robot execution (how)

**Suggestion**: Define a `nt_act::behavior_tree` module:
- `trait BTNode { fn tick(&mut self) -> NodeStatus; }` (SUCCESS/FAILURE/RUNNING)
- Control nodes: Sequence, Fallback, ReactiveFallback, Parallel, RecoveryNode
- Blackboard for typed port substitution between nodes
- Integration with Groot2-compatible XML serialization for visual editing
- Bridge: SEAL stages become BT leaf nodes; SEAL phase transitions become BT control flow

---

### DEFECT-426-3: No AI Model Harness / Runtime Governance Layer

**Severity**: CRITICAL
**Location**: NT-CORE (L5 Cognition), NT-SHIELD (L3 Embodiment)
**Evidence**: S10 (Lee et al. 2026)

**Gap**: The arXiv paper "Harness Engineering for Physical AI" (2026-06) identifies three axes that robot middleware must enforce for any AI model in the control loop:
1. **Projection**: Output region bounds — model outputs must stay within declared value ranges
2. **Isolation**: Inference budget — model must not exceed declared compute/time slot
3. **Transfer**: Fallback to verified baseline when distributional conditions fail

NeoTrix has Egress Privacy Guard (outbound filtering) but no **inbound model governance** for AI models running in the control path. The SEAL pipeline is an evolution loop, not a runtime safety harness.

**Impact**: If NeoTrix runs VLA/LLM policies in the perception→action loop (as Lyrical Luth's "Embodied AI" hooks enable), there is no mechanism to:
- Bound model outputs to safe regions before actuation
- Enforce inference time budgets
- Switch to verified classical fallback when model confidence degrades
- Trigger lifecycle state transitions on safety violations

**Suggestion**: Add `nt_shield::ai_harness` module:
```rust
struct HarnessProfile {
    output_region: OutputBounds,      // min/max per actuator axis
    inference_budget: Duration,       // max wall-clock per tick
    operating_regime: RegimeDecl,     // OOD detector config
    fallback_policy: FallbackPolicy,  // verified classical controller
}

trait HarnessEnforcer {
    fn project(&self, raw_output: &ModelOutput) -> Result<ClampedOutput, SafetyViolation>;
    fn isolate(&self, inference_start: Instant) -> bool; // true = within budget
    fn transfer(&self, regime: &RegimeState) -> Option<FallbackAction>;
}
```
Wire into GWT attention: safety violations broadcast as high-salience events.

---

### DEFECT-426-4: No Lifecycle State Machine for Module Management

**Severity**: MEDIUM
**Location**: All modules, NT-REPAIR (L6 Meta)
**Evidence**: S5, S11

**Gap**: ROS 2's managed node lifecycle (Unconfigured→Inactive→Active→Finalized) provides deterministic startup/shutdown/reconfiguration. NeoTrix modules have compile-time Constellation maturity (C0-C6) but no runtime lifecycle state machine.

**Impact**:
- Cannot gracefully reconfigure a running module without full restart
- NT-REPAIR cannot trigger in-place reconfiguration (only restart or kill)
- No separation between "loaded" and "active" — all modules run at full capacity always
- Hot-swap of providers (LLM backends) requires manual intervention

**Suggestion**: Define `trait Lifecycle` on all L1-L3 modules:
```rust
enum LifecycleState { Unconfigured, Inactive, Active, Finalized, Error }
trait Lifecycle {
    fn configure(&mut self, params: &Params) -> Result<()>;
    fn activate(&mut self) -> Result<()>;
    fn deactivate(&mut self) -> Result<()>;
    fn cleanup(&mut self) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
    fn on_error(&mut self, err: &Error) -> RecoveryAction;
}
```
NT-REPAIR manages lifecycle transitions. EventBus carries lifecycle events.

---

### DEFECT-426-5: No Context-Aware Task Generation (LLM→Behavior)

**Severity**: MEDIUM
**Location**: NT-MIND (L5 Cognition), NT-CORE (E8 Hexagram)
**Evidence**: S12, S15

**Gap**: CABTO (2026-03) and BTGenBot demonstrate LLM-driven automatic behavior tree generation from natural language task descriptions + environmental observations. NeoTrix's E8 Hexagram reasoning is a fixed 64-element grid; it cannot dynamically generate task decomposition trees from context.

**Impact**:
- New robot tasks require manual BT/FSM authoring
- No bridge between LLM reasoning and executable task plans
- E8 reasoning stays in abstract space; never produces actionable behavioral trees

**Suggestion**: Add `nt_mind::task_grounding` module:
- `fn ground_task(llm_plan: &str, world_state: &WorldState) -> Result<BehaviorTree>`
- Use LLM to generate BT XML from natural language + observation context
- Validate generated BT against safety invariants before execution
- Cache successful grounding patterns in KB for reuse

---

### DEFECT-426-6: No Multi-Robot / Fleet Coordination Abstraction

**Severity**: LOW (future)
**Location**: NT-ACT (L1 Action)
**Evidence**: S11, S14

**Gap**: 2026 production stacks separate fleet-level allocation ("who does what") from per-robot BT execution ("how"). NeoTrix is single-agent architecture with no fleet abstraction.

**Impact**: Cannot scale to multi-agent scenarios (warehouse fleets, swarm coordination).

**Suggestion**: Define `nt_act::fleet` trait:
```rust
trait FleetAllocator {
    fn assign_task(&self, task: &Task, fleet: &[RobotHandle]) -> Assignment;
    fn rebalance(&self, fleet_state: &FleetState) -> Vec<Reassignment>;
}
```
Each robot runs its own BT; fleet allocator reasons above the loop.

---

### DEFECT-426-7: No Real-Time Scheduling / Deterministic Execution Guarantees

**Severity**: MEDIUM
**Location**: NT-PHYSICAL (L3), NT-ACT (L1)
**Evidence**: S8, S10

**Gap**: 2026 humanoid middleware requires real-time guarantees for joint control loops (sub-ms jitter). NeoTrix uses async tokio executors with no RT scheduling policy.

**Impact**:
- Joint controller commands may miss deadlines under load
- No priority inversion protection
- GWT attention scheduling has no deadline awareness

**Suggestion**: Add `nt_physical::rt_scheduler` with:
- `SCHED_FIFO`/`SCHED_RR` thread pinning for critical control loops
- Deadline-monotonic priority assignment
- CPU isolation (cpuset) for RT domains
- Budget enforcement: max inference time per tick (ties to DEFECT-426-3)

---

## Summary

| # | Defect | Severity | Effort | Priority |
|---|--------|----------|--------|----------|
| 426-1 | No Middleware Abstraction Layer | HIGH | Large | P1 |
| 426-2 | No Behavior Tree Integration | HIGH | Medium | P1 |
| 426-3 | No AI Model Harness / Governance | CRITICAL | Large | P0 |
| 426-4 | No Lifecycle State Machine | MEDIUM | Small | P2 |
| 426-5 | No LLM→BT Task Grounding | MEDIUM | Medium | P2 |
| 426-6 | No Fleet Coordination | LOW | Large | P3 |
| 426-7 | No Real-Time Scheduling | MEDIUM | Medium | P2 |

**Total defects**: 7
**Critical**: 1 (DEFECT-426-3 — safety harness for AI in control loop)
**High**: 2 (middleware abstraction, behavior trees)
**Medium**: 3 (lifecycle, task grounding, RT scheduling)
**Low**: 1 (fleet coordination)

**Key insight**: The 2026 robotics convergence point is **middleware-as-harness** — robot middleware must enforce safety constraints on AI models running in the control loop, not just route messages. NeoTrix's NT-SHIELD and NT-CORE must absorb this pattern or risk architectural irrelevance when interfacing with real robotic systems.

# Iteration Batch 484 — NeoTrix Consciousness Architecture Research Loop

**Date**: 2026-09-06  
**Domain**: Embodied AI + Robot Learning + Human-Robot Interaction  
**Sources Scanned**: 15 papers/systems (arXiv, CVPR 2026, ICRA 2026, ACL 2026, Nature MI, Springer)

---

## 1. SOURCES CITED

### 1.1 Embodied AI
| Source | Title | Published |
|--------|-------|-----------|
| arXiv:2609.01281 | EmbodiedSkills: Unified Framework for Orchestrating VLA Agents | 2026-09-01 |
| Qwen-VLA (Robotics Center SV) | Unifying VLA Modeling across Tasks/Environments/Embodiments | 2026-05-28 |
| arXiv:2609.03927 | Toward Unified Robot Learning (Survey) | 2026-09-03 |
| arXiv:2606.05979 | World-Language-Action (WLA) Model | 2026-08-24 |
| CVPR 2026 | Mantis: Disentangled Visual Foresight VLA | 2026 |
| arXiv:2608.27550 | VLAct: Representation-Centric Continued Pre-training for VLAs | 2026-08-27 |
| Robotics Center SV | DM0: Embodied-Native VLA | 2026-02-16 |
| ACL 2026 | MIRTH: Mutual-Information Reasoning with Temporal Hubs | 2026 |

### 1.2 Robot Learning
| Source | Title | Published |
|--------|-------|-----------|
| arXiv:2607.01651 | AutoSERL: One Demonstration Is Enough for Real-World RL | 2026-07 |
| arXiv:2608.24741 | One-Shot LfD via Physical Interaction Identification | 2026-08-25 |
| ICRA 2026 | DemoDiffusion: One-Shot Human Imitation via Pre-trained Diffusion Policy | 2026 |
| arXiv:2606.09381 | ReGIL: Retrieval-Guided Imitation from Single Demonstration | 2026-06 |
| arXiv:2607.00033 | CHORD: Contact Wrench Guidance for Dexterous Manipulation | 2026-08-14 |
| Frontiers 2026 | ConceptACT: Episode-Level Concepts for Sample-Efficient IL | 2026-08-12 |
| CVPR 2026 | Lifelong IL with Multimodal Latent Replay | 2026 |
| arXiv:2609.03199 | RoboTok: Internet-Scale Human Demo Retrieval | 2026-09-02 |

### 1.3 Human-Robot Interaction
| Source | Title | Published |
|--------|-------|-----------|
| Springer Applied Intelligence | Model-Based Behavioral Control via Mathematical Model of Mind | 2026-03-09 |
| Nature MI | Shared Embodied Intelligence in Humanoid ergoCub | 2026-07-13 |
| arXiv:2608.22035 | Ludi0.1: Agentic System for Socially Intelligent Robots | 2026-08-22 |
| arXiv:2608.27225 | STEP: State-Aware Task Estimation/Planning for HRC | 2026-08-27 |
| Frontiers 2026 | Worker Expectations of Robot Social Skills and Autonomy | 2026-06-17 |
| ICSR+ART 2026 | International Conference on Social Robotics (Creative Robotics theme) | 2026-07 |
| HRI 2026 | HRI Empowering Society (21st ACM/IEEE) | 2026-03-16 |

---

## 2. DEFECTS FOUND IN DESIGN

### D-484-01: NT-PHYSICAL Missing VLA Action Decoupling Pattern

**Research**: Mantis (CVPR 2026) demonstrates that decoupling visual foresight prediction from action generation via meta-queries + DiT head improves both convergence speed and language retention. EmbodiedSkills (arXiv:2609.01281) formalizes this as a "shared executable-skill interface" where skill decisions are execution proposals with pre-check + post-verification.

**Defect**: NT-PHYSICAL defines `sensors`, `motors`, `safety kernel`, `power management`, and `body schema` as components, but has no explicit **action decoupling** or **skill execution proposal** interface. The six-layer architecture lacks a mechanism where motor commands are treated as "execution proposals" with prerequisite checks and outcome verification before/after physical execution. This is a critical gap for embodied AI integration — physical actions must be validated before execution and verified after, especially in safety-critical scenarios.

**Suggestion**: Introduce a `SkillExecutionProposal` trait in `l3_embodiment/traits.rs` that:
- Pre-checks: verifies environmental prerequisites (clearance, sensor confidence, power state)
- Post-verifies: confirms action outcome matches predicted state
- Records structured trajectories for SEAL pipeline feedback
- Maps to EmbodiedSkills' `ExecutableSkillInterface` pattern

---

### D-484-02: NT-MIND Missing World Model Integration for Long-Horizon Planning

**Research**: WLA-0 (arXiv:2606.05979) achieves 56.5% on RMBench (memory-dependent tasks) by unifying world modeling, language reasoning, and action synthesis. The survey arXiv:2609.03927 argues that representation learning, VLA models, and world models must be integrated (not isolated) for long-horizon temporal reasoning.

**Defect**: NT-MIND's SEAL pipeline operates on exploration→distillation→self-test→absorption cycles but lacks a **world model** component for predicting consequences of actions before execution. The ConsciousnessTree's 6-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core) is reactive, not predictive. There is no mechanism for NT-MIND to simulate hypothetical action sequences and their outcomes before committing to a skill crystallization.

**Suggestion**: Add a `WorldModelPredictor` module to NT-MIND that:
- Maintains a probabilistic forward model of the system state
- Enables "mental simulation" of SEAL pipeline decisions before execution
- Integrates with VSA HyperCube for concept-level prediction (not just pixel-level)
- Supports the "test-time scaling" pattern from WLA-0 for improved decision quality

---

### D-484-03: NT-FEEL Missing Cognitive Theory of Mind for Real-Time Adaptation

**Research**: The Springer Applied Intelligence paper (2026) deploys a Mathematical Model of Mind (MMM) — a control-theoretic ToM — inside a social robot's behavioral controller, achieving 16% engagement increase. The MMM tracks beliefs, goals, and emotions as dynamic state variables in a closed-loop predictive control framework.

**Defect**: NT-FEEL defines EmotionLabel (11 variants) and EmotionEngine but operates as a **reactive affect system** — it detects emotions and generates expressions, but does not maintain a **predictive model of the user's mental state**. There is no Theory of Mind (ToM) integration: NT-FEEL cannot anticipate user beliefs/goals/emotions, nor adapt its behavior proactively. This limits NeoTrix's ability to sustain long-term engagement in collaborative scenarios.

**Suggestion**: Integrate a `TheoryOfMind` submodule in NT-FEEL that:
- Maintains dynamic state variables for user beliefs, goals, emotions (MMM pattern)
- Uses model-predictive control to optimize action selection over anticipated user mental trajectories
- Exposes constraints (safety, social norms) as hard bounds in the optimization
- Feeds into GWT for attention routing based on predicted user state importance

---

### D-484-04: NT-ACT Missing One-Shot Demonstration Learning Pipeline

**Research**: AutoSERL (arXiv:2607.01651) achieves 100% success on insertion tasks from a single demonstration via automated intervention mechanisms. ReGIL (arXiv:2606.09381) uses a single demonstration as persistent external memory. DemoDiffusion (ICRA 2026) bridges embodiment gap from human demo to robot policy without paired data.

**Defect**: NT-ACT defines MCP tools, social media, code execution, and orchestration, but has no **one-shot demonstration learning** capability. The Domain skill mapping (dev/implementer → NT-ACT) covers code execution but not physical/embodied skill acquisition from demonstrations. There is no pipeline for ingesting a single human demonstration and converting it into an executable robot policy with automated recovery mechanisms.

**Suggestion**: Add a `DemonstrationIngestionPipeline` to NT-ACT that:
- Accepts single demonstration trajectories (video, motion capture, teleoperation log)
- Implements sliding window intervention + safety recovery (AutoSERL pattern)
- Uses retrieval-guided exploration (ReGIL pattern) for sample-efficient learning
- Stores learned policies in KB with maturity tracking (C0-C6 constellation)

---

### D-484-05: NT-WORLD Missing Cross-Embodiment Representation Transfer

**Research**: VLAct (arXiv:2608.27550) achieves 92.5% on RoboTwin 2.0 and outperforms GR00T-N1.6 on unseen humanoid embodiment using only 20% of downstream data. Qwen-VLA introduces "embodiment-aware prompt conditioning" for multi-platform support.

**Defect**: NT-WORLD defines UnifiedCrawler, fetchers, parsers, and classifiers for **digital world perception**, but has no mechanism for **cross-embodiment perception transfer**. When NeoTrix interfaces with different physical platforms (robot arms, humanoid, drone), there is no shared representation layer that enables knowledge learned on one embodiment to transfer to another. Each embodiment would require full retraining.

**Suggestion**: Implement a `CrossEmbodimentRepresentation` layer in NT-WORLD that:
- Uses VLM-prior preservation (VLAct pattern) to maintain shared visual-action semantics
- Supports embodiment-aware prompt conditioning for different robot morphologies
- Provides a unified action layout that maps across embodiments while allowing task-specific heads
- Integrates with VSA HyperCube for embodiment-agnostic concept representation

---

### D-484-06: NT-MEMORY Missing Lifelong Learning Without Catastrophic Forgetting

**Research**: CVPR 2026 "Lifelong IL with Multimodal Latent Replay" achieves SOTA on LIBERO with 10-17 point AUC gains and 65% less forgetting via Multimodal Latent Replay (MLR) + Incremental Feature Adjustment (IFA). ConceptACT (Frontiers 2026) adds episode-level semantic concepts for 40% faster convergence.

**Defect**: NT-MEMORY (SQLite KB, FTS5, embeddings) stores knowledge persistently but has no **continual learning** mechanism that prevents catastrophic forgetting when new knowledge is absorbed. The KB append-only model means old knowledge is never overwritten but also never actively maintained/distilled. There is no inter-task regularization to preserve learned skill separability as the knowledge base grows.

**Suggestion**: Add a `ContinualLearningGuard` to NT-MEMORY that:
- Implements MLR-style latent replay for compact memory-efficient experience storage
- Uses IFA-style angular margin constraints to maintain inter-task separability in embedding space
- Activates during SEAL absorption phase to prevent new skill crystallization from degrading existing skills
- Tracks knowledge embedding drift as a health signal for HeartbeatAggregator

---

### D-484-07: NT-CORE Missing Embodied Spatial CoT Reasoning

**Research**: DM0 (Robotics Center 2026) introduces "Embodied Spatial Scaffolding" that constructs spatial Chain-of-Thought reasoning to constrain the action solution space. MIRTH (ACL 2026) uses dual-scale temporal memory hubs + latent reasoning tokens optimized via mutual information.

**Defect**: NT-CORE's E8 Hexagram reasoning engine operates on abstract architectural states but lacks **spatial grounding** for physical reasoning. The ConsciousnessTree tracks cross-domain health but cannot reason about spatial relationships, physical constraints, or embodied geometry. When NeoTrix needs to reason about physical manipulation, navigation, or spatial planning, it falls back to external VLA models without native spatial reasoning capability.

**Suggestion**: Extend NT-CORE with `SpatialCoTReasoning` that:
- Integrates spatial state representations into E8 hexagram states (spatial-aware hexagrams)
- Uses dual-scale temporal memory (MIRTH pattern) for tracking spatial evolution
- Enables the GWT attention router to broadcast spatially-grounded reasoning across modules
- Maintains a body schema reference frame for all spatial judgments

---

## 3. SUGGESTIONS SUMMARY

| ID | Defect | Priority | Domain | Estimated Complexity |
|----|--------|----------|--------|---------------------|
| D-484-01 | Missing VLA Action Decoupling | HIGH | NT-PHYSICAL | Medium — trait + impl |
| D-484-02 | Missing World Model for Planning | HIGH | NT-MIND | High — new module |
| D-484-03 | Missing Theory of Mind | CRITICAL | NT-FEEL | High — new submodule + control loop |
| D-484-04 | Missing One-Shot Demo Learning | HIGH | NT-ACT | Medium — pipeline integration |
| D-484-05 | Missing Cross-Embodiment Transfer | MEDIUM | NT-WORLD | Medium — representation layer |
| D-484-06 | Missing Continual Learning Guard | HIGH | NT-MEMORY | Medium — guard + integration |
| D-484-07 | Missing Spatial CoT Reasoning | MEDIUM | NT-CORE | High — E8 extension |

### Critical Path Recommendation

The three CRITICAL/HIGH items that would most improve NeoTrix's embodied AI readiness:

1. **Theory of Mind in NT-FEEL** (D-484-03) — Foundation for all human-robot collaboration; blocks social robotics capability
2. **World Model in NT-MIND** (D-484-02) — Enables predictive reasoning; required for long-horizon task planning
3. **Action Decoupling in NT-PHYSICAL** (D-484-01) — Safety-critical prerequisite/postcondition verification; required for any physical embodiment integration

---

## 4. CROSS-REFERENCES

- **SEAL Pipeline**: D-484-02, D-484-06 require SEAL stage extensions
- **GWT**: D-484-03, D-484-07 require attention router upgrades
- **VSA HyperCube**: D-484-05, D-484-07 need embodiment-agnostic concept encoding
- **HeartbeatAggregator**: D-484-06 should feed embedding drift as health signal
- **Six-Layer Architecture**: D-484-01 (L3), D-484-02 (L5), D-484-03 (L4), D-484-04 (L1), D-484-05 (L2), D-484-06 (L1), D-484-07 (L5) — spans all 6 layers

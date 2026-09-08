# Iteration Batch 602 — Embodied AI / Sim-to-Real / Robot Foundation Models
**Date**: 2026-09-06 | **Predecessor**: Batch 601 (prefix-level NLI mismatch, schema collapse, atomic composition failure, content effect asymmetry, bidirectional conflict propagation, operator layer bottleneck)

---

## 1. NEW Defects Identified (vs Batch 601)

### D602-1: In-Context Causal Memory Injection Without Gradient Updates
**Source**: Zeva (arXiv:2608.30880, 2026-08-31)
**Finding**: Zeva enables in-context learning from a robot's own physical interaction experience while keeping the policy model frozen, using a dual-timescale causal memory. The robot accumulates causal interaction signals and injects them as context.
**NEW Defect vs B601**: B601 identified prefix-level NLI mismatch as a static alignment problem. Zeva reveals a **dynamic causal memory injection defect**: when frozen policy models receive injected causal signals, the causal extractor must disentangle action→state-change causation from spurious correlations in the interaction stream. If the causal memory retriever retrieves signals with high correlation but low causation, the injected context becomes a **causal poison** — the frozen model trusts it because it looks like valid context, but the underlying causal structure is wrong. This is the opposite of B601's NLI mismatch: here the surface form is correct but the causal structure is corrupted.
**NeoTrix Implication**: The SEAL pipeline's experience absorption must include causal structure verification — not just semantic alignment. KB experience entries need a `causal_graph` field that validates action→effect chains before injection.

### D602-2: Execution Proposal Validity Gap in Closed-Loop Skill Systems
**Source**: EmbodiedSkills (arXiv:2609.01281, 2026-09-01)
**Finding**: EmbodiedSkills treats each skill decision as an execution proposal with prerequisite checks and post-action verification. On memory-dependent tasks (RMBench), task-adapted VLA policies achieve only 12.5% success.
**NEW Defect vs B601**: B601 identified atomic composition failure as inability to compose primitives. EmbodiedSkills reveals a **propositional validity gap**: the runtime checks prerequisites before execution and verifies outcomes afterward, but the 12.5% failure on memory-dependent tasks shows that the **proposal→execution→verification** loop breaks when state evolves between proposal generation and verification. The proposal is valid at generation time but becomes invalid by verification time — a temporal validity decay. B601's composition failure was about building complex from simple; this is about simple becoming stale mid-execution.
**NeoTrix Implication**: SEAL pipeline phase transitions need a **validity timestamp** — each execution proposal must carry a staleness horizon. If verification exceeds the horizon, the proposal must be re-evaluated, not just retried.

### D602-3: Video-First Policy Collapse in Multi-View Egocentric Settings
**Source**: Genie Envisioner (GE, ICLR 2026)
**Finding**: GE-Base uses multi-view video diffusion to predict future head-view and wrist-view observations. Most prior video-to-action approaches use single-view generation, which is misaligned with multi-view egocentric perception. Latent compression for tractability loses fine-grained spatial and contact cues.
**NEW Defect vs B601**: B601 identified schema collapse in LLM inference. GE reveals a **video-latent compression collapse**: when video diffusion latents are compressed to reduce inference latency, the compression is agnostic to contact-critical spatial information. The latent space encodes "what the scene looks like" but discards "where force will be applied." This is a compression-schema mismatch — the latent space schema doesn't align with the task's physical schema. B601's schema collapse was about LLM output token space; this is about visual latent space losing physics-relevant dimensions.
**NeoTrix Implication**: VSA HyperCube embeddings must distinguish **appearance embeddings** (scene structure) from **contact embeddings** (force/collision geometry). The two should have independent compression paths.

### D602-4: Causal Wrench Prediction Coupling as Anti-Precision Mechanism
**Source**: Facet-0 (arXiv:2609.01596, 2026-09-01)
**Finding**: Facet-0 predicts action chunks together with future wrist-wrench profiles, using a distributional Action-Wrench Critic to distinguish motions with similar task progress but different contact outcomes. Achieves 82% on sub-millimeter tasks vs 15% baseline.
**NEW Defect vs B601**: B601 identified content effect asymmetry — the same action producing different outcomes in different contexts. Facet-0 reveals that **action-wrench coupling creates a precision anti-pattern**: when the model simultaneously predicts action AND expected wrench, a feedback loop emerges where high-confidence wrench predictions suppress exploration of alternative contact strategies. The model converges on a single contact mode per task, losing the ability to recover when that mode fails. This is the mirror image of B601's asymmetry: instead of identical actions diverging, identical predictions converge prematurely.
**NeoTrix Implication**: The attention routing (GWT) must maintain **contact mode diversity** as a saliency signal. If the system detects wrench prediction variance collapsing below a threshold, it should inject exploration noise into the action space.

### D602-5: Cross-Embodiment Dynamics Prior Transfer Failure
**Source**: DyPES-VLA (arXiv:2608.06374, 2026)
**Finding**: DyPES-VLA learns shared dynamics priors through future-prediction on action-free videos, then decodes through embodiment-specific MoE action heads. Achieves 98% on LIBERO but only 75.6% real-world average across three embodiments.
**NEW Defect vs B601**: B601 identified bidirectional conflict propagation. DyPES-VLA reveals a **dynamics-prior specificity gap**: the shared dynamics priors learned from future-prediction capture object motion and contact at a coarse temporal scale, but the embodiment-specific MoE heads need fine-grained dynamics at the control frequency. The future-prediction supervision (frame generation) operates at ~10Hz while control operates at ~50Hz. This temporal scale mismatch means the shared priors are **informative but not actionable** — they tell you what will happen but not precisely when/how at the control frequency. B601's conflict propagation was about bidirectional interference; this is about scale-dependent information loss in one direction.
**NeoTrix Implication**: The HyperCube must implement **multi-scale dynamics encoding** — separate embeddings for prediction-scale (~10Hz) and control-scale (~50Hz) dynamics, with explicit bridging between them.

### D602-6: Lifecycle Coupling Fragility in Closed-Loop Embodied Learning
**Source**: Arcadia (CVPR 2026)
**Finding**: Arcadia's four-stage lifecycle (exploration→reconstruction→shared representation→sim-from-real evaluation) is non-decomposable — removing any stage breaks the improvement loop. Real-world results: 46% navigation, 27% manipulation.
**NEW Defect vs B601**: B601 identified operator layer bottleneck. Arcadia reveals a **lifecycle coupling fragility**: the closed-loop requires all four stages to function, but each stage introduces its own error mode. Exploration noise corrupts reconstruction, reconstruction artifacts corrupt shared representation, representation gaps corrupt sim-from-real evaluation, and evaluation errors feed back into exploration. Errors compound multiplicatively across the cycle. The 27% manipulation success shows that even with all stages present, the compound error rate is ~73%. B601's bottleneck was about a single layer; this is about cascading errors across interdependent layers.
**NeoTrix Implication**: The SEAL pipeline needs **error isolation gates** between phases — if one phase's output exceeds an error threshold, it must not propagate to the next. This is the biological equivalent of synaptic pruning preventing error amplification.

### D602-7: Action-Sufficiency Gap Between Visual Richness and Control Utility
**Source**: GIFT (arXiv:2609.04193, 2026-09-03)
**Finding**: GIFT identifies an "action-sufficiency gap" — visual and predictive features are rich but don't contain enough control-relevant structure (geometry, affordance, goal-region). Adding three structural constraints improves VLA by 4.6 points, WAM by 12.6 points.
**NEW Defect vs B601**: B601 identified atomic composition failure. GIFT reveals a **representation sufficiency gap**: the features extracted by VLM backbones are information-rich but **control-poor**. The visual encoder sees everything but knows nothing about what matters for motor control. This is distinct from B601's composition failure (which was about combining primitives) — this is about the **primitives themselves being wrong at the representation level**. The VLM backbone's native features are optimized for semantic understanding, not physics-aware motor control.
**NeoTrix Implication**: The perception bridge (PerceptionBridge) must implement a **control-relevance filter** that extracts geometry/affordance/goal-region features before routing to action modules. The current design routes raw perception features; it should route control-structured features.

---

## 2. Sim-to-Real Transfer Findings

### D602-8: Abstract Simulator Grounding Requires History-Based Corrections
**Source**: ASTRA (arXiv:2604.15289)
**Finding**: Abstract simulators leave out key task details. ASTRA formalizes that corrections must be functions of state-action histories (not just current state) to handle partial observability from abstraction. Achieves 65% on modified Ant with morphological variation vs 21% direct transfer.
**NEW Defect vs B601**: B601's operator layer bottleneck was about single-layer constraint. ASTRA reveals that **abstraction-induced partial observability** creates a defect where history-free corrections systematically fail. When the simulator abstracts away dynamics, the correction function needs memory of past states to infer what was lost. This is a new class of defect: **temporal information loss from abstraction** that can't be fixed by better current-state modeling.
**NeoTrix Implication**: KB experience entries must carry **history windows**, not just snapshots. The absorption protocol must preserve temporal context chains.

### D602-9: Geometry-Aware Continual Adaptation Achieves 1/6 Data Efficiency
**Source**: GeCo-SRT (CVPR 2026)
**Finding**: Uses local geometric features (surface normals) as domain-invariant, task-invariant knowledge medium. MoE module dynamically activates geometry-specific experts. 52% average improvement, matches baseline with 16.7% data.
**NEW Defect vs B601**: This is a **positive finding** that addresses B601's bidirectional conflict propagation. GeCo-SRT shows that geometric features provide a conflict-free transfer medium because they are invariant across both domains and tasks. However, the limitation noted — "primarily focuses on bridging the observation gap" and "may limit applicability in non-geometric sim-to-real gaps" — reveals a new defect: **geometry-only invariance is insufficient for contact-rich tasks** where dynamics (friction, deformation, material properties) dominate.

### D602-10: Holistic Sim-to-Real Co-Training Synergy
**Source**: HyperSim (arXiv:2605.26638)
**Finding**: Combines high-fidelity Gaussian Splatting rendering, adversarial trajectory generation, and sim-and-real co-training. Achieves 95% SR with π0 policies. Adversarial trajectories provide 35% higher completion under perturbations.
**Positive Contribution**: Validates that co-training with domain-invariant representation learning is the strongest approach when combined with high-fidelity rendering. The synergy between foundation models + synthetic data + co-training is greater than any individual component.

### D602-11: Provably Safe Sim-to-Real Transfer via Reward-Free Safe RL
**Source**: arXiv:2609.01418 (2026-09-01)
**Finding**: Formalizes safe sim-to-real transfer within reward-free safe RL framework. Provides real-world sample complexity bounds characterizing the benefit of using the simulator in terms of sim-to-real mismatch.
**NEW Defect vs B601**: Reveals a **safety-safety tradeoff**: the simulator can be used to reduce real-world samples, but the safety constraints on real-world data collection interact with the sim-to-real mismatch in non-trivial ways. Tighter safety constraints reduce real-world samples but increase the required simulator accuracy, creating a **safety-accuracy coupling** that B601's operator bottleneck didn't capture.

---

## 3. Robot Foundation Model Findings

### D602-12: Hierarchical VLA with World-Model-Guided Test-Time Computation
**Source**: τ0-VLA (arXiv:2608.16885, 2026)
**Finding**: τ0-VLA uses memory-augmented high-level policy with propose-predict-evaluate loop. Allocates additional computation when uncertain. Trained on 40,115 hours of heterogeneous data. Unified 40-dimensional state/action space.
**NEW Defect vs B601**: B601 identified schema collapse in LLM inference. τ0-VLA reveals a **test-time computation allocation paradox**: the model allocates more computation when uncertain, but uncertainty estimation itself is unreliable for long-horizon tasks where early errors compound. The world model's predictions become less accurate for later subtasks, so the search over alternatives at later stages is searching over bad options. This is a **cascading uncertainty** defect — uncertainty about uncertainty.
**NeoTrix Implication**: GWT attention routing must implement **uncertainty decay awareness** — when the model is uncertain about its own uncertainty, it should default to conservative execution rather than expanded search.

### D602-13: World Action Model Unification of Policy and Simulation
**Source**: Riemann-1.0 (arXiv:2608.27033, 2026-08-27)
**Finding**: Fully causal autoregressive WAM that jointly models multi-view observations, robot states, and actions as causal state transitions. Single model functions as both executable policy AND multi-embodiment visual world simulator. 94.3% RoboTwin2.0, 99.0% LIBERO, 62.6% RoboCasa-365.
**Positive Contribution**: Demonstrates that unified causal world-action modeling is the strongest paradigm for scaling robot foundation models. Progressive embodied pretraining (human videos → gripper demos → robot trajectories) is validated as the optimal curriculum.
**NEW Defect vs B601**: The 62.6% on RoboCasa-365 (long-horizon compositional) vs 99% on LIBERO reveals a **compositionality ceiling**: even the strongest WAMs degrade significantly on compositional long-horizon tasks. The causal autoregressive formulation handles single-step causation well but struggles with multi-step composition — each autoregressive step compounds small errors.

### D602-14: Pragmatic VLA Scaling with Efficiency Focus
**Source**: LingBot-VLA (arXiv:2601.18692, 2026)
**Finding**: 20,000 hours of real-world data from 9 platforms. Achieves superiority over π0.5, GR00T N1.6, WALL-OSS. Throughput of 261 samples/sec on 8-GPU cluster (1.5-2.8x speedup over existing codebases).
**NEW Defect vs B601**: LingBot-VLA reveals a **training efficiency bottleneck**: despite architecture convergence, the actual training throughput is the limiting factor. The 1.5-2.8x speedup over existing codebases means the field has been wasting compute on inefficient data pipelines. This is an **infrastructure defect** — the architectures are solved but the engineering isn't.
**NeoTrix Implication**: The NT-ACT production orchestrator must implement efficient data pipeline patterns (overlapping I/O, asynchronous augmentation, mixed-precision training) as first-class citizen.

### D602-15: Dual-System Architecture as Default Skeleton
**Source**: Robot Foundation Models VLA 2026 survey (pdpspectra.com, 2026-06-07)
**Finding**: The 2026 convergence point is dual-system design: System 2 (VLM at 7-9Hz for reasoning) + System 1 (visuomotor policy at 200Hz for control). Figure Helix, π0, and Gemini Robotics-ER all implement this split.
**Positive Contribution**: Validates that the speed-accuracy tradeoff in embodied systems necessitates architectural separation of reasoning and control. The latent vector conditioning between systems is the key interface.
**NEW Defect vs B601**: Reveals a **clock domain crossing defect**: the 7-9Hz reasoning system and 200Hz control system must synchronize through a latent vector, but the latent vector's semantic content degrades between updates. The control system is executing decisions based on potentially stale latent conditioning. This is analogous to clock domain crossing in hardware — metastability at the interface.
**NeoTrix Implication**: The PerceptionBridge must implement **latent freshness tracking** — the control system should know the age of its conditioning latent and adjust confidence accordingly.

---

## 4. Summary: What's NEW vs Batch 601

| # | Defect | Source | Unique New Element |
|---|--------|--------|-------------------|
| D602-1 | Causal memory injection poison | Zeva | Frozen model + injected context = causal trust without verification |
| D602-2 | Execution proposal temporal validity decay | EmbodiedSkills | Proposal valid at gen time, invalid at verify time |
| D602-3 | Video-latent compression collapse | GE | Contact-critical spatial info lost in compression |
| D602-4 | Wrench prediction feedback convergence | Facet-0 | Coupled prediction suppresses contact mode diversity |
| D602-5 | Dynamics prior temporal scale mismatch | DyPES-VLA | Shared priors informative but not actionable at control frequency |
| D602-6 | Lifecycle coupling error cascading | Arcadia | Compound error rate ~73% across four coupled stages |
| D602-7 | Representation sufficiency gap | GIFT | Visual features are control-poor despite being information-rich |
| D602-8 | History-free abstraction correction failure | ASTRA | Partial observability from abstraction requires temporal memory |
| D602-9 | Geometry-only invariance limitation | GeCo-SRT | Non-geometric dynamics gaps (friction, deformation) unaddressed |
| D602-10 | Co-training synergy validation | HyperSim | FS+adversarial+co-training = strongest combination |
| D602-11 | Safety-accuracy coupling | Provably Safe | Tighter safety constraints require better simulator accuracy |
| D602-12 | Cascading uncertainty in test-time search | τ0-VLA | Uncertainty about uncertainty in long-horizon search |
| D602-13 | Compositionality ceiling in WAMs | Riemann-1.0 | 99% LIBERO vs 62.6% RoboCasa = compositionality gap |
| D602-14 | Training throughput bottleneck | LingBot-VLA | Architectures solved, engineering not |
| D602-15 | Clock domain crossing at latent interface | VLA survey | Stale latent conditioning in dual-system architectures |

---

## 5. Sources Cited

1. Zeva: In-Context Causal Learning — arXiv:2608.30880 (2026-08-31)
2. EmbodiedSkills — arXiv:2609.01281 (2026-09-01)
3. Genie Envisioner (GE) — ICLR 2026 proceedings
4. Facet-0 — arXiv:2609.01596 (2026-09-01)
5. DyPES-VLA — arXiv:2608.06374 (2026)
6. Arcadia — CVPR 2026 papers
7. GIFT — arXiv:2609.04193 (2026-09-03)
8. ASTRA (abstract sim2real) — arXiv:2604.15289
9. GeCo-SRT — CVPR 2026 papers
10. HyperSim — arXiv:2605.26638
11. Provably Safe Sim-to-Real — arXiv:2609.01418 (2026-09-01)
12. τ0-VLA — arXiv:2608.16885 (2026)
13. Riemann-1.0 — arXiv:2608.27033 (2026-08-27)
14. LingBot-VLA — arXiv:2601.18692 (2026)
15. Robot Foundation Models VLA 2026 survey — pdpspectra.com (2026-06-07)
16. Foundation Models in Robotics review — arXiv:2604.15395 (2026-04-16)
17. The Reality Gap in Robotics survey — rpg.ifi.uzh.ch (AR25)
18. simDP — MDPI Sensors 26(13) (2026-06-27)
19. Real2Sim2Real for Torque-Controlled Robots — arXiv:2608.22629 (2026-08-23)
20. RT-2 — robotics-transformer2.github.io (2023, baseline reference)
21. Sim-and-Real Co-Training Mechanistic Analysis — arXiv:2604.13645

# Iteration Batch 364 — Embodied AI × Social Simulation × Role-Playing Research

**Date**: 2026-09-06
**Domains**: Embodied AI, Social Simulation, Role-Playing AI
**Cycle**: 364

---

## Sources Cited

### Embodied AI
1. **Super Star** (arXiv:2608.24909, ACM MM 2026) — Streaming real-time interactive agents for digital humans with co-speech gesture generation, self-evolving training loop from user feedback
2. **ViBES** (CVPR 2026, Stanford) — Speech-Language-Behavior model with MoME architecture for conversational 3D agents with behavior intelligence and mixed-initiative interaction
3. **Mio** (arXiv:2512.13674v2, 2026) — Multimodal Interactive Omni-Avatar: Thinker/Talker/Face/Body/Renderer modules, hierarchical memory, diegetic knowledge graph, self-evolution via Deep Persona Alignment
4. **SentiAvatar** (arXiv:2604.02908v1) — Plan-then-infill architecture with Motion Foundation Model (200K+ sequences), 0.3s latency for 6s output
5. **Visually-grounded Humanoid Agents** (arXiv:2604.08509) — Two-layer (world-agent) paradigm with egocentric RGB-D perception and spatially-aware embodied planning
6. **Archon** (CVPR 2026) — Unified multimodal model across 7 modalities (text, audio, motion, semantic, color, image, video) with "Thinking in Modality" strategy
7. **Systematic Review: From Pre-Rendered to Autonomous** (MDPI, 2026) — 48-study PRISMA review identifying AI-Agency Paradox, Dynamic Uncanny Valley, hardware-software co-design criticality

### Social Simulation
8. **GASim** (ACL 2026) — Graph-accelerated hybrid framework: Graph-Optimized Memory + Graph Message Passing + Entropy-Driven Grouping, 9.94× speedup
9. **Social Simulations: ABM to Digital Twins** (arXiv:2607.13693v1) — Survey: LLM-enhanced ABM → Social Digital Twins, validation gap at structural/behavioral/outcome levels
10. **TopoSim** (arXiv:2604.18011) — Topology-aware execution layer using network structure as execution prior, 40-90% token reduction with preserved fidelity
11. **SocaSim** (arXiv:2607.06080) — Putnam's Social Capital Theory with LLM agents: trust dynamics, norm propagation, network evolution, Pearson r=0.974 human-agent alignment
12. **GRAPHIA** (ACL 2026) — Social graph data as RL supervision via GNN-based structural rewards, +35.98% structural similarity
13. **CAREB-MAS** (ACL Findings 2026) — Emergent relational order: emotion-ethics-belief chain produces Differential Order phenomena (guanxi, authority stratification)
14. **AgentSociety** (Frontiers of Information, 2026) — 10K+ agents, 5M interactions, large-scale social simulation testbed
15. **Agentopia** (arXiv:2606.07513) — Long-term life simulation (10 years, 100 agents), life reward training via rejection sampling, +15.6% on role-playing benchmarks
16. **AIvilization v0** (arXiv:2602.10429) — Large-scale artificial social simulation with unified agent architecture and adaptive profiles

### Role-Playing AI
17. **PersonaForge** (ACL Findings 2026) — Psychology-grounded dual-process: Big Five + Vaillant defense mechanisms, Inner Monologue workspace, 75% drift reduction, 96% performance at 13.4% token overhead
18. **Dynamic Persona Coherence** (ACL 2026) — L/M/S Psychological State Model: Identity-Layer Stability vs Adaptive-Layer Appropriateness, S-score +71% improvement (0.57→0.98)
19. **ThinkPersona** (ACL 2026) — Persona Graphs from 1,201 real-world interviews, QRA (Question-Reasoning-Answer) training paradigm
20. **PD-LLM** (ACL 2026) — Trait Activation Theory: Bipolar Latent Decomposition (bidirectional LoRA adapters), DIAMONDS taxonomy for situation-aware modulation, 77.73% aggregate win rate
21. **DREAM** (arXiv:2608.05170) — Event-aware Memory Graph (EMG) with temporal-causal structure, reduces future knowledge leakage to <10%, +72% causal consistency
22. **Psy-CoT** (arXiv:2606.27025v1) — Psychology-grounded chain-of-thought: Interaction Perception → Psychological Empathy → Logical Construction; RAPO: role-aware gradient weighting
23. **PHASE-Tree** (arXiv:2608.06975v1) — Multi-timescale character-state tree: immutable identity root + mutable persona/session/moment layers, +19.7% character-level scores

---

## Defects Found

### DEFECT-1: NT-PHYSICAL Lacks Streaming Embodiment Architecture
**Severity**: HIGH
**Evidence**: CONTEXT.md defines NT-PHYSICAL as "sensors, motors, safety kernel, power management, body schema" (line 30). No streaming pipeline, no co-speech gesture generation, no multimodal synchronization.
**Gap**: Super Star (ACM MM 2026), ViBES (CVPR 2026), and SentiAvatar all demonstrate that real-time interactive digital humans require: (1) causal autoregressive models for streaming motion generation without future-speech access, (2) MoME (Mixture-of-Modality-Experts) architectures for speech-language-behavior coordination, (3) <0.3s end-to-end latency. NeoTrix's NT-PHYSICAL has no specification for streaming embodiment or multimodal body generation.
**Impact**: Cannot support real-time avatar interaction, virtual character deployment, or embodied agent scenarios.

### DEFECT-2: NT-FEEL Emotion Engine Is Stateless Per-Turn
**Severity**: HIGH
**Evidence**: EmotionLabel (CONTEXT.md line 71) is an enum with 11 variants. No cumulative psychological state, no stress accumulation model, no distinction between transient affect and stable traits.
**Gap**: Dynamic Persona Coherence (ACL 2026) demonstrates that persona drift is primarily caused by failure to model cumulative stress — S-score improves from 0.57 to 0.98 with L/M/S state tracking. PersonaForge shows defense mechanisms require Inner Monologue cognitive workspace. NeoTrix's emotion system resets each turn with no memory of prior emotional trajectory.
**Impact**: Agents exhibit "robotic repetition" under sustained interaction or "catastrophic drift" under stress — both failure modes identified in literature.

### DEFECT-3: No Topology-Aware Social Simulation Framework
**Severity**: MEDIUM
**Evidence**: NT-WORLD defines "UnifiedCrawler, fetchers, parsers, classifiers, content extraction" (CONTEXT.md line 26). No social graph representation, no topology-aware execution, no agent-based modeling infrastructure.
**Gap**: TopoSim (arXiv:2604.18011) shows network topology as execution prior reduces LLM inference 40-90% while preserving fidelity. GASim (ACL 2026) achieves 9.94× speedup via graph-accelerated hybrid methods. GRAPHIA demonstrates social graphs as RL supervision signals. NeoTrix has no social graph abstraction or topology-aware scheduling.
**Impact**: Cannot simulate social dynamics, model community formation, or run large-scale agent societies efficiently.

### DEFECT-4: Missing Dual-Process Personality Architecture
**Severity**: MEDIUM
**Evidence**: NT-CORE has E8 reasoning and GWT attention routing (CONTEXT.md lines 11-12). No System 1/System 2 cognitive architecture for personality expression. AttentionManager routes between modes but doesn't model cognitive interference.
**Gap**: PersonaForge (ACL Findings 2026) demonstrates that high-dimensional personality constraints create "production interference" requiring a cognitive workspace (Inner Monologue) to resolve. Selective dual-process activation achieves 96% of full performance at 13.4% token overhead. NeoTrix's dual specialization (Weapon Set I/II) is task-routing, not personality-cognitive routing.
**Impact**: Cannot maintain personality consistency under complex social scenarios where trait conflicts arise.

### DEFECT-5: No Event-Aware Memory Graph for Narrative Coherence
**Severity**: HIGH
**Evidence**: NT-MEMORY stores KB nodes/edges with FTS5 search (CONTEXT.md line 25). No temporal ordering of events, no causal dependency tracking, no narrative structure.
**Gap**: DREAM (arXiv:2608.05170) shows Event-aware Memory Graphs reduce future knowledge leakage to <10% and improve causal consistency by 72%. ThinkPersona (ACL 2026) demonstrates Persona Graphs from real-world interviews with interconnected biographical facts, values, and events. NeoTrix's KB is flat entity-relation storage without temporal-causal structure.
**Impact**: Agents cannot maintain narrative coherence across long interactions, cannot prevent information leakage from future events, cannot reason about character development.

### DEFECT-6: Missing Trait Activation Theory Integration
**Severity**: MEDIUM
**Evidence**: NT-FEEL defines EmotionEngine (CONTEXT.md line 31) but no situation-aware trait modulation. No mapping from social context to personality expression intensity.
**Gap**: PD-LLM (ACL 2026) implements Trait Activation Theory: in strong situations (formal meetings), social norms suppress trait expression; in weak situations (casual gatherings), personality-driven behaviors are freely expressed. Bipolar Latent Decomposition + DIAMONDS taxonomy achieves 77.73% aggregate win rate. NeoTrix has no mechanism to suppress/amplify personality traits based on social context.
**Impact**: Agents express same personality intensity regardless of social setting, producing unrealistic behavior.

### DEFECT-7: No Life Reward / Well-Being Metric
**Severity**: LOW
**Evidence**: NT-CORE SelfModel tracks "capability/uncertainty/fatigue" and "identity/goals/weights" (CONTEXT.md lines 101-103). No social well-being metric, no life satisfaction measure, no social standing evaluation.
**Gap**: Agentopia (arXiv:2606.07513) defines life reward modeling social standing, subjective fulfillment, and economic status. Life reward training via rejection sampling improves LLM anthropomorphism by +15.6% and character fidelity by +16.4%. NeoTrix has no equivalent metric for evaluating agent social flourishing.
**Impact**: Cannot optimize agents for social competence or evaluate social interaction quality.

### DEFECT-8: No Multi-Timescale Character State Evolution
**Severity**: MEDIUM
**Evidence**: SelfModel types are static structural, dynamic performance, and value function (CONTEXT.md lines 101-103). No hierarchical state with immutable identity + mutable momentary expression.
**Gap**: PHASE-Tree (arXiv:2608.06975v1) demonstrates multi-timescale character-state tree with immutable identity root + mutable persona/session/moment layers, enabling localized updates without destabilizing unchanged traits. +19.7% character-level scores. NeoTrix's SelfModel doesn't support localized state updates.
**Impact**: Any persona update risks destabilizing the entire character representation.

### DEFECT-9: No Social Digital Twin Validation Framework
**Severity**: LOW
**Evidence**: NT-GOVERNANCE has "architecture arbitration" (CONTEXT.md line 35). No validation framework for social simulation fidelity at structural, behavioral, or outcome levels.
**Gap**: Social simulation survey (arXiv:2607.13693v1) identifies three-level validation: structural (does architecture reflect real system?), behavioral (do agents behave correctly?), outcome (do aggregate patterns match?). AgentSociety validates against real-world experiments on polarization, UBI, hurricanes. NeoTrix has no social simulation validation methodology.
**Impact**: Cannot verify that social simulations produce meaningful or accurate results.

### DEFECT-10: No Unified Cross-Modal Attention Architecture
**Severity**: MEDIUM
**Evidence**: Modules are siloed: NT-IO (speech/text), NT-PHYSICAL (motion), NT-FEEL (emotion), NT-WORLD (perception). No shared representation space or cross-modal attention mechanism.
**Gap**: Archon (CVPR 2026) unifies 7 modalities with modality-specific tokenizers + native autoregressive model + "Thinking in Modality" strategy for cross-modal reasoning. ViBES demonstrates MoME architecture with cross-expert attention. NeoTrix's GWT broadcasts information but doesn't perform cross-modal attention fusion.
**Impact**: Speech, motion, emotion, and perception operate independently without mutual influence, producing fragmented behavior.

---

## Suggestions

### S1: Streaming Embodiment Pipeline (→ NT-PHYSICAL)
Add to NT-PHYSICAL: `StreamingEmbodimentPipeline` trait with:
- `causal_motion_generator`: Autoregressive model that predicts body motion from streaming audio + motion history (no future access)
- `multimodal_synchronizer`: Coordinates speech, gesture, facial expression timing
- `latency_budget`: Hard constraint <300ms for 6s output (SentiAvatar benchmark)
- `self_evolving_loop`: User feedback integration for online adaptation (Super Star pattern)

### S2: L/M/S Psychological State Model (→ NT-FEEL)
Extend EmotionLabel with hierarchical state:
- **L-State (Long-term)**: Stable personality traits (Big Five scores), identity anchors — immutable across interactions
- **M-State (Medium-term)**: Accumulated stress, meaning-making, relationship sentiment — updated per session
- **S-State (Short-term)**: Transient affect, immediate emotional reaction — updated per turn
- **Cumulative Trajectory**: Track how M-state evolves from interaction history, preventing both robotic repetition and catastrophic drift

### S3: Social Graph Abstraction (→ NT-WORLD)
Add `SocialGraphLayer` to NT-WORLD:
- `SocialGraph`: Directed weighted graph with agent nodes, relationship edges (trust, influence, kinship)
- `TopologyAwareScheduler`: Group structurally similar agents for shared LLM inference (TopoSim pattern)
- `InfluencePropagator`: Graph attention network for message passing across social connections
- `EntropyDrivenGrouping`: Identify emergent core agents in information-diverse neighborhoods (GASim pattern)

### S4: Dual-Process Personality Engine (→ NT-CORE)
Extend AttentionManager with personality-cognitive routing:
- **System 1 (Fast)**: Default personality expression — habitual responses, style, mannerisms
- **System 2 (Slow)**: Inner Monologue workspace — activated when trait conflicts detected or high-stakes social decisions
- **Trigger Mechanism**: Rule-based (85.6% F1) or learnable (90.2% F1) activation based on conversation complexity
- **Token Budget**: Target 13.4% overhead for 96% of full dual-process performance

### S5: Event-Aware Memory Graph (→ NT-MEMORY)
Add `EventMemoryGraph` to NT-MEMORY:
- **Event Nodes**: Temporal ordering with timestamps, causal dependencies (caused_by, led_to edges)
- **Character Profiles**: Dual-granularity — stable traits from EMG + dynamic state from recent event chain
- **Temporal Retrieval**: Only retrieve events before current timestamp (prevents future knowledge leakage)
- **Causal Chain Retrieval**: Multi-hop traversal along causal edges for deep character reasoning

### S6: Situation-Aware Trait Modulation (→ NT-FEEL)
Add `TraitActivationModule` to NT-FEEL:
- **DIAMONDS Taxonomy**: Map social situations to 8 dimensions (Duty, Intellectual, Adversity, Mating, Negativity, Deception, Politics, Sociality)
- **Bipolar Expression**: For each Big Five trait, model both poles (e.g., Openness-Closed, Agreeableness-Antagonism)
- **Situation Suppression**: In strong situations (formal contexts), suppress trait expression; in weak situations, amplify
- **Context-Aware Routing**: Feed DIAMONDS classification to EmotionEngine for personality-appropriate emotional response

### S7: Life Reward Metric (→ NT-CORE)
Add `LifeReward` to SelfModel:
- **Social Standing**: Reputation score from social graph interactions
- **Subjective Fulfillment**: Goal achievement rate + meaning-making satisfaction
- **Economic Status**: Resource accumulation and exchange efficiency
- **Composite Well-Being**: Weighted sum for optimization target during agent evolution

### S8: Multi-Timescale State Tree (→ NT-CORE SelfModel)
Restructure SelfModel with PHASE-Tree pattern:
- **Identity Root** (immutable): Core values, personality baseline, character history
- **Persona Layer** (cross-episode): Evolving beliefs, relationship maps, accumulated experiences
- **Session Layer** (within-episode): Current goals, active concerns, mood trajectory
- **Moment Layer** (per-turn): Immediate affect, attention focus, micro-expressions
- **Localized Updates**: Each mutable layer addressable for independent modification without identity destabilization

### S9: Social Simulation Validation Protocol (→ NT-GOVERNANCE)
Add `SocialSimValidator` to NT-GOVERNANCE:
- **Structural Validation**: Verify agent population matches target demographics, network topology matches real patterns
- **Behavioral Validation**: Compare agent action distributions against empirical data (surveys, experiments)
- **Outcome Validation**: Match aggregate patterns (polarization, cooperation rates, opinion distributions) to real-world benchmarks
- **Counterfactual Testing**: Run perturbation experiments to verify causal reasoning

### S10: Cross-Modal Attention Fusion (→ NT-CORE GWT)
Extend GWT with cross-modal attention:
- **Modality-Specific Tokenizers**: Encode speech, motion, emotion, perception into shared token space
- **Cross-Expert Attention**: Each modality attends to all others via sparse attention (Archon pattern)
- **Thinking in Modality**: For complex cross-modal tasks, decompose into stepwise generation across modalities
- **Shared Representation Space**: Common embedding dimension for all modalities enabling mutual influence

---

## Defect Summary

| # | Defect | Severity | Domain | Suggestion |
|---|--------|----------|--------|------------|
| 1 | NT-PHYSICAL lacks streaming embodiment | HIGH | Embodied AI | S1 |
| 2 | NT-FEEL is stateless per-turn | HIGH | Role-Playing | S2 |
| 3 | No topology-aware social simulation | MEDIUM | Social Sim | S3 |
| 4 | Missing dual-process personality | MEDIUM | Role-Playing | S4 |
| 5 | No event-aware memory graph | HIGH | Role-Playing | S5 |
| 6 | Missing trait activation theory | MEDIUM | Role-Playing | S6 |
| 7 | No life reward metric | LOW | Social Sim | S7 |
| 8 | No multi-timescale character state | MEDIUM | Role-Playing | S8 |
| 9 | No social sim validation framework | LOW | Social Sim | S9 |
| 10 | No cross-modal attention fusion | MEDIUM | Embodied AI | S10 |

**High-severity defects**: 3 (DEFECT-1, DEFECT-2, DEFECT-5)
**Medium-severity defects**: 5 (DEFECT-3, DEFECT-4, DEFECT-6, DEFECT-8, DEFECT-10)
**Low-severity defects**: 2 (DEFECT-7, DEFECT-9)

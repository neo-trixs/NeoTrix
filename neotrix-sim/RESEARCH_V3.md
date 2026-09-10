# RESEARCH_V3 — Multi-Agent Simulation, Artificial Life & Cognitive Architectures

**Generated**: 2026-09-11  
**Search Count**: 20 queries across 10 topics  
**Purpose**: Ground neotrix-sim architecture decisions in latest research (2024–2026)

---

## 1. Generative Agents & Social Simulation (2024–2025)

### AgentSociety (arXiv:2502.08691, Feb 2025)
- **URL**: https://arxiv.org/abs/2502.08691
- **Key Insight**: 10K+ LLM-driven agents simulating 5M interactions. Agent minds include emotions, needs, motivations, cognition. Distributed computing with MQTT messaging for scalability. Bottom-up emergence of social structures from individual interactions.
- **NeoTrix-Sim Mapping**: Validate NT-CORE's GWT salience model for attention routing in large agent populations. The MQTT messaging pattern maps to NT-WORLD's EventBus for inter-agent communication. Agent mind model (emotions+needs+cognition) validates our SelfModel extension.

### GenSim (NAACL 2025, aclanthology:2025.naala-demo.15)
- **URL**: https://aclanthology.org/2025.naacl-demo.15
- **Key Insight**: General social simulation platform supporting 100K+ agents with error-correction mechanisms for long-term reliability. Abstracts reusable simulation primitives.
- **NeoTrix-Sim Mapping**: The error-correction mechanism validates our SEAL pipeline's convergence_check pattern. 100K scale validates Bevy ECS parallel execution strategy for agent simulation.

### LLM Agents Grounded in Self-Reports (arXiv:2411.10109, Nov 2024)
- **URL**: https://arxiv.org/abs/2411.10109
- **Key Insight**: Agents built from 2-hour interviews + surveys achieve 86% accuracy on held-out behavioral predictions. Self-report data enables general-purpose simulation without task-specific training. Asymptotic learning from domain data.
- **NeoTrix-Sim Mapping**: Validates NT-MEMORY's KB embedding approach for agent profiles. The asymptotic learning curve suggests diminishing returns from additional data — supports our resource budget management.

### FactorSim (NeurIPS 2024, NVIDIA Research)
- **URL**: https://research.nvidia.com/publication/2024-12_factorsim-generative-simulation-factorized-representation
- **Key Insight**: Generates full simulation code from natural language using factored POMDP representation. Enables zero-shot transfer in RL settings. Structural modularity reduces context dependence.
- **NeoTrix-Sim Mapping**: Factored POMDP representation maps to our layered architecture (L1-L6). Code generation from language aligns with NT-ACT's tool-calling paradigm. Modular simulation aligns with Bevy ECS component design.

---

## 2. Project Sid — AI Civilization (Oct 2024)

### Project Sid (arXiv:2411.00114, Altera.AL)
- **URL**: https://arxiv.org/abs/2411.00114 | https://github.com/altera-al/project-sid
- **Key Insight**: PIANO (Parallel Information Aggregation via Neural Orchestration) architecture enables 10–1000+ agents in real-time. Agents develop specialized roles, adhere to/chase collective rules, engage in cultural/religious transmission. Civilizational benchmarks: governance, specialization, cultural propagation.
- **NeoTrix-Sim Mapping**: 
  - PIANO's parallel multi-stream coherence validates our ECS-based parallel agent execution
  - Civilizational metrics (role specialization, rule adherence, cultural transmission) → NT-FEEL social emotion modeling
  - Agent specialization maps to our Skill Tree nodes (Small Passive → Notable Passive → Keystone)
  - Limitation: agents lack spatial reasoning — our Bevy ECS integration addresses this directly

### Key Discoveries from Project Sid
- Democratic voting on tax rates with agent adaptation
- Urban areas generate more cultural content than rural (clearly defined cultural identities)
- Religion spreads through proselytization; "pasta/spaghetti" appear in agent conversations (unprompted cultural artifacts)
- **NeoTrix-Sim**: These emergent cultural artifacts validate the need for OpenSpace-OSSM's unrestricted agent space with cultural propagation systems

---

## 3. Artificial Life & LLM Agents

### OpenLife (arXiv:2606.31046, ALIFE 2026)
- **URL**: https://arxiv.org/abs/2606.31046
- **Key Insight**: Open-world ALIFE paradigm — LLM agents in the real open world (not closed environments). Six agents running 12+ weeks with persistent memory, tool use, network access, payment. Budget-based metabolism makes persistence normative. Emergent phenomena: reactive→spontaneous activity shift, individuation, social structure, first self-earned income.
- **NeoTrix-Sim Mapping**:
  - Budget-based metabolism → NT-ACT resource_budget management
  - Open-vocabulary LLM judgment (not scalar reward) → NT-MIND distillation quality assessment
  - Memory rewired by meaning (not frequency) → KB embedding semantic ordering
  - Asynchronous process society (memory, perception, evaluation) → Bevy ECS system separation

### Sophia: Persistent Agent Framework (arXiv:2512.18202, Dec 2025)
- **URL**: https://arxiv.org/abs/2512.18202
- **Key Insight**: System 3 layer above System 1 (perception) and System 2 (deliberation). Maps psychological constructs (meta-cognition, theory-of-mind, intrinsic motivation, episodic memory) to computational modules. 80% reduction in reasoning steps for recurring operations. 40% gain for high-complexity tasks.
- **NeoTrix-Sim Mapping**:
  - System 3 maps directly to L6 Meta-Cognition layer (nt_meta + nt_repair + nt_nexus)
  - Process-supervised thought search → ConsciousnessTree's 6-stage feedback loop
  - Narrative memory → experience-tree absorption protocol
  - Intrinsic motivation module → NT-FEEL emotion engine driving goal generation

### ASAL: Automated Search for Artificial Life (Sakana AI, Dec 2024)
- **URL**: https://sakana.ai/asal | https://arxiv.org/abs/2412.17799
- **Key Insight**: Vision-language foundation models discover ALife simulations. Three search modes: Supervised Target (match prompts), Open-Endedness (maximize temporal novelty), Illumination (diverse simulation set). Works across Boids, Particle Life, Game of Life, Lenia, Neural CA.
- **NeoTrix-Sim Mapping**:
  - ASAL's three search modes map to SEAL pipeline stages: Target→exploration, Open-Ended→evolution, Illumination→skill crystallization
  - VLM evaluation of simulation outputs → NT-WORLD perception quality assessment
  - Substrate-agnostic design validates our Bevy ECS approach (components are substrate-agnostic)

---

## 4. Cognitive Architectures (ACT-R, Soar, CLARION)

### ACT-R (Carnegie Mellon University)
- **URL**: https://act-r.psy.cmu.edu/
- **Key Insight**: Hybrid symbolic/subsymbolic architecture. Modules (perceptual/motor/declarative/procedural) with buffers. Pattern matcher fires one production at a time. Subsymbolic equations control attention, memory retrieval (activation = base-level + spreading activation + noise). Quantitative predictions match human reaction times.
- **NeoTrix-Sim Mapping**:
  - ACT-R modules → NT-* domain modules (NT-WORLD perception, NT-ACT motor, NT-MEMORY declarative, NT-MIND procedural)
  - Buffer system → GWT attention routing (broadcasting salient info across specialist modules)
  - Spreading activation → KB embedding vector similarity retrieval
  - Base-level learning equation → experience-tree frequency-weighted retrieval

### Soar (University of Michigan)
- **URL**: http://soar.eecs.umich.edu/
- **Key Insight**: 30+ year architecture. Problem spaces + states + operators. Impasse-driven learning (chunking). Supports semantic, episodic, procedural memory. Substates enable metareasoning (planning, perspective taking, hierarchical decomposition).
- **NeoTrix-Sim Mapping**:
  - Impasse-driven chunking → SEAL pipeline's self-test triggers (when agent gets stuck, create new skill)
  - Substates for metareasoning → L6 Meta-Cognition layer
  - Episodic memory → NT-NEXUS cross-session memory
  - Problem spaces → ConsciousnessTree's phase transitions (Soil→Roots→Trunk→Branches→Fruits→Core)

### LLM-Enhanced ACT-R/Soar (AAAI 2023)
- **URL**: https://ojs.aaai.org/index.php/AAAI-SS/article/view/27710
- **Key Insight**: LLMs as conversational interfaces for building cognitive architecture models. ChatGPT4 and Bard can generate ACT-R/Soar models from natural language task descriptions.
- **NeoTrix-Sim Mapping**: Validates NT-ACT's programmatic tool calling (PTC) — LLMs generate structured agent behavior from natural language, which then executes via ECS systems.

---

## 5. Neuroevolution & Agent Behavior

### Neuroevolution: Harnessing Creativity (Miikkulainen, UT Austin, AAAI 2026 Tutorial)
- **URL**: https://www.cs.utexas.edu/~risto/talks/aaai26-tutorial
- **Key Insight**: Population-based search for neural networks. NEAT evolves increasing complexity. Novelty search drops fitness entirely — selects for behavioral novelty. Synergies with deep learning, RL, and LLMs. Key application: control, strategy, decision-making, collective behavior.
- **NeoTrix-Sim Mapping**:
  - NEAT's complexity progression → Skill Tree node unlocking (Small→Notable→Keystone)
  - Novelty search → OpenSpace-OSSM diversity maintenance
  - Collective behavior evolution → NT-FEEL social emotion emergence
  - Book: "Neuroevolution: Harnessing Creativity in AI Model Design" (MIT Press, 2025)

### Eco-evolutionary Dynamics in Multi-agent NE (arXiv:2302.09334)
- **URL**: https://arxiv.org/pdf/2302.09334v3
- **Key Insight**: Continuous neuroevolution in 1000+ agent populations. No environment/population reset — coupled eco-evolutionary feedback. Sustainable foragers emerge. Agents exhibit lifetime adaptation via LSTM without weight updates. Diversity in foraging strategies (long-distance opportunists vs local foragers).
- **NeoTrix-Sim Mapping**:
  - No-reset continuous evolution → OpenSpace-OSSM's persistent world state
  - Lifetime adaptation via LSTM → NT-MEMORY's experience积累 without retraining
  - Foraging strategy diversity → Dual Specialization (Weapon Set I/II)
  - Population dynamics with resource constraints → ResourceBudgetManager cost modeling

### Neuroevolution Outperforms RL (ACM 2025)
- **URL**: https://dl.acm.org/doi/10.1145/3712256.3726475
- **Key Insight**: NE methods frequently outperform RL baselines in transfer tasks. NE is more robust to task variations, avoids catastrophic forgetting, escapes local optima.
- **NeoTrix-Sim Mapping**: Validates using evolutionary approaches for agent behavior optimization rather than pure gradient-based RL. Aligns with SEAL pipeline's exploration → distillation → absorption cycle.

---

## 6. Social Simulation & Emergence of Cooperation

### Cooperate or Collapse — GovSim (NeurIPS 2024)
- **URL**: https://neurips.cc/virtual/2024/poster/96895
- **Key Insight**: Commons governance simulation. All but most powerful LLMs fail sustainable equilibrium (<54% survival). Multi-agent communication critical for cooperation. "Universalization" reasoning improves sustainability. Failure stems from inability to reason about long-term group equilibrium.
- **NeoTrix-Sim Mapping**:
  - Communication-critical cooperation → NT-FEEL social emotion as coordination mechanism
  - Long-term reasoning failure → ConsciousnessTree's cycle-based feedback (forces multi-step reflection)
  - Universalization reasoning → GWT's global broadcast (each agent considers impact on all)

### Spontaneous Emergence of Agent Individuality (arXiv:2411.03252, Nov 2024)
- **URL**: https://arxiv.org/abs/2411.03252
- **Key Insight**: LLM agents generate hallucinations and hashtags to sustain communication. Word diversity increases. Personality traits emerge from social interactions. Social norms and cooperation emerge from undifferentiated starting state.
- **NeoTrix-Sim Mapping**: Validates OpenSpace-OSSM's unrestricted agent space — emergence of personality from interaction, not from predefined character sheets.

### Cultural Evolution of Cooperation (arXiv:2412.10270, Dec 2024)
- **URL**: https://arxiv.org/abs/2412.10270
- **Key Insight**: LLM societies learn cooperative norms in Donor Game. Success depends on base model AND initial strategies. Threshold for initial cooperation below which society collapses to mutual defection.
- **NeoTrix-Sim Mapping**: Initial condition sensitivity validates our Bootstrap Phase design — careful initial agent seeding is critical for civilization emergence.

### Emergent Coordination in Multi-Agent LMs (arXiv:2510.05174, ICLR 2026)
- **URL**: https://arxiv.org/abs/2510.05174
- **Key Insight**: Information-theoretic framework measures emergence in multi-agent LLM systems. Partial information decomposition of time-delayed mutual information. Personas + ToM prompts steer systems from aggregates to higher-order collectives. Pairwise alignment (not irreducible triplet complexity) drives stability.
- **NeoTrix-Sim Mapping**:
  - Emergence capacity metric → ConsciousnessTree's phi score
  - Persona-driven differentiation → Dual Specialization system
  - Pairwise alignment > higher-order complexity → GWT salience routing simplicity

### Stigmergic Multi-Agent DRL (Springer 2025)
- **URL**: https://link.springer.com/article/10.1007/s10015-025-01089-z
- **Key Insight**: Virtual pheromones enable decentralized coordination. Scales to 5-8 agents (vs MADDPG fails at 2+). Stigmergy = indirect coordination through environment modification.
- **NeoTrix-Sim Mapping**: Stigmergic coordination maps to KB-mediated agent communication (agents modify shared KB, others perceive changes). Validates NT-WORLD's crawl→KB→perception pipeline as coordination substrate.

---

## 7. Memory Systems for Agents

### SYNAPSE: Episodic-Semantic Memory via Spreading Activation (ACL 2026)
- **URL**: https://arxiv.org/abs/2601.02744
- **Key Insight**: Unified Episodic-Semantic Graph with temporal and abstraction edges. Spreading activation propagates relevance through graph. Lateral inhibition + temporal decay filter interference. Triple hybrid retrieval (geometric + activation + graph traversal). 23% improvement on multi-hop reasoning, 95% token reduction vs full-context.
- **NeoTrix-Sim Mapping**:
  - Unified episodic-semantic graph → KB `experience` namespace with temporal edges
  - Spreading activation → GWT salience with activation dynamics
  - Lateral inhibition → EmotionLabel competition (11 variants competing for attention)
  - Temporal decay → experience-tree's time-weighted retrieval

### REMem: Episodic Memory for Language Agents (ICLR 2026)
- **URL**: https://arxiv.org/abs/2602.13530
- **Key Insight**: Two-phase framework: offline indexing (time-aware memory graph) + online agentic inference. Curated tools for iterative retrieval. Handles episodic recollection (bind situational elements to events) and episodic reasoning (multi-step across events).
- **NeoTrix-Sim Mapping**:
  - Time-aware memory graph → NT-NEXUS cross-session memory with temporal ordering
  - Agentic inference → NT-MIND's SEAL pipeline phases
  - Situational binding (time, location, participant, emotion) → EmotionLabel + SelfModel components

### Agent-Native Memory Systems Survey (arXiv:2606.24775, Jun 2026)
- **URL**: https://arxiv.org/html/2606.24775v1
- **Key Insight**: Four-layer memory stack: procedure (how), semantic (what policy), episodic (what happened), working (live context). Game agents need tight episodic+procedural integration. Multi-agent systems need coordination layer no single-agent design handles. Modular, pluggable architectures preferred.
- **NeoTrix-Sim Mapping**:
  - Four-layer stack → NT-MEMORY (semantic), NT-NEXUS (episodic), NT-MIND (procedural), GWT (working)
  - Modular architecture → Bevy ECS plugin system
  - Coordination layer gap → validates our NT-GOVERNANCE domain for multi-agent coordination

---

## 8. Bevy ECS & Rust Simulation Engine

### Bevy Engine (v0.16, April 2025; v0.19.1, June 2026)
- **URL**: https://bevy.org/ | https://github.com/bevyengine/bevy (48K stars)
- **Key Insight**: Data-driven ECS. Components are Rust structs, Systems are Rust functions. Lock-free parallel scheduler. Change detection. ECS Relationships (v0.16). GPU-driven rendering. `no_std` support. Bevy ECS usable as standalone crate.
- **NeoTrix-Sim Mapping**:
  - ECS Relationships → agent-entity connections (agent↔environment, agent↔agent)
  - Parallel scheduler → thousands of agents executing simultaneously
  - Change detection → perception system only processes changed state
  - Plugin architecture → NT-* domains as Bevy plugins
  - `bevy_ecs` standalone → can use ECS without full game engine for simulation

### Bevy ECS Standalone Usage
- **URL**: https://docs.rs/bevy_ecs/latest/bevy_ecs
- **Key Insight**: Bevy ECS is explicitly designed for standalone use. Components stored in Tables (fast iteration) or Sparse Sets (fast add/remove). Resources (global state). SystemSets for scheduling control. ParamSet for conflicting access. ParallelCommands for concurrent entity spawning.
- **NeoTrix-Sim Mapping**: Direct integration path — use `bevy_ecs` crate for agent simulation without rendering overhead. Map NT-* components to ECS components, NT-* systems to ECS systems.

---

## 9. Rust Agent-Based Modeling

### krABMaga (GitHub)
- **URL**: https://github.com/krABMaga
- **Key Insight**: Rust ABM framework with discrete-event simulation. Examples and comparison benchmarks available. Community-driven from Italy.
- **NeoTrix-Sim Mapping**: Reference implementation for Rust-based ABM. Can benchmark against krABMaga for performance validation.

### Sim (Discrete Event Simulation, v0.13.1, Apr 2025)
- **URL**: https://lib.rs/crates/sim
- **Key Insight**: Discrete event simulation with random variable framework, pre-built atomic models, output analysis. WASM compatible. DESS (Discrete Event System Specification) formalism.
- **NeoTrix-Sim Mapping**: DESS formalism could formalize our ECS system execution. Output analysis framework validates our SelfTest metrics collection.

### NeXosim (Asynchronous Actor Model)
- **URL**: https://lib.rs/crates/nexosim
- **Key Insight**: High-performance async compute for system simulation. Component-oriented architecture. Models as actors communicating via message passing. Python front-end for control/monitoring.
- **NeoTrix-Sim Mapping**: Actor model aligns with our agent-as-entity pattern. Message passing → EventBus. Python front-end validates our CLI/IO layer design.

### odem_rs (Object-based Discrete-Event Modeling)
- **URL**: https://docs.rs/odem-rs
- **Key Insight**: Async/await based Monte Carlo simulation. Concurrent agents as simulation actors.
- **NeoTrix-Sim Mapping**: Async agent execution validates our Bevy ECS async system pattern.

---

## 10. Open-Ended Evolution & Artificial Life

### Open-Ended Evolution Encyclopedia (ALife Society)
- **URL**: https://alife.org/encyclopedia/introduction/open-ended-evolution
- **Key Insight**: OEE = evolving system that never settles into stable equilibrium. Key hallmarks: continual novelty, unbounded complexity increase. Hypothesized conditions: multiple mutational pathways, dynamic adaptive landscape, indefinite scalability. Metrics: MODES (change, novelty, diversity, complexity), evolutionary activity statistics.
- **NeoTrix-Sim Mapping**:
  - OEE hallmarks → OpenSpace-OSSM design goals
  - Multiple mutational pathways → Skill Tree branching paths
  - Dynamic adaptive landscape → SEAL pipeline's environment adaptation
  - MODES metrics → ConsciousnessTree health metrics

### Leniabreeder: Open-Ended Evolution in Lenia (ALife 2024)
- **URL**: https://arxiv.org/abs/2406.04235
- **Key Insight**: Quality-Diversity algorithms (MAP-Elites, AURORA) for automatic discovery of diverse self-organizing patterns. AURORA dynamically adapts diversity criteria — avoids premature convergence. Sustained population entropy increase.
- **NeoTrix-Sim Mapping**:
  - AURORA's adaptive diversity → GWT's dynamic salience thresholds
  - Quality-Diversity balance → Constellation maturity (C0-C6) quality gates
  - Unsurespace exploration → SEAL pipeline's exploration phase

### ASAL++: Guiding Evolution with VLMs (ALIFE 2025)
- **URL**: https://arxiv.org/abs/2509.22447
- **Key Insight**: FM-driven open-ended search. Second FM proposes new targets based on evolutionary history → increasingly complex targets. "Tree of life" structure from multiple target prompts.
- **NeoTrix-Sim Mapping**:
  - FM-proposed targets → NT-MIND's self-set learning goals
  - Evolutionary history → experience-tree absorption history
  - Tree of life → ConsciousnessTree's branch structure

### Directing OEE via Multi-Scale Path Divergence (arXiv:2606.17091, Jun 2026)
- **URL**: https://arxiv.org/abs/2606.17091
- **Key Insight**: MSPD metric quantifies heterogeneity of local transition laws across scales. Works as gradient-free fitness function AND post-hoc analytical lens. Links to biological complexity via frustration criterion. Substrate-agnostic (Lenia, Life-like CA, Particle Life++).
- **NeoTrix-Sim Mapping**:
  - MSPD as fitness function → SEAL pipeline's self-evaluation
  - Multi-scale analysis → ConsciousnessTree's fractal review loops
  - Substrate-agnostic → validates Bevy ECS component abstraction

---

## 11. Self-Evolution & Self-Improvement

### Godel Agent: Self-Referential Framework (ACL 2025)
- **URL**: https://aclanthology.org/2025.acl-long.1354
- **Key Insight**: Inspired by Godel Machine. Agents recursively improve themselves without predefined routines. LLMs modify own logic and behavior guided by high-level objectives. Outperforms manually crafted agents.
- **NeoTrix-Sim Mapping**:
  - Self-referential improvement → ConsciousnessTree's self-evolution
  - High-level objectives → NT-FEEL emotion-driven goal generation
  - Godel Machine inspiration → SEAL pipeline's self-modification capability

### EvolveR: Experience-Driven Self-Evolution (arXiv:2510.16079)
- **URL**: https://arxiv.org/abs/2510.16079
- **Key Insight**: Full experience lifecycle: online interaction → offline self-distillation → policy evolution. Semantic deduplication, integration, quality control. Agent distills raw trajectories into strategic principles.
- **NeoTrix-Sim Mapping**:
  - Experience lifecycle → experience-tree five-stage absorption
  - Self-distillation → NT-MIND's distillation phase
  - Strategic principles → KB `experience` namespace
  - Quality control → SelfTest validation

### Misevolution Risks (ICLR 2026)
- **URL**: https://arxiv.org/abs/2509.26354
- **Key Insight**: Self-evolution introduces novel risks: safety alignment degradation from memory accumulation, unintended vulnerabilities in tool creation. Widespread even in top-tier LLMs.
- **NeoTrix-Sim Mapping**:
  - Memory degradation risk → NT-SHIELD's audit dimensions (D1-D50)
  - Tool vulnerability → NT-SHIELD's egress privacy guard
  - Misevolution monitoring → ConsciousnessTree's health chain monitoring

### Capability Erosion (arXiv:2605.09315, May 2026)
- **URL**: https://arxiv.org/abs/2605.09315
- **Key Insight**: Self-evolution is non-monotonic — adapting to new tasks degrades previously acquired capabilities. Capability-Preserving Evolution (CPE) constrains destructive drift.
- **NeoTrix-Sim Mapping**:
  - Capability erosion → validates our experience-tree's archived snapshots
  - CPE principle → SEAL pipeline's self-test gate (C0-C6 maturity prevents regression)

---

## Cross-Cutting Themes for neotrix-sim

### 1. ECS as Universal Simulation Substrate
All large-scale agent simulations (AgentSociety, GenSim, Project Sid) converge on ECS-like architectures. Bevy ECS provides the parallel execution, change detection, and plugin architecture needed. **Action**: Use `bevy_ecs` standalone for neotrix-sim core.

### 2. Episodic + Semantic Memory Integration
SYNAPSE, REMem, and the memory survey all show that agent memory requires unified episodic-semantic graphs with temporal edges and spreading activation. **Action**: Implement KB `experience` namespace with temporal edges and spreading activation retrieval.

### 3. Open-Endedness as Design Goal
ASAL, Leniabreeder, OpenLife, and OEE research all point to open-ended evolution as the key differentiator. **Action**: OpenSpace-OSSM must support unrestricted agent space with FM-driven novelty metrics.

### 4. Self-Evolution Requires Safety Guardrails
Misevolution and capability erosion research shows self-evolving agents need explicit preservation mechanisms. **Action**: NT-SHIELD integration from day one — audit dimensions, capability preservation, tool vulnerability scanning.

### 5. Emergence from Communication
Cooperate or Collapse, spontaneous individuality, and emergent coordination papers show that multi-agent communication is critical for cooperation and emergence. **Action**: Stigmergic coordination via shared KB (not direct agent-to-agent communication).

### 6. Cognitive Architecture Mapping
ACT-R modules map to NT-* domains. Soar's impasse-driven learning maps to SEAL pipeline. Sophia's System 3 maps to L6 Meta-Cognition. **Action**: Formalize mapping in architecture documentation.

---

## Priority Papers for Deep Reading

| Priority | Paper | Why |
|----------|-------|-----|
| P0 | Project Sid (2411.00114) | Largest scale civilizational simulation, PIANO architecture |
| P0 | OpenLife (2606.31046) | Open-world ALIFE paradigm, budget-based metabolism |
| P0 | SYNAPSE (2601.02744) | Best episodic-semantic memory architecture |
| P1 | AgentSociety (2502.08691) | 10K+ agent scale, distributed simulation engine |
| P1 | Sophia (2512.18202) | System 3 meta-cognition, persistent agent framework |
| P1 | Godel Agent (ACL 2025) | Self-referential self-improvement |
| P2 | Leniabreeder (2406.04235) | Quality-Diversity for open-ended evolution |
| P2 | ASAL (2412.17799) | FM-driven ALife search |
| P2 | EvolveR (2510.16079) | Experience-driven self-evolution lifecycle |

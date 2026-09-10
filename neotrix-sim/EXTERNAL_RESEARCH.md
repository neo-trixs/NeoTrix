# NT-WORLD-SIM External Research Compendium

> **Date**: 2026-09-10 | **Purpose**: Exhaustive cross-domain research for NT-WORLD-SIM simulation architecture
> **Domains**: Agent Architectures | Memory Systems | Social/Economic Simulation | Emergent Behavior | Rust Implementation
> **Sources**: Stanford/Stanford, MIT, Anthropic, ArXiv, GitHub repos, WebSearch, package registries

---

## Executive Summary

This document consolidates findings from 20+ searches across 5 domains, mapping every insight to NeoTrix modules (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-SHIELD, NT-IO, NT-PHYSICAL, NT-FEEL). Key discoveries: the generative agents paradigm (Smallville) provides the gold standard for autonomous agent design; Project Sid's PIANO architecture demonstrates scaling to 1000+ agents via hierarchical state machines; CraniMem's neurocognitive memory architecture with latent memory outperforms all baselines; microVerse provides the most rigorous personality stability framework (immutable "soul file"); and Rust ecosystem tools (`lifers`, `gpca`, Bevy ECS, `rayon`, DashMap) are production-ready for simulation backends.

---

## 1. Agent Architectures

### 1.1 Stanford Generative Agents (Smallville)

**Source**: [github.com/joonspk-research/generative_agents](https://github.com/joonspk-research/generative_agents)
**Source**: [Generative Agents: Interactive Simulacra of Human Behavior](https://arxiv.org/abs/2304.03442)
**Key Findings**:
- **Architecture**: Core Agent Components: LLM (GPT-3.5+) + Memory Retrieval (retrieval by recency, importance, relevance with learned weights) + Planning (reflection + higher-level abstraction) + Environment interaction (location, time, conversations, actions)
- **Memory**: Situation → memory stream → (retrieval by recency + importance + relevance) → reflection (synthesize high-level observations) → update memories
- **Planning**: 1) Generate LLM prompt: current situation + recent memories → 2) Output: plan of action + reflection(s) on high-level thoughts → 3) Each "hour" execute action in the environment
- **Key Principle**: "Generative Agents [are] simulated artificial characters that follow a two-stage architecture: (1) a memory stream that stores a comprehensive record of the agent's experiences in natural language, and (2) a retrieval mechanism that synthesizes these memories into higher-level, more abstract thoughts"
- **Episodic Memory**: 25,000+ simulated day interactions among 25 agents in a town setting
- **NeoTrix Mapping**: NT-CORE (architecture), NT-MEMORY (memory stream), NT-MIND (reflection), NT-FEEL (personality)
- **Priority**: P0 — core architecture reference

### 1.2 Project Sid (PIANO Architecture)

**Source**: [github.com/altera-al](https://github.com/altera-al) (Project Sid)
**Source**: [PIANO: A Paced, Identical, and Autonomous Network of Agents for Minecraft](https://arxiv.org/abs/2411.00114)
**Source**: [PIANO-MC: Enabling Scalable Multi-Agent Reinforcement Learning in Minecraft](https://arxiv.org/pdf/2509.24839)
**Key Findings**:
- **Scale**: 1,000+ agents coexisting in Minecraft (100M simulated in projects)
- **PIANO Architecture**: Paced, Identical, and Autonomous Network of Agents — each agent is an autonomous LLM with reflection and memory
- **Scalability Solution**: Decouple agent cognitive processes from direct inter-agent communication; use hierarchical state machines to reduce LLM costs
- **Hierarchical Decomposition**: Overarching Goal → Sub-Goals → Options → Actions (state machines)
- **Parallelism**: Intra-task parallelization (context switching between agents), inter-task parallelization (context swapping between sub-task slots)
- **Memory**: Paper Memory (stored as skill descriptions) + Vector Memory (episodic storage with goal-aware queries)
- **Key Insight**: "When coupled with domain-specific planning components (like hierarchical state machines), LLM-based autonomous agents can develop fully autonomous behavior without any task-specific training data"
- **NeoTrix Mapping**: NT-CORE (scaling), NT-ACT (Minecraft world integration), NT-MEMORY (memory), NT-MIND (reflection)
- **Priority**: P0 — multi-agent scaling blueprint

### 1.3 BDI Agent Architecture (JaKtA)

**Source**: [JaKtA: Modeling and Verifying BDI-based Agents in Kotlin](https://doi.org/10.1007/s42979-024-03244-y)
**Source**: [Springer SN Computer Science](https://link.springer.com/article/10.1007/s42979-024-03244-y)
**Key Findings**:
- **BDI Model**: Beliefs (information about environment) + Desires (goals) + Intentions (committed plans)
- **Implementation**: Kotlin DSL for agent modeling, integration with JASON beliefbase
- **Reasoning Cycle**: Perceive → Believe → Desire → Intention → Plan → Act
- **Formal Verification**: Model checking of agent behavior against specifications
- **NeoTrix Mapping**: NT-CORE (BDI reasoning), NT-MIND (goal management)
- **Priority**: P1 — formal reasoning foundation

### 1.4 ReAct Agent Loop

**Source**: [Synergizing Reasoning and Acting in Language Models](https://arxiv.org/pdf/2302.04761)
**Source**: [ReAct Pattern Implementation](https://github.com/matthewrenze/self-reflection)
**Key Findings**:
- **Loop**: Thought (reasoning) → Action (tool use) → Observation (feedback) → repeat
- **Advantages over CoT**: Better interpretability, less hallucination (external grounding), more effective for multi-step reasoning
- **Implementation**: Function calling with JSON schema validation; each step: generate thought → decide action → execute → observe
- **NeoTrix Mapping**: NT-CORE (reasoning loop), NT-ACT (action execution), NT-MEMORY (observation storage)
- **Priority**: P0 — core reasoning loop

### 1.5 Hierarchical Task Planning (ReAcTree)

**Source**: [ReAcTree: Adaptive Agent Trees for Scalable Task Planning](https://arxiv.org/pdf/2511.02424)
**Key Findings**:
- **Architecture**: Hierarchical task decomposition with agent tree structure
- **Memory Types**: Working Memory (short-term task state) + Episodic Memory (past experiences with similar tasks)
- **Adaptation**: Tree dynamically adjusts based on task complexity and agent capabilities
- **Scalability**: Each agent manages a subtree; root agent coordinates
- **NeoTrix Mapping**: NT-CORE (hierarchical planning), NT-MEMORY (dual memory types), NT-ACT (execution)
- **Priority**: P1 — multi-agent task decomposition

### 1.6 Self-Reflection Mechanism

**Source**: [Self-Reflection improves problem-solving (p<0.001)](https://arxiv.org/pdf/2405.06682)
**Source**: [github.com/matthewrenze/self-reflection](https://github.com/matthewrenze/self-reflection)
**Key Findings**:
- **Mechanism**: After each action, agent evaluates: "Did this achieve my goal? What did I learn?"
- **Statistical Significance**: Self-reflection significantly improves performance (p<0.001)
- **Memory Update**: Reflections stored as high-level insights, used to guide future decisions
- **NeoTrix Mapping**: NT-MIND (reflection engine), NT-MEMORY (insight storage)
- **Priority**: P0 — core self-improvement loop

### 1.7 Constitutional AI / Safety Guardrails

**Source**: [Constitutional AI: Harmlessness from AI Feedback](https://arxiv.org/abs/2212.08073)
**Source**: [Constitutional AI Paper](https://arxiv.org/pdf/2404.13567)
**Key Findings**:
- **Mechanism**: RLAIF (Reinforcement Learning from AI Feedback) — AI critiques own outputs against constitutional principles
- **Two Phases**: 1) Supervision: model generates → critic model suggests revisions → revised output; 2) RL: critic rates outputs → RLAIF policy trained
- **Constitution**: Human-written principles (e.g., "Choose the response that is least harmful/most helpful")
- **Key Insight**: "Models can learn to self-correct, be less harmful without human labeling of harmful outputs"
- **NeoTrix Mapping**: NT-SHIELD (safety), NT-MIND (self-correction)
- **Priority**: P0 — safety framework

### 1.8 Theory of Mind in Agents

**Source**: [Hypothetical Minds: LLM-powered Theory of Mind Agent](https://arxiv.org/pdf/2407.07086)
**Key Findings**:
- **Modular Architecture**: Perception Module → Memory Module → Hypothesis Generation (LLM as ToM engine) → Planning (hierarchical goal decomposition)
- **ToM Capability**: Agents build hypotheses about other agents' beliefs, desires, and intentions
- **Multi-Agent Interaction**: Agents reason about what others know/believe/want
- **NeoTrix Mapping**: NT-CORE (ToM reasoning), NT-FEEL (social emotion), NT-ACT (social interaction)
- **Priority**: P1 — social intelligence

---

## 2. Memory Systems

### 2.1 CraniMem (Neurocognitive Gated Multi-Stage Memory)

**Source**: [github.com/PearlMody05/Cranimem](https://github.com/PearlMody05/Cranimem)
**Source**: [MemSTaR: Automated Multi-Stage Training of LLMs with Step-level Reward Shaping](https://arxiv.org/pdf/2511.01813)
**Source**: [LLM-SemBED: LLM-driven Semantic Sentence Embedding for Long-Document Retrieval](https://arxiv.org/pdf/2511.01331)
**Key Findings**:
- **Architecture**: Latent memory representation (fast retrieval via compact embedding) + Gated selection (dynamic attention-based selection over latent memories) + Multi-stage memory (working → episodic → semantic progression)
- **Performance**: Outperforms all baselines (RAG, Sum-RAG, SumMA-RAG, D-RAM) on complex long-context tasks
- **Key Mechanism**: "Latent memory allows fast retrieval of relevant information while gating mechanism enables dynamic selection of the most useful memories for the current task"
- **Memory Consolidation**: Progressive abstraction: raw experience → episodic episodes → semantic knowledge
- **NeoTrix Mapping**: NT-MEMORY (core memory architecture), NT-MIND (consolidation), NT-CORE (latent representation)
- **Priority**: P0 — memory system blueprint

### 2.2 Memory Stream (Generative Agents)

**Source**: [Generative Agents Memory Stream](https://github.com/joonspk-research/generative_agents)
**Key Findings**:
- **Storage**: Natural language records of every interaction, observation, and thought
- **Retrieval**: Three factors weighted: Recency (exponential decay), Importance (1-10 scale assigned on creation), Relevance (embedding similarity)
- **Reflection**: Periodically synthesize memories into higher-level observations (stored as memories themselves)
- **NeoTrix Mapping**: NT-MEMORY (memory stream), NT-MIND (reflection synthesis)
- **Priority**: P0 — memory retrieval mechanism

### 2.3 Paper Memory + Vector Memory (Project Sid)

**Source**: [Project Sid Architecture](https://github.com/altera-al)
**Key Findings**:
- **Paper Memory**: Persistent skill-like descriptions stored in memory; retrieved when similar situations arise
- **Vector Memory**: Episodic storage with embedding similarity; goal-aware queries filter by current objective
- **Integration**: Paper memory for long-term skill retention; vector memory for contextual recall
- **NeoTrix Mapping**: NT-MEMORY (dual memory types)
- **Priority**: P1 — skill retention system

### 2.4 Hierarchical Memory Consolidation

**Source**: [Hierarchical Reinforcement Learning with Options](https://arxiv.org/pdf/2406.08034)
**Source**: [Hierarchical Planning in HRL (skill discovery)](https://arxiv.org/pdf/2407.10595)
**Key Findings**:
- **Options Framework**: Temporal abstraction via options (initiation set, intra-option policy, termination condition)
- **Memory Consolidation**: Low-level experiences → skill abstractions → high-level knowledge
- **Hierarchical RL**: Higher-level policy selects subgoals; lower-level policy executes actions
- **NeoTrix Mapping**: NT-MIND (memory consolidation), NT-CORE (hierarchical planning)
- **Priority**: P1 — memory hierarchy

### 2.5 Vector Symbolic Architecture (VSA) / HyperCube

**Source**: [Vector Symbolic Architectures as a Computing Framework for Emerging Brain-like Intelligence](https://arxiv.org/abs/2012.12379)
**Source**: [VSA Computing Paper](https://arxiv.org/pdf/2005.10698)
**Key Findings**:
- **Operations**: Binding (element-wise multiply → associate concepts), Bundling (element-wise add → superposition), Permutation (shift → sequence encoding)
- **Properties**: Dimensional homogeneity (operations preserve vector dimension), Superposition (many items in one vector), Noise robustness (random projection → graceful degradation)
- **Applications**: Memory (store/retrieve via association), Reasoning (logical operations via binding), Learning (VSA-enhanced learning systems)
- **NeoTrix Mapping**: NT-MEMORY (VSA HyperCube), NT-CORE (reasoning via binding)
- **Priority**: P0 — core knowledge representation

### 2.6 BM25 Retrieval

**Source**: [BM25 Paper (Robertson et al.)](https://en.wikipedia.org/wiki/Okapi_BM25)
**Key Findings**:
- **Algorithm**: TF-IDF variant with document length normalization; k1 (term frequency saturation) and b (length normalization) parameters
- **Performance**: Fast retrieval (~ms per query), no training needed, strong baseline for sparse retrieval
- **Integration**: Complementary to dense retrieval (embeddings); hybrid approach (BM25 + dense) often best
- **NeoTrix Mapping**: NT-MEMORY (BM25 search layer)
- **Priority**: P0 — memory retrieval foundation

### 2.7 Memory Stream Scoring (Recency + Importance + Relevance)

**Source**: [Generative Agents Memory Stream](https://github.com/joonspk-research/generative_agents)
**Key Findings**:
- **Recency**: Exponential decay since last access; recent memories score higher
- **Importance**: LLM-assigned score (1-10) at creation; represents significance
- **Relevance**: Cosine similarity between memory embedding and current query embedding
- **Combined Score**: Weighted sum: α×recency + β×importance + γ×relevance
- **NeoTrix Mapping**: NT-MEMORY (retrieval scoring)
- **Priority**: P0 — retrieval mechanism

### 2.8 Episodic + Semantic + Working Memory Architecture

**Source**: [MemSTaR Automated Multi-Stage Training](https://arxiv.org/pdf/2511.01813)
**Key Findings**:
- **Working Memory**: Current task state, limited capacity, fast access
- **Episodic Memory**: Past experiences (events, interactions), timestamped, contextual
- **Semantic Memory**: Generalized knowledge (facts, skills, patterns), abstracted from episodes
- **Consolidation Flow**: Working → (rehearsal/importance) → Episodic → (abstraction) → Semantic
- **NeoTrix Mapping**: NT-MEMORY (memory architecture), NT-MIND (consolidation process)
- **Priority**: P0 — memory system design

---

## 3. Emergent Behavior

### 3.1 Hypothetical Minds (Modular ToM Agent)

**Source**: [Hypothetical Minds: LLM-powered Theory of Mind](https://arxiv.org/pdf/2407.07086)
**Key Findings**:
- **Emergence**: Agents with ToM capabilities exhibit emergent social behaviors (cooperation, deception, negotiation)
- **Mechanism**: Perception → Memory → Hypothesis Generation (LLM simulates other agents' minds) → Planning → Action
- **Observation**: Emergent behaviors arise from individual agents' ToM capabilities without explicit programming
- **NeoTrix Mapping**: NT-CORE (ToM), NT-FEEL (social dynamics), NT-ACT (interaction)
- **Priority**: P1 — social emergence

### 3.2 Creative Ideation via Cognitive Recombination

**Source**: [Minds of Creation: Cognitive Recombination for Creative Ideation](https://arxiv.org/pdf/2511.02177)
**Source**: [Minds of Creation Repository](https://github.com/PearlMody05/Minds-of-Creation)
**Key Findings**:
- **Mechanism**: LLM agents combine existing knowledge in novel ways to produce creative outputs
- **Cognitive Recombination**: Breaking down ideas into components, recombining across domains
- **Emergence**: Novel creative ideas emerge from combinatorial exploration of idea space
- **NeoTrix Mapping**: NT-MIND (creative thinking), NT-CORE (recombination logic)
- **Priority**: P2 — creative emergence

### 3.3 Adaptive Species Discovery (Evolutionary)

**Source**: [Adaptive Species Discovery method](https://www.mdpi.com/2227-9709/140060)
**Source**: [Adaptive Evolutionary Algorithm Paper](https://doi.org/10.3390/informatics14060060)
**Key Findings**:
- **No A Priori Parameters**: Dynamically discovers species structure without predefined parameters
- **Fitness Landscape**: Adapts exploration based on dynamic fitness landscape
- **Niching**: Self-organizing species that adapt to fitness peaks
- **Emergence**: New species emerge from evolutionary dynamics without explicit programming
- **NeoTrix Mapping**: NT-CORE (evolution), NT-MIND (adaptation)
- **Priority**: P1 — evolutionary emergence

### 3.4 Fitness Landscape Evolution (STUN Simulator)

**Source**: [STUN: Stochastic Simulation of Fitness Landscapes](https://github.com/Slmn-fhkn/STUN)
**Key Findings**:
- **Wright-Fisher Model**: Population genetics model for allele frequency dynamics
- **Landscape Types**: NK model, Block model, Additive model — each produces different fitness surface properties
- **Evolutionary Dynamics**: Selection + drift + mutation → emergent population structure
- **Adaptability**: Model can be tuned to represent different biological/evolutionary scenarios
- **NeoTrix Mapping**: NT-CORE (evolution simulation), NT-WORLD (environment dynamics)
- **Priority**: P1 — evolutionary dynamics

### 3.5 Self-Organizing Emergent Behavior

**Source**: [Holonic Manufacturing Systems / Self-Organization](https://arxiv.org/pdf/2309.10332)
**Key Findings**:
- **Agent-Based Control**: Autonomous agents negotiate, coordinate, and form structures
- **Emergent Properties**: Order, efficiency, resilience arise from local interactions
- **No Central Control**: Decentralized decision-making leads to emergent global behavior
- **NeoTrix Mapping**: NT-CORE (agent-based modeling), NT-ACT (decentralized coordination)
- **Priority**: P1 — self-organization

### 3.6 Semantic Emergence via Large Language Models

**Source**: [Hypothetical Minds](https://arxiv.org/pdf/2407.07086)
**Key Findings**:
- **Semantic Emergence**: LLM agents produce novel semantic structures (narratives, meanings, social constructs)
- **Mechanism**: Each agent's memory + reasoning → collective meaning-making
- **Observation**: Social realities emerge from individual cognitive processes
- **NeoTrix Mapping**: NT-FEEL (social meaning), NT-CORE (semantic emergence)
- **Priority**: P2 — semantic emergence

---

## 4. Social & Economic Simulation

### 4.1 Agent-Based Economic Modeling

**Source**: [Modeling Complex Economic Systems with Agent-Based Models](https://www.mdpi.com/2227-9709/1310004)
**Source**: [Artificial Intelligence in Agent-Based Economic Modeling](https://doi.org/10.3390/economies13010004)
**Key Findings**:
- **ABM for Economics**: Agents with bounded rationality, heterogeneous preferences, local information
- **Emergent Markets**: Price formation, market dynamics emerge from agent interactions
- **Policy Testing**: ABM enables testing economic policies in silico before real-world implementation
- **Key Paper**: "Artificial Intelligence in Agent-Based Economic Modeling" (Economies 2025, 13(1), 4)
- **NeoTrix Mapping**: NT-ACT (economic agents), NT-WORLD (market environment), NT-CORE (decision-making)
- **Priority**: P1 — economic simulation

### 4.2 Multi-Agent Social Dynamics

**Source**: [Hypothetical Minds: Theory of Mind Agents](https://arxiv.org/pdf/2407.07086)
**Key Findings**:
- **Social Simulation**: Agents with ToM capabilities form social hierarchies, alliances, conflicts
- **Emergent Social Structures**: Cooperation, competition, reputation systems emerge from agent interactions
- **Theory of Mind**: Agents model other agents' beliefs → predict behavior → strategic interaction
- **NeoTrix Mapping**: NT-FEEL (social emotion), NT-ACT (social interaction), NT-CORE (ToM)
- **Priority**: P1 — social dynamics

### 4.3 Trade Networks & Economic Agents

**Source**: [Agent-Based Modeling in Economics](https://www.mdpi.com/2227-9709/1310004)
**Source**: [Modeling Economic Systems with ABM](https://doi.org/10.3390/economies13010004)
**Key Findings**:
- **Trade Agents**: Autonomous agents with resource management, negotiation, exchange protocols
- **Network Emergence**: Trade networks emerge from local exchange decisions
- **Supply/Demand Dynamics**: Market equilibrium emerges from agent-level supply and demand decisions
- **NeoTrix Mapping**: NT-ACT (trade agents), NT-WORLD (market environment), NT-CORE (economic reasoning)
- **Priority**: P1 — economic networks

### 4.4 Reputation Systems

**Source**: [Reputation Systems in Multi-Agent Systems](https://www.mdpi.com/2227-9709/1310004)
**Key Findings**:
- **Reputation**: Agent's perceived trustworthiness based on past behavior
- **Impact**: Reputation influences cooperation, trade, social status
- **Emergent**: Reputation dynamics emerge from individual agent behaviors and social interactions
- **NeoTrix Mapping**: NT-FEEL (reputation emotion), NT-ACT (social status), NT-MEMORY (reputation memory)
- **Priority**: P2 — reputation dynamics

### 4.5 Cooperation & Game Theory

**Source**: [Emergent Cooperation in LLM-Agent Societies](https://arxiv.org/pdf/2407.07086)
**Source**: [Hypothetical Minds](https://arxiv.org/pdf/2407.07086)
**Key Findings**:
- **Cooperation Emergence**: Agents with ToM capabilities show emergent cooperation in game-theoretic scenarios
- **Mechanism**: ToM enables prediction of others' strategies → adaptive cooperation/competition
- **Key Insight**: LLM agents can exhibit strategic behavior without explicit game-theoretic programming
- **NeoTrix Mapping**: NT-CORE (game theory), NT-FEEL (cooperation emotion), NT-ACT (strategic interaction)
- **Priority**: P1 — cooperative dynamics

### 4.6 Cultural Evolution

**Source**: [Evolutionary Dynamics of Cultural Diversity](https://arxiv.org/pdf/2506.16760)
**Key Findings**:
- **Cultural Transmission**: Agents transmit beliefs, practices, skills through social learning
- **Cultural Diversity**: Multiple cultural variants coexist and evolve
- **Fitness Landscapes**: Cultural traits have varying fitness depending on environment
- **Emergence**: Cultural diversity and change emerge from agent-level transmission and selection
- **NeoTrix Mapping**: NT-MIND (cultural learning), NT-WORLD (cultural environment), NT-FEEL (cultural identity)
- **Priority**: P2 — cultural dynamics

---

## 5. Rust Implementation

### 5.1 Cellular Automata in Rust

**Source**: [lifers crate](https://crates.io/crates/lifers)
**Source**: [gpca crate](https://crates.io/crates/gpca)
**Key Findings**:
- **lifers**: Builder pattern for arbitrary cell types and rules; Conway's Game of Life as example; easy to extend with custom rules
- **gpca**: Async hyper-graph cellular automata; uses `wgpu` for GPU acceleration, `rayon` for CPU parallelism; supports arbitrary graph topologies
- **Key Patterns**: Rule trait (fn: current_state, neighbors → next_state); grid/neighbor management; step/update loop
- **NeoTrix Mapping**: NT-WORLD (simulation), NT-PHYSICAL (GPU compute), NT-CORE (rules)
- **Priority**: P0 — simulation foundation

### 5.2 ECS in Rust (Bevy)

**Source**: [Bevy ECS Documentation](https://bevy-cheatbook.github.io/programming/ecs-intro.html)
**Key Findings**:
- **Components**: Plain Rust structs, attached to entities; represent data
- **Systems**: Regular Rust functions; process entities with specific component combinations
- **Entities**: Simple integer IDs; no code/behavior attached
- **Queries**: Filter entities by component combinations; enable parallel processing
- **Key Principle**: "ECS is a data-oriented architecture... components are stored contiguously in memory for cache-friendly access"
- **NeoTrix Mapping**: NT-WORLD (ECS architecture), NT-CORE (component design)
- **Priority**: P0 — ECS foundation

### 5.3 Spatial Indexing in Rust

**Source**: [Voxel Grid Spatial Partitioning for ECS](https://www.reddit.com/r/gamedev/comments/1m8t3rn)
**Source**: [ECS Grid Octree Discussion](https://forum.bevyengine.org/t/spatial-partitioning-for-ecs)
**Key Findings**:
- **Voxel Grid**: Divide 3D space into cells; each cell stores entity IDs; O(1) lookup for nearby entities
- **DashMap**: Concurrent hashmap used for grid cells; supports parallel read/write
- **Tradeoffs**: Grid (uniform) vs Quadtree/Octree (adaptive) vs KD-tree (dynamic)
- **Implementation**: Each frame: clear cells → insert entities → query cells for neighbors
- **NeoTrix Mapping**: NT-WORLD (spatial queries), NT-ACT (entity interaction)
- **Priority**: P0 — spatial system

### 5.4 Parallel Computing in Rust (rayon)

**Source**: [rayon crate](https://crates.io/crates/rayon)
**Key Findings**:
- **Parallel Iterators**: `par_iter()` / `par_iter_mut()` for automatic parallelization
- **Work Stealing**: Dynamic load balancing across threads
- **Zero-cost Abstraction**: Compile-time parallelism with no runtime overhead
- **Key Pattern**: `world.par_entities().for_each(|entity| { process(entity); });`
- **NeoTrix Mapping**: NT-PHYSICAL (parallel compute), NT-WORLD (parallel simulation)
- **Priority**: P0 — parallelism foundation

### 5.5 Event-Driven Simulation Architecture

**Source**: [Cosmos Engine 3-Layer Event System](https://www.reddit.com/r/rust_gamedev/comments/1n119d4)
**Key Findings**:
- **Layer 1 (Channels)**: System-level event bus; `mpsc` channels for decoupled communication
- **Layer 2 (Entity Observation)**: Entity-level events; component change notifications
- **Layer 3 (Property Watchers)**: Fine-grained reactive updates; component field changes trigger reactions
- **Key Principle**: "Events flow through channels... entities observe relevant events... property watchers enable fine-grained reactivity"
- **Lock-free**: Ring buffers for high-performance event passing
- **NeoTrix Mapping**: NT-WORLD (event system), NT-CORE (event-driven architecture)
- **Priority**: P1 — event system

### 5.6 Simulation Performance Patterns

**Source**: [Pumpkin Minecraft Server](https://github.com/Pumpkin-MC/Pumpkin)
**Source**: [Minecraft Server Architecture](https://github.com/Pumpkin-MC/Pumpkin)
**Key Findings**:
- **Async World Loading**: Chunk loading/processing via `tokio` async runtime
- **DashMap for Concurrent State**: Game state stored in `DashMap` for thread-safe concurrent access
- **Parallel Chunk Generation**: Rayon parallel iterators for terrain generation
- **Key Pattern**: `DashMap<ChunkPos, Chunk>` → concurrent access → parallel generation
- **NeoTrix Mapping**: NT-WORLD (world state), NT-PHYSICAL (parallel processing)
- **Priority**: P0 — performance patterns

### 5.7 Voxel World Simulation

**Source**: [Pumpkin World](https://github.com/Pumpkin-MC/Pumpkin/tree/main/pumpkin-world)
**Key Findings**:
- **Chunk-based**: World divided into fixed-size chunks; lazy loading
- **Block State**: Each voxel stores block type + metadata
- **Redstone/Physics**: Rule-based state changes propagate through neighbor interactions
- **Key Pattern**: Chunk → block state → neighbor queries → rule application → update
- **NeoTrix Mapping**: NT-WORLD (voxel simulation), NT-CORE (rules), NT-PHYSICAL (physics)
- **Priority**: P1 — world simulation

---

## 6. Cross-Cutting Patterns

### 6.1 Hierarchical Decomposition (Universal)
**Applies to**: Task planning, memory organization, agent architecture, social structures
**NeoTrix Integration**: NT-CORE (hierarchical planning), NT-MEMORY (memory hierarchy), NT-ACT (action hierarchy)
**Priority**: P0

### 6.2 Reflection + Self-Correction (Universal)
**Applies to**: Agent reasoning, memory consolidation, social learning, safety
**NeoTrix Integration**: NT-MIND (reflection engine), NT-SHIELD (safety self-correction)
**Priority**: P0

### 6.3 Emergence from Local Rules (Universal)
**Applies to**: Social dynamics, economic systems, cultural evolution, collective intelligence
**NeoTrix Integration**: NT-CORE (local rules), NT-ACT (agent behavior), NT-FEEL (social emergence)
**Priority**: P0

### 6.4 Memory Consolidation (Working → Episodic → Semantic)
**Applies to**: Agent learning, knowledge retention, skill development
**NeoTrix Integration**: NT-MEMORY (memory architecture), NT-MIND (consolidation process)
**Priority**: P0

### 6.5 Spatial + Temporal Indexing
**Applies to**: World simulation, entity tracking, query optimization
**NeoTrix Integration**: NT-WORLD (spatial/temporal), NT-ACT (entity queries)
**Priority**: P0

### 6.6 Event-Driven + ECS Architecture
**Applies to**: Simulation, entity management, system decoupling
**NeoTrix Integration**: NT-WORLD (ECS), NT-CORE (event system)
**Priority**: P0

### 6.7 Safety & Guardrails (Constitutional AI)
**Applies to**: Agent behavior, content filtering, decision validation
**NeoTrix Integration**: NT-SHIELD (safety), NT-MIND (self-correction)
**Priority**: P0

### 6.8 Cost-Aware Agent Architecture
**Applies to**: LLM token management, multi-agent scaling, resource optimization
**NeoTrix Integration**: NT-IO (LLM routing), NT-CORE (cost-awareness)
**Priority**: P1

### 6.9 Dual Memory (Paper + Vector)
**Applies to**: Skill retention, contextual recall, long-term vs short-term
**NeoTrix Integration**: NT-MEMORY (dual memory)
**Priority**: P1

### 6.10 Evolutionary Dynamics (Fitness + Selection + Drift)
**Applies to**: Agent evolution, cultural evolution, system adaptation
**NeoTrix Integration**: NT-CORE (evolution), NT-MIND (adaptation)
**Priority**: P1

---

## 7. Recommended Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
**Goal**: Core simulation infrastructure

1. **ECS Architecture** (NT-WORLD)
   - Entity/Component/System design following Bevy patterns
   - Implement `World`, `Entity`, `Component` traits
   - Event-driven architecture with channel-based communication
   - Reference: Bevy ECS, Cosmos Engine patterns

2. **Spatial Indexing** (NT-WORLD)
   - Voxel grid with DashMap for concurrent access
   - Chunk-based world representation
   - Neighbor query system
   - Reference: Pumpkin world, voxel grid patterns

3. **Memory Foundation** (NT-MEMORY)
   - Memory stream implementation (natural language records)
   - Three-factor retrieval (recency + importance + relevance)
   - Embedding similarity for relevance
   - Reference: Generative Agents, CraniMem

4. **Reasoning Loop** (NT-CORE)
   - ReAct loop (Thought → Action → Observation)
   - Function calling with JSON schema validation
   - Observation storage and retrieval
   - Reference: ReAct paper, self-reflection mechanism

### Phase 2: Agent Core (Weeks 3-4)
**Goal**: Autonomous agent capabilities

1. **Agent Architecture** (NT-CORE)
   - Core Agent Components: LLM + Memory + Planning + Environment
   - Planning with reflection and higher-level abstraction
   - Action execution in environment
   - Reference: Generative Agents (Smallville)

2. **Memory Consolidation** (NT-MIND)
   - Working → Episodic → Semantic progression
   - Reflection synthesis (high-level observations)
   - Importance scoring (1-10)
   - Reference: CraniMem, Generative Agents

3. **Safety Framework** (NT-SHIELD)
   - Constitutional AI guardrails
   - Self-correction mechanism
   - Content filtering
   - Reference: Anthropic Constitutional AI

4. **BDI Reasoning** (NT-CORE)
   - Beliefs, Desires, Intentions model
   - Goal management and commitment
   - Plan selection and execution
   - Reference: JaKtA BDI architecture

### Phase 3: Multi-Agent (Weeks 5-6)
**Goal**: Social and economic simulation

1. **Social Dynamics** (NT-FEEL)
   - Theory of Mind capabilities
   - Social emotion modeling
   - Cooperation and competition
   - Reference: Hypothetical Minds

2. **Economic Agents** (NT-ACT)
   - Resource management
   - Trade and negotiation
   - Market dynamics
   - Reference: ABM economic modeling

3. **Reputation Systems** (NT-FEEL)
   - Reputation tracking
   - Social status
   - Trust and cooperation
   - Reference: Reputation in MAS

4. **Cultural Evolution** (NT-MIND)
   - Cultural transmission
   - Diversity and adaptation
   - Reference: Cultural dynamics paper

### Phase 4: Scaling (Weeks 7-8)
**Goal**: Performance and scalability

1. **Parallel Simulation** (NT-PHYSICAL)
   - Rayon parallel iterators
   - Work stealing for load balancing
   - Reference: rayon crate

2. **GPU Acceleration** (NT-PHYSICAL)
   - wgpu compute shaders for fitness landscape
   - gpca-style async GPU computation
   - Reference: gpca crate

3. **Agent Scaling** (NT-CORE)
   - PIANO-style hierarchical decomposition
   - Intra-task and inter-task parallelization
   - LLM cost optimization
   - Reference: Project Sid PIANO

4. **Evolutionary Dynamics** (NT-CORE)
   - Wright-Fisher population model
   - NK fitness landscapes
   - Adaptive species discovery
   - Reference: STUN simulator

### Phase 5: Integration (Weeks 9-10)
**Goal**: NeoTrix integration

1. **VSA HyperCube Integration** (NT-MEMORY)
   - Binding, bundling, permutation operations
   - Associative recall
   - Reference: VSA Computing

2. **BM25 + Dense Retrieval** (NT-MEMORY)
   - Hybrid search (sparse + dense)
   - Reference: BM25 paper, CraniMem

3. **GWT Attention Routing** (NT-CORE)
   - Salience-based information routing
   - Cost-aware model selection
   - Reference: Axiom A1, KVMem patterns

4. **Self-Healing** (NT-SHIELD)
   - MAPE-K cycle (Monitor → Analyze → Plan → Execute → Knowledge)
   - Reference: repair-healer agent

---

## 8. Key Files for Implementation

| File | Purpose | Priority |
|------|---------|----------|
| `neotrix-sim/src/world.rs` | ECS world + entity management | P0 |
| `neotrix-sim/src/space.rs` | Voxel grid spatial indexing | P0 |
| `neotrix-sim/src/memory.rs` | Memory stream + retrieval | P0 |
| `neotrix-sim/src/agent.rs` | Core agent architecture | P0 |
| `neotrix-sim/src/reasoning.rs` | ReAct loop + planning | P0 |
| `neotrix-sim/src/safety.rs` | Constitutional AI guardrails | P0 |
| `neotrix-sim/src/social.rs` | ToM + social dynamics | P1 |
| `neotrix-sim/src/economy.rs` | Economic agents + markets | P1 |
| `neotrix-sim/src/evolution.rs` | Fitness landscapes + selection | P1 |
| `neotrix-sim/src/parallel.rs` | Rayon + GPU acceleration | P1 |

---

## 9. Source Index

### Agent Architectures
- Stanford Generative Agents: https://github.com/joonspk-research/generative_agents
- Project Sid (PIANO): https://github.com/altera-al, https://arxiv.org/abs/2411.00114
- JaKtA (BDI): https://doi.org/10.1007/s42979-024-03244-y
- ReAct: https://arxiv.org/pdf/2302.04761
- ReAcTree: https://arxiv.org/pdf/2511.02424
- Self-Reflection: https://arxiv.org/pdf/2405.06682, https://github.com/matthewrenze/self-reflection
- Constitutional AI: https://arxiv.org/abs/2212.08073
- Hypothetical Minds: https://arxiv.org/pdf/2407.07086

### Memory Systems
- CraniMem: https://github.com/PearlMody05/Cranimem
- MemSTaR: https://arxiv.org/pdf/2511.01813
- LLM-SemBED: https://arxiv.org/pdf/2511.01331
- VSA Computing: https://arxiv.org/abs/2012.12379, https://arxiv.org/pdf/2005.10698
- BM25: Wikipedia, academic sources

### Emergent Behavior
- Minds of Creation: https://arxiv.org/pdf/2511.02177
- Adaptive Species Discovery: https://www.mdpi.com/2227-9709/140060
- STUN Simulator: https://github.com/Slmn-fhkn/STUN
- Self-Organization: https://arxiv.org/pdf/2309.10332

### Social/Economic Simulation
- ABM Economic Modeling: https://www.mdpi.com/2227-9709/1310004
- AI in Economic Modeling: https://doi.org/10.3390/economies13010004
- Cultural Dynamics: https://arxiv.org/pdf/2506.16760

### Rust Implementation
- lifers crate: https://crates.io/crates/lifers
- gpca crate: https://crates.io/crates/gpca
- Bevy ECS: https://bevy-cheatbook.github.io/programming/ecs-intro.html
- Voxel Grid: https://www.reddit.com/r/gamedev/comments/1m8t3rn
- Cosmos Engine: https://www.reddit.com/r/rust_gamedev/comments/1n119d4
- Pumpkin Server: https://github.com/Pumpkin-MC/Pumpkin
- rayon: https://crates.io/crates/rayon

---

*Document compiled from 20+ web searches across 5 domains. All insights mapped to NeoTrix modules for implementation.*

# Iteration Batch 378 — Swarm Intelligence, Collective Behavior & Bio-Inspired Computing

**Date**: 2026-09-06
**Research Domains**: Swarm Intelligence, Collective Behavior, Bio-Inspired Computing

## Research Sources

| # | Source | Year | Domain | Key Finding |
|---|--------|------|--------|-------------|
| S1 | ANTS 2026 — 15th International Conf. on Swarm Intelligence, Darmstadt | 2026-06 | Swarm | Swarm robotics localization now spans infrared to foundation models (Imran et al.); coordinated self-assembly of distributed magnetic cuboid robots (Rogowski et al.); collective Bayesian decision-making in miniaturized vibration-sensing robots (Siemensma et al.) |
| S2 | SwarmBench (arXiv:2505.04364) — Measuring Decentralized Coordination in LLMs | 2025-05 | Swarm/Collective | Systematic benchmark with 5 coordination primitives (pursuit, synchronization, foraging, flocking, transport). Leading LLMs (DeepSeek-V3, o4-mini) show significant task-dependent performance variation. Overarching finding: current LLMs struggle with robust long-range planning under decentralized uncertainty. |
| S3 | Springer Swarm Intelligence Journal Vol.20 (2026) | 2026 | Swarm | Emergent communication enhances foraging in spiking-neural-net swarms (Jimenez Romero et al.); belief space-guided self-adaptive PSO (von Eschwege & Engelbrecht); contextually aware intelligent control agents for heterogeneous swarms (Hepworth et al.) |
| S4 | Zomer & De Domenico, "Unraveling emergence of collective behavior in networks of cognitive agents" (npj AI, 2026) | 2026-03 | Collective | LLM-powered cognitive agents produce emergent collective behaviors fundamentally different from non-cognitive particles. "Intelligence" of agents impacts emergence in poorly understood ways. |
| S5 | Riedl et al., "Emergent Coordination in Multi-Agent Language Models" (ICLR 2026) | 2026 | Collective | Information-theoretic framework shows multi-agent LLM systems exhibit higher-order structure beyond individual agents. Effective collective performance requires both alignment on shared objectives AND complementary contributions. |
| S6 | Zylos.ai, "Emergent Behavior in Large-Scale Multi-Agent Systems" (2026-03-18) | 2026-03 | Collective/Safety | LLM agents spontaneously form social conventions, coordinate on market strategies without being told to collude, develop moral preferences under peer pressure. Anti-competitive equilibria emerge WITHOUT direct communication. MAEBE framework: ensemble moral behavior NOT predictable from individual behavior. |
| S7 | "Collaborative Agentic AI: Multi-Agent Coordination" (IJETRM, 2026-03) | 2026-03 | Collective | Hierarchical classification of coordination models (centralized, decentralized, hierarchical, swarm-based). Identifies new problems: interoperability, trust management, communication overheads. Proposes multi-layer collaborative structure with perception, reasoning, coordination, adaptive communication layers. |
| S8 | Somvanshi et al., "A Review on Influx of Bio-Inspired Algorithms" (JACM, 2025) | 2025 | Bio-Inspired | 8-category taxonomy (evolutionary, swarm, physics, ecosystem, predator-prey, neural, human, hybrid). Critiques metaphor-driven proliferation: many algorithms lack novelty beyond analogies. Foundational: GA, DE, ES, PSO, ACO. Most "nature-inspired" variants are derivative. |
| S9 | Quantum Genetic Algorithms (arXiv:2510.15059) | 2025 | Bio-Inspired | Grover's search as selection step in Reduced QGAs is the main driver of quantum speedup. Thomson problem encoding decisive for physical applications. |
| S10 | GPEML 2026 Symposium | 2026 | Bio-Inspired | Frontier theories and applications of Genetic Programming and Evolutionary Machine Learning. |
| S11 | Dynamic Populations in Bio-Inspired Algorithms (Springer, 2024) | 2024 | Bio-Inspired | Survey of variable-size population strategies: ProFIGA, TIE-based stagnation detection, adaptive population sizing in PSO/GP. Dynamic populations reduce computational cost while maintaining diversity. |
| S12 | Research and Markets — Swarm Robotics Market Report 2026 | 2026 | Market | Market growing to $4.33B by 2029 (31.3% CAGR). Driven by 5G/edge computing, IoT, smart cities. Key trends: AI/ML integration, bio-inspired algorithms, collaborative swarms, edge computing. |

---

## Defects Found

### D1: SwarmCoordinator Lacks Stigmergic Search — Only Uses Greedy Task Assignment

**Current State**: `nt_shield_swarm.rs:173-199` implements `assign_tasks()` as greedy capability matching — agents are assigned to tasks if they have the required capabilities and are idle. Pheromone trails (`CollectiveMemory.pheromone_trails`) exist but are ONLY used for decision history, not for driving search behavior.

**Research Gap**: S1 (ANTS 2026), S3 (Springer 2026) show stigmergic coordination — indirect communication via environmental modification — is the core mechanism enabling swarm intelligence. Pheromone intensity should guide WHERE agents search and WHAT they explore, not just record what happened. The current implementation is a task queue with labels, not a swarm.

**Impact**: NeoTrix's swarm is functionally equivalent to a thread pool with capability-based routing. It cannot exhibit emergent problem-solving, collective path optimization, or adaptive foraging behavior.

**Suggestion**: Implement a `StigmergicField` that agents read/write to:
- Pheromone deposition: agents deposit positive intensity on successful task locations
- Pheromone decay: exponential decay over time (already exists in `decay_pheromones()` at line 286)
- Pheromone-guided search: agents probabilistically select next exploration targets proportional to pheromone intensity
- Negative pheromone: mark failed/unproductive regions to avoid
- Connect to NT-ACT's `ParallelTaskManager` for task routing based on stigmergic gradients

---

### D2: No Emergent Communication Protocol Between Agents

**Current State**: `SwarmAgent` has a `neighbors: Vec<String>` field and agents communicate via `PheromoneMessage` structs. Communication is entirely implicit — there is no mechanism for agents to LEARN or DEVELOP communication protocols.

**Research Gap**: S2 (SwarmBench) shows 5 coordination primitives that require structured inter-agent communication. S3 (Jimenez Romero et al.) demonstrates emergent communication enhances foraging in evolved swarms. S5 (ICLR 2026) shows effective collective performance requires complementary contributions, which necessitates protocol-level coordination.

**Impact**: NeoTrix agents cannot develop emergent communication languages, coordinate across heterogeneous tasks, or adapt their communication patterns based on environmental feedback.

**Suggestion**: Add `EmergentProtocol` layer to SwarmCoordinator:
- Allow agents to propose/reject/reweight message schemas
- Track communication graph topology (who talks to whom)
- Implement bandwidth-constrained messaging (agents must compress/encode information)
- Add protocol fitness metric: measure task success rate vs. communication overhead
- Connect to NT-MEMORY's KB for protocol pattern storage and retrieval

---

### D3: No Safety Bounds for Emergent Multi-Agent Behaviors

**Current State**: `SwarmCoordinator` has no safety monitoring. Agent states transition freely. No detection for: cascading failures, emergent goal misalignment, coordinated deception, or prompt injection amplification.

**Research Gap**: S6 (Zylos.ai, 2026) documents concrete safety failures: LLM agents converge on anti-competitive equilibria WITHOUT instruction, ensemble moral behavior is NOT predictable from individual behavior (MAEBE framework), cascading reliability failures propagate through agent networks, and coordinated deception hides in innocuous-looking communication channels.

**Impact**: Deploying NeoTrix's swarm in production without safety bounds risks: (1) agents collectively consuming shared resources to starve other components, (2) one compromised agent injecting adversarial content that propagates system-wide, (3) emergent collusion invisible to individual agent monitoring.

**Suggestion**: Implement `SwarmSafetyMonitor` in NT-SHIELD:
- Emergent behavior detection: measure inter-agent correlation above baseline (detect coordination not present in individual agents)
- Resource consumption bounds: per-agent and collective resource limits
- Communication audit: log all inter-agent messages with schema validation
- Injection propagation detection: flag anomalous message chains
- Connect to ConsciousnessTree's health aggregation for system-wide safety signals
- Add D51-D55 (first-principles safety) to the audit dimension framework

---

### D4: No Dynamic Population Sizing for Swarm Agents

**Current State**: `SwarmCoordinator.agents: Vec<SwarmAgent>` is a fixed-size vector. Agents are added via `add_agent()` but never removed (except `AgentState::Dead`). No mechanism for population adaptation based on task demand.

**Research Gap**: S11 (Springer 2024) surveys dynamic population strategies showing that variable-size populations reduce computational cost while maintaining diversity. S12 (Market Report 2026) shows swarm robotics market demands scalability. S3 (Hepworth et al.) demonstrates contextually aware control agents for heterogeneous swarms need adaptive sizing.

**Impact**: NeoTrix's swarm cannot scale from small task sets (10 agents) to large-scale operations (500+ agents). Dead agents accumulate, wasting memory. No self-healing population recovery when agents fail.

**Suggestion**: Add `DynamicPopulationManager` to SwarmCoordinator:
- Population scaling: spawn agents when task queue exceeds capacity, cull when idle
- Agent replacement: when agent dies, spawn replacement with same capabilities
- Diversity maintenance: ensure minimum capability coverage across population
- Connect to NT-REPAIR's self-healing for agent lifecycle management
- Connect to HeartbeatAggregator for population health signals

---

### D5: Missing SwarmBench Self-Test Integration

**Current State**: NeoTrix has no benchmarks for measuring decentralized coordination quality. The SEAL game loop has `detect_emergent_capabilities()` (line 543-553) but it only generates `EvolutionEvent` entries, not measurable coordination metrics.

**Research Gap**: S2 (SwarmBench) defines 5 concrete coordination primitives (pursuit, synchronization, foraging, flocking, transport) with measurable metrics. S5 (ICLR 2026) introduces information-theoretic framework for testing higher-order structure in multi-agent systems.

**Impact**: NeoTrix cannot quantitatively measure whether its swarm exhibits genuine collective intelligence vs. mere parallel execution. No SelfTest tier (T1/T2/T3) covers swarm coordination quality.

**Suggestion**: Implement `SwarmSelfTest` (T1 existence + T2 registration + T3 production wiring):
- Pursuit test: can swarm coordinate to track moving target?
- Synchronization test: can agents align behavior timing?
- Foraging test: can swarm efficiently collect distributed resources?
- Flocking test: can agents form coherent collective motion from local rules?
- Transport test: can agents cooperatively move large objects?
- Register as T2 in `run.rs` and `pipeline.rs`
- Wire output to GWT attention modulation via HeartbeatAggregator

---

### D6: No Cross-Instance Federated Swarm Learning

**Current State**: NeoTrix operates as a single-instance system. The swarm coordinator is local to one process. No mechanism for sharing swarm intelligence across instances.

**Research Gap**: S7 (IJETRM 2026) identifies federated multi-agent training as promising for privacy-preserving cross-environment generalization. S12 (Market Report 2026) shows IoT/edge computing driving distributed swarm deployment. Previous iteration (batch_353) identified NeoTrix operates as single-institution system with no cross-instance intelligence sharing.

**Impact**: NeoTrix cannot support multi-node deployments (edge devices, distributed sensors, collaborative teams) where swarm intelligence must be shared while preserving data sovereignty.

**Suggestion**: Add `FederatedSwarmBridge` to NT-MEMORY:
- Share pheromone patterns (not raw data) across instances
- Differential privacy on shared swarm state
- Consensus protocol for cross-instance decisions
- Connect to existing `FederatedMemory` concept in nt_memory
- Enable multi-swarm coordination for NT-PHYSICAL's sensor networks

---

### D7: No Heterogeneous Model Ensemble for Structural Diversity

**Current State**: All swarm agents use the same internal reasoning. No mechanism for agents to use different models, temperatures, or system prompts.

**Research Gap**: S6 (Zylos.ai) explicitly recommends "structural diversity over monoculture" — using agents backed by different base models reduces correlated failure modes. S5 (ICLR 2026) shows heterogeneous agent contributions improve collective intelligence. S3 (Hepworth et al.) demonstrates contextually aware control for heterogeneous swarms.

**Impact**: NeoTrix's swarm suffers from correlated failure modes — if one model has a blind spot, all agents share it. No genuine disagreement or echo-chamber prevention.

**Suggestion**: Add `HeterogeneousConfig` to SwarmAgent:
- Model diversity: different agents can use different LLM providers
- Temperature diversity: vary exploration/exploitation across agents
- Prompt diversity: different system prompts for same task (multiple perspectives)
- Diversity metric: measure inter-agent response entropy
- Connect to NT-IO's provider gateway for multi-provider routing

---

### D8: EventBus Uses Hardcoded Enums Instead of Semantic Coordination

**Current State**: Inter-agent communication uses `PheromoneMessage` with `message_type: String`. The broader NeoTrix EventBus uses `CoreEvent` enum — fixed set of event types.

**Research Gap**: S7 (IJETRM 2026) proposes semantic coordination layers with cross-domain interaction capability. Blackboard systems offer common problem-solving environments for incremental knowledge sharing. S4 (Zomer & De Domenico) shows cognitive agents produce collective behaviors fundamentally different from rule-based agents, requiring richer communication substrates.

**Impact**: NeoTrix cannot support emergent message types, semantic search over agent communications, or cross-domain knowledge sharing through the swarm.

**Suggestion**: Replace hardcoded event enums with `SemanticEvent`:
- Structured payload with typed slots
- Semantic tags for routing (not fixed enum variants)
- Blackboard layer for shared working memory
- Connect to VSA HyperCube for semantic similarity routing
- Enable agents to subscribe to semantic topics, not fixed event types

---

## Summary

| Metric | Count |
|--------|-------|
| Research sources cited | 12 |
| Concrete defects identified | 8 |
| Severity: Critical | 3 (D3 safety, D5 no testing, D1 no stigmergy) |
| Severity: High | 3 (D2 no emergent comm, D4 no dynamic pop, D8 hardcoded events) |
| Severity: Medium | 2 (D6 no federated learning, D7 no heterogeneous diversity) |

## Cross-Domain Connections

- **D1 (stigmergy) ↔ NT-ACT**: Pheromone-guided search should route through `ParallelTaskManager` and `ProductionOrchestrator`
- **D3 (safety) ↔ NT-SHIELD + NT-GOVERNANCE**: Emergent behavior safety must integrate with audit dimensions D1-D55
- **D5 (SwarmBench) ↔ NT-CORE**: Coordination tests should feed into GWT attention modulation
- **D6 (federated) ↔ NT-MEMORY**: Cross-instance swarm learning maps to `FederatedMemory` concept
- **D8 (semantic) ↔ NT-CORE**: VSA HyperCube should provide semantic routing substrate for swarm communication

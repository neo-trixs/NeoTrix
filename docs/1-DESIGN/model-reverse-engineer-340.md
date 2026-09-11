# Model Reverse Engineering — Cycle 340

**Date**: 2026-09-11
**Focus**: Multi-agent coordination architectures, efficient attention mechanisms, agent communication protocols, sparse decoding
**Sources**: arXiv (Sep 2026), ICML 2026, ACL 2026

---

## Paper/Model 1: ReActNet — Inference-Time Graph Engineering for Multi-Agent LLM Workflows

**Source**: arXiv:2609.05774 (Sep 4, 2026)
**Category**: Multi-Agent Orchestration via Temporal Workflow Graphs

### Core Mechanism
- **Task-conditioned temporal workflow graph**: Rather than optimizing a static topology, ReActNet synthesizes a temporal workflow graph that jointly specifies agent connectivity and edge-level communication semantics. Each graph snapshot corresponds to one reasoning stage, each edge carries a natural-language instruction specifying the message a source agent should provide to a target.
- **Graph compilation vs execution separation**: The compiled temporal graph is executed through structured message passing — agents update reasoning states by integrating previous states with messages from controller-assigned neighbors. A final aggregator synthesizes resulting states into the answer.
- **Training-free**: No reinforcement learning or gradient-based topology optimization required. The graph is compiled from the query and agent roles at inference time.
- **Consistent improvement**: Across knowledge reasoning, math, code generation, and GAIA-style tasks, ReActNet improves over fixed-topology and learned-topology baselines while maintaining competitive inference cost.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | Temporal workflow graph = GWT broadcast schedule. Each reasoning stage is a GWT cycle where salient information is broadcast to specialist modules. Edge instructions = GWT resonance signals directing information flow. | GWT should generate task-conditioned broadcast schedules — not fixed activation patterns but dynamic graphs per reasoning stage. ReActNet's edge instructions map to GWT's resonance-based routing. |
| **NT-MIND** | Graph compilation = SEAL pipeline's phase planning. Separating compilation from execution = separating SEAL planning from SEAL execution. Each SEAL phase could be a graph snapshot. | SEAL pipeline stages should be compilable as temporal graphs — phase-specific agent connectivity and communication semantics, executable through structured message passing. |
| **NT-ACT** | Structured message passing = EventBus with typed messages. Edge instructions specify not just what to send but why and how. Final aggregator = cross-domain synthesis. | EventBus messages should carry communication semantics (intent, expected response format) alongside content. Current EventBus is content-only. |

### Actionable Insight
**Separate graph compilation from graph execution.** ReActNet's key insight is that multi-agent coordination should be planned (compiled) separately from executed. NeoTrix's GWT currently routes attention in real-time; adding a compilation step — where the broadcast schedule is planned per-task before execution — would enable more efficient coordination. The temporal graph structure maps naturally to SEAL pipeline phases.

---

## Paper/Model 2: DeAR — Decentralized Agentic Reasoning via Capability Grounding

**Source**: arXiv:2608.17282 (Aug 18, 2026)
**Category**: Decentralized Multi-Agent Reasoning Framework

### Core Mechanism
- **Decentralized capability grounding**: Agents aren't assigned rigid labels (e.g., "Math Agent"). Instead, each agent grounds its identity in verifiable linguistic benchmarks from its model card. Query-dependent specialization emerges from capability matching, not role assignment.
- **Thought map navigation**: Agents navigate an Agentic Thought Map where nodes represent reasoning states and edges represent collaboration probabilities. A dynamic Collaboration Propensity Matrix encodes likelihood of beneficial interaction between agent pairs.
- **Topology update for adaptive error correction**: If a reasoning path fails, the system doesn't restart from scratch. Instead, it performs localized progressive backtracking — pruning failing edges step-by-step and re-navigating the adjusted topology.
- **No central coordinator**: Each agent autonomously evaluates potential utility of peering with every other agent. Local graph traversal replaces central routing.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE** | Thought map navigation = HyperCube associative recall. Collaboration propensity = VSA similarity scoring between domain capabilities. Decentralized routing = GWT without a central arbiter. | HyperCube should support dynamic adjacency — not fixed concept relationships but query-dependent collaboration probabilities between domains. |
| **NT-MIND** | Topology update = SEAL pipeline adaptive phase transitions. Progressive backtracking = experience-tree route table pruning. Capability grounding = skill node capability signatures. | SEAL pipeline phases should adaptively restructure based on intermediate results — not fixed phase sequences but dynamic transitions guided by capability matching. |
| **NT-ACT** | Decentralized agent selection = capability-based task routing. No central coordinator = EventBus with distributed routing decisions. | Task routing should be decentralized — each domain module evaluates its own capability match to incoming tasks, rather than a central dispatcher. |

### Actionable Insight
**Decentralize capability-based task routing.** DeAR eliminates the central coordinator by letting agents autonomously evaluate their fit for each task. NeoTrix's GWT currently acts as a central attention router; complementing it with decentralized capability grounding — where each domain module self-selects based on verifiable capability signatures — would reduce GWT bottleneck and enable emergent specialization.

---

## Paper/Model 3: BIGMAS — Brain-Inspired Graph Multi-Agent Systems

**Source**: arXiv:2603.15371 (Mar 2026, published ICML 2026)
**Category**: GWT-Inspired Multi-Agent Coordination

### Core Mechanism
- **Global Workspace Theory instantiation**: Specialized LLM agents organized as nodes in a dynamically constructed directed graph, coordinating exclusively through a centralized shared workspace. Directly implements GWT's dynamic coalition formation among distributed specialized processors.
- **Problem-adaptive GraphDesigner**: A meta-level agent constructs task-specific agent topologies per problem. Different problems yield different node compositions and topologies — routine tasks use compact pipelines, complex tasks recruit broader coalitions.
- **Global Orchestrator**: Leverages complete shared state for routing decisions, overcoming the local-view bottleneck of reactive approaches. All intermediate results are globally visible through the shared workspace.
- **Routing count as difficulty proxy**: The number of routing decisions by the Orchestrator functions as a natural proxy for instance-level difficulty — emerges without explicit scheduling logic, purely from global conditioning on workspace state.
- **Complementary to model-level reasoning**: Multi-agent architectural coordination provides gains orthogonal to model-level reasoning capability. BIGMAS improves both standard LLMs and LRMs, showing architectural organization is a fundamental performance determinant.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE** | BIGMAS IS GWT implemented for LLM agents. GraphDesigner = ConsciousnessTree's domain topology design. Shared workspace = EventBus with global state visibility. Orchestrator = GWT attention routing with full-state awareness. | ConsciousnessTree should dynamically construct domain interaction topologies per task — not fixed 6-layer architecture but adaptive coalitions. Routing count = system complexity metric. |
| **NT-MIND** | Dynamic coalition formation = SEAL pipeline adaptive phase topology. Task-specific graphs = domain module participation varies per SEAL cycle. | SEAL pipeline should recruit different domain modules per cycle based on task demands — not all 7 domains every time. |
| **NT-MEMORY** | Global shared workspace = KB with full cross-domain visibility. Every intermediate result globally visible = KB edges accessible from any domain. | KB should maintain global visibility — any domain's intermediate results should be queryable by any other domain during reasoning. |

### Actionable Insight
**Dynamic domain coalitions per task.** BIGMAS validates that GWT-inspired architecture works for LLM multi-agent systems. The key insight for NeoTrix: the 6-layer architecture should not be rigid. ConsciousnessTree should dynamically construct task-specific coalitions — some tasks need only NT-CORE + NT-MEMORY, others need all 7 domains. The GraphDesigner pattern maps to ConsciousnessTree's domain topology selection. Routing count as difficulty proxy provides a free complexity metric.

---

## Paper/Model 4: FFD — Faster Flash Decoding via Attention Sparsity

**Source**: arXiv:2609.00097 (Aug 31, 2026, ICML 2026)
**Category**: Hardware-Algorithm Co-Design for Efficient Long-Context Decoding

### Core Mechanism
- **Fused selector-computer kernel**: Replaces external metadata indices with content-aware scanning via low-bit quantization. The selector and computer are fused into a single CUDA kernel, eliminating the memory overhead of separate indexing.
- **Top-delta strategy**: Dynamically filters blocks to achieve distribution-adaptive sparsity without global synchronization. Blocks are filtered based on local attention score distribution, not global statistics.
- **Training-free and plug-and-play**: No model modification required. Drop-in replacement for standard attention decoding.
- **Results**: Up to 11.6x kernel-level speedup, scales to 256K context length, 2.37x end-to-end throughput improvement. Maintains model accuracy on RULER and LongBench.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE** | Attention sparsity = GWT's selective attention mechanism at the computation level. Top-delta filtering = salience-based attention allocation. Distribution-adaptive = context-dependent salience thresholds. | GWT should exploit attention sparsity — not all domain modules need full attention at every step. Top-delta strategy maps to GWT's ability to filter low-salience inputs. |
| **NT-IO** | Hardware-algorithm co-design = performance optimization for NT-IO's LLM inference pipeline. Fused kernel = optimized attention computation for provider abstraction layer. | LLM inference through NT-IO should leverage sparse attention optimizations for long-context tasks. Plug-and-play nature means no model changes needed. |

### Actionable Insight
**Exploit attention sparsity in GWT routing.** FFD proves that attention is intrinsically sparse — most computation is wasted on irrelevant tokens. GWT's salience-based routing should operate at the attention computation level, not just the information-routing level. Top-delta filtering provides a practical mechanism: only allocate full attention to the top-delta most salient domain modules per reasoning step.

---

## Paper/Model 5: Consilience — Conformally Calibrated Communication Control

**Source**: arXiv:2608.20564 (Aug 20, 2026)
**Category**: Certified Adaptive Communication for Multi-Agent Reasoning

### Core Mechanism
- **Conformal calibration**: Distribution-free, finite-sample guarantee on one-step regret of communication actions. At each discussion round, the controller's proposed action has bounded regret with marginal probability ≥ 1-α.
- **Communication interventions**: Four types — challenge, clarify, seek evidence, route. Each selected based on compact state capturing uncertainty, disagreement, evidence gain, redundancy, and premature consensus.
- **Acceptance mechanism**: Enforces the same guarantee for executed actions by replacing inadmissible proposals. Not just planning but execution certification.
- **Beats full-information baseline**: Sometimes surpasses a full-information baseline where every agent observes all evidence — certified adaptive communication is more valuable than more information.
- **Hidden-profile validation**: Tested on hidden-profile tasks where each agent holds only part of the evidence — the setting where communication coordination matters most.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | Communication interventions = GWT broadcast types (challenge/clarify/evidence/route). Compact state = GWT salience signal components (uncertainty, disagreement, evidence gain). Conformal guarantee = bounded regret on attention allocation decisions. | GWT salience should include uncertainty and disagreement signals alongside relevance. Communication type selection should be calibrated, not heuristic. |
| **NT-GOVERNANCE** | Acceptance mechanism = quality gates with formal guarantees. Conformal calibration = compliance verification with statistical confidence. Inadmissible proposal replacement = NT-GOVERNANCE policy enforcement. | Quality gates should have formal guarantees, not just heuristic checks. Conformal calibration provides a principled framework for governance thresholds. |
| **NT-SHIELD** | Certified actions = security guarantees on agent behavior. Regret bounds = worst-case risk bounds for tool calls. | NT-SHIELD should provide certified bounds on action risk, not just heuristic scanning. Conformal methods enable formal security guarantees. |

### Actionable Insight
**Calibrate communication with formal guarantees.** Consilience proves that certified adaptive communication outperforms both fixed schedules and unstructured debate — even surpassing full-information baselines. For NeoTrix: GWT routing should be conformally calibrated — each attention allocation decision comes with a bounded regret guarantee. This transforms GWT from heuristic routing to certified routing. The four intervention types (challenge/clarify/evidence/route) map to GWT broadcast semantics.

---

## Cross-Cutting Insights (Cycle 340)

### 1. Separation of Planning and Execution
Both ReActNet and BIGMAS separate graph compilation (planning) from graph execution. This maps to NeoTrix's need to separate SEAL phase planning from phase execution. Currently SEAL phases are statically defined; they should be dynamically compiled per-task.

### 2. Decentralization Complements Centralization
DeAR shows decentralized capability grounding outperforms centralized routing. BIGMAS shows centralized workspace outperforms decentralized memory. The resolution: decentralized capability self-selection + centralized shared state. NeoTrix should adopt this hybrid: domains self-select tasks (decentralized) but share state through KB (centralized).

### 3. Attention Sparsity is Intrinsic
FFD and BIGMAS both show attention is intrinsically sparse — only a small fraction of computation matters. GWT should exploit this: not all domains need full attention at every reasoning step. Top-delta filtering enables efficient routing.

### 4. Communication Needs Certification
Consilience shows that calibrated communication beats more information. For NeoTrix: the value is not in more cross-domain data but in better-structured, formally-certified communication between domains. GWT should certify its routing decisions.

### 5. Architecture is Orthogonal to Model Quality
BIGMAS demonstrates that multi-agent architectural coordination provides gains orthogonal to model-level reasoning. This validates NeoTrix's architectural approach — the 6-layer domain architecture provides value regardless of which LLM backbone is used.

## NeoTrix Implementation Priority

| Priority | Pattern | Paper | Target Component |
|----------|---------|-------|-----------------|
| P0 | Dynamic domain coalitions per task | BIGMAS | ConsciousnessTree topology |
| P0 | Separate planning from execution | ReActNet | SEAL pipeline |
| P1 | Decentralized capability self-selection | DeAR | GWT + domain modules |
| P1 | Conformally calibrated routing | Consilience | GWT salience |
| P2 | Attention sparsity exploitation | FFD | GWT attention allocation |
| P2 | Communication intervention types | Consilience | EventBus semantics |
| P3 | Routing count as difficulty proxy | BIGMAS | SelfModel complexity metric |

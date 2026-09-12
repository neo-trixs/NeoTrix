# Model Reverse Engineering — Cycle 392 (2026-09-12)

## 5 New Models/Papers

### 1. AgentInfer — Co-Design of Agent Inference Architecture and System
- **Paper**: arXiv:2512.18337v2 (Feb 2026)
- **Authors**: Weizhe Lin, Hui-Ling Zhen, Shuai Yang, Xian Wang, et al.
- **Key Idea**: Unified framework for end-to-end agent acceleration. Four synergistic components: AgentCollab (hierarchical dual-model reasoning — large model for planning, small model for execution), AgentSched (cache-aware hybrid scheduler), AgentSAM (suffix-automaton speculative decoding reusing multi-session semantic memory), AgentCompress (semantic compression distilling agent memory asynchronously). Forms a "Self-Evolution Engine" for long-horizon reasoning.
- **Results**: >50% reduction in ineffective token consumption. 1.8-2.5× overall speedup with preserved accuracy on BrowseComp-zh and DeepDiver benchmarks.
- **Core Pattern**: Hierarchical model delegation (large for planning, small for execution) + suffix-automaton memory reuse + async memory compression. Optimization target is task completion, not per-token throughput.
- **NeoTrix Mapping**:
  - **NT-CORE (GWT)**: AgentCollab = dual-model attention routing — expensive model for high-salience planning steps, cheap model for low-salience execution
  - **NT-IO (LLM)**: AgentSAM = suffix-automaton speculative decoding for multi-session KV-cache reuse
  - **NT-MEMORY**: AgentCompress = async memory distillation — semantic compression without disrupting active reasoning
  - **NT-MIND (SEAL)**: Self-Evolution Engine = behavior-level self-improvement through memory reuse
  - **Axiom A1 (Cost-Aware Routing)**: Dual-model delegation = routing cheap tasks to cheap models, expensive tasks to expensive models
  - **Axiom A2 (Context as Scarce Resource)**: AgentCompress = maintaining cognitive stability under context pressure

### 2. SparDA — Sparse Decoupled Attention with Forecast Projections
- **Paper**: arXiv:2606.04511 (Jun 2026)
- **Authors**: Yaosheng Fu, Guangxuan Xiao, Xin Dong, Song Han, Oreste Villa
- **Key Idea**: Introduces a fourth per-layer projection — Forecast — alongside Q, K, V. Forecast predicts which KV blocks the next layer will need, enabling lookahead selection that overlaps CPU-to-GPU prefetch with current-layer execution. Decoupled from attention query: one Forecast head per GQA group. Adds <0.5% parameters, trains only Forecast projections.
- **Results**: Up to 1.25× prefill speedup, 1.7× decode speedup over sparse-attention offload baseline. Up to 5.3× higher decode throughput than non-offload sparse baseline on single GPU.
- **Core Pattern**: Forecast projection = look-ahead KV-cache management. Predicts future attention needs to overlap prefetch with compute. GQA-compatible = minimal parameter overhead.
- **NeoTrix Mapping**:
  - **NT-CORE (GWT)**: Forecast projection = attention salience prediction — predicting which information will become salient in future layers
  - **NT-IO (LLM)**: GQA-compatible sparse attention = drop-in inference optimization
  - **NT-PHYSICAL**: CPU-GPU prefetch overlap = resource utilization optimization
  - **ConsciousnessTree**: Per-layer Forecast mirrors per-branch awareness prediction — each branch forecasts what next branch will need
  - **Axiom A2 (Context as Scarce Resource)**: Lookahead prefetching = maximizing utility of limited GPU KV-cache

### 3. Agent-Radar — Attention Steering for Multi-Agent Communication
- **Paper**: arXiv:2605.30136 (May 2026)
- **Authors**: (Multi-institution collaboration)
- **Key Idea**: Training-free context management that dynamically steers each agent's attention toward relevant context. Uses temporal and spatial decay mechanism to score sentence-level context relevance. Instead of compressing or pruning history, selectively amplifies attention to key instructions, critical constraints, and useful intermediate evidence. Integrates Selective Prompt Anchoring (SPA) as lightweight backend attention steering.
- **Results**: Consistent improvements across five benchmarks using three base LLMs. Preserves full transcript and topology while steering attention. No training required.
- **Core Pattern**: Attention steering (not compression/pruning) — amplify relevant context weights during inference. Temporal decay (recent = more relevant) + spatial decay (position-based relevance). Training-free = plug-and-play.
- **NeoTrix Mapping**:
  - **NT-CORE (GWT)**: Attention steering = salience-weighted broadcast — relevant context gets amplified attention weight
  - **NT-MEMORY**: Temporal + spatial decay = experience relevance scoring (recent + contextually close = more relevant)
  - **NT-IO (LLM)**: Training-free = drop-in optimization for existing agents
  - **NT-FEEL**: Social emotion context steering — attention to emotional cues in multi-agent dialogue
  - **ConsciousnessTree**: Per-agent attention steering mirrors per-branch awareness modulation

### 4. LLM Reasoning Is Latent, Not Chain of Thought
- **Paper**: arXiv:2604.15726 (Apr 2026)
- **Authors**: Wenshuo Wang (South China University of Technology)
- **Key Idea**: Position paper arguing LLM reasoning should be studied as latent-state trajectory formation, not surface chain-of-thought. Separates three often-confounded factors: S (surface CoT text), Z (task-relevant latent-state trajectory), and compute budget. Three hypotheses: H1 (reasoning is primarily latent-state dynamics), H2 (reasoning is primarily surface CoT), H0 (gains are just more serial compute). Current evidence supports H1 — latent states carry the actual reasoning commitments.
- **Results**: Compute-audited worked exemplars factorize surface traces, latent interventions, and matched budget expansions. H1 most strongly supported as default working hypothesis.
- **Core Pattern**: Reasoning lives in latent states, not surface text. Surface CoT is a projection of latent reasoning, not the reasoning itself. Evaluations should disentangle surface traces, latent states, and compute.
- **NeoTrix Mapping**:
  - **NT-CORE (E8 Hexagram)**: Latent-state reasoning = E8 states as latent reasoning trajectories, not surface-level CoT traces
  - **NT-CORE (GWT)**: Attention broadcasting operates on latent states, not surface tokens
  - **NT-MIND (distillation)**: Distillation should target latent states, not surface CoT text
  - **ConsciousnessTree**: Consciousness = latent-state trajectory formation, not verbal self-report
  - **NT-FEEL**: Emotional reasoning is latent (affects decisions) not surface (emotional words in CoT)
  - **SEAL (quality gates)**: Evaluate skill quality by latent-state impact, not surface-level trace quality

### 5. AgensFlow — Coordination-Policy Substrate for Multi-Agent Systems
- **Paper**: arXiv:2605.27466 (May 2026)
- **Authors**: Nicole Koenigstein
- **Key Idea**: Treats multi-agent coordination as an online policy-learning problem under partial observability. The system doesn't observe full user intent, latent task difficulty, evidence quality, or model reliability directly. Learns a policy graph over skills, models, and topology actions. Reward-signal auditability as first-class design. Coordination policy is learned online from repeated trajectories rather than fixed statically.
- **Results**: Learned, auditable routing improves coordination-heavy multi-agent workflows over static wiring. Use-case-conditioned coordination outperforms fixed topologies.
- **Core Pattern**: Multi-agent coordination as partially observable sequential decision process. Online policy learning over skill/model/topology choices. Auditable routing decisions. Static pipelines → learned routing.
- **NeoTrix Mapping**:
  - **NT-ACT (orchestration)**: AgensFlow = learned coordination policy for multi-agent task routing
  - **NT-CORE (GWT)**: Policy graph = attention routing decisions learned from trajectory outcomes
  - **NT-MIND (self-evolution)**: Online policy learning = behavioral self-improvement through accumulated coordination experience
  - **NT-MEMORY**: Trajectory-based learning = experience-tree branch optimization from coordination outcomes
  - **NT-SHIELD**: Auditable routing = explainable agent coordination decisions
  - **Axiom A1 (Cost-Aware Routing)**: Model binding selection per role = cost-optimized multi-agent delegation

## Cross-Paper Synthesis (Cycle 392)

### Emerging Theme: Latent-State Optimization
Three of five papers (AgentInfer, LLM Reasoning Is Latent, Agent-Radar) converge on the insight that **optimizing latent states matters more than optimizing surface representations**. AgentInfer compresses memory at the semantic level, Wang argues reasoning is latent, and Agent-Radar steers attention at the latent relevance level. This aligns with NeoTrix's E8 hexagram model where reasoning states are geometric, not textual.

### Emerging Theme: Predictive Resource Management
SparDA's Forecast projection and AgentInfer's AgentSAM both predict future resource needs to overlap prefetch with compute. This predictive resource management pattern maps to NeoTrix's Axiom A2 (Context as Scarce Resource) — anticipating future context needs to maximize utilization of limited resources.

### Emerging Theme: Learned Coordination Over Static Pipelines
AgensFlow and AgentInfer both demonstrate that learned coordination policies outperform static wiring. This reinforces NeoTrix's SEAL pipeline (self-evolving architecture loop) — coordination patterns should evolve from experience, not be hardcoded.

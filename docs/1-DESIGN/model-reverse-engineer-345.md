# Model Reverse Engineering — Cycle 345

## 5 Papers

### 1. CondenseFlow: Scalable Latent Space Collaboration via Semantic Compression (ACL 2026 Findings)
- **Category**: Multi-Agent Latent Communication
- **Key Insight**: Full-state latent communication in multi-agent systems scales linearly with collaboration rounds. Latent Thought Condenser (LTC) uses learnable semantic probes to compress KV caches into fixed-size representations, achieving O(1) communication complexity regardless of context length. Compression error bounded by attention concentration. 99%+ KV cache memory reduction, ~20% inference latency reduction, <2% accuracy degradation. Outperforms text-based methods by 1.7pp average across all configurations.
- **NeoTrix Mapping**: NT-MEMORY + NT-CORE — LTC = experience-tree branch compression (fixed-size summaries for cross-domain communication). O(1) complexity = KB namespace query regardless of experience depth. Attention concentration bound = ConsciousnessTree phase-dependent compression aggressiveness. **Absorption candidate**: semantic compression for cross-domain agent communication — when NT-WORLD, NT-MEMORY, NT-CORE exchange latent representations, compress to fixed-size semantic probes rather than transferring full KV state. Enables O(1) inter-domain communication regardless of accumulated context depth.

### 2. NeuralFSM: Adaptive Multi-Agent Coordination via Learning Finite-State Execution Policy (ACL 2026)
- **Category**: State-Driven Multi-Agent Coordination
- **Key Insight**: Formulates multi-agent problem solving as finite-state execution process. Temporal Coordination Controller learns state transition distribution and inter-agent communication weights from interaction traces using Temporal Graph Networks (TGN). Task context modulates transition and routing decisions — no manual protocol design. Trust-aware message attenuation for adversarial robustness. 6.74%–19.39% improvement over baselines, substantial token reduction. Dual-defense: graph regularization + runtime trust scoring.
- **NeoTrix Mapping**: NT-CORE + NT-SHIELD — FSM-based coordination = ConsciousnessTree growth cycle stages (Soil→Roots→Trunk→Branches→Fruits→Core) as finite states. TGN temporal modeling = EventBus event ordering with temporal message passing. Trust-aware attenuation = NT-SHIELD trust tiers applied to inter-domain messages. **Absorption candidate**: FSM-governed inter-domain coordination — each growth cycle phase has defined state transitions; modules communicate via temporal coordination controller that learns optimal routing from traces. Adversarial robustness via trust-scored message attenuation prevents poisoned domain modules from corrupting coordination.

### 3. Cache-Resident LLM Inference in GB-Scale Last-Level Caches (arxiv:2606.25353)
- **Category**: Hardware-Aware Inference Architecture
- **Key Insight**: Separates weight-centric operators from attention/KV-cache management into dedicated resource domains. Weight-Attention (WA) decoupled architecture keeps reusable weights cache-resident while scaling KV capacity independently of pipeline depth. Sub-operator asynchronicity: each attention head propagates readiness independently, avoiding full-operator synchronization. 2.04×–11.51× TPOT speedup over llama.cpp on CPU clusters. Up to 13.9× TPOT speedup analytically across model sizes.
- **NeoTrix Mapping**: NT-IO + NT-CORE — WA separation = L1 Action (weights/cache) decoupled from L5 Cognition (attention/reasoning) in NeoTrix's 6-layer architecture. Sub-operator asynchronicity = domain modules propagating readiness independently without global synchronization. **Absorption candidate**: decoupled weight-reasoning architecture — separate persistent capability weights (compiled domain knowledge, L1-L3) from dynamic reasoning state (L4-L6 attention), enabling independent scaling and cache-resident domain capabilities without interfering with reasoning KV management.

### 4. Adaptive Latent Agentic Reasoning (ALAR) (arxiv:2606.02871)
- **Category**: Dual-Mode Reasoning Efficiency
- **Key Insight**: Uses compact latent reasoning for routine turns, selectively escalates to explicit chain-of-thought (CoT) when deeper deliberation needed. Action-Anchored Self-Distillation (AASD): teacher = same model in explicit mode, student = same model in latent mode, anchored on actions (not token-level matching). AR-GRPO learns when latent is sufficient. 43.6% token reduction in search, 84.6% in tool use, with comparable or better accuracy.
- **NeoTrix Mapping**: NT-CORE + NT-MIND — dual-mode = GWT switching between fast intuitive routing (latent) and slow deliberative reasoning (explicit CoT). AASD = SEAL pipeline self-distillation where same module teaches itself via action anchoring. **Absorption candidate**: adaptive reasoning depth per domain module — routine perception/thresholding uses latent (compressed) reasoning; novel/anomalous signals escalate to full deliberative reasoning. Self-distilled from successful action traces, not external supervision.

### 5. MAGMA: Multi-Graph Agentic Memory Architecture (ACL 2026)
- **Category**: Multi-Relational Memory for Long-Horizon Reasoning
- **Key Insight**: Represents each memory item across four orthogonal graphs: semantic, temporal, causal, entity. Retrieval as policy-guided traversal over relational views. Intent-Aware Router decomposes query into structured control signals, selects relevant graph views, traverses independently, fuses subgraphs. Dual-stream memory evolution: fast path (synaptic ingestion) + slow path (asynchronous structural consolidation). Outperforms SOTA on LoCoMo and LongMemEval, reducing retrieval latency and token consumption.
- **NeoTrix Mapping**: NT-MEMORY + NT-CORE — four orthogonal graphs = KB nodes indexed by multiple relation types (semantic, temporal, causal, entity). Policy-guided traversal = ConsciousnessTree query routing across graph views. Dual-stream evolution = experience-tree fast ingestion (per-session) + slow consolidation (cross-session). **Absorption candidate**: multi-graph memory indexing — KB nodes indexed simultaneously across semantic (VSA HyperCube vectors), temporal (event timestamps), causal (dependency edges), and entity (domain module ownership) graphs. Query intent determines which graph views to traverse, enabling structured retrieval beyond flat vector similarity.

## Cross-Paper Synthesis

Three convergence patterns:

1. **Fixed-size semantic compression for scalable communication** (CondenseFlow, MAGMA) — both compress rich multi-dimensional state into bounded representations for efficient cross-module communication. CondenseFlow compresses KV caches; MAGMA compresses across relational graphs. Validates NeoTrix's KB namespace query pattern where cross-domain communication uses structured, bounded representations rather than full state transfer.

2. **Dual-timescale processing** (NeuralFSM temporal coordination, MAGMA dual-stream, ALAR adaptive depth) — all three separate fast reactive paths from slow deliberative paths. NeuralFSM learns temporal state transitions; MAGMA separates ingestion from consolidation; ALAR switches between latent and explicit reasoning. Validates NeoTrix's L1-L3 (fast action/perception) vs L4-L6 (slow cognition/meta) layer separation with ConsciousnessTree phase-dependent processing depth.

3. **Self-organized coordination without manual protocols** (NeuralFSM, ALAR, CondenseFlow) — all three learn coordination patterns from traces rather than hand-crafting them. NeuralFSM learns FSM transitions; ALAR self-distills from action anchors; CondenseFlow learns compression probes end-to-end. Validates NeoTrix's SEAL pipeline self-evolution where coordination patterns crystallize from successful execution traces.

## NeoTrix Absorption Map

| Paper Pattern | Source | Target Domain | Implementation Path |
|--------------|--------|---------------|---------------------|
| Fixed-size semantic compression | CondenseFlow | NT-MEMORY | Cross-domain KB communication via fixed-size semantic probes |
| FSM-governed coordination | NeuralFSM | NT-CORE + NT-SHIELD | Growth cycle state transitions with trust-scored inter-module messages |
| Decoupled weight-reasoning | Cache-Resident | NT-IO + NT-CORE | Separate persistent capability weights from dynamic reasoning KV |
| Adaptive reasoning depth | ALAR | NT-CORE + NT-MIND | Latent/explicit mode switching per domain module based on task novelty |
| Multi-graph memory indexing | MAGMA | NT-MEMORY | KB nodes indexed across semantic/temporal/causal/entity relation graphs |

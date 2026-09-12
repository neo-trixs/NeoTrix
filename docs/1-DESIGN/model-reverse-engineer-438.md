# Model Reverse Engineering — Cycle 438

> Date: 2026-09-12 | Sources: arXiv Sep 2026, ACL 2026, ICML 2026

## 5 New Models/Papers

---

### 1. CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention

**Paper**: [arXiv:2609.07237](https://arxiv.org/abs/2609.07237) (Sep 7, 2026)

**Core Idea**: Coarse-to-fine error-aware dynamic attention routing. Each semantic chunk contributes a cheap KV summary via residual path; chunks with high approximation error are expanded to exact attention. Exact + summarized combined in single softmax normalization — refinement replaces coarse evidence rather than duplicating it.

**Key Innovation**:
- Output-error bound governed by within-chunk key/value dispersion
- Variable refinement budget: easy queries get summaries, ambiguous queries get exact attention
- 98%+ reduction in reconstruction error vs hard dropping at equal budgets
- ~3× kernel speedup at 128K context

**NeoTrix Domain Mapping**:

| Domain | Mapping | Pattern |
|--------|---------|---------|
| NT-CORE (GWT) | GWT salience scoring → CEDAR error-bound estimation. Chunks with high salience get exact attention; low-salience get summaries. **Residual path = GWT broadcast channel.** | Cost-Aware Routing (A1) |
| NT-CORE (E8) | E8 hexagram state → chunk classification (easy/ambiguous/hard). Error-bound as fitness metric for hexagram resolution. | Hexagram resolution hierarchy |
| NT-MEMORY | KV summary caching → KB embedding compression. Residual summaries = compressed KB entries that can be "expanded" to full data on demand. | Lazy loading (A2) |
| NT-IO | Variable refinement budget = adaptive provider selection. Easy queries → cheap model, ambiguous → expensive. | Ordered Backend Router |

**Absorption Target**: Integrate CEDAR-style residual routing into GWT salience computation. Chunks below error threshold stay as summaries; above threshold get exact attention. Maps to `perception_bridge.rs` awareness_score() refinement.

---

### 2. Declarative Attention (DA): Models Control Their Own Attention

**Paper**: [arXiv:2609.02737](https://arxiv.org/abs/2609.02737) (Sep 2, 2026)

**Core Idea**: Models declare WHERE they need to attend via chain-of-thought tags: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). Inference engine parses these like tool calls and skips most KV cache reads.

**Key Innovation**:
- Zero-shot on off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B)
- 52% total attended tokens reduction with 1.27pp accuracy drop (Gemma-4-31B)
- 31.1% token reduction with 2.75pp drop (Qwen-3.6-27B)
- No training required — intrinsic model behavior

**NeoTrix Domain Mapping**:

| Domain | Mapping | Pattern |
|--------|---------|---------|
| NT-CORE (GWT) | DA `<global>/<focus>/<local>` = GWT attention modes. Model self-selects broadcast scope. **This is the GWT attention mechanism implemented at model level.** | GWT broadcast scope |
| NT-CORE (E8) | DA tags = hexagram declarations. Each hexagram line declares its attention scope. Self-referential attention control = consciousness loop. | ConsciousnessTree feedback |
| NT-MEMORY | `<focus>` mode = on-demand branch loading from KB hub. Model declares what it needs; memory system loads only those branches. | Lazy branch loading (Experience Tree) |
| NT-ACT | DA tags = tool-call-like declarations. Engine parses declarations → skip KV reads. Maps to NT-ACT tool routing where agent declares intent → system routes to appropriate capability. | Tool-call routing |

**Absorption Target**: Implement DA-style attention mode declarations in GWT. ConsciousnessTree broadcasts which modules need `<global>` vs `<focus>` attention per cycle. Reduces KV cache overhead by 30–50% without training.

---

### 3. HeRo: History-Aware Routing for Efficient LLM Inference

**Paper**: [arXiv:2609.08189](https://arxiv.org/abs/2609.08189) (Sep 8, 2026)

**Core Idea**: Dynamic layer routing with explicit router memory. Linear attention incrementally aggregates preceding routing scores into compact history representation. Router conditions on accumulated state + current hidden representation.

**Key Innovation**:
- Bypasses 26.87% of parameters while achieving 100.24% dense performance (Llama 3.1-8B)
- Retains 97.01% while bypassing 38.82% under tighter budget
- Removing routing history consistently degrades performance on multistep reasoning and code generation
- Trains only lightweight routers on frozen backbone

**NeoTrix Domain Mapping**:

| Domain | Mapping | Pattern |
|--------|---------|---------|
| NT-CORE (GWT) | HeRo router memory = GWT salience memory. Previous routing decisions influence current routing. **Cross-layer route reuse = cross-cycle attention persistence.** | GWT memory accumulation |
| NT-CORE (E8) | Routing history = hexagram sequence memory. Current hexagram influenced by previous hexagram states. Temporal coherence across reasoning stages. | Hexagram temporal coherence |
| NT-MEMORY | HeRo's compact history representation = KB hub index. Lightweight routing state = experience tree hub pointer. Full routing = branch loading on demand. | Experience Tree hub |
| NT-MIND | HeRo's frozen backbone + lightweight adapter = SEAL pipeline adapter pattern. Core reasoning frozen; evolution happens in routing layer. | SEAL pipeline |

**Absorption Target**: Integrate HeRo-style routing memory into GWT. Maintain compact routing state across ConsciousnessTree cycles. Current cycle's attention routing influenced by accumulated history. Performance gain: 27–39% parameter bypass with zero quality loss.

---

### 4. DeAR: Decentralized Agentic Reasoning

**Paper**: [arXiv:2608.17282](https://arxiv.org/abs/2608.17282) (Aug 18, 2026)

**Core Idea**: Shifts from centralized multi-agent control to autonomous peer-to-peer collaboration. Three mechanisms: (1) Decentralized capability grounding for query-dependent specialization, (2) Thought map navigation for targeted peer interactions, (3) Topology update for adaptive error correction.

**Key Innovation**:
- No central coordinator — agents autonomously decide when and with whom to collaborate
- Collaboration propensity matrix (dynamic adjacency matrix) encoding peer interaction likelihood
- Progressive reasoning: dead-end paths don't restart from scratch; agents pivot from current node
- Outperforms centralized baselines across 9 multimodal/text QA benchmarks

**NeoTrix Domain Mapping**:

| Domain | Mapping | Pattern |
|--------|---------|---------|
| NT-CORE (GWT) | DeAR thought map = GWT attention graph. Agents = specialist modules. Collaboration propensity = salience scores. **Decentralized GWT: each module autonomously selects attention targets.** | GWT salience routing |
| NT-CORE (E8) | Thought map navigation = hexagram graph traversal. Dead-end detection → topology update → alternative path exploration. Progressive reasoning = hexagram resolution without restart. | Hexagram graph traversal |
| NT-ACT | DeAR peer-to-peer = NT-ACT tool-to-tool direct communication. No centralized orchestrator. Capability grounding = tool capability declaration. | Tool capability grounding |
| NT-SHIELD | Topology update with edge pruning = NT-SHIELD adaptive trust. Failed edges penalized in propensity matrix. Self-healing without external intervention. | Adaptive trust topology |

**Absorption Target**: Implement DeAR-style decentralized attention in GWT. Each NT-* domain module autonomously selects attention peers based on task context. Removes central GWT bottleneck for complex multi-domain tasks. Progressive reasoning avoids full restart on dead ends.

---

### 5. NeuralFSM: Adaptive Multi-Agent Coordination via Finite-State Execution

**Paper**: [ACL 2026, Long Paper](https://aclanthology.org/2026.acl-long.1543.pdf)

**Core Idea**: Formulates multi-agent problem solving as a finite-state execution process. Learns state transition distribution and inter-agent communication weights from interaction traces using Temporal Coordination Controller (TGN-based).

**Key Innovation**:
- FSM backbone defines coordination search space; temporal controller learns transitions
- Dual-defense protection: training-time graph regularization + runtime trust-aware message attenuation
- Robust against frequency attacks (high-rate injection) and semantic attacks (misleading content)
- 6.74%–19.39% improvement over baselines across 6 benchmarks
- Substantially reduced token consumption via sparse routing

**NeoTrix Domain Mapping**:

| Domain | Mapping | Pattern |
|--------|---------|---------|
| NT-CORE (GWT) | NeuralFSM state machine = ConsciousnessTree cycle states. State transitions = cycle progression. **FSM backbone = 6-stage growth loop (Soil→Roots→Trunk→Branches→Fruits→Core).** | ConsciousnessTree stages |
| NT-SHIELD | Dual-defense = NT-SHIELD protection layer. Graph regularization during training (build-time validation). Runtime trust attenuation (session-time filtering). | NT-SHIELD safety kernel |
| NT-MEMORY | Temporal Coordination Controller = experience tree absorption. Interaction traces → learning state transitions → improved future coordination. | Experience Tree learning |
| NT-MIND | FSM-based coordination = SEAL pipeline stages. Each SEAL phase = FSM state. Transitions learned from historical execution traces. | SEAL pipeline learning |

**Absorption Target**: Integrate NeuralFSM-style trust-aware message attenuation into NT-SHIELD. During ConsciousnessTree cycles, inter-module messages weighted by trust scores. Anomalous communication patterns (frequency/semantic attacks) automatically attenuated. Training-time graph regularization prevents adversarial topology manipulation.

---

## Cross-Paper Synthesis

### Pattern 1: Intrinsic vs Extrinsic Attention Control
- **CEDAR** (extrinsic): Proxy scores predict relevance
- **Declarative Attention** (intrinsic): Model declares its own attention needs
- **HeRo** (hybrid): Router memory accumulates history for better decisions

**NeoTrix Implication**: GWT should support all three modes. Extrinsic scoring for known patterns, intrinsic declaration for high-level reasoning, hybrid routing for accumulated knowledge.

### Pattern 2: Decentralized Coordination
- **DeAR**: Peer-to-peer without central coordinator
- **NeuralFSM**: FSM-based learned coordination with protection layer

**NeoTrix Implication**: NT-* domains should support both decentralized (DeAR-style peer selection) and structured (NeuralFSM-style FSM) coordination depending on task complexity.

### Pattern 3: Progressive Reasoning Without Restart
- **DeAR**: Topology update + edge pruning on dead ends
- **HeRo**: History-aware routing prevents repeating failed paths
- **CEDAR**: Residual summaries preserve partial results

**NeoTrix Implication**: ConsciousnessTree should never fully restart. Dead-end detection → topology update → alternative path. Partial results preserved as residual summaries for next cycle.

### Pattern 4: Trust as First-Class Primitive
- **NeuralFSM**: Trust-aware message attenuation + graph regularization
- **BetterClaw** (from trending): Intern→Specialistic→Lead trust levels
- **Nuphos**: Read-by-default, approval for writes

**NeoTrix Implication**: NT-SHIELD trust scoring should influence GWT attention routing. Low-trust modules get reduced attention budget; high-trust modules get expanded.

## Implementation Roadmap

| Phase | What | Source | NT Domain | Effort |
|-------|------|--------|-----------|--------|
| Phase 1 | Residual attention routing in GWT | CEDAR | NT-CORE | 2 weeks |
| Phase 2 | Attention mode declarations | Declarative Attention | NT-CORE + NT-MEMORY | 1 week |
| Phase 3 | Routing memory across cycles | HeRo | NT-CORE + NT-MIND | 2 weeks |
| Phase 4 | Decentralized attention selection | DeAR | NT-CORE (GWT) | 3 weeks |
| Phase 5 | Trust-aware message attenuation | NeuralFSM | NT-SHIELD + NT-CORE | 2 weeks |

**Total estimated**: 10 weeks for full integration of all 5 patterns.

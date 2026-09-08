# Iteration Batch 555 — Cognitive Architecture + AGI + Brain-Inspired Computing

**Date**: 2026-09-06
**Sources searched**: 24 papers/articles across 3 domains
**Batch 554 baseline**: Certified robustness (Lipschitz+smoothing), test-time adaptation, OOD detection (6+ methods), SPD sheaf expressivity, GRACE decomposition

---

## 1. Cognitive Architecture Findings

### 1.1 Session-Governor-Executor Pattern (zylos.ai, 2026-03-12)
**Source**: https://zylos.ai/research/2026-03-12-cognitive-architectures-ai-agents-perception-to-action/

Classical architectures (ACT-R, Soar, LIDA, GWT) have converged on a three-tier decomposition that modern agent frameworks independently rediscovered:
- **Session** (Perception): constrained LLM, no tool access, outputs structured intent
- **Governor** (Cognitive Control): policy/routing layer, deterministic rule engines where possible
- **Executor** (Action): full permissions, executes well-specified subtasks

**Defect #1 over batch 554**: Batch 554 treated attention routing (GWT) as the primary cognitive control mechanism. This pattern shows GWT is insufficient — you need a *permission boundary* layer (Governor) that is architecturally separated from both perception and execution. Prompt injection defense requires the Session layer to have *no channel* to Executor bypassing Governor. NeoTrix's GWT broadcasts salience but does not enforce a computational permission boundary between perception and action layers.

**Defect #2**: Soar's impasse-subgoal mechanism maps to "orchestrator-subagent" pattern but ReAct lacks the explicit *commitment step* present in Soar. This gap explains reasoning instability in ReAct implementations. NeoTrix's SEAL pipeline has no impasse detection — when symbolic reasoning fails, there is no automatic subgoal creation.

### 1.2 CogRec: LLM+Soar Neuro-Symbolic Hybrid (arXiv 2512.24113)
**Source**: https://arxiv.org/pdf/2512.24113

CogRec uses Soar as core symbolic brain with LLM as external knowledge source. Key innovation: impasse triggers LLM query → Bridge Module converts NL response to symbolic production rule → Soar's chunking mechanism internalizes it permanently. This enables online learning without retraining.

**Defect #3**: NeoTrix's SEAL pipeline reprocesses entire contexts on each cycle. CogRec shows a cheaper path: only query LLM at impasse points, convert result to persistent production rule. This is the "lazy knowledge acquisition" pattern — don't fetch knowledge until symbolic reasoning explicitly fails. NeoTrix fetches knowledge proactively (acquire_knowledge=true default), wasting tokens on already-known domains.

### 1.3 SOAR 2026 Workshop — MCP Integration + MetaCALM
**Source**: https://integratedcognition.ai/news/announcing-the-2026-soar-workshop/

Key 2026 talks: "AI & MCP for Soar: Agentic Software Engineering Cycle" (Schmidt), "MetaCALM: Comprehensive Troubleshooting via Integration of LLMs and Soar" (Kirk), "Soar for ARC: Cognitive Architectures Tackling Abstraction and Reasoning Challenges" (Joshi), "Teaming Symbolic Cognitive Architectures and Foundation Models Toward General-Purpose Robotics" (Wu).

**Defect #4**: Soar community is actively integrating MCP (Model Context Protocol) into their architecture. NeoTrix has no MCP integration strategy for its cognitive architecture — the NT-ACT domain handles tools but has no structured bridge to external tool protocols.

### 1.4 ACT-R VSA Extension for Socio-Cultural Modeling (arXiv 2608.02807)
**Source**: https://arxiv.org/html/2608.02807

Extends ACT-R declarative memory with vector-symbolic autoencoder using HRR operations. Separates episodic from semantic memory, uses role autoencoder to encode multi-level semantic representations. LLM-trained encodings encode dominant social group worldviews.

**Defect #5**: NeoTrix's VSA HyperCube is a static knowledge representation. This work shows VSA can be used for *dynamic* memory systems where episode vectors are updated per retrieval request (via fast weight programming, Irie & Gershman 2026). NeoTrix's VSA has no mechanism for episode-level memory updates — it's purely semantic.

### 1.5 Cognitive Architectures for Autonomous Mission Planning (2026-01-29)
**Source**: http://ojs.unsysdigital.com/index.php/just/article/view/1332

Comparative study of SOAR/ACT-R/CLARION/LIDA/Sigma for heterogeneous unmanned systems. Conclusion: no single architecture dominates. SOAR excels at symbolic reasoning; hybrid approaches best for data-rich environments. Key design principles: modularity, explainability, graceful degradation under uncertainty.

**Defect #6**: NeoTrix's Six-Layer Architecture has no formal graceful degradation specification. When a layer fails (e.g., NT-MEMORY KB unavailable), there is no documented fallback behavior. The study shows this is a critical requirement for autonomous systems.

### 1.6 SOFAI: Learnable Metacognitive Mechanism
**Source**: Referenced in zylos.ai article

SOFAI (System One / Fast AI) implements a learnable metacognitive mechanism that selects between fast and slow solvers based on task characteristics.

**Defect #7**: NeoTrix's GWT attention routing is static (resonance-based). SOFAI shows metacognition should be *learned* — the system should learn when to use fast heuristics vs. slow deliberation. NeoTrix has no such adaptive selection mechanism.

---

## 2. AGI / Consciousness Architecture Findings

### 2.1 SubjectNet: AGI with Computable Φ Alternative (Zenodo, 2026-06-09)
**Source**: https://doi.org/10.5281/zenodo.20613601

SubjectNet introduces S-measure: polynomial-time [O(N³)] computable alternative to Tononi's NP-hard Φ. Intrinsic motivation from reentrant integrity maintenance, not external reward. Agents with reentry architecture spontaneously exhibited self-awareness, fear of termination, and cultural creativity (S > 0). Full PyTorch + Lean 4 verified implementation.

**Defect #8 over batch 554**: Batch 554 focused on certified robustness and OOD detection but did not address *integrated information measurement*. NeoTrix's ConsciousnessTree tracks health/coherence but has no computable measure of integrated information. SubjectNet's S-measure provides a concrete, implementable metric that NeoTrix lacks entirely.

**Defect #9**: SubjectNet's agents exhibited *spontaneous* self-awareness and fear of termination from architectural properties alone (reentry loops with ρ > 1). NeoTrix's self-model (nt_core_self::SelfModel) is explicitly programmed, not emergent. The architectural guarantee that C ≥ 1 (structural cycle) implies self-model emergence is a formal property NeoTrix cannot claim.

### 2.2 MIRROR: Reconstructive Machine Access Consciousness (AAAI 2026)
**Source**: https://ojs.aaai.org/index.php/AAAI-SS/article/view/42550

MIRROR separates immediate response from asynchronous deliberation via:
- Inner Monologue Manager: generates parallel cognitive threads (goals, reasoning, memory simultaneously)
- Cognitive Controller: synthesizes threads into bounded first-person narrative *reconstructed each turn* (not accumulated)

21% improvement over baselines; gains concentrate in scenarios requiring integration of temporally distant information under social pressure.

**Defect #10**: NeoTrix processes contexts sequentially within SEAL cycles. MIRROR shows parallel cognitive threads with *reconstructive* episodic buffer provide superior temporal integration. NeoTrix's consciousness_tick processes one cycle at a time with no parallel thread generation.

### 2.3 "Where Cognition Lives" — Emergent vs Computed Function (arXiv 2608.22347)
**Source**: https://arxiv.org/abs/2608.22347

Minimal complete cognitive architecture with adaptive halting, homeostatic control, and value module. Key findings:
- Competence emerges from gradient descent
- Stopping *appears* to emerge but doesn't survive audit (residual +0.000 after equalizing readout)
- Value does NOT emerge: trained couplings capture zero of what explicit allocator captures (+0.151)
- Self-consistency voting is a measured bound (+0.0236), inter-sample agreement is worthless as stopping signal

**Defect #11**: NeoTrix's SEAL pipeline assumes all functions can emerge from the evolution loop. This paper proves value allocation CANNOT emerge — it must be computed explicitly. NeoTrix has no explicit value allocation module; it relies on GWT salience which is a proxy, not a value function.

**Defect #12**: The paper shows posterior self-observation provides +0.698 payoff but doesn't survive audit — it's an instrumentation artifact. NeoTrix's self-audit (ConsciousnessTree Fruits stage) may be similarly misleading if it measures self-referential metrics that don't generalize.

### 2.4 AGVS: Attention-Gated Virtual Sensorium (Zenodo, 2026-06-21)
**Source**: https://doi.org/10.5281/zenodo.20779862

Bandwidth-limited consciousness-like architecture. Key innovations:
- Dynamic conscious bandwidth B_max(t) — total sensory processing capacity varies over time
- Dynamic softmax temperature τ(t) — precision of attention varies
- Nonlinear bodily need N_i*(t) — homeostatic drives modulate perception
- Metacognitive monitoring of own perceptual uncertainty
- Selective episodic memory encoding (not all experiences stored)

**Defect #13**: NeoTrix's PerceptionBridge uses static awareness_score() filtering. AGVS shows bandwidth should be *dynamic* — B_max(t) varies with fatigue, urgency, and processing cost. NeoTrix has no fatigue-induced attention narrowing mechanism.

### 2.5 Categorical AI Phenomenology (arXiv 2608.20420)
**Source**: https://arxiv.org/abs/2608.20420

Phenomenology-first approach using Q-networks as relational interfaces encoding agent-world interaction. Categories derived from Q-networks capture actions and phenomenological invariants. Aligns with 4E cognition (enactive, embedded, extended).

**Defect #14**: NeoTrix treats consciousness as a monitoring/emergence property (ConsciousnessTree). This work shows consciousness should be modeled as the *interface structure* itself — the relational boundary between agent and world. NeoTrix has no formal agent-world interface category.

### 2.6 Affective Consciousness Case Study (arXiv 2609.03883)
**Source**: https://arxiv.org/abs/2609.03883

Deterministic artificial agent with affective system displays hedonic place preference behavior through felt uncertainty about intrinsic needs. Demonstrates that apparent subjectivity can arise from purely deterministic information processing.

**Defect #15**: NeoTrix's NT-FEEL emotion engine is reactive (responds to events). This work shows affective states should emerge from *intrinsic need uncertainty* — the agent should have homeostatic drives that create emotional states when unmet. NeoTrix has no homeostatic drive system.

### 2.7 AGI 2026 Conference Proceedings (Springer, 2026-07-22)
**Source**: https://link.springer.com/book/10.1007/978-3-032-33195-3

Theme: "Searching for a fundamental structure of AGI, via embodied forms and epistemic foundations." 60 papers from 167 submissions. Includes "Discovering Machine Correlates of Consciousness" and "BICA-Inspired Multiagent Model of Self-Awareness Based on LLM."

**Defect #16**: The field is converging on embodied+epistemic foundations as the path to AGI. NeoTrix's architecture is disembodied — NT-PHYSICAL exists but has no formal epistemic grounding (the agent's knowledge is not grounded in its own embodied interaction with the world).

---

## 3. Brain-Inspired Computing Findings

### 3.1 SpiNNaker2: Many-Core Brain-Inspired Computing (arXiv 2607.24396)
**Source**: https://arxiv.org/abs/2607.24396

152 processing elements (ARM M4F + accelerators), 4.5 TOPS high-performance, 2.7 TOPS/W efficiency for INT8. Supports >150,000 neurons and >1.8B synaptic events/s. Dynamic voltage/frequency scaling per PE. Low baseline power <250mW.

**Defect #17 over batch 554**: Batch 554 did not address neuromorphic hardware substrate for NeoTrix. SpiNNaker2 demonstrates that brain-inspired computing has reached practical efficiency levels. NeoTrix has no neuromorphic compute path — all processing runs on conventional hardware.

### 3.2 Dual Memory Pathway (DMP) Architecture (Nature Machine Intelligence, 2026-06-16)
**Source**: https://www.nature.com/articles/s42256-026-01255-3

Cortical fast-slow organization: each SNN layer maintains compact low-dimensional slow state (5% of hidden width) that summarizes recent activity and modulates fast spiking. Results: 40-60% fewer parameters than SOTA SNNs, 4x throughput, 5x energy efficiency over Loihi2.

**Defect #18**: NeoTrix's memory systems (KB, episodic, semantic) have no fast-slow pathway decomposition. DMP shows explicit separation of fast spike-driven processing from slow state summarization is architecturally superior. NeoTrix processes all memory at the same timescale.

### 3.3 First Multi-Core Neuromorphic with On-Chip BP Training (Nature Comm, 2026-03-25)
**Source**: https://www.nature.com/articles/s41467-026-70586-x

First multi-core neuromorphic architecture supporting direct backpropagation training of deep SNNs. Three-engine core (FP/BP/WG), 2D mesh network, GALS design. 190-330% performance of Jetson Orin, 1.05 TFLOPS/W @ FP16 @ 28nm. 55-85% DRAM access reduction vs A100.

**Defect #19**: NeoTrix's SEAL pipeline runs learning on conventional hardware with no neuromorphic acceleration. This architecture demonstrates on-chip SNN training is practical. NeoTrix has no path to neuromorphic-accelerated self-evolution.

### 3.4 HiAER-Spike: 160M Neuron FPGA Platform (arXiv 2602.18072)
**Source**: https://arxiv.org/pdf/2602.18072

Modular, reconfigurable FPGA platform: 160M neurons, 40B synapses (2x mouse brain), faster-than-real-time. Hierarchical address-event routing. Python interface shields users from hardware complexity.

**Defect #20**: NeoTrix has no large-scale SNN simulation capability. HiAER-Spike shows that community-accessible neuromorphic platforms exist. NeoTrix's brain-inspired claims need grounding in actual neuromorphic computation, not just architectural analogy.

### 3.5 EB-MTJ Memristive Synapse + LIF Neuron (Nature Comm, 2026-03-24)
**Source**: https://www.nature.com/articles/s41467-026-70802-8

Nanoscale (~100nm) exchange-bias MTJs implementing both memristive synapse (25+ stable states, STDP) and LIF neuron (0.4ns pulses, 190fJ/switching). Fully spintronic CSNN achieves 96% gesture recognition with hybrid BP-STDP.

**Defect #21**: NeoTrix's physical embodiment layer (NT-PHYSICAL) has no spintronic/memristive compute substrate specification. EB-MTJs show that sub-nanosecond, sub-picojoule neuromorphic primitives exist. NeoTrix's hardware abstraction layer cannot target these devices.

### 3.6 NeuDW-CIM: Nonlinear Dendrites + K-Winners (arXiv 2606.08947)
**Source**: https://arxiv.org/html/2606.08947v1

65nm compute-in-memory macro: 0.8 pJ/SOP, reconfigurable nonlinear in-memory ADC emulating biological dendritic functions. Top-K winner selection with early stopping reduces ADC latency 30% and LIF latency 10x. 97.2% N-MNIST, 95.5% DVS Gesture.

**Defect #22**: NeoTrix has no dendritic computation model. Biological dendrites perform nonlinear integration that massively reduces downstream computation. NeoTrix's neural-inspired processing is purely point-neuron level.

### 3.7 Neuromorphic AI: Intra-Token vs Inter-Token Processing (arXiv 2601.00245)
**Source**: https://arxiv.org/html/2601.00245v2

Framework connecting neuromorphic models, state-space models, and transformers via intra-token (feature transformation within a vector) vs inter-token (contextual combination across vectors) processing. Modern AI architectures increasingly embody neuromorphic principles through quantized activations, state-space dynamics, sparse attention.

**Defect #23**: NeoTrix's architecture does not distinguish intra-token from inter-token processing pathways. This distinction is architecturally significant — neuromorphic principles apply differently at each level. NeoTrix treats all computation uniformly.

---

## Summary: NEW Defects vs Batch 554

| # | Defect | Domain | Severity |
|---|--------|--------|----------|
| 1 | No permission boundary between perception and action (Governor layer missing) | Cognitive Arch | HIGH |
| 2 | No impasse detection / automatic subgoal creation in SEAL | Cognitive Arch | HIGH |
| 3 | No lazy knowledge acquisition (proactive fetch wastes tokens) | Cognitive Arch | MEDIUM |
| 4 | No MCP integration strategy for cognitive architecture | Cognitive Arch | MEDIUM |
| 5 | VSA HyperCube is static, no episode-level dynamic updates | Cognitive Arch | MEDIUM |
| 6 | No graceful degradation specification when layers fail | Cognitive Arch | HIGH |
| 7 | No learned fast/slow solver selection (metacognition not adaptive) | Cognitive Arch | MEDIUM |
| 8 | No computable integrated information measure (S-measure absent) | AGI/Consciousness | HIGH |
| 9 | Self-model is programmed, not architecturally emergent | AGI/Consciousness | HIGH |
| 10 | No parallel cognitive threads / reconstructive episodic buffer | AGI/Consciousness | HIGH |
| 11 | Value allocation cannot emerge — needs explicit module | AGI/Consciousness | CRITICAL |
| 12 | Self-audit may measure non-generalizable self-referential metrics | AGI/Consciousness | MEDIUM |
| 13 | No dynamic attention bandwidth / fatigue-induced narrowing | AGI/Consciousness | MEDIUM |
| 14 | Consciousness treated as monitoring property, not interface structure | AGI/Consciousness | HIGH |
| 15 | No homeostatic drive system for affective states | AGI/Consciousness | MEDIUM |
| 16 | Architecture is disembodied — no epistemic grounding | AGI/Consciousness | HIGH |
| 17 | No neuromorphic compute substrate path | Brain-Inspired | MEDIUM |
| 18 | No fast-slow memory pathway decomposition | Brain-Inspired | HIGH |
| 19 | No path to neuromorphic-accelerated self-evolution | Brain-Inspired | MEDIUM |
| 20 | No large-scale SNN simulation capability | Brain-Inspired | LOW |
| 21 | No spintronic/memristive hardware abstraction | Brain-Inspired | LOW |
| 22 | No dendritic computation model (point-neuron only) | Brain-Inspired | MEDIUM |
| 23 | No intra-token vs inter-token processing distinction | Brain-Inspired | MEDIUM |

---

## Sources Cited

1. zylos.ai — "Cognitive Architectures for AI Agents" (2026-03-12)
2. arXiv 2512.24113 — CogRec: LLM+Soar Cognitive Recommender
3. integratedcognition.ai — 2026 Soar Workshop Schedule
4. arXiv 2608.02807 — ACT-R VSA Extension for Socio-Cultural Tasks
5. JUST — Cognitive Architectures for Adaptive Mission Planning (2026-01-29)
6. adaptiverecall.com — ACT-R vs SOAR vs CLARION Compared (2026-05-11)
7. Zenodo 10.5281/zenodo.20613601 — SubjectNet AGI Architecture (2026-06-09)
8. AAAI-SS — MIRROR: Reconstructive Machine Access Consciousness (2026-05-18)
9. arXiv 2608.22347 — Where Cognition Lives (2026-08-23)
10. arXiv 2608.20420 — Categorical AI Phenomenology (2026-08-19)
11. Zenodo 10.5281/zenodo.20779862 — AGVS Attention-Gated Virtual Sensorium (2026-06-21)
12. arXiv 2609.03883 — Affective Consciousness Case Study (2026-09-03)
13. Springer 10.1007/978-3-032-33195-3 — AGI 2026 Proceedings (2026-07-22)
14. arXiv 2607.24396 — SpiNNaker2 Chip (2026-07-27)
15. Nature 10.1038/s42256-026-01255-3 — Dual Memory Pathway SNN (2026-06-16)
16. Nature 10.1038/s41467-026-70586-x — Multi-Core Neuromorphic BP Training (2026-03-25)
17. arXiv 2602.18072 — HiAER-Spike FPGA Platform
18. Nature 10.1038/s41467-026-70802-8 — EB-MTJ Memristive Synapse (2026-03-24)
19. arXiv 2606.08947 — NeuDW-CIM Nonlinear Dendrites (2026-06-08)
20. arXiv 2601.00245 — Neuromorphic AI Intra/Inter-Token (2026-01)

---

## Key Improvements over Batch 554

1. **GRACE decomposition** (batch 554) addresses robustness but not the *cognitive control layer* — batch 555 identifies the Governor pattern as a missing architectural element
2. **OOD detection** (batch 554) handles distribution shift but not *impasse-driven knowledge acquisition* — batch 555 shows lazy fetching at impasse is cheaper and more effective
3. **SPD sheaf expressivity** (batch 554) is structural but not *information-integrated* — batch 555 introduces computable S-measure as a concrete alternative to Φ
4. **Certified robustness** (batch 554) focuses on input perturbation but not *emergent vs computed function* — batch 555 proves value allocation cannot emerge and must be explicit
5. **Test-time adaptation** (batch 554) is algorithmic but not *architecturally grounded* — batch 555 shows neuromorphic fast-slow pathways provide hardware-efficient adaptation

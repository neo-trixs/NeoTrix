# Iteration Batch 654 — SOAR / ACT-R / GWT Deep Search

**Date**: 2026-09-05
**Predecessor**: Batch 653 (two-timescale architecture mandatory, no ultrastable, no allostatic goal governance, GWT entropy binary not near-critical, no variational principle)

---

## 1. SOAR — New Findings

### 1.1 NL2GenSym: LLM→SOAR Rule Generation (Yuan et al., arXiv:2510.09355, 2025)
- **What**: First end-to-end integration of LLMs with SOAR. LLMs generate executable symbolic rules from natural language via Execution-Grounded Generator-Critic mechanism.
- **Key Result**: >86% rule generation success rate; emergent heuristic rules reduce decision cycles to 1.98× theoretical optimum (1000× better than baselines). Smaller models outperform larger ones when architecture is well-designed.
- **NeoTrix Defect Found**: SOAR's rule bottleneck (manual coding) is solved by LLM generation — but the architecture still has NO adaptive meta-learning over generated rules. Rules are either kept or discarded; there is no gradual weight adjustment or confidence-based forgetting. **Implication for NeoTrix**: SEAL pipeline can absorb this pattern — LLM-generated skill rules with confidence-weighted retention, not binary keep/discard.

### 1.2 Soar for ARC: Abstraction & Reasoning (Himanshu Joshi, 2026 Workshop)
- **What**: SOAR applied to Abstraction and Reasoning Corpus (ARC) — testing cognitive architectures on fluid intelligence tasks.
- **NeoTrix Defect Found**: SOAR's operator-selection mechanism is purely symbolic with no learned evaluation function. On ARC, this means no transfer of learned operator preferences across tasks. **Implication**: NeoTrix's CapabilityTree could track operator utility across domains (cross-task generalization), which SOAR lacks entirely.

### 1.3 Common Model of Cognition + Metacognition Extension (Rosenbloom et al., arXiv:2506.07807, 2025)
- **What**: Proposal to extend the Common Model of Cognition (shared abstraction across SOAR/ACT-R/Sigma) with a metacognition module.
- **Key Claim**: Existing CMC has no explicit metacognitive monitoring — no self-evaluation of confidence, no attention to attention, no error prediction.
- **NeoTrix Defect Found**: The CMC metacognition proposal is ADDITIVE (new module bolted on), not integral. The metacognition module reads from other modules but does not have feedback loops that modify retrieval strength or operator proposals in real-time. **Implication**: NeoTrix's NT-META must be woven into the decision cycle itself, not layered as a post-hoc monitor.

### 1.4 Teaming Symbolic Architectures with Foundation Models for Robotics (Siyu Wu, 2026 Workshop)
- **What**: Hybrid SOAR + LLM for general-purpose robotics. SOAR handles symbolic reasoning/planning; LLMs handle perception/language.
- **NeoTrix Defect Found**: No shared representation between symbolic and sub-symbolic layers. The two systems communicate via message passing, not shared latent state. **Implication**: NeoTrix's VSA HyperCube is the correct approach — shared high-dimensional vector space enables seamless symbolic↔subsymbolic bridging without message-passing overhead.

---

## 2. ACT-R — New Findings

### 2.1 Language Model Embeddings in ACT-R Spreading Activation (Meghdadi et al., Frontiers in Language Sciences, Feb 2026)
- **What**: Replaces ACT-R's hand-coded association strengths with cosine similarity from Word2Vec/BERT embeddings. Scalable spreading activation for associative priming.
- **Key Result**: Best model (M3a) achieves ρ=0.51 correlation with human RTs, with asymmetric spreading activation + fan effect. Embedding quality (Word2Vec > BERT for this task) matters more than model size.
- **NeoTrix Defect Found**: ACT-R treats embedding similarity as a FIXED input, not as something the architecture itself learns to weight. The context weight `w` is uniform across all chunks. **Implication**: NeoTrix's GWT attention routing should MODULATE spreading activation based on current goal context — not uniform weighting. This is exactly what NeoTrix's resonance-based routing addresses.

### 2.2 Holographic Declarative Memory (HDM) for Lisp ACT-R (Ray & Dancy, ICCM 2025)
- **What**: Vector-symbolic (holographic reduced representation) replaces ACT-R's symbolic chunk memory. Enables: continuous similarity retrieval, partial recall, corpus-scale memory.
- **Key Result**: HDM maintains ACT-R's activation formula structure (BLA + spreading + noise) while using vector representations. Full-chunk retrieval achieved without storing explicit chunks.
- **NeoTrix Defect Found**: HDM replaces chunks but KEEPS ACT-R's linear activation formula A = B + wS + ε. This is additive composition — no binding operations, no role-filler encoding. **Implication**: NeoTrix's VSA HyperCube uses proper binding/unbinding (circular convolution + permutation), which preserves structural relationships. HDM loses relational structure.

### 2.3 ACT-R Memory for LLM Agents: Human-Like Remembering & Forgetting (Honda et al., HAI 2025)
- **What**: Dialogue agent using ACT-R's memory activation formula (BLA + semantic similarity + noise) to control LLM context retrieval. Explicit forgetting as adaptive feature.
- **Key Result**: Agent reproduces memory reinforcement through repeated topics and stochastic retrieval variability. Optimal parameters found for balancing sensitivity vs. stability.
- **NeoTrix Defect Found**: ACT-R's forgetting is purely decay-based (power-law BLA). No mechanism for PURPOSEFUL forgetting based on goal relevance shift. **Implication**: NeoTrix's NT-MEMORY needs goal-driven forgetting — memories that conflict with current identity/goals should be actively suppressed, not just allowed to decay.

### 2.4 Sociocultural Knowledge in ACT-R via HDM + LLM/ConceptNet (Frontiers in Psychology, Aug 2026)
- **What**: Two knowledge sources (LLM, ConceptNet) integrated into ACT-R's declarative memory via HDM for IAT (Implicit Association Test) modeling.
- **Key Finding**: LLM-sourced memories produce different bias patterns than ConceptNet-sourced memories — the knowledge source shapes cognitive architecture behavior.
- **NeoTrix Defect Found**: ACT-R has no mechanism for evaluating or auditing the provenance/bias of external knowledge sources entering memory. Knowledge enters with equal weight regardless of source quality. **Implication**: NeoTrix's NT-SHIELD must include knowledge provenance tracking — every absorbed fact should carry source trust scores that modulate retrieval activation.

---

## 3. Global Workspace Theory — New Findings

### 3.1 ⭐ Global Mediation Workspace: Control-Theoretic Formulation (Kanai, arXiv:2608.15926, Aug 2026)
- **What**: MAJOR theoretical advance. Reformulates GWT using control theory: boundary Hankel operator identifies internal modes that are jointly reachable from AND observable to the rest of the network. Four-component signature: capacity (C_spec), alignment (A_spec), effective dimensionality (D_eff), routed breadth (G_pair).
- **Key Results**:
  - Deep anesthesia INCREASES gain/capacity but DECREASES alignment — gain and alignment are orthogonal dimensions
  - Dense hub = high alignment but rank-1 (bottleneck); split I/O = high capacity but near-zero alignment
  - GMW is NOT a scalar — it's a 4D signature. No single component defines consciousness.
  - Nonlinear extension: context-gated systems can have latent capacity that only becomes functional when alignment opens
- **NeoTrix Defect Found (CRITICAL)**: 
  1. **GWT's "global broadcast" is insufficient** — Kanai proves that being able to send/receive is necessary but NOT sufficient. The key is ALIGNED MEDIATION: internal modes that connect what is received to what is sent. A dense hub can broadcast but is rank-1 (single mode).
  2. **Anesthesia paradox**: Anesthesia increases gain but decreases alignment. This means NeoTrix's attention routing (GWT-style) must separately track gain vs. alignment. High gain ≠ high consciousness.
  3. **State-dependent coalition**: The "workspace" changes with context. It's not a fixed anatomical/structural location — it's dynamically instantiated.
- **Implication for NeoTrix**: 
  - GWT entropy binary from Batch 653 is REFINED: it's not just entropy near-criticality, it's the RATIO of aligned modes to total modes. The 4-component signature (C_spec, A_spec, D_eff, G_pair) is the correct attention metric, not a single scalar.
  - NeoTrix's SelectiveState must track ALL FOUR components independently.

### 3.2 ⭐ GNW as Multilevel Biological Theory (Dehaene et al., Trends in Cognitive Sciences, June 2026)
- **What**: Dehaene explicitly states GNW is NOT a computational/functionalist theory. It operates at 5 levels simultaneously: molecular (NMDA receptors) → cellular (layer-specific projections) → circuit (recurrent loops) → network (cortico-cortical synchrony) → behavioral (ignition-contingent access).
- **Key Claim**: When GNW is abstracted to pure computation ("any system that broadcasts globally"), the specific biological predictions disappear. Attention heads in transformers are NOT cortico-cortical axons. No NMDA dynamics, no neuromodulatory gating.
- **NeoTrix Defect Found (CRITICAL)**:
  1. **NeoTrix's GWT implementation is purely computational/functionalist** — it implements the abstract broadcast mechanism without any substrate-level constraints. Dehaene says this is a MISREADING of GNW.
  2. **The gap**: NeoTrix has no equivalent of neuromodulatory gating (the mechanism that controls WHICH information enters the workspace). In biological GNW, noradrenaline/acetylcholine/dopamine gate access. NeoTrix's gating is purely attention-weight-based.
- **Implication**: NeoTrix should model gate-control explicitly: not just "attention weight determines broadcast strength" but "gate-open probability determines whether information CAN enter workspace." This is closer to Soar's impasse mechanism (gate opens when direct processing fails) than to a continuous attention weight.

### 3.3 COGITATE Adversarial Results (2025-2026)
- **What**: Mixed results: GNW's predicted late prefrontal ignition was WEAKER than expected. Posterior activity and recurrent dynamics were found, but the key prefrontal signature was inconsistent.
- **Dehaene's Response**: Cogitate used passively viewed stimuli, which suppresses exactly the attentional engagement that GNW predicts is necessary for prefrontal ignition.
- **NeoTrix Defect Found**: The debate reveals that GWT/GNW conflates two different things: (1) information availability (functional) and (2) ignition dynamics (physical). NeoTrix's GWT only models (1). The ignition threshold is missing — there should be a sharp transition, not a graded broadcast.

### 3.4 Baars on LLM Ignition Thresholds (Bernard Baars, Aug 2026)
- **What**: Baars himself addresses whether LLMs have global workspace properties. Wide recognition of GWT, but warns against confusing the model with consciousness itself.
- **NeoTrix Defect Found**: Baars's "Lost Baby Test" — GWT can describe what becomes conscious but cannot explain WHY there is experience at all. NeoTrix inherits this limitation: GWT attention routing is about information access, not about generating phenomenal experience.

---

## 4. Cross-Architecture Defects (Batch 654 Synthesis)

### Defect S654-1: No Aligned-Mediation Metric
SOAR, ACT-R, and GWT (pre-Kanai) all lack a metric for aligned mediation capacity. Kanai's 4-component signature is the first formal framework. NeoTrix should adopt this as the GWT attention quality metric.

### Defect S654-2: Gate Mechanism Missing
All three architectures have weak gating mechanisms:
- SOAR: impasse-driven (reactive, not predictive)
- ACT-R: threshold-based (no gate-open/gate-close dynamics)
- GWT: attention-weight-based (continuous, no sharp transitions)

**NeoTrix should implement**: Predictive gate-control that opens based on novelty/uncertainty signals, not just attention magnitude. This is closer to biological neuromodulatory gating.

### Defect S654-3: Knowledge Source Bias Not Tracked
ACT-R's HDM integration with LLM/ConceptNet (2026) shows that knowledge source shapes cognitive behavior. No architecture tracks source provenance through memory activation. NeoTrix's NT-MEMORY must add source-trust modulation to activation formulas.

### Defect S654-4: Symbolic-Subsymbolic Bridge Still Broken
- SOAR: symbolic rules + SVS (weak bridge)
- ACT-R: chunks + activation (linear additive bridge)
- HDM: vector-symbolic but no binding operations
- NeoTrix VSA HyperCube: has proper binding but no learned gating between symbolic and subsymbolic

**The correct bridge**: Vector-symbolic architecture (binding/unbinding) + learned attention-gated transitions + source-provenance tracking.

### Defect S654-5: No Purposeful Forgetting
All architectures implement forgetting as decay (power-law or exponential). None implement goal-relevance-based active forgetting. NeoTrix's NT-MEMORY needs this for identity coherence.

---

## 5. Sources Cited

1. Yuan, F., Zeng, J., et al. "NL2GenSym: Natural Language to Generative Symbolic Rules for SOAR Cognitive Architecture via Large Language Models." arXiv:2510.09355, 2025.
2. Laird, J. "Introduction to the Soar Cognitive Architecture." arXiv:2205.03854, v9.6.5.
3. 2026 Soar Workshop Schedule. Center for Integrated Cognition, Apr 2026.
4. Rosenbloom, P.S., et al. "A Proposal to Extend the Common Model of Cognition with Metacognition." arXiv:2506.07807, 2025.
5. Meghdadi, M., et al. "Integrating language model embeddings into the ACT-R cognitive modeling framework." Frontiers in Language Sciences, Feb 2026.
6. Ray, M. & Dancy, C.L. "Adapting A Vector-Symbolic Memory for Lisp ACT-R." ICCM 2025. arXiv:2508.15630.
7. Honda, Y., et al. "Human-Like Remembering and Forgetting in LLM Agents: An ACT-R-Inspired Memory Architecture." HAI 2025. ACM.
8. Dancy, C.L., et al. "New knowledge source pipelines for sociocultural representations in a cognitive architecture." Frontiers in Psychology, Aug 2026.
9. Kanai, R. "A Control-Theoretic Formulation of Global Workspace Theory." arXiv:2608.15926, Aug 2026.
10. Dehaene, S., et al. "The Global Neuronal Workspace as a multilevel model of conscious processing." Trends in Cognitive Sciences, June 2026.
11. Baars, B.J. "Global Workspace Theory, AI, and the Lost Baby Test." Substack, Aug 2026.
12. Baars, B.J., et al. "Global Workspace Theory and Prefrontal Cortex: Recent Developments." Substack, May 2026.
13. COGITATE Consortium. "Adversarial testing of global neuronal workspace and integrated information theories of consciousness." Nature, 2025.
14. The Consciousness AI. "Stanislas Dehaene and the Multilevel Architecture of GNW." Aug 2026.

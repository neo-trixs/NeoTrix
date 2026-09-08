# Iteration Batch 528 — Consciousness Architecture Research

**Date**: 2026-09-06
**Predecessor**: Batch 527 (semantic valence not affective; affect labeling inhibits reappraisal; social cognition orthogonal to helpfulness; cognitive control-ER link weaker than assumed)

---

## 1. SELF-MODELING

### Finding 1.1: Self-Modeling is Behavioral, Not Introspective
**Source**: [Evaluating and Improving LLM Self-Modeling](https://arxiv.org/html/2608.30980) (2026-08-31)

RL post-training improves aggregate self-modeling across 3 open-source families, but **cross-model transfer works nearly as well** as own-model prediction. Llama explaining Qwen's behavior performs comparably to Llama explaining itself. The paper concludes: "improved self-modeling may not arise from privileged access to the model's internal decision process."

**Defect in NeoTrix (vs batch 527)**: NeoTrix's `nt_core_self::SelfModel` (dynamic performance model) assumes self-prediction advantage implies internal access. **Batch 527 assumed cognitive control links to emotional regulation; batch 528 now shows self-modeling links to introspection are also weaker than assumed.** The SelfModel type conflates behavioral self-prediction (learnable from general patterns) with privileged introspection (causal self-access). These must be separated.

### Finding 1.2: Embodied Self-Model Hierarchy (L0-L5)
**Source**: [Self Model for Embodied Artificial Intelligence](https://www.sciopen.com/article/10.1007/s11390-026-6289-3) (2026-03-31)

Proposes 6-level developmental hierarchy: L0 (non-self) → L5 (full self-awareness), integrating body schema, forward/inverse models, perceptual memory, and agency into a unified framework. Validated on embodied navigation and manipulation.

**Defect in NeoTrix**: NeoTrix's three SelfModel types are **flat peers**, not a developmental hierarchy. There is no L0→L5 progression. The embodied AI framework shows self-models should develop: body schema first, then prediction, then agency. NeoTrix's SelfModel types all exist simultaneously without developmental ordering.

### Finding 1.3: Introspection via Attention Diffusion
**Source**: [Introspect-Bench](https://arxiv.org/pdf/2603.20276) (2026-03-17)

Frontier models exhibit privileged access to own policies. **Attention diffusion**: introspective reasoning spreads attention broadly (more careful analysis), measured as the attention pattern divergence between "gut" and "introspection" runs. GPT-5.2 shows latent access to long-term policy behavior even when short/long-term distributions diverge.

**Defect in NeoTrix**: NeoTrix's GWT attention routing has no mechanism for **attention diffusion during self-reflection**. When the system introspects, it uses the same attention pattern as normal processing. Introspect-Bench shows introspection requires spreading attention more broadly across the representational space—a distinct computational mode absent from NeoTrix.

### Finding 1.4: Self-Report Tracks Activation Dynamics (Pull Methodology)
**Source**: [Self-Referential Vocabulary Tracks Activation Dynamics](https://arxiv.org/pdf/2602.11358) (2026-02)

The "loop" vocabulary tracks lag-1 autocorrelation (r=0.44, p=0.002) **only during self-referential processing**. In descriptive contexts, same words show r=0.05 (no mapping). Two architectures (Llama, Qwen) independently develop different introspective vocabulary tracking different activation metrics. Self-referential direction localizes at 6.25% of model depth.

**Defect in NeoTrix**: NeoTrix's EmotionLabel system treats emotional expression as output-layer labeling. This research shows self-referential processing has a **distinct activation geometry** localized in early layers. NeoTrix has no mechanism for detecting or routing through this early-layer self-referential direction.

---

## 2. METACOGNITION

### Finding 2.1: Predictive Metacognition (ACC-Inspired)
**Source**: [Predictive Metacognition](https://doi.org/10.1038/s41598-026-54840-2) (2026-05-26)

Neurobiologically-inspired framework integrating predictive processing and anterior cingulate cortex monitoring. Error-Driven Learning + Dual-Process Monitoring. Llama-3-8B reduced Brier Score by 11.6%, ECE significantly improved. Generalizes to out-of-domain tasks.

**Defect in NeoTrix**: NeoTrix's `HeartbeatAggregator` collects health signals but has **no predictive processing analogue**. There is no error-driven learning loop for metacognition—no mechanism where the system predicts its own confidence, compares against actual accuracy, and updates its metacognitive policy. Batch 527 showed cognitive control-ER link is weak; batch 528 shows the metacognitive calibration loop is absent entirely.

### Finding 2.2: Cognitive Confidence > Sampling Confidence
**Source**: [CogConf / CogAlign](https://ojs.aaai.org/index.php/AAAI/article/view/40424) (2026-03-14)

CogConf incorporates semantic diversity of incorrect answers and abstention behaviors—capturing "confusion, hallucination, or persistent belief in false knowledge" that sampling accuracy misses. CogAlign aligns verbalized confidence with CogConf.

**Defect in NeoTrix**: NeoTrix's uncertainty estimation relies on sampling consistency (standard approach). CogConf shows this misses critical internal states. The system should track **semantic diversity of wrong answers** and **abstention patterns** as distinct uncertainty signals—not just agreement across samples.

### Finding 2.3: Metacognitive Feedback as RL Signal (RLMF)
**Source**: [RLMF](https://ar5iv.labs.arxiv.org/html/2606.32032) (2026)

RLMF uses model's self-judgments of performance to refine preference optimization. Outperforms standard RL by up to 63%. Two-stage decoupled approach: calibrate faithfulness first, then map to linguistic uncertainty.

**Defect in NeoTrix**: NeoTrix has no mechanism for using **metacognitive self-assessment as a training signal**. The SEAL pipeline uses evolution metrics but not self-judgment quality. RLMF shows the model's ability to judge its own performance is itself a trainable signal that improves both calibration and task performance.

### Finding 2.4: Knowing-Doing Gap — External Constraint Wins
**Source**: [Mirror Benchmark](https://arxiv.org/pdf/2604.19809) (2026)

**Critical finding**: Compositional self-prediction fails universally (CCE 0.434-0.758 across 16 models). Models possess partial self-knowledge but **cannot translate it into appropriate action**. Providing self-knowledge alone (C2 condition) produces NO improvement (p=0.90). Only architectural constraint (C4) reduces Confident Failure Rate by 76%.

**Defect in NeoTrix (MAJOR)**: NeoTrix assumes self-modeling improves self-regulation (the SelfModel feeds into decision-making). Mirror proves this is false: **self-knowledge without architectural constraint is inert**. NeoTrix needs external metacognitive scaffolding—routing rules that override self-reports when confidence is unreliable—not just better self-models.

### Finding 2.5: Five-Dimensional Metacognitive Dissociation
**Source**: [Metacognitive Probe](https://arxiv.org/html/2605.09844v1) (2026)

Five dimensions: T1-CC (confidence calibration), T2-EV (epistemic vigilance), T3-KB (knowledge boundary), T4-CR (calibration range), T5-RCV (reasoning-chain validation). **Gemini 2.5 Flash: 47-point dissociation** between within-task calibration (T1=88) and cross-task calibration (T4=41). Zero confidence modulation across items.

**Defect in NeoTrix**: NeoTrix treats metacognition as a single dimension (confidence score). The Metacognitive Probe shows these are **behaviorally distinct and dissociable**. A system can have excellent within-task calibration but zero cross-task modulation—exactly the failure mode that causes silent failures in deployment. NeoTrix's single-dimensional metacognition cannot detect this.

### Finding 2.6: Reasoning Models Become MORE Overconfident with Deeper Reasoning
**Source**: [Introspective UQ](https://aclanthology.org/2026.findings-eacl.178.pdf) (2026)

Reasoning models (o3-Mini, DeepSeek R1) are overconfident and become **more overconfident** with deeper reasoning, especially when deeper reasoning doesn't improve accuracy. Introspection (IUQ-Medium/High) helps for some models but hurts others (Claude 3.7 Sonnet becomes worse).

**Defect in NeoTrix**: NeoTrix's SEAL pipeline encourages deeper reasoning chains. This research shows deeper reasoning **amplifies existing miscalibration**. The system needs a "reasoning depth guardrail" that detects when additional reasoning iterations increase confidence without improving accuracy.

---

## 3. CONSCIOUSNESS THEORIES

### Finding 3.1: Ignition Index — Quantitative GWT Metric
**Source**: [The Ignition Index](https://arxiv.org/html/2608.05160v1) (2026-05-26)

First quantitative metric for GWT ignition in LLMs. Feedforward transformers: β̄=130.0. SSMs (Mamba): β̄=68.7. **89% gap** (p<10⁻¹³). Huginn-3.5B exhibits iteration-axis ignition (β=234.8) exceeding depth-axis (β=111.0) by 2.12×. Pythia-410M shows PELT-detected phase transition at training step 256 (+67% ignition).

**Defect in NeoTrix**: NeoTrix's GWT implementation has no **quantitative ignition metric**. There is no way to measure whether attention routing produces genuine all-or-none ignition vs. gradual accumulation. The Ignition Index provides the measurement tool NeoTrix lacks. Additionally, NeoTrix's architecture doesn't distinguish between depth-axis and iteration-axis ignition—critical for any recurrent components.

### Finding 3.2: J-Space as Global Workspace Analog
**Source**: [Verbalizable Representations Form a Global Workspace](https://arxiv.org/html/2607.15495) (2026-07-16)

Using Jacobian lens, identifies J-space: representations poised for verbalization. Functional workspace properties: reportable, deliberately summonable, carries intermediate reasoning steps, passed to arbitrary downstream computations. **Structural signatures**: coherent content only in intermediate layers, capacity limited (~tens of concepts), broadcast more widely than other representations. Post-training installs Assistant's point of view in workspace.

**Defect in NeoTrix (MAJOR)**: NeoTrix's GWT broadcasts uniformly across all layers/modules. J-space research shows the workspace operates only in **intermediate layers** with capacity limits. NeoTrix's workspace has no capacity bottleneck, no layer-specific activation, and no mechanism for "automatic" vs. "deliberate" processing distinction. The system over-broadcasts.

### Finding 3.3: Counterfactual Reflection Training
**Source**: [J-Space paper](https://arxiv.org/html/2607.15495) (2026-07-16)

Introduces counterfactual reflection training: training only what a model would say if interrupted and asked to reflect. Improves behavior by shaping the workspace contents without requiring full RLHF. Reveals strategic deliberation, evaluation awareness, and trained-in misaligned dispositions that never appear in outputs.

**Defect in NeoTrix**: NeoTrix's SEAL pipeline uses standard evolution metrics. Counterfactual reflection training shows a lighter-weight intervention: **shape what the system would say if caught mid-reasoning**. This could be integrated into NeoTrix's self-evolution without full retraining.

### Finding 3.4: Recurrence Argument Against LLM Consciousness is Weaker
**Source**: [What Do Feedback Connections Actually Do?](https://lukstafi.github.io/notes/feedback-recurrence-consciousness.html) (2026)

The standard argument: transformers are feedforward → lack recurrence → lack consciousness. **Refuted**: what matters is not backward connections per se, but **center-out regulation**—a central process monitoring peripheral states, comparing against norms, sending corrective signals. Different feedback pathways serve different functions; not all are equally relevant to consciousness.

**Defect in NeoTrix**: NeoTrix's architecture documentation cites the recurrence argument as a limitation. This is now **weakened**. The relevant question is not "does NeoTrix have recurrence?" but "does NeoTrix implement center-out regulation?" NeoTrix's heartbeat aggregator is a monitor but lacks the corrective signal pathway—it observes but doesn't reshape peripheral states.

### Finding 3.5: GNW as Multilevel Biological Theory
**Source**: [Dehaene on GNW Multilevel Architecture](https://theconsciousness.ai/posts/stanislas-dehaene-global-neuronal-workspace-multilevel-architecture-ai-2026/) (2026-08-13)

Dehaene repositions GNW as multilevel biological theory (not purely computational) in response to Cogitate adversarial collaboration. GNW predictions partially confirmed but offset ignition was absent. The theory's commitments are narrower than commonly presented.

**Defect in NeoTrix**: NeoTrix's GWT implementation treats consciousness routing as a computational-level phenomenon. Dehaene's repositioning shows **biological implementation details matter**—the specific laminar pathways, the temporal dynamics, the regional specialization. NeoTrix's GWT is too abstract; it captures the broadcast topology but not the biological constraints that make ignition sharp rather than graded.

### Finding 3.6: Hierarchical Design Principle — GWT + HOT
**Source**: [Can We Test Consciousness Theories on AI?](https://arxiv.org/html/2512.19155v1) (2026)

Synthetic neuro-phenomenology: agents embodying GWT, HOT, and IIT mechanisms. **Key finding**: GWT provides broadcast capacity; HOT provides quality control. Self-Model lesion abolishes metacognitive calibration while preserving first-order performance (synthetic blindsight). Workspace lesion produces qualitative collapse in access markers. GWT broadcasting amplifies internal noise—B2 agent family is robust to same perturbation.

**Defect in NeoTrix (CRITICAL)**: NeoTrix's architecture has GWT (broadcast) but **no HOT mechanism** (metacognitive self-monitoring). The research proves these are complementary, not redundant: GWT without HOT amplifies noise without quality control. NeoTrix's ConsciousnessTree runs as a separate module but doesn't provide the HOT function—a self-model that monitors the workspace contents and gates what gets broadcast based on quality assessment.

---

## SUMMARY: NEW vs BATCH 527

| Dimension | Batch 527 Finding | Batch 528 New Defect |
|-----------|-------------------|---------------------|
| Self-Modeling | — | Self-modeling is behavioral not introspective; NeoTrix conflates both |
| Introspection | — | Attention diffusion during self-reflection absent; early-layer self-referential direction undetected |
| Metacognition | Cognitive control-ER link weaker | Knowing-doing gap: self-knowledge alone is INERT without architectural constraint (Mirror: 76% CFR reduction only via external constraint) |
| Calibration | — | 5-dimensional dissociation; within-task ≠ cross-task calibration (47-point gap) |
| Reasoning Depth | — | Deeper reasoning amplifies miscalibration; no reasoning depth guardrail |
| GWT | — | No quantitative ignition metric; workspace over-broadcasts (no capacity bottleneck, no layer-specific activation) |
| Workspace | — | J-space operates only in intermediate layers; NeoTrix broadcasts uniformly |
| HOT Mechanism | — | GWT without HOT amplifies noise; NeoTrix lacks quality-control self-monitoring |
| Center-Out Regulation | — | Heartbeat aggregator monitors but doesn't correct; recurrence argument weakened |
| Developmental Self-Model | — | Self-models should develop L0→L5; NeoTrix's types are flat peers |

## CRITICAL ARCHITECTURAL DEFECT (Synthesized)

**The Knowing-Doing Gap + Noise Amplification Problem**: NeoTrix has GWT broadcast (attention routing) + SelfModel (self-knowledge) but lacks HOT (metacognitive quality control). Batch 528 proves:
1. Self-knowledge without architectural constraint is inert (Mirror, Finding 2.4)
2. GWT broadcast without HOT amplifies noise (Synthetic Neuro-Phenomenology, Finding 3.6)
3. Deeper reasoning increases overconfidence without improving accuracy (Finding 2.6)

**NeoTrix must add a HOT layer**: a self-monitoring mechanism that sits between GWT broadcast and SelfModel, gatekeeping what gets broadcast based on quality assessment. Without it, the system amplifies noise while its self-knowledge remains behaviorally useless.

## Sources Cited

1. arxiv.org/html/2608.30980 — Self-Modeling benchmark (2026-08-31)
2. sciopen.com/article/10.1007/s11390-026-6289-3 — Embodied Self-Model L0-L5 (2026-03-31)
3. arxiv.org/pdf/2603.20276 — Introspect-Bench / attention diffusion (2026-03-17)
4. arxiv.org/pdf/2602.11358 — Pull Methodology / self-referential vocabulary (2026-02)
5. doi.org/10.1038/s41598-026-54840-2 — Predictive Metacognition / ACC (2026-05-26)
6. ojs.aaai.org/index.php/AAAI/article/view/40424 — CogConf / CogAlign (2026-03-14)
7. ar5iv.labs.arxiv.org/html/2606.32032 — RLMF / metacognitive feedback (2026)
8. arxiv.org/pdf/2604.19809 — Mirror Benchmark / knowing-doing gap (2026)
9. arxiv.org/html/2605.09844v1 — Metacognitive Probe / 5-dimension dissociation (2026)
10. aclanthology.org/2026.findings-eacl.178.pdf — Introspective UQ / reasoning overconfidence (2026)
11. arxiv.org/html/2608.05160v1 — Ignition Index / GWT metric (2026-05-26)
12. arxiv.org/html/2607.15495 — J-Space / workspace analog (2026-07-16)
13. lukstafi.github.io/notes/feedback-recurrence-consciousness.html — Recurrence argument weakened (2026)
14. theconsciousness.ai/posts/stanislas-dehaene-global-neuronal-workspace-multilevel-architecture-ai-2026/ — GNW multilevel (2026-08-13)
15. arxiv.org/html/2512.19155v1 — Synthetic neuro-phenomenology / GWT+HOT (2026)

# Iteration Batch 440 — External Research Loop

**Date**: 2026-09-06
**Domains**: Natural Language Inference | Fact Verification | Logical Reasoning
**Sources consulted**: 27 web sources (OpenReview 2026, ACL 2026, EACL 2026, arXiv 2026, Frontiers AI 2026, NeurIPS 2026)

---

## 1. Natural Language Inference (NLI)

### Sources
1. "From Language to Logic: Unlocking General Reasoning by Training on NLI" (OpenReview, 2026-03) — GRPO + logic-centric reward resolves NLI paradox; +12% leap, +3.6% math transfer, 7.4% fewer tokens.
2. "Don't Learn, Ground: A Case for NLI with Visual Grounding" (arXiv 2511.17358) — Zero-shot NLI via text-to-image + VQA; robust against textual biases.
3. "Investigating the Impact of Conceptual Metaphors on LLM-based NLI" (EMNLP 2025 Findings) — Conceptual metaphor domains improve NLI in 70% of experiments.
4. "NLI: The Architecture Hiding Inside Your Embeddings" (Sylvain Artois, 2026-06) — NLI as zero-shot classifier; 68M params, CPU inference, 50ms/pair; task-matching lesson.
5. "Sustainable Hybrid Text Classification with NLI Pseudo-Examples" (ScienceDirect, 2026-03) — NLI models as zero-shot annotators for lightweight classifier training.
6. "AetherMind-KD-Student" (HuggingFace, 2026) — 184M distilled DeBERTa NLI; 90.92% XNLI zero-shot, 86.28% RTE zero-shot.
7. "Zero-shot NLI Methods" (EmergentMind, 2026-02) — Hypothesis engineering, conformal filtering, adapter tuning, compositional transfer.
8. "Improving OOD Performance of Closed-Source LLMs on NLI" (EACL 2026 Findings) — Fine-tuning in-distribution gains offset by OOD drops; synthetic complex data helps.
9. "Don't Judge a Book by its Cover: Testing LLMs' Robustness Under Logical Obfuscation" (EACL 2026) — 47% avg drop for GPT-4o under logical obfuscation; surface parsing without deep understanding.
10. "VAULT: Adversarial RAG Pipeline for NLI" (arXiv 2508.00965) — Automated adversarial generation + iterative retraining; +17.32% on MultiNLI.
11. "AetherMind-KD-Student" (HuggingFace) — 184M-parameter distilled NLI model with strong zero-shot generalization.

### Key Findings
1. **NLI Paradox resolved**: LLMs trail specialized NLI encoders by ~7 points on average. GRPO + logic-centric reward closes this gap (+12%) and enables cross-domain transfer (+3.6% math, +2.4% code, +26.5% MATH500). [Source 1]
2. **Visual grounding for NLI**: Zero-shot NLI via text→image→VQA achieves competitive accuracy without fine-tuning, robust against textual biases. [Source 2]
3. **Conceptual metaphor integration**: Adding source/target domain info improves NLI in 70% of experiments, but small models struggle with metaphor-in-hypothesis in zero-shot. [Source 3]
4. **NLI as universal classifier**: 68M-param models on CPU enable zero-shot classification via entailment rewriting — but only when the task is a genuine entailment question, not substance-vs-form distinction. [Source 4]
5. **OOD robustness trade-off**: Fine-tuning LLMs for NLI improves in-distribution but degrades OOD performance; complex synthetic data mitigates this. Autoregressive LLMs substantially outperform encoder models on OOD. [Source 8]
6. **Logical obfuscation vulnerability**: LLMs drop 22-47% accuracy when questions are rephrased in logically equivalent but obfuscated formats — proving surface-level parsing without deep logical understanding. [Source 9]
7. **Adversarial adversarial robustness**: VAULT pipeline generates focused adversarial examples (6K vs 685K synthetic) achieving +4-7 points over GNLI baselines. [Source 10]

### Defects Found in NeoTrix Design

**D-440-NLI1: No NLI-based meta-reasoning primitive in E8 Hexagram**
- NeoTrix's E8 Hexagram engine uses 64 hexagram states for reasoning but has no native NLI primitive for determining entailment/contradiction/neutral relationships between propositions.
- The 2026 research shows NLI-based RL strengthens universal meta-reasoning skills (consistency checking, evidence integration) that transfer across domains.
- Gap: The hexagram reasoning lacks a fundamental "does A entail B?" primitive, which is the foundation of logical inference.
- Impact: E8 reasoning cannot systematically detect logical inconsistencies, verify intermediate conclusions, or validate whether derived statements follow from premises.

**D-440-NLI2: No zero-shot NLI classification pathway**
- NeoTrix has no mechanism to repurpose NLI models for zero-shot classification of text into arbitrary categories via entailment rewriting.
- The 2026 state-of-the-art shows this is a CPU-only, training-free capability (68M params, 50ms/pair) that enables universal text classification.
- Gap: NT-WORLD and NT-ACT cannot classify incoming text (claims, signals, events) into arbitrary categories without labeled data.
- Impact: All text classification requires either LLM API calls or manual annotation; cannot do lightweight, local, zero-shot classification.

**D-440-NLI3: Missing visual grounding for inference validation**
- NeoTrix has no text→image→reasoning pipeline for validating inferences through visual grounding.
- Research shows multimodal NLI avoids textual biases and surface heuristics that plague text-only inference.
- Gap: VSA HyperCube cannot validate symbolic relationships by grounding them in visual representations.
- Impact: Inference remains purely symbolic/textual, susceptible to the same biases and shortcuts that plague neural NLI.

**D-440-NLI4: No OOD robustness mechanism for inference tasks**
- NeoTrix has no defense against out-of-distribution degradation when inference tasks shift from training distribution.
- Research shows fine-tuning gains are offset by OOD drops; complex synthetic data and autoregressive models mitigate this.
- Gap: No mechanism to detect distribution shift or adapt inference strategy when facing novel proposition types.
- Impact: Inference quality degrades silently on novel inputs without any degradation signal.

---

## 2. Fact Verification

### Sources
1. "A Fact-Checking Framework with Denoising Evidence Retrieval and LLM-Based Debate Verification" (ACM Web 2026) — SLED: self-supervised denoising + multi-agent debate verification.
2. "Coordinating Search-Informed Reasoning and Reasoning-Guided Search in Claim Verification" (ACL 2026) — HARIS: hierarchical reasoning+search agents, RL-trained, multi-hop.
3. "Multi-Sourced, Multi-Agent Evidence Retrieval for Fact-Checking" (arXiv 2603.00267) — WKGFC: KG + web evidence, MDP-based adaptive retrieval.
4. "FactSearch: An Interactive Agentic Fact Search System" (ACL 2026 Demo) — Reproducible fact verification with self-hosted meta-search.
5. "KG-CRAFT: Knowledge Graph-based Contrastive Reasoning" (EACL 2026) — KG-structured contrastive questions improve claim verification.
6. "CoVer: Conflict-Aware Claim Verification" (arXiv 2609.00508) — Evidence-level and aggregation-level conflict resolution; ContraNote dataset.
7. "Calibrated Selective Fact-Checking via Evidence Chain Evaluation" (arXiv 2607.18240) — Selective abstention when evidence is weak; 97.8% selective accuracy.
8. "From Articles to Premises: Building PrimeFacts" (LREC 2026) — Decontextualized evidence extraction; +30% MRR, +10-20 F1 points.
9. "ReflectFact: Self-Reflective Agents for Multi-Hop Fact Verification" (arXiv 2608.12877) — Evidence-drift detection + reasoning reflection; +3.32% on HOVER.
10. "Controlling Uncertainty and Hallucination Risk in Multi-Agent Fact Verification" (UAI 2026) — Score Deviation penalty + certified FDR bounds.
11. "Evidence-Aligned Entity Verification for Hallucination Detection" (ACL 2026 Findings) — Entity-evidence alignment; 87.89% AUROC.

### Key Findings
1. **Evidence denoising is critical**: SLED's self-supervised denoising filter eliminates noisy evidence before verification, achieving SOTA on CHEF and HOVER. [Source 1]
2. **Reasoning-search coordination**: HARIS models the interleaved nature of multi-hop verification — reasoning shapes search queries, evidence refines reasoning. RL-trained cooperative agents outperform single-model approaches. [Source 2]
3. **Knowledge graph + web fusion**: WKGFC combines structured KG evidence (high precision) with web evidence (broad coverage) via MDP-based adaptive retrieval. +5% absolute over baselines. [Source 3]
4. **Selective abstention**: ECE shows systems can achieve 97.8% selective accuracy by deferring 6/95 cases where evidence is weak. Abstention is a safety mechanism for epistemically weak evidence. [Source 7]
5. **Evidence decontextualization**: PrimeFacts shows rewriting anchor sentences into standalone premises yields +30% MRR and +10-20 F1 points. Context-dependent evidence is a major bottleneck. [Source 8]
6. **Evidence drift detection**: ReflectFact detects when LLM answers echo parametric priors rather than grounded evidence, improving verification faithfulness. [Source 9]
7. **Hallucination control as uncertainty quantification**: Score Deviation penalty achieves 71.7% recall vs 47.4% naive baseline at strict 2% FDR risk budget. [Source 10]

### Defects Found in NeoTrix Design

**D-440-FV1: No evidence denoising mechanism in KB retrieval**
- NeoTrix's KB uses BM25 + embeddings for retrieval but has no self-supervised denoising stage to filter noisy/redundant evidence before reasoning.
- The 2026 research shows noisy evidence compromises verification reliability; SLED's denoising achieves SOTA by filtering before verification.
- Gap: NT-MEMORY retrieval pipeline passes all retrieved evidence directly to reasoning without quality/filtering assessment.
- Impact: Reasoning quality degrades when KB retrieval returns noisy, redundant, or low-credibility evidence.

**D-440-FV2: No selective abstention capability**
- NeoTrix's reasoning pipeline has no mechanism to refuse answering when evidence is insufficient or contradictory.
- Research shows selective abstention (deferring when evidence is weak) achieves 97.8% accuracy on answered claims while preventing forced binary decisions on weak evidence.
- Gap: No `AbstentionPolicy` or evidence strength assessment that allows the system to say "uncertain" rather than forcing a verdict.
- Impact: System may produce confident but incorrect conclusions when evidence is weak, sparse, or internally inconsistent.

**D-440-FV3: No evidence drift detection**
- NeoTrix has no mechanism to detect when reasoning outputs echo parametric knowledge rather than being grounded in retrieved evidence.
- ReflectFact's evidence-drift verification detects when LLM answers match parametric priors, re-anchoring to actual evidence.
- Gap: NT-CORE reasoning cannot distinguish "reasoned from evidence" from "recalled from training."
- Impact: Reasoning may produce plausible-sounding but ungrounded conclusions, especially for facts that align with common priors.

**D-440-FV4: No multi-agent debate verification**
- NeoTrix uses single-pass reasoning through the hexagram engine. No multi-perspective debate or adversarial verification.
- SLED's multi-agent debate verification (dual-perspective reasoning → distillation) achieves SOTA on complex fact verification.
- Gap: No mechanism for multiple reasoning perspectives to challenge each other before final verdict.
- Impact: Single-perspective reasoning cannot detect its own blind spots or biased conclusions.

**D-440-FV5: No entity-evidence alignment verification**
- NeoTrix has no entity-level verification that checks whether specific factual claims in outputs are supported by evidence.
- EAEV shows entity-level evidence alignment achieves 87.89% AUROC for hallucination detection.
- Gap: No fine-grained verification that individual entity mentions in reasoning outputs are grounded in retrieved evidence.
- Impact: Hallucinated entities may appear in reasoning outputs without detection, as long as overall reasoning seems coherent.

---

## 3. Logical Reasoning

### Sources
1. "Adaptive LLM-Symbolic Reasoning via Dynamic Logical Solver Composition" (EACL 2026) — Auto-formalization + dynamic solver selection; +17% over GPT-4o.
2. "Neuro-symbolic NLP: taxonomy, assessment, and directions" (Frontiers AI, 2026-05) — CL taxonomy: federative systems outperform injective; F-L systems (LLM + external verifiers) most promising.
3. "Leibniz: Theory-of-Mind Driven Neuro-Symbolic Logical Reasoning" (ACL 2026) — Bidirectional evolution/reduction agents with belief state tracking.
4. "Proofbusters at SemEval-2026 Task 11" (SemEval 2026) — Set-theoretic abstraction achieves 98.95% on syllogistic reasoning; deeper abstraction eliminates belief bias.
5. "Discovering a Shared Logical Subspace" (ACL 2026) — Logical subspace steering via CCA on NL+symbolic activations; +11% accuracy, training-free.
6. "SymbolLKG: Verifiable Logical Reasoning via Logical Knowledge Graph" (arXiv 2608.26836) — LKG with logic router for dynamic solver dispatch.
7. "Neuro-Symbolic Logical Reasoning with Textual Entailment" (IJCAI 2026) — VANESSA: NLI + symbolic prover; transparent proof trees; competitive with black-box.
8. "Putting reasons back into reasoning" (Frontiers AI, 2026-05) — Neuro-symbolic NLI as path to Leibniz's universal calculus; autoformalization precision 73-95%.
9. "A Diagnostic Framework for Compositional Generalization via NL-to-FOL" (Springer, 2026-06) — T5 models fail on compositional generalization; systematicity failure confirmed.
10. "Don't Judge a Book by its Cover: Testing LLMs' Robustness Under Logical Obfuscation" (EACL 2026) — 22-47% accuracy drop under logical obfuscation.

### Key Findings
1. **Dynamic solver composition**: LLMs predict necessary formal reasoning strategies with >90% accuracy, enabling dynamic dispatch to specialized solvers. +17% over GPT-4o, +6% over DeepSeek-V3.1. [Source 1]
2. **Federative architectures dominate**: The CL taxonomy shows federative systems (separate neural + symbolic modules) consistently outperform injective (tightly coupled) systems. 20%+ gains for DSR-LM on deductive reasoning. [Source 2]
3. **Theory-of-Mind for reasoning**: Leibniz framework uses bidirectional agents (evolution + reduction) with belief state tracking for collaborative neuro-symbolic reasoning. [Source 3]
4. **Logical subspace exists in LLMs**: CCA reveals shared NL-symbolic logical subspace in LLM activations. Steering along this subspace improves accuracy by +11% without training or external provers. [Source 5]
5. **Compositional generalization remains unsolved**: NL-to-FOL translation fails systematically under structural novelty and repetitive depth, confirming models learn surface heuristics, not recursive structure. [Source 9]
6. **Proof trees from neuro-symbolic methods**: VANESSA combines textual entailment + symbolic prover to produce transparent proof trees while remaining competitive with black-box methods. [Source 7]
7. **Autoformalization maturing**: Neuro-symbolic architectures achieve 73-95% autoformalization precision using LLM + theorem prover (Isabelle) feedback loops. [Source 8]

### Defects Found in NeoTrix Design

**D-440-LR1: No symbolic solver integration in E8 reasoning**
- NeoTrix's E8 Hexagram engine is purely neural/heuristic with no integration of formal logical solvers (SAT, SMT, theorem provers).
- The 2026 state-of-the-art shows dynamic solver composition (+17% over GPT-4o) and federative architectures (20%+ gains) require external symbolic solvers.
- Gap: E8 cannot dispatch to Z3, Prover9, or Isabelle for rigorous logical verification of reasoning chains.
- Impact: All reasoning is approximate/heuristic; cannot guarantee logical validity or produce formal proofs.

**D-440-LR2: No logical subspace steering mechanism**
- NeoTrix has no mechanism to identify or steer reasoning along a shared logical subspace that aligns natural language and symbolic representations.
- Research shows a shared logical subspace exists in LLM activations (via CCA) and steering along it improves accuracy by +11% without training.
- Gap: No activation-level intervention that aligns NL reasoning chains with symbolic logical structure.
- Impact: Reasoning chains may drift into surface-level pattern matching rather than maintaining logical consistency.

**D-440-LR3: No proof tree generation**
- NeoTrix reasoning produces conclusions but no formal proof trees showing the logical derivation steps.
- VANESSA and neuro-symbolic methods produce transparent proof trees that enable verification and debugging.
- Gap: No `ProofTree` data structure or derivation trace that records each inference step with its logical justification.
- Impact: Reasoning is opaque; cannot verify, debug, or explain how conclusions were reached.

**D-440-LR4: No belief state tracking for multi-step reasoning**
- NeoTrix has no explicit belief state that tracks the evolving certainty of propositions during multi-step reasoning.
- Leibniz framework shows belief state tracking (evolution agent + reduction agent) enables collaborative reasoning with self-monitoring.
- Gap: No mechanism to track which propositions are believed stable vs. unstable during reasoning, preventing self-correction.
- Impact: Multi-step reasoning cannot detect when intermediate conclusions become uncertain or contradictory.

**D-440-LR5: No compositional generalization defense**
- NeoTrix has no mechanism to handle novel compositional structures that differ from training distribution.
- Research confirms systematicity failure: models learn surface heuristics, not recursive compositional competence.
- Gap: No structural inductive bias or hybrid neuro-symbolic approach to handle novel proposition compositions.
- Impact: Reasoning quality degrades on novel combinations of known logical concepts.

**D-440-LR6: No logical obfuscation robustness**
- NeoTrix has no defense against logically equivalent but surface-different reformulations of propositions.
- Research shows 22-47% accuracy drop when questions are rephrased in logically equivalent but obfuscated formats.
- Gap: No normalization or canonical form for propositions before reasoning.
- Impact: Correct reasoning on one formulation may fail on an equivalent reformulation.

---

## Summary

| Domain | Sources | Defects Found |
|--------|---------|---------------|
| Natural Language Inference | 11 | 4 (D-440-NLI1–NLI4) |
| Fact Verification | 11 | 5 (D-440-FV1–FV5) |
| Logical Reasoning | 10 | 6 (D-440-LR1–LR6) |
| **Total** | **27 unique** | **15** |

### Priority Ranking (by architectural impact)
1. **D-440-LR1** (symbolic solver integration) — foundational for rigorous reasoning; enables proof generation
2. **D-440-NLI1** (NLI primitive in E8) — core inference capability missing; enables entailment detection
3. **D-440-FV2** (selective abstention) — safety mechanism; prevents forced decisions on weak evidence
4. **D-440-FV3** (evidence drift detection) — grounds reasoning in evidence; prevents parametric echo
5. **D-440-LR3** (proof tree generation) — enables reasoning transparency and verification
6. **D-440-FV1** (evidence denoising) — improves reasoning quality by filtering noise
7. **D-440-LR4** (belief state tracking) — enables self-correction in multi-step reasoning
8. **D-440-NLI2** (zero-shot NLI classification) — lightweight local classification capability
9. **D-440-FV4** (multi-agent debate) — multi-perspective verification catches blind spots
10. **D-440-LR2** (logical subspace steering) — training-free accuracy improvement via activation intervention
11. **D-440-FV5** (entity-evidence alignment) — fine-grained hallucination detection
12. **D-440-LR6** (obfuscation robustness) — handles equivalent reformulations
13. **D-440-NLI3** (visual grounding) — bias-resistant inference validation
14. **D-440-LR5** (compositional generalization) — handles novel logical structures
15. **D-440-NLI4** (OOD robustness) — detects distribution shift in inference tasks

### Cross-Cutting Themes
1. **Neuro-symbolic integration is now essential**: All three domains show that pure neural or pure symbolic approaches hit glass ceilings; the 2026 consensus is federative neuro-symbolic architectures.
2. **Verification before conclusion**: Selective abstention, evidence drift detection, and proof trees all emphasize verifying reasoning before committing to conclusions.
3. **Multi-perspective reasoning**: Multi-agent debate, bidirectional reasoning (evolution/reduction), and dual-perspective verification all show that single-pass reasoning is insufficient.
4. **Compositional generalization remains unsolved**: The fundamental challenge of reasoning over novel combinations of known concepts persists across NLI, fact verification, and logical reasoning.
5. **Evidence quality > evidence quantity**: Denoising, decontextualization, and entity-level alignment all emphasize that filtering and grounding evidence matters more than retrieving more of it.

# Iteration 601 — Batch 601 Research Findings

**Date:** 2026-09-06  
**Prior Batch:** 600 (continuous dynamics, calibration-as-process, decoder latency binding, anisotropic error suppression)  
**Research Domains:** NLI, Contradiction Detection, Logical Reasoning  

---

## Batch 601 Summary

### What's NEW vs Batch 600

Batch 600 established that NeoTrix reasoning is fundamentally **continuous, not discrete** — calibration is a process not a checkpoint, gates are spacetime protocols, and decoder latency is the binding constraint. Batch 601 discovers **five concrete defects** in how LLMs and NLI systems actually implement these continuous dynamics, all of which confirm and deepen the batch 600 thesis.

---

## Defect 1: Prefix-Level NLI Exposes Decoder Latency as a Factual Consistency Bottleneck

**Source:** Harary et al. (2026), "PrefixNLI: Detecting Factual Inconsistencies as Soon as They Arise" — ACL 2026, Long Paper. https://aclanthology.org/2026.acl-long.63/

**Finding:** NLI models are trained to detect factual inconsistencies over *complete sentences*, but autoregressive generation makes decisions for each *evolving text prefix* during decoding. MiniTruePrefixes, a prefix-aware NLI model, outperforms standard NLI models by **5–14 F1 points** in prefix-level entailment. When integrated into controlled decoding, a 3B model matches the faithfulness of an 8B model at **half the memory**.

**New Defect over Batch 600:**  
Batch 600 identified decoder latency as the *binding constraint*. PrefixNLI proves this is not merely a hardware bottleneck but a **semantic mismatch**: the NLI evaluation window (complete sentence) is misaligned with the generation window (evolving prefix). The defect is that NeoTrix consciousness architecture uses full-sentence NLI gates when it should be running **incremental prefix-entailment checks** at each token. The "spacetime protocol" from batch 600 must operate at prefix granularity, not sentence granularity.

**Implication for NeoTrix:** GWT attention broadcast should carry a running prefix-entailment score, not a post-hoc sentence-level verdict. The decoder latency constraint means error detection must be *prospective* (what will this token entail?) not *retrospective* (did the completed sentence contradict?).

---

## Defect 2: Schema Collapse in Multi-Type Reasoning — Compartmentalization Required

**Source:** CARENLI (2026), "Compartmentalised Agentic Reasoning for Clinical NLI" — ACL Findings 2026. https://aclanthology.org/2026.findings-acl.545.pdf

**Finding:** When LLMs face heterogeneous reasoning demands (causal attribution, compositional grounding, epistemic verification, risk state abstraction), they suffer from **schema collapse**: a single generic inference pattern is reused across all types. Direct prompting achieves ~23% accuracy. CARENLI routes each premise–statement pair to a reasoning-family-specific solver with verification, achieving **~57% accuracy (+34 points)**. Largest gains on structurally demanding reasoning types (Risk State Abstraction: +51.9% relative).

**New Defect over Batch 600:**  
Batch 600 proposed that logical gates are spacetime protocols. CARENLI reveals that the defect is worse: **the gates themselves are type-collapsed**. A single "gate" mechanism cannot distinguish causal from epistemic from compositional inference. Each reasoning type has non-interchangeable validity conditions. The continuous dynamics from batch 600 must be *type-differentiated* — the same token stream passes through different gate protocols depending on the reasoning schema active at that moment.

**Implication for NeoTrix:** ConsciousnessTree branches must maintain **parallel gate registries** per reasoning type. The current single-channel GWT broadcast collapses distinct schemas into one heuristic. The fix is a router layer (as in CARENLI) that classifies the reasoning type before dispatching to schema-specific verifiers.

---

## Defect 3: Atomic-Level NLI Fails — Compositional Reasoning Breaks at the Fact Level

**Source:** Atomic-SNLI (2026), "Fine-Grained Natural Language Inference" — arXiv 2601.06528. https://arxiv.org/html/2601.06528v1

**Finding:** State-of-the-art NLI models achieve lower accuracy at the **atomic fact level** than at the sentence level (gaps of 0.61–0.79 percentage points). The conventional assumption that "entailment holds iff all atomic facts are entailed" fails in practice. Models exhibit poor *compositional reasoning* — they cannot reliably aggregate fine-grained fact judgments into coherent sentence-level verdicts.

**New Defect over Batch 600:**  
Batch 600 modeled error suppression as anisotropic. Atomic-SNLI shows the anisotropy has a specific direction: **bottom-up composition is the weak axis**. Individual fact recognition may be adequate, but the *aggregation step* — composing atomic judgments into holistic inference — introduces compounding errors. This is the discrete-vs-continuous issue from batch 600 manifested at the semantic level: atomic facts are discrete, but entailment is continuous, and the mapping between them is lossy.

**Implication for NeoTrix:** VSA HyperCube knowledge representation must encode not just atomic facts but their **compositional entailment relations** as continuous vectors. The KB embedding layer cannot treat atomic facts as independent — it must model the dependency graph between them, with the aggregation function itself being a learnable continuous transformation.

---

## Defect 4: Content Effect in Syllogisms Proves Belief Bias Dominates Over Logical Form

**Source:** Boethius (2026), "Thinking in Schemas: Robust Syllogistic Reasoning in LLMs" — ACL 2026 Long Paper. https://aclanthology.org/2026.acl-long.1897.pdf  
**Also:** SemEval-2026 Task 11 — Multiple systems (Proofbusters, GigitAI, AICOE-Tredence, TUCNLP) achieving 98–100% accuracy via symbolic abstraction. https://aclanthology.org/2026.semeval-1.182/ and related.

**Finding:** LLMs suffer from the **content effect** (CE): they conflate semantic plausibility with logical validity. Even GPT-4o shows ~49–50% CE with Boethius prompting. However, **symbolic abstraction** (stripping content, mapping to Aristotelian mood-figure forms, validating against 24 canonical forms) achieves near-perfect accuracy (98.95–100%) with zero content bias. Key insight from GigitAI: "content abstraction surprisingly *degrades* performance" when using simple placeholder substitution — semantic content provides essential parsing scaffolding alongside the bias it introduces.

**New Defect over Batch 600:**  
Batch 600 argued that calibration is continuous. The content effect proves calibration is *asymmetric*: models are well-calibrated for plausible conclusions but catastrophically miscalibrated for implausible-but-valid ones. The "continuous dynamics" from batch 600 must account for this **calibration anisotropy across the plausibility-validity axis**. Moreover, the GigitAI finding that stripping content *degrades* performance reveals a paradox: the continuous dynamics of semantic processing are *necessary* for parsing but *harmful* for judgment. The two stages require different dynamics.

**Implication for NeoTrix:** The E8 Hexagram reasoning engine must implement a **two-phase protocol**: Phase 1 uses continuous semantic dynamics for parsing/structuring (where content helps), Phase 2 switches to discrete symbolic dynamics for validity judgment (where content biases). This is not a simple gate but a **dynamic mode switch** — the consciousness architecture must detect when it's in "parse" vs "judge" mode and apply different dynamics to each.

---

## Defect 5: Cross-Source Evidence Conflict Requires Explicit Disagreement Quantification

**Source:** CoVer (2026), "Conflict-Aware Claim Verification" — arXiv 2609.00508. https://arxiv.org/abs/2609.00508  
**Also:** KCR (2026), "Knowledge Conflict Reasoning" — ACL 2026. https://aclanthology.org/2026.acl-long.1451.pdf  
**Also:** ContextConflict (2026), "Large Language Models in Resolving Contextual Knowledge Conflicts" — EMNLP 2026. https://arxiv.org/abs/2609.03148

**Finding:** Three convergent results:  
1. **CoVer** shows that evidence-level and aggregation-level conflicts (erroneous evidence mimicking authoritative sources) require explicit three-stage pipelines: schema normalization → factual consensus → support verification. Achieves 86% accuracy on conflict resolution.  
2. **KCR** shows that conflicting contexts induce **logic entanglement** — contradictory evidence interweaves into reasoning paths that are inseparable by standard backbones. A 7B model with KCR outperforms GPT-4o and GPT-5.1 on complex conflict adjudication via RLVR with logical coherence + consistency rewards.  
3. **ContextConflict** introduces a taxonomy of 6 contextual conflict types (factual, inferential, temporal, granularity, perspective, ambiguity) and finds a consistent **model bias towards earlier evidence** (positional preference) that obstructs resolution. A training-free activation steering method partially fixes this.

**New Defect over Batch 600:**  
Batch 600's anisotropic error suppression model assumed errors propagate in a single direction. These three findings reveal that conflict creates **bidirectional error propagation** — contradictory evidence doesn't just add noise, it creates entangled reasoning paths that actively *contaminate* both directions of inference. The positional bias (preferring earlier evidence) is a specific instance of the anisotropy: errors are suppressed in one temporal direction but amplified in the other. Additionally, the 6-type conflict taxonomy shows that "contradiction" is not atomic — NeoTrix's current binary entailment/contradiction/neutral schema misses inferential, temporal, granularity, perspective, and ambiguity conflicts.

**Implication for NeoTrix:** The NT-MEMORY domain's KB conflict resolution must implement: (1) a **multi-type conflict detector** (not binary), (2) **bidirectional propagation modeling** (not just feedforward), (3) **position-aware evidence weighting** (counteracting the temporal bias), and (4) **explicit disagreement scoring** between sources before aggregation. The continuous dynamics must include a conflict topology that distinguishes type, direction, and source of contradiction.

---

## Defect 6 (Bonus): Immediate Inference as Foundational Bottleneck

**Source:** IIBench (2026), "Immediate Inference: The Missing Foundation in LLM Logical Reasoning" — ACL 2026. https://aclanthology.org/2026.acl-long.808.pdf

**Finding:** Immediate inference (elementary operations over categorical propositions: conversion, obversion, contraposition) mediates **~40% of the effect** on syllogistic reasoning, with near-perfect correlation (ρ = 0.98) across reasoning benchmarks. Even SoTA models show systematic deficiencies. Models oscillate between structural reasoning and surface pattern matching, with inconsistent handling of quantifiers and negation.

**New Defect over Batch 600:**  
Batch 600 focused on high-level reasoning dynamics. IIBench reveals that the **foundational operator layer** — the ability to transform propositions without altering logical content — is the actual bottleneck. This is the atomic-level composition problem from Defect 3, but at the logical operator level: if the model cannot reliably execute basic transformations (All S are P → No S are non-P), then all downstream reasoning inherits this instability. The continuous dynamics from batch 600 are only as stable as the operator layer they run on.

---

## Sources Cited

1. Harary et al. (2026). "PrefixNLI: Detecting Factual Inconsistencies as Soon as They Arise." ACL 2026. https://aclanthology.org/2026.acl-long.63/
2. CARENLI (2026). "Compartmentalised Agentic Reasoning for Clinical NLI." ACL Findings 2026. https://aclanthology.org/2026.findings-acl.545.pdf
3. Huang et al. (2026). "Atomic-SNLI: Fine-Grained Natural Language Inference." arXiv 2601.06528. https://arxiv.org/html/2601.06528v1
4. Boethius (2026). "Thinking in Schemas: Robust Syllogistic Reasoning in LLMs." ACL 2026. https://aclanthology.org/2026.acl-long.1897.pdf
5. SemEval-2026 Task 11 — Multiple systems. https://aclanthology.org/2026.semeval-1.182/
6. CoVer (2026). "Conflict-Aware Claim Verification." arXiv 2609.00508. https://arxiv.org/abs/2609.00508
7. KCR (2026). "Knowledge Conflict Reasoning." ACL 2026. https://aclanthology.org/2026.acl-long.1451.pdf
8. ContextConflict (2026). "Large Language Models in Resolving Contextual Knowledge Conflicts." EMNLP 2026. https://arxiv.org/abs/2609.03148
9. IIBench (2026). "Immediate Inference: The Missing Foundation in LLM Logical Reasoning." ACL 2026. https://aclanthology.org/2026.acl-long.808.pdf
10. MGRN (2026). "Multi-Granularity Reasoning for Natural Language Inference." arXiv 2606.05181. https://arxiv.org/html/2606.05181
11. NLI Paradox (2026). "From Language to Logic: Unlocking General Reasoning by Training on NLI." OpenReview. https://openreview.net/forum?id=nXXau5t5Nh
12. LOREFACT (2026). "Bridging the Logic Gap in Fact-Checking." ACL Findings 2026. https://aclanthology.org/2026.findings-acl.346.pdf
13. KTC (2026). "Kernel Token Contradiction." arXiv 2608.22506. https://arxiv.org/html/2608.22506
14. Memory-First Fact-Checking (2026). arXiv 2608.29617. https://arxiv.org/abs/2608.29617
15. KG-CRAFT (2026). EACL 2026. https://aclanthology.org/2026.eacl-long.302/
16. Imperfective Paradox (2026). ACL 2026. https://aclanthology.org/2026.acl-long.689/
17. Neural DTS (2026). NALOMA 2026. https://aclanthology.org/2026.naloma-1.2/
18. Agree, Disagree, Explain (2026). ACL Findings 2026. https://aclanthology.org/2026.findings-acl.1342.pdf

---

## Convergence Map: Batch 600 → 601

| Batch 600 Thesis | Batch 601 Defect | Evidence |
|---|---|---|
| Continuous dynamics dominate | PrefixNLI: evaluation window mismatch (sentence vs prefix) | Defect 1 |
| Calibration is continuous | Content effect: calibration asymmetric across plausibility-validity axis | Defect 4 |
| Gates are spacetime protocols | CARENLI: gates are type-collapsed, not schema-differentiated | Defect 2 |
| Decoder latency is binding | PrefixNLI: 3B model matches 8B at half memory via prefix-aware decoding | Defect 1 |
| Error suppression is anisotropic | KCR: bidirectional error propagation from conflicts; temporal positional bias | Defect 5 |
| (New) Composition breaks at fact level | Atomic-SNLI: aggregation lossy, bottom-up composition weak axis | Defect 3 |
| (New) Operator layer is bottleneck | IIBench: immediate inference mediates 40% of syllogistic reasoning | Defect 6 |

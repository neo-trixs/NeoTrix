# Iteration Batch 586 — ML Interpretability & Explainability Research

**Date:** 2026-09-06
**Batch 585 provenance:** Parser attack surface, protobuf OOM amplification, gRPC auth bypass, MQTT wildcard ACL bypass, SSE token leakage

---

## Search 1: ML Interpretability — SHAP, LIME, Feature Importance (2026)

### Key Sources
1. **tolumichael.com** — "Model Interpretability Techniques in AI (2026)" (2026-02-23)
2. **dataexpertise.in** — "XAI – SHAP, LIME & Model Interpretability" (2026-08-30)
3. **aiinsightsblogs.com** — "SHAP Values and LIME Explained" (2026-09-01)
4. **programming-helper.com** — "Explainable AI 2026: SHAP, LIME, Python" (2026-01-28)
5. **futureagi.com** — "AI Explainability in 2026: Tools, Techniques, Frameworks" (2026-05-14)
6. **semanticscholar.org** — "Comparative Study of SHAP and LIME" (2025-07-28)

### New Defects / Findings vs Batch 585

| # | Defect/Finding | Detail |
|---|---------------|--------|
| S1-1 | **Faithfulness gap in LIME for production** | LIME is sensitive to perturbation sampling and kernel choice; teams deploying LIME without validation produce misleading explanations. Batch 585 had no coverage of explanation faithfulness testing. |
| S1-2 | **SHAP computational cost at scale** | SHAP TreeExplainer is O(TLD²) — infeasible for real-time >100K feature attribution. No existing NT-MIND caching strategy for SHAP explanations. |
| S1-3 | **Counterfactual explanation as production requirement** | EU AI Act mandates "what-if" scenario explanations for high-risk systems. NeoTrix has no counterfactual explanation capability. |
| S1-4 | **EU AI Act compliance gap** | High-risk AI obligations phase in Aug 2026; NeoTrix lacks standardized explainability metric tracking. |
| S1-5 | **LIME for images/LLMs insufficient** | LIME super-pixel perturbation produces unstable explanations for deep nets; no adapter in NT-CORE for deep model attribution. |

---

## Search 2: Explainable AI — XAI, Feature Importance (2026)

### Key Sources
1. **ust.com** — "From Explainability to Control: 2026 Executive View" (2026)
2. **xaiworldconference.com** — 4th World Conference on XAI, Fortaleza, Brazil (2026-07-01)
3. **nature.com** — "Explainable AI needs formalization" (Haufe et al., 2026-04-24, npj AI vol.2 art.42)
4. **tandfonline.com** — "XAI for Multi-Class Supply Chain Delivery Prediction" (2026-08-28)
5. **ibm.com** — "What is Explainable AI" (updated 2026-02-26)
6. **sanishtech.com** — "XAI: Why Transparency Matters in 2026" (2026-03-11)

### New Defects / Findings vs Batch 585

| # | Defect/Finding | Detail |
|---|---------------|--------|
| S2-1 | **Formalization crisis (Nature 2026)** | Haufe et al. (npj AI 2026) prove standard feature attributions can produce false-positive explanations — attribution methods that "lie" are formally impossible to validate without causal context. NeoTrix has no formal attribution validation. |
| S2-2 | **cc-Shapley: causal context required** | Martin & Haufe (arXiv:2602.20396, 2026) show multivariate Shapley values need causal graph context to avoid spurious attributions. NT-CORE's current SHAP integration ignores causal structure. |
| S2-3 | **Impossibility theorems for feature attribution** | Bilodeau et al. (PNAS 2024) prove feature attribution cannot satisfy all desirable axioms simultaneously. Batch 585 assumed attributions are unconditionally valid. |
| S2-4 | **Mechanistic interpretability for LLMs** | Anthropic, Google DeepMind ship circuit-level tools; TransformerLens becoming production-relevant. NT-MIND has no mechanistic interpretability pathway. |
| S2-5 | **Faithfulness evaluator as production gate** | Post-hoc faithfulness evaluators now required for LLM CoT tracing. NeoTrix's consciousness loop lacks faithfulness validation on its own reasoning outputs. |

---

## Search 3: Attention Visualization — Attention Map, Transformer Interpretability (2026)

### Key Sources
1. **arxiv.org** — "Interpreting Transformers Through Attention Head Intervention" (Kadem & Zheng, 2026-03-02)
2. **simulations4all.com** — "Transformer Attention Mechanism Visualizer" (2026-02-21)
3. **lrec.elra.info** — "Explaining Explanations: Interpretability Methods for Discourse Analysis" (LREC 2026-05-01)
4. **sciencedirect.com** — "Interpretable Point Cloud Transformers via Entropy-Guided Attention" (2026)
5. **mbrenndoerfer.com** — "Attention Visualization: Extracting and Interpreting Weights" (2026-02-11)
6. **github.com/kryptologyst** — "Attention-Visualization-for-Transformers" toolkit

### New Defects / Findings vs Batch 585

| # | Defect/Finding | Detail |
|---|---------------|--------|
| S3-1 | **Paradigm shift: visualization → intervention** | Kadem & Zheng (2026) prove attention heatmap observation is correlational, not causal. Ablation/intervention needed. NT-CORE treats attention weights as directly interpretable — **incorrect**. |
| S3-2 | **Attention ≠ faithfulness** | Attention weights in GPT-class models are causal-masked (lower-triangular) — earlier tokens cannot attend to later ones. Interpretation must account for this asymmetry. NeoTrix's GWT attention routing has no such constraint check. |
| S3-3 | **Entropy-guided attention for uncertainty** | New method integrates information entropy with self-attention to quantify weight distribution uncertainty. NT-CORE's attention aggregation has no uncertainty quantification. |
| S3-4 | **BertViz limitations for long sequences** | Arc-based visualization fails at >30 tokens. Matrix heatmaps needed. NT-IO visualization pipeline has no long-sequence attention viz strategy. |
| S3-5 | **Attention rollout is approximate** | Attention rollout ignores value projections and nonlinearities; is not faithful. NT-MIND may be using rollout-style aggregation for cross-layer attention — needs audit. |

---

## Consolidated New Defects Summary (vs Batch 585)

| Severity | ID | Defect | Domain |
|----------|-----|--------|--------|
| **CRITICAL** | S2-1 | Formal proof that standard attributions can lie — no validation in NeoTrix | NT-CORE |
| **CRITICAL** | S3-1 | Attention weights treated as causal but are only correlational | NT-CORE/GWT |
| **HIGH** | S2-3 | Impossibility theorems invalidate all-attributions-valid assumption | NT-CORE |
| **HIGH** | S2-2 | cc-Shapley requires causal graph for valid multivariate attribution | NT-CORE |
| **HIGH** | S3-3 | No uncertainty quantification on attention weight distributions | NT-CORE |
| **HIGH** | S1-4 | EU AI Act compliance gap — no explainability metric tracking | NT-META |
| **HIGH** | S2-5 | Consciousness loop lacks faithfulness validation on reasoning outputs | NT-MIND |
| **MEDIUM** | S1-1 | LIME faithfulness unvalidated in production explanations | NT-MIND |
| **MEDIUM** | S1-2 | SHAP TreeExplainer O(X²) infeasible at >100K features | NT-MIND |
| **MEDIUM** | S3-2 | GPT causal mask not accounted for in attention interpretation | NT-CORE |
| **MEDIUM** | S3-4 | No long-sequence attention visualization capability | NT-IO |
| **MEDIUM** | S3-5 | Rollout-style attention aggregation is approximate/unfaithful | NT-MIND |
| **MEDIUM** | S1-3 | No counterfactual explanation capability for EU compliance | NT-ACT |
| **MEDIUM** | S2-4 | No mechanistic interpretability pathway for LLM components | NT-MIND |
| **LOW** | S1-5 | LIME super-pixel unstable for deep models | NT-MIND |

---

## What's NEW vs Batch 585

| Dimension | Batch 585 | Batch 586 |
|-----------|-----------|-----------|
| **Attack surface** | Protocol-level (protobuf, gRPC, MQTT, SSE) | **Interpretability-level** — explanation faithfulness as new attack surface |
| **Formal guarantees** | None | Impossibility theorems proven for feature attribution |
| **Causal vs correlational** | Not addressed | Attention ≠ causal (paradigm shift documented) |
| **Compliance** | Not addressed | EU AI Act explainability obligations phase-in Aug 2026 |
| **Uncertainty** | Not addressed | Entropy-guided attention for uncertainty quantification |
| **Counterfactual** | Not addressed | Counterfactual explanations as production requirement |
| **Mechanistic interpretability** | Not addressed | Circuit-level tools emerging for production LLMs |

---

## Sources Cited

1. Haufe et al., "Explainable AI needs formalization," *npj AI* vol.2 art.42, 2026. https://doi.org/10.1038/s44387-026-00095-1
2. Martin & Haufe, "cc-Shapley: Measuring Multivariate Feature Importance Needs Causal Context," arXiv:2602.20396, 2026.
3. Kadem & Zheng, "Interpreting Transformers Through Attention Head Intervention," arXiv:2601.04398v4, 2026-03-02.
4. Bilodeau et al., "Impossibility theorems for feature attribution," *PNAS* 121, 2024.
5. Escouflaire et al., "Explaining Explanations: Interpretability Methods for Discourse Analysis," LREC 2026. https://doi.org/10.63317/3mygtrz7g6vj
6. Tolulope Michael, "Model Interpretability Techniques in AI 2026," 2026-02-23. https://blog.tolumichael.com/what-are-model-interpretability-techniques-ai-2026/
7. Future AGI, "AI Explainability in 2026: Tools, Techniques, and Frameworks," 2026-05-14. https://futureagi.com/blog/ai-explainability-tools-techniques-2025/
8. Brenndoerfer, "Attention Visualization: Extracting and Interpreting Weights," 2026-02-11. https://mbrenndoerfer.com/writing/attention-visualization
9. "Interpretable Point Cloud Transformers via Entropy-Guided Attention," ScienceDirect, 2026.
10. Ball & Gu, "Explainable AI for Multi-Class Supply Chain Delivery Prediction," *J. Computer Information Systems*, 2026. https://doi.org/10.1080/08874417.2026.2723097

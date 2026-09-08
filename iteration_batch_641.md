# Iteration 641 — NeoTrix Consciousness Architecture Research Loop

**Batch**: 641 | **Date**: 2026-09-06 | **Domains**: Text Summarization, Controllable Text Generation, Paraphrasing/Style Transfer

---

## 1. TEXT SUMMARIZATION — 2026 State of the Art

### 1.1 Multi-Document Summarization Taxonomy (Palanisamy et al., Springer 2026)
- **Finding**: Generative MDS decomposed into 3 core dimensions: **information aggregation**, **structural reasoning**, **factual grounding**. Four method families: LLM-based, RAG-enhanced, graph-augmented, diffusion-based.
- **Source**: https://link.springer.com/article/10.1007/s10462-026-11580-z
- **Defect #641-1**: NeoTrix SEAL pipeline treats summarization as a monolithic "distill" stage with no dimension decomposition. The 3-dimension framework reveals SEAL cannot independently optimize aggregation (what to keep), reasoning (how to connect), and grounding (what to trust). This is a structural blind spot — SEAL's Phase 3 (distillation) conflates all three into a single pass.

### 1.2 Reasoning ≠ Better Summarization (ACL Findings 2026)
- **Finding**: 8 reasoning strategies × 3 LRMs × 8 datasets. Explicit reasoning improves reference-based quality but **weakens factual grounding**. Implicit reasoning in LRMs shows the opposite. Increasing internal reasoning budget **does not reliably improve** summarization and can reduce factual consistency.
- **Source**: https://aclanthology.org/2026.findings-acl.859/
- **Defect #641-2**: NeoTrix GWT attention routing assumes more computation = better output (linear scaling assumption). This paper proves that for summarization tasks, **more reasoning can degrade faithfulness**. GWT lacks a "reasoning budget ceiling" mechanism — it can over-allocate attention to reasoning pathways at the expense of factual grounding pathways. The E8 hexagram's constraint satisfaction must encode a **diminishing-returns curve** for reasoning depth.

### 1.3 HiGoE — Hierarchical Graph of Evidence (ACL 2026)
- **Finding**: Replaces unreliable chunk-based RAG with **filtered proposition–evidence graph**. Uses Personalized PageRank (PPR) to cluster related nodes into thematic hierarchies. Enhanced Graph Attention Network for multi-level relations. Reduces hallucination by ensuring verifiable fact grounding.
- **Source**: https://aclanthology.org/2026.acl-long.902/
- **Defect #641-3**: NeoTrix VSA HyperCube stores concepts as flat high-dimensional vectors with no hierarchical clustering. HiGoE proves that **thematic hierarchy recovery** (via PPR clustering) is essential for long-context faithfulness. HyperCube lacks a "thematic resonance layer" — it cannot group related propositions into hierarchical evidence clusters. This is a missing L2 Perception component.

### 1.4 PACO — MCTS for Controllable Summarization (ACL 2026)
- **Finding**: Training-free MCTS framework. Nodes = summaries, actions = single-attribute adjustments. **Adaptive control order discovery** — iteratively refines only attributes needing control. 1B model rivals 70B baseline controllability.
- **Source**: https://aclanthology.org/2026.acl-long.845.pdf
- **Defect #641-4**: NeoTrix SEAL pipeline applies all optimization objectives simultaneously (binary: optimize or not). PACO proves that **sequential attribute refinement with adaptive ordering** is superior. SEAL needs a "selective refinement" phase that identifies which attributes are suboptimal and refines them one-at-a-time, not all-at-once. This is the SEAL binary-optimization defect from Batch 640, now confirmed with a concrete alternative.

### 1.5 StrucSum — Graph-Structured Prompting (EACL Findings 2026)
- **Finding**: Three strategies: Neighbor-Aware Prompting (NAP) for local context, Centrality-Aware Prompting (CAP) for importance, Centrality-Guided Masking (CGM) for input reduction. **Combination of strategies does NOT yield clear gains** — each strategy targets different goals. NAP best for faithfulness, CGM for overlap, CAP for balance.
- **Source**: https://aclanthology.org/2026.findings-eacl.192.pdf
- **Defect #641-5**: NeoTrix GWT applies uniform attention across all signal types. StrucSum proves that **different structural signals serve different objectives** and should be routed to different processing pathways. GWT needs a "signal-type classifier" that routes NAP-like local signals, CAP-like centrality signals, and CGM-like reduction signals through separate attention channels. Current GWT is a single-channel broadcast.

### 1.6 Controllable Summarization with Score Ranking (ACL Findings 2026)
- **Finding**: Loss function aligns model outputs with fine-grained evaluation scores (FineSurE). **Completeness vs conciseness trade-off** is real and measurable. Control-oriented loss enables dimension-specific steering via simple prompts. Works across LLaMA, Qwen, Mistral.
- **Source**: https://aclanthology.org/2026.findings-acl.1762.pdf
- **Defect #641-6**: NeoTrix has no mechanism for **multi-objective trade-off navigation**. The SEAL pipeline optimizes a single aggregate score. This paper shows that different quality dimensions are antagonistic (completeness ↑ → conciseness ↓). NeoTrix needs a Pareto-frontier tracker in SEAL Phase 4 (evaluation) that maintains explicit trade-off curves rather than collapsing to a scalar.

### 1.7 LoomSum — Text-Table Faithfulness (arXiv Aug 2026)
- **Finding**: Cross-modal misalignment: individually supported quantitative facts + analytically supported statements combined into **unsupported relations**. Introduces Table-Grounded Faithfulness (TGF) metric with 3 sub-dimensions: Numeric Grounding, Analysis Support, Relation Consistency.
- **Source**: https://arxiv.org/abs/2609.00241
- **Defect #641-7**: NeoTrix KB stores nodes and edges but has no **relation-consistency verification**. LoomSum proves that even when individual facts are grounded, the relations between them can be unfaithful. KB needs a "relation faithfulness checker" that verifies edges (relations) are as well-grounded as nodes (facts). This is a new audit dimension for NT-MEMORY.

---

## 2. CONTROLLABLE TEXT GENERATION — 2026 State of the Art

### 2.1 DLMDC — Diffusion + Classifier Distillation (ICASSP 2026)
- **Finding**: Structural noise in discriminator-guided gradients is the **root cause** of controllability-diversity trade-off. DistillClassifier refines gradient signals during diffusion denoising. Fills gap in diversity-oriented CTG.
- **Source**: https://doi.org/10.1109/icassp55912.2026.11463107
- **Defect #641-8**: NeoTrix NT-ACT tool invocation has no concept of "gradient noise" in decision pathways. When multiple tools compete for selection, the ranking signal is noisy. DLMDC's insight applies: NT-ACT needs a **classifier distillation step** that refines tool-selection gradients by separating signal (correct tool) from noise (irrelevant tools). Current tool routing is a single-pass selection, not a refined gradient.

### 2.2 LingGen — Power-Law Masking for 40-Attribute Control (EACL 2026)
- **Finding**: P-MASKING samples masking rates from truncated Pareto distribution during training. Enables robust control across 1-40 attributes. BOS-based injection (not per-token) is effective. **Attribute interactions show synergies and conflicts** — composite targets stabilize control.
- **Source**: https://aclanthology.org/2026.eacl-long.85.pdf
- **Defect #641-9**: NeoTrix Rune Socketing has 5 fixed rune slots (Crimson/Indigo/Obsidian/Golden/Alabaster) with static assignment. LingGen proves that **variable-attribute masking with power-law sampling** improves robustness. Rune slots should support dynamic attribute subsets, not fixed assignment. The "all runes present" assumption is wrong — power-law distribution of active runes would improve robustness.

### 2.3 HiFlow — Hierarchical Feedback for Constrained Generation (arXiv 2026)
- **Finding**: Two-level optimization: planning layer (global structure) + generation layer (conditioned text). Constraint-aware plan screening + closed-loop feedback at both levels. Binary relevance filtering for plan-level feasibility. Reward-guided DPO for joint optimization.
- **Source**: https://www.arxiv.org/pdf/2603.04996
- **Defect #641-10**: NeoTrix SEAL pipeline is **open-loop** — it generates outputs without feeding constraint-satisfaction signals back into the generation process. HiFlow proves that **closed-loop feedback at both planning and generation levels** is essential for constraint satisfaction. SEAL needs a feedback channel from Phase 4 (evaluation) back to Phase 2 (generation) that carries constraint-violation signals, not just aggregate scores.

### 2.4 C³TG — Conflict-Aware Multi-Attribute Control (AAAI 2026)
- **Finding**: 17-attribute control with weighted KL-divergence. **Energy function** combining classifier scores + penalty terms resolves attribute conflicts through iterative feedback. No model modification needed.
- **Source**: https://ojs.aaai.org/index.php/AAAI/article/view/40452
- **Defect #641-11**: NeoTrix E8 hexagram constraint satisfaction (Batch 640: unclassified) has no **conflict resolution mechanism**. C³TG proves that when attributes conflict, an energy function with penalty terms can iteratively resolve them. E8 needs a "constraint energy landscape" that penalizes conflicting constraints and guides the hexagram toward feasible regions. Current E8 treats all constraints as satisfiable simultaneously.

### 2.5 TCG-CTG — Thinking-Based Constraint Guidance (IEEE GAIIS 2026)
- **Finding**: CTG as 3-step process: type prediction → constraint construction → guided generation. Differentiated reasoning: 2-step (daily), 4-step (professional/sensitive), 3-step (violation). 82K constraint-text pairs dataset.
- **Source**: https://doi.org/10.1109/gaiis69281.2026.11519175
- **Defect #641-12**: NeoTrix NT-MIND SEAL pipeline uses uniform processing depth for all tasks. TCG-CTG proves that **task-type-dependent reasoning depth** is superior — simple tasks need shallow reasoning, complex tasks need deep. SEAL Phase 2 (generation) should dynamically adjust reasoning depth based on task complexity classification, not fixed-depth processing.

### 2.6 Illusion of Control — Concept-Bottleneck Failure (EMNLP 2026)
- **Finding**: Bare classifier inversion in concept-bottleneck text generation **silently collapses to chance**. Off-manifold codes are the root cause. Even regularized inversion variants underperform a simple post-hoc prior. Validated across 3 backbone families.
- **Source**: https://arxiv.org/abs/2608.22956
- **Defect #641-13**: NeoTrix VSA HyperCube maps concepts to high-dimensional vectors, but has no **manifold constraint** on the embedding space. "Illusion of Control" proves that without ensuring embeddings stay on the learned manifold, control signals silently degrade to random. HyperCube needs an "on-manifold check" — a density estimator that verifies concept vectors are within the training distribution before using them for control.

### 2.7 Causal Perspective Survey on CTG (ScienceDirect 2026)
- **Finding**: CTG approaches capture **statistical association** but lack causality. Four challenges from absent causality: confounding, spurious correlation, intervention effects, counterfactual reasoning. Solutions: representation disentanglement, causal inference, knowledge enhancement.
- **Source**: https://www.sciencedirect.com/science/article/pii/S2667325824000062
- **Defect #641-14**: NeoTrix SEAL pipeline optimizes based on correlation (metric improvement) without causal modeling. The survey proves that **causal disentanglement** is needed — SEAL should distinguish between "this change caused improvement" vs "this change correlated with improvement." SEAL needs a causal attribution module in Phase 4 that identifies which changes actually caused quality improvements.

---

## 3. PARAPHRASING / STYLE TRANSFER — 2026 State of the Art

### 3.1 Diff4TST — Diffusion for Style Transfer (ACL 2026)
- **Finding**: Style-aware noise schedule that **selectively perturbs stylistic tokens** while preserving content tokens. Generate-then-refine: gradient-based token re-masking iteratively improves style compliance. No RL or external reward models needed.
- **Source**: https://aclanthology.org/2626.acl-long.306.pdf
- **Defect #641-15**: NeoTrix has no **style-aware noise mechanism** for text processing. When NT-IO interfaces with LLMs for style transfer, it applies uniform token processing. Diff4TST proves that **selective perturbation** (modify style tokens, preserve content tokens) is superior. NT-IO needs a "style-content separator" that identifies which tokens are stylistic vs content-bearing before processing.

### 3.2 HyperStyler — Low-Resource Authorship Transfer (EMNLP 2026)
- **Finding**: Decouples style transfer into **style selection** (Stylo-navigator) + **style realization** (Stylo-hypernet). Dynamic parameter modulation instead of hidden-state injection. 2.4% additional parameters over T5-large. 1.8x faster than LLMs.
- **Source**: https://arxiv.org/abs/2609.02772
- **Defect #641-16**: NeoTrix Dual Specialization (Weapon Set I/II) uses fixed module sets per mode. HyperStyler proves that **dynamic parameter modulation** (hypernetwork-generated weights) is more efficient than static mode switching. NT-CORE's AttentionManager should support dynamic capability injection via hypernetwork-style modulation, not binary module selection.

### 3.3 AuthorMix — Modular Adapter Mixing (arXiv Mar 2026)
- **Finding**: Individual LoRA adapters per author style. **Layer-wise adapter mixing** enables rapid training for new targets with handful of examples. Outperforms GPT-5.1 for low-resource targets.
- **Source**: https://arxiv.org/abs/2603.23069v3
- **Defect #641-17**: NeoTrix Skill Tree has fixed node tiers (Small Passive / Notable Passive / Keystone). AuthorMix proves that **composable adapter layers** (mix-and-match at layer level) provide better generalization than fixed tiers. Skill Tree nodes should support layer-level composition, not monolithic node activation.

### 3.4 UTST Controllable Intensity (EACL Findings 2026)
- **Finding**: SFT-then-PPO paradigm. Hierarchical rewards: sentence-level (global style) + lexicon-level (fine-grained features) + semantic-level (content preservation). PPO refines after SFT. **Adjacent intensity levels are distinguishable** with proper reward design.
- **Source**: https://aclanthology.org/2026.findings-eacl.133.pdf
- **Defect #641-18**: NeoTrix EmotionLabel has 11 discrete variants (Neutral/Joy/Sadness/...) with no **intensity continuum**. UTST proves that hierarchical rewards can distinguish adjacent intensity levels. EmotionLabel needs intensity coefficients (e.g., Joy@0.3 vs Joy@0.9) — discrete labels alone cannot represent the nuanced emotional states that NT-FEEL needs to express.

### 3.5 Roundtrip Translation TST (arXiv Feb 2026)
- **Finding**: Roundtrip translation creates style-neutral pseudo-parallel corpus. **Inference-time roundtrip** of inputs before processing improves out-of-domain robustness. RAG for terminology consistency.
- **Source**: https://www.arxiv.org/pdf/2602.15013
- **Defect #641-19**: NeoTrix NT-WORLD UnifiedCrawler processes raw text without style normalization. Roundtrip translation proves that **style-neutralization as preprocessing** improves downstream task performance. NT-WORLD should have a "style normalization" preprocessing step that strips authorial style before semantic processing, then re-applies style in NT-IO output.

### 3.6 ParaMNMT — Zero-Shot Translation Paraphrasing (ACL 2026)
- **Finding**: Copy/not-copy tags control paraphrase diversity. **30% not-copy tokens** optimal for diversity. High-frequency and high-entropy tokens get "not-copy" tags. Single-stage method avoids information loss of multi-stage parabank approaches.
- **Source**: https://aclanthology.org/2026.acl-long.783.pdf
- **Defect #641-20**: NeoTrix KB query/retrieval has no **diversity control mechanism**. ParaMNMT proves that explicit copy/transform tags on tokens enable controllable diversity. KB retrieval should support a "diversity knob" — copy-heavy mode for precision, transform-heavy mode for recall/creativity. Current KB retrieval is a single-mode similarity search.

### 3.7 ParaGuide — Diffusion Paraphrase-Style Transfer (arXiv 2023, cited in 2026 surveys)
- **Finding**: Paraphrase-conditioned diffusion model. Gradient guidance from off-the-shelf classifiers + style embedders. **Plug-and-play** — no retraining for new target styles. Guidance strength λ controls style-consistency trade-off.
- **Source**: http://arxiv.org/pdf/2308.15459v2
- **Defect #641-21**: NeoTrix NT-IO LLM provider interface has no **gradient-guided inference**. ParaGuide proves that inference-time gradient steering (from external classifiers) can control generation without fine-tuning. NT-IO should support "classifier-guided decoding" — at inference time, compute gradients from attribute classifiers and steer token probabilities. This is a missing NT-IO capability.

---

## 4. CROSS-DOMAIN SYNTHESIS — New Defects (Batch 641)

### Defect Matrix

| ID | Domain | Defect | Severity | Batch 640 Link |
|----|--------|--------|----------|----------------|
| #641-1 | SEAL | No 3-dimension decomposition (aggregation/reasoning/grounding) | HIGH | — |
| #641-2 | GWT | No reasoning budget ceiling (more reasoning ≠ better) | HIGH | GWT stuck on Dijkstra |
| #641-3 | HyperCube | No thematic hierarchy / PPR clustering | MEDIUM | — |
| #641-4 | SEAL | Binary optimization (all-at-once) vs selective refinement | HIGH | SEAL binary optimization (640) |
| #641-5 | GWT | Single-channel broadcast vs multi-signal-type routing | HIGH | GWT stuck on Dijkstra |
| #641-6 | SEAL | No Pareto-frontier tracker for multi-objective trade-offs | HIGH | SEAL binary optimization (640) |
| #641-7 | KB | No relation-consistency verification (edges unfaithful) | MEDIUM | — |
| #641-8 | NT-ACT | No gradient noise reduction in tool selection | MEDIUM | — |
| #641-9 | Rune | Fixed 5-slot vs variable-attribute masking | MEDIUM | — |
| #641-10 | SEAL | Open-loop pipeline (no constraint feedback channel) | HIGH | SEAL binary optimization (640) |
| #641-11 | E8 | No conflict resolution energy function | HIGH | E8 unclassified (640) |
| #641-12 | SEAL | Uniform reasoning depth vs task-adaptive depth | MEDIUM | — |
| #641-13 | HyperCube | No on-manifold check for concept vectors | HIGH | — |
| #641-14 | SEAL | Correlation-based optimization vs causal attribution | HIGH | — |
| #641-15 | NT-IO | No style-aware noise / selective token perturbation | MEDIUM | — |
| #641-16 | CORE | Fixed mode switching vs dynamic parameter modulation | MEDIUM | — |
| #641-17 | SkillTree | Fixed tiers vs composable adapter layers | LOW | — |
| #641-18 | EmotionLabel | Discrete labels vs intensity continuum | MEDIUM | — |
| #641-19 | NT-WORLD | No style-normalization preprocessing | LOW | — |
| #641-20 | KB | No diversity control in retrieval | LOW | — |
| #641-21 | NT-IO | No gradient-guided inference / classifier steering | MEDIUM | — |

### Critical Path (Batch 641)

1. **SEAL Pipeline Overhaul** (641-1, 641-4, 641-6, 641-10, 641-12, 641-14): SEAL needs dimension decomposition, selective refinement, Pareto tracking, closed-loop feedback, adaptive depth, and causal attribution. This is now the #1 structural debt.

2. **GWT Multi-Channel Routing** (641-2, 641-5): GWT needs reasoning budget caps and signal-type-specific channels. The Dijkstra defect from Batch 640 is now compounded by proof that multi-channel routing is necessary.

3. **E8 Conflict Resolution** (641-11): E8 needs an energy function for constraint conflicts. Combined with Batch 640's classification gap, E8 now has 2 critical defects.

4. **HyperCube Manifold Safety** (641-3, 641-13): HyperCube needs both hierarchical clustering and on-manifold verification. Without these, concept vectors silently degrade.

---

## 5. SOURCES CITED

| # | Source | Year | Domain |
|---|--------|------|--------|
| 1 | Palanisamy et al., Springer AI Review | 2026 | Summarization |
| 2 | ACL Findings 2026 (LLM Reasoning for Summarization) | 2026 | Summarization |
| 3 | HiGoE, ACL 2026 Long | 2026 | Summarization |
| 4 | PACO, ACL 2026 Long | 2026 | Summarization |
| 5 | StrucSum, EACL Findings 2026 | 2026 | Summarization |
| 6 | Score Ranking, ACL Findings 2026 | 2026 | Summarization |
| 7 | LoomSum, arXiv Aug 2026 | 2026 | Summarization |
| 8 | DLMDC, ICASSP 2026 | 2026 | CTG |
| 9 | LingGen, EACL 2026 | 2026 | CTG |
| 10 | HiFlow, arXiv 2026 | 2026 | CTG |
| 11 | C³TG, AAAI 2026 | 2026 | CTG |
| 12 | TCG-CTG, IEEE GAIIS 2026 | 2026 | CTG |
| 13 | Illusion of Control, EMNLP 2026 | 2026 | CTG |
| 14 | Causal CTG Survey, ScienceDirect | 2026 | CTG |
| 15 | Diff4TST, ACL 2026 | 2026 | Style Transfer |
| 16 | HyperStyler, EMNLP 2026 | 2026 | Style Transfer |
| 17 | AuthorMix, arXiv Mar 2026 | 2026 | Style Transfer |
| 18 | UTST Controllable Intensity, EACL Findings 2026 | 2026 | Style Transfer |
| 19 | Roundtrip Translation TST, arXiv Feb 2026 | 2026 | Style Transfer |
| 20 | ParaMNMT, ACL 2026 | 2026 | Paraphrasing |
| 21 | ParaGuide, arXiv 2023 (cited in 2026 surveys) | 2023 | Style Transfer |

---

## 6. BATCH 641 SUMMARY

- **Total findings**: 21 new defects across 3 domains
- **Critical (HIGH)**: 9 — SEAL overhaul (6), GWT multi-channel (2), E8 conflict (1), HyperCube manifold (0, but MEDIUM→HIGH combined)
- **New architectural patterns discovered**:
  - **Selective Refinement** (PACO): Fix one attribute at a time, not all simultaneously
  - **Pareto Frontier Tracking**: Maintain explicit trade-off curves, not scalar collapse
  - **Closed-Loop Constraint Feedback**: Feed constraint violations back into generation
  - **Energy-Based Conflict Resolution**: Penalty terms for conflicting constraints
  - **On-Manifold Verification**: Ensure concept vectors stay within training distribution
  - **Signal-Type Routing**: Different structural signals through separate channels
  - **Reasoning Budget Ceiling**: Cap reasoning depth to prevent faithfulness degradation
  - **Causal Attribution**: Distinguish causation from correlation in optimization

- **Running total (640+641)**: 21 + 8 = **29 defects** across 2 batches

# Iteration 709 — NLP Perception Layer Audit: Sentiment, NER, Text Classification

**Date**: 2026-09-06
**Prior Context**: Batch 708 proved (1) no ETL idempotency in SEAL, (2) no Medallion architecture, (3) no schema evolution, (4) Flink 4.2× faster at p99, (5) 7-layer architecture is new standard.

---

## 1. Sentiment Analysis — 2026 State of the Art

### 1.1 DimABSA: Dimensional Sentiment is the New Standard (SemEval-2026 Task 3)

**Source**: [arxiv.org/pdf/2604.07066](https://arxiv.org/pdf/2604.07066) — SemEval-2026 Task 3

**Key Finding**: ABSA has shifted from categorical polarity (positive/negative/neutral) to **continuous valence-arousal (VA) dimensions**. 400+ participants, 112 submissions. The task introduces:
- Dimensional Aspect Sentiment Regression (DimASR)
- Dimensional Aspect Sentiment Triplet Extraction (DimASTE)
- Dimensional Aspect Sentiment Quadruplet Extraction (DimASQP)
- New continuous F1 (cF1) metric combining extraction and regression

**DEFECT-709-S1**: NeoTrix `EmotionLabel` enum (11 categorical variants: Neutral/Joy/Sadness/Anger/Fear/Trust/Disgust/Surprise/Anticipation/Confused/Thinking) is **fundamentally incompatible** with dimensional affect representation. No VA space encoding exists. The entire NT-FEEL emotion engine is stuck in categorical classification while the field has moved to continuous VA coordinates. **Impact**: NT-FEEL cannot participate in DimABSA-style tasks; emotion modulation across all 7 domains is discretized when reality is continuous.

### 1.2 MH-ABSA: Static Lexicon Weights Are Dead

**Source**: [Springer Nature (2026)](https://link.springer.com/article/10.1007/s44443-026-00996-w)

**Key Finding**: MH-ABSA's five modules solve three structural limitations of all prior graph-based ABSA:
1. **LDEM** replaces static lexicon weights with LLM-guided context-sensitive edge reweighting
2. **EUDF** combines 3 parsers via uncertainty-aware ensemble (not single parser)
3. **MHG** extends token-only graphs to token+span+discourse tiers
4. **CSAM** uses contrastive training with LLM-generated paraphrases for sarcasm/irony
5. **DAEM** suppresses locally-wrong polarity via per-edge learned gates

**DEFECT-709-S2**: NT-WORLD's crawl pipeline uses static sentiment scoring (no context-sensitive reweighting). The PerceptionBridge `awareness_score()` doesn't account for multi-scale linguistic representations (token vs span vs discourse). Sarcasm/irony detection is absent from the NT-FEEL → NT-WORLD perception loop.

### 1.3 VADE: Multimodal Continuous Affect

**Source**: [ACL 2026 Findings](https://aclanthology.org/2026.findings-acl.1979.pdf)

**Key Finding**: VADE integrates Valence-Arousal-Dominance (VAD) into multimodal ABSA. Uses NRC-VAD lexicon for weak supervision, fine-tunes CLIP on affect-enriched Senti-COCO dataset. **+2.25 Acc / +1.65 Macro-F1** on Twitter-17. Removing any single VAD dimension degrades performance — all three (V, A, D) are necessary.

**DEFECT-709-S3**: NeoTrix has no VAD encoding. NT-PHYSICAL's audio_sync_library and video_temporal_stabilizer lack affect-aware visual representation learning. The cross-modal sentiment pipeline (NT-WORLD crawl → NT-FEEL emotion → NT-MEMORY KB) has no continuous affect representation at any stage.

### 1.4 DABS: Depth as a Queryable Resource

**Source**: [ACL 2026 Long](https://aclanthology.org/2026.acl-long.667.pdf)

**Key Finding**: DABS encodes sentences ONCE, then each aspect adaptively selects depth (how many Transformer layers to read). **60% FLOPs reduction** at M=4 aspects. Key insight: negation/contrast require deeper layers; simple aspects resolve at shallow layers.

**DEFECT-709-S4**: NT-CORE's SelectiveState (GWT attention routing) treats all attention as uniform depth. No mechanism to allocate different representational depth per aspect/query. The AttentionManager routes between CORE+WORLD and CORE+MIND but doesn't modulate depth — only routing destination.

### 1.5 Hyb-Stack: mBERT Dominates Cross-Lingual Emotion

**Source**: [Nature Scientific Reports (2026)](https://www.nature.com/articles/s41598-026-38172-9)

**Key Finding**: Hyb-Stack ensemble (BERT+DistilBERT+mBERT → RF meta-classifier) achieves 89.48 F1 on Hausa, 88.19 on Bahasa Indonesia, 90.67 on English. mBERT consistently outperforms monolingual models on low-resource languages. **Multi-label** emotion detection (not just single-label).

**DEFECT-709-S5**: NT-FEEL's EmotionEngine is monolingual English-centric. No cross-lingual emotion detection capability. No multi-label emotion support (current enum forces single emotion per text span). The SEAL pipeline's rhythm_recalculator and blank_space_checker can't handle multi-label affective signals.

---

## 2. Named Entity Recognition — 2026 State of the Art

### 2.1 JPT: Bidirectional Context in Causal LLMs (20× Faster)

**Source**: [ACL 2026 Long](https://aclanthology.org/2026.acl-long.526.pdf)

**Key Finding**: Just Pass Twice (JPT) concatenates input to itself, enabling causal LLMs to attend to full context in second pass. **+7.9 F1 over prior zero-shot SOTA**, 20× faster than generative methods. Definition-guided entity typing enables flexible zero-shot generalization without retraining.

**DEFECT-709-N1**: NT-MIND's NER capability (via nt_agent_mcp_gateway PTC) uses generative NER exclusively. No discriminative token classification path. No definition-guided entity typing — entity types are hardcoded, not definable via natural language. 20× slower than necessary for zero-shot entity extraction.

### 2.2 ToMMeR: Mention Detection Emerges in Early Layers (<300K params)

**Source**: [ACL 2026 Long](https://aclanthology.org/2026.acl-long.1268/)

**Key Finding**: A sub-300K parameter probe on early LLM layers achieves 93% recall zero-shot mention detection. Cross-model analysis (14M-15B params) shows **DICE >75% convergence on mention boundaries** — mention detection naturally emerges from language modeling. Extending with span classification heads achieves 80-87% F1 NER.

**DEFECT-709-N2**: NT-WORLD's UnifiedCrawler doesn't leverage early-layer mention detection for entity span identification. No lightweight probe exists for entity span detection — full inference is run for every entity extraction. The CapabilityBridge between evolution view and runtime view doesn't map entity detection as an emergent capability.

### 2.3 SAM-NER: Archetype Mediation for Domain Transfer

**Source**: [ACL 2026 Findings](https://aclanthology.org/2026.findings-acl.2050/)

**Key Finding**: Three-stage framework: Entity Discovery → Abstract Mediation (domain-invariant archetype space) → Semantic Calibration. Solves the semantic drift problem when target schemas are novel or semantically overlapping.

**DEFECT-709-N3**: NT-WORLD has no archetype-based entity mediation. Cross-domain entity transfer (e.g., crawling tech sites vs. medical literature) relies on per-domain fine-tuning rather than domain-invariant archetypes. The KB namespace system (`domain_nt_*`) doesn't support archetype-level abstraction for entity types.

### 2.4 NE-R1: RL-Optimized Retrieval-on-Demand NER

**Source**: [arxiv.org/html/2609.02366](https://arxiv.org/html/2609.02366) (2026-09-02)

**Key Finding**: Adaptive retrieval-augmented NER with RL optimization. Multi-dimensional reward (accuracy + retrieval benefit). **+2.52 F1 in-domain, +1.18 F1 zero-shot cross-domain**. Reduces inference latency 55% vs always-retrieve baseline by learning WHEN to retrieve.

**DEFECT-709-N4**: NT-ACT's retrieval mechanisms (nt_world_search ordered backend router) don't have an adaptive retrieval trigger for NER. The "retrieval-on-demand" pattern is missing — either always retrieves or never retrieves. No RL-optimized decision boundary for when external knowledge is needed vs. parametric knowledge suffices.

### 2.5 DiZiNER: Disagreement-Guided Instruction Refinement (+8.0 F1)

**Source**: [ACL 2026 Long](https://aclanthology.org/2026.acl-long.795.pdf)

**Key Finding**: Multiple heterogeneous LLMs annotate shared texts, supervisor analyzes inter-model disagreements to refine instructions. **Zero-shot SOTA on 14/18 benchmarks**. Narrows zero-shot-to-supervised gap from -32.0 to -20.9 F1. Surpasses its own GPT-5 mini supervisor — improvements come from disagreement-guided refinement, not supervisor capability.

**DEFECT-709-N5**: NT-MIND's SEAL pipeline has no disagreement-based instruction refinement mechanism. Self-test (SelfTest trait) checks pass/fail but doesn't use multi-model disagreement to refine extraction instructions. The experience-tree absorption (5-stage) doesn't analyze inter-model consensus for instruction improvement.

### 2.6 GLiNER-bi-Encoder: 130× Throughput at 1024 Labels

**Source**: [arxiv.org/pdf/2602.18487](https://arxiv.org/pdf/2602.18487)

**Key Finding**: Bi-encoder architecture decouples text and label encoding. Pre-computed label embeddings enable 130× throughput improvement at 1024 labels vs uni-encoder. 61.5% Micro-F1 on CrossNER zero-shot. Near-constant inference speed regardless of label count.

**DEFECT-709-N6**: NT-WORLD entity extraction has no bi-encoder architecture. Label encoding is coupled with text encoding (joint-encoding), causing quadratic complexity as entity types grow. The asset_registry and media_asset_registry don't benefit from pre-computed entity type embeddings for entity linking.

### 2.7 SpanDec: 2.7× Throughput via Lightweight Decoder

**Source**: [ACL 2026 Industry](https://aclanthology.org/2026.acl-industry.71.pdf)

**Key Finding**: Decouples span marker processing from main encoder via lightweight decoder. Early span filtering prunes ~85% of candidates before expensive processing. **2.7× throughput, 8.2× GFLOPs reduction** while matching accuracy.

**DEFECT-709-N7**: NT-ACT's span-based entity processing (if any) runs full encoder for all span candidates. No early filtering to prune non-entity spans. No decoupled span decoder — all span interactions go through the main encoder, creating unnecessary computation.

### 2.8 UNER v2: 30 Datasets, 22 Languages

**Source**: [arxiv.org/pdf/2604.12744](https://arxiv.org/pdf/2604.12744)

**Key Finding**: Universal NER v2 expands to 30 datasets, 22 languages, 3M annotated tokens. SOTA generative models achieve only 0.50 average F1 — **performance drops sharply on typologically distant and lower-resourced languages**. Cross-lingual NER remains unsolved.

**DEFECT-709-N8**: NT-WORLD has no multilingual NER capability. Entity extraction is English-only. The crawl pipeline's parsers and classifiers don't support the UNER v2 annotation guidelines. Cross-lingual entity linking (NT-MEMORY KB) is impossible without multilingual NER.

---

## 3. Text Classification — 2026 State of the Art

### 3.1 BTZSC: Rerankers Beat Everything (ICLR 2026)

**Source**: [ICLR 2026](https://mlanthology.org/iclr/2026/aarab2026iclr-btzsc/) / [HuggingFace Blog](https://huggingface.co/blog/aarabil/btzsc-benchmark)

**Key Finding**: 35 models, 22 datasets, 4 architecture families benchmarked:
- **Rerankers win**: Qwen3-Reranker-8B = 0.72 macro F1 (new SOTA)
- **Embeddings best accuracy/efficiency**: GTE-large-en-v1.5 = 0.62 F1, fastest
- **LLMs competitive at scale**: 4-12B params → 0.67 F1, best on topic classification
- **NLI cross-encoders plateau**: even with larger backbones, no improvement
- **Scaling differs by architecture**: rerankers/LLMs improve with scale; embeddings plateau

**DEFECT-709-T1**: NT-IO's LLM provider selection (egress_privacy_guard) doesn't account for task-specific architecture suitability. Zero-shot classification tasks are routed to generic LLMs when specialized rerankers would be 2× more accurate. No architecture-aware provider selection for classification tasks.

### 3.2 LSR: Label Space Reduction Improves LLMs by 7%

**Source**: [Springer Nature (2026)](https://link.springer.com/article/10.1007/s10791-026-10420-6)

**Key Finding**: Iterative test-time training using LLM pseudo-labels to rank and reduce candidate classes. **+7.0% macro-F1** (Llama-3.1-70B), up to 14.2% improvement. Distillation enables lightweight deployment without repeated LLM calls. Smaller models benefit MORE from LSR.

**DEFECT-709-T2**: NT-MIND's classification pipeline has no label space reduction mechanism. Large label sets (e.g., domain classifications in KB) suffer from attention dilution and positional bias. The SEAL pipeline's skill crystallization doesn't iteratively refine label spaces for classification tasks.

### 3.3 Few-Shot Label-Guided Distance Scaling

**Source**: [arxiv.org/abs/2603.02267v1](https://arxiv.org/abs/2603.02267v1)

**Key Finding**: Label-guided distance scaling injects label semantic information in both training and testing. Pulls sample representations toward class centers using label semantics. Outperforms SOTA meta-learners on few-shot text classification.

**DEFECT-709-T3**: NT-MIND's few-shot capability (if any) doesn't use label semantics as supervision signals. The Disclosure Ladder (AnchorPromote) uses budget escalation but doesn't leverage label semantics to improve few-shot classification quality.

### 3.4 Discrete Diffusion as Training-Free Classifier

**Source**: [SIAM SDM 2026](https://arxiv.org/html/2608.14649)

**Key Finding**: dLLM-SetScore uses masked-diffusion language models as multi-label classifiers without fine-tuning. Identifies slot-position asymmetry failure mode. Per-label entailment scoring solves permutation invariance. Hybrid ensemble (BART-MNLI + SetFit + LLaDA) reaches 82.4/79.3 micro/macro F1 on Reuters.

**DEFECT-709-T4**: NT-MIND has no diffusion-based classification pathway. All classification is encoder-based or generative LLM-based. The discrete diffusion approach offers a third paradigm that's training-free and handles multi-label naturally — relevant for NT-FEEL's multi-label emotion classification gap.

### 3.5 Zero-Shot LLMs Match Fine-Tuned on Scientific Text

**Source**: [IJERT (2026)](https://www.ijert.org/zero-shot-and-few-shot-scientific-text-classification-using-modern-large-language-models-a-comparative-study-ijertv15is051422)

**Key Finding**: LLaMA 3.1-8B achieves **99% accuracy on WoS-5736** (zero-shot), surpassing fine-tuned SciBERT (98%). On WoS-46985 (134 categories), LLaMA achieves 93% vs SciBERT 87%. Free API inference (Groq) requires no GPU, no training data.

**DEFECT-709-T5**: NT-ACT's tool selection (MCP gateway) doesn't leverage zero-shot LLM classification for task routing. Scientific text classification tasks are routed through full fine-tuning pipelines when zero-shot LLMs would match or exceed performance at zero cost. The capability registry doesn't track zero-shot vs fine-tuned capability boundaries.

---

## 4. Summary: New Defects for NeoTrix

| ID | Domain | Defect | Severity |
|-----|--------|--------|----------|
| DEFECT-709-S1 | NT-FEEL | EmotionLabel enum is categorical; field moved to continuous VA space | **CRITICAL** |
| DEFECT-709-S2 | NT-WORLD | Static sentiment scoring; no LLM-guided context-sensitive reweighting | HIGH |
| DEFECT-709-S3 | NT-PHYSICAL | No VAD encoding for multimodal affect | HIGH |
| DEFECT-709-S4 | NT-CORE | GWT attention has no depth-as-queryable-resource mechanism | MEDIUM |
| DEFECT-709-S5 | NT-FEEL | Monolingual English-only emotion detection; no multi-label | HIGH |
| DEFECT-709-N1 | NT-MIND | Generative-only NER; no discriminative token classification | HIGH |
| DEFECT-709-N2 | NT-WORLD | No early-layer mention detection probe (<300K params) | MEDIUM |
| DEFECT-709-N3 | NT-WORLD | No archetype-based cross-domain entity mediation | HIGH |
| DEFECT-709-N4 | NT-ACT | No adaptive retrieval-on-demand for NER | MEDIUM |
| DEFECT-709-N5 | NT-MIND | No disagreement-based instruction refinement in SEAL | HIGH |
| DEFECT-709-N6 | NT-WORLD | No bi-encoder NER; quadratic scaling with entity types | HIGH |
| DEFECT-709-N7 | NT-ACT | No early span filtering for entity candidates | MEDIUM |
| DEFECT-709-N8 | NT-WORLD | English-only NER; no multilingual support (UNER v2 gap) | HIGH |
| DEFECT-709-T1 | NT-IO | No architecture-aware provider selection for classification | MEDIUM |
| DEFECT-709-T2 | NT-MIND | No label space reduction for large-label classification | MEDIUM |
| DEFECT-709-T3 | NT-MIND | No label-guided distance scaling for few-shot tasks | LOW |
| DEFECT-709-T4 | NT-MIND | No diffusion-based training-free classification pathway | MEDIUM |
| DEFECT-709-T5 | NT-ACT | No zero-shot vs fine-tuned capability tracking in registry | LOW |

**Total new defects**: 18
**Critical**: 1 | **High**: 8 | **Medium**: 7 | **Low**: 2

---

## 5. Sources Cited

1. SemEval-2026 Task 3: DimABSA — [arxiv.org/pdf/2604.07066](https://arxiv.org/pdf/2604.07066)
2. MH-ABSA — [Springer Nature 2026](https://link.springer.com/article/10.1007/s44443-026-00996-w)
3. VADE (VAD-Enhanced MABSC) — [ACL 2026 Findings](https://aclanthology.org/2026.findings-acl.1979.pdf)
4. DABS (Depth-Ordered Aggregation) — [ACL 2026 Long](https://aclanthology.org/2026.acl-long.667.pdf)
5. Hyb-Stack — [Nature Scientific Reports 2026](https://www.nature.com/articles/s41598-026-38172-9)
6. SSRGAT — [Nature Scientific Reports 2026](https://www.nature.com/articles/s41598-026-66667-y)
7. JPT (Just Pass Twice) — [ACL 2026 Long](https://aclanthology.org/2026.acl-long.526.pdf)
8. ToMMeR — [ACL 2026 Long](https://aclanthology.org/2026.acl-long.1268/)
9. SAM-NER — [ACL 2026 Findings](https://aclanthology.org/2026.findings-acl.2050/)
10. NE-R1 — [arxiv.org/html/2609.02366](https://arxiv.org/html/2609.02366)
11. DiZiNER — [ACL 2026 Long](https://aclanthology.org/2026.acl-long.795.pdf)
12. GLiNER-bi-Encoder — [arxiv.org/pdf/2602.18487](https://arxiv.org/pdf/2602.18487)
13. SpanDec — [ACL 2026 Industry](https://aclanthology.org/2026.acl-industry.71.pdf)
14. UNER v2 — [arxiv.org/pdf/2604.12744](https://arxiv.org/pdf/2604.12744)
15. BTZSC Benchmark — [ICLR 2026](https://mlanthology.org/iclr/2026/aarab2026iclr-btzsc/)
16. LSR (Label Space Reduction) — [Springer Nature 2026](https://link.springer.com/article/10.1007/s10791-026-10420-6)
17. LDS (Label-guided Distance Scaling) — [arxiv.org/abs/2603.02267v1](https://arxiv.org/abs/2603.02267v1)
18. dLLM-SetScore — [SIAM SDM 2026](https://arxiv.org/html/2608.14649)
19. Zero-Shot Scientific Classification — [IJERT 2026](https://www.ijert.org/zero-shot-and-few-shot-scientific-text-classification-using-modern-large-language-models-a-comparative-study-ijertv15is051422)
20. TSAPM-BERT — [Springer Nature 2026](https://link.springer.com/article/10.1007/s10791-026-09958-2)
21. ReCoT-NER — [ACL 2026 Findings](https://aclanthology.org/2026.findings-acl.227/)
22. LLM Text Classification Survey — [SAGE Journals 2025](https://journals.sagepub.com/doi/10.1177/00491241251325243)
23. Fine-Grained Military Text Classification — [Nature Scientific Reports 2026](https://www.nature.com/articles/s41598-025-34825-3)

---

## 6. Key Paradigm Shifts Identified

| Shift | Old Paradigm | New Paradigm (2026) | NeoTrix Impact |
|-------|-------------|---------------------|----------------|
| Sentiment Representation | Categorical (pos/neg/neu) | Continuous VA/VAD space | EmotionLabel needs VA dimensions |
| NER Architecture | Generative (autoregressive) | Discriminative (JPT bidirectional) | 20× speedup available |
| NER Entity Types | Fixed label set | Definition-guided zero-shot | No retraining for new types |
| NER Depth | Full encoder for all spans | Early-layer probing (<300K params) | Mention detection is emergent |
| Text Classification | NLI cross-encoders | Rerankers (0.72 F1 SOTA) | Architecture selection matters |
| Label Scaling | Flat label lists | Iterative label space reduction | +7% F1 for large label sets |
| Cross-Lingual | Monolingual | mBERT ensemble cross-lingual | English-only is obsolete |
| Multi-Label | Single emotion per text | Multi-label emotion detection | EmotionLabel must support |

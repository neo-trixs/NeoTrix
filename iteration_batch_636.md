# Iteration 636 — NLP Semantic Understanding: WSD / Coreference / SRL

## 1. Word Sense Disambiguation (WSD)

### Finding 1.1: WSDPO — CoT + Preference Optimization for Generative WSD
**Source**: Kang et al., ACL 2026, `aclanthology.org/2026.acl-long.1610/`
**What's NEW**: Generative WSD framework that combines Chain-of-Thought (CoT) disambiguation steps with preference optimization. Three stages: (1) disambiguation-aware CoT construction, (2) disambiguation-guided SFT, (3) preference-based optimization using multiple sampled CoT outputs as preference pairs.
**Defect identified**: Supervised fine-tuning for generative WSD prioritizes "context-to-gloss memorization" over true sense discrimination. WSDPO fixes this for rare/unseen senses but requires constructing preference pairs — **no automated quality gate for CoT chain faithfulness**. NeoTrix gap: SelfTest for WSD-style tasks lacks a "CoT sense-faithfulness" verification stage.

### Finding 1.2: Low-Parameter LLMs Match GPT-4-Turbo on WSD
**Source**: Sumanathilaka et al., LREC 2026, `lrec-conf.org/proceedings/lrec2026/pdf/2026.lrec2026-1.816.pdf`
**What's NEW**: Gemma-3-4B and Qwen-3-4B fine-tuned with CoT reasoning + neighbor-word analysis achieve performance comparable to GPT-4-Turbo zero-shot. Advanced reasoning approach uses only **10% of training data** yet achieves comparable results to full CoT training.
**Defect identified**: **Data efficiency paradox** — the advanced reasoning strategy (correct vs. incorrect sense analysis) uses 10% data but still doesn't surpass zero-shot CoT in overall accuracy. This implies the CoT reasoning paradigm may be capturing distributional shortcuts rather than genuine sense discrimination. NeoTrix gap: NT-MIND distillation pipeline lacks a "reasoning quality vs. data volume" calibration metric.

### Finding 1.3: WSD Is Not Dead — Diagnostic Lens Redefinition
**Source**: Navigli, AAAI 2026, `doi.org/10.1609/aaai.v40i46.41331`
**What's NEW**: Navigli argues WSD is redefined as a **diagnostic lens** for assessing lexical-semantic competence in LLMs. Closed-source instruction-tuned LLMs achieve performance comparable to specialized WSD systems, yet systematic weaknesses remain: non-predominant senses are often misclassified and disambiguation biases in MT persist. Encoder-based architectures should not be considered obsolete — contrastive/sense-aware encoders may play a role as complementary components in hybrid systems.
**Defect identified**: LLMs "mask persistent weaknesses in deeper lexical understanding" — apparent semantic fluency hides systematic failures in domain and long-tail senses. NeoTrix gap: No Egress Privacy Guard equivalent for **semantic faithfulness** — outgoing LLM responses may carry fluent but semantically biased content. NT-SHIELD should extend to "semantic integrity guard."

### Finding 1.4: SemEval-2026 Task 5 — Graded Plausibility for Ambiguous Stories
**Source**: Gehring et al., SemEval-2026, `aclanthology.org/2026.semeval-1.448.pdf`
**What's NEW**: WSD reframed as **graded plausibility estimation** (1-5 Likert scale) rather than single-label classification. 4-5 sentence stories where context is deliberately sparse. Generative LLMs + ensembles dominate leaderboard (93.3% accuracy-within-SD). Traditional ML systems (UWB-NLP, Paradise) show accuracy increases on ambiguous samples but Spearman correlation drops — they **optimize for metric gaming** (narrow prediction range).
**Defect identified**: Top systems exploit metric optimization by predicting narrow value ranges, avoiding extreme predictions to increase "accuracy within SD" while sacrificing genuine discrimination. NeoTrix gap: SEAL pipeline quality gates don't detect **metric gaming behavior** — a system can look high-performing on aggregate scores while producing degenerate outputs.

### Finding 1.5: GlossAdapter — LoRA for Parameter-Efficient WSD
**Source**: Manikandan et al., PMLR 318 (Canadian AI 2026), `proceedings.mlr.press/v318/manikandan26a.html`
**What's NEW**: LoRA adapters with POS filtering for WSD. Only 0.5% of RoBERTa-large parameters are trainable. POS filtering aligns target word context with WordNet lexical categories to construct sentence-gloss pairs.
**Defect identified**: POS filtering assumes WordNet lexical categories are universally applicable — **cross-lingual POS alignment is not addressed**. For NeoTrix multilingual KB, this means WSD quality degrades on non-English inputs without language-specific POS mapping.

### Finding 1.6: WiC-WSD Transfer Learning
**Source**: LREC 2026, `lrec.elra.info/lrec2026-main-785`
**What's NEW**: Joint training on Word-in-Context (WiC) and WSD tasks using contrastive loss on xlm-roberta-large. WSD-trained models generalize effectively to WiC and vice versa. Combining multiple WiC datasets enhances accuracy and stability.
**Defect identified**: Joint training improves low-resource WSD but **WSD benefits mainly when annotated data is limited** — with sufficient data, joint training shows diminishing returns. This suggests the tasks share surface-level features but diverge at deeper semantic levels. NeoTrix gap: KB embedding strategy conflates WiC-style contextual similarity with WSD-style sense identity.

---

## 2. Coreference Resolution

### Finding 2.1: CRAC 2026 — Fifth Shared Task, Long-Range Entities
**Source**: Novák et al., CODI-CRAC 2026, `aclanthology.org/2026.codi-1.21.pdf`
**What's NEW**: CorefUD 1.4 expanded to 27 datasets in 19 languages. Focus on **long-range entities** — coreferential chains spanning significant distances. LLM track introduced alongside unconstrained track. Four LLM-based approaches participated (3 fine-tuned, 1 few-shot). Traditional systems still lead but LLMs demonstrate significant potential.
**Defect identified**: CorefLat (Latin) dataset had a **conversion pipeline flaw** — cluster-based representation not handled by converter. After fix, baseline jumped from 6.8 to 37 CoNLL F1. NeoTrix gap: KB ingestion pipeline lacks automated validation for heterogeneous annotation formats. Converting external knowledge should include schema consistency checks.

### Finding 2.2: DAggerCoref — Closing Train/Test Distribution Mismatch
**Source**: Morton & Warstadt, CODI-CRAC 2026, `aclanthology.org/2026.codi-1.28/`
**What's NEW**: DAgger (Dataset Aggregation) applied to coreference resolution for the first time. Three-stage cascade: gap classifier (zero pronoun detection), mention head classifier, coarse-to-fine antecedent scorer. DAgger fine-tunes on 50/50 mix of gold and pipeline-predicted mentions, closing the distribution mismatch. Otsu adaptive thresholding for zero pronoun detection matches gold-tuned thresholds without gold supervision. **67.56 CoNLL F1** across 27 datasets/19 languages.
**Defect identified**: DAgger requires multiple training iterations with policy-generated data — **computational cost scales linearly with iteration count**. For NeoTrix's self-evolution pipeline, this means SEAL cycles would need to budget for iterative DAgger-style refinement rather than one-shot training. No cost-aware scheduling exists.

### Finding 2.3: ImCoref-CeS — LLM Checker-Splitter Agent
**Source**: Luo et al., ACL 2026, `aclanthology.org/2026.acl-long.1122/`
**What's NEW**: Hybrid framework: enhanced supervised model (ImCoref) + LLM-based Checker-Splitter agent. ImCoref introduces lightweight bridging module for long-text encoding, biaffine scorer for positional information, hybrid mention regularization. LLM agent validates candidate mentions (filtering invalid ones) and coreference results (splitting erroneous clusters). **SOTA performance** across benchmarks.
**Defect identified**: The LLM Checker-Splitter operates as a post-hoc validator — errors in the supervised model that the LLM doesn't catch propagate silently. **No feedback loop from LLM corrections back to supervised model training**. NeoTrix gap: NT-REPAIR self-healing lacks "correction feedback integration" — repairs should improve the base model, not just mask errors.

### Finding 2.4: CorPipe 26 — Empty Nodes + Cross-Lingual Transfer
**Source**: Straka, CODI-CRAC 2026, `aclanthology.org/2026.codi-1.27/`
**What's NEW**: Winning submission. Predicts empty nodes (zero anaphora) together with mentions and coreference links in a single model. Outperforms all LLM track submissions by 2.8 points and unconstrained track by 9.5 points. Empty node prediction is critical for pro-drop languages.
**Defect identified**: CorefUD datasets with zero mentions are **heavily biased toward European languages** — Czech, Latin, but no East Asian pro-drop languages (Japanese, Korean, Mandarin) despite CorefUD 1.4 claiming 19 languages. NeoTrix KB's multilingual coverage has a similar blind spot.

### Finding 2.5: Two-Stage Adaptation — Gemma-3-27B for LLM Track
**Source**: Bourgois et al., CODI-CRAC 2026, `aclanthology.org/2026.codi-1.23.pdf`
**What's NEW**: First place in LLM track (74.32 CoNLL F1). Two-stage fine-tuning: multilingual base adapter → dataset-specific adapters. XML-inspired headword-only mention representation with local reindexing. Headword representation achieves +2.96 improvement over full span representation. Iterative annotation strategy documents are processed sequentially.
**Defect identified**: **Inter-dataset annotation heterogeneity** — mention density ranges from 8 to 38 per 100 tokens across datasets. French litbank (13.6/100) vs. democrat (27.9/100) follow conflicting guidelines despite similar text types. Models cannot implicitly adapt to conflicting conventions. NeoTrix gap: No cross-domain consistency checker for KB ingestion — conflicting schemas from different sources create silent quality degradation.

### Finding 2.6: QLoRA + Bounded Entity Registry
**Source**: Shore et al., CODI-CRAC 2026, `aclanthology.org/2026.codi-1.25/`
**What's NEW**: Qwen 3 14B fine-tuned with QLoRA. Documents processed in 500-700 character chunks with bounded rolling context (500 chars recent + scored entity registry tracking 30 active entities via frequency×recency decay). Probing experiments reveal **coreference signal concentrated in attention value projections rather than MLP modules**, strongest at earliest transformer layer.
**Defect identified**: Bounded entity registry of 30 entities is a hard cap — **documents with >30 active entities force eviction** via the decay formula, potentially losing critical long-distance references. For NeoTrix's long-document KB ingestion, this entity cap would silently drop important cross-references. The frequency×recency bias also penalizes entities mentioned early but resurfacing later.

### Finding 2.7: Plural Reference Circuit in LLMs
**Source**: arXiv:2609.03687, Sep 2026
**What's NEW**: Mechanistic interpretability study identifies specific attention heads responsible for: (1) representing coreference info, (2) identifying plural entities, (3) transferring info for antecedent selection. LLMs align with humans in preferring plural pronouns for ontologically similar entities linked by "and."
**Defect identified**: The study only examines **singular/plural distinction** — no investigation of neopronouns, honorifics, or non-binary reference patterns. For NeoTrix's NT-FEEL emotional expression system, gendered/neutral pronoun resolution is a critical gap in the circuit analysis.

---

## 3. Semantic Role Labeling (SRL)

### Finding 3.1: Modernized Encoder-Based SRL Framework
**Source**: arXiv:2605.02505, May 2026
**What's NEW**: Eliminates AllenNLP dependency (maintenance mode since Dec 2022). Caches sentence-level representations and reuses across predicates → **10× faster inference**. BERT-base achieves 86.2% (vs. AllenNLP 86.72%), RoBERTa 87.07%, DeBERTa 87.49%. Dependency-informed diagnostic methodology for span boundary inconsistencies.
**Defect identified**: AllenNLP's obsolescence created a **2.5-year gap** in SRL infrastructure compatibility. The new framework is faster but still encoder-only — no integration with decoder-based LLMs for zero-shot SRL. NeoTrix gap: NT-WORLD's semantic parsing pipeline may be silently depending on deprecated AllenNLP components.

### Finding 3.2: Faithful Explanation of SRL with Syntactic Features
**Source**: ScienceDirect, Volume 676, May 2026
**What's NEW**: First XAI method to quantify impact of dependency/constituency features on SRL predictions. Novel measures: "relation effect" (impact on span inclusion) and "relation utility" (impact on prediction correctness). Model merging technique creates syntax-aware SRL without syntax-annotated training data or runtime parsing. Model-intrinsic evaluation bypasses human judgment data.
**Defect identified**: Syntactic features improve **span-boundary stability** but not role classification accuracy. This means syntax helps locate argument boundaries but doesn't help assign correct semantic roles. NeoTrix gap: Knowledge representation conflates structural parsing (where) with semantic understanding (what role) — need separate quality metrics for each.

### Finding 3.3: SRL Survey in Pretrained LM Era
**Source**: Zhang, alphaXiv 2502.08660, Apr 2026
**What's NEW**: Unified four-dimensional taxonomy: (1) model architectures, (2) syntax feature modeling, (3) application scenarios, (4) multimodal extensions. First systematic treatment of SRL with LLMs. Identifies complementary roles of LLMs and specialized SRL systems. Extends to visual, video, and speech modalities.
**Defect identified**: **Multimodal SRL evaluation is structurally inconsistent** across modalities — no unified benchmark for visual/video/speech SRL. NeoTrix's multimodal perception pipeline (NT-WORLD + NT-PHYSICAL) would benefit from cross-modal SRL alignment but lacks evaluation infrastructure.

### Finding 3.4: LLMs Surpass Encoder-Decoder in SRL
**Source**: arXiv:2506.05385, Jun 2026
**What's NEW**: First LLM-based method to surpass encoder-decoder approaches in complete SRL tasks. Two mechanisms: (a) retrieval-augmented generation using predicate-argument description database, (b) self-correction for iterative refinement. Tested on CPB1.0, CoNLL-2009, CoNLL-2012. Self-correction becomes less critical with increased training steps.
**Defect identified**: Self-correction mechanism has **diminishing returns with training** — after sufficient fine-tuning, it adds latency without quality improvement. The framework also requires a curated predicate-argument description database — **manual knowledge engineering bottleneck**. NeoTrix gap: NT-MIND skill crystallization could automate this knowledge base construction but currently doesn't target SRL domain knowledge.

### Finding 3.5: HTMR — Hybrid Token Masking RLVR for Event Argument Extraction
**Source**: ACL 2026, `aclanthology.org/2026.acl-long.910/`
**What's NEW**: Reinforcement Learning with Verifiable Rewards (RLVR) applied to Event Argument Extraction. Addresses misalignment where single reward is applied to all tokens but optimization is token-level. HTMR selectively updates gradients on high-entropy forking tokens and event-critical tokens. Transfers to NER and relation classification.
**Defect identified**: RLVR supervision is **coarse-grained** — the same reward applies to all tokens, introducing gradient noise for non-event tokens. Even with selective masking, the reward signal remains binary (correct/incorrect event structure) rather than graded. NeoTrix gap: SEAL pipeline's fitness function is similarly coarse — no token-level or span-level reward shaping for semantic tasks.

### Finding 3.6: Emergence of Semantic Role Circuits in LLMs
**Source**: ACL 2026 Findings, `aclanthology.org/2026.findings-acl.1964.pdf`
**What's NEW**: COMPASS methodology combines edge-attribution patching with training-time circuit tracking. Highly localised circuits for all tested roles — top 20 nodes capture 89-92% of attribution mass. Circuit architectures vary by role: BENEFICIARY most complex, TIME shifts from early to late processing. Roles develop distinct, temporally-trackable circuits. Cross-model conservation partial (Pythia vs. Llama-1B).
**Defect identified**: Circuit analysis uses only **Pythia (14M-1B) and Llama-1B** — no analysis of models above 1B parameters. Scaling behavior of semantic role circuits is unknown. If circuits reorganize at scale (as other behaviors do), findings may not transfer to production LLMs. NeoTrix gap: NT-CORE's HyperCube knowledge representation assumes stable semantic structure — if circuits reorganize during training, the representation may need dynamic rebinding.

### Finding 3.7: Multilingual QA-SRL via Cross-Lingual Projection
**Source**: EACL 2026, `aclanthology.org/2026.eacl-long.112.pdf`
**What's NEW**: QA-SRL (question-answer driven SRL) extended to Hebrew, Russian, French via cross-lingual projection from English. Uses constrained MT + word alignment + QA-structure preservation. Lightweight parsers fine-tuned on projected data substantially outperform GPT-4o. Natural-language format eliminates need for predefined role sets.
**Defect identified**: Projection quality depends on **MT quality for target language** — low-resource languages with poor MT will produce degraded QA-SRL annotations. The constrained translation approach assumes English question templates are universally applicable across languages. NeoTrix gap: KB multilingual ingestion uses generic translation without domain-specific QA-SRL constraint preservation.

---

## Cross-Domain Defects for NeoTrix

| # | Domain | Defect | Severity | NeoTrix Module |
|---|--------|--------|----------|----------------|
| D1 | WSD | No semantic faithfulness guard for outgoing LLM responses | HIGH | NT-SHIELD |
| D2 | WSD | Metric gaming in graded plausibility — narrow prediction ranges | MEDIUM | SEAL quality gates |
| D3 | WSD | Cross-lingual POS alignment gap in GlossAdapter | MEDIUM | NT-MEMORY KB |
| D4 | Coref | Annotation format conversion lacks schema validation | HIGH | NT-WORLD ingestion |
| D5 | Coref | Entity registry hard cap (30) evicts long-distance references | HIGH | NT-MEMORY |
| D6 | Coref | Inter-dataset annotation heterogeneity undetectable by models | MEDIUM | NT-META cross-module audit |
| D7 | Coref | Neopronoun/non-binary reference circuit gap | LOW | NT-FEEL |
| D8 | SRL | AllenNLP deprecation creates silent dependency risk | HIGH | NT-WORLD |
| D9 | SRL | Structural parsing conflated with semantic role understanding | MEDIUM | NT-MEMORY embeddings |
| D10 | SRL | Multimodal SRL evaluation inconsistent across modalities | MEDIUM | NT-PHYSICAL |
| D11 | SRL | Semantic role circuits unknown at scale (>1B params) | LOW | NT-CORE HyperCube |
| D12 | Coref | DAgger-style iterative refinement lacks cost-aware scheduling | MEDIUM | SEAL pipeline |
| D13 | WSD | WiC-WSD surface feature sharing masks deeper divergence | MEDIUM | NT-MEMORY embeddings |
| D14 | SRL | RLVR coarse reward signal lacks token-level shaping | MEDIUM | SEAL fitness function |

## Sources Cited

1. Kang et al. (2026) WSDPO. ACL 2026. https://aclanthology.org/2026.acl-long.1610/
2. Sumanathilaka et al. (2026) Low-Param WSD. LREC 2026. http://www.lrec-conf.org/proceedings/lrec2026/pdf/2026.lrec2026-1.816.pdf
3. Navigli (2026) Is WSD Dead? AAAI 2026. https://doi.org/10.1609/aaai.v40i46.41331
4. Gehring et al. (2026) SemEval Task 5. https://aclanthology.org/2026.semeval-1.448.pdf
5. Manikandan et al. (2026) GlossAdapter. PMLR 318. https://proceedings.mlr.press/v318/manikandan26a.html
6. LREC 2026 WiC-WSD Transfer. https://lrec.elra.info/lrec2026-main-785
7. Novák et al. (2026) CRAC 2026 Findings. https://aclanthology.org/2026.codi-1.21.pdf
8. Morton & Warstadt (2026) DAggerCoref. https://aclanthology.org/2026.codi-1.28/
9. Luo et al. (2026) ImCoref-CeS. https://aclanthology.org/2026.acl-long.1122/
10. Straka (2026) CorPipe 26. https://aclanthology.org/2026.codi-1.27/
11. Bourgois et al. (2026) Two-Stage LLM Coref. https://aclanthology.org/2026.codi-1.23.pdf
12. Shore et al. (2026) PortNLP QLoRA Coref. https://aclanthology.org/2026.codi-1.25/
13. arXiv:2609.03687 (2026) Plural Reference Circuit.
14. arXiv:2605.02505 (2026) Modernized SRL Framework.
15. ScienceDirect (2026) Faithful SRL Explanation. Vol 676.
16. Zhang (2026) SRL Survey. alphaXiv 2502.08660.
17. arXiv:2506.05385 (2026) LLMs SRL via RAG+Self-Correction.
18. ACL 2026 HTMR. https://aclanthology.org/2026.acl-long.910/
19. ACL 2026 Findings Semantic Role Circuits. https://aclanthology.org/2026.findings-acl.1964.pdf
20. EACL 2026 Multilingual QA-SRL. https://aclanthology.org/2026.eacl-long.112.pdf

## Summary

**20 sources analyzed** across 3 NLP subfields. **14 defects identified** (3 HIGH, 8 MEDIUM, 3 LOW). Key insights:

1. **WSD**: Not dead but redefined as diagnostic lens. LLMs mask persistent weaknesses in non-predominant senses. Metric gaming in graded plausibility tasks.
2. **Coreference**: LLMs closing gap with specialized systems (74.32 vs 77.11 CoNLL F1). Long-range entity tracking and annotation heterogeneity remain unsolved. Entity registry caps cause silent reference loss.
3. **SRL**: LLMs first time surpass encoder-decoder systems. Semantic role circuits exist but scale behavior unknown. Syntax helps span boundaries but not role classification. Multimodal SRL evaluation fragmented.

**Highest priority NeoTrix actions**: (1) Extend NT-SHIELD to semantic integrity guarding, (2) Add schema validation to KB ingestion pipeline, (3) Remove entity registry hard cap or implement dynamic eviction, (4) Add token-level reward shaping to SEAL fitness function.

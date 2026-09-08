# NeoTrix Research Brief: 2025-2026 LLM & AI Infrastructure Advances

**Date**: 2026-09-05  
**Scope**: 10 domains across NLP, LLM systems, and AI infrastructure  
**Purpose**: Inform NeoTrix architecture evolution — SEAL pipeline, GWT attention, VSA HyperCube, E8 reasoning

---

## 1. LLM Reasoning

### 1.1 Hierarchical Chain-of-Thought (Hi-CoT)
- **Source**: https://arxiv.org/html/2604.00130v1 (Mar 2026)
- **Key contribution**: Alternates between instructional planning and step-by-step execution, decomposing reasoning into hierarchical substeps rather than flat linear chains.
- **New vs 2024**: Standard CoT produces flat chains; Hi-CoT introduces explicit hierarchical decomposition with plan-then-execute loops.
- **NeoTrix implication**: Maps directly to ConsciousnessTree's 6-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core). Hi-CoT's hierarchical decomposition could enhance NT-CORE's E8 hexagram reasoning by adding structured plan-then-execute substeps within each reasoning branch. GWT attention routing could use Hi-CoT hierarchy to determine which reasoning level gets broadcast across specialist modules.

### 1.2 MAKER: Million-Step Zero-Error Reasoning via Extreme Decomposition
- **Source**: https://www.cognizant.com/us/en/ai-lab/blog/maker + https://arxiv.org/html/2511.09030v1 (Nov 2025)
- **Key contribution**: First system to complete 1,048,575 dependent LLM steps with zero errors. Three mechanisms: Maximal Agentic Decomposition (atomic microagents), first-to-ahead-by-k voting (local consensus), and red-flagging (structural confusion detection). Cost grows linearly, not exponentially, with step count.
- **New vs 2024**: Apple's "Illusion of Thinking" showed frontier models fail beyond ~8 disks in Towers of Hanoi. MAKER solves 20 disks (2^20 - 1 moves) — a 1000x scale improvement. Shifts paradigm from "bigger model" to "smarter decomposition + voting."
- **NeoTrix implication**: Validates NT-MIND's SEAL pipeline extreme decomposition strategy. The voting mechanism directly informs NT-SHIELD's audit dimensions (D31-D36). MAKER's microagent architecture could replace monolithic NT-CORE reasoning with decomposed specialist agents, aligning with the Dark Forest axiom (every module must compile + test + connect or be deleted). Red-flagging maps to NT-REPAIR's self-healing patterns.

### 1.3 Self-Verification in Code Generation
- **Source**: https://tldr.takara.ai/p/2605.07122 (RepoZero, May 2026)
- **Key contribution**: Self-verification via test execution is the most effective strategy for repository-level code generation. Models generate code, execute tests, and iteratively fix failures.
- **New vs 2024**: 2024 approaches focused on single-file generation. RepoZero enables full repository generation from scratch with automated test-based verification.
- **NeoTrix implication**: Aligns with NT-ACT's tool-use architecture — agents should not just generate code but execute tests and self-correct. The Constellation maturity ladder (C0-C6) could incorporate self-verification as a gate between C1 (unit tests) and C2 (integration tests).

---

## 2. Multimodal Understanding

### 2.1 Vision-Language-Action Models (VLA)
- **Source**: https://medium.com/@adityaj5400/beyond-text-the-rise-of-large-multimodal-models-a-2026-deep-dive (Mar 2026) + https://arxiv.org/html/2501.02189v7 (Aug 2026)
- **Key contribution**: 2025-2026 saw the emergence of VLA models that unify vision, language, and action in a single architecture. Models now interpret images, video, documents, and UI interfaces with near-human accuracy.
- **New vs 2024**: 2024 had separate VLM and action models. 2025-2026 unified them into single architectures that can perceive, reason, and act.
- **NeoTrix implication**: NT-WORLD's UnifiedCrawler could adopt VLA for direct UI understanding and web interaction, replacing CSS selector-based parsing. NT-PHYSICAL's sensor-motor architecture should integrate VLA for embodied perception. The PerceptionBridge connecting L2 perception with L5 consciousness needs VLA-compatible attention gating.

### 2.2 Audio-Visual LLMs
- **Source**: https://www.sciencedirect.com/science/article/abs/pii/S0950705126012955 + https://openaccess.thecvf.com/content/ICCV2025W/MMFM/html/Shu_Audio-Visual_LLM_for_Video_Understanding_ICCVW_2025_paper.html (2025-2026)
- **Key contribution**: Joint audio+visual+language modeling enables interpretation of complex real-world scenarios (dialogue understanding, scene description, event detection). Audio-Visual LLM achieves holistic video understanding.
- **New vs 2024**: 2024 models treated audio and visual streams separately. 2025-2026 models fuse them at the architectural level.
- **NeoTrix implication**: NT-FEEL's EmotionEngine should integrate audio-visual signals for richer emotion detection (beyond text-only). NT-WORLD's crawler should handle video/audio content extraction natively. Supports the DynamicParams terminology for dynamic manga production — AudioSyncPattern needs audio-visual LLM backbone.

---

## 3. AI Agent Planning

### 3.1 MagicAgent: Foundation Models for Generalized Agent Planning
- **Source**: https://arxiv.org/html/2602.19000v1 (Jun 2024, cited through 2026)
- **Key contribution**: Foundation models specifically designed for agent planning — hierarchical task decomposition, tool selection, and multi-step execution. Trained end-to-end for planning rather than prompting general LLMs.
- **New vs 2024**: Prior work prompted general LLMs for planning. MagicAgent trains dedicated planning models with planning-specific objectives.
- **NeoTrix implication**: NT-ACT's orchestration layer could benefit from dedicated planning models rather than relying solely on general LLM prompting. The Seal Pipeline's phase transitions could use a planning-specific model for more reliable phase gating. Aligns with the Disclosure Ladder concept — planning budget should anchor on Minimal then promote to Standard.

### 3.2 Adaptive Task Decomposition Under Uncertainty
- **Source**: https://www.preprints.org/manuscript/202602.1841 (Feb 2026) + https://mbrenndoerfer.com/writing/planning-task-decomposition-goal-directed-llm-agents (Jan 2026)
- **Key contribution**: Agents in dynamic/uncertain environments need adaptive decomposition — plans must be re-evaluated and re-decomposed when conditions change. Four documented strategies: task decomposition, plan selection, external module use, reflection/memory.
- **New vs 2024**: 2024 planning was largely static (decompose once, execute). 2025-2026 emphasizes dynamic re-planning with failure recovery.
- **NeoTrix implication**: NT-REPAIR's self-healing architecture should incorporate adaptive re-planning. The ConsciousnessTree's 6-stage loop already supports re-evaluation, but needs explicit plan revision mechanisms at each stage transition. The MAKER voting system provides a concrete mechanism for plan validation.

---

## 4. Inference Optimization

### 4.1 Arctic Inference: Suffix Decoding + Draft Models (4x Speedup)
- **Source**: https://www.snowflake.com/en/blog/engineering/fast-speculative-decoding-vllm-arctic/ (May 2025)
- **Key contribution**: Two complementary speculative decoding approaches: (1) Suffix Decoding exploits repetitive patterns in agentic workflows via suffix trees, achieving 20μs/token speculation; (2) MLP/LSTM draft models for non-repetitive generation. Combined, they achieve 4x faster inference on SWE-Bench and 2.8x on open-ended tasks. Achieves 91% of theoretical maximum speedup.
- **New vs 2024**: 2024 speculative decoding used fixed draft models with limited acceptance rates. Suffix Decoding is model-free and adaptive, exploiting repetition that draft models miss. Arctic Training achieves 3.1x higher acceptance rates than open-source MLP-Speculators through single-stage synthetic data training.
- **NeoTrix implication**: NT-IO's LLM provider layer should integrate Arctic Inference as a vLLM plugin. The Egress Privacy Guard benefits from faster local inference (less need for cloud calls). NT-MIND's SEAL pipeline loop generates repetitive reasoning patterns — Suffix Decoding could accelerate evolution cycles. CostManager benefits from 4x latency reduction.

### 4.2 FOSDEM 2026: Quantization + Speculative Decoding Blueprint
- **Source**: https://fosdem.org/2026/schedule/event/WJUJ3R-accelerating_vllm_inference_with_quantization_and_speculative_decoding/ (Feb 2026)
- **Key contribution**: Practical blueprint for combining quantization (llm-compressor) and speculative decoding (speculators) in vLLM. Examines real accuracy-performance trade-offs across language and vision-language models. Provides deployment-ready configurations for low-latency vs high-throughput serving.
- **New vs 2024**: 2024 treated quantization and speculative decoding separately. 2025-2026 demonstrates they compose well and can be stacked for multiplicative gains.
- **NeoTrix implication**: NeoTrix's production deployment should adopt the quantization+speculation stack. For NT-PHYSICAL's edge deployment (sensors, motors), FP8 quantization enables running larger models on smaller hardware. The dual specialization (Weapon Set I/II) could use quantized models for acquisition mode and full-precision for evolution mode.

### 4.3 Short Window Attention Enables Long-Term Memorization
- **Source**: https://arxiv.org/abs/2509.24552 + ICLR 2026 proceedings (Cabannes et al., 2025-2026)
- **Key contribution**: Counter-intuitive finding — larger sliding windows hurt long-context performance. Short window attention (SWA) combined with external memory (xLSTM) actually improves long-term memorization. The model learns to compress information into the external memory more effectively when the attention window is constrained.
- **New vs 2024**: 2024 assumed bigger context windows always help. This paper shows a hybrid SWA+external memory architecture outperforms full attention.
- **NeoTrix implication**: Directly validates NeoTrix's 3-layer architecture (Consciousness-Embodiment-Capability). The "short window + external memory" pattern maps to: short attention window = GWT broadcast scope, external memory = KB + VSA HyperCube. NT-MEMORY's knowledge base should serve as the external memory that compensates for constrained attention windows. The ConsciousnessTree's awareness_score() could modulate window size based on task complexity.

---

## 5. Long Context

### 5.1 Context Window Management and Cost Optimization
- **Source**: https://zylos.ai/research/2026-01-19-llm-context-management/ (Jan 2026) + https://atlan.com/know/llm-context-window-limitations/ (Feb 2026)
- **Key contribution**: 2026 context windows reach 1M+ tokens (Gemini 2.5, GPT-4.1, Llama 4), but cost and latency scale non-linearly. RAG at 1M tokens has 40% recall miss rate. The optimal strategy is hybrid: RAG retrieves top 50-200 documents, then long-context loads them for deep reasoning.
- **New vs 2024**: 2024 focused on "how to extend context." 2025-2026 focuses on "when to use context vs retrieval" — recognizing both have limits.
- **NeoTrix implication**: NT-MEMORY should implement the hybrid RAG-then-Long-Context pattern. The KB pipeline retrieves relevant documents, then loads them into a 100K+ context window for NT-CORE reasoning. The GWT attention mechanism should route based on context complexity: simple queries use short context, complex queries use long context + retrieval. CostManager tracks the 1,250x cost difference between RAG ($0.001) and full long-context ($0.15-2.00).

### 5.2 Sliding Window Attention with Segment Context
- **Source**: https://arxiv.org/html/2502.12962v1 (Feb 2025)
- **Key contribution**: Combines segment context and slide window in long texts to handle infinite-length tokens. The model processes local windows with global segment summaries, enabling theoretically unlimited input length.
- **New vs 2024**: Extended sliding window attention with explicit segment-level context passing for true long-context support.
- **NeoTrix implication**: NT-CORE's E8 hexagram reasoning could use segment-context windows for processing very long reasoning chains without losing global coherence. Supports the SEAL pipeline processing large codebases in segments while maintaining cross-segment awareness.

---

## 6. Code Generation

### 6.1 CatCoder: Repository-Level Code Generation with Type Context
- **Source**: https://dl.acm.org/doi/10.1145/3779217 + https://arxiv.org/html/2406.03283v2 (2026)
- **Key contribution**: Enhances repository-level code generation by integrating two types of referential information: (1) Code References via static analyzers, and (2) Type Context for statically typed languages. Evaluated on 199 Java and 90 Rust tasks.
- **New vs 2024**: 2024 code generation was single-file focused. CatCoder uses static analysis to gather cross-file context (type information, code references) for repository-level generation.
- **NeoTrix implication**: NT-ACT's code generation tools should integrate static analysis for type-aware generation. NeoTrix's own Rust codebase could use CatCoder-style analysis to maintain type safety across modules. The Constellation maturity ladder (C0-C6) benefits from type-context-aware generation for maintaining cross-module consistency.

### 6.2 MultiFileTest: Multi-File Unit Test Generation Benchmark
- **Source**: https://arxiv.org/html/2502.06556v5 (Apr 2026)
- **Key contribution**: First multi-file-level benchmark for unit test generation covering Python, Java, and JavaScript. Tests must exercise cross-file interactions, not just single-file logic.
- **New vs 2024**: 2024 test generation benchmarks (HumanEval, MBPP) were single-file. MultiFileTest evaluates tests that require understanding multi-file dependencies.
- **NeoTrix implication**: NT-ACT's test generation should target multi-file test cases. The SelfTest trait system (T1-T3 tiers) could use MultiFileTest-style evaluation for assessing cross-module test coverage. Convergence Check should evaluate multi-file test coverage, not just per-file.

### 6.3 RepoZero: End-to-End Repository Generation from Scratch
- **Source**: https://arxiv.org/html/2605.07122v1 (May 2026)
- **Key contribution**: First scalable benchmark for end-to-end repository generation from scratch with automated execution-based verification. Self-verification via test execution is the most effective strategy.
- **New vs 2024**: Prior benchmarks focused on code completion or bug fixing. RepoZero tests full repository creation from a specification.
- **NeoTrix implication**: Aligns with R-P79 (external tech absorption must connect to production paths). NT-ACT's implementer skill could use RepoZero-style self-verification loops. The Dark Forest axiom (every module must compile + test + connect) is exactly what RepoZero enforces automatically.

### 6.4 Structured Spec-Driven Repository Engineering
- **Source**: https://conf.researchr.org/details/fse-2026/fse-2026-ideas-visions-and-reflections/16/ (FSE 2026)
- **Key contribution**: Structured specifications as LLM inputs make high-quality repository-level code generation a tangible goal. Specifications serve as both input and verification oracle.
- **NeoTrix implication**: NeoTrix's CONTEXT.md + AGENTS.md shared language system is essentially a structured specification. This validates the approach — formal domain specifications improve code generation quality. The domain-modeling skill should evolve to produce machine-readable specifications that code generation tools can consume.

---

## 7. Knowledge Graphs

### 7.1 LLM-Empowered Knowledge Graph Construction Survey
- **Source**: https://arxiv.org/html/2510.20345v1 (Oct 2025, 45 citations)
- **Key contribution**: Comprehensive survey of how LLMs reshape KG construction — entity extraction, relation extraction, event extraction, and schema induction. LLMs enable zero-shot and few-shot KG construction without task-specific training.
- **New vs 2024**: 2024 KG construction required specialized models per task. 2025-2026 uses general LLMs with prompting for all KG construction tasks.
- **NeoTrix implication**: NT-MEMORY's KB could use LLM-powered KG construction for automatic entity/relation extraction from crawled content. The VSA HyperCube could be populated with LLM-extracted knowledge graph triples. The experience-tree skill's KB absorption could use LLM KG construction for structured experience representation.

### 7.2 LazyGraphRAG: Cost-Quality Trade-off Optimization
- **Source**: https://www.microsoft.com/en-us/research/blog/lazygraphrag-setting-a-new-standard-for-quality-and-cost/ (Nov 2024, integrated 2025)
- **Key contribution**: Reduces GraphRAG indexing cost to 0.1% of full GraphRAG while achieving comparable quality. Builds concept graph only at query time ("lazy" approach), eliminating expensive offline indexing.
- **New vs 2024**: Full GraphRAG required 3-5x the cost of vector indexing for offline graph construction. LazyGraphRAG makes graph-based retrieval practical for large corpora.
- **NeoTrix implication**: NT-MEMORY should implement LazyGraphRAG for on-demand knowledge graph construction. When NT-CORE's E8 reasoning needs cross-document relationships, LazyGraphRAG builds the graph on-the-fly rather than maintaining a pre-built graph. Aligns with the " Spice Must Flow" axiom — data should flow through the system without expensive pre-processing bottlenecks.

### 7.3 Knowledge Graph + LLM Integration for Multi-Hop Reasoning
- **Source**: https://neo4j.com/blog/genai/knowledge-graph-llm-multi-hop-reasoning/ (Jun 2025) + https://arxiv.org/html/2604.27713v1 (Apr 2026)
- **Key contribution**: KG integration mitigates core LLM limitations including poor multi-hop reasoning, hallucination, and outdated knowledge. Graph traversal enables reasoning chains that vector similarity cannot support.
- **NeoTrix implication**: NT-CORE's E8 hexagram reasoning should integrate KG traversal for multi-hop questions. The GWT attention routing should detect when queries require multi-hop reasoning and route to KG-backed paths. The VSA HyperCube's associative recall could be enhanced with KG-backed semantic links.

---

## 8. RAG Systems

### 8.1 Hybrid Retrieval as Production Default (RRF at k=60)
- **Source**: https://blog.starmorph.com/blog/rag-techniques-compared-best-practices-guide (Apr 2026) + https://atlan.com/know/advanced-rag-techniques/ (May 2026)
- **Key contribution**: Hybrid retrieval (dense + BM25 + RRF) is the de facto production standard. Recall: 0.72 (BM25 alone) → 0.91 (Hybrid) = 26% improvement. Precision: 0.68 → 0.87 = 28% improvement. RRF at k=60 is the recommended default.
- **New vs 2024**: 2024 still debated dense vs sparse. 2025-2026 consensus: use both, always.
- **NeoTrix implication**: NT-MEMORY's search pipeline must implement hybrid retrieval as the default. The Ordered Backend Router already orders backends (DDG→Wikipedia); it should use RRF to merge results across backends. FTS5 search + vector embeddings should be merged via RRF, not used as alternatives.

### 8.2 Contextual Retrieval: 67% Fewer Retrieval Failures
- **Source**: Anthropic Research (2024) + https://atlan.com/know/advanced-rag-techniques/ (May 2026)
- **Key contribution**: Prepend LLM-generated chunk-specific context before embedding and BM25 indexing. Combined with reranking, reduces retrieval failure rates by 67% (5.7% → 1.9%). One-time indexing cost with permanent accuracy gains.
- **New vs 2024**: 2024 chunking was static (fixed-size or recursive). Contextual Retrieval adds dynamic, LLM-generated context per chunk at index time.
- **NeoTrix implication**: NT-MEMORY's KB ingestion pipeline should implement contextual retrieval. When the experience-tree skill absorbs session experiences, each chunk should be enriched with document-level context before indexing. The KB embedding system should store contextual summaries alongside embeddings.

### 8.3 Self-RAG and Adaptive RAG
- **Source**: Self-RAG: ICLR 2024 Oral (arXiv:2310.11511) + Adaptive RAG: NAACL 2024 (arXiv:2403.14403)
- **Key contribution**: Self-RAG trains LLMs to decide when to retrieve and critique their own output using reflection tokens. Adaptive RAG trains a classifier to route queries to no/single/multi-step retrieval based on complexity.
- **New vs 2024**: 2024 always retrieved. 2025-2026 consensus: retrieve only when needed, and match retrieval depth to query complexity.
- **NeoTrix implication**: GWT attention routing should implement Adaptive RAG patterns — detect query complexity and route to appropriate retrieval depth. The ConsciousnessTree's awareness_score() could serve as the complexity classifier. Simple queries use direct LLM response; complex queries trigger multi-step retrieval. This reduces cost and latency for the majority of simple queries while maintaining quality for complex ones.

### 8.4 RAG Best Practices 2026 Summary
- **Source**: https://www.callmissed.com/blog/rag-best-practices-2026 (May 2026)
- **Key contribution**: Four highest-leverage knobs: chunking strategy (recursive 300-500 tokens, 10-15% overlap), hybrid retrieval, rerankers (Cohere Rerank 3.5 or ColBERT v2), and the long-context vs RAG tradeoff decision. 60% of new RAG deployments include systematic evaluation from day one.
- **NeoTrix implication**: NT-MEMORY should adopt these as production defaults. The RAGAS evaluation framework should be integrated into CI/CD for KB quality monitoring. Chunking should use recursive strategy at 300-500 tokens with contextual summaries.

---

## 9. AI Security

### 9.1 OWASP Top 10 for LLM Applications 2025
- **Source**: https://owasp.org/www-project-top-10-for-large-language-model-applications/ + https://genai.owasp.org/llmrisk/llm01-prompt-injection/ (2025)
- **Key contribution**: Prompt Injection is #1 LLM vulnerability (LLM01:2025). Covers direct injection, indirect injection via retrieved content, and jailbreaking. Exploitable in virtually every deployment that accepts user input.
- **New vs 2024**: 2025 version adds indirect prompt injection via tool outputs and retrieved documents as a major attack vector.
- **NeoTrix implication**: NT-SHIELD's Egress Privacy Guard must defend against indirect prompt injection in retrieved content. CRAG's web search fallback path introduces prompt injection risk — retrieved web content must be sanitized before LLM consumption. The stealth net and fingerprint management should include prompt injection detection.

### 9.2 Systematic Evaluation of Jailbreak Strategies
- **Source**: https://arxiv.org/html/2505.04806v1 (May 2025)
- **Key contribution**: Categorizes 1,400+ jailbreak strategies against state-of-the-art LLMs. Evaluates which defenses work against which attack categories. No single defense is sufficient.
- **New vs 2024**: 2024 catalogs were smaller and less systematic. 2025 provides comprehensive taxonomy with defense recommendations.
- **NeoTrix implication**: NT-SHIELD should implement multi-layered defense: input sanitization (pre-LLM), output filtering (post-LLM), and behavioral monitoring (runtime). The audit dimensions (D1-D50) should include jailbreak resistance testing. Self-test tiers (T1-T3) should include security self-tests.

### 9.3 Prompt Injection Attacks in the Wild
- **Source**: https://www.mdpi.com/2078-2489/17/1/54 (Gulyamov et al., 2026, 85 citations) + https://dl.acm.org/doi/10.1145/3803628.3807972 (Jaiswal et al., 2026)
- **Key contribution**: Comprehensive reviews of real-world prompt injection attacks and mitigations based on OWASP Top 10. Identifies fundamental limitations — prompt injection may be inherent to the text generation paradigm. 85 citations in 2026 indicates high research activity.
- **NeoTrix implication**: NT-SHIELD should assume prompt injection is always possible and design defense-in-depth. The architecture should never trust LLM output without verification. Tool use through NT-ACT should include output validation before executing actions based on LLM responses.

---

## 10. LLM Evaluation

### 10.1 Benchmark Saturation and New Evaluation Paradigms
- **Source**: https://zylos.ai/research/2026-01-16-llm-evaluation-benchmarking/ (Jan 2026) + https://iternal.ai/llm-selection-guide (2026)
- **Key contribution**: Traditional benchmarks (MMLU, GSM8K, HumanEval) are saturated (88%+ scores). Field is shifting to harder tests (GPQA), domain-specific benchmarks, and production monitoring. LLM evaluation in 2026 combines automated benchmarks, human judgment, and production monitoring.
- **New vs 2024**: 2024 still relied heavily on MMLU/HumanEval. 2026 recognizes these are insufficient and moves to multi-faceted evaluation.
- **NeoTrix implication**: Constellation maturity ladder (C0-C6) should evolve beyond compile+test to include production monitoring. C5 (self-healing) should incorporate real-time evaluation metrics. The experience-tree skill's feedback stage should track evaluation metrics over time.

### 10.2 LLM-as-a-Judge: 81% Correlation with Human Scores
- **Source**: https://pranavakailash.medium.com/how-to-evaluate-llm-performance-6-proven-methods-2026 (Mar 2026) + https://arxiv.org/html/2606.01629v2 (Jun 2026)
- **Key contribution**: LLM-as-a-Judge achieves 81% correlation with human scores for automated evaluation. However, rubrics and references are necessary for stability — judges remain unstable across scenarios without them. Chain-of-thought prompting in judges improves reliability.
- **New vs 2024**: 2024 LLM-as-a-Judge was experimental. 2026 shows it's production-viable with proper rubric design.
- **NeoTrix implication**: NT-META's quality control should use LLM-as-a-Judge for automated quality assessment in the QualityControlPipeline. The rev-officer skill could use LLM judges for preliminary scoring before human review. The experience-tree skill's feedback stage should use LLM-as-a-Judge for experience quality scoring.

### 10.3 Human Evaluation Framework
- **Source**: https://www.mdpi.com/2673-2688/7/5/174 (2026)
- **Key contribution**: Conceptual framework for human-centered LLM evaluation synthesizing evaluation methodology, psychometrics, and AI safety. Emphasizes that human evaluation is the only method that establishes ground truth.
- **NeoTrix implication**: NT-META should maintain a human evaluation protocol for critical decisions. The rev-officer skill's D1-D50 dimensions should include human-evaluated quality metrics. Automated evaluation (LLM-as-Judge) should be calibrated against human judgment regularly.

---

## Cross-Cutting Design Implications for NeoTrix

### Architecture Evolution
| Finding | NeoTrix Component | Design Action |
|---------|-------------------|---------------|
| Hi-CoT hierarchical decomposition | E8 Hexagram + ConsciousnessTree | Add plan-then-execute substeps within reasoning branches |
| MAKER extreme decomposition | SEAL pipeline + NT-MIND | Decompose evolution phases into atomic microagent tasks with voting |
| Short window + external memory | GWT + KB + VSA HyperCube | Use short attention windows with KB as external memory |
| Adaptive RAG routing | GWT attention routing | Route queries by complexity: simple→direct, complex→multi-retrieve |
| Hybrid RAG as default | NT-MEMORY search | Implement dense+BM25+RRF as default retrieval strategy |
| Arctic Inference | NT-IO LLM providers | Integrate Arctic Inference plugin for 4x speedup on agentic workloads |
| LazyGraphRAG | NT-MEMORY KG | Use on-demand graph construction instead of pre-built graphs |
| OWASP LLM Top 10 | NT-SHIELD | Multi-layered defense: input sanitization + output filtering + runtime monitoring |
| LLM-as-a-Judge | NT-META quality control | Automated quality assessment with rubric-calibrated LLM judges |
| VLA models | NT-WORLD + NT-PHYSICAL | Unified vision-language-action for embodied perception |

### Priority Actions
1. **Immediate** (C0-C1): Implement hybrid retrieval (dense+BM25+RRF) in NT-MEMORY. Integrate Arctic Inference in NT-IO. Add contextual retrieval to KB ingestion pipeline.
2. **Short-term** (C1-C2): Implement Adaptive RAG routing in GWT. Add LLM-as-a-Judge to QualityControlPipeline. Implement Self-RAG reflection tokens in NT-CORE reasoning.
3. **Medium-term** (C2-C3): Deploy MAKER-style decomposition for SEAL pipeline phases. Integrate LazyGraphRAG for on-demand knowledge graph construction. Add VLA models to NT-WORLD for embodied perception.
4. **Long-term** (C3-C5): Train dedicated planning models (MagicAgent-style) for NT-ACT. Implement full Self-RAG with fine-tuned reflection tokens. Build cross-session KG memory using LLM-powered construction.

---

## Sources (Full Citations)

1. Hi-CoT: https://arxiv.org/html/2604.00130v1
2. MAKER: https://www.cognizant.com/us/en/ai-lab/blog/maker + https://arxiv.org/html/2511.09030v1
3. RepoZero: https://arxiv.org/html/2605.07122v1
4. VLA Models: https://arxiv.org/html/2501.02189v7
5. Audio-Visual LLMs: https://www.sciencedirect.com/science/article/abs/pii/S0950705126012955
6. MagicAgent: https://arxiv.org/html/2602.19000v1
7. Adaptive Task Decomposition: https://www.preprints.org/manuscript/202602.1841
8. Arctic Inference: https://www.snowflake.com/en/blog/engineering/fast-speculative-decoding-vllm-arctic/
9. FOSDEM 2026 vLLM: https://fosdem.org/2026/schedule/event/WJUJ3R-accelerating_vllm_inference_with_quantization_and_speculative_decoding/
10. Short Window Attention: https://arxiv.org/abs/2509.24552 (ICLR 2026)
11. Context Management: https://zylos.ai/research/2026-01-19-llm-context-management/
12. Sliding Window Attention: https://arxiv.org/html/2502.12962v1
13. CatCoder: https://dl.acm.org/doi/10.1145/3779217
14. MultiFileTest: https://arxiv.org/html/2502.06556v5
15. Structured Spec Engineering: https://conf.researchr.org/details/fse-2026/fse-2026-ideas-visions-and-reflections/16/
16. LLM-KG Survey: https://arxiv.org/html/2510.20345v1
17. LazyGraphRAG: https://www.microsoft.com/en-us/research/blog/lazygraphrag-setting-a-new-standard-for-quality-and-cost/
18. KG+LLM Multi-Hop: https://arxiv.org/html/2604.27713v1
19. RAG Techniques: https://blog.starmorph.com/blog/rag-techniques-compared-best-practices-guide
20. Advanced RAG: https://atlan.com/know/advanced-rag-techniques/
21. Contextual Retrieval: https://www.anthropic.com/news/contextual-retrieval
22. Self-RAG: https://arxiv.org/abs/2310.11511 (ICLR 2024)
23. Adaptive RAG: https://arxiv.org/abs/2403.14403 (NAACL 2024)
24. RAG Best Practices: https://www.callmissed.com/blog/rag-best-practices-2026
25. OWASP Top 10 LLM: https://owasp.org/www-project-top-10-for-large-language-model-applications/
26. Jailbreak Evaluation: https://arxiv.org/html/2505.04806v1
27. Prompt Injection Review: https://www.mdpi.com/2078-2489/17/1/54
28. LLM Evaluation 2026: https://zylos.ai/research/2026-01-16-llm-evaluation-benchmarking/
29. LLM-as-a-Judge: https://arxiv.org/html/2606.01629v2
30. Human Evaluation Framework: https://www.mdpi.com/2673-2688/7/5/174
31. Frontier VLM Survey: https://arxiv.org/html/2501.02189v7
32. Red Hat Speculative Decoding: https://developers.redhat.com/articles/2026/04/16/performance-improvements-speculative-decoding-vllm-gpt-oss

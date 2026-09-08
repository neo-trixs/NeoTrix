# Iteration Batch 631 — Conversational AI / Intent Recognition / Dialogue Management

**Date**: 2026-09-06
**Sources**: 16 papers across ACL 2026, arXiv 2026, ICASSP 2026, LREC 2026

---

## 1. CONVERSATIONAL AI FINDINGS

### 1.1 VoxMind — End-to-End Agentic Spoken Dialogue
- **Source**: ACL 2026 (aclanthology.org/2026.acl-long.459/)
- **Key**: "Think-before-Speak" mechanism + Multi-Agent Dynamic Tool Management. Async delegation decouples inference latency from toolset size. Task completion 34.88% → 74.57%.
- **Defect Found (D631-01)**: NeoTrix NT-IO dialogue path has no **asynchronous tool delegation** pattern. Current architecture blocks LLM inference on tool round-trips. VoxMind proves async delegation is critical for latency.
- **Improvement**: Implement agent-colocated async tool pool in NT-IO; LLM generates tool calls, auxiliary agent resolves them in parallel without blocking reasoning.

### 1.2 APEX-MEM — Agentic Semi-Structured Memory with Temporal Reasoning
- **Source**: ACL 2026 (aclanthology.org/2026.acl-long.749/)
- **Key**: Property graph + append-only temporal storage + multi-tool retrieval agent for conflict resolution. 88.88% on LOCOMO, 86.2% on LongMemEval.
- **Defect Found (D631-02)**: NeoTrix KB stores snapshots but lacks **temporal append-only event graph** with conflict resolution at query time. KB currently overwrites rather than appending with temporal grounding.
- **Improvement**: Add temporal provenance layer to KB nodes; each write appends timestamped version; retrieval agent resolves conflicts at query time (not write time).

### 1.3 ChatR1 — RL for Conversational Reasoning + Retrieval
- **Source**: ACL 2026 (aclanthology.org/2026.acl-long.103/)
- **Key**: Intent-aware reward providing turn-level feedback; interleaves search and reasoning across turns (not static pipeline). 3B/7B models outperform competitors.
- **Defect Found (D631-03)**: NeoTrix GWT attention routing is **static score-based** — no RL-trained intent-aware modulation. Routing decisions don't adapt based on evolving user goals across turns.
- **Improvement**: Introduce intent-aware attention modulation in GWT; track user goal evolution across turns, adjust routing weights dynamically via RL signal.

### 1.4 Inside Out — PersonaTree Core Memory
- **Source**: ACL 2026 (aclanthology.org/2026.acl-long.614/)
- **Key**: Hierarchical Schema (biopsychosocial model) → PersonaTree with ADD/UPDATE/DELETE/NO_OP operations via RL-trained MemListener. Adaptive: fast mode (read tree) vs agentic recall mode (deep retrieval).
- **Defect Found (D631-04)**: NeoTrix self-model types (nt_core_meta, nt_core_self, nt_core_self_model) are **static structural/performance/value models** — no dynamic persona evolution via structured tree operations. No fast-vs-agentic adaptive inference.
- **Improvement**: Add PersonaTree-inspired structured user model to NT-CORE; support lightweight read mode vs deep agentic recall mode for latency-sensitive vs detail-demanding queries.

### 1.5 RuleMem — Active Rule Memory
- **Source**: arXiv 2609.03915
- **Key**: Induces reusable Horn clauses from conversations; Rule Perplexity Consistency validation. 27.47 point improvement over baselines on LoCoMo.
- **Defect Found (D631-05)**: NeoTrix KB stores facts but has **no rule induction** from conversation history. No mechanism to abstract learned patterns into reusable logical rules.
- **Improvement**: Add rule induction layer to NT-MEMORY; periodically abstract conversation patterns into Horn clauses or production rules; validate via consistency checking.

### 1.6 DarwinTOD — Lifelong Self-Evolution for TOD
- **Source**: ACL 2026 (aclanthology.org/2026.acl-long.2050/)
- **Key**: Evolvable Strategy Bank + dual-loop (online peer critique + offline evolutionary operations). Continuous improvement without human intervention. Population-based strategy evolution.
- **Defect Found (D631-06)**: NeoTrix SEAL pipeline evolves **modules** but not **dialogue strategies**. No peer critique mechanism between strategy variants. No population-based evolution of conversational policies.
- **Improvement**: Add strategy evolution dimension to SEAL; maintain multiple dialogue strategy variants; peer critique loop for strategy fitness evaluation; offline evolutionary refinement.

### 1.7 TiMem — Temporal-Hierarchical Memory Consolidation
- **Source**: ACL 2026 (aclanthology.org/2026.findings-acl.1091/)
- **Key**: Temporal Memory Tree with explicit temporal containment; instruction-guided consolidation (no fine-tuning); complexity-aware recall (simple/hybrid/complex routing). 52.20% reduced recalled context.
- **Defect Found (D631-07)**: NeoTrix memory is **flat semantic search** — no temporal hierarchy, no consolidation pipeline, no query-complexity-adaptive recall depth.
- **Improvement**: Implement temporal memory tree in NT-MEMORY; consolidate episodic → semantic → persona representations; route queries by complexity to appropriate tree depth.

---

## 2. INTENT RECOGNITION FINDINGS

### 2.1 DOMA — Diffusion Language Models for ICSF
- **Source**: ICASSP 2026 (doi.org/10.1109/icassp55912.2026.11462605)
- **Key**: Diffusion language model refines ASR transcripts; adaptive prior for faster inference. 3.2% relative improvement, 34.8% latency reduction.
- **Defect Found (D631-08)**: NeoTrix has no **ASR error correction layer** before intent classification. Speech→text errors propagate unmitigated into downstream intent recognition.
- **Improvement**: Add diffusion-based ASR refinement module in NT-IO speech pipeline; use adaptive prior for low-latency correction.

### 2.2 SSS-GIN — Multi-Intent Semantic Scope Graph
- **Source**: ICNLP 2026 (doi.org/10.1109/icnlp69856.2026.11527603)
- **Key**: Token-centric heterogeneous graph for symmetrical fusion; affinity-gating for locality-aware masks; 57.7% OA on MixATIS.
- **Defect Found (D631-09)**: NeoTrix intent handling assumes **single-intent per utterance**. No graph-based multi-intent detection with cross-intent boundary modeling.
- **Improvement**: Add multi-intent graph interaction layer; model intent-slot dependencies as heterogeneous graph; affinity-gating for intent boundary detection.

### 2.3 LLM vs Fine-Tuned NLU Decision Framework
- **Source**: arXiv 2608.20371
- **Key**: Decision framework: fine-tuned RoBERTa beats LLM on ATIS (+11.8pp), but LLM wins on OOS detection (85.6 vs 58.1), ASR noise robustness (92.5 vs 80.0 at 0dB), and dynamic per-deployment schemas (~94% both).
- **Defect Found (D631-10)**: NeoTrix has **no OOS (out-of-scope) detection** in intent classification. No robustness strategy for ASR noise. No decision framework for when to use fine-tuned vs LLM-based NLU.
- **Improvement**: Implement OOS detection gate before intent classification; add ASR noise robustness testing; build routing logic: high-confidence schemas → fine-tuned model, OOS/noisy/dynamic → LLM fallback.

### 2.4 ECLM — Entity-Level LM with Chain of Intent
- **Source**: ACL 2025 (aclanthology.org/2025.acl-long.1061.pdf)
- **Key**: Reformulates slot-filling as entity recognition; Chain of Intent for step-by-step multi-intent decomposition. +3.7% on MixATIS, +21.2% vs vanilla LLM fine-tuning on MixSNIPS.
- **Defect Found (D631-11)**: NeoTrix slot filling is token-level BIO labeling — misalignment issues with autoregressive generation. No chain-of-intent decomposition for multi-intent.
- **Improvement**: Shift slot filling to entity-level detection (start/end positions); add Chain of Intent reasoning for multi-intent decomposition.

### 2.5 Zero-Shot SLU Architectural Survey
- **Source**: ACL 2026 IWSLT (aclanthology.org/2026.iwslt-1.1.pdf)
- **Key**: 13-language evaluation; hybrid architectures best for zero-shot SLU; E2E models lag significantly; cross-lingual transfer limited to linguistically similar languages.
- **Defect Found (D631-12)**: NeoTrix has **no multilingual intent classification** strategy. No architectural decision framework for cascaded vs E2E vs hybrid SLU.
- **Improvement**: Add multilingual intent classification support; implement hybrid architecture decision framework; map language families for transfer learning.

---

## 3. DIALOGUE MANAGEMENT FINDINGS

### 3.1 ReacTOD — Neuro-Symbolic Agentic NLU for Zero-Shot DST
- **Source**: ACL 2026 TrustNLP (aclanthology.org/2026.trustnlp-main.21.pdf)
- **Key**: Bounded ReAct loop + deterministic validator (action compliance, schema conformance, coreference consistency). 93.1% self-correction rate. 8B model surpasses prior zero-shot SOTA by +14pp.
- **Defect Found (D631-13)**: NeoTrix has **no deterministic validation layer** for dialogue state mutations. No schema conformance checking. No coreference consistency enforcement. LLM errors propagate unchecked.
- **Improvement**: Add deterministic validator before every state mutation in NT-IO dialogue pipeline; enforce action compliance, schema conformance, coreference consistency; bounded self-correction loop.

### 3.2 CoDial — Code for Dialogue (Interpretable TOD)
- **Source**: ACL 2026 (aclanthology.org/2026.acl-long.1980.pdf)
- **Key**: Converts task schema → heterogeneous graph → Colang guardrailing code. Interpretable dialogue flow; human feedback integration for iterative improvement.
- **Defect Found (D631-14)**: NeoTrix dialogue policies are **opaque LLM prompts** — no interpretable flow representation. No mechanism for human feedback to refine dialogue policies.
- **Improvement**: Implement schema-to-graph-to-code pipeline for NT-IO dialogue flows; add interpretable dialogue flow execution; enable human feedback loop for policy refinement.

### 3.3 GEM — Graph-Enhanced MoE + ReAct Agents for DST
- **Source**: arXiv 2605.04449
- **Key**: GNN captures dialogue structure + T5 for sequence modeling + ReAct agent for value generation. 65.19% JGA on MultiWOZ 2.2 (vs 38.43% pure LLM). Domain-weighted routing.
- **Defect Found (D631-15)**: NeoTrix has **no mixture-of-experts routing** for dialogue tasks. All dialogue processing goes through single path regardless of domain complexity.
- **Improvement**: Implement MoE routing in NT-IO; GNN for structural dialogue understanding, small encoder-decoder for slot extraction, LLM for complex reasoning; domain-weighted voting.

### 3.4 FlowSwitch — State-Aware Workflow Transitions
- **Source**: ACL 2026 IWSDS (aclanthology.org/2026.iwsds-1.2.pdf)
- **Key**: Detects workflow transitions mid-dialogue; hierarchical retrieval (role→domain→workflow); self-generated search queries outperform raw history. 50% reduction in search operations.
- **Defect Found (D631-16)**: NeoTrix has **no dynamic workflow switching**. Once a dialogue enters a workflow, it cannot robustly detect when user intent drifts to a different workflow.
- **Improvement**: Add workflow switch detector in NT-IO; hierarchical workflow retrieval pool; agent-generated search queries for workflow transition.

### 3.5 IDSS — Intent-Driven Situation States
- **Source**: arXiv 2608.15755
- **Key**: Separates tool-grounded facts from task-state judgments; constraint propagation across layers; provenance-aware entities; blocks infeasible intents.
- **Defect Found (D631-17)**: NeoTrix has **no separation between grounded facts and task-state judgments**. Tool outputs and user intents are mixed in flat context. No constraint propagation.
- **Improvement**: Implement dual-layer state in NT-IO: fact layer (tool-grounded) + state layer (user intents/constraints/status); cross-layer constraint propagation for infeasibility detection.

### 3.6 Speech-LLM for End-to-End Spoken DST
- **Source**: LREC 2026 (lrec-conf.org/proceedings/lrec2026/pdf/2026.lrec2026-1.206.pdf)
- **Key**: Full spoken conversation as input outperforms multimodal context; attention-pooling compression as strong trade-off.
- **Defect Found (D631-18)**: NeoTrix spoken dialogue always goes through **cascaded ASR→text→NLU pipeline**. No end-to-end spoken DST path. Error propagation from ASR is unmitigated.
- **Improvement**: Add E2E spoken DST path in NT-IO for speech-first interactions; attention-pooling compression for long spoken histories.

---

## DEFECT SUMMARY

| ID | Domain | Defect | Severity |
|----|--------|--------|----------|
| D631-01 | NT-IO | No async tool delegation in dialogue | HIGH |
| D631-02 | NT-MEMORY | No temporal append-only event graph | HIGH |
| D631-03 | NT-CORE | GWT static score-based, no intent-aware modulation | HIGH |
| D631-04 | NT-CORE | No dynamic persona evolution via structured ops | MEDIUM |
| D631-05 | NT-MEMORY | No rule induction from conversations | HIGH |
| D631-06 | NT-MIND | SEAL evolves modules not dialogue strategies | MEDIUM |
| D631-07 | NT-MEMORY | No temporal hierarchy, no complexity-adaptive recall | HIGH |
| D631-08 | NT-IO | No ASR error correction before intent classification | MEDIUM |
| D631-09 | NT-IO | Single-intent assumption, no multi-intent graph | HIGH |
| D631-10 | NT-IO | No OOS detection, no ASR noise robustness | HIGH |
| D631-11 | NT-IO | Token-level BIO slot filling, no entity-level | MEDIUM |
| D631-12 | NT-IO | No multilingual intent classification | MEDIUM |
| D631-13 | NT-IO | No deterministic validation for state mutations | CRITICAL |
| D631-14 | NT-IO | Opaque dialogue policies, no human feedback loop | MEDIUM |
| D631-15 | NT-IO | No MoE routing for dialogue tasks | HIGH |
| D631-16 | NT-IO | No dynamic workflow switching | HIGH |
| D631-17 | NT-IO | No fact/intent separation, no constraint propagation | HIGH |
| D631-18 | NT-IO | Cascaded ASR only, no E2E spoken DST | HIGH |

---

## CRITICAL DEFECTS (Priority Order)

1. **D631-13**: No deterministic validation for dialogue state mutations — silent failures in production
2. **D631-02**: No temporal append-only event graph — knowledge conflicts unresolved
3. **D631-05**: No rule induction — patterns never crystallize into reusable logic
4. **D631-09**: Single-intent assumption — 52% of real utterances have multiple intents
5. **D631-10**: No OOS detection — system confidently misclassifies unknown intents
6. **D631-17**: No fact/intent separation — tool outputs contaminate task reasoning

---

## NEW CAPABILITIES REQUIRED

| Capability | Source Inspiration | NeoTrix Target Module |
|------------|-------------------|----------------------|
| Async Tool Delegation | VoxMind | NT-IO |
| Temporal Event Graph | APEX-MEM, TiMem | NT-MEMORY |
| Intent-Aware GWT Modulation | ChatR1 | NT-CORE |
| PersonaTree Structured Ops | Inside Out | NT-CORE |
| Rule Induction Engine | RuleMem | NT-MEMORY |
| Dialogue Strategy Evolution | DarwinTOD | NT-MIND |
| Complexity-Adaptive Recall | TiMem | NT-MEMORY |
| Multi-Intent Graph | SSS-GIN | NT-IO |
| OOS Detection Gate | LLM/NLU Decision Framework | NT-IO |
| Deterministic State Validator | ReacTOD | NT-IO |
| Interpretable Dialogue Code | CoDial | NT-IO |
| MoE Dialogue Routing | GEM | NT-IO |
| Workflow Switch Detection | FlowSwitch | NT-IO |
| Fact/Intent Dual Layer | IDSS | NT-IO |
| E2E Spoken DST | Speech-LLM DST | NT-IO |

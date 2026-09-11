# Fable Dataset Research — Multi-Source Compilation

> Research date: 2026-09-10
> Sources: arXiv, Hugging Face, PapersWithCode, GitHub, BIG-bench

---

## 1. FABLE (Data-Flow Analysis Benchmark)

**Paper**: arXiv:2505.24258 (May 2025)
**Authors**: Vishal Pallagani, Nitin Gupta, John Aydin, Biplav Srivastava

| Field | Value |
|-------|-------|
| **Full Name** | FABLE: A Novel Data-Flow Analysis Benchmark on Procedural Text |
| **Task Type** | Procedural data-flow reasoning evaluation |
| **Data Scale** | 2,400 QA pairs (100 per domain-analysis combination) |
| **Domains** | Cooking recipes, Travel routes, Automated plans |
| **Analysis Types** | 8 classical software engineering analyses |
| **Evaluation** | Majority voting over 5 sampled completions per prompt |

### 8 Data-Flow Analysis Types

| Analysis | Description | Reasoning Dimension |
|----------|-------------|---------------------|
| Reaching Definitions | Entity/state introduced earlier reaches a later step without being invalidated | State |
| Very Busy Expressions | Entity/expression produced at one point must be consumed along future paths | Causal |
| Available Expressions | Computed/established state remains available for reuse at a later point | State |
| Live Variable Analysis | Entity/resource/constraint remains needed for future steps | State |
| Interval Analysis | Numeric ranges, durations, bounds, or step intervals across the procedure | Temporal |
| Type-State Analysis | Entities follow valid state transitions across steps | State |
| Taint Analysis | Propagation of contaminated/invalid information through later steps | Causal |
| Concurrency Analysis | Steps can be reordered or executed concurrently without violating dependencies | Temporal |

### Baseline Results

| Model | Params | Plans Δ | Travel Δ | Recipes Δ | Inference Speed |
|-------|--------|---------|----------|-----------|-----------------|
| deepseek-r1:8b | 8B | +34.4% | +31.4% | +37.1% | 20× slower |
| granite-code:8b | 8B | +5.8% | -0.5% | +3.9% | Baseline |
| llama3.1:8b | 8B | +3.0% | +0.2% | +11.8% | Baseline |

**Key Finding**: General-purpose and code-specific models perform close to random chance. Reasoning models improve but at 20× inference cost.

---

## 2. FABLE+ (Extended Benchmark)

**HuggingFace**: `throwaway-13/FABLE-plus`

| Field | Value |
|-------|-------|
| **Full Name** | FABLE+ (extended diagnostic benchmark) |
| **Task Type** | Procedural data-flow reasoning evaluation (extended) |
| **Data Scale** | 3,200 QA instances |
| **Domains** | 4 (cooking, travel, automated plans, **dialog**) |
| **Balance** | 800 instances per domain, 100 per analysis type |

### New in FABLE+: Dialog Domain

20 Questions-style information-seeking game with:
- Positive/negative/unknown answers
- Omitted dependencies
- Unsupported claims
- Tests constraint propagation, invalidation, and recovery from partial information

### Construction Pipeline

1. Domain-specific source data collection
2. Procedure parsing → explicit step/entity representations
3. Step-dependency graph + entity-flow/constraint-flow graph
4. Data-flow analysis template instantiation over graph
5. Ground-truth computation from structured representation
6. Balanced benchmark subset sampling
7. Quality checks (parsing, clarity, correctness)

---

## 3. Claude Fable 5 Distillation Datasets

Multiple datasets derived from Anthropic's "Claude Fable 5" model (Mythos-class, released June 2026).

### 3.1 Claude-Fable-5-5500x (HelioAI)

| Field | Value |
|-------|-------|
| **Source** | Fable 5 model |
| **Task Type** | Reasoning traces for SFT and process supervision |
| **Data Scale** | 5,469 examples |
| **Total Characters** | 54.01M (39.61M reasoning) |
| **Max Reasoning Length** | 161,847 chars |
| **Avg. Reasoning Length** | ~7,200 chars |
| **Languages** | Russian, English |
| **Format** | `{prompt, reasoning, answer}` |

### 3.2 Fable-5-Distill-Reasoning-462x (HelioAI)

| Field | Value |
|-------|-------|
| **Source** | Mythos V2 (full, unrestricted) |
| **Task Type** | Unrestricted full-parameter distillation |
| **Data Scale** | 462 examples |
| **Total Characters** | 104.7M (pure reasoning) |
| **Avg. Trace Length** | ~226K chars |
| **Max Trace Length** | 300K+ chars |
| **Domains** | Cybersecurity, biomedicine, software architecture, AI reasoning, formal math |
| **Key Feature** | No RLHF suppression, no token-budget truncation, no refusal patterns |

### 3.3 Fable-5-Distill-Merged-25k (WithinUsAI)

| Field | Value |
|-------|-------|
| **Task Type** | SFT dataset for chain-of-thought reasoning |
| **Data Scale** | 25,719 examples |
| **Format** | JSONL `{query, thinking}` |
| **File Size** | ~197 MB |
| **Domains** | 23+ technical domains |
| **Think Tags** | 100% `<think>` tagged |
| **Mean Thinking Length** | 6,220 chars |
| **Sources** | 4 merged + MD5 deduped |

### 3.4 Fable-5-Max-Reasoning-Filtered-250x (MoreThought)

| Field | Value |
|-------|-------|
| **Task Type** | FinTech architecture reasoning |
| **Data Scale** | 250 traces |
| **Total Size** | 40 MB |
| **Avg. Trace** | ~160 KB |
| **Domain** | Global banking SWIFT/crypto infrastructure |
| **Filter** | Quality-filtered by Qwen 2.5 7B, improved by GLM 5.2 |

---

## 4. Fable-Method (Agent Benchmark)

**GitHub**: `Sahir619/fable-method` (166 stars, 21 forks)

| Field | Value |
|-------|-------|
| **Full Name** | Fable-Method Distillation Benchmark |
| **Task Type** | Agent workflow compliance evaluation |
| **Data Scale** | 50 hand-curated coding/problem-solving prompts |
| **Framework** | Think / Act / Prove three-stage workflow |

### Metrics

| Metric | Definition |
|--------|-----------|
| Workflow Compliance Score | % of steps completed in strict Think/Act/Prove order |
| Reasoning Fidelity | Match rate between initial plan and final validated output |
| Tool Use Accuracy | % of correctly formatted, purposeful tool calls |
| Final Task Success | End-to-end task completion rate per benchmark prompt |

### Model Rankings (Aug 2026)

| Model | Compliance | Task Success |
|-------|-----------|--------------|
| Claude 3.5 Sonnet | 92% | 88% |
| GPT-4o | 84% | 79% |
| Gemini Advanced | 78% | 72% |
| Llama 3 70B (GPTQ) | 51% | 44% |

**Key Finding**: 17% of all runs fail the `Prove` stage entirely, even when output is correct.

---

## 5. MORABLES (Moral Fable Benchmark)

**Paper**: EMNLP 2025
**HuggingFace**: `cardiffnlp/Morables-PD` (709 entries)

| Field | Value |
|-------|-------|
| **Full Name** | MORABLES: Abstract Moral Reasoning in LLMs with Fables |
| **Task Type** | Multiple-choice moral inference from fables |
| **Data Scale** | 709 fables with morals |
| **Source** | Western literary tradition fables/short stories |
| **Format** | `{alias, title, story, moral}` |
| **Variants** | Standard MCQA, TF (True/False framing), NOTO (None-of-the-Others) |

### Key Results

- Largest models (Llama 3.3 70B: 73.6%, GPT-4o, Claude 3.5) outperform small models (Mistral 7B: 28.4%) by 40+ points
- Models contradict own answers ~20% when moral inference is reframed
- Reasoning-enhanced models contribute less than scale to performance
- Human eval shows weak correlation between semantic similarity and moral correctness

---

## 6. BIG-bench Understanding Fables

**GitHub**: `google/BIG-bench/bigbench/benchmark_tasks/understanding_fables`

| Field | Value |
|-------|-------|
| **Task Type** | Narrative comprehension via moral identification |
| **Data Scale** | 189 paraphrased fables |
| **Source** | Aesop's Fables (aesopfables.com) |
| **Format** | MCQA — 5 alternatives per fable |
| **Distractor Selection** | Semi-automatic: 10 most similar morals → 4 selected manually |

### Key Properties

- Fables manually paraphrased (different characters, sentence structure, register)
- Mean string similarity to originals: 0.26 (Levenshtein distance)
- Mean semantic similarity to originals: 0.78 (cosine similarity)
- Tests cross-domain generalization + narrative comprehension
- Tests anthropomorphic reasoning (animals with gendered pronouns)

### Baseline Performance

| Model | Accuracy |
|-------|----------|
| Random baseline | 20% |
| GPT-2 | 19% |
| BART-Large | 24% |
| GPT-2 NEO 1.3B | 28% |
| RoBERTa-Large | 22% |

---

## 7. TF1-EN-3M (3M Synthetic Moral Fables)

**Paper**: arXiv:2504.20605 (Apr 2025)
**HuggingFace**: `klusai/ds-tf1-en-3m`

| Field | Value |
|-------|-------|
| **Full Name** | Three Million Synthetic Moral Fables |
| **Task Type** | Story generation, moral reasoning, instruction following |
| **Data Scale** | 3,000,000 fables |
| **Generator** | Instruction-tuned models ≤8B params |
| **Cost** | ~$0.135 per 1,000 fables |
| **Hardware** | Single consumer GPU (<24GB VRAM) |
| **Generator** | Llama-3 8B variant |

### Narrative Structure (6-slot scaffold)

```
Character → Trait → Setting → Conflict → Resolution → Moral
```

### Quality Metrics

- Grammar score (GPT-based critic)
- Creativity score
- Moral clarity score
- Template adherence score
- Diversity metrics (reference-free)
- Readability metrics

### Also Available

- `klusai/ds-tf1-en-100k` — 100K subset
- `klusai/tinyfabulist` — Generation tool (YAML config, 100+ characters, 50+ traits, 50+ settings)

---

## 8. Tell Me A Story (Google DeepMind)

**GitHub**: `google-deepmind/tell_me_a_story`

| Field | Value |
|-------|-------|
| **Task Type** | Complex narrative generation |
| **Framework** | Agents' Room (multi-agent decomposition) |
| **Format** | `{example_id, inputs, targets}` (JSONL) |
| **Splits** | Train / Validation / Test |

### Key Insight

Decomposes narrative writing into specialized agents: plot crafting, character development, language use — then collaborative synthesis.

---

## 9. Tale-Frame (Structured Story Dataset)

**HuggingFace**: `guodaosun/tale-frame`

| Field | Value |
|-------|-------|
| **Task Type** | Controllable story generation |
| **Base Dataset** | TinyStories |
| **Annotations** | Entity/event/structure JSON + GPT-4 chosen/rejected |
| **Structure** | beginning → middle → climax → ending |

---

## Summary: Fable Dataset Taxonomy

| Category | Dataset | Scale | Primary Use |
|----------|---------|-------|-------------|
| **Data-Flow Reasoning** | FABLE | 2,400 | Procedural reasoning eval |
| **Data-Flow Reasoning** | FABLE+ | 3,200 | Extended procedural reasoning eval |
| **Agent Workflow** | Fable-Method | 50 | Agent compliance evaluation |
| **Moral Reasoning** | MORABLES | 709 | Abstract moral inference |
| **Narrative Comprehension** | BIG-bench fables | 189 | Story understanding |
| **Synthetic Fable Generation** | TF1-EN-3M | 3M | Story gen / moral reasoning / instruction following |
| **Narrative Generation** | Tell Me A Story | — | Multi-agent narrative writing |
| **Reasoning Distillation** | Claude Fable 5 variants | 462–25K | SFT / process supervision / long-context reasoning |
| **Structured Stories** | Tale-Frame | — | Controllable story generation |

---

## NeoTrix Relevance Assessment

### Direct Applicability

| NeoTrix Component | Fable Dataset Relevance | Priority |
|---|---|---|
| **SEAL Pipeline** | FABLE/FABLE+ data-flow analysis maps directly to SEAL's procedural understanding stages (distillation → self-test → absorption) | P1 |
| **ConsciousnessTree** | MORABLES moral reasoning maps to ConsciousnessTree's moral/ethical evaluation branches | P2 |
| **NT-MIND** | Claude Fable 5 distillation data ideal for NT-MIND's skill crystallization (long-context reasoning traces) | P1 |
| **GWT** | FABLE's data-flow tracking parallels GWT's attention routing across specialist modules | P2 |
| **NT-MEMORY** | TF1-EN-3M's structured fables as test cases for KB narrative storage/retrieval | P2 |
| **NT-ACT** | Fable-Method's Think/Act/Prove framework maps to NT-ACT's tool orchestration evaluation | P1 |

### Specific Integration Opportunities

1. **Procedural Reasoning Benchmark**: FABLE/FABLE+ can evaluate SEAL pipeline's ability to track state changes across evolution stages
2. **Reasoning Distillation**: Claude Fable 5 traces provide training signal for NT-MIND's distillation module
3. **Agent Evaluation**: Fable-Method's metrics (Workflow Compliance, Reasoning Fidelity, Tool Use Accuracy) align with NT-ACT's self-test dimensions
4. **Moral Reasoning**: MORABLES and TF1-EN-3M provide test cases for NT-FEEL's emotional/moral evaluation capabilities
5. **Knowledge Representation**: TF1-EN-3M's 6-slot scaffold (character→trait→setting→conflict→resolution→moral) mirrors VSA HyperCube's structured concept embedding

### Recommended Next Steps

- [ ] Evaluate FABLE+ on NT-CORE's reasoning module to establish procedural reasoning baseline
- [ ] Use Claude Fable 5 distillation traces to train NT-MIND's long-context reasoning
- [ ] Implement Fable-Method's Think/Act/Prove evaluation framework for NT-ACT tool orchestration
- [ ] Test MORABLES moral inference to calibrate NT-FEEL's ethical reasoning capabilities

---

*End of research compilation. 9 datasets analyzed, NeoTrix integration points identified.*

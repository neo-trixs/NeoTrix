# Model Reverse Engineering — Cycle 393 (2026-09-12)

## Papers & Models Analyzed

### 1. PARSER: Read in Parallel, Reason in Depth
**arXiv:2609.06702** | Sep 2026

**Core Innovation**: Decouples reading from reasoning in long-context agents. A bank of lightweight subagents each bound to a single chunk reads the entire document in parallel, while a lead agent reasons in depth through iterative scatter-gather rounds.

**Key Mechanism**:
- Subagents read chunks in parallel (frozen off-the-shelf models)
- Lead agent broadcasts queries, aggregates evidence, formulates deeper follow-ups
- All learnable behavior concentrated in lead agent (RL-optimized)
- Subagents remain frozen — no training needed

**Results**: 4B backbone outperforms strongest sequential baseline by +5.7pts avg, +12.0pts at 896K tokens. 9B surpasses DeepSeek-V4-Pro by +6.3pts. 11x latency reduction.

**NeoTrix Domain Mapping**:

| Domain | Integration Pattern |
|--------|-------------------|
| NT-WORLD | Parallel chunk reading for crawl pipeline — apply scatter-gather to document processing |
| NT-MEMORY | Decouple KB embedding read from reasoning — subagents for retrieval, lead for synthesis |
| NT-CORE | GWT attention routing could adopt scatter-gather: broadcast salience queries to specialists |
| NT-MIND | SEAL distillation could use parallel subagent evidence gathering before self-reflection |

**Absorption Candidate**: R-P42 — extend existing NT-WORLD parallel read paths, not new module.

---

### 2. Declarative Attention (DA)
**arXiv:2609.02737** | Sep 2026

**Core Innovation**: Model declares where it needs to attend within chain-of-thought. Partitions generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). Inference engine parses declarations like tool calls and skips KV cache reads.

**Key Mechanism**:
- Chain-of-thought emits declarative tokens: `<global>`, `<focus>`, `<local>`
- Engine parses these as attention-skip instructions
- Off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B) do this zero-shot
- 52% reduction in attended tokens with 1.27pp accuracy drop (shrinks with scale)

**NeoTrix Domain Mapping**:

| Domain | Integration Pattern |
|--------|-------------------|
| NT-CORE | GWT could emit declarative attention directives — E8 hexagram reasoning declares focus regions |
| NT-MEMORY | KB search could skip irrelevant namespaces based on DA declarations |
| NT-IO | KV cache optimizer could consume DA declarations for selective cache retention |
| NT-MIND | SEAL self-reflection could declare `<local>` for focused introspection vs `<global>` for cross-domain |

**Absorption Candidate**: Direct extension of kv_cache_optimizer.rs — add declarative attention parsing.

---

### 3. Gated-Memory Routing
**arXiv:2609.00237** | Aug 2026 (EMNLP 2026)

**Core Innovation**: Conditions each multi-agent decision on query + learned execution memory. Memory Write Gate commits only non-redundant reasoning steps. Retrieval Gate supplies compact relevant subset. Adaptive Halting Controller stops when memory sufficient.

**Key Mechanism**:
- Write Gate: filters redundant reasoning before memory commit
- Retrieval Gate: compact, relevant context per decision
- Adaptive Halting: stops execution when evidence sufficient
- Best accuracy (+2.44pts) with 31.9% cost reduction vs strongest baseline

**NeoTrix Domain Mapping**:

| Domain | Integration Pattern |
|--------|-------------------|
| NT-MEMORY | Direct map to experience-tree KB write path — Write Gate filters redundancy before absorption |
| NT-CORE | GWT attention could use Retrieval Gate pattern for compact context injection |
| NT-MIND | SEAL cycle could use Adaptive Halting for early termination when convergence detected |
| NT-ACT | Multi-agent orchestration (NT-ACT) could gate tool-call decisions on execution memory |

**Absorption Candidate**: R-P79 — apply gating pattern to experience-tree absorption pipeline, not new module.

---

### 4. Codebook Agent: Amortized Topology Design
**arXiv:2609.02264** | Sep 2026

**Core Innovation**: Vector-quantized autoencoder compresses successful topologies into 16-entry codebook. Reward-weighted MLP maps query to code distribution. MLP proxy reranks candidates in single forward pass. No iterative search at test time.

**Key Results**: 84.6 avg accuracy (vs 83.0 strongest prior), 2.4ms topology emission, 21.9-33.2% fewer LLM tokens.

**Key Insight**: Topologies that survive reward filter collapse to ~6 distinct graphs even with 64-entry codebook capacity. Edge count is *negatively* correlated with token consumption (Pearson r ≈ -0.4).

**NeoTrix Domain Mapping**:

| Domain | Integration Pattern |
|--------|-------------------|
| NT-CORE | E8 hexagram topology could be compressed to codebook — 6-8 archetypal communication graphs |
| NT-MIND | SEAL pipeline could learn topology codebooks from successful evolution cycles |
| NT-ACT | Multi-agent orchestration could select topologies from codebook rather than search |
| NT-MEMORY | KB graph queries could use amortized topology for efficient traversal |

**Absorption Candidate**: New capability within NT-CORE for topology learning — but must prove production value first.

---

### 5. Speculative Macro Commit (SMC)
**arXiv:2609.03236** | Sep 2026 (MLSP 2026)

**Core Innovation**: Two-tier agent system — large authoritative actor + fast speculative drafter. Drafter predicts and executes future action chains on isolated environment snapshot. Macro library mines recurring multi-action skeletons from training traces.

**Key Mechanism**:
- Macro library: recurring multi-action patterns extracted offline
- Speculative drafter (4B model) predicts action chains ahead of actor (27B)
- When actor matches first drafted action → commit remaining pre-executed steps
- 10.23% latency reduction over Speculative Actions baseline, 18.59% over sequential

**NeoTrix Domain Mapping**:

| Domain | Integration Pattern |
|--------|-------------------|
| NT-ACT | Tool-call chains could be speculatively executed — macro library of common NT-ACT patterns |
| NT-CORE | E8 reasoning could speculatively explore next reasoning branches |
| NT-MIND | SEAL pipeline could macro-commit successful phase transitions |
| NT-IO | CLI command chains could be speculatively pre-executed |

**Absorption Candidate**: R-P42 — extend existing tool-call paths with speculative pre-execution.

---

## Cross-Paper Pattern Synthesis

### Pattern A: Parallel-Read + Sequential-Reason
PARSER + Gated-Memory both confirm: **parallel read, sequential reason** is the winning architecture. NeoTrix should adopt this for NT-WORLD crawl + NT-CORE reasoning separation.

### Pattern B: Declarative Attention as Interface
DA shows models can self-declare attention regions. Combined with NeoTrix's GWT, this enables **consciousness-directed attention skipping** — the E8 hexagram system could emit focus directives.

### Pattern C: Codebook Compression of Behavioral Topologies
Codebook Agent's key insight (topologies collapse to ~6 archetypes) maps directly to NeoTrix's Skill Tree — skill nodes could be represented as topology codebook entries rather than full graphs.

### Pattern D: Speculative Execution as Latency Primitive
SMC's macro-commit pattern is broadly applicable: any multi-step tool chain can benefit from speculative pre-execution with a smaller drafter model.

### Pattern E: Gated Memory for Cost-Aware Context
Gated-Memory Routing's Write Gate + Retrieval Gate is the practical implementation of Axiom A2 (Context as Scarce Resource). Apply to experience-tree KB writes and GWT attention budget allocation.

## Axiom Alignment

| Paper | Axiom | Alignment |
|-------|-------|-----------|
| PARSER | A2 (Context as Scarce Resource) | Parallel reading reduces context bottleneck by 11x |
| Declarative Attention | A2 | Self-declared attention regions skip 52% of KV reads |
| Gated-Memory | A2 + A1 (Cost-Aware Routing) | Adaptive halting stops early, 31.9% cost reduction |
| Codebook Agent | A3 (Skill as Production Template) | Topology codebooks = compressed skill templates |
| SMC | A1 (Cost-Aware Routing) | Small drafter model speculatively pre-executes, large model only validates |

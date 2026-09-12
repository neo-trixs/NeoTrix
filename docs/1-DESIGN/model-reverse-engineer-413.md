# Model Reverse Engineer — Cycle 413

> **Date**: 2026-09-12
> **Sources**: arXiv (2609.xxxxx), ACL 2026, GitHub repos
> **Prior baseline**: cycles 318-412 (no duplicates)
> **Focus**: Efficient inference, attention mechanisms, agent coordination

---

## Paper 1: Declarative Attention (DA) — LLM Self-Directed Attention Routing

| Field | Value |
|-------|-------|
| **Paper** | [2609.02737](https://arxiv.org/abs/2609.02737) |
| **Date** | 2026-09-02 |
| **Tested on** | Gemma-4-31B, Qwen-3.6-27B |
| **Key metric** | 52% (Gemma) / 31% (Qwen) reduction in attended tokens; <3pp accuracy drop |

### Core Idea

The model **declares where it needs to attend** within its own chain-of-thought, partitioning generation into three modes:
- `<global>` — full context scan
- `<focus>` — specific region of the KV cache
- `<local>` — recent output only

The inference engine parses these declarations like tool calls and skips most KV cache reads.

### Architecture Analysis

| Component | Detail |
|-----------|--------|
| **Mechanism** | Model emits XML-like tags (`<global>`, `<focus>`, `<local>`) during CoT generation |
| **Parsing** | Inference engine intercepts tags as structured output, partitions KV access |
| **KV Savings** | 52% total attended tokens at zero-shot on off-the-shelf models |
| **Training** | Zero-shot — no fine-tuning required; works on existing models |
| **Scaling** | Accuracy gaps shrink with model scale |

### Why This Matters

This is **intrinsic** attention routing — the model itself decides what to read, not an external proxy scorer. Unlike external sparse attention methods (CEDAR, etc.), DA requires zero overhead scoring because the model's own generation process produces the routing signal.

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-CORE** (E8 引导者) | Map DA's 3-mode partition to GWT salience — `<global>` = broadcast, `<focus>` = selective attention, `<local>` = working memory | P0 |
| **NT-MEMORY** (知识守护者) | KV cache tiering: hot (focus) → warm (global) → cold (local-only), like KVMem's paged KV | P1 |
| **NT-IO** (界面使徒) | Implement DA tag parsing in inference engine; route parsed declarations to KV cache controller | P1 |
| **NT-MIND** (进化工匠) | Self-evolution of attention patterns — train DA tags for specific task domains (code, reasoning, creative) | P2 |

### NeoTrix Integration Point

```
// NT-CORE: GWT salience integration
enum AttentionMode {
    Global,   // full KV scan — high salience broadcast
    Focus,    // region-specific — targeted attention
    Local,    // recent output only — working memory
}

// DA tags parsed by NT-IO → routed to NT-MEMORY KV controller
impl DeclarativeAttention for GwtRouter {
    fn route(&self, tag: AttentionMode, kv_cache: &KvCache) -> AttentionWindow {
        match tag {
            Global => kv_cache.full_scan(),
            Focus => kv_cache.region_scan(tag.region_bounds()),
            Local => kv_cache.recent_window(tag.window_size()),
        }
    }
}
```

---

## Paper 2: CEDAR — Error-Bounded Residual Routing for Sparse Attention

| Field | Value |
|-------|-------|
| **Paper** | [2609.07237](https://arxiv.org/abs/2609.07237) |
| **Date** | 2026-09-07 |
| **Key metric** | ~3× kernel speedup at 128K context; 98%+ error reduction vs hard dropping |

### Core Idea

Coarse-to-fine Error-aware Dynamic Attention Routing (CEDAR) keeps the LM frozen while preserving global coverage:
1. Each semantic chunk contributes a **cheap KV summary** (residual attention path)
2. Chunks with high **estimated approximation error** are expanded to exact token attention
3. Exact + summarized contributions combined in **single softmax normalization** (refinement, not duplication)

### Architecture Analysis

| Component | Detail |
|-----------|--------|
| **Coarse Pass** | Semantic chunk-level KV summaries via lightweight projection |
| **Error Estimation** | Per-chunk approximation error bound derived from within-chunk key/value dispersion |
| **Refinement** | Variable-budget expansion: easy chunks stay summarized, ambiguous ones get exact attention |
| **Normalization** | Single softmax over exact + summarized — refinement replaces coarse evidence |
| **Frozen Model** | No fine-tuning required; post-hoc sparse attention |

### Key Insight

CEDAR solves the **hard selection problem**: existing sparse attention methods assign zero probability to omitted chunks (routing miss = permanent loss). CEDAR's residual path ensures every chunk contributes at least a summary, and refinement is allocated where it matters most.

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-MEMORY** (知识守护者) | CEDAR error-bounded summaries for KB retrieval — summarize indexed chunks, expand on error | P0 |
| **NT-CORE** (E8 引导者) | GWT attention budget: allocate refinement budget to high-error chunks (salience × error) | P1 |
| **NT-WORLD** (虚空探索者) | Crawl pipeline: summarize web pages at chunk level, refine for relevant sections | P2 |
| **NT-MIND** (进化工匠) | Error estimation as self-test signal — high error = needs more training data | P2 |

### NeoTrix Integration Point

```
// NT-MEMORY: Error-bounded KB retrieval
struct CedarRetriever {
    chunk_summarizer: ChunkSummarizer,  // coarse KV projection
    error_estimator: ErrorEstimator,     // per-chunk dispersion metric
    refinement_budget: usize,            // variable allocation
}

impl CedarRetriever {
    fn retrieve(&self, query: &[f32], kb: &KnowledgeBase) -> Vec<ChunkResult> {
        let summaries = kb.chunks().map(|c| self.chunk_summarizer.summarize(c));
        let errors = summaries.iter().map(|s| self.error_estimator.estimate(s));
        let budget = self.refinement_budget;
        // Allocate more budget to high-error chunks
        let refined = allocate_refinement(budget, errors);
        // Single softmax over exact + summarized
        combine_and_normalize(refined, summaries)
    }
}
```

---

## Paper 3: CondenseFlow — Scalable Latent Collaboration via Semantic Compression

| Field | Value |
|-------|-------|
| **Paper** | [ACL 2026 Findings](https://aclanthology.org/2026.findings-acl.669.pdf) |
| **Date** | 2026 (ACL Findings) |
| **Key metric** | >99% KV cache memory reduction; ~20% latency reduction; <2% accuracy degradation |

### Core Idea

Full-state latent communication in multi-agent LLM systems scales linearly with collaboration rounds. CondenseFlow introduces the **Latent Thought Condenser (LTC)** — a lightweight module using learnable semantic probes to compress KV caches into **fixed-size representations** (dimension K=64), achieving O(1) communication complexity.

### Architecture Analysis

| Component | Detail |
|-----------|--------|
| **LTC Module** | Cross-attention aggregation: learnable probes detect critical info along specific dimensions |
| **Compression** | Variable-length KV cache → fixed-size K=64 vector (99%+ memory reduction) |
| **Error Bound** | Compression error bounded by attention concentration (1 - ρ, where ρ = top-K attention mass) |
| **Multi-Round** | Cumulative error grows at most linearly (R·δ), empirically sub-linear |
| **Communication** | O(1) complexity regardless of context length or interaction rounds |

### Key Insight

Unlike heuristic pruning (H2O, SnapKV), LTC **discovers** information patterns through end-to-end learning. It's not just selecting which tokens to keep — it's learning what to compress and what to preserve for downstream reasoning across agents.

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-MEMORY** (知识守护者) | LTC-style semantic compression for inter-domain knowledge sharing | P0 |
| **NT-ACT** (行动执行者) | Fixed-size agent state transfer — compress agent context for multi-agent coordination | P0 |
| **NT-CORE** (E8 引导者) | Attention-concentration metric as GWT salience signal | P1 |
| **NT-MIND** (进化工匠) | End-to-end learned compression vs heuristic — evolution of compression strategy | P2 |

### NeoTrix Integration Point

```
// NT-ACT: Fixed-size agent state transfer
struct LatentThoughtCondenser {
    probes: Vec<SemanticProbe>,  // learnable attention probes
    compression_dim: usize,      // K=64 default
}

impl LatentThoughtCondenser {
    fn compress(&self, kv_cache: &KvCache) -> CompressedState {
        let critical = self.probes.iter()
            .map(|p| p.detect_critical_info(kv_cache))
            .collect();
        CompressedState::from_probes(critical, self.compression_dim)
    }

    fn decompress(&self, state: &CompressedState, target_agent: &Agent) -> KvPatch {
        // Agent receives fixed-size state, expands for local use
        target_agent.expand_from_compressed(state)
    }
}
```

---

## Paper 4: ReActNet — Inference-Time Graph Engineering for Multi-Agent Workflows

| Field | Value |
|-------|-------|
| **Paper** | [2609.05774](https://arxiv.org/abs/2609.05774) |
| **Date** | 2026-09-04 |
| **Key metric** | Consistently improves over fixed-topology and learned-topology baselines; competitive inference cost |

### Core Idea

Rather than optimizing a static topology, ReActNet **synthesizes a task-conditioned temporal workflow graph** that jointly specifies agent connectivity and edge-level communication semantics. Each graph snapshot = one reasoning stage; each edge = natural-language instruction for message passing.

### Architecture Analysis

| Component | Detail |
|-----------|--------|
| **Graph Compilation** | Query + role-specialized agents → sequence of directed communication graphs |
| **Edge Semantics** | Each edge carries NL instruction: "source agent should provide X to target agent" |
| **Execution** | Structured message passing: agents update states from controller-assigned neighbors |
| **Aggregation** | Final aggregator synthesizes all agent states into answer |
| **Training-Free** | No RL or gradient-based topology optimization needed |

### Key Insight

Multi-agent coordination depends not only on **which** agents communicate, but on engineering executable workflow graphs that encode **when, why, and how** information should flow during reasoning. The graph is task-conditioned — different tasks produce different temporal topologies.

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-ACT** (行动执行者) | Dynamic agent topology per task — compile task → agent graph → execute | P0 |
| **NT-CORE** (E8 引导者) | E8 hexagram as temporal communication graph — reasoning stage → graph snapshot | P1 |
| **NT-MIND** (进化工匠) | Learn reusable graph patterns from execution traces | P2 |
| **NT-IO** (界面使徒) | Inspectable agent coordination — human-readable edge instructions | P1 |

### NeoTrix Integration Point

```
// NT-ACT: Task-conditioned agent topology
struct ReactNetCompiler {
    agents: Vec<RoleSpecializedAgent>,
    stage_encoder: StageEncoder,
}

impl ReactNetCompiler {
    fn compile(&self, query: &str) -> TemporalGraph {
        let stages = self.stage_encoder.encode(query);
        stages.into_iter().map(|stage| {
            GraphSnapshot {
                nodes: self.agents.clone(),
                edges: self.generate_edges(stage, query),
                // Each edge: "Agent A sends [summary of X] to Agent B"
            }
        }).collect()
    }
}

// Execute: agents update from assigned neighbors at each stage
fn execute(graph: TemporalGraph, agents: &mut Vec<Agent>) -> Answer {
    for snapshot in graph.stages() {
        for edge in &snapshot.edges {
            let message = agents[edge.source].produce(edge.instruction);
            agents[edge.target].integrate(message);
        }
    }
    graph.aggregate(agents)
}
```

---

## Paper 5: Focus — Learnable Centroid Attention Routing

| Field | Value |
|-------|-------|
| **Paper** | [2604.03260](https://arxiv.org/abs/2604.03260) (March 2026, updated) |
| **Date** | 2026-03-12 |
| **Key metric** | 2× speedup with better quality than full attention (41.3 vs 42.8 PPL); 8.6× wall-clock at 1M tokens |

### Core Idea

Learnable centroids assign tokens to groups; distant attention restricted to same-group pairs while local attention at full resolution. All model weights stay frozen — purely additive. Centroid-only training (148K params) improves domain perplexity with zero degradation.

### Architecture Analysis

| Component | Detail |
|-----------|--------|
| **Centroids** | Learnable group assignments (16-dimensional routing) |
| **Routing** | Soft-gated training → hard top-k inference (k=2 of K=4 groups) |
| **Efficiency** | Different-group distant pairs never computed (dot product eliminated, not zeroed) |
| **Retrofit** | Works on pretrained models without fine-tuning (GPT-2, Mistral, LLaMA, Gemma, Qwen) |
| **Alignment** | Preserves TruthfulQA scores after adaptation (LoRA degrades) |

### Key Insight

Focus doesn't just approximate full attention — it **surpasses** it. Removing irrelevant attention pairs is not a cost; it's a benefit. The model performs better when it attends to fewer, more relevant tokens. The 16-dimensional routing is sufficient to capture selection.

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|-------------|----------|
| **NT-CORE** (E8 引导者) | Centroid routing as GWT attention filter — 16-dim routing = attention budget | P0 |
| **NT-MEMORY** (知识守护者) | Group-based KB indexing — centroids for chunk grouping, same-group retrieval | P1 |
| **NT-PHYSICAL** (具身骨架) | Retrofit efficient attention on local inference (TurboQuant llama.cpp) | P1 |
| **NT-MIND** (进化工匠) | Centroid learning as meta-cognition — which token groups matter for which tasks | P2 |

### NeoTrix Integration Point

```
// NT-CORE: Centroid-based attention routing
struct FocusRouter {
    centroids: Vec<Centroid>,  // learnable 16-dim group assignments
    k: usize,                  // top-k groups per token (default: 2)
}

impl FocusRouter {
    fn route(&self, token: &Token, all_tokens: &[Token]) -> Vec<Token> {
        let groups = self.centroids.iter()
            .enumerate()
            .sorted_by(|(_, c)| c.similarity(token))
            .take(self.k)
            .map(|(i, _)| i)
            .collect();

        all_tokens.iter()
            .filter(|t| self.shares_group(t, &groups))
            .collect()
    }

    fn shares_group(&self, token: &Token, my_groups: &[usize]) -> bool {
        self.centroids.iter().enumerate()
            .any(|(i, c)| my_groups.contains(&i) && c.assigns(token))
    }
}
```

---

## Cross-Paper Synthesis: 5 Meta-Patterns

| # | Pattern | Papers | NeoTrix Mapping |
|---|---------|--------|-----------------|
| 1 | **Intrinsic vs Extrinsic Attention** | DA (intrinsic) vs CEDAR/Focus (extrinsic) | NT-CORE: hybrid — model-declared + proxy-scored attention |
| 2 | **Error-Bounded Compression** | CEDAR (chunk error) + CondenseFlow (KV error bound) | NT-MEMORY: compression with provable quality bounds |
| 3 | **Task-Conditioned Topology** | ReActNet (graph per query) + Codebook Agent (topology codebook) | NT-ACT: compile task → optimal agent graph |
| 4 | **Frozen Model + Additive Routing** | Focus (centroid routing) + CEDAR (residual paths) | NT-PHYSICAL: retrofit efficiency without retraining |
| 5 | **O(1) Communication Complexity** | CondenseFlow (fixed-size state) + DA (parsed declarations) | NT-ACT: bounded inter-agent state transfer |

---

## NeoTrix Integration Roadmap

| Phase | Pattern | Domain | Implementation |
|-------|---------|--------|----------------|
| **P0** | DA attention routing | NT-CORE | GWT salience with 3-mode partition |
| **P0** | CondenseFlow LTC | NT-MEMORY | Semantic compression for KB sharing |
| **P1** | CEDAR error-bounded | NT-MEMORY | Error-aware chunk retrieval |
| **P1** | Focus centroid routing | NT-CORE | 16-dim attention filter |
| **P1** | ReActNet topology | NT-ACT | Task-conditioned agent graphs |
| **P2** | Cross-pattern synthesis | NT-MIND | Meta-learning of routing strategies |

---

*Generated: 2026-09-12 | Cycle 413 | 5 papers | 5 meta-patterns | 20 domain integration points*

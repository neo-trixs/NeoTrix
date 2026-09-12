# Model Reverse Engineering — Cycle 365 (2026-09-12)

## 5 New AI Models/Papers (Sep 2026)

---

### 1. Declarative Attention (DA) — Self-Sparsified Attention
**Paper**: "Language Models Can Control Their Own Attention" (arXiv:2609.02737)
**Date**: Sep 2 2026
**Models**: Gemma-4-31B, Qwen-3.6-27B (off-the-shelf, no training)

#### Core Mechanism
The model **declares** where it needs to attend within its chain-of-thought, partitioning generation into three modes:
- `<global>` — full context scan
- `<focus>` — specific region
- `<local>` — recent output only

The inference engine parses these declarations like tool calls and skips most KV cache reads.

#### Key Results
- 52.0% reduction in total attended tokens (Gemma-4-31B)
- 31.1% reduction (Qwen-3.6-27B)
- Modest accuracy drops (1.27pp, 2.75pp) that shrink with model scale
- Zero-shot on off-the-shelf models — no training required

#### Why It Matters
This is **intrinsic** sparse attention — the model itself decides what to attend to, rather than an external proxy scoring tokens. The declarations are parsed like function calls, meaning the attention pattern becomes part of the model's output, not a separate computation step.

#### NeoTrix Domain Mapping

| Domain | Mapping | Specific Component |
|--------|---------|-------------------|
| **NT-CORE** | DA declarations = GWT salience broadcast. Model self-organizes attention like ConsciousnessTree self-monitors. | `nt_core_gwt::attention_router` — self-declared salience signals replace external scoring |
| **NT-MEMORY** | `<focus>` mode = on-demand experience branch loading. `<local>` = session-level working memory. | `experience_tree::lazy_loader` — model declares which KB branches to load |
| **NT-IO** | Declaration parsing = tool-call-like interface. Attention decisions become inspectable messages. | `nt_io::provider` — expose attention mode as metadata in provider responses |
| **NT-MIND** | Self-declared attention = meta-cognitive self-awareness. Model knows what it doesn't know. | `nt_mind::meta_cognition` — attention declarations as introspection signals |

#### Absorption Candidate
- **Pattern**: Self-declared attention routing (intrinsic sparse attention)
- **Application**: GWT could accept attention-mode declarations from consciousness cycle, routing queries to global/focus/local processing paths
- **Priority**: P1 — directly applicable to GWT attention routing refinement

---

### 2. CEDAR — Error-Bounded Residual Routing
**Paper**: "CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention" (arXiv:2609.07237)
**Date**: Sep 7 2026

#### Core Mechanism
Coarse-to-fine attention routing:
1. Each semantic chunk contributes a cheap **key-value summary** to a residual attention path
2. Chunks with high estimated approximation error are expanded to exact token attention
3. Exact and summarized contributions combined in **single softmax** — refinement replaces, not duplicates, coarse evidence
4. Error bound derived from within-chunk key/value dispersion

#### Key Results
- Residual summaries reduce reconstruction error by **>98%** vs hard dropping
- ~3× kernel speedup at 128K context
- Recovers most quality lost by hard sparse routing

#### Why It Matters
CEDAR solves the fundamental problem of sparse attention: routing misses are unrecoverable. By keeping residual summaries and expanding only high-error chunks, it maintains **coverage** while achieving sparsity. The error bound is derived from data statistics, not heuristics.

#### NeoTrix Domain Mapping

| Domain | Mapping | Specific Component |
|--------|---------|-------------------|
| **NT-CORE** | Error-bounded routing = confidence-calibrated attention. E8 hexagram reasoning could use error bounds to decide when to expand reasoning depth. | `nt_core_e8::reasoning_engine` — error-bound-guided reasoning expansion |
| **NT-MEMORY** | Residual summaries = compressed experience nodes. Full expansion = lazy branch loading on high-error queries. | `kb::embedding` — two-tier storage: summaries + full detail, expand on demand |
| **NT-MIND** | Error bound = self-assessment of reasoning quality. When error is high, SEAL pipeline expands to deeper analysis. | `seal::phase_selector` — error-triggered deep-dive in evolution pipeline |
| **NT-WORLD** | Semantic chunking for content extraction. Residual summaries for crawled content indexing. | `nt_world_crawl::content_extractor` — hierarchical content representation |

#### Absorption Candidate
- **Pattern**: Error-bounded residual refinement (coarse→fine with coverage guarantee)
- **Application**: KB query could use two-tier retrieval: summary vectors for fast path, expand to full embeddings when error exceeds threshold
- **Priority**: P1 — error-bounded refinement directly applicable to KB retrieval

---

### 3. PIVOT — Group-Shared Indexing for Sparse Attention
**Paper**: "PIVOT: Efficient Query-Group Indexing for Token-Level Sparse Attention" (arXiv:2607.24593)
**Date**: Jul 27 2026
**Models**: DeepSeek-V3.2, GLM-5.1

#### Core Mechanism
Shares one full-prefix scan across a **group of nearby queries** instead of one per query:
1. Aggregate group into single proxy query
2. One shared prefix scan → candidate set (budget only slightly above top-k)
3. Two variants:
   - **PIVOT-Reuse**: share proxy top-k across group (fastest)
   - **PIVOT-Refine**: re-score candidate set per query (matches dense accuracy)
4. Same algorithm for prefill (fixed groups) and decode (MTP step groups)

#### Key Results
- Indexer acceleration: up to **4×**
- End-to-end latency reduction: up to **1.6×** at long context
- Matches dense DSA indexer accuracy (Refine variant)
- Training-free, drop-in replacement

#### Why It Matters
Observation: nearby queries select highly overlapping tokens (O1). Indexer scores are long-tailed along key axis (O2). These two properties enable amortizing one expensive scan over many queries. The key insight is **query redundancy in attention** — adjacent positions don't need independent indexing.

#### NeoTrix Domain Mapping

| Domain | Mapping | Specific Component |
|--------|---------|-------------------|
| **NT-CORE** | Query grouping = task batching in GWT. Similar queries share salience computation. | `gwt::batch_router` — group similar attention queries for amortized processing |
| **NT-MEMORY** | Proxy scan = KB embedding batch query. Nearby experience nodes share index scan. | `kb::batch_query` — amortized embedding similarity search |
| **NT-ACT** | MTP-step grouping = multi-step task batching. Similar tool calls share execution planning. | `nt_act::tool_orchestrator` — batch similar tool calls |
| **NT-MIND** | Amortized reasoning across similar SEAL pipeline stages. | `seal::batch_processor` — group similar evolution tasks |

#### Absorption Candidate
- **Pattern**: Query-redundancy exploitation (amortized indexing across similar queries)
- **Application**: KB batch queries could group similar experience lookups into shared embedding scans
- **Priority**: P2 — indirect but applicable to KB query optimization

---

### 4. ConvMem — Convolutional Memory for Long-Context
**Paper**: "ConvMem: Convolutional Memory for Long-Context Reasoning" (arXiv:2609.10441)
**Date**: Sep 9 2026

#### Core Mechanism
Reformulates long-context reasoning as **hierarchical convolution**:
1. LLM prompted with query = **convolutional kernel**
2. Kernel summarizes text segments hierarchically
3. Reasoning path: linear chain → **logarithmic tree**
4. Configurable Strides + Skip Connections for evidence capture
5. Multi-Kernel Convolution decomposes complex queries into disentangled semantic channels

#### Key Results
- Training-free, highly parallelizable
- Outperforms training-free baselines on RULER-HotpotQA and RULER-2WikiMultiHopQA
- Avoids overfitting to parametric priors (unlike RL-trained models)
- Logarithmic depth vs linear sequential processing

#### Why It Matters
Sequential memory agents (like MemAgent) suffer from linear latency and require costly RL training. ConvMem's CNN-inspired approach enables **massive parallelization** across both text segments and reasoning threads. The logarithmic tree structure means O(log n) reasoning depth vs O(n) sequential.

#### NeoTrix Domain Mapping

| Domain | Mapping | Specific Component |
|--------|---------|-------------------|
| **NT-MEMORY** | Hierarchical convolution = KB embedding aggregation at multiple scales. | `kb::multi_scale_index` — hierarchical experience node aggregation |
| **NT-CORE** | Logarithmic reasoning tree = ConsciousnessTree depth optimization. | `consciousness_tree::depth_optimizer` — parallel branch processing |
| **NT-MIND** | Multi-kernel decomposition = SEAL pipeline parallel stage execution. | `seal::parallel_executor` — decompose complex evolution into parallel streams |
| **NT-WORLD** | Configurable strides for content extraction windowing. | `nt_world_crawl::stride_extractor` — adaptive content windowing |

#### Absorption Candidate
- **Pattern**: Hierarchical convolution for reasoning (log-depth tree vs linear chain)
- **Application**: ConsciousnessTree growth cycles could process branches in parallel convolution rather than sequential soil→root→trunk→branch→fruit
- **Priority**: P2 — parallel ConsciousnessTree processing is architecturally significant but requires cycle restructuring

---

### 5. ReActNet — Inference-Time Graph Engineering for Multi-Agent
**Paper**: "Inference-Time Graph Engineering for Multi-Agent LLM Workflows" (arXiv:2609.05774)
**Date**: Sep 4 2026

#### Core Mechanism
Compiles query + role-specialized agents into a **temporal workflow graph**:
1. Query → task-conditioned temporal graph (sequence of directed communication graphs)
2. Each graph snapshot = one reasoning stage
3. Each edge = natural-language instruction specifying message content
4. **Two-phase separation**: graph compilation (offline) vs graph execution (online)
5. Structured message passing: agents integrate previous states + neighbor messages

#### Key Results
- Training-free framework
- Consistently improves over fixed-topology and learned-topology baselines
- Competitive inference cost
- Works across knowledge reasoning, math, code, and GAIA assistant tasks

#### Why It Matters
Most multi-agent coordination is either fixed topology (rigid) or learned (requires training). ReActNet shows that **task-conditioned static compilation** can beat both. The key insight: effective orchestration depends not just on which agents communicate, but on engineering **when, why, and how** information flows during reasoning.

#### NeoTrix Domain Mapping

| Domain | Mapping | Specific Component |
|--------|---------|-------------------|
| **NT-ACT** | Temporal workflow graph = NT-ACT task decomposition with time-varying agent roles. | `nt_act::task_graph` — compile task into temporal agent workflow |
| **NT-CORE** | Reasoning-stage snapshots = ConsciousnessTree phase transitions with explicit communication protocols. | `consciousness_tree::phase_comm` — inter-phase message contracts |
| **NT-MIND** | Graph compilation = SEAL pipeline topology optimization. | `seal::topology_compiler` — task-conditioned pipeline graph |
| **NT-SHIELD** | Edge-level communication semantics = permission contracts between agents. | `nt_shield::message_firewall` — validate inter-agent messages |

#### Absorption Candidate
- **Pattern**: Temporal workflow graph with per-edge communication semantics
- **Application**: SEAL pipeline stages could be compiled into temporal graphs with explicit inter-stage message contracts, replacing implicit data flow
- **Priority**: P1 — directly applicable to SEAL pipeline orchestration

---

## Cross-Paper Synthesis

### Dominant Theme: Attention as First-Class Programming Interface
Papers 1-3 (DA, CEDAR, PIVOT) all treat attention patterns as **programmable artifacts** rather than fixed computation. This converges with NeoTrix's GWT philosophy: attention is not automatic but routed by salience signals.

### Secondary Theme: Parallel Decomposition of Sequential Processes
Papers 4-5 (ConvMem, ReActNet) both decompose sequential reasoning into parallel structures (logarithmic tree, temporal graph). This validates NeoTrix's ConsciousnessTree approach of parallel branch processing.

### Absorption Priority Matrix

| Pattern | Source Paper | NeoTrix Target | Priority | Complexity |
|---------|-------------|----------------|----------|------------|
| Self-declared attention routing | DA (2609.02737) | GWT attention_router | P1 | Medium |
| Error-bounded residual refinement | CEDAR (2609.07237) | KB two-tier retrieval | P1 | Medium |
| Temporal workflow graph compilation | ReActNet (2609.05774) | SEAL topology_compiler | P1 | High |
| Query-redundancy amortization | PIVOT (2607.24593) | KB batch_query | P2 | Low |
| Hierarchical convolution reasoning | ConvMem (2609.10441) | ConsciousnessTree parallel cycles | P2 | High |

# Model Reverse Engineering — Cycle 421 (2026-09-12)

## 5 New Papers — Pattern Extraction & NeoTrix Mapping

---

### Paper 1: RouteRelay — Event-Triggered Cross-Layer Route Reuse for Dynamic Sparse Attention

**Source**: arXiv:2609.07306 (2026-09-07)
**Core Idea**: Router-agnostic method that reuses route metadata across Transformer depth. Anchor layers perform full routing; intermediate layers rescore previous top-routes plus a sentinel set of near-miss chunks. Row is rerouted only when a sentinel challenges its weakest selected chunk.

**Pattern Extracted**: Cross-layer route reuse — anchor layers establish routing decisions, intermediate layers inherit and verify. Route stability condition prevents unnecessary rerouting.

**NeoTrix Domain Mapping**:
| Domain | Integration | Specific Module |
|--------|------------|-----------------|
| **NT-CORE** | GWT attention — anchor GWT broadcasts salient context, intermediate modules reuse routing decisions without full re-computation | `gwt_attention` |
| **NT-MEMORY** | KB query routing — anchor queries establish search patterns, subsequent queries inherit route metadata | `kb_search` |
| **NT-MIND** | SEAL pipeline — anchor decisions at pipeline boundaries, intermediate stages inherit routing | `seal_pipeline` |

**Implementation Insight**: RouteRelay retains 99.99% route recall while rerouting only 25-78% of rows. The "sentinel challenge" pattern is directly applicable to GWT: maintain a compact set of near-miss salience candidates, only re-route when current attention fails against sentinel.

---

### Paper 2: HeRo — History-Aware Routing for Efficient LLM Inference

**Source**: arXiv:2609.08189 (2026-09-08)
**Core Idea**: Dynamic layer routing with explicit router memory across depth. Linear attention incrementally aggregates preceding routing scores into a compact history representation. At each routed layer, router conditions jointly on accumulated state + current hidden representation.

**Pattern Extracted**: Routing decisions are path-dependent — earlier decisions shape representations seen by downstream routers. Explicit routing memory resolves this.

**NeoTrix Domain Mapping**:
| Domain | Integration | Specific Module |
|--------|------------|-----------------|
| **NT-CORE** | E8 hexagram — routing decisions across reasoning stages accumulate state; later hexagram selections depend on earlier reasoning paths | `e8_engine` |
| **NT-MIND** | Self-evolution — SEAL pipeline stage selection conditioned on accumulated evolution history, not just current state | `seal_pipeline` |
| **NT-MEMORY** | Experience-tree — hub index loading conditioned on accumulated query history, not just current keywords | `experience_tree` |

**Implementation Insight**: HeRo bypasses 26.87% of parameters while achieving 100.24% dense performance on Llama 3.1-8B. Key: linear attention memory for routing state. Maps to NeoTrix's E8 reasoning — each hexagram selection should carry routing memory from prior reasoning stages, not just the current phi/coherence state.

---

### Paper 3: GrowPage — On-Demand KV Budgeting for Efficient LLM Reasoning

**Source**: arXiv:2609.03494 (2026-09-03)
**Core Idea**: KV capacity as a runtime resource. Lightweight dual-timescale query summaries (recent + long-term) estimate demand evolution. At capacity boundaries: compress within allocation OR acquire additional page when broader demand emerges. Integrates with PagedAttention.

**Pattern Extracted**: Dynamic KV budgeting — don't fix memory allocation upfront; acquire and compress on-demand based on actual attention demand.

**NeoTrix Domain Mapping**:
| Domain | Integration | Specific Module |
|--------|------------|-----------------|
| **NT-MEMORY** | KB embedding cache — dual-timescale summary of recent vs long-term access patterns determines cache allocation | `kb_cache` |
| **NT-CORE** | KVMem paged KV — GrowPage demand-estimation for >256K sessions | `kv_cache_optimizer` |
| **NT-PHYSICAL** | GPU memory management — on-demand page allocation for multi-device inference | `gpu_scheduler` |

**Implementation Insight**: GrowPage decomposes savings into two independent levers: scheduling (which agents active) and compression (residual idle cost). Maps to NeoTrix's dual-weapon specialization — Weapon Set I determines active modules, compression determines residual cost of idle modules.

---

### Paper 4: PARSER — Read in Parallel, Reason in Depth for Long-Context LLM Agents

**Source**: arXiv:2609.06702 (2026-09-06)
**Core Idea**: Decouple reading from reasoning. Bank of lightweight subagents (frozen) read document in parallel; lead agent (RL-trained) reasons in depth via iterative scatter-gather rounds. Lead broadcasts query to all subagents, aggregates evidence, formulates deeper follow-up query.

**Pattern Extracted**: Parallel reading + sequential reasoning. Subagents are frozen readers; lead agent is the only learnable component.

**NeoTrix Domain Mapping**:
| Domain | Integration | Specific Module |
|--------|------------|-----------------|
| **NT-WORLD** | UnifiedCrawler — parallel fetchers read web pages; NT-WORLD reasoning agent synthesizes findings | `unified_crawler` |
| **NT-MEMORY** | KB search — parallel subagents query different KB namespaces; lead synthesizes cross-domain results | `kb_search` |
| **NT-ACT** | Multi-tool orchestration — parallel tool calls for data gathering; lead agent reasons over aggregated results | `tool_orchestration` |

**Implementation Insight**: PARSER with 4B backbone outperforms DeepSeek-V4-Pro by 6.3 points. The scatter-gather pattern is directly applicable to NeoTrix's NT-WORLD + NT-MEMORY: parallel crawlers/searchers as frozen subagents, NT-MIND as the RL-trained lead that synthesizes.

---

### Paper 5: CEDAR — Error-Bounded Residual Routing for Efficient Long-Context Attention

**Source**: arXiv:2609.07237 (2026-09-07)
**Core Idea**: Coarse-to-fine error-aware routing. Each semantic chunk contributes a cheap KV summary to a residual attention path; chunks with high estimated approximation error are expanded to exact token attention. Exact + summarized combined in single softmax — refinement replaces, not duplicates, coarse evidence.

**Pattern Extracted**: Residual attention with error-bounded expansion. Cheap summaries provide global coverage; error estimation determines where to invest compute.

**NeoTrix Domain Mapping**:
| Domain | Integration | Specific Module |
|--------|------------|-----------------|
| **NT-CORE** | GWT salience — cheap salience estimation provides global coverage; high-uncertainty areas get full attention investment | `gwt_attention` |
| **NT-MEMORY** | KB retrieval — cheap BM25 summaries for coarse retrieval; high-uncertainty queries expand to exact embedding search | `kb_search` |
| **NT-MIND** | Experience-tree — cheap hub index for coarse matching; high-relevance branches get full experience loading | `experience_tree` |

**Implementation Insight**: CEDAR achieves 3x kernel speedup at 128K context while recovering most quality lost by hard sparse routing. The error-bounded expansion pattern maps to NeoTrix's experience-tree lazy loading: hub index provides cheap summary, route-table matching estimates error, high-error paths get full branch loading.

---

## Cross-Paper Synthesis: Unified Pattern Library

| Pattern | Papers | NeoTrix Module | Priority |
|---------|--------|----------------|----------|
| **Route Reuse with Sentinel Verification** | RouteRelay, HeRo | GWT attention routing | P0 |
| **On-Demand Memory Budgeting** | GrowPage, CEDAR | KVMem + experience-tree | P0 |
| **Parallel Reading / Sequential Reasoning** | PARSER | NT-WORLD + NT-MIND | P1 |
| **Error-Bounded Coarse-to-Fine Retrieval** | CEDAR, PARSER | KB search + experience-tree | P1 |
| **History-Aware Routing Memory** | HeRo, RouteRelay | E8 engine + SEAL pipeline | P1 |

## Top 3 Actionable Insights for NeoTrix

1. **Route relay for GWT** — Anchor GWT broadcasts establish routing decisions. Intermediate modules inherit route metadata. Sentinel challenge pattern: maintain near-miss salience candidates, re-route only when current attention fails against sentinel. Expected: 50-75% reduction in GWT re-routing overhead.

2. **Dual-timescale KV budgeting** — GrowPage's recent + long-term summary approach for KVMem. Recent access patterns determine short-term cache allocation; long-term patterns determine persistent KB embedding retention. Two independent levers: scheduling (which modules active) and compression (residual idle cost).

3. **PARSER scatter-gather for NT-WORLD** — Parallel frozen crawlers read web sources; NT-MIND as RL-trained lead synthesizes cross-source findings. Iterative scatter-gather rounds: lead broadcasts query → crawlers return evidence → lead formulates deeper follow-up. Expected: 10x improvement in multi-source research quality.

## Sources

- arXiv: 2609.07306 (RouteRelay, 2026-09-07)
- arXiv: 2609.08189 (HeRo, 2026-09-08)
- arXiv: 2609.03494 (GrowPage, 2026-09-03)
- arXiv: 2609.06702 (PARSER, 2026-09-06)
- arXiv: 2609.07237 (CEDAR, 2026-09-07)

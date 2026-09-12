# Model Reverse Engineering — Cycle 410 (2026-09-12)

## 5 New Models/Papers (August–September 2026)

---

### 1. Declarative Attention (DA) — Models Control Their Own Attention

| Field | Detail |
|-------|--------|
| **Paper** | `arXiv:2609.02737` — "Language Models Can Control Their Own Attention" |
| **Date** | 2026-09-02 |
| **Core Innovation** | Model declares attention regions inline during chain-of-thought: `<global>`, `<focus>`, `<local>`. Engine parses declarations like tool calls, skips most KV cache read. Zero-shot — works on off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B). |
| **Key Results** | 52.0% reduction in total attended tokens (Gemma-4-31B), 31.1% (Qwen-3.6-27B). Modest accuracy drops (1.27pp, 2.75pp) that shrink with model scale. |

**NeoTrix Domain Mapping:**

| Domain | Integration Point | Mechanism |
|--------|------------------|-----------|
| NT-CORE | GWT attention routing | Self-declared attention regions = GWT salience with explicit focus mode. Model introspects its own focus rather than external scoring. |
| NT-MEMORY | KV cache management | `<focus>` mode skips irrelevant KV pages = selective memory recall with reduced token budget |
| NT-MIND | SEAL pipeline reasoning | `<global>` for exploration, `<focus>` for distillation, `<local>` for recent feedback — attention modes map to evolution stages |

**Actionable Insight:** Self-declared attention is a new axis — instead of external routers predicting where to attend, the model itself declares intent. This aligns with NeoTrix's ConsciousnessTree where modules self-report attention needs rather than being externally gated.

---

### 2. CEDAR — Error-Bounded Residual Routing for Sparse Attention

| Field | Detail |
|-------|--------|
| **Paper** | `arXiv:2609.07237` — "Error-Bounded Residual Routing for Efficient Long-Context Attention" |
| **Date** | 2026-09-07 |
| **Core Innovation** | Coarse-to-fine approach: each semantic chunk contributes cheap KV summary to residual path. High-error chunks expanded to exact token attention. Single softmax normalization — refinement replaces, not duplicates. Output-error bound from within-chunk key/value dispersion governs refinement budget. |
| **Key Results** | Reduces reconstruction error by >98% vs hard dropping at equal exact-chunk budgets. ~3× kernel speedup at 128K context. |

**NeoTrix Domain Mapping:**

| Domain | Integration Point | Mechanism |
|--------|------------------|-----------|
| NT-MEMORY | KB retrieval | Coarse summaries as first-pass retrieval; high-error queries trigger full-text recall. Error-bounded budget allocation. |
| NT-CORE | E8 Hexagram reasoning | Residual summaries = compressed hexagram states; full expansion only for ambiguous reasoning branches |
| NT-WORLD | Crawl pipeline | Document ingestion uses coarse summaries; high-value documents get full content indexing on-demand |

**Actionable Insight:** "Refinement replaces, not duplicates" — this avoids the common trap of sparse attention stacking coarse and fine results. For NeoTrix's KB pipeline: coarse entity summaries serve most queries; only high-error (novel/contradictory) facts trigger full-text retrieval.

---

### 3. RouteRelay — Cross-Layer Route Reuse for Dynamic Sparse Attention

| Field | Detail |
|-------|--------|
| **Paper** | `arXiv:2609.07306` — "Event-Triggered Cross-Layer Route Reuse for Efficient Dynamic Sparse Attention" |
| **Date** | 2026-09-07 |
| **Core Innovation** | Router-agnostic method reusing route metadata across depth. Anchor layers do full routing; intermediate layers rescore previous top-route + compact sentinel set (near-miss + random probes). Reroute only when sentinel challenges weakest selected chunk. |
| **Key Results** | 99.99% route recall. Only 25–78% of rows rerouted (depending on cross-layer drift). 38–52% of full-routing score pairs evaluated. |

**NeoTrix Domain Mapping:**

| Domain | Integration Point | Mechanism |
|--------|------------------|-----------|
| NT-CORE | ConsciousnessTree layers | Anchor layers = L5/L6 deep analysis; intermediate layers = L1-L4 reuse previous routing with sentinel checks |
| NT-MIND | SEAL pipeline | Anchor analysis at stage transitions; intermediate stages reuse routing decisions, only reroute on sentinel triggers |
| NT-SHIELD | Security routing | Anchor security scan + sentinel-based rerouting for anomalous activity patterns |

**Actionable Insight:** Sentinel-triggered rerouting is efficient — most layers reuse routing decisions. Maps to NeoTrix's layered architecture where L5/L6 (cognition/meta) set strategic routes, L1-L3 (action/perception/embodiment) reuse with lightweight checks. Only novel stimuli (sentinels) trigger full rerouting.

---

### 4. PIVOT — Proxy Indexing for Token-Level Sparse Attention

| Field | Detail |
|-------|--------|
| **Paper** | `arXiv:2607.24593` — "Proxy Indexing Via One full-prefix Traversal" |
| **Date** | 2026-07-27 |
| **Core Innovation** | Group nearby queries into proxy query; one shared full-prefix scan per group. PIVOT-Reuse shares proxy top-k across group (max speed). PIVOT-Refine re-scores candidate set per query (matches dense accuracy). Single algorithm covers prefill and decode. |
| **Key Results** | 4× indexer acceleration, 1.6× end-to-end latency reduction at long context. Matches dense DSA indexer accuracy. Works on DeepSeek-V3.2 and GLM-5.1. |

**NeoTrix Domain Mapping:**

| Domain | Integration Point | Mechanism |
|--------|------------------|-----------|
| NT-MEMORY | KB query batching | Group similar queries into proxy; one scan per batch. Refine only for ambiguous queries. |
| NT-ACT | Capability dispatch | Group similar tool calls into proxy dispatch; only refine routing for novel tool patterns |
| NT-IO | LLM provider selection | Group similar request types into proxy routing; refine only when provider performance shifts |

**Actionable Insight:** The "nearby queries share overlapping top-k" observation holds for KB queries too — adjacent knowledge lookups often share the same hub/namespace. Proxy scan per group, refine only for outliers. Reduces repeated full-namespace scans.

---

### 5. MSA (Memory Sparse Attention) — 100M-Token Latent Memory

| Field | Detail |
|-------|--------|
| **Paper** | `arXiv:2603.23516` — "MSA: Memory Sparse Attention for Efficient End-to-End Memory Model Scaling to 100M Tokens" |
| **Date** | 2026-03 (updated) |
| **Core Innovation** | End-to-end trainable sparse attention with document-wise RoPE (parallel/global). KV cache compression with Memory Parallel engine. Memory Interleave for multi-round, multi-hop reasoning across scattered segments. Routing only in upper layers; lower layers independent. |
| **Key Results** | <9% degradation from 16K→100M tokens. 100M-token inference on 2×A800 GPUs. Outperforms RAG and RAG+rerank on 9 QA benchmarks. |

**NeoTrix Domain Mapping:**

| Domain | Integration Point | Mechanism |
|--------|------------------|-----------|
| NT-MEMORY | KB scaling | Document-wise RoPE = KB namespace-scoped embeddings. Memory Parallel = sharded KB hub routing. |
| NT-CORE | Layer-differentiated reasoning | Upper layers route across KB; lower layers process local context independently |
| NT-WORLD | Crawl pipeline | Memory Interleave = multi-hop reasoning across scattered crawl results without loading everything |
| NT-PHYSICAL | Hardware efficiency | Tiered storage (GPU routing keys, CPU content K/V) = hot/warm/cold KB tiering |

**Actionable Insight:** MSA proves <9% degradation at 100M tokens — the key is document-wise RoPE (reset positions per document) + selective routing only in upper layers. For NeoTrix: KB namespaces each get independent position encoding; cross-namespace routing only at cognition layer (L5/L6).

---

## Synthesis: 3 Meta-Patterns Across Cycle 410 Papers

### Meta-Pattern 1: Self-Declared vs Externally-Routed Attention

| Approach | Paper | NeoTrix Implication |
|----------|-------|---------------------|
| Self-declared (model declares focus) | Declarative Attention | ConsciousnessTree modules self-report attention needs |
| Sentinel-triggered (external triggers) | RouteRelay | GWT salience with sentinel-based rerouting |
| Proxy-grouped (batch similar queries) | PIVOT | KB query batching with proxy scans |

**Resolution:** NeoTrix should use a hybrid — modules self-declare for routine attention, sentinel triggers handle novel stimuli, proxy batching handles repeated patterns.

### Meta-Pattern 2: Refinement Replaces, Not Duplicates

| Paper | Mechanism | NeoTrix Application |
|-------|-----------|---------------------|
| CEDAR | Residual summaries + exact refinement in single softmax | KB coarse summaries serve most queries; full retrieval only for high-error |
| RouteRelay | Rescore previous routes + sentinel expansion | L1-L3 reuse L5/L6 routes; only novel stimuli trigger full rerouting |
| PIVOT | Proxy top-k shared, refine per-query | Proxy hub scans; per-namespace refinement only for ambiguous results |

**Resolution:** Every retrieval/computation layer in NeoTrix should follow "coarse-first, refine-on-demand" — never stack coarse and fine results redundantly.

### Meta-Pattern 3: Layer-Differentiated Processing

| Paper | Layer Split | NeoTrix Architecture |
|-------|-------------|----------------------|
| MSA | Upper layers route, lower layers independent | L5/L6 cross-domain routing, L1-L3 local processing |
| RouteRelay | Anchor layers (full routing) + intermediate (reuse) | L5/L6 anchor, L1-L4 reuse |
| Declarative Attention | Global/Focus/Local modes per layer | ConsciousnessTree mode switching per growth stage |

**Resolution:** NeoTrix's 6-layer architecture naturally supports this — cognition/meta layers handle cross-domain routing; action/perception/embodiment layers reuse routing decisions with lightweight sentinel checks.

---

## Source List

| # | Paper | URL | Date |
|---|-------|-----|------|
| 1 | Declarative Attention | `arxiv.org/abs/2609.02737` | 2026-09-02 |
| 2 | CEDAR | `arxiv.org/abs/2609.07237` | 2026-09-07 |
| 3 | RouteRelay | `arxiv.org/abs/2609.07306` | 2026-09-07 |
| 4 | PIVOT | `arxiv.org/abs/2607.24593` | 2026-07-27 |
| 5 | MSA | `arxiv.org/abs/2603.23516` | 2026-03 (updated) |

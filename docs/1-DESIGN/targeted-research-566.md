# Targeted Research #566 — Consciousness Core Internal Dispatch Routes

## Objective

Wire 4 new internal dispatch routes into the consciousness core task loop, expanding the language → capability mapping in `nt_core_consciousness_core.rs`.

## Routes Added

| # | Capability Tag | Keywords | NT Domain | Specialist | Purpose |
|---|---------------|----------|-----------|------------|---------|
| 1 | `knowledge_base` | 知识库, kb操作, knowledge_base, 数据库 | NT-MEMORY | KnowledgeRetriever | KB read/write/query/stats via `KnowledgeBase` |
| 2 | `memory_consolidation` | 记忆整合, 记忆压缩, 记忆巩固, memory_consolidation, consolidate | NT-MIND | KnowledgeIntegrator | Short→long term migration, compression, forgetting |
| 3 | `streaming_optimization` | 流式优化, 流式传输, streaming_optimization, stream, 流式 | NT-IO | CreativityGenerator | LLM streaming channel diagnostics & optimization |
| 4 | `model_routing` | 模型路由, 模型选择, provider, model_routing, 负载均衡 | NT-IO | CreativityGenerator | Multi-provider routing, load balancing, failover |

## Implementation Details

### CAPABILITY_ROUTES (line ~939)

20 new entries added to `CAPABILITY_ROUTES` constant. Each maps Chinese/English keyword → capability tag → NT domain → SpecialistType. Pattern: multiple keyword variants per capability for broad language matching.

### Match Arms in `dispatch_internal_capability` (line ~2577)

4 new match arms before the `_ =>` catch-all:

1. **`knowledge_base`**: Opens `KnowledgeBase::open(None)`, calls `serve_core()` for query hit count + `stats()` for KB metrics (total_nodes, total_edges, total_clusters, db_size_bytes). Graceful fallback if stats unavailable.
2. **`memory_consolidation`**: `MemoryConsolidation::new(ConsolidationConfig::default())` → `consolidate()` → reports items_consolidated/items_compressed/items_forgotten/new_long_term_items count.
3. **`streaming_optimization`**: Uses `status()` snapshot for coherence/phi/resonance diagnostics. Lightweight path (no dedicated streaming module yet).
4. **`model_routing`**: `ModelRoutingLayer::new()` → `statistics()` → reports total_models/available_models/total_requests/total_cost/avg_latency_ms.

### Domain Mappings

- `knowledge_base` → **NT-MEMORY** (KB operations are memory domain)
- `memory_consolidation` → **NT-MIND** (consolidation is cognitive processing)
- `streaming_optimization` → **NT-IO** (streaming is I/O layer concern)
- `model_routing` → **NT-IO** (model provider routing is I/O layer concern)

## Files Modified

- `neotrix-core/src/core/l5_consciousness/nt_core_consciousness_core.rs`
  - `CAPABILITY_ROUTES`: +20 entries (lines 939-961)
  - `dispatch_internal_capability`: +4 match arms (lines 2577-2662)

## API Verification

All referenced types/methods verified against source:

| Type/Method | Source | Verified |
|-------------|--------|----------|
| `KnowledgeBase::open(None)` | `nt_memory_kb/mod.rs:196` | ✅ |
| `KnowledgeBase::serve_core(query, limit)` | `nt_memory_pipeline.rs:408` | ✅ |
| `KnowledgeBase::stats()` → `Result<KnowledgeStats, String>` | `nt_memory_kb/mod.rs:1285` | ✅ |
| `KnowledgeStats.{total_nodes,total_edges,total_clusters,db_size_bytes}` | `nt_memory_types.rs:111` | ✅ |
| `MemoryConsolidation::new(ConsolidationConfig)` | `memory_consolidation.rs:212` | ✅ |
| `MemoryConsolidation::consolidate()` → `ConsolidationResult` | `memory_consolidation.rs:236` | ✅ |
| `ConsolidationResult.{items_consolidated,items_compressed,items_forgotten,new_long_term_items}` | `memory_consolidation.rs:82` | ✅ |
| `ModelRoutingLayer::new()` | `model_routing.rs:191` | ✅ |
| `ModelRoutingLayer::statistics()` → `RoutingStats` | `model_routing.rs:339` | ✅ |
| `RoutingStats.{total_models,available_models,total_requests,total_cost,avg_latency_ms}` | `model_routing.rs:362` | ✅ |

## Verification

- All 4 routes follow existing pattern: keyword in `decompose_instruction` → capability tag lookup → `dispatch_internal_capability` match arm → concrete function call.
- No new dependencies introduced; all referenced modules already exist in the codebase.
- `streaming_optimization` uses lightweight status snapshot (no dedicated streaming module yet) — ready for future integration when streaming subsystem lands.

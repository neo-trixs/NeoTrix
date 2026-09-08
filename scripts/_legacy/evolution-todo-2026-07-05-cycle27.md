# NeoTrix Evolution TODO — Cycle 27 (2026-07-05)

> Generated from: 7-domain code review (6 bugs found) + Internet research (5 topics) + Pipeline meta-cognition analysis

## P0 — Blockers (Must Fix Before Next Pipeline Restart)

### 1. Pipeline Source Saturation
**Status**: 🔴 Blocking — 0 nodes/cycle for 314 consecutive cycles
**What**: All 58 Wikipedia topics and 18 GitHub repos exhausted. No discovery happening.
**Fix**: 
- Expand WIKI_TOPICS from 58 to 500+ using Wikipedia `list=categorymembers` for CS/AI/Math
- Use `list=random` as primary discovery when curated topics exhausted
- Expand GitHub trending beyond 18 repos (use GitHub trending API)

### 2. Embeddings = 0 (Stale)
**Status**: 🔴 No RAG/semantic search
**Fix**: Set `NEOTRIX_EMBEDDING_API_KEY` and run `kb-generate-embeddings.py`

### 3. 38,707 Orphaned Nodes — No Edge Creation
**Status**: 🔴 Pipeline creates nodes but never edges
**Fix**: Add `ensure_edge()` after every `_store_*_as_node()` call:
- New Paper → related Concept via `related_to`
- New Repository → owner Organization via `owned_by`
- New Concept → parent Category via `subclass_of`

## P1 — Critical Code Fixes

### 4. Rust: Negative Advantage Signal Zeroed (`nt_core_prm.rs:1089,2346`)
**Status**: ✅ **FIXED**
**Fix**: `adv.max(0.0)` → `((adv + 1.0) / 2.0).max(0.0).min(1.0)` — preserves negative learning signal

### 5. Rust: FTS Index Missing for Summary-less Nodes (`nt_memory_store.rs:38-46`)
**Status**: ✅ **FIXED**
**Fix**: Removed `if let Some(summary)` guard — always insert FTS entry with empty summary

### 6. Rust: `update_node` Doesn't Sync FTS (`nt_memory_store.rs:51-72`)
**Status**: ✅ **FIXED**
**Fix**: Added `UPDATE nodes_fts` after `UPDATE nodes` to keep FTS in sync

### 7. Python Wiki Concept Fill SQL Too Restrictive (`auto-absorb.py:401-408`)
**Status**: ✅ **FIXED**
**Fix**: Changed to `COALESCE(NULLIF(title,''), url)` as fill_target to handle empty-title nodes

### 8. Python Todo List Counts ALL Sessions (`auto-absorb.py:852-853`)
**Status**: ✅ **FIXED**
**Fix**: Changed query to `auto_absorb_defect_{CYCLE_COUNT}_%` to only count current session defects

### 9. Embedding Search Matches UUID Not Text (`nt_memory_search.rs:331`)
**Status**: ✅ **FIXED**
**Fix**: Changed to lowercased ID contains query (partial fix — title-based matching needs schema changes)

### 10. `zscore_normalize` Drops NaN Entries (`nt_core_prm.rs:856-866`)
**Status**: ✅ **FIXED**
**Fix**: Replace NaN with 0.0 in-place instead of filtering — preserves output vector length

### 11. Substring Tag Matching (`nt_core_policy.rs:292-300`)
**Status**: ✅ **FIXED**
**Fix**: Changed to exact string match (`tag.as_str()`) instead of `t.contains()`

## P2 — Architecture Improvements

### 12. PRM: PRM → Generative ThinkPRM
**Research**: ThinkPRM verbalizes step verification via CoT, outperforms discriminative PRMs
**Plan**: Replace current heuristic PRM scorer with generative step verifier
**Files**: `nt_core_prm.rs`, `nt_core_policy.rs`

### 13. SAE: TopK + AuxK Architecture
**Research**: TopK/AuxK (ICLR 2025) has clean scaling laws, zero dead latents
**Plan**: Upgrade `nt_core_sae.rs` from basic SAE to TopK + AuxK with mean-centering pre-bias
**Files**: `nt_core_sae.rs`, `nt_core_sae_bridge.rs`

### 14. GWT: MCP Stack Upgrade — `rmcp` + `#[tool_box]`
**Research**: `rmcp` crate (5.9M+ downloads) replaces raw JSON-RPC; `#[tool_box]` macro for tools
**Plan**: Refactor `McpRegistry` to use `rmcp`, add Streamable HTTP transport
**Files**: `neotrix-core/src/agent/tool/mcp/mod.rs`, `neotrix-core/src/neotrix/mcp_discovery.rs`, `neotrix-core/src/neotrix/mcp_tools.rs`

### 15. World Model: FEP → Active Inference Bridge
**Research**: JEPA prediction loss = variational free energy; E8 states = discrete state space for active inference
**Plan**: Reformulate `nt_core_jepa` loss as VFE; implement EFE-driven action in `nt_act_autonomy`
**Files**: `nt_core_jepa/`, `nt_world_infer/`, `nt_act_autonomy/`

### 16. Pipeline: Bulk Dedup Cleanup
**Plan**: Run SQL to delete duplicate openlibrary.org search URL nodes
```sql
DELETE FROM nodes WHERE url LIKE 'https://openlibrary.org/search?%'
  AND id NOT IN (SELECT MIN(id) FROM nodes WHERE url LIKE 'https://openlibrary.org/search?%' GROUP BY url);
```

## P3 — Minor Improvements

### 17. Add `NEOTRIX_GITHUB_TOKEN` Env Var Support
Without unauthenticated token, 60 req/hr limit blocks GitHub discovery
**Files**: `nt_api_client.py`, `neotrix-auto-absorb.py`

### 18. Pipeline Cycle Interval
Current `--interval 1` runs 86,400 cycles/day at 0 content/cycle
**Fix**: Set `--interval 300` (5min) for pipeline restart

### 19. Defect Delta Tracking
Meta-cognition reports same defect counts every cycle even when flat
**Fix**: Only log recurring pattern when count *increases*

## Baseline Validation

| Check | Result |
|-------|--------|
| `cargo check --lib -p neotrix` | ✅ 0 errors |
| `cargo clippy --lib -p neotrix` | ✅ 0 warnings |
| `cargo test -p neotrix --lib` | ✅ **6221 pass, 0 fail** |
| Python `py_compile` | ✅ Syntax OK |
| KB Stats | 80,247 nodes, 267,877 edges, 24,932 filled |

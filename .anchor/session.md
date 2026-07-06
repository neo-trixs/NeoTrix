# Session Anchor

> This file anchors the AI agent's session context for the NeoTrix documentation system.
> Last updated: 2026-07-04

## Build Status

- `cargo check --lib -p neotrix`: ✅ **0 errors, 0 warnings**
- `cargo check --features full --lib -p neotrix`: ✅ **0 errors**
- `cargo clippy -p neotrix --lib`: ✅ **0 warnings**
- `cargo clippy -p neotrix-tauri`: ✅ **0 warnings**
- `cargo test -p neotrix --lib`: ✅ **6187 passed, 0 failed, 10 ignored** (up from 6139)

## Fixed Defects (This Session)

| # | Defect | Location | Fix |
|---|--------|----------|-----|
| 1 | **PRM double-learn** — `learn_from_scores` called twice with same args on aux path | `nt_core_prm.rs:566` | Replaced `&scores` with proper `aux_scores: Vec<ProcessScore>` so auxiliary reward is actually learned from instead of duplicating the primary update |
| 2 | **E8 hardcoded bias** — `set_e8_attention_weights(bias)` stored weights but discarded `bias`, `resonant_broadcast` hardcoded `0.3` | `workspace.rs:60,123,206` | Added `e8_attention_bias: f64` field to `GlobalWorkspace`, stored in `set_e8_attention_weights()`, used in `resonant_broadcast()`. Initialized to `0.3` in constructor for backward compat |
| 3 | **BM25 disconnected** — KB had `bm25.rs`, `bm25` field, `mark_bm25_dirty()`, but no `rebuild_bm25()` and `hybrid_search()` never used BM25 | `mod.rs` + `nt_memory_search.rs` | Added `KnowledgeBase::rebuild_bm25()` + wired BM25 as Tier 1b in `hybrid_search()` (between FTS5 and title LIKE). All 5 callers updated to pass BM25 index |
| 4 | **call_llm no timeout** — `block_on(gateway.complete())` had no timeout wrapping | `engine_core.rs:750` | Added `mpsc::channel` + `recv_timeout(120s)` guard around the block_on call |
| 5 | **ExternalKnowledgeAbsorbStage redundant KB** — opened its own `KB::open(None)` instead of using pipeline's existing KB | `pipeline.rs:1083` | Takes KB from `brain._nt_memory_kb` via `take()`, reopens fresh connection after explorer consumes it |
| 6 | **store_conversation_record missing** — schema existed, READ path existed, but no INSERT function | `mod.rs:1075` | Added `store_conversation_record()` with full 15-column INSERT matching the schema |
| 7 | **Layer violation borrow conflicts** — `run.rs:172` + `pipeline.rs:1117` — immutable borrow of `b.reasoning_engine`/`brain._nt_memory_kb` conflicting with mutable `self` calls | 2 files | Used `take()+replace` pattern to release borrow before calling `persist_pending_entries()` |

## Build (Cumulative)

- Total tests: **6187 passed, 0 failed, 10 ignored**
- 6 high-impact defects fixed across PRM/E8/GWT/KB/SEAL pipelines
- 12 new tests activated (from newly compiled conversation_records callers)

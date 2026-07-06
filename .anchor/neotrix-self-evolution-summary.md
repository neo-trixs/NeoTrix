# NeoTrix Self-Evolution Summary

## Cycle Overview
- **Current session**: 2026-07-05 — 3 architecture roams + 4 P0/P1 fixes
- **Git branch**: main
- **Build**: cargo check --lib 0 errors, cargo clippy 0 warnings, cargo test 6190/0/11

## Goal
Self-evolution cycle: deep architecture roam across 5 key pipelines → discover 49 weak links (KB:22, Gateway:15, EventBus+SEAL:12) → fix highest-impact defects → absorb external patterns

## Progress

### Done
- **System baseline**: cargo check --lib ✅, cargo clippy ✅, cargo test ✅ 6190 passed, 0 failed, 11 ignored (+54 total from start of cycle)
- **3 parallel architecture roams** — traced data pipelines across 49 weak-link sites

  | Pipeline | Gaps Found | Key Weak Links |
  |----------|-----------|----------------|
  | KB Search | 22 | fused_cache dead(allocated never used), fuse_signals() dead code, BM25 None on boot, query_to_avg_embedding() string-proxy, rrf_fuse() dead, graph disconnected, no cross-encoder, no query expansion |
  | GatewayV2 | 15 | check_proxy_l7() claimed in docs but absent, on_success() bypasses cooldown, 5 active bypass sites(api.rs/earn/CLI), semantic cache exact-match only, Pollinations stream single-chunk |
  | EventBus+SEAL | 12 | EventBus **never instantiated in production**(355 lines of dead infrastructure), Spider fake empty response, 42 block_on bridges, no health check, 14 recipe-only log-only stubs |

- **Fixed 4 P0/P1 defects:**

  | Defect | Pipeline | Fix |
  |--------|----------|-----|
  | Spider crawl_with_checkpoint() returns empty body | Crawl | Added optional fetcher fn param; `default_fetcher()` for backward compat |
  | KB fused_cache never read/written | KB Search | `search()` and `hybrid_rerank_search()` now check cache before SQL, store results after |
  | BM25 None on boot + no dirty marks on writes | KB Search | `open()` sets `bm25_dirty: true` + calls `rebuild_bm25()`; 6 write methods mark dirty + clear cache |
  | EventBus orphan (0 production callers) | EventBus | Created in BackgroundLoop::start(), 9 layer subscribers via `subscribe_all_layers_sync()`, emits TaskSubmitted on awareness tick |

- **External intelligence absorption**: Self-evolving agent patterns (journal-driven feedback loops, event-sourced ledgers), Kleos 4-channel hybrid search (FTS5+BM25+Vector+Graph+RRF), Symbiont/Swink-Agent constrained self-modification

### Blocked
- Rust toolchain 1.94.0 (CVE-2026-5222/5223 patched in 1.96.0) — upgrade pending minor compat check
- Docker sandbox (sandbox_v2/ missing entirely)
- E8→HypothesisNetwork action dispatch (hard `// TODO` in EWHR)

## Key Decisions
- **fetcher optional param**: `crawl_with_checkpoint()` now accepts `Option<&dyn Fn(&CrawlRequest) -> Result<CrawlResponse, String>>` — backward-compatible via `None` (uses `default_fetcher`)
- **cache invalidation on writes**: `mark_bm25_dirty()` now calls `fused_cache.lock()?.clear()` — cached results are stale after any node mutation
- **BM25 auto-rebuild**: `KnowledgeBase::open()` marks dirty + rebuilds — removes the "BM25 dead on boot" gap
- **EventBus lifecycle**: Created in `BackgroundLoop::start()`, dropped when loop shuts down (via `Drop` → `shutdown()` → joins all subscriber threads)

## Next Steps
1. **Fix P1: fuse_signals() dead code** — wire into hybrid_search() or remove
2. **Fix P1: entity_graph_scores() disconnected** — add graph-aware signal to search pipeline
3. **Fix P1: rrf_fuse() dead code** — use for reciprocal rank fusion between FTS5 and BM25
4. **Fix P1: GatewayV2 5 bypass sites** — api.rs(3), earn engine, CLI TUI side-LLM
5. **Fix P2: check_proxy_l7()** — implement HTTP HEAD probe as documented in AGENTS.md
6. **Fix P2: BM25 in-memory only** — persist to SQLite so index survives restart
7. **Fix remaining P1 query_to_avg_embedding()** — call real embedding API instead of string-match proxy

## Known Issues Requiring Manual Action
- `--features full` compile requires >20 heavy deps (sentry, rmcp, k256) — use `--lib` for fast iteration
- Embedding API calls need `NEOTRIX_EMBEDDING_API_KEY` env var
- KB with 86K+ nodes: BM25 rebuild on every boot is O(n) memory scan — needs incremental update

## External Intelligence Absorbed
- **Kleos**: 4-channel hybrid search (FTS5 + BM25 + Vector + Graph) with RRF fusion, cross-encoder reranking, FSRS-6 spaced repetition, 32-shard LRU cache — NeoTrix has all 4 channels but graph and RRF are dead code
- **Self-evolving agents (Symbiont, Swink-Agent, RainEngine)**: Journal-driven feedback loops, event-sourced durable ledgers, constrained self-modification via ReAct+Reflection — NeoTrix's SEAL pipeline aligns well but lacks event-sourced pattern
- **Agent architecture taxonomy 2026**: 8 canonical patterns across 4 quadrants (Task/Knowledge/Orchestration/Self-Evolution) — NeoTrix covers 7/8, missing event-sourced agent memory pattern

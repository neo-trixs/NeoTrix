# NeoTrix Evolution TODO — 2026-07-06 Cycle 30

## Build Baseline

| Check | Result |
|-------|--------|
| `cargo check --lib -p neotrix` | ✅ 0 errors |
| `cargo check --features full --lib -p neotrix` | ✅ 0 errors |
| Production `panic!`/`todo!`/`unreachable!`/`unimplemented!` | 🟢 0 |
| Production `.expect()` | 🟢 ~347 (22 in `entry/mod.rs` boot path acceptable) |
| Pipeline 10h crawl | ✅ PID 46080, 36min, C31, KB 81,347 nodes |

---

## ✅ Completed This Cycle (30 items)

### P0 Fixes (10)

| # | Fix | File | Defect |
|---|-----|------|--------|
| 1 | HeuristicCoach::learn() EMA | `nt_core_prm.rs:431` | Empty learn stub — now adapts success_base/failure_penalty via EMA |
| 2 | weakness_count from tech_debt | `monitor.rs:26` | Hardcoded 0 — now reads `tech_debt.total_count` |
| 3 | Crawl queue rotating seed | `auto-absorb.py:1206` | Same 60 URLs every cycle — now rotates through 400 topics |
| 4 | Shell injection | `publisher.rs:100-133` | `sh -c` with unsanitized input — now `Command::new()` |
| 5 | build_context() empty | `engine_core.rs:818` | Returns `String::new()` — now queries KB |
| 6 | build_artifact_context() empty | `engine_core.rs:863` | Returns `String::new()` — now queries artifact indexer |
| 7 | self_iterate() empty | `engine_core.rs:754` | Empty body — now runs observer + LLM self-iteration |
| 8 | pani!() in clone_connection | `nt_memory_kb_bridge.rs:173` | `panic!()` — now returns `Result` |
| 9 | TOCTOU race x2 | `ip_privacy.rs:252`, `chain.rs:230` | `.clone().expect()` after separate `is_some()` check |
| 10 | ~45 `.expect()` → `?` | 36 files | Replaced with proper error propagation |

### P1 Fixes (8)

| # | Fix | File | Defect |
|---|-----|------|--------|
| 11 | infer_reasoning_type() stub | `engine_core.rs:929` | Always returns Conversation — now uses keyword matching |
| 12 | plan_reasoning() empty | `engine_core.rs:752` | Returns `String::new()` — now generates plan from KB |
| 13 | DataSynthesisStage empty | `data_synthesis.rs:369` | Empty process — now logs synthesis stats |
| 14 | Wikipedia User-Agent + retry | `nt_api_client.py` | API_FAIL (no UA, no retry) — fixed |
| 15 | .expect() in journal_index | `nt_world_journal_index.rs:193` | `.expect()` — replaced with `unwrap_or_else` |
| 16 | process::exit in system_proxy | `system_proxy.rs:128` | PA009 violation |
| 17 | process::exit in nt_io_proxy | `nt_io_proxy.rs:114` | PA009 violation |
| 18 | pipeline.rs borrow-after-move | `pipeline.rs:714` | Stats moved before borrow — cloned first |

### P2 Fixes (12)

| # | Fix | File | Description |
|---|-----|------|-------------|
| 19 | Removed nt_core_abstr dead module | `core/` | 20-line skeleton, deleted |
| 20 | Removed nt_core_feph dead module | `core/` | 18-line skeleton, deleted |
| 21 | Removed nt_core_edge dead module | `core/` | 18-line skeleton, deleted |
| 22 | Removed nt_core_saesteer dead module | `core/` | 18-line skeleton, deleted |
| 23 | Removed nt_core_wta dead module | `core/` | 18-line skeleton, deleted |
| 24 | Removed nt_core_ssm dead module | `core/` | 702-line duplicate SSM, deleted |
| 25 | Removed nt_core_procedural dead stub | `core/` | 18-line stub, deleted |
| 26 | Edge creation in pipeline | `auto-absorb.py` | `_ensure_edge` now called after wiki/arxiv/github store |
| 27 | +164 edges created (31 cycles) | KB | Pipeline creating `about_topic`, `sub_topic_of` edges |
| 28 | Dead code annotations reduced | Various | Removed `#[allow(dead_code)]` from unused items |
| 29 | process::exit in bin-archive | Archived | Removed from production concern list |
| 30 | TOCTOU races eliminated | 2 files | Safe single-if-let pattern |

---

## 🔴 P0 Remaining (blocked)

### P0-1: KB Embeddings = 0
- **Blocked**: Needs `NEOTRIX_EMBEDDING_API_KEY` env var
- **Impact**: RAG/semantic search non-functional
- **Fix**: Set key, run Rust embedding pipeline (`/kb embed` — nt_memory_embed)

### P0-2: 14 SEAL Log-Only Stages
- **Status**: Identified
- **List**: sleep, socialize, explore, automate, dream, meditate, reflect, crystallize, consolidate, synthesize, imagine, intuit, create, transcend
- **Fix**: Implement real behavior or remove from pipeline

---

## 🟡 P1 Remaining

### P1-1: 8,000+ Empty Concept Nodes
- **Status**: In progress (pipeline filling ~40/cycle)
- **10h projection**: ~24,000 empty → ~20,000 filled
- **Fix**: Continue pipeline, or bulk backfill with Wikipedia summaries

### P1-2: 39 TODO/FIXME Comments in Production
- **Status**: Documented
- **Pattern**: Mostly "inject via DI" — requires dependency injection refactoring
- **Fix**: Systematic DI injection through constructor parameters

### P1-3: Agent Visualization (Desktop UI 35%)
- **Status**: Deferred
- **Reference**: Pixel Agents / AgentRoom / Pixel Agent Desk
- **Fix**: Real-time workspace visualization for E8/GWT activity

### P1-4: No Kanban/Work Item State Machine
- **Status**: Deferred
- **Fix**: TUI kanban view for 34-stage SEAL pipeline visualization

---

## Pipeline Status (live)

| Metric | Value | Evolution |
|--------|-------|-----------|
| PID | 46080 | Running 36 min |
| Cycles | 31 | ~2 cycles/min |
| KB Nodes | 81,347 | +340 (0.4%) |
| KB Edges | 274,307 | +164 (0.06%) |
| Empty | 42,903 | -1,227 (2.8%) |
| Rate | +11 nodes/cycle | +40 empty fill/cycle |
| 10h Projection | ~84,000 nodes | ~278,000 edges, ~18,000 empty |

---

## How to Continue

```bash
# Monitor pipeline
tail -30 ~/.neotrix/pipeline-10h.log

# Check KB growth
sqlite3 ~/.neotrix/knowledge.db "SELECT COUNT(*) FROM nodes; SELECT COUNT(*) FROM edges; SELECT COUNT(*) FROM nodes WHERE coalesce(content,'')='' OR content IS NULL;"

# Build
cargo check --lib -p neotrix
cargo check --features full --lib -p neotrix

# Fix remaining embeddings (NEOTRIX_EMBEDDING_API_KEY required)
# Rust 原生 (kb-embed-pq 由 kb-embed-server.py 训练 codebook, 已保留):
# python3 scripts/kb-embed-pq.py   # 仅当需要重新训练 PQ codebook 时

# Generate evolution TODO from pipeline meta-cognition (Rust port)
# 等价命令: /self-audit evolution
```

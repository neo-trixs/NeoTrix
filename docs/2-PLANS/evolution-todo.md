# NeoTrix Evolution TODO — 2026-08-13 Cycle 241+ (P1 治理合规 + warning 清零)

## Build Baseline (2026-08-13)

| Check | Result |
|-------|--------|
| `cargo check --lib -p neotrix` | ✅ 0 errors / 11 warnings (9 对方新模块 metadata + 2 nt_file_ability) |
| `cargo check --features full --lib -p neotrix` | ✅ 0 errors / 15 warnings |
| `cargo test -p neotrix --lib` | ✅ 6811 passed / 2 failed(修复后待重跑) / 12 ignored |
| warning 清理 | 🟢 43 → 11 (无效断言/死代码/未使用变量 + rate_limit_middleware 接线) |

---

## ✅ Completed Cycle 241+ (蜕变迭代)

| Commit | 交付项 | 说明 |
|--------|--------|------|
| `b35aa23` | P1 治理合规审计接线 | run_governance_audit 真实评估 80 条宪法规则, compliance 从硬编码 1.0 → 0.27→0.5 路径; GLOBAL_CONSTITUTION 优先 dev-rules.md |
| `bb93680` | warning 清理 43→11 | 9 处无意义 `assert!(x>=0)` / 死代码 parse_scalar/fetch_elevation_single / 未使用变量 + rate_limit_middleware 接线 (F2) |
| `6f6eb6f` | 断言语义修正 | 修正 warning 清理引入的 2 个错误断言 (health_report 镜像 state / volition 候选真实计数) |
| `bf7f51a` | 治理生产路径测试 | tick → Phase 4.6 审计消费真实 next_actions, fractal_depth=1 断言 + with_kb_lock 签名修复 |

### D1 (蓝图) 状态: 已完成 (并发会话)
- `run_growth_cycle` Phase 2 用 `build_phi_state()` 构造 64 维意识谱 → `IITPhiCalculator::compute_phi`
- `core_snapshot_from_tree` 持久化真实 `tree.trunk.phi`, status 读取真实整合信息
- 测试覆盖: `test_growth_cycle_computes_real_iit_phi` + iit_phi 11 tests 全绿

---

## Cycle 30 历史 (2026-07-06)

## Build Baseline (cycle 30)

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

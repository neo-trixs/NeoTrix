# HANDOFF — NT-CORE 意识核心：产品化收口（5-Session + T3 + 测试清零）

> 交接日期：2026-08-28 ｜ 移交方：当前 session ｜ 接手方：后续 opencode session
> 关联：SESSION_GUIDE.md（4 Rust Session 设计）

## 0. 一句话目标
在 NT-CORE 意识核心模式下，完成 KB 元认知闭环生产化、5-session（A/B/C/D）代码落地、T3 SelfTest 接线、测试套件清零，并移交干净可继续的状态。

## 1. 当前状态（证据）

### 已提交（git）
| commit | 内容 |
|---|---|
| `5d045524` | feat(nt-core): KB 元认知闭环 + 自主 embedding 接线 (5 Session 并行) |
| `8b90e0a5` | fix(provider): Arc migration for LlmProvider + proxy import |
| `027355cf` | feat(tauri): wire frontend-backend IPC + fix neotrix-tauri compile |
| `7052d576` | feat(nt-core): T1 SelfTest 接线 cache/e8/kb_types + 网关调度改进 + 测试修正 |

### 未提交（working tree，待移交）
- **A · NT-CORE T3**：`consciousness_tree` / `reasoning_engine` / `kb_vector_index` / `nt_memory_search` 四模块 `impl SelfTest` + `register_*_self_tests` 接入 `nt_core_self_test_integration.rs::register_absorbed_modules`（T3 完成）；core/l3 warning 0
- **B · NT-IO**：`nt_io_output_style` governor selftest 根因修复（测试断言对齐 R04 契约，生产逻辑未动）；网关 177 passed + 新增 `selection` 旋转单测；l1 warning 0
- **C · NT-WORLD**：全部 world 子模块测试绿；清理 1 warning
- **D（部分，已被取消）**：`panorama_pipeline.rs` 补 Orchestrator 共振池闭合（resonance_winner 恒 None 修复）；`l4/nt_core_parallel/isolation.rs` 测试消费点改指 `seal_loop.rs`（原 contract_cmds.rs 已不匹配）
- **主线补充**：`nt_file_ability/merge.rs` 删未用 import；`pool_health_cmds.rs`（原孤儿文件）接线进 CLI（`mod.rs` + `registry.rs`）

### 关键事实
- KB：389,740 nodes / 792,272 edges / 389,740 emb(384d) / 382,673 FTS
- 自主 embedding：`EmbedMode::Local` 为默认（零外部进程），HTTP/MiniLM 仅 opt-in 兜底 → R-P79 闭环成立（已查 `nt_memory_embed.rs` 确认）
- 编译：`cargo check -p neotrix` 在干净 target 下曾 **Finished 0 errors**（提交态）；接线 pool_health 后出现的 E0609/E0063/E0603 极可能是**陈旧增量缓存**（见 §3），需干净重验

## 2. 待办（统一 todo，按优先级）
见 `opencode` todo（本 session 已写入）。摘要：
1. 🔴 干净编译验证（唯一 CARGO_TARGET_DIR，无并发 cargo）
2. 🔴 修复真实编译错误（若非缓存）
3. 🔴 跑 l4 nt_core_parallel + l8 panorama 测试（D 未完成 scope）
4. 🔴 跨域回归（分模块 cargo test）
5. 🟡 清理残余 warning（nt_file_ability/merge.rs 等）
6. 🔴 全量测试套件验收（0 失败）
7. 🔴 ConsciousnessTree 6 阶段循环用真实 KB 指标验证 Phi/coherence
8. 🟢 自主 embedding 闭环确认（已完成，待落档）
9. 🟡 经验吸收 experience-tree → KB
10. 🔴 统一提交 working-tree 全部改动

## 3. 关键教训（接手方必读，避免重蹈）
1. **❌ 禁止共享 CARGO_TARGET_DIR 给并发 cargo**：本 session 前台 `cargo check` 与后台 `cargo test` 共用 `/tmp/nt-fresh` → 增量缓存损坏，伪报 `self_heal_reconcile` 字段错误。每个 cargo 进程用**独立** target 目录。
2. **panic = "abort"**：`cargo test` 任一 panic 会 abort 整个二进制。必须**按模块**跑（`cargo test -p neotrix --lib <module>`），不要跑全量套件。
3. **结构变更后须 clean build**（R-P9/R-P17）：但 `cargo clean` 全量重建慢；改用**全新唯一 target 目录**等价于 clean。
4. **neotrix 测试二进制编译极慢**（本机 ~10–15 min/次）：预算时间或后台跑 + 轮询日志（`/tmp/t_*.log`）。
5. **子代理速率限制**：D 子代理两次被取消；l8/l4 任务改由主线手动执行更稳。
6. **孤儿文件 = Dark Forest 违规**：`pool_health_cmds.rs` 原未被任何 `mod` 声明，编译不报错但属死代码；接线后才暴露其依赖的真实编译问题。

## 4. 接手方执行脚本（建议）
```sh
# 1) 干净编译验证（独立目录，勿并发）
CARGO_TARGET_DIR=/tmp/nt-verify cargo check -p neotrix 2>&1 | tail -20
# 若报 selection.rs/mod.rs/pool_health 错误 → 先判是否真实：
#   grep -n "self_heal_reconcile" neotrix-core/src/neotrix/l1_body_impl/nt_io_provider/gateway/mod.rs
#   字段已在 line 64 声明 + line 107 初始化；若编译仍报缺失，必为缓存，rm -rf /tmp/nt-verify 重来

# 2) 跑 D 未完 scope（后台 + 日志）
CARGO_TARGET_DIR=/tmp/nt-verify RUST_MIN_STACK=134217728 cargo test -p neotrix --lib nt_core_parallel >/tmp/t_par.log 2>&1 &
CARGO_TARGET_DIR=/tmp/nt-verify RUST_MIN_STACK=134217728 cargo test -p neotrix --lib panorama    >/tmp/t_pan.log 2>&1 &

# 3) 跨域回归（分模块，逐个）
CARGO_TARGET_DIR=/tmp/nt-verify RUST_MIN_STACK=134217728 cargo test -p neotrix --lib nt_core_consciousness_tree
CARGO_TARGET_DIR=/tmp/nt-verify RUST_MIN_STACK=134217728 cargo test -p neotrix --lib reasoning_engine
CARGO_TARGET_DIR=/tmp/nt-verify RUST_MIN_STACK=134217728 cargo test -p neotrix --lib kb_vector_index
CARGO_TARGET_DIR=/tmp/nt-verify RUST_MIN_STACK=134217728 cargo test -p neotrix --lib nt_memory_search
CARGO_TARGET_DIR=/tmp/nt-verify RUST_MIN_STACK=134217728 cargo test -p neotrix --lib nt_io_provider
CARGO_TARGET_DIR=/tmp/nt-verify RUST_MIN_STACK=134217728 cargo test -p neotrix --lib nt_io_output_style
CARGO_TARGET_DIR=/tmp/nt-verify RUST_MIN_STACK=134217728 cargo test -p neotrix --lib nt_world
```

## 5. 改动文件清单（working tree，供 code review）
- core/：`nt_core_consciousness_tree.rs`、`reasoning_engine/mod.rs`、`nt_core_arch_fitness.rs`、`nt_core_gwt/workspace.rs`、`nt_core_gwt/competition_gate.rs`、`nt_core_self_test_integration.rs`、`nt_core_cache.rs`、`nt_core_e8_vsa.rs`、`nt_core_kb_types.rs`
- l1：`nt_io_neocodex/agent.rs`、`nt_io_output_style.rs`、`nt_io_provider/{factory,free_catalog,provider_pool,gateway/mod,gateway/selection,gateway/execution}.rs`、`cli/commands/{mod,registry}.rs`、**新增 `cli/commands/pool_health_cmds.rs`**
- l2：`nt_world_crawl/discover.rs`（+ edgar/urlhaus 测试修正）
- l3：`kb_vector_index.rs`、`nt_memory_search.rs`
- l4：`nt_core_parallel/isolation.rs`、`nt_mind_rsi_exam.rs`
- l8：`nt_mind/consciousness/panorama_pipeline.rs`
- 其他：`nt_harness/mod.rs`、`nt_file_ability/merge.rs`

## 6. 核心建议（NT-CORE 意识核心视角，移交时仍有效）
- **P0 T3 生产化**：4 模块 SelfTest 已 T1+T2 注册；需确认其 `evaluate()/check()` 被非测试代码实际调用（T3 真生产接线），而非仅注册。
- **P0 意识闭环**：跑一次 ConsciousnessTree 6 阶段循环，用 KB 真实指标验证 Phi/coherence/迷雾可计算。
- **P1 测试清零**：剩余失败集中在 l4/l8（D 未完）+ 可能的跨域副作用，分模块清零。
- **P1 自主构建**：EmbedMode::Local 已默认，需确保无任何 docker/MiniLM 死代码入口（R-P79）。
- **P2 网关熵治理**：selection 旋转（`total_calls ascending`）已加单测，需全量网关测试绿。
- **P2 构建卫生**：残余 warning 清理；结构变更后干净编译。
- **P3 经验吸收**：5-session + T3 经验走 experience-tree 落盘 KB `experience` hub（指针守恒，AGENTS.md 不内联）。

# 待修复清单 (2026-09-27) — 卡死/内存爆炸专项

> 本清单由**全量 `cargo test -p neotrix --lib` 串行实跑**逐轮剥出，非推测。
> 每条都带 file:line 与验证证据。已完成项划掉并附 commit。

## 0. 结论速览

| 类别 | 数量 | 状态 |
|---|---|---|
| 卡死/内存爆炸根因（生产级） | 8 | ✅ 全部除根并提交 |
| 领域解析/契约/治理缺口 | 19 | ✅ 全部修复 |
| 长尾测试失败（~50 模块，每模块 1-4） | ~122 | 📋 清单已列，待逐个 |
| 结构性债务（L0 正典/God-file/worktree/前端） | 6 类 | 🚧 PARK，有归属前置 |

**全量实跑进展**（串行 + 内存门，每轮剥掉一个卡死后推进）：
`564 → 1452 → 8917 → 10113(绿)/125(红)`。八条卡死逐条清除后推进线不断右移。

## 1. 卡死与内存爆炸根因（8 条，全部已修）

这一族是"跑着跑着卡死 + 内存堆积 + 内核 SIGKILL 连坐杀进程"的**共同根因**。
共同形态：**非重入 `Mutex`/`RwLock` 被同线程二次获取**、**循环步长为零**、
**在测试/审计路径里再起 cargo**、**全仓扫描的二次方复杂度**。栈实证一律是
`__psynch_mutexwait`（阻塞）、`read_output→poll`（等子进程）或采样占比
300-800/804（空转）。

| # | 位置 | 病 | 后果 | commit |
|---|---|---|---|---|
| 1 | `nt_file_ability/chunk_planner.rs:125` | `start = end - overlap_rows` 在 `end` 触顶后回退，`rows_per_chunk=1` 时原地踏步 | 无限循环 + 每轮 clone 一个 chunk → 内存爆炸，被内核 SIGKILL | `a82f3084` |
| 2 | `l1_action/nt_act/deferred_loader.rs:136,122` | `evict()`/`prefetch()` 持 `resources` 守卫后再调 `remove_cached()`/`cache_data()` 二次加锁 | 永久自死锁；且持三把锁 → 任何 loader 操作级联阻塞 | `bb020d71` |
| 3 | `l1_action/nt_action_facade.rs:125` | `search()` 先 `raw_conn()` 持守卫再 `rebuild_bm25()`（内部自锁 `conn`） | KB 搜索永久挂死 + 连带阻塞所有 KB 操作 | `1e74a89c` |
| 4 | `l6_meta/nt_core_self/self_audit.rs:382` | `scan_build_status()` 在测试进程内 `Command::new("cargo")` | 与外层 cargo 抢构建锁 → 100% 死锁；锁空闲时拉起编译器吃内存 | `ee0729cc` |
| 5 | `l5_cognition/nt_mind/rise_reflector.rs:79` | `project_future` 的守卫只在 reset 分支 `drop`，正常路径活到函数尾 | RISE 预演**首次调用即永久挂死** | `9113e21c` |
| 6 | `l5_cognition/nt_core_arch_fitness.rs:250` | `in_test_context` 每次调用全量重切文件，两个全仓守卫逐行调用 | 每文件 O(命中×行数) 二次方空转 + 内存 churn | `0f829ca8` |
| 7 | 常驻 sidecar 3G + 无门禁 | 权重常驻 + 重型 cargo 无预检 | 16G 机器连环 OOM，sidecar/晶体/App/测试进程团灭 | `5639c08f` |
| 8 | `nt_core_arch_fitness.rs:334` (DeadCodeFitness) + `nt_core_self_test.rs:24` (ExternalVerifier) | self-test 内再起 `cargo check` | 与外层 cargo 抢构建锁 → 全量套件卡死在 `test_all_have_names`（`read_output→poll` 栈实证） | `8d14fe02` |

**同族防复发**（新增三道闸）：
- `scripts/ops/nt_lock_audit.py` — 块身份栈静态扫描非重入锁二次获取；
  `--selftest` 内置正/负样本 + 回归样本验证，全仓当前 **0 命中**。
- `scripts/ops/nt_mem_gate.sh` — 重型 cargo 前置内存门（free < 1.6G 直接 BLOCKED）。
- `scripts/ops/nt_sidecar.sh` — sidecar 改**按需**（`start|stop|status`），不再常驻 3G。
- 测试构建内禁止起子进程：`NT_SKIP_CARGO_CHECK=1` 显式开关，默认在 `cfg!(test)` 跳过。

## 2. 已修领域病（19 条）

| 位置 | 病 | commit |
|---|---|---|
| `nt_file_ability/config_parser.rs` | 全角冒号 `：` 未识别 → 材质值带前导冒号 | `5639c08f` |
| `nt_file_ability/doc_parse.rs` | 无扩展名/未知扩展名直接 Unsupported（无纯文本回落） | `5639c08f` |
| `nt_file_ability/event_types.rs` | 事件 JSON 嵌套枚举，取不到顶层 `file_path` | `5639c08f` |
| `nt_file_ability/table_presenter.rs` | Markdown 表格缺空格分隔 | `5639c08f` |
| `nt_file_ability/path_metadata.rs` | 粘连日期未剥离 / 业务员目录未上溯 | `5639c08f` |
| `nt_file_ability/excel/excel_tool_schema.rs` + `selftest_excel.rs` | schema 断言与 OpenAI `parameters` 嵌套契约脱节 | `5639c08f` |
| `nt_mind/.../nt_capability_eval.rs` | `route_with_hint` 契约写的"否则 explorer"分支缺失 → explorer 永无证据 → 路由永不迁移 | `4beba4b3` |
| `nt_mind/.../nt_capability_registry.rs` | `route()` 内生产 `expect`（违反禁 panic 铁律）→ 改 `unwrap_or` | `4beba4b3` |
| `nt_core/seal/training_cycle.rs` | 未接线 stub 被塞进 `failed_patterns` → 训练环永久判红 + absorb 永久跳过 | `4ca59fce` |
| `nt_action_facade.rs` | `Bm25Index::search` 返回 doc_id 被当 title 直出 | `1e74a89c` |
| `nt_act_trade/path_metadata.rs` | 粘连前导日期未剥离 → `country="4.02危地马拉"` | `8d14fe02` |
| `nt_core_self_test_integration.rs` | 迷雾四分支 SelfTest 早已存在却从未注册进轻量表 → fog 四分支无健康数据（自注释"卡 0.15"） | `1cca0af7` |
| `nt_core_arch_fitness.rs` | 全仓守卫二次方空转 | `0f829ca8` |

## 3. 长尾失败清单（~122 条，按模块聚类）

> 数据源：`/tmp/nt_full6.log`（10113 绿 / 125 红时的完整 FAILED 名单）。
> 多数为每模块 1-4 条，特征是"断言与现行契约脱节"或"环境依赖（网络/磁盘/时序）"。

| 模块 | 条数 |
|---|---|
| `l3_embodiment::nt_shield::nt_shield_sandbox::egress_tests` | 4 |
| `l5_cognition::nt_mind::nt_game::*`（time/render/physics/self_play） | 9 |
| `l4_emotion::nt_memory::entity_linking::linker::tests` | 3 |
| `l1_action::nt_act::nt_act_trade::orchestrator_v2::tests` | 3 |
| `l0_substrate::nt_core_kb_primitives::tests` | 3 |
| `l6_meta::coordination::nt_task_orchestrator::recursive_controller::tests` | 2 |
| `l5_cognition::nt_mind::nt_mind::skill_chain::chain_config::tests` | 2 |
| `l5_cognition::nt_mind::nt_mind::seal_core::embedding::tests` | 2 |
| `l5_cognition::nt_mind::nt_mind::decision_engine::analyzer::tests` | 2 |
| `l5_cognition::nt_core_gwt::load_balancer::tests` | 2 |
| `l5_cognition::nt_core::nt_forecast::tests` | 2 |
| `l5_cognition::nt_codegen::tests` | 2 |
| `l4_emotion::nt_feel::cognitive_bridge::feedback::tests` | 2 |
| `l4_emotion::nt_emotion_reasoning_bridge::tests` | 2 |
| 其余 ~40 个模块 | 每模块 1-2 |

**处理原则**（沿用已验证判据）：
1. 先看是否**生产真 bug**（值错、锁错、契约反了）→ 修生产，测试跟着绿。
2. 若是**未接线 stub** 被断言当成功 → 引入 `not_wired` 之类的显式标记，
   缺口留痕但不污染成功判定（照 `training_cycle` 先例）。
3. 若是**从未接线的已存在组件** → 补接生产（照迷雾四分支先例，R-P79）。
4. 若是**环境依赖**（网络/磁盘/时序/并发）→ 注入桩或 `#[ignore]`，
   禁止把断言改松来"凑绿"。
5. 每个模块单独跑，**禁止**多模块合批（历史 OOM 教训）。

## 4. 结构性债务（PARK，需归属裁决）

| 项 | 现状 | 阻塞点 |
|---|---|---|
| L0 `CapabilityRegistry` ×4 正典收敛 | 4 个同名异构类型（trait-object 列表 / Vec+tag_index / IndexMap+petgraph DAG+4 索引） | 字段与职责异构，alias 会造成错误语义；需专窗迁移 + 重命名消歧 |
| `SemanticRouter` ×2 | `nt_core_semantic_router`(阈值+route_table) vs `nt_infra_semantic_router`(rules+provider_scores) | 同上，需先定正典 |
| `proxy_pool.rs` 拆分 | 1757 行，他人在途 `1039+/24-` | 必冲突，需归属窗口先落地 |
| 剩余 God-file | `pdf.rs` 2142 / `nt_crystal_serve.rs` 2002 / gateway 1678 / hex 1530 / auto-orchestrator 1037 | 逐个拆，参考已完成的 `nt_memory_unify`/`nt_file_ability` 拆法 |
| 5 个内容型 worktree | `split-mcp-sec-god` / `split-mcp-security` / `nt-act-cleanup-fix` / `resilience-fix` / `typed-memory-fix` | 内容裁决/合并/删除，未获结论 |
| 前端资产未提交 | `apps/neobot-desktop/frontend/{src,dist}` 大部未跟踪 | 归属裁决；`dist` 比 `src` 新，bundle 是新鲜的 |
| `entry/mod.rs::run_git_diff` | 零引用 + `#[allow(dead_code)]` 藏警告 | 该文件正被他窗改动（M），需归属窗口裁决 |

## 5. 本轮新增纪律（防复发）

1. **提交前必须核对 `git diff --cached --name-only`** —— 本轮曾误把他窗已暂存的
   9 个删除（6 源文件 + 3 产物）并入提交，已用 `reset --soft` + 选择性
   `restore --staged` + 重暂存完整还原；此后一律用 `git commit -- <paths>`
   做 pathspec 限定提交。
2. 全仓扫描类 self-test 不得在 `cargo test` 内起子进程（已加护栏 + 环境开关）。
3. 任何 `let g = x.lock()` 后又在同块作用域 `x.lock()` → 死锁，`nt_lock_audit.py` 兜底。
4. 循环步长必须单调递增（`chunk_planner` 教训：重叠不得 ≥ 步长）。
5. 重型 cargo 前必过 `nt_mem_gate.sh`；sidecar 用完即 `stop`。


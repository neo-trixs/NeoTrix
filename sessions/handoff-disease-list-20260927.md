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


---

## 6. 长尾分诊结果（4 个只读子代理并行拆解，125 条全覆盖）

分诊判据：**P**=生产 bug / **S**=测试契约漂移 / **U**=未接线 stub / **E**=环境依赖。
并行方式：4 个 explore 子代理只读分诊（零 cargo），构建验证单线串行（16G 铁律）。

### 6.1 本轮已修（16 处生产 bug，均编译+测试双验或编译已验）

| 位置 | 病 | 类别 | 验证 |
|---|---|---|---|
| `nt_shield_sandbox/mod.rs:108` | `matched_allow \|\| !deny_all` → 任何白名单策略全放行 | P·安全 | check ✓ |
| `actions/security/security.rs:106` | `target_pattern="*"` 用 `contains` 字面量比对 → 通配规则永不 deny | P·安全 | check ✓ |
| `nt_io_browser_engine/fetch.rs:278` | 显式代理 URL 解析失败被 `.ok()` 吞 → 静默直连（违反 fail-closed） | P·安全 | check ✓ |
| `shield/circuit_breaker.rs:156,226` | `Instant::now().duration_since(Instant::now())` 当时间戳 → 半开恢复永不触发，熔断永久锁死 | P | check ✓ |
| `shield_approval/human_approval.rs:118` | `conf` id 恒为 `conf_0` → 并发审批撞号覆盖 | P | check ✓ |
| `asset_map/query/mod.rs:95,109` | `parse_or/and` 判定前未跳空白 → 复合查询条件被静默丢弃 | P | 125 绿 ✓ |
| `asset_map/query/mod.rs:372` | `Like` 无通配符退化为精确 `==` | P | 125 绿 ✓ |
| `source/subscription_source.rs:511` | 入队后不清缓冲 → 闭合标签重复推送同一条目 | P | 125 绿 ✓ |
| `nt_world_repomap/mod.rs:83` | 函数分支漏剥 `"fn "` → 函数名不可检索 | P | 125 绿 ✓ |
| `osint/fofa.rs:393` | domain 判定先于 email → 邮箱目标生成不了 `email=` 查询 | P | 125 绿 ✓ |
| `osint/whois_module.rs:78` | 在小写副本上取值 → 实体名被小写化 | P | 125 绿 ✓ |
| `sense/types.rs:126` | 双零向量余弦 = 0 → "相同状态相似度 0" | P | 125 绿 ✓ |
| `source/crypto.rs:11` | 32 字节 MD5 hex 喂 AES-128 → 一调就 panic | P | 125 绿 ✓ |
| `entity_linking/linker.rs:94,197` | 提及去重含 offset → 同名实体不合并 | P | 125 绿 ✓ |
| `entity_linking/extractor.rs:173` | 地点去重缺类型 → 被 Person 认领后静默吞掉 | P | 125 绿 ✓ |
| `entity_linking/linker.rs` | 补缩写匹配（MIT ↔ Massachusetts Institute of Technology） | P·新能力 | 125 绿 ✓ |
| `nt_memory_kb/cognitive_graph.rs:168` | `trace_causal` 边方向与 `reasoning_gaps` 相反 + 漏 `InferredFrom` | P | 125 绿 ✓ |
| `nt_core_consciousness_types.rs:353` | effort 加权上限 0.7 → `EffortTier::Max` 死代码 | P | 125 绿 ✓ |
| `nt_core_kb_primitives.rs:211` | `nodes.transaction_time` 缺 `DEFAULT 0`（edges 侧一直有） | P | 125 绿 ✓ |

### 6.2 待修（已分诊，按建议顺序）

**A. 一行级生产 bug（低风险高回报，建议下一批）**
- `chain_config.rs:31` `"Tdd"` → `"TDD"`（步骤超时表对 TDD 整体失效，2 测试）
- `nt_core_consciousness_tree.rs:31` 硬编码 11 分支 → `BranchKind::all().len()`（**这是出厂 self-test**，运行时健康门常红）
- `nt_core_parallel/isolation.rs:764` 测试路径已失效（`l8_autonomic_impl` → `l5_cognition/...`）
- `resource_budget.rs:338` `total_cost` 过滤掉非 `CostUSD` 行的 cost → 真实花费报 $0
- `nt_infra_breaker.rs:77` 窗口未满即评估 → 单次瞬时错误即熔断
- `nt_conversation.rs:25` 秒级 id → 同秒会话互相覆盖（数据丢失）
- `goal_lock/mod.rs:73` 缺 `unsafe` 关键词 → 危险回应漏判
- `mock_adapters.rs:674` harness 第 3 步必 `Err` → 集成测试永远不可能 Ok
- `orchestrator_v2.rs:858` 空结果分支丢 `worker_count`/`all_success` 字段
- `routing/intelligence.rs:225` 超预算固定扣 50 分 → 1.7x 与 33x 超预算同分，路由不确定
- `decision_engine/analyzer.rs:94` `gap < 0.01` 漏 0.01 差值；`:116` Medium 档不可达
- `render/physics.rs:207` `layers_compatible` 混淆 layer/mask → 跨层碰撞全部漏判
- `render/ui.rs:369` Anchor 算了 ax/ay/aw/ah 却不写回 bounds
- `persona_routing/mod.rs:122` 缺"设计"关键词
- `nt_cot_generator.rs:156` 所有 `LlmError` 一律映射 Network
- `llm_types.rs:65` `with_image_b64` 未补 `data:` 前缀（调用方注释却这么声称）
- `dynamic_memory_bank.rs:273` 相似度硬顶 0.6 < 阈值 0.7 → `_retrieve_identity` 永不返回
- `self_improvement.rs:667` `.round()` → `.floor()`（1-10 优先级差一档）
- `l6_meta/.../recursive_controller.rs:43` `atomize` 忽略 `;` 分隔 → 多任务树不分解
- `rotation_coordinator.rs:124` 返回缓存值 → 轮换抖动是死的
- `reasoning_protection.rs:77` `line_count <= 3` 时 CoT 原样输出，无省略标记
- `refusal_tamper.rs:50` 映射键是整句 → 请求后缀被吞
- `infrastructure_mapper.rs:222` 账号/IP 比阈值 5.0 过严（4.0 的极端信号被拒）
- `red_team/orchestrator.rs:79` 关键词表缺 `ignore all previous` 等 → 模拟器自己标 VULNERABLE 却得分 0

**B. 需设计裁决（不宜一行改）**
- `nt_core_kb_primitives.rs:188` `nodes.id` 是单列 PK → 双时态版本化不可能（需 PK→(id,transaction_time) + edges FK 重构 + 真实库迁移）
- `cascade/cascade.rs:216` `length_score = len/200` 过小 + `tick():126` 永久丢弃未达阈样本
- `nt_shield/circuit_breaker` 之外三处 `Severity` 派生 `Ord` 反向（`compliance/requirement.rs:9`、`threat_modeler.rs:29`、`neotrix-types/shared_types.rs:6`）→ 翻转会改变全局排序，需调用点审计

**C. 未接线 stub（要么实现要么 `#[ignore]`，禁止改松断言）**
- `nt_core_llm/mod.rs:91` `apply_context_budget` 空实现（上下文预算完全不生效）
- `nt_core_embed/mod.rs:12` `TextEmbedder` 是"字节位置袋"兼容桩 → 任意文本相似度≈0.83
- `reason/sleep/hebbian.rs:104` `consolidate_to_capability` 硬编码 0.0
- `nt_emotion_reasoning_bridge.rs:87` `process_events` 从不写入 `self.emotions` → 后续全空转
- `noise_handshake.rs:269` 缺 `_create_message3` → 握手无法完成
- `publish_gateway.rs:211` YouTube 上传未接线
- `dynamic_memory_bank.rs:253` `calculate_semantic_similarity` 只比 description 不比 name
- `nt_codegen.rs:54` `parse_yaml` 误用 `serde_json`（需引 serde_yaml）

**D. 环境依赖（注入桩或 `#[ignore]`）**
- `video_stitcher.rs:192` 真调 ffmpeg + 不存在的输入路径
- `experience_tree/mod.rs:896` `KnowledgeBase::open(None)` 命中真实 `$HOME` 库 + flock
- `nt_memory_galaxy_hygiene.rs:659` 硬编码 `2026-08-11` 已过期 47 天
- `governance/enforcement/audit.rs:125` `timestamp_now()` 秒级 → 两次 log 同秒
- `subdomain_harvester.rs:171` `.invalid` 域名依赖活体 DNS
- `nt_forecast.rs:768` `with_llm_narrator(None)` 仍发真实 LLM 请求（含 sleep/重试）

## 7. 并发纪律（本轮新增教训）

1. **他窗并发重构会周期性把 test 构建弄红**（本轮观测 3 次：`test_extractors.rs` 缺类型 →
   `test_orchestration.rs` 私有方法 → `nt_feel::writing_style` 未接通）。期间：
   - 用 `cargo check -p neotrix --lib`（排除 test cfg）验证生产改动；
   - 提交门禁会拒绝（`BUILD GATE FAILED`）——**这是正确行为，禁止 `--no-verify`**；
   - 等对方提交后重跑测试再提交（本轮最终 125 绿后放行）。
2. 内存门必须**每次发构建前看退出码**，不能只看输出行（本轮我漏看一次，
   在 180MB 空闲时起了 rustc，侥幸未 OOM）。
3. 提交一律 `git commit -- <paths>` pathspec 限定，避开共享暂存区。

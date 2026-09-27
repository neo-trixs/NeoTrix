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

---

## 8. 第二轮并行修复（4 代理并行编辑 + 单线验证）

**并行方式**：4 个 `general` 子代理各领一组**文件互斥**的清单，只做定点 `Edit`，
禁 cargo / 禁 git / 禁 Write；构建验证由主代理单线串行执行（16G 铁律）。
产出：4 个 stub 补实现 + 6 个环境依赖去抖动 + 12 处契约对齐 + 5 处新发现真 bug。

### 8.1 补上的真实现（此前是空壳）
- `nt_core_llm::apply_context_budget` — **上下文预算此前完全不生效**（直接 `default()`）。
  现按 W1.1 规则驱逐最旧非 System 消息（尾部当前请求永不驱逐），填 `is_cliff`/`messages_evicted`。
  签名改 `&mut Vec<Message>`（两处调用点本就传 `&mut`）。
- `nt_core_llm::estimate_tokens` — 字节数 `/4` 把汉字按 3 字节高估 3 倍；改 CJK 感知。
- `nt_core_llm::truncate_preserving` — **按字节切片会切在多字节字符中间 → panic**（违反禁 panic 铁律）；改 char 边界单调回退。
- `reason/sleep/hebbian.rs` — `consolidate_to_capability` 硬编码 0.0 → 确定性 Hebbian 强化（argmax 峰值维）。
- `dynamic_memory_bank` — 语义相似度只比 `description` 不比 `name` → 双路检索恒失效。
- `nt_emotion_reasoning_bridge` — `process_events` 从不写 `self.emotions` → 后续全空转，事件→情绪未接线。
- `governance/enforcement/audit.rs` — `timestamp_now()` 秒级 → 同秒两次 log 同时间戳；升纳秒 + 严格递增。
- `nt_forecast::with_llm_narrator(None)` — **None 仍发真实 LLM**（含 sleep/重试）；现真短路到确定性回退。
- `blank_space_checker::calculate_score` — 计分方向反了：全通过得 20 分（`100 - score`），`score>=80` 对任何合格夹具永不可达。

### 8.2 本轮新发现的真 bug
- **`inventory::add_item` 把 `ItemStack::add` 的"剩余量"当成"已加入量"从 remaining 里减**
  → 合并成功时 remaining 不归零，继续开新栈：加 5 再加 3，`count_item` 得 **11** 而非 8。
  （`ItemStack::add` 返回剩余量的契约由 `test_item_stack_add_capped` 钉住，是调用方用错。）
- `time_system` 魔法 `*10.0`（1 秒 = 10 游戏分）违反文档时钟。
- `vsa.rs` `diversity = distinct/64` 硬编码分母，恒达不到 0.5 阈值。
- `self_play_loop` Phase 2 把轨迹重复入队 → 缓冲区翻倍、半数样本 `advantage: None`。
- `load_balancer::compute_load_fractions` 无视真实选择记录做 tie-break argmax → 负载熵恒 0。
- `cad_route` 重复注册 `ImageGenerator`（`module_index` 按 `specialist_type` 索引，重复槽不可路由）。

### 8.3 环境依赖去抖动（6 项）
`video_stitcher`（真调 ffmpeg + 不存在路径 → 改断言命令生成）、`experience_tree`
（`KnowledgeBase::open(None)` 命中真实 `$HOME` 库 + flock → 改 `:memory:`）、
`galaxy_hygiene`（硬编码 `2026-08-11` 已过期 47 天 → 改相对时间）、
`subdomain_harvester`（`.invalid` 依赖活体 DNS → `#[ignore]` + 补确定性用例；
`SubdomainSource` 补 `#[serde(rename_all="snake_case")]` 与 `Display` 对齐）、
`audit` 时间戳、forecast narrator。

### 8.4 三处"看起来该翻、其实不能翻"
- `shared_types::Severity` 的派生 `Ord`（判别序 Critical=0 最"小"）看着反了，
  但 `l2_perception/nt_world/osint/sweep.rs:225` **显式依赖**该约定做 `min_severity` 过滤
  → 翻转会静默反转过滤器。改为让 cvss 测试对齐既定约定。
- `load_balancer` 均匀态辅助损失恒等于 `aux_loss_coef`（N·Σf·P = 1），不是 1/N；改测试。
- `estimate_tokens` ASCII 段取整方向：测试自称"单一事实源 P0-7"要求 11 字符 = 2 token
  （对齐 tiktoken），故用 floor 而非 ceil。

### 8.5 全量推进
`10113 绿/125 红` → `11339 绿/87 红`（本轮中途中断点）→ 第四轮全量进行中。

---

## 9. 完整后续待修复清单（51 条，逐条带根因与 file:line）

> 快照：`/tmp/nt_full8.log`（HEAD `51355bca`，全量 11493 绿 / 51 红）。
> 分诊：3 个只读代理并行拆解，**51/51 全覆盖**，分类 P=生产 bug / S=契约漂移 / U=未接线 stub / E=环境依赖。
> 分布：**P 17 · S 27 · U 4 · E 3**。

### 9.1 P · 生产 bug（17 条）——优先做

**P0 安全/正确性（10 条，多为一行改）**
| # | 位置 | 病 |
|---|---|---|
| 1 | `nt_core_capability/dependency.rs:136,154` | 拓扑排序**恒失败**：入度记在依赖侧、却从反向边递减 → 队列永不播种，`get_load_order`/`load_all` 永远 `Err` |
| 2 | `nt_core_capability/integrator.rs:30` | router 持 `init_*_capabilities` **之前**的 registry 克隆 → `route` 恒"无匹配能力"，整个 integrator 是空转 |
| 3 | `nt_agent_identity.rs:497` | 手搓 `uuid_v4()`（时钟+计数器）同一 tick 内碰撞 → `register` panic（也是测试抖动源）。换 `uuid::Uuid::new_v4()` |
| 4 | `nt_meta/gwt_router/cost_weight.rs:130` | 预算上限被静默丢弃，空 `budget_ok` 不兜底 → `RoutingReason::BudgetConstraint` 分支不可达 |
| 5 | `nt_meta/gwt_router/attention.rs:91` | 第 4 步按 `frac_sum` 重新归一化**抵消了上限**（0.6/0.7=0.857 ≫ 0.6）→ 需 water-fill 而非按和除 |
| 6 | `nt_core_self/dynamic_params.rs:188` | 解析器按行读，而 `format_as_description:221` 输出一行逗号串 → **读不出自己写的格式** |
| 7 | `nt_feel/cognitive_bridge/feedback.rs:101` | `Satisfaction｜Joy` 分支吸收失败信号 → 失败时**永远不会升级为 Frustration**（违背结构体文档） |
| 8 | `nt_memory/cascade/cascade.rs:128` | 晋升门用 `distill_threshold`(0.5) 而非 `attention_threshold`(0.3)，实测 attention=0.42 → **五级记忆流全死**，跨层召回恒空 |
| 9 | `nt_memory_kb/memory_orchestrator.rs:70` | `compress()` 触发线在 50 条而 `add_raw:62` 封顶 100 → 常规会话永不压缩 |
| 10 | `nt_feel/writing_style.rs:903` | 空串守卫被 `a==b → 1.0` 快路径遮蔽 → 退化输入返回 1.0 而非 0.0 |

**P1 需多一点工作量（7 条）**
| # | 位置 | 病 |
|---|---|---|
| 11 | `l2_perception/nt_core_vector_store/store_hnsw.rs:83-87` | 索引恒按**余弦**排序，但 `IndexConfig::default()` 是 **Hamming** → 报告的 distance 与排序自相矛盾（真索引 bug） |
| 12 | `l0_substrate/nt_core_speculative_decoding.rs:250` | `speedup = total/verification_count`，而每次验证只出 1 token → 恒等于 1.0。分母应为 `rejected+1` |
| 13 | `l5_cognition/nt_codegen.rs:54` | `parse_yaml` 调 `serde_json::from_str`（`.yml` 解析必炸）。需加 `serde_yaml = "0.9.34"` 到 workspace + core 两处 manifest（Cargo.lock 已有，可离线） |
| 14 | `nt_core_capability/monitor.rs:137,197` | 无 Call 事件时**静默丢弃** latency；`get_status` 只扫 registry → 指标不聚合 |
| 15 | `nt_core_capability/security.rs:257` | `check_ip` 仅精确串匹配，生产无任何 CIDR 输入 → IP 白名单形同虚设（需真 CIDR 匹配） |
| 16 | `shared_types.rs:130`（潜伏） | `on_failure` 在未达阈值时就置 `HalfOpen` → `half_open_probes_*` 统计恒死 |
| 17 | `nt_memory_kb/nt_memory_distill.rs:316` | teacher 标签 = 原始点积，与恒等 student 完全一致 → 误差恒 0、梯度恒 0，`after<before` 不可满足 |

### 9.2 S · 契约漂移（27 条，改夹具不碰生产）

- **⚠️ 由我本轮安全修复暴露的 2 条（重点）**：`nt_shield_sandbox/stateful_bench.rs:187,205-208,328`
  的 S4 场景**把修复前的"有洞"极性写成了断言**（`deny_all=true` 配非空白名单，
  修复后=白名单语义，`random.host` 应被拒，但步骤期望放行）。修法是互换两处策略的
  布尔值，让场景表达"显式白名单不放行未知主机"。**这两条等于在给漏洞背书，必须改。**
- `nt_core_aware` 4 条：`eps` 口径（1e-6 vs 1e-7）、`Default.health=1.0` 由**另一个通过
  的测试**钉住、`>` vs `>=` 边界、权重和为 1.0 算出的期望值写错。
- 熔断器 3 条（`shared_types.rs:97`、`self_healing/circuit_breaker.rs:117`）：**单位漂移**
  —— `new(1,1)` 的冷却是 1 **秒**，测试只 sleep 2 **毫秒**。
- `predictive_maintenance` 2 条：`detect()` 是移动平均 z-score，测试注入的 150.0 是 13.9σ
  而非"异常但非致命"，测试自己的算术错了。
- `nt_core_capability` 3 条：`Domain::Trade` 加入后域列表仍断言 11 项（两份重复的
  `layer_domain_coverage`）；`record_latency` 测试没注册 mock cap。
- `nt_feel` 4 条：负分可读度是**有意设计**（有 Pre-K 档）、3-gram 数被 `take(5)` 截断、
  TTL 是插入锚定（`get` 也计 turn）→ 用 `new(10, 3)`。
- `distiller.rs:252`：守卫是 `s.len() >= 4`，`"Short"` 5 字符也过（另：`len()` 是字节，
  1 个汉字也能过 → 应改 `chars().count()`）。
- `shield_core/audit.rs:1176`：夹具 `sk-test123` 只有 7 字符，而规则要求 `sk-` 后 ≥20。
- `shield_core/safety_kernel.rs:727`：`FileDelete` 在破坏性闸门就短路，根本到不了风险分
  分支 → 断言需含 "explicit confirmation"。
- `compliance::requirement.rs:191` / `threat_modeler.rs:180`：**这两个是各自独立的本地
  enum**（非 shared `Severity`），且**全仓无任何排序/比较调用点** → 可安全对齐断言
  （与 `osint/sweep.rs` 那个承重约定不同）。
- `parallel_task.rs:380`：夹具 `max_retries: 3` 让失败依赖被重排队，生产依赖阻塞逻辑**是对的**。
- `cad_full_wiring_verification`：见 9.4 决策项。

### 9.3 U · 未接线 stub（4 条）——实现或显式 `#[ignore]`，禁止改松断言
| 位置 | 缺口 |
|---|---|
| `nt_core_embed/mod.rs:12-20` | `TextEmbedder` 是"字节位置袋"→ 任意英文文本相似度 ≈0.83，排序无意义。**可在 ~30 行内做真实现**（分词 + FNV-1a 哈希 + 符号哈希 + L2 归一，零新依赖）；但它有 5 个生产调用方，改 `embed` 会重排全部检索结果 → 需评估 |
| `noise_handshake.rs:226` | `create_message3` 是 **private** 且其输出在 `_consume_message2` 里被 `let _ = msg3;` **丢弃** → responder 永远到不了 `Completed`；测试用 `[0u8;48]` 占位 + `unwrap_or_else` 吞错掩盖了它 |
| `publish_gateway.rs:205-212` | 5 个平台臂全部硬编码"not wired"（YouTube 无 dry_run/mock 路径） |
| `nt_codegen` / `nt_memory_distill` | 见 P 13 / P 17 |

### 9.4 需人工决策（3 条，不可机械修）
1. **双时态节点 schema**（`nt_core_kb_primitives.rs:188` + `:685-691`）
   `id TEXT PRIMARY KEY` 使同一节点的多版本无法共存。改复合 PK `(id, transaction_time)`
   需 v11 迁移 + 重指 4 个 FK（`:237,238,258,277,347`）；另一条路是承认"每 id 最新版本"
   这个 API 契约是虚构并重写两个测试。**注意：`nodes_as_of`/`node_history` 目前零生产调用方** ——
   所以真正的决策是"给它接调用方，还是删掉"。
2. **CAD self-test 来源可疑**（`nt_core_cad_consciousness.rs:332-336`）
   `CAD_SELFTESTS` 12 项里 8 项指向本仓不存在的 `neotrix/l2_world_impl/*.rs`
   （注册处 `nt_core_self_test_integration.rs:26,34` 已被注释为 "module not found"），
   列表里还有字面垃圾条目 `"// // cad_generator"`。更严重的是 `cad_wiring_map():164-209`
   把这些**死 file:line 当作"接线证据"**喂给 D16 晋升门 —— 这是自我欺骗面，
   **假"通过"比红测试更糟**。要么重新吸收模块，要么砍掉证据表。
3. **publish gateway**（见 9.3）—— 要么投 OAuth2 + `reqwest` 可续传上传，要么加一等公民
   `dry_run` 让测试走任务生命周期；直接删测试等于藏掉文档里承诺的能力。

### 9.5 E · 环境依赖（3 条）
- `healing/self_healing/health_monitor.rs:117-126` 2 条：5 个内建探针走**活体** sys-info
  （本机 mem/cpu/disk 直接踩 Critical 阈值）→ 注入固定状态集或 `#[ignore]`。
- `nt_core_guardian/mod.rs:75` 1 条：`results.len() <= 1` 取决于真实 `df`/`vm_stat`/
  `$HOME/.neotrix/knowledge.db`/`target/debug` 状态。
  **⚠️ 附带发现的高危项：`nt_core_guardian/repair.rs:119` 会执行 `cargo clean`** ——
  这条一旦被触发就会删掉 63G 活指纹，建议立刻单独加门禁。

---

## 10. 第三轮并行修复（28 项）— 已改未验，因内存门阻塞

**环境阻塞**：另两个窗口的 agent 进程 + 供 App 使用的 9B `llama-server` 常驻，
free 内存长期 < 1.6G 门限（最低观测 41k 页 = 660MB），按 R-BUILD-2 禁止起 cargo。
`llama-server` 跑的是 `qwen3.5-9b-fable`（供 NeoBot 本地推理），**不可停**。

**已完成编辑（29 项，working tree 内，未提交）**：

先做的高危项：
- `l6_meta/nt_core_guardian/repair.rs` — `ClearCache` 动作原本执行 **`cargo clean`**
  （会删掉整个 target/ 含 deps 活指纹，一次自愈动作即让全量重编数小时）。
  改为只删 `target/<profile>/incremental`（纯派生产物），并如实上报成败。

代理 A（l6_meta，8 项）：拓扑排序恒失败（入度记在依赖侧却从反向边递减，计数侧与递减侧
都错）｜integrator 持注册前的 registry 克隆致 route 恒"无匹配能力"（新增 `sync_router`）｜
预算上限静默丢弃（空 `budget_ok` 现在返回 BudgetConstraint 决策）｜attention 归一化抵消
上限（改 water-fill，保留按和除作为全顶格退化回退）｜dynamic_params 读不出自己写的
逗号串格式（且 `split_whitespace().nth(1)` 读不了"速度0.5s"无空格值）｜uuid 同 tick 碰撞
换 `Uuid::new_v4`｜两份 `layer_domain_coverage` 补 `Domain::Trade`｜nt_core_aware 4 处夹具

代理 B（l4_emotion，9 项）：失败信号不再被 `Satisfaction｜Joy` 吸收（一行修两测）｜
cascade 晋升门改用 attention_threshold，五级记忆流恢复｜空串守卫移到快路径之上｜
memory_orchestrator 压缩触发线｜可读度负分是设计（Pre-K 档）｜3-gram 被 take(5) 截断｜
TTL 插入锚定改 new(10,3)｜distiller 守卫改 `chars().count()`（原 `len()` 是字节，
1 个汉字就能过）｜nt_memory_distill teacher 改 `dot*0.5`（原标签与恒等 student 完全一致，
误差恒 0、梯度恒 0）

代理 C（l3 shield，7 项）：**stateful_bench S4 互换两处策略布尔值**（该场景原本把安全
修复前的"有洞极性"写成断言，等于给漏洞背书）｜noise_handshake 的 msg3 由 private +
丢弃改为可取用（加 `pending_message3` / `_take_message3()`，测试改喂真 msg3 并断言
不可重放）｜两处**本地** Severity 枚举对齐声明序（非 shared 那个承重类型）｜
audit 夹具 token 补到 ≥20 字符｜safety_kernel 断言接受 "explicit confirmation"｜
shared_types `on_failure` 未达阈值不再降级 HalfOpen

代理 D（跨域，4 项）：speedup 分母改 `rejected+1`（原恒等于 1.0）｜HNSW Hamming 配置
改为图外精确扫描（原本图按余弦排、却报 Hamming 距离，自相矛盾）｜`serde_yaml` 入
workspace+core 两处 manifest（Cargo.lock 已有，离线可解）｜`check_ip` 实现真 CIDR 匹配
（并同样修 blacklist，否则 CIDR deny 静默失效）

**恢复方式**：`sh scripts/ops/nt_mem_gate.sh; echo $?` → 0 后单模块定向跑
（`cargo test -p neotrix --lib -- <模块前缀> --test-threads=1`），绿了再
`git commit -- <paths>`。**禁止 `--no-verify`。**

### 10.1 ⚠️ 共享暂存区风险（请其它窗口注意）

本轮 29 项修改处于**已改未验**状态且**未提交**（因内存门阻塞，无法跑门禁）。
若其它窗口执行 `git add -A` + 裸 `git commit`，会把这批**未验证**代码卷进它的提交。
请其它窗口一律用 `git commit -- <paths>` 做 pathspec 限定提交（R-GIT-2）。

**阻塞实测**（22:4x）：
- `nt_mem_gate.sh` → `free_pages=12954`（**207MB**），`exit=2` BLOCKED
- 占用方：3 个 `opencode` 进程 1.87G + 1.48G + 0.89G、9B `llama-server` 670M（供 App，不可停）
- 副作用：sidecar / crystal 均已停（sidecar 已改按需），App 仅 20MB
- **最低点 41k 页（660MB）** 出现在第三批代理执行期间

---

## 10. 交叉核对补记（cycle `audit0927b`，结构性审计侧，2026-09-27 晚）

本节由另一条线的审计补入，**不覆盖上文结论**，只标注「哪些已被另一条线修掉」
与「防复发闸的假阴性」。两轮的数字口径不同（本文 11493 绿/51 红起于更早的基线），
以各自实测为准；差异源于并行修复，不是一方算错。

### 10.1 本线已修（与上文清单交叉后）
- `l6_meta::runtime_monitor::{test_get_health, test_monitor_and_get_metrics}` — 上文 §0「环境依赖」列为探针挂起；实为 **真死锁**（非环境问题）：`monitor()` 的守卫活到函数尾，而 `check_thresholds()` 内部再 `self.metrics.lock()`，std Mutex 不可重入。已修。
- `auto_inspector::{test_inspect_all, test_inspect_compilation, test_inspect_dependencies}` — 上文 §1 #4/#8 同族（测试进程内起 cargo）；`run_cargo` 与绕过 helper 的 `cargo audit --version` 探测两处都已加 `cfg!(test) || NT_SKIP_CARGO_CHECK` 短路。
- `nt_act_trade/tests/` 3 文件 311 个测试从不编译 — 上文未列。已复活 223（107+116 全绿），删 88（测已被 3bba2507 删除的 `nt_mind::sales_coaching`）。
- `nt_feel::writing_style`（上文 §0 环境依赖 2 项之一）— 该模块 1,241 LOC 从未被 `mod` 声明，从未编译；接上后 0 error 0 warning。
- HEAD 曾**无法独立编译**（全新 clone 必失败）：已入库文件 `use crate::l6_meta::nt_approval::…` 而 `mod nt_approval;` 只存在于未提交工作区。已补 29 文件 + 2 处声明。

### 10.2 ⚠ 防复发闸的假阴性（本文最有价值的补记）
`scripts/ops/nt_lock_audit.py` 报「全仓 0 命中」，**但它漏掉了上述两处死锁**。
原实现只扫**单函数体内**的二次 `lock()`（词法性质），而「A 持锁 → A 调 `self.B()`
→ B 再锁同一把」是**调用图**性质，词法扫描看不到。

已补第二趟 `audit_indirect`（跨函数），selftest 同步扩到 2 正 2 负样本。
调优过程记录：初版 12 命中里 **11 条是假阳性**，逐类排除后剩 1 条，且经手工核实为真
（`kb_search.rs` `pq_search` 持 `self.conn` 守卫时 `return self.semantic_search(...)`，
后者内部再 `self.conn.lock()` → pq 无命中时永久死锁；与本文 §1 #3 同族第三处）。

给扫描器加调用边时踩的四个坑（都已写进代码注释）：
1. 必须按 **token 位置交错**处理锁/括号/调用，不能「先记锁再数括号」——
   否则同行 `{ let g = ..lock(); }` 被判成函数体级持有 → good 样本假阳性。
2. `if let Ok(g) = ..lock() {` 的守卫属**内层**块，要记 `depth+1`。
3. 显式 `drop(var)` 与深度无关，且要用 let 绑定的**变量名**索引到字段名——
   `LET_LOCK_RE` 已吃掉 `.lock()`，tokio 形态 `.lock().await` 的 group(3) 只剩 `self.field`。
4. 语句级临时守卫 `*self.x.lock() = v;` 在分号即失效，**绝不能**记为持有。

> 判据: 每加一类就往 selftest 补一个 good 样本, 假阳性必须当场归零 ——
> **报狼的闸会被直接关掉**。另: 凡闸声称 0 命中, 都要拿一个已知实例反证它一次。

### 10.3 其它侧写
- `claude` 观察: `kb_search.rs` 那处死锁在本文成文后 2 分钟被另一窗独立修掉
  （`drop(conn)` + 同款注释）—— 说明两侧清单确实在并行收敛，不应互相覆盖。
- `.githooks/{post-checkout,pre-merge-commit}` 是**悬空符号链接**（深度错 +
  目标已在 `d3bfb953` 被删），致「有未提交改动时禁止 `git reset --hard`」的护栏
  **一直没生效**。已取回脚本并修正深度；实测 981 个未提交改动时拦截成功。
- `.gitignore` 的 `tests/` `benches/` `examples/` 三条无斜杠模式匹配**所有层级**，
  屏蔽了 10 个源码目录；已跟踪文件不受影响故长期潜伏，表现为「`git add <tests>` 被拒」。

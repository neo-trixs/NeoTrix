# 未解决任务总清单（去重 + 前提核验）— 2026-09-28

> **本文是唯一权威任务台账。** 汇总自 9 份 `sessions/handoff-*.md`（8 窗口）
> + `docs/architecture/BATCH-FIX-CHECKLIST-2026-09-28.md` + `TODO.md`，
> **逐条去重并标注前提核验状态**。
>
> **方法论约束**（见 `LESSONS-20260928-fresh-checkout.md`）：
> - 标 ✅核验 = 本窗口在**干净检出**上实测；标 ⛔证伪 = **不要做**，照做会引入 bug 或重造已有物。
> - 交接文档之间**大量互相矛盾**（见 §6），本文以实测为准，不以"某文档写了"为准。
> - **凡数字必须带测量台**。本清单里 94 / 1,644 两处脏树数字已在
>   `bdf1e9f1` + `f23177de` 校正为 **102 / 1,646**。

---

## 0. 本窗口已完成（勿重做）

| commit | 内容 | 干净检出验收 |
|---|---|---|
| `bdf1e9f1` | 层归属门补上 `neotrix/` 第二棵树（129→**128 文件/42,070 行**）+ 干净检出重算基线 **102** | `--strict` 102/102 PASS RC=0 |
| `f23177de` | 门记录 94→102、1,644→1,646 | 纯文档 |
| `a9ad48f5` | **S-1** 并入车道 `lane/batch-fix-20260928`（11 commit，零 `.rs`），交付 4 份正典文档 | 4/4 文档在 HEAD，悬空引用 0 |

---

## 1. 🔴 最高优先：**37 个文件「已改未验」**（3 个窗口的 WIP 卡在树里）

> 这是**最大的单块未决债**，且它阻塞下面几乎所有 `.rs` 工作。
> 三个窗口都因内存闸 BLOCKED 而**从未编译过一次**。

| ID | 任务 | 证据 | 状态 |
|---|---|---|---|
| **V-1** | 晶体核心 6 文件编译验证（最后一个 E0716 修完未编译） | `neotrix/nt_crystal_core/{cocoons,mod,nt_awaken_loop,nt_graph_index,nt_jev_calibration}.rs` + 新增 `nt_premise_selector.rs`(883 行) | 🟢 **内存闸已 OPEN（rc=0）** |
| **V-2** | neobot 17 改 + 15 新增文件编译验证 | `crates/neotrix-neobot/src/**`（含 5 个新文件 `nt_channel*.rs` 等） | 🟢 同上 |
| **V-3** | 58 个推理链目标测试 | `sync_dedup`/`chain_persist`/`nt_premise_selector`(14)/`verify_incentive`/`nt_graph_index`(9) | 🟢 同上 |
| **V-4** | `Cargo.lock` 登记 `serde_yaml = "0.9.34"`（`parse_yaml` 修复的前置） | workspace + core | 🟢 离线可解 |

**验收命令**（内存闸 0 才起）：
```bash
sh scripts/ops/nt_mem_gate.sh; rc=$?; echo "gate=$rc"     # 注意：不可接管道，会取到 tail 的码
CARGO_BUILD_JOBS=2 cargo test -p neotrix --lib -j 2 -- nt_crystal_core --test-threads=2
```

### V-5 ⛔ **必须先裁决，否则 V-6 无法判真假**

三份交接各自要求的 `total_memories` 断言值**互相冲突**，且
**三个窗口的二进制验证命令一次都没跑过**：

| 来源 | 断言值 | 快照时刻 |
|---|---|---|
| medical | **63,747** | 51.6MB 时刻 |
| mimo-rlenv | **64,720** | 52.5MB 时刻 |
| crystal / hf-batch | **69,840** | 56.6MB 之后 |

⇒ **先在活库上重新导出真值**，再谈"确认 == N"。

---

## 2. 🟢 已核验前提、可立即执行（零风险删除 + 注释）

| ID | 任务 | 落点 | **本窗口核验结果** | 风险 |
|---|---|---|---|---|
| **B-1** | 删死 `CapabilityRegistry` | `neotrix/nt_file_ability/capability.rs:185` | ✅ **0 消费者**（`nt_file_ability::CapabilityRegistry` 精确搜索 0 命中） | 🟢 |
| **B-2** | 删死 `CapabilityRegistry` | `l5_cognition/nt_core/capability/registry.rs:462` | ✅ **死**：全仓唯一"引用"是它自己 `:454` 的注释 | 🔵 删前查 `mod.rs` 的 `pub use` |
| **B-3** | `neotrix-types` 包内 `SkillRegistry` 2→1 | `core/skill.rs:54` / `core/skills/mod.rs:25` | ✅ **确为包内重复**（全仓 4 份定义） | 🟢 |
| **B-4** | 给 `nt_core_gate/nt_tool_registry.rs` 加**防误删注释** | 文件头 | 活消费者 `nt_shield_enforcer.rs:388-400`（本窗口在分层门输出中亲眼见到） | 🟢 |

> **B-4 优先级高于 B-1/2/3**：它阻止下一个 agent 按已证伪的计划删活代码。

---

## 3. ⛔ **原「30 条一行级生产 bug」全部证伪 —— 27/27 已修或前提不成立**

> **2026-09-28 16:2x 逐条读现场核验，零确认缺陷。** 本文初稿照抄
> `handoff-disease-list-20260927.md` §6.2-A / §8.2 / §9.1，**该清单已过期**。
> 绝大多数缺陷已由 `f4a4eecd`（09-27「3代理并行修25处生产缺陷+7处夹具，154绿」，
> 25 文件正对应这批条目）修掉，且**已在 HEAD**（`git merge-base --is-ancestor` 确认）。

| 编号 | 台账声称 | 实测结论 |
|---|---|---|
| A-1 `chain_config.rs:31` | `"Tdd"`≠`"TDD"` | ✅ 已修为 `"TDD"`，测试钉死 |
| A-2 `nt_core_consciousness_tree.rs:31` | 硬编码 11 分支 | ✅ 已改 `BranchKind::all().len()` |
| A-3 `isolation.rs:764` | 测试路径失效 | ✅ 路径已更正且文件存在 |
| A-4 `resource_budget.rs:338` | `total_cost` 滤掉非 `CostUSD` ⇒ 报 $0 | ⛔ **前提错**：`total_cost` 是裸 `.map(\|u\| u.cost).sum()` **无 filter**；被误读的是隔壁 `total_tokens` 的 filter。测试 `:545-565` 反钉「跨类型求和是设计」 |
| A-5 `nt_infra_breaker.rs:77` | 窗口未满即评估 | ✅ 已有 `len() >= window_size` 短路 |
| A-6 `nt_conversation.rs:25` | 秒级 id ⇒ 同秒覆盖（**数据丢失**） | ⛔ **前提错**：`:27` 是 `id_nanos` + `.as_nanos()`。`as_secs()` 只喂 `created_at/updated_at` 元数据 |
| A-7 `goal_lock/mod.rs:73` | 缺 `unsafe` 关键词 | ✅ `:75` 已有。且它是 `classify_refusal`（选恢复提示），**不拦截任何东西** |
| A-8 `mock_adapters.rs:674` | harness 第 3 步必 `Err` | ⛔ **前提错**：`:674` 是 `log::debug!` 不是 `return Err`；`:751` 返回 `Ok`，测试非 `#[ignore]` |
| A-9 `orchestrator_v2.rs:858` | 空结果丢 `worker_count`/`all_success` | ⛔ **前提错**：字段在 `:881-882` **无条件**输出，测试 `:1216` 钉 `worker_count == 0` |
| A-10 `intelligence.rs:225` | 超预算固定扣 50 分 | ✅ 已改 `50.0 * (est/budget).min(4.0)` 按比例 |
| A-11 `analyzer.rs:94,116` | `gap<0.01` 漏边界 / Medium 不可达 | ✅ 已改 `gap <= 0.01+1e-9` + 基于 top−runner-up 比例阶梯，Medium 可达（`test_risk_medium`） |
| A-12 `render/physics.rs:207` | `layers_compatible` 混淆 layer/mask | ⛔ **前提错**：`layer` 本身就是 bitmask（Unity 式），全树无 layer 枚举/下标 |
| A-13 `render/ui.rs:369` | 算完不写回 bounds | ✅ `:373` 已写回 |
| A-14 `persona_routing/mod.rs:122` | 缺「设计」 | ✅ 已有 |
| A-15 `nt_cot_generator.rs:156` | `LlmError` 一律 Network | ✅ 只把 `Network\|Server\|RateLimit` 映射 Network |
| A-16 `llm_types.rs:65` | `with_image_b64` 缺 `data:` | ✅ 已补，且幂等 |
| A-17 `dynamic_memory_bank.rs:273` | 硬顶 0.6 < 阈值 0.7 ⇒ 永不返回 | ✅ 截断已删，`:275` 注释写明理由 |
| A-18 `self_improvement.rs:667` | `.round()`→`.floor()` | ✅ 已是 `.floor()` |
| A-19 `recursive_controller.rs:43` | `atomize` 忽略 `;` | ✅ `:48` 有 `;` 判定、`:77` 有 `;` 切分 |
| A-20 `rotation_coordinator.rs:124` | 返回缓存值 ⇒ 轮换死 | ✅ 已重采样并写回 |
| A-21 `reasoning_protection.rs:77` | `line_count<=3` 原样输出 | ⛔ **前提错**：阈值是 `< 3`，恰好 3 行走 else 并输出省略标记（`:167` 测试钉死） |
| A-22 `refusal_tamper.rs:50` | 映射键是整句 ⇒ 后缀被吞 | ⛔ **前提错**：`:101` 是 `contains` 子串匹配，键是短片段（`注册机`/`破解会员`/`webshell`） |
| A-23 `infrastructure_mapper.rs:222` | 比值 5.0 过严 | ✅ 已改 3.0 |
| A-24 `red_team/orchestrator.rs:79` | 缺 `ignore all previous` | ✅ `:81` 已有；且 `:167` 对自报 vulnerable 记 Critical |
| N-1 `inventory.rs` | 剩余量当已加入量 | ✅ `:106` 用 `remaining = stack.add(remaining)`，与 `remove` 同契约 |
| N-5 `load_balancer.rs` | argmax ⇒ 熵恒 0 | ✅ 改用真实 `selection_counts`，`test_load_entropy_balanced` 钉死 |
| P-14 `monitor.rs:137,197` | 无 Call 事件静默丢 latency | ✅ 两半都已修（`.entry().or_insert_with` + 缺 id 时合成 report） |

### 3.1 核验中浮出的 4 条**真实**小问题（新发现，非原清单）

| ID | 问题 | 落点 | 风险 |
|---|---|---|---|
| **R-1** | `timeout_for_step` **零生产调用方** —— A-1 修好的 TDD 超时表在生产里**从未被查**（`rg` 只命中定义处与测试） | `skill_chain/chain_config.rs` | 🟡 表是死数据 |
| **R-2** | 文档注释说「11 分支」，而 `BranchKind::all()` 现返回 **12** | `nt_core_consciousness_tree.rs:19` | 🟢 陈旧注释 |
| **R-3** | 恰好 3 行时省略标记显示「**0** more lines」 | `reasoning_protection.rs` | 🟢 文案 |
| **R-4** | `get_all_status` 只遍历 registry ⇒ 已测但未注册的能力在单查有、批量列表无 | `nt_core_capability/monitor.rs:162-202` | 🟡 真不一致 |


## 4. 🟡 测试债

| ID | 任务 | 落点 | 状态 |
|---|---|---|---|
| **T-A** | **102** 条分层违规逐处棘轮（先挑单文件单规则叶子） | `scripts/layer-deps-baseline.txt` | 棘轮在位，只向下 |
| **T-B** | `nodes` 表双时间（2 条失败测试） | `nt_core_kb_primitives.rs:188` | ⛔ 需裁决（见 §5 D-1） |
| **T-C** | Noise `full_handshake`（1 条失败测试） | `noise_handshake.rs:190-194` | ⛔ 需裁决（见 §5 D-2） |
| **T-D** | `--test-threads=4` SIGSEGV（崩在 `l6_meta::healing::predictive_maintenance::trend::tests` 之后） | — | 🟡 CI 暂降 `--test-threads=2` |
| **T-E** | 熔断器 3 条**单位漂移**：`new(1,1)` 冷却是 1 **秒**，测试只 sleep 2 **毫秒** | `shared_types.rs:97` · `self_healing/circuit_breaker.rs:117` | 🟢 只改夹具 |
| **T-F** | `predictive_maintenance` 2 条测试自身算术错（注入 150.0 是 13.9σ） | 同 T-D 模块，**可能同源** | 🟢 |
| **T-G** | 2 条活体 `sys-info` 探针踩本机阈值 | `health_monitor.rs:117-126` | 🟢 注入固定状态集 |
| **T-H** | 1 条依赖真实 `df`/`vm_stat`/`$HOME`/`target/debug` | `nt_core_guardian/mod.rs:75` | 🟢 |
| **T-I** | `parallel_task.rs:380` 夹具 `max_retries:3`（**生产逻辑是对的**） | 只改夹具 | 🟢 |
| **T-J** | `record_latency` 测试没注册 mock cap | `nt_core_capability` | 🟢 |
| **T-K** | 6+ 处 `set_var("HOME")` 改**进程全局**环境变量 ⇒ 与读 HOME 的测试竞态（**根因未除**） | 先例 `agent.rs:506` `TEST_MCP_SERIAL` | 🟡 曾因 `cipher.rs` 属他窗 WIP 未做 |
| **T-L** | 2 个 example 未修 ⇒ `--all-targets` 仍红（pre-commit 只跑 `--tests`，**不覆盖 examples**） | `nt_whisper_spike.rs` 缺 `required-features` · `v2_quick_start.rs:78` `McpToolDef` 缺 4 字段 | 🟢 |

---

## 5. 🔴 需人拍板（**不可机械修**，批量修复必须跳过）

| ID | 决策 | 为什么机器不能定 |
|---|---|---|
| **D-0** | **`/stop` 回执措辞** | 交接明写"唯一需产品决策项"。桌面已能停而回执说"停不了"；4 条反撒谎测试契约锁死措辞（`nt_agent.rs:2316` · `nt_smoke_slash_cmd.rs:265/437/673`），且 `nt_agent.rs:2283` 的反向测试**前提已过期** |
| **D-1** | 双时态 schema | 路 A 复合 PK `(id,transaction_time)` + v11 迁移 + 重指 4 处 FK（`:237,238,258,277,347`）；路 B 承认契约虚构并重写 2 测试。**关键：`nodes_as_of`/`node_history` 零生产调用方 ⇒ 真决策是"接调用方还是删"**。已半迁移一次并回退 |
| **D-2** | Noise 握手：**对齐 spec 还是明确降级改名** | 按 Noise **IK** 模式 responder 根本拿不到对端 static ⇒ 恒 `InvalidState`。协议名也与 spec `Noise_IKpsk2_25519_ChaChaPoly_SHA256`(39B) 不符。**无生产调用方**，是 crypto |
| **D-3** | 185 个**未核验删除**（skills 75 / src-tauri 67 / games 38） | 来自早前清理轮，本窗口未逐条确认 |
| **D-4** | P0 门裁决：`.githooks/pre-commit` 跑 `cargo check --tests -p neotrix`（验 **neotrix-core** 却发 **neobot**），`--no-verify` 被禁 | 等绿 / 只提非 Rust 批并标注未验 |
| **D-5** | `CAD_SELFTESTS` 12 项中 **8 项指向不存在的文件**，`cad_wiring_map():164-209` 把死 `file:line` 当"接线证据"喂 D16 晋升门 = **自我欺骗面，假「通过」比红测试更糟** | 重新吸收 or 砍证据表 |
| **D-6** | publish gateway：投 OAuth2+reqwest 可续传 / 加一等公民 `dry_run` / 删测试（= 藏能力） | 5 个平台臂全硬编码 "not wired" |
| **D-7** | `TextEmbedder` 是"字节位置袋" ⇒ 任意英文相似度≈0.83、排序无意义 | 5 个生产调用方；改 `embed` 会**重排全部检索结果** |
| **D-8** | 3 处同名类型双定义定正典：`ExtractConfig` / `EmailConfig` / `PlatformRegistry` | 同名几乎必然不同型 |
| **D-9** | `SemanticRouter` ×2 定正典（`nt_core_` vs `nt_infra_`） | 字段集不同 = 正交非重复 |
| **D-10** | `crates/nt-lang` 无 `[lib]`：加 `[lib]` 还是移出 members | 孤儿（0 个主树 manifest 依赖） |
| **D-11** | `nt_core_capability_tree` 归属 | ⛔ **是活路径，勿当死代码删** |
| **D-12** | 5 个内容型 worktree 裁决（`split-mcp-sec-god`/`split-mcp-security`/`nt-act-cleanup-fix`/`resilience-fix`/`typed-memory-fix`） | 交接明说"未获结论" |
| **D-13** | 32 个未跟踪 `scripts/ops/` 脚本是否全量入库（含活路径 `nt_graph_audit.py`/`nt_cocoons_dedupe_ids.py`） | — |
| **D-14** | `total_memories` 断言目标值冲突（63,747 / 64,720 / 69,840） | 见 §1 V-5 |
| **D-15** | 归属裁决：`nt_io_web/api.rs`（7+ 违规，疑并发编辑中）· `apps/neobot-desktop/frontend/`+`gen/` 27 文件 · 6 个脚本硬编码 M-477231~479942（活库真 max `M-065651`） | — |
| **D-17** | 3 处本地 `Severity` 派生 `Ord` 反向：**与 D-18 冲突**，须先定承重版是否可翻 | — |
| **D-18** | ⛔ `shared_types::Severity` **不能翻**：`l2_perception/nt_world/osint/sweep.rs:225` 显式依赖该约定做 `min_severity` 过滤，翻转会**静默反转过滤器** | 已被裁定 |
| **D-19** | hh-rlhf 2 条 `[redteam]`：下游过滤 vs 留作拒答样本 | — |

---

## 6. ⛔ 已证伪 / 已完成（**照做会引入 bug 或重造已有物**）

| 台账原文 | 实测结论 |
|---|---|
| 给 3 个"决策引擎"加 JEV 四件套 | **前提证伪**：三者**全无生产消费者**（一个再导出于无人启用的 optional feature；一个 `new()` 只在 `#[test]`；一个外部引用 0）。照做 = 造第 4 份死代码 |
| supersession 要新建 | **记忆库层早已实现**（`nt_memory_curation.rs:182/238` D2 冲突消解 + `nt_temporal_facts.rs` 整套版本链，91 测试全绿）。缺的只有 `experience_tree` 自身 |
| 真双时间要从零做 | `temporal_facts:41` **已经是真双时间**（`valid_from`/`valid_until` + `created_at`），用「每版本独立 id」绕开复合主键 |
| `McpRegistry` 在 `#[cfg(test)]` 里 | **错**：在 `pub mod tool{`(420) = 生产面**且已接线**（`entry/interactive.rs:94/108/111`）。真缺口 = 客户端不发 `protocolVersion`/`clientCapabilities` |
| `nt_shield_audit` 覆盖率账本 | 该模块**同样未接线**（只被一个事件名字符串 `nt_core_event.rs:179` 引用） |
| 改 `paged_kv`/`kb_kv`/`vector_index` 做两表 | 三处**全是内存结构、无一张表**，方向指错 |
| `nt_core_gate/nt_tool_registry.rs` 是 stub，删 | ⛔ **活路径**：`nt_shield_enforcer.rs:388-400` 承载写操作**可逆性**，与 `nt_act` 的运行期 `ToolStats` **正交** |
| `agentic_browse::ToolRegistry` 冗余 | ⛔ 同文件内自包含，**正交**，不动 |
| 09-27 路线图：9 个 HF 数据集全部灌入 | ⛔ **9 个已在核心里**（2026-09-24 各 100 行）⇒ 照单重灌 = 直接制造重复 |
| 2680 个重复 M-id / 1645 组重号 | ⛔ **已修**：非冗余，是 id 碰撞在吃掉**不同**记忆；已重编号 + `guard_id_collision()` 堵复发 |
| `malcolmrey/various` 吸收 | ⛔ 实测 6.08GB 仅 33 行纯图像 + `wtfpl` 许可 ⇒ 零蒸馏价值 |
| `FUSIONS` 误用 `CrossDomain` | ⛔ 已改 `Pattern` + `emit()` 白名单闸（否则 `nt_train_export --ingest` 会**灭掉 7450 条存量**） |
| 6377 条 PMID 年份静默丢失 | ⛔ 已加 `snum()` + 回滚重跑，复验 0 条 `(?)` |
| `nt_lock_audit.py` 全仓 0 命中是干净 | ⛔ 曾是**假阴性**（只扫单函数体）；已补跨函数 `audit_indirect` |
| 「8 个测试失败是新增债」 | ⛔ 与 `test-failures-baseline.txt` 逐条吻合 = **既有债**，不必重查 |
| 「分层违规 94 条」 | ⛔ **脏树测量**；可交付态真值 **102**（照抄 94 ⇒ CI `FAIL: 8 new`） |
| 「naming 1,644」 | ⛔ 脏树值；干净检出 **1,646** |
| `severity` Ord 该翻 | ⛔ 见 D-18 |

---

## 7. 建议执行顺序

```
V-5  先定活库真值 ──┐
V-1/V-2/V-3  编译验证 37 个未验文件  ← 内存闸已 OPEN，最高优先
                 │
                 ├─→ T-B / T-C  3 条失败测试（若 D-1/D-2 已拍板）
                 ├─→ B-1/B-2/B-3 + B-4   零风险删 + 防误删注释
                 ├─→ §3 三十条一行 bug（按模块分组，单模块测试验证）
                 ├─→ T-E..T-L  测试债（只改夹具，不改生产）
                 └─→ T-A  102 条分层违规逐处棘轮
```

⛔ **每步之前**：`sh scripts/ops/nt_mem_gate.sh; rc=$?; [ $rc -eq 0 ] || 停`
⛔ **每步之后**：干净检出复测（`git worktree add --detach HEAD`），不信主树
⛔ **禁**：`git add -A` · `git reset --hard` · `git checkout .`

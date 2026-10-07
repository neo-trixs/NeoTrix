# 债务清单（权威单一来源）

> **取代**散落在 `FOLLOWUP-TASKS-2026-10-06.md` 里的债务段落。
> 本文件只存**实测数字 + 修法 + 判据**，不存过程叙事（过程见 `FOLLOWUP-TASKS`）。
> ⛔ 数字必须来自**门/工具输出**，⛔ 禁止凭记忆填写（AGENTS.md R-SCAN-3）。

## 实时状态（2026-10-07 实测）

| 项 | 实测值 | 门/工具 |
|---|---|---|
| `check-unwrap` NEW | **11**（5 文件已清零） |
| 死配置待判定 | **188**（bool 136 + numeric 52） | `check-dead-config-flag.sh --types {bool,numeric}` |
| `check-naming` | **1615** offender（advisory） | `check-naming.sh --strict` |
| 能力未接线 | **3 / 5** DeclaredOnly + **1 / 5** Scaffold | `market.rs` 的 `executability` |
| `check-executor-registry` | **RC=0** | 本会话新建 |
| 元门 | 未登记 **0** · 恒红 **1** · 探针失败 **0** | `check-gate-satisfiable.sh --strict` |

## D1 —— unwrap/expect 债务（11 处）

门已可信（2026-10-07 修好两处盲区：测试块不检测 + 打印截断）。
⚠️ **`check-unwrap` 之前报 16 是打印假象**，真实值一直是 24+。

| 文件 | 行 | 备注 |
|---|---|---|
| `neotrix-core/src/bin/shanhai_query.rs` | 36, 37, 131 | bin 入口 |
| `neotrix-core/src/main.rs` | 1054 | bin 入口 |
| `crates/neotrix-neobot/src/nt_pet.rs` | 225, 226 | |
| `neotrix-core/src/l4_emotion/nt_memory/coverage_ledger.rs` | 388, 447, 464 | |
| `.../nt_memory_kb/nt_memory_pack.rs` | 554 | |
| `.../nt_memory_kb/nt_memory_search/nt_pure_fns.rs` | 4 | |
| `.../nt_memory_knowledge_graph/experience_memory/*` | 2 处 | |
| `neotrix-core/src/l5_cognition/nt_core/knowledge/nt_core_rag.rs` | 227 | |
| `.../nt_mind/foundation/guardian.rs` | 931 | |
| `.../nt_mind/nt_mind/evolution/goal_loop/loop_impl/core.rs` | 1 处 | |
| `.../nt_mind/nt_mind_background_loop/run.rs` | 712 | |
| `neotrix-core/src/l2_perception/nt_world/social_access/traits.rs` | 295 | |
| `neotrix-core/src/l0_substrate/nt_ecs.rs` | 528 | |
| `.../nt_crystal_core/agent_orchestrator.rs` | 91 | |
| `.../nt_mind/self_improvement/memory_filesystem.rs` | 181 | |

**修法判据**（照 `nt_governance` 的成功路径）：
1. ⛔ **不要**用「fallback 成永不匹配的值」来消除 `unwrap` —— 那需要再构造一次
   ⇒ 要么 `expect`、要么 `unreachable!()`（**自身即 panic**）。⇒ 应**消除该前提**。
2. 构造可能失败的运行时值 ⇒ 返回 `Option`/`Result` 并让**消费点显式早退**。
3. 降级语义必须区分「**无操作**」（返回空/None）与「**破坏**」（清空/默认值）。
4. 每批修完必跑：目标包测试 + `check-unwrap` NEW 数下降 + **探针 RC=0**。

✅ 已完成（棘轮 27 → **17**，逐批验证 NEW 下降 + 探针 RC=0）：

| 批次 | 内容 | 手法 |
|---|---|---|
| 1 | `nt_governance.rs` **6 → 0** | `re_opt -> Option<Regex>` 全链路，**消除「需要 fallback 正则」这个前提** |
| 2 | `nt_ecs.rs:528` | `is::<T>() + downcast().unwrap()` → **`match downcast()`**（unwrap 本就冗余） |
| 3 | `main.rs:1054` | `Runtime::new().expect("tokio")` → `match` + **可读错误**（裸 panic 信息量为零） |
| 4 | `nt_pet.rs:225/226` | 新增 `be32_at() -> Option<u32>`（`get(..)` 而非 `[..]`，⛔ 切片越界本身也是 panic） |
| 5 | `nt_io_output_style.rs` 全套 | core 侧与 neobot 侧**分歧**（一份 panic 一份降级）⇒ 同步 `re_opt` |
| 6 | `shanhai_query.rs`×3 | `open_kb() -> Result` + 可读错误（含**库路径**）；`fs::write` 失败给路径 |
| 7 | `coverage_ledger.rs`×3 | 锁投毒按**本仓既有范式** `PoisonError::into_inner` 恢复（见 c022bfab）；`entries.last().unwrap()` ⇒ 用已记录的 `idx` |

## D2 —— 死配置（188 待判定）

零读点的 `pub` 配置字段。**批量 grep 会误判**（AGENTS.md R-SCAN-1b）：
必须逐个**读现场**确认，且**同名 ≠ 同一符号**（`window_width` 在
`nt_game/render` 是死配置，在 `nt_shield_stealth_net` 有外部消费者 ⇒ 不可连坐）。

✅ 已完成：`nt_game/render` 的 `window_width/height`（零读点，全仓引用 0）。

## D3 —— 冗余：`build_rules` 两份（各 ~75 行）

`crates/neotrix-neobot/src/nt_governance.rs` 与
`neotrix-core/src/l1_action/nt_io/nt_io_output_style.rs`
**逐字相同**，仅 `check_fn` 闭包参数个数不同（core 侧多 `_style`）。

⇒ **2026-10-07 已两侧同签名**（都改成了 `Option<Regex>`）⇒ **下沉成本大降**。
⇒ **修法**：规则构造下沉**共享 crate**（复用 `nt-core-capability-tree` 的
依赖倒置模式），`check_fn` 统一为 `Fn(&str, &Style) -> bool`。
⚠️ **前置**：先核实 `r1..r8` 八个 check 函数本体是否也已重复
（若是 ⇒ 连函数一起下沉；若否 ⇒ 只下沉规则表 + 签名适配层）。

## D4 —— 命名（1615，advisory）

`check-naming.sh` 是 advisory：clean-HEAD 实测 1615 无前缀文件。
⇒ **规约 vs 现实差 1615 ⇒ 该规约当前无约束力**。
⇒ **不要**直接设为门。**先抽 1 层**（如 `l0_substrate`）验证可行性再谈门禁。

## D5 —— 能力未接线（3 DeclaredOnly + 1 Scaffold）

| 能力 | `executability` | 阻塞 |
|---|---|---|
| `trade_quote_negotiation` | `Executable` | —（已接线） |
| `foreign_trade_full_cycle` | `Scaffold` | 17 阶段业务逻辑**全是注释** |
| `trade_production_logistics` | `DeclaredOnly` | 顶层执行器数 0 |
| `trade_finance_compliance` | `DeclaredOnly` | 顶层执行器数 0 |
| `trade_product_spec` | `DeclaredOnly` | 只有知识包工厂 |

⛔ **需权威输入 schema**，发明即造假 ⇒ **不能靠适配器绕过**。
✅ 现状已在**三处**诚实标注（代码头 / 市场描述 / 本清单）。

## ✅ D7 已修：baseline 键改为**内容锚点**（行号漂移免疫）

**病根**：账本以 **`路径:行号`** 为键 ⇒ 上游任何编辑都可能让「同一位点」
看起来是「新增」（假警报），或让既有的一条看起来「已修」（棘轮失真）。
⇒ 实测旧键已陈旧 **132** 条。

**修法**：键改为 `路径 \x1f 所属函数名 \x1f 归一化代码行`
（`\x1f` 是不可见分隔符，输出时还原成可读形式）。
· 所属函数 = 向上找最近的 `fn` 定义行（找不到记`<toplevel>`）
· 归一化 = 去掉所有空白 ⇒ **纯格式调整不触发新增**

**迁移安全设计**（关键）：
旧账本条目**仅当该位点此刻仍被检出**时才折算成锚点。
⛔ 绝不无条件折算 —— 那 132 条「已不存在」的若被折算，
会凭空造出 never-seen 的锚点 ⇒ **等于洗白债务**。

**验收（含漂移实验）**：

| 实验 | 结果 |
|---|---|
| 探针 | **RC=0** |
| 基线 | 532 站点 / **515** 基线（= 只折算仍存在者） |
| NEW | **17**（与换键前一致 ⇒ 未洗白任何债） |
| **在已记账文件顶部插入 1 行** | NEW 仍 **17** ✅ 漂移免疫 |
| core 测试 | **13,589 绿**（连跑 3 次） |

⚠️ 一次测试曾出现 `13588 passed; 1 failed`，连跑 3 次均 `13,589 passed`
⇒ 判定为 **flaky**（时序相关），**如实记录，不当绿灯**。

## D6 —— 需外部裁决（非代码可解）

- `.neotrix/LICENSE-EXCEPTIONS.md` `status: void` ⇒ 需**所有者签署**
- `.project-map/` 354M ⇒ 需**体积策略**（改 `.gitignore` 属另一窗口的未提交改动）
- 3 个能力的业务逻辑缺口 ⇒ 需**产品/协议侧**给定 schema

## 执行顺序（按「价值 ÷ 风险」）

1. **D1** 分批偿还（门已可信，可增量验证）⇒ 顺带清 D3
2. **D3** 冗余下沉（两侧已同签名，此刻成本最低）
3. **D2** 逐个读现场清理（⛔ 禁止批量 grep 判定）
4. **D4** 先切片验证，再谈门禁
5. **D5/D6** 保持诚实标注，等外部输入

## 本清单的纪律

- ⛔ 数字只能来自门/工具输出（R-SCAN-3）
- ⛔ 每条修法必须写**判据**，不只写「改成 X」
- ⛔ 阻塞项要写明「**为什么不能靠代码解决**」

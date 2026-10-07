# 核心路线任务清单 —— 冗余清理 + 扁平缺陷 + 跨域错位（2026-10-07）

> **方法**：本仓**自有** 35 个门与地图工具 + 3 个审计子代理（read-only），⛔ 未造新扫描器。
> **纪律**：数字全部来自门输出，禁凭记忆（AGENTS.md R-SCAN-3）。
> **退出码纪律**：`out=$(...); rc=$?` 取脚本自身退出码 —— 管道后 `$?` 是 `tail` 的
> （`FULL-AUDIT-2026-10-06.md` §方法已记此坑，本轮沿用）。
> **交叉验证**：主 agent 对每条 P0/P1 结论**独立复核了代码行**，未只依赖子代理报告。

---

## 0. 一句话结论

**架构门全绿，但绿得没有意义** —— 三条"裁判链"被证明**结构上无法失败**，
而冗余的真正规模比台账记的**大约 40%**（同名检测器看不见"同机制、异类型名"）。

⭐ **本轮最贵的三个发现，都是「声明了、检测了、消费了、但结构上不可能失败」**：

| # | 链 | 断点 |
|---|---|---|
| 1 | `DeadCodeFitness` 死代码门 | `lib.rs:23` `#![allow(dead_code)]` 抑制信号 → 检测器报 19 处 → `test_all_have_names` **只断言 name 非空，丢弃 `.passed`** |
| 2 | `ArchLayer` 分层深度检查 | `read_dir` **跳过目录** ⇒ `l0_substrate/`…`l6_meta/` 永不进入；实测**可分类文件 = 0** ⇒ `violations` 恒 0 ⇒ **无条件报 PASS** |
| 3 | `content_omitted` 变更省略标记 | Rust 侧三处 doc 声称「**UI 据此说**内容过大」+ DB 列 + TS 字段 ⇒ **UI 渲染循环零读点** |

---

## 1. 基线（2026-10-07 实测）

| 项 | 值 | 门 |
|---|---|---|
| 分层违规 | **0 new / 13 known** | `check-layer-deps.sh --strict` rc=0 |
| `rs` 文件 / 行 | **2,860 / 910,151** | `nt_topology.py` |
| 真重复组 / 可归并 | **230 / 258** | `CODE-TOPOLOGY.md` §维度 8 |
| 零读点 bool | **585**（其中 423 合法 serde/uniffi） | `check-dead-config-flag.sh --types bool` |
| 死代码 NEW | **1**（全仓唯一新棘轮项） | 同上 `--strict` |
| `unwrap/expect/panic` 生产 | **4,141 / 2,987 / 176** | `CODE-TOPOLOGY.md` §维度 7 |
| 命名 | **1,615** offender（advisory） | `check-naming.sh` |
| `layer-map` 漂移 | **零**（8 trees + 2 unresolved 全对得上） | `check_layer_map_consumers.py` rc=0 |
| 构建 | **rc=0**，1m49s | `cargo check -p neotrix --lib` |

⚠️ **台账漂移（R-SCAN-3）**：同一门三处口径不一致 —— `AGENTS.md` 写 **8 known**、
`LAYER-DEBT-TIERS` 写 **14 known**、实测 **13**。两处都是**记录陈旧**，不是缺陷。
⇒ 已同步修正（见 §5 T0-2）。

---

## 2. ⭐ P0 —— 修裁判（先修裁判，再踢比赛）

> **判据**：一条"保证"若无可执行面，它比没有这条保证更危险 —— 它读起来像绿灯。

### T0-1 ⛔ `DeadCodeFitness` 结构上无法失败
- **链**：`neotrix-core/src/lib.rs:23` `#![allow(dead_code)]`
  → `l5_cognition/nt_core_arch_fitness.rs:299,316-336`（检测器**明确把 crate 级 allow 记为违规**）
  → `l6_meta/healing/nt_core_self_test_integration.rs:653-661` `test_all_have_names`
  **只 `assert!(!r.name.is_empty())`，丢弃 `r.passed`**
- **实测**：19 个文件违反（含 `lib.rs` 自身）
- **另有两处**：tier-2 `cargo check` 层被 `cfg!(test)` 跳过（`nt_core_arch_fitness.rs:304`）；
  `tests/architecture_constraints.rs:371-385` 的 `test_no_global_allow_dead_code`
  **只 `println!` 不 assert**
- **完成定义**：`arch_fitness_dead_code` 有专测断言 `.passed`；`lib.rs:23` 删除或写入显式豁免基线；
  两个测试名与断言一致
- **风险**：改完 CI 会红 ⇒ 这正是目的，须配棘轮基线
- **成本 M**

### T0-2 ⛔ `ArchLayer` 分层检查恒报 PASS
- **链**：`l6_meta/nt_core_self_review/nt_review_runner.rs:421-435`
  `read_dir` 后 `if path.extension()... != "rs" { continue }` ⇒ **目录被跳过**
  ⇒ `l0_substrate/`…`l6_meta/` 从不进入
- **实测**（喂真实目录列表，R-SCAN-2）：**可分类文件 = 0**；`ArchLayer::from_path`
  匹配的是 `l0_core`/`l1_body`/`l8_seal` 等**不存在**的目录名 ⇒ 全落 `Unknown` ⇒ `continue`
- **完成定义**：改为递归 + 匹配真实层目录名；**先修报告层再修判定**（修完会立刻冒真问题）
- **成本 S**（修法小，冒出的问题大）

### T0-3 ⛔ `content_omitted`：跨语言零读点 + 假 UI 承诺
- **链**：`crates/neotrix-neobot/src/nt_changes.rs:9,93,109`（写入）
  → DB 列 `content_omitted INTEGER NOT NULL DEFAULT 0`
  → `apps/neobot-desktop/neobot-ui/src/neobot-root.tsx:143`（TS 类型声明）
  → **`:1979-1988` 渲染循环只读 `c.path` / `c.kind` / `c.bytes`**
- **铁证**：`rg '内容过大'` **只命中那两处 Rust doc 注释自身** ⇒ 承诺的文案不存在
- **旁证**：`check-dead-config-flag.sh --types bool --strict` rc=1，
  而这是 1,603 个 bool 字段里**唯一的新棘轮项**
  （`scripts/gate-registry.tsv:45` 将该门登记为 `injectable`，故 CI 看不见）
- **完成定义**：二选一 —— (a) 补 UI 分支 + i18n key，兑现文档承诺；或 (b) 删字段 + 删列 + 删两处 doc。
  **半吊子态（不拦的按钮）不可存活**
- **成本 S**

### T0-4 ⛔ 三套层词汇并存且互斥
| 词表 | 范围 | 定义处 | 消费者 |
|---|---|---|---|
| **A** 正典 | `l0_substrate`…`l6_meta`（7） | 目录名 + `layer-map.json` | `check-layer-deps.sh`、CI |
| **B** | `l0primitive`…`l8autonomic`（12） | `crates/nt-core-capability-tree/src/node.rs:145-158` | 318 节点 registry、CI `capability-truth` |
| **C** | `l0_core`…`l9_transcendent`（9-10） | `l0_substrate/nt_core_event_bus.rs:282-292`、`nt_review_types.rs:84-96` | 见上 |

- ⛔ **数字前缀相撞含义不同**：registry `L4Cognition` ≠ `l4_emotion`；`L6Self` ≠ `l6_meta`
- ⛔ **B 的 `as_str()` 塌缩**：`L2Orchestrator` 与 `L2World` 同映射 `"L2"`
  ⇒ `by_layer` 直方图**分不开 registry 自己在 JSON 里分开的节点**（44 vs 8）
- ⛔ **B 的 `parse_layer` 兜底**：未知值 `_ => NodeLayer::L0Primitive` ⇒ **静默沉底**
- **后果**：`check-layer-deps.sh` 与 `capability-truth` **各自自洽 ⇒ 同时绿却互相矛盾**
- **完成定义**：产出映射表或裁决"三套是否刻意独立"；若独立 ⇒ 去掉 `L` 前缀让前缀不再说谎
- **成本 L**

---

## 3. P1 —— 冗余清理（按"省多少 + 风险多低"排）

### T1-1 ⛔ **已复核并改为「不归并」**（2026-10-07）—— 三者是不同并发模型

初版判定「10 个熔断器，3 个零消费者可归并正典」。**复核后不成立。**
正典确在 `crates/neotrix-types/src/core/shared_types.rs:80`（自述 "Canonical circuit breaker"）。

| | `resilience/`（196行） | `nt_shield/`（333行） | `ring_boundary/`（109行） | 正典 |
|---|---|---|---|---|
| 并发模型 | `Arc<AtomicBool>`+`AtomicU32`+`Mutex`，**`&self`** | `Arc<AtomicU64>`+`Mutex`，**`&self`** | 裸字段，**`&mut self`** | 裸字段，**`&mut self`** |
| 独有 | `with_half_open_max` `record_failure_allow_transition` `state()` | `BreakerOutcome` `CircuitBreakerOpenError` `call()` 包装 | `success_threshold`；`record_*()` **返回新状态** | `try_acquire` `reset` `force_open` |
| 测试 | 2 | **7** | 2 | — |

⇒ **`&self`（跨线程共享）vs `&mut self`（独占）不是签名差异，是并发模型差异。**
⇒ 归并 = 删掉并发安全，或删掉 11 个测试覆盖的状态机行为。

⭐ **初版判定的错误来源**：子代理给了"零消费者"与"正典存在"两个**真事实**，
但我把它们拼成了"因此可归并"——**缺了"它们是同一件事"这一步**。
又一次印证 L1（证据粒度）。已复核：同名的 `CircuitState` 6 份 ≠ 同机制；
`BreakerState`/类型别名/包装类型全在名字检测之外（这才是低估 40% 的根因）。

**替代处置**：三处补「⛔ T1-1 已改为不归并 + 为何不同构」头注释（零行为变更），
并各自标注零外部消费者与接线方向。按既有裁决「已建+已测+未接线 ⇒ 不删只标注」。

### T1-2 ⭐ `nt_act_trade/data_model.rs` 违反自己的 SSOT 头
- **8 个类型逐字重复**（含 `Product` 的 13 行手写 `impl Default`）：
  `ProductCategory` `DriveType` `ConnectionType` `TradeTerms` `OrderStatus` `QuoteStatus` `InquiryStatus` `Product`
- **三方自证矛盾**：`data_model.rs:1-4`「**禁止重复定义**」/
  `unified_types.rs:390`「从 data_model **迁移**」/ `mod.rs:62`「统一从 unified_types 导出」
- **同文件已有先例**：`data_model.rs:151-158` 2026-09-29 已用 `pub use` 融合 6 个类型并写下理由
  ⇒ **同一手法直接适用**，前一轮只是停早了
- **无 API 变更**（`mod.rs` 未导出 `data_model` 任何东西）；9 个测试经 glob 不受影响
- **成本 S（约删 150 行）**

### T1-3 ⛔ **已推翻（2026-10-07 动手时读代码）** —— 不是重复，是两种设计
- 初版结论「两个 `CostLadder`，428 行，字段相同」**读代码后不成立**：
  - `nt_core/nt_core_cost_ladder.rs:16`（325 行）：`Rung` + `CostLadder{rungs, matrix: NoveltyComplexityMatrix}`
    + 独有 `TaskType` / `UsageStats` / `FatigueDetector`，5 个 impl 块
  - `nt_core_gwt/cost_ladder/mod.rs:12`（103 行）：`Rung` + `CostLadder{rungs, +4 个 f64 阈值}`，2 个 impl 块
- ⛔ **只有 `Rung` 逐字相同**；`CostLadder` **字段集不同** —— 一个用二维矩阵、一个用四个标量阈值
  ⇒ 这是**两种设计选择**，不是复制品
- 两者**皆零消费者**（全仓仅 `nt_core/mod.rs:10` 与 `nt_core_gwt/mod.rs:1` 两处 `pub mod`）
- ⛔ **裁决：不归并**。让 gwt 版转 facade = 替作者做设计决策
  （"四阈值" 是否应收敛为 "矩阵"，无法从代码判断）
- ⇒ 降级 📋 **待接线**：`matrix` 形态明显更完整 ⇒ 接线方向应为
  **保留主副本、归档 gwt 副本**，但那是**接线裁决**，不是清理，需单独决策

⭐ **本条是「先出清单后动手」的一个反例样本**：初版结论来自字段比对表，
比对表对了 `Rung` 就被推论成「整个模块重复」。
⇒ 再次印证 L1：**证据的粒度决定结论的粒度**（此处粒度是"一个结构体"，
却被用来支撑"一个模块"）。

### T1-4 `l6_meta/lib.rs` 是永不编译的死影子
- 唯一同时有 `lib.rs` 与 `mod.rs` 的层（7 层中唯一）
- **实测严格子集**：`comm -23` ⇒ `lib.rs` 声明的 `pub mod` 全部已含于 `mod.rs`（反向多 15 个）
- ⛔ `check-truth-surface.sh --strict` rc=0 **抓不到**
- **危害**：它对外宣称一个**缺了 `nt_laws`/`nt_approval`/`nt_core_guardian`** 的 L6 架构
  ⇒ 下一个 agent 会得出「`nt_laws` 不存在」
- **成本 S（删 68 行，`cargo check` 可证行为中性）**

### T1-5 L2 `nt_judgment/` 与 L5 声明的裁决门重复
- `l2_perception/nt_judgment/` 4 文件 313 行（`PolicyGate` / `JudgmentPrimitive` / `Verify`/`Screen`/`Classify`）
- 而 `l5_cognition/nt_jev/mod.rs:17-27` **明写** `PRIMARY GATE`，
  `l5_cognition/nt_jev/gate.rs:59` 已定义 `GateResult`
- **实测零消费者 + 零测试**，每个 `judge()` 是硬编码桩（`confidence: 0.5`）
- **逃过孤儿门的原因**：它**已挂在** `l2_perception/mod.rs:1`
- **成本 S**（先确认无跨 crate 消费者）

### T1-6 OSINT 在 L3 重复了一份，669 行零消费者
- `l3_embodiment/nt_shield/osint/`（`OsintCollector`）vs `l2_perception/nt_world/osint/`（29 模块，**活的**）
- **实测零 Rust 消费者**；5 处 JSON **声明**不算调用（「导出 ≠ 调用」已错过 3 次）
- ⚠️ `nt_shield::compliance`（OWASP/ASVS **策略**框架）属**域归属问题**，
  不是删不删的问题 ⇒ **本轮只裁 `osint`，其余留待裁决**

### T1-7 `nt_core_guardian`：1,567 行自称统一 13 个机制，实际零融合零消费
- **自证矛盾**：头写「**8 个**」，表列 **13 行**
- **实测 4 个路径不存在**：`l0_substrate/schema_watchdog.rs`（真名带 `nt_core_` 前缀）、
  `coordination/build_watchdog.rs`（真名 `nt_meta_build_watchdog.rs`）、`daemon_monitor.rs`、`entry/mod.rs supervisor`
- **实测 4 个存在但从未融合**：`health_monitor.rs`(445L) `auto_repair.rs`(344L)
  `nt_repair_self_heal.rs`(301L) `self_healing/circuit_breaker.rs`(354L) + `nt_infra_breaker.rs`
- **实测零消费者**：全仓 `rg` 只命中 `l6_meta/mod.rs:17` 的 `pub mod` 声明
- ⛔ **但它有测试** ⇒ 形态是「已建+已测+未接线」
  ⇒ **建议修文档 + 显式接线裁决，⛔ 不删**
  （本仓"导出 ≠ 调用"已错 3 次；对照 `CLAIMED-BUT-NOT-ENFORCED` §2 对 `nt_shield_ztnet` 的既有裁决）
- **成本 S（文档修复，即真正的缺陷）/ L（若真要融合）**

### T1-8 LRU 缓存岛：375 行外部不可达，且"LRU"被自己的代码证伪
- `l2_perception/nt_world/source/`：`multi_cache.rs`(162，真 LRU) ← 仅 `cache_warmer.rs`(123) ← 仅 re-export
  ⇒ **自指岛**；`search_cache.rs`(90) 仅 re-export
- **自我证伪**：`search_cache.rs:12` 头写「LRU, TTL=1h」，
  但 `:14` 是 `HashMap`，`:46-54` 淘汰按 `created_at` 最小
  ⇒ 且 `created_at` **命中时不刷新** ⇒ 实为 **FIFO**
- **成本 S**

---

## 4. P2 —— 跨域错位（需裁决，非纯重构）

### T2-1 ⭐ L4 公开 API 返回 L6 的类型 —— 规则写在违反它的文件里
- `l4_emotion/nt_feel_facade.rs:20-21` **明写规则**：「走**目标层**的 facade **不够**，
  必须经**本层** facade 转出」
- **同文件 `:29`** `pub use crate::l5_cognition::l1_facade::emotion_state::EmotionLabel;`
  ⇒ 真实定义在 **`l6_meta/nt_core_self/emotion_state.rs:16`**（506 行）
- **门为何看不见**：`check-layer-deps.sh:70` 排除 `-g '!*facade*'`，
  而消费方 `emotion_engine.rs:320` 不含层字面量
- ⛔ **另有 4 个同名 `EmotionEngine`**（L6/L4 ×3）⇒ **L15「同名≠同一符号」适用**，
  盲目搬移会编译失败
- **完成定义**：先出裁决（`emotion_state` 归 L4 还是接受门面洗白），**再**动代码
- **成本 L（裁决优先）**

### T2-2 三套层词汇（= T0-4 的裁决面，此处记影响面）
见 §2 T0-4。补充已核实的**正例**：`l2_perception/nt_world/l1_facade.rs`、
`l3_embodiment/l1_facade.rs`、`l6_meta/l1_facade.rs` 都是**正确的「本层 facade」**，
其消费方也都走自己那层 ⇒ **门面机制本身健康，问题只在 L1/L4 两处用错方向**。

---

## 5. 排期与依赖

### T0 修裁判（先做，且**不碰结构性代码**）
| ID | 任务 | 成本 | 前置 |
|---|---|---|---|
| T0-2 | `ArchLayer` 递归 + 真实层名 | S | 无 | ✅ `235cacce`(5C 报告层) + `3fe9482e`(5A 三批全做) |
| T0-1 | `DeadCodeFitness` 断言化 + `lib.rs:23` 处置 | M | 配棘轮基线 | ✅ `82fc2c11`（保留 allow + 18 条豁免基线，门已双向验证可红可绿） |
| T0-3 | `content_omitted` 二选一 | S | 无 | ✅ **他窗已实现**（i18n 双语 + `neobot-root.tsx:1999` 渲染分支） |
| T0-4 | 三套层词汇裁决（文档） | L | 无（纯裁决） | 📋 待决（5A 引入第四套词汇，已在代码注释标注） |
| T0-3 | `content_omitted` 二选一 | S | 无 |
| T0-1 | `DeadCodeFitness` 断言化 + `lib.rs:23` 处置 | M | 配棘轮基线 |
| T0-4 | 三套层词汇裁决（文档） | L | 无（纯裁决） |

⭐ **顺序理由**：T0-2/T0-3 修完会**立刻冒出新问题** ⇒ 必须先有 T0-1 这类可信裁判，
否则新冒出的问题又会被"记录成已知"。

### T1 冗余清理（可并行，互不依赖）
| ID | 任务 | 成本 | 风险 | 状态 |
|---|---|---|---|---|
| T1-2 | `data_model` 8 类型转 `pub use` | S | 无（先例在同文件） | ✅ `b2f6be18` −160 行 |
| T1-3 | ~~`CostLadder` 二合一~~ | — | — | ⛔ **已推翻**（两种设计，见 §T1-3） |
| T1-4 | 删 `l6_meta/lib.rs` | S | 无（严格子集已证） | ✅ `43e2380c` −68 行 |
| T1-5 | 删 `l2_perception/nt_judgment/` | S | 先查跨 crate 消费者 | ✅ `43e2380c` −313 行 |
| T1-7a | `nt_core_guardian` **文档**修复 | S | 无 | 🟨 本轮 |
| T1-8 | 缓存岛 + "LRU"→FIFO 头修正 | S | 无 | 🟡 已修头（`ebf1d458`），剩可达性标注 |
| T1-1 | ~~3 个零消费者熔断器归正典~~ | — | — | ⛔ **已推翻**（三种并发模型，见 §T1-1）→ ✅ 改为标注 |
| T1-6 | 裁 `nt_shield::osint` | S | `compliance` 留待裁决 |
| T1-7b | `nt_core_guardian` 接线裁决 | L | 依赖裁决，不删 |

### T2 跨域（全部需先裁决）
| ID | 任务 | 成本 | 前置 |
|---|---|---|---|
| T2-1 | `EmotionLabel` 归属裁决 + 迁移 | L | 4 个同名 `EmotionEngine` 逐一核实 |
| T0-4 | 层词汇映射表 | L | 与 T2-1 同一决策面 |

---

## 6. ⛔ 本轮**不做**的（附实体依据）

| 不做 | 依据 |
|---|---|
| 批量归并 258 组"可归并" | 全是**候选**不是结论（`CODE-TOPOLOGY.md` §维度 8 自述"下一个同类缺陷仍可能存在"）；逐组需读 doc comment 判语义 |
| 修 `check-unwrap` 剩余 3 处 | 全部**需 API 变更**（`DEBT-LEDGER-2026-10-07.md` 已记：各试 4+ 版失败，正解在函数签名） |
| 动 4 个活的熔断器 | 构造签名不同（滑窗/强开/健康惩罚），可能带正典缺的语义 |
| 删 `nt_core_guardian` | 已建+已测+未接线 ⇒ 本仓既有裁决是"不删" |
| 删 `nt_shield::compliance` | OWASP/ASVS 是**策略**内容，域归属需裁决 |
| 归并 `l1_facade*` 家族 | `AGENTS.md` §4.2 明定 facade 是跨层引用**唯一合法通道** |
| 修 `lib.rs:23` 而不配基线 | 会让 CI 全红 ⇒ 须与 T0-1 同批 |
| 13 条已知分层违规 | 门已棘轮保护（0 new）；清它们属结构性改动，须 `cargo clean && cargo build` **跑两遍** |

---

## 6.5 ⭐ 补腿：外部资料 + 底层模型逆向推理（本节为第二轮追加）

> 首版只做了「仓内工具 + 子代理」两腿。用户原话要求「**结合搜索外部信息的所有技术资料
> **和**从底层模型逆向推理」⇒ 补第三、四腿。
> ⛔ 本节仍不新建并行模块，全部落既有骨架（`.neotrix/layer-map.json` 的 7 个 domain）。

### 6.5.1 第四腿：从 LLM 本身的机制逆向推出「判据」（而非抄检测器）

⭐ **不抄工具，抄判据。** 外部 Rust 重复检测器（`find-dup-defs` / `cargo-dupes` /
`dupehound` / `cddm` / `reDUP`）的**实现**我方不需要 —— 我方已有
`scripts/ops/nt_dup_types.py`。但它们**默认开启的检测层**暴露了我方判据的盲区：

| 检测层 | 外部做法 | 我方现状 |
|---|---|---|
| **Type-1** 同名同体 | 名字门（`nt_dup_types.py` 按 `(name, fields)` 分组） | ✅ 已有 |
| **Type-2** 改名同体 | alpha-renamed AST 规范化（标识符→`_v0.._vn`） | ❌ **无** |
| **Type-3** 改名+改写同体 | IDF 加权余弦 / L2AP simjoin | ❌ **无** |

⇒ **这正是我方 `CircuitState ×6` 低估 40% 的根因**：Type-1 只认名字，
而 `BreakerState` / `CircuitBreaker`（类型别名）/ 包装类型全在 Type-1 之外。

⭐⭐ **从 LLM 机制反推的判据（这是"逆向推理"的部分）**：
| LLM 机制 | 迁移到架构治理 |
|---|---|
| **同 token 序列**才能被同一模型高效处理 | ⇒ 只有**名字相同**才算重复，会漏掉改名同体的机制 |
| **embedding 按语义而非字面** | ⇒ 应按**方法签名集合**聚类，而非类型名字面 |
| **MoE 靠 router 分发到同专家** | ⇒ 重复的判据应是「**谁能处理同一类请求**」，不是「谁叫同一个名字」 |

⇒ **落到一句可执行的判据改造**：
`nt_dup_types.py` 的分组键从 `(name, fields)` 改为 `(method_signature_set, fields)`，
即可覆盖 Type-2/Type-3，**且零新模块**（改一个函数）。

⚠️ **诚实边界**：改判据会让数字**上升**（会多报）⇒ 按 D-16 与本轮 L8，
**必须配 `--calibrate` 式阈值校准**（外部工具的共识做法：先出直方图，
再按 p50/p75/p90 定阈值），**否则就是第二个误报门**（对照 `check-term-viz` 336 误报前车）。

### 6.5.2 第三腿：外部资料给出的**两条定量判据**（可直接用于排期）

| 资料 | 关键结论 | 对我方的意义 |
|---|---|---|
| **arXiv 0707.4166**《Parsimony Principles for Software Components》 | MDL 判「抽象的恰当粒度」= **使 use case 描述最短**的那个分量；且明确警告**抽象能力↑ ⇒ 逆问题复杂度↑ 且迅速变得不可计算** | ⇒ ⭐ **本仓 facade 家族的正确性判据在此**：facade 是"抽象能力"，它让跨层描述变短，但代价是逆问题变难。⇒ **门禁只能查「引用方向对不对」，不能查「这个抽象值不值得」** |
| **cs/0508023**《Libraries and Reuse: Entropy, Kolmogorov Complexity, Zipf》 | 域熵 `H` 决定复用上限：最多复用 `(1-H)`；`H→0` 才是"强复用" | ⇒ ⭐ **本仓 910k 行的域熵必然不低 ⇒ 「强复用」在本仓不是可达目标** ⇒ T1 各项的价值应按**局部 `H`** 估，不能按"消除重复的总行数"估 |
| **Fowler《Anemic Domain Model》** | 数据与服务分离 ⇒ 付了领域模型的代价却不享其收益 | ⇒ 我方 `data_model.rs` 重复 8 类型 + `nt_core_guardian` 零消费，两者都是**「声称是模型，实为袋子」**的变体 |
| **《Monolith First》/《Don't start with monolith》**（互相反对） | ⛔ **两篇权威文章结论相反** | ⇒ **不引用为依据**。教训：一个话题查到两篇相反的权威文章时，正确动作是**记录分歧**而非选一边 |
| **Lenhard et al. ECSA 2017** | 现有 code smell 工具**不足以**判定架构腐化，需针对具体系统调优 | ⇒ **解释了为什么我方 35 个门需要人工裁决记录**（`FULL-AUDIT-2026-10-06` 的 50+ 门正是"针对本系统调优"的产物）⇒ ⛔ 别指望换个工具就免掉裁决 |

### 6.5.3 熔炼落点（**全部化为已有**，零新建模块）

| 外部判据 | 落既有骨架 | 状态 |
|---|---|---|
| Type-2/3 重复检测 | `scripts/ops/nt_dup_types.py`（改分组键，**不改架构**） | 📋 intake（需配阈值校准） |
| 抽象粒度 MDL 判据 | **D-16 最短描述公理**（本仓已立，`NEOTRIX-MASTER-BLUEPRINT.md` §20） | ✅ **强化**，已上位 |
| 码长口径 = **token 数**（`0707.4166` 实测 token 数对重命名/注释/空白不变） | `nt_output_distill.rs` / `context_mgmt.rs` 已有 `token_count` | ✅ **已有**，D-16 的「两项同时报」有了具体口径 |
| 域熵 `H` 估复用上限 | ⛔ **我方无 Shannon 度量**（实测 `shannon|entropy_bits` 仅 2 处无关命中） | 📋 intake —— 且**不应先做**：D-16 才是它的前提 |
| facade 抽象的正当性判据 | `AGENTS.md` §4.2 + `l*_facade.rs` 家族 | ⚠️ **门只查方向**（已实测健康），**查不了值不值得** |

⭐ **最重要的一个结论**：
**外部资料没能给我方任何一条"该建的检测器"，只给了两条"该改的判据"和一条"该承认的天花板"。**
⇒ 按 NTS-B10.2（化为已有）与本轮 §6（不做清单），本节**零新建模块**是正确结果，不是偷懒。

---

## 6.6 ⭐ 第三轮追加：**Domain 轴**错位（第二轮只做了 Layer 轴）

> ⛔ **一个前提纠正，它改变了整个任务**：registry 里的 `layer` 字段**不是目录层轴**。
> 实测 `l0primitive` 在全仓只出现 3 处（枚举定义 `node.rs:145-158`、解析器 `roadmap.rs:63`、一张表）
> ⇒ **318 个节点没有一个带目录层**。它是**第三条轴**（能力高度：primitive→composite→autonomic→autonomic），
> 与目录层轴**正交**。按字面 cross-tab 出来的不是错位，是第三轴对域。
> ⇒ 下表按**目录层 × 域**交叉（63/318 前缀不可解析，单列）。

### layer × domain 全交叉表（318 节点）

| domain ⇩ / layer ⇨ | NO_CODE | l0_sub | l1_act | l2_per | l3_emb | l4_emo | l5_cog | l6_meta | 合计 |
|---|---|---|---|---|---|---|---|---|---|
| core | 7 | 12 | 0 | 2 | 0 | 0 | 5 | 3 | **29** |
| mind | 14 | 0 | 0 | 0 | 0 | 0 | 24 | 9 | **47** |
| memory | 4 | 0 | 0 | 0 | 0 | 32 | 0 | 0 | **36** |
| world | 9 | 0 | 0 | 31 | 0 | 0 | 0 | 0 | **40** |
| act | 5 | 0 | 46 | 0 | 0 | 0 | 1 | 0 | **52** |
| shield | 1 | 0 | 0 | 0 | 25 | 0 | 0 | 0 | **26** |
| io | 7 | 0 | 37 | 0 | 0 | 1 | 0 | 0 | **45** |
| meta | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 9 | **17** |
| nexus | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | **6** |
| governance | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 6 | **9** |
| repair | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 8 | **9** |
| **合计** | **63** | 12 | 83 | 33 | 25 | 33 | 30 | 36 | **318** |

⭐ **这张表接近完美对角**：255 个可解析节点中只有 **13 格偏离**对角。
⇒ **这不是一条被装饰过的轴，而是一条穿了两个名字的轴**（`world↔l2`、`shield↔l3`、
`memory↔l4`、`act↔l1`、`io↔l1` 近乎 100% 纯）。
⇒ 而后加的 **4 个域（meta/nexus/governance/repair）全部塌进同一个 L6 桶**：24 节点 / 4 域 / 1 目录。

### 新发现（全部主 agent 复核过代码行）

| ID | 级别 | 位置 | 错位内容 | 成本 |
|---|---|---|---|---|
| **D-1** | High | `l6_meta/mod.rs:20-21` | ⛔ **NT-REPAIR 代码住在 NT-GOVERNANCE 目录里**：`pub use coordination as nt_governance; pub use healing as nt_repair;` 而 `coordination/nt_mind_repair/mod.rs:1` 自述「NT-REPAIR 自愈层」⇒ **一个域两个目录，无边界** | M |
| **D-2** | High | `evolution.rs:427` | ⛔ **跨域重复对唯一的重复检测器结构性不可见**：`acc.entry((n.domain, n.layer))` ⇒ 按 `(domain, layer)` 分组 ⇒ 跨域同 tag 永不同组。实测 `auto_fix` 双注册：`nt_mind_autofixer::autofixer`(mind) 与 `nt_mind_repair::autofixer`(repair)，同 `l1composite`、**同 wiring_evidence 字符串** ⇒ 全库 **14/742 个 `provides` tag 跨域**，全部不可见。`:414` 注释自称「发现分散重复能力」—— 只发现**同域**分散 | M |
| **D-3** | Medium | `nt_mind_repair/skill_improver/mod.rs:4` | 注释声称「**集成 nt_mind_autofixer::autofixer**」⇒ 实测**零引用**（`AutoFixer|auto_fix` 零命中；全文唯一 `use` 是测试里的 `use super::*`）⇒ 474 行模块文档断言了一个不存在的依赖。⛔ `check-doc-claims.sh` 只查「文档声称零消费者」，**方向相反**，故漏过 | S |
| **D-4** | Medium | `capability_registry.json` | **NT-GOVERNANCE 一个实现两种 id 拼法**：`nt_governance::*`(6) vs `governance::enforcement`(1) ⇒ 而 `nt_governance` 是**整个 `coordination/` 的别名**（`mod.rs:20`），`governance` 却是真子目录 ⇒ 同代码两套命名，`fuse.rs:188` 按前缀分流会漏 | S |
| **D-5** | Medium | `SKILL.md:132` / `nt_absorb_mapper.rs:19` | ⭐ **吸收工作流文档写 7 域，枚举有 11 域**，而 `nt_absorb_mapper.rs:19` 白纸黑字「7 域」+ `branch_capabilities()` 只返回 7 个 key。**但真正的真相是：这两个 mapper 都是死代码**（`branch_capabilities` 零非测试调用；`seal/domain_mapper.rs` 是未编译孤儿）⇒ **活的是 `l1_action/nt_capability_bridge.rs:70` `ROUTE_TABLE`，它经 `parse_domain` 处理全部 11 域**。⇒ 不是「文档陈旧」，是**「文档描述的执行路径上根本没有这段代码」** | M |
| **D-6** | Low | `capability_registry.json` | **63/318（20%）节点 id 解析不到任何代码**（35 个 `exp::` 虚拟节点 + 28 个），其中 **29 个声称能力 tag**、8 个参与 registry 边。⛔ `capability-truth` 与 `nt-registry-determinism.sh` **都不校验 id 是否可解析到代码** ⇒ Dark Forest 三要素（编译+测试+消费者）缺第三项 | M |

### ⛔ 第四腿再逆推：从 LLM 的 **attention sink** 反推我方的遗忘策略

⭐ **这条来自真实机制的意外发现**，且直接命中我方生产代码：

| LLM 机制（实测有文献） | 迁移到我方 |
|---|---|
| **attention sink**：softmax 分母恒为 1，模型把"想弱注意力"的多余概率质量**倾倒到最早的少数 token**；**任何删掉 sink 的 eviction 方案都会让生成质量崩塌** | ⇒ **按"最旧"剪枝是危险的默认** |
| StreamingLLM 的解法：保留**最前 4 个** + 最近窗口 `W`，其余丢弃 | ⇒ 显式的 **sentinel 保留区** |

**实测我方现状**：`decay_forgetting/pruner.rs:49` `prune_by_retention`
按 `age` + `access_count` 算 `compute_retention` 再 `retain(>= min_retention)`
⇒ **纯年龄/访问驱动，无任何"最早期条目"保护**。

⚠️ **诚实边界**（必须一起记，否则就是又一次"看类比硬套"）：
- attention sink 是**softmax 数值性质**，LLM 特有；
- 我方是**文件系统式记忆 + 手写 retention 公式**，**无 softmax 分母**，**机制不同源**。
⇒ 因此**这不是缺陷判定**，而是 📋 **一个被外部机制提示的待验假设**：
「本仓最早写入的记忆是否承担了超出其 retention 分数的锚点作用？」
⇒ 验证需要 held-out 实验（先移除最早期条目，看下游 recall 是否塌），
**不是改代码能回答的** ⇒ 按 NTS-B10.2 记 intake，**不进 T0/T1**。

⭐ **但同一条资料给出了两个「我方已有」的强确认**：
| vLLM/TensorRT 机制 | 我方对应物 | 实测 |
|---|---|---|
| **paged KV + 内容哈希前缀复用**（prefix caching，共享前缀只存一次） | `tiered_memory/` | ✅ **已有**：`traits.rs:179` `budget_evictions` 优先级驱逐、`:199` 计数 |
| **分优先级 LRU**（priority 0-100，低优先级先逐出） | 同上 | ✅ **已有**：即上条 |
| `shared_prefix` 显式机制 | — | ❌ 0 处（但由上面的分优先级驱逐覆盖同一目的） |

⇒ **结论：这两条无需新建任何东西。**（第二轮四腿的落点纪律在此继续成立。）

---

## 7. 方法论沉淀（并入 `LESSONS-2026-10-07-open-gate-record-truth.md`）

⭐⭐ **新增第 8 条：绿色不等于有效。**

本轮 3 条 P0 里，**2 条是"报告了 PASS 但结构上不可能失败"**。
判据（可直接复用）：

> 一条"保证"要成立，必须**同时**满足三条：
> ① 检测器**能被触发**（喂真实输入能冒出失败）
> ② 结果**被消费为门**（断言 `.passed`，不是只断言 name 非空）
> ③ 抑制器**不覆盖检测器自己的视野**
>
> 本轮三条链各断在一条：
> `DeadCodeFitness` 断 ②+③（`#![allow(dead_code)]` 压掉信号 + 只断言 name）
> `ArchLayer` 断 ①（目录被 `continue` 跳过，可分类文件 = 0）
> `content_omitted` 断 ②（写了三处 doc 承诺，UI 零读点）

⭐⭐ **附带一条量级修正**：**同名检测器看不见"同机制、异类型名"。**
`CircuitState ×6` 的台账数字**低估约 40%** —— 因为 `BreakerState`、
`CircuitBreaker`（别名）、包装类型都躲开了名字比对。
⇒ 任何"可归并 N 组"的数字，都应视为**下界**。
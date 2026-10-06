# 后续任务总账 —— 2026-10-06

> 单份权威清单。每项都带**可验证的完成信号**，不接受「看起来做完了」。
> 数字全部来自实测，不从记忆。
> 本账取代散落在各处的 todo；新增项必须先登记到这里再动手。

## 0. 当前基线（实测）

| 项 | 值 |
|---|---:|
| `check-silent-failure --strict` | RC=0 |
| `check-dead-config-flag --strict`（bool / numeric） | RC=0 |
| `check-orphan-dirs --strict` | RC=0（孤儿 2 个 / 12 `.rs`，基线已裁决 2，新增 0） |
| `map-check --strict` | RC=0 |
| `check-test-baseline --strict` | RC=0 |
| `check_layer_map_consumers` | RC=0 |
| `nt_map_reconcile` | **38 断言 / 38 HOLDS / 0 VIOLATED** |
| `check-doc-drift --strict` | **RC=1**（见 T3.1） |
| neobot 测试 | 561 passed / 0 failed / 1 ignored |
| core 测试 | 13,580 passed / 0 failed / 38 ignored |

## P0 —— 阻塞生产闭环

### T0.1 执行端口 / B1 方案 C

**问题**：`capability_invoke` 在生产中**永不上桌**。挂载点
（`nt_http_engine.rs:470` `tool_schemas`）与播种点
（`nt_mind_background_loop/mod.rs:180`）在生产中从不同进程 ⇒ 注册表恒空
⇒ `market_ids` 恒空 ⇒ `:471` 的守卫恒假。

**这不是「忘了调」**：neobot 不得反向依赖 core，能力元数据的真身在
core L1 `nt_act_trade`。方案详见 `B1-CAPABILITY-INVOKE-WIRING-2026-10-06.md`。

**⇒ 裁决：选方案 C**（2026-10-06，依据见下）。**方案 B 不再需要。**

### 裁决依据（实测字段比对）

| 字段 | `CapabilityNode`（`nt-core-capability-tree`，**两侧都能用**） | `MarketEntry`（neobot） |
|---|---|---|
| `id` / `kind` / `domain` | ✅ 已有 | ✅ 需要 |
| `category` / `version` / `license` / `description` / `tags` / `maturity` | ❌ **不在共享 crate** | ✅ 需要 |

⇒ 能力元数据目前**分裂在两处**：3 个字段在共享 crate，6 个在别处。

**为什么 C 而非 B**：B 与 C 不是同类选项 —— B 答「数据怎么送过去」，
C 答「数据住在哪」。若数据住在共享 crate，neobot 直接读它**已经依赖**的
crate（`Cargo.toml:42`）⇒ 送数据这一步自动消失 ⇒ **B 变成多余**。
那 6 个字段是**关于该能力的描述性元数据**，它的家就是能力节点本身，不是 neobot。

#### ② 实施路径已被前序窗口做完（2026-10-06 核实）—— 真正的缺口只剩一个

逐项核实结果：

| 环节 | 状态 | 证据 |
|---|---|---|
| 读侧（市场元数据 → `MarketEntry`） | ✅ **已实现** | `nt_capability_market::project()` 已按 `meta_keys::CATEGORY/VERSION/LICENSE/DESCRIPTION` 从 `node.metadata` 读 |
| 写侧（播种时打标） | ✅ **已实现** | `consciousness_runtime.rs:1208-1215` 的 `must_be_registered` 每次 bootstrap 都调`apply_market_meta(n, category, ..)`（2026-10-04） |
| 播种被调用 | ✅ **已接线** | `consciousness_runtime.rs:138` 在 `ConsciousnessRuntime::new()` 里调 `bootstrap_trade_capabilities()` |
| **播种点与挂载点是否同进程** | ❌ **唯一缺口** | 播种在 core `ConsciousnessRuntime::new()`；挂载在 neobot `HttpEngine::tool_schemas`（`nt_http_engine.rs:470`）。neobot **不得**依赖 core ⇒ **无路径让两者相遇** |

⇒ **收敛后的精确根因**：不是「元数据没定义」，不是「没播种」，而是
**播种代码住在 core、消费代码住在 neobot，而依赖方向禁止 neobot 触达 core**
⇒ 注册表在服务进程里恒空 ⇒ `market_ids` 恒空 ⇒ `tool_schemas` 的守卫恒假。

⇒ **方案 C 的精确形式**：把**播种动作**（连同 `apply_market_meta` 所需的
`TradeCategory`）下沉到 `nt-core-capability-tree`，让
`ConsciousnessRuntime::new()`（core）与 neobot 的入口路径**都能触发同一份播种**。
这样「登记了」与「能被找到」在**两个 crate 里都是同一个承诺**，
而真源只有一份。

⚠️ 我在② 里写的「用已有 `metadata`、避开 70 处 E0063」这个方案方向对但**描述不准确**：
读侧本来就在用 `metadata`，**不需要我做任何改动**。真正的改动是把播种下沉。

#### ③ 「把播种下沉」也不可行 —— 依赖方向决定了没有装配层（2026-10-06 实测）

核实工作区依赖后**排除了方案 C 的字面形式**：

- workspace 成员含 `neotrix-core` 与 `neotrix-neobot`，但
  **`apps/neobot-desktop` 只依赖 `neotrix-neobot`，不依赖 `neotrix-core`**；
  逐个 `Cargo.toml` 核实：**没有任何 crate 同时依赖两者**。
- 而 `consciousness_runtime.rs:127` 已在用 `neotrix_neobot::nt_capability_canary::expect`
  ⇒ **core 依赖 neobot**，方向固定为 `core → neobot`
  ⇒ neobot 不能反向依赖 core
  ⇒ **按当前依赖方向，不存在能同时看到两侧的装配层**（这不是疏漏，是方向的必然结果）。

且「把 5 个 registrar 下沉到共享 crate」**不可行** —— 它们构造节点时引用
core L1 的领域类型（`TradeCategory`、`TradeCapability` 等），不是纯数据。

⇒ **唯一可行形式：把「市场清单」做成共享 crate 里的纯数据。**

| | 内容 | 家 |
|---|---|---|
| **市场事实**（id / category / version / license / description） | 纯数据，无core 类型 | **新增**：`nt-core-capability-tree::market_manifest()` |
| **可执行实现**（registrar / TradeCapability） | 引用 core L1 类型 | 维持 core（`bootstrap_trade_capabilities`） |

#### ⑤ 最后一个阻塞点：**version / license 不能随清单下沉**（2026-10-06 实测）

`TRADE_ABILITY_VERSION = env!("CARGO_PKG_VERSION")`
（`capability_registry.rs:981`）⇒ 它取的是 **core 这个 crate 的包版本**。

⇒ **清单若放进共享 crate 并自带 version，`env!` 会解析成共享 crate 的版本
⇒ 静默改掉对外报告的版本号。** 而 `meta_keys` 注释明确
`VERSION` / `LICENSE` **缺失即不可上架** ⇒ 清单单独存在**不足以**让
neobot 侧 `listable()` 为真。

⇒ 字段必须按**语义**切分，而非按「谁方便」：

| 字段 | 语义 | 归属 | 理由 |
|---|---|---|---|
| `id` / `category` / `description` | **这个能力是什么** | **共享 crate**（纯数据） | 跨 crate 稳定，与实现无关 |
| `version` | **本仓实现版本** | **core**（`env!`） | 移动会静默改变其含义 |
| `license` | **本仓的许可决策** | **core**（`LicenseRef-NeoTrix-Internal`） | 这是关于**本仓实现**的声明，不是关于能力概念的 |

⇒ **由此剩一个真实的策略决策（不是技术问题）**：neobot 侧要让 `listable()`
为真必须拿到 version + license；它既不能依赖 core，**又不能自己编**
（编出来就会与 core 漂移）。

**候选（需裁决）**：
- **A** neobot 从自己的 `env!("CARGO_PKG_VERSION")` 取 + 复用同一 license 串
  ⇒ 两侧版本号会不同（除非两 crate 版本严格同步）⇒ 需显式接受或加一致性检查；
- **B** `trade_manifest(version, license)` 改为**显式入参** ⇒ 无漂移，
  但调用方仍须从 core 拿值 ⇒ 又回到「谁同时依赖两侧」的死结；
- **C** 判定二者属**市场呈现面**，由 neobot 声明，core 的 `apply_market_meta`
  不再写这两项 ⇒ 需确认不破坏现有回查断言。

⛔ 我**不擅自选** —— 这是关于「对外声称什么版本、什么许可」的表述决策；
编一个值会让市场门变绿却让**对外声明与实现脱节**，比现在不绿更糟。
⇒ 两侧都加载**同一份清单** ⇒ 真源一份：
- neobot 入口加载清单 ⇒ `market_ids` 非空 ⇒ `tool_schemas` 的守卫不再恒假；
- core 的 `must_be_registered` 继续注册实现并 `apply_market_meta`
  **写同样的键**（幂等）⇒ 两边写同一份事实，不会漂移。

**这不是「第二个真身」**：清单只存**市场事实**，实现只在 core；
两侧写的是**同一个键空间**，且 core 侧幂等覆盖。

**验收信号不变**：`capability_invoke` 出现在 `tool_schemas`。

#### ④（原③）加 6 个类型化字段是错的，勿采纳

我原计划「给 `CapabilityNode` 加 6 个类型化字段」。**核实后发现这条路是错的**：

- `CapabilityNode` **已有 `metadata: HashMap<String, serde_json::Value>`**（`node.rs:363`），
  且**已在活跃使用**（`registry.rs` 用 `contract_de…`、`epistemic` 等字符串键）；
- `CapabilityNode` **不derive `Default`**，三个构造器都用 `Self { .. }` 字面量
  ⇒ 加字段会触发 **E0063，波及 70 处构造点**（正是 `A48` 记载的陷阱）。

⇒ **最优解：在共享 crate 定义规范键名，让 neobot 的 `project()`
从 `node.metadata` 读这 6 个字段。**

| 维度 | 加 6 个类型化字段 | 用已有 `metadata` + 规范键名 |
|---|---|---|
| 破坏面 | **70 处构造点**（E0063） | **0** |
| 真源位置 | 共享 crate ✔ | 共享 crate ✔ |
| 可扩展性 | 改一次要改结构体 | 加键即可 |
| 跨语言消费 | 需序列化字段 | `serde_json::Value` 已跨 FFI |

**执行路径**（下一项）：
1. 共享 crate 定义规范键名常量（如 `MARKET_CATEGORY` / `MARKET_VERSION` /
   `MARKET_LICENSE` / `MARKET_DESCRIPTION` / `MARKET_TAGS` / `MARKET_MATURITY`）；
2. `nt_capability_market::project()` 从 `node.metadata` 读它们填 `MarketEntry`；
3. core 侧 5 个 trade 能力在**播种时**写入这些键
   （`consciousness_runtime.rs:121-125` 已是唯一播种点）；
4. 验收：`neobot-check-market.sh` 不再报「5 个能力从未被调用」以外的陈旧项，
   且 `capability_invoke` 出现在 `tool_schemas`。

### 前置：必须先消除一个同名冲突（实测发现）

存在**两个 `CapabilityNode`**：

| 位置 | 形态 |
|---|---|
| `crates/nt-core-capability-tree/src/node.rs:328` | **结构体**（有 `new_primitive` / `new_composite` / `new_constellation`） |
| `neotrix-core/src/l0_substrate/nt_core_traits.rs:27` | **同名 trait** |

⇒ 又是 L15「同名 ≠ 同一符号」。**先统一这两个，再谈把元数据下沉** ——
否则会把描述性字段加到错误的符号上。

#### ① 已裁决并执行（2026-10-06）：**消歧，不合并**

实测两个概念**都活着、且语义不同**：

| | A. core trait | B. 共享 crate 结构体 |
|---|---|---|
| 形态 | trait（行为接口） | struct（数据模型） |
| 语义 | **运行期能力提供者**，可被编排、可声明依赖 | **能力树节点** `id`/`domain`/`kind`/`provides` |
| 活的证据 | 4 个活跃 impl（`ConsonanceOrchestrator`/`TranscendentLoop`/`EvolutionHarness`/`MetaObserver`） | 70 处构造 + neobot 依赖 |

⇒ **不是「合并两个死物」，是给两个活概念消歧。** 正确处置是**改名，不是合并**。

**但改名不现在做**——影响面已量化：改名 trait → `CapabilityProvider`
触及 **60 处 / 18 个文件**，含 B1 核心的 5 个 trade 能力，
且 `DIR-REMEDY` 记过「活路径误判」事故。

**本轮实际执行（最小且能消除真实风险）**：
在两个符号上各加**语义消歧文档**，含对照表 + **明令禁止把市场元数据加到 trait 上**
+ 元数据的正确归宿 + 已量化的改名计划。
⇒ 直接消除「加错字段」这个真实风险（我此前正是担心这个才不敢动手）。

**改名作为独立任务 T4.4 保留**，待有整块时间做 60 处活代码改名 + 完整验证。

- ⛔ **方案 A（neobot 自带清单自播种）已判定不可接受** —— 会产生第二个真身。
  依据：本仓已记录四起「两份同源数据」资产最终两份都腐化。
- ✅ 推荐 **方案 C**：把能力清单下沉到 `nt-core-capability-tree`
  （neobot 已依赖它，`Cargo.toml:42`），两侧共读。退而取其次选方案 B（注入端口）。

**完成信号**：`neobot-check-market.sh` 能观测到 `capability_invoke` 出现在
`tool_schemas`（**当前该门看不到，需先补这条断言**）。

### T0.2 统一 capability id 空间

`expect()` 登记能力树 id（`NT-MEMORY::trade::trade_product_spec` 等 5 个），
`signal()` 收到 `TradeCapability` meta id（`PriceCalculatorCapability` 等）
⇒ 两个空间不相交。

`07aabb9f` 已让这个不相交**可见**（`unmatched_signals()` + CLI 展示），
但**统一映射本身未做** —— 它依赖 T0.1 的执行端口（需要知道每个能力的真实 id）。

**完成信号**：`neobot capability canary` 的「未登记信号」为 0。

## P1 —— 逐条工作（**不可自动化**，见 T4.2）

### T1.1 188 个待人工判定死字段

| 模式 | 外部消费者 | 配置噪声 | **待判定** |
|---|---:|---:|---:|
| bool | 423 | 31 | **135** |
| numeric | 225 | 37 | **53** |

分布：`nt_mind` 33 · `nt_shield` 32 · `nt_io` 31 · `nt_act` 22 ·
`nt_memory` 11 · `nt_core` 7 · `nt_world` 7 · `nt_media` 6 ·
`consciousness_tree` 5。

判定口径只有两类：**配置先行、能力缺位**（要实现子系统）或
**产出无人看**（观测/遥测字段）。两者都不是「删字段」能解决的。

**建议分批**：`nt_shield`(32) 与 `nt_io`(31) 各自成组，可并行审计。

### T1.2 ~95 条真实死链

四类已裁定：

| 类 | 数量 | 处置 |
|---|---:|---|
| ① 散文片段被误当路径 | 8 | **不追**（收紧正则会引入漏报） |
| ② 真实死链 | ~95 | **逐条人工判断** |
| ③ 历史文档指向已归档目录 | 13 | **不修**（删=篡改历史；恢复不划算） |
| ④ basename 自动重指 | — | ⛔ **已证伪，禁止**（见 T4.2） |

### T1.3 neobot 孤儿三件套（1,355 行，读点=1）

`nt_token_guard`(379) / `nt_daemon`(109) / `nt_git`(867)。
`nt_git` 与 core 的 `git_integration.rs:11` **重复** ⇒ 先裁决归属再动。
`DIR-REMEDY` 记过「活路径误判」事故，不直接删。

### T1.4 CODEBASE-WIKI-2026-09-21.md 数字陈旧

`2,426` vs 实测 `2,865`；`798,707` vs `911,763` 行。
该档的生成器（`/wiki generate`）已不存在 ⇒ 只能手工修或归档。

## P2 —— 需策略裁决

### T2.1 `.project-map/` 354M 被 gitignore

`.gitignore:323` 忽略整个目录，而它是 `nt_mapgen.py` 的**唯一产物**
⇒ 别的 agent 与 CI 拿不到，而 CODE-TOPOLOGY 曾称其为「1 秒重建的索引」。

⛔ 不能顺手改 `.gitignore`：会一次性把 354M 纳入版本库
（`edges-all.jsonl` 176M + `edges-neotrix-core.jsonl` 165M）。
需与仓库体积策略一起裁决。可选方向：提交精简索引 + 大文件继续忽略。

### T2.2 其余

- `ARCHITECTURE-MAP-ROADMAP-V2.md` 的「50 个失败测试」vs baseline 实为 0 字节
- `NT-MEMORY::trade::*` 5 个能力与 `TradeCapability` 的映射表缺失（T0.2 的前提）

## T3 —— 门与治理的已知缺口

### T3.1 `check-doc-drift --strict` 红于既有项

实测：旧扫描面（6 份根文档）**同样 RC=1** ⇒ 非本次引入。
红点是 `missing //! module docs: 125`。**死链部分已绿**（129 known / 0 新增）。

⇒ 需决定：125 项未文档化 `.rs` 是补 `//!` 头还是棘轮化。

### T3.2 `nt_map_reconcile` 覆盖面仍窄

38 条断言只覆盖 4 份文档。CODEBASE-WIKI / layer-map 的数字仍无断言。

### T3.3 `check-naming` 仍是空洞绿

规约 vs 现实差 1,646 个无前缀文件 ⇒ advisory PASS ≠ 合规。
`scripts/gate-registry.tsv` 登记 23 道门，**地图类工具 0 登记**
⇒ `map-check.sh` / `nt_topology.py` / `nt_mapgen.py` 不受「注入违规必须变红」审计。

## T4 —— 元治理（已完成项的固化，防复发）

### T4.1 已建立的纪律

1. **门必须能被证伪** —— 每个新门都做变异验证
2. **同名 ≠ 同一符号**（L15）—— 用于门、幽灵检测、枚举插入
3. **写盘前必须验证**：`assert` 替换发生 + 确认无截断副作用 + 改后重跑门
4. **分类优于清零**：死链 129 里真该修 ~95；死字段 904 里待判定 188
5. **不写无法验证的结论**：生成器只渲染算得出的事实

### T4.4 CapabilityNode 改名（已量化，待整块时间）

trait → `CapabilityProvider`。**60 处 / 18 文件**，含 5 个 trade 能力。
要求：改名 + `cargo check` + 全量测试 + 门终态，**不得与能力接线混做**。

### T4.2 已实测证伪、禁止重试的方案

| 方案 | 证伪证据 |
|---|---|
| basename 自动重指死链 | `crates/neotrix-decision-engine/src/types.rs` → `ffi/types.rs`（不同文件）；`nt_game/src/nt_clock.rs` → `nt_mind/nt_game/nt_clock.rs`（无关模块） |
| 收紧 doc-drift 正则以消除散文误报 | 会引入**漏报**；门漏报比误报更危险 |
| neobot 自带能力清单（B1 方案 A） | 产生第二个真身；本仓四起同源数据资产均腐化 |
| 「cargo check 通过 ⇒ 能力可用」 | A51–A55 五个模块 check 干净却藏 UTF-8 panic / PPR 数学 bug / 单位错配 |

## 执行顺序建议

```
T0.1（架构裁决，需你确认方案 C）
  └→ T0.2（依赖 T0.1）
T1.1 nt_shield(32) ∥ T1.1 nt_io(31)   ← 可并行审计
T1.2 逐条死链
T1.3 → T1.4
T3.1 / T3.3（门自身治理，不依赖任何裁决）
T2.1（需体积策略）
```

**不建议**在 T0.1 裁决前推进 T0.2 —— 它需要知道每个能力的真实执行 id。
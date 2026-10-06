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

### T4.5 派发端口的签名与执行形态不匹配（2026-10-06 实测，下一窗口先做）

我已在共享 crate 落地 `nt_core_capability_tree::dispatch`（接口完整、4 个测试）
并让 neobot 消费（`4c42ae7c`）。但**核实后发现两个障碍**：

|障碍 | 证据 |
|---|---|
| **执行是 `async`，端口是同步 `fn` 指针** | `TradeCapability::execute_trade` 是 `async fn`（`nt_act_trade/capability_registry.rs:32`） |
| **无全局注册表** | `TradeCapabilityRegistry` 只有 `new() -> Self`（实例）；`create_default_registry()` 非测试引用为 0 ⇒ 无捕获 `fn` **无处取实例** |

⇒ **在解决这两点前，`register_dispatcher` 不应被真实实现调用。**
当前生产 `dispatch` 返回 `None` ⇒ 调用方fail-closed ⇒ **与接线前行为一致**（未引入回归）。

### ⭐⭐⭐ P0.2 派发端口**无法统一接线**——5 个能力的执行面**异构**（2026-10-06 实测）

在把 `DispatchFn` 对齐 async、并给 `TradeCapabilityRegistry` 加了全局落点之后，
我去核实「派发对象到底是谁」，结论是**负面的**，且这是本轮最有价值的一条：

#### ① `CapabilityNode` **没有执行入口**

共享 crate `node.rs` 里**无** `handler` / `executor` / `invoke` 字段
（grep `pub (handler|executor|invoke|execute|run)\w*:` 只命中 `runeword*` 噪声）。
⇒ **能力树节点是纯元数据/结构**，不是可执行注册表。
⇒ 树 id → 可执行函数之间**没有任何结构性连接**，只有「按约定对齐 id」。

#### ② 5 个能力的可执行面**各不相同**

| 能力 | 可执行入口 | 形态 |
|---|---|---|
| `foreign_trade_full_cycle` | `execute_trade_full_cycle(TradeContext) -> TradeResult` | 自由函数 ✅ |
| `trade_quote_negotiation` | `execute_quote_negotiation(...)` | 自由函数 ✅ |
| `trade_production_logistics` | `customs_clearance` / `manage_bl` / `track_shipment` | **仅 engine 方法**（`ProductionEngine`/`LogisticsEngine` 是 unit struct + `Default`） |
| `trade_finance_compliance` | `verify_settlement` / `declare_tax_refund` / `reconcile_accounts` | **仅 `FinanceEngine` 方法** |
| `trade_product_spec` | `get_product_knowledge_pack(ProductType)` | **知识包工厂，不是执行器** |

⇒ **不存在统一的「树 id → 执行」契约。** 要接线就得为每个能力
**新造一份输入 schema**（JSON → 哪个方法的哪些参数）。

#### ③ 结论：**当前的 fail-closed 是正确行为，不是缺陷**

在没有统一执行契约前，`dispatch` 返回 `None` ⇒ 调用方 fail-closed
⇒ **不会把「没实现」伪装成「执行成功」**。

### ✅ P0.2-a 已接线 2 / 5 —— schema 就是 Rust 类型，零发明（2026-10-06）

复核 `TradeCapabilitySpec { context: TradeContext, result: TradeResult }` 后发现
**权威 schema 一直存在**：`TradeContext` 等类型都派生 `Serialize + Deserialize`
⇒ **schema 自描述**。而 `capability_spec()` 本身只是**空值样本**
（`buyer_id: String::new()`）⇒ **我先前把它当成 schema，才误判「需发明」**。

⇒ 新增 `nt_act_trade/tree_dispatch.rs`，注册**2 个**：

| 能力 | 执行器 | 输入 |
|---|---|---|
| `foreign_trade_full_cycle` | `execute_trade_full_cycle(TradeContext)` | `TradeContext` |
| `trade_quote_negotiation` | `execute_quote_negotiation(..)` | `{requirement, product_spec, market_env}` |

**接入生产**：`ConsciousnessRuntime::new()`，与 `bootstrap_trade_capabilities()`
**同处、同生命周期**。注册失败**不吞**（返回失败项 + `log::warn`）。

**4 个测试**：注册后 `None`→`Some`；真实执行（**输入取自仓内
`capability_spec().context`，零虚构数据**）；非法输入必须 `Err`；
**未注册的 3 个必须仍 `None`**。

### ✅ P0.3 neobot 侧无法 drive 派发 future —— 已用可判定分流解决（2026-10-06）

`execute_capability_invoke` 是**同步**函数（工具派发链全是 `Result<ToolResult>`），
而端口产出 `BoxFuture`。

⛔ **绝不能在 tokio runtime 内 `block_on`**（会 panic：
`Cannot start a runtime from within a runtime`），而「同步工具链被 async
调用方包着」是极常见情形。

⇒ 新增 `nt_dispatch_drive::drive`，用 `Handle::try_current()` **可判定**分流：

| 上下文 | 行为 |
|---|---|
| 不在 runtime 内（CLI / 同步驱动） | 起临时 `current_thread` runtime → **真实执行** |
| 在 runtime 内 | 返回明确 `DISPATCH_REQUIRES_SYNC_CONTEXT` ⇒ **fail-closed**（⛔ 不 panic、不假装成功） |

⇒ `Ok(Some(fut))` 现在**真实执行**，成功后才 `record_dispatch` 计数
（闭环要求「成功后计数」）。

⚠️ **嵌套 Result 语义**（编译器抓到我把期望写错）：
外层 `Err` = 驱动失败；外层 `Ok(Err(..))` = **驱动成功但实现失败**
⇒ 调用方**三层全匹配**，⛔ 不可把内层错误当驱动失败。

### ⚠️ 我在本轮**又踩了一次 R-SCAN-4**（反引号被 shell 执行）

写测试注释时用了 `` `Ok(Err(..))` ``，**zsh 当命令替换执行了**
（`command not found: Err`）⇒ 注释里的内容被**吃掉**，
只剩 `外层  = **驱动失败**`。

⚠️ `bash -n` 抓不到（语法完全合法），**只有核对写入内容才发现**。
⇒ 该纪律在 AGENTS.md 已记录，我在**同一个会话里**又犯。
⇒ 本次修复：**注释里不写反引号**，改用裸 `Ok(内层 Err)`。

### ⭐⭐⭐ P0.4 我接线的那个执行器**是桩** —— 而门本可以抓到它的输入形态（2026-10-06）

复查 `execute_trade_full_cycle` 的函数体：

```rust
machine.current_phase = TradePhase::Ft01CustomerDevelopment;
// ... execute FT01 logic          ← 注释，不是代码
machine.advance(TradePhase::Ft02RequirementConfirmation).ok();
```

⇒ **17 个阶段的「执行」全是注释**；函数只设置 `current_phase`。
且 10 处 `.ok()` **静默丢弃阶段推进错误**。

⇒ **含义**：`foreign_trade_full_cycle` 现在会被派发、会被计数、会被打金丝雀，
**而它没有真正做任何贸易逻辑**。
这正是本会话一路在治的病，**我在最后一棒踩了进去**。

⇒ ⛔ **不做的事**：不把派发接线撤掉（管道是对的，schema 是真的），
但**必须**让「它是个桩」这件事可见，且**不能**靠「执行成功」来宣称它工作。

#### ✅ 顺带修掉：门的一个真实覆盖漏洞（同一处形态）

`check-silent-failure` 的 opener 只认 `let _ = ...`
⇒ **`machine.advance(..).ok();` 这种裸语句从未进入扫描**。

⇒ 按证据放宽 opener（只加「以 `.ok()` 收尾的裸调用语句」，不放宽到任意裸语句）
⇒ 立刻出现：
- `.ok()` 丢弃 Result 共 **69 处**（新增报告项，每次运行打印，**不设门**）
- **2 处此前不可见的新命中**（真实缺陷，已修）：
  - `entry/sandbox_features.rs:63` `save_features` 写失败被吞 ⇒ 调用方以为已保存
  - `l5_cognition/nt_mind/foundation/guardian.rs:583` **守卫组件**快照写失败被吞
    ⇒ 出事时没有快照可查，**而守卫正是出事时靠它的**

⇒ 两处都改为 `log::warn!`（门的 OBSERVE 通道 ⇒ 既修缺陷又过门）。

### ✅ P0.5 `.ok()` 分诊产出：**3 处真实静默失败**（2026-10-06）

把 `.ok()` 做成**每次打印的报告项**（不设门），再按风险分诊，得到 3 处真缺陷：

| # | 位置 | 后果 |
|---|---|---|
| 1 | `entry/sandbox_features.rs:63` | 特征持久化写失败被吞 ⇒ 调用方以为已保存 |
| 2 | `l5_cognition/.../guardian.rs:583` | **守卫组件**快照写失败被吞 ⇒ 出事时无快照可查 |
| 3 | `l3_embodiment/.../nt_mcp_registry.rs` ×6 | **安全工具**注册失败被吞 ⇒ **安全面被悄悄缩小** |

⇒ 三者同属一类：**在「出事时靠它」的组件里，把失败变成沉默**。

### ✅ opener 的两个覆盖缺口（同批修掉）

| 缺口 | 逃过的形态 | 实测 |
|---|---|---|
| 只认 `let _ =` | `foo(..).ok();`（裸语句） | `full_cycle.rs` 10 处阶段推进错误 |
| 只认 `let _ =` | `let _killed: T = ..ok();`（**命名绑定**） | `nt_agent.rs` 8 处清理路径 |

⇒ 两次都按**证据**放宽（不加宽到任意语句），并各自留下实测数字。
⇒ `.ok()` 报告数 **57 → 76**，覆盖面扩大后**门立刻多抓到 1 处新命中**：

### ✅ 第 4 处真实缺陷：审计链三处静默失败叠加（`bin/nt_crystal_serve.rs:1275`）

```rust
if let Ok(mut file) = OpenOptions::new()..open(&path) {          // ① 打开失败 ⇒ 静默跳过
    let _written = writeln!(file, "{}", serde_json::to_string(&line)
        .unwrap_or_default());                                    // ③ 序列化失败 ⇒ 写空串
}                                                                 // ② 写入错误 ⇒ 丢弃
```

⇒ 审计链会**静默丢记录**，或写入一条**看起来有效但为空的损坏记录**。
⇒ 三者全部可见化（序列化失败 / 打开失败 / 写入失败各自 `warn`）。

### 分诊中的两次自我纠正（R-SCAN-1b 再次生效）

`checkpoint.rs` 5 处、`edit_history.rs` 7 处初判为缺陷 ⇒ **读代码发现全在
`#[cfg(test)]`**（前者是 roundtrip 测试，后者是临时文件清理，
且 `remove_file` 本就在门的 `AMBIGUOUS` 白名单）⇒ **都不是缺陷**。

⇒ 我的 grep 按**行内容**过滤、没看**模块归属**。
**裸 grep 的命中不构成证据** —— 本轮第 3、4 次靠读代码才站得住。

### ✅ P0.5-b 分诊工具化：`scripts/ops/nt_ok_audit.py`（2026-10-06）

人工 grep 分诊连错两次的根因是「按**行内容**过滤、没看**模块归属**」，
故把分诊**变成工具**：

- 先定位每个 `#[cfg(test)]` 起点，**括号配平**求真实结束行，**按行区间**排除
- 额外排除**整文件即测试模块**（`tests.rs` / `test_*.rs` / `*_test.rs`）——
  本仓存在这种形态，`#[cfg(test)]` 区间法**抓不到**
- 区分 **IO/写类（高风险）** 与普通，输出 `file:line` + 语句

⇒ 效果：`.ok()` 从「裸 grep 170 处（含测试）」收敛到
**生产 325 处 / 其中 IO-写类 25 处**。

### 分诊结论（25 处逐处读代码）

| 类别 | 结论 |
|---|---|
| `child.kill()/wait()` ×4、`handle.join()` | 清理路径，进程已退出时本就报错 ⇒ **正确** |
| `fs::read_to_string(..).ok()` 返回 `Option` | `.ok()` **就是**转换，调用方处理 `None` ⇒ **正确** |
| `skill_loader.rs` 的 `metadata().ok()` | 用作**可选缓存时间戳** ⇒ best-effort **正确** |
| `streaming/http.rs:273` `writer.flush().await.ok()` | **取消路径**上的 best-effort ⇒ **可接受** |
| `guardian.rs:549` `create_dir_all(..).ok()` | **真缺陷**（根因被吞）⇒ 已修 |

⇒ **高风险 25 处里，真缺陷只有 1 处**（本提交）。
⇒ 这也说明：把 `.ok()` 设为门禁会**误伤 24 处**，进一步印证
「语义不明确处用**报告项**而非门禁」是唯一站得住的做法。

**元教训（累计 3 次）**：扩大门覆盖面的动作，**本身**就会产出新缺陷，
但**分诊结论必须逐处读代码确认**，否则就是把噪声当信号。

⚠️ **这正是「门必须能被证伪」的价值**：一个只看「自己关心的形态」的门，
会以 PASS 的面貌放过真缺陷。**扩大覆盖面的那一刻，门才第一次真的在工作。**

⇒ 另 3 个（`production_logistics` / `finance_compliance` / `trade_product_spec`）
**顶层执行器数为 0** ⇒ 接线须**发明** schema ⇒ **不注册，保持 fail-closed**。

⛔ **我没有写那 5 个适配器**，尽管技术上可行（引擎都是 `Default` 可构造）。
理由：输入 schema 只能靠**发明**。而本轮已两次因「发明出的东西看着健康」
造成真实伤害（清单 id 一次、canary 形态一次）。
⇒ **在没有权威 schema 来源前，宁可保持 fail-closed 并把缺口写明。**

**待决**：这5 个能力的输入 schema 权威来源是
① 各模块既有 `capability_spec()` / `TradeCapabilitySpec` 是否已含schema；
② 需产品/协议侧给定。
⇒ 查证方向已写明，未执行。

### ⭐⭐ P0.1 **修正**：不是「id 空间分裂」，是**我连错了两层能力**（2026-10-06）

我先写「manifest id 必须能在 `TradeCapabilityRegistry` 查到」的测试，它红了，
我据此断定「canary 空间与实现空间分裂」，并把清单改成了 `trade.*`（`f67942ea`）。

**那个结论是错的**，`f67942ea` 已回退。核实链条：

| 证据 | 结果 |
|---|---|
| `git log -S 'NT-MEMORY::trade::trade_product_spec'` |首次出现于 **`7187e0b9`（2026-10-03，早于本轮）** |
| 该提交信息 | 「仓里有 **5 个** `register_xxx_capability(registry: &mut CapabilityTreeRegistry)`」 |
| 5 个实现模块逐个 grep | `nt_trade_product_spec` / `finance_compliance` / `full_cycle` / `production_logistics` / `quote_negotiation` **各自持有字面同串 id + 各自有注册调用** |

⇒ **那 5 个 `NT-*::trade::*` 是既有契约，且每个都有真实实现** —— 清单首版是对的。

⇒ **真正的结构是「两层能力」，不是「一套 id 分裂」**：

| 层 | id 形态 | 内容 | 数量 |
|---|---|---|---|
| **能力树**（`CapabilityTreeRegistry`） | `NT-域::模块::实例` | 外贸全链 / 生产物流 / 报价谈判 / 金融合规 / 产品规格 | **5** |
| **L1 内部**（`TradeCapabilityRegistry`） | `trade.x`（点号） | 价格计算 / 产品匹配 / 风控 / 供应商匹配 | **4** |

**它们是不同的能力，不是同一能力的两种 id。**
我拿 L1 内部那 4 个去核对能力树的 5 个 ⇒ 必然全不匹配 ⇒
我把这个「比错对象」当成了「空间分裂」。

**教训（本轮第六次「先提方案后核实」，也是最贵的一次）**：
**测试红了，第一反应应是「我比错了什么」，而不是「被测物有缺陷」。**
我改的是**正确的清单**，去迁就一个**问错了的测试**。
⇒ 凡写「X 必须等于 Y」的承重测试，**先确认 X 与 Y 属同一层**。

⇒ 已删除那个比对错误层的测试；派发端口的注册对象仍待定
（应对能力树，而非 `TradeCapabilityRegistry`）。

### ✅ T4.5-b「无全局落点」已解决，但**我的因果诊断是错的**（2026-10-06）

我原以为障碍是「注册表不可跨线程共享」，于是给 `Arc<dyn TradeCapability>`
补 `+ Send + Sync` 并写了「承重断言」。

**变异测试直接证伪了我**（这是本轮最值钱的一次变异）：

| 步骤 | 结果 |
|---|---|
| 去掉全部 6 处 `+ Send + Sync` | **build RC=0**（我预期失败） |
| 查父 trait | `UnifiedCapability: Send + Sync`（`nt_core_capability_types.rs:504`） |

⇒ **约束本就成立**，我补的是**冗余且装饰**的东西，还称之为「证据」。
⇒ 已回退约束、删除那个假断言。

**真正的前提只有两条**，第一条已修：
1. ✅ **async 签名**（`77e9688d`）
2. ✅ **全局落点**（`global_trade_registry()`，本提交）——
   `static` 本身就是 `Send + Sync` 的编译期证据，无需人工断言

⇒ **教训（本轮第五次「先提方案后核实」）：
编译器已保证的东西，不要补约束再称之为「证据」。**
断言必须经得起**去掉它就编译失败**的变异检验；我的没经得起。

**下一步（二选一，需实测后定）**：
- **A** `DispatchFn` 改为 `fn(&str, &Value) -> BoxFuture<Result<Value,String>>`
  ⇒ 需装箱future + 明确运行时归属（谁提供 executor）；
- **B** 在共享 crate 增设**全局注册表落点**，且执行入口改为**同步外壳**
  （如 `block_on`）⇒ ⚠️ 需先确认不会在 async 上下文中嵌套运行时。

⚠️ 本条是本轮**第四次**「先提方案后核实」的实例（前三：A/B/C 三个方案）。
**纪律写进文档 ≠ 内化** —— 我已把T4.1 第5 条改为可执行形式：
**凡提出「接口/端口/签名」类方案，必须先核实被调用方的真实签名与实例获取方式。**


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

## 附：治理元门本轮的四条实证（2026-10-06）

`check-gate-satisfiable.sh` 报「未登记 1 / 恒红 4」。逐条判读后：

| 项 | 判定 | 处理 |
|---|---|---|
| `check-dead-config-flag` 未登记 | **缺审**（非坏门） | 补**探针** + 登记 `gate-registry.tsv` ⇒ 未登记归 0 |
| `check-claims-numbers` 恒红 | **我造成的陈旧数字** | 我加了第 84 条索引却没同步 `AGENTS.md` ⇒ 已改（83→84、75→84）⇒ **PASS** |
| `check-doc-claims` 恒红 | **文档断言被代码反驳** | `FULL-AUDIT` 的 B4「`maybe_compact_context` 未接生产」**已不成立**：实测 `nt_loop_core.rs:25` 在 `AgentLoop::turn()` 主路径调用，且有生产调用方 ⇒ 已定正 |
| `check-unwrap` 24 new / `check-doc-drift` 125 | **存量债，非本会话引入** | 24 new 分布在 7 个文件，**本会话 30 个提交命中 0**；`unwrap-baseline.txt` 未被我改过 ⇒ **不碰** |

### ⭐ 探针首版失败，暴露「注入形态必须匹配门的真实契约」

`check-dead-config-flag` 的探针**第一次就 FAIL**（门不报我注入的东西）。

⇒ 根因：`_RE_FIELD_BOOL` 要求 **`pub`**（门只管「对外声称的配置面」），
   而我注入的是**私有**字段 `foo: bool`。
⇒ 改注入为 `pub` 后 **PASS**。

⚠️ 这与 `check-orphan-dirs` 探针里记的坑**同族**：
**选错注入形态会得到 rc=0 / 门不响的假阴性**，
而「门没响」很容易被读成「门是空的」⇒ 实为**注入无效**。

⇒ 元规则：**探针的注入形态必须先读门的正则/契约再写**，
   否则「探针通过」与「探针报错」都可能是自欺。

### 「恒红」的正确处置

`check-gate-satisfiable` 自己给出判据：
① 门真的坏了 → 修门；② 存量债已记账 → 搬进baseline，`--strict` 只挡新增；
⛔ 绝不要为了让门变绿而**调大 baseline**或**删掉 --strict**。

⇒ 本轮据此：**只修「我造成的」**（claims-numbers、doc-claims），
   **不动非本会话引入的存量债**（unwrap 24、doc-drift 125）。

## 附二：全域审计实测清单 + 决策建议（2026-10-06 收官）

### 13 个门的真实状态（实测，非记忆）

| 门 | RC | 判定 |
|---|---|---|
| `check-silent-failure` | 0 | 绿 |
| `check-dead-config-flag` | 0 | 绿 |
| `check-orphan-dirs` | 0 | 绿 |
| `check-doc-drift` | 0 | 绿（本会话修好格式碰撞后） |
| `check-doc-claims` | 0 | 绿 |
| `check-claims-numbers` | 0 | 绿 |
| `check-agent-config` | 0 | 绿 |
| `check-layer-deps` | 0 | 绿 |
| `map-check` | 0 | 绿 |
| `check-test-baseline` | 0 | 绿 |
| `check-gate-satisfiable` | 1 | 元门恒红 **1**（`check-unwrap` 存量债） |
| `check-unwrap` | 1 | **24 new 存量债**，分布 7 文件，本会话引入 **0** |
| `check-naming` | 1 | **1615 offender**，advisory，规约 vs 现实差极大 |

### ✅ 已闭合（本会话）

- **自我进化闭环端到端实证**（新增 `e2e_evolution_tests`）：
  `SelfIteratingBrain::iterate(TaskType)` → **写** `evaluation_history`
  → `GoalContractStage::process` → **读**并构造 `RewardLedger`
  → `decide_autonomy` → **回写** `brain.autonomy`
  ⇒ 测试断言「history 增长 + autonomy 被改写」。
  ⛔ 刻意**不断言具体 autonomy 数值**（那会把「链路通」与「阈值好」绑死）。
  ⚠️ 实测两个易错点：`TaskType` 住`l2_perception`（同层无 re-export）、
  且**无 `Default`** ⇒ 不能 `TaskType::default()`。

### ⛔ 阻塞 / 需外部裁决（本会话**不能**完成）

| # | 项 | 性质 |
|---|---|---|
| 1 | 3 个贸易能力无顶层执行器（`production_logistics`/`finance_compliance`/`trade_product_spec`） | **业务逻辑缺口**：需权威输入schema，发明即造假 |
| 2 | `foreign_trade_full_cycle` 是阶段脚手架（17 阶段逻辑全是注释） | 同上；已在代码头/市场描述/台账三处标注 |
| 3 | `check-unwrap` 24 new | **存量债**（本会话引入 0）⇒ 按纪律不碰 |
| 4 | `check-naming` 1615 offender | **规约无约束力**（advisory）⇒ 需先定切片策略 |
| 5 | `.neotrix/LICENSE-EXCEPTIONS.md` `status: void` | **需所有者签署** |
| 6 | `.project-map/` 354M 体积策略 | **需体积决策** |

### 决策建议（按 价值/风险 排序）

1. **先立「执行器登记制」**：任何能力上架前，必须同时有
   （a）可调用入口　（b）输入 schema 来源　（c）至少一个**非桩**证据测试。
   ⇒ 本会话 4 处缺陷全部源于「上架了但不可用/是桩」，此制可一次性根治该类。
2. **`check-naming` 暂不设为门**：1615 的规约 vs 现实差太大，
   先按层抽 1 层做切片，验证可行再谈门禁。
3. **`check-unwrap` 存量债单独立项**：按文件分批修，
   每批必须**变异验证**（注入一个 unwrap 看门是否报），避免「批量改完门不动」。
4. **外部吸收一律走「证据三件套」**：许可证核实 + 生产接线 + 变异/回归测试；
   ⛔ 不接受「调研完成」当作吸收完成（R-P79）。


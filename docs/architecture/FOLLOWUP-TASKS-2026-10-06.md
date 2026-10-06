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

### 前置：必须先消除一个同名冲突（实测发现）

存在**两个 `CapabilityNode`**：

| 位置 | 形态 |
|---|---|
| `crates/nt-core-capability-tree/src/node.rs:328` | **结构体**（有 `new_primitive` / `new_composite` / `new_constellation`） |
| `neotrix-core/src/l0_substrate/nt_core_traits.rs:27` | **同名 trait** |

⇒ 又是 L15「同名 ≠ 同一符号」。**先统一这两个，再谈把元数据下沉** ——
否则会把描述性字段加到错误的符号上。

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
# 能力 / 意识涌现 · 进化路线

> 判据来源：用户 2026-10-02 给的「**涌现 = 像人一样创造新的事物**」。
> 本文把那句话**落成可验证的判据**，并给出落点。
> ⛔ **本文不含任何未验证的路径** —— 每个代码节点都经 `ls` / `rg` 核实；
> 凡是「需属主裁决」的都单列，不混进实施项。

## 0. ⭐ 本文的诚实边界

- ✅ 下面 §2 的三条事实**全部实测**，给 `文件:行`。
- ⛔ §5 的路线**没有排期**，因为我无法估计「无人工干预」该做到什么程度才算达标 ——
  那是产品决策（见 §6 待裁决第 1 条）。
- ⛔ 本文**不**主张本仓已有涌现。⭐ 实测结论恰恰相反，见 §3。

---

## 1. 把「像人一样创造新事物」翻译成可验证判据

人「创造新事物」时做四件事，每件都**可留证**：

| 人的行为 | 机器上的对应 | 可验证性 |
|---|---|---|
| 识别自己做不出来的事 | 发现能力缺口 | ✅ 可枚举 |
| 造出新工具/新方法 | **新增能力节点** | ✅ `registry.register()` 已存在 |
| 声明它有多强 | 节点 maturity 声称 | ✅ 节点字段 |
| 拿出证据 | 证据支撑 | ✅⭐ **`maturity_audit()` 已经强制** |

⇒ **判据（本文提出）**：

> **涌现 = 能力树在无人工干预下新增了节点，且该节点通过 `audit-maturity --strict`。**

⭐ 这个判据的可验证性来自一个**已经存在**的机制：`maturity_audit()`
（`crates/nt-core-capability-tree/src/cli.rs:945` 调用）
拒绝「声称成熟度超过证据支撑」的节点。
⇒ 我们**不需要**新造度量体系，只需要让运行时真的去注册节点。

⚠️ **为什么不加「数量增长」作为判据**：那会奖励刷节点。
`maturity_audit` 的反虚标设计说明本仓已经识别过这类风险。

---

## 2. 三条实测事实（本轮新查证）

| # | 事实 | 证据 |
|---|---|---|
F1 | ⭐ **能力树可新增节点** | `registry.rs:168 pub fn register(&mut self, node: CapabilityNode) -> Result<(), RegistryError>` |
F2 | ⭐ **成熟度虚标已被门拦** | `cli.rs:945 registry.maturity_audit()`；CI 在 `ci.yml:368-377` 跑 `audit-maturity --strict` |
F3 | ⭐⭐ **但运行时不消费它** | 全仓 `nt-core-capability-tree` 只出现在：自身 crate 文件、`Cargo.toml`、`ARCHITECTURE-MAP-ROADMAP-V2.md`、`scripts/check-doc-claims.sh`、`scripts/dup-types-baseline.txt`、`TODO.yml`、handoff 文档。⛔ `neotrix-core/src` 与 `crates/neotrix-neobot/src` **零引用** |

### 2.1 ⛔ 勘误（2026-10-03，接线时实测发现）

⛔ **commit `ee2cc276` 的 message 里有一句假陈述**：「`nt-core-capability-tree`
**无任何 crate 依赖它**」。

**真相**：`neotrix-core/Cargo.toml:103` **早就有** `nt_core_capability_tree = { workspace = true }`。

**根因**：该句是用 `grep 'nt-core-capability-tree'`（**连字符**）得出 0 命中，
而 Cargo 依赖项里写的是 `nt_core_capability_tree`（**下划线**）
⇒ ⛔ 我把「一次拼错分隔符的 grep 没命中」升级成了「架构结论」。

⭐ 这正是 `AGENTS.md` **R-SCAN-1b** 的形态，我自己踩了：**裸 grep 的零命中不构成证据**。

**逐条复核后，其余结论全部成立**：

| 陈述 | 判定 | 依据 |
|---|---|---|
| `crates/neotrix-neobot` 之前**没有**该依赖 | ✅ 真 | `git show ee2cc276~1:…/Cargo.toml` = 0 命中 ⇒ 加依赖**必要非冗余** |
| `neotrix-core/src` **零源码引用** | ✅ 真 | `rg -c 'nt_core_capability_tree\|CapabilityTreeRegistry' neotrix-core/src` = 0（28 MB 全扫）⇒ **F3 缺口成立** |
| 本表 F3 行 | ✅ **写得对** | 它列了 `Cargo.toml`，且断言限定在 **源码零引用**，未越界 |

⭐ **教训沉淀**：Cargo 包名在 `[[package]] name` 里是**连字符**、在依赖键里是**下划线**
⇒ 查依赖必须**两种分隔符都查**，或直接 `grep workspace = true` / 查 `[workspace.dependencies]`。

---

## 3. ⭐ 现状判定：不是「还没做」，是「**做了一半且那半没接线**」

- ✅ 有节点模型、有 registry、有依赖边（`add_dependency`）、有成熟度模型、
  有反虚标审计、有 CLI、有 CI 门。
- ⛔ **没有任何运行时消费者**（F3）。

⭐ **准确的差距不是「缺一个涌现引擎」，而是「已建好的能力树是一套被测量、却不被使用与生长的对象」。**

⇒ 对标差距（用户关心的「补齐与对标产品的差距」在这一层的具体形态）：

| 维度 | 本仓 | 差距 |
|---|---|---|
节点能否新增 | ✅ 能（`register`） | — |
成熟度是否防虚标 | ✅ 有门 | — |
**谁在运行时新增节点** | ⛔ **无** | ⭐ **这是唯一的结构性缺口** |
节点是否被实际调度/路由 | ⛔ 无消费者 | 同上 |

---

## 4. 意识侧现状（同样只列实测）

真实模块（8 个目录，`neotrix-core/src/l5_cognition/` 下）：
`consciousness_core` · `nt_consciousness` · `nt_mind_background_loop/handlers_consciousness` ·
`nt_mind/nt_mind/consciousness` · `nt_game/consciousness` ·
`nt_core_consciousness_tree` · `nt_core_consciousness` · `nt_core/nt_consciousness_core`

⭐ **存活判定（区分「导出」与「调用」，因本仓已因此误判 3 次）**：
`nt_core_consciousness_tree` **有生产消费者** ——
`neotrix-core/src/l4_emotion/nt_feel_facade.rs`、
`neotrix-core/src/l1_action/nt_action_facade.rs`。

⭐ **意识 ⇄ 能力的连接点确实存在**（同一文件同时提到两者）：
`l4_emotion/nt_memory/nt_memory_kb/nt_memory_seed.rs` · `nt_absorb_mapper.rs`。

⇒ **意识侧不是「无」，而是「有消费者但产出未被记入能力树」** ——
这正是 §5 第 1 步的接线点。

---

## 5. 路线（三步，每步都可独立验收）

### 第 1 步 ⭐ 接线：让**已存在的**意识消费者把节点写进能力树
- 落点：`neotrix-core/src/l4_emotion/nt_feel_facade.rs`（已有消费者，改动面最小）
- 做什么：新增一个**运行期 registry**（进程内），在意识消费者判定出「新能力已就绪」时
  `register()` 一个节点，maturity 按 `maturity_audit()` 认可的档位填。
- ⭐ 验收：`audit-maturity --strict` 仍 rc=0，**且** CI 的 `capability-truth` job
  打印的节点数**比接线前多** ⇒ 证明节点确实被新增。
- ⛔ 不做的事：不自造 maturity 词汇表（复用现有）；不改 `maturity_audit` 判据。

### 第 2 步 ⭐ 可验证判据落成门
- 新增门 `neobot-check-emergence`：断言
  1. 运行期 registry 存在**且**有生产消费者（防「建了没人用」——这是本仓最常见的病）；
  2. 每个 `register()` 调用的 maturity 声称都能被 `maturity_audit()` 接受；
  3. ⭐ **反向护栏**：节点数**不增长**即判红 —— 钉住「涌现」必须是行为，不是静态资产。
- ⭐ 为什么第 3 条重要：否则「接线完成」会退化成「一次性注册固定几个节点」。

### 第 3 步 ⭐ 缺口驱动：新增节点必须由**观察到缺口**触发
- 落点：`nt_memory/nt_memory_kb/nt_absorb_mapper.rs`（意识⇄能力的既有连接点）
- 做什么：把「知识图谱里出现无法归类的簇」映射为能力缺口 ⇒ 新节点。
- ⭐ 这才是「像人一样创造新事物」的机器形态：**先发现自己不会，再造工具**。

---

## 6. 待裁决（⛔ 我不单方面决定）

1. ⭐ **「涌现达标」的阈值是什么？** 本文给了判据形式（新增节点 + 过审计），
   但「一个项目周期内应新增多少」「允许人工提供多少引导」没有答案。
2. ⭐ **`nt_core_capability_tree` 的运行时 registry 放哪？**
   候选：`neotrix-core/src/l1_action/nt_action_facade.rs`（l1，会被 L2–L6 依赖）
   vs `crates/neotrix-neobot/`（独立 crate，不占 L 层，⛔ 但那是 app 层）。
   ⭐ 这是**层归属决策**，而 `check-layer-deps` 会管 L 层 ⇒ 选错会被门拦。
3. ⭐ **意识消费者的「能力就绪」判据由谁定义？**
   本仓有 `converge` / `schema_gaps`（`nt_audit.rs:231` 实证）⇒ 是否复用它们作为缺口信号。

## 7. ⏳ 本轮**未**做（明确列出，避免被当成已完成）

- ⛔ 未实现第 1 步（等 §6 第 2 条裁决）
- ⛔ 未写 `neobot-check-emergence` 门（等第 1 步落地，否则门只能钉「不存在的东西」）
- ⛔ 未定阈值（§6 第 1 条）

---

## 附录 A（2026-10-03 晚补）：⭐⭐ 运行时能力树为空的**根因已定位**

§1 的判据是「涌现 = 能力树**新增节点**」。本轮把该判据**第一次**接进运行期
（`ee2cc276` 注册表 + `ae3b3644` 意识侧生产 + `dc5b3e41` 门）之后，
⭐⭐ **发现了更根本的一层**：

### A.1 能力树在启动时是**空的**——因为 8 个节点生产者**零生产调用方**

实测（`scripts/check-ext-wiring.py` 基线，209 个注册 API 中 50 个零调用）
筛出这 8 个**能力节点生产者**，它们的签名一致：

```rust
pub fn register_xxx_capability(registry: &mut CapabilityTreeRegistry) -> CapabilityNode
```

⛔ 而 `grep` 全工作区：**零个生产调用方** ⇒ ⭐ **启动时没有任何代码往能力树里放节点。**
⇒ §1 的判据虽然通了（`consciousness::gap::q*` 会新增节点），
但那些节点是**意识自己长出来的缺口**，**不是系统真实能力的目录**。

### A.2 ⭐ 依赖链已解出 ⇒ 接线是**确定的拓扑序**，不是猜

实测各节点的 `requires`（`CapabilityNode::new_*` 第二/三参数）：

```
  trade_product_spec        requires=[]                                        ← 根
        ↓
  trade_quote_negotiation   requires=[trade_product_spec]
        ↓
  ├─ trade_production_logistics  requires=[trade_product_spec, trade_quote_negotiation]
  └─ trade_finance_compliance    requires=[trade_product_spec, trade_quote_negotiation]
        ↓
  foreign_trade_full_cycle  requires=[trade_quote_negotiation,
                                      trade_production_logistics,
                                      trade_finance_compliance]
```

⭐ **这是一个 4 层 DAG，拓扑序唯一**（`product_spec` 必须最先，
`full_cycle` 必须最后）⇒ 接线可以写成一段**确定性启动序列**，
⛔ 而不是「挨个试着调」。

### A.3 其余 4 个（同族但形态不同，需单独定性）

| 函数 | 状态 |
|---|---|
| `register_social_access_capability` | ⛔ 未匹配到 `CapabilityNode::new_*` ⇒ **可能不是同一形态**，需先定性 |
| `register_ocr_capability` | ⛔ 同上 |
| `seo.rs::register_capability` | ✅ `id=nt_act::seo::analyze`，`requires=[seo.visibility]` |
| `excel_capability.rs::register_nodes` | ✅ `id=nt_file_ability::excel::xlsx_read`，`requires=[]` |

### A.4 ⭐ 因此下一步**不是**再设计，而是「按拓扑序接线 + 让门盯住」

1. ⭐ 在**启动路径**（`ConsciousnessRuntime` 装配处或某个明确的 boot 序列）
   按 A.2 的拓扑序调用这 4 个 trade 生产者 ⇒ ⭐ **能力树第一次被真实能力填满**。
2. ⭐ 注册**必须**经 `nt_capability_registry`（而非各自 `&mut` 局部 registry），
   否则它们填的是**临时树**，运行期查询仍为空。
3. ⭐ 加门：断言「启动后能力树节点数 ≥ 预期下限」⇒ ⭐ 与 `neobot-check-emergence`
   的 `COLD_START` 三态配合（接线到位后 `COLD_START` 应转为 `PASS`）。
4. ⛔ **不得**用「直接把 8 个函数塞进某个 mod 就完事」的方式 ——
   `register()` 对同 id 返回 `AlreadyExists`（实测 `registry.rs:169-171`），
   拓扑序错了会**静默失败**（幂等成功）⇒ ⭐ 必须**按序**且**有断言**。

### A.5 与 §5 的关系

§5 第 1 步（运行期注册表）✅ `ee2cc276`
§5 第 2 步（意识侧生产）✅ `ae3b3644`
§5 第 3 步（涌现门，三态）✅ `dc5b3e41`
⭐ **新增第 4 步：把「系统真实能力」灌进树**（本附录）。
⇒ 在这一步完成前，涌现判据只在**意识自生缺口**这一条窄路径上成立。

# `_superseded/` — 已归档的架构文档

> 归档不是删除。本目录的文件**内容仍然有效或含独有溯源**，只是不再代表当前架构。
> 活跃文档在 `docs/architecture/`；`DOCUMENTATION-MAP.md` 是文档规范正典。

---

## 2026-09-29 归档：`neotrix-core/docs/architecture/` 三份

`DOCUMENTATION-MAP.md:81` 明令「`neotrix-core/docs/` 禁止存研究笔记、分析报告」，
而那里有 3 份 **744 行**的架构文档。逐份取证后裁决 —— **不是「已被取代」一刀切，
而是三种不同处置**：

### `FUSION_ARCHITECTURE-2026-09.md`（436 行，分析日期 09-11）→ 归档

**部分被取代，但含独有溯源。**

- 被 `design-fusion-analysis-2026-09-20.md` 覆盖：13 个章节一一对应且后者更全
  （外部技术提取 / 能力架构诊断 / 融合方案 / 多Agent巡检 / 投入产出 / 关键决策），
  且多出 UI-Layouts、图标生态、设计系统等新维度
- ⛔ **但以下内容全仓别处没有**（`git grep` 逐项确认）：
  - `Bluehook`（小钻风破甲）的破甲分析 —— 外部威胁模型来源
  - 「43 种攻击 + 15 种防御」全景
  - `Fable Dataset` 分析
- ⇒ 这些外部研究溯源删了就永久丢失，**故归档不删**

### `ANALYSIS_ARCHITECTURE-2026-09.md`（178 行，09-14）→ 归档

**方法论仍有效，实测数据已作废。**

- 提出的三个分析维度「聚焦冗余 / 扁平缺陷 / 跨域错位」**已被现行文档广泛沿用**
  （`design-fusion-analysis-2026-09-20.md` / `BATCH-FIX-CHECKLIST-2026-09-28.md` /
  `capability-topology-map.md`）⇒ 方法论不是孤例，**有继承**
- ⛔ 但其代码度量**严重过时**：文中写 `931行`，实际 `neotrix-core` 已是
  **789,902 行**（`find neotrix-core/src -name '*.rs' | xargs cat | wc -l` 实测）
  ⇒ 拿它做规模判断会得到错误结论

### `CRYSTAL_ARCHITECTURE-2026-09.md`（130 行）→ 归档

**描述的架构完全不存在。**

其架构图与模块清单列出的 8 个类型，逐个 `rg '(struct|enum|trait|fn|mod) X'` 实测：

| 类型 | 现状 |
|---|---|
| `MeltingEngine` | ⛔ 已不存在 |
| `CrystalRouter` | ⛔ 已不存在 |
| `CrystalSpeculator` | ⛔ 已不存在 |
| `EcsSceneBridge` | ⛔ 已不存在 |
| `CrystalCard` | ⛔ 已不存在 |
| `CrystalStateMachine` | ⛔ 已不存在 |
| `CrystalWorld` | ⛔ 已不存在 |
| `CrystalError` | ⛔ 已不存在 |

**8/8 全部不存在** ⇒ 它描述的是一个从未落地或已被彻底重构掉的架构。
⇒ 内容对当前零参考价值，**但仍归档而非删除** —— 因为「曾经这样设计过、后来变了」
本身是有价值的考古信息，且删除不可逆。

---

## 2026-09-29 归档：`docs/2-PLANS/` 两份路线图

`DOCUMENTATION-MAP.md` §2 规定 `2-PLANS/ └── ROADMAP-*.md` 存放路线图。实际只有 2 份，
且**均为 2026-09-20 的架构审计快照**（同日、同一审计族：一份 2,383 files/789K LOC，
一份 2,159 files/~700K LOC）⇒ 已 `git mv` 至本目录的 `2-PLANS/` 子目录。
**零删除。** `docs/2-PLANS/` 留 `README.md` 占位（否则该目录随内容消失，
`DOCUMENTATION-MAP.md:72` 的目录树与 `check-doc-drift.sh:153` 的 glob 示例会悬空）。

**取代者**：`docs/architecture/FINAL-ROADMAP-2026-09-29.md` —— §0 自称
「排期与状态以本文件为准」，`AGENTS.md` §6 称其为「唯一排期真源」。

### `ROADMAP-ARCHITECTURE-FUSION-2026-09-20.md`（330 行，中文）→ 归档

**度量全失效，且核心任务项已被后续裁决实证否决。**

- 「现状诊断」7 项**逐条实测失效** —— 其中层级失衡方向完全未预见：
  L4 文件数 **19 → 255**（本文 Sprint 3 目标写 "~80"）
- ⛔ 本文的 **T1.2「合并 14 个 `TaskStatus` 版本」已被否决**：
  `docs/architecture/FIVE-ENTITY-TASK-CHECKLIST.md` T42 记录同一组计数
  （`TaskType×14/TaskStatus×14/Severity×13/RiskLevel×12`）并裁决
  「Top18 全 LEGIT/DEFER，**零合并**」—— 同名多为正当领域分离，
  批量合并会破坏字段/变体/引用兼容性 ⇒ **本文的这条 P0 动作若执行会造成真实回归**
- 本文 T4.3 提议编写的 `scripts/check_architecture.sh` **至今不存在**；
  实际职责由 `check-layer-deps.sh` / `check-doc-drift.sh` 等账本棘轮门承担

### `ROADMAP-FUSION-ULTIMATE-2026-09-20.md`（213 行，英文）→ 归档

**度量全失效，待办身份已消失。**

- §2「Layer Size」7 行**全部失效**（2026-09-29 实测）：
  L0 43→48 · L1 646→505 · L2 335→361 · L3 253→247 · **L4 19→255** ·
  L5 656→701 · L6 207→234
- §5「成功指标」Current 列同样作废（TODO/FIXME 260 → 实测 192）
- P0 吸收目标 8 项**已有 6 项落地**（`find` 实测存在 `nt_judgment` / `nt_harness` /
  `nt_web_perception` / `nt_routing` / `nt_council` / `nt_security`）
  ⇒ 其「待办」身份消失；余 2 项（`nt_ability_net` / `nt_agent/ode`）
  **全仓 `rg` 零命中**，是未落地条目而非现行排期

### 保留理由（归档不删）

- `ROADMAP-ARCHITECTURE-FUSION` 的**「重复类型 Top 10」逐项取证表** ——
  `DIR-AUDIT-2026-09-27.md` 只记了 8 类家族，粒度更粗，Top10 计数别处没有
- `ROADMAP-FUSION-ULTIMATE` 的 **35 仓吸收矩阵**（star 数 + 目标模块 + 层归属）——
  `docs/architecture/absorption-sources/` 是 483 仓 CSV 排名/许可台账，
  不含这个「仓 → 目标模块」的映射

⚠️ 两份的 **star 数 / 文件数 / LOC 全部是 2026-09-20 快照**，
与本目录 `ANALYSIS_ARCHITECTURE-2026-09.md` 同病：**方法论可抄，实测数据已作废**。

### 与本目录命名惯例的一处偏离（已知）

本目录此前是**平铺**结构，命名 `{原名}-2026-09.md`（后缀 = 文档自身声明的分析日期）。
本次为 2-PLANS 单独开子目录 `2-PLANS/`，**保留原文件名不动**
（原名已含 `-2026-09-20` 自述日期，再套惯例会变成 `...-2026-09-20-2026-09.md`）。
文件名保留 ⇒ 可用 `git log --follow` 追溯。

---

## 命名

`{原名}-2026-09.md` —— 后缀是**文档自身声明的分析日期**，不是归档日期。
（沿用同目录 `EVOLUTION-ROADMAP-CODE-NODES-2026-09-27.md` 的惯例。）

## 剩余内容

`neotrix-core/docs/plans/` 下仍有 2 份 09-14 的设计稿
（`DESIGN_CRYSTAL_CORE.md` / `DESIGN_COLIBRI_ABSORPTION.md`）——
它们**符合** `YYYY-MM-DD_` 前缀，故 `check-layout.sh` 不判违规。

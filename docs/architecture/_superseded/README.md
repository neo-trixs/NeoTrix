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

## 命名

`{原名}-2026-09.md` —— 后缀是**文档自身声明的分析日期**，不是归档日期。
（沿用同目录 `EVOLUTION-ROADMAP-CODE-NODES-2026-09-27.md` 的惯例。）

## 剩余内容

`neotrix-core/docs/plans/` 下仍有 2 份 09-14 的设计稿
（`DESIGN_CRYSTAL_CORE.md` / `DESIGN_COLIBRI_ABSORPTION.md`）——
它们**符合** `YYYY-MM-DD_` 前缀，故 `check-layout.sh` 不判违规。

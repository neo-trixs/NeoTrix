# `2-PLANS/` — 路线图（当前为空）

> 归入 `DOCUMENTATION-MAP.md` §2 的 `2-PLANS/  # 路线图 └── ROADMAP-*.md`。
> **本 README 是占位，不是路线图。** 目录保留而不删除，因为
> `DOCUMENTATION-MAP.md:72` 的目录树与 `scripts/check-doc-drift.sh:153`
> 的 glob 排除示例都指向这个路径；目录若随内容一起消失，两处会变成悬空引用。

## 2026-09-29 归档：本目录原有 2 份路线图

两份均为 **2026-09-20 的架构审计快照**，同日、同一审计族（一份 2,383 files / 789K LOC，
一份 2,159 files / ~700K LOC），已全部移入
`docs/architecture/_superseded/2-PLANS/`（`git mv`，零删除）：

| 原文件 | 归档后位置 | 归档判据（2026-09-29 实测） |
|---|---|---|
| `ROADMAP-ARCHITECTURE-FUSION-2026-09-20.md` | `_superseded/2-PLANS/` | 「现状诊断」7 项度量全失效（L4 19→255）；T1.2「合并 14 个 `TaskStatus`」已被 `FIVE-ENTITY-TASK-CHECKLIST.md` T42 实证否决（零合并）；T4.3 提议的 `check_architecture.sh` 至今不存在 |
| `ROADMAP-FUSION-ULTIMATE-2026-09-20.md` | `_superseded/2-PLANS/` | §2「Layer Size」7 行全失效；§5 成功指标 Current 列作废；P0 目标 8 项中 6 项已落地（`nt_judgment`/`nt_harness`/`nt_web_perception`/`nt_routing`/`nt_council`/`nt_security` 实测存在），「待办」身份消失 |

**取代者**：排期正典 = `docs/architecture/FINAL-ROADMAP-2026-09-29.md`
（§0 自称「排期与状态以本文件为准」；`AGENTS.md` §6 称其为「唯一排期真源」）。

**为什么归档而不删除**（沿用 `_superseded/README.md` 的先例）：
`ROADMAP-ARCHITECTURE-FUSION` 的「重复类型 Top 10」逐项取证表、
`ROADMAP-FUSION-ULTIMATE` 的 35 仓吸收矩阵（star 数 + 目标模块 + 层归属），
都是 `DIR-AUDIT-2026-09-27.md`（8 类家族）与 `absorption-sources/`（483 仓 CSV）之外的独有溯源。
**方法论可抄，实测数字已作废** —— 引用其数字前必须重新实测。

## 放在本目录的门槛

新路线图入本目录前须回答：**它比 `FINAL-ROADMAP-2026-09-29.md` 多了什么排期增量？**
答不出来就不该进来 —— 否则本目录会重新退化成快照堆积场。

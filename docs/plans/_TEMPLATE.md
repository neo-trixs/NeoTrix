# 变更工件模板（artifact template）

> **吸收源（思想，非代码）**：Anthropic《The AI-native SDLC playbook》(2026-08-21) 六阶段
> Plan→Design→Build→Test→Deploy→Maintain 的「**committed artifact chain**」论点。
> 完整原文事实源：`docs/architecture/ABSORPTION-AI-NATIVE-SDLC-2026-10-06.md`。
>
> **正典分工**：本文件只讲「怎么写一个工件」。规矩在 `AGENTS.md`，不在这里重复
> （指针守恒）。谁该被改动、门怎么跑 → `AGENTS.md` §0 决策树。

---

## 0. 为什么有这份东西

那条 playbook 的核心论点是一句可判定的话：**代码不再是瓶颈，人速的环节才是**——
plan / review / deploy 三段仍按人速走。于是它的解法不是「加人」，而是把每一步的产物
**提交进版本控制**，让下一个阶段读同一个文件；这条提交链本身就是审计轨迹。

**本仓的落地缺口**（2026-10-06 实测，非推理）：

| 判据 | 实测 |
|---|---|
| `intent.md` 全仓命中 | **0** |
| `proto-spec` 全仓命中 | **0** |
| 每变更一个「施工前」工件 | **无** |
| 已有 `docs/plans/` | **66 件**，但全是**主题级**方案，非每变更工件 |
| 已有 `sessions/handoff-*.md` | **70 件**，**回顾性**（§3 已完成 / §4 正在改的文件） |

⇒ 回顾性交接件**不能**替代施工前工件：交接件在**代码写完之后**才产生，
review 时拿不到「原本打算改哪些文件」这个基线，只能对着 diff 空审。

---

## 1. 一件工件，三段状态

playbook 用 `intent.md` → `spec.md` → `plan.md` 三件。**本仓不照抄三个文件**——
那会在 `docs/plans/` 旁边再造第二套真源，与指针守恒冲突。改为**一件工件、三段状态**：

```
status: intent  →  status: spec  →  status: plan  →  status: done
   ↑                   ↑                ↑
 谁的问题/为什么      需求+设计        改哪些文件/什么顺序/风险/证据
 （发起者原话）      （可施工）        （可被 diff 反查）
```

**状态就是阶段标记**，机器可判 ⇒ `scripts/check-artifact-chain.sh` 靠它分档。

---

## 2. 模板（复制下面这段改名即可）

文件名：`docs/plans/YYYY-MM-DD-<topic>.md`（与既有 66 件同约定）

```markdown
# <一句话标题>

status: intent
date: YYYY-MM-DD
owner: <发起者>

## 问题
<发起者原话描述的现象。不要写解决方案。>

## 目标
<更好是什么样。一句话。>

## 影响面
<受影响的用户 / 系统 / 层。列出路径。>

## 约束
<硬约束。例：零 unsafe、不加 PII、零全量重建。>

## 未决问题
<还没定的。工程看到之前必须回答的，用 ⛔ 标出。>

<!-- ===== 以下在 status 推进到 spec / plan 时才填 ===== -->

## 需求与设计
<怎么解。与本仓既有公理（层归属 / 零 unsafe / 指针守恒）对齐的方式。>

## 改动文件
<**逐个列路径**。这是工件与 diff 的唯一连接点 ——
 check-artifact-chain.sh 拿它反查实际改动。>

## 施工顺序
1. <第一步>
2. <第二步>

## 风险
<最可能坏掉的一环 + 兜底。>

## 证据
<可量化的完工判据。例：`cargo test -p neotrix --lib <filter>` 全绿；
 <门> rc=0。写不出可量化判据 = 还没想清楚。>
```

---

## 3. 六节为什么是这六节

每一节都对应 playbook 里一条**可判定的控制目标**，不是排版偏好：

| 节 | playbook 出处 | 少了会怎样 |
|---|---|---|
| 问题 | Plan 阶段 `## Problem` | 改完无法反查「是不是解决了那个问题」 |
| 目标 | `## Proposed outcome` | 无法判定 done |
| 影响面 | `## Affected users and systems` | 漏改调用方（R-P79 意义上的「导出 ≠ 接入」） |
| 约束 | `## Constraints` | 事后争论「当时为什么这么定」 |
| 未决问题 | `## Open questions` | 未决问题被静默吞掉，等于没记 |
| **改动文件** | Build 阶段 `## Files that change` | **review 无基线**——这是最贵的一节 |
| 施工顺序 | `## Order of work` | 一次过改 8 个文件，回归不可定位 |
| 风险 | `## Risks` | 最坏情况无人预判 |
| 证据 | `## Proof` | 「done」由 agent 自称 ⇒ 违反 `evals/VERIFICATION.md:10` |

---

## 4. 铁律：偏离计划要同 commit 改工件

playbook Build 阶段第 7 步：*When implementation departs from the plan,
update `plan.md` in the same commit.*

**本仓表述**：`status: plan` 之后，实际改动超出 `## 改动文件` 清单 ⇒
要么补清单，要么改代码。**二选一，不许既不改清单也不改代码**——
那种 diff 的真实意图已经不可判，review 只能降级成逐行空审。

本仓已有同型纪律可抄：`.githooks/prepare-commit-msg` 的 `DELETION-INTENT`
（提交含删除必须声明意图）。**声明与改动同 commit** 是同一族机制。

---

## 5. 门：`scripts/check-agent-config.sh` 与 `scripts/check-artifact-chain.sh`

- 本件**不自我豁免**：门在 `scripts/gate-registry.tsv` 有登记，
  在 `.github/workflows/ci.yml` 有跑点（写下来 ≠ 守得住）。
- 门是**棘轮**不是**普查**：既有 66 件工件一节都不全，若普查即「一出生就恒红」
  —— 恒红的门等于没有门（本仓门纪律）。故只对**基线之后新增**的工件判红。

## 6. 一句话判据

> **提交链就是审计轨迹**：谁要了什么、agent 产出了什么、谁批准了什么。
> 没有工件的那段提交，只有一行 commit message —— 那不是审计轨迹，是留言。
# 吸收：x_gift_bot / Compositor / Agent Orchestrator（2026-10-03晚间）

> ## ⚠️ 分栏声明（2026-10-04 补做）
> **【源】**源里原文 · **【实测】**我查过的带 `file:line` · **【推】**我的推断，**可能错**

> 本篇只记**判据与可移植设计**，未取任何代码。
> ⚠️ 两个URL **已收到但未核实**，故**未入库**：`casp.ac/reports/intelligence-explosion`、
> `markfulton/agent-cookie-sync` ⇒ 未填许可字段（不猜）。

| 源 | ★ | commits | 语言 | 许可 |
|---|---|---|---|---|
| `mizorewww/x_gift_bot` | 102 | 50 | Go + TS | **MIT** |
| `robbietilton/Compositor` | 7.2k | 343 | Swift（macOS） | **MIT** |
| `Untrivial-ai/agent-orchestrator` | 12.6k | 3,056 | Go + TS | ⛔ **Apache-2.0**（非 MIT，copyleft 弱但有专利条款） |

---

## 一、`agent-orchestrator`（AO）—— 本轮对我们最相关

### 1.1 ⭐⭐ **文档优先级被写成「契约」，而不是靠自觉**
`docs/documentation-map.md` 的一句话是本篇最有价值的东西：
> which docs are human-facing, which are **machine-readable contracts**,
> and **which wins on drift**

⇒ **漂移时谁赢**被写成文档，而不是留给记忆。

### 1.2 ⭐ 这条**通过了**我自己的前置条件门
`ABSORPTION-PRECONDITION-GATE` 要求三问：

| 前置 | 本例 |
|---|---|
|① **危险面存在吗** | ✅ **存在，且本会话已实证 4 次**：AGENTS.md 索引「19 条」实为 75；我说 browse面板「缺底边」而它有；`EMERGENCE` 文档写「刻意不改 5 处expect」而我已改；`sandbox.rs` 我说未守卫而守卫在上方 2 行 |
| ② 路径可达吗 | ✅ `docs/architecture/` 是本仓真相面载体 |
| ③ 是我方的吗 | ✅ 本仓文件 |

⇒ 四次「陈旧断言」事件 ⇒ **危险面已证实**，不是推测。
⇒ ⛔ 但**本轮仍不落代码/门脚本**：门要能判「哪份文档赢」，
   需先有**可枚举的文档类别**；凭空写会产出第二个误报门
   （`check-term-viz` 336 误报的前车之鉴）。

### 1.3 另外三条可移植设计
· **每个 worker 独立 branch + worktree**；standalone agent 用 AO 自管的无branch 目录
  ⇒ 「有仓库给 worktree、无仓库给托管目录」是**二分而非二选一**。
· **Kanban 状态由事实派生**（session / PR / CI / review），
  不是人工标记：Working · Needs you · In review · Ready to merge
  ⇒ 状态机**不许手填**，只能由证据算出。
· **浏览器 profile 按 worker 隔离** ⇒「并行 UI 任务不共享状态」。

### 1.4 职责边界写得很干净
> The orchestrator owns planning and delegation;
> **workers own implementation, tests, commits, and PRs.**

⇒ 我们 `multi_agent` 的 coordinator/crew 可以照此划界。

---

## 二、`x_gift_bot` —— 值得学的不是功能，是**失败语义**

> 「付款结果不明时，系统**宁可标记「待核实」也不会重复扣款**」；
> 用户用**原兑换码**点「重新检查并继续兑换」自动对账。

⭐ 与 cumora 的 seen-cursor **同原则、反方向**：
cumora 扣留过期回复让它重决策；这里是**承认未知 → 标记待核实 → 事后用同一凭证对账**。
⇒ 两者都拒绝在**结果未知时猜测**。

其余可移植点：
· **凭据逐条 AES-256-GCM 加密**，密钥由**密码文件**派生 ⇒ 丢失即永久不可恢复（明确写出）
· 兑换码**只存摘要与尾号，不存明文**
· `./bin/xgift status` 六条记录全 `verified` ⇒ **验证命令是一等运维接口**
· `XGIFT_PAYMENTS_ENABLED=false` ⇒ 暂停时**兑换码不消耗**（暂停语义显式）

⛔ 未落任何东西：按前置门，**我方是否存在「不可逆动作在结果未知时重试」的危险面，
本轮未查证** ⇒ 不提方案。

---

## 三、`Compositor` —— 一条与 agent 相关的设计
> a `.comp` is **a folder of PNG layers and a manifest**,
> and an open project **updates live as it's written**

⇒ **文件格式本身就是 API**：agent 直接写目录即可编辑工程，无需专用接口。
⭐ 与 lightpanda 的「agent 写一次、之后确定性执行」、Strata 的「双端点」
同族—— **可编程性来自「暴露朴素、可被外部写的中间表示」**。

---

## 四、⛔ 本轮明确**未**做
- `casp.ac/reports/intelligence-explosion` —— 收到但**未核实**，未入库。
  ⭐ 它直指「智能爆炸」，与本仓`EMERGENCE-PLAN` 同题；
  **核实后**可能影响涌现计划的判据，值得单独一次会话处理。
- `markfulton/agent-cookie-sync` —— 收到但**未核实**，未入库。
- 未取任何代码（三源均只读设计）。
- 未写「文档优先级」门（理由见 §1.2末）。

---

# 分栏后的自查：本文哪些是【推】

| 我的说法 | 类型 | 现状 |
|---|---|---|
| Compositor「文件格式本身就是 API」（纯目录 + manifest 即可被 agent 读写） | **【推】· 已验，我方已有等价物** | ✅ **【实测】**我方**有**「文件变化 → 状态更新」通路：`nt_io/nt_io_hotreload/`（基于 `notify`）＋ `entry/interactive.rs:184` 已把 `~/.neotrix/plugins/` 的 HMR 接上（`revertible_effects` C4 接线）<br>⇒ **不是缺口**，是**已实现** |
| x_gift_bot「结果不明 ⇒ 标记待核实」适用于我方 | ⛔ **已推翻** | 我方不是「重复扣款」；而是 `RecoveryAction` **有**消费者（`gateway/execution.rs:632`）。见 `d9fd17f7` |
| ⭐ AO「文档漂移优先级契约」是本轮最该做 | **【推】· 已部分落地** | ✅ 已在 `REFACTORING-UI-SKILL` 篇做分栏试点（`fd88d509`）；本篇正在补做 |
| agent-cookie-sync「硬编码拒绝名单」适用于我方 | ⛔ **未查证** | 我方 `social_access` 的会话面**本轮未查**；⇒ 不提方案 |
| 「Compositor 是**新**能力缺口」 | ⛔ **已推翻** | 同上第一行：HMR 通路已存在 |

⇒ ⭐⭐ 本篇的净结果：**两处我以为的「缺口」，一处已实现、一处判断错误。**
⇒ 这与前几篇的净结果（多为「已实现」）**方向一致** ——
⭐ **一个反复出现的模式：外部方案解决的那些问题，我方往往已经有等价物；
真正缺的很少，且缺的通常是「**表述纪律**」而不是「能力」。**
⇒ ⚠️ 但也要防另一面：**若我不查就提建议，就会把已有能力说成缺口**
⇒ 那会浪费的不只是时间，还会**误导后续判断**（正如我今天错报过两次）。


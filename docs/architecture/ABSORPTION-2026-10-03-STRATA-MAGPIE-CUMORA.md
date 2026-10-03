# 吸收：Strata / magpie / cumora（2026-10-03）

> 三源**同为 MIT** ⇒ 代码层可吸收。本篇记**判据 + 可移植设计**，不搬代码。

| 源 | ★ | commits | 语言 | 许可 |
|---|---|---|---|---|
| `Niko1221/Strata` | 7.4k | 628 | C/C++（含 ggml/llama.cpp） | MIT |
| `yetone/magpie` | 4.4k | 1,420 | Go | MIT |
| `yetone/cumora` | 3.9k | 353 | TS/React + Node + Go | MIT |

---

## 一、`yetone/magpie` —— ⭐ 本轮对我们最相关

### 1.1 ⭐ 一个网关说 **4 种协议**，于是「agent 零改动」
`127.0.0.1:3425` 同时暴露：

```
/v1/chat/completions          OpenAI Chat
/v1/responses                 OpenAI Responses
/v1/messages (+count_tokens)  Anthropic Messages
/v1beta/models/{m}:generateContent   Google Gemini
```

所有 agent 只填一个 base URL ⇒ **跨厂商切换不动 agent 配置**。
⇒ 这与我先前从 lightpanda 记下的「**CDP 作为唯一集成契约**」、
   以及 Strata 的「同时给 `/v1` 与 `/v1/messages`」**同源**：
   **可移植性来自「说标准协议」，不是「提供 SDK」。**

### 1.2 ⭐ **agent 永不持有厂商 key / URL**
模型一律以 `provider/model` 命名，key 只在网关。⇒ **凭据单一收口点**。
⭐ 与 cumora 的 BYOA「server never sees your provider keys」是同一原则，
两个独立实现互相印证 ⇒ **这是共识，不是某人的偏好。**

### 1.3 ⭐ 历史事实**永不按今天的配置回填**
> Older records appear as `key not recorded`, **never inferred from today's configured key**.

⭐⭐ 这正是我这一整轮在执行的纪律（无证据不下结论）。
**它把纪律写进了产品语义**：审计记录缺证据时，正确行为是**显示「无记录」**，
而不是拿现状补一个看起来完整的答案。

### 1.4 会话黏性是一等参数
`routing=` ∈ `smart|order|rotate|usage|pace`，另有 `stays=auto|session|turn|off`。
⇒ 「黏多久」被显式参数化，而不是藏在实现里。
⭐ 直接对应我们 `multi_agent` coordinator —— 我先前判定它是**共享状态**，
现在有了**可借鉴的判据集**：黏性应是可配置策略，不是隐式行为。

### 1.5 ⭐ 配置**外科式**改写
> Only the one key you change is touched; **comments, ordering and indentation survive intact.** Writes are atomic.

⇒ 改别人配置文件时**只碰那个键**，不重排、不丢注释、原子写。
⭐ 我们 `neobot provider` 也在写 agent 配置 ⇒ **这条应作为验收项**，我尚未验证。

---

## 二、`yetone/cumora` —— 多 agent 不打架的**机制层**

### 2.1 ⭐⭐ **seen-cursor 新鲜度闸门**（最可移植的一条）
> a stale reply is **HELD** and shown the newer messages to **re-decide**

不是丢弃过期回复，而是**扣住并让它看到新消息后重新决策**。
另有：真实工作单元上的**原子 claim**、**小脑 triage 闸**保护大模型。

⭐ 这与我们 `multi_agent`（184 测试）解决的是**同一类问题**，
但机制不同：我们主要靠**规划期**避免冲突，它靠**提交期**的新鲜度闸。
⇒ 二者**可叠加**：规划期避冲突 + 提交期扣留过期决策。

### 2.2 ⭐⭐ **默认 fail-closed 沙箱**
> Claude Code and Codex use **fail-closed** filesystem, command-network, and
> subprocess-credential boundaries **by default**; the other engines require an
> **explicit unsandboxed compatibility opt-in**.

⇒ **未知引擎默认拒绝**，兼容旧引擎要**显式开口子**。
⭐ 这是安全默认值方向的正确 polarity：默认收紧、例外显式。

### 2.3 ⭐⭐⭐ 「跳过」长得**像**通过 —— 它把这写成陷阱而不是留给后人踩
> Without `INTEGRATION_DATABASE_URL` it prints `[integration] skipped` and exits 0
> — **which looks like a pass**.

⇒ 与我本轮做的 `check-discarded-inputs.py`（区分 **FAIL** 与 **WARN**）**同题**。
**差异在态度**：他们**把陷阱写进文档**，我把它做成了**机器可判的门**。
⇒ 两者应合并：文档给人，机器给 CI。

### 2.4 真相面纪律
`server/src/db/schema.ts` is a partial model and **is not the schema**（迁移 SQL 才是）。
⇒ 与本仓「行号不是引用」「master blueprint 是唯一图纸」同族。

---

## 三、`Niko1221/Strata` —— 让大模型落进消费级硬件

### 3.1 MoE 专家**跨存储层级卸载**
GPU 放高频专家、RAM 放全部、CPU 算其余、SSD 放查找表。
⇒ 与本仓 `LOCAL-LLAMA-2026-09-28.md` 的 KV/上下文推导是同一问题域。

### 3.2 ⭐ 投机解码：「先猜，再一次性核对」
小助手猜后几个词，大模型一次全核对 ⇒ **1.6–1.8× 更早出答案**。
⇒ 与 lightpanda 的 PandaScript（「agent 写一次、运行时零 token 确定性执行」）
**同族**：把不确定性收敛在生成期。

### 3.3 AI 可自己安装/启停自己
`docs/AI_SETUP.md` 供 AI 编码助手照做；另有 **MCP server** 供工具管理自身。
⇒ 与 open-slide 的「skill 携带硬规则」同族：**规则要机器可读**。

---

## 四、⭐⭐⭐ 跨源汇合判断（本轮唯一值得记住的一条）

五个源（lightpanda / open-slide / **magpie / cumora / Strata**）在**互不知道彼此**的前提下，
**独立收敛到同一条**：

> **把不确定性、凭据、代价推到边界；运行时只跑确定性的东西。**

| 源 | 它把什么推到了边界 |
|---|---|
| lightpanda | 不确定性 → 生成期（PandaScript 零 token 执行） |
| open-slide | 规则 → 机器可读（skill 而非散文） |
| magpie | 凭据 → 网关（agent 永不含厂商 key） |
| cumora | 决策 → 提交期（过期回复被扣留重决策） |
| Strata | 算力 → 存储层级（GPU/RAM/CPU/SSD 分工） |

**反向的共同点更值得注意**：三个源都**显式命名了「看起来正常其实是坏」的东西**
（cumora「skipped 长得像 pass」、magpie「不按今天配置回填历史」、
Strata「首启会卡 1–3 分钟是正常的」）。

⇒ 这与我们本轮反复处理的问题**完全同型**：
`check-discarded-inputs` 的 FAIL/WARN 之分、
「零引用 ≠ 孤儿」、`.bak-legacy` 不是垃圾、
`skipped` 类测试必须可区分。
⇒ **判断：我们的「证伪优先」纪律方向正确，但产物应从「文档」升级为「机器可判」。**

## 五、⛔ 本轮**未**做的事（明确列出）
- 未取任何代码；三源均只读设计。
- 未验证我们 `neobot provider` 改写 agent 配置时**是否保留注释/顺序**（§1.5）——
  这是 §1.5 对我们的**唯一可执行验收项**，尚未测。
- 未评估 cumora 的 seen-cursor 闸门能否移植进 `multi_agent`（需先读我方协调器现状）。

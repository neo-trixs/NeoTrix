# 三仓吸收判定 — morluto/rea · sh1ma/Angelic-Angel · neilsonnn/image-blaster

> **判据**：`ABSORPTION-PRECONDITION-GATE-2026-10-03.md`（三前置门）。
> **纪律**：本文件严格分三栏 —— 【源】外部原文 /【实测】本机跑出来的 /【推】我的推断。
> **核实时间**：2026-10-05。

---

## 0. 取证方法（本身就是个成果）

本轮**没有**用外部 HTTP 工具，而是用 NeoTrix 自己的 CLI 出口：

```sh
neotrix web fetch https://api.github.com/repos/<owner>/<repo> | python3 -c "import sys,json;print(json.load(sys.stdin)['license'])"
```

⭐ **这一步当场撞出两个真缺陷并已修复**（`a507b59b`）——
「用 NeoTrix 自己的能力干活」比「读文档」更能暴露问题：

| 缺陷 | 影响面 | 修法 |
|---|---|---|
| `fetch()` 取 `page.content()`＝**渲染产物** | **所有 JSON API** 返回 `<html><pre>` 包壳 | 取 `document.body.innerText`（源文本） |
| `init_tracing()` 未指定 writer | tracing 默认写 **stdout** ⇒ 污染**全 CLI** 管道 | `.with_writer(std::io::stderr)` |

⚠️ 上一笔 `4f6f6647` 的验收样本是 `example.com`（HTML 页），
**恰好避开了缺陷面** ⇒ 已把「验收样本必须覆盖能力声明的全部形态」
写进 `a507b59b` 的教训段。

---

## 1. LICENSE 三仓全部 MIT（SPDX 核实）

【实测】GitHub API `license.spdx_id`：

| 仓 | SPDX | stars | 语言 | 最后 push | 可否取用代码 |
|---|---|---|---|---|---|
| `morluto/rea` | **MIT** | 2125 | TypeScript | 2026-10-04 | ✅ |
| `sh1ma/Angelic-Angel` | **MIT** | 595 | Rust | 2026-03-05 | ✅ |
| `neilsonnn/image-blaster` | **MIT** | 8232 | TypeScript | 2026-05-15 | ✅ |

⇒ **三仓均无 AGPL / CC-BY-NC 限制**（对比：`lightpanda` AGPL 不可取码、
`blueprint-animation` CC BY-NC 需确认商用）。

---

## 2. `morluto/rea` — ⭐ 吸收「证据分层」词汇表

### 【源】它是什么
> "Reverse engineer anything with agents, from app behavior down to native binaries."

1406 文件，TypeScript，MIT。含 **3 篇 ADR**（`docs/adr/`），是本轮最有价值的部分。

### 【源】ADR-0003 的核心区分（原文术语）
| 术语 | 定义（意译） |
|---|---|
| **canonical static observation** | 在版本化 parser 契约下，**直接从目标字节解码**的有界值 |
| **reconstruction** | 由 canonical observation 导出的**类源码输出**（含反编译 C#） |
| **structural inference** | 跨 observation 的结构判断 |
| **analyst inference** | 分析者的猜测 |

原文关键论断（已核实为 ADR 原文）：
> "That reconstruction is valuable, but it is not the original source and
> cannot replace the underlying metadata and CIL evidence."

### 【推】为什么这对我方重要
我方 `llm_judge` 刚引入 `CriteriaSource::{SelfReported, Independent}`
（`is_emergence_evidence()`），要解决的是**同一类问题**：
「产出的自我评价不能当证据」。

⭐ **但 rea 的四级词汇表比我方二级更完备**：
`Independent` 只区分「同源/异源」，而 rea 区分
**canonical（源字节）/ reconstruction（重建）/ inference（推断）**。
⇒ 我方 `SelfReported` 把「重建」和「推断」混成了一类。

**失败长什么样**：`Independent` 判官给出的分数，
若其输入本身就是上游的 reconstruction（而非 canonical observation），
我方会**误判为涌现证据**。

**如何验证**：构造一个「判官输入 = 上游重建产物」的场景，
断言 `is_emergence_evidence()` 返回 `false`。

### 本轮裁决：**记账 + 下一批实施**
⛔ 不在本批改 `llm_judge` 的枚举 —— 那是**破坏性 API 变更**，
且我方已有 36 组反向锁依赖现语义。须独立一批、带迁移路径。

📌 下一批动作：新增 `EvidenceTier::{Canonical, Reconstruction, Inference}`
（默认 `Reconstruction`，取最坏假设），与 `CriteriaSource` 正交。

---

## 3. `sh1ma/Angelic-Angel` — 📋 只记设计，不接代码

### 【源】它是什么
> "A server for streaming tweets by emulating browser Web Push."

14 文件，Rust，MIT，595★。数据流：
```
Twitter/X ──push──▶ Mozilla AutoPush ◀──WebSocket── Angelic-Angel ──POST──▶ Webhook
```
明确定位：**不抓取、不轮询**（"No scraping … the same mechanism browsers use"）。

### ⭐ 前置门②不过 ⇒ 不接
【推】它的核心机制 = **长连接 push 订阅**，依赖两个外部基础设施：
- `push.services.mozilla.com`（Mozilla AutoPush 公共端点）
- Twitter 的 push 订阅 API（需 `auth_token` + `ct0` cookie）

⇒ 我方**没有 push 订阅方**，接过来意味着：
1. 引入一个我方**无法运维**的外部依赖（它 2026-03 后未更新）；
2. 需要 `auth_token`/`ct0` —— 我方硬规则是**不得收集/导出凭据**，
   与 `social_access` 已确立的「Rust 侧不接触 cookie 值」**直接冲突**。

### 但有两条设计值得记
1. 【源】**重连策略**：指数退避 `5s × 2^n`，上限 5 分钟；
   UAID 失效自动重注册；服务端 backoff（close code **4774**）⇒ 等 30 分钟。
   ⇒ 我方任何 WebSocket/流式连接都缺这套退避规格。
2. 【源】**注册与监听分离**：只在 `register` 时调 API，`listen` 期间**零 API 调用**。
   ⇒ 这是良好的边界纪律（把「有凭据的动作」压到最小窗口）。

---

## 4. `neilsonnn/image-blaster` — 📋 只接一条纪律

### 【源】它是什么
> "An image-to-world skillset for Claude."

140 文件，TypeScript，MIT，**8232★**（本轮三仓中最高）。是一组 Claude Code skill。

### ⭐ 前置门②不过 ⇒ 不接它的代码
【推】其脚本重度依赖**付费外部 API**：
`fal-queue.mjs`（fal.ai 队列）、`hunyuan-3d.mjs`、`meshy-3d.mjs`、
`fal-elevenlabs-sfx.mjs`（11.8KB SFX）、World Labs。
⇒ 我方无这些 API 凭据，接过来即得到一堆**必然失败**的调用路径。

### ✅ 但一条纪律直接可接（已在本批落地）
【源】`image-blast-project/SKILL.md` 第 7 条：
> "Recommend downstream actions **only after no-cost setup/analysis is complete**"

即：**先做完零成本的分析，再推荐付费/重资的下游动作**。

【推】这正是我方反复出现的病：
- 涌现计划容易先谈「上多大规模」再谈「能否廉价验证」；
- 能力清单容易先列「能做什么」再列「先做什么」。

⇒ 已写进 `EMERGENCE-PLAN-2026-10-02.md` 的失效条件（见下）。

---

## 5. 本批实际入库

| 提交 | 内容 | 性质 |
|---|---|---|
| `a507b59b` | `web fetch` JSON 端点取源文本 + tracing 落 stderr | 🔧 **我方真缺陷**（用自己能力时撞出） |
| `d1af58fd` / `d00f799a` / 本批 `headless` | 6 处手画框迁移（全部实测错位量化） | 🔧 既有缺陷 |
| 本文件 + `repos.csv` | 三仓判定入台账（512 源） | 📋 记账 |

---

## 6. 下一批（按价值排序）

| 优先 | 动作 | 依据 |
|---|---|---|
| **P0** | `EvidenceTier::{Canonical, Reconstruction, Inference}` 与 `CriteriaSource` 正交接线 | rea ADR-0003；我方 `is_emergence_evidence()` 现漏「重建」层 |
| **P1** | 「零成本验证先行」纪律入 `EMERGENCE-PLAN` 失效条件 | image-blaster SKILL.md 第 7 条 |
| **P1** | 我方 WebSocket/流式连接补退避规格（5s×2^n / cap 5min） | Angelic-Angel 重连策略 |
| P2 | `reanotrix` 不接 REA 代码本体（多 provider + Ghidra/Hopper 桥接，我方无对应工具链） | rea 依赖 Hopper/Ghidra |

---

## 7. ⛔ 本轮的否定结论（留档，防后人重犯）

- ⛔ **不接** Angelic-Angel 的 push 长连接：需凭据 + 外部基建，违我方凭据边界。
- ⛔ **不接** image-blaster 的 3D/SFX 脚本：全依赖付费 API，接来即死路。
- ⛔ **不接** rea 的 REA 本体：依赖 Hopper/Ghidra 专有工具链，我方无。
- ⛔ **不在本批改** `CriteriaSource` 枚举：破坏性变更 + 36 组反向锁依赖现语义。
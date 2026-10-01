# 许可例外记录（人工签署，门不自动放行）

> 本文件是 `scripts/check-license.sh` 的**唯一**放行通道。
> 一条 deny 命中若无对应签署 ⇒ 门 FAIL。签署不是"消除问题"，
> 是**把决定权与责任显式落到人**（工具无法代替法务判断）。
>
> ⛔ 签署前必须读该 vendored 树自己的 `VENDOR.md` 许可节，那里写清了条款原文。

## 格式

每条一个 `## ACKNOWLEDGE-<n>` 段，字段必须齐全，缺一即视为未签署。
⛔ 下面代码块里是**格式说明**，门会跳过围栏内的内容 ——
**绝不要把真实路径填进模板当签署**，模板本身长得像签署是本文件最大的坑。

```
## ACKNOWLEDGE-<n>
tree:        <vendored 树路径，例如 apps/foo/bar>
clause:      <命中的条款关键字>
decision:    <accepted-with-condition | removed | upstream-permission-obtained>
condition:   <可验证的约束；decision=accepted-with-condition 时必填>
owner:       <签署人姓名>
date:        <YYYY-MM-DD>
review_by:   <YYYY-MM-DD>
evidence:    <授权书/法务意见/商业属性判定 的链接或路径>
```

`decision` 三选一：
- `accepted-with-condition` —— 接受风险，但必须写明 `condition` 与 `review_by`
- `removed` —— 该树已从仓库移除（须与实际一致，否则门仍红）
- `upstream-permission-obtained` —— 已取得书面授权（`evidence` 必填）

---

## 当前状态：**无有效签署 —— ACKNOWLEDGE-1 已于 2026-10-01 自动失效**

### 失效记录（不可删除）

- **2026-10-01**：项目所有者确认 **NeoTrix / NeoBot 为商用**（_distribution is commercial_）。
- ACKNOWLEDGE-1 的 `condition #3` 原文：
  > 「若判定为商用而未取得上游书面授权，则本 ACKNOWLEDGE **自动失效**」
- ⇒ **该例外按其自身条款失效**，`check-license.sh` 随之由 WARN 转 **FAIL**。
  这是设计意图，不是故障：`condition` 写得足够具体，所以事实一旦成立，
  后果无需临场判断。

### 事实认定（基于上游条款原文逐条对照）

上游 `apps/neobot-desktop/frontend/LICENSE.details`：

> 1. 不得用于**二次开发**（含修改、改编、衍生）以获取商业利益…；
>    **直接使用**本软件本身用于商业目的仍被允许。
> 2. 与 MIT 冲突时，**以本附加条款为准**。

| 条款要件 | 本仓事实 | 证据 |
|---|---|---|
| 是否「二次开发」 | **是** | `VENDOR.md` 自带 NeoBot 增量 diff 表（14 项，含 4 个新增文件）；且明确「⛔ 除上表外未动」不成立 —— 改过就是改过 |
| 是否「商业」 | **是** | 所有者 2026-10-01 确认 |
| 上游书面授权 | **无** | 未取得 |
| ⇒ 结论 | **当前状态不满足该附加条款** | — |

⚠️ 上表的「商业」一栏是**所有者的事实陈述**，非 agent 判定。
条款中「直接使用允许商用 / 二次开发禁止」的分界在何种情形下适用，
存在解释空间，**最终须由上游书面确认或法务判断**。

### 规模（供决策，非情绪）

| 项 | 值 |
|---|---|
| vendored 文件数 | **1,147**（git 跟踪） |
| 体积 | **645 MB** |
| ts/tsx 源码 | **982 文件 / 103,240 行** |
| 我方增量 | **14 项**（含 4 个新增文件：`neobot-root.tsx` / `api-panel.ts` / `api-panel.css` / `dom.ts`） |
| 构建硬依赖 | `tauri.conf.json`: `frontendDist=frontend/dist`、`beforeBuildCommand=pnpm run build` |

⇒ **移除该树会直接打断 NeoBot 桌面端构建**（`cargo build -p neobot-desktop` 走 Tauri 构建链）。
这不是删目录能了结的量级，属架构级决定，**agent 不擅自执行**。

### 处置选项（须由所有者选，agent 不代决）

| 选项 | 动作 | 代价 / 前提 |
|---|---|---|
| **A（条款要求的那条）** | 向上游 `dsh-tauri/deepseek-harness-desktop` 取得**书面授权** | 唯一能保留现状的路径。需你发起 |
| **B** | 移除该树，改为自研实现 | 断构建；103K 行不能直接抄（须 clean-room 重写）；多处逻辑派生自它 |
| **C** | 仅内部使用、不随商业产品分发修改版 | 「直接使用」边界模糊，**须上游确认**，不能自行认定 |
| **D** | 回退我方增量至上游 0.19.1 原样 | 丢掉 NeoBot 全部差异化；技术上可做（上游原件逐字保留在树内） |

⛔ **禁止的处置**：为了让门变绿而删 deny 名单、改门脚本、或把本条失效记录删除。
门转绿的唯一正当原因是「取得了上游书面授权」并据此**新签**一条 ACKNOWLEDGE。


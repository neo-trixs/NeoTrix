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

## 当前状态：**已重新签署 ACKNOWLEDGE-2**（取代已失效的 ACKNOWLEDGE-1）

### 事实变更记录（不可删除）

| 日期 | 事实 | 影响 |
|---|---|---|
| 2026-10-01 | 所有者确认**商用** | ACKNOWLEDGE-1 的 `condition #3` 触发 → **自动失效** |
| 2026-10-01 | 所有者改为**开源非商用**（同一日推翻上一条） | 失效原因消失 → 以 **ACKNOWLEDG-2** 重新签署 |

⚠️ 同日两次相反认定，说明**认定会变**。因此 ACKNOWLEDGE-2 的首要条件就是
**让失效可被机器复现**（见 condition #1）。

### 机器核验：认定与仓库内容一致【实测】

对全仓（排除 target/.worktrees/frontend）搜 8 个商用信号
`stripe / paddle / lemonsqueezy / subscription plan / enterprise license /
paid tier / pricing page / per-seat` ⇒ **8 条命中，逐条读原文全部是假阳性**：

- 7 条是 `PaddleOCR` / `PaddleOcrEngine` 命中 `paddle` 子串
- 1 条是 `nt_mcp_scan_secrets.rs:70` 的**密钥扫描黑名单** `"stripe-api-key"`
  （即本仓在*扫描*这类 key，不是用它收款）

⇒ 仓库内**零**定价/支付信号，「开源非商用」不是口头声明，与内容一致。

---

## ACKNOWLEDGE-2

tree:        apps/neobot-desktop/frontend
clause:      No Commercial Secondary Development (frontend/LICENSE.details)
decision:    accepted-with-condition
owner:       NeoTrix 项目所有者 —— 决定由所有者作出，agent 于 2026-10-01 代为记录
             （不填具体人名：agent 不得冒充个人签署）
date:        2026-10-01
review_by:   2026-12-01
evidence:    VENDOR.md「2026-10-01 评估结论」节 + 本文件「机器核验」节
does_not_resolve: 见下方「残留风险」—— 签署**不**等于上游已授权

condition:

1. **一旦出现任何商用迹象，本条立即自动失效**（无需人工判断）。
   判据：新增支付/定价集成、改为非 MIT 许可、发布付费层、企业版/订阅版、
   或以「支持/咨询/托管」形式对该功能收费。
   ⚠️ 鉴于同日已发生一次认定反转，本条不得依赖「记得回来改」。

2. 上游 `LICENSE` 与 `LICENSE.details` **逐字保留**，不得删除或改写
   —— 由 `scripts/check-license.sh` 机器强制

3. MIT 署名与许可声明必须传播到**分发产物**中（MIT §「保留声明」义务）；
   尤其 vendored 树随二进制/安装包分发时，其 `LICENSE` + `LICENSE.details`
   必须随之到达用户

4. 2026-12-01 前必须重审

### 残留风险（签署**不**解决，需上游确认或法务判断）

条款原文禁止的是「以获取**商业利益**为目的的二次开发」，并写明
「**直接使用**本软件本身用于商业目的仍被允许」。

- ✅ **本项目自身非商用** ⇒ 不落入该禁止范围。回到基础 MIT，
  MIT 允许修改与再分发。**这是本条签署的依据。**
- ⚠️ **但根 `LICENSE` 是 MIT** ⇒ 一旦 NeoBot 以 MIT 发布，**下游获得整棵树
  （含 vendored 部分）的权利**，而**商业下游用户**所做的正是条款禁止的
  「商业二次开发」。
  ⇒ 该限制是随文件走的，**我们的非商用认定管不住下游**。
  这是本条**未解决**的核心问题，**只有两条出路**：
  (a) 上游书面确认「非商用二次开发 + MIT 再分发」在其允许范围内；
  (b) 把 vendored 部分从 MIT 可再分发范围中排除（技术上是可行的隔离，
      但会改变 NeoBot 的分发形态，属架构级决定）。
- ⚠️ 「非商用」在实践中难界定：免费但用于盈利场景、免费增值引流等。
  本条以**所有者声明**为准，不做技术判定。

⛔ 禁止的处置：为了让门变绿而删 deny 名单、改门脚本、或删除失效/变更记录。
门转绿的唯一正当原因是「取得了上游书面授权」并据此新签。


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

## 当前状态：**已签署 1 条**（见 ACKNOWLEDGE-1）

`check-license.sh` 因此由 FAIL 转 WARN（非静默放行：每轮 CI 仍打印该条）。
签署的**效力边界**见 ACKNOWLEDGE-1 的 `does_not_resolve` 字段 ——
**签署不等于许可方已授权**，它只把「谁决定接受这个风险」落到纸面。

---

## ACKNOWLEDGE-1

<!-- 字段必须**裸露**（不在 ``` 围栏内）。解析器会跳过围栏内容 ——
     那道跳过正是为了防止上面的格式模板被当成真签署。
     我第一次就是把字段包进围栏 ⇒ 门读不到 ⇒ 仍判「无人签署」。 -->

tree:        apps/neobot-desktop/frontend
clause:      No Commercial Secondary Development (frontend/LICENSE.details)
             原文：「不得用于二次开发（含修改、改编、衍生）以获取商业利益…；冲突时以本附加条款为准」
decision:    accepted-with-condition
owner:       NeoTrix 项目所有者 —— 决定由所有者作出，agent 于 2026-10-01 代为记录
             （刻意不填具体人名：agent 不得冒充个人签署）
date:        2026-10-01
review_by:   2026-12-01
evidence:    apps/neobot-desktop/frontend/VENDOR.md「⛔ 许可：不是纯 MIT」一节
             （条款原文 + 三点必知 + 本条 condition 的落地方式）
does_not_resolve: 本签署不构成法律意见；不判定 NeoTrix 实际分发是否构成 commercial
             （属所有者事实判断）；若上游与本条冲突，签署不产生豁免力

condition:

1. 上游 LICENSE 与 LICENSE.details **逐字保留**，不得删除或改写
   —— 已由 `scripts/check-license.sh` 强制（非人工纪律，是门）
2. 任何**商用分发**前必须重新评估本条，并在 VENDOR.md 记录评估结论
3. 若判定为商用而未取得上游书面授权，则本 ACKNOWLEDGE **自动失效**，
   须移除该 vendored 树（注意：本仓多处逻辑派生自它，属架构级决定）
4. 2026-12-01 前必须重审（条款可能变更 / 商业属性可能变更）

⛔ 仍然禁止的处置：**为了让门变绿而删 deny 名单或改门脚本。**
门转绿的原因是「有人显式签了字并写了条件」，不是「问题被隐藏」。

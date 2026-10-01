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

## 当前状态：**无任何签署**

⇒ `check-license.sh` 现在 FAIL，**这是正确状态**，不是待修的 bug。

原因见 `apps/neobot-desktop/frontend/VENDOR.md` 的
「⛔ 许可：不是纯 MIT」一节：上游 `LICENSE.details` 规定
「不得用于以商业利益为目的的二次开发，冲突时以附加条款为准」，
而本目录**正在被持续修改**。是否构成商用二次开发属项目所有者判断，
不由 agent 代签。

# open-design 接入评估（C1，2026-10-08）

> 源：`nexu-io/open-design`（Apache-2.0，~99,918★，TypeScript）。
> 定位：「The First Collaborative Design Agent Workspace」——协同设计 agent 工作区，
> 原生支持 DeepSeek Harness(`dsh`) 运行时（structured thinking / tool calls /
> model discovery / cancellation / session resume），产物留在 OpenDesign 工作流里做
> live preview 与交付。

## 评估结论

**裁决：📋 路线图（P2 能力），本会话不接线。** 三条依据：

1. **形态不匹配当前 neobot 主线**：neobot 的运行面是 CLI + 多渠道（A2 WeCom 刚落地），
   open-design 是独立的桌面/协同工作区产品。把它当「库」吸收 = 平行引入一个 UI 栈。
2. **能力可经 MCP/渠道复用**：open-design 已原生支持 dsh runtime ⇒ 若未来 neobot
   需要设计工作流，走「neobot 作为设计 agent 的上游运行时」这条路，而不是把它
   内嵌进 neobot。
3. **价值判定**：99,918★ 说明「设计 agent 工作区」是真实强需求，但属产品能力
   而非架构缺陷 ⇒ 不影响 L0–L6 主线的正确性面。

## 备选（不做内嵌）

- ✅ **对照**：C2 桌面发布规格已覆盖「双通道 + 问后再装」，open-design 的
  「DeepSeek Harness 作为官方首发 agent runtime」是它的最佳实践参照。
- 📋 **未来接线路径**：若用户提出设计工作流需求 → neobot 暴露一个
  OpenDesign-compatible 的 agent 描述符（capability/model/tool discovery），
  让 OpenDesign 把 neobot 当 runtime 调起（与它支持 dsh 同一形状）。
- ❌ **不做**：把 open-design 的 TypeScript 代码搬进 neotrix。

## 与 code map 路线的关系

- 不动源码 ⇒ code map 无增量（与 Trendshift 批次同型：KB 入库即可）。
- 已在 KB `absorption-2026-10-08` domain 入库（`batch_1791440632_*`，
  capability 标注为 `neobot-UI`）。

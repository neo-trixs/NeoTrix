# B1 接通方案 —— `capability_invoke` 生产不可达：为何不能顺手修

> 2026-10-06 · 由 neobot 接线审计产出，并经实测核实
> 状态：**未修**，需要架构裁决。已修的前置见 `3c858062`（fail-closed）与 `371a4207`（计数守卫）。

## 现象

`capability_invoke` 在生产中**永不上桌**。链路：

1. 挂载点：`nt_http_engine.rs:470` `capability_market_ids()`，
   `:471` 的 `if !market_ids.is_empty()` 决定是否把工具 push 进 `tool_schemas`
2. 播种点：`neotrix-core/.../nt_mind_background_loop/mod.rs:180` 构造
   `consciousness_runtime`，由它把 5 个 trade 能力写入注册表

实测结果：**生产中这两处从不在同一进程** ⇒ 注册表恒空 ⇒ `market_ids` 恒空
⇒ `:471` 守卫**恒假** ⇒ 工具永不上桌。
`nt_http_engine.rs:1051` 的测试在测试内播种，故全绿。

## 根因不是「忘了调」，而是依赖方向

- `neotrix-core` → `neotrix-neobot` 是既有方向；neobot **不得**反向依赖 core
  （`nt_agent.rs:1703-1704` 已把这条写进代码注释）
- ⇒ neobot **无法**调用 core 的 `BackgroundLoop` 去播种
- ⇒ 也不能直接读 core `nt_act_trade` 里的权威元数据

## 候选方案与代价

### 方案 A：neobot 自带能力清单（自播种）

在 neobot 里声明「我能上架哪些能力」，用 `nt-core-capability-tree`（**已依赖**，
`Cargo.toml:42`）写入注册表。

- ✅ 成本最低，能立刻让工具上桌
- ⛔ **代价：在 neobot 复制一份能力元数据（id/kind/domain/category/license/
  maturity/version）⇒ 产生第二个真身。**
  本仓已反复记录此类问题：`layer-map.json` 的幽灵 consumer、
  `nt_git` 与 core `git_integration.rs:11` 的重复类型、
  `capability-topology-map` 与 `all-capability-analysis` 的同源同错双胞胎。
  ⇒ **本文件判定：方案 A 不可接受，除非先建立单一真身与同步机制。**

### 方案 B：把「谁播种」变成可注入端口

在 neobot 定义 `CapabilitySeed` trait（或直接收`Vec<MarketEntry>`），
由**上层装配代码**注入 —— 装配层同时能看到 core 的播种结果与 neobot 的 HttpEngine。

- ✅ 单一真身，元数据只在一处
- ✅ 与既有方向兼容（装配层可同时依赖两者）
- ⛔ 需要新增装配点，改动面比 A 大

### 方案 C：下沉能力清单到共享 crate

把「能力市场清单」从 core 下沉到 `nt-core-capability-tree`（neobot 已依赖它）。

- ✅ 单一真身 + 两侧都能读，最符合依赖方向
- ⛔ 需把 core 侧现有播种逻辑迁过去，是一次真正的重构

## 建议

**选方案 C**，退而求其次选 B。**不选 A。**

理由：A 用「立刻上桌」换「第二个真身」。本仓的历史教训里，
凡是「两份同源数据」的资产，最终两份都腐化（见 `_superseded/` 里那四份
同源同错的地图档）⇒ 上桌之后仍会变成「看着健康、实际陈旧」，
正是本轮一直在治的病。

## 在裁决之前的兜底

`3c858062` 已把 `capability_invoke` 改为 **fail-closed**：
即使工具上桌而本体未执行，也只得到 `ok:false` +
`CAPABILITY_BODY_NOT_EXECUTED`，且**不计数**。
⇒ 裁决期间不会产生「模型调用 → 报成功 → 计数+1 但零执行」的假绿通路。

## 验收信号（裁决完成后看这两条）

```sh
bash scripts/ops/neobot-check-market.sh
```

- 「注册但从未被调用」应从 5 项降下来（随真实调用出现）
- 「金丝雀 fired_count == 0」应不再 5/5
- 另需一条新断言：`capability_invoke` 是否出现在 `tool_schemas` 里
  （B1 修好的直接判据，目前该门看不到）
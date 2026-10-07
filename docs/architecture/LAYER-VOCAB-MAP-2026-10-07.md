# 层词汇映射表（LAYER-VOCAB-MAP，2026-10-07）

> **目的**（裁道 T0-4）：`check-layer-deps.sh`、`capability-truth`、`nt_review`
> 三方各自自洽 ⇒ 同时绿却可能互相矛盾。根因是同一套 `L<n>` 数字前缀
> 在三套词汇里含义不同。本表是这三套的**唯一映射裁决**，判据：
> 凡提及 `L0`…`L9`，必须同时声明属于哪套词汇。

## 三套词汇现状

| 词汇 | 范围 | 定义处 | 消费方 |
|---|---|---|---|
| **A · 目录层（正典）** | `l0_substrate`…`l6_meta`（7）+ `outside` | 目录名 + `.neotrix/layer-map.json` + `ArchLayer::REAL_DIRS` | `check-layer-deps.sh`、`nt_review`、CI |
| **B · 能力高度** | `NodeLayer::{L0Primitive, L1Composite, L2Orchestrator, L2World, L3DomainService, L3Memory, L4Application, L4Cognition, L5Conscious, L6Self, L7Capability, L8Autonomic}`（12） | `crates/nt-core-capability-tree/src/node.rs:145-158` | 318 节点 registry、`capability-truth` |
| **C · 事件总线路由标签** | `EventRouteLayer::{Body, World, Memory, Knowledge, Reasoning, SelfTier, Capability, Autonomic, Meta}`（9；label 仍是 `"L1"`…`"L9"` 日志标签） | `neotrix-core/src/l0_substrate/nt_core_event_bus.rs:282` | 仅 event_bus 内部（测试 43 条） |

## 数字前缀相撞分析（例：`L4` / `L6`）

| 前缀 | A（目录层） | B（能力高度） | C（事件路由） |
|---|---|---|---|
| `L4` | `l4_emotion` | `L4Cognition` / `L4Application` | `Knowledge` |
| `L6` | `l6_meta` | `L6Self` | `SelfTier` |

结论：三套**不存在 1:1 映射**——B 把「L4」分配给思维层（Cognition），A 分配给情绪层（l4_emotion）；
早期的 `LayerId`（C 旧名）同样与 A/B 均不对等。

## 裁决（2026-10-07 实测落地）

1. **A 是唯一的目录层载体**。所有层归属问题一律查 `layer-map.json` + `REAL_DIRS`。
2. **B 与 C 是各自的独立语义**。
   - B 以能力节点高度为粒度，JSON 键沿用小写串，**不得把 B 的
     `L4Cognition`/`L6Self` 强行折叠进 A 的 `l4_*`/`l6_*`**。
   - C 已去掉 Rust 侧数字前缀（`LayerId` → `EventRouteLayer`，
     变体 `L5Reasoning` → `Reasoning`）；其 `"L1"…"L9"` 仅为日志标签，不代表层归属。
3. **B 的 `as_str()` 塌缩**（`L2Orchestrator` 与 `L2World` 同映射 `"L2"` 等）
   与 `parse_layer` 兜底 `_ => L0Primitive` 为已知缺陷，待单列修复。
4. 新增「层」概念前必须先问：它归 A、B 还是 C？不得引入第四套前缀。
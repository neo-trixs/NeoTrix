# as_str() audit — T0-4 (NodeLayer)

判据：registry.rs `by_layer` histogram 与 `.neotrix/capability_registry.json` 318 节点 `layer` 字段是否同一种串格式；旧 as_str 是否 collapse。

| 项 | 结论 |
|---|---|
| `NodeLayer::as_str()`（node.rs:161-176） | 新串：`l0primitive`/`l1composite`/`l2orchestrator`/`l2world`/`l3domainservice`/`l3memory`/`l4application`/`l4cognition`/`l5conscious`/`l6self`/`l7capability`/`l8autonomic`（12 类） |
| serde 规则 | `#[serde(rename_all = "lowercase")]` ⇒ 序列化产物 = as_str 同串 ⇒ 已对齐 |
| `registry.rs:474` by_layer histogram key | `node.layer.as_str()` ⇒ 新串，与 registry JSON `layer` 同格式 ✅ |
| `cli.rs:591` / `cli.rs:927` | 仅展示 `layer.as_str()`，无解析依赖 |
| `evolution.rs:446` | `layer.as_str()` 仅进入 graft-plan 可读文案；`constellation.as_str()` 同理（文案）。无统计依赖旧串 ✅ |
| `roadmap.rs:62-74` / `cli.rs:486-499` parse_layer | 输入侧兼容旧别名（`l2`→L2Orchestrator、`L2`→…），不产生输出冲突 |
| `nt_audit.rs:79`（neotrix-audit） | 展示 `hit.layer.as_str()`，无分组消费 |
| git 变更 | T0-4 提交 `4e1aae76`「NodeLayer::as_str 不再坍缩 + parse_layer 未知值拒绝」 |

## registry JSON 318 节点 layer 直方图（实测）

| layer 串 | 计数 |
|---|---|
| l0primitive | 108 |
| l1composite | 89 |
| l2orchestrator | 44 |
| l2world | 8 |
| l3domainservice | 20 |
| l3memory | 4 |
| l4cognition | 15 |
| l5conscious | 1 |
| l6self | 13 |
| l7capability | 12 |
| l8autonomic | 4 |
| l4application | 0（12 类中唯一空档） |
| 合计 | 318，distinct 11 |

## collapse 判定

旧 as_str 曾把 `L2Orchestrator`/`L2World` 都输出 `L2`、`L3*`→`L3`、`L4*`→`L4` ⇒ histogram 与 registry JSON 中已分开的节点（44 vs 8、20 vs 4、15 vs 0）无法区分（`ROADMAP-REDUNDANCY-FLAT-MISALIGN-2026-10-07.md:95` 已记录同一症状）。新串下 by_layer key 与 registry `layer` 同源 ⇒ collapse 已除。

## 残留 semantics 冲突（同类问题未收口）

| 轴 | as_str 输出 | registry JSON 真值 | 是否同格式 |
|---|---|---|---|
| `NodeLayer` | `l2orchestrator`… | 同左 | ✅ 一致 |
| `ConstellationLevel` | `c0compile`/`c1unittest`/`c2integrationtest`/`c4mainpipeline`/`c5selfhealing`/`c6evolutionloop` | 同左 | ✅ 已对齐 |
| `Domain` | `core`/`mind`…（snake_case） | 一致于 serde `rename_all="snake_case"` | ✅ 已对齐 |

结论：`NodeLayer`、`ConstellationLevel`、`Domain` 三轴 as_str 均已与 registry JSON schema 对齐；此前 T0-4 表述中的两轴冲突为早期口径，已失效。

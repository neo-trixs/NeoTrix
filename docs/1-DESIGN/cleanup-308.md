# 跨层引用扫描报告 — cleanup-308

扫描时间: 2026-09-11 18:36:05
扫描路径: `/Users/neo/Downloads/neotrix/neotrix-core/src`
排除文件: \*test\*, \*facade\*, \*mod.rs\*

## 摘要

跨层引用 **5 处**，全部源自 `l5_cognition`。无其他层间交叉。L5 违规集中在 `nt_mind` 子模块（进化循环 + 后台维护），属于认知层过度下探行动层实现细节，违反 Layered Architecture 依赖规则（上层可依赖下层，但禁止跨层直连非相邻层）。

---

## 源层: `l1_action` (0 处引用)

> 无跨层引用

## 源层: `l2_perception` (0 处引用)

> 无跨层引用

## 源层: `l3_embodiment` (0 处引用)

> 无跨层引用

## 源层: `l4_emotion` (0 处引用)

> 无跨层引用

## 源层: `l5_cognition` (5 处引用)


### `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs`

```
679:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch};
680:        use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{
940:        use crate::l6_meta::coordination::self_improvement::SystemMetrics;
```


### `l5_cognition/nt_mind/evolution/self_diagnose.rs`

```
10:use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
```


### `l5_cognition/nt_mind/evolution/evolution_loop.rs`

```
67:pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
```



## 源层: `l6_meta` (0 处引用)

> 无跨层引用

---

## 违规方向分析

| 源文件 | 目标层 | 方向 | 依赖类型 |
|--------|--------|------|----------|
| `l5_cognition/nt_mind/handlers_maintenance.rs:679` | `l1_action::nt_memory` | L5→L1 ⬇️ | 数据访问 (CommunityDetector) |
| `l5_cognition/nt_mind/handlers_maintenance.rs:680` | `l1_action::nt_memory` | L5→L1 ⬇️ | 数据访问 (KB store) |
| `l5_cognition/nt_mind/handlers_maintenance.rs:940` | `l6_meta::coordination` | L5→L6 ⬆️ | 逆向依赖 (SystemMetrics) |
| `l5_cognition/nt_mind/evolution/self_diagnose.rs:10` | `l1_action::nt_act` | L5→L1 ⬇️ | 类型引用 (ProjectSnapshot) |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs:67` | `l1_action::nt_act` | L5→L1 ⬇️ | 类型 re-export (ProjectSnapshot) |

**依赖方向**:
- L5→L1: 4 处 (认知层直接访问行动层，违反分层隔离)
- L5→L6: 1 处 (认知层反向依赖元认知层，逆向依赖)

---

## 修复建议

| 编号 | 文件 | 问题 | 建议 |
|------|------|------|------|
| F1 | `handlers_maintenance.rs:679-680` | L5 直接访问 L1 KB 内部类型 | 抽取 `l1_action::nt_memory` 的 trait interface，L5 通过 trait 调用 |
| F2 | `handlers_maintenance.rs:940` | L5 引用 L6 SystemMetrics | SystemMetrics 下沉到 `l5_cognition` 或抽取 shared types crate |
| F3 | `self_diagnose.rs:10` | L5 引用 L1 ProjectSnapshot | ProjectSnapshot 移入 `l5_cognition` 共享类型，或提取到 shared lib |
| F4 | `evolution_loop.rs:67` | L5 re-export L1 类型 | 同 F3，类型归属需重新审视 |

---

## 统计

| 源层 | 跨层引用数 |
|------|-----------|
| `l1_action` | 0 |
| `l2_perception` | 0 |
| `l3_embodiment` | 0 |
| `l4_emotion` | 0 |
| `l5_cognition` | 5 |
| `l6_meta` | 0 |

| **总计** | **5** |

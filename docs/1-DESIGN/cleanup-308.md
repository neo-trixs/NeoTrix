# 跨层引用扫描报告 — cleanup-308

扫描时间: 2026-09-11 18:36:05
扫描路径: `/Users/neo/Downloads/neotrix/neotrix-core/src`
排除文件: \*test\*, \*facade\*, \*mod.rs\*

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

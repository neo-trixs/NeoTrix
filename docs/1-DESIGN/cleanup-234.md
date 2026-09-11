# cleanup-234 — 代码库状态统计

**日期**: 2026-09-11

## 跨域引用

```
l5_cognition→l1_action:   2
l5_cognition→l2_perception: 1
l5_cognition→l6_meta:      2
```

**合规**: 仅 L5→低层/高层引用，符合六层架构方向（上层依赖下层）。

## Facade 模块

| 文件 | 层 |
|------|-----|
| `l2_perception/nt_world/l1_facade.rs` | L2 |
| `l3_embodiment/l1_facade.rs` | L3 |
| `l5_cognition/act_facade.rs` | L5 |
| `l5_cognition/io_facade.rs` | L5 |
| `l5_cognition/io_skills_facade.rs` | L5 |
| `l5_cognition/kb_facade.rs` | L5 |
| `l5_cognition/l2_facade.rs` | L5 |
| `l5_cognition/l3_facade.rs` | L5 |
| `l5_cognition/l6_facade.rs` | L5 |
| `l6_meta/l1_facade.rs` | L6 |

**共 10 个 facade 文件**，L5 占 6 个（认知层是 facade 主要消费者）。

## 统计

| 指标 | 数值 |
|------|------|
| 总 .rs 文件 | 1855 |
| 总行数 | 606,530 |

## 跨域引用分析

- **违规引用**: 0（无同层引用、无下层引用上层）
- **Facade 依赖**: L5 通过 facade 访问 L1/L2/L3/L6，符合隔离规则
- **剩余跨域**: L5→L6 (2) 为元认知层引用，合理（向上引用元数据）

# Cleanup 228 — 代码库状态快照

> 生成时间: 2026-09-08

## 跨域引用

| 源→目标 | 引用数 |
|---------|--------|
| l2_perception→l1_action | 1 |
| l5_cognition→l1_action | 2 |
| l5_cognition→l2_perception | 1 |
| l5_cognition→l6_meta | 2 |
| l6_meta→l5_cognition | 1 |

**违规检测**: l2_perception→l1_action 违反 L2→L1 禁止规则（L2 禁止直接依赖 L1）

## Facade 模块

```
l2_perception/nt_world/l1_facade.rs
l3_embodiment/l1_facade.rs
l5_cognition/act_facade.rs
l5_cognition/io_facade.rs
l5_cognition/io_skills_facade.rs
l5_cognition/kb_facade.rs
l5_cognition/l2_facade.rs
l5_cognition/l3_facade.rs
l5_cognition/l6_facade.rs
l6_meta/l1_facade.rs
```

## 统计

| 指标 | 数值 |
|------|------|
| .rs 文件总数 | 1851 |
| 代码行总数 | 605,905 |

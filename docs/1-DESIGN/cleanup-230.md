# Codebase Status — Cleanup 230

## 跨域引用

| Source → Target | Count |
|----------------|-------|
| l5_cognition → l1_action | 2 |
| l5_cognition → l2_perception | 1 |
| l5_cognition → l6_meta | 2 |
| l6_meta → l5_cognition | 1 |

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

| Metric | Value |
|--------|-------|
| Total .rs files | 1851 |
| Total lines | 605,916 |

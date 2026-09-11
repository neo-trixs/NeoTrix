# Cleanup 232 — 代码库统计快照

## 跨域引用

| From → To | Count |
|-----------|-------|
| l5_cognition → l1_action | 2 |
| l5_cognition → l2_perception | 1 |
| l5_cognition → l6_meta | 2 |
| l6_meta → l5_cognition | 1 |

**依赖方向**: l6_meta ↔ l5_cognition 双向引用；l5_cognition → l1_action/l2_perception 下行引用。

## Facade 模块

```
neotrix-core/src/l2_perception/nt_world/l1_facade.rs
neotrix-core/src/l3_embodiment/l1_facade.rs
neotrix-core/src/l5_cognition/act_facade.rs
neotrix-core/src/l5_cognition/io_facade.rs
neotrix-core/src/l5_cognition/io_skills_facade.rs
neotrix-core/src/l5_cognition/kb_facade.rs
neotrix-core/src/l5_cognition/l2_facade.rs
neotrix-core/src/l5_cognition/l3_facade.rs
neotrix-core/src/l5_cognition/l6_facade.rs
neotrix-core/src/l6_meta/l1_facade.rs
```

**统计**: 10 个 Facade 文件，其中 l5_cognition 层占 7 个（上行暴露 act/io/kb/l2/l3/l6），l3_embodiment/l2_perception 各 1 个（暴露 l1），l6_meta 1 个（暴露 l1）。

## 汇总

| 指标 | 值 |
|------|-----|
| Total .rs files | 1,854 |
| Total lines | 606,516 |
| Facade files | 10 |
| Cross-layer deps | 6 |

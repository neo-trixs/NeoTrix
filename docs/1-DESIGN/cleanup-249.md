# 跨域引用检查 — cleanup-249

**日期**: 2026-09-11

## 汇总

| 引用方向 | 引用数 | 实际代码引用 | 注释引用 |
|----------|--------|-------------|---------|
| l5_cognition→l1_action | 2 | 0 | 2 (注释) |
| l5_cognition→l2_perception | 1 | 0 | 1 (注释) |
| l5_cognition→l6_meta | 2 | 0 | 2 (注释) |

**总计**: 5 条引用，全部为注释中的文档说明，无实际跨层 `use` 导入。

## 结论

**六层架构跨域隔离良好**。所有跨域引用均为注释（`//`），非真实代码依赖：

- `l5_cognition→l1_action` — 2 条注释，描述已废弃的 `pub use` 路径
- `l5_cognition→l2_perception` — 1 条注释，说明 L2 类型访问方式
- `l5_cognition→l6_meta` — 2 条注释，说明 L6 类型访问方式

**无真实跨层依赖违反**，架构约束有效。

## 明细

### l5_cognition → l1_action (注释)

```
l5_cognition/nt_mind/evolution/self_diagnose.rs:32 — 已注释 pub use
l5_cognition/nt_mind/evolution/evolution_loop.rs:21 — 已注释 pub use
```

### l5_cognition → l2_perception (注释)

```
l5_cognition/mod.rs:16 — L2 类型访问说明
```

### l5_cognition → l6_meta (注释)

```
l5_cognition/mod.rs:22 — L6 类型访问说明
l5_cognition/traits.rs:137 — 向上依赖警告
```

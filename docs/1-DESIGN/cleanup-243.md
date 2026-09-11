# Cleanup 243 — 跨域引用审计

**日期**: 2026-09-11 13:21  
**类型**: 架构合规性检查  
**范围**: neotrix-core 六层架构跨域依赖

---

## 1. 跨域引用汇总

| 引用方向 | 数量 | 状态 |
|----------|------|------|
| `l5_cognition → l1_action` | 2 | ✅ 已注释 |
| `l5_cognition → l2_perception` | 1 | ✅ 文档说明 |
| `l5_cognition → l6_meta` | 2 | ✅ 已注释/文档说明 |
| **其他跨域** | **0** | ✅ 无违规 |

**总跨域引用**: 5 处  
**活跃依赖**: 0 处（全部已注释或仅为文档说明）

---

## 2. 详情

### 2.1 `l5_cognition → l1_action` (2 处)

```rust
// l5_cognition/nt_mind/evolution/self_diagnose.rs:32
// pub use crate::l1_action::nt_act::nt_l1_shared_types::{...};

// l5_cognition/nt_mind/evolution/evolution_loop.rs:21
// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;
```

**分析**: 两处均已注释掉，无实际依赖。

### 2.2 `l5_cognition → l2_perception` (1 处)

```rust
// l5_cognition/mod.rs:16
/// L2 类型通过此模块访问, 避免散布 `use crate::l2_perception::*`。
```

**分析**: 文档注释，说明访问方式，无实际 `use` 语句。

### 2.3 `l5_cognition → l6_meta` (2 处)

```rust
// l5_cognition/mod.rs:22
/// L6 类型通过此模块访问, 避免散布 `use crate::l6_meta::*`。

// l5_cognition/traits.rs:137
// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations
```

**分析**: 一处为文档注释，一处为解释性注释，无实际依赖。

---

## 3. 架构合规性评估

| 维度 | 评估 |
|------|------|
| **层级依赖方向** | ✅ 符合（仅 L5 文档层有向上下文注释） |
| **活跃跨域依赖** | ✅ 0 处（全部已注释） |
| **Facade 模式** | ✅ 已排除测试和 facade 文件 |
| **向上依赖** | ✅ 无（L5→L6 仅为文档说明） |
| **向下依赖** | ✅ 无（L5→L1/L2 已注释） |

---

## 4. 结论

**架构健康度**: ✅ **完全合规**

六层架构跨域引用控制良好：
- 所有活跃依赖均为同层或向下依赖
- 所有潜在违规（向上依赖）已通过注释消除
- 文档层（L5）保留了对上下层的说明性注释，但无实际代码依赖
- 无 `l1_action`/`l2_perception`/`l3_embodiment`/`l4_emotion` 向上引用

**无需清理操作**。

---

## 5. 建议

1. **可选优化**: 将 L5 mod.rs 中的文档注释移至 `CONTEXT.md` 或架构文档中，保持代码层零跨域引用
2. **监控**: 在 CI 中添加 `use crate::l{N}` 跨层检测脚本，防止未来引入违规依赖

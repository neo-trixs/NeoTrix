# 跨域引用分析

## 跨域引用统计

| 来源 → 目标 | 引用次数 |
|---|---|
| l5_cognition → l1_action | 4 |
| l5_cognition → l6_meta | 2 |
| l5_cognition → l2_perception | 1 |

## 具体引用详情

### l5_cognition → l1_action (4)

```rust
// nt_mind/evolution/self_diagnose.rs:10
use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;

// nt_mind/evolution/self_diagnose.rs:33 (注释)
// pub use crate::l1_action::nt_act::nt_l1_shared_types::{...};

// nt_mind/evolution/evolution_loop.rs:21 (注释)
// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;

// nt_mind/evolution/evolution_loop.rs:67
pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
```

### l5_cognition → l6_meta (2)

```rust
// mod.rs:22 (注释)
/// L6 类型通过此模块访问, 避免散布 `use crate::l6_meta::*`。

// traits.rs:137 (注释)
// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations
```

### l5_cognition → l2_perception (1)

```rust
// mod.rs:16 (注释)
/// L2 类型通过此模块访问, 避免散布 `use crate::l2_perception::*`。
```

## 分析

### l5_cognition 跨域引用

l5_cognition（认知层）是唯一有跨域引用的层级，共 7 处引用：

1. **→ l1_action (4)**:
   - 2 处实际代码引用（`ProjectSnapshot` 类型）
   - 2 处注释引用（已注释掉或仅作文档说明）

2. **→ l6_meta (2)**:
   - 2 处均为注释，用于文档说明如何避免向上依赖

3. **→ l2_perception (1)**:
   - 1 处为注释，用于文档说明如何避免向上依赖

### 跨域引用特点

1. **单向依赖**: 所有跨域引用都来自 l5_cognition，其他层级无跨域引用
2. **引用集中**: l5_cognition 是跨域依赖的中心节点
3. **依赖方向**: 认知层依赖行动层、感知层、元认知层
4. **注释为主**: 大部分跨域引用为注释或文档说明，实际代码引用较少

## 建议

1. **实际代码引用**: 仅 `ProjectSnapshot` 类型被实际引用，考虑将其移动到共享模块
2. **注释引用**: 注释中的跨域引用可以保留，作为架构说明
3. **依赖解耦**: 如果 `ProjectSnapshot` 被频繁引用，考虑通过 trait 或共享类型模块解耦

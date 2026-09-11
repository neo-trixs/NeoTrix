# cleanup-266: 跨域引用检查

日期: 2026-09-11
执行: `grep -rn "use crate::$to" neotrix-core/src/$from`

## 结果汇总

```
=== 跨域引用 ===
l5_cognition→l1_action: 4
l5_cognition→l2_perception: 1
l5_cognition→l6_meta: 2
```

## 详细引用

### l5_cognition → l1_action (4 处)

| 文件 | 行号 | 引用内容 |
|------|------|----------|
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | 10 | `use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` |
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | 33 | `// pub use crate::l1_action::nt_act::nt_l1_shared_types::{...` (注释) |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 21 | `// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;` (注释) |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 67 | `pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` |

**实际活跃引用**: `ProjectSnapshot` 类型从 L1→L5 认知层使用（2 处）

### l5_cognition → l2_perception (1 处)

| 文件 | 行号 | 引用内容 |
|------|------|----------|
| `l5_cognition/mod.rs` | 16 | `/// L2 类型通过此模块访问, 避免散布 use crate::l2_perception::*。` (注释) |

**实际活跃引用**: 0 处（仅文档注释）

### l5_cognition → l6_meta (2 处)

| 文件 | 行号 | 引用内容 |
|------|------|----------|
| `l5_cognition/mod.rs` | 22 | `/// L6 类型通过此模块访问, 避免散布 use crate::l6_meta::*。` (注释) |
| `l5_cognition/traits.rs` | 137 | `// use crate::l6_meta::* 造成向上依赖。Concrete implementations` (注释) |

**实际活跃引用**: 0 处（仅文档/注释）

## 分析

### 依赖方向（仅活跃代码）

```
L5 认知层 ──→ L1 行动层 (4 处引用, 2 处活跃)
```

### 违规判定

| 引用 | 违规 | 说明 |
|------|------|------|
| L5→L1 | ✅ 违规 | 上层依赖下层，但项目快照类型 (`ProjectSnapshot`) 本应属于共享层 |
| L5→L2 | ❌ 无违规 | 仅注释，无实际依赖 |
| L5→L6 | ❌ 无违规 | 仅注释，无实际依赖 |

### 建议

1. `ProjectSnapshot` 应提升至 `l1_action::traits` 或 `nt_shared_types`，避免 L5 直接依赖 L1 内部模块
2. L5→L2 和 L5→L6 注释中的防护说明有效，无需修改

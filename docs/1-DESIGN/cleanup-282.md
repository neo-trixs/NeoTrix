# Cleanup #282 — 跨域引用分析

## 跨域引用

| 从 → 到 | 引用数 (总) | 活跃引用 |
|----------|-------------|----------|
| l5_cognition → l1_action | 4 | 2 |
| l5_cognition → l2_perception | 1 | 0 (仅注释) |
| l5_cognition → l6_meta | 2 | 0 (仅注释) |

## 活跃引用详情

### l5_cognition → l1_action (2处)

| 文件 | 引用内容 |
|------|----------|
| `nt_mind/evolution/self_diagnose.rs:10` | `use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` |
| `nt_mind/evolution/evolution_loop.rs:67` | `pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;` |

**分析**: 两处均引用 `ProjectSnapshot` 类型，用于 SEAL 进化循环中获取项目状态快照。此为**数据类型依赖**，非行为依赖。

## 架构合规分析

### ✅ 合规引用
- **l5_cognition → l6_meta** (2): 仅存在于注释中，实际无活跃引用，架构合规

### ⚠️ 需审查引用
- **l5_cognition → l1_action** (2): 
  - 引用类型：`ProjectSnapshot` (数据结构)
  - 合理性：SEAL 进化循环需要读取项目状态，但直接引用 L1 层类型
  - 建议：可考虑将 `ProjectSnapshot` 提升至共享类型层 (`core/`)，或通过 trait 抽象解耦

## 总结

实际跨层依赖仅 **1处** (2个文件引用同一类型)，均非循环依赖。`ProjectSnapshot` 作为数据类型被 L5 层引用，虽架构上可接受，但建议后续重构时考虑将其移至 `core/` 层以保持分层纯净性。

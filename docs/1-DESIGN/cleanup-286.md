# Cleanup 286 — 跨域引用检查

**日期**: 2026-09-11
**检查范围**: `neotrix-core/src/{l1_action,l2_perception,l3_embodiment,l4_emotion,l5_cognition,l6_meta}`

---

## 1. 跨域引用统计

| 跨域方向 | 数量 | 状态 |
|---------|------|------|
| l5_cognition → l1_action | 4 | ⚠️ 需审查 |
| l5_cognition → l2_perception | 1 | ✅ 注释/文档 |
| l5_cognition → l6_meta | 2 | ✅ 注释/文档 |
| **总计** | **7** | |

## 2. 实际跨域引用（排除注释/文档）

### 2.1 l5_cognition → l1_action (4 处)

| 文件 | 行号 | 引用内容 | 类型 |
|------|------|---------|------|
| `nt_mind/evolution/self_diagnose.rs` | 10 | `use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` | **代码** |
| `nt_mind/evolution/evolution_loop.rs` | 67 | `pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` | **代码** |
| `nt_mind/evolution/self_diagnose.rs` | 33 | `// pub use crate::l1_action::...` | 注释 |
| `nt_mind/evolution/evolution_loop.rs` | 21 | `// pub use crate::l1_action::...` | 注释 |

**分析**: L5 (认知层) 直接引用 L1 (行动层) 的 `ProjectSnapshot` 类型。这违反了层级依赖方向（上层不应直接依赖下层，应通过 facade 或 trait）。

### 2.2 l5_cognition → l2_perception (1 处)

| 文件 | 行号 | 引用内容 | 类型 |
|------|------|---------|------|
| `mod.rs` | 16 | `/// L2 类型通过此模块访问...` | 文档注释 |

**状态**: 无实际代码依赖，仅文档说明。

### 2.3 l5_cognition → l6_meta (2 处)

| 文件 | 行号 | 引用内容 | 类型 |
|------|------|---------|------|
| `mod.rs` | 22 | `/// L6 类型通过此模块访问...` | 文档注释 |
| `traits.rs` | 137 | `// use crate::l6_meta::*` 造成向上依赖 | 注释 |

**状态**: 无实际代码依赖，仅注释说明避免向上依赖。

## 3. 反向引用分析（被依赖热力图）

| 被引用层 | 被引用次数 | 主要引用者 |
|---------|-----------|-----------|
| **l1_action** | 51 | core/*, cli/*, l5_cognition/* |
| **l5_cognition** | 47 | core/*, l6_meta/* |
| **l3_embodiment** | 16 | l5_cognition/* |
| **l2_perception** | 8 | l5_cognition/*, l6_meta/* |
| **l6_meta** | 8 | core/*, l5_cognition/* |
| **l4_emotion** | 1 | core/* |

**观察**: L1 (行动层) 被依赖最多 (51次)，是基础设施层。L5 (认知层) 被依赖 47 次，是核心逻辑层。

## 4. Facade 模式使用情况

| 层 | Facade 模块 | 用途 |
|----|------------|------|
| l2_perception | `l1_facade` | 封装 L1 依赖 (KnowledgeBase, download 等) |
| l3_embodiment | `l1_facade` | 封装 L1 依赖 |
| l5_cognition | `kb_facade`, `io_facade`, `io_skills_facade`, `act_facade`, `l3_facade`, `l2_facade`, `l6_facade` | 多层 facade 封装 |
| l6_meta | `l1_facade` | 封装 L1 依赖 |

**评价**: Facade 模式已广泛使用，但 L5 仍有直接跨层引用需清理。

## 5. 问题清单

### P0 — 必须修复

| # | 问题 | 位置 | 修复方案 |
|---|------|------|---------|
| 1 | L5 直接引用 L1 `ProjectSnapshot` | `self_diagnose.rs:10`, `evolution_loop.rs:67` | 将 `ProjectSnapshot` 移入 L5 的 `act_facade` 或提升到共享类型层 |

### P1 — 建议优化

| # | 问题 | 位置 | 修复方案 |
|---|------|------|---------|
| 2 | L5 有 6 个 facade 模块，过于分散 | `l5_cognition/mod.rs` | 考虑合并为 `l5_cognition/facades.rs` 统一管理 |
| 3 | L1 被 51 处引用，存在"上帝层"风险 | 跨层引用分析 | 评估 L1 是否应拆分为更细粒度的子模块 |

## 6. 依赖方向合规性

```
L6 (元认知) ──→ L5 (认知) ──→ L4 (情感)
                    │              │
                    ▼              ▼
               L3 (具身) ──→ L2 (感知)
                    │              │
                    └──────────────┘
                         │
                         ▼
                    L1 (行动) ←── 所有层可依赖
```

**合规状态**:
- ✅ L6 → L5: 合规（通过 `l6_facade`）
- ✅ L5 → L3: 合规（通过 `l3_facade`）
- ✅ L5 → L2: 合规（通过 `l2_facade`）
- ❌ L5 → L1: **不合规**（直接引用，未通过 facade）

## 7. 建议修复步骤

1. **立即修复 P0-1**: 在 `l5_cognition/act_facade.rs` 中 re-export `ProjectSnapshot`，修改 `self_diagnose.rs` 和 `evolution_loop.rs` 使用 facade
2. **审查 P1-2**: 评估 L5 facade 合并的收益/成本
3. **长期规划 P1-3**: 监控 L1 引用增长，必要时拆分

---

*生成自 `cleanup-286` 跨域引用检查*

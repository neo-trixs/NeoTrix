# 跨层引用扫描报告

## 扫描范围
- 目标目录: `neotrix-core/src/`
- 架构: 6层架构 (L1 Action → L6 Meta-Cognition)
- 排除: test模块, facade文件
- 扫描时间: 2026-09-11

## 6层架构定义

| 层 | 目录 | 包含模块 |
|---|---|---|
| L1 Action | `l1_action/` | nt_act, nt_io, nt_memory, nt_infra_* |
| L2 Perception | `l2_perception/` | nt_world, nt_sense |
| L3 Embodiment | `l3_embodiment/` | nt_physical, nt_shield, nt_feel |
| L4 Emotion | `l4_emotion/` | nt_feel (核心) |
| L5 Cognition | `l5_cognition/` | nt_core, nt_mind |
| L6 Meta-Cognition | `l6_meta/` | nt_meta, nt_repair, nt_nexus |

## 跨层引用分析

### 跨层引用规则
- 允许: 同层内模块引用
- 允许: 上层引用下层 (如 L5 → L1)
- 禁止: 下层引用上层 (如 L1 → L5)
- 禁止: 跨层循环依赖

### L1 l1_action → 其他层引用

**未发现违规引用**

### L2 l2_perception → 其他层引用

**未发现违规引用**

### L3 l3_embodiment → 其他层引用

**未发现违规引用**

### L4 l4_emotion → 其他层引用

**未发现违规引用**

### L5 l5_cognition → 其他层引用

**发现违规引用:**

- `neotrix-core/src/l5_cognition/mod.rs:22:/// L6 类型通过此模块访问, 避免散布 `use crate::l6_meta::*`。`
- `neotrix-core/src/l5_cognition/traits.rs:137:// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations`

### L6 l6_meta → 其他层引用

**未发现违规引用**

## 违规引用详情

### 详细违规列表

### 详细违规列表


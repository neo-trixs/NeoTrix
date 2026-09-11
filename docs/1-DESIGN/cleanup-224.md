# Cleanup #224 — 跨域引用最终状态

**Date**: 2026-09-11
**检测命令**: `grep -rn "use crate::$to" neotrix-core/src/$from --include="*.rs" | grep -v test | grep -v facade`

## 全量跨域检测结果

| 源→目标 | 引用数 | 状态 |
|---------|--------|------|
| l2_perception→l1_action | 1 | ⚠️ 存在 |
| l3_embodiment→l1_action | 6 | ⚠️ 存在 |
| l5_cognition→l1_action | 2 | ⚠️ 存在 |
| l5_cognition→l2_perception | 5 | ⚠️ 存在 |
| l5_cognition→l6_meta | 2 | ⚠️ 存在 |
| l6_meta→l5_cognition | 1 | ⚠️ 存在 |

**总计跨域引用**: 17 处 (非 test/facade)

## 依赖图分析

```
L6 Meta-Cognition ←→ L5 Cognition (双向)
L5 Cognition → L1 Action (2)
L5 Cognition → L2 Perception (5)
L5 Cognition → L6 Meta (2)
L3 Embodiment → L1 Action (6)
L2 Perception → L1 Action (1)
```

### 层级依赖规则

- **L1 (Action)**: 无向上引用 → ✅ 叶节点
- **L2 (Perception)**: 仅引用 L1 → ✅ 单向下行
- **L3 (Embodiment)**: 仅引用 L1 → ✅ 单向下行
- **L4 (Emotion)**: 无跨域引用 → ✅ 独立
- **L5 (Cognition)**: 引用 L1, L2, L6 → ⚠️ 下行+上行
- **L6 (Meta)**: 引用 L5 → ⚠️ 下行

### 违规依赖

| 违规类型 | 位置 | 描述 |
|---------|------|------|
| L5→L6 上行 | l5_cognition→l6_meta: 2 | L5 认知层引用 L6 元认知层 (应为 L6 引用 L5) |
| L6→L5 下行 | l6_meta→l5_cognition: 1 | L6 引用 L5 (合法，但与 L5→L6 形成双向) |

**双向依赖**: L5 ↔ L6 (5处总计)

## 清理状态

| 层 | 当前引用 | 最大允许 | 状态 |
|----|---------|---------|------|
| L1 Action | 9 (被引用) | — | ✅ 被引用最多 |
| L2 Perception | 6 (1出+5入) | 1出 | ⚠️ L5引用L2需审查 |
| L3 Embodiment | 6 (6出) | — | ✅ 单向下行 |
| L4 Emotion | 0 | — | ✅ 隔离 |
| L5 Cognition | 9 (2+5+2出) | — | ⚠️ 扇出过大 |
| L6 Meta | 3 (1出+2入) | — | ⚠️ 双向依赖 |

## 建议

1. **L5↔L6 双向**: 最高优先级，元认知层与认知层不应双向引用
2. **L5→L2 (5处)**: 检查是否可通过 trait 抽象消除直接依赖
3. **L3→L1 (6处)**: 具身层引用行动层，需确认是否为必要依赖

---
*由 cleanup-224 跨域检测生成*

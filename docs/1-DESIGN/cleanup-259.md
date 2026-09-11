# cleanup-259 — 跨域引用分析

Date: 2026-09-11

## 跨域引用

```
l5_cognition→l1_action:        4
l5_cognition→l2_perception:    1
l5_cognition→l6_meta:          2
```

## 解读

| 引用 | 计数 | 合规性 |
|------|------|--------|
| l5_cognition→l1_action | 4 | ⚠️ 违反层级依赖：Cognition 向下依赖 Action 层 |
| l5_cognition→l2_perception | 1 | ⚠️ 违反层级依赖：Cognition 向下依赖 Perception 层 |
| l5_cognition→l6_meta | 2 | ✅ 合规：向上依赖元认知层允许（进化/自省） |

## 依赖矩阵 (6×6)

```
            l1_action  l2_perception  l3_embodiment  l4_emotion  l5_cognition  l6_meta
l1_action       -          0              0             0            0           0
l2_perception   0          -              0             0            0           0
l3_embodiment   0          0              -             0            0           0
l4_emotion      0          0              0             -            0           0
l5_cognition    4          1              0             0            -           2
l6_meta         0          0              0             0            0           -
```

**唯一违规源**: `l5_cognition` — 向下引用 l1_action(4) 和 l2_perception(1)。

## 建议

- **l5_cognition→l1_action (4处)**: 通过 trait 抽象或 Event Bridge 解耦，避免认知层直接调用行动层
- **l5_cognition→l2_perception (1处)**: 通过 PerceptionBridge trait 接口解耦

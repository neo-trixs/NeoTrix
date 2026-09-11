# cleanup-246: 跨域引用扫描

**Date**: 2026-09-11
**Scope**: neotrix-core/src/{l1_action,l2_perception,l3_embodiment,l4_emotion,l5_cognition,l6_meta}

## 扫描结果

```
=== 跨域引用 ===
l5_cognition→l1_action:  2
l5_cognition→l2_perception:  1
l5_cognition→l6_meta:  2
```

## 分析

| 引用 | 数量 | 风险 |
|------|------|------|
| l5_cognition→l1_action | 2 | ⚠️ 顶层向下引用行动层 |
| l5_cognition→l2_perception | 1 | ⚠️ 顶层向下引用感知层 |
| l5_cognition→l6_meta | 2 | ✅ 正常 — 元认知与认知层交互 |

## 结论

- **l5_cognition 向下引用 l1_action (2处)**: 认知层直接依赖行动层，违反单向依赖原则（上层不应直接依赖下层的具体实现）。应通过 trait 抽象或事件总线解耦。
- **l5_cognition 向下引用 l2_perception (1处)**: 认知层直接依赖感知层，同上。
- **l5_cognition↔l6_meta (2处)**: 认知层与元认知层的双向交互是正常架构设计（Meta-Cognition 需要读取认知状态）。

## 建议

1. 排查 `l5_cognition` 中对 `l1_action` 和 `l2_perception` 的具体引用点
2. 评估是否可通过 trait 抽象、EventBus 或注入依赖方式解耦
3. L5→L1/L2 的引用应走 L3 (Embodiment) 中间层或通过 trait 接口

# cleanup-275 — 跨域引用检查

**日期**: 2026-09-11

## 跨域引用统计

| 来源域 → 目标域 | 引用数 |
|----------------|--------|
| l5_cognition → l1_action | 4 |
| l5_cognition → l2_perception | 1 |
| l5_cognition → l6_meta | 2 |

## 分析

仅 `l5_cognition`（认知层）存在跨域引用：

- **→ l1_action (4)**: 认知层依赖行动层，可能用于工具调用或动作执行
- **→ l2_perception (1)**: 认知层依赖感知层，可能用于感知数据获取
- **→ l6_meta (2)**: 认知层依赖元认知层，可能用于元认知协调

**无违规方向**：高层（L6/L5）引用低层（L1/L2）属于合理依赖方向，不存在反向依赖。

## 依赖拓扑

```
L6 Meta-Cognition
    ↑ (L5→L6: 2)
L5 Cognition
    ↓ (L5→L1: 4, L5→L2: 1)
L4 Emotion
L3 Embodiment
L2 Perception
L1 Action
```

**结论**: 跨域引用清晰，无循环依赖，无低层反向依赖高层。

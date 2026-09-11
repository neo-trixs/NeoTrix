# Cleanup-252: 跨域引用扫描

**日期**: 2026-09-11 14:19

## 跨域引用 (不含 test/facade)

```
=== 跨域引用 ===
l5_cognition→l1_action:        2
l5_cognition→l2_perception:    1
l5_cognition→l6_meta:          2
```

## 分析

| 路径 | 引用数 | 评估 |
|------|--------|------|
| l5_cognition→l1_action | 2 | ⚠️ 上层直接引用下层实现层 |
| l5_cognition→l2_perception | 1 | ⚠️ 认知层直接引用感知层 |
| l5_cognition→l6_meta | 2 | ✅ 向元认知层引用合理 |

## 其他层间引用

- **l1_action**: 0 跨层引用
- **l2_perception**: 0 跨层引用
- **l3_embodiment**: 0 跨层引用
- **l4_emotion**: 0 跨层引用

## 结论

- **总跨层引用数**: 5 (全部来自 l5_cognition)
- **l5_cognition** 是唯一有跨层引用的层，且方向混乱 (同时向下引用 l1/l2，向上引用 l6)
- 架构合规性: **中等** — 需要检查 l5→l1/l2 引用是否可通过 trait 抽象消除

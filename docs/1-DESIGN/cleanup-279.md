# cleanup-279 — 跨域引用扫描

**Date**: 2026-09-11

## 跨域引用矩阵

| 源层 | 目标层 | 引用数 | 类型 |
|------|--------|--------|------|
| l5_cognition | l1_action | 4 | 认知层 → 行动层 |
| l5_cognition | l2_perception | 1 | 认知层 → 感知层 |
| l5_cognition | l6_meta | 2 | 认知层 → 元认知层 |

## 依赖拓扑

```
l6_meta ─────┐
             │ (2 refs)
l5_cognition ┼─→ l1_action (4 refs)
             │
             └─→ l2_perception (1 ref)

l4_emotion   (无跨层引用)
l3_embodiment (无跨层引用)
```

## 分析

- **l5_cognition** 是唯一的跨层依赖源 — 依赖 l1、l2、l6
- **l4_emotion**、**l3_embodiment** 完全独立，无跨层引用
- l5→l1 (4) 是最大依赖通道 — 认知层直接调用行动层

## 健康度评估

- 跨层依赖总数：**7**
- 唯一依赖源：**1** (仅 l5_cognition)
- 依赖方向：**单向** (l5 向下/向上引用，无循环)
- 结论：**架构清洁** — 无循环依赖，l5 作为中枢协调者符合设计

# 跨域引用状态 — Cleanup 222

## 检测时间
2026-09-11

## 全量跨域检测结果

| 路径 | 跨域引用数 |
|------|-----------|
| l1_action→l2_perception | 3 |
| l2_perception→l1_action | 6 |
| l3_embodiment→l1_action | 13 |
| l5_cognition→l1_action | 36 |
| l5_cognition→l2_perception | 5 |
| l5_cognition→l3_embodiment | 2 |
| l5_cognition→l6_meta | 8 |
| l6_meta→l1_action | 2 |
| l6_meta→l5_cognition | 1 |

## 总计
- 跨域引用总数: **76**
- 涉及跨域方向: **9**

## 依赖方向分析

### 高频依赖 (≥10)
| 路径 | 数量 | 说明 |
|------|------|------|
| l5_cognition→l1_action | 36 | 认知层大量依赖行动层工具/动作 |
| l3_embodiment→l1_action | 13 | 具身层依赖行动层 |

### 中频依赖 (2-9)
| 路径 | 数量 | 说明 |
|------|------|------|
| l5_cognition→l6_meta | 8 | 认知层依赖元认知层协调 |
| l2_perception→l1_action | 6 | 感知层依赖行动层 |
| l5_cognition→l2_perception | 5 | 认知层依赖感知层数据 |
| l1_action→l2_perception | 3 | 行动层反向依赖感知层 |
| l5_cognition→l3_embodiment | 2 | 认知层依赖具身层 |
| l6_meta→l1_action | 2 | 元认知层依赖行动层 |

### 低频依赖 (1)
| 路径 | 数量 | 说明 |
|------|------|------|
| l6_meta→l5_cognition | 1 | 元认知层依赖认知层 |

## 六层架构理想依赖模型

```
L6 (Meta)  → L5 (Cognition) → L4 (Emotion)
                                      ↓
L3 (Embodiment) → L2 (Perception) → L1 (Action)
```

**理想流向**: 高层→低层，禁止低层→高层逆向依赖（接口/特质除外）

## 风险项

### ⚠️ 逆向依赖
| 问题 | 路径 | 数量 | 风险等级 |
|------|------|------|---------|
| L1→L2 逆向 | l1_action→l2_perception | 3 | 中 |
| L2→L1 跨层 | l2_perception→l1_action | 6 | 低 |
| L6→L1 逆向 | l6_meta→l1_action | 2 | 高 |
| L5→L6 逆向 | l5_cognition→l6_meta | 8 | 高 |

### 📊 健康度评估
- **总跨域引用**: 76
- **理想依赖 (L6→L5→L4→L3→L2→L1)**: 约 49 (64.5%)
- **逆向/异常依赖**: 约 27 (35.5%)

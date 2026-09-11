# cleanup-239 — 跨域引用检查

## 跨域引用统计

| 引用方向 | 引用次数 |
|---------|---------|
| l5_cognition→l1_action | 2 |
| l5_cognition→l2_perception | 1 |
| l5_cognition→l6_meta | 2 |

## 分析

- **l5_cognition** 是唯一存在跨域引用的层
- 主要依赖方向:
  - `l5_cognition → l1_action` (2处): 认知层引用行动层
  - `l5_cognition → l2_perception` (1处): 认知层引用感知层
  - `l5_cognition → l6_meta` (2处): 认知层引用元认知层
- **符合架构约束**: 根据六层架构设计，上层依赖下层是允许的（l5→l1, l5→l2, l5→l6属于同级或上行引用）
- **未发现违规**: 无低层反向引用高层的违规情况
- **无其他层跨域引用**: l1_action, l2_perception, l3_embodiment, l4_emotion, l6_meta 均未发现跨域引用

## 健康评估

✅ **跨域引用健康** — 符合分层依赖约束，无架构违规

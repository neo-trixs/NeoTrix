# Cleanup 255 — 跨域引用检查

**检查时间**: 2026-09-11  
**检查范围**: `neotrix-core/src/{l1_action,l2_perception,l3_embodiment,l4_emotion,l5_cognition,l6_meta}`  
**检查方式**: `grep -rn "use crate::$to"` (排除 test/facade)

## 结果

| 来源 → 目标 | 引用数 |
|-------------|--------|
| l5_cognition → l1_action | 2 |
| l5_cognition → l2_perception | 1 |
| l5_cognition → l6_meta | 2 |

**总计**: 3 个跨域引用，5 处引用点

## 分析

- **l5_cognition** 是主要的跨域引用者（5/5 引用）
- 其他层（l1-l4, l6）**零跨域引用**，符合架构分层
- l5_cognition 向上引用 l6_meta (2处) 和向下引用 l1_action (2处)、l2_perception (1处)

## 建议

| 引用 | 风险 | 建议 |
|------|------|------|
| l5→l1 | 低 | 允许 — 认知层可调度行动层（任务执行） |
| l5→l2 | 低 | 允许 — 认知层可读取感知层（信息输入） |
| l5→l6 | 低 | 允许 — 认知层可调用元认知层（自省/修复） |

**结论**: 跨域引用均符合架构设计，无需清理。

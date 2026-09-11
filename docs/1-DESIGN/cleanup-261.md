# Cleanup 261 — 跨域引用检查

**日期**: 2026-09-11
**范围**: neotrix-core/src/ 六层架构跨域引用扫描
**排除**: test / facade 文件

## 跨域引用结果

| 引用方向 | 引用次数 | 状态 |
|---------|---------|------|
| l5_cognition → l1_action | 4 | ⚠️ |
| l5_cognition → l2_perception | 1 | ⚠️ |
| l5_cognition → l6_meta | 2 | ⚠️ |

## 完整矩阵

| 从 ↓ \ 到 → | l1_action | l2_perception | l3_embodiment | l4_emotion | l5_cognition | l6_meta |
|-------------|-----------|---------------|---------------|------------|--------------|---------|
| **l1_action** | — | 0 | 0 | 0 | 0 | 0 |
| **l2_perception** | 0 | — | 0 | 0 | 0 | 0 |
| **l3_embodiment** | 0 | 0 | — | 0 | 0 | 0 |
| **l4_emotion** | 0 | 0 | 0 | — | 0 | 0 |
| **l5_cognition** | **4** | **1** | 0 | 0 | — | **2** |
| **l6_meta** | 0 | 0 | 0 | 0 | 0 | — |

## 分析

- 仅 l5_cognition 存在跨域引用，其他层无跨域依赖 ✅
- l5_cognition → l1_action (4处)：需确认是否符合层依赖规则（高层可依赖低层）
- l5_cognition → l2_perception (1处)：需确认是否应抽象为 trait 接口
- l5_cognition → l6_meta (2处)：⚠️ 高层引用更高层，违反分层依赖方向

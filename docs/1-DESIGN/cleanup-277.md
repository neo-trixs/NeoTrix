# 跨域引用审计 — Cycle 277

**日期**: 2026-09-11 15:57
**检测方式**: `grep -rn "use crate::$to" neotrix-core/src/$from --include="*.rs" | grep -v test | grep -v facade`

## 跨域引用结果

| 源层 | 目标层 | 引用数 |
|------|--------|--------|
| l5_cognition | l1_action | 4 |
| l5_cognition | l2_perception | 1 |
| l5_cognition | l6_meta | 2 |

**总计**: 7 处跨域引用

## 分析

**违规引用**:
- `l5_cognition → l1_action` (4处) — L5 认知层直接依赖 L1 行动层，破坏分层架构
- `l5_cognition → l2_perception` (1处) — L5 认知层直接依赖 L2 感知层
- `l5_cognition → l6_meta` (2处) — L5 认知层直接依赖 L6 元认知层（向上依赖）

**合规层**:
- l1_action: 无跨域引用 ✓
- l2_perception: 无跨域引用 ✓
- l3_embodiment: 无跨域引用 ✓
- l4_emotion: 无跨域引用 ✓
- l6_meta: 无跨域引用 ✓

## 修复建议

1. **l5_cognition → l1_action**: 通过 trait 抽象或事件总线解耦，避免直接调用
2. **l5_cognition → l2_perception**: 通过感知层 trait 接口访问
3. **l5_cognition → l6_meta**: 元认知层应向下提供接口，而非向上依赖

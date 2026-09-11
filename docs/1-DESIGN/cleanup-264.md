# cleanup-264.md — 跨域引用检查

**日期**: 2026-09-11

## 跨域引用

| from | to | count |
|------|-----|-------|
| l5_cognition | l1_action | 4 |
| l5_cognition | l2_perception | 1 |
| l5_cognition | l6_meta | 2 |

### 分析

- **l5_cognition → l1_action**: 4 处引用（认知层依赖行动层）
- **l5_cognition → l2_perception**: 1 处引用（认知层依赖感知层）
- **l5_cognition → l6_meta**: 2 处引用（认知层依赖元认知层）

### 其他层级

- l1_action、l2_perception、l3_embodiment、l4_emotion、l6_meta 无跨域引用（排除 test/facade）

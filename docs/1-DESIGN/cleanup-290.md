# Cleanup 290: 跨域引用检查

**日期**: 2026-09-11  
**类型**: 架构审计  
**范围**: neotrix-core/src 6层架构跨域引用

---

## 跨域引用矩阵

检查 `use crate::$to` 在各层目录中的引用（排除 test 和 facade）：

```
=== 跨域引用 ===
l5_cognition→l1_action:        4
l5_cognition→l2_perception:    1
l5_cognition→l6_meta:          2
```

## 分析

### l5_cognition → l1_action (4处)

**问题**: 认知层(L5)直接引用行动层(L1)，违反分层依赖方向（L5应依赖L2/L3，而非L1）。

**风险**: 认知层耦合底层执行细节，违反"上层依赖下层"原则。

### l5_cognition → l2_perception (1处)

**状态**: 合理。L5依赖L2感知层是允许的（认知依赖感知）。

### l5_cognition → l6_meta (2处)

**问题**: 认知层(L5)引用元认知层(L6)，形成反向依赖。L6是最高层，L5不应依赖L6。

**风险**: 循环依赖风险，破坏层级拓扑。

## 待修复项

| # | 引用 | 严重度 | 修复建议 |
|---|------|--------|----------|
| 1 | l5_cognition→l1_action (4) | 高 | 提取接口到L2/L3，或使用事件总线解耦 |
| 2 | l5_cognition→l6_meta (2) | 高 | L6通过trait暴露能力，L5不应直接import L6 |

## 合规引用（无问题）

- l5_cognition → l2_perception: 1处（认知依赖感知，合理）
- l5_cognition → l3_embodiment: 0处（正确）
- l5_cognition → l4_emotion: 0处（正确）
- l5_cognition → l5_cognition: 自引用（排除）

## 后续动作

1. 定位 l5_cognition→l1_action 的 4 处引用，评估是否可解耦
2. 定位 l5_cognition→l6_meta 的 2 处引用，评估是否可反转依赖
3. 补充 l4_emotion 和 l6_meta 的跨域引用统计（如有遗漏）

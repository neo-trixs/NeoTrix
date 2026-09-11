# Cleanup #289 — 跨域引用检查

**日期**: 2026-09-11
**检查范围**: neotrix-core/src/ 6 层目录

## 跨域引用汇总

```
l5_cognition→l1_action: 4
l5_cognition→l2_perception: 1
l5_cognition→l6_meta: 2
```

## 依赖方向

```
L5 (Cognition) ──┬──→ L1 (Action)     : 4 次引用
                  ├──→ L2 (Perception) : 1 次引用
                  └──→ L6 (Meta)       : 2 次引用
```

## 分析

- **l5_cognition → l1_action**: 4 处引用，需检查是否可通过 trait 抽象解耦
- **l5_cognition → l2_perception**: 1 处引用，可能为感知数据读取
- **l5_cognition → l6_meta**: 2 处引用，元认知回调

**其他层**: 无跨域引用（L1/L2/L3/L4/L6 未向上或向下直接引用）

## 清理建议

1. `l5_cognition→l6_meta` — 元认知回调，架构允许，保持
2. `l5_cognition→l1_action` — 检查是否可通过 eventbus 解耦
3. `l5_cognition→l2_perception` — 检查是否为必要的感知数据读取

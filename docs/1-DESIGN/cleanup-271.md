# Cleanup 271: 跨域引用检查

**检查时间**: 2026-09-11

## 跨域引用统计

```
l5_cognition→l1_action: 4
l5_cognition→l2_perception: 1
l5_cognition→l6_meta: 2
```

## 分析

- **l5_cognition** 是跨域引用的焦点，共 7 处跨域依赖：
  - → l1_action: 4 处（最高）
  - → l6_meta: 2 处
  - → l2_perception: 1 处
- 其余层（l1-l4, l6）无跨域引用问题

## 关注点

l5_cognition（认知层）对低层的依赖需要审查是否为合理的上层依赖下层架构，还是存在反向依赖。

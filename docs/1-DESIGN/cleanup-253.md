# Cleanup 253: 跨域引用分析

> Generated: 2026-09-10

## 跨域引用

```
l5_cognition→l1_action: 2
l5_cognition→l2_perception: 1
l5_cognition→l6_meta: 2
```

## 分析

- **l5_cognition** 是唯一产生跨域引用的层
- 向下引用: l1_action (2) + l2_perception (1)
- 向上引用: l6_meta (2)
- 其他层 (l1_action, l2_perception, l3_embodiment, l4_emotion, l6_meta) 无跨域依赖

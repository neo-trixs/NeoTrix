# 跨域引用检查

## 结果

| 从 → 到 | 引用数 |
|----------|--------|
| l5_cognition → l1_action | 2 |
| l5_cognition → l2_perception | 1 |
| l5_cognition → l6_meta | 2 |

## 分析

- **l5_cognition** 是唯一产生跨域引用的层
- 引用方向：向下（→l1_action, →l2_perception）和向上（→l6_meta）
- 其余层（l1_action, l2_perception, l3_embodiment, l4_emotion, l6_meta）无跨域引用

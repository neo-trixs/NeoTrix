# cleanup-276 — 跨域引用扫描

日期: 2026-09-11

## 跨域引用

| from → to | count |
|-----------|-------|
| l5_cognition→l1_action | 4 |
| l5_cognition→l2_perception | 1 |
| l5_cognition→l6_meta | 2 |

## 分析

**l5_cognition** 是唯一存在跨域引用的层，共 7 处引用：

- → l1_action (4): 认知层引用行动层
- → l2_perception (1): 认知层引用感知层
- → l6_meta (2): 认知层引用元认知层

其余层 (l1_action, l2_perception, l3_embodiment, l4_emotion, l6_meta) 无跨域引用。

**结论**: l5_cognition 层依赖较多，需审查是否可通过 trait 抽象或依赖倒置解耦。

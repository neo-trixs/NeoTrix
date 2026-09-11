# cleanup-284.md — 跨域引用检查

**日期**: 2026-09-11
**命令**: `grep -rn "use crate::$to" neotrix-core/src/$from --include="*.rs" | grep -v test | grep -v facade`

## 跨域引用统计

| from → to | count | 说明 |
|-----------|-------|------|
| l5_cognition → l1_action | 4 | 认知层引用行动层 |
| l5_cognition → l2_perception | 1 | 认知层引用感知层 |
| l5_cognition → l6_meta | 2 | 认知层引用元认知层 |

## 分析

仅 **l5_cognition** 存在跨域引用，共 7 处：

- **l5_cognition → l1_action (4)**: 最大跨域依赖。需确认是否为合法依赖（认知层需要调用行动层工具）还是应抽取 trait 抽象。
- **l5_cognition → l2_perception (1)**: 认知层引用感知层，可能是 ConsciousnessTree 调用感知桥接。
- **l5_cognition → l6_meta (2)**: 认知层引用元认知层，可能为 SEAL pipeline 相关调用。

## 依赖方向分析

```
L6 Meta-Cognition  ←──── L5 Cognition
L5 Cognition       ←──── (无，叶子层)
L4 Emotion         ←──── (无)
L3 Embodiment      ←──── (无)
L2 Perception      ←──── L5 Cognition (1)
L1 Action          ←──── L5 Cognition (4)
```

**结论**: L5 Cognition 是唯一主动跨域的层，依赖方向符合架构设计（上层可调下层）。无逆向依赖（L1-L4 不引用 L5-L6），架构方向合规。

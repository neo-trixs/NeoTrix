# cleanup-244: 跨域引用检查

## 跨域引用

```
l5_cognition→l1_action:        2
l5_cognition→l2_perception:    1
l5_cognition→l6_meta:          2
```

## 分析

- **l5_cognition→l1_action (2)**: 认知层引用行动层，可能的依赖问题
- **l5_cognition→l2_perception (1)**: 认知层引用感知层
- **l5_cognition→l6_meta (2)**: 认知层引用元认知层，符合架构（向下依赖）

## 建议

l5_cognition 向下引用 l1_action/l2_perception 需要审查：
- 检查是否为合理的跨层调用（如调用低层基础设施）
- 检查是否应通过 trait 抽象而非直接引用
- 确认是否违反单向依赖原则（上层可引用下层，下层不可引用上层）

## 依赖流向

```
l6_meta (元认知) → 向下依赖 l5
l5_cognition (认知) → 向下依赖 l1, l2, l6 (向上依赖异常)
l4_emotion (情感) → 无跨域引用
l3_embodiment (具身) → 无跨域引用
l2_perception (感知) → 无跨域引用
l1_action (行动) → 无跨域引用
```

## 结论

当前跨域引用共 5 处，全部来自 l5_cognition。l5 向上引用 l6 是合理的（元认知层是顶层），但向下引用 l1/l2 需要进一步审查确认是否为合法依赖。

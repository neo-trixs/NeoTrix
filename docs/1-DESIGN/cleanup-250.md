## 跨域引用

| 来源层 | 目标层 | 引用次数 |
|--------|--------|----------|
| l5_cognition | l1_action | 2 |
| l5_cognition | l2_perception | 1 |
| l5_cognition | l6_meta | 2 |

---

## 分析

**符合架构规则的引用：**
- `l5_cognition→l6_meta`: Cognition 调用 Meta-Cognition，上层依赖下层 ✅

**需要检查的引用：**
- `l5_cognition→l1_action`: Cognition 直接调用 Action 层（跳过 L2/L3/L4）
- `l5_cognition→l2_perception`: Cognition 直接调用 Perception 层（跳过 L3/L4）

---

## 建议

1. **l5→l6** 是正常的跨层依赖，无需修改
2. **l5→l1** 和 **l5→l2** 需要具体分析是哪些模块在引用，判断是否合理

要深入分析具体的跨层引用文件位置吗？

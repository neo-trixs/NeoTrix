# 跨域引用检查

执行命令:
```bash
grep -rn "use crate::$to" neotrix-core/src/$from --include="*.rs" | grep -v test | grep -v facade
```

## 结果

| From | To | Count |
|------|-----|-------|
| l5_cognition | l1_action | 4 |
| l5_cognition | l2_perception | 1 |
| l5_cognition | l6_meta | 2 |

**总跨域引用**: 7

## 分析

- **l5_cognition** 是最活跃的引用发起方 (7 次)
- l5_cognition → l1_action (4): 认知层引用行动层 — 可能是调用工具/执行器
- l5_cognition → l2_perception (1): 认知层引用感知层
- l5_cognition → l6_meta (2): 认知层引用元认知层

### 其他层级

l1_action, l2_perception, l3_embodiment, l4_emotion, l6_meta 均无跨域引用。

---

生成时间: 2026-09-11

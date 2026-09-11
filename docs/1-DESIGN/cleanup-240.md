# cleanup-240: 跨域引用检查

**日期**: 2026-09-11
**检查命令**:
```bash
for from in l1_action l2_perception l3_embodiment l4_emotion l5_cognition l6_meta; do
  for to in l1_action l2_perception l3_embodiment l4_emotion l5_cognition l6_meta; do
    if [ "$from" != "$to" ]; then
      count=$(grep -rn "use crate::$to" neotrix-core/src/$from --include="*.rs" 2>/dev/null | grep -v test | grep -v facade | wc -l)
      if [ "$count" -gt "0" ]; then
        echo "$from→$to: $count"
      fi
    fi
  done
done
```

## 跨域引用统计

| 起始层 | 目标层 | 引用次数 |
|--------|--------|----------|
| l5_cognition | l1_action | 2 |
| l5_cognition | l2_perception | 1 |
| l5_cognition | l6_meta | 2 |

**总跨域引用**: 5

## 分析

- **l5_cognition** 是主要的跨域引用源（共 5 处）
- **l1_action** 被 l5_cognition 引用 2 次
- **l2_perception** 被 l5_cognition 引用 1 次
- **l6_meta** 被 l5_cognition 引用 2 次

## 建议

l5_cognition 作为认知层引用底层执行(l1)和感知(l2)是合理的架构设计，但引用元认知层(l6)需要注意循环依赖风险。建议进一步分析具体引用位置以确认依赖方向是否符合架构规范。

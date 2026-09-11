# Cleanup #241 — 跨域引用检查

> 检查日期: 2026-09-11

## 检查命令

```bash
echo "=== 跨域引用 ==="
for from in l1_action l2_perception l3_embodiment l4_emotion l5_cognition l6_meta; do
  for to in l1_action l2_perception l3_embodiment l4_emotion l5_cognition l6_meta; do
    if [ "$from" != "$to" ]; then
      count=$(grep -rn "use crate::$to" /Users/neo/Downloads/neotrix/neotrix-core/src/$from --include="*.rs" 2>/dev/null | grep -v test | grep -v facade | wc -l)
      if [ "$count" -gt "0" ]; then
        echo "$from→$to: $count"
      fi
    fi
  done
done
```

## 结果

| 跨域引用 | 数量 | 状态 |
|----------|------|------|
| l5_cognition → l1_action | 2 | ⚠️ 上层→下层 |
| l5_cognition → l2_perception | 1 | ⚠️ 上层→下层 |
| l5_cognition → l6_meta | 2 | ⚠️ 上层→同层 |

## 分析

### 依赖方向问题

按六层架构设计，依赖应遵循 **上层依赖下层**（L6→L5→L4→L3→L2→L1），禁止下层依赖上层。

当前 l5_cognition 反向引用了 l1_action 和 l2_perception，属于 **上层→下层依赖**，需逐条审查：

1. **l5_cognition → l1_action (2处)** — 认知层引用行动层，需确认是否应通过 trait 接口解耦
2. **l5_cognition → l2_perception (1处)** — 认知层引用感知层，同上
3. **l5_cognition → l6_meta (2处)** — 认知层引用元认知层，属于**同层/上层依赖**，不符合分层方向

### 未引用的正常域

l1_action、l2_perception、l3_embodiment、l4_emotion 均无跨域引用，保持干净。

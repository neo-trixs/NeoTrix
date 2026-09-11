# 跨域引用检查报告

生成时间: 2026-09-09

## 检查方法

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

## 结果

| 引用方向 | 引用次数 | 违规说明 |
|---------|---------|---------|
| l5_cognition→l1_action | 4 | 上层引用下层，违反单向依赖 |
| l5_cognition→l2_perception | 1 | 上层引用下层，违反单向依赖 |
| l5_cognition→l6_meta | 2 | 同层/跨层引用，违反分层隔离 |

## 分析

**违规数: 3 条边，共 7 次引用**

### 架构原则

正确依赖方向应为 **下层→上层**（L1→L2→L3→L4→L5→L6），或同层内引用。

### 违规详情

1. **l5_cognition → l1_action (4次)** — 认知层直接引用行动层，跨 4 层
2. **l5_cognition → l2_perception (1次)** — 认知层直接引用感知层，跨 3 层
3. **l5_cognition → l6_meta (2次)** — 认知层引用元认知层，向上跨层

## 建议

1. l5_cognition→l1_action: 通过 trait 抽象或事件总线解耦
2. l5_cognition→l2_perception: 引入 L2→L3→L4→L5 的桥接链路
3. l5_cognition→l6_meta: 元认知层应通过回调/事件向下通知，非直接引用

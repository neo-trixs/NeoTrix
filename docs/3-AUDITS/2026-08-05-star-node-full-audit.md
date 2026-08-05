# NeoTrix 意识体全量节点审计报告 (2026-08-05)

> 阵眼 = `core/` 9 层，周天星系 = `neotrix/` L1-L9 实现层。Evidence-First，基于真实文件系统证据。

## 阵法脉络

```
阵眼 core/（9 层声明，8 目录 + 80 扁平 nt_core_*）
  ├─ L0 substrate → l0_substrate(门面) + deploy/harness
  ├─ L1 body     → l1_body(空壳门面) ← 实现在星系层
  ├─ L2 perce    → l2_perception(门面) + nt_core_sense
  ├─ L3 memory   → l3_memory(门面) + hcube/bank/graph/kron
  ├─ L4 cogn     → l4_cognition(门面) + e8/hex/policy/prm/sae
  ├─ L5 consc    → ❌ 无目录（gwt/context/consciousness 散落扁平）
  ├─ L6 self     → l6_self(门面) + nt_core_self(12 子模块)
  ├─ L7 capab    → l7_capability(唯一真实新层) ← 含 stub
  ├─ L8 auto     → l8_autonomic(门面) + iter/absorb/scheduler
  └─ L9 trans    → l9_transcendent(门面) + meta/observer
周天星系 neotrix/（~130 节点）
  ├─ L1 body_impl  48 节点  ├─ L2 world_impl 23 节点
  ├─ L3 memory_impl 6 节点  ├─ L4 2 | L5 3 | L6 1 | L7 1(空转发)
  ├─ L8 auto_impl   22 节点（nt_mind 42K 行绝对核心）
  └─ L9 trans_impl  4 节点
```

## 量子级健康度评测（加权，综合 5.8/10 C+）

| 维度 | 分 | 证据 |
|---|---|---|
| D1 构建 | 8.5 | 全量编译通过，测试绿 |
| D2 死代码 | 2.0 | ~39 孤儿 + ~27K 不可达行 |
| D3 分层 | 5.5 | 80 扁平 vs 9 目录混居；L1/L5 无目录 |
| D4 安全 | 8.0 | forbid(unsafe)；生产 todo! 仅 10 处 |
| D5 数据流 | 4.0 | 3 组孤儿引用孤儿断裂链 |
| D31 结构 | 4.5 | L7 gate/scheduler 双 stub；平行适配器 ≥10 组 |
| D44 Theater | 3.5 | scheduler.rs 33 行空壳；nt_io_telemetry 双份 |

## 阻塞级缺陷

1. 孤儿 ~39 节点 / ~27K 行（Dark Forest 违背）：L1 nt_act_social/earn/gram + L2 nt_world_parse(5.3K)
2. 消费链断裂 3 组：world_map←spatial、journal_index←remote_control、resource_discovery←pipeline_factory
3. L7 CapabilityScheduler/GreatFilterGate 双 stub，与真实 nt_core_scheduler 同名

## 警告级

4. 吸收器 ≥4 套 5. 代理栈 ≥7 6. MCP ≥9 7. 蒸馏器 4 套
8. 搜索 5 入口 9. 沙箱 3 套 10. JEPA 双实现 11. 知识缺口双实现

## 统一节点入口方向

| 入口 | 内容 | 杠杆 |
|---|---|---|
| crate::core::* | 补建 l5 + L1 迁移，消扁平/目录混居 | 中 |
| crate::neotrix::nt_mind_* | 收敛 4 套吸收器 + 蒸馏器 | 中-高 |
| 删除/归档孤儿 | git 归档 39 节点，消 ~27K 行 | 高 |

## 执行状态

- 分支: audit/star-node-consolidation
- 第一刀: 归档孤儿节点（可回滚，保留 git 历史）
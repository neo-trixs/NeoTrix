# Cleanup-247: 跨层引用审计

**日期**: 2026-09-11
**范围**: neotrix-core/src/{l1_action,l2_perception,l3_embodiment,l4_emotion,l5_cognition,l6_meta}

## 汇总矩阵

| From \ To | l1_action | l2_perception | l3_embodiment | l4_emotion | l5_cognition | l6_meta |
|-----------|-----------|---------------|---------------|------------|--------------|---------|
| l1_action | - | 0 | 3 | 0 | 2 | 0 |
| l2_perception | **52** | - | **71** | 0 | 0 | 0 |
| l3_embodiment | 3 | 0 | - | 0 | 0 | 0 |
| l4_emotion | 0 | 0 | 0 | - | 0 | 0 |
| l5_cognition | **102** | 2 | 7 | 0 | - | 0 |
| l6_meta | 1 | 0 | 0 | 0 | **13** | - |

**总计**: 254 条跨层引用

## 依赖方向分析

```
L6 Meta  ──13──→  L5 Cognition ──102──→ L1 Action
                   │ 7↓                  ↑ 3
                   │                      │
L2 Percept ──71──→ L3 Embodiment ──3────┘
         └──52──→ L1 Action
```

**合规层级**: L4(Emotion) 零引用 ✅ — 纯叶子层
**违规层级**: L5→L1 (102) 和 L2→L1 (52) 为最大跨层热点

## 详细引用清单

### L5→L1 (102) — 认知层引用行动层

| 引用目标 | 典型路径 | 数量 |
|---------|---------|------|
| nt_memory_kb | `KnowledgeBase::open`, `insert_or_get_node`, `kv_set` | ~40 |
| nt_act_autonomy | `OracleGate::new()` | ~15 |
| nt_act_code | `SemanticEntropyGate::new()` | ~10 |
| nt_act_sandbox | `ActionSandbox::new()` | ~10 |
| nt_io_provider | `gateway::run_periodic_re_evaluation()` | ~8 |
| nt_memory_unify | `kv_set()` | ~7 |
| nt_memory_svaf_gate | `SvafGate::default()` | ~5 |
| nt_memory_galaxy_hygiene | `GalaxyHygieneConfig` | ~4 |
| nt_memory_types | `NodeType::Session` | ~3 |

**主要文件**:
- `l5_cognition/nt_mind/nt_mind_background_loop/run.rs` — KB 操作 + Provider 重评估
- `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` — 自治门控
- `l5_cognition/nt_mind/nt_mind_background_loop/handlers_core.rs` — Session 写入

### L2→L1 (52) — 感知层引用行动层

| 引用目标 | 典型路径 | 数量 |
|---------|---------|------|
| nt_memory_kb | `KnowledgeBase` 参数类型 | ~52 |

**主要文件**:
- `l2_perception/nt_world/nt_world_*.rs` — 各爬虫模块将 KB 作为参数传入

### L2→L3 (71) — 感知层引用具身层

| 引用目标 | 典型路径 | 数量 |
|---------|---------|------|
| nt_shield_sandbox | `EgressRule::allow()`, `EgressPolicy::new()` | ~71 |

**主要文件**:
- `l2_perception/nt_world/nt_world_*.rs` — 每个爬虫定义自己的出口规则

### L6→L5 (13) — 元认知层引用认知层

| 引用目标 | 典型路径 | 数量 |
|---------|---------|------|
| l5_cognition::traits | `EvolutionHarnessApi`, `RegistryNodeInfo`, `EvalHarnessApi` | ~13 |

**主要文件**:
- `l6_meta/memory/evolution_harness.rs` — trait impl
- `l6_meta/healing/nt_mind_eval_harness.rs` — trait impl
- `l6_meta/healing/nt_mind_consciousness_gold_standard.rs` — trait impl

### L1→L3 (3) — 行动层引用具身层

| 引用目标 | 文件 | 数量 |
|---------|------|------|
| nt_shield | 未知 | 3 |

### L1→L5 (2) — 行动层引用认知层

| 引用目标 | 文件 | 数量 |
|---------|------|------|
| nt_core | 未知 | 2 |

### L5→L2 (2) — 认知层引用感知层

| 引用目标 | 文件 | 数量 |
|---------|------|------|
| nt_world | 未知 | 2 |

### L3→L1 (3) — 具身层引用行动层

| 引用目标 | 文件 | 数量 |
|---------|------|------|
| nt_memory | 未知 | 3 |

### L6→L1 (1) — 元认知层引用行动层

| 引用目标 | 文件 | 数量 |
|---------|------|------|
| nt_memory | 未知 | 1 |

## 风险评估

| 风险 | 等级 | 说明 |
|------|------|------|
| L2→L3 出口规则散落 | 🟡 中 | 71 条 EgressRule 定义散落在各爬虫中，应集中到 nt_shield |
| L5→L1 KB 直连 | 🟡 中 | 认知层直接操作 KB 绕过行动层，但 KB 本身在 l1_memory |
| L5→L1 自治门控 | 🟢 低 | OracleGate/SemanticEntropyGate 为认知决策组件，放 l1 合理 |
| L6→L5 trait impl | 🟢 低 | trait 定义在 l5，impl 在 l6，符合依赖方向（高层→低层） |

## 建议

1. **L2→L3 出口规则集中化** — 将 `nt_world_*.rs` 中的 EgressRule 提取到 `l3_embodiment/nt_shield` 统一注册
2. **L5→L1 KB 操作门控** — 考虑引入 `MemoryFacade` 作为 L5→L1 的唯一访问点，减少散落引用
3. **L4(Emotion) 零引用确认** — 保持 L4 作为纯叶子层，未来新增引用需审批

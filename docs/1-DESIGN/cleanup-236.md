# cleanup-236 — 跨域引用审计

**日期**: 2026-09-11
**范围**: neotrix-core/src/ 六层架构跨域 `crate::` 引用
**方法**: `grep -rn "crate::$to::" --include="*.rs"` 排除 test/facade/注释

---

## 总览矩阵

```
              L1_ACT  L2_PERC  L3_EMB  L4_FEEL  L5_CO  L6_META
L1_ACTION       -       0        3↓      0       2↑      0
L2_PERCEP      52↑      -       71↑      0       0       0
L3_EMBOD        3↓      0        -       0       0       0
L4_EMOTION      0       0        0       -       0       0
L5_CO         103↓     2↓       7↓      1↓      -      14↓
L6_META         1↓      0        0       0      13↑      -
```

**↑ = 向上引用 (低层→高层), ↓ = 向下引用 (高层→低层)**

## 统计

| 方向 | 引用数 | 严重度 |
|------|--------|--------|
| l5_cognition → l1_action | 103 | 🔴 重度 |
| l2_perception → l3_embodiment | 71 | 🔴 重度 |
| l2_perception → l1_action | 52 | 🟡 中度 |
| l5_cognition → l6_meta | 14 | 🟢 可接受 |
| l6_meta → l5_cognition | 13 | 🟢 可接受 |
| l5_cognition → l3_embodiment | 7 | 🟢 可接受 |
| l1_action → l3_embodiment | 3 | 🟢 可接受 |
| l3_embodiment → l1_action | 3 | 🟢 可接受 |
| l1_action → l5_cognition | 2 | 🟢 可接受 |
| l5_cognition → l2_perception | 2 | 🟢 可接受 |
| l5_cognition → l4_emotion | 1 | 🟢 可接受 |
| l6_meta → l1_action | 1 | 🟢 可接受 |

**总计**: 272 条活跃跨域引用

---

## 🔴 重度问题分析

### 1. l5_cognition → l1_action (103 refs)

**分布**:
- `handlers_consciousness.rs`: 大量引用 `l1_action::nt_memory` 和 `l1_action::nt_act`
- `handlers_core.rs`: `l1_action::nt_memory::nt_memory_kb`
- `run.rs`: `l1_action::nt_io::nt_io_provider`
- `mod.rs`: `l1_action::nt_memory::nt_memory_kb`

**主要引用类型**:
- `nt_memory_kb::KnowledgeBase` — KB 操作 (insert_or_get_node, kv_set 等)
- `nt_act_autonomy::oracle_gate` — Oracle Gate
- `nt_act_code::semantic_entropy` — Semantic Entropy Gate
- `nt_act_sandbox::ActionSandbox` — Action Sandbox
- `nt_io_provider::gateway` — Gateway re-evaluation
- `nt_memory_svaf_gate::SvafGate` — SVAF Gate
- `nt_memory_galaxy_hygiene::GalaxyHygieneConfig` — Galaxy Hygiene

**根因**: L5 认知层直接操作 L1 行动层的基础设施 (KB、Gate、Sandbox)，违反分层原则。认知层应通过 trait 接口访问，不应直接依赖具体实现。

**建议修复**:
1. 在 `l5_cognition/traits.rs` 定义 `KbAccess`, `GateAccess`, `SandboxAccess` trait
2. L1 实现这些 trait，通过 dependency injection 注入 L5
3. 消除所有 `crate::l1_action::` 直接引用

---

### 2. l2_perception → l3_embodiment (71 refs)

**分布**:
- `nt_world_urlhaus.rs`: `nt_shield_sandbox::EgressRule/EgressPolicy`
- `nt_world_usgs.rs`: `nt_shield_sandbox::EgressRule`
- `nt_world_bgpview.rs`: `nt_shield_sandbox::EgressRule`
- `nt_world_gdelt.rs`: `nt_shield_sandbox::EgressRule`

**主要引用类型**:
- `nt_shield_sandbox::EgressRule` — 出站规则定义
- `nt_shield_sandbox::EgressPolicy` — 出站策略组合

**根因**: 感知层爬虫直接创建安全层的出站规则，将策略定义与执行耦合。

**建议修复**:
1. 在 `l2_perception/traits.rs` 定义 `EgressPolicyFactory` trait
2. 爬虫返回 `EgressPolicyDescriptor` (声明式)，由 L3 统一编译执行
3. 或将 EgressRule 定义下沉到 shared crate

---

### 3. l2_perception → l1_action (52 refs)

**分布**:
- `nt_world_urlhaus.rs`: `nt_memory_kb::KnowledgeBase`
- `nt_world_usgs.rs`: `nt_memory_kb::KnowledgeBase`
- `nt_world_bgpview.rs`: `nt_memory_kb::KnowledgeBase`
- `nt_world_gdelt.rs`: `nt_memory_kb::KnowledgeBase`

**主要引用类型**:
- `nt_memory_kb::KnowledgeBase` — KB 写入 (insert_or_get_node)

**根因**: 爬虫直接写入 KB，感知层与存储层耦合。

**建议修复**:
1. 定义 `KnowledgeSink` trait，感知层写入抽象接口
2. L1 实现具体 KB 写入
3. 或通过 EventBus 发布事件，由 L1 消费者写入

---

## 🟢 可接受引用

### l5_cognition ↔ l6_meta (14 + 13 = 27 refs)

**分布**:
- `handlers_consciousness.rs`: `l6_meta::memory::evolution_harness`
- `evolution_harness.rs`: `l5_cognition::traits::EvolutionHarnessApi`

**性质**: L5-L6 共享 trait 定义 (`EvolutionHarnessApi`, `GoldStandardApi`, `EvalHarnessApi`)，通过 trait 抽象实现双向引用，符合架构设计。

### l5_cognition → l3_embodiment (7 refs)

**分布**:
- `handlers_consciousness.rs`: `nt_shield_audit` (审计/监控)
- `nt_mind_skill_engine.rs`: `nt_shield::tool_inspection_stack`

**性质**: 认知层调用安全层的审计/检查功能，属于合理的跨层调用 (安全检查是横切关注点)。

### l1_action ↔ l3_embodiment (3 + 3 = 6 refs)

**分布**:
- `factory.rs`: `nt_shield::policy::PolicyDecision`
- `api_proxy.rs`: `nt_io_provider::types::LlmResponse`

**性质**: 行动层调用安全策略决策 (L1→L3)，安全层代理 IO 类型转换 (L3→L1)，属于接口层合理耦合。

### l1_action → l5_cognition (2 refs)

**分布**:
- `agent.rs`: `SelfIteratingBrain` 类型引用

**性质**: 仅类型引用，用于函数签名，非功能依赖。

---

## l4_emotion 状态

**零跨域引用** — L4 情感层完全隔离，无任何外部依赖。符合设计预期。

---

## 修复优先级

| 优先级 | 引用 | 修复方案 | 预估工作量 |
|--------|------|---------|-----------|
| P0 | l5→l1 (103) | 定义 trait + DI 注入 | 大 |
| P1 | l2→l3 (71) | EgressPolicy 声明化 | 中 |
| P2 | l2→l1 (52) | KnowledgeSink trait | 中 |
| P3 | 其他 (46) | 保持现状/微调 | 小 |

---

## 附录: 按文件分布

### l5_cognition → l1_action 主要文件

| 文件 | 引用数 | 主要引用 |
|------|--------|---------|
| `handlers_consciousness.rs` | ~40 | oracle_gate, semantic_entropy, action_sandbox, svaf_gate, galaxy_hygiene |
| `run.rs` | ~15 | KnowledgeBase, gateway |
| `mod.rs` | ~10 | KnowledgeBase |
| `handlers_core.rs` | ~8 | KnowledgeBase |
| `handlers_daily_intel.rs` | ~5 | kv_set |
| `handlers_maintenance.rs` | ~5 | knowledge_store |
| `knowledge_pipeline.rs` | ~3 | KnowledgeStore |
| `nt_mind_skill_engine.rs` | ~7 | tool_inspection_stack |
| `pipeline.rs` | ~5 | ExplorationEngine |
| `builder.rs` | ~2 | ReasoningBank |
| 其他 | ~3 | 分散引用 |

### l2_perception → l3_embodiment 主要文件

| 文件 | 引用数 | 主要引用 |
|------|--------|---------|
| `nt_world_urlhaus.rs` | ~15 | EgressRule, EgressPolicy |
| `nt_world_usgs.rs` | ~10 | EgressRule |
| `nt_world_bgpview.rs` | ~8 | EgressRule |
| `nt_world_gdelt.rs` | ~10 | EgressRule |
| `nt_world_*.rs` (其他) | ~28 | EgressRule/EgressPolicy |

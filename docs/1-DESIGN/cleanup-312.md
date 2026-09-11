# Cleanup #312 — 跨层引用扫描报告

> 扫描时间: 2026-09-11  
> 范围: `neotrix-core/src/l1_action/` → `l6_meta/`  
> 排除: `*test*`, `*facade*`

## 1. 层级文件统计

| Layer | 文件数 | 说明 |
|-------|--------|------|
| l1_action | 387 | 行动层 (nt_act/nt_io/nt_memory) |
| l2_perception | 212 | 感知层 (nt_world/nt_sense) |
| l3_embodiment | 179 | 具身层 (nt_physical/nt_shield/nt_feel) |
| l4_emotion | 6 | 情感层 (nt_feel core) |
| l5_cognition | 337 | 认知层 (nt_core/nt_mind) |
| l6_meta | 60 | 元认知层 (nt_meta/nt_repair/nt_nexus) |

## 2. 跨层引用矩阵

```
           →L1    →L2    →L3    →L4    →L5    →L6
L1_action    —      0      3      0      2      0
L2_percep   62      —     75      0      0      0
L3_embody    3      0      —      0      0      0
L4_emotion   0      0      0      —      0      0
L5_cogniz  115      3     12      1*     —     21
L6_meta      1      0      0      0     16      —
```

\* L5→L4 为注释行, 不计入

**总跨层引用: 314**

## 3. 违规热力排名

| 排名 | 违规方向 | 数量 | 严重度 | 说明 |
|------|----------|------|--------|------|
| 1 | L5→L1 | 115 | 🔴 | 认知层重度依赖行动层(KB/HTTP/Act) |
| 2 | L2→L3 | 75 | 🟡 | 感知层依赖具身层(Shield Egress) |
| 3 | L2→L1 | 62 | 🟡 | 感知层依赖行动层(KB存储) |
| 4 | L5→L6 | 21 | 🔴 | 认知层向上依赖元认知层(**方向违规**) |
| 5 | L6→L5 | 16 | 🔴 | 元认知层向下依赖认知层(**双向循环**) |
| 6 | L5→L3 | 12 | 🟡 | 认知层依赖具身层(Shield审计) |
| 7 | L3→L1 | 3 | 🟢 | 具身层依赖行动层(IO Provider) |
| 8 | L1→L3 | 3 | 🟢 | 行动层依赖具身层(Shield策略) |
| 9 | L1→L5 | 2 | 🟢 | 行动层依赖认知层(SelfIteratingBrain) |
| 10 | L5→L2 | 3 | 🟢 | 认知层依赖感知层(ExplorationEngine) |
| 11 | L6→L1 | 1 | 🟢 | 元认知层依赖行动层(KB) |

## 4. 关键架构问题

### 4.1 🔴 L5↔L6 双向循环 (37 imports)

L5 (认知) 和 L6 (元认知) 之间存在双向依赖, 违反分层架构的单向依赖原则。

**L5→L6 (21)**:
- `handlers_consciousness.rs` 大量调用 `l6_meta::nt_repair::*`、`l6_meta::memory::evolution_harness`
- `pipeline.rs` 注册 `l6_meta::nt_repair` 的 SelfTest

**L6→L5 (16)**:
- `evolution_harness.rs` 实现 `l5_cognition::traits::EvolutionHarnessApi`
- `nt_mind_consciousness_monitor.rs` 实现 `l5_cognition::traits::ConsciousnessMonitorApi`
- `nt_mind_eval_harness.rs` 实现 `l5_cognition::traits::EvalHarnessApi`

**建议**: L6 trait 定义下沉到 L5 的 `traits.rs` (已有部分), L6 只实现接口; L5→L6 的调用通过 trait object 或 EventBus 解耦。

### 4.2 🔴 L5→L1 重度耦合 (115 imports)

认知层直接引用行动层的具体类型:

| 被引用模块 | 次数 | 主要用途 |
|-----------|------|---------|
| `nt_memory_kb::*` | ~70 | KnowledgeBase/nt_http/nt_memory_store |
| `nt_act::*` | ~25 | oracle_gate/semantic_entropy/sandbox |
| `nt_io::*` | ~20 | provider/session_recovery/user_avatar |

**建议**:
- KB 访问通过 `l5_cognition::kb_facade` 已存在, 但大量代码仍直接引用 `l1_action::nt_memory`
- Act 组件 (oracle_gate 等) 应通过 trait 注入, 而非直接构造

### 4.3 🟡 L2→L1/L3 耦合 (137 imports)

感知层的两个主要耦合:

**L2→L1 (62)**: 所有 World Fetcher 直接引用 `l1_action::nt_memory::nt_memory_kb::KnowledgeBase`
- `nt_world_urlhaus.rs`, `nt_world_usgs.rs`, `nt_world_gdelt.rs` 等 13+ 个 fetcher
- `osint/mod.rs` 引用 `nt_memory_crawl` 和 `nt_memory_store`

**L2→L3 (75)**: 所有 World Fetcher 直接引用 `l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule/EgressPolicy`
- 每个 fetcher 都定义 `*_egress_rule()` 和 `*_egress_policy()` 函数
- 集中在 `nt_world_*.rs` 和 `osint/*.rs`

**建议**:
- KB 引用统一走 facade (已有 `l5_cognition::kb_facade`)
- EgressPolicy 提取为独立 crate 或下沉到 L1 的 shared_types

## 5. 清理优先级

| 优先级 | 任务 | 预估工作量 |
|--------|------|-----------|
| P0 | 拆解 L5↔L6 双向循环 | 中 (trait 下沉 + trait object) |
| P1 | L5→L1 KB引用统一走 facade | 低 (替换 use 路径) |
| P1 | L2→L1 KB引用统一走 facade | 低 (替换 use 路径) |
| P2 | L2→L3 EgressPolicy 抽象 | 中 (独立模块/facade) |
| P2 | L5→L1 Act组件 trait 注入 | 高 (重构构造方式) |
| P3 | L5→L3 Shield审计 facade | 低 |

## 6. 合规层 (无违规)

- **L4 (Emotion)**: 零跨层引用 ✅
- **L3→L2**: 零引用 ✅
- **L1→L2/L4/L6**: 零引用 ✅
- **L6→L2/L3/L4**: 零引用 ✅

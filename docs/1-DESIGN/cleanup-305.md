# Cleanup 305: 跨层引用扫描报告

**日期**: 2026-09-11
**范围**: `neotrix-core/src/` 6层架构 (l1_action → l6_meta)
**排除**: `*_test*`, `facade*`, `#[cfg(test)]` 块内

---

## 架构依赖规则

```
L6 (Meta)  →  L5 (Cognition)  →  L4 (Emotion)  →  L3 (Embodiment)  →  L2 (Perception)  →  L1 (Action)
```

**允许方向**: 高层 → 低层 (L6→L5, L5→L1, 等)
**禁止方向**: 低层 → 高层 (向上依赖)
**特殊规则**: `core::` (nt_core) 属于 L5 Cognition 层，低层引用 `core::` 等同于 L? → L5

---

## Facade 机制 (10 文件)

facade 文件是合法的跨层访问通道，将向上依赖收敛到单一入口：

| Facade 文件 | 所在层 | 访问目标层 |
|---|---|---|
| `l2_perception/nt_world/l1_facade.rs` | L2 | L1 |
| `l3_embodiment/l1_facade.rs` | L3 | L1 |
| `l5_cognition/act_facade.rs` | L5 | L1 |
| `l5_cognition/io_facade.rs` | L5 | L1 |
| `l5_cognition/io_skills_facade.rs` | L5 | L1 |
| `l5_cognition/kb_facade.rs` | L5 | L1 |
| `l5_cognition/l2_facade.rs` | L5 | L2 |
| `l5_cognition/l3_facade.rs` | L5 | L3 |
| `l5_cognition/l6_facade.rs` | L5 | L6 |
| `l6_meta/l1_facade.rs` | L6 | L1 |

---

## 违规清单 (非 facade, 非 test)

### 🔴 P0: 低层 → 高层 (向上依赖)

#### l2_perception → l5_cognition (通过 `core::`)

`core::` = `nt_core` = L5 Cognition 层。L2 大量引用 `core::` 构成向上依赖。

| 文件 | 行号 | import |
|---|---|---|
| `nt_world/nt_world_agent_reach.rs` | 7 | `core::nt_core_self_test::{SelfTest, SelfTestRegistry}` |
| `nt_world/nt_world_monitor.rs` | 7-8 | `core::nt_core_self_test::{SelfTest, SelfTestRegistry}`, `core::nt_core_traits::KnowledgeSink` |
| `nt_world/nt_world_dsh_explore.rs` | 7 | `core::nt_core_self_test::{SelfTest, SelfTestRegistry}` |
| `nt_world/nt_world_semantic_extract.rs` | 374 | `core::nt_core_math::cosine_similarity_f32` |
| `nt_world/nt_world_github_absorber.rs` | 11 | `core::nt_core_kb_types::{NodeType, RelationType}` |
| `nt_world/nt_world_ods.rs` | 7-9 | `core::nt_core_kb_types::NodeType`, `core::nt_core_self_test::*`, `core::nt_core_traits::KnowledgeSink` |
| `nt_world/nt_world_myip.rs` | 11 | `core::nt_core_self_test::SelfTest` |
| `nt_world/nt_world_exploration_engine.rs` | 15-16 | `core::nt_core_kb_types::NodeType`, `core::nt_core_traits::KnowledgeSink` |
| `nt_world/nt_world_code_search.rs` | 6 | `core::nt_core_code_search::*` |
| `nt_world/nt_world_osint_arsenal.rs` | 11 | `core::nt_core_self_test::SelfTest` |
| `nt_world/nt_world_intel_selftest.rs` | 12 | `core::nt_core_self_test::{SelfTest, SelfTestRegistry}` |
| `nt_world/nt_world_video_pipeline.rs` | 1998, 2265 | `core::nt_core_self_test::SelfTest` (test 内) |
| `nt_world/asset_map/asset_map_capability.rs` | 4 | `core::nt_core_capability::*` |
| `nt_world/crawl/mapper.rs` | 3-9 | `core::CapabilityVector`, `core::nt_core_edit::MicroEdit`, `core::nt_core_hcube::*`, `core::nt_core_knowledge::*`, `core::nt_core_bank::*` |
| `nt_world/crawl/unified.rs` | 11-15 | `core::CapabilityVector`, `core::nt_core_bank::ReasoningBank`, `core::nt_core_hcube::*` |
| `nt_world/crawl/asset_graph.rs` | 20 | `core::nt_core_math::normalize_url` |
| `nt_world/crawl/agentic_browse.rs` | 6 | `core::nt_core_self_test::SelfTest` |
| `nt_world/osint/mod.rs` | 33 | `core::nt_core_kb_types::NodeType` |
| `nt_world/osint/sweep.rs` | 6 | `core::nt_core_self_test::SelfTest` |
| `nt_world/osint/metadata.rs` | 8 | `core::nt_core_self_test::SelfTest` |
| `nt_world/osint/repo_reverse_prompt.rs` | 6 | `core::nt_core_self_test::SelfTest` |
| `nt_world/sense/nt_world_model_types.rs` | 3, 10 | `core::nt_core_math::cosine_similarity_f64`, `core::nt_core_knowledge::TaskType` |
| `nt_world/sense/nt_world_sense_hub.rs` | 1 | `core::nt_core_sense::*` |
| `nt_world/sense/mod.rs` | 57 | `core::nt_core_hcube::cube::KnowledgeHyperCube` |
| `nt_world/sense/visual_cortex.rs` | 2 | `core::nt_core_sense::*` |
| `nt_world/sense/auditory_cortex.rs` | 2 | `core::nt_core_sense::*` |
| `nt_world/sense/omniscient_view.rs` | 1 | `core::nt_core_sense::*` |
| `nt_world/sense/loss.rs` | 4 | `core::nt_core_math::cosine_similarity_f64` |
| `nt_world/sense/world_model.rs` | 3 | `core::nt_core_td::*` |
| `nt_world/sense/world_consciousness.rs` | 1 | `core::nt_core_sense::*` |
| `nt_world/sense/real_sensors/screen.rs` | 4 | `core::nt_core_sense::{Sensor, SensorSample, ...}` |
| `nt_world/sense/real_sensors/mic.rs` | 4 | `core::nt_core_sense::{Sensor, SensorSample, ...}` |
| `traits.rs` | 95 | `core::nt_core_traits::KnowledgeSink` |

**统计**: ~30+ 处 L2 → L5(`core::`) 引用

#### l3_embodiment → l5_cognition (通过 `core::`)

| 文件 | 行号 | import |
|---|---|---|
| `nt_shield/nt_shield_impl/mod.rs` | 18-19 | `core::nt_core_gwt::GWTContext`, `core::nt_core_e8::E8` |
| `nt_shield/nt_shield_impl/nt_shield_pentest_agent.rs` | 14 | `core::nt_core_gwt::GWTContext` |
| `nt_shield/nt_shield_core_traits_impl.rs` | 6 | `core::nt_core_traits::*` |
| `nt_shield/nt_shield_oversight.rs` | 20 | `core::nt_core_self_test::{SelfTest, SelfTestRegistry}` |
| `nt_shield/nt_shield/policy.rs` | 3 | `core::nt_core_self_test::SelfTest` |
| `nt_shield/nt_shield/permissions.rs` | 5 | `core::nt_core_self_test::SelfTest` |
| `nt_shield/nt_shield/guard.rs` | 15 | `core::nt_core_self_test::SelfTest` |
| `nt_shield/nt_shield/safety_kernel.rs` | 11 | `core::nt_core_self_test::SelfTest` |
| `nt_shield/nt_shield/perm_chain.rs` | 6 | `core::nt_core_self_test::SelfTest` |
| `nt_shield/nt_shield/nt_shield_skill_router.rs` | 5 | `core::nt_core_self_test::{SelfTest, SelfTestRegistry}` |
| `nt_shield/nt_shield/check_registry.rs` | 792 | `core::nt_core_self_test::SelfTest` |
| `nt_shield/nt_shield/mod.rs` | 7 | `core::nt_core_guard_chain` |
| `nt_shield/nt_shield_stealth_net/proxy_pool.rs` | 13 | `core::nt_core_resource_pool::*` |
| `nt_shield/nt_shield_stealth_net/network_pool.rs` | 24 | `core::nt_core_resource_pool::*` |
| `nt_shield/nt_shield_stealth_net/pool_types.rs` | 3 | `core::nt_core_resource_pool::PooledResource` |
| `nt_shield/nt_shield_stealth_net/geo_proxy.rs` | 33,213,221 | `core::nt_core_di` |
| `nt_shield/nt_shield_stealth_net/config.rs` | 20 | `core::nt_core_di` |
| `nt_shield/nt_shield_stealth_net/self_iterating.rs` | 340 | `core::nt_core_knowledge::TaskType` |
| `nt_shield/nt_shield_sandbox/device.rs` | 166 | `core::nt_core_self_test::SelfTest` |
| `nt_shield/nt_shield_sandbox/judge.rs` | 172,476,503 | `core::nt_core_telemetry::*` |
| `nt_shield/nt_shield_sandbox/stateful_bench.rs` | 11 | `core::nt_core_self_test::SelfTest` |
| `nt_shield/nt_shield_traffic/analyzer.rs` | 5 | `core::nt_core_self_test::SelfTest` |
| `nt_shield/nt_shield_traffic/fingerprint.rs` | 165 | `core::nt_core_self_test::SelfTest` |
| `shield_capability.rs` | 6 | `core::nt_core_capability::*` |

**统计**: ~24 处 L3 → L5(`core::`) 引用

#### l4_emotion → l5_cognition (通过 `core::`)

| 文件 | 行号 | import |
|---|---|---|
| `nt_feel/nt_feel/emotion_engine.rs` | 8 | `core::nt_core_self::emotion_state::*` |

**统计**: 1 处

### 🟡 P1: 高层 → 低层 (合法但绕过 facade)

#### l5_cognition → l1_action (直接引用，绕过 facade)

| 文件 | 行号 | import | 是否有对应 facade |
|---|---|---|---|
| `nt_mind/evolution/self_diagnose.rs` | 10 | `l1_action::nt_act::nt_act_types::ProjectSnapshot` | ❌ 无 facade |
| `nt_mind/evolution/evolution_loop.rs` | 67 | `l1_action::nt_act::nt_act_types::ProjectSnapshot` | ❌ 无 facade |
| `nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 679-680 | `l1_action::nt_memory::nt_memory_kb::nt_memory_community::*`, `nt_memory_store::*` | ✅ kb_facade 已覆盖 |

**统计**: 3 处直接引用 (2 处缺 facade)

#### l5_cognition → l6_meta (合法方向，但绕过 facade)

| 文件 | 行号 | import |
|---|---|---|
| `nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 940 | `l6_meta::coordination::self_improvement::SystemMetrics` |

**统计**: 1 处

### 🟢 P2: 合规 (通过 facade)

| 路径 | Facade | 状态 |
|---|---|---|
| L2 → L1 | `l2_perception/nt_world/l1_facade.rs` | ✅ |
| L3 → L1 | `l3_embodiment/l1_facade.rs` | ✅ |
| L5 → L1 | `l5_cognition/{act,io,io_skills,kb}_facade.rs` | ✅ |
| L5 → L2 | `l5_cognition/l2_facade.rs` | ✅ |
| L5 → L3 | `l5_cognition/l3_facade.rs` | ✅ |
| L5 → L6 | `l5_cognition/l6_facade.rs` | ✅ |
| L6 → L1 | `l6_meta/l1_facade.rs` | ✅ |

---

## 依赖矩阵 (热力图)

```
目标↓ \ 源→  L1     L2     L3     L4     L5     L6
L1            -      -      -      -      -      -
L2            ✅     -      -      -      -      -
L3            ✅     -      -      -      -      -
L4            -      -      -      -      -      -
L5            ✅     🔴     🔴     🟡     -      -
L6            -      -      -      -      -      -

✅ = facade 合规    🔴 = core:: 向上依赖    🟡 = 1处 direct
```

---

## 修复建议

### P0 修复 (l2/l3/l4 → core:: 向上依赖)

**根因**: `core::` (nt_core) 本应是 L5 层的基础，但被低层大量直接引用。

**方案 A (推荐): 将 SelfTest/KnowledgeSink 等下沉到独立 trait crate**

```
neotrix-traits/ (新 crate)
├── self_test.rs      ← SelfTest, SelfTestRegistry
├── knowledge.rs      ← KnowledgeSink, NodeType
├── math.rs           ← cosine_similarity, normalize_url
└── sense.rs          ← Sensor, SensoryEvent
```

低层 crate 依赖 `neotrix-traits`，不依赖 `neotrix-core`。

**方案 B: 创建 `core_facade` 模块**

在 `neotrix-core/src/core/` 下创建 facade 层，仅暴露低层需要的类型。

**方案 C: 渐进迁移 (最小改动)**

将 `SelfTest` 相关引用迁移到 facade 或独立 crate，保留其他 `core::` 引用作为技术债。

### P1 修复 (l5 直接引用 l1)

| 引用 | 建议 |
|---|---|
| `ProjectSnapshot` | 添加到 `act_facade.rs` 或创建 `types_facade.rs` |
| `handlers_maintenance.rs` 内联引用 | 已有 `kb_facade.rs`，改为 `use super::super::kb_facade::*` |

---

## 统计

| 类别 | 数量 |
|---|---|
| Facade 文件 | 10 |
| L2 → L5(`core::`) 违规 | ~30 |
| L3 → L5(`core::`) 违规 | ~24 |
| L4 → L5(`core::`) 违规 | 1 |
| L5 → L1 直接引用 | 3 |
| L5 → L6 直接引用 | 1 |
| **总违规** | **~59** |
| Facade 合规引用 | ~60+ |

---

## 优先级排序

1. **P0-1**: 创建 `neotrix-traits` crate 下沉 `SelfTest` (影响 L2 + L3 ~54 处)
2. **P0-2**: 将 `KnowledgeSink`/`NodeType` 下沉到 traits crate (影响 L2 ~8 处)
3. **P0-3**: 将 `nt_core_math` 函数下沉到 traits crate (影响 L2 ~4 处)
4. **P1-1**: `ProjectSnapshot` 添加到 `act_facade.rs` (2 处)
5. **P1-2**: `SystemMetrics` 添加到 `l6_facade.rs` (1 处)

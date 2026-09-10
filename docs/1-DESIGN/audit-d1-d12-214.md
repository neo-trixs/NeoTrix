# NeoTrix 架构审计报告 — D1-D12

**审计时间**: 2026-09-10
**审计范围**: neotrix-core/src (全部源码)
**审计方法**: 静态代码扫描 + 模式分析

---

## D1: 模块健康度

**24 个 nt_core_* 模块扫描结果：**

| 模块 | 代码行 | 函数数 | TODO/FIXME | 问题比率 | 状态 |
|------|--------|--------|------------|----------|------|
| nt_core_e8 | 13,557 | 21 | 0 | 0% | ✅ 健康 |
| nt_core_self | 12,344 | 28 | 0 | 0% | ✅ 健康 |
| nt_core_hcube | 8,820 | 49 | 0 | 0% | ✅ 健康 |
| nt_core_gwt | 8,023 | 67 | 0 | 0% | ✅ 健康 |
| nt_core_capability | 6,345 | 24 | 0 | 0% | ✅ 健康 |
| nt_core_consciousness | 3,739 | 31 | 0 | 0% | ✅ 健康 |
| nt_core_meta | 3,970 | 23 | 5 | 0.12% | ⚠️ 低风险 |
| nt_core_consciousness_tree | 3,141 | 40 | 0 | 0% | ✅ 健康 |
| nt_core_gate | 3,106 | 57 | 0 | 0% | ✅ 健康 |
| nt_core_self_review | 2,855 | 28 | 0 | 0% | ✅ 健康 |
| nt_core_prm | 3,714 | 5 | 0 | 0% | ✅ 健康 |
| nt_core_scheduler | 1,854 | 1 | 0 | 0% | ⚠️ 仅 1 个公开函数 |
| nt_core_knowledge | 1,653 | 33 | 0 | 0% | ✅ 健康 |
| nt_core_bank | 1,376 | 18 | 0 | 0% | ✅ 健康 |
| nt_core_vector_store | 1,265 | 4 | 0 | 0% | ⚠️ 仅 4 个公开函数 |
| nt_core_context | 1,205 | 31 | 0 | 0% | ✅ 健康 |
| nt_core_sense | 1,110 | 40 | 4 | 0.36% | ⚠️ 低风险 |
| nt_core_aura | 948 | 13 | 0 | 0% | ✅ 健康 |
| nt_core_resource_pool | 779 | 1 | 0 | 0% | ⚠️ 仅 1 个公开函数 |
| nt_core_plan | 681 | 0 | 0 | 0% | ⚠️ 无公开函数 |
| nt_core_aware | 639 | 0 | 0 | 0% | ⚠️ 无公开函数 |
| nt_core_absorb | 450 | 21 | 0 | 0% | ✅ 健康 |
| nt_core_iter | 363 | 17 | 0 | 0% | ✅ 健康 |
| nt_core_data_pipeline | 323 | 10 | 0 | 0% | ✅ 健康 |

**D1 结论**: 24/24 模块 TODO 比率 ≤0.36%，整体健康。4 个模块公开函数 ≤1 (nt_core_plan/nt_core_aware/nt_core_resource_pool/nt_core_scheduler) 需检查是否为空壳。

---

## D2: 错误处理

**总览**:
- `unwrap()`/`expect()`: **4,944** 处
- `panic!`/`unimplemented!`/`todo!`: **113** 处

**按层分布 (unwrap/expect)**:

| 层 | unwrap/expect | 占比 | panic!/todo! |
|----|--------------|------|-------------|
| L1 Action | 1,898 | 38.4% | 13 |
| L5 Cognition | 808 | 16.3% | 29 |
| core | 842 | 17.0% | 33 |
| neotrix | 537 | 10.9% | 10 |
| L3 Embodiment | 314 | 6.3% | 16 |
| L2 Perception | 228 | 4.6% | 6 |
| cli | 131 | 2.6% | 2 |
| bin | 112 | 2.3% | 0 |
| L6 Meta | 51 | 1.0% | 3 |
| L4 Emotion | 3 | 0.1% | 0 |

**D2 结论**: L1 Action 层 unwrap 密度最高 (1,898 处, 38.4%)，为最大风险区域。core + L5 合计 33.3%，panic 分布集中。4,944 处 unwrap 建议逐步替换为 `Result<T, NtError>` 传播。

---

## D3: EventBus 覆盖

**使用 `CoreEvent::` 的文件 (7 个)**:

| 文件 | 用途 |
|------|------|
| `core/nt_core_event.rs` | 事件定义 |
| `neotrix/nt_core_event_bus.rs` | EventBus 实现 |
| `l3_embodiment/nt_shield/mod.rs` | 安全事件 |
| `l5_cognition/nt_mind/.../handlers_consciousness.rs` | 意识循环处理 |
| `l5_cognition/nt_mind/.../handlers_game.rs` | 游戏化处理 |
| `l5_cognition/nt_mind/.../handlers_maintenance.rs` | 维护处理 |
| `l5_cognition/nt_mind/.../run.rs` | 后台循环 |

**D3 结论**: EventBus 仅在 7 个文件中使用，覆盖极低。L1/L2/L4 层几乎未接入事件总线，事件驱动架构尚未成熟。

---

## D4: KB 覆盖

**使用 kv_store/kv_get/kv_set 的文件 (20 个)**:

主要消费者: CLI commands (kanban/kb/doctor)、consciousness core、self 模块、gate、llm、knowledge、forecast。

**D4 结论**: KB 使用集中在 CLI 和核心推理路径，L1-L4 层直接使用较少，通过上层封装间接依赖。

---

## D5: Layer Traits

| 层 | traits.rs | Trait 定义 | 实现数 |
|----|-----------|-----------|--------|
| L0 Kernel | ❌ 无 | - | - |
| L1 Action | ✅ | `L1Capability`, `MessagingProvider` | 3 |
| L2 Perception | ✅ | `PerceptionLayer` | 6 |
| L3 Embodiment | ✅ | `EmbodimentLayer` | 7 |
| L4 Emotion | ✅ | `EmotionLayer` | 9 |
| L5 Cognition | ✅ | `CognitionLayer` | 9 |
| L6 Meta | ✅ | `MetaLayer` | 8 |

**D5 结论**: 6 层中 5 层有 traits.rs。L0 Kernel 缺失 trait 契约。各层 trait 设计完善，统一使用 `initialize()` + `snapshot()` 模式。

---

## D6: Async 分布

| 层 | async fn 数量 |
|----|-------------|
| L3 Embodiment | 349 |
| L1 Action | 202 |
| L2 Perception | 64 |
| L5 Cognition | 75 |
| L6 Meta | 18 |
| L4 Emotion | 0 |

**D6 结论**: L3 Embodiment 异步最多 (349)，L4 Emotion 完全同步 (0 async)。L4 可能需要评估是否需要异步化以处理高吞吐情感信号。

---

## D7: Unsafe 使用

**总 unsafe 出现次数**: 77 处

**Top 10 位置**:

| 文件 | unsafe 数 |
|------|----------|
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 16 |
| `l1_action/nt_act/nt_act_code/code_writer.rs` | 5 |
| `neotrix/nt_consciousness_core/constitution.rs` | 4 |
| `l5_cognition/nt_mind/nt_mind/infrastructure/code_review.rs` | 4 |
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | 4 |
| `l3_embodiment/nt_shield/nt_shield/audit.rs` | 4 |
| `cli/laws.rs` | 4 |
| `l6_meta/coordination/governance.rs` | 3 |
| `l5_cognition/nt_mind/seal_core/constitutional_stage.rs` | 3 |
| `l1_action/nt_act/goal_generator.rs` | 3 |

**D7 结论**: 77 处 unsafe 违反 R-P1 (`#![forbid(unsafe_code)]`)。evolution_loop.rs (16 处) 为最大违规源。需逐个审查并消除。

---

## D8: Top 依赖

| 依赖 | 引用次数 | 类型 |
|------|---------|------|
| `serde::{Deserialize, Serialize}` | 540 | 序列化 |
| `std::collections::HashMap` | 504 | 数据结构 |
| `std::sync::Arc` | 151 | 并发 |
| `std::path::PathBuf` | 86 | IO |
| `tokio::sync::RwLock` | 66 | 异步并发 |
| `std::collections::VecDeque` | 61 | 数据结构 |
| `std::time::Instant` | 56 | 计时 |
| `std::time::{SystemTime, UNIX_EPOCH}` | 53 | 时间 |
| `rusqlite::Connection` | 53 | 数据库 |
| `std::time::Duration` | 46 | 计时 |

**D8 结论**: 依赖健康。serde (540) 和 HashMap (504) 为最高频依赖。rusqlite (53) 直接使用较多，建议统一通过 KB 层封装。

---

## D9: Thin Modules

| 指标 | 数值 |
|------|------|
| 总 mod.rs 文件 | 244 |
| Thin (<15 行) | 89 |
| Thin 比率 | **36.3%** |

**典型 Thin Modules**:
- `nt_world/mod.rs`: 1 行
- `l4_emotion/mod.rs`: 2 行
- `l5_cognition/mod.rs`: 3 行
- `video/mod.rs`: 3 行
- `plugin/mod.rs`: 2 行

**D9 结论**: 36.3% 的 mod.rs 为 thin module (<15 行)，仅为 re-export 集合。虽属正常，但部分可能为空壳，需结合 D1 公开函数数检查。

---

## D10: 重复模式

**`pub fn new()` 统计**: **947** 处

**Top 目录**:

| 目录 | new() 数量 |
|------|----------|
| `l5_cognition/nt_mind/seal_core/self_iterating` | 68 |
| `core` | 55 |
| `l2_perception/nt_world` | 44 |
| `neotrix/nt_consciousness_core` | 28 |
| `l3_embodiment/nt_shield/stealth_net` | 25 |
| `l3_embodiment/nt_shield` | 23 |
| `core/nt_core_self` | 23 |
| `l1_action/nt_act/actions` | 21 |
| `l1_action/nt_io` | 19 |
| `l1_action/nt_act/crypto` | 18 |

**D10 结论**: 947 个 `pub fn new()` 分布合理，无异常集中。SEAL self_iterating (68) 和 core (55) 为主要构造器密集区。

---

## D11: Integration Gaps

| 指标 | 数值 |
|------|------|
| 使用 EventBus 的文件 | 16 |
| 总 .rs 文件数 | 1,889 |
| EventBus 覆盖率 | **0.8%** |

**D11 结论**: EventBus 覆盖率仅 0.8%，绝大多数模块未接入事件系统。事件驱动架构为未来目标，当前为直接调用模式。

---

## D12: Technical Debt

**TODO/FIXME 总数**: **279** 处

**Top 10 热点文件**:

| 文件 | TODO 数 |
|------|--------|
| `l5_cognition/nt_mind/evolution/autofixer.rs` | 25 |
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | 19 |
| `cli/commands/kanban_cmds.rs` | 19 |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 18 |
| `l1_action/nt_io/nt_io_output_style.rs` | 7 |
| `l1_action/nt_act/trade/orchestrator.rs` | 7 |
| `l1_action/nt_io/consistency_adapter.rs` | 6 |
| `l4_emotion/nt_feel/vtuber.rs` | 5 |
| `l3_embodiment/nt_physical/video_post_processor.rs` | 5 |
| `l1_action/nt_act/checkpoint_persistence.rs` | 5 |

**D12 结论**: 279 处 TODO 集中在 L5 evolution (62 处, 22.2%) 和 CLI (19 处)。autofixer/self_diagnose/evolution_loop 为最大债务热点。

---

## 综合评估

### 健康度评分

| 维度 | 评分 | 说明 |
|------|------|------|
| D1 模块健康 | ⭐⭐⭐⭐⭐ | 24/24 模块 TODO ≤0.36% |
| D2 错误处理 | ⭐⭐ | 4,944 unwrap + 113 panic |
| D3 EventBus | ⭐⭐ | 覆盖率 0.8%，事件驱动未成熟 |
| D4 KB 覆盖 | ⭐⭐⭐ | 核心路径覆盖，L1-L4 间接依赖 |
| D5 Layer Traits | ⭐⭐⭐⭐ | 5/6 层有 trait 契约 |
| D6 Async | ⭐⭐⭐⭐ | 分布合理，L4 同步可接受 |
| D7 Unsafe | ⭐ | 77 处 unsafe 违反 R-P1 |
| D8 依赖 | ⭐⭐⭐⭐⭐ | 依赖健康，无异常 |
| D9 Thin Modules | ⭐⭐⭐ | 36.3% thin，属正常范围 |
| D10 重复 | ⭐⭐⭐⭐ | 947 new() 分布合理 |
| D11 Integration | ⭐⭐ | EventBus 0.8% 覆盖率 |
| D12 Tech Debt | ⭐⭐⭐ | 279 TODO，L5 evolution 为热点 |

### Top 3 行动项

1. **P0 — unsafe 消除**: 77 处 unsafe 违反 R-P1，evolution_loop.rs (16) 为首要目标
2. **P1 — unwrap 替换**: 4,944 处 unwrap，L1 Action 层 (1,898) 优先治理
3. **P1 — TODO 清理**: 279 处 TODO，L5 evolution (62 处) 优先收敛

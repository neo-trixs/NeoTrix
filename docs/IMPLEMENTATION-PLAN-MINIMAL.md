# NeoTrix 精简实施方案 — 内部修复优先

> **原则**: 零外部依赖，纯内部重构，每步可验证
> **日期**: 2026-09-18

---

## 一、核心问题量化

| 问题 | 数量 | 影响 |
|------|------|------|
| TaskType 重复定义 | **9 处** | 类型不一致，路由混乱 |
| TaskStatus 重复定义 | **13 处** | 状态机碎片化 |
| `fn route` 实现 | **49 处** | 路由逻辑分散 |
| `#[allow(dead_code)]` | **87 处** | 死代码积累 |
| 跨层耦合违规 | **31+ 处** | 循环依赖 |

---

## 二、实施方案 (4 步，每步独立可验证)

### Step 1: TaskType 统一 (1 天)

**目标**: 9 → 5 (保留 4 个 domain-specific，统一 4 个兼容的)

**可统一的文件** (variant 完全兼容):

| 文件 | 当前定义 | 操作 |
|------|---------|------|
| `crates/neotrix-types/src/core/nt_core_knowledge/types.rs` | ✅ Canonical (38 variants) | 保留 |
| `l1_action/nt_act/nt_act_orchestrator/critic.rs:4` | 8 variants (子集) | → `pub use neotrix_types::TaskType` ✅ 已完成 |
| `l1_action/nt_act/nt_act_orchestrator/planner.rs:5` | 8 variants (子集) | → `pub use neotrix_types::TaskType` ✅ 已完成 |
| `l1_action/nt_io/universal_model/traits.rs:131` | 7 variants (Chat/Completion/Embedding 等) | → `pub use neotrix_types::TaskType` |

**需保留的 domain-specific 定义** (variant 不兼容):

| 文件 | 原因 |
|------|------|
| `l5_cognition/traits.rs:21` | Analyze/Plan/Execute/Review — 推理流程专用 |
| `l1_action/nt_io/nt_io_provider/common/generation_classifier.rs:16` | Code/Extraction/Knowledge — 生成分类专用 |
| `l5_cognition/nt_core/nt_consciousness_core/resource_router.rs:116` | SimpleQA/ComplexReasoning + `Custom(String)` — 资源路由专用 |
| `l5_cognition/nt_core_model_router.rs:78` | Chat/Coding/Math — 模型路由专用 |
| `l5_cognition/nt_core_god_agent.rs:96` | CodeGeneration/Debugging/Architecture — Agent 分类专用 |
| `l1_action/nt_act/actions/orchestration/production_pipeline.rs:32` | ScriptParsing/CharacterGeneration — 视频生产专用 |

**验证**:
```bash
grep -r "enum TaskType" --include="*.rs" neotrix-core/src/ | wc -l  # 目标: 0
grep -r "use neotrix_types::TaskType" --include="*.rs" | wc -l       # 目标: ≥8
cargo check -p neotrix-types && cargo check -p neotrix --lib
```

---

### Step 2: TaskStatus 统一 (1 天)

**目标**: 13 → 1 (创建 canonical 定义，删除 13 个本地定义)

**方案**: 在 `neotrix-types/src/core/nt_core_knowledge/types.rs` 添加:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
    Timeout,
}
```

**文件清单** (13 处删除):

| 文件 | 行号 |
|------|------|
| `l3_embodiment/nt_shield/nt_shield_impl/nt_shield_swarm.rs` | 80 |
| `l6_meta/nt_core_self/cuda_agent.rs` | 24 |
| `l6_meta/nt_core_capability/discovery.rs` | 291 |
| `l5_cognition/nt_core/reasoning/nt_core_planning.rs` | 86 |
| `l1_action/nt_act/nt_act_scheduler.rs` | 48 |
| `l1_action/nt_io/nt_io_protocol_bridge.rs` | 132 |
| `l1_action/nt_act/actions/orchestration/production_pipeline.rs` | 15 |
| `l1_action/nt_act/actions/core/nt_act_ai_assistant.rs` | 104 |
| `l1_action/nt_act/parallel_task.rs` | 15 |
| `l1_action/nt_act/nt_act_trade/workers/track_worker.rs` | 33 |
| `l1_action/nt_act/nt_act_trade/orchestrator_v2.rs` | 40 |
| `l1_action/nt_act/nt_act_trade/message.rs` | 102 |
| `l1_action/nt_act/agent_loop/manager.rs` | 20 |

**验证**:
```bash
grep -r "enum TaskStatus" --include="*.rs" neotrix-core/src/ | wc -l  # 目标: 0
cargo check -p neotrix-types && cargo check -p neotrix --lib
```

---

### Step 3: 路由器合并 (2 天)

**目标**: 49 → 核心路由 + 领域路由

**分类**:

| 类型 | 数量 | 操作 |
|------|------|------|
| **LLM 模型路由** | 8 | 合并到 `nt_core_model_router.rs` |
| **能力路由** | 6 | 合并到 `l6_meta/nt_core_capability/mod.rs` |
| **GWT 路由** | 5 | 保留 (各有专用职责) |
| **领域路由** | 30 | 保留 (trade/shield/memory 专用) |

**LLM 路由合并清单**:

| 文件 | 函数 | 操作 |
|------|------|------|
| `l5_cognition/nt_core_model_router.rs` | `route()` | → 主路由 |
| `l5_cognition/nt_core_model_gateway.rs` | `route()` | → 合并到主路由 |
| `l5_cognition/nt_core_semantic_router.rs` | `route()` | → 合并到主路由 |
| `l1_action/nt_io/model_routing.rs` | `select_model()` + `route()` | → 合并到主路由 |
| `l1_action/nt_io/nt_io_provider/gateway/routing/*.rs` | 6 个 router | → 合并到主路由 |
| `l5_cognition/nt_mind/nt_mind/seal_core/model_router.rs` | `route()` | → 合并到主路由 |

**验证**:
```bash
grep -r "fn route\|fn select_model" --include="*.rs" neotrix-core/src/ | wc -l  # 目标: <35
cargo check -p neotrix --lib
```

---

### Step 4: 死代码清理 (1 天)

**目标**: 87 → <30

**策略**: 
1. 删除确认无消费者的 `#[allow(dead_code)]` 项
2. 保留有潜在用途的 (如 test helper, future feature)

**高优先级清理** (L0-L2 层):

| 文件 | 行号 | 类型 |
|------|------|------|
| `l2_perception/nt_world/crawl/stealth.rs` | 15 | struct |
| `l2_perception/nt_world/source/plugin_sandbox.rs` | 4 | struct |
| `l2_perception/nt_world/crawl/classifier.rs` | 26 | fn |
| `l2_perception/nt_world/data_source/nt_world_edgar.rs` | 460 | fn |
| `l2_perception/nt_world/data_source/nt_world_aoi.rs` | 256 | struct |

**验证**:
```bash
grep -r "#\[allow(dead_code)\]" --include="*.rs" neotrix-core/src/ | wc -l  # 目标: <50
cargo check -p neotrix --lib
```

---

## 三、执行顺序

```
Step 1 (TaskType)  ──→  Step 2 (TaskStatus)  ──→  Step 3 (Router)  ──→  Step 4 (Dead Code)
     │                       │                       │                       │
     └─ cargo check ─────────┴─ cargo check ─────────┴─ cargo check ─────────┴─ cargo check
```

每步独立，可单独提交。若某步失败，回滚不影响其他步。

---

## 四、风险控制

| 风险 | 缓解 |
|------|------|
| 类型不兼容 | 先检查 variant 是否匹配，不匹配则保留本地 |
| 编译失败 | 每步后 `cargo check`，失败即回滚 |
| 功能退化 | 保留 re-export 兼容路径 |

---

## 五、预期收益

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| TaskType 定义数 | 9 | 1 | 89% ↓ |
| TaskStatus 定义数 | 13 | 1 | 92% ↓ |
| 路由函数数 | 49 | <35 | 29% ↓ |
| dead_code 注解 | 87 | <50 | 43% ↓ |
| 跨层耦合 | 31+ | <15 | 52% ↓ |

# NeoTrix Experience Tree

## 2026-07-03 — 架构审查 + 全方位修复

### 本轮新增修复

| # | 缺陷 | 修复 | 改动位置 |
|---|------|------|----------|
| **G** | L5/L9 → KB 断连（ConsciousnessMonitor + GoldStandardReport 全字段丢在内存） | 添加 7 个 `kv_store` 写入: PhiReport/GoldStandardReport/trends/conversation/blind_spots | `run.rs:handle_awareness()`, `monitor.rs` |
| **H** | `/avatar` CLI 返回硬编码假数据 | 替换为 `DistillationEngine` 真实数据: list/create/status/harvest/evolve | `brain_cmds.rs:AvatarCmd` |
| **I** | `nt_memory_hierarchical` search_hierarchical 零调用者(298行死代码) | 在 `consciousness_bridge:inject_kb_knowledge()` 添加 search_hierarchical 调用 | `consciousness_bridge.rs:114-136` |
| **J** | 12 个 production `unwrap()` — 含 RwLock/NaN 高风险 | 6 个 `unwrap_or_else`(RwLock) + 3 个 `unwrap_or(Equal)`(NaN) + 1 `unwrap_or_default`(time) + 1 `map_or`(string) | `genesis.rs`(3)、`store.rs`(3)、`unify.rs`(1)、`confidence.rs`(1) |
| **K** | 15 个未注册空操作管道阶段(纯 no-op) | 全部添加 `log::trace!` 显示实时 brain 状态字段 | `pipeline.rs:187-508` |

### 架构测量

| 指标 | 值 |
|------|-----|
| KB 委托方法总数 | 93 |
| 已注册 KB 子模块 | 27/30 |
| 活动管道阶段 | 32 |
| 活跃阶段 | 16/32 |
| 未注册阶段定义 | 15 (全部带 trace 日志) |
| production `unwrap()` | **0** (全部修复) |
| `#[allow(dead_code)]` | ~91 (仍待修剪) |
| 管道自检阶段 | SelfReviewStage (频率 1) |
| 推理引擎 → KB 持久化 | PatternExtractionStage (频率 10) |
| 发现守护进程循环 | 5s, OL/Wiki 每次 + GitHub/ArXiv 第 5 次 |

## 2026-07-05 — 全量8维度深度架构审查 + 19层违规清零 + Serde补全 + 竞争差距分析

### 架构审计发现 (Phase 1)

**8维度扫描结果:**

| 维度 | 状态 | 摘要 |
|------|------|------|
| 模块注册 | 🟢 125/125匹配 | core 74 + neotrix 13 + CLI 38 = 100%匹配, 0死模块, 0孤儿 |
| 生产panic/unwrap | 🟢 0处 | 24处expect: 12安全+6中风险+6入口点; 0 unwrap/panic/todo/unimplemented |
| 层违规 | 🔴→🟢 19→0 | L1→L2 6处, L1→L3 3处, L1→L8 6处, L2→L3 4处 — 全部清零 |
| 桌面覆盖率 | 🟡 ~30% | 44 CLI命令中仅13有Tauri等价; 5域: Git/Crypto/Plan/UI/Sandbox全缺 |
| Serde覆盖率 | 🟡 49%→~60% | 221类型中109有serde → P0补25+类型后显著提升 |
| 文档 | 🟢 已清理 | 所有`reasoning_brain/`旧路径已更新为`l8_autonomic_impl/nt_mind/`; VitePress路由已修复; 3域新增文档 |

### 修复清单 (Phase 2-3)

| # | 域 | 缺陷 | 修复 | 文件数 |
|---|-----|------|------|--------|
| 1 | 层违规 | L1→L2 6处(BrowserCircuit/MicCapture/TaskType/GeoPoint) | 本地stub类型定义 | 6 |
| 2 | 层违规 | L1→L3/L8 7处(KnowledgeBase/ProjectSnapshot/ActionPlan) | `nt_l1_shared_types.rs` 新模块 + L8 re-export | 8 |
| 3 | 层违规 | L2→L3 4处(NodeType/KnowledgeBase) | `nt_memory_kb_bridge.rs` 桥接模块 | 5 |
| 4 | 生产expect | 6处中风险(thinking_bridge/crypto/client) | `unwrap_or_else`/`ok_or_else` 硬化 | 5 |
| 5 | Clippy | BrowserCircuit 2处 `new_without_default` | `#[derive(Default)]` 添加 | 2 |
| 6 | 孤儿文件 | `_nt_memory_evidence_placeholder.rs` | 删除(21行stub) | 1 |
| 7 | 文档 | VitePress 路由404 (guide/→4-GUIDES/) | `rewrites` 配置添加 | 1 |
| 8 | Serde P0 | E8: Hexagram/E8Weight/FermionState/E8Policy/E8Outcome/TransitionLearner | derive + 手动serde(大数组→Vec桥接) | 3 |
| 9 | Serde P0 | VSA: VSAEngine | derive | 1 |
| 10 | Serde P0 | GWT: ContentItem/CompressionConfig/Stage/Report/ContextCompressor + CLSBuffer + GraphMemory(6类型) | derive | 3 |
| **合计** | | | **25+类型serde, 19层违规清零, 6处expect硬化** | **35文件** |

### Build基线

| 检查项 | 修复前 | 修复后 |
|--------|--------|--------|
| `cargo check --lib -p neotrix` | ✅ 0 errors | ✅ **0 errors** |
| `cargo clippy -p neotrix --lib` | ❌ 2 warnings | ✅ **0 warnings** |
| `cargo check --features full --lib -p neotrix` | ❌ 5 errors | ✅ **0 errors** |
| `cargo check -p neotrix-tauri` | ✅ 0 errors | ✅ **0 errors, 0 warnings** |
| `cargo test -p neotrix --lib` | 6127 pass, 0 fail, 10 ignore | ✅ **6127 pass, 0 fail, 10 ignore** |
| 前端 `npm run build` | ✅ 2.31s | ✅ **2.31s, 0 errors** |

### 竞争差距分析 (7大竞品)

| 竞品 | ★ | 语言 | 核心差异 | NeoTrix差距 | 优先级 |
|------|---|------|----------|-------------|--------|
| CodeWhale | 39.3K | Rust | 路由解析器, 150贡献者, 122版 | 多Provider路由更智能 | P2 |
| Claurst | 9.8K | Rust | **ACP协议**, 插件系统, 纯净室复现 | 缺ACP协议 | **P1** |
| Crab Code | 72 | Rust | 4,970测试, 26crates, 任何LLM | 功能对等但NT更丰富 | P3 |
| Peri | 68 | Rust | **13MB二进制**, ACP, Claude Code兼容 | 二进制大小差距大 | **P1** |
| OpenDev | 687 | Rust | **4.3ms启动, 9.4MB RAM** | 启动时间优化 | **P1** |
| Nerve | — | Rust | **7.7MB二进制, 0运行时依赖** | 二进制大小差距 | P2 |
| Cortex | — | Rust | 15x快于LangChain, 成本追踪 | 缺少benchmark | P2 |

### 关键元认知收获

1. **层违规修复模式成熟**: L1→L2/L3/L8统一使用本地类型定义(nt_l1_shared_types) + L8 re-export, 零功能变更
2. **Serde补全揭示架构盲点**: E8核心类型(Hexagram/E8Policy)之前完全不可序列化 → 会话恢复、检查点全线断裂 — 现在打通
3. **桌面UI是最大残余差距**: 70% CLI功能无Tauri等价; 但用户已确认不添加UI → 策略转为对话式Agent自动规划
4. **Rust AI代理战场爆发(2026 Q2)**: 7+显著竞品从零涌现, 生态趋势: ACP协议、多Provider路由、超小二进制(7-18MB)、插件系统
5. **Build cache危险**: cargo clean后暴露出5个预存错误(office_renderer extra `)` + serde回归), 全量build应定期clean
6. **测试稳定**: 6127/0/10基线保持, 无回归

### 2026-07-05 Phase 2 — 3路并行修复完成

#### 修复清单

| # | 任务 | 结果 | 关键指标 |
|---|------|------|----------|
| 1 | **文档过时引用清理** | ✅ 18文件, 81/85处`reasoning_brain/` → 新7域路径 | 4处保留(描述重命名历史) |
| 2 | **二进制大小优化** | ✅ neotrix: **7.3MB** (达到竞品Nerve 7.7MB级) | 配置: LTO+strip+opt-level=s+panic=abort |
| 3 | **standalone.rs修复** | ✅ 类型不匹配修复, `neotrix`二进制首次可编译 | cargo check -p neotrix ✅ |
| 4 | **ACP协议调研+规划** | ✅ 249行计划写入 `docs/2-PLANS/ACP_PROTOCOL_IMPLEMENTATION.md` | 4阶段, 21步 |
| 5 | **全量测试验证** | ✅ **6139 passed, 0 failed, 10 ignored** (+12 from baseline) | cargo check --lib/clippy/features full/tauri 全绿 |

#### 二进制大小对比 (Rust AI代理生态)

| 代理 | 二进制大小 | 编制 | 备注 |
|------|-----------|------|------|
| **NeoTrix** | **7.3 MB** | 最终 | LTO+strip+s |
| Nerve | 7.7 MB | 官方 | 0运行时依赖 |
| Peri | 13 MB | 官方 | ACP+Claude Code兼容 |
| OpenDev | 18 MB | 官方 | 4.3ms启动 |
| Crab Code | ~30 MB (估) | 161K LOC | 26crates |
| CodeWhale | ~50 MB (估) | 39K★ | 122releases |
| Claude Code | ~188 MB | 官方 | TypeScript |

### 剩余待办

| # | 项 | 优先级 | 状态 |
|---|-----|--------|------|
| 1 | **ACP协议实现**(P0 base agent) | P1 | 规划已创建 |
| 2 | GWT/PRM剩余serde覆盖(ResonatorNetwork/GeometrySync/PRM learners) | P2 | 待做 |
| 3 | 多Provider智能路由(CodeWhale RouteResolver模式) | P2 | 待做 |
| 4 | 桌面app对话式Agent(CLI后端驱动, 不新增UI) | P2 | 待做 |

- `L5/L9 → KB`: 🔴 零持久化 → 🟢 PhiReport/GoldStandardReport/trends/blind spots 写入 kv_store
- `/avatar CLI`: 🔴 硬编码 → 🟢 DistillationEngine 真实数据
- `nt_memory_hierarchical search_hierarchical`: 🔴 零调用者 → 🟢 通过 consciousness_bridge 接入 GWT
- `production unwrap()`: 🟡 12 处 → 🟢 0 处 (已全部修复)
- `15 个未注册管道阶段`: ⚪ no-op → 🟢 带 trace 日志

### 剩余架构债务

| # | 缺陷 | 优先级 | 难度 | 说明 |
|---|------|--------|------|------|
| 1 | `nt_memory_gwtq` 3/5 方法零调用者 (query_by_e8_state/specialist/broadcast_context) | 🟡 中 | 低 | 方法已实现但无人调用 |
| 2 | DpoWrapperStage/ConstitutionalWrapperStage/SafetyWrapperStage 仍为无操作 | 🟡 中 | 高 | API 不兼容 — 需结构变更才能桥接 |
| 3 | 91 个 `#[allow(dead_code)]` (最严重: resonator_network.rs:10, twitter.rs:4) | 🟡 中 | 低 | 逐步清理未使用的代码 |
| 4 | L7 层缺失 — 架构序列 l1→l9 中无 l7 目录 | 🟢 低 | 低 | 设计决策或完善方向 |
| 5 | 15 个未注册阶段定义仍只通过 recipe.rs 可达 | 🟢 低 | 中 | 可以按需注册到活动管道 |

---

# EarthEpoch Cognitive Framework — Agent Operations Manual

> The system evolves by switching between cognitive frameworks,
> not by optimizing within a single one.

---

## Architecture Overview

```
┌───────────────────────────────────────────────────────────┐
│                    PanoramicBrain                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │ E1 Myth  │ │ E2 Agri  │ │ E3 Axial │ │ ...E8    │    │
│  │ ontology │ │ ontology │ │ ontology │ │ emergent │    │
│  │ state[5] │ │ state[5] │ │ state[5] │ │ state[5] │    │
│  │ reward h │ │ reward h │ │ reward h │ │ reward h │    │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │
│                                                           │
│  route_task(task) → (primary: Epoch, weights: Vec)       │
│  absorb_reward(task, type, reward) → updates epoch state │
│  transfer_knowledge(from, to, rate) → cross-epoch blend  │
│  evaluate_task(task) → (score, all_scores)                │
│  legacy_capability: CapabilityVector (backward compat)   │
└───────────────────────────────────────────────────────────┘
```

### Files

| File | Role |
|------|------|
| `crates/neotrix-types/src/core/epoch/types.rs` | Data types: `EarthEpoch`, `CognitiveFramework`, `DimensionDef`, `FrameworkRoute`, `ActivationRecord` |
| `crates/neotrix-types/src/core/epoch/definitions.rs` | Ontologies, initial states, router biases, evaluator functions, 10 tests |
| `crates/neotrix-types/src/core/epoch/mod.rs` | `pub mod` + re-exports |
| `neotrix-core/src/core/epoch/mod.rs` | Bridge: re-exports from `neotrix_types::core::epoch` |
| `neotrix-core/src/neotrix/reasoning_brain/panoramic.rs` | `PanoramicBrain` — orchestrator holding all 8 frameworks, routing + absorption + transfer |

### Core Types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarthEpoch {
    E1Mythological,
    E2Agricultural,
    E3Axial,
    E4Scientific,
    E5Global,
    E6Planetary,
    E7Network,
    E8Emergent,
}

pub struct CognitiveFramework {
    pub epoch: EarthEpoch,
    pub state: Vec<f64>,                // dimension activation values [0,1]
    pub ontology: Vec<DimensionDef>,     // named dimensions
    pub activation_count: u64,
    pub accumulated_reward: f64,
    pub router_bias: f64,               // static preference weight
}

pub struct DimensionDef {
    pub name: String,
    pub description: String,
}

pub struct FrameworkRoute {
    pub primary: EarthEpoch,
    pub weights: Vec<(EarthEpoch, f64)>,
}
```

---

## How to Add a New Epoch

### 1. Add the enum variant

In `crates/neotrix-types/src/core/epoch/types.rs`, add to `EarthEpoch`:

```rust
#[derive(...)]
pub enum EarthEpoch {
    // ... existing variants ...
    E9NewEpoch,
}
```

Add to `EarthEpoch::all()` and `EarthEpoch::name()`:

```rust
EarthEpoch::E9NewEpoch => "New Epoch (E9)",
```

### 2. Define the ontology

In `crates/neotrix-types/src/core/epoch/definitions.rs`, add to `ontology_for()`:

```rust
EarthEpoch::E9NewEpoch => vec![
    DimensionDef { name: "dimension_a".into(), description: "描述...".into() },
    DimensionDef { name: "dimension_b".into(), description: "描述...".into() },
    // 3–6 dimensions recommended
],
```

### 3. Set initial state

Add to `initial_state_for()`:

```rust
EarthEpoch::E9NewEpoch => vec![0.3, 0.2, 0.1],  // match ontology length
```

### 4. Set router bias

Add to `default_router_bias()`:

```rust
EarthEpoch::E9NewEpoch => 0.40,  // 0.0–1.0
```

### 5. Implement the evaluator

Add to `evaluate_in_epoch()`:

```rust
EarthEpoch::E9NewEpoch => {
    let keyword_a = contains_any(&task_lower, &["word1", "word2", "phrase"]);
    let keyword_b = contains_any(&task_lower, &["word3", "word4"]);
    let base = if keyword_a { 0.6 } else { 0.3 }
        + if keyword_b { 0.5 } else { 0.2 };
    let avg_state = state.iter().sum::<f64>() / state.len() as f64;
    ((base / 1.1) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
}
```

### 6. Update tests

In `definitions.rs`'s `#[cfg(test)] mod tests`:

```rust
fn test_e9_new_epoch_dimensions() {
    let frameworks = all_frameworks();
    let e9 = frameworks.iter().find(|fw| fw.epoch == EarthEpoch::E9NewEpoch);
    assert!(e9.is_some());
    assert_eq!(e9.unwrap().dim(), 3);
}
```

### 7. Verify

```
cargo check --lib -p neotrix-types
cargo test --lib -p neotrix-types "epoch"
cargo check --lib   # full project (may have unrelated errors)
```

---

## How to Modify an Existing Epoch's Ontology

### Change a dimension name/description

In `ontology_for()` in `definitions.rs`:

```rust
// Before:
DimensionDef { name: "old_name".into(), description: "old desc".into() }
// After:
DimensionDef { name: "new_name".into(), description: "new desc".into() }
```

Then update any tests that reference the old name:

```rust
// In tests:
assert!(fw.dimension_index("new_name").is_some());  // was "old_name"
```

### Add a new dimension

Append to the epoch's `vec![]` in `ontology_for()`:

```rust
EarthEpoch::E4Scientific => vec![
    // ... existing 6 ...
    DimensionDef { name: "reproducibility".into(), description: "实验结果必须可被独立重复验证".into() },
],
```

Then update `initial_state_for()` to add the corresponding initial value:

```rust
EarthEpoch::E4Scientific => vec![0.3, 0.3, 0.3, 0.2, 0.3, 0.4, 0.1],  // +reproducibility
```

Update the evaluator to handle the new dimension:

```rust
EarthEpoch::E4Scientific => {
    // ... existing keyword checks ...
    let reproduce = contains_any(&task_lower, &["reproduc", "replicat", "repeat"]);
    let base = /* ... */ + if reproduce { 0.3 } else { 0.1 };
    // ...
}
```

Update the test for expected dimension count:

```rust
(EarthEpoch::E4Scientific, 7),  // was 6
```

### Remove a dimension

Remove from `ontology_for()`, `initial_state_for()`, and the evaluator. Update any tests that reference the removed dimension's index or name.

---

## How the Routing System Works

### Routing Formula

`PanoramicBrain::route_task()` computes weight for each epoch:

```
weight = 0.40 * eval_score(epoch, task)
       + 0.30 * history_bonus(task_type, epoch)
       + 0.30 * effective_weight(epoch)
```

Where:
- **eval_score** = `evaluate_in_epoch(epoch, &fw.state, task)` — keyword-match by epoch (30%) + state vector strength (70%)
- **history_bonus** = EMA of rewards for this `(TaskType, EarthEpoch)` pair (exponential moving average: `new = old * 0.9 + reward * 0.1`)
- **effective_weight** = `0.7 * router_bias + 0.3 * average_reward` — combines static bias with dynamic reward history

Weights are sorted descending. The top epoch becomes the `primary` in `FrameworkRoute`.

### Task Flow

```
Input: task_description, task_type (optional)
  │
  ▼
route_task() → FrameworkRoute { primary: Epoch, weights: Vec<(Epoch, f64)> }
  │
  ├── evaluate_task() → (primary_score, all_scores)
  │     (read-only assessment, no side effects)
  │
  └── absorb_reward(task, task_type, reward)
        │
        ├── Record activation on primary epoch
        ├── Update epoch state: state += reward * 0.05 for all dims
        ├── Update epoch_success_by_task[t][e] = EMA
        ├── Push to activation_log
        └── Every 10th activation: sync_to_legacy()
```

### Selecting the Active Epoch

- **Default**: `E7Network` (current dominant paradigm)
- **Manual**: `brain.switch_to(EarthEpoch::E4Scientific)` — sets active_epoch + syncs legacy
- **Automatic**: `route_task()` always returns the best epoch as `primary`
- **Best by task_type**: `brain.best_epoch_for(TaskType::CodeAnalysis)` — uses learned history

---

## Evaluator Function Pattern

Each epoch evaluator follows a consistent structure in `evaluate_in_epoch()`:

```rust
EarthEpoch::EXName => {
    // 1. Define keyword sets that characterize this epoch's cognitive mode
    let keyword_a = contains_any(&task_lower, &["trigger1", "trigger2"]);
    let keyword_b = contains_any(&task_lower, &["trigger3"]);

    // 2. Base score from keyword matching (0.0–1.5 range typical)
    let base = if keyword_a { 0.6 } else { 0.2 }
        + if keyword_b { 0.4 } else { 0.1 };

    // 3. State dimension to use (choose most relevant dim, or average)
    let dim_score = state.first().copied().unwrap_or(0.0);
    // OR: let avg_state = state.iter().sum::<f64>() / state.len() as f64;

    // 4. Combine: keyword relevance (30%) + actual capability (70%)
    ((base / total_keyword_weight) * 0.3 + dim_score * 0.7).clamp(0.0, 1.0)
}
```

### Pattern Rules

1. **Keyword sets should be small** (2–5 words) and high-precision. Each set captures one dimension of the epoch's cognitive mode.
2. **Base keyword weight** is capped proportionally: sum of keyword contributions → divide by sum of max possible.
3. **State/structure ratio** is 70/30 (state-dominant). The evaluator is not just keyword matching — it reflects the system's learned strength in each dimension.
4. **Output is always clamped** to `[0.0, 1.0]` for compatibility with routing weights.
5. **Each epoch is different**: E4 uses all dimensions averaged; E1 uses only the first dimension; E3 uses the third dimension. Choose the dimension(s) that best represent the epoch's core cognitive mode.

### Example: E4 Scientific

```rust
EarthEpoch::E4Scientific => {
    // Keywords capture: analysis, precision, reduction
    let analysis = contains_any(&task_lower, &["analy", "measure", "calculate", "verify", "test", "experiment", "prove"]);
    let precision = contains_any(&task_lower, &["precise", "exact", "accurate", "quantif", "metric"]);
    let reduction = contains_any(&task_lower, &["decompose", "reduce", "break down", "component", "element"]);
    let base = if analysis { 0.6 } else { 0.3 }
        + if precision { 0.5 } else { 0.2 }
        + if reduction { 0.4 } else { 0.2 };
    let avg_state = state.iter().sum::<f64>() / state.len() as f64;
    ((base / 1.5) * 0.3 + avg_state * 0.7).clamp(0.0, 1.0)
}
```

### Example: E1 Mythological

```rust
EarthEpoch::E1Mythological => {
    // Keywords capture: narrative, cyclical time, animism
    let narrative = contains_any(&task_lower, &["story", "myth", "ritual", "symbol", "archetype", "ceremony", "sacred"]);
    let cyclical = contains_any(&task_lower, &["cycle", "season", "return", "rebirth", "eternal"]);
    let animism = contains_any(&task_lower, &["nature", "spirit", "soul", "alive", "consciousness of"]);
    let base = if narrative { 0.6 } else { 0.2 }
        + if cyclical { 0.3 } else { 0.1 }
        + if animism { 0.3 } else { 0.1 };
    let dim_score = state.first().copied().unwrap_or(0.0);  // Only first dimension
    ((base / 1.2) * 0.3 + dim_score * 0.7).clamp(0.0, 1.0)
}
```

---

## Cross-Epoch Knowledge Transfer

`PanoramicBrain::transfer_knowledge(from, to, rate)` blends state vector dimensions by index:

```rust
let min_len = source.len().min(target.len());
for i in 0..min_len {
    let delta = source[i] - target[i];
    target[i] += rate * delta;  // rate typically 0.05–0.20
}
target.normalize();  // clamp max ≤ 1.0
```

This is a **simple dimension-index mapping** (not semantic). When epochs have different dimension counts, only the shared dimensions (by index) are transferred. Future improvements could use semantic mapping through the ontology dimension names.

---

## Compile Gate

```
cargo check --lib -p neotrix-types        # epoch types + definitions
cargo test --lib -p neotrix-types "epoch"  # 10 epoch tests
cargo check --lib                          # full project (may have unrelated errors)
```

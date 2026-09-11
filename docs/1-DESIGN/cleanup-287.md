# Cleanup-287: 跨域引用审计

**日期**: 2026-09-11  
**审计范围**: `neotrix-core/src/l{1-6}_*` 层间 `use crate::` 引用  
**命令**: `grep -rn "use crate::$to" --include="*.rs" | grep -v test | grep -v facade`

---

## 1. 层间引用拓扑

| 源→目标 | 数量 | 严重性 |
|---------|------|--------|
| l5_cognition → l1_action | **106** | **CRITICAL** |
| l5_cognition → l6_meta | 2 | OK (注释) |
| l5_cognition → l2_perception | 1 | OK (注释) |
| l6_meta → 任意下层 | 0 | OK |
| l4_emotion → 上层 | 0 | OK |
| l3_embodiment → 任意 | 0 | OK |
| l2_perception → 任意 | 0 | OK |
| l1_action → 任意 | 0 | OK |

**结论**: 唯一违反分层原则的是 **l5_cognition → l1_action**，共 106 处直接引用，涉及 27 个源文件。

---

## 2. l5_cognition→l1_action 引用分类

### 2.1 按 L1 子模块分布

| L1 子模块 | 引用数 | 占比 |
|-----------|--------|------|
| `nt_memory` | 52 | 49% |
| `nt_act` | 45 | 42% |
| `nt_io` | 9 | 9% |

### 2.2 按引用模式分布

| 模式 | 引用数 | 说明 |
|------|--------|------|
| `nt_memory_kb::*` | ~40 | KB 操作 (KnowledgeBase, kv_set, fetch_safe_http) |
| `nt_act_autonomy::*` | ~15 | OracleGate, CrossSessionMemory |
| `nt_act_code::*` | ~15 | SemanticEntropyGate, EvolutionLoopProvider |
| `nt_act_actions::sandbox` | ~8 | ActionSandbox, SandboxVerdict |
| `nt_io_provider::gateway` | ~3 | Gateway re-evaluation |
| `nt_io_session_recovery` | ~2 | SessionRecoveryManager |
| `nt_io_user_avatar` | ~1 | DistillationEngine |
| `nt_io_standalone` | ~3 | ReasoningMethod |

### 2.3 受影响源文件 (27 files)

```
nt_mind_background_loop/
  ├── handlers_core.rs           (1)
  ├── run.rs                     (2)
  ├── mod.rs                     (1)
  ├── handlers_daily_intel.rs    (1)
  ├── handlers_consciousness.rs  (5)
  └── knowledge_pipeline.rs      (2)

nt_mind/
  ├── web_miner.rs               (4)
  ├── self_evolver.rs            (1)
  ├── auto_crystallizer.rs       (1)
  ├── nt_mind_skill_engine.rs    (6)
  ├── co_evolution.rs            (1)
  ├── consciousness/
  │   └── hypercube_bridge.rs    (1)
  ├── knowledge/
  │   ├── web_miner.rs           (4)
  │   └── knowledge_engine/
  │       └── search.rs          (3)
  ├── evolution/
  │   ├── self_diagnose.rs       (2)
  │   ├── evolution_loop.rs      (30+)
  │   ├── self_evolver.rs        (1)
  │   ├── co_evolution.rs        (1)
  │   └── agent_capability/
  │       └── mod.rs             (1)
  ├── reason/
  │   ├── attention_router.rs    (1)
  │   └── reasoning_engine/
  │       └── engine_core.rs     (3)
  └── seal_core/
      └── self_iterating/
          ├── pipeline.rs        (10+)
          └── loop_impl/
              └── seal_loop.rs   (1)

foundation/
  ├── seal_pipeline.rs           (20+)
  ├── knowledge_store.rs         (8)
  └── l1_wrappers.rs             (4)

nt_core/
  └── nt_core_parallel/
      └── contract.rs            (2)
```

---

## 3. 违规分析

### 3.1 核心违规: L5 认知层直接调用 L1 行动层

**架构规则**: L5 (Cognition) 不应直接依赖 L1 (Action)。L1 是能力网，L5 是认知层，应通过 facade 或 trait 抽象访问。

**实际行为**:
- L5 的 `nt_mind` 直接调用 L1 的 `nt_memory` (KB 操作)
- L5 的 `nt_mind` 直接调用 L1 的 `nt_act` (OracleGate, Sandbox, SemanticEntropy)
- L5 的 `nt_mind` 直接调用 L1 的 `nt_io` (Gateway, SessionRecovery)

### 3.2 已有 Facade 未完全使用

`l5_cognition/mod.rs` 定义了 facade 模块:
```rust
pub mod kb_facade;      // L1 memory facade
pub mod io_facade;      // L1 IO facade
pub mod act_facade;     // L1 act facade
pub mod l2_facade;      // L2 perception facade
pub mod l6_facade;      // L6 meta facade
```

但 27 个文件仍在直接引用 `crate::l1_action::*`，未走 facade。

### 3.3 典型违规路径

| 路径 | 调用 | 建议 |
|------|------|------|
| `nt_mind/web_miner.rs` | `nt_memory_kb::nt_http::fetch_safe_http` | 走 `kb_facade` |
| `nt_mind/nt_mind_skill_engine.rs` | `nt_memory_kb::nt_memory_unify::now` | 走 `kb_facade` |
| `foundation/seal_pipeline.rs` | `nt_act::nt_act_autonomy::oracle_gate::OracleGate` | 走 `act_facade` |
| `foundation/l1_wrappers.rs` | `nt_io::nt_io_session_recovery::SessionRecoveryManager` | 走 `io_facade` |
| `handlers_consciousness.rs` | `nt_act::nt_act_sandbox::ActionSandbox` | 走 `act_facade` |

---

## 4. 修复方案

### Phase 1: 补全 Facade (低风险)

| Facade | 需要补充的 API |
|--------|---------------|
| `kb_facade` | `fetch_safe_http`, `kv_set`, `now`, `KnowledgeBase::open`, `NodeType`, `EvolutionPatternType`, `EvolutionRecord` |
| `act_facade` | `OracleGate`, `SemanticEntropyGate`, `ActionSandbox`, `CrossSessionMemorySelfTest`, `EvolutionLoopProvider` |
| `io_facade` | `SessionRecoveryManager`, `DistillationEngine`, `gateway::run_periodic_re_evaluation`, `ReasoningMethod` |

### Phase 2: 逐文件迁移 (中风险)

优先级排序 (按引用数):
1. `evolution/evolution_loop.rs` — 30+ 引用
2. `foundation/seal_pipeline.rs` — 20+ 引用
3. `nt_mind_skill_engine.rs` — 6 引用
4. `handlers_consciousness.rs` — 5 引用
5. `foundation/knowledge_store.rs` — 8 引用

### Phase 3: 验证

```bash
# 迁移后验证
cargo check -p neotrix --all-targets
cargo test -p neotrix --lib

# 跨域引用复核
grep -rn "use crate::l1_action" neotrix-core/src/l5_cognition --include="*.rs" | grep -v test | grep -v facade | wc -l
# 目标: 0
```

---

## 5. 其他层健康状态

| 层 | 状态 | 说明 |
|----|------|------|
| l6_meta | ✅ | 无向下引用 |
| l4_emotion | ✅ | 无向上引用 |
| l3_embodiment | ✅ | 无跨层引用 |
| l2_perception | ✅ | 无跨层引用 |
| l1_action | ✅ | 无向上引用 |

---

## 6. 参考

- **六层架构规则**: L1→L2→L3→L4→L5→L6 单向依赖
- **Facade 模式**: `l5_cognition/mod.rs` 中 `kb_facade`, `io_facade`, `act_facade`
- **AGENTS.md**: R-P42 吸收强化现有节点，禁止平行适配器模块

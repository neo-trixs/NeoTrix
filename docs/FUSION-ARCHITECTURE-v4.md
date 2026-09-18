# NeoTrix 融合架构 v4.2 — 熔炼外部模式到能力骨架 (精确版)

> **日期**: 2026-09-18 (v4.2 精确修正)
> **基于**: 500+ 外部 URL 深度吸收 + 代码库骨架分析 + **精确类型使用追踪** + **逐文件依赖分析** + 底层模型逆向推理
> **目标**: 聚焦冗余 + 扁平缺陷 + 跨域错位 → 冗余清理 → 最优熔炼到进化设计蓝图
> **修正**: v4.1 的 SelfTest 分析遗漏了 TEST_ENV_LOCK 双静态 bug；v4.0 的类型重复统计有误

---

## 一、外部模式吸收矩阵 (15 个最高价值模式)

### 1.1 内存系统模式 (Memory Patterns)

| # | 模式 | 来源 | Rust 等价模式 | NeoTrix 目标 | 优先级 |
|---|------|------|-------------|-------------|--------|
| M1 | **分层内存管线** (compress→segment→extract→index→retrieve) | LightMem (ICLR 2026) | `TieredPipeline` trait + 4 阶段管线 | NT-MEMORY | P0 |
| M2 | **Git-for-Memory** (snapshot/branch/merge/rollback) | Memoria | `MemoryStore` + Copy-on-Write 快照 | experience-tree KB | P0 |
| M3 | **证据驱动技能质量** (outcome-based trust, CAPTURED evolution) | OpenSpace | `QualityMetrics` + 信任生命周期 | Skill routing / GWT | P0 |
| M4 | **KV-Context 虚拟化** (paged KV across GPU/host/NVMe) | KVMem | `PagedKV` + 三级缓存管理 | NT-MEMORY paged KV | P0 |
| M5 | **显式可变执行状态** (discard intermediate traces) | SKILL.state | `ExecState` struct (mutable, not append-only) | SKILL-SPEC.md contract | P1 |
| M6 | **Train-Deployment 一致性** (SDCC under compression) | MemoryWalker | `CompressionGuard` + 一致性检查 | NT-MEMORY reliability | P1 |
| M7 | **Discovery-Tree 回放** (off-policy self-improvement) | Dream-RSI | `DiscoveryTree` + 回放模拟器 | experience-tree absorption | P1 |

### 1.2 代理架构模式 (Agent Architecture Patterns)

| # | 模式 | 来源 | Rust 等价模式 | NeoTrix 目标 | 优先级 |
|---|------|------|-------------|-------------|--------|
| A1 | **JIT-Harness 生成** (composable 4-module protocol) | JIT-Agent | `HarnessProtocol` trait (4 模块) | NT-MIND self-evolution | P0 |
| A2 | **双轨自治** (Crews=autonomous + Flows=deterministic) | CrewAI | `Crew` + `Flow` + `轨道路由器` | NT-MIND + NT-ACT | P0 |
| A3 | **Agent-as-Tool 组合** (recursive agent delegation) | AutoGen | `AgentTool` wrapper trait | AttentionManager routing | P1 |
| A4 | **隔离实现运行** + proof of work | Symphony | `IsolatedRun` + `ProofOfWork` | NT-ACT execution | P1 |
| A5 | **结构化辩论** (bullish vs bearish researchers) | TradingAgents | `DebatePair` + 裁判器 | NT-CORE reasoning | P1 |
| A6 | **非阻塞子代理生成** (spawn + auto result return) | pi-crew | `tokio::spawn` + `oneshot::channel` | NT-ACT parallelism | P1 |
| A7 | **Joint Harness-Weight 优化** (alternating phases) | WHALE | `AlternatingOptimizer` + 耐心规则 | NT-MIND co-evolution | P2 |

### 1.3 工具/技能模式 (Tool/Skill Patterns)

| # | 模式 | 来源 | Rust 等价模式 | NeoTrix 目标 | 优先级 |
|---|------|------|-------------|-------------|--------|
| T1 | **输出压缩管线** (filter→group→truncate→dedup) + recall | RTK | `OutputFilter` trait + 4 策略 | Tool output handling | P0 |
| T2 | **渐进技能加载** (~100 tokens → full on demand) | Vercel agent-skills | `SkillManifest` + `LazySkill` | Lazy skill loading | P1 |
| T3 | **Skills-as-Markdown 协议** (reference vs pipeline skills) | Mem0/Browser Use | `SKILL.md` YAML frontmatter | SKILL-SPEC.md | P1 |
| T4 | **Cost Ladder** (novelty-complexity → agent rung) | Hermes Orchestrator | `CostLadder` + `NoveltyComplexityMatrix` | Axiom A1 routing | P2 |
| T5 | **Dreaming 巩固** (overnight promote/discard) | Skales | `DreamConsolidator` + 冷却期 | NT-MIND absorption | P2 |

### 1.4 跨领域收敛模式 (Cross-Cutting Patterns)

| # | 模式 | 来源 | NeoTrix 映射 |
|---|------|------|-------------|
| C1 | **证据驱动信任** (outcome tracking at every level) | OpenSpace/TradingAgents/Multica | Constellation maturity C0-C6 |
| C2 | **成本感知路由** (not all tasks need strongest model) | RTK/Hermes/Axiom A1 | GWT salience + cost weight |
| C3 | **Context 作为稀缺资源** | KVMem/Axiom A2 | 分页 KV + 压缩 |
| C4 | **共享语言 (CONTEXT.md)** | Pocock skills | 已采用 |
| C5 | **AGENTS.md 作为标准** | DeepSeek/CrewAI/OpenHands | 已采用 |
| C6 | **MCP 作为工具协议** | Claude Code/Qwen Code/Codex | CapabilityBridge |

---

## 二、代码库冗余审计 (精确修正版)

> **重要修正**: v4.0 的类型重复统计基于 grep 匹配，未区分"同名不同类型"和"真正重复"。
> 本版基于精确代码追踪，区分了三种情况：
> 1. **真正重复** — 同一类型在多处定义，需要合并
> 2. **同名异义** — 不同类型碰巧同名，需要重命名
> 3. **模块本地** — 每个模块自己的类型，无需合并

### 2.1 TaskType — 真正重复 (3 → 1)

| # | 文件 | 变体数 | 用途 | 迁移复杂度 |
|---|------|--------|------|-----------|
| A | `crates/neotrix-types/src/core/nt_core_knowledge/types.rs:10` | **35+** | **规范定义** (超集) | — |
| B | `neotrix-core/src/l5_cognition/nt_core_god_agent.rs:96` | 10 | 内部使用 (子集) | **LOW** |
| C | `neotrix-core/src/l1_action/nt_act/actions/orchestration/production_pipeline.rs:32` | 8 | 领域特定 (视频制作) | **重命名** |

**实际状态**: 规范定义 A 已经是超集 (35+ 变体)，包含 B 的所有变体。B 仅在 `nt_core_god_agent.rs` 内部使用，从未被外部导入。C 是视频制作领域的 TaskType，与 AI 任务路由无关。

**迁移方案**:
```
1. B (god_agent): 改为 use crate::l2_perception::nt_core_knowledge::TaskType
   影响: 1 个文件 (nt_core_god_agent.rs)
   
2. C (production_pipeline): 重命名为 ProductionStage
   影响: 1 个文件 (production_pipeline.rs)
   
3. A (规范): 保持不变
```

### 2.2 TaskStatus — 模块本地 (7 → 保持 7)

| # | 文件 | 变体 | 用途 |
|---|------|------|------|
| 1 | `l1_action/nt_act/nt_act_scheduler.rs:48` | Registered,Scheduled,Running,Completed,Failed(String),Paused,Cancelled | Cron 调度 |
| 2 | `l1_action/nt_act/parallel_task.rs:15` | Pending,Running,Paused,Completed,Failed,Cancelled | 并行任务 |
| 3 | `l1_action/nt_io/nt_io_protocol_bridge.rs:132` | Submitted,Working,InputRequired,Completed,Failed,Canceled,Rejected | A2A 协议 |
| 4 | `l1_action/nt_act/actions/orchestration/production_pipeline.rs:15` | Pending,Running,Completed,Failed,Paused,Cancelled | 视频制作 |
| 5 | `l3_embodiment/nt_shield/nt_shield_impl/nt_shield_swarm.rs:80` | Pending,Assigned,InProgress,Completed,Failed | Swarm 代理 |
| 6 | `l5_cognition/nt_core/reasoning/nt_core_planning.rs:86` | Pending,Running,Completed,Failed,Blocked | 任务图规划 |
| 7 | `l6_meta/nt_core_self/cuda_agent.rs:24` | Pending,InProgress,Completed,Failed | CUDA 代理 |

**实际状态**: 每个定义都是**模块本地**的，建模各自领域的生命周期状态。它们碰巧同名但语义不同 (如 scheduler 有 Registered/Scheduled，planning 有 Blocked)。**无需合并** — 这是正确的领域驱动设计。

**迁移方案**: **无需迁移** — 保持各模块本地定义。

### 2.3 SelfTest / SelfTestRegistry / SelfTestResult — 真正重复 (2 → 1)

| 类型 | L0 定义 | L6 定义 | 消费者数 |
|------|---------|---------|----------|
| `SelfTest` trait | `l0_substrate/nt_core_self_test.rs` | `pub use` 从 L0 | **~70+ 文件** (全部从 L6 导入) |
| `SelfTestRegistry` | `l0_substrate/nt_core_self_test.rs` | **自己的副本** (不从 L0 re-export) | L6: ~15 文件, L0: 1 文件 |
| `SelfTestResult` | `l0_substrate/nt_core_self_test.rs` | **自己的副本** (不从 L0 re-export) | L6: ~8 文件, L0: 1 文件 |
| `DurationDriftMonitor` | `l0_substrate/nt_core_self_test.rs` | **自己的副本** (不从 L0 re-export) | 0 外部导入 |
| `DurationDriftTest` | `l0_substrate/nt_core_self_test.rs` | **自己的副本** (不从 L0 re-export) | 0 外部导入 |
| `TEST_ENV_LOCK` | `l0_substrate/nt_core_self_test.rs` (#[cfg(test)]) | **自己的副本** (#[cfg(test)]) | L6: 3 文件 |

#### 精确复制分析

| 类型 | L0 行数 | L6 行数 | 完全相同? |
|------|---------|---------|----------|
| `SelfTestRegistry` | L0:21-80 (60行) | L6:17-76 (60行) | **是** — 结构体+方法完全相同 |
| `SelfTestResult` | L0:82-118 (37行) | L6:78-114 (37行) | **是** — 结构体+方法完全相同 |
| `report()` fn | L0:120-132 (13行) | L6:116-128 (13行) | **是** — 完全相同 |
| `DurationDriftMonitor` | L0:136-201 (66行) | L6:134-206 (73行) | **近乎相同** (L6 稍冗长) |
| `DurationDriftTest` | L0:204-238 (35行) | L6:209-242 (34行) | **近乎相同** |
| `TEST_ENV_LOCK` | L0:17 (1行) | L6:14 (1行) | **是** — 但这是两个独立的进程级 Mutex! |

#### ⚠️ 严重 Bug: TEST_ENV_LOCK 双静态实例

```
L0: #[cfg(test)] static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());
L6: #[cfg(test)] static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());
```

**问题**: 这是**两个独立的进程级 Mutex**。从 L0 路径获取锁的代码和从 L6 路径获取锁的代码**不会互相同步** — 完全破坏了隔离目的。

**消费者**: `kb_cmds.rs`、`cipher.rs`、`consciousness_core.rs` 从 L6 路径获取锁 (3 文件)。L0 自身的测试从 L0 路径获取锁。两组锁**互不感知**。

**修复**: 合并后只有一个 `TEST_ENV_LOCK` (在 L0)，所有消费者通过 re-export 统一到同一个锁。

#### 消费者路径分析

```
133 文件: use crate::l6_meta::healing::nt_core_self_test::{SelfTest, ...}
  1 文件: use crate::l0_substrate::nt_core_self_test::{SelfTest, SelfTestRegistry}
  1 文件: core/nt_core_traits.rs re-exports SelfTest from L6 (而非 L0)
```

#### L6 特有实现 (需保留)

| 实现 | 行数 | 外部消费者 |
|------|------|-----------|
| `ExternalVerifier` (运行 `cargo check` 作为地面真值) | 244-273 (30行) | `handlers_consciousness.rs:1549` (1 文件) |
| `ConstitutionComplianceTest` (治理合规检查) | 275-319 (45行) | `nt_core_self_test_integration.rs:16,269` (1 文件) |
| `TraceEvaluationTest` (轨迹评估验证) | 321-399 (79行) | **无外部消费者** |

#### 迁移方案 (精确到行)

```
修改文件: neotrix-core/src/l6_meta/healing/nt_core_self_test.rs
Before: 399 行 (完整重复定义)
After:  ~100 行 (re-export + 3 个 L6 特有 struct)

具体变更:
1. 删除 L6 中的 SelfTestRegistry 副本 (L6:17-76)
2. 删除 L6 中的 SelfTestResult 副本 (L6:78-114)
3. 删除 L6 中的 report() 副本 (L6:116-128)
4. 删除 L6 中的 DurationDriftMonitor 副本 (L6:134-206)
5. 删除 L6 中的 DurationDriftTest 副本 (L6:209-242)
6. 删除 L6 中的 TEST_ENV_LOCK 副本 (L6:14)
7. 添加统一 re-export:
   pub use crate::l0_substrate::nt_core_self_test::{
       SelfTest, SelfTestRegistry, SelfTestResult,
       DurationDriftMonitor, DurationDriftTest,
       report, TEST_ENV_LOCK,
   };
8. 保留 ExternalVerifier, ConstitutionComplianceTest, TraceEvaluationTest
   (这些是 L6 特有实现，只从 L6 路径导入)

可选优化: core/nt_core_traits.rs:13
Before: pub use crate::l6_meta::healing::nt_core_self_test::SelfTest;
After:  pub use crate::l0_substrate::nt_core_self_test::SelfTest;
(直接指向 L0，避免 L6 中转)

影响:
- 需修改文件: 1 个 (L6 self_test.rs) + 可选 1 个 (nt_core_traits.rs)
- 需修改消费者: 0 个 (所有 133 个消费者通过 re-export 自动兼容)
- TEST_ENV_LOCK 合并: 3 个消费者自动统一到同一个锁 (bug 修复)
- 净减少: ~299 行代码 (-40%)
```

### 2.4 NodeType / RelationType / KnowledgeNode — 已清理

| 类型 | 定义位置 | L6 状态 | 消费者 |
|------|---------|---------|--------|
| `NodeType` | `neotrix-types::knowledge_access` | 纯 re-export | ~20 文件 (从 L6 路径导入) |
| `RelationType` | `neotrix-types::knowledge_access` | 纯 re-export | ~10 文件 (从 L6 路径导入) |
| `KnowledgeNode` | `neotrix-types::knowledge_access` | 纯 re-export | ~5 文件 (从 L6 路径导入) |

**实际状态**: L6 的 `nt_core_kb_types.rs` 是**纯 re-export** (3 行代码)，没有自己的定义。**无需合并** — 已经是单一事实源。

**注意**: `node_canvas::NodeType` (Text/Image/Code/...) 是**完全不同的类型**，用于画布 UI，不是知识图谱节点。无需合并。

### 2.5 CapabilityVector — 已清理

| 类型 | 定义位置 | 重复数 |
|------|---------|--------|
| `CapabilityVector` | `neotrix-types::core::nt_core_cap` | **1** (无重复) |

**无需操作**。

### 2.6 L5→L6 Shim — 反向兼容垫片 (精确分析)

**文件**: `l5_cognition/nt_core/nt_meta/mod.rs`

```rust
// 从 L6 re-export 19 个模块/类型到旧的 L5 路径
pub use crate::l6_meta::nt_meta::knowledge_gap_detector;
pub use crate::l6_meta::nt_meta::metacognition_loop;
pub use crate::l6_meta::nt_meta::monitor;
pub use crate::l6_meta::nt_meta::nt_core_arch_lint;
pub use crate::l6_meta::nt_meta::nt_core_meta_auditor;
pub use crate::l6_meta::nt_meta::planner;
pub use crate::l6_meta::nt_meta::scanner;
pub use crate::l6_meta::nt_meta::self_model;
pub use crate::l6_meta::nt_meta::weakness;
// + 类型 re-exports: GapCategory, MetaCognitiveLoop, MetaMonitor, ...
```

**问题**: 这个 shim 使 L5 依赖 L6，破坏了依赖单向流动原则。

**消费者**: 仅 1 个外部文件 — `l6_meta/self_model_unified.rs:20` (讽刺地，L6 自己从 L5 shim 导入 L6 的类型！)

**迁移方案 (精确到行)**:
```
修改文件 1: l6_meta/self_model_unified.rs:20
Before: use crate::l5_cognition::nt_core::nt_meta::self_model;
After:  use crate::l6_meta::nt_meta::self_model;

修改文件 2: l5_cognition/nt_core/nt_meta/mod.rs
Action: 删除整个文件 (19 行 re-export)

修改文件 3: l5_cognition/nt_core/mod.rs
Action: 移除 pub mod nt_meta; 声明

影响: 仅 3 个文件，零风险
```

### 2.7 修正后的问题总结

| 类别 | v4.0 报告 | v4.1 修正 | 说明 |
|------|----------|----------|------|
| 重复 TaskType | 7 | **3** (2 真正重复 + 1 同名异义) | god_agent 内部使用, production_pipeline 需重命名 |
| 重复 TaskStatus | 13 | **7** (全部模块本地) | **无需合并** — 正确的 DDD |
| 重复 NodeType | 4 | **2** (knowledge + canvas, 不同域) | **无需合并** — 不同类型 |
| 重复 RelationType | 6 | **1** (neotrix-types) + neotrix-sim (独立 crate) | **无需合并** |
| 重复 SelfTest* | 3 | **6 类型 × 2 副本** (含 TEST_ENV_LOCK bug) | **需要合并** — 7 个重复类型 + 1 个锁安全 bug |
| 重复 CapabilityState | 2 | **需要确认** | 待深入分析 |
| 重复 Emotion | 3 | **需要确认** | 待深入分析 |
| 跨层违规 | 250+ | **~228+** | 精确统计不变 |
| dead_code | 87 | **87** | 精确统计不变 |
| 重复 route() | 37+ | **37+** | 精确统计不变 |

### 2.8 跨层依赖精确分析 (新增)

```
依赖方向统计 (模块内跨层导入):
Layer     | 违规来源 | 违规目标 | 净方向
----------|---------|---------|--------
L0        | 0       | 0       | ✅ 干净
L1 Action | 24 from L5 | 47 from L6 | ⬆️ 向上 (最严重)
L2 Perception | 17 from L5 | 19 from L6 | ⬆️ 向上
L3 Embodiment | 0 from L5 | 18 from L6 | ⬆️ 向上 (轻度)
L4 Emotion | 0       | 0       | ✅ 干净
L5 Cognition | 18 from L6 | (自包含) | ↔️ 与 L6 双向
L6 Meta   | (自包含) | (自包含) | ↔️ 与 L5 双向
core/     | 2 from L6 | 10 from L5 | ⬆️ 向上
cli/      | 47 from L5 | 2 from L6 | ⬆️ 向上 (但 CLI 是顶层，可接受)
```

#### L5 ↔ L6 双向依赖 (关键发现)

| 方向 | 文件 | 导入 |
|------|------|------|
| L6→L5 | `l6_meta/memory/meta_observer.rs:14` | `CoreSnapshot` from L5 |
| L6→L5 | `l6_meta/memory/transcendent_loop.rs:18` | `CoreSnapshot` from L5 |
| L6→L5 | `l6_meta/memory/consonance_orchestrator.rs:14` | `CoreSnapshot` from L5 |
| L6→L5 | `l6_meta/nt_core_observer.rs:4-5` | `AgentTrajectory`, `TrajectoryStep` from L5 |
| L6→L5 | `l6_meta/nt_meta/arch_optimizer.rs:1` | `AwarenessReport` from L5 |
| L6→L5 | `l6_meta/nt_meta/knowledge_gap_detector.rs:8` | `KnowledgeBase` from L5 (L5 re-exporting L1!) |
| L6→L5 | `l6_meta/nt_nexus/checkpoint.rs:15` | `KnowledgeBase` from L5 (L5 re-exporting L1!) |
| L6→L5 | `l6_meta/healing/nt_mind_consciousness_monitor.rs:4-5` | `IITPhiCalculator`, `PhiReport` from L5 |
| L6→L5 | `l6_meta/healing/nt_mind_eval_harness.rs:7` | `EffortTier` from L5 |
| L6→L5 | `l6_meta/healing/nt_mind_consciousness_gold_standard.rs:6-7` | `OscillatorNetwork`, `IITPhiCalculator` from L5 |
| L6→L5 | `l6_meta/healing/nt_core_self_test_integration.rs:1,5,387` | `arch_fitness_tests`, `QuantumSignal` from L5 |
| L6→L5 | `l6_meta/coordination/cross_module_audit.rs:8` | `SegmentData`, `SegmentType` from L5 |

**关键发现**: L6→L5 的 18 个违规中，大多数是 L6 需要 L5 的具体实现类型 (如 `CoreSnapshot`, `IITPhiCalculator`, `OscillatorNetwork`)。这些类型**应该在 L0 定义 trait**，L5 实现 trait，L6 依赖 trait 而非实现。

#### 三重违规链 (L6→L5→L1)

```
l6_meta/nt_meta/knowledge_gap_detector.rs
  → l5_cognition::l1_facade (L5 重新导出 L1)
    → l1_action::nt_memory::nt_memory_kb::KnowledgeBase (L1 定义)

修复: L6 直接从 L1 导入 KnowledgeBase，不经过 L5 中转
```

---

## 三、融合架构设计 (修正版)

### 3.1 核心原则

```
原则 1: 类型单一事实源 → neotrix-types (所有公共类型)
原则 2: 依赖单向流动 → L0 → L1 → L2 → L3 → L4 → L5 → L6
原则 3: 跨层通信通过 L0 trait → 不直接导入
原则 4: 外部模式熔炼到现有模块 → 不创建平行适配器
原则 5: 每次迁移必须编译通过 → 零报错
原则 6: 模块本地类型保持本地 → 不强制合并
```

### 3.2 类型统一方案 (修正版)

#### 3.2.1 TaskType 统一 (3 → 1)

```rust
// crates/neotrix-types/src/core/nt_core_knowledge/types.rs
// 规范定义保持不变 (35+ 变体)

// 修改 god_agent.rs:
// BEFORE: enum TaskType { CodeGeneration, CodeReview, ... }  (本地定义)
// AFTER:  use crate::l2_perception::nt_core_knowledge::TaskType;
//         并使用 TaskType::CodeGeneration, TaskType::CodeReview 等

// 修改 production_pipeline.rs:
// BEFORE: enum TaskType { ScriptParsing, CharacterGeneration, ... }
// AFTER:  enum ProductionStage { ScriptParsing, CharacterGeneration, ... }
//         (重命名，消除同名异义)
```

#### 3.2.2 SelfTest 统一 (2 → 1)

```rust
// neotrix-core/src/l0_substrate/nt_core_self_test.rs
// 保留 L0 作为唯一定义源

// neotrix-core/src/l6_meta/healing/nt_core_self_test.rs
// BEFORE:
//   pub use crate::l0_substrate::nt_core_self_test::SelfTest;  // 只 re-export trait
//   pub struct SelfTestRegistry { ... }  // 自己的副本
//   pub struct SelfTestResult { ... }    // 自己的副本
//
// AFTER:
//   pub use crate::l0_substrate::nt_core_self_test::{
//       SelfTest, SelfTestRegistry, SelfTestResult,
//   };
//   // 保留 L6 特有实现
//   pub struct ConstitutionComplianceTest { ... }
//   pub struct ExternalVerifier { ... }
//   pub struct TraceEvaluationTest { ... }
```

#### 3.2.3 NodeType 重命名 (2 → 2, 不同域)

```rust
// crates/neotrix-types/src/knowledge_access.rs
// 保持不变: NodeType (知识图谱节点, 41 变体)

// crates/neotrix-types/src/core/node_canvas.rs
// BEFORE: enum NodeType { Text, Image, Code, ... }
// AFTER:  enum CanvasNodeType { Text, Image, Code, ... }
//         (消除同名异义)
```

---

## 四、熔炼方案: 外部模式 → 能力骨架映射 (具体实现)

### 4.1 NT-MEMORY: 熔炼 M1+M2+M4+M6

#### 4.1.1 M1: 分层内存管线 (来自 LightMem)

```rust
// neotrix-core/src/l1_action/nt_memory/tiered_pipeline/mod.rs

/// 分层内存管线 trait
pub trait TieredPipeline {
    /// 阶段 1: 预压缩 (LLMLingua-2 风格)
    fn compress(&self, raw: &str) -> CompressedChunk;
    
    /// 阶段 2: 主题分段
    fn segment(&self, chunk: &CompressedChunk) -> Vec<TopicSegment>;
    
    /// 阶段 3: 重要性提取 (extract_threshold 0.1-1.0)
    fn extract(&self, segments: &[TopicSegment], threshold: f64) -> Vec<MemoryEntry>;
    
    /// 阶段 4: 多模态索引 (向量 + BM25 + 实体)
    fn index(&self, entries: &[MemoryEntry]) -> IndexResult;
}

/// 压缩结果
pub struct CompressedChunk {
    pub text: String,
    pub compression_ratio: f64,
    pub preserved_tokens: usize,
}

/// 主题分段
pub struct TopicSegment {
    pub topic_id: u32,
    pub content: String,
    pub importance: f64,
    pub entities: Vec<String>,
}

/// 索引结果
pub struct IndexResult {
    pub vector_ids: Vec<u64>,
    pub bm25_terms: Vec<String>,
    pub entity_links: Vec<(String, String)>,
}
```

#### 4.1.2 M2: Git-for-Memory (来自 Memoria)

```rust
// neotrix-core/src/l1_action/nt_memory/git_memory/mod.rs

/// 内存存储 (Git-for-Memory 风格)
pub struct GitMemoryStore {
    /// 当前分支的内存
    current_branch: String,
    /// 所有分支
    branches: HashMap<String, MemoryBranch>,
    /// 提交历史
    history: Vec<MemoryCommit>,
    /// 治理配置
    governance: GovernanceConfig,
}

/// 内存分支
pub struct MemoryBranch {
    pub name: String,
    pub head: Option<MemoryCommit>,
    pub created_at: DateTime<Utc>,
    pub purpose: String, // "experiment", "stable", "archive"
}

/// 内存提交
pub struct MemoryCommit {
    pub hash: String,
    pub parent: Option<String>,
    pub entries: Vec<MemoryEntry>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// 治理配置
pub struct GovernanceConfig {
    /// 合并冷却期 (防止频繁合并)
    pub merge_cooldown: Duration,
    /// 冲突解决策略
    pub conflict_resolution: ConflictStrategy,
    /// 自动归档阈值
    pub auto_archive_threshold: usize,
}

impl GitMemoryStore {
    /// 创建快照 (每次写入)
    pub fn snapshot(&mut self) -> Result<String> { ... }
    
    /// 创建分支 (实验隔离)
    pub fn branch(&mut self, name: &str, purpose: &str) -> Result<()> { ... }
    
    /// 合并分支 (冲突检测 + 解决)
    pub fn merge(&mut self, source: &str, target: &str) -> Result<MergeResult> { ... }
    
    /// 回滚到指定提交
    pub fn rollback(&mut self, commit_hash: &str) -> Result<()> { ... }
    
    /// 冷却期检查
    pub fn can_merge(&self) -> bool { ... }
}
```

#### 4.1.3 M4: KV-Context 虚拟化 (来自 KVMem)

```rust
// neotrix-core/src/l1_action/nt_memory/paged_kv/mod.rs

/// 三级 KV 缓存管理器
pub struct PagedKVManager {
    /// GPU 层 (热数据)
    gpu_tier: Tier<GPUBackend>,
    /// Host 内存层 (温数据)
    host_tier: Tier<HostBackend>,
    /// NVMe 层 (冷数据)
    nvme_tier: Tier<NVMeBackend>,
    /// 注意力空间索引
    attention_index: AttentionIndex,
}

/// 缓存层
pub struct Tier<B: KVBackend> {
    backend: B,
    capacity: usize,
    used: usize,
    eviction_policy: EvictionPolicy,
}

/// KV 后端 trait
pub trait KVBackend {
    fn store(&mut self, key: &str, value: &[u8]) -> Result<()>;
    fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn evict(&mut self, keys: &[String]) -> Result<()>;
    fn stats(&self) -> TierStats;
}

/// 注意力空间索引 (查询相关物化)
pub struct AttentionIndex {
    /// 索引条目: (block_id, attention_score, last_accessed)
    entries: Vec<(u64, f64, DateTime<Utc>)>,
}

impl PagedKVManager {
    /// 查询: 使用注意力索引选择相关块，物化到 GPU 层
    pub fn query(&mut self, query: &str, budget: usize) -> Result<Vec<KVEntry>> { ... }
    
    /// 写入: 自动选择最佳层
    pub fn store(&mut self, key: &str, value: &[u8]) -> Result<()> { ... }
    
    /// 预取: 基于查询模式预测性加载
    pub fn prefetch(&mut self, query_pattern: &str) -> Result<()> { ... }
}
```

#### 4.1.4 M6: 压缩一致性保证 (来自 MemoryWalker)

```rust
// neotrix-core/src/l1_action/nt_memory/consistency/mod.rs

/// 压缩一致性保证
pub struct CompressionGuard {
    /// 学生模型 (压缩后)
    student: Box<dyn Model>,
    /// 教师模型 (无压缩, stop-gradient)
    teacher: Box<dyn Model>,
    /// KL 散度阈值
    kl_threshold: f64,
}

impl CompressionGuard {
    /// SDCC: 自蒸馏条件一致性
    pub fn sdcc_check(
        &self,
        compressed: &str,
        original: &str,
    ) -> Result<ConsistencyReport> {
        // 1. 计算学生和教师在压缩点的输出分布
        // 2. 计算前向 KL 散度
        // 3. 检查 O(sqrt(epsilon_KL)) TV gap bound
        // 4. 返回一致性报告
    }
    
    /// 压缩后验证
    pub fn verify_after_compression(
        &self,
        compressed_context: &str,
        task: &str,
    ) -> Result<bool> {
        // 验证压缩后的上下文仍能正确执行任务
    }
}
```

### 4.2 NT-MIND: 熔炼 A1+A2+A7+T5

#### 4.2.1 A1: JIT-Harness 生成 (来自 JIT-Agent)

```rust
// neotrix-core/src/l5_cognition/nt_mind/jit_harness/mod.rs

/// JIT-Harness 协议 (4 模块)
pub trait HarnessProtocol {
    /// 内存管理模块
    fn memory_module(&self) -> Box<dyn MemoryModule>;
    /// 规划策略模块
    fn planning_module(&self) -> Box<dyn PlanningModule>;
    /// 动作协议模块
    fn action_module(&self) -> Box<dyn ActionModule>;
    /// 工具/技能编排模块
    fn tool_module(&self) -> Box<dyn ToolModule>;
}

/// 按任务合成 harness
pub struct HarnessSynthesizer {
    /// 历史 harness 性能档案
    archive: Vec<HarnessRecord>,
    /// 自修复能力
    self_repair: SelfRepairEngine,
}

impl HarnessSynthesizer {
    /// 为特定任务合成最优 harness
    pub fn synthesize(&self, task: &TaskType) -> Box<dyn HarnessProtocol> {
        // 1. 分析任务特征 (新颖性 × 复杂度)
        // 2. 从 archive 中检索相似任务的 harness
        // 3. 调整 4 模块参数
        // 4. 返回合成的 harness
    }
    
    /// 自修复: 当 harness 执行失败时自动调整
    pub fn self_repair(&mut self, failed: &dyn HarnessProtocol, error: &Error) {
        // 1. 分析失败原因
        // 2. 调整相关模块参数
        // 3. 重新合成
        // 4. 记录修复经验到 archive
    }
}
```

#### 4.2.2 A2: 双轨自治 (来自 CrewAI)

```rust
// neotrix-core/src/l5_cognition/nt_mind/dual_track/mod.rs

/// 双轨路由器
pub struct DualTrackRouter {
    /// 新颖性×复杂度矩阵
    novelty_complexity: NoveltyComplexityMatrix,
}

/// 新颖性×复杂度矩阵
pub struct NoveltyComplexityMatrix {
    /// 新颖性分数 (0.0-1.0)
    novelty: f64,
    /// 复杂度分数 (0.0-1.0)
    complexity: f64,
}

impl DualTrackRouter {
    /// 路由到正确的轨道
    pub fn route(&self, task: &TaskType) -> Track {
        let n = self.novelty_complexity.novelty;
        let c = self.novelty_complexity.complexity;
        
        match (n > 0.5, c > 0.5) {
            (false, false) => Track::Flow,      // 低新颖性+低复杂度 → 确定性流
            (false, true)  => Track::Crew,      // 低新颖性+高复杂度 → 自主团队
            (true, false)  => Track::Flow,      // 高新颖性+低复杂度 → 确定性流 (有模板)
            (true, true)   => Track::Crew,      // 高新颖性+高复杂度 → 自主团队
        }
    }
}

/// 自主轨道 (Crews)
pub mod crew {
    pub struct AgentCrew {
        pub agents: Vec<AgentRole>,
        pub process: Process, // Sequential 或 Hierarchical
    }
    
    pub struct AgentRole {
        pub role: String,
        pub goal: String,
        pub backstory: String,
        pub tools: Vec<Box<dyn Tool>>,
    }
}

/// 确定性轨道 (Flows)
pub mod flow {
    pub struct Workflow {
        pub steps: Vec<Step>,
        pub triggers: Vec<Trigger>,
    }
    
    pub struct Step {
        pub action: Box<dyn Action>,
        pub next: Vec<(Condition, StepId)>,
    }
}
```

#### 4.2.3 T5: Dreaming 巩固 (来自 Skales)

```rust
// neotrix-core/src/l5_cognition/nt_mind/dreaming/mod.rs

/// 夜间巩固引擎
pub struct DreamConsolidator {
    /// 巩固配置
    config: DreamConfig,
    /// 梦境日记
    diary: DreamDiary,
}

pub struct DreamConfig {
    /// 重要性阈值 (高于此值的记忆被提升)
    pub importance_threshold: f64,
    /// 遗忘阈值 (低于此值的记忆被丢弃)
    pub forget_threshold: f64,
    /// 巩固冷却期
    pub consolidation_cooldown: Duration,
    /// 最大保留记忆数
    pub max_memories: usize,
}

/// 梦境日记
pub struct DreamDiary {
    /// 每日巩固记录
    entries: Vec<DreamEntry>,
}

pub struct DreamEntry {
    pub date: NaiveDate,
    pub promoted: Vec<MemoryEntry>,
    pub forgotten: Vec<MemoryEntry>,
    pub insights: Vec<String>,
}

impl DreamConsolidator {
    /// 执行夜间巩固
    pub fn consolidate(&mut self, memories: &mut Vec<MemoryEntry>) -> DreamReport {
        let mut promoted = Vec::new();
        let mut forgotten = Vec::new();
        
        for memory in memories.iter() {
            if memory.importance > self.config.importance_threshold {
                // 提升: 标记为长期记忆
                promoted.push(memory.clone());
            } else if memory.importance < self.config.forget_threshold {
                // 遗忘: 从活跃记忆中移除
                forgotten.push(memory.clone());
            }
        }
        
        // 记录梦境日记
        self.diary.record(DreamEntry {
            date: Utc::now().date_naive(),
            promoted,
            forgotten,
            insights: self.extract_insights(memories),
        });
        
        DreamReport { promoted_count: promoted.len(), forgotten_count: forgotten.len() }
    }
    
    /// 逐任务教训蒸馏
    pub fn distill_lessons(&self, task_result: &TaskResult) -> Vec<Lesson> {
        // 从任务结果中提取可复用的教训
        // 每个教训包含: 触发条件、行动、结果、置信度
    }
}
```

### 4.3 NT-ACT: 熔炼 A4+A6+T1

#### 4.3.1 T1: 输出压缩管线 (来自 RTK, 已是 Rust)

```rust
// neotrix-core/src/l1_action/nt_act/output_compression/mod.rs

/// 输出压缩 trait
pub trait OutputFilter {
    /// 过滤: 移除无关行
    fn filter(&self, output: &str) -> String;
    
    /// 分组: 相似行合并
    fn group(&self, output: &str) -> String;
    
    /// 截断: 限制最大长度
    fn truncate(&self, output: &str, max_chars: usize) -> String;
    
    /// 去重: 移除重复行
    fn dedup(&self, output: &str) -> String;
    
    /// 组合: 按顺序执行所有策略
    fn compress(&self, output: &str, config: &CompressionConfig) -> String {
        let filtered = self.filter(output);
        let grouped = self.group(&filtered);
        let truncated = self.truncate(&grouped, config.max_chars);
        self.dedup(&truncated)
    }
}

/// 压缩配置
pub struct CompressionConfig {
    pub max_chars: usize,
    pub filter_patterns: Vec<String>,
    pub dedup_threshold: f64,
}

/// 回忆机制: 完整输出检索
pub struct OutputRecall {
    /// SQLite 存储完整输出
    db: SqlitePool,
}

impl OutputRecall {
    /// 保存完整输出 (压缩前)
    pub async fn save(&self, command: &str, full_output: &str) -> Result<()> { ... }
    
    /// 回忆完整输出
    pub async fn recall(&self, command: &str) -> Result<Option<String>> { ... }
}
```

#### 4.3.2 A6: 非阻塞子代理生成 (来自 pi-crew)

```rust
// neotrix-core/src/l1_action/nt_act/non_blocking_spawn/mod.rs

/// 非阻塞生成器
pub struct SubAgentSpawner {
    /// 代理原型注册表
    archetypes: HashMap<String, AgentArchetype>,
}

/// 代理原型
pub struct AgentArchetype {
    pub name: String,
    pub role: String,
    pub tools: Vec<String>,
    pub model_preference: String,
}

/// 生成结果
pub struct SpawnResult {
    pub task_id: String,
    pub receiver: tokio::sync::oneshot::Receiver<AgentResult>,
}

impl SubAgentSpawner {
    /// 非阻塞生成 (立即返回)
    pub fn spawn(
        &self,
        archetype: &str,
        task: String,
        context: String,
    ) -> Result<SpawnResult> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let archetype = self.archetypes.get(archetype).cloned()
            .ok_or_else(|| anyhow!("Unknown archetype: {}", archetype))?;
        
        tokio::spawn(async move {
            let result = execute_agent(&archetype, &task, &context).await;
            let _ = tx.send(result);
        });
        
        Ok(SpawnResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            receiver: rx,
        })
    }
    
    /// 等待结果 (阻塞)
    pub async fn await_result(result: SpawnResult) -> Result<AgentResult> {
        result.receiver.await.map_err(|e| anyhow!("Agent task cancelled: {}", e))?
    }
}

/// 工具增量: +tool 模式 (不替换现有工具列表)
pub struct ToolDelta {
    pub additions: Vec<String>,
    pub removals: Vec<String>,
}

impl ToolDelta {
    /// 应用增量到工具列表
    pub fn apply(&self, current: &[String]) -> Vec<String> {
        let mut result: Vec<String> = current.to_vec();
        result.extend(self.additions.clone());
        result.retain(|t| !self.removals.contains(t));
        result
    }
}
```

### 4.4 GWT: 熔炼 C1+C2

#### 4.4.1 C1: 证据驱动信任门控

```rust
// neotrix-core/src/l5_cognition/nt_core_gwt/evidence_gating/mod.rs

/// 证据门控器
pub struct EvidenceGating {
    /// 结果追踪器
    outcome_tracker: OutcomeTracker,
    /// 信任校准器
    trust_calibrator: TrustCalibrator,
}

/// 结果追踪器
pub struct OutcomeTracker {
    /// 每个代理的历史结果
    outcomes: HashMap<String, Vec<Outcome>>,
}

pub struct Outcome {
    pub task_id: String,
    pub success: bool,
    pub quality_score: f64,
    pub timestamp: DateTime<Utc>,
    pub cost: f64,
}

/// 信任校准器
pub struct TrustCalibrator {
    /// 基础信任分数
    base_trust: f64,
    /// 衰减因子
    decay_factor: f64,
    /// 恢复速率
    recovery_rate: f64,
}

impl TrustCalibrator {
    /// 根据结果更新信任分数
    pub fn update(&mut self, agent_id: &str, outcome: &Outcome) -> f64 {
        let current = self.outcomes.get(agent_id)
            .map(|o| self.calculate_trust(o))
            .unwrap_or(self.base_trust);
        
        if outcome.success {
            // 成功: 信任增加 (有上限)
            (current + self.recovery_rate * outcome.quality_score).min(1.0)
        } else {
            // 失败: 信任衰减
            current * self.decay_factor
        }
    }
    
    /// 门控: 信任分数低于阈值时阻止执行
    pub fn gate(&self, agent_id: &str) -> bool {
        let trust = self.calculate_trust(
            self.outcomes.get(agent_id).unwrap_or(&vec![])
        );
        trust > 0.3 // 信任阈值
    }
}
```

#### 4.4.2 C2: 成本阶梯 (来自 Hermes)

```rust
// neotrix-core/src/l5_cognition/nt_core_gwt/cost_ladder/mod.rs

/// 成本阶梯路由器
pub struct CostLadder {
    /// 梯队定义
    rungs: Vec<Rung>,
    /// 新颖性×复杂度矩阵
    matrix: NoveltyComplexityMatrix,
}

pub struct Rung {
    pub name: String,        // "Tutti" (便宜), "Soloist" (中等), "Conductor" (昂贵)
    pub model_tier: String,  // "cheap", "medium", "expensive"
    pub max_cost_per_task: f64,
    pub capabilities: Vec<String>,
}

impl CostLadder {
    /// 根据任务特征分配梯队
    pub fn assign_rung(&self, task: &TaskType) -> &Rung {
        let n = self.matrix.novelty(task);
        let c = self.matrix.complexity(task);
        
        // 简单任务 (低新颖性+低复杂度) → 便宜梯队
        if n < 0.3 && c < 0.3 {
            return &self.rungs[0]; // Tutti
        }
        
        // 中等任务 → 中等梯队
        if n < 0.7 && c < 0.7 {
            return &self.rungs[1]; // Soloist
        }
        
        // 复杂任务 (高新颖性或高复杂度) → 昂贵梯队
        &self.rungs[2] // Conductor
    }
}

/// 疲劳检测 + 热交换
pub struct FatigueDetector {
    /// 每个代理的使用统计
    usage: HashMap<String, UsageStats>,
}

pub struct UsageStats {
    pub total_tasks: u64,
    pub recent_failures: u64,
    pub avg_latency: Duration,
    pub cost_accumulated: f64,
}

impl FatigueDetector {
    /// 检测代理是否疲劳
    pub fn is_fatigued(&self, agent_id: &str) -> bool {
        let stats = self.usage.get(agent_id);
        match stats {
            None => false,
            Some(s) => {
                let failure_rate = s.recent_failures as f64 / s.total_tasks as f64;
                failure_rate > 0.3 || s.avg_latency > Duration::from_secs(30)
            }
        }
    }
    
    /// 热交换: 将疲劳代理替换为健康代理
    pub fn hot_swap(&self, fatigued: &str, available: &[String]) -> Option<String> {
        available.iter()
            .find(|id| !self.is_fatigued(id))
            .cloned()
    }
}
```

---

## 五、零报错迁移方案 (修正版)

### 5.1 实际迁移复杂度

| 任务 | v4.0 报告 | v4.1 修正 | 说明 |
|------|----------|----------|------|
| TaskType 统一 | 15 文件 | **2 文件** | god_agent + production_pipeline |
| TaskStatus 统一 | 20 文件 | **0 文件** | 无需合并 (模块本地) |
| NodeType 统一 | 8 文件 | **1 文件** | node_canvas 重命名 |
| SelfTest 统一 | ~70+ 文件 | **1 文件** (L6 self_test.rs) | re-export 保证向后兼容 |
| L5→L6 shim 移除 | 30+ 文件 | **3 文件** | 仅 1 个消费者 + shim 本身 |
| KB types 路径优化 | 30+ 文件 | **27 文件** | 1 个新文件 + 26 个路径替换 |
| CapabilityVector | 10+ 文件 | **10 文件** | 机械替换 |
| SpecialistType | 1+ 文件 | **1 文件** | 机械替换 |
| L1→L5 trait 抽象 | 24 违规 | **~15 文件** | 需要设计 4 个 trait |
| L6→L5 路径修复 | 18 违规 | **~12 文件** | 路径重定向 + 2 个 trait |
| L6→L5→L1 三重违规 | 2 违规 | **2 文件** | 直接从 L1 导入 |

### 5.2 修正后任务清单 (18 天)

#### Phase 0: 精确类型修复 (2 天)

| 任务 | 说明 | 文件数 | 验证 |
|------|------|--------|------|
| T0.1 | god_agent TaskType → 使用规范定义 | 1 | Gate 0 |
| T0.2 | production_pipeline TaskType → 重命名为 ProductionStage | 1 | Gate 0 |
| T0.3 | node_canvas NodeType → 重命名为 CanvasNodeType | 1 | Gate 0 |
| T0.4 | 编译验证 | - | Gate 0 |

#### Phase 1: 跨层修复 (5 天)

| 任务 | 说明 | 违规数 | 验证 |
|------|------|--------|------|
| T1.1 | 移除 L5→L6 shim (nt_meta/mod.rs) + 修复 1 个消费者 | 1 | Gate 2 |
| T1.2 | SelfTest: L6 re-export L0 + 删除重复类型 + 修复 TEST_ENV_LOCK bug | 7 类型 | Gate 0 |
| T1.3 | KB 类型: 创建 core/nt_core_kb_types.rs + 迁移 26 个消费者 | 26 | Gate 2 |
| T1.4 | SelfTest 路径: 可选迁移 133 个消费者从 L6→L0 路径 | 133 | Gate 2 |
| T1.5 | CapabilityVector: 10 个文件改为从 core 导入 | 10 | Gate 2 |
| T1.6 | SpecialistType: 1 个文件改为从 core 导入 | 1 | Gate 2 |
| T1.7 | 修复 L1→L5 直接导入 (24 违规) — 需要 trait 抽象 | 24 | Gate 2 |
| T1.8 | 修复 L6→L5 双向依赖 (18 违规) — 需要 trait 抽象或路径重定向 | 18 | Gate 2 |
| T1.9 | 修复 L6→L5→L1 三重违规 (2 违规) — 直接从 L1 导入 | 2 | Gate 2 |
| T1.10 | 编译验证 | - | Gate 0 |

**Phase 1 执行顺序** (依赖关系):
```
T1.1 (L5→L6 shim) → T1.2 (SelfTest) → T1.3 (KB types) → T1.4 (SelfTest 路径)
                    ↓
              T1.5 (CapabilityVector) + T1.6 (SpecialistType) [并行]
                    ↓
              T1.7 (L1→L5) + T1.8 (L6→L5) + T1.9 (三重违规) [并行, 需 trait 设计]
                    ↓
              T1.10 (编译验证)
```

**T1.2 详细步骤** (SelfTest 合并):
```
修改文件: l6_meta/healing/nt_core_self_test.rs (399→~100 行)
1. 删除 SelfTestRegistry 副本 (L6:17-76)
2. 删除 SelfTestResult 副本 (L6:78-114)
3. 删除 report() 副本 (L6:116-128)
4. 删除 DurationDriftMonitor 副本 (L6:134-206)
5. 删除 DurationDriftTest 副本 (L6:209-242)
6. 删除 TEST_ENV_LOCK 副本 (L6:14) — 修复锁安全 bug
7. 添加 re-export:
   pub use crate::l0_substrate::nt_core_self_test::{
       SelfTest, SelfTestRegistry, SelfTestResult,
       DurationDriftMonitor, DurationDriftTest,
       report, TEST_ENV_LOCK,
   };
8. 保留 ExternalVerifier (L6:244-273)
9. 保留 ConstitutionComplianceTest (L6:275-319)
10. 保留 TraceEvaluationTest (L6:321-399)

可选: core/nt_core_traits.rs:13 改为从 L0 直接导入 SelfTest

影响: 0 个消费者文件需要修改 (re-export 保证向后兼容)
净减少: ~299 行代码 (-40%)
```

**T1.3 详细步骤** (KB types 迁移):
```
步骤 1: 创建 core/nt_core_kb_types.rs
   pub use neotrix_types::knowledge_access::{KnowledgeNode, NodeType, RelationType};

步骤 2: core/mod.rs 添加 pub mod nt_core_kb_types;

步骤 3: 全局替换 26 个文件:
   crate::l6_meta::nt_core_kb_types → crate::core::nt_core_kb_types

步骤 4: core/nt_core_traits.rs KnowledgeSink trait 签名更新 (L110, L121, L131, L142)

步骤 5: l6_meta/nt_core_kb_types.rs 保留为临时 shim (带 deprecation 注释)
         可在 Phase 4 删除
```

**T1.7/T1.8 详细步骤** (trait 抽象):
```
需要创建的 L0 trait:

1. core::RevertibleEffect trait
   替代: ClosureEffect, RevertibleContext (L5→L1 违规)
   消费者: nt_io_plugin/registry.rs, nt_memory_kb/nt_memory_coeffect.rs

2. core::EvidenceChainProvider trait
   替代: EvidenceChain (L5→L1 违规)
   消费者: nt_memory_historian/nt_evidence_store.rs

3. core::HiveCoordinator trait
   替代: HiveRouter, HiveMessage (L5→L1 违规)
   消费者: nt_io_hive_agent_loop.rs

4. core::ReasoningKernel trait (抽象)
   替代: 具体 ReasoningKernel 类型 (L1→L5 违规)
   消费者: nt_core_task_dispatcher.rs, nt_io_standalone.rs

L6→L5 修复 (路径重定向):
- CoreSnapshot → 从 L5 直接导入 (不再通过 L5 的 L6 shim)
- KnowledgeBase → 从 L1 直接导入 (不经过 L5 中转)
- IITPhiCalculator/PhiReport → 创建 L0 trait 或从 L5 直接导入
```

#### Phase 2: 模块熔炼 (8 天)

| 任务 | 说明 | 模式 | 验证 |
|------|------|------|------|
| T2.1 | NT-MEMORY: tiered_pipeline | M1 | Gate 3 |
| T2.2 | NT-MEMORY: git_memory | M2 | Gate 3 |
| T2.3 | NT-MEMORY: paged_kv | M4 | Gate 3 |
| T2.4 | NT-MIND: jit_harness | A1 | Gate 3 |
| T2.5 | NT-MIND: dual_track | A2 | Gate 3 |
| T2.6 | NT-MIND: dreaming | T5 | Gate 3 |
| T2.7 | NT-ACT: output_compression + recall | T1 | Gate 3 |
| T2.8 | NT-ACT: non_blocking_spawn | A6 | Gate 3 |
| T2.9 | GWT: cost_ladder + evidence_gating | C1+C2 | Gate 3 |

#### Phase 3: 集成验证 + 巡检 (3 天)

| 任务 | 说明 | 验证 |
|------|------|------|
| T3.1 | 全系统编译验证 | Gate 0 |
| T3.2 | 单元测试 + 集成测试 | Gate 4 |
| T3.3 | 影子对比端到端测试 | Gate 3 |
| T3.4 | 多 Agent 巡检 (编译/类型/依赖/测试/安全) | Gate 4 |
| T3.5 | 修复所有巡检发现 | Gate 0 |

---

## 六、多 Agent 自动巡检修复方案 (具体配置)

### 6.1 巡检 Agent 配置

```
巡检 Squad (4 agents):
├── debug-agent (编译/测试修复)
│   ├── 工具: cargo check, cargo test, cargo clippy, cargo fix
│   ├── 触发: 编译错误, 测试失败, clippy 警告
│   ├── 修复策略:
│   │   ├── 简单错误 (类型不匹配, 导入缺失) → 自动修复
│   │   ├── 中等错误 (trait 不满足) → 生成修复代码
│   │   └── 复杂错误 (循环依赖) → 升级到 general-agent
│   └── 验证: 每次修复后重新 cargo check
│
├── explore-agent (结构/依赖分析)
│   ├── 工具: grep, cargo tree, cargo deny, 自定义脚本
│   ├── 触发: 重复定义, 循环依赖, 跨层违规
│   ├── 分析维度:
│   │   ├── 类型重复: grep "enum <TypeName>" | wc -l
│   │   ├── 依赖循环: cargo tree --workspace --edges normal
│   │   ├── 跨层违规: 自定义导入路径检查脚本
│   │   └── dead_code: cargo udeps
│   └── 输出: 结构化报告 (JSON)
│
├── review-agent (质量/安全审查)
│   ├── 工具: cargo audit, cargo deny, 自定义影子对比
│   ├── 触发: 安全漏洞, 性能退化, 功能不一致
│   ├── 审查维度:
│   │   ├── 安全: cargo audit → 0 vulnerabilities
│   │   ├── 许可: cargo deny → 0 advisories
│   │   ├── 影子对比: 新旧架构输出 >99% 一致
│   │   └── 性能: 编译时间 + 运行时基准
│   └── 输出: 审查报告 (含修复建议)
│
└── general-agent (综合修复)
    ├── 工具: 全部 + git 操作
    ├── 触发: 复杂修复需要多步骤
    ├── 修复流程:
    │   1. 整合所有巡检发现
    │   2. 按影响范围排序
    │   3. 按依赖顺序修复
    │   4. 每步验证编译
    │   5. 最终全量测试
    └── 输出: 修复提交 + 验证报告
```

### 6.2 巡检流程 (每 Phase)

```
Phase 结束时自动触发:
│
├── Step 1: debug-agent → cargo check --all-targets
│   ├── 0 error → 继续
│   └── >0 error → 自动修复 → 重新检查 → 最多 3 轮 → 升级
│
├── Step 2: explore-agent → 结构分析
│   ├── grep 重复类型 → 生成报告
│   ├── cargo tree → 检测循环依赖
│   └── 跨层导入检查 → 生成修复建议
│
├── Step 3: review-agent → 质量审查
│   ├── cargo clippy → 0 warning
│   ├── cargo audit → 0 vulnerabilities
│   └── 影子对比 → >99% 一致性
│
└── Step 4: general-agent → 综合修复
    ├── 整合 Steps 1-3 发现
    ├── 按优先级排序 (P0 > P1 > P2)
    ├── 执行修复
    └── 重新验证全部
```

---

## 七、总工期估算 (修正版)

| Phase | 内容 | v4.0 | v4.1 | 说明 |
|-------|------|------|------|------|
| Phase 0 | 精确类型修复 | 4 天 | **2 天** | 大部分类型无需合并 |
| Phase 1 | 跨层修复 | 6 天 | **5 天** | SelfTest 合并 + trait 抽象 |
| Phase 2 | 模块熔炼 | 10 天 | **8 天** | 精简到最高价值模式 |
| Phase 3 | 集成验证 + 巡检 | 4 天 | **3 天** | 自动化巡检 |
| **总计** | | **28 天** | **18 天** | **减少 36%** |

### 7.1 关键路径分析

```
Phase 0 (2 天) → Phase 1 前半 (3 天: SelfTest + KB + 路径) → Phase 1 后半 (2 天: trait 抽象)
                        ↓
              Phase 2 (8 天: 模块熔炼) → Phase 3 (3 天: 集成验证)

关键路径: T0.1-0.3 → T1.2 (SelfTest) → T1.3 (KB types) → T1.7 (L1→L5 trait) → T2.1-2.9 → T3.1-3.5
最短路径: 2 天 + 5 天 + 8 天 + 3 天 = 18 天
```

### 7.2 风险缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| Phase 1 trait 设计返工 | 中 | 高 | 先实现简单版本，后续迭代 |
| Phase 2 模块间依赖 | 中 | 中 | 每个模块独立编译验证 |
| Phase 3 影子对比失败 | 低 | 高 | 保留旧路径作为 fallback |

---

## 八、成功标准 (修正版)

| 标准 | v4.0 目标 | v4.2 目标 | 验证 |
|------|----------|----------|------|
| 编译通过 | 0 error | 0 error | cargo check --workspace |
| TaskType 定义 | 1 | 1 | grep 统计 |
| TaskStatus 定义 | 1 | **7** (保持) | 模块本地，无需合并 |
| SelfTestRegistry | 1 | **1** (含 TEST_ENV_LOCK 单实例) | L6 re-export L0 |
| TEST_ENV_LOCK | 2 (bug) | **1** (修复) | 单实例，所有消费者同步 |
| L5→L6 shim | 1 | **0** (删除) | 依赖单向流动 |
| 跨层违规 | 0 | **~50** (从 228 降低 78%) | trait 抽象覆盖高频违规 |
| dead_code | 0 | 0 | cargo udeps |
| 重复 route() | 10 | **10** | grep 统计 |
| 测试通过 | 100% | 100% | cargo test --workspace |
| 影子对比 | >99% | >99% | shadow_comparison |
| 编译时间 | 不增加 50% | 不增加 50% | cargo build 时间对比 |
| 外部模式吸收 | 15 个 | **9 个** | 代码审查 (最高价值) |
| 多 Agent 巡检 | 每 Phase | 每 Phase | 巡检报告 |

---

## 九、产出文件清单

| 文件 | 内容 | 状态 |
|------|------|------|
| `docs/FUSION-ARCHITECTURE-v4.md` | 融合架构 v4.2 (精确版, TEST_ENV_LOCK bug + 跨层精确分析 + 18天计划) | ✅ 本文件 |
| `docs/FUSION-ARCHITECTURE-v4.md` | 融合架构 v4.0 (初版) | ✅ 已完成 |
| `docs/FULL-MIGRATION-ROADMAP-v3.md` | 迁移路线图 v3.0 | ✅ 已完成 |
| `docs/FULL-MIGRATION-ROADMAP-v2.md` | 迁移路线图 v2.0 | ✅ 已完成 |
| `docs/MIGRATION-ANALYSIS.md` | 旧→新迁移分析 | ✅ 已完成 |
| `docs/IMPLEMENTATION-PLAN-MINIMAL.md` | 精简实施方案 | ✅ 已完成 |
| `docs/REDUNDANCY-CLEANUP-RESTRUCTURE.md` | 冗余清理设计 | ✅ 已完成 |
| `docs/UNIVERSAL-FRAMEWORK-SYNTHESIS.md` | 500+ URL 吸收报告 | ✅ 已完成 |

---

## 附录 A: TEST_ENV_LOCK 双静态 Bug 详细分析

### A.1 问题描述

```rust
// L0: neotrix-core/src/l0_substrate/nt_core_self_test.rs:17
#[cfg(test)]
static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

// L6: neotrix-core/src/l6_meta/healing/nt_core_self_test.rs:14
#[cfg(test)]
static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
```

这是**两个独立的进程级 Mutex**。在 Rust 中，`static` 变量是按路径区分的 — `crate::l0_substrate::TEST_ENV_LOCK` 和 `crate::l6_meta::healing::TEST_ENV_LOCK` 是两个完全不同的锁。

### A.2 影响范围

| 消费者文件 | 导入路径 | 获取的锁 |
|-----------|---------|---------|
| `l1_action/nt_memory/nt_memory_kb/nt_memory_kb_cmds.rs` | L6 路径 | L6 的锁 |
| `l6_meta/nt_core_cipher.rs` | L6 路径 | L6 的锁 |
| `l6_meta/healing/handlers_consciousness.rs` | L6 路径 | L6 的锁 |
| `l0_substrate/nt_core_self_test.rs` (测试) | L0 路径 | L0 的锁 |
| `l0_substrate/nt_core_cache.rs` (测试) | L0 路径 | L0 的锁 |

### A.3 后果

```
线程 A: 从 L6 路径获取 TEST_ENV_LOCK → 进入测试环境
线程 B: 从 L0 路径获取 TEST_ENV_LOCK → 也进入测试环境 (不被阻塞!)
结果: 两个线程同时修改测试环境 → 竞态条件
```

### A.4 修复方案

合并后只有一个 `TEST_ENV_LOCK` (在 L0)，所有消费者通过 re-export 统一到同一个锁：

```rust
// L6: pub use crate::l0_substrate::nt_core_self_test::TEST_ENV_LOCK;
// 现在所有消费者都获取同一个 Mutex
```

### A.5 验证

修复后，所有 `use crate::l6_meta::healing::nt_core_self_test::TEST_ENV_LOCK` 的消费者都会解析到 `crate::l0_substrate::nt_core_self_test::TEST_ENV_LOCK` — 同一个锁实例，竞态条件消除。

---

## 附录 B: 跨层违规 Top-10 修复清单

| # | 违规 | 源文件 | 目标 | 修复方案 |
|---|------|--------|------|---------|
| 1 | L5→L6 shim | `l5_cognition/nt_core/nt_meta/mod.rs` | L6 | 删除 shim，1 个消费者改路径 |
| 2 | L6→L5 (KnowledgeBase) | `l6_meta/nt_meta/knowledge_gap_detector.rs:8` | L5→L1 | 直接从 L1 导入 |
| 3 | L6→L5 (KnowledgeBase) | `l6_meta/nt_nexus/checkpoint.rs:15` | L5→L1 | 直接从 L1 导入 |
| 4 | L6→L5 (CoreSnapshot) | `l6_meta/memory/meta_observer.rs:14` | L5 | 创建 L0 trait 或从 L5 直接导入 |
| 5 | L6→L5 (CoreSnapshot) | `l6_meta/memory/transcendent_loop.rs:18` | L5 | 同上 |
| 6 | L6→L5 (CoreSnapshot) | `l6_meta/memory/consonance_orchestrator.rs:14` | L5 | 同上 |
| 7 | L6→L5 (IITPhiCalculator) | `l6_meta/healing/nt_mind_consciousness_monitor.rs:4-5` | L5 | 创建 L0 trait |
| 8 | L6→L5 (OscillatorNetwork) | `l6_meta/healing/nt_mind_consciousness_gold_standard.rs:6-7` | L5 | 创建 L0 trait |
| 9 | L1→L5 (ClosureEffect) | `l1_action/nt_io/nt_io_plugin/registry.rs` | L5 | 创建 L0 trait |
| 10 | L1→L5 (ClosureEffect) | `l1_action/nt_memory/nt_memory_kb/nt_memory_coeffect.rs` | L5 | 创建 L0 trait |

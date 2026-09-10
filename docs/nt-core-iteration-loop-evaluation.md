# NeoTrix 全量迭代评测报告 (Iteration Loop v2.0)

## 基线指标
| 指标 | 数值 |
|------|------|
| 总文件数 | 1,878 |
| 总代码行数 | 625,207 |
| 编译错误 | 0 (clean) |
| 编译警告 | 0 (deny(warnings)) |
| SelfTest 实现 | 122 |
| Config 结构体 | 350 |
| Gateway 模块 | 25 文件 / 7,518 行 / 55 结构体 |
| 重复 Trait | 9 个 (跨域定义) |
| 跨层违规 | 100+ 处 (L2→L3, L5→L1 等) |

## 三大问题诊断

### 1. 聚焦冗余 (Focus Redundancy) — 9 个重复 Trait

| 重复 Trait | 域 A | 域 B | 影响 |
|-----------|------|------|------|
| EvolutionLoopProvider | L1 nt_act_code | L5 nt_mind | L1→L5 跨层桥接 |
| ReasoningProvider | L5 nt_mind | core | 推理接口双重定义 |
| Orchestrator | L1 nt_act | L5 nt_mind | 编排逻辑重复 |
| AgentExecutor | L1 nt_act | core | 执行器接口分裂 |
| AbsorbValidator | L2 nt_world | L5 nt_mind | 吸收验证重复 |
| UserDistillation | L5 nt_mind | core | 蒸馏接口重复 |
| SessionRecovery | L6 meta | core | 恢复接口重复 |
| SelfIteration | L5 nt_mind | core | 自迭代接口重复 |
| Eli5Explainer | L5 nt_mind | core | 解释器接口重复 |

**根因**: L1 为了解耦 L5 定义了桥接 trait，但 L5 也有自己的版本 → 两套接口并存

### 2. 扁平缺陷 (Flattening Defects) — 350 个 Config 结构体

| 域 | Config 数 | 影响 |
|----|----------|------|
| L1 actions | 23 | 每个 action 独立 Config，无聚合层 |
| L5 consciousness_core | 21 | 认知核心 Config 碎片化 |
| L3 shield_impl | 19 | 安全模块内部无子域划分 |
| core | 19 | 核心层 Config 泛滥 |
| L1 memory_kb | 16 | 记忆模块 Config 碎片化 |
| L3 stealth_net | 15 | 隐身网络 Config 碎片化 |
| L6 coordination | 14 | 协调层 Config 碎片化 |
| L1 io | 14 | IO 层 Config 碎片化 |

**根因**: 缺少统一配置模型 → 每个模块自建 Config → 350 个独立结构体

### 3. 跨域错位 (Cross-Domain Misalignment) — 100+ 跨层导入

| 违规类型 | 数量 | 示例 |
|----------|------|------|
| L2→L3 (感知→具身) | ~30 | osint 导入 shield redaction/receipt |
| L2→L1 (感知→行动) | ~20 | world 导入 memory_kb/knowledge |
| L5→L1 (认知→行动) | ~15 | mind 导入 act code/evolution |
| L5→L3 (认知→具身) | ~10 | mind 导入 shield poc_engine |
| L6→L1 (元认知→行动) | ~5 | meta 导入 io provider |

**根因**: 层级边界定义模糊 → 职责渗透 → 一处修改影响多层

## galaxy-tree 架构融合点

| galaxy-tree 模式 | NeoTrix 映射 | 状态 | 优先级 |
|-----------------|-------------|------|--------|
| P1: Model Routing | GWT salience + cost weight | ✅ | - |
| P2: Isolation-per-Task | Worktree isolation | ⚠️ 部分 | P1 |
| P3: Profile-Driven | SelfModel extension | ✅ | - |
| P4: Ordered Backend Fallback | Ordered Backend Router | ✅ | - |
| P5: Skill as Template | SKILL-SPEC.md contract | ✅ | - |
| 0.26 Cost-Aware Routing | GWT + token cost | ✅ | - |
| 0.26 Context Virtualization | KVMem paged KV | ⚠️ 部分 | P1 |
| C.44 Dual-Temporal Facts | Graphiti + MELD | ❌ 未实现 | P2 |
| C.45 Unified Document IR | AnyDoc + MarkItDown | ⚠️ 部分 | P2 |
| 0.41b Agent Framework | SEAL pipeline | ✅ | - |
| 0.41d Reasoning & CoT | E8 + reasoning_engine | ✅ | - |
| 0.41l Meta-Cognition | ConsciousnessTree | ✅ | - |
| 0.41n MCTS Search | Bayesian experiment | ⚠️ 部分 | P2 |
| 0.41o Swarm Behavior | EventBus + ActorRef | ⚠️ 部分 | P3 |

## 核心路线任务清单

### Phase 1: 冗余清理 (立即可做)
- [ ] **R1**: 合并 9 个重复 Trait — 统一到 core/ 或 L5，删除 L1 桥接版本
- [ ] **R2**: Config 统一 — 建立 `NeoTrixConfig` 根配置，子模块 Config 继承
- [ ] **R3**: Gateway 瘦身 — 55 结构体合并 → 目标 25 以内
- [ ] **R4**: Dead Code 清理 — 122 个 SelfTest 中标记未使用的
- [ ] **R5**: SelfTest 精简 — 122 个实现评估必要性

### Phase 2: 扁平缺陷修复
- [ ] **F1**: Actions Config 聚合 — 23 个独立 Config → ActionRegistry 统一配置
- [ ] **F2**: Consciousness Core Config 聚合 — 21 个 Config → ConsciousnessConfig
- [ ] **F3**: Shield Config 聚合 — 19 个 Config → ShieldConfig
- [ ] **F4**: Gateway 子模块 — router/cache/breaker/recovery → 4 个子目录

### Phase 3: 跨域错位修正
- [ ] **X1**: 消除 L2→L3 违规 — shield 类型下沉到 L2 或提升到 core
- [ ] **X2**: 消除 L2→L1 违规 — KB 类型统一到 core knowledge 层
- [ ] **X3**: 消除 L5→L1 违规 — evolution_loop_provider 桥接清理
- [ ] **X4**: Event Bus 类型对齐 — ActorEnvelope → pub

### Phase 4: 架构进化 (galaxy-tree 融合)
- [ ] **A1**: OPRD Engine 接入 SEAL Pipeline
- [ ] **A2**: E8 Evolution Loop 生产化
- [ ] **A3**: Universal Model Adapter — 统一多模型接口
- [ ] **A4**: Knowledge Graph Bridge — KB → VSA HyperCube
- [ ] **A5**: Self-Healing Auto-Repair — HeartbeatAggregator → 自动修复

### Phase 5: 测试与验证
- [ ] **T1**: 测试编译修复 — 156+ 测试文件编译错误
- [ ] **T2**: Integration Test 覆盖
- [ ] **T3**: Benchmark 基线
- [ ] **T4**: CI/CD Pipeline

### Phase 6: 多 Agent 自动巡检
- [ ] **M1**: Compile Guardian — 每次提交前 cargo check
- [ ] **M2**: Dead Code Detector — 定期扫描未使用 pub items
- [ ] **M3**: Cross-Layer Auditor — 检测层级违规
- [ ] **M4**: Config Consistency Checker
- [ ] **M5**: SelfTest Coverage Auditor

## 优先级排序

| 优先级 | 任务 | 收益 | 风险 |
|--------|------|------|------|
| P0 | R1 合并重复 Trait | 消除 9 个跨层桥接 | 中 |
| P0 | R3 Gateway 瘦身 | 降低单模块复杂度 60% | 中 |
| P1 | R2 Config 统一 | 消除 350 个独立 Config | 中 |
| P1 | F1 Actions Config 聚合 | 消除 23 个碎片 Config | 低 |
| P1 | X1-X3 跨层修正 | 修正 100+ 违规 | 高 |
| P2 | A1 OPRD→SEAL | 进化闭环 | 低 |
| P2 | A3 Universal Adapter | 多模型支持 | 中 |
| P3 | T1 测试修复 | 测试覆盖 | 低 |
| P3 | M1-M5 自动巡检 | 持续质量 | 低 |

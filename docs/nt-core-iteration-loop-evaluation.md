# NeoTrix 全量迭代评测报告 (Iteration Loop v1.0)

## 基线指标
| 指标 | 数值 |
|------|------|
| 总文件数 | 1,878 |
| 总代码行数 | 625,207 |
| 编译错误 | 0 (clean) |
| 编译警告 | 0 (deny(warnings)) |
| SelfTest 实现 | 122 |
| Config 结构体 | 350 |
| Gateway 模块 | 25 文件 / 7,518 行 / 55 结构体 / 3 traits |
| map_err 闭包 | 1,919 处 |
| Format 错误串 | 46 处 |
| 跨层违规 | 0 (架构清洁) |

## 三大问题诊断

### 1. 聚焦冗余 (Focus Redundancy)

| 冗余类型 | 位置 | 影响 |
|----------|------|------|
| Gateway 模块膨胀 | `gateway/` 25 文件 7518 行 | 单模块复杂度失控，55 结构体互相引用 |
| Config 结构体泛滥 | 350 个 `*Config` struct | 每个模块自建 Config，无统一配置模型 |
| SelfTest 过度实现 | 122 个 `impl SelfTest` | SelfTest 实现数远超实际检测需求 |
| 重复 trait 定义 | core(17) + nt_io(16) + nt_act(9) | 同名 trait 跨域定义，类型不统一 |
| 重复错误处理 | 1919 个 map_err + 46 个 format!("Failed to") | 无统一 ErrorType，每个模块重写错误转换 |

**根因**: 模块自治过度 → 每个模块自建类型系统 → 无法跨域复用

### 2. 扁平缺陷 (Flattening Defects)

| 缺陷类型 | 位置 | 影响 |
|----------|------|------|
| Gateway 内部层级缺失 | `gateway/` 无子模块分层 | 25 文件平铺，路由/缓存/熔断/恢复混杂 |
| Consciousness Core 膨胀 | `nt_consciousness_core/` 46 文件 | 认知核心承载过多职责（响应解析/任务编排/知识蒸馏） |
| Shield 模块分裂 | `nt_shield_impl/` 19 Config | 安全模块内部无清晰子域划分 |
| Actions 模块碎片化 | `nt_act/actions/` 23 Config | 每个 action 独立 Config，无聚合层 |

**根因**: 缺少中间聚合层 → 模块内部平铺 → 认知负荷过高

### 3. 跨域错位 (Cross-Domain Misalignment)

| 错位类型 | 位置 | 影响 |
|----------|------|------|
| L1 IO 承载 L5 职责 | `gateway/` 含智能路由/ML预测/自愈 | IO 层混入认知决策 |
| L5 认知含 IO 实现 | `response_parser.rs` 直接解析 HTTP | 认知层依赖 IO 细节 |
| Memory 含感知逻辑 | `nt_memory_zim_absorber.rs` 含 HTML 解析 | 记忆层混入感知处理 |
| Event Bus 类型泄露 | `ActorEnvelope` private 但被 pub 方法引用 | 事件总线内部类型暴露不一致 |

**根因**: 层级边界定义模糊 → 职责渗透 → 修改一处影响多层

## 核心路线任务清单

### Phase 1: 冗余清理 (立即可做)
- [ ] **R1**: Gateway 模块分层重构 — 路由/缓存/熔断/恢复 → 4 个子模块
- [ ] **R2**: Config 统一 — 建立 `NeoTrixConfig` 根配置，子模块 Config 继承
- [ ] **R3**: ErrorType 统一 — 建立 `nt_core_error` 统一错误域，消除 1919 个 map_err
- [ ] **R4**: Dead Code 清理 — 122 个 SelfTest 中标记未使用的 → `#[allow(dead_code)]`
- [ ] **R5**: Gateway 瘦身 — 55 结构体合并 → 目标 25 以内

### Phase 2: 扁平缺陷修复
- [ ] **F1**: Consciousness Core 拆分 — response_parser → L2, task_orchestrator → L5, knowledge_distiller → L5
- [ ] **F2**: Shield 子域划分 — stealth_net / sandbox / audit → 独立子模块
- [ ] **F3**: Actions 聚合层 — 统一 ActionRegistry，消除 23 个独立 Config
- [ ] **F4**: Gateway 子模块 — router/ cache/ breaker/ recovery/ telemetry

### Phase 3: 跨域错位修正
- [ ] **X1**: IO 层回归 — gateway 中的 ML 预测/智能路由 → L5 cognition
- [ ] **X2**: Response Parser 下沉 — HTTP 响应解析 → L2 perception
- [ ] **X3**: ZIM Absorber 重构 — HTML 解析 → L2, KB 写入 → L1 memory
- [ ] **X4**: Event Bus 类型对齐 — ActorEnvelope → pub 或完全 pub(crate)

### Phase 4: 架构进化 (galaxy-tree 融合)
- [ ] **A1**: OPRD Engine 接入 SEAL Pipeline — OPRD 发现 → SEAL 自动吸收
- [ ] **A2**: E8 Evolution Loop 生产化 — 当前仅编译级，需接入真实 Provider 调用
- [ ] **A3**: Universal Model Adapter — 统一 GPT-4o/Claude/DeepSeek/Qwen3 调用接口
- [ ] **A4**: Knowledge Graph Bridge — KB 实体 → VSA HyperCube 向量映射
- [ ] **A5**: Self-Healing Auto-Repair — HeartbeatAggregator 信号 → 自动修复执行

### Phase 5: 测试与验证
- [ ] **T1**: 测试编译修复 — 156+ 测试文件编译错误
- [ ] **T2**: Integration Test 覆盖 — 关键路径端到端测试
- [ ] **T3**: Benchmark 基线 — 建立编译时间/运行时性能基准
- [ ] **T4**: CI/CD Pipeline — cargo check + test + clippy 全通过

### Phase 6: 多 Agent 自动巡检
- [ ] **M1**: Compile Guardian — 每次提交前自动 cargo check
- [ ] **M2**: Dead Code Detector — 定期扫描未使用 pub items
- [ ] **M3**: Cross-Layer Auditor — 检测层级违规
- [ ] **M4**: Config Consistency Checker — Config 结构体一致性检查
- [ ] **M5**: SelfTest Coverage Auditor — SelfTest 覆盖率审计

## 优先级排序

| 优先级 | 任务 | 收益 | 风险 |
|--------|------|------|------|
| P0 | R1 Gateway 分层 | 降低单模块复杂度 60% | 中 |
| P0 | R3 ErrorType 统一 | 消除 1919 处重复 | 低 |
| P1 | R2 Config 统一 | 消除 350 个独立 Config | 中 |
| P1 | F1 Consciousness Core 拆分 | 解耦认知核心 | 高 |
| P1 | X1 IO 层回归 | 修正层级边界 | 中 |
| P2 | A1 OPRD→SEAL | 进化闭环 | 低 |
| P2 | A3 Universal Adapter | 多模型支持 | 中 |
| P3 | T1 测试修复 | 测试覆盖 | 低 |
| P3 | M1-M5 自动巡检 | 持续质量 | 低 |

## galaxy-tree 架构融合点

| galaxy-tree 模式 | NeoTrix 映射 | 状态 |
|-----------------|-------------|------|
| P1: Model Routing | GWT salience + cost weight | ✅ 已实现 |
| P2: Isolation-per-Task | Worktree isolation + paged memory | ⚠️ 部分 |
| P3: Profile-Driven | SelfModel extension | ✅ 已实现 |
| P4: Ordered Backend Fallback | Ordered Backend Router | ✅ 已实现 |
| P5: Skill as Template | SKILL-SPEC.md contract | ✅ 已实现 |
| C.41 OS Sandbox | nt_shield_sandbox | ⚠️ 部分 |
| C.44 Dual-Temporal Facts | Graphiti + MELD | ❌ 未实现 |
| C.45 Unified Document IR | AnyDoc + MarkItDown | ⚠️ 部分 |
| 0.26 Cost-Aware Routing | GWT + token cost | ✅ 已实现 |
| 0.26 Context Virtualization | KVMem paged KV | ⚠️ 部分 |

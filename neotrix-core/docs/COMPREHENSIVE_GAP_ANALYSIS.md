# 全量缺陷审计与补齐清单

> 审计日期: 2026-09-14
> 审计范围: Batch 1 (Agent框架+RSI+Colibri) + Batch 2 (OpenUI/Anthropic/HybridClaw/OSINT/arXiv/GPT-6/Apple/GPU-Doodle/iOS) + knife (逆向工程)
> 目标: 形成通用方案适用所有外部模型，实现聚焦冗余 + 扁平缺陷 + 跨域错位

---

## 一、吸收模式全量清单

### 1.1 已吸收模式 (✅)

| # | 模式 | 来源 | NeoTrix 模块 | 状态 |
|---|------|------|-------------|------|
| 1 | Memory Hierarchy (VRAM/RAM/NVMe) | Colibri | `nt_game::CrystalMemoryHierarchy` | ✅ 已实现 |
| 2 | JIT Weights (按需加载) | Colibri | `nt_game::CrystalJITWeights` | ✅ 已实现 |
| 3 | Router (热度驱动) | Colibri | `nt_game::CrystalRouter` | ✅ 已实现 |
| 4 | Speculator (推测解码) | Colibri | `nt_game::CrystalSpeculator` | ✅ 已实现 |
| 5 | ECS (Entity-Component-System) | Unity/Bevy | `nt_game::CrystalECS` | ✅ 已实现 |
| 6 | Scene Tree (场景树) | Godot | `nt_game::CrystalSceneTree` | ✅ 已实现 |
| 7 | Signal (信号系统) | Godot | `nt_game::CrystalSignalSystem` | ✅ 已实现 |
| 8 | Resource (资源系统) | Godot | `nt_game::CrystalResourceManager` | ✅ 已实现 |
| 9 | Card/Deck (卡组系统) | NueDeck | `nt_game::CrystalCard/Deck` | ✅ 已实现 |
| 10 | State Machine (状态机) | 通用 | `nt_game::CrystalStateMachine` | ✅ 已实现 |
| 11 | Behavior Tree (行为树) | 游戏AI | `nt_game::CrystalBehaviorTree` | ✅ 已实现 |
| 12 | Event Bus (事件总线) | LuaFramework | `nt_game::CrystalEventBus` | ✅ 已实现 |
| 13 | Melting Engine (熔炼引擎) | 自研 | `nt_game::MeltingEngine` | ✅ 已实现 |

### 1.2 待吸收模式 (❌/⚠️)

| # | 模式 | 来源 | NeoTrix 模块 | 状态 | 优先级 |
|---|------|------|-------------|------|--------|
| **Batch 1: Agent框架** | | | | | |
| 14 | ACP (Agent客户端协议) | OpenHands | `nt_act::AcpProtocol` | ❌ 缺失 | P0 |
| 15 | Crews (角色化Agent团队) | crewAI | `nt_core::CrewManager` | ⚠️ 有基础 | P1 |
| 16 | Flows (事件驱动工作流) | crewAI | `nt_mind::FlowEngine` | ⚠️ 有基础 | P1 |
| 17 | RepoMap (代码语义地图) | Aider | `nt_world::RepoMap` | ⚠️ 有基础 | P1 |
| 18 | Git Integration (自动提交/回滚) | Aider | `nt_memory::GitIntegration` | ⚠️ 有基础 | P2 |
| 19 | Lint+Test (自动验证循环) | Aider | `nt_shield::AutoValidator` | ⚠️ 有基础 | P2 |
| 20 | JSON-First (声明式Agent定义) | crewAI | `nt_core::AgentConfig` | ❌ 缺失 | P1 |
| **Batch 1: RSI 9路径** | | | | | |
| 21 | Metaⁿ (递归策略层) | Metaⁿ | `nt_meta::RecursiveStrategy` | ⚠️ 有基础 | P1 |
| 22 | Recuris (记忆进化) | Recuris | `nt_memory::MemoryEvolution` | ⚠️ 有基础 | P1 |
| 23 | MetaSkill-Evolve (技能+元技能共进化) | MetaSkill-Evolve | `nt_mind::MetaSkillEvolution` | ⚠️ 有基础 | P1 |
| 24 | Q-Evolve (策略进化) | Q-Evolve | `nt_core::QEvolution` | ⚠️ 有基础 | P1 |
| 25 | RISE (未来自我蒸馏) | RISE | `nt_mind::RISEReflector` | ⚠️ 有基础 | P2 |
| 26 | SkillGLoW (过程技能记忆) | SkillGLoW | `nt_memory::SkillProceduralMemory` | ⚠️ 有基础 | P2 |
| 27 | MGM (智能体脚手架自修改) | MGM | `nt_repair::MetaModification` | ⚠️ 有基础 | P2 |
| 28 | RQGM (评估器共进化) | RQGM | `nt_meta::EvolvingEvaluator` | ❌ 缺失 | P1 |
| 29 | DGM (代码/脚手架进化) | DGM | `nt_mind::EvolutionArchive` | ❌ 缺失 | P1 |
| **Batch 2: 前沿技术** | | | | | |
| 30 | OpenUI OUI-1 (生成式UI) | OpenUI | `nt_io::GenerativeUI` | ❌ 缺失 | P2 |
| 31 | Anthropic Defending Code (漏洞管道) | Anthropic | `nt_shield::VulnerabilityPipeline` | ⚠️ 有基础 | P1 |
| 32 | HybridClaw (企业运行时) | HybridClaw | `nt_io::EnterpriseRuntime` | ⚠️ 有基础 | P2 |
| 33 | Maigret (OSINT异步) | Maigret | `nt_world::AsyncOSINT` | ⚠️ 有基础 | P2 |
| 34 | arXiv 2609.05881 (缺陷制品检测) | arXiv | `nt_shield::ArtifactValidator` | ❌ 缺失 | P1 |
| 35 | arXiv 2609.07876 (TLCM层校正) | arXiv | `nt_core::TLCMLayerCorrection` | ❌ 缺失 | P1 |
| 36 | GPT-6 Astra (异步工具执行) | OpenAI | `nt_act::AsyncToolExecutor` | ❌ 缺失 | P0 |
| 37 | GPT-6 Astra (延迟加载) | OpenAI | `nt_act::DeferredLoader` | ❌ 缺失 | P1 |
| 38 | GPT-6 Astra (中途转向) | OpenAI | `nt_mind::MidTurnSteering` | ❌ 缺失 | P1 |
| 39 | GPT-6 Astra (运行时监控) | OpenAI | `nt_meta::RuntimeMonitor` | ❌ 缺失 | P1 |
| 40 | Apple Design (Liquid Glass) | Apple | `nt_io::LiquidGlassRenderer` | ⚠️ 有基础 | P2 |
| 41 | GPU-Doodle (WebGPU推理) | GPU-Doodle | `nt_io::WebGPUInference` | ❌ 缺失 | P2 |
| 42 | iOS iBoot (固件安全) | iOS | `nt_shield::FirmwareAnalyzer` | ⚠️ 有基础 | P3 |
| **knife: 逆向工程** | | | | | |
| 43 | 二进制解析 (PE/ELF/Mach-O) | knife | `nt_shield::BinaryAnalyzer` | ❌ 缺失 | P0 |
| 44 | 缓解措施审计 | knife | `nt_shield::MitigationAuditor` | ❌ 缺失 | P0 |
| 45 | 危险调用分析 | knife | `nt_shield::SinkAnalyzer` | ❌ 缺失 | P0 |
| 46 | 漏洞审计 | knife | `nt_shield::VulnerabilityAuditor` | ❌ 缺失 | P1 |
| 47 | 函数恢复 | knife | `nt_world::FunctionRecovery` | ❌ 缺失 | P1 |
| 48 | CFG构建 | knife | `nt_world::CFGBuilder` | ❌ 缺失 | P1 |
| 49 | 交叉引用分析 | knife | `nt_world::XRefAnalyzer` | ❌ 缺失 | P1 |
| 50 | MCP服务器 | knife | `nt_act::McpBinaryAnalyzer` | ❌ 缺失 | P1 |
| 51 | YARA扫描 | knife | `nt_shield::YaraScanner` | ❌ 缺失 | P2 |
| 52 | IOC提取 | knife | `nt_world::IOCExtractor` | ❌ 缺失 | P2 |
| 53 | 内核驱动分析 | knife | `nt_shield::DriverAnalyzer` | ❌ 缺失 | P2 |
| 54 | 二进制补丁 | knife | `nt_shield::BinaryPatcher` | ❌ 缺失 | P3 |

---

## 二、架构层缺陷映射

### 2.1 L6 Meta-Cognition (元认知层)

| 缺陷ID | 缺陷描述 | 严重度 | 来源 | 修复方案 | 工时 |
|--------|---------|--------|------|---------|------|
| L6-01 | 缺少递归策略层级 | 🟡 中 | Metaⁿ | 实现`nt_meta::RecursiveStrategy` | 4h |
| L6-02 | 缺少评估器进化 | 🟡 中 | RQGM | 实现`nt_meta::EvolvingEvaluator` | 4h |
| L6-03 | 缺少运行时监控 | 🟡 中 | GPT-6 Astra | 实现`nt_meta::RuntimeMonitor` | 3h |
| L6-04 | 缺少缺陷制品检测 | 🟡 中 | arXiv 2609.05881 | 实现`nt_shield::ArtifactValidator` | 3h |
| L6-05 | 缺少威胁建模 | 🟡 中 | Anthropic | 实现`nt_shield::ThreatModeler` | 4h |
| **小计** | | | | | **18h** |

### 2.2 L5 Cognition (认知层)

| 缺陷ID | 缺陷描述 | 严重度 | 来源 | 修复方案 | 工时 |
|--------|---------|--------|------|---------|------|
| L5-01 | 缺少TLCM层校正 | 🟡 中 | arXiv 2609.07876 | 实现`nt_core::TLCMLayerCorrection` | 4h |
| L5-02 | 缺少延迟加载 | 🟡 中 | GPT-6 Astra | 实现`nt_act::DeferredLoader` | 2h |
| L5-03 | 缺少中途转向 | 🟡 中 | GPT-6 Astra | 实现`nt_mind::MidTurnSteering` | 3h |
| L5-04 | 缺少代码语义理解 | 🟡 中 | Aider | 集成`tree-sitter` AST解析 | 4h |
| L5-05 | 缺少策略进化 | 🟡 中 | Q-Evolve | 实现`nt_core::QEvolution` | 4h |
| L5-06 | 缺少未来蒸馏 | 🟡 中 | RISE | 实现`nt_mind::RISEReflector` | 3h |
| **小计** | | | | | **20h** |

### 2.3 L4 Emotion (情感层)

| 缺陷ID | 缺陷描述 | 严重度 | 来源 | 修复方案 | 工时 |
|--------|---------|--------|------|---------|------|
| L4-01 | 情感层独立，无直接外部映射 | 🟢 低 | - | 保持现状 | 0h |
| **小计** | | | | | **0h** |

### 2.4 L3 Embodiment (具身层)

| 缺陷ID | 缺陷描述 | 严重度 | 来源 | 修复方案 | 工时 |
|--------|---------|--------|------|---------|------|
| L3-01 | 缺少ACP协议 | 🔴 高 | OpenHands | 实现`nt_act::AcpProtocol` | 4h |
| L3-02 | 缺少异步工具执行 | 🔴 高 | GPT-6 Astra | 实现`nt_act::AsyncToolExecutor` | 3h |
| L3-03 | 缺少企业审批工作流 | 🟡 中 | HybridClaw | 实现`nt_io::ApprovalWorkflow` | 3h |
| L3-04 | 缺少漏洞管道 | 🟡 中 | Anthropic | 实现`nt_shield::VulnerabilityPipeline` | 4h |
| L3-05 | 缺少二进制分析 | 🔴 高 | knife | 实现`nt_shield::BinaryAnalyzer` | 8h |
| L3-06 | 缺少缓解措施审计 | 🔴 高 | knife | 实现`nt_shield::MitigationAuditor` | 4h |
| L3-07 | 缺少危险调用分析 | 🔴 高 | knife | 实现`nt_shield::SinkAnalyzer` | 4h |
| L3-08 | 缺少YARA扫描 | 🟡 中 | knife | 实现`nt_shield::YaraScanner` | 4h |
| L3-09 | 缺少内核驱动分析 | 🟡 中 | knife | 实现`nt_shield::DriverAnalyzer` | 6h |
| L3-10 | 缺少二进制补丁 | 🟢 低 | knife | 实现`nt_shield::BinaryPatcher` | 6h |
| **小计** | | | | | **46h** |

### 2.5 L2 Perception (感知层)

| 缺陷ID | 缺陷描述 | 严重度 | 来源 | 修复方案 | 工时 |
|--------|---------|--------|------|---------|------|
| L2-01 | 缺少代码语义地图 | 🟡 中 | Aider | 实现`nt_world::RepoMap` | 4h |
| L2-02 | 缺少函数恢复 | 🟡 中 | knife | 实现`nt_world::FunctionRecovery` | 6h |
| L2-03 | 缺少CFG构建 | 🟡 中 | knife | 实现`nt_world::CFGBuilder` | 4h |
| L2-04 | 缺少交叉引用分析 | 🟡 中 | knife | 实现`nt_world::XRefAnalyzer` | 4h |
| L2-05 | 缺少IOC提取 | 🟡 中 | knife | 实现`nt_world::IOCExtractor` | 3h |
| L2-06 | 缺少OSINT异步优化 | 🟡 中 | Maigret | 优化`nt_world::AsyncOSINT` | 2h |
| **小计** | | | | | **23h** |

### 2.6 L1 Action (行动层)

| 缺陷ID | 缺陷描述 | 严重度 | 来源 | 修复方案 | 工时 |
|--------|---------|--------|------|---------|------|
| L1-01 | 缺少向量检索 | 🔴 高 | 通用 | 实现`nt_memory::VectorIndex` | 6h |
| L1-02 | 缺少记忆宫殿 | 🟡 中 | 通用 | 实现`nt_nexus::MemoryPalace` | 3h |
| L1-03 | 缺少JSON-first配置 | 🟡 中 | crewAI | 实现`nt_core::AgentConfig` | 4h |
| L1-04 | 缺少生成式UI | 🟡 中 | OpenUI | 实现`nt_io::GenerativeUI` | 4h |
| L1-05 | 缺少WebGPU推理 | 🟡 中 | GPU-Doodle | 实现`nt_io::WebGPUInference` | 4h |
| L1-06 | 缺少Liquid Glass渲染 | 🟡 中 | Apple | 实现`nt_io::LiquidGlassRenderer` | 4h |
| **小计** | | | | | **25h** |

---

## 三、冗余清理清单

### 3.1 模块级冗余 (4,810行)

| 冗余组 | 涉及模块 | 行数 | 清理方案 | 工时 |
|--------|---------|------|---------|------|
| ECS实现×3 | `crystal_ecs.rs` + `archetype_ecs.rs` + `universal_ecs.rs` | ~800 | 统一到`crystal_ecs.rs` | 2h |
| 场景树×2 | `crystal_scene.rs` + `scene_tree.rs` | ~350 | 统一到`crystal_scene.rs` | 1h |
| 信号系统×2 | `crystal_signal.rs` + `signal.rs` | ~330 | 统一到`crystal_signal.rs` | 1h |
| 卡牌系统×2 | `crystal_card.rs` + `card.rs` | ~700 | 统一到`crystal_card.rs` | 2h |
| 事件总线×2 | `crystal_event.rs` + `event.rs` | ~260 | 统一到`crystal_event.rs` | 1h |
| 资源管理×2 | `crystal_resource.rs` + `resource.rs` | ~200 | 统一到`crystal_resource.rs` | 1h |
| 行为树×2 | `crystal_behavior.rs` + `behavior_tree.rs` | ~270 | 统一到`crystal_behavior.rs` | 1h |
| 状态机×2 | `crystal_state.rs` + `state_machine.rs` | ~400 | 统一到`crystal_state.rs` | 1h |
| 熔炼引擎×2 | `melting_engine.rs`(新版) + `melting_engine.rs`(旧版) | ~500 | 保留新版 | 1h |
| **小计** | | **~4,810** | | **11h** |

### 3.2 代码级冗余

| 冗余模式 | 出现次数 | 清理方案 | 工时 |
|---------|---------|---------|------|
| `unwrap()` 调用 | 181处(nt-world-sim) | 替换为`?`或`unwrap_or` | 4h |
| `panic!()` 调用 | 115处(neotrix-core) | 生产代码替换为`Result` | 4h |
| 重复`pub use`导出 | 12处 | 去重 | 1h |
| 重复类型定义 | 6处 | 统一定义 | 1h |
| **小计** | | | **10h** |

---

## 四、跨域错位修复清单

### 4.1 接口错位

| 错位 | NeoTrix当前 | 外部标准 | 对齐方案 | 工时 |
|------|-------------|---------|---------|------|
| Agent定义 | Rust struct | JSON/YAML声明 | 支持JSON-first定义 | 4h |
| 工作流 | `ConsciousnessTree` | `Flow`事件驱动 | 添加`@start/@listen/@router` | 6h |
| 工具协议 | `CapabilityRegistry` | MCP/ACP | 实现MCP/ACP适配器 | 8h |
| 记忆格式 | `kv_store` | 向量DB+图DB | 添加向量/图存储后端 | 6h |
| 评估框架 | 固定benchmark | 动态评估器 | 实现评估器进化 | 4h |
| 进化策略 | `SEAL`单策略 | 多策略共进化 | 添加策略存档 | 3h |
| **小计** | | | | **31h** |

### 4.2 域边界违规

| 违规 | 位置 | 描述 | 修复方案 | 工时 |
|------|------|------|---------|------|
| 游戏代码在核心模块 | `engine/architecture.rs` | 游戏逻辑混入引擎层 | 移动到 `game/` | 2h |
| 意识逻辑在游戏模块 | `game/game_flow.rs` | 意识状态混入游戏循环 | 移动到 `core/` | 2h |
| 混合抽象层级 | `engine/card_ui.rs` | UI逻辑与卡牌逻辑混合 | 分离 UI 和逻辑 | 4h |
| 命名不一致 | `CardType` vs `CrystalCardType` | 同一概念不同命名 | 统一命名规范 | 2h |
| **小计** | | | | **10h** |

---

## 五、测试补全清单

| 测试类型 | 当前状态 | 目标 | 工时 |
|---------|---------|------|------|
| 单元测试 | 9,978 (neotrix-core) + 729 (nt-world-sim) | 覆盖率 > 80% | 16h |
| 集成测试 | 基础 | 全链路覆盖 | 8h |
| 性能基准 | 无 | query/route/inference吞吐 | 4h |
| 安全审计 | 基础 | 0 unsafe验证 | 4h |
| 文档生成 | 基础 | rustdoc全量 | 8h |
| **小计** | | | **40h** |

---

## 六、生产就绪清单

| 任务 | 当前状态 | 目标 | 工时 |
|------|---------|------|------|
| Tauri桌面端修复 | 黑屏 | 正常显示 | 4h |
| CLI体验优化 | 基础 | 命令补全/帮助 | 2h |
| 错误信息美化 | 基础 | 用户友好提示 | 2h |
| 日志系统 | 基础 | 分级/轮转/格式化 | 2h |
| 配置热加载 | 无 | 运行时修改 | 2h |
| **小计** | | | **12h** |

---

## 七、核心任务全量清单

### Phase 0: 冗余清理 (21h) — 🔴 最高优先级

| # | 任务 | 影响范围 | 工时 | 依赖 |
|---|------|---------|------|------|
| 0.1 | 统一ECS: 删除`archetype_ecs.rs`/`universal_ecs.rs` | nt-world-sim | 2h | 无 |
| 0.2 | 统一场景树: 删除`scene_tree.rs` | nt-world-sim | 1h | 无 |
| 0.3 | 统一信号: 删除`signal.rs` | nt-world-sim | 1h | 无 |
| 0.4 | 统一卡牌: 合并`card.rs`到`crystal_card.rs` | nt-world-sim | 2h | 无 |
| 0.5 | 统一事件: 删除`event.rs` | nt-world-sim | 1h | 无 |
| 0.6 | 统一资源: 合并`resource.rs`到`crystal_resource.rs` | nt-world-sim | 1h | 无 |
| 0.7 | 统一行为树: 合并`behavior_tree.rs`到`crystal_behavior.rs` | nt-world-sim | 1h | 无 |
| 0.8 | 统一状态机: 合并`state_machine.rs`到`crystal_state.rs` | nt-world-sim | 1h | 无 |
| 0.9 | 清理unwrap/panic: 181处unwrap + 115处panic | 全局 | 8h | 无 |
| 0.10 | 清理重复导出: mod.rs去重 | 全局 | 1h | 无 |
| 0.11 | 清理重复类型定义: 6处 | 全局 | 1h | 无 |
| 0.12 | 修复域边界违规: 4处 | 全局 | 4h | 无 |

### Phase 1: 核心协议与基础设施 (29h) — 🔴 高优先级

| # | 任务 | 影响范围 | 工时 | 依赖 |
|---|------|---------|------|------|
| 1.1 | 实现ACP协议: `nt_act::AcpProtocol` | nt-act | 4h | 无 |
| 1.2 | 实现向量检索: `nt_memory::VectorIndex` | nt-memory | 6h | 无 |
| 1.3 | 实现异步工具执行: `nt_act::AsyncToolExecutor` | nt-act | 3h | 无 |
| 1.4 | 实现JSON-first配置: `nt_core::AgentConfig` | nt-core | 4h | 无 |
| 1.5 | 实现二进制分析: `nt_shield::BinaryAnalyzer` | nt-shield | 8h | 无 |
| 1.6 | 实现缓解措施审计: `nt_shield::MitigationAuditor` | nt-shield | 4h | 1.5 |

### Phase 2: 分析引擎与安全 (46h) — 🔴 高优先级

| # | 任务 | 影响范围 | 工时 | 依赖 |
|---|------|---------|------|------|
| 2.1 | 实现危险调用分析: `nt_shield::SinkAnalyzer` | nt-shield | 4h | 1.5 |
| 2.2 | 实现漏洞审计: `nt_shield::VulnerabilityAuditor` | nt-shield | 4h | 2.1 |
| 2.3 | 实现函数恢复: `nt_world::FunctionRecovery` | nt-world | 6h | 1.5 |
| 2.4 | 实现CFG构建: `nt_world::CFGBuilder` | nt-world | 4h | 2.3 |
| 2.5 | 实现交叉引用分析: `nt_world::XRefAnalyzer` | nt-world | 4h | 2.3 |
| 2.6 | 实现MCP服务器: `nt_act::McpBinaryAnalyzer` | nt-act | 8h | 1.5, 2.1-2.5 |
| 2.7 | 实现漏洞管道: `nt_shield::VulnerabilityPipeline` | nt-shield | 4h | 2.2 |
| 2.8 | 实现YARA扫描: `nt_shield::YaraScanner` | nt-shield | 4h | 1.5 |
| 2.9 | 实现IOC提取: `nt_world::IOCExtractor` | nt-world | 3h | 1.5 |
| 2.10 | 实现内核驱动分析: `nt_shield::DriverAnalyzer` | nt-shield | 6h | 1.5, 2.1 |
| 2.11 | 实现缺陷制品检测: `nt_shield::ArtifactValidator` | nt-shield | 3h | 无 |
| 2.12 | 实现威胁建模: `nt_shield::ThreatModeler` | nt-shield | 4h | 2.7 |

### Phase 3: 认知与进化 (20h) — 🟡 中优先级

| # | 任务 | 影响范围 | 工时 | 依赖 |
|---|------|---------|------|------|
| 3.1 | 实现TLCM层校正: `nt_core::TLCMLayerCorrection` | nt-core | 4h | 无 |
| 3.2 | 实现延迟加载: `nt_act::DeferredLoader` | nt-act | 2h | 无 |
| 3.3 | 实现中途转向: `nt_mind::MidTurnSteering` | nt-mind | 3h | 无 |
| 3.4 | 实现运行时监控: `nt_meta::RuntimeMonitor` | nt-meta | 3h | 无 |
| 3.5 | 实现评估器进化: `nt_meta::EvolvingEvaluator` | nt-meta | 4h | 无 |
| 3.6 | 实现多分支存档: `nt_mind::EvolutionArchive` | nt-mind | 4h | 无 |

### Phase 4: 感知与UI (25h) — 🟡 中优先级

| # | 任务 | 影响范围 | 工时 | 依赖 |
|---|------|---------|------|------|
| 4.1 | 实现代码语义地图: `nt_world::RepoMap` | nt-world | 4h | 无 |
| 4.2 | 实现记忆宫殿: `nt_nexus::MemoryPalace` | nt-nexus | 3h | 无 |
| 4.3 | 实现生成式UI: `nt_io::GenerativeUI` | nt-io | 4h | 无 |
| 4.4 | 实现WebGPU推理: `nt_io::WebGPUInference` | nt-io | 4h | 无 |
| 4.5 | 实现Liquid Glass渲染: `nt_io::LiquidGlassRenderer` | nt-io | 4h | 无 |
| 4.6 | 实现企业审批工作流: `nt_io::ApprovalWorkflow` | nt-io | 3h | 无 |
| 4.7 | 优化OSINT异步: `nt_world::AsyncOSINT` | nt-world | 2h | 无 |
| 4.8 | 实现固件分析: `nt_shield::FirmwareAnalyzer` | nt-shield | 4h | 无 |
| 4.9 | 集成tree-sitter AST解析 | nt-world | 4h | 无 |

### Phase 5: RSI路径补齐 (23h) — 🟡 中优先级

| # | 任务 | 影响范围 | 工时 | 依赖 |
|---|------|---------|------|------|
| 5.1 | 实现递归策略: `nt_meta::RecursiveStrategy` | nt-meta | 4h | 无 |
| 5.2 | 实现记忆进化: `nt_memory::MemoryEvolution` | nt-memory | 4h | 无 |
| 5.3 | 实现技能共进化: `nt_mind::MetaSkillEvolution` | nt-mind | 4h | 无 |
| 5.4 | 实现策略进化: `nt_core::QEvolution` | nt-core | 4h | 无 |
| 5.5 | 实现未来蒸馏: `nt_mind::RISEReflector` | nt-mind | 3h | 无 |
| 5.6 | 实现过程技能记忆: `nt_memory::SkillProceduralMemory` | nt-memory | 2h | 无 |
| 5.7 | 实现脚手架自修改: `nt_repair::MetaModification` | nt-repair | 2h | 无 |

### Phase 6: 接口对齐 (31h) — 🟡 中优先级

| # | 任务 | 影响范围 | 工时 | 依赖 |
|---|------|---------|------|------|
| 6.1 | 实现事件驱动工作流: `@start/@listen/@router`装饰器 | nt-mind | 6h | 无 |
| 6.2 | 实现MCP适配器: `nt_act::McpAdapter` | nt-act | 4h | 无 |
| 6.3 | 实现OpenTelemetry集成: 替换`log` | 全局 | 3h | 无 |
| 6.4 | 实现配置外部化: 支持TOML/JSON配置 | 全局 | 3h | 无 |
| 6.5 | 实现二进制补丁: `nt_shield::BinaryPatcher` | nt-shield | 6h | 无 |
| 6.6 | 实现Git集成: `nt_memory::GitIntegration` | nt-memory | 4h | 无 |
| 6.7 | 实现自动验证: `nt_shield::AutoValidator` | nt-shield | 3h | 无 |

### Phase 7: 测试与文档 (40h) — 🟡 中优先级

| # | 任务 | 影响范围 | 工时 | 依赖 |
|---|------|---------|------|------|
| 7.1 | 跨模块集成测试: 全链路 | tests/ | 8h | Phase 1-6 |
| 7.2 | 补充单元测试: 覆盖率 > 80% | tests/ | 16h | Phase 1-6 |
| 7.3 | 性能基准: query/route/inference吞吐 | tests/bench/ | 4h | Phase 1-6 |
| 7.4 | 安全审计: 0 unsafe验证 | tests/ | 4h | Phase 1-6 |
| 7.5 | 文档生成: rustdoc全量 | docs/ | 8h | Phase 1-6 |

### Phase 8: 生产就绪 (12h) — 🟢 低优先级

| # | 任务 | 影响范围 | 工时 | 依赖 |
|---|------|---------|------|------|
| 8.1 | Tauri桌面端修复: 黑屏问题 | src-tauri/ | 4h | 无 |
| 8.2 | CLI体验优化: 命令补全/帮助 | cli/ | 2h | 无 |
| 8.3 | 错误信息美化: 用户友好提示 | 全局 | 2h | 无 |
| 8.4 | 日志系统: 分级/轮转/格式化 | 全局 | 2h | 无 |
| 8.5 | 配置热加载: 运行时修改 | 全局 | 2h | 无 |

---

## 八、总计

| Phase | 工时 | 优先级 | 里程碑 |
|-------|------|--------|--------|
| Phase 0: 冗余清理 | 21h | 🔴 最高 | 消除4,810行冗余代码 |
| Phase 1: 核心协议与基础设施 | 29h | 🔴 高 | ACP/向量检索/异步执行/二进制分析 |
| Phase 2: 分析引擎与安全 | 46h | 🔴 高 | 完整二进制分析链+安全审计 |
| Phase 3: 认知与进化 | 20h | 🟡 中 | TLCM/延迟加载/评估器进化 |
| Phase 4: 感知与UI | 25h | 🟡 中 | RepoMap/生成式UI/WebGPU |
| Phase 5: RSI路径补齐 | 23h | 🟡 中 | 9条RSI路径补齐 |
| Phase 6: 接口对齐 | 31h | 🟡 中 | 事件驱动/MCP/OTEL |
| Phase 7: 测试与文档 | 40h | 🟡 中 | 全链路测试+文档 |
| Phase 8: 生产就绪 | 12h | 🟢 低 | Tauri/CLI/日志 |
| **总计** | **247h** | | |

---

## 九、依赖关系图

```
Phase 0 (冗余清理)
    ↓
Phase 1 (核心协议) ←── 无依赖
    ↓
Phase 2 (分析引擎) ←── 依赖 Phase 1.5 (BinaryAnalyzer)
    ↓
Phase 3 (认知进化) ←── 无依赖
    ↓
Phase 4 (感知UI) ←── 无依赖
    ↓
Phase 5 (RSI路径) ←── 无依赖
    ↓
Phase 6 (接口对齐) ←── 无依赖
    ↓
Phase 7 (测试文档) ←── 依赖 Phase 1-6
    ↓
Phase 8 (生产就绪) ←── 无依赖
```

---

## 十、关键决策点

### 决策 1: 二进制分析深度?
- **选项A**: 仅基础解析 (PE/ELF/Mach-O)
- **选项B**: 基础 + 安全审计 (缓解+漏洞)
- **选项C**: 完整分析链 (解析+审计+MCP+YARA)
- **建议**: 选项C，knife 已提供完整实现

### 决策 2: MCP工具范围?
- **选项A**: 10个核心工具
- **选项B**: 20个常用工具
- **选项C**: 30+完整工具集
- **建议**: 选项B，渐进式扩展

### 决策 3: RSI路径优先级?
- **选项A**: 仅Metaⁿ+RQGM (核心)
- **选项B**: 核心+Q-Evolve+RISE (中等)
- **选项C**: 全部9条路径
- **建议**: 选项B，验证效果后再扩展

### 决策 4: 测试覆盖目标?
- **选项A**: 60% (基础)
- **选项B**: 80% (良好)
- **选项C**: 95% (优秀)
- **建议**: 选项B，平衡质量与效率

---

## 十一、风险评估

| 风险 | 可能性 | 影响 | 缓解 |
|------|--------|------|------|
| 工期超支 | 高 | 高 | Phase 0优先，渐进交付 |
| 架构冲突 | 中 | 中 | 接口抽象，独立模块 |
| 性能回归 | 低 | 高 | 基准测试，性能监控 |
| 安全漏洞 | 中 | 高 | 安全审计，渗透测试 |
| 外部依赖 | 低 | 中 | 最小化依赖，本地优先 |
| 测试覆盖不足 | 中 | 中 | 持续集成，覆盖率检查 |

---

## 十二、预期成果

### 12.1 能力提升

| 能力 | 当前 | 融合后 | 提升 |
|------|------|--------|------|
| Agent协议 | 无 | ACP/MCP | ✅ |
| 向量检索 | 无 | ANN索引 | ✅ |
| 异步执行 | 同步 | 异步+背压 | ✅ |
| 二进制分析 | 无 | PE/ELF/Mach-O | ✅ |
| 安全审计 | 基础 | 高级漏洞扫描 | ✅ |
| 生成式UI | 静态模板 | AI生成 | ✅ |
| TLCM | 无 | 层校正 | ✅ |
| 9条RSI路径 | 基础 | 完整 | ✅ |

### 12.2 架构优势

| 优势 | 描述 |
|------|------|
| **统一协议** | ACP/MCP兼容，跨Agent通信 |
| **语义搜索** | 向量检索，知识语义匹配 |
| **异步高效** | 背压控制，资源感知调度 |
| **二进制安全** | 完整分析链，漏洞自动发现 |
| **AI生成UI** | 动态UI生成，个性化体验 |
| **推理优化** | TLCM层校正，提升推理质量 |
| **自我进化** | 9条RSI路径，持续自我改进 |

---

*全量缺陷审计与补齐清单完成。基于 Batch 1 + Batch 2 + knife 三大吸收源，覆盖 6 层架构、54 个缺陷、247h 工作量。*

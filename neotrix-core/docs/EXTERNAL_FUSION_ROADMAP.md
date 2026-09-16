# 外部技术熔炼 — 逆向推理融合方案

## 一、外部技术源提取的通用模式

### 1.1 Agent框架模式 (OpenHands/crewAI/Aider)

| 模式 | 来源 | NeoTrix映射 | 融合状态 |
|------|------|-------------|---------|
| **Agent Canvas** — 多后端统一控制面 | OpenHands | `nt_io` CLI/Tauri | ⚠️ 已有CLI，缺多后端切换 |
| **Crews** — 角色化自主Agent团队 | crewAI | `nt_core` 多Agent调度 | ⚠️ 有ParallelScheduler，缺角色定义 |
| **Flows** — 事件驱动工作流+状态 | crewAI | `nt_mind` 进化循环 | ⚠️ 有ConsciousnessTree，缺事件驱动 |
| **RepoMap** — 代码库语义地图 | Aider | `nt_world` 爬虫/感知 | ⚠️ 有crawl，缺代码语义理解 |
| **ACP** — Agent客户端协议 | OpenHands | `nt_act` 工具协议 | ❌ 缺失 |
| **Git Integration** — 自动提交/回滚 | Aider | `nt_memory` 持久化 | ⚠️ 有KB，缺Git级版本控制 |
| **Lint+Test** — 自动验证循环 | Aider | `nt_shield` 安全 | ⚠️ 有cargo test，缺自动修复 |
| **JSON-First** — 声明式Agent定义 | crewAI | `nt_core` 配置 | ❌ 缺失 |

### 1.2 递归自我改进模式 (RSI 9路径)

| 模式 | 来源 | NeoTrix映射 | 融合状态 |
|------|------|-------------|---------|
| **Metaⁿ** — 递归策略层 | Metaⁿ | `nt_meta` 元认知 | ⚠️ 有meta，缺递归层级 |
| **Recuris** — 记忆进化 | Recuris | `nt_memory` KB | ⚠️ 有KB，缺检索策略进化 |
| **MetaSkill-Evolve** — 技能+元技能共进化 | MetaSkill-Evolve | `nt_mind` 技能树 | ⚠️ 有Constellation，缺元技能 |
| **Q-Evolve** — 策略进化 | Q-Evolve | `nt_core` SelfModel | ⚠️ 有值函数，缺步骤级学习 |
| **RISE** — 未来自我蒸馏 | RISE | `nt_mind` SEAL | ⚠️ 有SEAL，缺轨迹外推 |
| **SkillGLoW** — 过程技能记忆 | SkillGLoW | `nt_memory` 经验 | ⚠️ 有experience-tree，缺过程化 |
| **MGM** — 智能体脚手架自修改 | MGM | `nt_repair` 自愈 | ⚠️ 有repair，缺跨任务比较 |
| **RQGM** — 评估器共进化 | RQGM | `nt_meta` 治理 | ⚠️ 有治理，缺评估器进化 |
| **DGM** — 代码/脚手架进化 | DGM | `nt_mind` 进化 | ⚠️ 有进化循环，缺多分支存档 |

### 1.3 推理优化模式 (Colibri+论文)

| 模式 | 来源 | NeoTrix映射 | 融合状态 |
|------|------|-------------|---------|
| **内存多层级** — VRAM/RAM/NVMe统一 | Colibri | `nt_game` MemoryHierarchy | ✅ 已吸收 |
| **权重JIT** — 按需加载 | Colibri | `nt_game` JITWeights | ✅ 已吸收 |
| **路由驱动** — 热度放置 | Colibri | `nt_game` Router | ✅ 已吸收 |
| **推测解码** — MTP+验证 | Colibri | `nt_game` Speculator | ✅ 已吸收 |
| **N-gram推测** — 语法强制草稿 | N-gram论文 | `nt_game` Speculator | ⚠️ 有基础实现，缺语法树 |
| **AliceAI-T5-35B** — MoE稀疏激活 | Yandex | `nt_core` 路由 | ❌ 缺失 |

### 1.4 记忆/知识模式

| 模式 | 来源 | NeoTrix映射 | 融合状态 |
|------|------|-------------|---------|
| **Agent Memory Atlas** — 记忆地图 | memanto/atlas | `nt_nexus` 跨会话 | ⚠️ 有nexus，缺地图可视化 |
| **MemPalace** — 记忆宫殿 | mempalace | `nt_nexus` | ❌ 缺失 |
| **KB Embedding** — 向量存储 | public-apis | `nt_memory` | ⚠️ 有KV，缺向量检索 |

---

## 二、聚焦冗余分析 (Focused Redundancy)

### 2.1 模块级冗余

| 冗余组 | 涉及模块 | 行数 | 冗余类型 | 清理方案 |
|--------|---------|------|---------|---------|
| **ECS实现×3** | `crystal_ecs.rs` + `archetype_ecs.rs`(nt-world-sim) + `universal_ecs.rs`(nt-world-sim) | ~800 | 三套ECS | 统一到`crystal_ecs.rs` |
| **场景树×2** | `crystal_scene.rs` + `scene_tree.rs`(nt-world-sim) | ~350 | 两套场景树 | 统一到`crystal_scene.rs` |
| **信号系统×2** | `crystal_signal.rs` + `signal.rs`(nt-world-sim) | ~330 | 两套信号 | 统一到`crystal_signal.rs` |
| **卡牌系统×2** | `crystal_card.rs` + `card.rs`(nt-world-sim) | ~700 | 两套卡牌 | 统一到`crystal_card.rs` |
| **事件总线×2** | `crystal_event.rs` + `event.rs`(nt-world-sim) | ~260 | 两套事件 | 统一到`crystal_event.rs` |
| **资源管理×2** | `crystal_resource.rs` + `resource.rs`(nt-world-sim) | ~200 | 两套资源 | 统一到`crystal_resource.rs` |
| **行为树×2** | `crystal_behavior.rs` + `behavior_tree.rs`(nt-world-sim) | ~270 | 两套行为树 | 统一到`crystal_behavior.rs` |
| **状态机×2** | `crystal_state.rs` + `state_machine.rs`(nt-world-sim) | ~400 | 两套状态机 | 统一到`crystal_state.rs` |
| **熔炼引擎×2** | `melting_engine.rs` + `melting_engine.rs`(旧版) | ~500 | 两版熔炼 | 保留新版 |

**冗余总行数**: ~4,810行可清理

### 2.2 代码级冗余

| 冗余模式 | 出现次数 | 示例 | 清理方案 |
|---------|---------|------|---------|
| `unwrap()` 调用 | 181处(nt-world-sim) | 到处可见 | 替换为`?`或`unwrap_or` |
| `panic!()` 调用 | 115处(neotrix-core) | 测试中可接受，生产中不行 | 生产代码替换为`Result` |
| 重复`pub use`导出 | 12处 | mod.rs重复导出 | 去重 |
| 重复类型定义 | 6处 | `ExpertId`/`CrystalValue`等 | 统一定义 |

---

## 三、扁平缺陷分析 (Flat Defects)

### 3.1 架构级缺陷

| 缺陷 | 严重度 | 描述 | 修复方案 |
|------|--------|------|---------|
| **ACP协议缺失** | 🔴 高 | 无Agent间通信协议 | 实现`nt_act::AcpProtocol` |
| **向量检索缺失** | 🔴 高 | KB只有KV，无语义搜索 | 集成`nt_memory::VectorIndex` |
| **代码语义理解缺失** | 🟡 中 | 无法解析代码AST | 集成`tree-sitter` |
| **评估器进化缺失** | 🟡 中 | 评估标准固定 | 实现`nt_meta::EvolvingEvaluator` |
| **多分支存档缺失** | 🟡 中 | 进化只保留最新 | 实现`nt_mind::EvolutionArchive` |
| **记忆宫殿缺失** | 🟡 中 | 无空间记忆 | 实现`nt_nexus::MemoryPalace` |

### 3.2 代码级缺陷

| 缺陷 | 数量 | 示例 | 修复方案 |
|------|------|------|---------|
| **编译错误** | ~175 | `crystal_state.rs`缺失 | 已修复 |
| **未使用导入** | ~30 | 各模块 | `cargo fix` |
| **类型不匹配** | ~12 | `Box<dyn Error>` vs `CrystalError` | 统一错误类型 |
| **生命周期问题** | ~5 | `CrystalArchetype`借用 | 重构存储 |

---

## 四、跨域错位分析 (Cross-Domain Dislocation)

### 4.1 领域映射错位

| 错位 | NeoTrix概念 | 外部对应 | 对齐方案 |
|------|-------------|---------|---------|
| **Agent定义** | Rust struct | JSON/YAML声明 | 支持JSON-first定义 |
| **工作流** | `ConsciousnessTree` | `Flow`事件驱动 | 添加`@start/@listen/@router` |
| **工具协议** | `CapabilityRegistry` | MCP/ACP | 实现MCP适配器 |
| **记忆格式** | `kv_store` | 向量DB+图DB | 添加向量/图存储后端 |
| **评估框架** | 固定benchmark | 动态评估器 | 实现评估器进化 |
| **进化策略** | `SEAL`单策略 | 多策略共进化 | 添加策略存档 |

### 4.2 接口错位

| 接口 | NeoTrix当前 | 外部标准 | 对齐方案 |
|------|-------------|---------|---------|
| **CLI** | 自定义命令 | OpenHands ACP | 实现ACP兼容 |
| **配置** | Rust代码 | JSON/TOML | 支持声明式配置 |
| **日志** | `log::info!` | OpenTelemetry | 集成OTEL |
| **测试** | `cargo test` | pytest/Go test | 保持Rust原生 |

---

## 五、核心路线任务清单

### Phase 1: 冗余清理 (16h)

| # | 任务 | 影响范围 | 工时 |
|---|------|---------|------|
| 1.1 | 统一ECS: 删除`archetype_ecs.rs`/`universal_ecs.rs`，保留`crystal_ecs.rs` | nt-world-sim | 2h |
| 1.2 | 统一场景树: 删除`scene_tree.rs`，保留`crystal_scene.rs` | nt-world-sim | 1h |
| 1.3 | 统一信号: 删除`signal.rs`，保留`crystal_signal.rs` | nt-world-sim | 1h |
| 1.4 | 统一卡牌: 合并`card.rs`到`crystal_card.rs` | nt-world-sim | 2h |
| 1.5 | 统一事件: 删除`event.rs`，保留`crystal_event.rs` | nt-world-sim | 1h |
| 1.6 | 统一资源: 合并`resource.rs`到`crystal_resource.rs` | nt-world-sim | 1h |
| 1.7 | 统一行为树: 合并`behavior_tree.rs`到`crystal_behavior.rs` | nt-world-sim | 1h |
| 1.8 | 统一状态机: 合并`state_machine.rs`到`crystal_state.rs` | nt-world-sim | 1h |
| 1.9 | 清理unwrap/panic: 181处unwrap + 115处panic | 全局 | 4h |
| 1.10 | 清理重复导出: mod.rs去重 | 全局 | 2h |

### Phase 2: 缺陷补齐 (24h)

| # | 任务 | 影响范围 | 工时 |
|---|------|---------|------|
| 2.1 | 实现ACP协议: `nt_act::acp` | nt-act | 4h |
| 2.2 | 实现向量检索: `nt_memory::VectorIndex` | nt-memory | 6h |
| 2.3 | 实现代码AST解析: 集成`tree-sitter` | nt-world | 4h |
| 2.4 | 实现评估器进化: `nt_meta::EvolvingEvaluator` | nt-meta | 4h |
| 2.5 | 实现多分支存档: `nt_mind::EvolutionArchive` | nt-mind | 3h |
| 2.6 | 实现记忆宫殿: `nt_nexus::MemoryPalace` | nt-nexus | 3h |

### Phase 3: 错位对齐 (20h)

| # | 任务 | 影响范围 | 工时 |
|---|------|---------|------|
| 3.1 | JSON-first Agent定义: `nt_core::AgentConfig` | nt-core | 4h |
| 3.2 | 事件驱动工作流: `@start/@listen/@router`装饰器 | nt-mind | 6h |
| 3.3 | MCP适配器: `nt_act::McpAdapter` | nt-act | 4h |
| 3.4 | OpenTelemetry集成: 替换`log` | 全局 | 3h |
| 3.5 | 配置外部化: 支持TOML/JSON配置 | 全局 | 3h |

### Phase 4: 测试补全 (12h)

| # | 任务 | 影响范围 | 工时 |
|---|------|---------|------|
| 4.1 | 跨模块集成测试: 全链路 | tests/ | 4h |
| 4.2 | 性能基准: query/route/inference吞吐 | tests/bench/ | 2h |
| 4.3 | 安全审计: 0 unsafe验证 | tests/ | 2h |
| 4.4 | 文档生成: rustdoc全量 | docs/ | 4h |

### Phase 5: 生产就绪 (12h)

| # | 任务 | 影响范围 | 工时 |
|---|------|---------|------|
| 5.1 | Tauri桌面端修复: 黑屏问题 | src-tauri/ | 4h |
| 5.2 | CLI体验优化: 命令补全/帮助 | cli/ | 2h |
| 5.3 | 错误信息美化: 用户友好提示 | 全局 | 2h |
| 5.4 | 日志系统: 分级/轮转/格式化 | 全局 | 2h |
| 5.5 | 配置热加载: 运行时修改 | 全局 | 2h |

---

## 六、总计

| Phase | 工时 | 优先级 |
|-------|------|--------|
| Phase 1: 冗余清理 | 16h | 🔴 最高 |
| Phase 2: 缺陷补齐 | 24h | 🔴 高 |
| Phase 3: 错位对齐 | 20h | 🟡 中 |
| Phase 4: 测试补全 | 12h | 🟡 中 |
| Phase 5: 生产就绪 | 12h | 🟢 低 |
| **总计** | **84h** | |

## 七、熔炼到能力骨架的映射

```
外部模式 → NeoTrix 6层架构映射
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
L6 Meta    ← Metaⁿ递归策略 + RQGM评估器进化 + DGM多分支
L5 Cognition ← Q-Evolve策略 + RISE未来蒸馏 + SkillGLoW技能
L4 Emotion ← (无直接映射，情感层独立)
L3 Embodiment ← MGM脚手架 + ACP协议
L2 Perception ← RepoMap代码理解 + 向量检索
L1 Action  ← Crews角色 + Flows工作流 + Git集成
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

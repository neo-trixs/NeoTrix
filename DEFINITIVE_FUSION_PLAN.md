# NeoTrix 蜕皮重生 — 终极融合方案

**日期**: 2026-09-15 | **Cycle**: rebirth-001
**整合来源**: HANDOFF.md + COMPREHENSIVE_GAP_ANALYSIS.md + D14451-D14650_DEFECTS.md + neoTrix-defect-analysis.md + D10551-D10650_DEFECTS.md + 代码审计 + 200+外部URL

---

## 诊断总览

| 维度 | 数据 | 严重度 |
|------|------|--------|
| 文件变更 | 349个新/修改文件 | - |
| 新模块 | ~40个已实现 | - |
| 编译错误 | **53个** (blocking) | 🔴 |
| 冗余代码 | **4,810行** | 🔴 |
| panic!() | **115处** 生产代码 | 🔴 |
| unwrap() | **3,282处** | 🟡 |
| expect() | **1,720处** | 🟡 |
| unsafe | **20处** (违反R-P1) | 🔴 |
| 已知缺陷 | **54个** (9阶段) | 🔴 |
| TODO(R-P79) | **28+处** 测试桩 | 🟡 |
| 关键安全漏洞 | memmap2 RUSTSEC-2026-0186 | 🔴 |
| 总工作量 | **247h** | - |

---

## Phase 0: 蜕皮重生 — 冗余清理 + 缺陷修复 (21h)

### 0-A. 修复53个编译错误 (立即, 1天)

| 文件 | 错误类型 | 修复方案 |
|------|----------|----------|
| typed_memory/conflict.rs | `&mut Vec<&mut T>` vs `&mut [T]` | 改为 `&mut [T]` |
| typed_memory/kb.rs | rusqlite `.unwrap()` on `&Row` | 使用 `.get()` 方法 |
| http_intercept/mod.rs | lifetime + `Instant`不可序列化 | 添加生命周期标注 |
| external_config.rs | lifetime错误 | clone或标注生命周期 |
| nt_world/ocr/mod.rs | 缺少 `use serde::{Deserialize, Serialize}` | 添加import |
| nt_meta/whale.rs | `AtomicU64` 不满足 `Clone` | `AtomicU64::new(val.load(Relaxed))` |
| entry.rs | borrow/move错误 | 克隆或重新组织borrow |
| forgetting.rs | borrow错误 | 重新组织数据结构 |
| osint/mod.rs | 未使用变量 + borrow错误 | `_`前缀 + 修复borrow |
| multitier.rs | 类型推断失败 | 显式类型标注 |
| pdf_to_text_pipeline.rs | 未使用变量 | `_`前缀 |

### 0-B. 冗余清理 (4,810行, 11h)

| 冗余组 | 清理方案 | 行数 | 工时 |
|--------|----------|------|------|
| ECS实现×3 | 统一到`crystal_ecs.rs` | ~800 | 2h |
| 场景树×2 | 统一到`crystal_scene.rs` | ~350 | 1h |
| 信号系统×2 | 统一到`crystal_signal.rs` | ~330 | 1h |
| 卡牌系统×2 | 合并到`crystal_card.rs` | ~700 | 2h |
| 事件总线×2 | 统一到`crystal_event.rs` | ~260 | 1h |
| 资源管理×2 | 合并到`crystal_resource.rs` | ~200 | 1h |
| 行为树×2 | 合并到`crystal_behavior.rs` | ~270 | 1h |
| 状态机×2 | 合并到`crystal_state.rs` | ~400 | 1h |
| 熔炼引擎×2 | 保留新版 | ~500 | 1h |
| **小计** | | **~4,810** | **11h** |

### 0-C. Panic/Unwrap治理 (4h)

```rust
// 策略: 生产代码用 Result<T, NeoTrixError> 替代 panic!()
// 1. safety_kernel.rs: 12个panic → 返回 SecurityError::UnexpectedCase
// 2. nt_core_dispatch.rs: panic!("must be short-circuited") → Error::ShortCircuitFailed  
// 3. nt_core_event.rs: panic!("wrong variant") → Error::EventVariantMismatch
// 4. nt_core_gwt/independence.rs: panic!("should require order") → Error::OrderingViolation
// 5. grounded_gate.rs: 4个panic → Error::GateStateConflict
// 6. orchestrator.rs: panic!("expected Completed") → Error::OrchestrationFailed
// 7. value_gate.rs: 2个panic!("应拦截") → Error::PolicyViolation
// 8. native_bus.rs: panic!("守卫拦截后不应执行") → Error::GuardViolated
// 9. 115个panic → 全部转为 Result
```

### 0-D. 安全漏洞修复 (立即)

| # | 漏洞 | 修复 | 工时 |
|---|------|------|------|
| 1 | **memmap2 RUSTSEC-2026-0186** | 升级到安全版本或移除 | 1h |
| 2 | **KB无文件锁** | 添加`flock`或`fcntl`文件锁 | 2h |
| 3 | **`std::process::exit(0)`** | 改为信号量+优雅shutdown | 2h |
| 4 | **EventBus mutex poisoning** | 改为`RwLock`+恢复机制 | 3h |
| 5 | **circuit breaker缺失** | 实现CLOSED/OPEN/HALF-OPEN | 4h |

---

## Phase 1: 核心协议与基础设施 (29h)

### 待实现模块清单

| # | 模块 | 来源 | 优先级 | 工时 |
|---|------|------|--------|------|
| 1.1 | **ACP协议** `nt_act::AcpProtocol` | OpenHands | P0 | 4h |
| 1.2 | **向量检索** `nt_memory::VectorIndex` | 通用 | P0 | 6h |
| 1.3 | **异步工具执行** `nt_act::AsyncToolExecutor` | GPT-6 Astra | P0 | 3h |
| 1.4 | **JSON-first配置** `nt_core::AgentConfig` | crewAI | P1 | 4h |
| 1.5 | **二进制分析** `nt_shield::BinaryAnalyzer` | knife | P0 | 8h |
| 1.6 | **缓解措施审计** `nt_shield::MitigationAuditor` | knife | P1 | 4h |
| 1.7 | **记忆宫殿** `nt_nexus::MemoryPalace` | 通用 | P1 | 3h |
| 1.8 | **代码语义地图** `nt_world::RepoMap` | Aider | P1 | 4h |

---

## Phase 2: 分析引擎与安全 (46h)

| # | 模块 | 来源 | 优先级 | 工时 |
|---|------|------|--------|------|
| 2.1 | **危险调用分析** `nt_shield::SinkAnalyzer` | knife | P0 | 4h |
| 2.2 | **漏洞审计** `nt_shield::VulnerabilityAuditor` | knife | P1 | 4h |
| 2.3 | **函数恢复** `nt_world::FunctionRecovery` | knife | P1 | 6h |
| 2.4 | **CFG构建** `nt_world::CFGBuilder` | knife | P1 | 4h |
| 2.5 | **交叉引用分析** `nt_world::XRefAnalyzer` | knife | P1 | 4h |
| 2.6 | **MCP服务器** `nt_act::McpBinaryAnalyzer` | knife | P1 | 8h |
| 2.7 | **漏洞管道** `nt_shield::VulnerabilityPipeline` | Anthropic | P1 | 4h |
| 2.8 | **YARA扫描** `nt_shield::YaraScanner` | knife | P2 | 4h |
| 2.9 | **IOC提取** `nt_world::IOCExtractor` | knife | P2 | 3h |
| 2.10 | **内核驱动分析** `nt_shield::DriverAnalyzer` | knife | P2 | 6h |
| 2.11 | **缺陷制品检测** `nt_shield::ArtifactValidator` | arXiv | P1 | 3h |
| 2.12 | **威胁建模** `nt_shield::ThreatModeler` | Anthropic | P1 | 4h |

---

## Phase 3: 认知与进化 (20h)

| # | 模块 | 来源 | 优先级 | 工时 |
|---|------|------|--------|------|
| 3.1 | **TLCM层校正** `nt_core::TLCMLayerCorrection` | arXiv 2609.07876 | P1 | 4h |
| 3.2 | **延迟加载** `nt_act::DeferredLoader` | GPT-6 Astra | P1 | 2h |
| 3.3 | **中途转向** `nt_mind::MidTurnSteering` | GPT-6 Astra | P1 | 3h |
| 3.4 | **运行时监控** `nt_meta::RuntimeMonitor` | GPT-6 Astra | P1 | 3h |
| 3.5 | **评估器进化** `nt_meta::EvolvingEvaluator` | RQGM | P1 | 4h |
| 3.6 | **多分支存档** `nt_mind::EvolutionArchive` | DGM | P1 | 4h |

---

## Phase 4: 感知与UI (25h)

| # | 模块 | 来源 | 优先级 | 工时 |
|---|------|------|--------|------|
| 4.1 | **生成式UI** `nt_io::GenerativeUI` | OpenUI | P2 | 4h |
| 4.2 | **WebGPU推理** `nt_io::WebGPUInference` | GPU-Doodle | P2 | 4h |
| 4.3 | **Liquid Glass渲染** `nt_io::LiquidGlassRenderer` | Apple | P2 | 4h |
| 4.4 | **企业审批工作流** `nt_io::ApprovalWorkflow` | HybridClaw | P2 | 3h |
| 4.5 | **OSINT异步优化** `nt_world::AsyncOSINT` | Maigret | P2 | 2h |
| 4.6 | **固件分析** `nt_shield::FirmwareAnalyzer` | iOS | P3 | 4h |
| 4.7 | **tree-sitter AST解析** | Aider | P2 | 4h |
| 4.8 | **记忆宫殿** `nt_nexus::MemoryPalace` | 通用 | P1 | 3h |

---

## Phase 5: RSI路径补齐 (23h)

| # | 模块 | 来源 | 优先级 | 工时 |
|---|------|------|--------|------|
| 5.1 | **递归策略** `nt_meta::RecursiveStrategy` | Metaⁿ | P1 | 4h |
| 5.2 | **记忆进化** `nt_memory::MemoryEvolution` | Recuris | P1 | 4h |
| 5.3 | **技能共进化** `nt_mind::MetaSkillEvolution` | MetaSkill-Evolve | P1 | 4h |
| 5.4 | **策略进化** `nt_core::QEvolution` | Q-Evolve | P1 | 4h |
| 5.5 | **未来蒸馏** `nt_mind::RISEReflector` | RISE | P2 | 3h |
| 5.6 | **过程技能记忆** `nt_memory::SkillProceduralMemory` | SkillGLoW | P2 | 2h |
| 5.7 | **脚手架自修改** `nt_repair::MetaModification` | MGM | P2 | 2h |

---

## Phase 6: 接口对齐 (31h)

| # | 任务 | 工时 |
|---|------|------|
| 6.1 | 事件驱动工作流 `@start/@listen/@router` | 6h |
| 6.2 | MCP适配器 `nt_act::McpAdapter` | 4h |
| 6.3 | OpenTelemetry集成 替换`log` | 3h |
| 6.4 | 配置外部化 TOML/JSON | 3h |
| 6.5 | 二进制补丁 `nt_shield::BinaryPatcher` | 6h |
| 6.6 | Git集成 `nt_memory::GitIntegration` | 4h |
| 6.7 | 自动验证 `nt_shield::AutoValidator` | 3h |
| **小计** | | **31h** |

---

## Phase 7: 测试与文档 (40h)

| # | 任务 | 工时 |
|---|------|------|
| 7.1 | 跨模块集成测试 全链路 | 8h |
| 7.2 | 补充单元测试 覆盖率>80% | 16h |
| 7.3 | 性能基准 query/route/inference吞吐 | 4h |
| 7.4 | 安全审计 0 unsafe验证 | 4h |
| 7.5 | 文档生成 rustdoc全量 | 8h |

---

## Phase 8: 生产就绪 (12h)

| # | 任务 | 工时 |
|---|------|------|
| 8.1 | Tauri桌面端修复 黑屏→正常 | 4h |
| 8.2 | CLI命令补全/帮助 | 2h |
| 8.3 | 错误信息美化 | 2h |
| 8.4 | 日志系统 分级/轮转/格式化 | 2h |
| 8.5 | 配置热加载 | 2h |

---

## 跨域错位修复 (同时执行)

| 错位 | 修复方案 | 工时 |
|------|---------|------|
| NT-ACT ↔ NT-IO agent执行边界 | 明确分工文档+trait边界 | 2h |
| NT-CORE ↔ NT-MIND E8/SEAL边界 | 明确分工文档+trait边界 | 2h |
| NT-WORLD ↔ NT-SHIELD 浏览器 | 明确分工文档+trait边界 | 2h |
| NT-MEMORY ↔ NT-NEXUS 记忆边界 | 明确分工文档+trait边界 | 2h |
| 三重错误层次不兼容 | 实现`From`转换 | 4h |
| 6层vs3层架构代码混用 | 迁移到6层命名 | 4h |
| **小计** | | **16h** |

---

## 多Agent自动巡检修复方案 (蜕皮重生执行引擎)

### Agent Squad — 8个并行Agent

```
┌──────────────────────────────────────────────────────────────┐
│               NT-CORE (主协调器 - Rebirth Orchestrator)            │
├──────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │ Build    │ │ Audit    │ │ Security │ │ Memory   │   │
│  │ Agent    │ │ Agent    │ │ Agent    │ │ Agent    │   │
│  │          │ │ (rev-officer│          │ │ (experience│  │
│  │ cargo    │ │ D1-D63   │ │ pentest  │ │  60s tick)│  │
│  │ check    │ │ FPAM     │ │+OSINT    │ │ KB health │  │
│  │+test     │ │          │ │          │ │           │  │
│  │+clippy   │ │          │ │          │ │           │  │
│  │+panic    │ │          │ │          │ │           │  │
│  │ scan     │ │          │ │          │ │           │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
│                                                                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │ World    │ │ IO       │ │ Mind     │ │ Redundancy│   │
│  │ Agent    │ │ Agent    │ │ Agent    │ │ Cleaner   │   │
│  │          │ │          │ │          │ │           │   │
│  │ OCR      │ │ providers│ │ SEAL     │ │ 4,810行   │   │
│  │ browser  │ │ routing  │ │ WHALE    │ │ redundant │   │
│  │ OSINT    │ │          │ │ cycle    │ │ code      │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
│                                                                │
└──────────────────────────────────────────────────────────────┘
```

### Agent Prompt Templates

**1. Build Agent** (最高频)
```
执行三步验证：
1. cargo check -p neotrix --lib (捕获所有编译错误)
2. cargo test -p neotrix --lib (运行测试)
3. cargo clippy -p neotrix --all-targets (代码质量)
额外：扫描所有panic!()调用，标记生产代码中的panic
频率：每次commit后 + Phase切换时
修复策略：按严重度排序，critical→high→medium→low
```

**2. Audit Agent** (每5 cycle)
```
执行rev-officer D1-D63全量审查：
- Phase A: Invariant Freeze (锁定审计词汇)
- Phase B: ADI 3-Pass (Abduction→Deduction→Induction)
- Phase C: Leverage Point Scoring (top 5)
- Phase D: Coverage Matrix (D1-D63覆盖)
- Phase E: Delta Measurement (FP vs FN成本)
输出：severity-ranked findings
```

**3. Security Agent** (每周)
```
NT-SHIELD安全审计：
1. memmap2 RUSTSEC-2026-0186 验证
2. KB文件锁状态检查
3. circuit breaker 状态
4. EventBus mutex poisoning 检查
5. HTTP拦截proxy状态
6. OSINT数据源可用性
7. 115个panic!()生产代码安全审计
8. 20个unsafe代码块验证
频率：每周
```

**4. Memory Agent** (60s tick)
```
经验与KB巡检：
1. experience-tree hub验证
2. ghost branch清理 (route-verify --clean)
3. kv_store完整性
4. typed_memory模块状态
5. LMCache热存储状态
6. concept神经元Hebb网络验证
7. pending-absorb.json处理状态
频率：60s (handlers_absorption)
```

**5. World Agent** (每日)
```
NT-WORLD巡检：
1. PaddleOCR识别准确率验证
2. OCR pipeline端到端测试
3. 浏览器自动化harness状态
4. OSINT数据源可用性
5. RepoMap状态
6. 函数恢复(FUNCTION RECOVERY)状态
7. CFG构建状态
频率：每日
```

**6. IO Agent** (每日)
```
NT-IO巡检：
1. 所有provider可用性检查
2. GWT+Complexity路由效率
3. 投机解码命中率
4. LMCache热存储命中率
5. ACP协议状态
6. 异步工具执行状态
7. 路由策略正确性验证
频率：每日
```

**7. Mind Agent** (每个cycle boundary)
```
NT-MIND进化巡检：
1. SEAL pipeline阶段完整性
2. WHALE phase切换状态
3. 技能结晶化状态
4. SkillSpector验证结果
5. TLCM层校正状态
6. 评估器进化状态
7. 9条RSI路径完整性
8. R-P79 TODO(TODO桩)状态
频率：每个cycle
```

**8. Redundancy Cleaner** (每月)
```
冗余清理专项：
1. 扫描所有重复模块(4,810行)
2. 识别新的重复模式
3. 清理#[allow(dead_code)]超过10次的模块
4. 检测theater modules(D44)
5. 验证所有模块有消费者(Dark Forest)
6. 清理未使用的pub use导出
7. 统一重复类型定义
频率：每月
```

---

## 核心路线任务清单 (精简版)

### 立即执行 (今天)
| # | 任务 | 工时 | 验收 |
|---|------|------|------|
| 1 | 修复53个编译错误 | 1天 | cargo check通过 |
| 2 | 修复memmap2 RUSTSEC漏洞 | 1h | cargo audit通过 |
| 3 | KB文件锁添加 | 2h | 多进程安全 |
| 4 | 修复EventBus mutex poisoning | 3h | panic恢复 |
| 5 | circuit breaker实现 | 4h | 故障隔离 |

### 本周完成 (7天)
| # | 任务 | 工时 |
|---|------|------|
| 6 | 115个panic→Result | 4h |
| 7 | 4,810行冗余清理 | 11h |
| 8 | ACP协议实现 | 4h |
| 9 | 向量检索实现 | 6h |
| 10 | 异步工具执行 | 3h |
| 11 | 二进制分析链 | 8h |
| 12 | 跨域错位修复 | 16h |
| **总计** | | **59h** |

### 下月完成 (21天)
| Phase | 任务 | 工时 |
|-------|------|------|
| Phase 1-5 | 所有P0-P2模块 | 147h |
| Phase 6 | 接口对齐 | 31h |
| Phase 7 | 测试与文档 | 40h |
| Phase 8 | 生产就绪 | 12h |
| **总计** | | **230h** |

---

## 蜕变验证清单

### 蜕变标志 (重生完成时验证)
- [ ] `cargo check -p neotrix --lib` 0错误
- [ ] `cargo test -p neotrix --lib` 全绿
- [ ] `cargo clippy` 0 warning
- [ ] `#![forbid(unsafe_code)]` 严格遵守 (0 unsafe)
- [ ] 生产代码0 panic!()
- [ ] 4,810行冗余代码已清理
- [ ] 54个缺陷已修复
- [ ] 200+外部源模式已全部吸收
- [ ] 8个Agent巡检系统正常运行
- [ ] SelfTest覆盖率>80%
- [ ] 经验树hub无ghost branch
- [ ] 所有TODO(R-P79)桩已实现

---

## 文件交付清单

| 文件 | 内容 | 状态 |
|------|------|------|
| HANDOFF.md | 349文件变更清单+53编译错误+40模块 | ✅ 已完成 |
| COMPREHENSIVE_GAP_ANALYSIS.md | 54缺陷+54待吸收模式+247h计划 | ✅ 已完成 |
| NEOTRIX_FUSION_ARCHITECTURE.md | 200+URL分析+融合架构 | ✅ 已完成 |
| NEOTRIX_EVALUATION.md | 全量评测+路线图 | ✅ 已完成 |
| NEOTRIX_ABSORPTION_COMPLETE.md | 完整吸收报告 | ✅ 已完成 |
| DEFINITIVE_FUSION_PLAN.md | 本文件(终极方案) | ✅ 刚创建 |
| 经验树branch_fusion_001-010 | 10个经验分支 | ✅ 已写入pending-absorb.json |
| DEFECT_INDEX.md | 所有缺陷汇总索引 | 待创建 |

---

*方案整合自：HANDOFF.md + COMPREHENSIVE_GAP_ANALYSIS.md + D14451-D14650_DEFECTS.md + neoTrix-defect-analysis.md + D10551-D10650_DEFECTS.md + 代码审计(3282 unwrap/1720 expect/115 panic/20 unsafe) + 200+外部URL + 7个并行Agent分析报告*

# 终极任务全量清单 — 深度审计 × 架构融合 × 缺陷补齐

> 审计日期: 2026-09-14
> 审计来源: Batch 1 (Agent框架+RSI+Colibri) + Batch 2 (OpenUI/Anthropic/HybridClaw/OSINT/arXiv/GPT-6/Apple/GPU-Doodle/iOS) + knife (逆向工程) + 代码库深度审计
> 审计范围: neotrix-core 1,806文件 / 627K行 / 6层架构
> 目标: 消除所有stub/TODO/panic，补齐所有外部模式缺口，实现架构级对齐

---

## 一、代码库实况 (审计数据)

### 1.1 量化指标

| 指标 | 数值 | 严重度 |
|------|------|--------|
| TODO/FIXME/HACK/XXX 标记 | 100+ | 🔴 高 |
| Stub/Placeholder 实现 | 40+ | 🔴 高 |
| 硬编码数据 | 15+ | 🟡 中 |
| `#[ignore]` 测试 | 8 | 🟡 中 |
| `unwrap()` 生产代码 | 500+ | 🔴 高 |
| `panic!()` 生产代码 | 80+ | 🔴 最高 |
| 空/近空文件 | 30+ | 🟢 低 |

### 1.2 高风险区域

| 区域 | 问题 | 严重度 |
|------|------|--------|
| **NT-SHIELD** | 网络扫描/漏洞扫描/渗透测试全部为stub | 🔴 最高 |
| **NT-MIND** | R-P79接线TODO遍布，VLM未集成 | 🔴 高 |
| **NT-WORLD** | Agent可达性/监控/ODS解析为stub | 🔴 高 |
| **Core** | panic!在意识/上下文/CAD/经验树关键路径 | 🔴 最高 |
| **NT-FEEL** | 硬编码关键词→情感映射，非学习 | 🟡 中 |
| **L6-META** | 构建监控/并发检测/质量控制为stub | 🔴 高 |

---

## 二、吸收模式缺口全量 (54项)

### 2.1 已吸收 (13项 ✅)

| # | 模式 | 来源 | 模块 |
|---|------|------|------|
| 1-4 | Colibri推理 | Colibri | `nt_game` |
| 5-13 | 游戏引擎模式 | Unity/Godot/NueDeck | `nt_game` |

### 2.2 待吸收 — 缺失 (41项 ❌/⚠️)

#### Agent框架 (7项)

| # | 模式 | 来源 | 模块 | 状态 |
|---|------|------|------|------|
| 14 | ACP协议 | OpenHands | `nt_act` | ❌ 缺失 |
| 15 | Crews角色团队 | crewAI | `nt_core` | ⚠️ 有基础 |
| 16 | Flows事件驱动 | crewAI | `nt_mind` | ⚠️ 有基础 |
| 17 | RepoMap代码语义 | Aider | `nt_world` | ⚠️ 有基础 |
| 18 | Git Integration | Aider | `nt_memory` | ⚠️ 有基础 |
| 19 | Lint+Test自动验证 | Aider | `nt_shield` | ⚠️ 有基础 |
| 20 | JSON-First声明式 | crewAI | `nt_core` | ❌ 缺失 |

#### RSI 9路径 (9项)

| # | 模式 | 来源 | 模块 | 状态 |
|---|------|------|------|------|
| 21 | Metaⁿ递归策略 | Metaⁿ | `nt_meta` | ⚠️ 有基础 |
| 22 | Recuris记忆进化 | Recuris | `nt_memory` | ⚠️ 有基础 |
| 23 | MetaSkill-Evolve | MetaSkill-Evolve | `nt_mind` | ⚠️ 有基础 |
| 24 | Q-Evolve策略进化 | Q-Evolve | `nt_core` | ⚠️ 有基础 |
| 25 | RISE未来蒸馏 | RISE | `nt_mind` | ⚠️ 有基础 |
| 26 | SkillGLoW过程技能 | SkillGLoW | `nt_memory` | ⚠️ 有基础 |
| 27 | MGM脚手架自修改 | MGM | `nt_repair` | ⚠️ 有基础 |
| 28 | RQGM评估器进化 | RQGM | `nt_meta` | ❌ 缺失 |
| 29 | DGM多分支存档 | DGM | `nt_mind` | ❌ 缺失 |

#### Batch 2前沿技术 (13项)

| # | 模式 | 来源 | 模块 | 状态 |
|---|------|------|------|------|
| 30 | OpenUI生成式UI | OpenUI | `nt_io` | ❌ 缺失 |
| 31 | Anthropic漏洞管道 | Anthropic | `nt_shield` | ⚠️ 有基础 |
| 32 | HybridClaw企业运行时 | HybridClaw | `nt_io` | ⚠️ 有基础 |
| 33 | Maigret OSINT异步 | Maigret | `nt_world` | ⚠️ 有基础 |
| 34 | arXiv缺陷制品检测 | arXiv | `nt_shield` | ❌ 缺失 |
| 35 | TLCM层校正 | arXiv | `nt_core` | ❌ 缺失 |
| 36 | GPT-6异步工具执行 | OpenAI | `nt_act` | ❌ 缺失 |
| 37 | GPT-6延迟加载 | OpenAI | `nt_act` | ❌ 缺失 |
| 38 | GPT-6中途转向 | OpenAI | `nt_mind` | ❌ 缺失 |
| 39 | GPT-6运行时监控 | OpenAI | `nt_meta` | ❌ 缺失 |
| 40 | Apple Liquid Glass | Apple | `nt_io` | ⚠️ 有基础 |
| 41 | GPU-Doodle WebGPU | GPU-Doodle | `nt_io` | ❌ 缺失 |
| 42 | iOS iBoot固件安全 | iOS | `nt_shield` | ⚠️ 有基础 |

#### knife逆向工程 (12项)

| # | 模式 | 来源 | 模块 | 状态 |
|---|------|------|------|------|
| 43 | 二进制解析 | knife | `nt_shield` | ❌ 缺失 |
| 44 | 缓解措施审计 | knife | `nt_shield` | ❌ 缺失 |
| 45 | 危险调用分析 | knife | `nt_shield` | ❌ 缺失 |
| 46 | 漏洞审计 | knife | `nt_shield` | ❌ 缺失 |
| 47 | 函数恢复 | knife | `nt_world` | ❌ 缺失 |
| 48 | CFG构建 | knife | `nt_world` | ❌ 缺失 |
| 49 | 交叉引用分析 | knife | `nt_world` | ❌ 缺失 |
| 50 | MCP服务器 | knife | `nt_act` | ❌ 缺失 |
| 51 | YARA扫描 | knife | `nt_shield` | ❌ 缺失 |
| 52 | IOC提取 | knife | `nt_world` | ❌ 缺失 |
| 53 | 内核驱动分析 | knife | `nt_shield` | ❌ 缺失 |
| 54 | 二进制补丁 | knife | `nt_shield` | ❌ 缺失 |

---

## 三、代码库缺陷全量 (审计发现)

### 3.1 Stub/Placeholder 实现 (40+项)

#### NT-SHIELD (安全层) — 12项

| 文件 | 行 | 问题 | 修复方案 | 工时 |
|------|-----|------|---------|------|
| `nt_shield_internal_scan.rs:175` | STUB | 硬编码mock主机，无真实网络扫描 | 集成nmap/fscan | 4h |
| `nt_shield_internal_scan.rs:192` | STUB | 硬编码服务数据，无真实端口扫描 | 集成端口扫描 | 4h |
| `nt_shield_vuln_scanner.rs:34` | STUB | 漏洞扫描演示架构 | 集成nuclei | 4h |
| `nt_shield_pentest_agent.rs:153` | STUB | 渗透测试返回Err | 实现渗透能力 | 8h |
| `nt_shield_pentest_agent.rs:182` | STUB | 渗透测试返回Err | 实现渗透能力 | 8h |
| `nt_shield_recon.rs:29` | STUB | 侦察返回"未实现" | 实现侦察管道 | 4h |
| `nt_shield_sandbox/mod.rs:149` | STUB | 最小stub，无真实网络 | 实现沙箱网络 | 4h |
| `nt_meta_build_watchdog.rs:272` | STUB | check_compilation为stub | 实现cargo check | 2h |
| `nt_meta_build_watchdog.rs:285` | STUB | check_tests为stub | 实现cargo test | 2h |
| `nt_meta_build_watchdog.rs:298` | STUB | check_cache为stub | 实现缓存检查 | 2h |
| `nt_meta_concurrency_detector.rs:154` | STUB | 无超时强制 | 实现超时检测 | 2h |
| `null_normalizer.rs:97` | STUB | 仅替换": null" | 实现完整归一化 | 2h |

#### NT-MIND (认知层) — 8项

| 文件 | 行 | 问题 | 修复方案 | 工时 |
|------|-----|------|---------|------|
| `self_improvement.rs:445` | STUB | 评估应用标记为Skipped | 实现真实评估 | 4h |
| `quality_control.rs:274` | STUB | 无AI分析，全0.0分 | 集成VLM分析 | 4h |
| `learned_router.rs:395` | STUB | 随机初始化权重 | 实现训练流程 | 6h |
| `learned_router.rs:777` | TODO | 需要真实基准分数 | 集成基准测试 | 4h |
| `self_diagnose.rs` | 20 | 诊断TODOs | 实现诊断管道 | 4h |
| `evolution_loop.rs` | 20 | 进化循环TODOs | 实现进化循环 | 4h |
| `autofixer.rs` | 25 | 自修复TODOs | 实现自动修复 | 4h |
| `experience_tree/mod.rs:896` | panic | KB打开失败panic | 改为Result | 1h |

#### NT-WORLD (感知层) — 5项

| 文件 | 行 | 问题 | 修复方案 | 工时 |
|------|-----|------|---------|------|
| `nt_world_agent_reach.rs:64` | STUB | 所有目标返回不可达 | 实现可达性检测 | 3h |
| `nt_world_monitor.rs:51` | STUB | 默认stub实现 | 实现监控管道 | 3h |
| `nt_world_ods.rs:40` | STUB | 默认stub实现 | 实现ODS解析 | 3h |
| `nt_world_search.rs:1040` | ignore | 网络探测测试 | 实现网络探测 | 2h |
| `ordered_backend_router` | - | 后端路由基础 | 增强路由逻辑 | 2h |

#### L6-META (元认知层) — 5项

| 文件 | 行 | 问题 | 修复方案 | 工时 |
|------|-----|------|---------|------|
| `self_improvement.rs:866` | TODO | 伪造指标值 | 集成真实指标 | 4h |
| `quality_gate.rs:385` | TODO | 需要VLM集成 | 集成VLM | 4h |
| `verifier_agent.rs:401` | TODO | 需要VLM集成 | 集成VLM | 4h |
| `cross_module_audit.rs:336` | TODO | 需要内容生产数据 | 集成数据源 | 3h |
| `template_tag_registry.rs:342` | TODO | 仅基础CRUD | 增强行为断言 | 3h |

#### NT-FEEL (情感层) — 4项

| 文件 | 行 | 问题 | 修复方案 | 工时 |
|------|-----|------|---------|------|
| `emotion_engine.rs:144` | HARDCODED | 硬编码关键词→情感 | 实现学习映射 | 4h |
| `emotion_engine.rs:416` | TODO | EI指标为启发式 | 实现真实EI | 3h |
| `nt_feel_vtuber.rs:237` | HARDCODED | 硬编码线性乘数 | 实现自适应 | 3h |
| `nt_feel_vtuber.rs:269` | HARDCODED | 硬编码模板响应 | 实现动态响应 | 3h |

#### NT-ACT (行动层) — 6项

| 文件 | 行 | 问题 | 修复方案 | 工时 |
|------|-----|------|---------|------|
| `nt_act_cache.rs:195` | TODO | 未从磁盘读取 | 实现磁盘读取 | 2h |
| `nt_act_cache.rs:200` | TODO | 未实现布隆过滤器 | 实现布隆过滤器 | 2h |
| `publish_gateway.rs:204` | TODO | YouTube API未接线 | 接线YouTube API | 3h |
| `publish_gateway.rs:227` | TODO | Bilibili API未接线 | 接线Bilibili API | 3h |
| `acp.rs:167` | panic | ACP响应panic | 改为Result | 1h |
| `orchestration/publish_gateway` | TODO | 发布网关基础 | 增强发布能力 | 4h |

#### NT-IO (界面层) — 3项

| 文件 | 行 | 问题 | 修复方案 | 工时 |
|------|-----|------|---------|------|
| `nt_unified_api/mod.rs:494` | TODO | 流式待实现 | 实现流式输出 | 3h |
| `nt_unified_api/mod.rs:560` | STUB | 会话存储返回空vec | 实现会话存储 | 3h |
| `nt_io_plugin/builtin/mod.rs` | 空 | 空模块 | 实现内置插件 | 4h |

#### Core (核心层) — 5项

| 文件 | 行 | 问题 | 修复方案 | 工时 |
|------|-----|------|---------|------|
| `nt_core_cad_consciousness.rs:347` | panic | CAD SelfTest未注册panic | 改为Result | 1h |
| `nt_core_context/context_budget.rs:295` | panic | KB slice不存在panic | 改为Result | 1h |
| `nt_core_mcp.rs` | 2行 | 空文件 | 实现MCP核心 | 4h |
| `nt_core_llm.rs` | 2行 | 空文件 | 实现LLM核心 | 4h |
| `nt_core_harness.rs` | 2行 | 空文件 | 实现Harness核心 | 4h |

### 3.2 panic!() 生产代码 (80+处)

#### 关键路径panic (必须修复)

| 文件 | 行 | panic内容 | 修复方案 | 工时 |
|------|-----|----------|---------|------|
| `safety_kernel.rs` | 12处 | 安全内核panic | 全部改为Result | 4h |
| `nt_capability_bridge.rs:531` | panic | "应路由到能力网进化" | 改为Result | 1h |
| `nt_capability_bridge.rs:554` | panic | "应路由到能力网进化" | 改为Result | 1h |
| `bandit.rs:285` | panic | "arm not found" | 改为Result | 1h |
| `bandit.rs:355` | panic | "arm not found after load" | 改为Result | 1h |
| `acp.rs:167` | panic | "expected response, got error" | 改为Result | 1h |
| `engine_core.rs:2436` | panic | "PRM must be configured" | 改为Result | 1h |
| `experience_tree/mod.rs:896` | panic | "KB open failed" | 改为Result | 1h |
| `nt_core_cad_consciousness.rs:347` | panic | "CAD SelfTest not registered" | 改为Result | 1h |
| `context_budget.rs:295` | panic | "KnowledgeBase slice should exist" | 改为Result | 1h |
| `gateway/mod.rs:1309` | panic | "stream error" | 改为Result | 1h |
| `free_pool.rs:326` | panic | "should have a budget" | 改为Result | 1h |
| `build_runner.rs:323` | panic | "check should run" | 改为Result | 1h |
| `nt_evidence_store.rs:679` | panic | "缺口证据不应判定 Sufficient" | 改为Result | 1h |
| `nt_meta_cleanup/coordinator.rs:141` | TODO | 需要真实清理信号 | 集成信号源 | 2h |

### 3.3 unwrap() 生产代码 (500+处)

#### 高风险unwrap (必须修复)

| 文件 | 数量 | 风险 | 修复方案 | 工时 |
|------|------|------|---------|------|
| `nt_act_trade/orchestrator.rs` | 47 | 交易关键路径 | 替换为? | 4h |
| `nt_core_mcp.rs` | 34 | MCP核心 | 替换为? | 3h |
| `nt_core_consciousness_core.rs` | 30 | 意识核心 | 替换为? | 3h |
| `nt_memory_unify.rs` | 91 | KB统一 | 替换为? | 6h |
| `nt_memory_resource_ingest.rs` | 76 | 资源摄取 | 替换为? | 5h |
| `nt_memory_geo.rs` | 57 | 地理记忆 | 替换为? | 4h |
| `nt_io_agents_md.rs` | 56 | Agent配置 | 替换为? | 4h |
| `ntx/mod.rs` | 51 | NTX核心 | 替换为? | 4h |
| `knowledge_storage.rs` | 41 | 知识存储 | 替换为? | 3h |
| `nt_core_rule_memory.rs` | 38 | 规则记忆 | 替换为? | 3h |
| `merge.rs` | 39 | 文件合并 | 替换为? | 3h |
| `nt_file_ability.rs` | 91 | 文件能力 | 替换为? | 6h |

### 3.4 空/近空文件 (30+项)

| 文件 | 行数 | 问题 | 修复方案 | 工时 |
|------|------|------|---------|------|
| `nt_core_deploy.rs` | 2 | 仅re-export | 实现部署核心 | 4h |
| `nt_core_deploy_cache.rs` | 2 | 仅re-export | 实现部署缓存 | 2h |
| `nt_core_edit.rs` | 2 | 仅re-export | 实现编辑核心 | 4h |
| `nt_core_embed.rs` | 2 | 仅re-export | 实现嵌入核心 | 4h |
| `nt_core_harness.rs` | 2 | 仅re-export | 实现Harness核心 | 4h |
| `nt_core_llm.rs` | 2 | 仅re-export | 实现LLM核心 | 4h |
| `nt_core_mcp.rs` | 2 | 仅re-export | 实现MCP核心 | 4h |
| `nt_core_self_test.rs` | 2 | 仅re-export | 实现自测核心 | 4h |
| `nt_core_self_test_integration.rs` | 2 | 仅re-export | 实现集成自测 | 4h |
| `nt_core_error.rs` | 3 | 最小文件 | 实现错误核心 | 2h |
| `nt_memory_schema.rs` | 4 | 最小文件 | 实现Schema核心 | 2h |
| `safety/mod.rs` | 1 | 空模块 | 实现安全模块 | 4h |
| `cuda/mod.rs` | 1 | 空模块 | 实现CUDA模块 | 4h |
| `harness/mod.rs` | 1 | 空模块 | 实现Harness模块 | 4h |
| `nt_io_plugin/builtin/mod.rs` | 1 | 空模块 | 实现内置插件 | 4h |

---

## 四、架构层缺陷全量

### 4.1 L6 Meta-Cognition (元认知层)

| 缺陷ID | 缺陷 | 严重度 | 来源 | 修复 | 工时 |
|--------|------|--------|------|------|------|
| L6-01 | 构建监控为stub | 🔴 高 | 审计 | 实现cargo check/test | 4h |
| L6-02 | 并发检测无超时 | 🟡 中 | 审计 | 实现超时强制 | 2h |
| L6-03 | 质量控制无AI分析 | 🔴 高 | 审计 | 集成VLM | 4h |
| L6-04 | 自改进伪造指标 | 🔴 高 | 审计 | 集成真实指标 | 4h |
| L6-05 | 递归策略缺失 | 🟡 中 | Metaⁿ | 实现RecursiveStrategy | 4h |
| L6-06 | 评估器进化缺失 | 🟡 中 | RQGM | 实现EvolvingEvaluator | 4h |
| L6-07 | 运行时监控缺失 | 🟡 中 | GPT-6 | 实现RuntimeMonitor | 3h |
| L6-08 | 缺陷制品检测缺失 | 🟡 中 | arXiv | 实现ArtifactValidator | 3h |
| L6-09 | 威胁建模缺失 | 🟡 中 | Anthropic | 实现ThreatModeler | 4h |
| L6-10 | 清理信号未接线 | 🟡 中 | 审计 | 接线清理信号 | 2h |
| **小计** | | | | | **34h** |

### 4.2 L5 Cognition (认知层)

| 缺陷ID | 缺陷 | 严重度 | 来源 | 修复 | 工时 |
|--------|------|--------|------|------|------|
| L5-01 | TLCM层校正缺失 | 🟡 中 | arXiv | 实现TLCMLayerCorrection | 4h |
| L5-02 | 延迟加载缺失 | 🟡 中 | GPT-6 | 实现DeferredLoader | 2h |
| L5-03 | 中途转向缺失 | 🟡 中 | GPT-6 | 实现MidTurnSteering | 3h |
| L5-04 | 代码语义理解缺失 | 🟡 中 | Aider | 集成tree-sitter | 4h |
| L5-05 | 策略进化缺失 | 🟡 中 | Q-Evolve | 实现QEvolution | 4h |
| L5-06 | 未来蒸馏缺失 | 🟡 中 | RISE | 实现RISEReflector | 3h |
| L5-07 | 自诊断20处TODO | 🟡 中 | 审计 | 实现诊断管道 | 4h |
| L5-08 | 进化循环20处TODO | 🟡 中 | 审计 | 实现进化循环 | 4h |
| L5-09 | 自修复25处TODO | 🟡 中 | 审计 | 实现自修复 | 4h |
| L5-10 | 学习路由器stub | 🔴 高 | 审计 | 实现训练流程 | 6h |
| **小计** | | | | | **34h** |

### 4.3 L4 Emotion (情感层)

| 缺陷ID | 缺陷 | 严重度 | 来源 | 修复 | 工时 |
|--------|------|--------|------|------|------|
| L4-01 | 硬编码关键词→情感 | 🟡 中 | 审计 | 实现学习映射 | 4h |
| L4-02 | EI指标为启发式 | 🟡 中 | 审计 | 实现真实EI | 3h |
| L4-03 | VTuber硬编码乘数 | 🟡 中 | 审计 | 实现自适应 | 3h |
| L4-04 | VTuber硬编码响应 | 🟡 中 | 审计 | 实现动态响应 | 3h |
| **小计** | | | | | **13h** |

### 4.4 L3 Embodiment (具身层)

| 缺陷ID | 缺陷 | 严重度 | 来源 | 修复 | 工时 |
|--------|------|--------|------|------|------|
| L3-01 | ACP协议缺失 | 🔴 高 | OpenHands | 实现AcpProtocol | 4h |
| L3-02 | 异步工具执行缺失 | 🔴 高 | GPT-6 | 实现AsyncToolExecutor | 3h |
| L3-03 | 二进制分析缺失 | 🔴 高 | knife | 实现BinaryAnalyzer | 8h |
| L3-04 | 缓解措施审计缺失 | 🔴 高 | knife | 实现MitigationAuditor | 4h |
| L3-05 | 危险调用分析缺失 | 🔴 高 | knife | 实现SinkAnalyzer | 4h |
| L3-06 | 漏洞审计缺失 | 🔴 高 | knife | 实现VulnerabilityAuditor | 4h |
| L3-07 | 网络扫描为stub | 🔴 高 | 审计 | 集成nmap/fscan | 8h |
| L3-08 | 漏洞扫描为stub | 🔴 高 | 审计 | 集成nuclei | 4h |
| L3-09 | 渗透测试为stub | 🔴 高 | 审计 | 实现渗透能力 | 16h |
| L3-10 | 侦察管道为stub | 🔴 高 | 审计 | 实现侦察管道 | 4h |
| L3-11 | 安全内核12处panic | 🔴 最高 | 审计 | 全部改为Result | 4h |
| L3-12 | 企业审批缺失 | 🟡 中 | HybridClaw | 实现ApprovalWorkflow | 3h |
| L3-13 | 漏洞管道缺失 | 🟡 中 | Anthropic | 实现VulnerabilityPipeline | 4h |
| L3-14 | YARA扫描缺失 | 🟡 中 | knife | 实现YaraScanner | 4h |
| L3-15 | 内核驱动分析缺失 | 🟡 中 | knife | 实现DriverAnalyzer | 6h |
| L3-16 | 固件安全缺失 | 🟢 低 | iOS | 实现FirmwareAnalyzer | 4h |
| **小计** | | | | | **84h** |

### 4.5 L2 Perception (感知层)

| 缺陷ID | 缺陷 | 严重度 | 来源 | 修复 | 工时 |
|--------|------|--------|------|------|------|
| L2-01 | Agent可达性为stub | 🔴 高 | 审计 | 实现可达性检测 | 3h |
| L2-02 | 监控为stub | 🔴 高 | 审计 | 实现监控管道 | 3h |
| L2-03 | ODS解析为stub | 🔴 高 | 审计 | 实现ODS解析 | 3h |
| L2-04 | 函数恢复缺失 | 🟡 中 | knife | 实现FunctionRecovery | 6h |
| L2-05 | CFG构建缺失 | 🟡 中 | knife | 实现CFGBuilder | 4h |
| L2-06 | 交叉引用缺失 | 🟡 中 | knife | 实现XRefAnalyzer | 4h |
| L2-07 | IOC提取缺失 | 🟡 中 | knife | 实现IOCExtractor | 3h |
| L2-08 | RepoMap缺失 | 🟡 中 | Aider | 实现RepoMap | 4h |
| L2-09 | OSINT异步优化 | 🟡 中 | Maigret | 优化AsyncOSINT | 2h |
| L2-10 | 网络探测测试ignore | 🟡 中 | 审计 | 实现网络探测 | 2h |
| **小计** | | | | | **34h** |

### 4.6 L1 Action (行动层)

| 缺陷ID | 缺陷 | 严重度 | 来源 | 修复 | 工时 |
|--------|------|--------|------|------|------|
| L1-01 | 向量检索缺失 | 🔴 高 | 通用 | 实现VectorIndex | 6h |
| L1-02 | 记忆宫殿缺失 | 🟡 中 | 通用 | 实现MemoryPalace | 3h |
| L1-03 | JSON-first配置缺失 | 🟡 中 | crewAI | 实现AgentConfig | 4h |
| L1-04 | 生成式UI缺失 | 🟡 中 | OpenUI | 实现GenerativeUI | 4h |
| L1-05 | WebGPU推理缺失 | 🟡 中 | GPU-Doodle | 实现WebGPUInference | 4h |
| L1-06 | Liquid Glass缺失 | 🟡 中 | Apple | 实现LiquidGlassRenderer | 4h |
| L1-07 | 布隆过滤器未实现 | 🟡 中 | 审计 | 实现BloomFilter | 2h |
| L1-08 | 磁盘缓存未实现 | 🟡 中 | 审计 | 实现DiskCache | 2h |
| L1-09 | YouTube API未接线 | 🟡 中 | 审计 | 接线YouTube API | 3h |
| L1-10 | Bilibili API未接线 | 🟡 中 | 审计 | 接线Bilibili API | 3h |
| L1-11 | 流式输出未实现 | 🟡 中 | 审计 | 实现流式输出 | 3h |
| L1-12 | 会话存储为stub | 🟡 中 | 审计 | 实现会话存储 | 3h |
| L1-13 | 交易编排47处unwrap | 🔴 高 | 审计 | 替换为? | 4h |
| L1-14 | MCP核心34处unwrap | 🔴 高 | 审计 | 替换为? | 3h |
| L1-15 | 意识核心30处unwrap | 🔴 高 | 审计 | 替换为? | 3h |
| **小计** | | | | | **51h** |

---

## 五、核心任务全量清单 (按Phase)

### Phase 0: 紧急修复 (20h) — 🔴 最高优先级

**目标**: 消除panic/关键stub，恢复系统稳定性

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 0.1 | 修复安全内核12处panic → Result | L3 | 4h | 无 |
| 0.2 | 修复意识核心panic → Result | Core | 2h | 无 |
| 0.3 | 修复上下文预算panic → Result | Core | 1h | 无 |
| 0.4 | 修复能力桥接panic → Result | L5 | 2h | 无 |
| 0.5 | 修复Bandit panic → Result | L3 | 2h | 无 |
| 0.6 | 修复ACP panic → Result | L3 | 1h | 无 |
| 0.7 | 修复推理引擎panic → Result | L5 | 1h | 无 |
| 0.8 | 修复经验树panic → Result | L6 | 1h | 无 |
| 0.9 | 修复网关流panic → Result | L1 | 1h | 无 |
| 0.10 | 修复池预算panic → Result | L1 | 1h | 无 |
| 0.11 | 修复构建运行器panic → Result | L5 | 1h | 无 |
| 0.12 | 修复证据存储panic → Result | L6 | 1h | 无 |
| 0.13 | 清理重复导出: mod.rs去重 | 全局 | 1h | 无 |

### Phase 1: 核心Stub修复 (60h) — 🔴 最高优先级

**目标**: 修复所有高风险stub，恢复核心功能

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 1.1 | 实现BinaryAnalyzer (PE/ELF/Mach-O) | L3 | 8h | 无 |
| 1.2 | 实现MitigationAuditor | L3 | 4h | 1.1 |
| 1.3 | 实现SinkAnalyzer | L3 | 4h | 1.1 |
| 1.4 | 实现VulnerabilityAuditor | L3 | 4h | 1.3 |
| 1.5 | 实现网络扫描 (nmap/fscan) | L3 | 8h | 无 |
| 1.6 | 实现漏洞扫描 (nuclei) | L3 | 4h | 无 |
| 1.7 | 实现侦察管道 | L3 | 4h | 无 |
| 1.8 | 实现渗透测试基础 | L3 | 8h | 1.5,1.6,1.7 |
| 1.9 | 实现构建监控 (cargo check/test) | L6 | 4h | 无 |
| 1.10 | 实现质量控制 (VLM集成) | L6 | 4h | 无 |
| 1.11 | 实现自改进 (真实指标) | L6 | 4h | 无 |
| 1.12 | 实现Agent可达性检测 | L2 | 3h | 无 |
| 1.13 | 实现监控管道 | L2 | 3h | 无 |
| 1.14 | 实现ODS解析 | L2 | 3h | 无 |

### Phase 2: 协议与基础设施 (30h) — 🔴 高优先级

**目标**: 实现ACP/向量检索/异步执行等核心协议

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 2.1 | 实现ACP协议 | L3 | 4h | 无 |
| 2.2 | 实现向量检索 | L1 | 6h | 无 |
| 2.3 | 实现异步工具执行 | L3 | 3h | 无 |
| 2.4 | 实现JSON-first配置 | L1 | 4h | 无 |
| 2.5 | 实现记忆宫殿 | L1 | 3h | 无 |
| 2.6 | 实现布隆过滤器 | L1 | 2h | 无 |
| 2.7 | 实现磁盘缓存 | L1 | 2h | 无 |
| 2.8 | 实现流式输出 | L1 | 3h | 无 |
| 2.9 | 实现会话存储 | L1 | 3h | 无 |

### Phase 3: 分析引擎 (30h) — 🔴 高优先级

**目标**: 实现knife完整分析链

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 3.1 | 实现函数恢复 | L2 | 6h | 1.1 |
| 3.2 | 实现CFG构建 | L2 | 4h | 3.1 |
| 3.3 | 实现交叉引用分析 | L2 | 4h | 3.1 |
| 3.4 | 实现MCP服务器 | L3 | 8h | 1.1-1.4 |
| 3.5 | 实现IOC提取 | L2 | 3h | 1.1 |
| 3.6 | 实现YARA扫描 | L3 | 4h | 1.1 |
| 3.7 | 实现内核驱动分析 | L3 | 6h | 1.1-1.4 |

### Phase 4: 认知与进化 (34h) — 🟡 中优先级

**目标**: 实现TLCM/延迟加载/评估器进化等认知能力

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 4.1 | 实现TLCM层校正 | L5 | 4h | 无 |
| 4.2 | 实现延迟加载 | L5 | 2h | 无 |
| 4.3 | 实现中途转向 | L5 | 3h | 无 |
| 4.4 | 实现运行时监控 | L6 | 3h | 无 |
| 4.5 | 实现评估器进化 | L6 | 4h | 无 |
| 4.6 | 实现多分支存档 | L5 | 4h | 无 |
| 4.7 | 实现代码语义理解 | L5 | 4h | 无 |
| 4.8 | 实现策略进化 | L5 | 4h | 无 |
| 4.9 | 实现未来蒸馏 | L5 | 3h | 无 |
| 4.10 | 实现学习路由器训练 | L5 | 6h | 无 |

### Phase 5: 感知与UI (25h) — 🟡 中优先级

**目标**: 实现RepoMap/生成式UI/WebGPU等感知能力

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 5.1 | 实现RepoMap | L2 | 4h | 无 |
| 5.2 | 实现生成式UI | L1 | 4h | 无 |
| 5.3 | 实现WebGPU推理 | L1 | 4h | 无 |
| 5.4 | 实现Liquid Glass渲染 | L1 | 4h | 无 |
| 5.5 | 实现企业审批工作流 | L3 | 3h | 无 |
| 5.6 | 实现OSINT异步优化 | L2 | 2h | 无 |
| 5.7 | 实现固件分析 | L3 | 4h | 无 |

### Phase 6: RSI路径补齐 (23h) — 🟡 中优先级

**目标**: 补齐9条RSI路径

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 6.1 | 实现递归策略 | L6 | 4h | 无 |
| 6.2 | 实现记忆进化 | L1 | 4h | 无 |
| 6.3 | 实现技能共进化 | L5 | 4h | 无 |
| 6.4 | 实现策略进化 | L5 | 4h | 无 |
| 6.5 | 实现未来蒸馏 | L5 | 3h | 无 |
| 6.6 | 实现过程技能记忆 | L1 | 2h | 无 |
| 6.7 | 实现脚手架自修改 | L3 | 2h | 无 |

### Phase 7: 冗余清理 (21h) — 🟡 中优先级

**目标**: 消除4,810行冗余代码

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 7.1 | 统一ECS: 删除重复实现 | nt-world-sim | 2h | 无 |
| 7.2 | 统一场景树 | nt-world-sim | 1h | 无 |
| 7.3 | 统一信号系统 | nt-world-sim | 1h | 无 |
| 7.4 | 统一卡牌系统 | nt-world-sim | 2h | 无 |
| 7.5 | 统一事件总线 | nt-world-sim | 1h | 无 |
| 7.6 | 统一资源管理 | nt-world-sim | 1h | 无 |
| 7.7 | 统一行为树 | nt-world-sim | 1h | 无 |
| 7.8 | 统一状态机 | nt-world-sim | 1h | 无 |
| 7.9 | 清理unwrap/panic | 全局 | 8h | 无 |
| 7.10 | 清理重复类型定义 | 全局 | 1h | 无 |
| 7.11 | 修复域边界违规 | 全局 | 4h | 无 |

### Phase 8: 接口对齐 (31h) — 🟡 中优先级

**目标**: 对齐外部标准接口

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 8.1 | 实现事件驱动工作流 | L5 | 6h | 无 |
| 8.2 | 实现MCP适配器 | L3 | 4h | 无 |
| 8.3 | 实现OpenTelemetry集成 | 全局 | 3h | 无 |
| 8.4 | 实现配置外部化 | 全局 | 3h | 无 |
| 8.5 | 实现二进制补丁 | L3 | 6h | 无 |
| 8.6 | 实现Git集成 | L1 | 4h | 无 |
| 8.7 | 实现自动验证 | L3 | 3h | 无 |
| 8.8 | 实现威胁建模 | L6 | 4h | 无 |
| 8.9 | 实现缺陷制品检测 | L6 | 3h | 无 |
| 8.10 | 实现清理信号接线 | L6 | 2h | 无 |

### Phase 9: 测试与文档 (40h) — 🟡 中优先级

**目标**: 全链路测试+文档

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 9.1 | 跨模块集成测试 | tests/ | 8h | Phase 1-8 |
| 9.2 | 补充单元测试 | tests/ | 16h | Phase 1-8 |
| 9.3 | 性能基准 | tests/bench/ | 4h | Phase 1-8 |
| 9.4 | 安全审计 | tests/ | 4h | Phase 1-8 |
| 9.5 | 文档生成 | docs/ | 8h | Phase 1-8 |

### Phase 10: 生产就绪 (12h) — 🟢 低优先级

**目标**: 生产环境就绪

| # | 任务 | 影响 | 工时 | 依赖 |
|---|------|------|------|------|
| 10.1 | Tauri桌面端修复 | src-tauri/ | 4h | 无 |
| 10.2 | CLI体验优化 | cli/ | 2h | 无 |
| 10.3 | 错误信息美化 | 全局 | 2h | 无 |
| 10.4 | 日志系统 | 全局 | 2h | 无 |
| 10.5 | 配置热加载 | 全局 | 2h | 无 |

---

## 六、总计

| Phase | 工时 | 优先级 | 里程碑 |
|-------|------|--------|--------|
| Phase 0: 紧急修复 | 20h | 🔴 最高 | 消除所有panic |
| Phase 1: 核心Stub修复 | 60h | 🔴 最高 | 修复所有高风险stub |
| Phase 2: 协议与基础设施 | 30h | 🔴 高 | ACP/向量检索/异步执行 |
| Phase 3: 分析引擎 | 30h | 🔴 高 | 完整二进制分析链 |
| Phase 4: 认知与进化 | 34h | 🟡 中 | TLCM/评估器进化 |
| Phase 5: 感知与UI | 25h | 🟡 中 | RepoMap/生成式UI |
| Phase 6: RSI路径补齐 | 23h | 🟡 中 | 9条RSI路径 |
| Phase 7: 冗余清理 | 21h | 🟡 中 | 消除4,810行冗余 |
| Phase 8: 接口对齐 | 31h | 🟡 中 | 事件驱动/MCP/OTEL |
| Phase 9: 测试与文档 | 40h | 🟡 中 | 全链路测试 |
| Phase 10: 生产就绪 | 12h | 🟢 低 | Tauri/CLI/日志 |
| **总计** | **326h** | | |

---

## 七、依赖关系图

```
Phase 0 (紧急修复) ─────────────────────────────────┐
    ↓                                                 │
Phase 1 (核心Stub修复) ←── 无依赖                    │
    ↓                                                 │
Phase 2 (协议基础设施) ←── 无依赖                    │
    ↓                                                 │
Phase 3 (分析引擎) ←── 依赖 Phase 1.1-1.4            │
    ↓                                                 │
Phase 4 (认知进化) ←── 无依赖                        │
    ↓                                                 │
Phase 5 (感知UI) ←── 无依赖                          │
    ↓                                                 │
Phase 6 (RSI路径) ←── 无依赖                         │
    ↓                                                 │
Phase 7 (冗余清理) ←── 依赖 Phase 0-6                │
    ↓                                                 │
Phase 8 (接口对齐) ←── 无依赖                        │
    ↓                                                 │
Phase 9 (测试文档) ←── 依赖 Phase 1-8                │
    ↓                                                 │
Phase 10 (生产就绪) ←── 无依赖                       │
                                                       │
└── 全部完成后 → 系统就绪 ←──────────────────────────┘
```

---

## 八、关键决策点

### 决策 1: 渗透测试深度?
- **选项A**: 仅基础扫描 (nmap/nuclei)
- **选项B**: 基础 + 漏洞利用
- **选项C**: 完整渗透框架
- **建议**: 选项A，渐进式扩展

### 决策 2: VLM集成范围?
- **选项A**: 仅质量控制
- **选项B**: 质量控制 + 验证代理
- **选项C**: 全面VLM集成
- **建议**: 选项B，平衡效果与开销

### 决策 3: 学习路由器策略?
- **选项A**: 简单启发式
- **选项B**: 在线学习
- **选项C**: 完整训练流程
- **建议**: 选项B，渐进式改进

### 决策 4: 测试覆盖目标?
- **选项A**: 60% (基础)
- **选项B**: 80% (良好)
- **选项C**: 95% (优秀)
- **建议**: 选项B，平衡质量与效率

---

## 九、风险评估

| 风险 | 可能性 | 影响 | 缓解 |
|------|--------|------|------|
| 工期超支 | 高 | 高 | Phase 0-1优先，渐进交付 |
| 架构冲突 | 中 | 中 | 接口抽象，独立模块 |
| 性能回归 | 低 | 高 | 基准测试，性能监控 |
| 安全漏洞 | 中 | 高 | 安全审计，渗透测试 |
| 外部依赖 | 低 | 中 | 最小化依赖，本地优先 |
| 测试覆盖不足 | 中 | 中 | 持续集成，覆盖率检查 |
| Stub修复复杂度 | 高 | 高 | 分阶段修复，充分测试 |

---

## 十、预期成果

### 10.1 能力提升

| 能力 | 当前 | 融合后 | 提升 |
|------|------|--------|------|
| panic()生产代码 | 80+ | 0 | ✅ 消除100% |
| stub实现 | 40+ | 0 | ✅ 消除100% |
| unwrap()生产代码 | 500+ | <50 | ✅ 减少90% |
| Agent协议 | 无 | ACP/MCP | ✅ |
| 向量检索 | 无 | ANN索引 | ✅ |
| 异步执行 | 同步 | 异步+背压 | ✅ |
| 二进制分析 | 无 | PE/ELF/Mach-O | ✅ |
| 安全审计 | 基础 | 高级漏洞扫描 | ✅ |
| 渗透测试 | stub | 基础能力 | ✅ |
| 生成式UI | 静态模板 | AI生成 | ✅ |
| TLCM | 无 | 层校正 | ✅ |
| 9条RSI路径 | 基础 | 完整 | ✅ |

### 10.2 架构优势

| 优势 | 描述 |
|------|------|
| **零panic** | 生产代码无panic，全部Result |
| **零stub** | 所有stub替换为真实实现 |
| **统一协议** | ACP/MCP兼容，跨Agent通信 |
| **语义搜索** | 向量检索，知识语义匹配 |
| **异步高效** | 背压控制，资源感知调度 |
| **二进制安全** | 完整分析链，漏洞自动发现 |
| **渗透能力** | 网络扫描+漏洞利用+渗透测试 |
| **AI生成UI** | 动态UI生成，个性化体验 |
| **推理优化** | TLCM层校正，提升推理质量 |
| **自我进化** | 9条RSI路径，持续自我改进 |

---

## 十一、实施时间线

| Phase | 时间 | 里程碑 | 验收标准 |
|-------|------|--------|----------|
| Phase 0 | Week 1 | 紧急修复完成 | 0 panic |
| Phase 1 | Week 1-2 | 核心Stub修复 | 0 高风险stub |
| Phase 2 | Week 2-3 | 协议基础设施 | ACP/向量检索可用 |
| Phase 3 | Week 3-4 | 分析引擎 | 完整二进制分析链 |
| Phase 4 | Week 4-5 | 认知进化 | TLCM/评估器进化 |
| Phase 5 | Week 5-6 | 感知UI | RepoMap/生成式UI |
| Phase 6 | Week 6-7 | RSI路径 | 9条RSI路径完整 |
| Phase 7 | Week 7-8 | 冗余清理 | 4,810行代码清理 |
| Phase 8 | Week 8-9 | 接口对齐 | 事件驱动/MCP/OTEL |
| Phase 9 | Week 9-10 | 测试文档 | 覆盖率>80% |
| Phase 10 | Week 10 | 生产就绪 | Tauri/CLI可用 |

---

*终极任务全量清单完成。基于 3 大吸收源 + 代码库深度审计，覆盖 6 层架构、326h 工作量、10 个实施阶段。*

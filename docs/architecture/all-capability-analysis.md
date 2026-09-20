# NeoTrix 全能力域统一分析

> 生成日期: 2026-09-20
> 数据源: 2,413 Rust 文件 + 7 crates + 68 skills + 65 前端组件
> 覆盖: 12 个能力域, 60+ 散落实现, 4 个死代码文件

---

## 一、能力域总览

| # | 能力域 | 实现数 | 重复数 | 状态 | 优先级 |
|---|--------|--------|--------|------|--------|
| 1 | Search/Retrieval | 6+ | 3 份 BM25+RRF | 🔴 严重碎片化 | P0 |
| 2 | Memory | 20+ | 88 文件巨石 | 🔴 巨石+碎片 | P0 |
| 3 | Gateway/Router | 6+ | 3 份 Gateway | 🔴 严重重复 | P0 |
| 4 | Evolution | 10+ | 2 SelfModel, 2 SEAL | 🔴 严重重复 | P1 |
| 5 | Agent/Loop | 9 | 2 Hive, 3+ Loop | 🟡 部分重复 | P1 |
| 6 | Pipeline | 7 | 2 Pipeline trait | 🟡 部分重复 | P1 |
| 7 | Security/Shield | 16 | 2 检查树 | 🟡 拆分 | P2 |
| 8 | Health/Monitoring | 8 | 3+ 断路器 | 🟡 碎片化 | P2 |
| 9 | Cache/Storage | 9 | 4+ KV, 3+ Cache | 🟡 重复 | P2 |
| 10 | IO/Provider | 11 | 3+ 断路器 | 🟢 相对清晰 | P2 |
| 11 | Config | 6 | 3+ 层级 | 🟢 可接受 | P3 |
| 12 | Event/Message | 7 | 无统一总线 | 🟡 缺失 | P3 |

---

## 二、12 域详细分析

### 2.1 Search/Retrieval — 6+ 实现, 3 份 BM25+RRF

| # | 模块 | 层级 | 核心算法 | 重叠 |
|---|------|------|---------|------|
| 1a | `nt_memory/hybrid_retrieval/` | L4 | BM25+TF-IDF+Entity+Temporal+RRF | 基准 |
| 1b | `nt_core_hybrid_search.rs` | L5 | BM25+Vector+RRF (代码) | 与 1a 重复 |
| 1c | `neotrix-gateway/hybrid_search.rs` | L5 | BM25+Vector+RRF (**第 3 份**) | 与 1a/1b 重复 |
| 1d | `nt_infra_unified_search.rs` | L1 | 统一入口 (路由层) | 编排层 |
| 1e | `nt_core_code_search.rs` | L2 | ripgrep+RRF | 部分重叠 |
| 1f | `nt_memory_search.rs` | L4 | Graph+FTS5+Vector | 与 1a 重叠 |
| 1g | `retrieval_engine.rs` | L4 | 可配置检索管线 | 与 1d 重叠 |
| 1h | `nt_memory_adaptive_rag.rs` | L4 | 查询重写+评分循环 | 与 1f 重叠 |
| 1i | `nt_memory_e8_agent.rs` | L4 | E8 状态机驱动检索 | 独特 |
| 1k | `bm25.rs` | L4 | 独立 BM25 | 与 1a 重复 |
| 1l | `knowledge_mgmt.rs` | L5 | Gateway 知识搜索 | 与 1c 重复 |

**判定**: 🔴 3 份 BM25+RRF, 5+ SearchResult 类型
**目标**: 统一为 `neotrix-search` crate — 一个 SearchBackend trait, 一个 HybridRetriever, 一个 SearchResult 类型

### 2.2 Memory — 20+ 实现, 88 文件巨石

| # | 模块 | 层级 | 重叠 |
|---|------|------|------|
| 2a | `nt_memory/` (根) | L4 | 五层级联 (sensory→working→short→episodic→long) |
| 2b | `nt_memory_kb/` (**88 files!**) | L4 | 🟡 **巨石模块** |
| 2c | `nt_core_graph_memory.rs` | L1 | PageRank 图记忆 |
| 2d | `nt_memory_spatial/` | L1 | 地理空间记忆 |
| 2e | `nt_memory_historian/` | L4 | 证据/时间事实 |
| 2f | `typed_memory/` | L4 | 类型语义记忆 |
| 2g | `consolidation/` | L4 | 短期→长期晋升 |
| 2h | `distillation/` | L4 | 记忆蒸馏 |
| 2i | `tiered_memory/` | L4 | 分层记忆 |
| 2j | `nt_memory_knowledge_graph/` | L4 | 知识图谱记忆 |
| 2k | `lmcache_hotstore.rs` | L4 | KV 缓存 (256 MiB) |
| 2l | `context_fs.rs` | L4 | 上下文文件系统 |
| 2m | `trinity.rs` | L4 | 腾讯 Agent Memory |
| 2n | `memory_types.rs` | L4 | Grok 三类型 |
| 2o | `nt_core_second_brain.rs` | L5 | 第二大脑 |
| 2p | `layered_memory.rs` (types) | L0 | 5 层存根 |
| 2q | `second_brain.rs` (consciousness) | L5 | 意识感知第二大脑 |
| 2r | `vector_index.rs` | L4 | 向量索引 |
| 2s | `nt_memory_paged_kv.rs` | L4 | 分页 KV |
| 2t | `decay_forgetting/` | L4 | 艾宾浩斯遗忘 |
| 2u | `admission_control/` | L4 | 准入控制 |
| 2v | `entity_linking/` | L4 | 实体链接 |
| 2w | `selective_memory.rs` | L4 | 选择性记忆 |

**判定**: 🔴 88 文件巨石 + 15+ 重叠
**目标**: 拆分为 focused crates: `neotrix-memory-core`, `neotrix-memory-kb`, `neotrix-memory-spatial`, `neotrix-memory-historian`

### 2.3 Gateway/Router — 6+ 实现, 3 份 Gateway

| # | 模块 | 层级 | 重叠 |
|---|------|------|------|
| 4a | `model_gateway.rs` (gateway crate) | L5 | 🟡 Gateway #1 |
| 4b | `nt_core_model_gateway.rs` (core) | L5 | 🟡 Gateway #2 (**重复**) |
| 4c | `provider/gateway/` (1679 lines) | L1 | 🟢 **真实 Gateway** |
| 4d | `model_router.rs` (gateway crate) | L5 | 🟡 Router #1 |
| 4e | `nt_core_model_router.rs` (core) | L5 | 🟡 Router #2 (**重复**) |
| 4f | `semantic_router.rs` (gateway) | L5 | 🔴 **空文件** |
| 4g | `nt_infra_semantic_router.rs` | L1 | 关键词+嵌入+LLM |
| 4h | `model_routing.rs` (IO) | L1 | IO 层路由 |
| 4i | `gate.rs` (gateway) | L5 | Guardrail Gate |
| 4j | `gate.rs` (reasoning) | L5 | 推理 Gate |
| 4k | `nt_core_gate/` | L5 | 核心 Gate |
| 4l | `triage.rs` (tauri) | — | 轻量分类器 |
| 4m | `provider/gateway/routing/` | L1 | 能力路由+推理路由 |

**判定**: 🔴 3 份 Gateway, 2 份 Router, 4+ Gate
**目标**: 删除 4a/4b 重复, 保留 4c 为唯一入口; 统一 Gate 到一处

### 2.4 Evolution/Self-Improvement — 5+ 实现, 2 空文件

| # | 模块 | 层级 | 重叠 |
|---|------|------|------|
| 12a | `evolution/` (L5) | L5 | SelfEvolver+AutoFixer+SelfDiagnose |
| 12b | `evolution/evolution_loop/` (L6) | L6 | OODA 循环 |
| 12c | `self_improvement/` (L5) | L5 | MemGPT 模式 |
| 12d | `evolution.rs` (reasoning crate) | — | 🔴 **空文件** |
| 12e | `self_improvement.rs` (multi-agent) | — | 🔴 **空文件** |
| 12f | `planner.rs` (types) | L0 | 进化规划器 |
| 12g | `metacognition_loop.rs` (types) | L0 | 元认知循环 |
| 12h | `unified_self_model.rs` (types) | L0 | 统一自我模型 |
| 12i | `self_model.rs` (types) | L0 | 自我模型 (**重复**) |
| 12k | `arch_fitness.rs` (reasoning) | L5 | 架构适应度 |
| 12l | `nt_core_arch_fitness.rs` | L5 | 架构适应度 (**重复**) |
| 12p | `seal/` (L5, 16-stage) | L5 | SEAL 自迭代 |
| 12q | `seal.rs` (reasoning crate) | L5 | SEAL (**重复**) |
| 12r | `auto_repair.rs` (L6) | L6 | 自动修复 |

**判定**: 🔴 5+ 进化系统, 2 SelfModel, 2 SEAL, 2 ArchFitness, 2 空文件
**目标**: 统一进化管道; 删除重复和死代码

### 2.5 Agent/Loop — 9 实现, Hive 重复

| # | 模块 | 层级 | 重叠 |
|---|------|------|------|
| 3a | `nt_io_agent_loop.rs` | L1 | 核心 AgentLoop |
| 3b | `nt_io_hive_agent_loop.rs` | L1 | Hive 装饰器 |
| 3c | `nt_mind_background_loop/` | L5 | 30+ 字段主循环 |
| 3d | `background_loop_manager.rs` (L6) | L6 | 多实例管理 |
| 3e | `background_loop.rs` (multi-agent) | L5 | 多 Agent 循环 |
| 3f | `hive.rs` (gateway) | L5 | **Gateway Hive (重复)** |
| 3g | `hive.rs` (multi-agent) | L5 | **Multi-Agent Hive (重复)** |
| 3h | `nt_memory_e8_agent.rs` | L4 | E8 状态机循环 |
| 3j | `nt_core_iter_agent.rs` (types) | L0 | 迭代 Agent |

**判定**: 🔴 Hive 双份, 3+ BackgroundLoop
**目标**: 统一 Hive 到一个 crate, BackgroundLoopManager 管理所有循环类型

### 2.6 Pipeline — 7 实现, 2 Pipeline trait

| # | 模块 | 层级 | 重叠 |
|---|------|------|------|
| 7a | `nt_core_platform/pipeline.rs` | L0 | 基础 Pipeline trait |
| 7b | `pipeline_registry.rs` | L0 | 注册表 |
| 7c | `nt_memory_pipeline.rs` | L4 | 记忆管线 |
| 7d | `retrieval_engine.rs` | L4 | 检索管线 |
| 7f | `nt_mind_unified_pipeline.rs` | L5 | 统一思维管线 |
| 7l | `tiered_pipeline/` | L4 | 🟡 **第 2 份 Pipeline trait** |

**判定**: 🟡 2 份 Pipeline trait
**目标**: 统一到一处

### 2.7 Security/Shield — 16 实现, 2 检查树

| # | 模块 | 层级 | 重叠 |
|---|------|------|------|
| 5a | `nt_shield/` (39 modules) | L3 | 🟢 主安全系统 |
| 5b | `shield_core/` | L3 | 上下文边界+守卫链 |
| 5c | `guard/` | L3 | InputGatekeeper+OutputSentinel+PromptGuardian |
| 5d | `defense/` | L3 | 防御层+守卫栏遍历 |
| 5e | `evasion/` | L3 | 逃逸引擎 |
| 5f | `nt_shield_sandbox/` | L3 | 沙箱 |
| 5g | `nt_shield_stealth_net/` | L3 | 隐身网络 |
| 5h | `scanners/` | L3 | 安全扫描 |
| 5i | `nt_security/` | L3 | 🟡 **独立安全模块 (重复)** |
| 5j | `nt_core_guard_chain.rs` | L0 | 核心守卫链 |
| 5k | `nt_safety_monitor.rs` | L6 | 元层安全监控 |
| 5l | `gate.rs` (gateway) | L5 | Gateway 守卫栏 |
| 5m | `SecurityPlugin` (tauri) | — | 桌面安全插件 |
| 5n | `permission_dialog.rs` | — | 权限对话框 |
| 5o | `vault.rs` | — | 凭证保险库 |
| 5p | `nt_memory_write_guard.rs` | L4 | 记忆写守卫 |

**判定**: 🟡 nt_security 独立于 nt_shield
**目标**: 合并 nt_security 到 nt_shield; 统一所有 Guard 实现

### 2.8 Health/Monitoring — 8 实现, 3+ 断路器

| # | 模块 | 层级 | 重叠 |
|---|------|------|------|
| 11a | `platform/health.rs` | L0 | 基础 HealthCheck |
| 11b | `self_healing/health_monitor.rs` | L6 | ComponentMonitor |
| 11c | `consciousness_monitor.rs` | L6 | 意识监控 |
| 11f | `nt_core_self_test.rs` | L6 | 自测试 |
| 11g | `runtime_monitor.rs` | L6 | 运行时监控 |
| 11h | `metacognitive_evaluator.rs` (types) | L0 | 认知健康报告 |
| 11n | `provider/health/` | L1 | 提供商健康 |
| 11o | `nt_infra_breaker.rs` | L1 | 基础设施断路器 |

**判定**: 🟡 5+ 健康系统, 3+ 断路器
**目标**: 统一健康系统: 一个 HealthCheck trait + 一个 ComponentMonitor

### 2.9 Cache/Storage — 9 实现, 4+ KV

| # | 模块 | 层级 | 重叠 |
|---|------|------|------|
| 9a | `consolidation/cache.rs` | L4 | TTL 缓存 |
| 9b | `lmcache_hotstore.rs` | L4 | LRU KV (256 MiB) |
| 9c | `spatial/cache.rs` | L1 | 空间瓦片缓存 |
| 9d | `graph_cache.rs` | L4 | 图查询缓存 |
| 9j | `kv_store` (tauri) | — | SQLite KV |
| 9k | `nt_memory_paged_kv.rs` | L4 | 分页 KV |
| 9l | `addressable_store.rs` | L4 | 寻址存储 |
| 9h | `compaction.rs` (provider) | L1 | 上下文压缩 |
| 9i | `context_compaction.rs` | L4 | 记忆上下文压缩 |

**判定**: 🟡 4+ KV, 3+ Cache
**目标**: 统一 KV 接口, 合并 Cache 实现

### 2.10 Config — 6 实现, 3+ 层级

| # | 模块 | 层级 | 重叠 |
|---|------|------|------|
| 10a | `default-config.toml` | — | 默认配置 |
| 10b | `config.rs` (core) | — | 核心配置 |
| 10c | `config.rs` (tauri) | — | 桌面配置 |
| 10d | `platform/config.rs` | L0 | 平台配置 |
| 10e | `background_loop/config.rs` | L5 | 循环配置 |
| 10f | `provider/config/` | L1 | 提供商配置 |

**判定**: 🟡 3+ 配置层级
**目标**: 集中化配置层次

### 2.11 Event/Message — 7 实现, 无统一总线

| # | 模块 | 层级 | 重叠 |
|---|------|------|------|
| 8a | Hive EventLog | L5 | Hive 事件日志 |
| 8b | element_bus.rs | L5 | 元素级消息总线 |
| 8c | nt_core_event.rs (types) | L0 | 核心事件类型 |
| 8d | channels.rs (tauri) | — | 桌面 IPC |
| 8g | HiveRouter | L5 | 中央消息路由 |
| 8h | nt_io_messaging.rs | L1 | IO 层消息 |
| 8i | nt_io_notify.rs | L1 | 通知系统 |

**判定**: 🟡 无统一事件总线
**目标**: 创建 nt_core_event_bus 统一事件系统

---

## 三、死代码检测

| 文件 | 大小 | 状态 |
|------|------|------|
| `crates/neotrix-gateway/src/semantic_router.rs` | 0 bytes | 🔴 空文件 |
| `crates/neotrix-reasoning/src/evolution.rs` | 0 bytes | 🔴 空文件 |
| `crates/neotrix-multi-agent/src/self_improvement.rs` | 0 bytes | 🔴 空文件 |
| `crates/neotrix-multi-agent/src/experience_tree.rs` | 0 bytes | 🔴 空文件 |

---

## 四、统一融合优先级矩阵

| 优先级 | 域 | 问题 | 重复数 | 工时 | 目标 |
|--------|-----|------|--------|------|------|
| **P0** | Memory | 88 文件巨石 + 15+ 重叠 | 20+ | 40h | 拆分为 focused crates |
| **P0** | Search | 3 份 BM25+RRF, 5+ SearchResult | 6+ | 16h | neotrix-search crate |
| **P0** | Gateway | 3 份 Gateway, 2 份 Router | 6+ | 20h | 单一入口 |
| **P1** | Evolution | 5+ 进化系统, 2 SelfModel | 10+ | 24h | 统一进化管道 |
| **P1** | Agent Loop | Hive 双份, 3+ BackgroundLoop | 5+ | 16h | 统一循环管理 |
| **P1** | Pipeline | 2+ Pipeline trait | 7+ | 8h | 统一 trait |
| **P2** | Security | nt_security vs nt_shield | 2 trees | 12h | 合并 |
| **P2** | Health | 5+ 健康系统, 3+ 断路器 | 8+ | 12h | 统一健康系统 |
| **P2** | Cache/Storage | 4+ KV, 3+ Cache | 9+ | 10h | 统一接口 |
| **P3** | Config | 3+ 配置层级 | 6+ | 6h | 集中化 |
| **P3** | Event | 无统一总线 | 7+ | 8h | nt_core_event_bus |

---

## 五、全量融合方案 (Final)

| Phase | 内容 | 工时 | 状态 |
|-------|------|------|------|
| Phase 1 | 冗余清理 (design skill 5→3) | 18h | ✅ 完成 |
| Phase 2 | 搜索统一 (neotrix-search) | 16h | 待执行 |
| Phase 3 | 记忆拆分 (nt_memory_kb → focused crates) | 40h | 待执行 |
| Phase 4 | Gateway 统一 (3→1) | 20h | 待执行 |
| Phase 5 | Agent Loop 统一 | 16h | 待执行 |
| Phase 6 | Evolution 统一 | 24h | 待执行 |
| Phase 7 | Pipeline/Security/Health/Cache 统一 | 42h | 待执行 |
| Phase 8 | nt_design_visual 实现 | 52h | 待执行 |
| Phase 9 | 多Agent自动巡检修复 | 18h | 部分完成 |
| **总计** | | **246h** | |

---

## 六、量化统计

| 维度 | 发现 |
|------|------|
| 总实现数 | **60+** 散落实现 |
| 重复实现 | **30+** 重复 |
| 空文件 | **4** 个死代码文件 |
| 巨石模块 | **1** 个 (nt_memory_kb/ 88 files) |
| 跨层违规 | **2** 个 (L0 re-export from L6) |
| 需要新建的 crate | **3** (neotrix-search, neotrix-memory-*, neotrix-health) |
| 需要删除的重复 | **15+** 个模块 |
| 全量融合工时 | **246h** |

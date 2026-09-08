# Evolution Task List — Program Family Hierarchy

> 57 个能力星辰 → 10 个程序族 → 6 层架构。扁平 EVO 编号已废弃，统一为族-星辰-层级三级结构。

## 架构总览

```
L6 Meta-Cognition  ── [Meta] 族 + [Evolution] 族
L5 Cognition        ── [Reasoning] 族
L4 Emotion          ── (NT-FEEL 原生)
L3 Embodiment       ── [Physical] 族 + [Security] 族
L2 Perception       ── [Perception] 族
L1 Action           ── [Action] 族 + [Memory] 族 + [IO] 族 + [Infrastructure] 族
```

---

## Family 1: Reasoning (推理族) — NT-CORE

根星辰: ExpertStore 三层放置 (VRAM/RAM/NVMe)

```
Reasoning (推理族)
├── ★ F1-R01  ExpertStore 三层放置          ─── NT-CORE        [C4]
├── ★ F1-R02  PilotPrefetcher 生产接线       ─── NT-CORE        [C4]
├── ★ F1-R03  程序化 3D 内容管线             ─── NT-CORE        [C1]
├── ★ F1-R04  物理仿真引擎                   ─── NT-CORE        [C1]
├── ★ F1-R05  交互式物理可视化               ─── NT-CORE        [C1]
├── ★ F1-R06  程序化音频合成                 ─── NT-CORE        [C1]
├── ★ F1-R07  黏土材质管线                   ─── NT-CORE        [C1]
├── ★ F1-R08  EventBus 解耦事件总线          ─── NT-CORE        [C2]
└── ★ F1-R09  语义意图解析器                 ─── NT-CORE        [C1]
```

**族内依赖**: R01→R02 (ExpertStore 被 Pilot 消费), R03→R04→R05 (3D→物理→可视化), R03→R06→R07 (3D→音频→材质), R08 被所有星辰订阅

---

## Family 2: Perception (感知族) — NT-WORLD

根星辰: 浏览器自动化增强

```
Perception (感知族)
├── ★ F2-P01  浏览器自动化增强               ─── NT-WORLD       [C2]
└── ★ F2-P02  多模态输入路由                 ─── NT-WORLD       [C1]
```

**族内依赖**: P01→P02 (浏览器输出经多模态路由分发)

---

## Family 3: Memory (记忆族) — NT-MEMORY

根星辰: 语义搜索增强

```
Memory (记忆族)
├── ★ F3-M01  语义搜索增强 (BM25+Vector)     ─── NT-MEMORY      [C3]
├── ★ F3-M02  KB Schema 版本化               ─── NT-MEMORY      [C1]
├── ★ F3-M03  KB 语义 Diff                   ─── NT-MEMORY      [C1]
└── ★ F3-M04  知识图谱推理引擎               ─── NT-MEMORY      [C1]
```

**族内依赖**: M01→M02 (搜索依赖 Schema), M01→M03→M04 (搜索→Diff→推理)

---

## Family 4: Action (行动族) — NT-ACT

根星辰: 多 Agent DAG 编排器

```
Action (行动族)
├── ★ F4-A01  多 Agent DAG 编排器            ─── NT-ACT         [C3]
├── ★ F4-A02  JARVIS 多模态接口              ─── NT-ACT+FEEL    [C1]
├── ★ F4-A03  Task Board + Worktree          ─── NT-ACT         [C2]
├── ★ F4-A04  Agent 通信协议 ACP             ─── NT-ACT         [C2]
├── ★ F4-A05  Agent 注册 + 生命周期           ─── NT-ACT         [C2]
├── ★ F4-A06  State Snapshot 恢复            ─── NT-ACT         [C2]
└── ★ F4-A07  Workflow Snapshot 集成         ─── NT-ACT         [C2]
```

**族内依赖**: A01→A03→A07 (DAG→TaskBoard→WorkflowSnapshot), A01→A04→A05 (DAG→ACP→Registry), A06→A07 (Snapshot→WorkflowSnapshot)

---

## Family 5: IO (界面族) — NT-IO

根星辰: PDF 处理管线

```
IO (界面族)
├── ★ F5-I01  PDF 处理管线                   ─── NT-IO          [C3]
├── ★ F5-I02  Ollama 容器化部署              ─── NT-IO          [C2]
├── ★ F5-I03  语音交互 Agent                 ─── NT-IO          [C2]
├── ★ F5-I04  无限画布白板                   ─── NT-IO          [C2]
├── ★ F5-I05  Liquid Glass SDF 渲染          ─── NT-IO          [C1]
├── ★ F5-I06  Chat Channel 多端接入          ─── NT-IO          [C2]
├── ★ F5-I07  插件注册表动态加载             ─── NT-IO          [C2]
├── ★ F5-I08  配置管理器 + 热更新            ─── NT-IO          [C2]
└── ★ F5-I09  模块文档生成器                 ─── NT-IO          [C1]
```

**族内依赖**: I02→I03 (容器→语音), I07→I08 (插件→配置), I09 读取所有模块元数据

---

## Family 6: Security (安全族) — NT-SHIELD

根星辰: Agent 网络隔离沙箱

```
Security (安全族)
├── ★ F6-S01  Agent 网络隔离沙箱             ─── NT-SHIELD      [C3]
├── ★ F6-S02  对抗安全加固                   ─── NT-SHIELD      [C2]
├── ★ F6-S03  安全审计追踪                   ─── NT-SHIELD      [C2]
├── ★ F6-S04  自主保护 (CircuitBreaker)      ─── NT-SHIELD      [C2]
└── ★ F6-S05  自适应限流器                   ─── NT-SHIELD      [C2]
```

**族内依赖**: S01→S02 (沙箱→对抗), S01→S03 (沙箱→审计), S04→S05 (保护→限流)

---

## Family 7: Meta (元认知族) — NT-META

根星辰: ProbeRegistry 自审计模块

```
Meta (元认知族)
├── ★ F7-T01  ProbeRegistry 自审计           ─── NT-META        [C3]
├── ★ F7-T02  Skill 质量验证框架             ─── NT-META        [C3]
├── ★ F7-T03  Skill Eval Loop 验证           ─── NT-META        [C2]
├── ★ F7-T04  统一遥测 Telemetry             ─── NT-META        [C2]
├── ★ F7-T05  Telemetry-Probe 聚合桥         ─── NT-META        [C2]
├── ★ F7-T06  性能回归测试框架               ─── NT-META        [C2]
└── ★ F7-T07  测试编排框架                   ─── NT-META        [C2]
```

**族内依赖**: T01→T05→T04 (Probe→Bridge→Telemetry), T02→T03 (质量→EvalLoop), T06→T07 (性能→编排)

---

## Family 8: Evolution (进化族) — NT-MIND

根星辰: 技能域分类器 + C0-C6 成熟度

```
Evolution (进化族)
├── ★ F8-E01  技能域分类器 + C0-C6           ─── NT-MIND        [C3]
├── ★ F8-E02  进化搜索循环                   ─── NT-MIND        [C2]
└── ★ F8-E03  Agent 效能评分                 ─── NT-MIND        [C1]
```

**族内依赖**: E01→E03 (分类→评分), E02→E03 (搜索→评分)

---

## Family 9: Infrastructure (基础设施族) — NT-NEXUS / NT-GOVERNANCE

根星辰: 跨 Session Agent 聚合

```
Infrastructure (基础设施族)
├── ★ F9-INF01  跨 Session Agent 聚合        ─── NT-NEXUS       [C2]
├── ★ F9-INF02  实时健康仪表盘               ─── NT-NEXUS       [C2]
├── ★ F9-INF03  分布式追踪桥                 ─── NT-NEXUS       [C2]
├── ★ F9-INF04  跨域能量流监控               ─── NT-GOVERNANCE  [C2]
└── ★ F9-INF05  架构健康可视化               ─── NT-GOVERNANCE  [C2]
```

**族内依赖**: INF01→INF02 (聚合→仪表盘), INF03→INF02 (追踪→仪表盘), INF04→INF05 (能量→可视化)

---

## Family 10: Physical (具身族) — NT-PHYSICAL / NT-REPAIR

根星辰: 资源池管理器

```
Physical (具身族)
├── ★ F10-PH01  资源池管理器                 ─── NT-PHYSICAL    [C1]
└── ★ F10-PH02  错误恢复 Circuit Breaker     ─── NT-REPAIR      [C2]
```

**族内依赖**: PH02 保护 PH01 (CircuitBreaker 保护 ResourcePool)

---

## 跨族依赖图

```
                    ┌─────────────────────────────────────┐
                    │         L6 Meta-Cognition           │
                    │  [Meta] T01~T07  [Evolution] E01~E03 │
                    └──────────┬──────────┬───────────────┘
                               │          │
                    ┌──────────▼──────────▼───────────────┐
                    │         L5 Cognition                │
                    │  [Reasoning] R01~R09                │
                    └──┬────────┬────────┬───────────────┘
                       │        │        │
            ┌──────────▼──┐ ┌───▼────┐ ┌─▼──────────────┐
            │ L4 Emotion  │ │L3 Emb  │ │  L2 Perception │
            │  (NT-FEEL)  │ │[Phys]  │ │  [Perception]  │
            │             │ │PH01~02 │ │  P01~P02       │
            └─────────────┘ │[Secur] │ └───────┬────────┘
                            │S01~S05 │         │
                            └───┬────┘         │
                    ┌───────────▼──────────────▼──────────┐
                    │         L1 Action                   │
                    │  [Action] A01~A07                   │
                    │  [Memory] M01~M04                   │
                    │  [IO] I01~I09                       │
                    │  [Infra] INF01~INF05                │
                    └─────────────────────────────────────┘
```

### 关键跨族依赖

| 跨族依赖 | 说明 |
|----------|------|
| T01(Probe) → INF02(Dashboard) | Probe 数据喂给仪表盘 |
| T04(Telemetry) → INF03(Trace) | 遥测喂给追踪 |
| E01(Classifier) → A01(DAG) | 分类器指导 DAG 调度 |
| M01(HybridSearch) → E02(EvoSearch) | 搜索喂给进化搜索 |
| S01(Sandbox) → A04(ACP) | 沙箱保护 ACP 通信 |
| R08(EventBus) → 所有族 | 事件总线贯穿全架构 |
| I08(Config) → 所有族 | 配置管理贯穿全架构 |
| PH02(CircuitBreaker) → S04(SelfProtect) | 错误恢复→自主保护 |

---

## 熔炼任务 (EVO-58 ~ EVO-67)

> 扁平条目合并为族内统一模块，消除冗余。**状态: ✅ 全部执行完成。**

| ID | 任务 | 类型 | 冗余对 | 操作 |
|----|------|------|--------|------|
| **EVO-58** | CircuitBreaker + SelfProtection 熔合 | 族内合并 | F10-PH02 + F6-S04 | ✅ SelfProtection 合入 circuit_breaker.rs |
| **EVO-59** | EventBus + Bridge 统一入口 | 族内合并 | F1-R08 + F1-R08 bridge | ✅ EventRouter/DeadLetter 合入 event_bus.rs |
| **EVO-60** | Telemetry + ProbeBridge 统一 | 族内合并 | F7-T04 + F7-T05 | ✅ ProbeTelemetryBridge 合入 telemetry.rs |
| **EVO-61** | DomainClassifier + IntentParser 统一 | 族内合并 | F8-E01 + F1-R09 | ✅ IntentParser 合入 skill_domain_classifier.rs |
| **EVO-62** | ConfigManager + HotReload 统一 | 族内合并 | F5-I08 + F5-I08 hot | ✅ ConfigHotReload 合入 config_manager.rs |
| **EVO-63** | SnapshotStore + WorkflowSnapshot 统一 | 族内合并 | F4-A06 + F4-A07 | ✅ WorkflowSnapshot 合入 state_snapshot.rs |
| **EVO-64** | Sandbox + AuditTrail + Protection 统一安全层 | 族内合并 | F6-S01 + F6-S03 + F6-S04 | ✅ AuditTrail 合入 agent_sandbox.rs |
| **EVO-65** | DAG + TaskBoard + AgentRegistry 统一编排层 | 族内合并 | F4-A01 + F4-A03 + F4-A05 | ✅ TaskBoard + Registry 合入 nt_act_multi_agent_dag.rs |
| **EVO-66** | HybridSearch + SemanticDiff + GraphReasoner 知识三角统一 | 族内合并 | F3-M01 + F3-M03 + F3-M04 | ✅ SemanticDiff + GraphReasoner 合入 hybrid_search.rs |
| **EVO-67** | ModuleInterface trait 统一接口 | 架构缺口 | 全局 | ✅ 创建 module_interface.rs + DefaultModuleInterface |

---

## 审计修复任务 (EVO-68 ~ EVO-77)

> 基于全域架构审计发现的冗余+扁平+错位缺陷，执行精准修复

| ID | 任务 | 审计维度 | 状态 | 操作 |
|----|------|----------|------|------|
| **EVO-68** | Severity 统一适配器 | D5-Roam 重复类型 | ✅ Done | `neotrix-types::shared::Severity` 已创建，替代 22 个本地定义 |
| **EVO-69** | NtDomain 统一适配器 | D5-Roam 域枚举分裂 | ✅ Done | `neotrix-types::shared::NtDomain` 已创建，替代 4 个不一致定义 |
| **EVO-70** | CircuitBreaker 统一适配器 | D5-Roam 7 个重复 | ✅ Done | `neotrix-types::shared::CircuitBreaker` 已创建 |
| **EVO-71** | RateLimiter 统一适配器 | D5-Roam 6 个重复 | ✅ Done | `neotrix-types::shared::RateLimiter` 已创建 |
| **EVO-72** | nt_io_* 跨域迁移 | D3-Layers 错位 | ✅ Done | 12 个 nt_io_* 从 nt_core → nt_io |
| **EVO-73** | shield_audit/approval 迁移 | D3-Layers 错位 | ✅ Done | nt_shield_audit + nt_shield_approval 从 nt_meta → nt_shield |
| **EVO-74** | nt_mind_repair 迁移 | D3-Layers 错位 | ✅ Done | nt_mind_repair 从 nt_meta → nt_repair |
| **EVO-75** | Dead mod 清理脚本 | D2-Dead 死声明 | ⏳ 待执行 | 扫描所有 mod.rs，删除声明但文件不存在的条目 |
| **EVO-76** | unified/ 去重决策 | D5-Roam 双树 | ⏳ 待执行 | 197 个分化文件：选定 unified/ 为事实源，生成迁移脚本 |
| **EVO-77** | 扁平文件重组计划 | D5-Roam 扁平缺陷 | ✅ 执行中 | nt_core reasoning/ 子目录已建, nt_act workflow/ 子目录已建 |

### EVO-77 扁平文件重组方案

> 统一树共 890 个扁平 .rs 文件。按命名模式分组为子目录，减少认知负载。
> 执行策略：每个域独立 PR，先清理重复文件，再建子目录，最后更新 mod.rs re-export。

#### 重组前清理 (nt_act 重复文件)

nt_act 有 16 个扁平副本（子目录已存在），直接删除:

```
删除 16 个扁平副本 (保留子目录版本):
  ✗ nt_act/airdrop.rs          (→ nt_act_crypto/)
  ✗ nt_act/bridge.rs           (→ nt_act_crypto/)
  ✗ nt_act/chain.rs            (→ nt_act_crypto/)
  ✗ nt_act/cipher.rs           (→ nt_act_crypto/)
  ✗ nt_act/collector.rs        (→ nt_act_crypto/)
  ✗ nt_act/gas.rs              (→ nt_act_crypto/)
  ✗ nt_act/portfolio.rs        (→ nt_act_crypto/)
  ✗ nt_act/self_evolve.rs      (→ nt_act_crypto/)
  ✗ nt_act/token.rs            (→ nt_act_crypto/)
  ✗ nt_act/wallet.rs           (→ nt_act_crypto/)
  ✗ nt_act/wallet_store.rs     (→ nt_act_crypto/)
  ✗ nt_act/yields.rs           (→ nt_act_crypto/)
  ✗ nt_act/command.rs          (→ nt_act_voice/)
  ✗ nt_act/monitor.rs          (→ nt_act_crypto/)
  ✗ nt_act/trigger.rs          (→ nt_act_voice/)
  ✗ nt_act/transcribe.rs       (→ nt_act_voice/)
```

#### 域重组方案 (按影响排序)

**nt_core (57 files → 7 subdirs)**

| 子目录 | 文件 | 说明 |
|--------|------|------|
| `reasoning/` | physics_engine, physics_viz, procedural_3d, audio_synthesis, clay_material, rhythm_recalculator, transition_gradient, blank_space_checker | 推理+动态漫节奏 |
| `knowledge/` | nt_core_kg_traversal, nt_core_knowledge_mgmt, nt_core_knowledge_repr, nt_core_rag, prompt_cache, prompt_enhancer, video_prompt_cache | 知识管理+提示 |
| `safety/` | nt_core_safety_alignment, nt_core_xai, contract, isolation | 安全+可解释 |
| `visual/` | face_consistency, visual_consistency, style_harmonizer, narrative_structuring, storyboard_extractor | 视觉一致性+叙事 |
| `reasoning/e8/` | nt_core_e8, nt_core_e8_predictor, nt_core_e8_vsa, nt_core_field_resonance, nt_core_golden_ratio, nt_core_integrated_information, nt_core_resonance_complexity | E8+场共振+整合信息 |
| `agent/` | nt_core_autonomous_ai, nt_core_planning, nt_core_model_router, coordinator, executor, diversity, hands | 自主AI+规划 |
| `types.rs` | types.rs | 核心类型 |

**nt_act (清理后 ~98 files → 8 subdirs)**

| 子目录 | 文件 | 说明 |
|--------|------|------|
| `crypto/` | (已有子目录) | DeFi+链上 |
| `voice/` | (已有子目录) | 语音+转写 |
| `workflow/` | task_board, task_state_dag, state_snapshot, state_graph, workflow_snapshot, edit_history, safe_applier | 工作流+状态 |
| `agent/` | agent_registry, per_agent, conflict_resolver, behavioral_verifier, awareness_monitor | Agent管理 |
| `orchestrator/` | production_orchestrator, parallel_task, deterministic_scheduler, pipeline_autofixer, resource_budget | 编排+调度 |
| `security/` | nt_act_security, nt_act_sandbox, nt_act_circuit_breaker, nt_act_disk_guard, error_classifier | 安全+沙箱 |
| `knowledge/` | knowledge_distiller, pattern_extractor, semantic_entropy, coverage_analyzer, test_writer | 知识+测试 |
| `code/` | code_writer, ast_searcher, recipe_refactor, yagni_ladder, harness_scaffold | 代码生成 |

**nt_memory (102 files → 6 subdirs)**

| 子目录 | 文件 | 说明 |
|--------|------|------|
| `kb/` | bm25, kb_cognition, kb_vector_index, knowledge_storage, cache | KB核心 |
| `evidence/` | nt_evidence_* (7 files) | 证据链 |
| `discovery/` | nt_discovery_* (3 files) | 知识发现 |
| `absorption/` | nt_absorb_mapper, nt_memory_adaptive_rag, nt_memory_agent_driven | 吸收+RAG |
| `session/` | nt_memory_agent_session, nt_memory_commit_tracker, nt_memory_coeffect, nt_memory_blocks | 会话+版本 |
| `embed/` | (向量嵌入相关) | 嵌入 |

**nt_world (115 files → 5 subdirs)**

| 子目录 | 文件 | 说明 |
|--------|------|------|
| `media/` | (已有子目录) | 媒体处理 |
| `crawler/` | (已有子目录) | 爬虫 |
| `perception/` | (已有子目录) | 感知 |
| `asset/` | asset_registry, media_asset_registry, template_registry | 资产管理 |
| `search/` | hybrid_search (→ nt_memory), ordered_backend_router | 搜索路由 |

**nt_io (118 files → 6 subdirs)**

| 子目录 | 文件 | 说明 |
|--------|------|------|
| `platform/` | platform_gateway, chat_channel, pdf_pipeline | 平台集成 |
| `generation/` | (视频/图片生成) | 生成能力 |
| `llm/` | (LLM provider相关) | LLM接口 |
| `skill/` | nt_io_*_skills (12 files, 已迁入) | 技能吸收 |
| `tools/` | plugin_registry, config_manager, doc_generator | 工具管理 |
| `media/` | infinite_canvas, liquid_glass | 媒体UI |

**nt_shield (92 files → 4 subdirs)**

| 子目录 | 文件 | 说明 |
|--------|------|------|
| `stealth/` | nt_shield_stealth_net (已有) | 隐身网络 |
| `proxy/` | nt_shield_proxy_kernel (已有) | 代理 |
| `audit/` | audit_trail, nt_shield_audit, adversarial_hardening | 审计+对抗 |
| `policy/` | nt_shield_approval, self_protection, adaptive_rate_limiter | 策略+限流 |

#### 重组依赖图

```
EVO-77a: nt_act 重复清理 (16 files) ──→ EVO-77b: nt_act 子目录化
                                          │
EVO-77c: nt_core 子目录化 ────────────────┤
                                          │
EVO-77d: nt_memory 子目录化 ──────────────┤
                                          │
EVO-77e: nt_io 子目录化 ─────────────────┤
                                          │
EVO-77f: nt_world 子目录化 ──────────────┤
                                          │
EVO-77g: nt_shield 子目录化 ─────────────┘
```

#### 执行规则

1. 每次只重组一个域，确保 `cargo check -p neotrix` 通过
2. 子目录化时，mod.rs 保留 `pub use submod::*;` re-export 保持 API 兼容
3. 重复文件先删除再重组，避免重复清理
4. 每个域重组后更新 `evolution-task-list.md` 实现记录

### 审计修复依赖图

```
EVO-68~71 (类型统一) ──→ EVO-75 (Dead mod) ──→ EVO-76 (去重)
                              │
EVO-72~74 (错位迁移) ────────┘
                              │
                        EVO-77 (重组计划)
```

### 审计发现摘要

| 类别 | 发现数 | 已修复 | 待修复 |
|------|--------|--------|--------|
| 重复类型 (Severity/NtDomain/CB/RL) | 41 个定义 | 4 (统一适配器) | 37 (逐模块替换) |
| 跨域错位 | 17+ 模块 | 15 (迁移完成) | 2 (nt_trade_* 太复杂) |
| 扁平缺陷 | 179 文件 | 0 | 179 (需重组计划) |
| 双树分化 | 197 文件 | 0 | 197 (需决策) |
| 死 mod 声明 | ~30 个 | 0 | ~30 (需清理脚本) |

### 熔炼后文件变更

```
删除 9 个冗余文件:
  ✗ nt_shield/self_protection.rs        (→ circuit_breaker.rs)
  ✗ nt_core/event_bus_bridge.rs          (→ event_bus.rs)
  ✗ nt_meta/probe_telemetry_bridge.rs    (→ telemetry.rs)
  ✗ nt_core/intent_parser.rs             (→ skill_domain_classifier.rs)
  ✗ nt_io/config_hot_reload.rs           (→ config_manager.rs)
  ✗ nt_act/workflow_snapshot.rs          (→ state_snapshot.rs)
  ✗ nt_shield/audit_trail.rs             (→ agent_sandbox.rs)
  ✗ nt_act/task_board.rs                 (→ nt_act_multi_agent_dag.rs)
  ✗ nt_act/agent_registry.rs             (→ nt_act_multi_agent_dag.rs)
  ✗ nt_memory/semantic_diff.rs           (→ hybrid_search.rs)
  ✗ nt_memory/graph_reasoner.rs          (→ hybrid_search.rs)

新增 1 个:
  ✓ nt_core/module_interface.rs          (ModuleInterface trait)

净减 10 个文件，消除 12 处冗余
```

---

## 实现记录 (57 星辰)

| 族 | 星辰 | 域 | 文件 | 行数 | 关键类型 |
|----|------|-----|------|------|----------|
| Reasoning | F1-R01 | NT-CORE | `expert_store.rs` | +2KB | PlacementTier::Nvme, 3-tier |
| Reasoning | F1-R02 | NT-CORE | `pilot_prefetch.rs` | — | PILOT route history |
| Reasoning | F1-R03 | NT-CORE | `procedural_3d.rs` | 392 | MeshData, Perlin, 6 primitives |
| Reasoning | F1-R04 | NT-CORE | `physics_engine.rs` | 450 | PhysicsWorld, RigidBody, Joint |
| Reasoning | F1-R05 | NT-CORE | `physics_viz.rs` | ~350 | Verlet, spring, fluid, SVG |
| Reasoning | F1-R06 | NT-CORE | `audio_synthesis.rs` | 1400 | Oscillator, BiquadFilter, ADSR |
| Reasoning | F1-R07 | NT-CORE | `clay_material.rs` | 726 | ClayMaterial, fractal bumpmap |
| Reasoning | F1-R08 | NT-CORE | `event_bus.rs` | ~500 | EventBus, glob matching |
| Reasoning | F1-R09 | NT-CORE | `intent_parser.rs` | 520 | IntentParser, EntityExtractor |
| Perception | F2-P01 | NT-WORLD | `browser_automation.rs` | ~400 | Workflow, VisualUnderstanding |
| Perception | F2-P02 | NT-WORLD | `multimodal_router.rs` | 530 | MultimodalRouter, Modality |
| Memory | F3-M01 | NT-MEMORY | `hybrid_search.rs` | 450 | BM25+Vector, RRF fusion |
| Memory | F3-M02 | NT-MEMORY | `schema_migration.rs` | ~510 | SchemaRegistry, Migration |
| Memory | F3-M03 | NT-MEMORY | `semantic_diff.rs` | 888 | SemanticDiff, ConceptDelta |
| Memory | F3-M04 | NT-MEMORY | `graph_reasoner.rs` | ~600 | GraphReasoner, InferenceRule |
| Action | F4-A01 | NT-ACT | `nt_act_multi_agent_dag.rs` | 15KB | WorkflowDag, AgentPool |
| Action | F4-A02 | NT-ACT | `jarvis_interface.rs` | 432 | MultimodalInput, EmotionEngine |
| Action | F4-A03 | NT-ACT | `task_board.rs` | 470 | TaskBoard, 8-state FSM |
| Action | F4-A04 | NT-ACT | `acp_protocol.rs` | 761 | AcpProtocol, handshake |
| Action | F4-A05 | NT-ACT | `agent_registry.rs` | ~500 | AgentRegistry, LoadBalancer |
| Action | F4-A06 | NT-ACT | `state_snapshot.rs` | ~733 | SnapshotStore, undo/redo |
| Action | F4-A07 | NT-ACT | `workflow_snapshot.rs` | 531 | WorkflowSnapshotManager |
| IO | F5-I01 | NT-IO | `pdf_pipeline.rs` | 938 | PdfPipeline |
| IO | F5-I02 | NT-IO | `ollama_container.rs` | 333 | Container lifecycle |
| IO | F5-I03 | NT-IO | `voice_agent.rs` | 585 | Multi-turn VAD |
| IO | F5-I04 | NT-IO | `infinite_canvas.rs` | 763 | 6 element types, SVG |
| IO | F5-I05 | NT-IO | `liquid_glass.rs` | 470 | SDF, Snell refraction |
| IO | F5-I06 | NT-IO | `chat_channel.rs` | 944 | 5 platforms, rate limiter |
| IO | F5-I07 | NT-IO | `plugin_registry.rs` | ~580 | PluginTrait, sandbox |
| IO | F5-I08 | NT-IO | `config_manager.rs` | ~470 | ConfigManager, ConfigSchema |
| IO | F5-I09 | NT-IO | `doc_generator.rs` | 450 | DocGenerator, ModuleDoc |
| Security | F6-S01 | NT-SHIELD | `agent_sandbox.rs` | 12KB | 3 isolation levels |
| Security | F6-S02 | NT-SHIELD | `adversarial_hardening.rs` | 821 | RedTeam, 14 patterns |
| Security | F6-S03 | NT-SHIELD | `audit_trail.rs` | ~530 | HashChain, IntegrityResult |
| Security | F6-S04 | NT-SHIELD | `self_protection.rs` | 788 | CascadeDetector |
| Security | F6-S05 | NT-SHIELD | `adaptive_rate_limiter.rs` | 537 | TokenBucket, AIMD |
| Meta | F7-T01 | NT-META | `probe_registry.rs` | 12KB | AuditProbe trait |
| Meta | F7-T02 | NT-META | `skill_quality.rs` | 466 | 5 checks, security audit |
| Meta | F7-T03 | NT-META | `skill_eval_loop.rs` | 878 | EvalLoop, DetCheck |
| Meta | F7-T04 | NT-META | `telemetry.rs` | 559 | TelemetryCollector, Histogram |
| Meta | F7-T05 | NT-META | `probe_telemetry_bridge.rs` | 467 | ProbeTelemetryBridge |
| Meta | F7-T06 | NT-META | `perf_regression.rs` | ~540 | PerfBenchmark, t-test |
| Meta | F7-T07 | NT-META | `test_orchestrator.rs` | 506 | TestOrchestrator, MockAgent |
| Evolution | F8-E01 | NT-MIND | `skill_domain_classifier.rs` | 19KB | 13 domains, C0-C6 |
| Evolution | F8-E02 | NT-MIND | `evolution_search.rs` | 594 | Organism, MutationStrategy |
| Evolution | F8-E03 | NT-MIND | `agent_scorer.rs` | 618 | ScoreDecay, Leaderboard |
| Infrastructure | F9-INF01 | NT-NEXUS | `cross_session_aggregator.rs` | 530 | 10 agent types |
| Infrastructure | F9-INF02 | NT-NEXUS | `health_dashboard.rs` | 659 | AlertRule, TrendDetector |
| Infrastructure | F9-INF03 | NT-NEXUS | `trace_bridge.rs` | 472 | TracePropagator, Baggage |
| Infrastructure | F9-INF04 | NT-GOVERNANCE | `energy_flow.rs` | 656 | NtDomain, FlowViolation |
| Infrastructure | F9-INF05 | NT-GOVERNANCE | `arch_visualizer.rs` | ~500 | HealthBar, ASCII diagram |
| Physical | F10-PH01 | NT-PHYSICAL | `resource_pool.rs` | ~500 | GpuAllocator, MemoryBudget |
| Physical | F10-PH02 | NT-REPAIR | `circuit_breaker.rs` | ~500 | ErrorRecovery, FallbackRegistry |

## 吸收来源追踪

| 来源 | 族 | 星辰 | KB Entry |
|------|----|------|----------|
| Colibrì ExpertStore | Reasoning | F1-R01, F1-R02 | branch_10_0_51a409 |
| Clay Safari | Reasoning | F1-R03, F1-R06, F1-R07 | branch_10_0_51a409 |
| MuJoCo | Reasoning | F1-R04, F1-R05 | branch_10_0_51a409 |
| Skyvern | Perception | F2-P01 | branch_10_0_51a409 |
| FineWeb-10B | Memory | F3-M01 | branch_10_0_51a409 |
| GenOffice/SuperHermes | Action | F4-A01 | branch_10_0_51a409 |
| pub-local-jarvis | Action | F4-A02 | branch_10_0_51a409 |
| Codeg | Action+IO | F4-A03~A07, F5-I06 | cycle_11 |
| pdf2pdf | IO | F5-I01 | branch_10_0_51a409 |
| ollama-container | IO | F5-I02 | branch_10_0_51a409 |
| Tel-Agent | IO | F5-I03 | branch_10_0_51a409 |
| Excalidraw | IO | F5-I04 | branch_10_0_51a409 |
| Hyalite | IO | F5-I05 | cycle_11 |
| ripwire | Security | F6-S01 | branch_10_0_51a409 |
| exploitarium | Security | F6-S02 | branch_10_0_51a409 |
| SysData | Meta | F7-T01 | branch_10_0_51a409 |
| SkillSpector | Meta | F7-T02 | branch_10_0_51a409 |
| Vercel design.md | Meta | F7-T03 | cycle_11 |
| autoresearch | Evolution | F8-E02 | branch_10_0_51a409 |
| Skill repos (5个) | Evolution | F8-E01 | branch_10_0_51a409 |
| ciechanow.ski | Reasoning | F1-R05 | branch_10_0_51a409 |

---

## 深度分析 — 冗余消除实施方案

### 一、CircuitBreaker 统一 (9→1)

#### 1.1 实现矩阵

| 位置 | 结构体 | 核心特性 | 使用域 |
|------|--------|----------|--------|
| `core/nt_core_observer_error.rs` | `CircuitBreaker` | 状态机 (Closed/Open/HalfOpen), failure_threshold, half_open_timeout | NT-CORE 观察者 |
| `core/nt_core_error_recovery.rs` | `CircuitBreakerStrategy` | HashMap 按 key 隔离, threshold + cooldown | NT-CORE 错误恢复 |
| `l1_action/nt_act/nt_act_circuit_breaker.rs` | `CircuitBreaker` | **最完整**: Config/Stats/Manager, FallbackStrategy, 半开探针限制 | NT-ACT 熔断 |
| `l1_action/nt_infra_breaker.rs` | `CircuitBreaker` | BreakerState + BreakerConfig, 滑动窗口 VecDeque | NT-ACT 基础设施 |
| `l1_action/nt_io/nt_io_provider/circuit_breaker.rs` | `CircuitBreaker` | 滑动窗口, half_open_max_probes | NT-IO provider |
| `l5_cognition/nt_mind/evolution/goal_loop/types.rs` | `CircuitBreaker` | 额外 stall_count 跟踪 | NT-MIND 进化 |
| `l5_cognition/nt_mind/evolution/value_gate.rs` | `CircuitBreaker` | 内部使用 | NT-MIND 门控 |
| `neotrix/nt_act/client.rs` | `CircuitBreakerState` | 客户端封装 | NT-ACT 客户端 |
| `neotrix_types::shared` | `CircuitBreaker` | **统一基础**: state, thresholds, timeout, counters | 全局 |

#### 1.2 统一方案

**策略**: `neotrix_types::shared::CircuitBreaker` 作为基础类型，各域通过 extension trait 添加特化能力。

**Step 1**: 扩展 `neotrix_types::shared::CircuitBreaker`

```rust
// 新增字段 (向后兼容)
pub struct CircuitBreaker {
    // 现有字段...
    pub name: String,
    pub state: BreakerState,
    pub failure_threshold: usize,
    pub success_threshold: usize,
    pub timeout_ms: u64,
    pub consecutive_failures: usize,
    pub consecutive_successes: usize,
    pub last_failure_time: u64,
    pub total_calls: u64,
    pub total_failures: u64,
    // 新增: 滑动窗口 + stall 跟踪
    pub sliding_window: VecDeque<bool>,      // nt_io_provider 特性
    pub window_size: usize,                   // nt_io_provider 特性
    pub stall_count: usize,                   // goal_loop 特性
    pub max_stalls: usize,                    // goal_loop 特性
    pub half_open_max_probes: u32,            // nt_io_provider 特性
    pub half_open_probes_used: u32,           // nt_io_provider 特性
}
```

**Step 2**: 各域替换

| 域 | 文件 | 操作 |
|----|------|------|
| NT-CORE | `nt_core_observer_error.rs` | `use neotrix_types::shared::{CircuitBreaker, BreakerState};` 删除本地定义 |
| NT-CORE | `nt_core_error_recovery.rs` | `CircuitBreakerStrategy` → 重命名为 `ErrorRecoveryStrategy` (不合并，语义不同) |
| NT-ACT | `nt_act_circuit_breaker.rs` | 删除本地 CircuitBreaker，使用 shared + 添加 `CircuitBreakerManager` 扩展 |
| NT-ACT | `nt_infra_breaker.rs` | 删除本地 CircuitBreaker，使用 shared |
| NT-IO | `nt_io_provider/circuit_breaker.rs` | 删除本地 CircuitBreaker，使用 shared |
| NT-MIND | `goal_loop/types.rs` | 删除本地 CircuitBreaker，使用 shared |
| NT-MIND | `value_gate.rs` | 删除本地 CircuitBreaker，使用 shared |
| NT-ACT | `client.rs` | `CircuitBreakerState` → 使用 shared::BreakerState |

**Step 3**: 验证

```sh
cargo check -p neotrix
cargo test -p neotrix --lib -- circuit_breaker
```

#### 1.3 风险

- **字段类型差异**: 本地用 `u32`/`Instant`，shared 用 `usize`/`u64`。需统一为 `usize`/`u64`
- **方法签名差异**: 各实现的 `new()` 参数不同。需保留各域的构造函数作为 extension
- **FallbackStrategy**: 仅 `nt_act_circuit_breaker` 有，需提升到 shared

---

### 二、RateLimiter 统一 (5→1)

#### 2.1 实现矩阵

| 位置 | 结构体 | 核心特性 | 使用域 |
|------|--------|----------|--------|
| `l1_action/nt_act/nt_act_rate_limiter.rs` | `RateLimiter` | **最完整**: HashMap<String, TokenBucket>, Config/Stats, 自适应 | NT-ACT 限流 |
| `l1_action/nt_io/nt_io_provider/rate_limiter.rs` | `RateLimiter` | RPM + TPM 双桶, max_retries | NT-IO provider |
| `l5_cognition/nt_mind/evolution/goal_loop/types.rs` | `RateLimiter` | 简单滑动窗口, max_calls_per_hour | NT-MIND 进化 |
| `l3_embodiment/nt_shield/adaptive_rate_limiter.rs` | 无 RateLimiter struct (使用 shared) | AIMD 算法 | NT-SHIELD |
| `neotrix_types::shared` | `RateLimiter` | Token bucket 基础 | 全局 |

#### 2.2 统一方案

**策略**: `neotrix_types::shared::RateLimiter` 作为基础 token bucket，各域通过 wrapper 添加特化。

**Step 1**: 扩展 shared RateLimiter

```rust
pub struct RateLimiter {
    pub name: String,
    pub max_rate_per_sec: usize,
    pub burst_size: usize,
    pub window_ms: u64,
    pub tokens: f64,
    pub last_refill_time: u64,
    // 新增: RPM/TPM 双桶支持
    pub rpm_bucket: Option<TokenBucket>,
    pub tpm_bucket: Option<TokenBucket>,
    // 新增: 滑动窗口支持
    pub call_timestamps: VecDeque<u64>,
    pub max_calls_per_window: Option<usize>,
}
```

**Step 2**: 各域替换

| 域 | 文件 | 操作 |
|----|------|------|
| NT-ACT | `nt_act_rate_limiter.rs` | 删除本地 RateLimiter，使用 shared + `RateLimiterManager` 扩展 |
| NT-IO | `nt_io_provider/rate_limiter.rs` | 删除本地 RateLimiter，使用 shared (rpm_bucket + tpm_bucket) |
| NT-MIND | `goal_loop/types.rs` | 删除本地 RateLimiter，使用 shared (call_timestamps) |

---

### 三、CacheEntry 统一 (7→1)

#### 3.1 实现矩阵

| 位置 | 结构体 | 核心特性 | 使用域 |
|------|--------|----------|--------|
| `core/nt_core_cache.rs` | `CacheEntry` | value: String, inserted_at | NT-CORE 缓存 |
| `core/nt_core_context/ccr.rs` | `CacheEntry` | original, fingerprint, access_count | NT-CORE 压缩 |
| `core/nt_core_deploy_cache.rs` | `CacheEntry` | program_bytes, compiled_target, LRU | NT-CORE 部署 |
| `l1_action/nt_act/nt_act_cache.rs` | `CacheEntry` | key, value: JSON, ttl, tags | NT-ACT 缓存 |
| `l1_action/nt_memory/nt_memory_kb/nt_memory_sweep_20260815.rs` | `CacheEntry` | key, value: Vec<f64>, access_count | NT-MEMORY 向量 |
| `l2_perception/nt_world/nt_world_cleanup/cache_detector.rs` | `CacheEntry` | name, cache_type, path_resolver | NT-WORLD 检测 |
| `l5_cognition/nt_core/prompt_cache.rs` | `CacheEntry` | hash, prompt, response | NT-CORE 提示 |
| `neotrix_types::shared` | `CacheEntry<V>` | **泛型**: key, value, created_at, expires_at, access_count | 全局 |

#### 3.2 统一方案

**策略**: `neotrix_types::shared::CacheEntry<V>` 已是泛型，可覆盖大部分场景。特殊实现保留为 domain-specific wrapper。

**Step 1**: 各域替换

| 域 | 文件 | 操作 |
|----|------|------|
| NT-CORE | `nt_core_cache.rs` | `CacheEntry { value: String }` → `CacheEntry<String>` |
| NT-CORE | `nt_core_context/ccr.rs` | 保留 (fingerprint 特殊，不合并) |
| NT-CORE | `nt_core_deploy_cache.rs` | 保留 (program_bytes 特殊，不合并) |
| NT-ACT | `nt_act_cache.rs` | `CacheEntry { value: JSON }` → `CacheEntry<serde_json::Value>` |
| NT-MEMORY | `nt_memory_sweep` | `CacheEntry { value: Vec<f64> }` → `CacheEntry<Vec<f64>>` |
| NT-WORLD | `cache_detector.rs` | 保留 (CacheType 特殊，不合并) |
| NT-CORE | `prompt_cache.rs` | 保留 (prompt+response 特殊，不合并) |

**结论**: 仅 3 个可直接替换，4 个保留为 domain-specific。

---

### 四、同名文件去重方案

#### 4.1 高优先级 (真正重复)

| 文件名 | 副本数 | 操作 |
|--------|--------|------|
| `nt_io_*.rs` (12 对) | 2 | ✅ 已完成: 删除 nt_core 副本 |
| `nt_trade_*.rs` (6 对) | 2 | ✅ 已完成: 移动到 nt_act |
| `arch_visualizer.rs` | 2 | 删除 `nt_meta/nt_governance/` 副本 (保留 `nt_governance/`) |
| `energy_flow.rs` | 2 | 删除 `nt_meta/nt_governance/` 副本 (保留 `nt_governance/`) |
| `hybrid_search.rs` | 2 | 合并 `nt_memory` (meta) → `nt_memory` (action) |

#### 4.2 中优先级 (语义不同，名称相同)

| 文件名 | 副本数 | 分析 | 操作 |
|--------|--------|------|------|
| `tests.rs` | 20 | 各模块独立测试，不合并 | 保留 |
| `types.rs` | 28 | 各模块独立类型定义 | 保留 |
| `engine.rs` | 5 | 各域引擎实现不同 | 保留 |
| `config.rs` | 4 | 各域配置不同 | 保留 |
| `registry.rs` | 5 | 各域注册表不同 | 保留 |
| `monitor.rs` | 4 | 各域监控不同 | 保留 |

#### 4.3 低优先级 (可能重复)

| 文件名 | 副本数 | 操作 |
|--------|--------|------|
| `bm25.rs` | 2 | 合并 `nt_mind/infrastructure/tests/bm25.rs` → `nt_memory_kb/bm25.rs` |
| `consciousness_bridge.rs` | 2 | 合并 `l7_capability/` → `nt_mind/consciousness/` |
| `context_budget.rs` | 2 | 合并 `nt_core_context/` → `nt_io_provider/` |
| `self_model.rs` | 2 | 合并 `nt_core_meta/` → `nt_core_self/` |

---

### 五、执行计划

#### Phase 1: CircuitBreaker 统一 (预计 2h)

```
1. 扩展 neotrix_types::shared::CircuitBreaker 字段 (+5 fields)
2. nt_core_observer_error.rs — 替换为 shared import
3. nt_act_circuit_breaker.rs — 替换为 shared + 保留 Manager
4. nt_infra_breaker.rs — 替换为 shared
5. nt_io_provider/circuit_breaker.rs — 替换为 shared
6. goal_loop/types.rs — 替换为 shared
7. value_gate.rs — 替换为 shared
8. client.rs — CircuitBreakerState → BreakerState
9. cargo check + cargo test
```

#### Phase 2: RateLimiter 统一 (预计 1h)

```
1. 扩展 neotrix_types::shared::RateLimiter 字段 (+3 fields)
2. nt_act_rate_limiter.rs — 替换为 shared + 保留 Manager
3. nt_io_provider/rate_limiter.rs — 替换为 shared
4. goal_loop/types.rs — 替换为 shared
5. cargo check + cargo test
```

#### Phase 3: CacheEntry 统一 (预计 30min)

```
1. nt_core_cache.rs — CacheEntry<String>
2. nt_act_cache.rs — CacheEntry<serde_json::Value>
3. nt_memory_sweep — CacheEntry<Vec<f64>>
4. cargo check + cargo test
```

#### Phase 4: 同名文件去重 (预计 30min)

```
1. 删除 arch_visualizer.rs / energy_flow.rs 重复副本
2. 合并 hybrid_search.rs
3. 合并 bm25.rs / consciousness_bridge.rs / context_budget.rs / self_model.rs
4. cargo check + cargo test
```

#### Phase 5: 验证 (预计 30min)

```
1. cargo check -p neotrix — 全量编译
2. cargo test -p neotrix --lib — 单元测试
3. 重新审计: 检查冗余计数
```

---

## 风险分析 — 完整评估

### 一、CircuitBreaker 统一风险矩阵

| 风险 | 严重度 | 影响范围 | 缓解策略 |
|------|--------|----------|----------|
| **R1: State 枚举命名冲突** | HIGH | 6 个文件定义了 CircuitState/BreakerState | 统一为 `BreakerState`，所有引用点批量替换 |
| **R2: Open 变体数据差异** | HIGH | `nt_core_observer_error` 的 `Open { since: usize }` vs 其他 `Open` | shared 使用 `Open`，`since` 字段下沉到 struct 字段 |
| **R3: 字段类型不一致** | MEDIUM | `u32` vs `u64` vs `usize` 混用 | shared 统一为 `usize`，各域构造时转换 |
| **R4: 方法签名不兼容** | HIGH | `allow_request()` vs `allow()` vs `on_success()` vs `record_success()` | shared 提供核心方法，各域通过 extension trait 添加特化 |
| **R5: CircuitBreakerStrategy 语义不同** | LOW | 仅 1 个文件 | 不合并，重命名为 `ErrorRecoveryStrategy` 避免混淆 |
| **R6: BreakerRegistry 全局状态** | MEDIUM | `nt_infra_breaker` 有 `static` 全局注册表 | 保留 BreakerRegistry 作为 wrapper，内部使用 shared |
| **R7: Client 私有 CircuitState** | LOW | `neotrix/nt_act/client.rs` 是私有类型 | 替换为 `use neotrix_types::shared::BreakerState` |
| **R8: 编译级联错误** | HIGH | 修改类型定义可能引发大量级联错误 | 分文件逐步替换，每步 cargo check |

#### R1 详细分析: State 枚举命名

```
现状:
  CircuitState (3处): nt_core_observer_error, nt_act_circuit_breaker, goal_loop/types
  BreakerState (3处): nt_infra_breaker, nt_io_provider/circuit_breaker, neotrix_types::shared

方案:
  统一为 BreakerState (shared 已定义)
  所有 CircuitState 引用 → BreakerState
  影响文件: 3 个 (nt_core_observer_error, nt_act_circuit_breaker, goal_loop/types)
```

#### R2 详细分析: Open 变体

```
现状:
  nt_core_observer_error: Open { since: usize } — 记录熔断时间
  其他: Open — 纯状态

方案:
  shared BreakerState: Open (无数据)
  nt_core_observer_error 需要 since → 在 struct 中添加 last_opened_at: Option<u64> 字段
  影响: 仅 nt_core_observer_error 需额外适配
```

#### R4 详细分析: 方法签名

```
核心方法 (shared 必须提供):
  allow_request() -> bool          (nt_core_observer_error, nt_act)
  record_success()                 (nt_core_observer_error, goal_loop)
  record_failure() -> bool         (goal_loop 返回是否 terminal)
  reset()                          (nt_act, nt_io_provider)

特化方法 (各域 extension trait):
  nt_act: call<T,F,E>() — 带 fallback 的泛型调用
  nt_io_provider: health_penalty(), force_open(), cooldown_elapsed()
  goal_loop: record_stall(), is_terminal()
  nt_infra_breaker: error_rate(), allow(capability_id)

方案:
  shared 提供: allow_request(), record_success(), record_failure(), reset()
  各域保留特化方法在 extension trait 或 wrapper struct 中
```

### 二、RateLimiter 统一风险矩阵

| 风险 | 严重度 | 影响范围 | 缓解策略 |
|------|--------|----------|----------|
| **R9: TokenBucket 实现重复** | MEDIUM | nt_act 和 nt_io_provider 各有独立 TokenBucket | shared 提供统一 TokenBucket，各域 import |
| **R10: 方法命名不一致** | LOW | `try_acquire()` vs `try_request()` | shared 统一为 `try_acquire()`，各域适配 |
| **R11: RPM/TPM 双桶特殊性** | MEDIUM | 仅 nt_io_provider 有 | 作为 optional 字段 `rpm_bucket`/`tpm_bucket` |
| **R12: 滑动窗口 vs Token Bucket** | MEDIUM | goal_loop 用滑动窗口，其他用 token bucket | shared 同时支持两种模式 (via optional fields) |
| **R13: 分布式限流** | LOW | nt_act 有 `分布式: bool` 字段 | 作为 optional 字段保留 |

### 三、CacheEntry 统一风险矩阵

| 风险 | 严重度 | 影响范围 | 缓解策略 |
|------|--------|----------|----------|
| **R14: 泛型 V 约束** | LOW | shared 已是 `CacheEntry<V: Clone>` | 无冲突，直接使用 |
| **R15: access_count 类型差异** | LOW | `u64` vs `u32` | shared 统一为 `u64` |
| **R16: 特殊字段不可泛化** | LOW | fingerprint/program_bytes/cache_type | 这些保留为 domain-specific，不合并 |
| **R17: TTL 表示差异** | LOW | `Option<Duration>` vs `Option<u64>` | shared 使用 `Option<u64>` (秒) |

### 四、同名文件去重风险矩阵

| 风险 | 严重度 | 影响范围 | 缓解策略 |
|------|--------|----------|----------|
| **R18: arch_visualizer 双份** | LOW | nt_governance 和 nt_meta/nt_governance 各一份 | 检查内容是否一致，删除重复 |
| **R19: energy_flow 双份** | LOW | 同上 | 同上 |
| **R20: hybrid_search 双份** | MEDIUM | nt_memory 两个目录各一份 | 检查内容差异，合并或保留 |
| **R21: bm25 双份** | LOW | nt_mind/tests 和 nt_memory_kb | 测试文件可安全删除 |

### 五、执行依赖图

```
Phase 0: 验证 neotrix_types 编译 ──→ Phase 1: CircuitBreaker
                                       │
                                 Phase 2: RateLimiter
                                       │
                                 Phase 3: CacheEntry
                                       │
                                 Phase 4: 同名文件去重
                                       │
                                 Phase 5: 全量验证
```

### 六、回滚策略

每个 Phase 执行前创建 git stash:
```sh
git stash push -m "pre-phase-N-circuit-breaker"  # Phase 1
git stash push -m "pre-phase-N-rate-limiter"      # Phase 2
git stash push -m "pre-phase-N-cache-entry"       # Phase 3
git stash push -m "pre-phase-N-dedup"             # Phase 4
```

回滚:
```sh
git stash pop  # 恢复到执行前状态
```

### 七、验证检查表

| Phase | 验证命令 | 期望结果 |
|-------|----------|----------|
| 0 | `cargo check -p neotrix-types` | 编译通过 |
| 1 | `cargo check -p neotrix` | 无 CircuitBreaker 相关错误 |
| 1 | `cargo test -p neotrix --lib -- circuit_breaker` | 测试通过 |
| 2 | `cargo check -p neotrix` | 无 RateLimiter 相关错误 |
| 2 | `cargo test -p neotrix --lib -- rate_limiter` | 测试通过 |
| 3 | `cargo check -p neotrix` | 无 CacheEntry 相关错误 |
| 4 | `cargo check -p neotrix` | 无重复模块错误 |
| 5 | `cargo check -p neotrix` | 全量编译通过 |
| 5 | `cargo test -p neotrix --lib` | 全量测试通过 |

---

## 完整 TODO 列表

### Phase 0: 基础验证 (10min)
- [ ] **T0.1** `cargo check -p neotrix-types` — 验证 shared types 编译
- [ ] **T0.2** 检查 `neotrix_types::shared::CircuitBreaker` 字段完整性
- [ ] **T0.3** 检查 `neotrix_types::shared::RateLimiter` 字段完整性
- [ ] **T0.4** 检查 `neotrix_types::shared::CacheEntry<V>` 泛型兼容性

### Phase 1: CircuitBreaker 统一 (2h)
- [ ] **T1.1** git stash — 保存当前状态
- [ ] **T1.2** 扩展 `neotrix_types::shared::CircuitBreaker` — 添加 sliding_window, stall_count, max_stalls, half_open_max_probes, half_open_probes_used 字段
- [ ] **T1.3** 扩展 `neotrix_types::shared::BreakerState` — 添加 `since: Option<u64>` 到 Open 变体
- [ ] **T1.4** `core/nt_core_observer_error.rs` — 替换 CircuitState→BreakerState, CircuitBreaker→shared import, 适配 Open { since } → Open + last_opened_at 字段
- [ ] **T1.5** `l1_action/nt_act/nt_act_circuit_breaker.rs` — 替换 CircuitState→BreakerState, CircuitBreaker→shared import, 保留 CircuitBreakerManager 作为 wrapper
- [ ] **T1.6** `l1_action/nt_infra_breaker.rs` — 替换 BreakerState→shared import, 保留 BreakerRegistry 作为 wrapper
- [ ] **T1.7** `l1_action/nt_io/nt_io_provider/circuit_breaker.rs` — 替换 BreakerState→shared import, CircuitBreaker→shared import
- [ ] **T1.8** `l5_cognition/nt_mind/evolution/goal_loop/types.rs` — 替换 CircuitState→BreakerState, CircuitBreaker→shared import
- [ ] **T1.9** `l5_cognition/nt_mind/evolution/value_gate.rs` — 替换 CircuitBreaker→shared import
- [ ] **T1.10** `neotrix/nt_act/client.rs` — CircuitState→BreakerState
- [ ] **T1.11** `core/nt_core_error_recovery.rs` — 重命名 CircuitBreakerStrategy→ErrorRecoveryStrategy (不合并)
- [ ] **T1.12** `cargo check -p neotrix` — 验证编译
- [ ] **T1.13** `cargo test -p neotrix --lib -- circuit_breaker` — 验证测试

### Phase 2: RateLimiter 统一 (1h)
- [ ] **T2.1** git stash — 保存当前状态
- [ ] **T2.2** 扩展 `neotrix_types::shared::RateLimiter` — 添加 rpm_bucket, tpm_bucket, call_timestamps, max_calls_per_window 字段
- [ ] **T2.3** `l1_action/nt_act/nt_act_rate_limiter.rs` — 替换 RateLimiter→shared import, 保留 RateLimiterManager 作为 wrapper
- [ ] **T2.4** `l1_action/nt_io/nt_io_provider/rate_limiter.rs` — 替换 RateLimiter→shared import, 保留 TokenBucket 作为 shared import
- [ ] **T2.5** `l5_cognition/nt_mind/evolution/goal_loop/types.rs` — 替换 RateLimiter→shared import
- [ ] **T2.6** `cargo check -p neotrix` — 验证编译
- [ ] **T2.7** `cargo test -p neotrix --lib -- rate_limiter` — 验证测试

### Phase 3: CacheEntry 统一 (30min)
- [ ] **T3.1** git stash — 保存当前状态
- [ ] **T3.2** `core/nt_core_cache.rs` — CacheEntry → `use neotrix_types::shared::CacheEntry` (String)
- [ ] **T3.3** `l1_action/nt_act/nt_act_cache.rs` — CacheEntry → `use neotrix_types::shared::CacheEntry` (serde_json::Value)
- [ ] **T3.4** `l1_action/nt_memory/nt_memory_kb/nt_memory_sweep_20260815.rs` — CacheEntry → `use neotrix_types::shared::CacheEntry` (Vec<f64>)
- [ ] **T3.5** `cargo check -p neotrix` — 验证编译

### Phase 4: 同名文件去重 (30min)
- [ ] **T4.1** git stash — 保存当前状态
- [ ] **T4.2** `arch_visualizer.rs` — 检查 nt_governance/ 和 nt_meta/nt_governance/ 内容差异，删除重复
- [ ] **T4.3** `energy_flow.rs` — 检查 nt_governance/ 和 nt_meta/nt_governance/ 内容差异，删除重复
- [ ] **T4.4** `hybrid_search.rs` — 检查 nt_memory 两个目录内容差异，合并或删除
- [ ] **T4.5** `bm25.rs` — 删除 nt_mind/infrastructure/tests/bm25.rs (测试文件)
- [ ] **T4.6** `consciousness_bridge.rs` — 检查 l7_capability/ 和 nt_mind/consciousness/ 差异
- [ ] **T4.7** `context_budget.rs` — 检查 nt_core_context/ 和 nt_io_provider/ 差异
- [ ] **T4.8** `self_model.rs` — 检查 nt_core_meta/ 和 nt_core_self/ 差异
- [ ] **T4.9** `cargo check -p neotrix` — 验证编译

### Phase 5: 全量验证 (30min)
- [ ] **T5.1** `cargo check -p neotrix` — 全量编译
- [ ] **T5.2** `cargo test -p neotrix --lib` — 全量单元测试
- [ ] **T5.3** 重新审计 — 检查冗余计数是否下降
- [ ] **T5.4** 更新 evolution-task-list.md — 标记完成状态

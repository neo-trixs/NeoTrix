# 域重组计划 (Domain Reorganization)

## 目标

将当前基于层级 (l1-l10) 的目录结构重组为基于域 (NT-*) 的清晰结构。

## 当前问题

1. **层级命名混乱**: l1_body_impl 包含 NT-ACT 模块, l6_self_impl 也包含 NT-ACT 模块
2. **域分散**: NT-ACT 分散在 l1, l6, 外部 nt_act 目录
3. **层级与域不对应**: 10个层级 vs 13个域，没有清晰映射

## 目标结构

```
neotrix-core/src/neotrix/
├── nt_core/           # NT-CORE: E8, GWT, HyperCube, Self
├── nt_mind/           # NT-MIND: SEAL, distillation, skill crystallization
├── nt_memory/         # NT-MEMORY: SQLite KB, FTS5, embeddings
├── nt_world/          # NT-WORLD: UnifiedCrawler, fetchers, parsers
├── nt_act/            # NT-ACT: MCP tools, social media, code
├── nt_io/             # NT-IO: LLM providers, CLI, web server
├── nt_shield/         # NT-SHIELD: stealth net, proxy pool
├── nt_physical/       # NT-PHYSICAL: sensors, motors, safety
├── nt_feel/           # NT-FEEL: EmotionEngine, regulation
├── nt_meta/           # NT-META: meta-cognition
├── nt_repair/         # NT-REPAIR: self-healing
├── nt_governance/     # NT-GOVERNANCE: architecture oversight
├── nt_nexus/          # NT-NEXUS: cross-domain coordination
└── mod.rs             # 根模块
```

## 模块映射表

### NT-CORE (E8, GWT, HyperCube, Self)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `l4_cognition_impl/nt_core_blueprint.rs` | `nt_core/nt_core_blueprint.rs` | 蓝图提取 |
| `l4_cognition_impl/nt_core_design_extract.rs` | `nt_core/nt_core_design_extract.rs` | 设计提取 |
| `l4_cognition_impl/nt_core_doop.rs` | `nt_core/nt_core_doop.rs` | Doop |
| `l4_cognition_impl/nt_core_gencad.rs` | `nt_core/nt_core_gencad.rs` | GenCAD |
| `l4_cognition_impl/nt_core_kernel.rs` | `nt_core/nt_core_kernel.rs` | 内核 |
| `l4_cognition_impl/nt_core_parallel/` | `nt_core/nt_core_parallel/` | 并行处理 |
| `l4_cognition_impl/nt_core_simplify.rs` | `nt_core/nt_core_simplify.rs` | 简化 |
| `l4_cognition_impl/nt_core_three_scope_map.rs` | `nt_core/nt_core_three_scope_map.rs` | 三域映射 |
| `l5_consciousness_impl/nt_core_fep_iit/` | `nt_core/nt_core_fep_iit/` | FEP-IIT |
| `l5_consciousness_impl/nt_core_iit_phi.rs` | `nt_core/nt_core_iit_phi.rs` | IIT Phi |
| `l5_consciousness_impl/nt_core_signal/` | `nt_core/nt_core_signal/` | 信号系统 |
| `core/nt_core_self/` | `nt_core/nt_core_self/` | 自我模型 |
| `core/nt_core_meta/` | `nt_core/nt_core_meta/` | 元认知 |
| `core/nt_core_hcube/` | `nt_core/nt_core_hcube/` | HyperCube |
| `core/nt_core_e8/` | `nt_core/nt_core_e8/` | E8 |
| `core/nt_core_gwt/` | `nt_core/nt_core_gwt/` | GWT |

### NT-MIND (SEAL, distillation, skill crystallization)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `l8_autonomic_impl/nt_mind/` | `nt_mind/nt_mind/` | 核心 Mind |
| `l8_autonomic_impl/nt_mind_absorption_registry.rs` | `nt_mind/nt_mind_absorption_registry.rs` | 吸收注册 |
| `l8_autonomic_impl/nt_mind_autofixer.rs` | `nt_mind/nt_mind_autofixer.rs` | 自动修复 |
| `l8_autonomic_impl/nt_mind_background_config.rs` | `nt_mind/nt_mind_background_config.rs` | 后台配置 |
| `l8_autonomic_impl/nt_mind_background_loop/` | `nt_mind/nt_mind_background_loop/` | 后台循环 |
| `l8_autonomic_impl/nt_mind_benchmark.rs` | `nt_mind/nt_mind_benchmark.rs` | 基准测试 |
| `l8_autonomic_impl/nt_mind_build_runner.rs` | `nt_mind/nt_mind_build_runner.rs` | 构建运行器 |
| `l8_autonomic_impl/nt_mind_cleanup.rs` | `nt_mind/nt_mind_cleanup.rs` | 清理 |
| `l4_cognition_impl/nt_mind_bpco.rs` | `nt_mind/nt_mind_bpco.rs` | BPCO |
| `l4_cognition_impl/nt_mind_gasp.rs` | `nt_mind/nt_mind_gasp.rs` | GASP |

### NT-MEMORY (SQLite KB, FTS5, embeddings)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `l3_memory_impl/nt_memory_babeldoc.rs` | `nt_memory/nt_memory_babeldoc.rs` | BabelDoc |
| `l3_memory_impl/nt_memory_graphify.rs` | `nt_memory/nt_memory_graphify.rs` | Graphify |
| `l3_memory_impl/nt_memory_historian/` | `nt_memory/nt_memory_historian/` | 历史学家 |
| `l3_memory_impl/nt_memory_kb/` | `nt_memory/nt_memory_kb/` | 知识库 |
| `l3_memory_impl/nt_memory_knowledge_graph/` | `nt_memory/nt_memory_knowledge_graph/` | 知识图谱 |
| `l3_memory_impl/nt_memory_leann_store.rs` | `nt_memory/nt_memory_leann_store.rs` | LeanN Store |
| `l3_memory_impl/nt_memory_pdf_math_translate.rs` | `nt_memory/nt_memory_pdf_math_translate.rs` | PDF 数学翻译 |
| `l3_memory_impl/nt_memory_spatial/` | `nt_memory/nt_memory_spatial/` | 空间记忆 |
| `l3_memory_impl/nt_memory_yopedia.rs` | `nt_memory/nt_memory_yopedia.rs` | Yopedia |

### NT-WORLD (UnifiedCrawler, fetchers, parsers)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `l2_world_impl/nt_world_absorber/` | `nt_world/nt_world_absorber/` | 吸收器 |
| `l2_world_impl/nt_world_adsb.rs` | `nt_world/nt_world_adsb.rs` | ADS-B |
| `l2_world_impl/nt_world_agent_reach.rs` | `nt_world/nt_world_agent_reach.rs` | Agent Reach |
| `l2_world_impl/nt_world_aoi.rs` | `nt_world/nt_world_aoi.rs` | AOI |
| `l2_world_impl/nt_world_bgpview.rs` | `nt_world/nt_world_bgpview.rs` | BGPView |
| `l2_world_impl/nt_world_browse/` | `nt_world/nt_world_browse/` | 浏览 |
| `l2_world_impl/nt_world_browse_auto/` | `nt_world/nt_world_browse_auto/` | 自动浏览 |
| `l2_world_impl/nt_world_code_search.rs` | `nt_world/nt_world_code_search.rs` | 代码搜索 |
| `l2_world_impl/nt_world_crawl/` | `nt_world/nt_world_crawl/` | 爬虫 |
| `l2_world_impl/nt_world_sense/` | `nt_world/nt_world_sense/` | 感知 |

### NT-ACT (MCP tools, social media, code)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `l1_body_impl/nt_act_action_cache.rs` | `nt_act/nt_act_action_cache.rs` | 动作缓存 |
| `l1_body_impl/nt_act_autonomy/` | `nt_act/nt_act_autonomy/` | 自主性 |
| `l1_body_impl/nt_act_code/` | `nt_act/nt_act_code/` | 代码 |
| `l1_body_impl/nt_act_crypto.rs` | `nt_act/nt_act_crypto.rs` | 加密 |
| `l1_body_impl/nt_act_disk_guard.rs` | `nt_act/nt_act_disk_guard.rs` | 磁盘守卫 |
| `l1_body_impl/nt_act_goal/` | `nt_act/nt_act_goal/` | 目标 |
| `l1_body_impl/nt_act_media.rs` | `nt_act/nt_act_media.rs` | 媒体 |
| `l1_body_impl/nt_act_orchestrator/` | `nt_act/nt_act_orchestrator/` | 编排器 |
| `l1_body_impl/nt_act_sandbox.rs` | `nt_act/nt_act_sandbox.rs` | 沙盒 |
| `l1_body_impl/nt_act_seo.rs` | `nt_act/nt_act_seo.rs` | SEO |
| `l6_self_impl/nt_act_agentiker_code_intel.rs` | `nt_act/nt_act_agentiker_code_intel.rs` | Agentiker Code Intel |
| `l6_self_impl/nt_act_agentiker_plan.rs` | `nt_act/nt_act_agentiker_plan.rs` | Agentiker Plan |
| `l6_self_impl/nt_act_cashclaw.rs` | `nt_act/nt_act_cashclaw.rs` | CashClaw |
| `l6_self_impl/nt_act_frontier_agent.rs` | `nt_act/nt_act_frontier_agent.rs` | Frontier Agent |
| `l6_self_impl/nt_act_hermeskill.rs` | `nt_act/nt_act_hermeskill.rs` | HermesSkill |
| `l6_self_impl/nt_act_loop_engine.rs` | `nt_act/nt_act_loop_engine.rs` | Loop Engine |
| `l6_self_impl/nt_act_omni_route.rs` | `nt_act/nt_act_omni_route.rs` | Omni Route |
| `l6_self_impl/nt_act_openbot.rs` | `nt_act/nt_act_openbot.rs` | OpenBot |
| `l6_self_impl/nt_act_openexecutive.rs` | `nt_act/nt_act_openexecutive.rs` | OpenExecutive |
| `l6_self_impl/nt_act_rustdesk.rs` | `nt_act/nt_act_rustdesk.rs` | RustDesk |
| `nt_act/` (外部) | `nt_act/` | 保持不变 |

### NT-IO (LLM providers, CLI, web server)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `l7_capability_impl/nt_io_ai_image_prompts.rs` | `nt_io/nt_io_ai_image_prompts.rs` | AI Image Prompts |
| `l7_capability_impl/nt_io_cozyclay.rs` | `nt_io/nt_io_cozyclay.rs` | CozyClay |
| `l7_capability_impl/nt_io_eli5.rs` | `nt_io/nt_io_eli5.rs` | ELI5 |
| `l7_capability_impl/nt_io_excalidraw.rs` | `nt_io/nt_io_excalidraw.rs` | Excalidraw |
| `l7_capability_impl/nt_io_generative_media_skills.rs` | `nt_io/nt_io_generative_media_skills.rs` | Generative Media |
| `l7_capability_impl/nt_io_hermes_community.rs` | `nt_io/nt_io_hermes_community.rs` | Hermes Community |
| `l7_capability_impl/nt_io_hermes_quota.rs` | `nt_io/nt_io_hermes_quota.rs` | Hermes Quota |
| `l7_capability_impl/nt_io_pi_agent_desktop.rs` | `nt_io/nt_io_pi_agent_desktop.rs` | Pi Agent Desktop |
| `l7_capability_impl/nt_io_promo_bgm.rs` | `nt_io/nt_io_promo_bgm.rs` | Promo BGM |
| `l8_autonomic_impl/nt_io_web/` | `nt_io/nt_io_web/` | Web 服务 |

### NT-SHIELD (stealth net, proxy pool)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `core/nt_shield_sandbox/` | `nt_shield/nt_shield_sandbox/` | 沙盒 |
| `core/nt_shield_stalthnet/` | `nt_shield/nt_shield_stalthnet/` | 隐身网 |

### NT-PHYSICAL (sensors, motors, safety)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `l1_body_impl/nt_physical_sensors/` | `nt_physical/nt_physical_sensors/` | 传感器 |
| `l1_body_impl/nt_physical_motors/` | `nt_physical/nt_physical_motors/` | 电机 |
| `l1_body_impl/nt_physical_safety/` | `nt_physical/nt_physical_safety/` | 安全 |
| `l1_body_impl/nt_physical_power/` | `nt_physical/nt_physical_power/` | 电源 |

### NT-FEEL (EmotionEngine, regulation)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `core/nt_core_self/emotion_state.rs` | `nt_feel/nt_feel_emotion_state.rs` | 情感状态 |
| `core/nt_core_self/affective_interface.rs` | `nt_feel/nt_feel_affective_interface.rs` | 情感接口 |
| `core/nt_core_self/intrinsic_motivation.rs` | `nt_feel/nt_feel_intrinsic_motivation.rs` | 内在动机 |

### NT-META (meta-cognition)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `l5_consciousness_impl/nt_governance_human_oversight.rs` | `nt_meta/nt_meta_human_oversight.rs` | 人工监督 |
| `l7_capability_impl/nt_governance/` | `nt_meta/nt_governance/` | 治理 |

### NT-REPAIR (self-healing)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `l9_transcendent_impl/nt_mind_consciousness_monitor.rs` | `nt_repair/nt_repair_consciousness_monitor.rs` | 意识监控 |
| `l9_transcendent_impl/nt_mind_eval_harness.rs` | `nt_repair/nt_repair_eval_harness.rs` | 评估工具 |

### NT-NEXUS (cross-domain coordination)

| 当前位置 | 目标位置 | 模块名 |
|---------|---------|--------|
| `l10_transcendent_impl/consonance_orchestrator.rs` | `nt_nexus/nt_nexus_consonance.rs` | 协调器 |
| `l10_transcendent_impl/evolution_harness.rs` | `nt_nexus/nt_nexus_evolution.rs` | 演化工具 |
| `l10_transcendent_impl/meta_observer.rs` | `nt_nexus/nt_nexus_meta_observer.rs` | 元观察者 |
| `l10_transcendent_impl/transcendent_loop.rs` | `nt_nexus/nt_nexus_transcendent_loop.rs` | 超越循环 |

## 执行步骤

1. **创建目标目录结构**
2. **移动模块文件**
3. **更新 mod.rs 声明**
4. **更新 use 语句**
5. **验证构建**
6. **运行测试**
7. **更新文档**

## 注意事项

- 每次移动后验证构建
- 保持向后兼容 (可添加 re-export)
- 分阶段执行，避免一次性大规模变更

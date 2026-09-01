# 模块映射表 (Module Mapping Table)

## 目标

将所有模块从当前的层级结构映射到目标的域结构。

## NT-CORE 域

### 来源: core/ 中的基础模块

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `core/nt_core_self/` | `nt_core/nt_core_self/` | 自我模型 | 动态性能模型 |
| `core/nt_core_meta/` | `nt_core/nt_core_meta/` | 元认知 | 静态结构身份 |
| `core/nt_core_hcube/` | `nt_core/nt_core_hcube/` | HyperCube | 知识表示 |
| `core/nt_core_e8/` | `nt_core/nt_core_e8/` | E8 | 推理引擎 |
| `core/nt_core_gwt/` | `nt_core/nt_core_gwt/` | GWT | 注意力路由 |
| `core/nt_core_sense/` | `nt_core/nt_core_sense/` | 感知类型 | 基础感知定义 |
| `core/nt_core_signal/` | `nt_core/nt_core_signal/` | 信号系统 | 状态向量 |
| `core/nt_core_error.rs` | `nt_core/nt_core_error.rs` | 错误类型 | 基础错误 |
| `core/nt_core_heartbeat.rs` | `nt_core/nt_core_heartbeat.rs` | 心跳聚合 | 健康监控 |

### 来源: l4_cognition_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l4_cognition_impl/nt_core_blueprint.rs` | `nt_core/nt_core_blueprint.rs` | 蓝图 | 架构蓝图 |
| `l4_cognition_impl/nt_core_design_extract.rs` | `nt_core/nt_core_design_extract.rs` | 设计提取 | 设计模式提取 |
| `l4_cognition_impl/nt_core_doop.rs` | `nt_core/nt_core_doop.rs` | Doop | 依赖分析 |
| `l4_cognition_impl/nt_core_gencad.rs` | `nt_core/nt_core_gencad.rs` | GenCAD | CAD生成 |
| `l4_cognition_impl/nt_core_kernel.rs` | `nt_core/nt_core_kernel.rs` | 内核 | 核心逻辑 |
| `l4_cognition_impl/nt_core_parallel/` | `nt_core/nt_core_parallel/` | 并行处理 | 并行计算 |
| `l4_cognition_impl/nt_core_simplify.rs` | `nt_core/nt_core_simplify.rs` | 简化 | 代码简化 |
| `l4_cognition_impl/nt_core_three_scope_map.rs` | `nt_core/nt_core_three_scope_map.rs` | 三域映射 | 作用域映射 |

### 来源: l5_consciousness_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l5_consciousness_impl/nt_core_fep_iit/` | `nt_core/nt_core_fep_iit/` | FEP-IIT | 自由能原理 |
| `l5_consciousness_impl/nt_core_iit_phi.rs` | `nt_core/nt_core_iit_phi.rs` | IIT Phi | 整合信息论 |
| `l5_consciousness_impl/nt_core_signal/` | `nt_core/nt_core_signal/` | 信号系统 | 选择性状态 |

## NT-MIND 域

### 来源: l8_autonomic_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l8_autonomic_impl/nt_mind/` | `nt_mind/nt_mind/` | 核心 Mind | SEAL 管道 |
| `l8_autonomic_impl/nt_mind_absorption_registry.rs` | `nt_mind/nt_mind_absorption_registry.rs` | 吸收注册 | 经验吸收 |
| `l8_autonomic_impl/nt_mind_autofixer.rs` | `nt_mind/nt_mind_autofixer.rs` | 自动修复 | 自愈 |
| `l8_autonomic_impl/nt_mind_background_config.rs` | `nt_mind/nt_mind_background_config.rs` | 后台配置 | 配置管理 |
| `l8_autonomic_impl/nt_mind_background_loop/` | `nt_mind/nt_mind_background_loop/` | 后台循环 | 异步任务 |
| `l8_autonomic_impl/nt_mind_benchmark.rs` | `nt_mind/nt_mind_benchmark.rs` | 基准测试 | 性能测试 |
| `l8_autonomic_impl/nt_mind_build_runner.rs` | `nt_mind/nt_mind_build_runner.rs` | 构建运行器 | 构建自动化 |
| `l8_autonomic_impl/nt_mind_cleanup.rs` | `nt_mind/nt_mind_cleanup.rs` | 清理 | 代码清理 |

### 来源: l4_cognition_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l4_cognition_impl/nt_mind_bpco.rs` | `nt_mind/nt_mind_bpco.rs` | BPCO | 后向传播 |
| `l4_cognition_impl/nt_mind_gasp.rs` | `nt_mind/nt_mind_gasp.rs` | GASP | 通用抽象 |

## NT-MEMORY 域

### 来源: l3_memory_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l3_memory_impl/nt_memory_babeldoc.rs` | `nt_memory/nt_memory_babeldoc.rs` | BabelDoc | 文档解析 |
| `l3_memory_impl/nt_memory_graphify.rs` | `nt_memory/nt_memory_graphify.rs` | Graphify | 图谱化 |
| `l3_memory_impl/nt_memory_historian/` | `nt_memory/nt_memory_historian/` | 历史学家 | 历史记录 |
| `l3_memory_impl/nt_memory_kb/` | `nt_memory/nt_memory_kb/` | 知识库 | SQLite KB |
| `l3_memory_impl/nt_memory_knowledge_graph/` | `nt_memory/nt_memory_knowledge_graph/` | 知识图谱 | 图数据库 |
| `l3_memory_impl/nt_memory_leann_store.rs` | `nt_memory/nt_memory_leann_store.rs` | LeanN Store | 向量存储 |
| `l3_memory_impl/nt_memory_pdf_math_translate.rs` | `nt_memory/nt_memory_pdf_math_translate.rs` | PDF 数学翻译 | 文档翻译 |
| `l3_memory_impl/nt_memory_spatial/` | `nt_memory/nt_memory_spatial/` | 空间记忆 | 空间索引 |
| `l3_memory_impl/nt_memory_yopedia.rs` | `nt_memory/nt_memory_yopedia.rs` | Yopedia | 百科全书 |

## NT-WORLD 域

### 来源: l2_world_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l2_world_impl/nt_world_absorber/` | `nt_world/nt_world_absorber/` | 吸收器 | 内容吸收 |
| `l2_world_impl/nt_world_adsb.rs` | `nt_world/nt_world_adsb.rs` | ADS-B | 航空追踪 |
| `l2_world_impl/nt_world_agent_reach.rs` | `nt_world/nt_world_agent_reach.rs` | Agent Reach | 代理 reach |
| `l2_world_impl/nt_world_aoi.rs` | `nt_world/nt_world_aoi.rs` | AOI | 兴趣区域 |
| `l2_world_impl/nt_world_bgpview.rs` | `nt_world/nt_world_bgpview.rs` | BGPView | 网络分析 |
| `l2_world_impl/nt_world_browse/` | `nt_world/nt_world_browse/` | 浏览 | 网页浏览 |
| `l2_world_impl/nt_world_browse_auto/` | `nt_world/nt_world_browse_auto/` | 自动浏览 | 自动化浏览 |
| `l2_world_impl/nt_world_code_search.rs` | `nt_world/nt_world_code_search.rs` | 代码搜索 | 代码检索 |
| `l2_world_impl/nt_world_crawl/` | `nt_world/nt_world_crawl/` | 爬虫 | 网页爬虫 |
| `l2_world_impl/nt_world_sense/` | `nt_world/nt_world_sense/` | 感知 | 感知系统 |

## NT-ACT 域

### 来源: l1_body_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l1_body_impl/nt_act_action_cache.rs` | `nt_act/nt_act_action_cache.rs` | 动作缓存 | 缓存系统 |
| `l1_body_impl/nt_act_autonomy/` | `nt_act/nt_act_autonomy/` | 自主性 | 自主决策 |
| `l1_body_impl/nt_act_code/` | `nt_act/nt_act_code/` | 代码 | 代码执行 |
| `l1_body_impl/nt_act_crypto.rs` | `nt_act/nt_act_crypto.rs` | 加密 | 加密工具 |
| `l1_body_impl/nt_act_disk_guard.rs` | `nt_act/nt_act_disk_guard.rs` | 磁盘守卫 | 磁盘保护 |
| `l1_body_impl/nt_act_goal/` | `nt_act/nt_act_goal/` | 目标 | 目标管理 |
| `l1_body_impl/nt_act_media.rs` | `nt_act/nt_act_media.rs` | 媒体 | 媒体处理 |
| `l1_body_impl/nt_act_orchestrator/` | `nt_act/nt_act_orchestrator/` | 编排器 | 任务编排 |
| `l1_body_impl/nt_act_sandbox.rs` | `nt_act/nt_act_sandbox.rs` | 沙盒 | 沙盒执行 |
| `l1_body_impl/nt_act_seo.rs` | `nt_act/nt_act_seo.rs` | SEO | 搜索优化 |

### 来源: l6_self_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l6_self_impl/nt_act_agentiker_code_intel.rs` | `nt_act/nt_act_agentiker_code_intel.rs` | Agentiker Code Intel | 代码智能 |
| `l6_self_impl/nt_act_agentiker_plan.rs` | `nt_act/nt_act_agentiker_plan.rs` | Agentiker Plan | 计划生成 |
| `l6_self_impl/nt_act_cashclaw.rs` | `nt_act/nt_act_cashclaw.rs` | CashClaw | 金融工具 |
| `l6_self_impl/nt_act_frontier_agent.rs` | `nt_act/nt_act_frontier_agent.rs` | Frontier Agent | 前沿代理 |
| `l6_self_impl/nt_act_hermeskill.rs` | `nt_act/nt_act_hermeskill.rs` | HermesSkill | 技能系统 |
| `l6_self_impl/nt_act_loop_engine.rs` | `nt_act/nt_act_loop_engine.rs` | Loop Engine | 循环引擎 |
| `l6_self_impl/nt_act_omni_route.rs` | `nt_act/nt_act_omni_route.rs` | Omni Route | 路由系统 |
| `l6_self_impl/nt_act_openbot.rs` | `nt_act/nt_act_openbot.rs` | OpenBot | 开放机器人 |
| `l6_self_impl/nt_act_openexecutive.rs` | `nt_act/nt_act_openexecutive.rs` | OpenExecutive | 开放执行 |
| `l6_self_impl/nt_act_rustdesk.rs` | `nt_act/nt_act_rustdesk.rs` | RustDesk | 远程桌面 |

## NT-IO 域

### 来源: l7_capability_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l7_capability_impl/nt_io_ai_image_prompts.rs` | `nt_io/nt_io_ai_image_prompts.rs` | AI Image Prompts | 图像提示 |
| `l7_capability_impl/nt_io_cozyclay.rs` | `nt_io/nt_io_cozyclay.rs` | CozyClay | Clay 工具 |
| `l7_capability_impl/nt_io_eli5.rs` | `nt_io/nt_io_eli5.rs` | ELI5 | 简化解释 |
| `l7_capability_impl/nt_io_excalidraw.rs` | `nt_io/nt_io_excalidraw.rs` | Excalidraw | 绘图工具 |
| `l7_capability_impl/nt_io_generative_media_skills.rs` | `nt_io/nt_io_generative_media_skills.rs` | Generative Media | 生成媒体 |
| `l7_capability_impl/nt_io_hermes_community.rs` | `nt_io/nt_io_hermes_community.rs` | Hermes Community | 社区工具 |
| `l7_capability_impl/nt_io_hermes_quota.rs` | `nt_io/nt_io_hermes_quota.rs` | Hermes Quota | 配额管理 |
| `l7_capability_impl/nt_io_pi_agent_desktop.rs` | `nt_io/nt_io_pi_agent_desktop.rs` | Pi Agent Desktop | 桌面代理 |
| `l7_capability_impl/nt_io_promo_bgm.rs` | `nt_io/nt_io_promo_bgm.rs` | Promo BGM | 背景音乐 |

### 来源: l8_autonomic_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l8_autonomic_impl/nt_io_web/` | `nt_io/nt_io_web/` | Web 服务 | Web 服务器 |

## NT-SHIELD 域

### 来源: core/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `core/nt_shield_sandbox/` | `nt_shield/nt_shield_sandbox/` | 沙盒 | 安全沙盒 |
| `core/nt_shield_stalthnet/` | `nt_shield/nt_shield_stalthnet/` | 隐身网 | 隐身网络 |

## NT-PHYSICAL 域

### 来源: l1_body_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l1_body_impl/nt_physical_sensors/` | `nt_physical/nt_physical_sensors/` | 传感器 | 传感器系统 |
| `l1_body_impl/nt_physical_motors/` | `nt_physical/nt_physical_motors/` | 电机 | 电机控制 |
| `l1_body_impl/nt_physical_safety/` | `nt_physical/nt_physical_safety/` | 安全 | 安全系统 |
| `l1_body_impl/nt_physical_power/` | `nt_physical/nt_physical_power/` | 电源 | 电源管理 |

## NT-FEEL 域

### 来源: core/nt_core_self/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `core/nt_core_self/emotion_state.rs` | `nt_feel/nt_feel_emotion_state.rs` | 情感状态 | 情感定义 |
| `core/nt_core_self/affective_interface.rs` | `nt_feel/nt_feel_affective_interface.rs` | 情感接口 | 情感交互 |
| `core/nt_core_self/intrinsic_motivation.rs` | `nt_feel/nt_feel_intrinsic_motivation.rs` | 内在动机 | 动机系统 |

## NT-META 域

### 来源: l5_consciousness_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l5_consciousness_impl/nt_governance_human_oversight.rs` | `nt_meta/nt_meta_human_oversight.rs` | 人工监督 | 人工监督 |

### 来源: l7_capability_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l7_capability_impl/nt_governance/` | `nt_meta/nt_governance/` | 治理 | 架构治理 |

## NT-REPAIR 域

### 来源: l9_transcendent_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l9_transcendent_impl/nt_mind_consciousness_monitor.rs` | `nt_repair/nt_repair_consciousness_monitor.rs` | 意识监控 | 健康监控 |
| `l9_transcendent_impl/nt_mind_eval_harness.rs` | `nt_repair/nt_repair_eval_harness.rs` | 评估工具 | 评估系统 |

## NT-NEXUS 域

### 来源: l10_transcendent_impl/

| 当前路径 | 目标路径 | 模块名 | 说明 |
|---------|---------|--------|------|
| `l10_transcendent_impl/consonance_orchestrator.rs` | `nt_nexus/nt_nexus_consonance.rs` | 协调器 | 协调系统 |
| `l10_transcendent_impl/evolution_harness.rs` | `nt_nexus/nt_nexus_evolution.rs` | 演化工具 | 演化系统 |
| `l10_transcendent_impl/meta_observer.rs` | `nt_nexus/nt_nexus_meta_observer.rs` | 元观察者 | 观察系统 |
| `l10_transcendent_impl/transcendent_loop.rs` | `nt_nexus/nt_nexus_transcendent_loop.rs` | 超越循环 | 超越系统 |

## 统计

| 域 | 模块数量 | 主要来源 |
|----|---------|---------|
| NT-CORE | 15+ | core/, l4_cognition_impl/, l5_consciousness_impl/ |
| NT-MIND | 10+ | l8_autonomic_impl/, l4_cognition_impl/ |
| NT-MEMORY | 9+ | l3_memory_impl/ |
| NT-WORLD | 10+ | l2_world_impl/ |
| NT-ACT | 20+ | l1_body_impl/, l6_self_impl/, nt_act/ |
| NT-IO | 10+ | l7_capability_impl/, l8_autonomic_impl/ |
| NT-SHIELD | 2+ | core/ |
| NT-PHYSICAL | 4+ | l1_body_impl/ |
| NT-FEEL | 3+ | core/nt_core_self/ |
| NT-META | 2+ | l5_consciousness_impl/, l7_capability_impl/ |
| NT-REPAIR | 2+ | l9_transcendent_impl/ |
| NT-GOVERNANCE | (包含在 NT-META) | - |
| NT-NEXUS | 4+ | l10_transcendent_impl/ |

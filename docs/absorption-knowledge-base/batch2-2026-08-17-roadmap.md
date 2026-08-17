# Batch2 吸收 · 全架构能力对标与进化路线 (2026-08-17)

> **输入**: 28 唯一 URL (11 arXiv papers + 15 GitHub repos + 1 HF dataset + 1 blog) → 预检 28/28 可达。
> **方法**: C1-C6 契约五组并行研究 (MIND:7 / MEMORY:5 / WORLD:5 / IO:4 / ACT+SHIELD:6) → 五 artifact 落盘 `notes/absorption-20260817-b2-{mind,memory,world,io,act-shield}.md` → 主 agent 逐条验证 (wc -l + 条目数 + KB 入库抽查)。
> **KB**: 21 新增 (Rust CLI `neotrix-experience absorb-node` 权威计数) + 7 已存在去重 (hermes-agent / pi / 2608.09867 / agency-agents / memvid / ego-lite / hyperresearch)。能力映射 2007/2007 (100%)。
> **纪律**: R-P42 (强化现有节点, 禁平行适配器) · R-P79 (同 session 生产接线, 缺消费者即拒) · R-P100 (能力注册能力树) · R-P16 (编辑后 re-read 验证持久化)。
> **范围**: 本轮 = KB 入库 + 能力映射 + 路线图; 代码接线按 seabatch 模式延期到后续 session (R-P79 消费者已具名)。

---

## 0. 吸收摘要

| 组 | 源数 | Reinforce | New 候选 | 最强源 | 顶层主题 |
|----|------|-----------|----------|--------|----------|
| MIND | 7 | 18 | 4 | PI-blog (Measuring Autonomous AI Research) | 实验设计纪律 (seed 梯度/影子评测/符号回归/持久实验区) |
| MEMORY | 5 | 8 节点 | 15 | hyperresearch (研究知识库) | 来源质量评分 / untrusted 围栏 / 引文门 / 断点续跑 |
| WORLD | 5 | 21 | 5 | text-albumentations (约束选择) | 语法约束选择 / bbox 引用锚点 / 多信号 evaluator |
| IO | 4 | 12 | 7 | OasisKV (预测式分层缓存) | lookahead 预取 / 约束解码 / 凭据执行分离 |
| ACT+SHIELD | 6 | 12 | 7 | arxiv 2608.09867 (威胁模型) | 加密 CoT 生命周期防护 / keep-or-revert / 声音注入 |

**合计**: 28 源 → **Reinforce 68 机制注入既有节点** (R-P42 合规) + **New 候选 38** (全部具名消费者, R-P79 满足)。

**跨域最高信号主题 (本批共鸣)**:
1. **实验/反馈信号层级化** (S1/S5/S7 MIND + awesome-autoresearch): 失败轨迹必需、step-level 优先、seed 梯度付费 (1→3→8)、re-ablate on change — 与 fractal review + RstFlywheel 天然同构。
2. **来源可信度工程** (hyperresearch + 2608.09867): 复合质量评分 + retraction 归零 + untrusted 内容围栏 + 引文逐字门 + 加密推理块会话绑定 — 威胁模型驱动, P0 安全优先级。
3. **约束即选择** (text-alb + neural-txt): 语法约束选择器 (不能选不存在之物) + constrained decoding (JSON 强制) — 防幻觉双通道。
4. **预测式缓存/预取** (OasisKV + magic-context + qlib): lookahead 预取 + cache-aware 延迟裁剪 + Point-in-Time 快照 — 把反应式缓存升级为预测式。

---

## 1. 特性对标: 能力网逐节点 Gap 矩阵

标记: ✅ 已有且成熟 · 🟡 已有但缺关键机制 (本批补强) · ❌ 缺失/弱 (本批 New 目标)。

### NT-MIND (进化工匠) — 进化域

| 外部模式 (来源) | NeoTrix 现状 | 缺口 (本批补强点) | Tier |
|-----------------|--------------|-------------------|------|
| 影子评测: 原作者对 agent 产出打分 (2607.27191) | 🟡 `nt_mind_eval_harness` (SelfTest 自动验证) | 无"领域原作者评审"通道 | **New** `nt_mind_eval_shadow_review` |
| EML 单一二元算子完备性 (2603.21852) | 🟡 `nt_core_hcube` (VSA 向量运算) | 无原语完备性/最小原语集节点 | **New** `nt_core_eml_primitive` |
| 梯度符号回归: EML-tree + Adam 恢复闭式公式 (2603.21852) | 🟡 `nt_core_hcube::latent_recurrent` | 无符号回归节点 | **New** `nt_mind_symbolic_regression` |
| 持久 IPython 内核 = 累积实验工作区 (PI-blog) | 🟡 `nt_core_hcube::latent_recurrent` | 无"持久实验工作区"概念 | **New** `nt_mind_persistent_lab` |
| 反射式 prompt 进化 GEPA (awesome-autoresearch) | 🟡 `nt_mind_skill_engine` | 无自然语言反思进化优化器 | **New** `nt_mind_evolve_reflect` |
| 5 失败模式清单 + 回溯缺失检测 (2607.27191) | 🟡 `nt_mind_self_diagnose` | 行为漂移特征编码 + dead-end 信号 | Reinforce |
| 小规模实验超参敏感度/曲面维度 (2608.11859) | 🟡 `nt_mind_eval_harness::SmallScaleMethod` | hyperparam-sweep-first 门 + 跨规模外推校验 | Reinforce |
| SPOT 探测预算 + outcome 校准蒸馏 (2608.04419) | 🟡 `nt_mind_skill_engine`/`nt_mind_distiller` | probe_budget_allocator + outcome-calibrated target | Reinforce |
| SFS-DPO 自我验证两阶段 + step-level 偏好 (2608.11573) | 🟡 `nt_mind_evolution_loop::TrainPipeline` | correction-stage gating + step-level 偏好对 | Reinforce |
| frozen verifier + seed 梯度付费 + re-ablate (PI-blog) | 🟡 `nt_mind_guard`/`nt_mind_eval_harness`/`nt_mind_evolution_loop` | 统计阈值验证器 + seed escalation + post-merge re-ablation | Reinforce |
| 独立 LLM monitor 每小时审计 (PI-blog) | 🟡 `nt_mind_background_loop` (60s tick) | 观察者与执行者分离的独立 monitor | Reinforce |
| keep-or-revert + GOAL.md fitness-first (awesome-autoresearch) | 🟡 `nt_act_autonomy` | 单次单改动 + 强制可测 fitness | Reinforce |

### NT-MEMORY (知识守护者) — 知识域

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| 复合 quality_score + retraction 归零 + PageRank (hyperresearch) | 🟡 `nt_memory_kb::retrieval_matrix` | 来源质量加权排序缺 | **New** `nt_memory_quality_rank` |
| untrusted-source fencing `<untrusted-source url>` (hyperresearch) | 🟡 `nt_memory_kb` 摄取端 | 外部内容标记 data-not-instructions 缺 | **New** `nt_memory_untrusted_fence` |
| 引文逐字完整性门禁 + 数字一致性 (hyperresearch) | 🟡 `nt_memory_curation::fidelity_ledger` | 经验引用逐字可回溯门缺 | **New** `nt_memory_quote_gate` |
| 断点续跑 manifest (`run resume`) (hyperresearch) | 🟡 experience-tree `close --cycle` | 经验/进化周期断点续跑缺 | **New** `nt_memory_run_manifest` |
| Capture taxonomy 固定记忆类别 (magic-context) | 🟡 experience-tree 五阶段 | 结构化经验分类受控词表缺 | **New** `nt_memory_capture_taxonomy` |
| Decay rendering 确定性降级 (magic-context) | 🟡 `nt_core_gwt::mode_router` | 上下文预算内确定性降采样缺 | **New** `nt_memory_decay_render` |
| Cache-aware queued reduction (magic-context) | 🟡 `nt_core_parallel::staged_shared_context` | 延迟裁剪保 prompt cache 缺 | **New** `nt_memory_cache_drops` |
| Stable project identity 而非路径 (magic-context) | 🟡 `nt_memory_kb` 命名空间 | 记忆跟随 repo 身份缺 | **New** `nt_memory_repo_identity` |
| 副本聚类独立投票 (hyperresearch) | 🟡 `nt_memory_curation::dedup` | 内容相似度聚类合并权重缺 | **New** `nt_memory_independence` |
| embedding 模型绑定 (memvid) | 🟡 `nt_memory_kb` | embedding 漂移防护缺 | **New** `nt_memory_embed_binding` |
| 记忆胶囊导出/分享契约 (memvid) | 🟡 `nt_memory_kb::single_file_memory` | 跨机迁移/过期/加密分享缺 | **New** `nt_memory_capsule` |
| 指令模板多样化 20+ 提取格式 (paper_instructions_300K) | 🟡 experience-tree 蒸馏 | 受控提取指令模板库缺 | **New** `nt_memory_sft_extract_templates` |
| 推理路径与输出分离存储 (paper_instructions_300K) | 🟡 `nt_core_hcube::latent_recurrent` | reasoning-trace 分离缺 | **New** `nt_memory_reasoning_trace` |
| 规范指令 verbatim 单一真源 (hyperresearch) | 🟡 `nt_core_parallel::staged_shared_context` | 任务指令 verbatim 固化缺 | **New** `nt_core_query_gospel` |
| 内外威胁双分类 (2504.01990 survey) | 🟡 NT-SHIELD | 内在/外在威胁分类缺 (跨域) | **New** `nt_core_threat_taxonomy` |
| Historian 分层 compartment + 重要性评分 (magic-context) | 🟡 `nt_memory_historian::temporal_context_graph` | tiered 摘要 + importance score 缺 | Reinforce |
| Markdown-is-truth / 索引可重建 (hyperresearch) | 🟡 `nt_memory_kb` (SQLite 唯一真源) | 真源+可重建索引分层审计缺 | Reinforce |
| Provenance breadcrumbs rooted tree (hyperresearch) | 🟡 `nt_memory_provenance::signed_provenance` | 建议-来源链 + 断链 lint 缺 | Reinforce |
| Dreamer 睡眠巩固循环 (magic-context) | 🟡 `nt_memory_curation::fidelity_ledger` | verify-against-codebase + retrospective 缺 | Reinforce |
| Recall 多源一体 ctx_search (magic-context) | 🟡 `nt_memory_kb::retrieval_matrix` | git 提交索引 + 多源联合缺 | Reinforce |

### NT-WORLD (虚空探索者) — 感知域

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| 语法约束选择器 MetaAugmentation (text-alb) | 🟡 `nt_world_absorber::self_curriculum` | 不能选择不存在之物 — 防幻觉选择缺 | **New** `nt_world_absorber::constrained_selection` |
| Tagged PDF + veraPDF 程序化验证 (opendataloader) | 🟡 `nt_file_ability` | 结构标签 + 自动验证缺 | **New** `nt_file_ability::structure_tag` |
| 真实 OS 环境评测 harness (DSAgentBench) | 🟡 `nt_shield_sandbox` + NT-ACT | 端到端 env 评测 harness 缺 | **New** `nt_core_self::env_eval_harness` |
| 确定性多信号 evaluator (DSAgentBench) | 🟡 `converge_check` | 产出物级验证 (非 code-only) 缺 | **New** `nt_core_self::multi_signal_eval` |
| Point-in-Time 数据库 (qlib) | 🟡 `nt_memory_kb` | 时序快照防 look-ahead 缺 | **New** `nt_memory_kb::point_in_time` |
| NestedExecutor 多层嵌套优化 (qlib) | 🟡 `nt_agent_orchestrator` | 多粒度策略嵌套缺 | **New** `nt_agent_orchestrator::nested_executor` |
| XY-Cut++ 阅读顺序 + bbox 引用锚点 (opendataloader) | 🟡 `nt_file_ability` (FileModel) | 语义元素 + bbox 锚点缺 | Reinforce |
| 确定性本地 + Hybrid AI 路由 (opendataloader) | 🟡 `nt_file_ability::doc-parse` | 简单/复杂页自动路由缺 | Reinforce |
| 语义类型 JSON schema (opendataloader) | 🟡 `nt_file_ability` (FileModel) | 语义类型枚举扩展缺 | Reinforce |
| 基础模型策展目录 + 可加载元数据 (geoai) | 🟡 `nt_world_absorber::api_registry_meta` | 可加载能力目录字段缺 | Reinforce |
| 廉价预筛门 PassageQuality (text-alb) | 🟡 `nt_world_absorber::self_curriculum` | 微预算二元质量门缺 | Reinforce |
| extractive QA 引用验证 (text-alb) | 🟡 `nt_world_absorber` | 逐字引用保留门缺 (cite-ledger 对齐) | Reinforce |
| 声明式 workflow config + qrun (qlib) | 🟡 SEAL pipeline | 声明式 stage 配置层缺 | Reinforce |
| 因子表达式引擎 (qlib) | 🟡 `nt_mind_distiller` | 表达式 DAG 派生特征缺 | Reinforce |

### NT-IO (界面使徒) — 界面域

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| 凭据留宿主 + 执行进微 VM (pi Gondolin) | 🟡 `nt_io_agent_loop` + `nt_shield_sandbox` | 凭据/执行隔离缺 | **New** `host_cred_exec_split` (高) |
| fail-closed 公网隧道 (unsloth `--secure`) | 🟡 `nt_io_web` | 隧道失败拒绝启动 + 密码轮换缺 | **New** `secure_tunnel_fail_closed` |
| 离线结构化 NLP (neural-txt 135M) | 🟡 `nt_io_multimodal_transform` | 本地结构化抽取缺 | **New** `local_nlp_tasks` |
| 奖励打分 rank 候选 (neural-txt) | 🟡 `nt_io_agent_loop` | rank-based 候选选择缺 | **New** `reward_rank_selector` |
| 差分渲染 TUI (pi-tui) | 🟡 neotrix-tauri | 只重绘变化单元格缺 | **New** `differential_render_tui` |
| 会话脱敏发布 (pi-share-hf) | 🟡 `nt_memory_kb` + `nt_io_telemetry` | 真实 session 发布通道缺 | **New** `session_publish` |
| 供应链锁定 (unsloth/pi) | 🟡 dev-rules D9 | shrinkwrap allowlist + 定时 audit 缺 | **New** `supply_chain_lock` (过程规则) |
| OasisKV 分层 KV + lookahead 预取 + 后台管线 | 🟡 `nt_io_provider::ResponseCache` (反应式 prefetch, gateway.rs:370) | 预测式预取 + 分层 staging 缺 | Reinforce |
| constrained decoding (neural-txt Outlines) | 🟡 `nt_io_output_style::output_governor` | 约束解码替代启发式校验缺 | Reinforce |
| 推理轨迹双面 API (neural-txt) | 🟡 `nt_io_agent_loop` | reasoning-trace 保留/剥离缺 | Reinforce |
| rollouts + beam 双解码 (neural-txt) | 🟡 `nt_io_provider` (单候选) | 多候选/beam 解码缺 | Reinforce |
| 厂商中立 telemetry 契约 (pi-telemetry) | 🟡 `nt_io_telemetry` | schema 化 + conformance test 缺 | Reinforce |
| 后端运行时选择持久化 (unsloth) | 🟡 `nt_io_provider::factory` | backend 探测/选择持久化缺 | Reinforce |
| local-subagent 路由 (unsloth) | 🟡 `nt_io_provider` | 主 provider + 本地 fallback 缺 | Reinforce |

### NT-ACT / NT-SHIELD (行动执行者 / 影卫)

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| 加密 CoT 块生命周期管理 (2608.09867) | 🟡 `nt_shield_audit::reasoning_trace_guard` | 会话密钥派生 + 互换检测缺 | **New** `nt_shield_coh_guard` (P0) |
| 声音/风格注入层 (agency-agents Whimsy + writing-skills persona) | 🟡 `nt_agent_orchestrator` | 防千篇一律的 voice 层缺 | **New** `nt_act_voice` |
| 文体规程注册表 (writing-skills) | 🟡 `nt_act_orchestrator` | 每文体可执行规程缺 | **New** `nt_act_style_registry` |
| 共享登录态浏览器桥 (ego-lite) | 🟡 `nt_shield_sandbox` + `nt_world_crawl` | 浏览器自动化桥缺 (需 credential vault 授权) | **New** `nt_shield_browser_state` |
| 自然语言 cron + 多平台投递 (hermes-agent) | 🟡 `nt_act_autonomy` + `nt_io_provider` | 调度器缺 | **New** `nt_act_scheduler` |
| Swarm 实验认领/最优配置同步 (awesome-autoresearch) | 🟡 `nt_agent_orchestrator` + `nt_mind` | 多 agent 共享进化缺 | **New** `nt_act_swarm` |
| Personality+Process+Metrics 四件套 (agency-agents) | 🟡 `nt_agent_orchestrator::expert_team_diff` | 专家性格/声音路由缺 | Reinforce |
| Reality Checker + Evidence Collector 双门 (agency-agents) | 🟡 `nt_shield_agentic_scan::poc_verification_gate` | 交付前 evidence gate 缺 | Reinforce |
| AI-Generated Code Security Auditor (agency-agents) | 🟡 `nt_shield_audit` | vibe-code 专项审查规则 (secrets/RLS/PI sinks) 缺 | Reinforce |
| 命令审批 allowlist (hermes-agent) | 🟡 `nt_shield_sandbox::egress_control` | 命令审批清单缺 | Reinforce |
| 轨迹压缩 + PII 剥离 (hermes-agent) | 🟡 `nt_shield_audit::decision_trail` | 轨迹压缩审计 + PII 剥离门缺 | Reinforce |
| Plan-hash 审批 + append-only ledger (awesome-autoresearch) | 🟡 `nt_act_orchestrator` | 实验计划哈希审批 + 防篡改账本缺 | Reinforce |
| 共享登录态隔离 browser space (ego-lite) | 🟡 `nt_shield_sandbox::device_sandbox` | 独立 browser 空间缺 | Reinforce |
| 加密 CoT 会话绑定/日志 PII 扫描/注入检测/分歧检测 (2608.09867) | 🟡 `nt_shield_audit` + `nt_io_provider` | reasoning_trace_guard 四项防护缺 | Reinforce |

---

## 2. New 候选清单 (38, 全部具名消费者, R-P79 满足)

| # | bud | 域 | 消费者 | 优先级 |
|---|-----|----|--------|--------|
| N1 | `nt_shield_coh_guard` | NT-SHIELD | nt_io_provider + nt_shield_audit | P0 (安全威胁) |
| N2 | `nt_memory_untrusted_fence` | NT-MEMORY | nt_memory_kb 摄取端 (NT-WORLD/NT-IO 输入端, NT-SHIELD 复用) | P0 |
| N3 | `nt_memory_quality_rank` | NT-MEMORY | nt_memory_kb::retrieval_matrix + nt_core_knowledge_graph | P0 |
| N4 | `nt_core_self::multi_signal_eval` | NT-CORE | converge_check 产出门 | P0 |
| N5 | `nt_world_absorber::constrained_selection` | NT-WORLD | self_curriculum 选择器 | P1 |
| N6 | `nt_memory_sft_extract_templates` | NT-MEMORY | experience-tree 蒸馏 + nt_core_knowledge_graph | P1 |
| N7 | `nt_memory_run_manifest` | NT-MEMORY | SEAL pipeline / experience-tree 断点 | P1 |
| N8 | `nt_memory_embed_binding` | NT-MEMORY | nt_memory_kb 写入校验 | P1 |
| N9 | `nt_core_eml_primitive` | NT-CORE | nt_core_hcube (VSA 统一算子) | P1 |
| N10 | `nt_mind_symbolic_regression` | NT-MIND | nt_mind_distiller (可逆规则蒸馏) | P1 |
| N11 | `nt_mind_eval_shadow_review` | NT-MIND | nt_mind_background_loop::handlers_absorption | P1 |
| N12 | `nt_io_provider::lookahead_prefetch` | NT-IO | ResponseCache::prefetch (gateway.rs:370) | P1 |
| N13 | `host_cred_exec_split` | NT-IO | nt_io_agent_loop + nt_shield_sandbox | P1 |
| N14 | `nt_act_voice` | NT-ACT | nt_agent_orchestrator + nt_io_provider | P2 |
| N15 | `nt_mind_persistent_lab` | NT-MIND | nt_mind_evolution_loop | P2 |
| N16 | `nt_mind_evolve_reflect` | NT-MIND | nt_mind_skill_engine + nt_act_autonomy | P2 |
| N17 | `nt_core_self::env_eval_harness` | NT-CORE | NT-ACT 执行验证 | P2 |
| N18 | `nt_memory_kb::point_in_time` | NT-MEMORY | nt_world_absorber 摄入时序数据 | P2 |
| N19 | `nt_agent_orchestrator::nested_executor` | NT-ACT | NT-ACT 多粒度任务编排 | P2 |
| N20 | `nt_memory_quote_gate` | NT-MEMORY | fidelity_ledger + cite-ledger | P2 |
| N21 | `nt_memory_independence` | NT-MEMORY | curation dedup + retrieval | P2 |
| N22 | `nt_memory_capture_taxonomy` | NT-MEMORY | nt_memory_kb 写入 + AGENTS.md 门禁 | P2 |
| N23 | `nt_memory_decay_render` | NT-MEMORY | nt_core_gwt::mode_router | P2 |
| N24 | `nt_memory_cache_drops` | NT-MEMORY | nt_core_parallel::staged_shared_context | P2 |
| N25 | `nt_memory_repo_identity` | NT-MEMORY | nt_memory_kb 命名空间键 | P2 |
| N26 | `nt_memory_reasoning_trace` | NT-MEMORY | nt_core_hcube::latent_recurrent + SelfTest T3 | P2 |
| N27 | `nt_memory_capsule` | NT-MEMORY | experience-tree 跨机迁移 | P3 |
| N28 | `nt_core_query_gospel` | NT-CORE | nt_core_parallel::staged_shared_context | P3 |
| N29 | `nt_core_threat_taxonomy` | NT-CORE (跨域) | nt_shield_sandbox + nt_memory_untrusted_fence | P3 |
| N30 | `nt_file_ability::structure_tag` | NT-WORLD | 文档无障碍/结构输出 | P3 |
| N31 | `nt_act_swarm` | NT-ACT | nt_agent_orchestrator + nt_mind | P3 |
| N32 | `nt_act_style_registry` | NT-ACT | nt_act_orchestrator + nt_io_provider | P3 |
| N33 | `nt_shield_browser_state` | NT-SHIELD | nt_shield_sandbox + nt_world_crawl (credential vault 授权) | P3 |
| N34 | `nt_act_scheduler` | NT-ACT | nt_act_autonomy + nt_io_provider | P3 |
| N35 | `secure_tunnel_fail_closed` | NT-IO | nt_io_web + nt_shield_sandbox Egress Policy | P3 |
| N36 | `local_nlp_tasks` | NT-IO | nt_io_multimodal_transform + nt_memory_kb | P3 |
| N37 | `reward_rank_selector` | NT-IO | nt_io_agent_loop | P3 |
| N38 | `differential_render_tui` / `session_publish` / `supply_chain_lock` | NT-IO | neotrix-tauri / nt_memory_kb / 构建管线 | P3 (备选) |

## 3. 进化迭代路线 (Roadmap)

> 原则: 每 Phase 一内聚特征组, 强化既有节点 (R-P42) + New 具名消费者 (R-P79)。落地即 `neotrix-capability` Strengthen/Bud 登记 (R-P100)。

### Phase 0 — 安全与来源可信度 (最高 ROI, 5 项)
1. **加密 CoT 生命周期防护** (2608.09867, N1 + Reinforce reasoning_trace_guard): `nt_shield_coh_guard` 会话密钥派生 + 互换检测 + 轮换; `reasoning_trace_guard` 加会话绑定/日志 PII 扫描/注入检测/分歧检测四项。消费者: nt_io_provider + nt_shield_audit。
2. **来源可信度工程** (hyperresearch, N2+N3+N4): `nt_memory_untrusted_fence` (外部内容 data-not-instructions 围栏) + `nt_memory_quality_rank` (质量×引用权威×图中心度排序) + `nt_core_self::multi_signal_eval` (converge_check 产出物级验证)。
3. **实验设计纪律** (PI-blog + awesome-autoresearch, Reinforce nt_mind_*): seed 梯度付费 (1→3→8) + frozen verifier + re-ablate-on-change + keep-or-revert + GOAL.md fitness-first, 全部注入既有 nt_mind_guard/eval_harness/evolution_loop/act_autonomy。
4. **约束即选择** (text-alb + neural-txt, N5 + Reinforce output_governor): `constrained_selection` (self_curriculum 防幻觉选择) + constrained decoding (结构化输出强制)。
5. **影子评测** (2607.27191, N11): `nt_mind_eval_shadow_review` — 原作者评审通道接入吸收质量回评。

### Phase 1 — 记忆与推理增强 (5 项)
6. **提取模板库 + 推理轨迹分离** (paper_instructions_300K, N6+N26): 经验蒸馏受控指令模板 + reasoning-trace 分离存储。
7. **断点续跑 + 嵌入绑定** (hyperresearch + memvid, N7+N8): experience-tree 断点 manifest + embedding 漂移防护。
8. **原语完备性 + 符号回归** (2603.21852, N9+N10): EML 单一算子 (VSA 统一) + 梯度符号回归恢复闭式规律。
9. **预测式缓存升级** (OasisKV, N12 + Reinforce ResponseCache): lookahead 预取 + 分层 staging + 后台管线, 把 gateway.rs:370 反应式 prefetch 升级为预测式。
10. **凭据/执行分离** (pi Gondolin, N13): `host_cred_exec_split` — 凭据留宿主 + 执行进沙箱, 对齐 Egress Policy。

### Phase 2 — 声音/编排/浏览器 (5 项)
11. **声音注入层 + 文体注册表** (agency-agents + writing-skills, N14+N32): 防千篇一律 voice + 每文体可执行规程。
12. **持久实验区 + 反射式进化** (PI-blog + GEPA, N15+N16): 累积式实验驱动 + 自然语言反思进化。
13. **端到端评测 harness** (DSAgentBench, N17): 真实 OS 环境评测。
14. **时序快照 + 嵌套编排** (qlib, N18+N19): Point-in-Time + NestedExecutor。
15. **浏览器状态桥** (ego-lite, N33): 共享登录态浏览器自动化 (走 credential vault 授权, 审计)。

### Phase 3 — 记忆机制深度 + 备选 (其余 New)
16. **记忆机制深度** (magic-context + hyperresearch, N20-N25, N27-N29): 引文门/独立性/分类词表/降级渲染/cache 裁剪/repo 身份/胶囊/query gospel/威胁分类。
17. **IO 与协作备选** (N35-N38 + N30 + N31 + N34): 隧道/本地 NLP/rank 选择器/差分渲染/结构标签/swarm/调度器。

### ToS / 边界 (不接线, 仅机制参考)
- **ego-lite 共享登录态**: 高风险模式 (凭据共享), 必须走 nt_shield_credential_vault 授权 + 审计; 不直接裸用用户 cookies (N33 加授权链)。

---

## 4. 结论

- **能力网覆盖度**: 28 源全部落在既有 7 域 + 11 分支内, **68 Reinforce / 38 New 候选** — 与 seabatch 相比本批 New 比例显著升高 (深度能力缺口: 来源可信度/符号回归/预测式缓存/声音注入), 但每个 New 均具名消费者, R-P79 接线路径明确。
- **最深缺口域**: NT-MEMORY (来源质量/untrusted 围栏/引文门/断点/模板库 — 15 New), NT-MIND (符号回归/影子评测/持久实验区)。
- **最高安全信号**: 2608.09867 (加密 CoT 威胁模型) → 威胁驱动的 P0 防护优先于功能吸收。
- **跨源互证**: 反馈/实验信号层级化 (MIND 多源) 与 hyperresearch 来源可信度工程 (MEMORY) 是本批两个最内聚主题, 建议作为 P0 落地。
- **验证**: 五 artifact (wc -l + 条目数 + 引用抽查) + KB 入库 (21 insert/7 dup/0 fail) + 能力映射 2007/2007 已通过; 本文件为对标/路线总成。
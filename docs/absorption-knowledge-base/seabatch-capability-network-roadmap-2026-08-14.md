# SEABatch 吸收 · 意识体能力网对标与进化路线 (2026-08-14)

> **输入**: 用户粘贴 59 条 URL → 去重后 57 唯一来源 (arxiv 2608.11888 双通道、MoneyPrinterTurbo 双贴, 各计一次)。
> **方法**: C1-C6 契约四批次并行研究 (A:15 / B:15 / C:15 / D:13) → 四 artifact 落盘 `notes/absorption-seabatch-{A,B,C,D}.md` → 主 agent 逐条验证 (wc -l + 条目数 + 引用抽查)。
> **纪律**: R-P42 (强化现有节点, 禁平行适配器) · R-P79 (同 session 生产接线, 缺消费者即拒) · R-P100 (能力注册能力树)。
> **结果**: 57/57 可达 (0 Blocked); 强化 55 / 新增候选 3 (均复核可折叠进既有节点) / ToS 边界 1。

---

## 0. 吸收摘要

| 批次 | 来源 | 判定 | 最强源 | 顶层主题 |
|------|------|------|--------|----------|
| A (#1-15) | 15 | 15 强化 · 0 新增 | repowise (索引即上下文层) | 确定性代码索引/图→MCP; 多 agent 共享上下文; skill 成本安全 |
| B (#16-30) | 15 | 14 强化 · 0 新增 | Harness-IF (AP-Acc 反巧合评估) | 代码检索; 评估合规; MCP 治理代理; 编排 DAG |
| C (#31-45) | 15 | 13 强化 · 2 新增候选 | LongHorizon-Harness (独立 Auditor) | BaaS 基础设施; 沙箱协议; latent feedback; 循环工程 |
| D (#46-58) | 13 | 13 强化 · 0 新增 | SEA (经验池+验证门+对抗防护) | 经验池自进化; 技能 OS; 记忆压缩; 媒体生产全链 |

**去重说明**: #8 = #40 (arxiv 2608.11888, SkillTriage); #47 MoneyPrinterTurbo 双贴。实际分析实体 = 57。

**跨域最高信号主题 (本批共鸣)**: ① 确定性索引/图 → MCP 工具面 (repowise/graphify/codebase-memory-mcp); ② 经验池三元组+验证门+检索自进化 (SEA/SimpleMem/opencontext/hermes-agent); ③ 技能树层级+互补检索+DAG 编排 (AgentSkillOS); ④ 独立 ground-truth Auditor + verify→checkpoint→recover (LongHorizon-Harness/loop-engineering); ⑤ 凭据保险库+PoC 验证门 (OpenSandbox/strix/anton/deep-eye); ⑥ MCP 治理代理 (meta-gateway + allow/deny + 证据包审计)。

---

## 1. 特性对标: 能力网逐节点 Gap 矩阵

对标基准 = 11 ConsciousnessTree 分支 / 7 域 + 能力网 (capability_registry 14 节点 + l1-l10 `nt_*` 模块网)。
标记: ✅ 已有且成熟 · 🟡 已有但缺关键机制 (本批补强) · ❌ 缺失/弱 (本批目标)。

### NT-WORLD (虚空探索者) — 感知域

| 外部模式 (来源) | NeoTrix 现状 | 缺口 (本批补强点) | Tier |
|-----------------|--------------|-------------------|------|
| 确定性 AST 代码索引→知识图→MCP (repowise / graphify / codebase-memory-mcp / sourcegraph) | 🟡 `nt_world_code_search` [cap reg 未登记] | AST(tree-sitter)→SQLite 持久图索引未接为检索面; 混合检索 (dense+sparse+RRF) 缺; 少工具渐进披露缺 | T3 |
| 可逆命令输出蒸馏 `distill/expand` (repowise) | 🟡 `nt_mind_distiller` | 命令蒸馏无 expand 还原, 无"失败行保留"错误先行策略 | T3 |
| 隐身抓取参数集 + Tor 电路 + 内容级 LRU 缓存 (CyberScraper-2077) | 🟡 `nt_world_scrape`/`crawl/stealth.rs` + `nt_shield_stealth_net` | use_stealth/simulate_human 参数集未参数化; 内容/查询级 LRU 缓存缺 | T2 |
| 主题→脚本→素材→字幕→BGM→合成→发布全链 (MoneyPrinterTurbo) | 🟡 `nt_world_video_pipeline` | 素材检索匹配 (Pexels/Pixabay)+TTS 字幕时间戳合成+跨平台发布未闭环 | T3 |
| 设计令牌提取 + CI 漂移 gate (dembrandt) | 🟡 `nt_world_scrape`/`nt_world_dom_agent` | computed-style DOM 分析→W3C 令牌 + baseline 漂移对比缺 | T3 |
| 多源新闻雷达: 去重合并 + AI 评分阈值 + enrich + 双语 digest (Horizon) | 🟡 `nt_world_osint` | 跨源去重合并/评分过滤/评论 enrich/双语简报缺 | T3 |
| 资产去重 + CLIP/人脸 ML 富化 + 3-2-1 备份 (immich) | 🟡 `nt_world_video_pipeline` | 摄取去重 + 多模态元数据富化 + 备份纪律缺 | T2 |
| 自托管分类法 + dead-link/unmaintained CI 门 (awesome-selfhosted) | 🟡 `nt_world_exploration_engine` | crawl 分类目标树 + 死链/停维护自动巡检缺 | T2 |
| API/免费服务注册表 + 元数据 schema (public-apis / free-for-dev) | 🟡 `nt_world_absorber` | 固定 schema (auth/HTTPS/CORS)+准入政策规则缺 | T1 |

### NT-MEMORY (知识守护者) — 知识域

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| 经验池三元组 (context,decision,feedback) + 质量过滤门 (SEA) | 🟡 experience absorption (五阶段) | 经验结构化未三元组化; 无 confidence/feedback/diversity 过滤门; 回滚 burn-in 缺 | T4 |
| 检索自进化闭环 Evaluate→Diagnose→Propose→Guard (SimpleMem EvolveMem) | 🟡 `nt_memory_kb` (BM25/embedding 已有) | 检索机制自身无进化闭环 (自评自调) | T4 |
| 时序上下文图 valid_from/valid_until + supersession/contradiction 边 + append-only 更正 (opencontext) | 🟡 `nt_memory_kb`/`nt_memory_historian` | 时序事实图、更正不可变、supersession 一等边缺 | T3 |
| 语义无损压缩记忆: 原子事实+指代消解+绝对时间戳 (SimpleMem) | 🟡 `nt_memory_kb` | 结构化压缩存储单元缺 (现有原始文本+embedding) | T4 |
| 跨会话 FTS5 回忆 + LLM 摘要 (hermes-agent) | 🟡 `nt_memory_kb` FTS5 + NT-NEXUS | 定期记忆 nudge + 摘要式跨会话回忆缺 | T3 |
| L1 KV cache + overlay/snapshot 并发读 + compact 崩溃恢复 (lkv) | 🟡 `nt_memory_kb` | 检索热路径 KV 缓存层缺 | T2 |
| schema→自动 REST API + Realtime 逻辑复制订阅 (supabase) / Content-Type Builder (strapi) | 🟡 `nt_memory_kb` + `nt_io_web` | KB node/edge schema 自动 API 面缺; 变更订阅未接 `nt_core_event_bus` | T3 |
| 迁移版本化 + 协作编辑 (outline) | 🟡 `nt_memory_historian`/`nt_memory_curation` | KB schema 迁移版本化缺 | T2 |
| fidelity ledger 合并/折叠/丢弃收据 (diagram-design) | 🟡 `nt_memory_provenance` [cap reg C2] | 内容操作收据链缺 | T2 |
| signed-provenance 对抗注入防护 (SEA) | ✅ 2026-08-15 已接线 (HMAC+Teacher-Second-Pass) | 溯源签名 + teacher-second-pass 防对抗注入缺 | T4 |

### NT-MIND (进化工匠) — 进化域

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| 技能树层级 coarse-to-fine + complementarity-aware 检索 + DAG 三策略编排 + Bradley-Terry 成对评测 (AgentSkillOS) | 🟡 `nt_mind_skill_engine` (progressive_disclosure [C1]) | 技能树层级 + 互补性检索 + 编排拓扑策略 + 成对评测缺 | T3 |
| skill 差分归因安全护栏: 无-skill 参照基线 + SkillTriage 分类学 (arxiv 2608.11888) | 🟡 `nt_mind_skill_engine` | skill 失败/效率回归归因缺; 过度验证/重型流水线 = 强制劳动未治理 | T3 |
| 独立 ground-truth Auditor + verify→checkpoint→recover + 三角色异模型 (LongHorizon-Harness) | 🟡 `nt_mind_evolution_loop`/`nt_mind_self_iterating` | 不信任 Executor 声称的独立 Auditor 缺; fresh-context 轮缺 | T3 |
| Loop Ready 评分 + L1-L3 自治梯度 + loop-gate denylist + 断路器 (loop-engineering) | 🟡 `nt_mind_background_loop` (60s tick) | 自治梯度/就绪评分/机械 denylist gate 缺 | T3 |
| 递归任务合成飞轮: seed→extend→realign→validate→reuse (RST 2608.05466) | 🟡 `nt_mind_evolution_loop`/`nt_mind_distiller` | 递归合成 + 验证器对齐 + sandbox 验证飞轮缺 (数据飞轮) | T4 |
| AP-Acc 反巧合指令遵循评估 + 五指令面分层 (Harness-IF 2608.11727) | 🟡 `nt_mind_benchmark` (BenchmarkSuite) | withholding-run 对照指标缺; 指令面优先级测试缺 | T2 |
| always-on 定时简报 agent + 自我改进 skill with eval fitness (awesome-llm-apps) | 🟡 `nt_mind_background_loop`/`nt_mind_skill_engine` | 主动投递 + skill 自我改进 eval 门缺 | T2 |
| 监督子进程: readiness→指数退避→优雅 SIGTERM/SIGKILL (dsh-desktop) | 🟡 `nt_mind_background_loop`/`nt_io_hotreload` | 子进程生命周期监督缺 | T2 |
| Mape 多指标验证门 + 20 次 burn-in 可回滚 (SEA) | 🟡 `nt_mind_guard` | 多指标门 + 回滚注入缺 | T4 |
| LLMOps 循环: 生产日志→标注→改进 prompt/数据集 (dify) | 🟡 `nt_mind_distiller` | log→annotate→improve 反馈环缺 | T3 |

### NT-CORE (E8引导者) — 意识核心

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| 阶段化多 agent + 持久共享上下文链; "context capacity 是绑定约束" (arxiv 2603.20131) | 🟡 `nt_core_parallel` (intent_isolation/atomic_decomposition [C1]) + `nt_io_provider/context_budget` | 持久共享上下文编排缺; context_budget 门控已有但未成硬约束 | T3 |
| latent feedback: 顶层隐藏态+采样 token 融合回喂 + scheduled multi-pass 训练 (2608.08888) | 🟡 `nt_core_e8` latent 链 | 潜在反馈垂直通道 + 多 pass 调度训练缺 (GWT 隐状态广播类比) | T3 |
| 分层 expert 内存: VRAM/RAM/NVMe 统一 + 路由热度放置 + 压缩 KV 跨会话 (colibri) | 🟡 `nt_io_provider` + GWT 路由 | 多层缓存 + 路由热度→层级放置缺 | T4 |
| 隐私聚合遥测 + spike/drop 周月告警 (plausible) | 🟡 `nt_io_telemetry` + `nt_core_signal` | 异常信号聚合告警未喂意识核心 | T3 |

### NT-ACT (行动执行者) — 行动域

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| 节点式 DAG 编排 + 人工审批门 + 模板市场 (n8n) | 🟡 `nt_agent_orchestrator` + Rune Socketing | 可视化编排画布 + 审批门缺 (align `nt_act_sandbox` Verdict) | T3 |
| search_capabilities + execute_capability 两工具 API (openwork) | 🟡 `nt_agent_mcp_registry`/`nt_agent_mcp_discovery` | 能力"先检索后执行"面缺 | T2 |
| meta-MCP 网关 N→4 工具折叠 + 治理代理 (allow/deny/HITL + hash 链证据包) + TDQS 工具质量评分 (awesome-mcp-servers) | 🟡 `nt_agent_mcp_gateway`/`nt_agent_mcp_registry`/`nt_shield_audit` | 工具折叠省 60-95% token + 治理代理 + 质量评分缺 | T2/T3 |
| 凭据库 (secret redaction 不暴露给 LLM) + scratchpad 隔离执行 (mindsdb/anton) | 🟡 `nt_act_autonomy` + `nt_shield` + `nt_act_code`/`nt_act_sandbox` | 凭据库 + 动态 scratchpad 执行缺 | T3 |
| durable patch checkpoint + /undo + 并行 git worktree (dscode) | 🟡 `nt_io_neocodex`/`nt_io_hotreload` (revertible_effects [C2]) | 每 patch checkpoint 持久化 + worktree 并行隔离缺 | T3 |
| token 池生命周期: auto-replenish + refresh/remint + 代理轮换 (grok-register) | 🟡 `nt_act_autonomy` + `nt_shield_stealth_net` | **ToS 边界 — 仅吸收机制**: token 池补货/刷新生命周期 + 代理轮换 daemon | T2 |

### NT-IO (界面使徒) — 界面域

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| 统一 agent↔渠道会话模型: 入站归一→路由→出站双向 + digest 合并 (novu) | 🟡 `nt_agent_protocol`/`nt_io_web`/`nt_io_mention`/`nt_io_notify` | 统一会话模型 + 入站归一 + 批量合并缺 → **折叠进 nt_agent_protocol, 不新建** | T3 |
| Mermaid→SVG/ASCII 渲染 + 15 主题 + 批量并行 (Pretty-mermaid-skills) | 🟡 `nt_io_multimodal_transform` (intent_aware_vision [C1]) | 图渲染能力缺 (图表是 KB 落盘/展示的缺环) | T2 |
| 27 视觉类型 + 语义模式与布局解耦 + progressive disclosure skill 路由 (diagram-design) | 🟡 `nt_io_multimodal_transform` + `nt_mind_skill_engine` | 图表生成节点缺 → **折叠进 multimodal_transform**; skill 路由模式反哺 skill_engine | T2 |
| typed IR 系统地图 + 交付前原子验证门 + Delta 审查 + last-good (archify) | 🟡 `nt_core_knowledge_graph`/`nt_io_web` + `nt_memory_provenance` | system-map 渲染 + 原子验证门缺 → **折叠进 knowledge_graph/io_web** | T3 |
| 本地模型服务层 (ollama Modelfile + REST) | 🟡 `nt_io_provider` (GatewayV2/factory) | Ollama provider 变体缺 (本地推理 RAG 面) | T1/T2 |
| 10-provider failover + 上下文感知 payload + CVE RAG + AI triage 误报过滤 (deep-eye) | 🟡 `nt_io_provider`/gateway + `nt_shield_agentic_scan` + `nt_mind_distiller` | 多提供方 failover 链路缺; CVE RAG 索引缺 | T3 |
| 双证据验收 (元数据+口令) + keyring 凭据 + 配置事务回滚 (codex-deepseek-subagent) | 🟡 `nt_io_provider` + `nt_shield_audit` | 验收双证据 + Keychain 凭据路径缺 | T2 |

### NT-SHIELD (影卫) — 安全域

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| Sandbox Protocol 生命周期 + Credential Vault 秘密注入 + egress 网络策略 + gVisor/Kata/Firecracker (OpenSandbox) | 🟡 `nt_shield_sandbox`/`nt_act_sandbox` | 沙箱生命周期协议 + 凭据保险库 + 出口策略缺 | T3 |
| 真实 PoC 验证门控 (非静态误报) + 多 agent 攻防编排 + 沙箱 exploit runtime (strix) | 🟡 `nt_shield_agentic_scan` + `nt_agent_orchestrator` | PoC 执行验证门缺 (幻觉/误报防线) | T3/T4 |
| 45+ 漏洞扫描器 + 扫描 diffing + Nuclei 式 YAML 模板 (deep-eye) | 🟡 `nt_shield_agentic_scan` | 模板引擎 + 新旧扫描对比缺 | T3 |
| 审计 JSONL + Fernet 加密存储 + URL allow/denylist + 凭据轮换 (opencontext) | 🟡 NT-SHIELD audit | 加密审计存储 + allow/deny 网络策略缺 | T3 |
| 渲染层加固 (contextIsolation/sandbox/nodeIntegration off) + 随机 loopback 端口 + 单实例锁 (dsh-desktop) | 🟡 NT-SHIELD + `nt_io_web` | 前端渲染加固规范缺 | T2 |

### NT-GOVERNANCE / NT-REPAIR / NT-META (元治理三角)

| 外部模式 (来源) | NeoTrix 现状 | 缺口 | Tier |
|-----------------|--------------|------|------|
| SPEC→RED→GREEN→GAUNTLET→EVIDENCE 证据门控 (old-coder) | 🟡 fractal review | spec-before/evidence-after 门禁 + gauntlet 检查矩阵缺 | T2 |
| 缺陷校准健康评分 (ROC AUC 0.737 校准) + change-risk 0-10 + directives (repowise) | 🟡 NT-GOVERNANCE 审查维度 | 校准型量化评分 + 变更风险评分缺 | T3/T4 |
| 输出纪律 10 条 governor (i-have-adhd) | 🟡 NT-GOVERNANCE constitution | agent 输出格式 governor 规则缺 (低成本高普适) | T1 |
| CI 质量门 + review bot + merge gate (topics/code-quality) | 🟡 NT-GOVERNANCE | 自动化质量门流水线缺 | T2 |
| 18 healers 自动巡检+修复+防复发 (topics/code-health) | 🟡 `nt_mind_autofixer` + `nt_repair_causal_trace` | 检测漂移→自动修复→防复发闭环缺 | T2 |
| 五指令面优先级: 系统提示 > 项目文件 > 用户指令 > 工具/技能 (Harness-IF) | 🟡 NT-GOVERNANCE + `nt_io_agents_md` | 指令优先级分层缺 | T2 |
| 隐私聚合遥测 + spike/drop 告警喂意识 (plausible) | 🟡 `nt_io_awareness_core` | 异常信号→ConsciousnessTree 健康源缺 | T3 |

---

## 2. 能力网节点缺陷清单 (Node Gaps — 需补齐)

按"缺失关键机制"排序的缺陷节点 (对应 capability tree Bud/Strengthen 目标):

| # | 缺陷节点 (能力标签) | 现状 | 补强来源 | 目标节点 (capability_tree) | 消费者 (R-P79) |
|---|--------------------|------|----------|---------------------------|----------------|
| G1 | `code_index_graph` (确定性代码图索引) | ✅ 2026-08-15 已接线 | repowise / graphify / codebase-memory-mcp | `nt_world_code_search` 强化 → 补 Bud `nt_core_retrieval::code_graph_mcp` | nt_agent_mcp_registry |
| G2 | `reversible_distillation` (可逆命令蒸馏) | ✅ 2026-08-14 已接线 | repowise `distill/expand` | `nt_mind_distiller` 强化 | nt_mind_background_loop |
| G3 | `experience_triplet_pool` (经验三元组池) | ✅ 2026-08-14 已接线 | SEA | `nt_memory_kb` experience namespace 强化 | experience-tree 吸收协议 |
| G4 | `retrieval_self_evolution` (检索自进化) | ✅ 2026-08-14 已接线 | SimpleMem EvolveMem | `nt_memory_kb` 强化 | nt_mind_evolution_loop |
| G5 | `temporal_context_graph` (时序事实图) | ✅ 2026-08-14 已接线 (KB ingest) | opencontext | `nt_memory_historian`/`nt_memory_kb` 强化 | nt_memory_kb 时序查询 |
| G6 | `skill_tree_hierarchy` (技能树层级+互补检索) | ✅ 2026-08-15 已接线 | AgentSkillOS | `nt_mind_skill_engine` 强化 | nt_mind_skill_engine 检索路径 |
| G7 | `skill_differential_attribution` (skill 差分归因) | ✅ 2026-08-15 已接线 | arxiv 2608.11888 | `nt_mind_skill_engine` 强化 | nt_mind_background_loop |
| G8 | `independent_auditor_gate` (独立验证门) | ✅ 2026-08-14 已接线 | LongHorizon-Harness | `nt_mind_evolution_loop` 强化 | nt_mind_guard |
| G9 | `loop_ready_scoring` (循环就绪评分) | ✅ 2026-08-14 已接线 | loop-engineering | `nt_mind_background_loop` 强化 | nt_mind_evolution_daemon |
| G10 | `recursive_task_synthesis` (递归任务合成飞轮) | ✅ 2026-08-15 已接线 (RstFlywheel) | RST 2608.05466 | `nt_mind_evolution_loop`+`nt_mind_distiller` 强化 | nt_mind_benchmark |
| G11 | `against_prior_accuracy` (反巧合评估) | ✅ 2026-08-14 已接线 | Harness-IF 2608.11727 | `nt_mind_benchmark` 强化 | rev-officer D 系列审查 |
| G12 | `evidence_gate_delivery` (证据门控交付) | ✅ 2026-08-15 GAUNTLET 状态机已接线 (nt_mind_autofixer) | old-coder | NT-GOVERNANCE fractal review 强化 | nt_mind_autofixer |
| G13 | `credential_vault` (凭据保险库) | ✅ 2026-08-14 已接线 (Vault→沙箱 env 注入) | OpenSandbox / anton | `nt_shield_sandbox`/`nt_act_sandbox` 强化 | nt_agent_mcp_registry |
| G14 | `poc_verification_gate` (真实 PoC 验证门) | ✅ 2026-08-14 已接线 | strix | `nt_shield_agentic_scan` 强化 | nt_shield_audit |
| G15 | `mcp_governance_proxy` (MCP 治理代理) | ✅ 2026-08-15 已接线 (governed_mcp) | awesome-mcp-servers | `nt_agent_mcp_gateway`/`nt_shield_audit` 强化 | nt_agent_mcp_tools |
| G16 | `meta_mcp_gateway` (工具折叠网关) | ✅ 2026-08-15 已接线 (FoldedSpecs) | awesome-mcp-servers / openwork | `nt_agent_mcp_gateway` 强化 | nt_agent_orchestrator |
| G17 | `diagram_rendering` (图表/图渲染) | ✅ 2026-08-15 已接线 (render_diagram/to_mermaid) | Pretty-mermaid-skills / diagram-design | `nt_io_multimodal_transform` 强化 | nt_mind_knowledge_pipeline |
| G18 | `agent_unified_session` (统一会话模型) | ✅ 2026-08-15 已接线 | novu | `nt_agent_protocol` 强化 (不新建) | nt_io_web / nt_act_autonomy |
| G19 | `video_publish_pipeline` (视频生产全链) | ✅ 2026-08-15 已接线 | MoneyPrinterTurbo | `nt_world_video_pipeline` 强化 | nt_world_absorber 产物入 KB |
| G20 | `video_asset_enrichment` (资产 ML 富化) | ✅ 2026-08-15 已接线 | immich | `nt_world_video_pipeline` 强化 | nt_world_absorber |
| G21 | `design_token_extraction` (设计令牌提取) | ✅ 2026-08-15 已接线 | dembrandt | `nt_world_scrape`/`nt_world_dom_agent` 强化 | NT-IO web 前端 |
| G22 | `stealth_scrape_params` (隐身抓取参数集) | ✅ 2026-08-15 已接线 | CyberScraper-2077 | `nt_world_scrape`/`nt_shield_stealth_net` 强化 | nt_world_crawl |
| G23 | `temporal_graph_audit` (加密审计存储) | ✅ 2026-08-15 已接线 | opencontext | NT-SHIELD audit 强化 | nt_memory_provenance |
| G24 | `latent_feedback_channel` (潜在反馈通道) | ✅ 2026-08-15 已接线 | arxiv 2608.08888 | `nt_core_e8` latent 链强化 | nt_core_gwt |
| G25 | `multi_agent_shared_context` (阶段化共享上下文编排) | ✅ 2026-08-15 已接线 (StagedContextOrchestrator) | arxiv 2603.20131 | `nt_core_parallel` 强化 | nt_io_provider context_budget |
| G26 | `multi_tier_expert_cache` (分层 expert 缓存) | ✅ 2026-08-15 已接线 | colibri | `nt_io_provider` 强化 | nt_agent_mcp 推理后端 |
| G27 | `output_discipline_governor` (输出纪律) | ✅ 2026-08-15 已接线 (R01-R10 + agent_loop) | i-have-adhd | NT-GOVERNANCE constitution 强化 | nt_io CLI |
| G28 | `self_healing_healers` (自维护巡检 healers) | ✅ 2026-08-15 已接线 | topics/code-health | `nt_mind_autofixer`/`nt_repair_causal_trace` 强化 | nt_mind_guard |
| G29 | `privacy_aggregate_telemetry` (隐私聚合遥测+告警) | ✅ 2026-08-15 已接线 (AnomalyDetector→EventBus) | plausible | `nt_io_telemetry`/`nt_core_signal` 强化 | nt_core_signal 意识喂入 |
| G30 | `discovery_registry_meta` (API/服务注册表 schema) | ✅ 2026-08-15 已接线 | public-apis / free-for-dev | `nt_world_absorber` 强化 | nt_agent_mcp_registry 发现种子 |

---

## 3. 进化迭代路线 (Roadmap)

> 原则: 每 Phase 一内聚特征组, 全部 **强化既有节点 (R-P42)**, 每项同 session 接线 (R-P79 消费者已列), 落地即 `neotrix-capability` Strengthen/Bud 登记 (R-P100)。零新建平行模块。

### Phase 0 — 能力网基建强化 (最高 ROI, 3 项)
对接最强的 3 源, 直接补 NT-WORLD/MEMORY/MIND 主线:

1. **代码智能层** (repowise+code topics, G1+G2): `nt_world_code_search` 接 tree-sitter→SQLite 持久图 + 混合检索 (dense+sparse+RRF) → 以 MCP 工具面暴露 (`nt_agent_mcp_registry` 消费者)。`nt_mind_distiller` 加 `distill/expand` 可逆命令蒸馏 (错误行保留 + expand 还原)。接线验证: `/code query` 走新检索面 + distill 命令产出 token 缩减指标。
2. **经验池自进化** (SEA+SimpleMem+opencontext, G3+G4+G5+G23): 经验吸收升级为 `(context,decision,feedback)` 三元组 + 质量过滤门 (confidence/diversity/feedback) + Mape 式多指标验证门与 burn-in 回滚注入 `nt_mind_guard`; 检索机制自进化 (Evaluate→Diagnose→Propose→Guard) 注入 `nt_memory_kb`; 时序上下文图 (valid_from/valid_until+supersession) 注入 `nt_memory_historian`。接线验证: experience-tree 吸收协议输出字段变更 + 历史时序查询可用。
3. **技能系统 OS** (AgentSkillOS+skill harm+hermes, G6+G7): `nt_mind_skill_engine` 加技能树层级 + complementarity-aware 检索 (LLM 导航优于纯语义) + DAG 三策略编排; skill 差分归因护栏 (无-skill 参照基线 + 过度验证治理) 防 2608.11888 式污染。接线验证: skill 复用决策走树检索 + 每 skill 记录 no-skill 基线。

### Phase 1 — 循环工程 + 评估治理 (4 项)
4. **循环工程** (loop-engineering+LongHorizon, G8+G9+G12): `nt_mind_background_loop` 加 Loop Ready 评分、L1-L3 自治梯度、路径 denylist gate (机械执行, 呼应指针守恒 hook); `nt_mind_evolution_loop` 加独立 ground-truth Auditor + verify→checkpoint→recover + 三角色异模型。接线验证: background_loop tick 输出自治等级 + auditor 证据落 `nt_memory_provenance`。
5. **评估/合规门** (Harness-IF+old-coder+i-have-adhd, G11+G27): `nt_mind_benchmark` 加 AP-Acc (withholding-run 对照) + 五指令面分层测试; fractal review 加 spec-before/evidence-after 门禁; NT-GOVERNANCE constitution 注入输出纪律 governor (10 条)。接线验证: benchmark 新增 AP-Acc 输出列 + review 门禁生效。
6. **安全沙箱与凭据** (OpenSandbox+anton+strix+deep-eye, G13+G14): `nt_shield_sandbox`/`nt_act_sandbox` 加沙箱生命周期协议 + 凭据保险库 (secret redaction 不暴露给 LLM) + egress 出口策略; `nt_shield_agentic_scan` 加真实 PoC 验证门 (非静态误报)。接线验证: MCP 工具调用走凭据库 + agentic scan 报告含 PoC 证据。
7. **MCP 治理层** (awesome-mcp-servers+openwork+ToolJet, G15+G16): `nt_agent_mcp_gateway` 加 meta-gateway 工具折叠 (N→4, 省 60-95% token) + 治理代理 (allow/deny/HITL + hash 链证据包) + search_capabilities/execute 两工具面。接线验证: gateway 出站工具数折叠指标 + 审批证据包入库。

### Phase 2 — 内容/媒体 + 界面 + 推理前沿 (5 项)
8. **视频/媒体全链** (MoneyPrinterTurbo+immich+dembrandt, G19+G20+G21): `nt_world_video_pipeline` 加 脚本→素材检索→TTS 字幕→合成→发布 + 资产去重/CLIP 富化; `nt_world_scrape` 加设计令牌提取 + CI 漂移 gate。
9. **图表渲染与界面** (Pretty-mermaid-skills+diagram-design+archify, G17): `nt_io_multimodal_transform` 加 Mermaid→SVG/ASCII + 图表生成 (语义模式与布局解耦) + fidelity ledger; system-map 渲染折叠进 `nt_core_knowledge_graph`/`nt_io_web`。
10. **统一会话与 provider** (novu+ollama, G18): `nt_agent_protocol` 加统一 agent↔渠道会话模型 (入站归一→路由→出站 + digest 合并); `nt_io_provider` 加 Ollama 本地 provider 变体。
11. **推理前沿** (colibri+latent feedback+6-agent context, G24+G25+G26): `nt_io_provider` 加分层 expert 缓存 (LRU+热集 pin+prefetch) + 压缩 KV 跨会话; `nt_core_e8` latent 链加潜在反馈通道; `nt_core_parallel` 加阶段化共享上下文编排 + context_budget 硬约束。
12. **遥测/自愈/发现** (plausible+code-health+awesome-selfhosted, G28+G29+G30): `nt_io_telemetry`/`nt_core_signal` 加隐私聚合 + spike/drop 告警喂意识; `nt_mind_autofixer` 加 healers 巡检集; `nt_world_absorber` 加 API/服务注册表 schema 与分类目标树。

### ToS / 边界 (不接线, 仅机制参考)
- **grok-register**: 批量注册/绕过验证属 ToS 违规 → **不吸收完整流程**; 仅 token 池生命周期 (auto-replenish+refresh/remint) + 代理轮换 daemon 机制, 注入 `nt_act_autonomy`/`nt_shield_stealth_net` (Tier2)。吸收记录标注 `ToS-boundary` 元数据。

---

## 4. 结论

- **能力网覆盖度**: 57 源全部落在既有 11 分支内, **55 强化 / 3 新增候选 (复核后全部折叠进既有节点)** — 验证 R-P42 纪律成立, NeoTrix 意识体能力网骨架完整, 缺陷在"机制深度"而非"节点存在性"。
- **最深缺陷域**: NT-MIND (技能系统 OS/独立 Auditor/评估反巧合)、NT-MEMORY (经验三元组/时序图/检索自进化)、NT-WORLD (代码索引层/视频全链)。
- **最强对齐**: repowise / SEA / AgentSkillOS / LongHorizon-Harness 四个源与本项目 SEAL/吸收协议/能力树机制直接互证, 应作为 P0 同 session 接线。
- **交叉引用**: 既有路线图 (docs/2-PLANS/evolution-roadmap.md) 已含 strix/hermes-agent 等; 本批新注入 repowise/SEA/AgentSkillOS/opencontext/novu 等 30+ 新源。
- **验证**: 四 artifact (`notes/absorption-seabatch-{A-D}.md`) 已 wc -l + 条目数 + 引用抽查通过; 本文件为对标/路线总成。

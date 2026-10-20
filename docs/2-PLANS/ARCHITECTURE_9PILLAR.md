# NeoTrix 9-Pillar Architecture v4

> **Date**: 2026-06-30 | 从7域→9支柱重构，吸收32个盲点 + 外部150+ repos/papers 分析

---

## 总览：从7域到9支柱

原有7领域模型缺少两个横切关注点，导致模块错位：

| 原7域 | 问题 | → 9支柱 |
|-------|------|---------|
| NT-CORE | 推理核, OK | **P1. NT-CORE** |
| NT-MIND | 膨胀113子模块, 含agent逻辑 | **P2. NT-MIND** (纯自我进化, 移出agent) |
| NT-MEMORY | 记忆, OK | **P3. NT-MEMORY** |
| NT-WORLD | 感知, OK | **P4. NT-WORLD** |
| NT-ACT | 行动, OK | **P5. NT-ACT** |
| NT-IO | 人机界面, 但LLM provider错位在此 | **P6. NT-IO** (纯界面, 移出provider) |
| NT-SHIELD | 安全, OK | **P7. NT-SHIELD** |
| — | agent逻辑散落 | **P8. NT-AGENT** (从NT-MIND/NT-ACT剥离) |
| — | LLM provider错位在NT-IO | **P9. NT-PROVIDER** (从NT-IO剥离) |

---

## 9支柱模块映射

### P1: NT-CORE（推理核）

角色：零外部依赖的理论核心层。E8+GWT+IIT+VSA+SAE+元认知+意识。

```
nt_core_e8          → 248维E8数学, 240根向量, 64卦, 费米子生成
nt_core_hex         → 64态推理机, 6轴(抽象/范围/方法/深度/模式/立场)
nt_core_policy      → E8策略: GRPO组采样 + 推理时Beam/MCTS搜索
nt_core_observer    → +1观察者 + PRM头(64→32→1) + 错误恢复三层栈
nt_core_hcube       → 4096维VSA MAP超立方体 (→FHRR D=2048迁移)
nt_core_gwt         → GWT全局工作空间: 竞争点火 + 5层压缩 + 谐振器网络
nt_core_meta        → 元认知: 扫描/分析/监控/规划/报告
nt_core_self        → 硅基自我模型: attention_schema + HOT高阶思维
nt_core_consciousness → 意识模块: 20文件(volition/valence/awakening等)
nt_core_bank        → 推理银行: L1/L2/L3三级记忆
nt_core_ssm         → Mamba-2 SSD (N=256)
nt_core_sae         → 稀疏自编码器 (TopK, 256潜变量)
nt_core_jepa        → JEPA世界模型 (ViT骨干 + 动作条件预测器)
nt_core_abstr       → 对比抽象 (Hopfield能量聚类)
nt_core_cdwm        → 因果解缠世界模型
nt_core_cap         → 能力向量 (23维)
nt_core_td          → 时序差分学习
nt_core_walsh       → Walsh-Hadamard记忆索引
nt_core_kron        → Kronecker结构旋转
nt_core_router      → SmartRouter (任务复杂度路由)
nt_core_arch        → ArchitectAgent (agent/designer/implementer/verifier)
nt_core_epoch       → 地史纪元 (E1-E8)
nt_core_fep_iit     → FEP + IIT桥 (主动推理修复)
nt_core_iit_phi     → 集成信息φ计算 (因果效应谱系修复)
```

**关键改造**:
- PRM头注入observer + GRPO替换epsilon-greedy
- GWT竞争点火 (winner-take-all取代累加广播)
- 5层上下文压缩管线
- E8嵌入VSA超向量 (消除E8-GWT梯度壁垒)
- Mamba-1→Mamba-2 SSD升级

---

### P2: NT-MIND（自我进化）

角色：SEAL自迭代管线 + 技能优化 + 衰老诊断。精简为纯进化域。

```
nt_mind_seal        → SEAL 28~34阶段管线 (新增DPO/Constitution/ProceduralMemory/Safety)
nt_mind_brain       → NeoTrix思维核心
nt_mind_skill       → 技能优化 (BoundedEdit/ValidationGate/EpochSlowUpdate)
nt_mind_strat       → 自我编辑策略 (Conservative/Aggressive/DGM)
nt_mind_adapt       → 环境适配器 (HarnessAdapt/跨模型迁移)
nt_mind_age         → 衰老诊断 (4指标)
nt_mind_scan        → 秘密扫描 (13正则)
nt_mind_self_diagnose → 自诊断
nt_mind_benchmark   → 基准测试
nt_mind_dgm         → DGM扩散生成式编辑
nt_mind_hmeta       → 超元智能体
nt_mind_sleep       → 离线记忆巩固
nt_mind_distiller   → 蒸馏
nt_mind_background_loop → 24/7后台循环
nt_mind_evolution_loop  → 进化循环
nt_mind_evolution_daemon → 进化守护进程
nt_mind_consciousness_gold_standard → 意识金标准
nt_mind_consciousness_monitor  → 意识监控
```

**关键改造**:
- 移出agent/知识/记忆/工具到对应域
- 新增DPOStage: GWT广播轨迹→(accepted, rejected)对→DPO更新
- 新增ConstitutionalSelfCritiqueStage: 基于宪法的生成→批判→修订
- 新增SafetyCheckStage: Plan→Check→Act/Refuse (MOSAIC)
- 新增ProceduralMemoryStage: 成功E8序列→KB可重用技能

---

### P3: NT-MEMORY（持久记忆）

角色：SQLite知识库 + FTS5 + 嵌入 + 三层记忆 (Episodic/Semantic/Procedural)。

```
nt_memory_kb        → SQLite知识库 (22节点类型/19关系)
  ├ nt_memory_types.rs   → 类型定义 + ConversationRecord/EvolutionRecord
  ├ nt_memory_schema.rs  → DDL定义
  ├ nt_memory_store.rs   → CRUD + 去重
  ├ nt_memory_search.rs  → FTS5 + BM25 + 嵌入混合搜索
  ├ nt_memory_graph.rs   → BFS + 子图查询
  ├ nt_memory_embed.rs   → 嵌入API (OpenAI兼容)
  ├ nt_memory_crawl.rs   → 知识爬虫 (Wiki/ArXiv/GitHub)
  ├ nt_memory_seed.rs    → 88种子节点
  ├ nt_memory_ingest.rs  → 知识摄取器
  ├ nt_memory_api.rs     → HTTP API
  ├ nt_memory_gwtq.rs    → E8/GWT查询
  └ nt_memory_integration.rs → WebMiner桥
nt_memory_cortex    → 皮层记忆 (维度标记/多模态)
nt_memory_knowledge_populator → 知识填充
```

**关键改造**:
- 新增ProceduralMemory表: (skill_id, e8_sequence, trigger_conditions, success_rate)
- 隐私架构: stateless mode + 数据主权边界
- Episodic→Semantic consolidation管线

---

### P4: NT-WORLD（感知交互）

角色：连接到物理/数字世界的感知层。浏览器/爬虫/JEPA世界模型。

```
nt_world_jepa       → JEPA世界模型 (ViT编码器+掩码策略+动作条件预测器)
nt_world_e8         → E8隐式世界模型 (Hadamard变换/时间演化/河图洛书)
nt_world_infer      → 主动推理 (自由能原理/预期自由能最小化)
nt_world_pred       → 超立方体预测
nt_world_browse     → 浏览器自动化 (反检测/拟人化)
nt_world_browse_auto → 自动浏览器
nt_world_crawl      → 爬虫 (自适应/分类/隐身/富集)
nt_world_scrape     → 网页抓取
nt_world_search     → 网络搜索
nt_world_sense      → 感官输入 (听觉/视觉皮层)
nt_world_vision     → 视觉处理
nt_world_code_search → 代码搜索
```

**关键改造**:
- JEPA重构: ViT骨干 + 多块掩码 + 动作条件预测器 (=E8推理模拟器)
- 主动推理循环: 精度(precision) + 预期自由能 → 策略选择

---

### P5: NT-ACT（行动工具）

角色：对物理/数字世界的操作能力。加密/社交/代码/目标/编排。

```
nt_act_crypto       → 加密金融引擎 (钱包/DEX/桥/收益/Gas)
nt_act_earn         → 收益引擎
nt_act_sync         → 文件同步
nt_act_social       → 社交连接器 (Twitter/Reddit/YT/TikTok/Bilibili/Douyin/IG)
nt_act_spear        → SPEAR协议
nt_act_gram         → NeoGram消息
nt_act_code         → 自代码生成
nt_act_goal         → 自我目标管理
nt_act_autonomy     → 自主决策
nt_act_voice        → 语音交互
nt_act_orchestrator → 编排器 (Planner/Worker/Critic + 对抗)
nt_act_project_manager → 项目管理
nt_act_sub_agent_middleware → 子代理中间件
```

**关键改造**:
- 移除 `nt_act_mcp.rs` (精确重复 `nt_agent_mcp_discovery.rs` → 删除)
- Planner/Executor/Reflector角色分离

---

### P6: NT-IO（人机界面）

角色：与人类交互的接口。CLI + TUI + Server + Tauri + 通知。

```
nt_io_cli           → CLI (27命令, clap)
nt_io_tui           → TUI (Ratatui, 11文件)
nt_io_server        → HTTP/WS/WebRTC服务器
nt_io_proxy         → 代理守护进程
nt_io_web           → Web UI
nt_io_notify        → 通知系统
nt_io_lsp           → LSP客户端
nt_io_hotreload     → 热重载
nt_io_plugin        → 插件系统
nt_io_remote        → 远程连接
nt_io_logging       → 日志
nt_io_mention       → 提及解析
nt_io_user_avatar   → 用户头像
nt_io_avatar_channel → 头像通道
nt_io_push_channel  → 推送通道
nt_io_standalone    → 独立模式
nt_io_telemetry     → 遥测 (feature-gated)
```

**关键改造**:
- 移出nt_io_provider到P9
- 纯界面层, 不包含业务逻辑

---

### P7: NT-SHIELD（安全防护）

角色：安全边界。金库/权限/护栏/提示注入/沙箱。

```
nt_shield_vault     → 密钥保险库
nt_shield_perm      → 权限系统 (模式链: plan/acceptEdits/bypassPermissions)
nt_shield_rails     → 护栏系统
nt_shield_prompt    → 提示注入防护
nt_shield_sandbox   → 沙箱执行 (Wasm/Docker)
nt_shield_audit     → 安全审计
nt_shield_sentry    → Sentry集成
nt_shield_manager   → 安全管理器
nt_shield_stealth_net → 隐身网络 (feature-gated)
```

**关键改造**:
- 模式链: plan(只读)→acceptEdits(自动批准)→bypassPermissions(全自动)
- 推测性分类: 工具执行时并行运行允许分类器
- CVSS漏洞评分

---

### P8: NT-AGENT（代理运行时）⭐新增支柱

角色：MCP注册表/子代理/工作流/团队协调。从NT-MIND膨胀体中剥离。

```
nt_agent_mcp_discovery  → PATH扫描 + JSON-RPC验证
nt_agent_mcp_tools      → 内置MCP工具
nt_agent_orchestrator   → 代理编排器 (Planner/Worker/Critic)
nt_agent_subagent       → 子代理执行
nt_agent_protocol       → 代理协议 (TCP+UDP发现)
nt_agent_team           → 多代理团队 (debate/hierarchy/sequential)
nt_agent_workflow       → 工作流引擎 (DAG + 状态图)
nt_agent_skills         → 技能引擎
nt_agent_tools          → MCP Registry (4传输, 缓存, 健康检查)
```

**关键改造**:
- 从agent/目录迁移到neotrix/nt_agent/域
- MCP 4传输→2传输(Stdio+Streamable HTTP)
- OAuth 2.1 + RFC 8707
- Planner/Executor/Reflector角色分离

---

### P9: NT-PROVIDER（LLM服务层）⭐新增支柱

角色：LLM提供商抽象/路由/压缩/发现/成本管理。从NT-IO剥离。

```
nt_provider_types       → 提供商类型/enums
nt_provider_factory     → 提供商工厂
nt_provider_openai      → OpenAI提供商
nt_provider_anthropic   → Anthropic提供商
nt_provider_gemini      → Gemini提供商
nt_provider_ollama      → Ollama提供商
nt_provider_compaction  → 上下文压缩
nt_provider_discovery   → 提供商发现/网格
nt_provider_routing     → 智能路由 (tiered/成本感知/fallback)
nt_provider_search_router → 搜索路由
nt_provider_catalog     → 免费模型目录
nt_provider_deploy      → 边缘部署 (量化/AOT/LoRA)
```

**关键改造**:
- 从 `nt_io_provider/` 迁移到 `nt_provider/`
- 混合编排: 本地/云 tiered routing + 离线fallback
- 边缘部署管线: ONNX→INT4量化→硬件检测→AOT编译

---

## 重构优先级

### P0（当前Sprint）
```
1. 删除 nt_act_mcp.rs (精确重复 nt_agent_mcp_discovery.rs)  ✓ 立即
2. 创建 nt_agent_protocol.rs (当前为broken declaration)
3. 创建 nt_agent_subagent.rs (当前为broken declaration)
4. nt_core_observer → PRM头注入
5. nt_core_policy → GRPO + Beam Search
6. GWT竞争点火
7. 5层压缩管线
8. E8嵌入VSA超向量
```

### P1（Next Sprint）
```
9. nt_mind_seal → DPOStage + ConstitutionalCritique + SafetyCheck
10. ProceduralMemoryStage
11. MCP 4传→2传 + OAuth 2.1
12. JEPA重构 (ViT + 动作条件预测器)
13. Mamba-1→Mamba-2(SSD)
14. Planner/Executor/Reflector分离
```

### P2（优化）
```
15. HyperCube MAP→FHRR D=2048
16. GWT共鸣→谐振器网络
17. Feature Steering (SAE clamping)
18. IIT φ修复 (因果效应谱系)
19. 主动推理修复 (FEP桥)
20. E8过渡可微分
```

---

## 代码质量问题（已发现的即时修复）

| 问题 | 位置 | 严重度 | 修复 |
|------|------|--------|------|
| 重复文件 (sha相同) | `nt_act_mcp.rs` == `nt_agent_mcp_discovery.rs` | **高** | 删除nt_act_mcp.rs |
| 声明但文件不存在 | `nt_agent_protocol` | **高** | 创建存根文件 |
| 声明但文件不存在 | `nt_agent_subagent` | **高** | 创建存根文件 |
| 23个bin入口 (仅4个注册) | `src/bin/` | 中 | 清理实验脚本 |
| 两个tool目录 | `agent/tools/` + `agent/tool/` | 中 | 合并 |

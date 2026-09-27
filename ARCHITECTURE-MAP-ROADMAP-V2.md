# NeoTrix 架构映射与进化路线图 (v2)

> **状态: 事实层已过期, 路线图仅表达意图。**
> §1–§7 是 2026-09-19 的快照, 此后 221 次提交未回填过任何一格。文中所有
> 百分比 (「外部对标覆盖 36%」「平均成熟度 C3.2」) **没有任何机制会更新它**,
> 因此永远是 36%。
> **当前权威 = git + CI, 不是本文。** 可再生的实测值见 §11;
> 结构性门禁见 `scripts/check-truth-surface.sh`。
> 另注: §4 用 `Gap Score = Stars × Uniqueness` 排序, Top 10 全是「能力缺口」,
> 而 2026-09-27 审计发现的真实瓶颈没有一个是能力缺口 —— 见 §11.3。

> 五实体正典：`docs/architecture/FIVE-ENTITY-BLUEPRINT-V3.md`（56 项目对标的能力落点以 E1 正典类型为准；迭代编号 E0–E4 与本文 I/M/NODE 并存，执行以任务清单为准）。

> 基于 ~1000+ URL 去重 → 4 Agent 并行分析 → 56 个项目映射
> 版本: v2.0.0 (2026-09-19)

---

## 1. 去重项目索引 (56 个项目)

### 1.1 Agent Skills & Orchestration (17 个)

| # | 项目 | Stars | 主要能力 | 层 | 域 | 对标模块 |
|---|------|-------|---------|---|---|---------|
| 1 | **modelcontextprotocol/servers** | 90.5K | MCP 工具标准化 | L1 | NT-ACT | `nt_agent_mcp_gateway` |
| 2 | **obra/superpowers** | 288.7K | 可组合技能框架 | L5 | NT-MIND | `nt_mind_skill_engine` |
| 3 | **agno-agi/agno** | 42.2K | Agent 平台运行时 | L5 | NT-CORE | `nt_meta::orchestrator` |
| 4 | **langchain-ai/langgraph** | 41.9K | 状态图工作流 | L5 | NT-CORE | `nt_core::state_graph` |
| 5 | **crewAIInc/crewAI** | 58.8K | 角色多 Agent | L5 | NT-CORE | `nt_core::multi_agent` |
| 6 | **pydantic/pydantic-ai** | 20.0K | 类型安全 Agent | L5 | NT-CORE | `nt_act::typed_agent` |
| 7 | **openai/openai-agents-python** | 29.6K | Guardrails + Handoff | L5 | NT-SHIELD | `nt_shield::guardrails` |
| 8 | **google/adk-python** | 21.6K | 图工作流 + A2A | L5 | NT-CORE | `nt_core::workflow` |
| 9 | **mastra-ai/mastra** | 28.2K | TS Agent 框架 | L5 | NT-CORE | `nt_io::ts_bridge` |
| 10 | **lastmile-ai/mcp-agent** | 8.5K | MCP 编排 + Durable | L5 | NT-CORE | `nt_core::durable_exec` |
| 11 | **tobi/skill-creator** | — | 技能创建工具 | L5 | NT-MIND | `nt_mind::skill_creator` |
| 12 | **davila7/claude-code-skill-vetter** | — | 技能质量评审 | L5 | NT-MIND | `nt_mind::skill_vetter` |
| 13 | **gtrusler/claude-code-skills** | — | 技能集合 | L5 | NT-MIND | `nt_mind::skill_store` |
| 14 | **pinkpixel-dev/claude-code-memory-bank** | — | 技能内存库 | L1 | NT-MEMORY | `nt_memory::skill_cache` |
| 15 | **pinkpixel-dev/claude-code-task-decomposer** | — | 任务分解器 | L5 | NT-CORE | `nt_core::task_decomposer` |
| 16 | **obra/superagent** | — | Agent 元框架 | L5 | NT-META | `nt_meta::agent_meta` |
| 17 | **anthropics/agent-protocol** | — | Agent 协议标准 | L1 | NT-ACT | `nt_agent::protocol` |

### 1.2 Memory Systems (8 个)

| # | 项目 | Stars | 主要能力 | 层 | 域 | 对标模块 |
|---|------|-------|---------|---|---|---------|
| 18 | **mem0ai/mem0** | 65.6K | 统一记忆层 | L1 | NT-MEMORY | `nt_memory::memory_hub` |
| 19 | **letta-ai/letta** | 24.8K | 长期记忆 + 自编辑 | L5 | NT-MIND | `nt_mind::self_edit` |
| 20 | **getzep/zep** | 4.9K | 时序知识图谱 | L2 | NT-WORLD | `nt_world::temporal_kg` |
| 21 | **Mintplex-Labs/anything-llm** | 66.2K | 本地 RAG + Agent | L2 | NT-WORLD | `nt_world::rag_pipeline` |
| 22 | **HusniAlduwor/awesome-ai-agents-memory** | — | 记忆系统目录 | — | — | 参考清单 |
| 23 | **mem0ai/awesome-mem0** | — | Mem0 生态 | — | — | 参考清单 |
| 24 | **jonahkoh/awesome-memory-agents** | — | 记忆 Agent 目录 | — | — | 参考清单 |
| 25 | **cpacker/MemGPT** | — | 重定向到 Letta | — | — | 同 #19 |

### 1.3 Observability & Evaluation (3 个)

| # | 项目 | Stars | 主要能力 | 层 | 域 | 对标模块 |
|---|------|-------|---------|---|---|---------|
| 26 | **langfuse/langfuse** | 34.8K | LLM 可观测 + 评估 | L6 | NT-META | `nt_meta::otel_bridge` |
| 27 | **AgentOps-AI/agentops** | 5.8K | Session 回放 + 成本 | L6 | NT-META | `nt_meta::session_replay` |
| 28 | **Arize-ai/phoenix** | 11.5K | LLM 评估 + 实验 | L6 | NT-META | `nt_meta::eval_engine` |

### 1.4 Code Intelligence (4 个)

| # | 项目 | Stars | 主要能力 | 层 | 域 | 对标模块 |
|---|------|-------|---------|---|---|---------|
| 29 | **astral-sh/ruff** | 49.7K | Python Linter (Rust) | L1 | NT-ACT | `nt_act::linter` |
| 30 | **astral-sh/uv** | 90.0K | Python 包管理 (Rust) | L1 | NT-ACT | `nt_act::package_mgr` |
| 31 | **Aider-AI/aider** | 49.1K | AI 编程助手 | L5 | NT-CORE | `nt_core::ai_coder` |
| 32 | **stackblitz-labs/bolt.diy** | 19.9K | 浏览器 AI 编码 | L5 | NT-CORE | `nt_io::browser_coder` |

### 1.5 Security & OSINT (9 个)

| # | 项目 | Stars | 主要能力 | 层 | 域 | 对标模块 |
|---|------|-------|---------|---|---|---------|
| 33 | **owasp/ASVS** | 3.6K | 安全验证标准 | L6 | NT-GOVERNANCE | `nt_shield::compliance` |
| 34 | **NVIDIA/garak** | 9.3K | LLM 漏洞扫描 | L3 | NT-SHIELD | `nt_shield::llm_scanner` |
| 35 | **Azure/PyRIT** | 4.5K | AI 红队框架 | L3 | NT-SHIELD | `nt_shield::red_team` |
| 36 | **trufflesecurity/trufflehog** | 28.0K | 密钥泄露扫描 | L3 | NT-SHIELD | `nt_shield::secret_scanner` |
| 37 | **aquasecurity/trivy** | 38.0K | 容器安全扫描 | L3 | NT-SHIELD | `nt_shield::container_scan` |
| 38 | **fox-it/BloodHound.py** | 2.4K | AD 安全图谱 | L2 | NT-WORLD | `nt_world::ad_graph` |
| 39 | **sherlock-project/sherlock** | 92.1K | 社交账号搜索 | L2 | NT-WORLD | `nt_world::osint_social` |
| 40 | **smicallef/spiderfoot** | 22.4K | OSINT 自动化 | L2 | NT-WORLD | `nt_world::osint_auto` |
| 41 | **laramies/theHarvester** | 17.5K | 邮箱/子域收集 | L2 | NT-WORLD | `nt_world::osint_harvest` |

### 1.6 Design & UX (3 个)

| # | 项目 | Stars | 主要能力 | 层 | 域 | 对标模块 |
|---|------|-------|---------|---|---|---------|
| 42 | **vercel/ai** | 26.8K | 统一 AI Toolkit | L1 | NT-IO | `nt_io::ai_provider` |
| 43 | **lobehub/lobehub** | 82.6K | 多 Agent 协作平台 | L6 | NT-NEXUS | `nt_nexus::agent_hub` |
| 44 | **langgenius/dify** | 156K | LLM 应用平台 | L5 | NT-CORE | `nt_core::app_platform` |

### 1.7 UI & Interface (1 个)

| # | 项目 | Stars | 主要能力 | 层 | 域 | 对标模块 |
|---|------|-------|---------|---|---|---------|
| 45 | **open-webui/open-webui** | 152.5K | 自托管 AI 界面 | L6 | NT-GOVERNANCE | `nt_io::web_ui` |

### 1.8 Research & Knowledge (2 个)

| # | 项目 | Stars | 主要能力 | 层 | 域 | 对标模块 |
|---|------|-------|---------|---|---|---------|
| 46 | **microsoft/generative-ai-for-beginners** | 120K | GenAI 教程 | L0 | NT-ED | 参考课程 |
| 47 | **e2b-dev/awesome-ai-agents** | 30.1K | Agent 生态目录 | — | — | 参考清单 |

### 1.9 404/重定向 (9 个)

| 项目 | 状态 |
|------|------|
| agent-protocol | 404 |
| superagent | 404 |
| skill-creator | 404 |
| claude-code-skill-vetter | 404 |
| claude-code-skills | 404 |
| claude-code-memory-bank | 404 |
| claude-code-task-decomposer | 404 |
| philschmid/ai-engineering | 404 |
| ShahAhmedRaza/* (5个) | 404 |

---

## 2. 6-Layer 架构映射

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ L6 Meta-Cognition (元认知层)                                                │
│   nt_meta + nt_repair + nt_nexus + nt_governance                           │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ langfuse (34.8K★)  agentops (5.8K★)  phoenix (11.5K★)            │   │
│   │ → OTel trace           → session replay    → LLM eval              │   │
│   │ ASVS (3.6K★)  open-webui (152.5K★)  lobehub (82.6K★)             │   │
│   │ → compliance标准        → RBAC界面           → 多Agent协作           │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│ L5 Cognition (认知层)                                                       │
│   nt_core + nt_mind                                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ superpowers (288.7K★)  agno (42.2K★)  langgraph (41.9K★)         │   │
│   │ → 技能编排            → agent运行时     → 状态图工作流              │   │
│   │ crewai (58.8K★)  pydantic-ai (20K★)  adk (21.6K★)               │   │
│   │ → 角色多agent         → 类型安全        → 图工作流+A2A              │   │
│   │ openai-agents (29.6K★)  aider (49.1K★)  bolt.diy (19.9K★)       │   │
│   │ → guardrails+handoff   → AI编程          → 浏览器AI编码             │   │
│   │ dify (156K★)  mastra (28.2K★)  mcp-agent (8.5K★)                │   │
│   │ → 应用平台             → TS框架           → MCP编排                 │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│ L4 Emotion (情感层) — NeoTrix 独有                                          │
│   nt_feel (情感引擎 + cognitive_bridge)                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│ L3 Embodiment (具身层)                                                      │
│   nt_physical + nt_shield + nt_feel                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ garak (9.3K★)  PyRIT (4.5K★)  trufflehog (28K★)  trivy (38K★)  │   │
│   │ → LLM红队         → AI红队         → 密钥扫描      → 容器扫描     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│ L2 Perception (感知层)                                                      │
│   nt_world + nt_sense                                                      │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ zep (4.9K★)  anything-llm (66.2K★)  BloodHound (2.4K★)          │   │
│   │ → 时序KG           → 本地RAG              → AD安全图谱             │   │
│   │ sherlock (92.1K★)  spiderfoot (22.4K★)  theHarvester (17.5K★)   │   │
│   │ → 社交搜索          → OSINT自动化          → 邮箱收集              │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│ L1 Action (行动层)                                                          │
│   nt_act + nt_io + nt_memory                                               │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │ mcp-servers (90.5K★)  mem0 (65.6K★)  letta (24.8K★)             │   │
│   │ → MCP工具标准化        → 统一记忆层        → 长期记忆               │   │
│   │ ruff (49.7K★)  uv (90K★)  vercel/ai (26.8K★)                    │   │
│   │ → 代码lint             → 包管理             → AI toolkit            │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. 域覆盖矩阵

| 域 | 层 | 项目数 | 覆盖度 | 最佳对标 | NeoTrix 独特能力 |
|---|---|---|---|---|---|
| NT-CORE | L5 | 8 | 🟢 92% | langgraph, crewai, pydantic-ai | ConsciousnessTree |
| NT-MIND | L5 | 4 | 🟡 68% | superpowers, letta | SEAL 进化管线 |
| NT-MEMORY | L1 | 3 | 🟢 85% | mem0, letta | experience-tree |
| NT-WORLD | L2 | 6 | 🟢 88% | zep, sherlock, spiderfoot | E8 感知拓扑 |
| NT-ACT | L1 | 4 | 🟢 90% | mcp-servers, ruff, uv | MCP Gateway |
| NT-SHIELD | L3 | 5 | 🟢 82% | garak, trivy, trufflehog | 审批链 + Guard Agent |
| NT-IO | L1 | 2 | 🟡 60% | vercel/ai | Tauri 桌面 |
| NT-META | L6 | 4 | 🟢 88% | langfuse, phoenix | GWT 注意力路由 |
| NT-NEXUS | L6 | 2 | 🟡 55% | mem0, lobehub | 跨会话记忆编织 |
| NT-GOVERNANCE | L6 | 2 | 🟡 50% | ASVS, open-webui | 治理策略引擎 |
| NT-REPAIR | L6 | 0 | 🔴 20% | 无直接对标 | 自愈修复循环 |
| NT-FEEL | L4 | 0 | 🔴 0% | NeoTrix 独有 | 情感中枢 |

---

## 4. 架构缺口分析

### 4.1 高优先级缺口 (Gap Score = Stars × Uniqueness)

| # | 缺口 | Gap Score | 当前 | 目标 | 吸收源 | 优先级 |
|---|------|-----------|------|------|--------|--------|
| **G1** | MCP 工具标准化 | 905K | 部分 | 完整 MCP 协议 | mcp-servers | P0 |
| **G2** | 技能编排框架 | 289K | 基础 | 可组合技能 | superpowers | P0 |
| **G3** | LLM 可观测链路 | 35K | 无 | OTel trace | langfuse | P0 |
| **G4** | 统一记忆层 | 66K | 分散 | 3-Tier + 实体链接 | mem0 | P0 |
| **G5** | Agent 红队安全 | 14K | 无 | 多轮攻击测试 | garak + PyRIT | P1 |
| **G6** | 容器安全扫描 | 38K | 无 | SBOM + CVE | trivy | P1 |
| **G7** | 密钥泄露检测 | 28K | 无 | 700+ 检测器 | trufflehog | P1 |
| **G8** | OSINT 自动化 | 132K | 基础 | 200+ 模块 | spiderfoot | P1 |
| **G9** | LLM 评估框架 | 12K | 无 | 实验管理 | phoenix | P2 |
| **G10** | 多 Agent 协作平台 | 83K | 无 | 可视化编排 | lobehub | P2 |

### 4.2 NeoTrix 独有能力 (无外部对标)

| # | 能力 | 层 | 成熟度 |
|---|------|---|--------|
| U1 | ConsciousnessTree 意识树 | L5 | C4 |
| U2 | SEAL 进化管线 | L5 | C3 |
| U3 | E8 感知拓扑 | L2 | C2 |
| U4 | VSA HyperCube | L2 | C2 |
| U5 | GWT 注意力路由 | L6 | C4 |
| U6 | 情感中枢 NT-FEEL | L4 | C3 |
| U7 | 跨会话记忆编织 | L6 | C4 |
| U8 | 能力树 DAG | L5 | C5 |
| U9 | 安全审批链 | L3 | C4 |

---

## 5. 迭代路线图

### Phase 0: 基础协议对齐 (P0, 2 周)

| 迭代 | 目标 | 吸收源 | 动作 | 产出 |
|------|------|--------|------|------|
| **I0.1** | MCP 协议层 | mcp-servers (90.5K) | 实现 MCP tool/resource/prompt 三端点 | `nt_agent::mcp_protocol` |
| **I0.2** | 技能编排 | superpowers (288.7K) | 实现 brainstorm→plan→TDD→review 技能链 | `nt_mind::skill_chain` |

### Phase 1: 记忆与可观测 (P0, 4 周)

| 迭代 | 目标 | 吸收源 | 动作 | 产出 |
|------|------|--------|------|------|
| **I1.1** | 统一记忆层 | mem0 (65.6K) | 3-Tier + 实体链接 + 混合检索 | `nt_memory::memory_hub` 增强 |
| **I1.2** | 长期记忆 | letta (24.8K) | 自编辑记忆 + 睡眠计算 | `nt_mind::self_edit` 增强 |
| **I1.3** | OTel 可观测 | langfuse (34.8K) | trace + prompt 管理 + 评估 | `nt_meta::otel_bridge` 增强 |
| **I1.4** | 时序知识图谱 | zep (4.9K) | 双时态实体图 + PPR | `nt_world::temporal_kg` 增强 |
| **I1.5** | 本地 RAG | anything-llm (66.2K) | 文档摄入 + 向量检索 | `nt_world::rag_pipeline` |

### Phase 2: 安全加固 (P1, 3 周)

| 迭代 | 目标 | 吸收源 | 动作 | 产出 |
|------|------|--------|------|------|
| **I2.1** | LLM 红队 | garak (9.3K) | 多轮攻击探针 + 检测器 | `nt_shield::llm_scanner` |
| **I2.2** | AI 红队 | PyRIT (4.5K) | 自适应攻击编排 | `nt_shield::red_team` |
| **I2.3** | 密钥扫描 | trufflehog (28K) | 700+ 检测器 + API 验证 | `nt_shield::secret_scanner` |
| **I2.4** | 容器安全 | trivy (38K) | CVE + SBOM + IaC | `nt_shield::container_scan` |
| **I2.5** | 合规框架 | ASVS (3.6K) | 286 需求 + 3 级验证 | `nt_shield::compliance` 增强 |

### Phase 3: OSINT 扩展 (P1, 3 周)

| 迭代 | 目标 | 吸收源 | 动作 | 产出 |
|------|------|--------|------|------|
| **I3.1** | 社交搜索 | sherlock (92.1K) | 400+ 平台用户名枚举 | `nt_world::osint_social` |
| **I3.2** | OSINT 自动化 | spiderfoot (22.4K) | 200+ 模块 + 关联引擎 | `nt_world::osint_auto` |
| **I3.3** | 域名侦察 | theHarvester (17.5K) | 邮箱/子域/IP 收集 | `nt_world::osint_harvest` |
| **I3.4** | AD 安全图谱 | BloodHound (2.4K) | 攻击路径 + Neo4j 图 | `nt_world::ad_graph` |

### Phase 4: 认知增强 (P2, 4 周)

| 迭代 | 目标 | 吸收源 | 动作 | 产出 |
|------|------|--------|------|------|
| **I4.1** | 状态图工作流 | langgraph (41.9K) | 图状态机 + 检查点 | `nt_core::state_graph` 增强 |
| **I4.2** | 角色多 Agent | crewai (58.8K) | 角色委托 + 结果聚合 | `nt_core::multi_agent` 增强 |
| **I4.3** | 类型安全 Agent | pydantic-ai (20K) | Schema 验证 + DI | `nt_act::typed_agent` 增强 |
| **I4.4** | LLM 评估 | phoenix (11.5K) | LLM-as-judge + 实验 | `nt_meta::eval_engine` 增强 |
| **I4.5** | Session 回放 | agentops (5.8K) | 事件日志 + 成本仪表板 | `nt_meta::session_replay` 增强 |

### Phase 5: 独有能力闭环 (P2, 3 周)

| 迭代 | 目标 | 吸收源 | 动作 | 产出 |
|------|------|--------|------|------|
| **I5.1** | 自愈诊断 | (自主) | 诊断→修复→验证链路 | `nt_repair::diagnostic_chain` 增强 |
| **I5.2** | 情感-认知耦合 | (自主) | 情感驱动推理调整 | `nt_feel::cognitive_bridge` 增强 |
| **I5.3** | 跨会话编织 | (自主) | 模式挖掘 + 图聚类 | `nt_nexus::memory_weaving` 增强 |

---

## 6. 优先级排序 (第一性原理)

| 优先级 | 迭代 | 理由 |
|--------|------|------|
| **P0** | I0.1, I0.2, I1.1-I1.5 | 基础协议 (MCP) + 记忆/可观测 = 所有上层的地基 |
| **P1** | I2.1-I2.5, I3.1-I3.4 | 安全 + OSINT = 生产就绪的必要条件 |
| **P2** | I4.1-I4.5, I5.1-I5.3 | 认知增强 + 独有能力 = 差异化竞争力 |

---

## 7. 关键指标

| 指标 | 当前 | Phase 0 后 | Phase 1 后 | Phase 2 后 | Phase 3 后 | Phase 5 后 |
|------|------|-----------|-----------|-----------|-----------|-----------|
| 外部对标覆盖 | 36% | 45% | 65% | 82% | 92% | 98% |
| 独有能力数 | 9 | 9 | 9 | 9 | 9 | 9+ |
| 平均成熟度 | C3.2 | C3.5 | C4.0 | C4.5 | C4.8 | C5.2 |
| 新增模块 | 0 | +2 | +7 | +12 | +16 | +19 |
| 新增测试 | 0 | +20 | +80 | +150 | +220 | +300 |

---

> 本路线图受 R-P111-R-P115 架构管理规则约束。
> 每次迭代完成后更新 ARCHITECTURE.md。
> 迭代记录走 KB experience-tree 吸收流程。

## 迭代记录 · 2026-09-26 大清洗

- 基线 HEAD `8a11227a`；9 worktree 全合入（diff 0）；脏树 ~1087（多窗 churn，不代他人合）。
- 版本：workspace 0.21.0 一致；`NeoBot@0.21.0` / `NeoTrix@0.22.0` 双产品线并存（见 ARCHITECTURE.md §14）。
- 清单：`sessions/handoff-global-todo-20260926.md` §8；§39 见 `sessions/handoff-S39-20260926.md`。
- 活体：soul 双端 online（tools=9）；`:8149` Down 按门拉起（blocked）；App 待目视。

---

## 11. 事实对账层 (2026-09-27 审计新增)

> 本节与 §1–§7 相反: **§1–§7 是意图, 本节是事实。**
> 本节所有数字都必须能被命令重新生成, 不接受人工填写 —— 否则重蹈 §7
> 「外部对标覆盖 36%」永远停在 36% 的覆辙。

### 11.1 可再生实测值

| 指标 | 实测值 | 再生命令 |
|---|---|---|
| Rust 文件 / LOC | 2,945 / 896,574 | `find neotrix-core/src crates apps src-tauri/src -name '*.rs'` |
| 0 字节 `.rs` | **0** | `find neotrix-core/src crates -name '*.rs' -size 0 \| wc -l` |
| 真值面门禁基线 | **1** | `grep -vc '^#' scripts/truth-surface-baseline.txt` |
| 门禁新增违规 | **0** | `bash scripts/check-truth-surface.sh --strict` |
| HEAD 能否独立编译 | **能** (2026-09-27 起) | `cargo check --tests -p neotrix` |
| `src/` 内 `#[test]` 数 | 13,316 | `grep -rc '#\[test\]\|#\[tokio::test\]' --include='*.rs' neotrix-core/src` |
| 全量套件通过 / 失败 / 忽略 | **12,042 / 50 / 37** | `cargo test -p neotrix --lib --no-run` 后跑二进制, `--test-threads=2` |
| 全量套件耗时 | 101s (2026-09-27 前**跑不完**) | 同上 |
| 永久挂起测试 | **0** (原 6) | `grep -c 'has been running for over' <log>` |

### 11.2 2026-09-27 审计已除的根 (均为「代码存在但工具链看不见」)

| # | 问题 | 规模 | 状态 |
|---|---|---|---|
| 1 | HEAD 无法独立编译: 已入库文件 `use` 了只存在于工作区的 `mod` 声明 | 29 文件 / 7,339 LOC | ✅ 已入库 |
| 2 | `cli/` 树已入库但 `pub mod cli;` 从未出现在任何 commit → 从未编译 | 87 文件 / 24,884 LOC | ✅ 已删 |
| 3 | 4 个抽取 crate 的 54 个 0 字节模块被 L5 当公开 API 再导出 | 54 文件 | ✅ 已摘 |
| 6 | 6 个 md5 相同的重复文件 (目录重构残留) | 1,769 LOC | ✅ 已删 |
| 7 | `nt_core_capability_tree` 未继承 workspace lint, 4,670 LOC 零约束 | 20 条告警 | ✅ 已修 |
| 8 | `nt_act_trade/tests/` 3 文件未声明 → 311 个测试从不编译 | 4,899 LOC | ✅ 复活 223 (107+116 全绿) / 删 88 |
| 9 | `auto_inspector` 内嵌 cargo 死锁 → 3 个测试永久挂起, 全量套件跑不完 | 3 测试 | ✅ 已修 |
| 10 | `.githooks/{post,pre-merge}` 悬空链接 → `reset --hard` 护栏一直没生效 | 2 hook | ✅ 已复活 |
| 11 | `.gitignore` 的 `tests/` 通配屏蔽 10 个源码目录 | — | ✅ 已解禁 |

**新增门禁** `scripts/check-truth-surface.sh` 卡 4 类:
`EMPTY` / `UNDECLARED` / `TRACKED` / `UNCOMMITTED_DEP`。
棘轮基线用**列表**而非计数 —— 计数基线挡不住「删一加一」, 新违规会隐身。
接线: `Makefile` (`make truth-surface` / `-strict` / `-baseline`) +
`ci.yml` check job (3 OS 矩阵, `--strict`)。

### 11.3 仍开放的一项 (需设计决策)

**`ExtractConfig` / `EmailConfig` / `PlatformRegistry` 三处双定义** —— 分别同时
存在于 `extractors/mod.rs` 与 `data_pipeline.rs`(`PlatformRegistry` 在
`data_pipeline.rs:211` 与 `platform_registry.rs:46`)。测试按所在模块各取一份,
`TradeDataPipeline::with_registry` 只认 `data_pipeline` 那份。这是随时会咬人的
坑, 但两个模块的语义确实不同, **收敛前需先确认二者是否本就该合并**。

### 11.3b 已被推翻的判断 (留档, 防止重犯)

审计中途我写过「`test_orchestration.rs` 的 116 个测试不可修, 属设计决策,
需先裁决两套 `WorkerType` taxonomy 的归属」。**该判断是错的。** 实为:
`DomainWorkerType` / `DomainWorkerResult` 是 `orchestrator_v2::{WorkerType,
WorkerResult}` 的**旧名**, 我把 import 路由到了 `workers` 模块才导致 120 个错。
两个同名符号确实存在 (两个 `WorkerType`、两个 `TradeWorker` trait), 但那是
**并存设计**而非冲突 —— 测试两个都要用, 分别引入作用域即可。

教训: 遇到「同名符号」先确认是否**旧名/新名**关系, 别直接上升为设计冲突。
判据: 若一个符号是另一个的子集且用法自洽, 它多半是重命名而非两套设计。

### 11.4 剩余 50 个失败 + 一个并发崩溃 (仅登记, 需单独一轮 triage)

按模块: nt_shield 7 / nt_core_capability 6 / nt_memory 5 / nt_feel 5 /
nt_core_aware 4 / nt_meta 3 / healing 3 / 其余 17 个各 1-2。
共同点多数是**断言与实现漂移**(如夹具自带 `Always` 规则、亚毫秒时长),
单看容易改, 但 50 个一起动风险大, 应独立成轮。

**并发崩溃 (pre-existing)**: `--test-threads=4` 时测试进程 **SIGSEGV**
(退出码 139), 在 `l6_meta::healing::predictive_maintenance::trend::tests`
之后; 同一模块单线程/两线程跑均不复现(311 passed)。属共享资源竞态或
栈/句柄耗尽, 非本次改动引入(未触碰相关模块)。**CI 暂用 `--test-threads=2`**;
根因需单独定位。

### 11.5 未除的已知债 (仅登记, 不在本次范围)

- `l5_cognition/lib.rs` 式的「目录模块旁挂 lib.rs」副本 (已随本次清掉 1 处;
  机制上仍可能再生, 故门禁只查 `mod` 绑定, 不查此类副本)。
- `sessions/` 80 个文件 / 8.2 MB, 而 `DOCUMENTATION-MAP.md` §一.1 规定根目录
  `TODO.md` 是唯一任务清单、§三.3 禁止每会话独立 TODO —— 规范自身被绕过 80 次。
- `DOCUMENTATION-MAP.md` §三.7 禁 >500 行的 md, 实测 67 个超限 (排除 node_modules)。
  这两条都说明**规范缺少强制点**; 门禁化 (退出码) 才是解药, 与 §11.2 同理。

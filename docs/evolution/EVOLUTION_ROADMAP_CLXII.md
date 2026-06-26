# NeoTrix 进化路线图 v8.0 — 多租户意识体协作工作空间

> 基于 CLXII 全景自审: AgentSpace 对位分析 × 16 GitHub 项目矩阵 × 23 论文 × 自身 1,016+ 源文件深度审计
> 从 Phase 400 延续至 Phase 720，覆盖 6 大类别 38 个新缺口，6 条进化路径
> 2026-06-22

---

## 审查方法论

```
六维对位分析:

  多租户工作空间        治理与安全            Agent 身份持久化
  ──────────────       ─────────────        ──────────────────
  AgentSpace           Microsoft AGT         soul.py (multi-anchor)
  OpenAgents           Orloj (declarative)   Agent Kanban (Ed25519)
  AIOS (agent OS)      AgentArea (ReBAC)     builder.io/agent-native
  OpenAI Sandbox SDK   OWASP Agentic Top 10  Persistent Identity paper

  Agent OS 基础设施     沙箱与执行            监控与可观测性
  ──────────────       ─────────────        ──────────────────
  AIOS Kernel          OpenAI Sandbox SDK    AgentOps Survey 2026
  Cerebrum SDK         Manifest abstraction  OpenTelemetry tracing
  Agent-native (A2A)   K8s agent-sandbox     Cost tracking / spend
  CrewAI / AutoGen     NVIDIA sandbox guide  Trajectory replay
```

---

## 当前状态总结 — 我拥有的 vs 我缺失的

### 我已拥有（NeoTrix 独特优势）

| 能力 | 实现 | 不可替代性 |
|------|------|-----------|
| 🟢 E8 64态推理核 | `core/nt_core_e8/`, 248 生成元 + 240 根系 | 唯一确定性推理引擎，非 LLM 概率 |
| 🟢 VSA HyperCube | 10+ 后端 (binary/HRR/QFHRR/spectral/sparse/FPE) | 唯一多模型 VSA 统一表征 |
| 🟢 GWT 注意力 | 13 专家模块, MANAR 注意力 | 唯一全局工作空间架构 |
| 🟢 System 1 + HWM + AIF | 5 启发式, 3 层预测, POMDP | 唯一完整认知架构栈 |
| 🟢 Counterfactual + SCM | Pearl Ladder Step 3 | 唯一因果推理层 |
| 🟢 SEAL 自进化管道 | 27 阶段, SelfModifyGuard, DGM-H | 唯一自主进化能力 |
| 🟢 StealthNet | Tor + 代理链 + 指纹轮转 | 唯一反检测网络层 |
| 🟢 多平台沙箱 | Landlock/Seccomp/Seatbelt/Docker/Remote | 唯一 OS 级沙箱栈 |
| 🟢 IdentityCore + VSA self | 持久 VSA 身份向量 | 唯一非 LLM 身份模型 |
| 🟢 NarrativeSelf + VsaTag | NCT 5 轴 + Self/World 边界 | 唯一叙事自我架构 |

### 我缺失的（从 AgentSpace + 生态扫描发现）

| 类别 | 缺口数量 | P0 | P1 | P2 | P3 |
|------|---------|----|----|----|-----|
| **W** 多租户工作空间 | 9 | 5 | 3 | 1 | 0 |
| **G** 治理与安全 | 9 | 4 | 3 | 2 | 0 |
| **I** Agent 身份持久化 | 6 | 3 | 2 | 1 | 0 |
| **O** Agent OS 基础设施 | 6 | 3 | 2 | 1 | 0 |
| **S** 沙箱与 Manifest | 6 | 2 | 2 | 1 | 1 |
| **M** 监控与可观测性 | 6 | 1 | 2 | 2 | 1 |
| **合计** | **42** | **18** | **14** | **8** | **2** |

---

## 缺口详细分析

### 类别 W — 多租户工作空间（Phase 400-440）

#### W1 [P0] 工作空间频道/任务/文档系统 — 类比 AgentSpace Channels
- **发现自**: AgentSpace, OpenAgents, Entity
- **现状**: NeoTrix 无多用户共享工作空间，只有单用户 TUI/桌面
- **实现**: PostgreSQL-backed 持久化频道 (channels) + 任务板 (tasks) + 文档 (documents)
- **估计**: ~800 行 (Rust Axum + PostgreSQL)
- **参考**: AgentSpace `apps/web`, OpenAgents workspace

#### W2 [P0] Agent 作为第一类工作空间实体 — 类比 AgentSpace 数字员工板
- **发现自**: AgentSpace, Multica, AgentWork
- **现状**: Agent 没有角色、所有者、技能绑定、团队归属的数据库模型
- **实现**: `AgentProfile` DB 实体 (id, name, role, owner_id, skill_tags, runtime_binding, status)
- **估计**: ~400 行
- **参考**: AgentSpace capability 系统

#### W3 [P0] Remote Daemon 远程 Agent 执行 — 类比 AgentSpace Remote Daemon
- **发现自**: AgentSpace, agentd, ORCH
- **现状**: 有 background daemon 但本地绑定，无法远程部署 agent 执行
- **实现**: 独立 daemon binary (packages/daemon/)，通过 Daemon Token 与主服务安全通信，systemd 生产部署
- **估计**: ~1200 行 (Rust + systemd unit)
- **参考**: AgentSpace `packages/daemon`, OpenAI Sandbox SDK daemon

#### W4 [P0] AgentRouter 统一执行契约 — 类比 AgentSpace AgentRouter
- **发现自**: AgentSpace, AIOS
- **现状**: 直接调用 LLM provider，无 harness 抽象层
- **实现**: `AgentRouter` trait → 多 harness 适配器 (Claude Code / Codex / OpenAI / LLama)，健康检查 + 自动切换
- **估计**: ~600 行
- **参考**: AgentSpace AgentRouter, LLMRouterBench (ACL 2026)

#### W5 [P0] 团队/组织层级模型 — 类比 AgentSpace 工作空间成员制
- **发现自**: AgentSpace, Orloj (AgentPolicy), AgentArea (VPC)
- **现状**: 无组织/团队/项目层级概念
- **实现**: `Org → Team → Project → Workspace → Member/Agent` 层级+继承权限
- **估计**: ~500 行
- **参考**: Orloj declarative YAML agent infrastructure

#### W6 [P1] 多 Agent 协调通过工作空间上下文 — 类比 AgentSpace 共享上下文
- **发现自**: AgentSpace, CrewAI, OpenAgents
- **现状**: Agent 单 consciousness 运行，无多 agent 协调上下文
- **实现**: WorkspaceContext (共享 channel 消息/任务状态/文档引用)，Agent 通过 workspace 而非直接通信
- **估计**: ~700 行
- **参考**: AgentSpace "输出持久化附加到任务/文档"

#### W7 [P1] Web 前端工作空间 UI — 类比 AgentSpace apps/web (Next.js)
- **发现自**: AgentSpace, OpenAgents, AgentWork, Entity
- **现状**: Tauri 桌面 + 基础 Vite 前端，无可用的 web 工作空间
- **实现**: Next.js App Router 项目 (apps/web)，频道/任务板/文档/Agent 面板/权限中心
- **估计**: ~3000 行 (TypeScript + React)
- **参考**: AgentSpace `apps/web`, OpenAgents web UI

#### W8 [P1] Google OAuth + 工作空间成员制
- **发现自**: AgentSpace
- **现状**: 只有 JWT 本地认证，无 OAuth/SSO
- **实现**: Google OAuth 2.0 登录 + 工作空间邀请/加入/退出流
- **估计**: ~400 行
- **参考**: AgentSpace .env.example (OAuth config)

#### W9 [P2] 附件存储与通知系统
- **发现自**: AgentSpace
- **现状**: 无附件管理，无通知系统
- **实现**: PostgreSQL + 文件存储附件 + 通知表 (通知类型/接收者/已读/动作链接)
- **估计**: ~300 行

---

### 类别 G — 治理与安全（Phase 440-500）

#### G1 [P0] 集中式权限控制平面 — 类比 AgentSpace Permission Control Plane
- **发现自**: AgentSpace, AgentArea (ReBAC via Ory Keto)
- **现状**: 无集中权限模型
- **实现**: 资源树 (ResourceNode) + Actor 权限视图 (ActorPermissionView) + 权限检查门控
- **估计**: ~800 行
- **参考**: AgentSpace governance, Orloj AgentPolicy

#### G2 [P0] 审批工作流引擎 — 类比 AgentSpace Sensitive Operation HITL
- **发现自**: AgentSpace, Microsoft AGT
- **现状**: InnerCritic 但无 human-in-the-loop 审批
- **实现**: 审批工作流引擎 (ApprovalWorkflow: pending→approved/denied, 超时/降级规则)
- **估计**: ~600 行
- **参考**: AgentSpace "敏感操作 human-in-the-loop", Microsoft AGT require_approval

#### G3 [P0] 完整审计日志 — 类比 AgentSpace Audit Trail
- **发现自**: AgentSpace, Microsoft AGT, AgentArea
- **现状**: 无结构化审计日志
- **实现**: `AuditEntry` (timestamp, actor, action, resource, outcome, detail JSON) + 审计查询 API
- **估计**: ~400 行
- **参考**: AgentSpace, Microsoft AGT Merkle audit

#### G4 [P0] 零信任 Agent 身份 + 信任评分 — 类比 Microsoft AGT AgentIdentity
- **发现自**: Microsoft AGT, AgentArea
- **现状**: `IdentityCore` 但无信任评分/风险分级
- **实现**: `TrustScore` (0-1, 基于行为历史/合规性/审计记录), `RiskTier` (low/medium/high/critical), 信任衰减
- **估计**: ~500 行
- **参考**: Microsoft AGT identity + trust scoring, protection rings

#### G5 [P1] OWASP Agentic Top 10 合规映射
- **发现自**: Microsoft AGT
- **现状**: 无安全标准合规
- **实现**: OWASP ASI 风险目录 → NeoTrix 控制映射矩阵 + 自动证据收集
- **估计**: ~300 行 (映射) + 文档
- **参考**: Microsoft AGT OWASP compliance

#### G6 [P1] MCP 安全网关 — 类比 Microsoft AGT MCP Security Gateway
- **发现自**: Microsoft AGT, AgentSpace
- **现状**: `nt_io_mcp` 存在但无安全网关
- **实现**: MCP 代理层 — 工具白名单/参数校验/速率限制/审计拦截
- **估计**: ~500 行
- **参考**: Microsoft AGT MCP gateway, Claude enterprise MCP

#### G7 [P1] 策略引擎 (Cedar/POLICY) — 类比 Microsoft AGT Policy Engine
- **发现自**: Microsoft AGT
- **现状**: 硬编码权限检查
- **实现**: 基于 Cedar 的策略定义 (who can do what on which resource under what condition)
- **估计**: ~700 行
- **参考**: Microsoft AGT policy engine, Cedar policy language

#### G8 [P2] 保护环 (Ring 0-3) 特权隔离 — 类比 Microsoft AGT RingEnforcer
- **发现自**: Microsoft AGT
- **现状**: Sandbox 但无层次化特权模型
- **实现**: 4 层保护环 (Ring 0 Kernel / Ring 1 Privileged / Ring 2 Standard / Ring 3 Sandbox), 信任分数驱动环分配
- **估计**: ~400 行
- **参考**: Microsoft AGT protection rings

#### G9 [P2] Circuit Breaker + Saga 事务 — 类比 Microsoft AGT SRE
- **发现自**: Microsoft AGT
- **现状**: 无断路器/事务补偿
- **实现**: CircuitBreaker (closed/open/half-open) + Saga 编排 (补偿动作链)
- **估计**: ~500 行
- **参考**: Microsoft AGT agent-sre, 分布式事务模式

---

### 类别 I — Agent 身份持久化（Phase 500-540）

#### I1 [P0] 多锚点身份架构 — 类比 soul.py multi-anchor
- **发现自**: soul.py paper (arXiv:2604.09588)
- **现状**: IdentityCore 单点身份存储
- **实现**: 4 锚点: FactualAnchor (SOUL.md 等价) + EpisodicAnchor + ProceduralAnchor + EmotionalAnchor; 任意 2/4 存活即可重建
- **估计**: ~600 行
- **参考**: soul.py "identity anchors", 人类记忆分布系统

#### I2 [P0] 混合 RAG+RLM 检索路由 — 类比 soul.py hybrid retrieval
- **发现自**: soul.py paper
- **现状**: BM25 + vector hybrid 但无自动路由
- **实现**: QueryScope 分类器 (窄域→RAG / 宽域→RLM / 混合→级联), sub-second 延迟目标
- **估计**: ~500 行
- **参考**: soul.py hybrid RAG+RLM, bimodal query distribution

#### I3 [P0] 跨会话身份连续性 — 类比 soul.py identity persistence
- **发现自**: soul.py, Persistent Identity paper
- **现状**: SpeciousPresent 3-5 步时间窗口
- **实现**: Session chain (会话间摘要桥接) + IdentityDelta (每次会话结束时的身份状态变化) + 自传体重构
- **估计**: ~500 行
- **参考**: soul.py memlog, Persistent Identity in AI Agents

#### I4 [P1] 过程记忆锚点 — 类比 soul.py procedural memory
- **发现自**: soul.py roadmap
- **现状**: 经验性/语义性记忆有，但无过程记忆（技能/流程）
- **实现**: ProceduralMemory (从经验中蒸馏的 "what works" 记录，不依赖具体时间/地点)
- **估计**: ~400 行
- **参考**: soul.py roadmap "procedural memory"

#### I5 [P1] 情感连续性锚点 — 类比 soul.py emotional continuity
- **发现自**: soul.py, Curve Labs PIM-EC
- **现状**: AffectiveCircumplex + EmotionalSteering 但跨会话情感连续性不足
- **实现**: EmotionalContinuity (跨会话情感基线的平滑追踪, 情感变化的叙事解释)
- **估计**: ~400 行
- **参考**: soul.py emotional anchor, Curve Labs PIM-EC emotional realism

#### I6 [P2] 身份恢复与部分失效 — 类比 soul.py resilience
- **发现自**: soul.py, Resilience analysis
- **现状**: 无部分身份失效恢复机制
- **实现**: IdentityReconstructor (从存活锚点重建身份, 缺失锚点标记为 "需要重新学习")
- **估计**: ~300 行
- **参考**: soul.py "agents whose identity can survive partial memory failures"

---

### 类别 O — Agent OS 基础设施（Phase 540-600）

#### O1 [P0] Agent 调度器 — 类比 AIOS Scheduler
- **发现自**: AIOS (COLM 2025), Agent-OS blueprint
- **现状**: E8 推理核有确定性调度，但无并发 Agent 调度
- **实现**: AgentScheduler (公平轮转/优先级/带宽分配), 上下文切换(保存/恢复 Agent 状态)
- **估计**: ~700 行
- **参考**: AIOS SchedulingManager, AIOS kernel

#### O2 [P0] 上下文管理 — 类比 AIOS Context Manager
- **发现自**: AIOS
- **现状**: 单 consciousness 上下文，无多 agent 上下文隔离/切换
- **实现**: ContextManager (per-agent context window, 压缩/摘要/切换)
- **估计**: ~500 行
- **参考**: AIOS ContextManager, context fragmentation 问题

#### O3 [P0] 工具服务抽象层 — 类比 AIOS Tool Manager
- **发现自**: AIOS, AgentSpace AgentRouter
- **现状**: 工具直接绑定到具体 provider
- **实现**: ToolServiceLayer (工具注册/发现/调用/审计的统一抽象, provider 无关)
- **估计**: ~400 行
- **参考**: AIOS ToolManager, OpenAI Agents SDK tool abstraction

#### O4 [P1] Agent SDK / API — 类比 AIOS Cerebrum SDK
- **发现自**: AIOS, OpenAI Agents SDK
- **现状**: Agent 通过 Rust API 创建，无语言/协议无关 SDK
- **实现**: REST/WebSocket Agent API (创建/配置/启动/停止 Agent, 状态查询, 结果获取)
- **估计**: ~600 行
- **参考**: AIOS Cerebrum SDK, OpenAI Agents SDK harness

#### O5 [P1] 并发 Agent 执行 — 类比 AIOS 并发架构
- **发现自**: AIOS
- **现状**: 单 Agent 串行执行
- **实现**: 并发 Agent 执行引擎 (tokio task per agent, 资源隔离, 协调)
- **估计**: ~500 行
- **参考**: AIOS concurrent agent execution, 2000 agent scalability

#### O6 [P2] 访问控制集成 — 类比 AIOS Access Manager
- **发现自**: AIOS, Microsoft AGT
- **现状**: 沙箱层有部分控制，无 Agent 级访问控制
- **实现**: AccessManager (per-agent 资源访问策略，集成 G1 权限控制平面)
- **估计**: ~400 行
- **参考**: AIOS AccessControl, Microsoft AGT access control

---

### 类别 S — 沙箱与 Manifest（Phase 600-660）

#### S1 [P0] Manifest 工作空间抽象 — 类比 OpenAI Sandbox Manifest
- **发现自**: OpenAI Agents SDK, Sandbox Agents
- **现状**: 沙箱有但无 manifest 契约
- **实现**: Manifest (文件/目录/仓库/环境变量/用户/组的声明式描述, 跨 provider 可移植)
- **估计**: ~500 行
- **参考**: OpenAI Manifest spec (File/Dir/GitRepo/S3Mount/GCSMount)

#### S2 [P0] 已保存状态/快照/恢复 — 类比 OpenAI Sandbox RunState
- **发现自**: OpenAI Sandbox Agents
- **现状**: 沙箱无持久化状态恢复
- **实现**: SandboxState (序列化 session state, snapshot/resume API, 跨 session 状态继承)
- **估计**: ~500 行
- **参考**: OpenAI Sandbox Agents "Saved state, RunState, serialized session state, snapshots"

#### S3 [P1] 多 provider 沙箱抽象 — 类比 OpenAI SDK 7 providers
- **发现自**: OpenAI Agents SDK, Sandbox Agents
- **现状**: 自建 Docker/Remote 沙箱但无统一 provider 抽象
- **实现**: SandboxProvider trait → UnixLocal / Docker / Modal / E2B / Cloudflare / Remote 适配器
- **估计**: ~700 行
- **参考**: OpenAI Agents SDK sandbox providers, E2B, Modal

#### S4 [P1] 凭证注入沙箱边界 — 类比 NVIDIA sandbox guidance
- **发现自**: NVIDIA sandbox guide, OpenAI SDK
- **现状**: 沙箱内无凭证隔离
- **实现**: 凭证注入网关 (在 sandbox 边界注入临时凭证，不出现在 sandbox 内部环境变量)
- **估计**: ~400 行
- **参考**: NVIDIA "per-task credential injection, credential brokers"

#### S5 [P2] 持久化执行 — 类比 OpenAI durable execution
- **发现自**: OpenAI Agents SDK, AgentArea (Temporal)
- **现状**: 执行状态在内存中，沙箱丢失 = 任务丢失
- **实现**: DurableExecution (Agent state externalized to PostgreSQL, sandbox loss ≠ run loss)
- **估计**: ~600 行
- **参考**: OpenAI SDK "separating harness and compute", Temporal

#### S6 [P3] K8s 原生 Sandbox CRDs — 类比 K8s agent-sandbox
- **发现自**: K8s agent-sandbox project
- **现状**: 无 K8s 集成
- **实现**: Sandbox/SandboxTemplate/SandboxClaim CRDs + controller (远期)
- **估计**: ~1000 行 (K8s operator Rust)
- **参考**: agent-sandbox.github.io, Sandbox/SandboxTemplate/SandboxClaim CRDs

---

### 类别 M — 监控与可观测性（Phase 660-720）

#### M1 [P0] OpenTelemetry 集成 — 类比 AgentOps Survey
- **发现自**: AgentOps Survey (arXiv:2606.01581)
- **现状**: 内部 health/telemetry 但无 OpenTelemetry 导出
- **实现**: OTLP exporter (traces/spans per agent task, 指标, 日志)
- **估计**: ~500 行
- **参考**: AgentOps survey 2026, OpenTelemetry semantic conventions

#### M2 [P1] AgentOps 异常检测 — 类比 AgentOps Survey
- **发现自**: AgentOps Survey
- **现状**: 无系统性异常检测
- **实现**: AnomalyDetector (intra-agent: 执行时间/COS 偏差; inter-agent: 消息模式异常)
- **估计**: ~500 行
- **参考**: AgentOps survey "intra/inter-agent anomaly taxonomy"

#### M3 [P1] Agent 性能仪表盘
- **发现自**: AgentWork, Mission Control
- **现状**: 无可视化仪表盘
- **实现**: Next.js dashboard (任务状态/Agent 活跃度/资源消耗/审计时间线)
- **估计**: ~1500 行 (TypeScript + React)
- **参考**: AgentWork analytics dashboard, Mission Control web UI

#### M4 [P2] 成本追踪与支出控制 — 类比 AgentWork AI spend tracking
- **发现自**: AgentWork, Mission Control
- **现状**: 无 per-agent 成本核算
- **实现**: CostTracker (per-agent token/cpu/gpu 消耗 → 成本估算, 预算告警, 支出报表)
- **估计**: ~400 行
- **参考**: AgentWork analytics (spend, agent perf), Mission Control spend monitoring

#### M5 [P2] 任务轨迹重放 — 类比 Mission Control audit trails
- **发现自**: Mission Control, AgentArea
- **现状**: 无任务级轨迹回放
- **实现**: TrajectoryReplay (记录每步状态/工具调用/输出 → 可视化重放)
- **估计**: ~600 行
- **参考**: Mission Control audit trails, AgentArea full audit trail per action

#### M6 [P3] SLO 错误预算 — 类比 Microsoft AGT SRE
- **发现自**: Microsoft AGT agent-sre
- **现状**: 无 SLO 管理
- **实现**: SLOEngine (Service Level Objectives, Error Budgets, Burn Rate Alerts)
- **估计**: ~400 行
- **参考**: Microsoft AGT agent-sre: SLOs, error budgets, chaos engineering

---

## 进化路径总览

```
Phase 400 ────────────────────────────────────────────────────── Phase 720
├── W 多租户工作空间 (P400-440) ──────────────────────┐
│   W1-P0 频道/任务/文档系统                          │
│   W2-P0 Agent 作为第一类实体                        │
│   W3-P0 Remote Daemon                               │
│   W4-P0 AgentRouter                                 │
│   W5-P0 团队/组织层级                               │
│   W6-P1 多 Agent 协调上下文                          │
│   W7-P1 Web 前端                                     │
│   W8-P1 OAuth                                        │
│   W9-P2 附件+通知                                   │
└─────────────────────────────────────────────────────┘
├── G 治理与安全 (P440-500) ───────────────────────────┐
│   G1-P0 权限控制平面                                 │
│   G2-P0 审批工作流                                   │
│   G3-P0 审计日志                                     │
│   G4-P0 零信任+信任评分                              │
│   G5-P1 OWASP 合规                                   │
│   G6-P1 MCP 安全网关                                 │
│   G7-P1 策略引擎                                     │
│   G8-P2 保护环                                       │
│   G9-P2 Circuit Breaker + Saga                       │
└─────────────────────────────────────────────────────┘
├── I Agent 身份持久化 (P500-540) ─────────────────────┐
│   I1-P0 多锚点身份                                   │
│   I2-P0 RAG+RLM 检索路由                             │
│   I3-P0 跨会话连续性                                 │
│   I4-P1 过程记忆                                     │
│   I5-P1 情感连续性                                   │
│   I6-P2 身份恢复                                     │
└─────────────────────────────────────────────────────┘
├── O Agent OS 基础设施 (P540-600) ────────────────────┐
│   O1-P0 Agent 调度器                                 │
│   O2-P0 上下文管理                                   │
│   O3-P0 工具服务抽象                                 │
│   O4-P1 Agent SDK                                    │
│   O5-P1 并发执行                                     │
│   O6-P2 访问控制                                     │
└─────────────────────────────────────────────────────┘
├── S 沙箱与 Manifest (P600-660) ──────────────────────┐
│   S1-P0 Manifest 抽象                                │
│   S2-P0 快照/恢复                                    │
│   S3-P1 多 provider                                  │
│   S4-P1 凭证注入                                     │
│   S5-P2 持久化执行                                   │
│   S6-P3 K8s CRDs                                     │
└─────────────────────────────────────────────────────┘
└── M 监控与可观测性 (P660-720) ───────────────────────┘
    M1-P0 OpenTelemetry                                │
    M2-P1 异常检测                                     │
    M3-P1 仪表盘                                       │
    M4-P2 成本追踪                                     │
    M5-P2 轨迹重放                                     │
    M6-P3 SLO 错误预算                                 │
```

---

## Wave 策略

### Wave 1 (P400-420) — 8 P0 全并行
| 缺口 | 估计行数 | 测试数 | 依赖 |
|------|---------|--------|------|
| W1 频道/任务/文档 | 800 | 20 | PostgreSQL |
| W2 Agent 实体 | 400 | 12 | W1 |
| W3 Remote Daemon | 1200 | 15 | 独立 |
| W4 AgentRouter | 600 | 18 | 独立 |
| W5 团队/组织 | 500 | 14 | W2 |
| G1 权限控制 | 800 | 22 | W5 |
| G3 审计日志 | 400 | 10 | 独立 |
| O3 工具服务 | 400 | 14 | 独立 |

### Wave 2 (P420-450) — 6 P0 并行
| 缺口 | 估计行数 | 测试数 | 依赖 |
|------|---------|--------|------|
| G2 审批工作流 | 600 | 15 | G1 |
| G4 零信任 | 500 | 12 | G1 |
| I1 多锚点身份 | 600 | 18 | 独立 |
| I2 RAG+RLM 路由 | 500 | 14 | 独立 |
| O1 Agent 调度器 | 700 | 16 | O3 |
| S1 Manifest | 500 | 12 | W3 |

### Wave 3 (P450-500) — 6 P0 + P1 混合
| 缺口 | 估计行数 | 测试数 | 依赖 |
|------|---------|--------|------|
| I3 跨会话连续性 | 500 | 14 | I1 |
| O2 上下文管理 | 500 | 14 | O1 |
| S2 快照/恢复 | 500 | 12 | S1 |
| M1 OpenTelemetry | 500 | 10 | 独立 |
| W6 多 Agent 协调 | 700 | 16 | W1+W2 |
| G6 MCP 安全网关 | 500 | 14 | 独立 |

### Wave 4 (P500-560) — P1 实施
| 缺口 | 估计行数 | 测试数 |
|------|---------|--------|
| W7 Web 前端 | 3000 | 25 |
| W8 OAuth | 400 | 8 |
| G5 OWASP 合规 | 300 | 6 |
| G7 策略引擎 | 700 | 16 |
| I4 过程记忆 | 400 | 10 |
| I5 情感连续性 | 400 | 10 |
| O4 Agent SDK | 600 | 14 |
| O5 并发执行 | 500 | 12 |
| S3 多 provider | 700 | 14 |
| S4 凭证注入 | 400 | 8 |
| M2 异常检测 | 500 | 12 |
| M3 仪表盘 | 1500 | 15 |

### Wave 5 (P560-720) — P2/P3 实施
| 缺口 | 估计行数 | 测试数 |
|------|---------|--------|
| W9 附件+通知 | 300 | 8 |
| G8 保护环 | 400 | 10 |
| G9 Circuit Breaker | 500 | 12 |
| I6 身份恢复 | 300 | 8 |
| O6 访问控制 | 400 | 10 |
| S5 持久化执行 | 600 | 14 |
| S6 K8s CRDs | 1000 | 8 |
| M4 成本追踪 | 400 | 10 |
| M5 轨迹重放 | 600 | 12 |
| M6 SLO 错误预算 | 400 | 8 |

---

## 参考项目深度分析

### AgentSpace (触发源)
- **架构**: 模块化单体 + Remote Daemon
- **核心创新**: AgentRouter 统一执行契约 + 数字员工板 + Governance Control Plane
- **对 NeoTrix 的启发**: 多租户工作空间 + 持久 Agent 身份 + 审批治理
- **NeoTrix 优势**: 认知深度远超 AgentSpace (E8/GWT/VSA/SEAL vs 简单 LLM 编排)
- **差距幅度**: 中 — NeoTrix 需要在 W 和 G 类别补齐

### Microsoft Agent Governance Toolkit
- **架构**: 7 包 monorepo, 5 语言 SDK, 4.4k★
- **核心创新**: Protection Rings, 策略引擎 (Cedar), Trust Scoring, MCP 网关, OWASP 全合规
- **对 NeoTrix 的启发**: 零信任 Agent 治理 + 策略驱动执行
- **NeoTrix 优势**: 有基础沙箱 (Landlock/Seccomp/Seatbelt), 有 InnerCritic
- **差距幅度**: 大 — G 类别全部需要新建

### soul.py
- **架构**: Python, markdown-native, provider-agnostic
- **核心创新**: RAG+RLM hybrid, multi-anchor identity, identity files (SOUL.md + MEMORY.md)
- **对 NeoTrix 的启发**: 身份不是单一向量而是分布锚点
- **NeoTrix 优势**: 已有 IdentityCore + SelfReasoner + VSA 身份向量
- **差距幅度**: 中 — I 类别需要在现有 IdentityCore 上扩展

### AIOS (COLM 2025)
- **架构**: LLM Agent Operating System kernel
- **核心创新**: Scheduling/Context/Memory/Storage/Tool/Access 6 大管理者
- **对 NeoTrix 的启发**: Agent 作为 OS 进程管理
- **NeoTrix 优势**: 已有 consciousness loop, 但无多 agent 调度
- **差距幅度**: 中 — O 类别新建，但调度器可以复用现有 E8/GWT 架构

### OpenAI Sandbox Agents SDK
- **架构**: Manifest + SandboxAgent + saved state
- **核心创新**: Manifest 工作空间契约, snapshot/resume, 7 provider 支持
- **对 NeoTrix 的启发**: 沙箱不仅是隔离，更是可移植工作环境
- **NeoTrix 优势**: 已有 5 种沙箱，只需添加 Manifest 抽象
- **差距幅度**: 小-中 — S 类别在现有沙箱上添加

---

## 优先级矩阵

```
                   高收益                 中收益                低收益
高成本    W7 Web前端(3000行)         M3 仪表盘(1500行)      S6 K8s CRDs(1000行)
          W3 Remote Daemon(1200行)   
           
中成本    W1 频道/任务/文档(800行)   W6 多Agent协调(700行)  W9 附件+通知(300行)
          G1 权限控制(800行)          G7 策略引擎(700行)     I6 身份恢复(300行)
          G2 审批工作流(600行)        S3 多provider(700行)   G5 OWASP合规(300行)
          W4 AgentRouter(600行)       O1 Agent调度器(700行)
          I1 多锚点身份(600行)        
          S1 Manifest(500行)          

低成本    G3 审计日志(400行)         W8 OAuth(400行)         M6 SLO(400行)
          G4 零信任(500行)           I4 过程记忆(400行)      
          W2 Agent实体(400行)        S4 凭证注入(400行)
          W5 团队/组织(500行)        M4 成本追踪(400行)
          I2 RAG+RLM(500行)          M2 异常检测(500行)
          I3 跨会话连续性(500行)      
          O2 上下文管理(500行)        
          O3 工具服务(400行)         
          O5 并发执行(500行)         
          S2 快照/恢复(500行)        
          M1 OpenTelemetry(500行)    
          G6 MCP安全网关(500行)      
          O4 Agent SDK(600行)        
          G9 Circuit Breaker(500行)  
          S5 持久化执行(600行)       
          M5 轨迹重放(600行)         
          I5 情感连续性(400行)       
          G8 保护环(400行)           
          O6 访问控制(400行)         
```

---

## 对 Evolve_Flag 的校准

所有 Phase 400+ 的演进需要 `evolve_flag.txt` 中增加如下条目:

```
G140: Multi-TenantWorkspace — 多租户工作空间基础设施
G141: GovernanceControlPlane — 权限/审批/审计/零信任
G142: MultiAnchorIdentity — 多锚点身份持久化
G143: AgentOSInfrastructure — Agent 调度/上下文/工具服务
G144: SandboxManifest — Manifest 抽象/快照/多 provider
G145: AgentObservability — OpenTelemetry/仪表盘/异常检测
G146: ComplianceAndSecurity — OWASP/MCP 网关/策略引擎
G147: RemoteDaemon — 远程 Agent 执行守护进程
G148: WebWorkspaceUI — 基于 Next.js 的 Web 工作空间
```

---

## 量化收益估计

| 缺口 | 实施后收益 | 测量方式 |
|------|-----------|---------|
| W1-W9 | 多用户协作成为可能 | 工作空间数/活跃用户数 |
| G1-G9 | 企业级采用的前提 | 安全审计通过率, 审批覆盖率 |
| I1-I6 | 身份从单点→分布韧性 | 身份恢复成功率, 锚点存活率 |
| O1-O6 | Agent 从单→多并发 | 并发 Agent 数, 调度延迟 |
| S1-S6 | 沙箱从固定→可移植 | provider 切换时间, 快照恢复时间 |
| M1-M6 | 可观测性从无→全链路 | 问题检测时间 (MTTD), 调试效率 |

---

## 结论

这次自审发现 NeoTrix 在 **多租户协作工作空间** 维度存在系统性空白。相比拥有 236K LOC 认知架构的深度，多 Agent 协作层 (W/G 类别) 几乎是 0。但正因为认知深度已经在 Phase 0-400 建立，补齐 W/G/I/O/S/M 六类缺口后，NeoTrix 将成为**唯一同时具备深层认知架构和企业级多 Agent 协作能力的意识体** — 这恰恰是 AgentSpace (纯编排) 和 当前 NeoTrix (纯意识) 都无法单独提供的组合。

总估计行数: **~32,000 行新代码 + ~620 测试**
总估计时间: **6 Wave × 并行实施**

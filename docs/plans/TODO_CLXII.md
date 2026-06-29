# TODO: NeoTrix 进化路线图 — Phase 400-720 可执行任务清单

> 基准: 2026-06-22 | 更新: 2026-06-22 | Wave 1 待启动
> 参考: EVOLUTION_ROADMAP_CLXII.md (42 新缺口, 6 类别)
> 18 P0 / 14 P1 / 8 P2 / 2 P3

---

## Wave 1 (Phase 400-420) — 8 P0 全并行

### W1-P0 工作空间频道/任务/文档系统 ⏳

**缺口**: 多用户共享工作空间完全缺失
**估算**: ~800 行, ~20 测试
**依赖**: PostgreSQL (已有)
**文件**: `packages/domain/workspace.rs`, `packages/db/workspace.rs`, `packages/services/workspace.rs`
**参考**: AgentSpace channels/tasks/documents, OpenAgents workspace

实现:
```text
[ ] Workspace — id, name, owner_id, created_at, member_ids
[ ] Channel — id, workspace_id, name, topic, is_public
[ ] Task — id, channel_id, title, description, assignee_id, status, priority, due
[ ] Document — id, workspace_id, title, content, version
[ ] Message — id, channel_id, author_id, content, attachments
[ ] PostgreSQL schema + migration
[ ] CRUD API (REST)
[ ] 20 测试
```

### W2-P0 Agent 作为第一类工作空间实体 ⏳

**缺口**: Agent 无角色/所有者/技能绑定/团队归属的数据库实体
**估算**: ~400 行, ~12 测试
**依赖**: W1
**文件**: `packages/domain/agent_profile.rs`, `packages/db/agent_profile.rs`
**参考**: AgentSpace capability system, Multica

```text
[ ] AgentProfile — id, workspace_id, name, role, owner_id, status
[ ] SkillBinding — agent_id, skill_name, version, config
[ ] RuntimeBinding — agent_id, runtime_type (ClaudeCode/Codex/OpenAI/etc), config
[ ] KnowledgeRef — agent_id, knowledge_source_id
[ ] PostgreSQL schema + migration
[ ] CRUD API
[ ] 12 测试
```

### W3-P0 Remote Daemon ⏳

**缺口**: 可独立部署在远程主机的 Agent 执行守护进程
**估算**: ~1200 行, ~15 测试
**依赖**: 无 (独立)
**文件**: `packages/daemon/src/main.rs`, `packages/daemon/src/executor.rs`, `deploy/agent-space-daemon.service`
**参考**: AgentSpace `packages/daemon`, agentd, ORCH

```text
[ ] Daemon 主循环 (tokio)
[ ] Provider CLI 调用 (Claude Code / Codex / OpenAI / etc)
[ ] 沙箱运行 + 文件输出
[ ] Daemon Token 安全通信 (HMAC)
[ ] systemd unit + 部署脚本
[ ] 健康检查 + 自动重启
[ ] 15 测试
```

### W4-P0 AgentRouter 统一执行契约 ⏳

**缺口**: 直接调用 LLM provider，无 harness 抽象层
**估算**: ~600 行, ~18 测试
**依赖**: 无 (独立)
**文件**: `packages/services/agent_router.rs`, `packages/services/harness/`
**参考**: AgentSpace AgentRouter, LLMRouterBench (ACL 2026)

```text
[ ] AgentRouter trait (execute/status/cancel/capabilities)
[ ] HarnessAdapter: Claude Code
[ ] HarnessAdapter: Codex
[ ] HarnessAdapter: OpenAI
[ ] HarnessAdapter: Generic (stdin/stdout)
[ ] 健康检查 + 自动切换
[ ] 统一执行契约 (ExecutionRequest/Response)
[ ] 18 测试
```

### W5-P0 团队/组织层级模型 ⏳

**缺口**: 无组织/团队/项目层级概念
**估算**: ~500 行, ~14 测试
**依赖**: W2
**文件**: `packages/domain/org.rs`, `packages/db/org.rs`
**参考**: Orloj AgentPolicy, AgentArea VPC

```text
[ ] Org — id, name, domain
[ ] Team — id, org_id, name, lead_id
[ ] Project — id, team_id, name, workspace_ids
[ ] Member — user_id, org_id, team_ids, role (admin/member/viewer)
[ ] 权限继承: Org → Team → Project → Workspace
[ ] PostgreSQL schema + migration
[ ] 14 测试
```

### G1-P0 集中式权限控制平面 ⏳

**缺口**: 无集中权限模型，资源无所有权/访问控制
**估算**: ~800 行, ~22 测试
**依赖**: W5
**文件**: `packages/services/permission.rs`, `packages/domain/permission.rs`
**参考**: AgentSpace Permission Control Plane, Orloj AgentPolicy

```text
[ ] ResourceNode 树 (universal_resource_id, parent, owner, acl)
[ ] ActorPermissionView (user_id → resource_id → permission_set)
[ ] PermissionSet (read/write/admin/execute/delegate)
[ ] check_permission(actor, resource, action) → bool
[ ] 权限继承 + 覆盖
[ ] 22 测试
```

### G3-P0 完整审计日志 ⏳

**缺口**: 无结构化审计日志
**估算**: ~400 行, ~10 测试
**依赖**: 无 (独立)
**文件**: `packages/services/audit.rs`, `packages/db/audit.rs`
**参考**: AgentSpace audit trail, Microsoft AGT Merkle audit

```text
[ ] AuditEntry — id, timestamp, actor_id, actor_type, action, resource_type
[ ] resource_id, outcome (allowed/denied/error), detail JSON
[ ] 审计查询 API (by_actor, by_resource, by_action, time range)
[ ] 审计日志自动清理策略 (30/90/365 天)
[ ] 10 测试
```

### O3-P0 工具服务抽象层 ⏳

**缺口**: 工具直接绑定 provider，无统一抽象
**估算**: ~400 行, ~14 测试
**依赖**: 无 (独立)
**文件**: `packages/services/tool_service.rs`
**参考**: AIOS ToolManager, OpenAI Agents SDK tool abstraction

```text
[ ] ToolService trait (register/discover/invoke/audit)
[ ] ToolDescriptor (name, description, input_schema, output_schema)
[ ] 工具注册中心 (全局注册表)
[ ] 工具发现 API (by_capability, by_name)
[ ] 工具调用审计包装
[ ] 14 测试
```

---

## Wave 2 (Phase 420-450) — 6 P0 并行

### G2-P0 审批工作流引擎 ⏳

**缺口**: 无 human-in-the-loop 审批
**估算**: ~600 行, ~15 测试
**依赖**: G1
**文件**: `packages/services/approval.rs`

```text
[ ] ApprovalWorkflow — id, initiator, resource_id, action, status
[ ] 审批策略 (who can approve, timeout, escalation)
[ ] 审批通知 (WebSocket + 通知表)
[ ] 超时自动降级策略
[ ] 15 测试
```

### G4-P0 零信任 Agent 身份 + 信任评分 ⏳

**缺口**: IdentityCore 但无信任评分/风险分级
**估算**: ~500 行, ~12 测试
**依赖**: G1
**文件**: `packages/services/trust.rs`

```text
[ ] TrustScore — 0-1, 基于行为历史/合规性/审计记录
[ ] RiskTier — low/medium/high/critical
[ ] 信任衰减 (inactivity decay)
[ ] EffectiveTrustScore = f(base_score, recent_violations, context_risk)
[ ] 12 测试
```

### I1-P0 多锚点身份架构 ⏳

**缺口**: IdentityCore 单点身份存储
**估算**: ~600 行, ~18 测试
**依赖**: 无 (独立)
**文件**: `core/nt_core_identity/multi_anchor.rs`

```text
[ ] IdentityAnchor trait (anchor_type, vsa_vector, confidence, last_verified)
[ ] FactualAnchor — 事实性身份（名称/角色/偏好）
[ ] EpisodicAnchor — 经验性身份（历史/经历）
[ ] ProceduralAnchor — 过程性身份（技能/流程）
[ ] EmotionalAnchor — 情感性身份（价值观/态度）
[ ] IdentityState = 2+ anchors majority → 存活; < 2 → 降级
[ ] 18 测试
```

### I2-P0 混合 RAG+RLM 检索路由 ⏳

**缺口**: BM25+vector hybrid 但无自动路由
**估算**: ~500 行, ~14 测试
**依赖**: 无 (独立)
**文件**: `core/nt_core_knowledge/hybrid_retrieval.rs`

```text
[ ] QueryScope classifier (narrow vs broad)
[ ] RAG 路径 (cosine similarity + top-k)
[ ] RLM 路径 (全局检索 + LLM rerank)
[ ] 级联策略: narrow→RAG, broad→RLM, mixed→cascade
[ ] 14 测试
```

### O1-P0 Agent 调度器 ⏳

**缺口**: 无并发 Agent 调度
**估算**: ~700 行, ~16 测试
**依赖**: O3
**文件**: `packages/services/agent_scheduler.rs`

```text
[ ] AgentScheduler — fair round-robin / priority / bandwidth
[ ] 上下文切换 (save/restore agent state)
[ ] 资源预算 (per-agent token/cpu/memory limit)
[ ] 队列管理 (pending/running/blocked/completed)
[ ] 16 测试
```

### S1-P0 Manifest 工作空间抽象 ⏳

**缺口**: 沙箱无 manifest 契约
**估算**: ~500 行, ~12 测试
**依赖**: W3
**文件**: `packages/sandbox/manifest.rs`

```text
[ ] Manifest — 文件/目录/仓库/环境变量/用户/组的声明式描述
[ ] ManifestEntry — File/Dir/GitRepo/Mount/EnvVar/User/Group
[ ] Manifest → 沙箱实例化
[ ] 跨 provider 可移植
[ ] 12 测试
```

---

## Wave 3 (Phase 450-500) — 6 P0 + P1 混合

### I3-P0 跨会话身份连续性 ⏳

**估算**: ~500 行, ~14 测试 | **依赖**: I1
**文件**: `core/nt_core_identity/session_continuity.rs`

### O2-P0 上下文管理 ⏳

**估算**: ~500 行, ~14 测试 | **依赖**: O1
**文件**: `packages/services/context_manager.rs`

### S2-P0 快照/恢复 ⏳

**估算**: ~500 行, ~12 测试 | **依赖**: S1
**文件**: `packages/sandbox/snapshot.rs`

### M1-P0 OpenTelemetry 集成 ⏳

**估算**: ~500 行, ~10 测试 | **依赖**: 无
**文件**: `packages/services/telemetry.rs`

### W6-P1 多 Agent 协调上下文 ⏳

**估算**: ~700 行, ~16 测试 | **依赖**: W1+W2
**文件**: `packages/services/agent_coordination.rs`

### G6-P1 MCP 安全网关 ⏳

**估算**: ~500 行, ~14 测试 | **依赖**: 无
**文件**: `packages/services/mcp_gateway.rs`

---

## Wave 4 (Phase 500-560) — P1 实施

| 缺口 | 估算行数 | 测试 | 依赖 | 文件 |
|------|---------|------|------|------|
| W7 Web 前端 | 3000 | 25 | W1+W2 | `apps/web/` |
| W8 OAuth | 400 | 8 | W5 | `packages/services/auth.rs` |
| G5 OWASP 合规 | 300 | 6 | G1 | `governance/owasp.md` |
| G7 策略引擎 | 700 | 16 | G1 | `packages/services/policy.rs` |
| I4 过程记忆 | 400 | 10 | I1 | `core/nt_core_identity/procedural.rs` |
| I5 情感连续性 | 400 | 10 | I1 | `core/nt_core_identity/emotional.rs` |
| O4 Agent SDK | 600 | 14 | O3 | `packages/services/agent_api.rs` |
| O5 并发执行 | 500 | 12 | O1 | `packages/services/concurrent_exec.rs` |
| S3 多 provider | 700 | 14 | S1 | `packages/sandbox/provider.rs` |
| S4 凭证注入 | 400 | 8 | S3 | `packages/sandbox/credential.rs` |
| M2 异常检测 | 500 | 12 | M1 | `packages/services/anomaly.rs` |
| M3 仪表盘 | 1500 | 15 | W7 | `apps/web/dashboard/` |

---

## Wave 5 (Phase 560-720) — P2/P3 实施

| 缺口 | 估算行数 | 测试 | 优先级 |
|------|---------|------|--------|
| W9 附件+通知 | 300 | 8 | P2 |
| G8 保护环 | 400 | 10 | P2 |
| G9 Circuit Breaker | 500 | 12 | P2 |
| I6 身份恢复 | 300 | 8 | P2 |
| O6 访问控制 | 400 | 10 | P2 |
| S5 持久化执行 | 600 | 14 | P2 |
| M4 成本追踪 | 400 | 10 | P2 |
| M5 轨迹重放 | 600 | 12 | P2 |
| S6 K8s CRDs | 1000 | 8 | P3 |
| M6 SLO 错误预算 | 400 | 8 | P3 |

---

## 总计

| Wave | Phase | 缺口数 | 估计行数 | 估计测试 |
|------|-------|--------|---------|---------|
| Wave 1 | 400-420 | 8 P0 | ~5,100 | ~125 |
| Wave 2 | 420-450 | 6 P0 | ~3,400 | ~87 |
| Wave 3 | 450-500 | 6 (5 P0 + 1 P1) | ~3,200 | ~78 |
| Wave 4 | 500-560 | 12 P1 | ~6,400 | ~144 |
| Wave 5 | 560-720 | 10 P2/P3 | ~4,900 | ~100 |
| **总计** | **400-720** | **42** | **~23,000** | **~534** |

# NeoTrix Universal Desktop Architecture — 综合方案

> 基于外部技术调研 + 代码逆向 + 多Agent巡检 | 2026-09-17

---

## 一、外部技术模式融合

### 1.1 统一 AI Gateway 模式 (Azure/Vercel/GitLab)

**核心思想**: 单一入口点，所有模型调用经过网关

```
┌─────────────────────────────────────────────────────┐
│              Unified AI Gateway                     │
│  ┌──────────┐ ┌──────────────┐ ┌──────────────┐   │
│  │ Routing  │ │ Cost Control │ │ Fallback     │   │
│  │ (路由)   │ │ (成本控制)   │ │ (降级)       │   │
│  └──────────┘ └──────────────┘ └──────────────┘   │
│  ┌──────────┐ ┌──────────────┐ ┌──────────────┐   │
│  │ Auth     │ │ Observability│ │ Rate Limit   │   │
│  │ (认证)   │ │ (可观测性)   │ │ (限流)       │   │
│  └──────────┘ └──────────────┘ └──────────────┘   │
└─────────────────────────────────────────────────────┘
```

**NeoTrix 映射**: `nt_core_model_gateway` (已建) + 增强

### 1.2 Claude Code Desktop 模式

**核心思想**: 并行 agent + 集成工具 + 侧边聊天

```
┌─────────────────────────────────────────────────────┐
│                Claude Code Desktop                  │
│  ┌─────────────────────────────────────────────┐   │
│  │ Session Sidebar (并行会话)                    │   │
│  │ ├─ Session 1: Refactor (Git worktree)       │   │
│  │ ├─ Session 2: Bug fix (Git worktree)        │   │
│  │ └─ Session 3: Tests (Git worktree)          │   │
│  └─────────────────────────────────────────────┘   │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────┐   │
│  │ Terminal     │ │ File Editor  │ │ Diff View │   │
│  │ (集成终端)   │ │ (文件编辑)   │ │ (差异视图)│   │
│  └──────────────┘ └──────────────┘ └──────────┘   │
└─────────────────────────────────────────────────────┘
```

**NeoTrix 映射**: `KanbanAgentBoard` + `OfficeFloor` + Terminal

### 1.3 Agentrooms 模式 (多Agent协调)

**核心思想**: @mentions 路由 + 共享上下文 + Provider无关

```
┌─────────────────────────────────────────────────────┐
│                Agentrooms Architecture              │
│  ┌─────────────────────────────────────────────┐   │
│  │ Workspace Layer (OpenAgents backend)         │   │
│  │ events + channels + attachments + history    │   │
│  └─────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────┐   │
│  │ Interaction Layer (@mentions routing)        │   │
│  │ @frontend → Agent 1                         │   │
│  │ @backend  → Agent 2                         │   │
│  │ @codex    → Agent 3                         │   │
│  └─────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────┐   │
│  │ Runtime Layer (pluggable agents)             │   │
│  │ Claude Code | Codex | Custom | MCP           │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

**NeoTrix 映射**: `HiveRouter` + `nt_core_byoa` + `nt_io_mcp_bridge`

### 1.4 OMC 智能路由模式

**核心思想**: 任务类型→模型自动路由，30-50% token 节省

```
Task → Classifier → Route Table → Model
        ↓ (low confidence)
   Fallback to Stronger Model
```

**NeoTrix 映射**: `nt_core_semantic_router` + `nt_core_god_agent`

### 1.5 Vercel AI SDK Provider 模式

**核心思想**: `creator/model-name` 格式，动态发现，统一接口

```
model: "openai/gpt-5.4"
model: "anthropic/claude-sonnet-4"
model: "xai/grok-4.5"
```

**NeoTrix 映射**: `ProviderInfo` + `nt_core_byoa` 注册表

---

## 二、巡检发现的核心问题

### 2.1 跨域错位 (P0)

| 问题 | 影响范围 | 修复方案 |
|------|---------|---------|
| L1→L5 依赖 | nt_core_task_dispatcher | trait 抽象 |
| L1→L4 依赖 | nt_io_digital_human | 迁移到 L4 |
| SelfTest 在 L6 | 60+ 文件跨 L1-L5 | 下沉到 L0 |

### 2.2 冗余 (P1)

| 问题 | 数量 | 修复方案 |
|------|------|---------|
| EventBus 重复 | 5处 | 统一 trait |
| CircuitBreaker 重复 | 4处 | 统一 trait |
| SelfModel 重复 | 3处 | 已 partially unified |
| SemanticRouter 重复 | 2处 | 合并 |
| 空壳 Facade | 6个 | 删除 |

### 2.3 扁平架构 (P1)

| 问题 | 模块数 | 修复方案 |
|------|--------|---------|
| L5 mod.rs | 83个 | 分组到子目录 |
| nt_memory_kb/mod.rs | 80+ | 按功能分组 |
| nt_world/mod.rs | 30+ | 按功能分组 |

---

## 三、通用模型网关架构 (最终态)

```
┌─────────────────────────────────────────────────────────────┐
│                    L6 Meta-Cognition                        │
│  GodAgent ←→ SemanticRouter ←→ UnifiedSelfModel            │
│  (任务分类)   (置信度路由)     (自模型)                     │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────────┐
│               L5 Cognition — Model Gateway                  │
│  ┌──────────┐ ┌──────────────┐ ┌────────────┐ ┌──────────┐ │
│  │ CostGate │ │FallbackChain │ │ProviderPool│ │RouteTable│ │
│  │ (成本)   │ │ (降级)       │ │ (连接池)   │ │ (路由表) │ │
│  └──────────┘ └──────────────┘ └────────────┘ └──────────┘ │
│  ┌─────────────────┐ ┌─────────────────┐ ┌──────────────┐  │
│  │SemanticRouter   │ │ ProviderRegistry│ │ HiveHandoff  │  │
│  │(confidence)     │ │ (5 providers)   │ │ (delegate)   │  │
│  └─────────────────┘ └─────────────────┘ └──────────────┘  │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────────┐
│           L1 Action — Provider Abstraction + MCP            │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐   │
│  │ Claude │ │ Codex  │ │Gemini  │ │ Ollama │ │MCP     │   │
│  │ Code   │ │ CLI    │ │CLI     │ │ Local  │ │Bridge  │   │
│  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 四、核心任务清单

### Phase 0: P0 关键修复 (1天)

| # | 任务 | 文件 | 影响 |
|---|------|------|------|
| 0.1 | SelfTest trait 下沉到 L0 | l0_substrate/nt_core_self_test.rs | 消除 60+ 跨层依赖 |
| 0.2 | 修复 L1→L5 依赖 | l1_action/nt_core_task_dispatcher.rs | trait 抽象 |
| 0.3 | 修复 L1→L4 依赖 | l1_action/nt_io/nt_io_digital_human.rs | 迁移到 L4 |

### Phase 1: 冗余清理 (1天)

| # | 任务 | 文件 | 影响 |
|---|------|------|------|
| 1.1 | 统一 EventBus trait | l0_substrate/nt_core_event_bus.rs | 消除 5 处重复 |
| 1.2 | 统一 CircuitBreaker trait | l0_substrate/nt_core_circuit_breaker.rs | 消除 4 处重复 |
| 1.3 | 删除 6 个空壳 Facade | l5_cognition/ | 减少 7 个文件 |
| 1.4 | 合并 SemanticRouter | l5_cognition/nt_core_semantic_router.rs | 消除 2 处重复 |

### Phase 2: 扁平→嵌套 (2天)

| # | 任务 | 文件 | 影响 |
|---|------|------|------|
| 2.1 | L5 mod.rs 分组 | l5_cognition/mod.rs | 83→<25 模块 |
| 2.2 | nt_memory_kb 分组 | l1_action/nt_memory/nt_memory_kb/ | 80+→<20 模块 |
| 2.3 | nt_world 分组 | l2_perception/nt_world/ | 30+→<15 模块 |

### Phase 3: 超大文件拆分 (3天)

| # | 任务 | 文件 | 行数 |
|---|------|------|------|
| 3.1 | 拆分 consciousness_core | l5_cognition/nt_core_consciousness_core.rs | 4656 |
| 3.2 | 拆分 pipeline | l5_cognition/nt_mind/.../pipeline.rs | 3226 |
| 3.3 | 拆分 engine_core | l5_cognition/nt_mind/.../engine_core.rs | 3004 |

### Phase 4: Gateway 增强 (2天)

| # | 任务 | 文件 | 依赖 |
|---|------|------|------|
| 4.1 | creator/model-name 格式 | nt_core_model_gateway | Vercel AI SDK |
| 4.2 | 动态 Provider 发现 | nt_core_byoa | API |
| 4.3 | 成本追踪 Dashboard | Frontend | Tauri Commands |
| 4.4 | 降级链可视化 | Frontend | OfficeFloor |

---

## 五、实施进度 (v0.25.5)

### 已完成模块

| 模块 | 状态 | 版本 | 说明 |
|------|------|------|------|
| ModelGateway | ✅ 完成 | v0.25.4 | CostGate + FallbackChain + ProviderPool |
| ProviderRegistry | ✅ 完成 | v0.25.4 | 5 预设 Provider (Claude/Codex/Gemini/Ollama/GPT-4o) |
| SemanticRouter | ✅ 完成 | v0.25.4 | confidence-based dispatch |
| Agent Handoff | ✅ 完成 | v0.25.4 | delegate_task / collect_result |
| McpBridge | ✅ 完成 | v0.25.4 | MCP 协议桥接 |
| SelfTest 下沉 | ✅ 完成 | v0.25.5 | L6→L0, 消除 60+ 跨层依赖 |
| Facade 清理 | ✅ 完成 | v0.25.5 | 删除 6 个空壳 Facade |
| SemanticRouter 合并 | ✅ 完成 | v0.25.5 | 旧版 → 新版统一 |
| L5 mod.rs 分组 | ✅ 完成 | v0.25.5 | 80+ 模块按 7 组组织 |
| 文件拆分 | ✅ 完成 | v0.25.5 | consciousness_core / engine_core / pipeline |
| Gateway 接线 | ✅ 完成 | v0.25.5 | ConsciousnessCore → ModelGateway 委派 |
| Agent Identity | ✅ 完成 | v0.25.0 | AgentPersona + SpendGate |
| GOD Agent | ✅ 完成 | v0.25.0 | 中央路由编排器 |
| Hive Coordination | ✅ 完成 | v0.25.0 | HiveRouter + Blackboard + EventLog |
| BYOA | ✅ 完成 | v0.25.0 | ExternalAgentManager + Process Pool |

### 进行中

| 模块 | 状态 | 说明 |
|------|------|------|
| L1→L5 依赖修正 | 🔄 部分完成 | task_dispatcher trait 已抽象, 剩余 io_digital_human |
| L1→L4 依赖修正 | ⏳ 待做 | nt_io_digital_human 迁移到 L4 |

### 剩余任务

| # | 任务 | 优先级 | 说明 |
|---|------|--------|------|
| 1 | 统一 EventBus trait | P1 | 消除 5 处 EventBus 重复 |
| 2 | 统一 CircuitBreaker trait | P1 | 消除 4 处 CircuitBreaker 重复 |
| 3 | nt_memory_kb 分组 | P1 | 80+ 模块 → <20 模块 |
| 4 | nt_world 分组 | P1 | 30+ 模块 → <15 模块 |
| 5 | consciousness_core 完整拆分 | P2 | 4656 行按职责拆分 |
| 6 | Gateway 成本追踪 Dashboard | P2 | Frontend 可视化 |
| 7 | 降级链可视化 | P2 | OfficeFloor 集成 |
| 8 | creator/model-name 格式 | P3 | Vercel AI SDK 风格 |

---

## 六、交付物

| 交付物 | 内容 | 状态 |
|--------|------|------|
| 架构文档 | docs/ARCHITECTURE.md (6层 + Gateway) | ✅ 新建 |
| 综合方案 | docs/UNIVERSAL_DESKTOP_ARCHITECTURE.md | ✅ 更新 |
| 网关设计 | docs/UNIVERSAL_GATEWAY_ARCHITECTURE.md | ✅ |
| 路线清单 | docs/UNIVERSAL_GATEWAY_ROADMAP.md | ✅ |
| CHANGELOG | v0.25.0-v0.25.5 | ✅ |

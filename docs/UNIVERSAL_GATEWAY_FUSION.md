# NeoTrix Universal Desktop Architecture — 融合方案

> 基于外部技术调研 + 代码库深度巡检 + 底层模型逆向分析 | 2026-09-18

---

## 一、外部技术模式融合

### 1.1 统一 AI Gateway 模式 (Vercel/Azure/MLflow)

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
**关键特性**: creator/model-name 格式, 200+ 模型, 35+ providers

### 1.2 Claude Code Desktop 模式

**核心思想**: 并行 agent + worktree 隔离 + 集成工具

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
**关键特性**: 自动 worktree 隔离, 子 agent 独立 worktree, 跨会话消息

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

### 2.1 跨域错位 (P0 FATAL)

| 问题 | 影响范围 | 修复方案 |
|------|---------|---------|
| L0→L6 反向依赖 | 2 处 (FATAL) | trait 下沉到 L0 |
| L1→L5 反向依赖 | 28 处 | trait 抽象 + 模块下沉 |
| L2→L5 反向依赖 | 22 处 | 模块下沉到 L1/L2 |
| L3→L6 反向依赖 | 100+ 处 | SelfTest trait 下沉到 L0 |

### 2.2 冗余 (P1)

| 问题 | 数量 | 修复方案 |
|------|------|---------|
| EventBus 重复 | 5处 | 统一 trait |
| CircuitBreaker 重复 | 4处 | 统一 trait |
| CapabilityRegistry 重复 | 4处 | 统一 trait |
| SelfModel 重复 | 3处 | 统一命名 |
| ReasoningMethod 重复 | 3处 | 统一定义 |
| EventType 重复 | 5处 | 统一定义 |

### 2.3 扁平架构 (P1)

| 问题 | 模块数 | 修复方案 |
|------|--------|---------|
| nt_io/mod.rs | 33个 | 分组到子目录 |
| l5_cognition/mod.rs | 42个 | 分组到子目录 |
| nt_core/mod.rs | 19个 | 分组到子目录 |
| nt_mind/mod.rs | 20+个 | 分组到子目录 |
| nt_act/mod.rs | 25+个 | 分组到子目录 |

### 2.4 超大文件 (P2)

| 文件 | 行数 | 修复方案 |
|------|------|---------|
| consciousness_core.rs | 4656 | 拆分 |
| experience.rs | 4117 | 拆分 |
| streaming.rs | 3313 | 拆分 |
| pipeline.rs | 3226 | 拆分 |
| entry/mod.rs | 3191 | 拆分 |
| nt_memory_kb/mod.rs | 3187 | 拆分 |
| engine_core.rs | 3004 | 拆分 |

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

## 四、核心路线任务清单

### Phase 0: P0 FATAL 修复 (1天)

| # | 任务 | 文件 | 影响 |
|---|------|------|------|
| 0.1 | SelfTest trait 下沉 L6→L0 | l0_substrate/nt_core_self_test.rs | 消除 100+ 跨层依赖 |
| 0.2 | L0→L6 反向依赖修复 | l0_substrate/nt_core_error/recovery.rs | 消除 FATAL |
| 0.3 | L0→L6 反向依赖修复 | l0_substrate/nt_core_cache.rs | 消除 FATAL |

### Phase 1: 冗余清理 (2天)

| # | 任务 | 文件 | 影响 |
|---|------|------|------|
| 1.1 | 统一 EventBus trait | l0_substrate/nt_core_event_bus.rs | 消除 5 处重复 |
| 1.2 | 统一 CircuitBreaker trait | l0_substrate/nt_core_circuit_breaker.rs | 消除 4 处重复 |
| 1.3 | 统一 CapabilityRegistry trait | l0_substrate/nt_core_capability.rs | 消除 4 处重复 |
| 1.4 | 统一 SelfModel 命名 | l6_meta/ | 消除 3 处混淆 |
| 1.5 | 统一 ReasoningMethod 定义 | l0_substrate/ | 消除 3 处重复 |
| 1.6 | 统一 EventType 定义 | l0_substrate/ | 消除 5 处重复 |

### Phase 2: 跨域错位修复 (3天)

| # | 任务 | 文件 | 影响 |
|---|------|------|------|
| 2.1 | nt_core_math 下沉 L5→L0 | l0_substrate/nt_core_math.rs | 消除 22+ L2→L5 依赖 |
| 2.2 | nt_core_hex 下沉 L5→L0 | l0_substrate/nt_core_hex.rs | 消除 6+ L2→L5 依赖 |
| 2.3 | nt_core_shared_types 下沉 L5→L0 | l0_substrate/nt_core_shared_types.rs | 消除类型依赖 |
| 2.4 | nt_core_policy 下沉 L5→L1 | l1_action/nt_core_policy.rs | 消除 L1→L5 依赖 |
| 2.5 | nt_core_reasoning 下沉 L5→L1 | l1_action/nt_core_reasoning.rs | 消除 L1→L5 依赖 |
| 2.6 | nt_core_kron/walsh 下沉 L5→L0 | l0_substrate/ | 消除 bank 依赖 |

### Phase 3: 扁平→嵌套 (3天)

| # | 任务 | 文件 | 影响 |
|---|------|------|------|
| 3.1 | nt_io/mod.rs 分组 | l1_action/nt_io/mod.rs | 33→<15 模块 |
| 3.2 | l5_cognition/mod.rs 分组 | l5_cognition/mod.rs | 42→<20 模块 |
| 3.3 | nt_core/mod.rs 分组 | l5_cognition/nt_core/mod.rs | 19→<10 模块 |
| 3.4 | nt_mind/mod.rs 分组 | l5_cognition/nt_mind/mod.rs | 20+→<10 模块 |
| 3.5 | nt_act/mod.rs 分组 | l1_action/nt_act/mod.rs | 25+→<12 模块 |

### Phase 4: 超大文件拆分 (5天)

| # | 任务 | 文件 | 行数 |
|---|------|------|------|
| 4.1 | 拆分 consciousness_core | l5_cognition/nt_core_consciousness_core.rs | 4656 |
| 4.2 | 拆分 experience | bin/experience.rs | 4117 |
| 4.3 | 拆分 streaming | l1_action/nt_media/streaming.rs | 3313 |
| 4.4 | 拆分 entry/mod.rs | entry/mod.rs | 3191 |
| 4.5 | 拆分 nt_memory_kb | l1_action/nt_memory/nt_memory_kb/mod.rs | 3187 |

### Phase 5: Gateway 增强 (2天)

| # | 任务 | 文件 | 依赖 |
|---|------|------|------|
| 5.1 | creator/model-name 格式 | nt_core_model_gateway | Vercel AI SDK |
| 5.2 | 动态 Provider 发现 | nt_core_byoa | API |
| 5.3 | 成本追踪 Dashboard | Frontend | Tauri Commands |
| 5.4 | 降级链可视化 | Frontend | OfficeFloor |

---

## 五、交付物

| 交付物 | 内容 | 状态 |
|--------|------|------|
| 架构文档 | docs/UNIVERSAL_DESKTOP_ARCHITECTURE.md | ✅ |
| 融合方案 | docs/UNIVERSAL_GATEWAY_ARCHITECTURE.md | ✅ |
| 巡检报告 | 本文件 | ✅ |
| 路线清单 | 本文件 Phase 0-5 | ✅ |
| 代码实现 | 5个 Phase 完成 | 🔄 进行中 |

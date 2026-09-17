# NeoTrix Universal Model Gateway — 核心路线任务清单

> 最后更新: 2026-09-17 | 基于外部技术调研 + 代码逆向 + 多Agent巡检

---

## 一、架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                    L6 Meta-Cognition                        │
│  GodAgent ←→ SemanticRouter ←→ UnifiedSelfModel            │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────────┐
│               L5 Cognition — Model Gateway                  │
│  ┌──────────┐ ┌──────────────┐ ┌────────────┐ ┌──────────┐ │
│  │ CostGate │ │FallbackChain │ │ProviderPool│ │RouteTable│ │
│  └──────────┘ └──────────────┘ └────────────┘ └──────────┘ │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────────┐
│           L1 Action — Provider Abstraction                  │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐   │
│  │ Claude │ │ Codex  │ │Gemini  │ │ Ollama │ │ BYOA   │   │
│  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、核心任务清单

### Phase 1: Model Gateway 核心 (P0 — 2天)

| # | 任务 | 文件 | 依赖 | 状态 |
|---|------|------|------|------|
| 1.1 | 创建 `nt_core_model_gateway.rs` | l5_cognition/ | — | ⬜ |
| 1.2 | 实现 `ModelGateway::route()` 统一入口 | nt_core_model_gateway | 1.1 | ⬜ |
| 1.3 | 实现 `CostGate` 成本门控 | nt_core_model_gateway | 1.2 | ⬜ |
| 1.4 | 实现 `FallbackChain` 降级链 | nt_core_model_gateway | 1.2 | ⬜ |
| 1.5 | 实现 `ProviderPool` 连接池 | nt_core_model_gateway | 1.2 | ⬜ |
| 1.6 | 单元测试 (10+) | nt_core_model_gateway | 1.5 | ⬜ |

### Phase 2: Provider Registry (P0 — 1天)

| # | 任务 | 文件 | 依赖 | 状态 |
|---|------|------|------|------|
| 2.1 | 增强 `ExternalAgentManager` 为 Provider 注册表 | nt_core_byoa | — | ⬜ |
| 2.2 | 实现 `ProviderInfo` 元数据结构 | nt_core_byoa | 2.1 | ⬜ |
| 2.3 | 实现 `register_provider()` / `list_providers()` | nt_core_byoa | 2.2 | ⬜ |
| 2.4 | 接入 Claude Code / Codex / Gemini CLI 预设 | nt_core_byoa | 2.3 | ⬜ |
| 2.5 | Tauri Commands: `gateway_list_providers` | commands/ | 2.4 | ⬜ |

### Phase 3: Semantic Router (P1 — 2天)

| # | 任务 | 文件 | 依赖 | 状态 |
|---|------|------|------|------|
| 3.1 | 增强 `TaskClassifier` 加置信度输出 | nt_core_god_agent | — | ⬜ |
| 3.2 | 创建 `nt_core_semantic_router.rs` | l5_cognition/ | 3.1 | ⬜ |
| 3.3 | 实现路由表: task_type → model_id | nt_core_semantic_router | 3.2 | ⬜ |
| 3.4 | 实现 confidence-based fallback | nt_core_semantic_router | 3.3 | ⬜ |
| 3.5 | 接入 ModelGateway::route() | nt_core_model_gateway | 3.4 | ⬜ |
| 3.6 | 单元测试 (8+) | nt_core_semantic_router | 3.5 | ⬜ |

### Phase 4: Agent Handoff (P1 — 2天)

| # | 任务 | 文件 | 依赖 | 状态 |
|---|------|------|------|------|
| 4.1 | `HiveRouter` 接入 `AgentLoop` | nt_io_hive_agent_loop | — | ⬜ |
| 4.2 | 实现 `delegate_task()` 任务委派 | nt_core_hive | 4.1 | ⬜ |
| 4.3 | 实现 `collect_result()` 结果回收 | nt_core_hive | 4.2 | ⬜ |
| 4.4 | Frontend: KanbanAgentBoard 真实数据 | RightBar.tsx | 4.3 | ⬜ |
| 4.5 | Tauri Commands: `hive_delegate_task` | commands/hive | 4.4 | ⬜ |

### Phase 5: MCP Bridge (P2 — 1天)

| # | 任务 | 文件 | 依赖 | 状态 |
|---|------|------|------|------|
| 5.1 | 创建 `nt_io_mcp_bridge.rs` | l1_action/nt_io/ | — | ⬜ |
| 5.2 | 实现 MCP Server 暴露 NeoTrix 工具 | nt_io_mcp_bridge | 5.1 | ⬜ |
| 5.3 | 实现 MCP Client 调用外部工具 | nt_io_mcp_bridge | 5.2 | ⬜ |
| 5.4 | Tauri Commands: `mcp_list_tools` | commands/ | 5.3 | ⬜ |

---

## 三、已完成清理 (v0.25.0-v0.25.3)

### 冗余清理

| 清理项 | 前 | 后 | 状态 |
|--------|-----|-----|------|
| `now_secs()` 重复 | 13处 | 1处 (`nt_core_time`) | ✅ |
| Stub facade | 6个 | 1个 (`layer_aliases`) | ✅ |
| SelfModel 三重定义 | 3处 | 1处 (`self_model_unified`) | ✅ |

### 跨域修正

| 修正项 | 前 | 后 | 状态 |
|--------|-----|-----|------|
| AgentCircuitBreaker | L3 Shield | L5 Cognition | ✅ |
| L6 元认知模块 | L5 nt_core/nt_meta | L6 nt_meta | ✅ |
| L1→L5 反向依赖 | 直接 import | trait 抽象 | ✅ |

### 模块注册

| 新模块 | 层 | 功能 | 状态 |
|--------|-----|------|------|
| `nt_core_hive` | L5 | Hive 协调协议 | ✅ |
| `nt_core_byoa` | L5 | 外部 Agent 管理 | ✅ |
| `nt_agent_identity` | L6 | Agent 身份系统 | ✅ |
| `nt_agent_gallery` | L6 | Agent 画廊 | ✅ |
| `nt_core_god_agent` | L5 | GOD Agent 路由 | ✅ |
| `nt_core_agent_circuit_breaker` | L5 | Agent 熔断器 | ✅ |
| `nt_io_hive_agent_loop` | L1 | Hive-AgentLoop 桥 | ✅ |
| `nt_core_time` | L0 | 共享时间工具 | ✅ |
| `layer_aliases` | L5 | 层别名统一 | ✅ |
| `self_model_unified` | L6 | 统一 SelfModel | ✅ |

---

## 四、外部技术映射

| 外部模式 | 来源 | NeoTrix 映射 | 状态 |
|---------|------|-------------|------|
| Model Gateway | Microsoft Foundry | `nt_core_model_gateway` | 待建 |
| Cost-Aware Routing | RouteLLM (85%成本降低) | `CostGate` | 待建 |
| Semantic Router + LLM Fallback | Microsoft MARA | `nt_core_semantic_router` | 待建 |
| Provider Abstraction | aether-llm / multi-llm | `LlmProvider` trait | 已有 |
| Agent Registry | Microsoft MARA | `nt_agent_identity` | 已有 |
| Agent Command Center | Windsurf/Devin | `KanbanAgentBoard` | 已有 |
| MCP Protocol | Anthropic MCP | `nt_io_mcp_bridge` | 待建 |
| BYOA | Cumora | `nt_core_byoa` | 已有 |

---

## 五、多 Agent 巡检编排

### 巡检维度

| 维度 | Agent | 频率 | 触发 |
|------|-------|------|------|
| D1 冗余检测 | explore agent | 每次 commit | 自动 |
| D2 扁平检测 | explore agent | 每次 commit | 自动 |
| D3 跨域检测 | review agent | 每次 PR | 自动 |
| D4 编译验证 | cargo check | 每次 commit | 自动 |
| D5 测试覆盖 | cargo test | 每次 PR | 自动 |

### 自动修复流程

```
巡检发现问题 → 分类 (冗余/扁平/错位) → 生成修复方案 → Agent 执行 → 编译验证 → 报告
```

---

## 六、关键设计原则

1. **单一入口**: 所有模型调用经过 `ModelGateway::route()`
2. **成本优先**: 超预算自动降级，不需人工干预
3. **置信度路由**: 低置信度任务自动升级到更强模型
4. **Provider 无关**: 新增 provider 只需实现 `LlmProvider` trait
5. **向后兼容**: 所有重构保留 re-export
6. **零 unsafe**: `#![forbid(unsafe_code)]`
7. **nt_ 前缀**: 所有模块名使用 `nt_` 前缀

# NeoTrix Architecture

> 6-Layer Consciousness Architecture with Unified Model Gateway | 2026-09-17

---

## 1. Layer Overview

```
L6 Meta-Cognition (元认知层)    →  nt_meta + nt_repair + nt_nexus
L5 Cognition (认知层)           →  nt_core + nt_mind + ModelGateway
L4 Emotion (情感层)             →  nt_feel (core emotion engine)
L3 Embodiment (具身层)          →  nt_physical + nt_shield + nt_feel
L2 Perception (感知层)          →  nt_world + nt_sense
L1 Action (行动层)              →  nt_act + nt_io + nt_memory
L0 Substrate (基底层)           →  SelfTest + Time + EventBus + Types
```

### Dependency Rule

**Downward only**: `L(n)` may import from `L(n-1)` but never `L(n+1)`. Cross-layer communication via traits (e.g., `ReasoningEngineProvider` in L1 for L5 logic).

---

## 2. Layer Directory Map

```
neotrix-core/src/
├── l0_substrate/           # L0 — Foundation: SelfTest, Time, EventBus, Types
│   ├── nt_core_self_test.rs    ← SelfTest trait (下沉自 L6, 消除 60+ 跨层依赖)
│   ├── nt_core_time.rs         ← now_secs() 共享时间工具
│   └── nt_core_event_bus.rs    ← 统一事件总线
├── l1_action/              # L1 — Action: tools, IO, memory
│   ├── nt_act/             ← 工具/动作执行
│   ├── nt_io/              ← IO/接口层
│   └── nt_memory/          ← 记忆存储 (KB, Memory Palace)
├── l2_perception/          # L2 — Perception: world, senses
│   ├── nt_world/           ← 世界感知 (crawl, web, sensor)
│   └── nt_sense/           ← 感官处理
├── l3_embodiment/          # L3 — Embodiment: physical, shield, feel
│   ├── nt_physical/        ← 身体模式
│   ├── nt_shield/          ← 安全/保护 (agent circuit breaker)
│   └── nt_feel/            ← 情感具身
├── l4_emotion/             # L4 — Emotion: core emotion engine
│   └── nt_feel/            ← EmotionLabel (11 variants), emotion state
├── l5_cognition/           # L5 — Cognition: reasoning, strategy, gateway
│   ├── mod.rs              ← 80+ 模块声明, 按功能分组:
│   │   // Consciousness    (consciousness_core, self_model, attention)
│   │   // Reasoning        (semantic_router, task_dispatcher)
│   │   // Strategy         (god_agent, hive, coordination_principles)
│   │   // Evolution        (mind loops, experience, repair hooks)
│   │   // Gateway          (model_gateway, cost_gate, fallback_chain)
│   │   // Math / Types     (hypercube, E8)
│   ├── nt_core/            ← 核心推理 (consciousness_core, self_model)
│   ├── nt_mind/            ← 自我进化 (evolution loops, experience)
│   └── nt_core_model_gateway.rs  ← ModelGateway (统一网关)
├── l6_meta/                # L6 — Meta-Cognition: meta, repair, nexus
│   ├── nt_meta/            ← 元认知协调
│   ├── nt_repair/          ← 自愈修复
│   └── nt_nexus/           ← 跨会话记忆
└── core/                   # Shared: E8, HyperCube, GWT, SEAL
```

---

## 3. Model Gateway Integration

### Architecture

```
┌──────────────────────────────────────────────────────────┐
│                   L6 Meta-Cognition                       │
│  GodAgent ←→ SemanticRouter ←→ UnifiedSelfModel           │
│  (任务分类)   (置信度路由)     (自模型)                    │
└──────────────────────┬────────────────────────────────────┘
                       │
┌──────────────────────▼────────────────────────────────────┐
│              L5 Cognition — Model Gateway                  │
│  ┌──────────┐ ┌──────────────┐ ┌────────────┐            │
│  │ CostGate │ │FallbackChain │ │ProviderPool│            │
│  │ (成本)   │ │ (降级)       │ │ (连接池)   │            │
│  └──────────┘ └──────────────┘ └────────────┘            │
│  ┌─────────────────┐ ┌─────────────────┐                  │
│  │SemanticRouter   │ │ ProviderRegistry│                  │
│  │(confidence)     │ │ (5 providers)   │                  │
│  └─────────────────┘ └─────────────────┘                  │
└──────────────────────┬────────────────────────────────────┘
                       │
┌──────────────────────▼────────────────────────────────────┐
│          L1 Action — Provider Abstraction + MCP           │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ │
│  │ Claude │ │ Codex  │ │Gemini  │ │ Ollama │ │MCP     │ │
│  │ Code   │ │ CLI    │ │CLI     │ │ Local  │ │Bridge  │ │
│  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ │
└──────────────────────────────────────────────────────────┘
```

### Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `ModelGateway` | `l5_cognition/nt_core_model_gateway.rs` | 统一网关入口 |
| `CostGate` | `l5_cognition/nt_core_model_gateway.rs` | 月预算 + 模型预算门控 |
| `FallbackChain` | `l5_cognition/nt_core_model_gateway.rs` | coding/reasoning/simple 三条降级链 |
| `ProviderPool` | `l5_cognition/nt_core_model_gateway.rs` | 连接池管理 |
| `SemanticRouter` | `l5_cognition/nt_core_semantic_router.rs` | confidence-based 语义路由 |
| `ProviderRegistry` | `l5_cognition/nt_core_byoa.rs` | 5 预设 Provider 注册 |
| `McpBridge` | `l1_action/nt_io/nt_io_mcp_bridge.rs` | MCP 协议桥接 |

### Flow

```
Task → GodAgent.classify() → SemanticRouter.route(confidence)
  ├─ high confidence → 首选模型 (CostGate 检查)
  ├─ low confidence → FallbackChain → 强模型
  └─ provider unavailable → ProviderPool.next() → retry
```

---

## 4. L5 Cognition Module Groups

| Group | Modules | Purpose |
|-------|---------|---------|
| **Consciousness** | consciousness_core, self_model, attention_manager | 意识核心 + 自模型 + 注意力 |
| **Reasoning** | semantic_router, task_dispatcher, circuit_breaker | 任务分类/路由/熔断 |
| **Strategy** | god_agent, hive, coordination_principles, agent_circuit_breaker | 多 Agent 协调 |
| **Evolution** | mind loops, experience, repair hooks | 自我进化闭环 |
| **Gateway** | model_gateway, cost_gate, fallback_chain, provider_pool | 统一模型网关 |
| **Math/Types** | hypercube, E8, EMDR | 数学基础 + 类型系统 |

---

## 5. Key Cross-Cutting Concerns

### SelfTest (L0)

- `SelfTest` trait 下沉到 L0，消除 60+ 跨层依赖
- 所有 L1-L6 模块通过 `impl SelfTest for ...` 注入测试能力

### EventBus (L0)

- 统一事件总线，所有层通过 event bus 通信
- 消除 5 处 EventBus 重复实现

### Agent Circuit Breaker (L3 → L5)

- Agent 行为控制 (steer→constrain→stop) 归入 L5 认知层
- 区别于 Service Circuit Breaker (L3 安全基础设施)

### SemanticRouter Merge

- `nt_infra_semantic_router` (旧) → `nt_core_semantic_router` (新)
- 消除 2 处重复，统一 confidence-based dispatch

---

## 6. Constellation Maturity

| Level | Name | Gate |
|-------|------|------|
| C0 | 编译通过 | `cargo build` |
| C1 | 单测通过 | `cargo test --lib` |
| C2 | 集成测试 | `cargo test` |
| C3 | Benchmark | `cargo bench` |
| C4 | 主流水线 | CI/CD green |
| C5 | 自愈/自适应 | SelfTest + auto-repair |

---

**Last updated**: 2026-09-17 (v0.25.5)

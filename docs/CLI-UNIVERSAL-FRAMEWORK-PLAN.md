# NeoTrix CLI 通用模型框架 — 实施方案

> **日期**: 2026-09-17
> **目标**: 统一所有外部 LLM 模型适配，消除冗余，修复扁平缺陷，纠正跨域错位
> **约束**: R-P1 (零 unsafe) / R-P42 (吸收不平行) / R-P79 (同 session 接线) / R-P84 (预检门)

---

## 一、现状审计 (Code Audit Results)

### 1.1 冗余热点 (Redundancy Hotspots)

| # | 冗余类型 | 数量 | 严重度 | 根因 |
|---|---------|------|--------|------|
| R1 | TaskType 重复定义 | **10 处** | 🔴 Critical | 各层独立定义，无单一事实源 |
| R2 | ModelRouter 重复实现 | **8+ 处** | 🔴 Critical | 3 代路由器未合并 |
| R3 | ModelTier 不兼容定义 | **3 套** | 🔴 Critical | Fast/Balanced/Powerful vs Frontier/Strong vs T0-T4 |
| R4 | RoutingStrategy 重复 | **3 处** | 🟡 High | 每个路由器自带策略枚举 |
| R5 | CapabilityRegistry 重复 | **4 处** | 🟡 High | l6_meta / l5_cognition / nt_file_ability / nt_core_capability_tree |
| R6 | CapabilityStatus/State/Metrics | **2 套完全相同** | 🟡 High | nt_file_ability 是 l6_meta 的逐字拷贝 |
| R7 | KnowledgeProvider trait | **2 处** | 🟢 Medium | neotrix-types + l2_perception 各定义一次 |
| R8 | dead_code 抑制 | **100+ 处** | 🟡 High | 系统性死代码积累无清理 |
| R9 | config.toml 分散 | **3+ 个文件** | 🟢 Medium | 无统一配置层级 |
| R10 | CLI/Tauri 命令重叠 | **30+90 命令** | 🟢 Medium | 无共享命令定义 |

### 1.2 扁平缺陷 (Flat Defects)

| # | 缺陷 | 影响 | 位置 |
|---|------|------|------|
| D1 | 10 套 TaskType 互不兼容 | 跨层任务分类不一致 | 全局 |
| D2 | 3 套 ModelTier 无法互转 | 路由决策不一致 | l5/l1 |
| D3 | 无统一 streaming 接口 | 每个适配器自实现 SSE 解析 | l1_action/nt_io |
| D4 | 无统一 tool calling 抽象 | OpenAI/Anthropic/Gemini 格式差异未屏蔽 | l1_action/nt_io |
| D5 | 无 cost-aware 路由 | Axiom A1 (Cost-Aware Routing) 未落地 | l5_cognition |
| D6 | context compaction 缺失 | Axiom A2 (Context as Scarce Resource) 未落地 | l5_cognition |
| D7 | config 加载无层级 | 全局/项目/本地/环境变量未合并 | config.rs |

### 1.3 跨域错位 (Cross-Domain Misalignment)

| # | 错位 | 描述 |
|---|------|------|
| M1 | L1 适配器 ≠ L5 路由器 | L1 有 `model_routing.rs` + `learned_router.rs`，L5 有 `nt_core_model_router.rs` + `nt_core_model_gateway.rs`，两者独立决策 |
| M2 | SEAL 路由器 vs 主路由器 | `seal_core/model_router.rs` 有独立的 T0-T4 分级，与主路由的 Fast/Balanced/Powerful 不兼容 |
| M3 | GWT salience vs 模型路由 | `nt_core_gwt/cost_router.rs` 有独立的 cost router，与 `nt_core_model_router.rs` 的 cost-aware 路由重复 |
| M4 | Tauri ProviderManager vs Core | `src-tauri/src/service/provider_manager.rs` 有独立的 Provider 生命周期管理，与 core 的 `nt_io_provider` 不同步 |

---

## 二、外部参考架构 (Reference Architecture)

### 2.1 五大工具模式提炼

| 模式 | 来源 | NeoTrix 映射 |
|------|------|-------------|
| **Vercel AI SDK LanguageModelV2** | OpenCode (75+ providers) | → `UnifiedModelAdapter` trait |
| **5-Level Context Compaction** | Claude Code (3960 行 TS) | → `ContextCompactor` in L6 |
| **Per-Turn ML Router** | Cursor (600K+ labeled requests) | → `GwtSalienceRouter` with cost weight |
| **3-Tier Model System** | Aider (Main/Weak/Editor) | → `ModelTier` unified enum |
| **Hub-Spoke Multi-Agent** | Cline (gRPC/protobuf) | → `SubAgentRegistry` already exists |
| **LiteLLM Universal Adapter** | LiteLLM (100+ providers) | → OpenAI-format canonical wire type |
| **IETF SSE Standard Draft** | IETF (2025) | → `SseEventParser` unified streaming |
| **Cascade Routing** | ETH Zurich (ICML 2025) | → GWT escalation via salience threshold |

### 2.2 LLM API 统一格式 (Wire Format)

**结论: OpenAI Chat Completions 是事实标准**。DeepSeek/Ollama/Groq/Together 原生兼容，Anthropic/Gemini 需适配层。

```
Canonical Request:
  model: String
  messages: Vec<Message>          // role + content + tool_calls
  tools: Vec<Tool>                // name + description + parameters (JSON Schema)
  tool_choice: ToolChoice         // auto | none | required | named
  temperature: Option<f64>
  max_tokens: Option<u32>
  stream: bool
  stream_options: Option<StreamOptions>

Canonical Response:
  id: String
  choices: Vec<Choice>            // message + finish_reason
  usage: Usage                    // prompt_tokens + completion_tokens

Streaming (SSE):
  data: {"choices": [{"delta": {"content": "..."}}]}
  data: [DONE]
```

**Provider 适配差异**:

| 差异点 | OpenAI | Anthropic | Gemini |
|--------|--------|-----------|--------|
| system 位置 | message role | 顶层 `system` 字段 | `systemInstruction` |
| max_tokens | optional | **required** | optional |
| tool 参数名 | `parameters` | `input_schema` | `parameters` |
| tool call 格式 | `tool_calls[].function` | `content[].type=tool_use` | `functionCalls[]` |
| tool result 格式 | `role: tool` | `type: tool_result` | `functionResponse` |
| streaming 事件 | `data: {delta}` | `event: content_block_delta` | `event: content.delta` |
| 流终止 | `data: [DONE]` | `event: message_stop` | `event: interaction.complete` |
| thinking | `reasoning_effort` | `thinking: {type, budget}` | 无原生支持 |
| 缓存 | 隐式自动 | 显式 `cache_control` | 隐式自动 |

---

## 三、目标架构 (Target Architecture)

### 3.1 统一模型框架核心层

```
                    ┌─────────────────────────────────────┐
                    │        UnifiedModelRouter            │
                    │  (GWT salience + cost weight +      │
                    │   cascade escalation + circuit       │
                    │   breaker)                           │
                    └──────────┬──────────────────────────┘
                               │
                    ┌──────────▼──────────────────────────┐
                    │      UnifiedModelAdapter            │
                    │  trait:                              │
                    │    fn send(request) → Response       │
                    │    fn stream(request) → Stream       │
                    │    fn count_tokens(text) → usize     │
                    └──────────┬──────────────────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          │                    │                    │
  ┌───────▼──────┐   ┌────────▼───────┐   ┌───────▼──────┐
  │ OpenAIAdapter│   │AnthropicAdapter│   │ GeminiAdapter│
  │ (native)     │   │ (translate)    │   │ (translate)  │
  └──────────────┘   └────────────────┘   └──────────────┘
          │                    │                    │
  ┌───────▼──────┐   ┌────────▼───────┐   ┌───────▼──────┐
  │  Ollama      │   │  DeepSeek      │   │  Custom      │
  │  (compat)    │   │  (compat)      │   │  (OpenAI)    │
  └──────────────┘   └────────────────┘   └──────────────┘
```

### 3.2 消除的冗余 (Redundancy Elimination)

| 消除项 | 从 | 到 | 方式 |
|--------|-----|-----|------|
| 10× TaskType | 10 个 enum | 1 个 canonical + From 转换 | neotrix-types 单一事实源 |
| 8× ModelRouter | 8 个实现 | 1 个 `UnifiedModelRouter` | 吸收到 L5 GWT |
| 3× ModelTier | 3 个 enum | 1 个 canonical | neotrix-types |
| 3× RoutingStrategy | 3 个 enum | 1 个 canonical | neotrix-types |
| 4× CapabilityRegistry | 4 个实现 | 1 个 `CapabilityRegistry` | l6_meta 为事实源 |
| 2× CapabilityStatus | 2 个相同 enum | 1 个 | 从 l6_meta re-export |
| config 分散 | 3+ config.toml | 1 层级配置 | config.rs 统一加载 |
| dead_code | 100+ suppressions | 删除或接线 | Sprint 4 清理 |

---

## 四、Sprint 计划 (Execution Plan)

### Sprint 0: 类型统一 (Type Unification) — 2 天

**目标**: 消除 R1 (TaskType) + R3 (ModelTier) + R4 (RoutingStrategy)

| 任务 | 文件 | 变更 | 验证 |
|------|------|------|------|
| T0.1 | `crates/neotrix-types/src/core/nt_core_knowledge/types.rs` | 扩展 TaskType 为超集 (保留旧名+追加新变体) | `cargo check -p neotrix-types` |
| T0.2 | `crates/neotrix-types/src/core/nt_core_model.rs` (新建) | 定义统一 ModelTier + RoutingStrategy | `cargo check -p neotrix-types` |
| T0.3 | `neotrix-core/src/l5_cognition/nt_core_model_router.rs` | 删除本地 TaskType/ModelTier，re-export neotrix-types | `cargo check -p neotrix --lib` |
| T0.4 | `neotrix-core/src/l5_cognition/nt_core_model_unified.rs` | 删除本地 TaskType，re-export neotrix-types | 同上 |
| T0.5 | `neotrix-core/src/l1_action/nt_act/nt_act_orchestrator/critic.rs` | 删除本地 TaskType，import neotrix-types | 同上 |
| T0.6 | `neotrix-core/src/l1_action/nt_act/nt_act_orchestrator/planner.rs` | 同上 | 同上 |
| T0.7 | `neotrix-core/src/l5_cognition/nt_core/nt_consciousness_core/resource_router.rs` | 同上 | 同上 |
| T0.8 | `neotrix-core/src/l1_action/nt_io/nt_io_provider/common/generation_classifier.rs` | 同上 | 同上 |
| T0.9 | `neotrix-core/src/l1_action/nt_io/universal_model/traits.rs` | 同上 | 同上 |
| T0.10 | `neotrix-core/src/l1_action/nt_act/actions/infra/model_router.rs` | 同上 | 同上 |
| T0.11 | `neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/model_router.rs` | 同上 | 同上 |
| T0.12 | `neotrix-core/src/l5_cognition/nt_core_god_agent.rs` | 本地 TaskType → From<canonical> | 同上 |
| T0.13 | `neotrix-core/src/l1_action/nt_act/actions/orchestration/production_pipeline.rs` | 保留本地 (领域专用) + From<canonical> | 同上 |

**验收**: `cargo check -p neotrix --lib` 0 新增 error；grep `enum TaskType` 仅 neotrix-types 1 处 + 领域专用 1 处 (production_pipeline)

### Sprint 1: 统一模型适配器 (Unified Adapter) — 3 天

**目标**: 消除 R2 (8× ModelRouter) + D3 (streaming) + D4 (tool calling)

| 任务 | 文件 | 变更 | 验证 |
|------|------|------|------|
| T1.1 | `neotrix-core/src/l5_cognition/nt_core_model_unified.rs` | 定义 `UnifiedModelAdapter` trait + `ModelRequest`/`ModelResponse` | 编译通过 |
| T1.2 | `neotrix-core/src/l1_action/nt_io/nt_io_provider/` | 实现 `OpenAIAdapter: UnifiedModelAdapter` | 编译+单测 |
| T1.3 | 同上 | 实现 `AnthropicAdapter: UnifiedModelAdapter` (system 字段转换) | 编译+单测 |
| T1.4 | 同上 | 实现 `GeminiAdapter: UnifiedModelAdapter` (contents→messages 转换) | 编译+单测 |
| T1.5 | 同上 | 实现 `OllamaAdapter: UnifiedModelAdapter` (OpenAI-compat pass-through) | 编译+单测 |
| T1.6 | 同上 | 实现 `SseEventParser` — 统一 OpenAI/Anthropic/Gemini SSE 解析 | 编译+单测 |
| T1.7 | 同上 | 实现 `ToolCallConverter` — 统一 tool 定义/调用/结果格式 | 编译+单测 |
| T1.8 | 同上 | 实现 `TokenCounter` — tiktoken-rs + provider fallback | 编译+单测 |

**验收**: 所有 adapter 通过统一的 `test_adapter_roundtrip` 测试

### Sprint 2: 统一路由器 (Unified Router) — 3 天

**目标**: 消除 M1-M4 (跨域错位) + D5 (cost-aware)

| 任务 | 文件 | 变更 | 验证 |
|------|------|------|------|
| T2.1 | `neotrix-core/src/l5_cognition/nt_core_model_router.rs` | 重写为 `UnifiedModelRouter` — 吸收 gateway/seal/gwt 路由逻辑 | 编译+单测 |
| T2.2 | 同上 | 实现 cost-aware scoring: `quality(λ) × cost_weight` (Axiom A1) | 编译+单测 |
| T2.3 | 同上 | 实现 cascade escalation: salience < threshold → 升级模型 | 编译+单测 |
| T2.4 | 同上 | 实现 circuit breaker: 3 consecutive failures → fallback | 编译+单测 |
| T2.5 | 同上 | 删除 `nt_core_model_gateway.rs` 的独立路由逻辑 | 编译 |
| T2.6 | 同上 | 删除 `seal_core/model_router.rs` 的独立 T0-T4 | 编译 |
| T2.7 | 同上 | 删除 `nt_core_gwt/cost_router.rs` 的独立 cost router | 编译 |
| T2.8 | 同上 | 删除 `l1_action/nt_io/model_routing.rs` 的独立路由 | 编译 |
| T2.9 | 同上 | 删除 `l1_action/nt_act/actions/infra/model_router.rs` 的独立路由 | 编译 |

**验收**: 仅 `UnifiedModelRouter` 1 处路由逻辑；`grep -r "fn route\|fn select_model\|fn choose_model" --include="*.rs"` 仅 1 处

### Sprint 3: Context Compaction + Config 统一 — 2 天

**目标**: 消除 D6 (context compaction) + D7 (config) + R9 (config 分散)

| 任务 | 文件 | 变更 | 验证 |
|------|------|------|------|
| T3.1 | `neotrix-core/src/l6_meta/nt_context_compactor.rs` (新建) | 3 级压缩: L1 Tool Result Pruning → L2 History Snip → L3 Summary | 编译+单测 |
| T3.2 | `neotrix-core/src/config.rs` | 统一配置层级: 全局 → 项目 → 本地 → 环境变量 | 编译+单测 |
| T3.3 | `neotrix-core/src/config.rs` | `ModelConfig` 结构: provider/model/api_key/base_url/cost_weights | 编译 |
| T3.4 | `neotrix-core/src/config.rs` | `model list` / `model set` CLI 统一入口 | 编译 |

**验收**: `neotrix config model list` 显示所有已配置 provider；context compaction 单测通过

### Sprint 4: 冗余清理 + Dead Code — 2 天

**目标**: 消除 R5-R8 (CapabilityRegistry 重复 + dead_code)

| 任务 | 文件 | 变更 | 验证 |
|------|------|------|------|
| T4.1 | `neotrix-core/src/l1_action/nt_io/universal_model/` | nt_file_ability 的 CapabilityStatus/State/Metrics 改为 re-export l6_meta | 编译 |
| T4.2 | `neotrix-core/src/l2_perception/nt_core_knowledge/types.rs` | KnowledgeProvider 改为 re-export neotrix-types | 编译 |
| T4.3 | 各模块 | 删除 `#[allow(dead_code)]` 中确认无消费者的项 | 编译 |
| T4.4 | `neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs.bak` | 删除 .bak 旧文件 | — |
| T4.5 | `neotrix-core/src/l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs.bak` | 同上 | — |

**验收**: `grep -r "#\[allow(dead_code)\]" --include="*.rs" | wc -l` 减少 50%+

### Sprint 5: CLI 统一 + 生产接线 — 2 天

**目标**: 消除 R10 (CLI/Tauri 重叠) + R-P79 (生产接线)

| 任务 | 文件 | 变更 | 验证 |
|------|------|------|------|
| T5.1 | `neotrix-core/src/cli/commands/model_cmds.rs` | 统一 model 子命令: list/set/test/status | 编译 |
| T5.2 | 同上 | `model test <provider> <model>` — 测试 adapter 连通性 | 编译+运行 |
| T5.3 | 同上 | `model route <query>` — 展示路由决策过程 | 编译+运行 |
| T5.4 | `neotrix-core/src/l5_cognition/nt_core_model_router.rs` | 接线到 EventBus: 路由决策 → 意识层消费 | 编译 |
| T5.5 | 同上 | 接线到 KB: 路由日志 → experience namespace | 编译 |

**验收**: `neotrix model route "write a Python function"` 输出路由决策 + 成本估算

---

## 五、核心路线任务清单 (Priority Roadmap)

### P0 — 必须完成 (阻塞后续)

| ID | 任务 | Sprint | 预估 | 依赖 |
|----|------|--------|------|------|
| P0.1 | TaskType 10→1 统一 | S0 | 2h | — |
| P0.2 | ModelTier 3→1 统一 | S0 | 1h | — |
| P0.3 | RoutingStrategy 3→1 统一 | S0 | 1h | — |
| P0.4 | UnifiedModelAdapter trait 定义 | S1 | 2h | P0.1 |
| P0.5 | OpenAI adapter 实现 | S1 | 3h | P0.4 |
| P0.6 | Anthropic adapter 实现 | S1 | 4h | P0.4 |
| P0.7 | Gemini adapter 实现 | S1 | 4h | P0.4 |
| P0.8 | SseEventParser 统一解析 | S1 | 3h | P0.5-P0.7 |
| P0.9 | ToolCallConverter 统一格式 | S1 | 2h | P0.5-P0.7 |
| P0.10 | UnifiedModelRouter 重写 | S2 | 4h | P0.10 |

### P1 — 高优先级 (核心价值)

| ID | 任务 | Sprint | 预估 | 依赖 |
|----|------|--------|------|------|
| P1.1 | Cost-aware scoring (Axiom A1) | S2 | 3h | P0.10 |
| P1.2 | Cascade escalation | S2 | 2h | P0.10 |
| P1.3 | Circuit breaker | S2 | 2h | P0.10 |
| P1.4 | 删除 8 个旧路由器 | S2 | 2h | P0.10, P1.1-P1.3 |
| P1.5 | Context Compactor 3 级 | S3 | 4h | — |
| P1.6 | Config 统一层级 | S3 | 3h | — |
| P1.7 | model CLI 统一 | S5 | 3h | P0.10 |

### P2 — 中优先级 (质量提升)

| ID | 任务 | Sprint | 预估 | 依赖 |
|----|------|--------|------|------|
| P2.1 | CapabilityRegistry 4→1 | S4 | 3h | — |
| P2.2 | dead_code 清理 50%+ | S4 | 4h | — |
| P2.3 | TokenCounter (tiktoken-rs) | S1 | 2h | — |
| P2.4 | 路由决策 → EventBus 接线 | S5 | 2h | P0.10 |
| P2.5 | 路由日志 → KB 接线 | S5 | 2h | P0.10 |

### P3 — 低优先级 (长期)

| ID | 任务 | Sprint | 预估 | 依赖 |
|----|------|--------|------|------|
| P3.1 | Per-turn ML router (Cursor 模式) | Future | 2d | P1.1 |
| P3.2 | Tauri/Core 命令共享 | Future | 1d | — |
| P3.3 | IETF SSE 标准对齐 | Future | 1d | P0.8 |
| P3.4 | 模型 metadata 远程目录 (models.dev 模式) | Future | 2d | — |

---

## 六、风险分析

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 类型统一导致大量编译错误 | 高 | 中 | 保留旧 variant 名 + 添加新 variant，渐进迁移 |
| 旧路由器删除后功能回归 | 中 | 高 | Sprint 2 每步编译验证 + 单测覆盖 |
| Anthropic/Gemini 适配层遗漏 edge case | 中 | 中 | 用真实 API 端到端测试 (非 mock) |
| dead_code 删除误删活跃代码 | 低 | 高 | R-P76 四重验证: import/str dispatch/CLI/pub item |
| 外部 API 变更导致适配器失效 | 低 | 中 | Adapter trait 设计为 provider-specific 扩展点 |

---

## 七、成功度量

| 指标 | 当前 | 目标 | 度量方式 |
|------|------|------|---------|
| TaskType 定义数 | 10 | 1 (+1 领域专用) | `grep "enum TaskType" --include="*.rs"` |
| ModelRouter 实现数 | 8+ | 1 | `grep "fn route\|fn select_model" --include="*.rs"` |
| ModelTier 定义数 | 3 | 1 | `grep "enum ModelTier" --include="*.rs"` |
| CapabilityRegistry 数 | 4 | 1 | `grep "struct CapabilityRegistry" --include="*.rs"` |
| dead_code 抑制数 | 100+ | <50 | `grep "#\[allow(dead_code)\]" --include="*.rs" \| wc -l` |
| 支持 provider 数 | 4 (分散) | 6+ (统一) | `model list` 输出 |
| 路由决策可观测性 | 0 | 100% | EventBus + KB 接线 |

---

## 八、实施顺序 (Dependency Graph)

```
Sprint 0 (类型统一)
    │
    ├──→ Sprint 1 (统一适配器)
    │        │
    │        ├──→ Sprint 2 (统一路由器)
    │        │        │
    │        │        ├──→ Sprint 5 (CLI + 接线)
    │        │        │
    │        │        └──→ Sprint 4 (冗余清理)
    │        │
    │        └──→ Sprint 3 (Context + Config)
    │
    └──→ Sprint 4 (可并行部分)
```

**关键路径**: Sprint 0 → Sprint 1 → Sprint 2 → Sprint 5
**总预估**: 14 人天 (约 3 周)

---

## 九、与 AGENTS.md 架构对齐

| NeoTrix 层 | 统一框架组件 | 职责 |
|------------|-------------|------|
| L1 Action | `UnifiedModelAdapter` implementations | 实际 API 调用 |
| L2 Perception | `SseEventParser` | 流式事件解析 |
| L5 Cognition | `UnifiedModelRouter` | 路由决策 + cost-aware |
| L6 Meta | `ContextCompactor` | 上下文压缩 |
| NT-SHIELD | `CircuitBreaker` | 容错 + 降级 |
| NT-IO | CLI `model` 子命令 | 用户接口 |
| NT-CORE | Config 统一加载 | 配置管理 |

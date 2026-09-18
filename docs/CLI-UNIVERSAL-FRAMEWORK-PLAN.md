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

### Sprint 6: Bend 语言集成 — 11 天 (可与 S3-S5 并行)

**目标**: 引入 Bend LAWS.bend 证明系统 + 并行计算核心，将 R-P1~R-P80 从注释变成数学证明

**前提**: S0 (TaskType 统一) + Bend 编译器安装验证

| 任务 | 文件 | 变更 | 验证 |
|------|------|------|------|
| T6.1 | 系统 | 安装 Bend (`curl -fsSL https://bend-lang.com/install.sh \| sh`) + 验证 FFI 桥接 | `bend --version` + hello world |
| T6.2 | `bend/laws/neo_trix_laws.bend` (新建) | LAWS.bend 形式化 R-P1~R-P10 核心规则 | `bend check laws/neo_trix_laws.bend` |
| T6.3 | `bend/compute/e8_encode.bend` (新建) | E8 HyperCube 编码迁移到 Bend (并行) | `bend run compute/e8_encode.bend` |
| T6.4 | `bend/compute/vector_search.bend` (新建) | 向量搜索并行化 | benchmark vs Rust |
| T6.5 | `bend/compute/seal_train.bend` (新建) | SEAL 训练循环并行化 | benchmark vs Rust |
| T6.6 | `bend/compute/route_score.bend` (新建) | 模型路由评分并行化 | benchmark vs Rust |
| T6.7 | `neotrix-core/src/l5_cognition/nt_bend_bridge.rs` (新建) | Rust↔Bend FFI 桥接层 (C 动态库) | `cargo check -p neotrix --lib` |
| T6.8 | `neotrix-core/build.rs` | 编译 Bend→C→动态库 + 链接 | `cargo build -p neotrix` |

**验收**:
- `bend check laws/neo_trix_laws.bend` 通过
- E8 编码 Bend 版本 vs Rust 版本性能对比 (目标: 10x+ on GPU)
- `cargo check -p neotrix --lib` 0 新增 error

**详细设计**: 见附录 M

### Sprint 7: Trendshift 项目吸收 — 5 天 (Sprint 1-3 完成后)

**目标**: 从 Trendshift 全榜趋势中吸收高价值项目的核心模式

**前提**: S1 (统一适配器) + S2 (统一路由器)

| 任务 | 项目 | 吸收点 | NeoTrix 映射 | 预估 |
|------|------|--------|-------------|------|
| T7.1 | alibaba/open-code-review (11.4k★) | hybrid 审查架构 (deterministic + LLM) | NT-MIND rev-officer 增强 | 2天 |
| T7.2 | cloudflare/security-audit-skill (7.5k★) | 多阶段安全审计 (独立验证 + 机器可读发现) | NT-SHIELD 安全审计 | 1天 |
| T7.3 | Graphify-Labs/graphify (119.6k★) | AST→知识图谱 (本地确定性解析) | NT-MEMORY 知识图谱 | 2天 |
| T7.4 | mattpocock/skills (262.4k★) | 技能标准化框架 | SKILL-SPEC.md 对齐 | 1天 |
| T7.5 | deepseek-ai/deepseek-harness (21.6k★) | harness 优化模式 | NT-CORE 性能优化 | 1天 |
| T7.6 | JustVugg/colibri (9.2k★) | 744B MoE 本地推理 (25GB RAM) | NT-WORLD 本地模型路由参考 | 1天 |

**验收**:
- open-code-review hybrid 审查模式原型验证
- graphify AST→知识图谱管线与 NT-MEMORY 集成验证
- 所有吸收遵循 R-P42 (吸收强化现有节点，禁止平行适配器)

**详细设计**: 见附录 N

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

### P4 — Bend 集成 (Sprint 6, 可与 S3-S5 并行)

| ID | 任务 | Sprint | 预估 | 依赖 |
|----|------|--------|------|------|
| P4.1 | Bend 安装 + FFI 验证 | S6 | 1d | — |
| P4.2 | LAWS.bend 形式化 R-P1~R-P10 | S6 | 2d | P4.1 |
| P4.3 | E8 HyperCube Bend 并行化 | S6 | 3d | P4.1 |
| P4.4 | 向量搜索 Bend 并行化 | S6 | 2d | P4.1 |
| P4.5 | SEAL 训练 Bend 并行化 | S6 | 2d | P4.1 |
| P4.6 | 模型路由评分 Bend 并行化 | S6 | 1d | P0.1, P4.1 |
| P4.7 | Rust↔Bend FFI 桥接层 | S6 | 1d | P4.3-P4.6 |

### P5 — Trendshift 项目吸收 (Sprint 7, S1-S3 完成后)

| ID | 任务 | Sprint | 预估 | 依赖 |
|----|------|--------|------|------|
| P5.1 | open-code-review hybrid 审查吸收 | S7 | 2d | P0.10 |
| P5.2 | security-audit-skill 多阶段审计吸收 | S7 | 1d | — |
| P5.3 | graphify AST→知识图谱吸收 | S7 | 2d | — |
| P5.4 | mattpocock/skills 技能标准化对齐 | S7 | 1d | — |
| P5.5 | deepseek-harness 优化模式吸收 | S7 | 1d | — |
| P5.6 | colibri 本地推理参考吸收 | S7 | 1d | — |

---

## 六、风险分析

### 6.1 核心框架风险 (S0-S5)

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 类型统一导致大量编译错误 | 高 | 中 | 保留旧 variant 名 + 添加新 variant，渐进迁移 |
| 旧路由器删除后功能回归 | 中 | 高 | Sprint 2 每步编译验证 + 单测覆盖 |
| Anthropic/Gemini 适配层遗漏 edge case | 中 | 中 | 用真实 API 端到端测试 (非 mock) |
| dead_code 删除误删活跃代码 | 低 | 高 | R-P76 四重验证: import/str dispatch/CLI/pub item |
| 外部 API 变更导致适配器失效 | 低 | 中 | Adapter trait 设计为 provider-specific 扩展点 |

### 6.2 Bend 集成风险 (S6)

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| Bend 编译器 bug (99% AI-written) | 高 | 高 | 仅用于计算密集模块，Rust 做 fallback |
| FFI 性能开销超预期 | 中 | 中 | 批量调用减少跨语言次数，benchmark 验证 |
| Bend breaking changes (v2 年轻) | 高 | 中 | 锁定版本 + 抽象层隔离 (nt_bend_bridge.rs) |
| 团队学习成本 | 中 | 低 | Bend 语法类似 Python，渐进引入 |
| Bend 无 HTTP/TLS/JSON | 确定 | 低 | Rust 处理 IO，Bend 仅计算 |

### 6.3 Trendshift 吸收风险 (S7)

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 吸收变成平行适配器 (违反 R-P42) | 中 | 高 | 强制检查: 吸收必须增强现有节点 |
| 吸收过时/不稳定项目 | 低 | 中 | 仅吸收 C4+ 成熟度项目 |
| 吸收后忘记接线生产 (违反 R-P79) | 中 | 高 | 同 session 接线，不延期 |
| Trendshift 项目 Star 虚高 | 低 | 低 | 交叉验证 GitHub commit activity |

---

## 七、成功度量

### 7.1 核心指标 (S0-S5)

| 指标 | 当前 | 目标 | 度量方式 |
|------|------|------|---------|
| TaskType 定义数 | 10 | 1 (+1 领域专用) | `grep "enum TaskType" --include="*.rs"` |
| ModelRouter 实现数 | 8+ | 1 | `grep "fn route\|fn select_model" --include="*.rs"` |
| ModelTier 定义数 | 3 | 1 | `grep "enum ModelTier" --include="*.rs"` |
| CapabilityRegistry 数 | 4 | 1 | `grep "struct CapabilityRegistry" --include="*.rs"` |
| dead_code 抑制数 | 100+ | <50 | `grep "#\[allow(dead_code)\]" --include="*.rs" \| wc -l` |
| 支持 provider 数 | 4 (分散) | 6+ (统一) | `model list` 输出 |
| 路由决策可观测性 | 0 | 100% | EventBus + KB 接线 |

### 7.2 Bend 指标 (S6)

| 指标 | 目标 | 度量方式 |
|------|------|---------|
| LAWS.bend 规则覆盖 | R-P1~R-P10 形式化 | `bend check laws/neo_trix_laws.bend` |
| E8 编码并行加速 | 10x+ (GPU) | benchmark vs Rust baseline |
| 向量搜索并行加速 | 5x+ | benchmark vs Rust baseline |
| FFI 调用延迟 | <1ms per call | benchmark |

### 7.3 Trendshift 指标 (S7)

| 指标 | 目标 | 度量方式 |
|------|------|---------|
| 吸收项目数 | 3+ 个高价值项目 | 代码审查确认 |
| R-P42 合规率 | 100% (无平行适配器) | `cargo check` + 代码审查 |
| R-P79 合规率 | 100% (同 session 接线) | 每个吸收项有对应生产接线 |

---

## 八、实施顺序 (Dependency Graph)

```
                    ┌─────────────────────────────────────────────────┐
                    │           Sprint 0: 类型统一 (已完成 T0.1)       │
                    │  TaskType ✅ | ModelTier | RoutingStrategy      │
                    └────────────────────┬────────────────────────────┘
                                         │
                    ┌────────────────────▼────────────────────────────┐
                    │         Sprint 1: 统一适配器                     │
                    │  UnifiedModelAdapter + SseEventParser +         │
                    │  ToolCallConverter + 4 Provider Adapters        │
                    └────┬───────────┬───────────┬───────────────────┘
                         │           │           │
            ┌────────────▼──┐  ┌─────▼──────┐  ┌▼─────────────────┐
            │ Sprint 2:     │  │ Sprint 3:  │  │ Sprint 4:        │
            │ 统一路由器    │  │ Compaction │  │ 冗余清理         │
            │ CostAware+CB  │  │ + Config   │  │ Dead Code 50%+   │
            └───────┬──────┘  └─────┬──────┘  └──────────────────┘
                    │                │
            ┌───────▼────────────────▼──────┐
            │       Sprint 5: CLI + 接线     │
            │  model CLI + EventBus + KB     │
            └───────────────┬───────────────┘
                            │
              ┌─────────────┼─────────────────────┐
              │             │                     │
   ┌──────────▼──┐  ┌──────▼──────┐  ┌──────────▼──────────┐
   │ Sprint 6:   │  │ Sprint 7:   │  │ Future:             │
   │ Bend 集成   │  │ Trendshift  │  │ Per-turn ML Router  │
   │ LAWS+并行   │  │ 项目吸收    │  │ Tauri/Core 共享     │
   │ (可与S3-S5  │  │ (S1-S3后)   │  │ IETF SSE           │
   │  并行)      │  │             │  │ models.dev 目录     │
   └─────────────┘  └─────────────┘  └─────────────────────┘

关键路径: S0 → S1 → S2 → S5 (14 人天)
并行路径: S3 ∥ S2, S4 ∥ S2/S3, S6 ∥ S3-S5, S7 after S1-S3
总预估: 25 人天 (约 5 周, 含并行优化)
```

---

## 九、与 AGENTS.md 架构对齐

### 9.1 核心框架组件 (S0-S5)

| NeoTrix 层 | 统一框架组件 | 职责 |
|------------|-------------|------|
| L1 Action | `UnifiedModelAdapter` implementations | 实际 API 调用 |
| L2 Perception | `SseEventParser` | 流式事件解析 |
| L5 Cognition | `UnifiedModelRouter` | 路由决策 + cost-aware |
| L6 Meta | `ContextCompactor` | 上下文压缩 |
| NT-SHIELD | `CircuitBreaker` 容错 + 降级 |
| NT-IO | CLI `model` 子命令 | 用户接口 |
| NT-CORE | Config 统一加载 | 配置管理 |

### 9.2 Bend 集成组件 (S6)

| NeoTrix 层 | Bend 组件 | 职责 |
|------------|----------|------|
| L5 Cognition | `nt_bend_bridge.rs` | Rust↔Bend FFI 桥接 |
| L5 Cognition | `bend/compute/e8_encode.bend` | E8 HyperCube 并行计算 |
| L5 Cognition | `bend/compute/vector_search.bend` | 向量搜索并行化 |
| L5 Cognition | `bend/compute/seal_train.bend` | SEAL 训练并行化 |
| L5 Cognition | `bend/compute/route_score.bend` | 模型路由评分并行化 |
| L6 Meta | `bend/laws/neo_trix_laws.bend` | LAWS.bend 规则形式化 |

### 9.3 Trendshift 吸收组件 (S7)

| NeoTrix 层 | 吸收增强 | 来源项目 |
|------------|---------|---------|
| L5 Cognition (NT-MIND) | `HybridReviewer` 增强 rev-officer | alibaba/open-code-review |
| L3 Embodiment (NT-SHIELD) | 6 阶段安全审计管线 | cloudflare/security-audit-skill |
| L1 Action (NT-MEMORY) | `AstToKnowledgeGraph` 增强图谱 | Graphify-Labs/graphify |
| L5 Cognition (NT-MIND) | SKILL-SPEC.md 对齐 | mattpocock/skills |
| L5 Cognition (NT-CORE) | harness 优化模式 | deepseek-ai/deepseek-harness |
| L2 Perception (NT-WORLD) | 本地推理参考 | JustVugg/colibri |

---

## 附录 A: Sprint 0 精确文件改动规格

### A.1 TaskType 扩展 (neotrix-types) — ✅ 已完成 (T0.1)

**文件**: `crates/neotrix-types/src/core/nt_core_knowledge/types.rs`

**策略**: 保留全部 12 个旧 variant 名 + 追加新 variant。不删除任何已有 variant。

**关键决策**: 使用 `Custom` (unit variant) 而非 `Custom(String)`，以保留 `Copy` trait。`Custom(String)` 会导致 `#[repr(inttype)]` 要求、`as_str()` 生命周期问题、以及 bank_impl 中的 move 错误。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    // ── 旧 12 variants (判别值保留) ──
    General = 0,
    Design = 1,
    CodeAnalysis = 2,
    CodeGeneration = 3,
    CodeReview = 4,
    Security = 5,
    Planning = 6,
    Reflection = 7,
    UIDesign = 8,
    Research = 9,
    Learning = 10,
    MetaCognition = 50,

    // ── 从 model_router 合并 ──
    Chat,
    Math,
    Creative,
    DataProcessing,
    Multimodal,

    // ── 从 resource_router 合并 ──
    SimpleQA,
    ComplexReasoning,
    CreativeWriting,
    KnowledgeRetrieval,
    DataAnalysis,

    // ── 从 generation_classifier 合并 ──
    Extraction,
    Summarization,
    ToolUse,

    // ── 从 universal_model 合并 ──
    Completion,
    Embedding,
    Reranking,
    ImageGeneration,
    AudioGeneration,
    VideoGeneration,

    // ── 从 god_agent 合并 ──
    Debugging,
    Architecture,
    Documentation,
    Testing,
    SystemAdmin,
    FileOperations,
    AgentTask,

    // ── 扩展 ──
    Custom,  // unit variant — 保留 Copy trait (非 Custom(String))
}
```

**实现状态**: ✅ 已持久化，38 变体 + Custom，`as_str()` + `from_description()` 方法已实现。

### A.2 删除的本地 TaskType (8 处)

| 文件 | 行号 | 状态 | 动作 |
|------|------|------|------|
| `l5_cognition/nt_core_model_router.rs` | 77-95 | ⬜ 待办 | 删除 → `pub use neotrix_types::core::TaskType;` |
| `l5_cognition/nt_core_model_unified.rs` | 16 | ✅ 已完成 | 已是 re-export，无需改动 |
| `l1_action/nt_act/nt_act_orchestrator/critic.rs` | 3-13 | ✅ 已完成 | 删除本地 TaskType |
| `l1_action/nt_act/nt_act_orchestrator/planner.rs` | 4-14 | ✅ 已完成 | 删除本地 TaskType |
| `l5_cognition/nt_core/nt_consciousness_core/resource_router.rs` | 116-131 | ⬜ 待办 | 删除 → `pub use neotrix_types::core::TaskType;` |
| `l1_action/nt_io/nt_io_provider/common/generation_classifier.rs` | 16-33 | ⬜ 待办 | 删除 → `pub use neotrix_types::core::TaskType;` |
| `l1_action/nt_io/universal_model/traits.rs` | 131-139 | ⬜ 待办 | 删除 → `pub use neotrix_types::core::TaskType;` |
| `l5_cognition/nt_core_god_agent.rs` | 96-117 | ⬜ 待办 | 本地 TaskType → From<canonical> |

### A.3 需更新的 match arm 映射 — ✅ 已完成

| 文件 | 状态 | 动作 |
|------|------|------|
| `seal_core/self_edit.rs:82` | ✅ 已完成 | 添加 wildcard arms 映射新变体 |
| `consciousness/consciousness_bridge.rs:56` | ✅ 已完成 | 添加 wildcard arms 映射新变体 |

| 旧 variant | 新 variant | 状态 | 影响文件 |
|-----------|-----------|------|---------|
| `TaskType::Coding` | `TaskType::CodeGeneration` | ✅ 已完成 | model_router.rs (28处), model_unified tests (3处) |
| `TaskType::GeneralChat` | `TaskType::General` | ⬜ 待办 | god_agent.rs (5处) |
| `TaskType::Code` | `TaskType::CodeGeneration` | ⬜ 待办 | generation_classifier.rs (5处) |
| `TaskType::Knowledge` | `TaskType::KnowledgeRetrieval` | ⬜ 待办 | generation_classifier.rs (3处) |
| `TaskType::Reasoning` | `TaskType::ComplexReasoning` | ⬜ 待办 | generation_classifier.rs (3处) |

### A.4 ModelTier 统一 — ⬜ 待办 (T0.2)

**文件**: `crates/neotrix-types/src/core/nt_core_knowledge/types.rs` (追加)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelTier {
    Fast,
    Balanced,
    Powerful,
    Specialized(String),
}
```

**删除的本地 ModelTier**:
- `nt_core_model_router.rs:17-24` → re-export
- `nt_core_model_unified.rs:154-164` → re-export

### A.5 RoutingStrategy 统一 — ⬜ 待办 (T0.2)

**文件**: `crates/neotrix-types/src/core/nt_core_knowledge/types.rs` (追加)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoutingStrategy {
    CostOptimized,
    LatencyOptimized,
    QualityOptimized,
    LoadBalanced,
    Fallback,
    LearningBased,
}
```

**删除的本地 RoutingStrategy**:
- `l1_action/nt_io/model_routing.rs` → re-export
- `l1_action/nt_act/actions/infra/model_router.rs:98-109` → re-export

---

## 附录 B: Sprint 1 UnifiedModelAdapter Trait 设计

### B.1 核心 Trait (追加到 nt_core_model_unified.rs)

```rust
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;

/// 统一模型适配器 — 所有 provider 实现此 trait
///
/// 设计原则: OpenAI Chat Completions 作为 canonical wire format
/// Provider adapter 负责双向翻译 (OpenAI ↔ Provider native)
#[async_trait::async_trait]
pub trait UnifiedModelAdapter: Send + Sync {
    /// Provider 标识名 (e.g., "openai", "anthropic", "gemini")
    fn provider_name(&self) -> &str;

    /// 同步发送请求
    fn send(&self, request: &ModelRequest) -> Result<ModelResponse, AdapterError>;

    /// 流式发送请求
    fn stream(
        &self,
        request: &ModelRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, AdapterError>> + Send>>, AdapterError>;

    /// Token 计数 (provider 特定分词器)
    fn count_tokens(&self, text: &str) -> usize;

    /// 能力查询
    fn supports_tool_calling(&self) -> bool { true }
    fn supports_streaming(&self) -> bool { true }
    fn supports_thinking(&self) -> bool { false }
    fn max_context_tokens(&self) -> usize;
    fn max_output_tokens(&self) -> usize;

    /// 成本查询 (per 1K tokens, USD)
    fn cost_per_1k_input(&self) -> f64;
    fn cost_per_1k_output(&self) -> f64;
}
```

### B.2 StreamEvent (统一流式事件)

```rust
/// 统一流式事件 — 所有 provider 的 SSE 解析为统一格式
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 文本增量
    TextDelta(String),
    /// 思考增量 (Anthropic/Gemini extended thinking)
    ThinkingDelta(String),
    /// 工具调用增量
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: String,
    },
    /// 使用量
    Usage {
        prompt_tokens: u32,
        completion_tokens: u32,
    },
    /// 完成
    Done { finish_reason: FinishReason },
    /// 错误
    Error(String),
}

#[derive(Debug, Clone)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
}
```

### B.3 AdapterError

```rust
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Rate limited: retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },
    #[error("Context overflow: {used}/{limit} tokens")]
    ContextOverflow { used: usize, limit: usize },
    #[error("Invalid request: {0}")]
    BadRequest(String),
    #[error("Provider error ({status}): {message}")]
    ProviderError { status: u16, message: String },
    #[error("Tool call parse error: {0}")]
    ToolParseError(String),
    #[error("SSE parse error: {0}")]
    SseParseError(String),
}
```

### B.4 OpenAI Adapter 转换逻辑

```rust
pub struct OpenAIAdapter {
    api_key: String,
    base_url: String,  // default: "https://api.openai.com/v1"
    model: String,
    client: reqwest::Client,
}

impl UnifiedModelAdapter for OpenAIAdapter {
    fn provider_name(&self) -> &str { "openai" }

    fn send(&self, request: &ModelRequest) -> Result<ModelResponse, AdapterError> {
        // 1. Canonical → OpenAI format (几乎零转换，OpenAI 就是 canonical)
        let openai_req = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": request.content}
            ],
            "temperature": 0.7,
            "max_tokens": 4096,
            "tools": request.tools.as_ref().map(|t| convert_tools_to_openai(t)),
        });

        // 2. POST to /v1/chat/completions
        // 3. Parse response → Canonical ModelResponse
        // OpenAI response 直接映射，无需格式转换
    }

    fn stream(&self, request: &ModelRequest) -> Result<...> {
        // SSE format: data: {delta}\n\n + data: [DONE]
        // 直接用 SseEventParser::parse_openai() 解析
    }
}
```

### B.5 Anthropic Adapter 转换逻辑

```rust
pub struct AnthropicAdapter {
    api_key: String,
    model: String,  // e.g., "claude-opus-5"
    client: reqwest::Client,
}

// 关键转换:
// 1. system message → 顶层 "system" 字段 (不是 message role)
// 2. max_tokens 必填 (OpenAI 可选)
// 3. tools[].parameters → tools[].input_schema
// 4. tool_calls[].function → content[].type=tool_use
// 5. tool result → content[].type=tool_result
// 6. SSE 事件类型完全不同:
//    event: message_start → StreamEvent::Usage (initial)
//    event: content_block_start → (block metadata)
//    event: content_block_delta → StreamEvent::TextDelta / ThinkingDelta
//    event: message_delta → StreamEvent::Done + final Usage
//    event: message_stop → stream end
```

### B.6 Gemini Adapter 转换逻辑

```rust
pub struct GeminiAdapter {
    api_key: String,
    model: String,  // e.g., "gemini-3.8-flash"
    client: reqwest::Client,
}

// 关键转换:
// 1. messages → contents (role: "user"/"model", parts: [{text}])
// 2. tools → functionDeclarations: [{name, description, parameters}]
// 3. tool_call → functionCalls: [{name, args}]
// 4. tool result → functionResponse: {name, response: {result}}
// 5. SSE 事件:
//    event: interaction.start → metadata
//    event: content.delta → StreamEvent::TextDelta
//    event: step.start → new step marker
//    event: interaction.complete → StreamEvent::Done + Usage
```

### B.7 SseEventParser

```rust
pub struct SseEventParser {
    buffer: String,
    current_event_type: Option<String>,
}

impl SseEventParser {
    pub fn new() -> Self { ... }

    /// 解析一行 SSE 数据，返回解析后的 StreamEvent
    pub fn parse_line(&mut self, line: &str, format: ProviderFormat) -> Option<StreamEvent> {
        match format {
            ProviderFormat::OpenAI => self.parse_openai(line),
            ProviderFormat::Anthropic => self.parse_anthropic(line),
            ProviderFormat::Gemini => self.parse_gemini(line),
            ProviderFormat::OllamaNative => self.parse_ollama_native(line),
        }
    }

    fn parse_openai(&mut self, line: &str) -> Option<StreamEvent> {
        // "data: {\"choices\":[{\"delta\":{\"content\":\"...\"}}]}" → TextDelta
        // "data: [DONE]" → Done
    }

    fn parse_anthropic(&mut self, line: &str) -> Option<StreamEvent> {
        // "event: content_block_delta" → set event type
        // "data: {\"type\":\"text_delta\",\"text\":\"...\"}" → TextDelta
        // "data: {\"type\":\"thinking_delta\",\"thinking\":\"...\"}" → ThinkingDelta
        // "event: message_stop" → Done
    }

    fn parse_gemini(&mut self, line: &str) -> Option<StreamEvent> {
        // "event: content.delta" → set event type
        // "data: {\"type\":\"text\",\"text\":\"...\"}" → TextDelta
        // "event: interaction.complete" → Done
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProviderFormat {
    OpenAI,
    Anthropic,
    Gemini,
    OllamaNative,
}
```

### B.8 ToolCallConverter

```rust
/// Canonical tool 定义
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}

/// Canonical tool call
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Canonical tool result
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

pub struct ToolCallConverter;

impl ToolCallConverter {
    /// Canonical tools → Provider-specific format
    pub fn to_openai(tools: &[ToolDefinition]) -> serde_json::Value {
        // [{type:"function", function:{name, description, parameters}}]
    }

    pub fn to_anthropic(tools: &[ToolDefinition]) -> serde_json::Value {
        // [{name, description, input_schema}]
    }

    pub fn to_gemini(tools: &[ToolDefinition]) -> serde_json::Value {
        // [{functionDeclarations: [{name, description, parameters}]}]
    }

    /// Provider-specific tool call → Canonical
    pub fn tool_call_from_openai(raw: &serde_json::Value) -> Result<ToolCall, ...> {
        // message.tool_calls[].function.{name, arguments}
    }

    pub fn tool_call_from_anthropic(raw: &serde_json::Value) -> Result<ToolCall, ...> {
        // content[].type=tool_use.{id, name, input}
    }

    /// Canonical tool result → Provider-specific format
    pub fn result_to_openai(result: &ToolResult) -> serde_json::Value {
        // {role:"tool", tool_call_id, content}
    }

    pub fn result_to_anthropic(result: &ToolResult) -> serde_json::Value {
        // {type:"tool_result", tool_use_id, content}
    }
}
```

---

## 附录 C: Sprint 2 UnifiedModelRouter 算法

### C.1 路由算法伪代码

```
function route(request: ModelRequest) -> RouteDecision:
    // Phase 1: Classify complexity (heuristic, no ML per LLMRouterBench)
    complexity = classify_complexity(request.content, request.task_type)

    // Phase 2: Filter eligible models
    candidates = models.filter(|m|
        m.capabilities.meets(request.preferences.required_capabilities)
        && m.context_window >= estimated_tokens(request)
        && m.cost_per_1k <= request.preferences.max_cost_per_query
    )

    // Phase 3: Score each candidate
    scored = candidates.map(|m| {
        quality = estimate_quality(request.task_type, m)
        cost_norm = normalize_cost(m.cost_per_1k, min_cost, max_cost)
        health = circuit_breaker.health_penalty(m.provider)
        latency_bonus = latency_factor(request.latency_tolerance, m.estimated_latency)

        score = quality × (1 - cost_weight × cost_norm) × health × latency_bonus
        (score, m)
    }).sort_by(|a,b| b.score.cmp(a.score))

    // Phase 4: Cascade escalation
    best = scored.first()
    if best.score < quality_threshold:
        // Try next tier
        escalated = scored.find(|(_, m)| m.tier > best.tier && m.cost <= budget)
        if escalated.is_some():
            best = escalated

    // Phase 5: Record decision + circuit breaker
    circuit_breaker.record(best.provider)
    RouteDecision { model: best.model, cost: best.estimated_cost, reasoning: ... }
```

### C.2 复杂度分类器 (5 信号)

```rust
fn classify_complexity(content: &str, task_type: &TaskType) -> f64 {
    let mut score = 0.0;

    // Signal 1: 内容长度 (长 = 更复杂)
    score += (content.len() as f64 / 1000.0).min(0.3);

    // Signal 2: 代码块存在
    if content.contains("```") { score += 0.15; }

    // Signal 3: 多步骤关键词
    let multi_step_keywords = ["first", "then", "finally", "step 1", "步骤"];
    if multi_step_keywords.iter().any(|k| content.to_lowercase().contains(k)) {
        score += 0.15;
    }

    // Signal 4: 推理关键词
    let reasoning_keywords = ["analyze", "compare", "evaluate", "prove", "分析", "证明"];
    if reasoning_keywords.iter().any(|k| content.to_lowercase().contains(k)) {
        score += 0.2;
    }

    // Signal 5: 任务类型加权
    score += match task_type {
        TaskType::Chat | TaskType::SimpleQA => 0.0,
        TaskType::CodeGeneration | TaskType::Debugging => 0.2,
        TaskType::Research | TaskType::ComplexReasoning => 0.3,
        TaskType::Architecture | TaskType::Security => 0.35,
        _ => 0.1,
    };

    score.min(1.0)
}
```

### C.3 质量估算函数

```rust
fn estimate_quality(task_type: &TaskType, model: &ModelMeta) -> f64 {
    // 基础质量 = tier 映射
    let base = match model.tier {
        ModelTier::Fast => 0.6,
        ModelTier::Balanced => 0.8,
        ModelTier::Powerful => 0.95,
        ModelTier::Specialized(ref domain) => {
            if task_matches_domain(task_type, domain) { 0.95 }
            else { 0.5 }
        }
    };

    // 可靠性调整
    base * model.reliability as f64
}
```

### C.4 路由决策数据结构

```rust
pub struct RouteDecision {
    pub selected_model: String,
    pub provider: String,
    pub tier: ModelTier,
    pub estimated_cost: f64,
    pub estimated_latency_ms: u64,
    pub quality_score: f64,
    pub cascade_escalated: bool,
    pub reasoning: String,  // 人类可读的决策原因
}
```

---

## 附录 D: Sprint 3 Context Compactor 算法

### D.1 三级压缩管线

```rust
pub struct ContextCompactor {
    config: CompactionConfig,
    token_counter: Box<dyn TokenCounter>,
}

pub struct CompactionConfig {
    pub enabled: bool,
    pub trigger_threshold: f64,      // default: 0.8 (80% of context window)
    pub l1_max_tool_result_chars: usize, // default: 10_000
    pub l2_keep_recent_turns: usize,     // default: 10
    pub l2_keep_system: bool,            // default: true
    pub l3_summary_model: String,        // default: "gpt-4o-mini"
    pub max_compaction_attempts: usize,  // default: 3 (R-P38 retry cap)
}

impl ContextCompactor {
    /// 检查是否需要压缩，返回压缩后的消息列表
    pub fn compact(
        &self,
        messages: &[Message],
        context_limit: usize,
    ) -> Result<CompactionResult, CompactionError> {
        let current_tokens = self.token_counter.count_messages(messages);
        let usage_ratio = current_tokens as f64 / context_limit as f64;

        if usage_ratio < self.config.trigger_threshold {
            return Ok(CompactionResult::Unchanged);
        }

        let mut result = messages.to_vec();
        let mut total_saved = 0;

        // L1: Tool Result Pruning (零成本)
        let (pruned, saved) = self.prune_tool_results(&result);
        result = pruned;
        total_saved += saved;

        // L2: History Snip (零成本)
        if self.token_counter.count_messages(&result) as f64 / context_limit as f64
            > self.config.trigger_threshold
        {
            let (snipped, saved) = self.snip_history(&result, context_limit);
            result = snipped;
            total_saved += saved;
        }

        // L3: Summary Compaction (1 cheap model call)
        if self.token_counter.count_messages(&result) as f64 / context_limit as f64
            > self.config.trigger_threshold
        {
            let (summarized, saved) = self.summarize(&result, context_limit).await?;
            result = summarized;
            total_saved += saved;
        }

        Ok(CompactionResult::Compacted {
            messages: result,
            tokens_saved: total_saved,
            levels_applied: /* ... */,
        })
    }
}
```

### D.2 L1: Tool Result Pruning

```rust
fn prune_tool_results(&self, messages: &[Message]) -> (Vec<Message>, usize) {
    let mut result = Vec::new();
    let mut saved = 0;

    for msg in messages {
        if msg.role == Role::Tool && msg.content.len() > self.config.l1_max_tool_result_chars {
            let original_len = msg.content.len();
            let head = &msg.content[..500.min(original_len)];
            let tail = &msg.content[original_len.saturating_sub(200)..];
            let pruned_content = format!(
                "{}\n\n... [truncated, {} chars total] ...\n\n{}",
                head, original_len, tail
            );
            saved += original_len - pruned_content.len();
            result.push(Message { content: pruned_content, ..msg.clone() });
        } else {
            result.push(msg.clone());
        }
    }

    (result, saved)
}
```

### D.3 L2: History Snip

```rust
fn snip_history(&self, messages: &[Message], context_limit: usize) -> (Vec<Message>, usize) {
    let mut result = Vec::new();
    let mut saved = 0;

    // 1. 保留 system message
    if self.config.l2_keep_system {
        if let Some(sys) = messages.iter().find(|m| m.role == Role::System) {
            result.push(sys.clone());
        }
    }

    // 2. 保留最近 N 轮
    let keep_from = messages.len().saturating_sub(self.config.l2_keep_recent_turns * 2);
    for msg in &messages[keep_from..] {
        result.push(msg.clone());
    }

    // 3. 计算节省
    let original_tokens: usize = messages.iter().map(|m| self.token_counter.count(m)).sum();
    let new_tokens: usize = result.iter().map(|m| self.token_counter.count(m)).sum();
    saved = original_tokens.saturating_sub(new_tokens);

    (result, saved)
}
```

### D.4 L3: Summary Compaction

```rust
async fn summarize(&self, messages: &[Message], context_limit: usize) -> Result<...> {
    // 1. 构建摘要 prompt
    let summary_prompt = format!(
        "Summarize this conversation into structured sections:\n\
         1. Context: What was the user trying to do?\n\
         2. Decisions: What choices were made?\n\
         3. Current State: What's the current progress?\n\
         4. Next Steps: What remains to be done?\n\
         5. Key Code: Any important code snippets or file paths.\n\n\
         Conversation:\n{}",
        messages_to_text(messages)
    );

    // 2. 用 cheap model 生成摘要
    let summary = self.cheap_model.send(&ModelRequest {
        content: summary_prompt,
        task_type: TaskType::Summarization,
        ..Default::default()
    }).await?;

    // 3. 替换: [system] + [summary] + [last 3 turns]
    let mut result = Vec::new();
    if let Some(sys) = messages.iter().find(|m| m.role == Role::System) {
        result.push(sys.clone());
    }
    result.push(Message {
        role: Role::User,
        content: format!("[Conversation Summary]\n{}", summary.output),
    });
    // 保留最后 3 轮
    let last_3 = messages.len().saturating_sub(6);
    result.extend_from_slice(&messages[last_3..]);

    Ok((result, /* tokens_saved */))
}
```

---

## 附录 E: Sprint 3 Config 层级

### E.1 配置文件查找顺序

```
1. ~/.config/neotrix/config.toml          (全局默认)
2. ./neotrix.toml                          (项目级, 可提交)
3. ./neotrix.local.toml                    (本地级, .gitignore)
4. NEOTRIX_* 环境变量                       (最高优先级)
```

### E.2 TOML Schema

```toml
[model]
provider = "openai"
model = "gpt-5.6"
# api_key 从环境变量 NEOTRIX_API_KEY 读取, 不写入文件

[routing]
strategy = "CostOptimized"       # CostOptimized | LatencyOptimized | QualityOptimized
cost_weight = 0.5                # 0.0=纯成本优先, 1.0=纯质量优先
quality_threshold = 0.7          # 低于此分触发 cascade 升级
cascade_enabled = true

[compaction]
enabled = true
trigger_threshold = 0.8          # 80% context 时触发
summary_model = "gpt-4o-mini"    # 用于 L3 摘要的 cheap model

[providers.openai]
model = "gpt-5.6"
base_url = "https://api.openai.com/v1"
# api_key 从 NEOTRIX_OPENAI_API_KEY 读取

[providers.anthropic]
model = "claude-opus-5"
# api_key 从 NEOTRIX_ANTHROPIC_API_KEY 读取

[providers.gemini]
model = "gemini-3.8-flash"
# api_key 从 NEOTRIX_GEMINI_API_KEY 读取
```

### E.3 Rust Config 结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoTrixConfig {
    pub model: ModelSection,
    pub routing: RoutingSection,
    pub compaction: CompactionSection,
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSection {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingSection {
    pub strategy: RoutingStrategy,
    pub cost_weight: f64,
    pub quality_threshold: f64,
    pub cascade_enabled: bool,
}

impl NeoTrixConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Self::load_global()?;
        config.merge(Self::load_project()?);
        config.merge(Self::load_local()?);
        config.apply_env_overrides();
        Ok(config)
    }
}
```

---

## 附录 F: Sprint 2 Circuit Breaker

```rust
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: BreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    config: BreakerConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BreakerState {
    Closed,    // 正常
    Open,      // 故障中
    HalfOpen,  // 测试中
}

#[derive(Debug, Clone)]
pub struct BreakerConfig {
    pub failure_threshold: u32,   // default: 3
    pub recovery_timeout: Duration, // default: 60s
    pub half_open_max_calls: u32, // default: 1
}

impl CircuitBreaker {
    pub fn record_success(&mut self) {
        match self.state {
            BreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.half_open_max_calls {
                    self.state = BreakerState::Closed;
                    self.failure_count = 0;
                }
            }
            BreakerState::Closed => {
                self.failure_count = 0;
            }
            _ => {}
        }
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            BreakerState::Closed if self.failure_count >= self.config.failure_threshold => {
                self.state = BreakerState::Open;
            }
            BreakerState::HalfOpen => {
                self.state = BreakerState::Open;
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        if self.state == BreakerState::Open {
            if let Some(last) = self.last_failure_time {
                if last.elapsed() >= self.config.recovery_timeout {
                    self.state = BreakerState::HalfOpen;
                    self.success_count = 0;
                }
            }
        }
    }

    /// 返回 0.0-1.0 的健康惩罚因子 (0.0 = 完全断开, 1.0 = 完全健康)
    pub fn health_penalty(&self) -> f64 {
        match self.state {
            BreakerState::Closed => 1.0,
            BreakerState::HalfOpen => 0.5,
            BreakerState::Open => 0.0,
        }
    }

    pub fn is_available(&self) -> bool {
        self.state != BreakerState::Open
    }
}
```

---

## 附录 G: 测试策略

### G.1 单元测试 (每个 Sprint)

| Sprint | 测试覆盖 | 验证方式 |
|--------|---------|---------|
| S0 | TaskType From 转换、as_str() 映射、ModelTier 兼容 | `cargo test -p neotrix-types` |
| S1 | 每个 adapter roundtrip (canonical → provider → canonical) | `cargo test -p neotrix --lib` |
| S2 | 路由决策一致性、cascade 触发、circuit breaker 状态机 | 同上 |
| S3 | L1/L2/L3 各级压缩效果、trigger threshold、retry cap | 同上 |
| S4 | CapabilityRegistry 统一后无编译错误 | `cargo check --all-targets -p neotrix` |
| S5 | model CLI 子命令端到端、EventBus 事件发射 | `cargo test -p neotrix --lib` + 手动运行 |
| S6 | Bend FFI 桥接、LAWS.bend 检查、并行 benchmark | `bend check` + `cargo test -p neotrix --lib` |
| S7 | 吸收模块集成验证、R-P42 合规检查 | `cargo check -p neotrix --lib` + 代码审查 |

### G.2 集成测试 (Sprint 1 完成后)

```rust
// tests/model_adapter_integration.rs
#[tokio::test]
async fn test_openai_adapter_roundtrip() {
    let adapter = OpenAIAdapter::new(/* test config */);
    let request = ModelRequest {
        content: "Say hello".to_string(),
        task_type: TaskType::Chat,
        ..Default::default()
    };
    let response = adapter.send(&request).unwrap();
    assert!(!response.output.is_empty());
    assert!(response.cost > 0.0);
}

#[tokio::test]
async fn test_anthropic_adapter_system_field() {
    // 验证 system message 被正确提取为顶层字段
    let adapter = AnthropicAdapter::new(/* test config */);
    let request = ModelRequest {
        content: "Hello".to_string(),
        task_type: TaskType::Chat,
        ..Default::default()
    };
    // 验证发送的 HTTP body 中 system 是顶层字段
}
```

### G.3 回滚计划

| Sprint | 回滚策略 |
|--------|---------|
| S0 | `git revert` 整个 commit; 旧 variant 名保留所以无破坏 |
| S1 | 删除 adapter 目录; trait 定义保留在 unified.rs |
| S2 | 恢复旧路由器文件; 删除新 router |
| S3 | 删除 compactor; config 改回单层 |
| S4 | 恢复 dead_code 注解; 恢复被删文件 |
| S5 | 恢复旧 CLI 命令; 删除 EventBus 接线 |
| S6 | 禁用 feature "bend"; Rust fallback 自动接管 |
| S7 | 恢复被增强模块的原始版本 |

### G.4 验证命令 (每个 Sprint 完成后)

```bash
# ── 基础编译检查 ──
cargo check -p neotrix-types && cargo check -p neotrix --lib

# ── 单元测试 ──
cargo test -p neotrix-types && cargo test -p neotrix --lib

# ── 冗余度量 ──
grep -r "enum TaskType" --include="*.rs" | wc -l    # 目标: ≤2
grep -r "enum ModelTier" --include="*.rs" | wc -l   # 目标: 1
grep -r "fn route\|fn select_model" --include="*.rs" | wc -l  # 目标: 1
grep -r "#\[allow(dead_code)\]" --include="*.rs" | wc -l  # 目标: <50

# ── S6 Bend 集成验证 ──
bend check bend/laws/neo_trix_laws.bend          # LAWS.bend 通过
bend run bend/compute/e8_encode.bend              # 并行计算可运行
cargo check -p neotrix --features bend --lib      # 启用 Bend feature 编译
cargo check -p neotrix --lib                      # 禁用 Bend feature 编译 (fallback)

# ── S7 Trendshift 吸收验证 ──
cargo check -p neotrix --lib                      # 无新增 error
grep -r "struct.*Adapter\|struct.*Router" --include="*.rs" | wc -l  # 无平行适配器
```

---

## 附录 H: Sprint 4 Dead Code 清理清单

### H.1 按域分类的 87 处 `#[allow(dead_code)]`

| 域 | 文件 | 行号 | 死代码项 | 分类 | 理由 |
|----|------|------|---------|------|------|
| **L6 Meta** | `nt_meta/arch_optimizer.rs` | 320 | struct/method | WIRE | 架构优化器未接入 consciousness loop |
| **L6 Meta** | `nt_core_capability/discovery.rs` | 285 | struct | WIRE | 能力发现未接入 registry |
| **L6 Meta** | `nt_core_capability/cache.rs` | 15 | struct | WIRE | 缓存未接入 routing |
| **L6 Meta** | `coordination/nt_meta_concurrency_detector.rs` | 17 | struct | KEEP | 并发检测器，可能被调用 |
| **L6 Meta** | `coordination/nt_meta_concurrency_tester.rs` | 15 | struct | DELETE | 测试专用 |
| **L6 Meta** | `coordination/governance.rs` | 16 | struct | WIRE | 治理未接入 |
| **L6 Meta** | `coordination/nt_meta_sentrux.rs` | 17 | struct | WIRE | 安全监控未接入 |
| **L6 Meta** | `coordination/nt_meta_integration_patterns.rs` | 16 | struct | WIRE | 集成模式未接入 |
| **L6 Meta** | `coordination/nt_meta_integration_points.rs` | 16 | struct | WIRE | 集成点未接入 |
| **L6 Meta** | `memory/nt_memory_knowledge_pipeline.rs` | 18 | struct | WIRE | 知识管线未接入 |
| **L6 Meta** | `memory/nt_memory_experience_tree.rs` | 15 | struct | WIRE | 经验树未接入 |
| **L5 Cognition** | `nt_core/visual/*` (4 files) | various | structs | DELETE | 视觉一致性模块，无消费者 |
| **L5 Cognition** | `nt_core/cuda/nt_core_cuda_rl.rs` | 16 | struct | DELETE | CUDA RL，无 GPU 环境 |
| **L5 Cognition** | `nt_core/nt_io_context_mgmt.rs` | 17,187 | structs | DELETE | IO 上下文管理，未使用 |
| **L5 Cognition** | `nt_core/nt_consciousness_core/response_parser.rs` | 165 | struct | WIRE | 响应解析器未接入 |
| **L5 Cognition** | `nt_core/reasoning/nt_core_planning.rs` | 18 | struct | WIRE | 规划器未接入 |
| **L5 Cognition** | `nt_core/reasoning/nt_core_fsm_topology.rs` | 17 | struct | DELETE | FSM 拓扑，未使用 |
| **L5 Cognition** | `nt_mind/mind_modules/seal/seal_enhanced.rs` | 15 | struct | WIRE | 增强 SEAL 未接入 |
| **L5 Cognition** | `nt_mind/mind_modules/other/skill_chain.rs` | 16,337 | structs | WIRE | 技能链未接入 |
| **L5 Cognition** | `nt_mind/evolution/evolution_daemon.rs` | 92 | struct | WIRE | 进化守护进程未接入 |
| **L5 Cognition** | `nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | 51 | handler | WIRE | 意识处理器未接入 |
| **L5 Cognition** | `nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 383 | handler | WIRE | 维护处理器未接入 |
| **L5 Cognition** | `nt_mind/nt_mind_background_loop/handlers_core.rs` | 371 | handler | WIRE | 核心处理器未接入 |
| **L5 Cognition** | `nt_mind/nt_mind_background_loop/run.rs` | 35 | struct | KEEP | 后台循环主入口 |
| **L5 Cognition** | `nt_mind/nt_mind/control_distillation.rs` | 193,290 | structs | WIRE | 控制蒸馏未接入 |
| **L5 Cognition** | `nt_mind/nt_mind/graph_build.rs` | 9 | struct | WIRE | 图构建未接入 |
| **L5 Cognition** | `nt_mind/nt_mind/infrastructure/open_source_benchmark.rs` | 18 | struct | KEEP | 基准测试 |
| **L5 Cognition** | `nt_mind/nt_mind/knowledge/knowledge_engine/search.rs` | 12 | struct | WIRE | 知识搜索未接入 |
| **L5 Cognition** | `nt_mind/nt_mind/experience_tree/self_reflection.rs` | 76 | struct | WIRE | 自省未接入 |
| **L5 Cognition** | `nt_mind/foundation/guardian.rs` | 887 | struct | WIRE | 守护者未接入 |
| **L5 Cognition** | `nt_mind/nt_mind_hook.rs` | 135 | struct | WIRE | hook 未接入 |
| **L5 Cognition** | `nt_mind/nt_game/builtin/game_2048.rs` | 27 | struct | KEEP | 游戏模块 |
| **L5 Cognition** | `consciousness_core/core.rs` | 452 | struct | WIRE | 意识核心未接入 |
| **L5 Cognition** | `nt_core_multi_agent.rs` | 177 | struct | WIRE | 多智能体未接入 |
| **L3 Embodiment** | `nt_shield/nt_shield_impl/nt_shield_binary_patcher.rs` | 1835 | struct | DELETE | 二进制补丁，未使用 |
| **L3 Embodiment** | `nt_shield/nt_shield_audit_phases/hunt_phase/mod.rs` | 109 | struct | WIRE | 猎杀阶段未接入 |
| **L3 Embodiment** | `nt_shield/nt_shield_audit_phases/validate_phase/mod.rs` | 32 | struct | WIRE | 验证阶段未接入 |
| **L3 Embodiment** | `nt_shield/nt_shield_action_authorizer.rs` | 211 | struct | WIRE | 动作授权未接入 |
| **L3 Embodiment** | `nt_shield/shield_core/vault.rs` | 193 | struct | WIRE | vault 未接入 |
| **L3 Embodiment** | `nt_shield/shield_core/check_registry.rs` | 8,19 | structs | DELETE | 检查注册表，未使用 |
| **L3 Embodiment** | `nt_shield/proxy_detection/account_cluster.rs` | 112,137 | structs | WIRE | 代理检测未接入 |
| **L3 Embodiment** | `nt_shield/nt_shield_ztnet/crypto/cookie.rs` | 29 | struct | KEEP | 加密 cookie |
| **L2 Perception** | `nt_world/crawl/stealth.rs` | 15 | struct | WIRE | 隐身爬虫未接入 |
| **L2 Perception** | `nt_world/crawl/classifier.rs` | 26 | struct | WIRE | 爬虫分类器未接入 |
| **L2 Perception** | `nt_world/source/plugin_sandbox.rs` | 4 | struct | WIRE | 插件沙箱未接入 |
| **L2 Perception** | `nt_world/source/text/lyrics/multi.rs` | 22 | struct | WIRE | 歌词多源未接入 |
| **L2 Perception** | `nt_world/source/audio/qqmusic.rs` | 10 | struct | WIRE | QQ 音乐未接入 |
| **L2 Perception** | `nt_world/data_source/nt_world_edgar.rs` | 460 | struct | WIRE | SEC EDGAR 未接入 |
| **L2 Perception** | `nt_world/data_source/nt_world_aoi.rs` | 256 | struct | WIRE | AOI 数据源未接入 |
| **L1 Action** | `nt_io/model_adapter.rs` | 105 | struct | DELETE | 旧适配器，将被 UnifiedModelAdapter 替代 |
| **L1 Action** | `nt_io/nt_io_provider/gateway/routing/agent_routing.rs` | 184 | struct | DELETE | 旧路由，将被 UnifiedModelRouter 替代 |
| **L1 Action** | `nt_io/nt_io_provider/gateway/routing/search_router.rs` | 33 | struct | WIRE | 搜索路由未接入 |
| **L1 Action** | `nt_io/nt_io_provider/gateway/resilience.rs` | 434 | struct | WIRE | 韧性模块未接入 |
| **L1 Action** | `nt_io/nt_io_provider/gateway/types.rs` | 333 | struct | KEEP | 网关类型 |
| **L1 Action** | `nt_io/nt_io_browser_engine.rs` | 172 | struct | WIRE | 浏览器引擎未接入 |
| **L1 Action** | `nt_io/nt_io_messaging.rs` | 307 | struct | WIRE | 消息系统未接入 |
| **L1 Action** | `nt_io/platform_gateway.rs` | 147 | struct | WIRE | 平台网关未接入 |
| **L1 Action** | `nt_infra_scatter_gather.rs` | 51 | struct | WIRE | Scatter-gather 未接入 |
| **L1 Action** | `nt_act/actions/media/three_d_dev.rs` | 19 | struct | WIRE | 3D 开发未接入 |
| **L1 Action** | `nt_act/actions/infra/multi_region_scheduler.rs` | 91 | struct | WIRE | 多区域调度未接入 |
| **L1 Action** | `nt_act/actions/orchestration/production_pipeline.rs` | 129 | struct | KEEP | 生产管线 |
| **L1 Action** | `nt_act/actions/orchestration/production_orchestrator.rs` | 129 | struct | KEEP | 生产编排器 |
| **L1 Action** | `nt_act/nt_act_trade/mock_adapters.rs` | 25 | struct | DELETE | mock 适配器 |
| **L1 Action** | `nt_act/nt_act_long_running_agent.rs` | 179,263 | structs | WIRE | 长运行 agent 未接入 |
| **L1 Action** | `nt_memory/nt_memory_historian/dmn_consolidation/mod.rs` | 62 | struct | WIRE | DMN 巩固未接入 |
| **L1 Action** | `nt_memory/nt_memory_kb/nt_memory_api.rs` | 110 | struct | WIRE | KB API 未接入 |
| **L1 Action** | `nt_memory/nt_memory_kb/nt_memory_hierarchical.rs` | 228 | struct | WIRE | 层次记忆未接入 |
| **L1 Action** | `nt_memory/nt_memory_kb/nt_memory_tech_reserve.rs` | 222 | struct | WIRE | 技术储备未接入 |
| **L1 Action** | `nt_memory/nt_memory_kb/nt_discovery_github_topics.rs` | 349 | struct | WIRE | GitHub 发现未接入 |
| **L4 Emotion** | `nt_feel/nt_feel_vtuber.rs` | 22 | struct | WIRE | VTuber 情感未接入 |
| **CLI** | `agent_cmds.rs` | 670 | struct | DELETE | 旧 agent 命令 |
| **CLI** | `laws.rs` | 109,244 | scanner | KEEP | dead_code 检测器本身 |
| **Tauri** | `nt_file_ability/image_super_resolution.rs` | 588 | struct | WIRE | 图像超分未接入 |

### H.2 清理优先级

| 优先级 | 动作 | 数量 | 预估 |
|--------|------|------|------|
| **P0 DELETE** | 确认死代码，直接删除 | 8 | 1h |
| **P1 WIRE** | Sprint 1-3 接线后自然消除 | ~50 | 由 Sprint 1-3 覆盖 |
| **P2 KEEP** | 保留，移除 dead_code 注解 | 10 | 30min |
| **P3 DEFER** | 需进一步调查 | ~19 | 后续 session |

### H.3 验证命令

```bash
# 清理前基线
grep -r "#\[allow(dead_code)\]" --include="*.rs" neotrix-core/src/ | wc -l
# 目标: 87 → <40

# 清理后验证
cargo check -p neotrix --lib 2>&1 | grep "^error" | wc -l
# 目标: 0 新增 error
```

---

## 附录 I: Sprint 5 CLI + EventBus 接线规格

### I.1 CLI `model` 子命令扩展

**文件**: `neotrix-core/src/cli/commands/model_cmds.rs`

当前状态: 127 行，支持 `list/set/current` 三个子命令。

**新增子命令**:

```rust
// 新增到 match sub { ... }

"test" if args.len() >= 2 => {
    // /model test <provider> [model]
    // 发送 "Say hello" 测试请求，测量延迟/成本
    let provider = &args[1];
    let model = args.get(2).map(|s| s.as_str()).unwrap_or("default");
    // 1. 从 config 加载 provider 配置
    // 2. 创建 adapter
    // 3. 发送测试请求
    // 4. 输出: provider, model, latency_ms, cost, tokens, status
}

"status" => {
    // /model status
    // 显示: 当前 provider/model, circuit breaker 状态, 最近成本
    // 输出格式:
    //   Provider: openai / gpt-5.6
    //   Circuit Breaker: CLOSED (healthy)
    //   Session Cost: $0.0234
    //   Last 10 requests: avg 450ms, 99.2% success
}

"route" if args.len() >= 2 => {
    // /model route <query>
    // Dry-run 路由决策，不实际调用 API
    // 输出格式:
    //   Query: "write a Python function"
    //   Complexity: 0.35 (medium)
    //   Task Type: CodeGeneration
    //   Selected: gpt-5.6 (Fast) via openai
    //   Estimated Cost: $0.0012
    //   Cascade: no (quality 0.82 > threshold 0.7)
}

"budget" => {
    // /model budget [set <amount>]
    // 显示/设置 session 预算
    // 输出: Budget: $5.00 | Used: $0.23 | Remaining: $4.77
}
```

### I.2 EventBus 事件定义

**文件**: `neotrix-core/src/core/nt_core_event.rs`

追加到 `CoreEvent` enum:

```rust
// ── NT-CORE Model Routing ───────────────────────────────────────
#[serde(rename = "model_routed")]
ModelRouted {
    provider: String,
    model: String,
    task_type: String,
    complexity: f64,
    quality_score: f64,
    estimated_cost: f64,
    cascade_escalated: bool,
},

#[serde(rename = "model_fallback")]
ModelFallback {
    from_provider: String,
    to_provider: String,
    reason: String,
},

#[serde(rename = "circuit_breaker_tripped")]
CircuitBreakerTripped {
    provider: String,
    failure_count: u32,
    state: String,  // "open" | "half_open"
},

#[serde(rename = "cost_exceeded")]
CostExceeded {
    budget: f64,
    spent: f64,
    request_cost: f64,
},
```

### I.3 EventBus 接线点

**在 UnifiedModelRouter 中发射事件**:

```rust
// route() 方法中:
fn route(&self, request: &ModelRequest) -> RouteDecision {
    let decision = self.compute_route(request);

    // 发射路由事件
    self.event_bus.emit(CoreEvent::ModelRouted {
        provider: decision.provider.clone(),
        model: decision.selected_model.clone(),
        task_type: format!("{:?}", request.task_type),
        complexity: decision.complexity,
        quality_score: decision.quality_score,
        estimated_cost: decision.estimated_cost,
        cascade_escalated: decision.cascade_escalated,
    });

    // 如果 cascade 升级了
    if decision.cascade_escalated {
        self.event_bus.emit(CoreEvent::ModelFallback {
            from_provider: decision.original_provider.clone(),
            to_provider: decision.provider.clone(),
            reason: "quality_below_threshold".to_string(),
        });
    }

    decision
}

// circuit breaker 触发时:
fn on_circuit_breaker_trip(&self, provider: &str, failures: u32) {
    self.event_bus.emit(CoreEvent::CircuitBreakerTripped {
        provider: provider.to_string(),
        failure_count: failures,
        state: "open".to_string(),
    });
}
```

### I.4 KB 持久化

**路由决策存储格式** (knowledge.db → `routing` namespace):

```json
{
  "request_id": "req-001",
  "timestamp": 1695000000000,
  "task_type": "CodeGeneration",
  "complexity": 0.35,
  "selected_provider": "openai",
  "selected_model": "gpt-5.6",
  "quality_score": 0.82,
  "estimated_cost": 0.0012,
  "actual_cost": 0.0011,
  "latency_ms": 450,
  "cascade_escalated": false,
  "success": true
}
```

**查询模式**:
```rust
// 最近 N 次路由决策
kb.query_prefix("routing:", limit=100)

// 按 provider 统计成功率
kb.query_filter(|k,v| v.selected_provider == "openai" && k.age < 86400)

// 按 task_type 统计平均成本
kb.query_aggregate(|v| v.task_type, |v| v.actual_cost, AggregateOp::Mean)
```

### I.5 成本追踪

```rust
pub struct CostTracker {
    session_budget: f64,
    session_spent: f64,
    daily_budget: f64,
    daily_spent: f64,
    request_history: Vec<CostRecord>,
}

impl CostTracker {
    pub fn can_afford(&self, estimated_cost: f64) -> bool {
        self.session_spent + estimated_cost <= self.session_budget
            && self.daily_spent + estimated_cost <= self.daily_budget
    }

    pub fn record(&mut self, cost: f64) {
        self.session_spent += cost;
        self.daily_spent += cost;
        self.request_history.push(CostRecord {
            cost,
            timestamp: Instant::now(),
        });
    }

    pub fn session_summary(&self) -> CostSummary {
        CostSummary {
            total: self.session_spent,
            budget: self.session_budget,
            remaining: self.session_budget - self.session_spent,
            request_count: self.request_history.len(),
            avg_cost: self.session_spent / self.request_history.len().max(1) as f64,
        }
    }
}
```

---

## 附录 J: Edge Case 处理 + 降级策略

### J.1 类型统一 Edge Cases

| 场景 | 风险 | 处理 |
|------|------|------|
| `match` 无 `_` 分支 + 新增 variant | 编译 warning (非 error) | 添加 `_ => default` 分支 |
| 序列化/反序列化旧格式 | `serde` 默认忽略未知字段 | 添加 `#[serde(other)]` 到 `Custom` variant |
| `From` impl 自引用 | 编译 error | 删除 `From` impl，类型已统一 |
| neotrix-types 版本号 | semver breaking | bump 到 0.22.0 (major API change) |

### J.2 Adapter Edge Cases

| 场景 | 处理 |
|------|------|
| 未识别的响应格式 | 返回 `AdapterError::ProviderError` + 原始 JSON |
| SSE 中断 | `SseEventParser` 保留 buffer，下次 `parse_line` 继续 |
| tool_call_id 不匹配 | 返回 `AdapterError::ToolParseError` |
| 返回不同模型 | `ModelResponse.model_used` 记录实际模型，路由层比较 |
| context 溢出 | 检测 `429`/`context_length_exceeded` → 触发 Compactor |

### J.3 路由 Edge Cases

| 场景 | 处理 |
|------|------|
| 所有 provider circuit-broken | 返回 `AdapterError::AllProvidersUnavailable` + 建议等待 |
| 超出预算 | `CostTracker.can_afford()` 检查 → 返回 cheapest 可用模型 |
| 最便宜模型不可用 | cascade 升级到下一个可用模型 |
| 两个模型分数相同 | 按 tier 优先 (Fast > Balanced > Powerful)；同 tier 按 cost 排序 |

### J.4 Context Compactor Edge Cases

| 场景 | 处理 |
|------|------|
| 压缩丢失关键信息 | L3 保留最后 3 轮 + 工具调用结果；可配置 `keep_turns` |
| Summary model 失败 | 重试 1 次 → 降级到 L2 only (仅 snip，不 summary) |
| 已超 limit | 跳过 L1/L2，直接 L3 (强制 summary) |
| 工具调用序列中 | 不压缩包含 `tool_call`/`tool_result` 的消息对 |

### J.5 降级策略矩阵

| 组件 | 降级路径 |
|------|---------|
| UnifiedModelAdapter | provider 适配失败 → 降级到 OpenAI-compat pass-through |
| UnifiedModelRouter | 路由计算失败 → 降级到 round-robin |
| SseEventParser | 解析失败 → 降级到 raw bytes 转发 |
| ContextCompactor | compaction 失败 → 降级到 L2 only |
| CircuitBreaker | 状态异常 → 降级到 Closed (全放行) |
| CostTracker | 记录失败 → 降级到无预算限制 |

---

## 附录 K: 迁移兼容性保证

### K.1 渐进式迁移策略

```
Sprint 0: 类型统一 (纯 additive，不删除旧代码)
  ↓ 验证: cargo check 通过
Sprint 1: 适配器实现 (新代码，不影响旧路径)
  ↓ 验证: 新旧 adapter 并存
Sprint 2: 路由器切换 (新 router + 旧 router 暂留)
  ↓ 验证: 新 router 通过所有测试
  ↓ 删除旧 router
Sprint 3: Compaction + Config (新模块，不影响现有功能)
Sprint 4: 清理 (删除死代码)
Sprint 5: 接线 (EventBus + KB)
Sprint 6: Bend 集成 (feature-gated，fallback 保留)
  ↓ 验证: 禁用 feature 仍可编译
Sprint 7: Trendshift 吸收 (增强现有模块，不新建)
```

### K.2 每 Sprint 最小 viable 变更

| Sprint | 最小变更 | 可回滚性 |
|--------|---------|---------|
| S0 | neotrix-types 追加 variant + re-export | 100% — 仅添加，不修改 |
| S1 | nt_core_model_unified.rs 追加 trait + structs | 100% — 新文件/新代码 |
| S2 | nt_core_model_router.rs 重写 + 旧文件暂留 | 90% — 删除旧 router 可恢复 |
| S3 | 新建 nt_context_compactor.rs + config.rs 修改 | 95% — 新模块可删除 |
| S4 | 删除 dead_code 注解 + 部分文件 | 80% — 删除需谨慎 |
| S5 | model_cmds.rs 扩展 + event 追加 | 95% — CLI 可回退 |
| S6 | Bend FFI 桥接 + compute 模块 (feature-gated) | 100% — 禁用 feature 即可 |
| S7 | 增强现有模块 (rev_officer/shield/memory) | 95% — 恢复原始版本 |

### K.3 外部消费者兼容

| 消费者 | 影响 | 缓解 |
|--------|------|------|
| Tauri 桌面端 | ProviderManager 使用旧类型 | Sprint 2 后同步更新 Tauri 层 |
| CLI 用户 | `/model` 命令行为不变 | 仅追加子命令，不修改现有行为 |
| KB 数据 | 旧路由日志格式 | 添加 migration 脚本 |
| 配置文件 | config.toml 字段名不变 | 新字段有默认值 |
| Bend 桥接 | feature-gated，不影响非 Bend 用户 | 禁用 feature 时 Rust fallback 接管 |
| Trendshift 吸收 | 增强现有模块，不改变公开 API | 仅内部实现变更 |

### K.4 版本策略

```
neotrix-types: 0.21.0 → 0.22.0 (TaskType/ModelTier/RoutingStrategy 新增)
neotrix:       0.21.0 → 0.22.0 (UnifiedModelAdapter trait 是新增 API)
               0.22.0 → 0.23.0 (Bend feature-gated 集成 + Trendshift 吸收)
```

---

## 附录 L: 跨 Sprint 依赖图

```
                    ┌─────────────────────────────────────────────────┐
                    │           Sprint 0: 类型统一 (已完成 T0.1)       │
                    │  TaskType ✅ | ModelTier | RoutingStrategy      │
                    └────────────────────┬────────────────────────────┘
                                         │
                    ┌────────────────────▼────────────────────────────┐
                    │         Sprint 1: 统一适配器                     │
                    │  UnifiedModelAdapter + SseEventParser +         │
                    │  ToolCallConverter + 4 Provider Adapters        │
                    └────┬───────────┬───────────┬───────────────────┘
                         │           │           │
            ┌────────────▼──┐  ┌─────▼──────┐  ┌▼─────────────────┐
            │ Sprint 2:     │  │ Sprint 3:  │  │ Sprint 4:        │
            │ 统一路由器    │  │ Compaction │  │ 冗余清理         │
            │ CostAware+CB  │  │ + Config   │  │ Dead Code 50%+   │
            └───────┬──────┘  └─────┬──────┘  └──────────────────┘
                    │                │
            ┌───────▼────────────────▼──────┐
            │       Sprint 5: CLI + 接线     │
            │  model CLI + EventBus + KB     │
            └───────────────┬───────────────┘
                            │
              ┌─────────────┼─────────────────────┐
              │             │                     │
   ┌──────────▼──┐  ┌──────▼──────┐  ┌──────────▼──────────┐
   │ Sprint 6:   │  │ Sprint 7:   │  │ Future:             │
   │ Bend 集成   │  │ Trendshift  │  │ Per-turn ML Router  │
   │ LAWS+并行   │  │ 项目吸收    │  │ Tauri/Core 共享     │
   │ (可与S3-S5  │  │ (S1-S3后)   │  │ IETF SSE           │
   │  并行)      │  │             │  │ models.dev 目录     │
   └─────────────┘  └─────────────┘  └─────────────────────┘

关键路径: S0 → S1 → S2 → S5 (14 人天)
并行路径: S3 ∥ S2, S4 ∥ S2/S3, S6 ∥ S3-S5, S7 after S1-S3
总预估: 25 人天 (约 5 周, 含并行优化)
```

---

## 附录 M: Bend 语言集成详细设计

### M.1 LAWS.bend 形式化规则

```bend
# bend/laws/neo_trix_laws.bend

# === R-P1: 零 unsafe ===
law zero_unsafe:
  for module: Module
  {contains_unsafe(module) == False{} : Bool}

# === R-P42: 吸收不平行 ===
law no_parallel_adapters:
  for node: Node, adapter: Adapter
  {is_parallel_adapter(node, adapter) == False{} : Bool}

# === R-P79: 同 session 接线 ===
law wired_in_session:
  for feature: Feature
  {is_wired_to_production(feature) == True{} : Bool}

# === TaskType 单一事实源 ===
law task_type_single_source:
  for t: TaskType
  {defined_in_neotrix_types(t) == True{} : Bool}

# === 路由器单一事实源 ===
law router_single_source:
  {count_routing_implementations() <= 1 : Bool}

# === Cost-Aware Routing (Axiom A1) ===
law cost_aware_routing:
  for request: ModelRequest, decision: RouteDecision
  {decision.estimated_cost <= request.budget : Bool}

# === Context Budget (Axiom A2) ===
law context_budget:
  for session: Session
  {session.context_tokens <= session.max_context : Bool}

# === Circuit Breaker ===
law circuit_breaker:
  for provider: Provider
  {provider.consecutive_failures < 3 || provider.circuit_open == True{} : Bool}
```

### M.2 Rust↔Bend FFI 桥接层

```rust
// neotrix-core/src/l5_cognition/nt_bend_bridge.rs

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[cfg(feature = "bend")]
extern "C" {
    fn bend_e8_encode(input: *const c_char, output: *mut c_char) -> c_int;
    fn bend_vector_search(
        query: *const c_char,
        corpus: *const c_char,
        k: c_int,
        output: *mut c_char,
    ) -> c_int;
    fn bend_route_score(
        request: *const c_char,
        candidates: *const c_char,
        output: *mut c_char,
    ) -> c_int;
}

pub struct BendBridge {
    available: bool,
}

impl BendBridge {
    pub fn new() -> Self {
        let available = cfg!(feature = "bend");
        if available {
            tracing::info!("Bend bridge initialized (FFI available)");
        } else {
            tracing::info!("Bend bridge: feature disabled, using Rust fallback");
        }
        Self { available }
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    /// E8 HyperCube encoding via Bend (parallel)
    pub fn e8_encode(&self, input: &str) -> Result<String, BendError> {
        if !self.available {
            return Err(BendError::FeatureDisabled);
        }
        // FFI call to Bend
        let c_input = CString::new(input).map_err(|_| BendError::InvalidInput)?;
        let mut output = [0u8; 4096];
        let result = unsafe {
            bend_e8_encode(c_input.as_ptr(), output.as_mut_ptr() as *mut c_char)
        };
        if result == 0 {
            let c_str = unsafe { CStr::from_ptr(output.as_ptr() as *const c_char) };
            Ok(c_str.to_string_lossy().into_owned())
        } else {
            Err(BendError::ComputationFailed)
        }
    }

    /// Fallback to Rust when Bend unavailable
    pub fn e8_encode_fallback(&self, input: &str) -> Result<String, BendError> {
        // Delegate to existing Rust implementation
        crate::core::nt_core_hcube::encode(input)
            .map_err(|_| BendError::ComputationFailed)
    }
}

#[derive(Debug)]
pub enum BendError {
    FeatureDisabled,
    InvalidInput,
    ComputationFailed,
    FfiError,
}
```

### M.3 Bend 计算模块文件结构

```
bend/
├── laws/
│   ├── neo_trix_laws.bend          # LAWS.bend 核心规则
│   └── PROOF.bend                  # AI 编写证明
├── compute/
│   ├── e8_encode.bend              # E8 HyperCube 编码
│   ├── vector_search.bend          # 向量搜索并行化
│   ├── seal_train.bend             # SEAL 训练循环
│   └── route_score.bend            # 模型路由评分
└── tests/
    ├── test_e8.bend
    ├── test_vector.bend
    └── test_route.bend
```

### M.4 Bend 集成验证清单

| 验证项 | 命令 | 预期 |
|--------|------|------|
| Bend 安装 | `bend --version` | 2.x.x |
| LAWS.bend 检查 | `bend check laws/neo_trix_laws.bend` | 0 errors |
| E8 并行 benchmark | `bend run compute/e8_encode.bend` | 10x+ vs Rust |
| FFI 桥接 | `cargo check -p neotrix --lib` | 0 新增 error |
| Rust fallback | 禁用 feature "bend" 编译 | 正常编译 |

---

## 附录 N: Trendshift 项目吸收详细设计

### N.1 alibaba/open-code-review 混合审查吸收

**项目**: 11.4k★, 周增 569, Alibaba 出品
**核心模式**: hybrid 架构 (deterministic pipeline + LLM agent)

**吸收方案**:

```rust
// 增强 nt_mind_rev_officer (不新建模块)
// 1. 添加 deterministic 审查管线 (预检查)
// 2. LLM agent 仅处理需要推理的部分

pub struct HybridReviewer {
    deterministic: DeterministicPipeline,  // AST lint + 依赖检查 + 安全扫描
    llm_agent: LlmAgent,                  // 深度推理审查
}

impl HybridReviewer {
    pub async fn review(&self, code: &CodeDiff) -> ReviewReport {
        // Phase 1: 快速确定性检查 (<1s)
        let deterministic_findings = self.deterministic.run(code).await;

        // Phase 2: 仅对复杂变更调用 LLM
        if self.needs_llm_review(code, &deterministic_findings) {
            let llm_findings = self.llm_agent.review(code).await;
            self.merge(deterministic_findings, llm_findings)
        } else {
            deterministic_findings
        }
    }
}
```

**验收**: hybrid 模式比纯 LLM 快 3x+，发现率不降

### N.2 cloudflare/security-audit-skill 多阶段审计吸收

**项目**: 7.5k★, Cloudflare 出品
**核心模式**: 6 阶段审计管线 (静态→配置→动态→依赖→agent→报告)

**吸收方案**: 增强 NT-SHIELD 安全审计

```
阶段 1: 静态代码安全 (AST-level)
阶段 2: 配置安全 (TOML/YAML/ENV)
阶段 3: 动态测试 (fuzzing)
阶段 4: 依赖审计 (cargo audit)
阶段 5: Agent 行为审计 (prompt injection)
阶段 6: 机器可读发现 (SARIF/JSON)
```

**验收**: 6 阶段管线可独立运行，输出 SARIF 格式

### N.3 Graphify-Labs/graphify AST→知识图谱吸收

**项目**: 119.6k★, 年增 12k
**核心模式**: 本地确定性解析 → 知识图谱 (不依赖 LLM)

**吸收方案**: 增强 NT-MEMORY 知识图谱

```rust
// 增强 nt_core_knowledge (不新建模块)
// 添加 AST→知识图谱管线

pub struct AstToKnowledgeGraph {
    parser: TreeSitterParser,  // 本地确定性解析
    graph: KnowledgeGraph,     // NT-MEMORY 图谱
}

impl AstToKnowledgeGraph {
    pub fn ingest(&mut self, source: &str, language: &str) -> GraphDiff {
        let ast = self.parser.parse(source, language);
        let entities = self.extract_entities(&ast);  // 函数/结构体/模块
        let relations = self.extract_relations(&ast); // 调用/继承/依赖
        self.graph.merge(entities, relations)
    }
}
```

**验收**: 代码→知识图谱管线可独立运行，输出 Cypher/GQL 查询

### N.4 吸收验证清单

| 项目 | 验收标准 | R-P42 检查 |
|------|---------|-----------|
| open-code-review | hybrid 模式原型验证 | ✅ 增强 rev_officer，无新模块 |
| security-audit-skill | 6 阶段管线可运行 | ✅ 增强 NT-SHIELD，无新模块 |
| graphify | AST→图谱管线可运行 | ✅ 增强 NT-MEMORY，无新模块 |
| skills 标准化 | SKILL-SPEC.md 对齐 | ✅ 文档更新，无新代码 |
| deepseek-harness | 优化模式验证 | ✅ 增强 NT-CORE，无新模块 |
| colibri | 本地推理参考 | ✅ 文档参考，无新代码 |

---

## 附录 O: Sprint 进度追踪

### O.1 当前进度 (截至 2026-09-17)

| Sprint | 状态 | 完成任务 | 待办 |
|--------|------|---------|------|
| S0: 类型统一 | 🟡 进行中 | T0.1 ✅ (TaskType 扩展 38 变体 + Custom) | T0.2-T0.13 |
| S1: 统一适配器 | ⬜ 未开始 | — | 全部 |
| S2: 统一路由器 | ⬜ 未开始 | — | 全部 |
| S3: Compaction | ⬜ 未开始 | — | 全部 |
| S4: 冗余清理 | ⬜ 未开始 | — | 全部 |
| S5: CLI 接线 | ⬜ 未开始 | — | 全部 |
| S6: Bend 集成 | ⬜ 未开始 | — | 全部 |
| S7: Trendshift | ⬜ 未开始 | — | 全部 |

### O.2 已完成的预修复 (非 Sprint 计划内)

| 修复 | 文件 | 类型 |
|------|------|------|
| TaskType 扩展为 38 变体 + Custom | neotrix-types types.rs | ✅ 持久化 |
| 语法修复: orphaned use 语句 | nt_core_consciousness_core.rs | ✅ |
| 语法修复: 缺少 use 关键字 | nt_mind_eval_harness.rs | ✅ |
| 语法修复: 缺少 use 语句 | nt_repair_facade.rs | ✅ |
| match exhaustiveness: self_edit.rs | seal_core/self_edit.rs | ✅ |
| match exhaustiveness: consciousness_bridge.rs | consciousness_bridge.rs | ✅ |
| Coding→CodeGeneration 替换 | nt_core_model_router.rs (28处) | ✅ |
| Coding→CodeGeneration 替换 | nt_core_model_unified.rs tests (3处) | ✅ |

### O.3 Sprint 0 待完成任务

| ID | 任务 | 文件 | 状态 |
|----|------|------|------|
| T0.2 | 定义统一 ModelTier + RoutingStrategy | neotrix-types nt_core_model.rs | ⬜ |
| T0.3 | nt_core_model_router.rs 删除本地 TaskType | l5_cognition/nt_core_model_router.rs | ⬜ |
| T0.4 | nt_core_model_unified.rs 删除本地 TaskType | l5_cognition/nt_core_model_unified.rs | ⬜ |
| T0.5 | critic.rs 删除本地 TaskType | nt_act_orchestrator/critic.rs | ✅ |
| T0.6 | planner.rs 删除本地 TaskType | nt_act_orchestrator/planner.rs | ✅ |
| T0.7 | resource_router.rs 删除本地 TaskType | nt_consciousness_core/resource_router.rs | ⬜ |
| T0.8 | generation_classifier.rs 删除本地 TaskType | nt_io_provider/common/generation_classifier.rs | ⬜ |
| T0.9 | universal_model/traits.rs 删除本地 TaskType | nt_io/universal_model/traits.rs | ⬜ |
| T0.10 | actions/infra/model_router.rs 删除本地 TaskType | nt_act/actions/infra/model_router.rs | ⬜ |
| T0.11 | seal_core/model_router.rs 删除本地 TaskType | seal_core/model_router.rs | ⬜ |
| T0.12 | nt_core_god_agent.rs 本地 TaskType → From | l5_cognition/nt_core_god_agent.rs | ⬜ |
| T0.13 | production_pipeline.rs 保留本地 + From | orchestration/production_pipeline.rs | ⬜ |

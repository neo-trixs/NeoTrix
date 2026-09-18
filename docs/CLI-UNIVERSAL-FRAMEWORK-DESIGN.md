# Neotrix CLI 通用模型适配框架 — 全量设计方案

> **日期**: 2026-09-18 | **版本**: v1.0 | **状态**: 设计完成
>
> **目标**: 结合搜索外部AI (OpenCode/Claude/CLI) 全部技术资料 + 底层模型逆向推理，
> 融合方案架构到已有能力骨架，形成 Neotrix CLI 通用方案（适用所有外部模型），
> 实现**聚焦冗余 + 扁平缺陷 + 跨域错位**。

---

## 第一部分：现状全景

### 1.1 Neotrix 项目规模

| 指标 | 数量 |
|------|------|
| Workspace crates | 10 |
| 六层架构目录 | L0-L6 (7层) |
| 二进制目标 | 14 个 |
| CLI 命令 | 60+ |
| 源文件 (.rs) | 800+ |
| 主要子系统 | 9 个 (NT-CORE/MIND/MEMORY/WORLD/ACT/IO/SHIELD/PHYSICAL/FEEL) |
| Feature flags | 17 |
| 基准测试 | 10 |

### 1.2 已有的模型相关资产

Neotrix 已经拥有**三个独立的模型路由器**：

| 路由器 | 层级 | 路径 | 功能 |
|--------|------|------|------|
| `ModelRouter` | L1 Action | `nt_act/actions/infra/model_router.rs` | QualityTier 路由 (Draft/Preview/Final/Ultra) |
| `RealTimeModelRouter` | L5 Cognition | `nt_core_model_router.rs` | Tier 路由 (Fast/Balanced/Powerful/Specialized) + 成本追踪 |
| `ModelRouter` (SEAL) | L5 Cognition | `nt_mind/seal_core/model_router.rs` | SEAL 专用路由器 |

另有 **4 套模型适配器**：

| 适配器系统 | 路径 | 外部模型 |
|------------|------|----------|
| `nt_io_llm/adapters/` | L1 | OpenAI, Anthropic, Ollama, Catalog |
| `universal_model/` | L1 | OpenAI, Gemini, Anthropic, Ollama + fallback |
| `nt_io_provider/` | L1 | 多 provider 网关 + routing 子系统 |
| `model_adapter.rs` | L1 | LoRA/IP-Adapter/ControlNet (图像模型) |

---

## 第二部分：外部 AI CLI 工具技术逆向

### 2.1 OpenCode 架构核心

**三层接口**：CLI/TUI + Web (SolidJS) + Desktop (Tauri)

**Provider 系统** (关键吸收点)：
- **Route 抽象**：`Route = Protocol + Endpoint + Auth + Transport`，模型携带路由值，无全局注册表
- **7 层配置合并**：built-in → models.dev → user config → env → auth → plugin → config
- **ProviderTransform**：隔离 Provider 特有怪癖（Anthropic 过滤空内容、Mistral 标准化 tool ID）
- **SDK 缓存**：xxHash32 哈希 provider config，复用相同 SDK 实例
- **模糊搜索**：fuzzysort 处理拼写错误的模型名

**Tool 系统** (关键吸收点)：
- **22+ 内置工具**：bash, read, write, edit, apply_patch, glob, grep, websearch, task (子代理), skill, batch (25 并行)
- **懒加载**：`init()` 首次使用时才调用，启动不受工具数量影响
- **模型自适应过滤**：GPT 用 `apply_patch`，其他用 `edit`+`write`
- **9 种 edit 匹配策略**：处理 LLM 文本偏差

**Agent 系统**：
- **6 个内置 agent**：build (默认), plan (只读), general (子代理), explore (搜索), compaction (内部), title (内部)
- **权限规则集**定义 agent 能力，而非 prompt

### 2.2 Claude Code 架构核心

**3 协议集成**：ACP (客户端→Agent) + Claude SDK (Agent→API) + MCP (Agent↔工具)

**5 级 Context 压缩** (关键吸收点)：
1. **T1 Microcompact** — 缓存重排
2. **T2 Snip** — LRU 归档
3. **T3 Grouped** — 分组摘要
4. **T4 Auto compact** — 分叉 agent 摘要
5. **T5 Reactive** — 413 错误紧急压缩

**Tool-result 清除** (关键吸收点)：手术式替换旧 `tool_result` 块为占位符，保留 `tool_use` 记录。比完整压缩便宜得多。

**权限系统**：
- **5 级评估**：deny > allow > ask > default(ask)
- **4 层设置**：user → project → local → enterprise
- **Bash 规则**：精确匹配或前缀匹配 + shell 操作符检测

**Prompt Cache 管理** (关键吸收点)：
- System prompt 缓存（90% 成本节省）
- 提醒注入在对话消息中，不在 system prompt，避免缓存失效

### 2.3 其他工具关键模式

**Aider** — 3 层模型系统：
- Main (编辑) / Weak (提交信息/摘要) / Editor (架构模式实现)
- `model-settings.yml` 定义每个模型的层级关联
- 4 种编辑格式：diff, whole, udiff, editor-diff — 按模型能力选择

**Cline** — Hub-Spoke 架构：
- 守护进程协调会话，spoke workers 执行 agents
- 23 个 .proto 文件 (gRPC/protobuf)
- LLM 自声明 (`requires_approval` 字段) — 模型决定命令是否安全

**Windsurf/Cascade** — 两层架构：
- 规划层 (SWE-1 模型, AST 语义图) → 生成层 (前沿模型)
- 768 维嵌入，M-Query 检索
- `.windsurfrules` 项目级配置

### 2.4 CLI 框架

**clap (Rust)** — 5 层管线：Definition → Lexical → Parsing → Validation → Output
**ratatui** — 立即模式渲染，3 种应用模式 (TEA/Component/Flux)

### 2.5 模型路由模式

| 策略 | 信号 | 开销 | 质量 | 复杂度 |
|------|------|------|------|--------|
| Cost-based | Token 价格 (静态) | 低 | 无 | 低 |
| Latency-based | 历史 p50/p90 | 低 | 无 | 低-中 |
| Semantic | 查询嵌入/分类器 | 中-高 | 高 | 高 |
| Fallback chains | 错误/超时信号 | 低 | 保持 | 低 |
| Weighted LB | RPM/TPM 限制 | 极低 | 无 | 极低 |
| Budget-ceiling | 成本估算+升级 | 中 | 自适应 | 中-高 |

**路由管线** (3 阶段)：
1. **Pre-router**：低成本规则或分类器
2. **Post-generation verifier**：估算响应质量/不确定性
3. **Escalation policy**：接受/优化/拒绝/推迟到更强模型

---

## 第三部分：聚焦冗余 + 扁平缺陷 + 跨域错位

### 3.1 聚焦冗余（13 处类型重复）

| 冗余类型 | 出现次数 | 位置 |
|----------|----------|------|
| **TaskType enum** | **11 处独立定义** | nt_core_model_router, goal_cmds, nt_world_model, nt_io_provider, nt_act_orchestrator, nt_core_god_agent, production_pipeline, resource_router, nt_io_neocodex, nt_core_model_unified, l1_action/traits |
| **ModelProvider/ModelCapability** | 3 处 | nt_core_model_router, nt_core_model_skills, nt_io_provider/types |
| **CapabilityRegistry** | 3 处 | nt_core_capability_tree, nt_act_trade, nt_file_ability |
| **SelfModel** | 4 处 | nt_core_self/self_model.rs, silicon_self.rs, self_model_unified.rs, nt_core_self_model.rs |
| **ModelGateway/Router** | 3 处 | nt_core_model_gateway, nt_core_model_router, nt_io_provider/gateway |
| **Config (default_model)** | 3 处 | config.rs, model_cmds.rs, env var |

### 3.2 扁平缺陷（11 类问题）

| 缺陷类型 | 数量 | 严重度 |
|----------|------|--------|
| TODO/FIXME/HACK/XXX | 90+ | 中 |
| dead_code 抑制 | 90+ | 中 |
| 100+ 行函数 | 10+ | 中 |
| 无 MCP server 实现 | 1 | 高 |
| 无 provider-agnostic SSE 解析器 | 1 | 高 |
| 无 tool-result 清除机制 | 1 | 高 |
| 无分层 Context 压缩 | 1 | 高 |
| 无预检 token 计数 | 1 | 高 |
| 无请求规范化层 | 1 | 中 |
| 无缓存边界管理 | 1 | 中 |
| 无取消传播 | 1 | 低 |

### 3.3 跨域错位（4 处）

| 错位 | 描述 | 影响 |
|------|------|------|
| L5/L1 execute 重复 | L5 `RealTimeModelRouter` 和 L1 `ModelRouter` 执行相同路由逻辑 | 两套 fallback chain 不同步 |
| CLI 重新实现核心功能 | `/model` 命令硬编码模型列表，绕过 `nt_core_model_skills::REGISTRY` | 列表与实际可用模型脱节 |
| L1 Provider Gateway vs L5 Gateway | L1 `gateway/` (10+ routing 文件) 和 L5 `nt_core_model_gateway.rs` 职责重叠 | 路由决策分散在两层 |
| SEAL Router 隔离 | `nt_mind/seal_core/model_router.rs` 独立路由器 | 与其他路由器不共享 cost 数据 |

---

## 第四部分：Neotrix CLI 通用方案架构

### 4.1 核心设计原则

```
                    ┌─────────────────────────────────┐
                    │     Neotrix CLI Universal        │
                    │     Model Adaptation Framework   │
                    └────────────┬────────────────────┘
                                 │
          ┌──────────────────────┼──────────────────────┐
          │                      │                      │
   ┌──────▼──────┐      ┌───────▼───────┐      ┌───────▼───────┐
   │  聚焦冗余    │      │  扁平缺陷     │      │  跨域错位      │
   │  Focus       │      │  Flatten      │      │  Cross-Domain │
   │  Redundancy  │      │  Defects      │      │  Misalignment │
   └──────┬──────┘      └───────┬───────┘      └───────┬───────┘
          │                      │                      │
   ┌──────▼──────┐      ┌───────▼───────┐      ┌───────▼───────┐
   │• TaskType    │      │• SSE Parser   │      │• Route统一     │
   │  统一为1处   │      │• Tool-result  │      │• Gateway 合并  │
   │• ModelProvider│      │  清除          │      │• CLI→核心      │
   │  统一为1处   │      │• 分层压缩     │      │  映射          │
   │• Config      │      │• 预检token    │      │• SEAL 路由器   │
   │  单一入口    │      │• MCP server   │      │  统一          │
   └─────────────┘      └───────────────┘      └───────────────┘
```

### 4.2 架构熔炼：6 层集成方案

```
L6 Meta ─── ContextCompactor (5级压缩), PromptCacheManager
     │
L5 Cognition ── UnifiedModelRouter (合并3个路由器)
     │           ├── SemanticClassifier (任务分类)
     │           ├── CostAwareRouter (GWT salience)
     │           └── FallbackChain (统一退化链)
     │
L4 Emotion ──── (无直接LLM模式映射)
     │
L3 Embodiment ── (无直接LLM模式映射)
     │
L2 Perception ── SseEventParser (SSE归一化)
     │           ├── OpenAiSseParser
     │           ├── AnthropicSseParser
     │           └── GeminiSseParser
     │
L1 Action ───── UnifiedLlm (单一LLM接口)
                ├── ProviderTransform (Provider怪癖隔离)
                ├── McpServer/Client (MCP协议)
                ├── ToolResultLifecycle (工具结果生命周期)
                └── PreFlightTokenCount (预检token计数)
```

### 4.3 关键新增模块设计

#### 4.3.1 `UnifiedModelRouter` (合并 3 个路由器)

**吸收来源**：OpenCode Route + OMO Semantic Category + Cursor Classifier

```rust
// 统一路由器 — 合并 L1/L5/SEAL 三个路由器
pub struct UnifiedModelRouter {
    // 路由层
    semantic_classifier: SemanticClassifier,  // 任务→语义类别
    cost_router: CostAwareRouter,              // GWT salience 成本感知
    fallback_chain: FallbackChain,             // 统一退化链
    
    // Provider 层 (吸收 OpenCode Route 模式)
    routes: HashMap<String, Route>,            // model_id → Route
    provider_facades: Vec<ProviderFacade>,     // 可组合的 provider 组
    
    // 监控层
    cost_tracker: CostTracker,                 // 统一成本追踪
    health_checker: HealthChecker,             // Provider 健康检查
}

// Route = Protocol + Endpoint + Auth + Transport (吸收 OpenCode)
pub struct Route {
    pub protocol: Protocol,     // HTTP/SSE/WebSocket/gRPC
    pub endpoint: String,       // base URL
    pub auth: AuthConfig,       // API key / token / IAM
    pub transport: Transport,   // reqwest / tungstenite / tonic
}

// 语义分类器 (吸收 OMO + Cursor)
pub enum TaskCategory {
    VisualEngineering,  // 视觉工程
    Ultrabrain,         // 深度推理
    Quick,              // 快速问答
    Deep,               // 深度分析
    CodeEdit,           // 代码编辑
    CodeReview,         // 代码审查
    Creative,           // 创意写作
    DataProcessing,     // 数据处理
    Agent,              // 多步代理
    Custom(String),
}
```

#### 4.3.2 `SseEventParser` (Provider-Agnostic SSE)

**吸收来源**：OpenCode ProviderTransform + llm-sse

```rust
// 统一 SSE 事件
pub enum SseEvent {
    Text { delta: String },
    Reasoning { delta: String },          // 推理 token 分离
    ToolCallStart { id: String, name: String },
    ToolCallDelta { id: String, arguments: String },
    ToolCallEnd { id: String },
    Usage { input: usize, output: usize, cached: usize },
    Error { code: String, message: String },
    Done,
}

// Provider 特有解析器
pub trait SseParser: Send + Sync {
    fn parse_line(&self, line: &str) -> Option<SseEvent>;
    fn normalize(&self, event: SseEvent) -> SseEvent;  // ProviderTransform
}

// ProviderTransform (吸收 OpenCode)
pub struct ProviderTransform;
impl ProviderTransform {
    // Anthropic: 过滤空内容
    // Mistral: 标准化 tool ID
    // OpenAI: 清理 surrogate pair
    // 通用: 缓存边界标记
    pub fn apply(provider: &str, event: SseEvent) -> SseEvent { ... }
}
```

#### 4.3.3 `ContextCompactor` (5 级压缩)

**吸收来源**：Claude Code 5-tier cascade

```rust
// 5 级 Context 压缩 (吸收 Claude Code)
pub struct ContextCompactor {
    tiers: Vec<Box<dyn CompactionTier>>,
}

pub trait CompactionTier: Send + Sync {
    fn should_apply(&self, usage: f64, budget: usize) -> bool;
    fn compact(&self, messages: Vec<Message>) -> Vec<Message>;
}

// T1: Microcompact — 缓存重排，零损失
// T2: Snip — LRU 归档旧 tool_result
// T3: Grouped — 分组摘要（9 段结构化模板）
// T4: Auto — 分叉子 agent 摘要
// T5: Reactive — 413 紧急压缩
```

#### 4.3.4 `ToolResultLifecycle` (工具结果生命周期)

**吸收来源**：Claude Code tool-result clearing

```rust
// 工具结果生命周期管理
pub struct ToolResultLifecycle {
    max_age: Duration,           // tool_result 最大存活时间
    max_tokens: usize,           // 单个 tool_result 最大 token
    archive_threshold: f64,      // context 使用率阈值触发归档
}

impl ToolResultLifecycle {
    // 手术式替换：保留 tool_use 记录，清除 tool_result 内容
    pub fn archive_stale_results(&self, messages: &mut Vec<Message>) {
        for msg in messages.iter_mut() {
            if msg.is_tool_result() && self.is_stale(msg) {
                msg.content = "[Archived: tool result exceeds age/token limit]".into();
            }
        }
    }
}
```

#### 4.3.5 `McpServer` + `McpClient`

**吸收来源**：MCP 2026-07-28 spec + Claude Code acp MCP

```rust
// MCP Server — 将 UnifiedCapability 暴露为 MCP 工具
pub struct McpServer {
    capabilities: Vec<UnifiedCapability>,
    cache_ttl: Duration,
}

// MCP Client — 消费外部 MCP 服务器
pub struct McpClient {
    servers: HashMap<String, McpConnection>,
    tool_cache: HashMap<String, ToolSchema>,
}
```

#### 4.3.6 `PreFlightTokenCounter`

**吸收来源**：tiktoken + Anthropic countTokens

```rust
// 预检 token 计数
pub trait TokenCounter: Send + Sync {
    fn count_tokens(&self, model: &str, messages: &[Message]) -> usize;
}

// 实现策略：
// 1. 优先使用 provider API (Anthropic countTokens, OpenAI tiktoken)
// 2. 回退到 tiktoken-rs 本地计数
// 3. 最终回退到 chars / 4 启发式
```

---

## 第五部分：冗余清理方案

### 5.1 TaskType 统一 (11→1)

**策略**：在 `neotrix-types` crate 中定义唯一的 `TaskType`，所有层引用。

**清理清单**：
1. `neotrix-types/src/task_types.rs` — **保留**为单一定义
2. `nt_core_model_router.rs` — **删除**本地 TaskType，import from neotrix-types
3. `nt_core_model_unified.rs` — **删除**本地 TaskType，import from neotrix-types
4. `nt_core_god_agent.rs` — **删除**本地 TaskType，import from neotrix-types
5. `goal_cmds.rs` — **删除**本地 TaskType，import from neotrix-types
6. `nt_world_model.rs` — **删除**本地 TaskType，import from neotrix-types
7. `nt_io_provider/types.rs` — **删除**本地 TaskType，import from neotrix-types
8. `nt_act_orchestrator/planner.rs` — **删除**本地 TaskType，import from neotrix-types
9. `production_pipeline.rs` — **删除**本地 TaskType，import from neotrix-types
10. `resource_router.rs` — **删除**本地 TaskType，import from neotrix-types
11. `nt_io_neocodex/provider.rs` — **删除**本地 TaskType，import from neotrix-types
12. `l1_action/traits.rs` — **删除**本地 TaskType，import from neotrix-types

### 5.2 ModelProvider/ModelCapability 统一 (3→1)

**策略**：保留 `nt_io_provider/types.rs` 中的 `LlmProvider` + `ModelMeta`，删除其他。

### 5.3 路由器合并 (3→1)

**策略**：创建 `UnifiedModelRouter` 替代三个独立路由器。

### 5.4 Config 统一

**策略**：仅保留 `config.rs` 中的 `default_model`/`provider`，`model_cmds.rs` 使用动态 `REGISTRY`。

---

## 第六部分：迭代任务全量评测

### 阶段 0：紧急修复 (Sprint 0, 1 周)

| # | 任务 | 工作量 | 影响 | 风险 |
|---|------|--------|------|------|
| 0.1 | TaskType 统一为 1 处定义 | 2d | P0 — 消除 11 处重复 | 低 |
| 0.2 | 移除 model_cmds.rs 硬编码列表 | 0.5d | P1 — 使用 REGISTRY | 低 |
| 0.3 | 验证编译通过 | 0.5d | P0 | 低 |

### 阶段 1：基础协议层 (Sprint 1-2, 4 周)

| # | 任务 | 工作量 | 依据 | 外部模式 |
|---|------|--------|------|----------|
| 1.1 | 创建 `SseEventParser` — Provider-Agnostic SSE | 5d | P0 | OpenCode ProviderTransform |
| 1.2 | 创建 `ToolResultLifecycle` — 工具结果生命周期 | 3d | P0 | Claude Code tool clearing |
| 1.3 | 创建 `PreFlightTokenCounter` — 预检 token 计数 | 3d | P0 | tiktoken/countTokens |
| 1.4 | 创建 `Route` 抽象 — Protocol+Endpoint+Auth+Transport | 3d | P1 | OpenCode Route |
| 1.5 | 创建 `ModelPricingDb` — 400+ 模型定价数据库 | 2d | P1 | LiteLLM prices |
| 1.6 | `ProviderTransform` — Provider 怪癖隔离 | 3d | P1 | OpenCode transform.ts |

### 阶段 2：路由统一 (Sprint 3-4, 4 周)

| # | 任务 | 工作量 | 依据 | 外部模式 |
|---|------|--------|------|----------|
| 2.1 | 创建 `SemanticClassifier` — 任务语义分类 | 5d | P1 | OMO category + Cursor classifier |
| 2.2 | 创建 `UnifiedModelRouter` — 合并 3 个路由器 | 8d | P1 | 聚焦冗余 |
| 2.3 | 统一 FallbackChain — 一个退化链 | 3d | P1 | 聚焦冗余 |
| 2.4 | 统一 CostTracker — 一个成本追踪 | 2d | P1 | 聚焦冗余 |
| 2.5 | 接入 `UnifiedModelRouter` 到 CLI `/model` | 2d | P1 | 扁平缺陷 |

### 阶段 3：Context 管理 (Sprint 5-6, 4 周)

| # | 任务 | 工作量 | 依据 | 外部模式 |
|---|------|--------|------|----------|
| 3.1 | 创建 `ContextCompactor` — 5 级压缩 | 8d | P1 | Claude Code 5-tier |
| 3.2 | 集成 `ToolResultLifecycle` 到 TUI | 3d | P0 | Claude Code tool clearing |
| 3.3 | 创建 `PromptCacheManager` — 缓存边界管理 | 3d | P1 | Claude Code cache |
| 3.4 | 创建 `ContextFork` — 子代理上下文隔离 | 5d | P2 | Claude Code subagent |
| 3.5 | 9 段结构化压缩模板 | 2d | P2 | Claude Code 9-section |

### 阶段 4：协议互操作 (Sprint 7-8, 4 周)

| # | 任务 | 工作量 | 依据 | 外部模式 |
|---|------|--------|------|----------|
| 4.1 | 创建 `McpServer` — 暴露 UnifiedCapability | 8d | P1 | MCP 2026-07-28 |
| 4.2 | 创建 `McpClient` — 消费外部 MCP 服务器 | 8d | P1 | MCP spec |
| 4.3 | 实现 ACP Agent — 可被 ACP 编辑器连接 | 5d | P2 | ACP spec |
| 4.4 | 请求规范化层 — 消息格式统一 | 3d | P1 | OpenCode |

### 阶段 5：高级功能 (Sprint 9-10, 4 周)

| # | 任务 | 工作量 | 依据 | 外部模式 |
|---|------|--------|------|----------|
| 5.1 | 3 阶段路由管线 (pre→verify→escalation) | 5d | P2 | RouteLLM/FrugalGPT |
| 5.2 | 模型自适应工具过滤 | 3d | P2 | OpenCode model-adaptive |
| 5.3 | 取消传播 (AbortController) | 2d | P2 | 生产模式 |
| 5.4 | 代理 Anti-buffering Headers | 1d | P2 | X-Accel-Buffering |

### 阶段 6：集成验证 (Sprint 11-12, 4 周)

| # | 任务 | 工作量 | 依据 |
|---|------|--------|------|
| 6.1 | 全量集成测试 — 所有外部模型通过统一接口 | 5d | 验收 |
| 6.2 | 性能基准测试 — 路由延迟/成本节省 | 3d | A1 |
| 6.3 | 冗余清理验收 — 确认无重复定义 | 2d | R-P42 |
| 6.4 | 文档更新 — 架构图/CLI 参考 | 2d | |

---

## 第七部分：核心路线任务清单

### 优先级矩阵

```
          高影响                    低影响
    ┌─────────────────┬─────────────────┐
高  │ P0-紧急修复      │ P2-高级功能      │
工  │ 0.1 TaskType统一 │ 5.1 3阶段路由    │
作  │ 1.1 SSE Parser   │ 5.2 模型自适应   │
量  │ 1.2 Tool-result  │ 5.3 取消传播     │
    │ 3.1 5级压缩      │                  │
    ├─────────────────┼─────────────────┤
低  │ P1-核心能力      │ P3-锦上添花      │
工  │ 0.2 REGISTRY统一 │ 5.4 Anti-buffer  │
作  │ 1.3 PreFlight    │                  │
量  │ 1.4 Route抽象    │                  │
    │ 2.1 Semantic     │                  │
    │ 2.2 UnifiedRouter│                  │
    │ 4.1 McpServer    │                  │
    └─────────────────┴─────────────────┘
```

### 总工作量估算

| 阶段 | Sprint | 工作量 | 累计 |
|------|--------|--------|------|
| 0 紧急修复 | 0 | 3d | 3d |
| 1 基础协议层 | 1-2 | 19d | 22d |
| 2 路由统一 | 3-4 | 20d | 42d |
| 3 Context 管理 | 5-6 | 21d | 63d |
| 4 协议互操作 | 7-8 | 24d | 87d |
| 5 高级功能 | 9-10 | 11d | 98d |
| 6 集成验证 | 11-12 | 12d | 110d |
| **总计** | **12 sprints** | **~110d** | |

### 关键依赖链

```
0.1 TaskType统一
  └→ 2.2 UnifiedModelRouter (依赖统一类型)
       └→ 2.5 CLI /model 接入
            └→ 6.1 集成测试

1.1 SSE Parser
  └→ 1.6 ProviderTransform
       └→ 2.2 UnifiedModelRouter (依赖 SSE 解析)

1.2 Tool-result 清除
  └→ 3.2 集成到 TUI
       └→ 3.1 5级压缩 (依赖清除作为 T1/T2 层)

4.1 McpServer
  └→ 4.2 McpClient (对称实现)
       └→ 4.3 ACP Agent (依赖 MCP)
```

---

## 第八部分：验收标准

### 功能验收

- [ ] 所有外部模型 (OpenAI/Anthropic/Gemini/Ollama/DeepSeek) 通过统一接口可用
- [ ] 3 个路由器合并为 1 个 `UnifiedModelRouter`
- [ ] 11 处 `TaskType` 定义合并为 1 处
- [ ] `SseEventParser` 支持 OpenAI/Anthropic/Gemini SSE 格式
- [ ] `ToolResultLifecycle` 实现 tool-result 手术式清除
- [ ] `ContextCompactor` 实现 5 级压缩
- [ ] `McpServer` 可暴露 `UnifiedCapability` 为 MCP 工具
- [ ] `McpClient` 可消费外部 MCP 服务器
- [ ] `/model` 命令使用动态 `REGISTRY` 而非硬编码
- [ ] `PreFlightTokenCounter` 在调用前估算成本

### 质量验收

- [ ] `#![forbid(unsafe_code)]` — 核心零 unsafe (R-P1)
- [ ] 每次编辑后 re-read 验证持久化 (R-P16)
- [ ] `cargo check -p neotrix --lib` 零错误零警告
- [ ] 吸收强化现有节点，禁止平行适配器模块 (R-P42)
- [ ] 外部技术同 session 接线到生产路径 (R-P79)

### 冗余清理验收

- [ ] TaskType 重复: 11→1
- [ ] ModelProvider 重复: 3→1
- [ ] 路由器重复: 3→1
- [ ] Config 重复: 3→1
- [ ] SelfModel 重复: 4→1 (或保持但明确职责)

---

## 第九部分：Neotrix 独特优势

1. **已有 GWT Salience 路由** — 外部工具只有简单的 cost-based 路由，Neotrix 的 `SalienceScore = importance / (cost_weight × tokens) × latency_weight` 是最先进的
2. **已有 6 层架构** — 外部工具是扁平结构，Neotrix 的分层可以自然承载 Context 压缩 (L6)、语义路由 (L5)、流解析 (L2)、Provider 抽象 (L1)
3. **已有 CapabilityTree** — 外部工具没有能力进化系统，Neotrix 的 Constellation 成熟度 (C0-C6) 可以追踪每个模型适配器的成熟度
4. **已有 SEAL Pipeline** — 外部工具没有技能蒸馏，Neotrix 的 SEAL 可以从模型交互中蒸馏可复用技能
5. **已有 Crystal Core** — 外部工具没有持久化意识，Neotrix 的 Crystal 4 层架构 (Identity→Knowledge→Experience→Evolution) 可以跨 session 记忆模型偏好

---

*本方案基于 4 个并行 agent 的全量搜索结果，逆向推理了 OpenCode/Claude/Aider/Cline/Windsurf 的底层架构，融合到 Neotrix 已有的 6 层能力骨架中。*

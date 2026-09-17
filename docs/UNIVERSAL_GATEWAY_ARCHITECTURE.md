# NeoTrix Universal Model Gateway — Architecture Design

## 1. 核心理念

**"一个网关，适配所有外部模型"**

NeoTrix 桌面 App 作为通用 AI-native 开发工具，需要支持：
- 内置模型 (NeoTrix 自有推理)
- 外部 CLI agents (Claude Code, Codex, Gemini CLI)
- API 模型 (OpenAI, Anthropic, Google, Ollama)
- 本地模型 (llama.cpp, Ollama)

## 2. 架构分层

```
┌─────────────────────────────────────────────────────┐
│                   L6 Meta-Cognition                 │
│  GodAgent (任务分类) ←→ SemanticRouter (路由决策)    │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│              L5 Cognition — Model Gateway            │
│  ┌─────────────┐ ┌──────────────┐ ┌──────────────┐ │
│  │ CostGate    │ │ FallbackChain│ │ ProviderPool │ │
│  │ (成本门控)  │ │ (降级链)     │ │ (连接池)     │ │
│  └─────────────┘ └──────────────┘ └──────────────┘ │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│          L1 Action — Provider Abstraction            │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐       │
│  │ Claude │ │ Codex  │ │Gemini  │ │ Ollama │ ...   │
│  │ Provider│ │Provider│ │Provider│ │Provider│       │
│  └────────┘ └────────┘ └────────┘ └────────┘       │
└─────────────────────────────────────────────────────┘
```

## 3. 核心组件

### 3.1 ModelGateway (L5 — nt_core_model_gateway)

统一入口，所有模型调用经过此处。

```rust
pub struct ModelGateway {
    /// Provider 注册表
    providers: HashMap<String, Box<dyn LlmProvider>>,
    /// 成本门控
    cost_gate: CostGate,
    /// 降级链
    fallback_chain: FallbackChain,
    /// 路由策略
    strategy: RoutingStrategy,
}

impl ModelGateway {
    /// 统一调用入口
    pub async fn route(&self, request: GatewayRequest) -> Result<GatewayResponse, GatewayError> {
        // 1. 成本检查
        if !self.cost_gate.allow(&request) {
            return self.fallback_chain.cheaper(&request).execute().await;
        }
        // 2. 路由到目标 provider
        let provider = self.select_provider(&request)?;
        // 3. 执行 + 降级
        match provider.execute(request.clone()).await {
            Ok(resp) => Ok(resp),
            Err(_) => self.fallback_chain.next(&request).execute().await,
        }
    }
}
```

### 3.2 CostGate (成本门控)

```rust
pub struct CostGate {
    /// 每模型预算 (USD/月)
    budgets: HashMap<String, f64>,
    /// 已消耗
    consumed: HashMap<String, f64>,
    /// 成本权重 (task_type → model → weight)
    cost_weights: HashMap<String, HashMap<String, f64>>,
}

impl CostGate {
    /// 判断是否允许调用
    pub fn allow(&self, request: &GatewayRequest) -> bool {
        let budget = self.budgets.get(&request.model).unwrap_or(&f64::MAX);
        let used = self.consumed.get(&request.model).unwrap_or(&0.0);
        used < budget
    }
}
```

### 3.3 FallbackChain (降级链)

```rust
pub struct FallbackChain {
    /// 按成本排序的降级序列
    chains: HashMap<String, Vec<String>>,  // task_type → [model_id]
}

impl FallbackChain {
    /// 获取更便宜的替代模型
    pub fn cheaper(&self, request: &GatewayRequest) -> &str {
        self.chains.get(&request.task_type)
            .and_then(|chain| chain.iter().find(|m| **m != request.model))
            .unwrap_or(&"ollama:local")
    }
}
```

### 3.4 SemanticRouter (语义路由)

```rust
pub struct SemanticRouter {
    /// 任务分类器
    classifier: TaskClassifier,
    /// 置信度阈值
    confidence_threshold: f64,
    /// 路由表: task_type → preferred_model
    route_table: HashMap<String, String>,
}

impl SemanticRouter {
    pub fn route(&self, input: &str) -> RoutingDecision {
        let classification = self.classifier.classify(input);
        if classification.confidence >= self.confidence_threshold {
            RoutingDecision {
                model: self.route_table.get(&classification.task_type).cloned(),
                confidence: classification.confidence,
                reason: format!("High confidence: {}", classification.task_type),
            }
        } else {
            RoutingDecision {
                model: Some("gpt-4o".to_string()), // 兜底强模型
                confidence: classification.confidence,
                reason: "Low confidence, using strongest model".to_string(),
            }
        }
    }
}
```

## 4. Provider 注册表

### 4.1 已有 Provider (nt_io_provider)

- OpenAI-compatible (GPT-4o, DeepSeek, etc.)
- Anthropic (Claude)
- Ollama (local)

### 4.2 新增 Provider (BYOA 扩展)

- Claude Code CLI (stdio)
- Codex CLI (stdio)
- Gemini CLI (stdio)

### 4.3 统一接口

所有 provider 实现 `LlmProvider` trait:
- `execute(request) -> Response`
- `stream(request) -> Stream`
- `capabilities() -> Capabilities`
- `cost_estimate(request) -> f64`

## 5. 路由策略

### 5.1 任务类型 → 模型映射

| 任务类型 | 首选模型 | 降级模型 | 成本权重 |
|---------|---------|---------|---------|
| code_generation | Claude Code | Codex | 0.8 |
| code_review | Claude | Gemini | 0.6 |
| debugging | Claude Code | GPT-4o | 0.9 |
| documentation | Gemini | Claude | 0.3 |
| testing | Codex | Claude | 0.5 |
| architecture | Claude Opus | GPT-4o | 1.0 |
| simple_query | Ollama:local | Gemini Flash | 0.1 |

### 5.2 成本感知路由

```
Task → Classifier → CostGate → Provider → Response
                    ↓ (超预算)
              FallbackChain → Cheaper Provider
```

## 6. 实现阶段

| Phase | 组件 | 优先级 | 预估 |
|-------|------|--------|------|
| P1 | ModelGateway 核心 | P0 | 2天 |
| P2 | CostGate 成本门控 | P0 | 1天 |
| P3 | FallbackChain 降级链 | P0 | 1天 |
| P4 | SemanticRouter 语义路由 | P1 | 2天 |
| P5 | Provider Registry 注册表 | P1 | 1天 |
| P6 | Tauri Commands 接线 | P1 | 1天 |
| P7 | Frontend Gateway UI | P2 | 2天 |

## 7. 与现有模块关系

```
nt_core_model_router (现有) 
    ↓ 增强
nt_core_model_gateway (新建)
    ↓ 依赖
nt_io_provider (现有 LLM providers)
    ↓ 扩展
nt_core_byoa (现有 BYOA agents)
    ↓ 接入
nt_agent_identity (现有 agent 身份)
```

## 8. 关键设计决策

1. **单一入口**: 所有模型调用必须经过 `ModelGateway::route()`
2. **成本优先**: 超预算自动降级，不需人工干预
3. **置信度路由**: 低置信度任务自动升级到更强模型
4. **Provider 无关**: 新增 provider 只需实现 `LlmProvider` trait
5. **可观测**: 每次路由记录决策原因，供审计和优化

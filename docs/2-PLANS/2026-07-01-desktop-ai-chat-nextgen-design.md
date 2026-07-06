# NextGen AI 桌面聊天应用 — 架构设计规格 v1.0

> **涌现自**: 7轮外部吸收 (15+竞品, 12+论文, 20+技术实践), 3轮盲点迭代
> **设计哲学**: 非克隆, 非堆叠, 是涌现 — 每个决策背后都有论文/竞品/实践的证据链

---

## 目录

1. [核心设计原则](#1-核心设计原则)
2. [总体架构](#2-总体架构)
3. [L0: 核心引擎层](#3-l0-核心引擎层)
4. [L1: 流式UX层](#4-l1-流式ux层)
5. [L2: 知识记忆层](#5-l2-知识记忆层)
6. [L3: 可扩展层](#6-l3-可扩展层)
7. [数据流详解](#7-数据流详解)
8. [关键决策记录 (ADRs)](#8-关键决策记录-adrs)
9. [与其他方案对比](#9-与其他方案对比)
10. [实施路线图](#10-实施路线图)

---

## 1. 核心设计原则

### 1.1 四条涌现定律

| # | 定律 | 来源证据 | 影响 |
|---|------|---------|------|
| 1 | **流式是体验, 不是传输** | Andes QoE论文 (Liu'24): 首令牌延迟 + 平滑节奏 = 核心QoE; Just-in-Time论文 (CHI'26): 认知节奏对齐消除挫败感 | 流式层必须感知认知负载, 不做匀速吐字 |
| 2 | **缓存是架构, 不是优化** | SemantiCache/Zijian-Ni: 30-60%成本缩减; vCache: 在线学习最优阈值; 工业界已验证 | 三层缓存(精确+语义+提示)必须内建于Gateway, 非附加 |
| 3 | **MCP是生态, 不是协议** | MCP 2026-07-28: 无状态HTTP + MCP Apps + Tasks; 所有主要AI客户端已支持 | 工具生态不是可选特性, 是应用存在的基础 |
| 4 | **本地优先是权利, 不是模式** | KathaGPT/ShodhRAG/Pern: 所有竞品都支持离线LLM; Apple PCC: 隐私架构成为监管要求 | 应用必须能完全离线运行, 云API是可选的加速器 |

### 1.2 技术选型矩阵

```
┌─────────────────────────────────────────────────────────────────┐
│  层         │ 选型                     │ 竞品证据               │
├─────────────────────────────────────────────────────────────────┤
│ 应用外壳     │ Tauri 2.x (Rust)         │ Jessie/Nexus AI/ATHENA  │
│ 前端框架     │ React 19 + TypeScript    │ 8/10 竞品使用           │
│ 样式系统     │ Tailwind CSS v4          │ ShodhRAG/Nexus AI      │
│ 状态管理     │ Zustand + TanStack Query │ Nexus AI/Plugable Chat │
│ 流式IPC      │ Tauri Channel API        │ Tauri官方推荐, 替代events│
│ 二进制IPC    │ tauri-conduit(可选)       │ 64KB负载11.2x加速       │
│ 流式渲染     │ react-markdown + shiki    │ ATHENA/Jessie           │
│ 本地数据库   │ SQLite (rusqlite) + FTS5  │ 所有竞品                │
│ 向量存储     │ LanceDB (嵌入式)          │ ShodhRAG使用证明         │
│ 嵌入模型     │ ONNX Runtime (本地)       │ ShodhRAG E5多语言       │
│ 语义缓存     │ 内建3层缓存               │ vCache/SemantiCache     │
│ MCP客户端    │ 原生Rust MCP              │ MCP 2026-07-28新规范    │
│ 本地LLM      │ Sidecar + llama.cpp      │ KathaGPT/Pern/ATHENA    │
│ 密钥存储     │ keyring (系统级)          │ 设计规范/ATHENA已用     │
│ 插件系统     │ WASM + Manifest          │ LangBot/Mainframe       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 总体架构

### 2.1 四层同心圆架构 (非分层, 是领域)

```
                    ┌─────────────────────────┐
                    │    L3: 可扩展层           │
                    │  Plugin Runtime          │
                    │  MCP Apps Host           │
                    │  Skills Registry         │
                    │   ↑ 能力注册             │
                    ├─────────────────────────┤
                    │    L2: 知识记忆层         │
                    │  RAG Engine              │
                    │  3-Layer Cache           │
                    │  Episodic Memory         │
                    │   ↑ 检索+缓存            │
                    ├─────────────────────────┤
                    │    L1: 流式UX层           │
                    │  Cognitive Stream        │
                    │  Skeleton System         │
                    │  Cost Transparency       │
                    │   ↑ 事件+状态             │
                    ├─────────────────────────┤
                    │    L0: 核心引擎层         │
                    │  GatewayV2               │
                    │  Multi-Provider          │
                    │  Local LLM Sidecar       │
                    │  MCP Client              │
                    └─────────────────────────┘
```

**设计意图**: 不是传统的分层架构 (上层依赖下层), 而是**能力同心圆** — 外层扩展内层, 每层可独立替换, 数据流在层间通过类型化事件总线传递。

### 2.2 数据流全景

```
User Input
    │
    ▼
┌──────────────────────────────────────────────┐
│  L1 UX Layer                                  │
│  ┌──────────┐  ┌──────────┐  ┌─────────────┐ │
│  │ Skeleton │  │ Cost     │  │ Branching   │ │
│  │ System   │  │ Display  │  │ Engine      │ │
│  └────┬─────┘  └────┬─────┘  └──────┬──────┘ │
│       │              │               │         │
│  ┌────▼──────────────▼───────────────▼──────┐ │
│  │  CognitiveStream (Channel API)           │ │
│  │  - Token pacing                          │ │
│  │  - Reasoning step extraction             │ │
│  │  - Auto-scroll + cursor management       │ │
│  └────────────────┬─────────────────────────┘ │
└───────────────────┼───────────────────────────┘
                    │ invoke(stream_chat, ...)
                    ▼
┌──────────────────────────────────────────────┐
│  L0 Core Engine                               │
│  ┌──────────────────────────────────────────┐ │
│  │  GatewayV2                               │ │
│  │  ┌─────────┐ ┌──────────┐ ┌───────────┐ │ │
│  │  │Semantic │ │Prompt    │ │Circuit    │ │ │
│  │  │Cache    │ │Cache     │ │Breaker    │ │ │
│  │  └────┬────┘ └────┬─────┘ └─────┬─────┘ │ │
│  │       │           │              │        │ │
│  │  ┌────▼───────────▼──────────────▼─────┐ │ │
│  │  │  Provider Router                    │ │ │
│  │  │  - Cloud (Anthropic/OpenAI/Gemini)  │ │ │
│  │  │  - Local (Sidecar/llama.cpp)        │ │ │
│  │  │  - MCP Tools                        │ │ │
│  │  └────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────┘ │
└───────────────────┬───────────────────────────┘
                    │ SSE / stdio / HTTP
                    ▼
        Cloud APIs / Local LLM / MCP Servers
```

---

## 3. L0: 核心引擎层

### 3.1 GatewayV2 — 统一LLM网关

#### 证据链
- LiteLLM: 回退链+模型映射 → 迭代式3次回退
- Grob (Rust): 内联DLP, 90μs开销
- NeoTrix GatewayV2: 已验证的Circuit Breaker + Rate Limiter + Provider Pool

#### 设计

```rust
pub struct GatewayV2 {
    providers: HashMap<String, Box<dyn LlmProvider>>,
    state: RwLock<GatewayState>,
    semantic_cache: Arc<SemanticCache>,    // ← 内建, 非附加
    prompt_cache: Arc<PromptCache>,        // ← Anthropic KV Cache适配
    circuit_breaker: Arc<CircuitBreaker>,
    rate_limiter: Arc<RateLimiter>,
}

pub struct SemanticCache {
    // 三层: exact → embedding → prompt prefix
    exact: DashMap<CacheKey, CachedResponse>,
    embedding: HnswIndex<Embedding, CachedResponse>,
    prefix: LruCache<String, CachedResponse>,
    embedder: Box<dyn Embedder>,            // 本地ONNX, 零API调用
    threshold: f32,                         // 默认0.92 (vCache验证)
    ttl: Duration,
    // 论文级特性: 在线学习阈值 (vCache)
    adaptive_threshold: Option<AdaptiveThreshold>,
}

pub struct PromptCache {
    // Anthropic prompt caching support
    // KV cache reuse: 缓存token 90% off
    enabled: bool,
    break_points: Vec<CacheControlPoint>,
}
```

**流式Channel API** (替代Event System):

```rust
#[tauri::command]
async fn stream_chat(
    channel: Channel<StreamChunk>,    // ← Channel API: 点对点, 高效
    request: ChatRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 1. 检查语义缓存
    if let Some(cached) = state.gateway.semantic_cache.lookup(&request).await {
        for chunk in cached.replay() {
            channel.send(chunk)?;     // Channel.send() 比 emit() 轻量
        }
        return Ok(());
    }

    // 2. 构造流式请求 (考虑prompt caching)
    let stream = state.gateway.stream(&request).await?;

    // 3. Channel API 转发
    //    Channel 自动管理消息排序和清理
    while let Some(chunk) = stream.next().await {
        channel.send(chunk)?;
    }

    Ok(())
}
```

**为什么Channel API**: Tauri官方benchmark显示Channel比Event在流式场景中:
- 减少50%序列化开销 (Channel直接发送类型化数据)
- 自动生命周期管理 (Channel关闭时自动清理)
- 点对点不污染全局命名空间

### 3.2 Multi-Provider + 回退架构

```
 ProviderRouter
    │
    ├── Tier 1 (优先免费) ─── Groq / Pollinations / OpenRouter
    │     fallback ↓
    ├── Tier 2 (主要API) ─── Anthropic / OpenAI / Gemini
    │     fallback ↓
    ├── Tier 3 (本地) ────── Sidecar llama.cpp / Ollama
    │     fallback ↓
    └── Tier 4 (全部) ────── 强制HalfOpen重试所有Provider
```

**复合评分**:
```
score = (success_rate² / p95_latency) × cost_factor × health
```
优先免费 + 质量路由, 非简单轮询。

### 3.3 Local LLM Sidecar

#### 证据链
- KathaGPT: Just-in-Time下载llama-server, 统一streaming代码路径
- Pern: 嵌入llama-server进程管理
- Tauri官方: sidecar模式成熟

#### 设计

```rust
pub struct LocalLlmSidecar {
    // Just-in-Time 部署: 检测→下载→启动
    state: SidecarState,          // NotInstalled | Downloading | Ready | Running
    binary_path: PathBuf,
    api_base: String,             // 127.0.0.1:11435 (loopback only)
    health_check: JoinHandle<()>,

    // 模型管理
    models: Vec<LocalModel>,      // 从HuggingFace拉取GGUF
    active_model: Option<String>,
}

// 统一streaming代码路径:
// 无论是Cloud还是Local, 都走 stream_openai_compatible()
async fn stream_chat(request: ChatRequest) -> Result<Stream> {
    match resolve_route(&request).await? {
        Route::Cloud { provider, model } => {
            provider.stream(&request).await
        }
        Route::Local { model } => {
            LocalLlmSidecar::ensure_running(&model).await?;
            stream_openai_compatible(
                "http://127.0.0.1:11435/v1/chat/completions",
                &request,
            ).await
        }
    }
}
```

---

## 4. L1: 流式UX层

### 4.1 CognitiveStream — 认知感知流式引擎

#### 证据链
- **Andes QoE论文** (Liu'24): 定义QoE = f(TTFT, rhythm) — 首令牌延迟 + 节奏平滑度
- **Just-in-Time Tokens** (CHI'26): 结构化单元对齐阅读节奏, 减少42%挫败感
- **Streaming Fast and Slow** (UIST'25): 认知负载感知调速, 节省30%计算资源
- **Streaming LLMs Survey** (ACL'26): 首个系统化streaming LLM分类

#### 设计

```typescript
class CognitiveStream {
  // 三阶段节奏控制
  phase: 'init' | 'streaming' | 'complete';

  // 阶段1: 初始化 (0-2s)
  // 问题: 白屏等待 → 用户频繁切换任务 (CHI'26: DG1)
  // 解决: 1. 立即显示 skeleton (20%更快感知)
  //       2. 预填充预计答案形状 (代码/文本/表格)
  async onInit(): void {
    this.showSkeleton('analyzing');   // 问题分析骨架
    await this.waitForFirstToken();   // Channel API 首次回调
    this.skeleton.hide();
  }

  // 阶段2: 流式 (2-30s)
  // 问题: 匀速吐字忽略内容复杂度
  // 解决: 结构化单元缓冲 + 认知节奏控制
  async onToken(token: string, metadata: TokenMetadata): void {
    this.buffer.append(token);

    // 结构化单元检测 (CHI'26: DG2)
    if (this.buffer.completesUnit()) {
      const unit = this.buffer.drain();
      const cognitiveLoad = estimateLoad(unit);  // (UIST'25)
      const delay = this.calculatePacing(cognitiveLoad);
      
      await this.delay(delay);  // 认知对齐暂停
      this.display(unit);       // 释放完整单元
    }
  }

  // 认知负载估计 (UIST'25 核心贡献)
  private estimateLoad(unit: TextUnit): CognitiveLoad {
    return {
      codeDensity: countCodeTokens(unit) / unit.length,
      entityDensity: countNewEntities(unit),
      structuralDepth: countNesting(unit),
      // high → 减慢pacing, small units
      // low  → 加快pacing, large units
    };
  }

  // 阶段3: 完成
  // 问题: 突然结束缺乏反馈
  // 解决: 渐出光标, 显示统计
  onComplete(stats: StreamStats): void {
    this.cursor.fadeOut();
    this.showCostBadge(stats);       // 成本透明
    this.showLatency(stats.duration); // 延迟透明
  }
}
```

### 4.2 Skeleton System — 非加载态的骨架

#### 证据链
- SaaS UX研究: Skeleton比spinner感觉快20%
- Designing for LLM Latency (AI/TLDR): Skeleton设定空间预期, 布局无跳变
- 300-700ms脉冲周期最优

#### 设计

```tsx
// 智能骨架: 根据请求类型预测输出形状
function SmartSkeleton({ requestType }: { requestType: RequestType }) {
  // 分析请求 → 预测响应形状
  const shape = useMemo(() => predictShape(request.text), [request.text]);

  return (
    <div className="assistant-entry" data-testid="skeleton">
      {shape === 'code' && <CodeSkeleton lines={7} language={shape.language} />}
      {shape === 'list' && <ListSkeleton items={4} />}
      {shape === 'table' && <TableSkeleton rows={3} cols={4} />}
      {shape === 'text' && <TextSkeleton paragraphs={2} />}
      {shape === 'analysis' && (
        <div className="skeleton-thinking">
          <ThinkingStages stages={['理解问题', '分析上下文', '生成回答']} />
        </div>
      )}
    </div>
  );
}
```

### 4.3 Reasoning Steps Streaming — 流式推理过程

#### 证据链
- Claude: 在回答前显示思考过程, 用户信任度+37%
- Atlas agent traces: 流式展示推理步骤+工具调用, 让AI感觉有能力而非魔法
- LLM UX Patterns (arablex): 步骤流式让失败可追溯

#### 设计

```typescript
// 从StreamChunk中提取结构化事件
type StreamEvent =
  | { type: 'reasoning'; content: string }      // 推理步骤
  | { type: 'tool_call'; tool: string; args: any } // 工具调用
  | { type: 'tool_result'; tool: string; result: string }
  | { type: 'delta'; content: string }           // 常规token
  | { type: 'done'; usage: TokenUsage };

// 前端渲染
function MessageStream({ events }: { events: StreamEvent[] }) {
  return (
    <div className="message-stream">
      {events.map((event, i) => {
        switch (event.type) {
          case 'reasoning':
            return <CollapsibleReasoning key={i} content={event.content} />;
          case 'tool_call':
            return <ToolCallBadge key={i} tool={event.tool} args={event.args} />;
          case 'tool_result':
            return <CollapsibleResult key={i} result={event.result} />;
          case 'delta':
            return <StreamingText key={i} text={event.content} />;
        }
      })}
      <StreamingCursor active={!isDone} />
      <CostBadge usage={usage} />  // ← 成本透明: 实时token/latency
    </div>
  );
}
```

### 4.4 Cost Transparency — 成本内建于UI

#### 证据链
- LLM UX Patterns (arablex): "Put cost where the work happens" — 用户需要对不可见成本有感知
- NeoTrix CostTracker: 已验证的模型定价表 + 实时追踪

#### 设计

```tsx
function CostBadge({ usage, model }: { usage: TokenUsage; model: string }) {
  const cost = calculateCost(model, usage);
  return (
    <span className="cost-badge" title={`${usage.prompt_tokens}→${usage.completion_tokens}`}>
      ≈${cost.toFixed(4)}
      <span className="cost-detail">
        {model} · {formatDuration(usage.latency_ms)}
      </span>
    </span>
  );
}
```

### 4.5 系统快捷键增强

#### 证据链
- Nexus AI: Cmd+K command palette
- Tauri global_shortcut plugin 官方支持
- Menubar app 模式 (macOS)

#### 设计方案

| 快捷键 | 行为 | 实现方式 |
|--------|------|---------|
| `Cmd/Ctrl + K` | Command Palette (全局模糊搜索) | 前端 + Rust索引 |
| `Cmd/Ctrl + Shift + H` | 全局唤起/隐藏窗口 | `tauri_plugin_global_shortcut` |
| `Cmd/Ctrl + Shift + M` | 切换深色/浅色 | 系统级热键 |
| `Cmd/Ctrl + ;` | 内联AI输入 (任何应用) | 进阶: 全局选区增强 |
| `Cmd/Ctrl + P` | 快速切换到最近会话 | `tauri_plugin_global_shortcut` |

系统托盘菜单 (macOS菜单栏/Windows系统托盘):

```
┌─────────────────────┐
│ 💬 NovaChat          │
│─────────────────────│
│ Show Window          │
│ New Conversation     │
│─────────────────────│
│ ⚡ Quick Ask...      │  ← 弹出小窗口快速提问
│─────────────────────│
│ Model: Sonnet 4     │  ← 快捷切换模型
│─────────────────────│
│ Quit                 │
└─────────────────────┘
```

### 4.6 流式渲染性能优化

#### 证据链
- CHI'26 Just-in-Time Tokens: 使用 `requestAnimationFrame` 合并渲染帧
- AssetHoard: JSON序列化70MB → 8s扫描的教训
- Tauri benchmark: 双层React状态更新会导致卡顿

```typescript
// 流式渲染节流器
class StreamRenderer {
  private buffer: string[] = [];
  private rafId: number | null = null;

  push(token: string) {
    this.buffer.push(token);
    if (!this.rafId) {
      this.rafId = requestAnimationFrame(() => this.flush());
    }
  }

  private flush() {
    const batch = this.buffer.join('');
    this.buffer = [];
    this.rafId = null;
    // 合并后一次性更新, 避免逐token触发React响应式
    this.setState(prev => prev + batch);
  }
}
```

---

## 5. L2: 知识记忆层

### 5.1 Hybrid RAG Engine

#### 证据链
- ShodhRAG: LanceDB + Tantivy + RRF融合, 在单一Tauri二进制中
- NexusRAG: 3路并行 (向量+KG+关键词), 交叉编码器rerank
- CoreRag: HyDE扩展 + 多查询融合 + 时间衰减评分
- Knovex: SQLite FTS5 + 稠密ANN + 节段图检索

#### 设计

```rust
pub struct RagEngine {
    dense: LanceDb,              // 向量索引 (嵌入式, 零服务)
    sparse: TantivyIndex,        // 全文搜索
    reranker: CrossEncoder,      // ONNX: bge-reranker-v2-m3
    knowledge_graph: LightRag,   // 实体-关系图 (可选)
    embedder: Box<dyn Embedder>, // E5多语言 (ONNX, 本地)
}

impl RagEngine {
    async fn retrieve(&self, query: &str, top_k: usize) -> Vec<Chunk> {
        // 阶段1: 并行检索
        let (dense, sparse) = tokio::join!(
            self.dense.search(query, top_k * 3),   // over-fetch
            self.sparse.search(query, top_k * 3),
        );

        // 阶段2: RRF融合 (Reciprocal Rank Fusion)
        let fused = reciprocal_rank_fusion(dense, sparse);

        // 阶段3: 交叉编码器重排
        let reranked = self.reranker.rerank(query, fused, top_k).await?;

        Ok(reranked)
    }
}
```

### 5.2 三层缓存 (内建于Gateway)

```
Request → [Layer 1: Exact Match]
              │ 命中 → 零延迟返回
              │ 未命中 ↓
           [Layer 2: Semantic Match]
              │ 命中 → <50ms 返回 (30-60%成本节省)
              │ 未命中 ↓
           [Layer 3: Prompt Prefix Cache]
              │ 命中 → KV Cache复用 (90%输入token折扣)
              │ 未命中 ↓
           [Provider API Call]
              │
              ▼
           Cache Response (异步回填 Layer 1 + 2)
```

```rust
pub struct SemanticCache {
    exact: DashMap<CacheKey, CacheEntry>,    // O(1) SHA256
    semantic: HnswIndex<f32, CacheEntry>,     // O(log N) HNSW
    embedder: Arc<dyn Embedder>,              // 本地ONNX
    threshold: f32,                            // 默认0.92
    adaptive: Option<AdaptiveThreshold>,       // vCache在线学习

    // 失效策略
    ttl: Duration,
    max_entries: usize,                        // LRU驱逐
    invalidate_on_write: bool,                 // 写操作后驱逐相关缓存
}
```

**vCache风格自适应阈值**: 使用在线学习跟踪缓存命中/误报率, 动态调整0.85-0.98阈值区间, 保证用户定义的错误率上界。

### 5.3 Episodic Memory

超越纯SQLite会话历史, 加入语义记忆层:

```rust
pub struct EpisodicMemory {
    // 用户偏好学习: 常用模型, 常用温度, 主题偏好
    user_preferences: UserProfile,

    // 跨会话检索: "上个月讨论的那个bug"
    cross_session: CrossSessionIndex,

    // 模式识别: 用户经常问哪类问题
    pattern_detector: PatternDetector,
}
```

---

## 6. L3: 可扩展层

### 6.1 MCP Integration (核心差异化)

#### 证据链
- MCP 2026-07-28规范: 无状态HTTP, MCP Apps, Tasks扩展
- Jessie: MCP Host (stdio + HTTP, 含回退)
- Nexus AI: 完整MCP工具系统
- CoreRag: MCP Server暴露知识库

```rust
pub struct McpHost {
    servers: Vec<McpServerConnection>,
    // 2026-07-28: 无状态HTTP, 无需session store
    transport: McpTransport,  // stdio | streamable_http
    app_bridge: Option<AppBridge>,  // MCP Apps UI渲染器
}

pub struct McpServerConnection {
    name: String,
    transport: McpTransport,
    capabilities: ServerCapabilities,  // 初始化时协商
    cache: Arc<SemanticCache>,         // tools/list缓存 (带ttlMs)
}
```

**MCP Apps支持** (2026年差异化特性):

```
MCP Server → ui://resource URI → Host渲染sandboxed iframe
                                   ↓
                            JSON-RPC over postMessage
                                   ↓
                           LLM ↔ Host ↔ Server
```

用户可在对话中操作Figma设计、更新Asana任务、编辑Slack消息 — 所有交互通过标准MCP审计路径。

### 6.2 Plugin System — WASM + Manifest

#### 证据链
- LangBot: 进程隔离插件 (stdio/WebSocket双模式)
- Mainframe: manifest.json + activate() 模式
- xNet: 四层复杂度 (Scripts/Extensions/Services/Integrations)
- ByteDance UI-TARS: AgentPlugin + AgentComposer

```rust
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn activate(&self, ctx: PluginContext) -> Result<()>;
    fn deactivate(&self) -> Result<()>;

    // 贡献点
    fn tools(&self) -> Vec<ToolDefinition>;
    fn commands(&self) -> Vec<CommandDefinition>;
    fn ui_panels(&self) -> Vec<PanelDefinition>;
    fn hooks(&self) -> Vec<HookRegistration>;
}
```

**安全模型**: 插件声明capabilities, 运行时强制执行:
- `storage` → SQLite数据库访问
- `ui:panels` → UI面板注册
- `network` → HTTP请求
- `fs:read` / `fs:write` → 文件系统

### 6.3 Skills Library

```typescript
interface Skill {
  id: string;
  name: string;
  description: string;
  trigger: SkillTrigger;   // 关键词/正则/意图
  systemPrompt: string;
  tools: string[];          // 需要启用的工具
  samples: Example[];       // 示例对话
}
```

---

## 7. 数据流详解

### 7.1 完整请求生命周期

```
1. User 输入 "分析这个项目的架构"
    │
2. L1: SmartSkeleton 预测 "analysis" 形状
    │   → 显示 "理解项目" → "扫描结构" → "生成分析" 三阶段进度
    │
3. L1: CognitiveStream 创建 Channel<StreamChunk>
    │
4. L0: GatewayV2.invoke(stream_chat, request)
    │
5. L0: SemanticCache.lookup(request)
    │   → 命中: 启动缓存的replay, 走快速路径
    │   → 未命中: 继续
    │
6. L0: ProviderRouter.resolve(request)
    │   → 检查网络 → 选择最优Provider
    │
7. L1: Channel.send(chunk) 实时推送
    │
8. L2: (可选) RAG检索增强
    │   → 检索相关知识 → 注入上下文
    │
9. L0: Cache.async_store(response) 异步缓存
    │   → exact: SHA256 精确匹配
    │   → semantic: 嵌入后加入HNSW索引
    │   → prefix: Anthropic prompt caching标记
    │
10. L1: StreamStats 显示成本 + 延迟
```

### 7.2 离线模式数据流

```
User Input
    │
    ├─ Online? ──→ GatewayV2 (Cloud API)
    │
    └─ Offline? ──→ LocalLlmSidecar
                      │
                      ├─ llama.cpp on CPU/GPU
                      ├─ Qwen3-4B GGUF (默认)
                      ├─ 或用户已下载的其他GGUF
                      │
                      └─ RAG (本地):
                           ├─ LanceDB向量检索
                           ├─ FTS5全文搜索
                           └─ 本地嵌入 (ONNX)
```

---

## 8. 关键决策记录 (ADRs)

### ADR-1: 为什么用Channel API不用Event System

- **状态**: Accepted
- **证据**: Tauri官方文档 + DeepWiki分析; AssetHoard 120K文件性能教训
- **决策**: 所有流式IPC使用Channel<StreamChunk>, 不再使用 `emit('stream-chunk')`
- **理由**: Channel点对点避免全局命名空间污染, 自动生命周期管理, 类型安全
- **后果**: 多窗口场景需要为每个窗口创建独立Channel

### ADR-2: 为什么三层缓存内建于Gateway

- **状态**: Accepted
- **证据**: SemantiCache 30-60%节省; vCache用户定义错误率保证; Anthropic prompt caching 90%折扣
- **决策**: 缓存不是附加中间件, 是Gateway核心路径的一部分
- **后果**: 需要在首次请求增加~5ms嵌入时间, 但后续命中节省5-30s

### ADR-3: 为什么选择LanceDB而非ChromaDB

- **状态**: Accepted  
- **证据**: ShodhRAG生产验证; 嵌入式无需服务进程; Apache Arrow零拷贝
- **决策**: 使用LanceDB作为嵌入式向量数据库
- **理由**: 与ChromaDB相比减少了"启动Chroma服务"的开销; 与Rust生态集成更紧密

### ADR-4: 为什么使用Sidecar而非内嵌llama.cpp

- **状态**: Accepted
- **证据**: KathaGPT/Pern已验证; Tauri官方推荐; llama-cpp-2 crate存在但版本不兼容风险
- **决策**: llama.cpp作为sidecar进程管理, 统一OpenAI-compatible API
- **理由**: sidecar崩溃不影响主进程; 独立升级; 统一流式路径

### ADR-5: 为什么选择tanstack query管理服务端状态

- **状态**: Accepted
- **证据**: Nexus AI使用; TanStack Query v5 + Zustand v5黄金组合
- **决策**: Zustand管理UI状态, TanStack Query管理服务端状态 (会话列表/消息历史)
- **理由**: TanStack Query自带缓存/重试/乐观更新; Zustand单独处理高频流式写入

---

## 9. 与现有方案对比

| 维度 | NovaChat (原始设计) | 本设计 | Claude Desktop | ChatGPT Desktop |
|------|-------------------|--------|----------------|-----------------|
| 提供商支持 | 仅Anthropic | 8+提供商 + 本地 | 仅Anthropic | 仅OpenAI |
| 本地LLM | ❌ | ✅ Sidecar llama.cpp | ❌ | ❌ |
| RAG知识库 | ❌ | ✅ Hybrid (向量+FTS+rerank) | ❌ | ❌ |
| MCP支持 | ❌ | ✅ 2026-07-28规范 + MCP Apps | ✅ (有限) | ✅ (有限) |
| 语义缓存 | ❌ | ✅ 3层内建 | ❌ | ❌ |
| 插件系统 | ❌ | ✅ WASM + Manifest | ❌ | ❌ |
| 离线能力 | ❌ (浏览记录) | ✅ (完整对话+本地推理) | ❌ | ❌ |
| 流式UX | 基本 | 认知负载感知 + 骨架 | 基本 | 基本 |
| 成本透明 | token计数 | 实时$+延迟显示 | ❌ | ❌ |
| 系统托盘 | ❌ | ✅ 菜单栏 + 全局快捷键 | ✅ | ❌ |
| 自动更新 | ❌ | ✅ (Tauri updater) | ✅ | ✅ |
| i18n | ❌ | ✅ 从v1开始 | ✅ | ✅ |
| 会话分支 | ❌ | ✅ 任意消息分支 | ❌ | ❌ |
| 导出格式 | Markdown | MD/JSON/PDF + 截图 | MD | MD |
| 推理步骤 | ❌ | ✅ 流式推理过程 | ✅ | ❌ |
| 二进制大小 | ~15MB | ~15MB + 可选~2GB GGUF | ~200MB | ~300MB |

---

## 10. 实施路线图

### Phase 0: 骨架 (1-2周)
- [ ] Tauri 2 + React 19 脚手架, 三栏布局
- [ ] Design tokens CSS变量 (浅/深色主题)
- [ ] Channel API 流式打通 (invoke → Channel.send → 渲染)
- [ ] SQLite + FTS5 会话持久化
- [ ] 系统托盘 + 全局快捷键 `Cmd+Shift+H`

### Phase 1: 核心引擎 (2-3周)
- [ ] GatewayV2: Anthropic + OpenAI + Gemini 提供商
- [ ] Semantic Cache (exact + embedding)
- [ ] Prompt Cache (Anthropic KV cache)
- [ ] Streaming UX: 骨架系统 + 认知节奏控制
- [ ] 多会话管理 + 搜索 + 自动标题生成

### Phase 2: 知识 + 工具 (2-3周)
- [ ] MCP Client (stdio + HTTP/SSE)
- [ ] MCP Apps支持 (sandboxed iframe渲染器)
- [ ] Hybrid RAG: LanceDB + Tantivy
- [ ] 本地嵌入 (ONNX Runtime)
- [ ] 文档上传 + 问答

### Phase 3: 离线 + 插件 (2-3周)
- [ ] Local LLM Sidecar (llama.cpp)
- [ ] 统一 offline/online 流式路径
- [ ] Plugin System (WASM + Manifest)
- [ ] i18n (EN/ZH 初始)
- [ ] Conversation Branching

### Phase 4: 完善 (持续)
- [ ] Cost Transparency 仪表板
- [ ] Episodic Memory (跨会话检索)
- [ ] 性能优化 (binary IPC)
- [ ] 自动更新 + 签名
- [ ] E2E测试覆盖

---

## 附: 证据索引

| 编号 | 来源 | 类型 | 提取的核心洞见 |
|------|------|------|--------------|
| E1 | Andes QoE (Liu'24) | 论文 | QoE = f(TTFT, rhythm); token级抢占调度提升4.7x QoE |
| E2 | Just-in-Time Tokens (CHI'26) | 论文 | 结构化单元对齐阅读节奏, 减少42%挫败感 |
| E3 | Streaming Fast&Slow (UIST'25) | 论文 | 认知负载感知调速, 节省30%计算资源 |
| E4 | Streaming LLMs Survey (ACL'26) | 论文 | 首个系统化streaming分类: 并发/顺序/增量 |
| E5 | Jessie | 竞品 | OpenRouter统一多模型 + MCP Host + Tavily搜索 |
| E6 | Nexus AI | 竞品 | 多提供商 + MCP + Cmd+K + shadcn/ui |
| E7 | ATHENA | 竞品 | 本地LLM + PDF/EPUB/CSV + 完全离线 |
| E8 | ShodhRAG | 竞品 | LanceDB+Tantivy RAG + E5嵌入 + ONNX + Rust原生 |
| E9 | KathaGPT | 竞品 | Sidecar llama.cpp + 统一流式路径 + 模型目录 |
| E10 | MCP 2026-07-28 | 规范 | 无状态HTTP + MCP Apps + Tasks扩展 |
| E11 | Tauri Channel API | 官方 | 点对点流式, 生命周期自动管理 |
| E12 | tauri-conduit | 工具 | 64KB负载11.2x加速 (202μs vs 2.27ms) |
| E13 | SemantiCache | 工具 | 30-60%成本节省, HNSW索引 |
| E14 | LangBot Plugin | 竞品 | 进程隔离 + stdio/WebSocket双模式 |
| E15 | Mainframe Plugin | 竞品 | manifest + activate + 能力声明 |
| E16 | NeoTrix GatewayV2 | 内部 | 已验证的Circuit Breaker + Provider Pool |
| E17 | LLM UX Patterns | 指南 | 成本透明 + 推理步骤流式 + 骨架屏 |
| E18 | AssetHoard Tauri实践 | 案例 | 70MB JSON教训: 保持重数据在Rust端 |

---

> **版本**: v1.0 | **涌现日期**: 2026-07-01
> **设计者**: NeoTrix AI (依据6轮外部吸收)
> **协议**: 本文档为架构设计规范, 不包含任何第三方机密信息

# AI-Native Desktop Application Architecture Patterns — Research Report

**Date**: 2026-09-12
**Sources**: 30+ articles, 8 open-source projects analyzed
**Purpose**: Extract reusable patterns for NeoTrix Tauri desktop architecture

---

## Executive Summary

分析了 Ollama Desktop、LM Studio、Jan.ai、Open WebUI、GPT4All 以及 20+ 其他开源项目后，提取出 5 个核心架构模式。所有成功项目共享一个共识：**本地推理引擎 + 统一 API 抽象 + 进程隔离 = 可维护的 AI 桌面应用**。

---

## Pattern 1: Sidecar Inference Engine（旁路推理引擎）

### 核心思想
本地 LLM 推理作为独立子进程运行，桌面应用通过 OpenAI 兼容 API 与之通信。推理引擎崩溃不影响 UI。

### 开源项目证据
| 项目 | 推理引擎 | 通信方式 | 端口 |
|------|---------|---------|------|
| **Ollama** | llama.cpp (Go runner) | HTTP REST + SSE | 11434 |
| **LM Studio** | llama.cpp (llmster daemon) | HTTP REST + SSE | 1234 |
| **Jan.ai** | llama.cpp (Tauri plugin) | OpenAI 兼容 API | 1337 |
| **GPT4All** | llama.cpp (C++ binding) | HTTP REST | 4891 |
| **KathaGPT** | llama.cpp (Rust sidecar) | loopback HTTP | 11435 |
| **Pern** | llama.cpp (Tauri managed) | SSE | 4891 |
| **Locally Uncensored** | Ollama (Tauri proxy) | HTTP proxy | 11434 |

### 架构伪代码

```rust
// === SidecarProcess — 推理引擎生命周期管理 ===

struct SidecarProcess {
    handle: ChildProcess,
    port: u16,
    base_url: String,
    health_checker: HealthChecker,
    restart_policy: RestartPolicy,
}

impl SidecarProcess {
    /// JIT 模式：首次推理时才启动引擎
    async fn ensure_running(&mut self) -> Result<()> {
        if !self.is_alive() {
            self.spawn()?;
        }
        // 等待 HTTP 端口就绪 (最多 30s)
        self.health_checker.wait_until_ready(&self.base_url, Duration::from_secs(30)).await?;
        Ok(())
    }

    /// 内存管理：Keep-Alive + TTL 自动卸载
    fn manage_ttl(&mut self, idle_timeout: Duration) {
        // LM Studio 模式：闲置 60min 自动卸载模型
        // Ollama 模式：引用计数 + 5min keep-alive
    }
}

// === UnifiedInferenceRouter — 统一推理路由 ===

enum ModelRoute {
    Local { engine: SidecarProcess, model: String },
    Cloud { provider: CloudProvider, api_key: SecretString },
}

async fn route_request(route: &ModelRoute, request: ChatRequest) -> impl Stream<Item = Token> {
    match route {
        ModelRoute::Local { engine, model } => {
            engine.ensure_running().await?;
            stream_openai_compatible(&engine.base_url, model, request).await
        }
        ModelRoute::Cloud { provider, api_key } => {
            stream_cloud_api(provider, api_key, request).await
        }
    }
}

// === 模型管理目录 ===

struct ModelCatalog {
    models: Vec<ModelEntry>,
    downloader: AsyncDownloader,
}

struct ModelEntry {
    name: String,
    source: ModelSource,      // HuggingFace / Ollama registry
    format: ModelFormat,      // GGUF / SafeTensors
    quantization: QuantLevel, // Q4_K_M, Q5_K_S, etc.
    required_ram: u64,        // bytes
    required_vram: Option<u64>,
    checksum: Sha256,
}

impl ModelCatalog {
    /// 下载进度通过 SSE 推送到前端
    async fn download_model(&self, model: &ModelEntry) -> impl Stream<Item = DownloadProgress> {
        let tmp_path = self.downloader.download_to_temp(&model.source, &model.checksum).await;
        // 完成后 rename 防止损坏文件
        fs::rename(&tmp_path, &model.local_path()).await?;
        yield DownloadProgress::Complete;
    }
}
```

### NeoTrix 映射
| NeoTrix 组件 | 对应模式 | 当前状态 |
|-------------|---------|---------|
| `nt_io` (界面使徒) | Sidecar 生命周期管理 | 需要实现 |
| `nt_act` (行动执行者) | 统一路由分发 | 部分存在 |
| `nt_world` (虚空探索者) | 模型下载/管理 | 需要实现 |
| `nt_physical` (具身骨架) | GPU 检测/资源调度 | 需要实现 |
| Tauri v2 Shell | WebView ↔ Rust IPC | ✅ 已有 |

---

## Pattern 2: Typed Provider Abstraction（类型化提供者抽象）

### 核心思想
统一接口适配 N 个 LLM 提供者（本地/云端），UI 和业务逻辑永远不直接调用特定 API。格式适配器处理协议差异。

### 开源项目证据
| 项目 | 支持提供者数 | 适配器模式 |
|------|------------|-----------|
| **Open WebUI** | 20+ (Ollama, OpenAI, Anthropic, Google...) | Pipe 抽象 |
| **Jan.ai** | 15+ (本地 + 云) | Extension 系统 |
| **Grasberg** | 120+ providers | Native adapters |
| **Orkas** | 7+ (Claude, OpenAI, Gemini, DeepSeek...) | BYO keys |
| **xiaodazi** | 6 providers + Ollama | Format adapters |
| **Accomplish** | 15+ providers | Adapter pattern |

### 架构伪代码

```rust
// === LLMProvider trait — 所有提供者的统一契约 ===

#[async_trait]
trait LLMProvider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> ProviderCapabilities;

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn chat_stream(&self, request: ChatRequest) -> Result<Pin<Box<dyn Stream<Item = TokenChunk>>>>;

    // 扩展能力（可选）
    async fn embed(&self, text: &str) -> Result<Vec<f32>> { unimplemented!() }
    async fn list_models(&self) -> Result<Vec<ModelInfo>> { unimplemented!() }
    fn supports_tools(&self) -> bool { false }
    fn supports_vision(&self) -> bool { false }
}

// === 格式适配器 — 处理协议差异 ===

struct OpenAIAdapter { base_url: String, api_key: SecretString }
struct AnthropicAdapter { api_key: SecretString }
struct OllamaAdapter { base_url: String }

// xiaodazi 的做法：Claude content blocks vs OpenAI function calling vs Gemini parts
// 统一转换为内部 ChatRequest 格式
impl LLMProvider for OpenAIAdapter {
    async fn chat_stream(&self, request: ChatRequest) -> Result<Stream<Item = TokenChunk>> {
        let openai_req = self.to_openai_format(&request);
        let response = reqwest::Client::new()
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&openai_req)
            .send().await?;
        Ok(parse_sse_stream(response))
    }
}

// === 模型路由器 — 故障转移 + 健康跟踪 ===

struct ModelRouter {
    providers: Vec<Box<dyn LLMProvider>>,
    health: HealthTracker,
    fallback_chain: Vec<ProviderId>,
}

impl ModelRouter {
    async fn route(&self, request: ChatRequest) -> Result<Box<dyn Stream<Item = TokenChunk>>> {
        // 1. 检查用户指定的 provider
        if let Some(provider) = self.find_provider(&request.model) {
            if self.health.is_healthy(provider.id()) {
                return provider.chat_stream(request).await;
            }
        }
        // 2. 自动故障转移：按 fallback_chain 顺序尝试
        for provider_id in &self.fallback_chain {
            let provider = self.get_provider(provider_id);
            match provider.chat_stream(request.clone()).await {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    self.health.record_failure(provider_id, e);
                    continue;
                }
            }
        }
        Err(AllProvidersFailed)
    }
}

// === 离线/在线混合策略 ===

enum HybridStrategy {
    LocalFirst { cloud_fallback: bool },
    CostAware { budget_per_token: f64 },
    PrivacyFirst { sensitive_patterns: Vec<Regex> },
    LatencyAware { max_local_ms: u64 },
}

fn select_provider(strategy: &HybridStrategy, task: &Task) -> ProviderId {
    match strategy {
        HybridStrategy::LocalFirst { .. } => {
            if task.is_privacy_sensitive() || !task.needs_frontier_model() {
                ProviderId::Ollama  // 本地优先
            } else {
                ProviderId::Claude  // 需要高质量时回退云端
            }
        }
        HybridStrategy::CostAware { budget } => {
            if task.estimated_tokens() as f64 * LOCAL_COST < *budget {
                ProviderId::Ollama  // 本地 = 0 token cost
            } else {
                ProviderId::CheapestCloud  // 选择最便宜的云端
            }
        }
        _ => ProviderId::Ollama,
    }
}
```

### NeoTrix 映射
| NeoTrix 组件 | 对应模式 | 当前状态 |
|-------------|---------|---------|
| `nt_io::llm_providers` | LLMProvider trait | 需要统一 |
| `nt_core` (E8引导者) | 路由决策 (GWT salience) | 部分存在 |
| `nt_shield` (影卫) | 隐私检测/过滤 | 需要实现 |
| GWT 注意力路由 | Cost-Aware Routing (Axiom A1) | ✅ 已有概念 |
| KB embedding vs VSA embedding | 提供者抽象 | ✅ 已有 |

---

## Pattern 3: Secure IPC Boundary（安全 IPC 边界）

### 核心思想
WebView 被视为不可信输入源。所有 OS 级操作通过类型化、权限受限的 Rust 命令暴露。没有 "localhost = trust"。

### 开源项目证据
| 项目 | IPC 安全模型 | 权限控制 |
|------|------------|---------|
| **NOVA Desktop** | Named Pipe + nonce + DPAPI | Capability-scoped commands |
| **KathaGPT** | Rust 命令 (127.0.0.1 only) | CSP 白名单 |
| **Accomplish** | 主进程隔离 | 文件夹级权限 + 操作审批 |
| **xiaodazi** | Rust 命令 + HITL 确认 | 危险操作需人工确认 |
| **OpenHuman** | Tauri IPC + Ollama localhost | CSP 限制 |
| **Locally Uncensored** | Rust 代理层 | CORS bypass 通过 Rust |

### 架构伪代码

```rust
// === Capability-Scoped Commands — 权限受限的 Tauri 命令 ===

/// 每个命令声明自己的权限范围
#[tauri::command]
#[permissions("model:read, model:load")]
async fn load_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<ModelStatus, AppError> {
    // 前置检查：模型是否在白名单中
    if !state.model_catalog.is_trusted(&model_id) {
        return Err(AppError::UntrustedModel { model_id });
    }
    state.model_manager.load(&model_id).await
}

/// 危险操作需要 HITL (Human-In-The-Loop) 确认
#[tauri::command]
#[permissions("file:write")]
async fn execute_file_action(
    state: State<'_, AppState>,
    action: FileAction,
    plan_hash: Sha256,
) -> Result<(), AppError> {
    // 重新验证前置条件（NOVA 模式：不信任旧批准）
    action.validate_preconditions(&state.current_state)?;

    // 哈希绑定：操作必须匹配已批准的计划
    if action.plan_hash() != plan_hash {
        return Err(AppError::PlanMismatch);
    }

    // 拒绝符号链接/联接点（防止重定向攻击）
    if action.is_reparse_point() {
        return Err(AppError::ReparsePointRejected);
    }

    state.file_action_engine.execute(action).await
}

// === CSP 配置 — 严格网络隔离 ===

const CSP_POLICY: &str = "
    default-src 'self';
    connect-src 'self' http://localhost:11434 http://localhost:1234;
    img-src 'self' data: blob:;
    script-src 'self';
    style-src 'self' 'unsafe-inline';
";

// === 操作审批状态机 ===

enum ActionState {
    Draft,           // Agent 准备提案，未改变持久状态
    PreviewReady,    // 展示将发生什么
    PendingApproval, // 等待用户确认
    Approved,        // 用户已批准
    Committed,       // 已执行
    Failed { reason: String, recoverable: bool },
    Reverted,        // 已回滚
}

// === 审计日志 ===

struct AuditLog {
    events: Vec<AuditEvent>,
}

struct AuditEvent {
    timestamp: DateTime<Utc>,
    action_type: String,
    target: String,
    actor: Actor,         // User | Agent { provider, model }
    plan_hash: Option<Sha256>,
    result: ActionResult,
    rollback_available: bool,
}
```

### NeoTrix 映射
| NeoTrix 组件 | 对应模式 | 当前状态 |
|-------------|---------|---------|
| `nt_shield` (影卫) | 安全边界/权限控制 | ✅ 核心职责 |
| Tauri v2 capabilities | 命令级权限 | ✅ 已有 |
| `nt_meta` (元吸收者) | 审计日志/操作追踪 | 需要实现 |
| `nt_feel` (情感中枢) | HITL 确认流程 | 需要实现 |
| R-P1 `#![forbid(unsafe_code)]` | Rust 内存安全 | ✅ 已有 |

---

## Pattern 4: Three-Process Architecture（三进程架构）

### 核心思想
桌面应用拆分为三个独立进程：UI (WebView)、Backend (Rust/Go)、Inference (Sidecar)。每个进程可独立崩溃和恢复。

### 开源项目证据
| 项目 | 进程模型 | 后端语言 |
|------|---------|---------|
| **Tinybot** | React → Tauri Rust → Agent Runtime | Rust + TS |
| **Horizon AI** | React → Tauri Rust → Python Dispatcher | Rust + Python |
| **Pern** | React → Tauri Rust → llama-server | Rust + C++ |
| **OpenFlux** | TypeScript → Tauri Rust → Node.js Gateway | Rust + Node |
| **NOVA** | React → Tauri Rust → Python Core | Rust + Python |
| **xiaodazi** | Vue → Tauri Rust → FastAPI (Python) | Rust + Python |
| **local-ai-workspace** | React → Tauri Rust → FastAPI + Ollama | Rust + Python |
| **Knovex** | React → Electron → FastAPI (Python) | Electron + Python |

### 架构伪代码

```rust
// === 三进程架构 ===

// 进程 1: WebView (不可信 — 用户输入)
// React/Vue/Svelte → Tauri invoke()

// 进程 2: Rust Core (可信 — 所有 OS 操作)
struct TauriApp {
    // 子进程管理
    inference_process: Option<SidecarProcess>,
    python_process: Option<SidecarProcess>,  // 可选

    // 状态
    model_manager: ModelManager,
    audit_log: AuditLog,
    capability_registry: CapabilityRegistry,
}

impl TauriApp {
    fn setup(&self, app: &mut App) {
        // 启动时：自动检测并启动推理引擎
        app.setup(|app| {
            let state = app.state::<AppState>();

            // 自动启动 Ollama (如果安装了)
            if let Ok(ollama) = find_ollama_binary() {
                spawn_ollama_serve(&state);
            }

            // 自动启动 ComfyUI (如果找到了)
            if let Ok(comfyui) = find_comfyui() {
                spawn_comfyui(&state);
            }

            Ok(())
        });
    }
}

// === CORS 代理 — 解决 WebView 限制 ===

/// Tauri production mode: WebView origin 是 tauri://localhost
/// 无法直接调用 localhost:11434 (CORS 限制)
/// 解决方案：所有 localhost 请求通过 Rust 代理
#[tauri::command]
async fn proxy_localhost(
    url: String,
    method: String,
    body: Option<String>,
) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    let req = match method.as_str() {
        "POST" => client.post(&url).json(&body.unwrap_or_default()),
        _ => client.get(&url),
    };
    let resp = req.send().await?.text().await?;
    Ok(resp)
}

// === 流式响应 — SSE 代理 ===

#[tauri::command]
async fn proxy_localhost_stream(
    app: AppHandle,
    url: String,
    body: String,
) -> Result<(), AppError> {
    let client = reqwest::Client::new();
    let mut resp = client.post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send().await?;

    while let Some(chunk) = resp.chunk().await? {
        // 通过 Tauri 事件系统推送到 WebView
        app.emit("stream-chunk", &chunk)?;
    }
    app.emit("stream-end", ())?;
    Ok(())
}

// === 跨平台进程管理 ===

struct ManagedProcess {
    name: String,
    handle: Option<Child>,
    port: Option<u16>,
    stdout_handle: Option<JoinHandle<()>>,
    stderr_handle: Option<JoinHandle<()>>,
}

impl ManagedProcess {
    async fn start(&mut self, cmd: Command) -> Result<()> {
        self.handle = Some(cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?);

        // 关键：drain stdout/stderr 防止 buffer deadlock
        // Locally Uncensored 的教训：不 drain 会导致随机冻结
        let stdout = self.handle.as_mut().unwrap().stdout.take().unwrap();
        self.stdout_handle = Some(tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                line.clear();
            }
        }));
        Ok(())
    }

    async fn ensure_running(&mut self) -> Result<()> {
        if self.is_running() { return Ok(()); }
        self.start(self.build_command()).await?;
        self.wait_until_ready(Duration::from_secs(30)).await
    }
}
```

### NeoTrix 映射
| NeoTrix 组件 | 对应模式 | 当前状态 |
|-------------|---------|---------|
| `src-tauri/` | Tauri v2 Shell | ✅ 已有 |
| `neotrix-core/src/` | Rust Core 逻辑 | ✅ 已有 |
| `nt_io` | WebView IPC 适配 | 需要实现 |
| `nt_physical` | 进程生命周期管理 | 需要实现 |
| Tauri v2 events | 流式响应 | 需要实现 |

---

## Pattern 5: Local-First Knowledge Layer（本地优先知识层）

### 核心思想
所有用户数据（对话历史、记忆、文档索引、模型配置）存储在本地 SQLite/文件系统中。可选的云同步是后来添加的，不是核心依赖。

### 开源项目证据
| 项目 | 存储方案 | 向量搜索 | 离线能力 |
|------|---------|---------|---------|
| **GPT4All** | SQLite (LocalDocs) | BM25 + vector hybrid | ✅ 完全离线 |
| **Open WebUI** | SQLite/PostgreSQL | ChromaDB/Milvus | ✅ 可选 |
| **xiaodazi** | SQLite + FTS5 + sqlite-vec | Mem0 + FTS5 | ✅ 完全离线 |
| **Ollama** | SQLite + content-addressable blobs | N/A | ✅ 完全离线 |
| **LM Studio** | 文件系统 + YAML config | N/A | ✅ 完全离线 |
| **Knovex** | SQLite WAL + ChromaDB | ChromaDB | ✅ 完全离线 |
| **Wolffish** | Markdown 文件 + SQLite FTS5 | N/A | ✅ 完全离线 |
| **Grasberg** | SQLite (encrypted) | N/A | ✅ 完全离线 |

### 架构伪代码

```rust
// === 本地知识层 ===

/// NeoTrix 已有 SQLite KB，这里扩展为完整的本地知识架构
struct LocalKnowledgeLayer {
    // 持久化存储
    chat_db: SqlitePool,        // 对话历史 (WAL mode)
    memory_db: SqlitePool,      // 长期记忆
    vector_store: VectorStore,  // 语义搜索
    blob_store: BlobStore,      // 模型文件/附件

    // 索引
    bm25_index: Fts5Index,      // 关键词搜索
    embedding_cache: EmbeddingCache,  // 本地嵌入缓存
}

// === 5 层查询解析 (Knovex 模式) ===

enum QueryLayer {
    L0,  // 预计算摘要/洞察 — 0 tokens, <1ms
    L1,  // BM25 + cosine → 定位到 §section — 0 tokens, <20ms
    L2,  // Section-scoped LLM — ~300 tokens, 1-3s
    L3,  // Multi-section synthesis — ~900 tokens, 2-5s
    L4,  // Full-document fallback — ~4000+ tokens, 5-15s
}

impl LocalKnowledgeLayer {
    async fn query(&self, question: &str) -> QueryResult {
        // L0: 查预计算摘要
        if let Some(answer) = self.get_precomputed_summary(question).await {
            return QueryResult::from_layer(answer, QueryLayer::L0);
        }

        // L1: BM25 + 向量混合搜索
        let sections = self.hybrid_search(question, 5).await;
        if sections.high_confidence() {
            return QueryResult::from_layer(sections.best_answer(), QueryLayer::L1);
        }

        // L2-L4: 逐步升级到 LLM 调用
        match self.escalate_to_llm(question, &sections).await {
            Ok(answer) => answer,
            Err(_) => QueryResult::no_answer(),
        }
    }

    // Knovex 的关键洞察：70% 的问题在 L0+L1 就能回答，0 token 成本
    fn hybrid_search(&self, query: &str, top_k: usize) -> Vec<SearchResult> {
        let bm25_results = self.bm25_index.search(query, top_k * 2);
        let vector_results = self.vector_store.cosine_search(query, top_k * 2);
        // RRF (Reciprocal Rank Fusion) 合并
        reciprocal_rank_fusion(bm25_results, vector_results, top_k)
    }
}

// === 长期记忆系统 (Open WebUI 模式) ===

struct MemorySystem {
    memory_table: SqliteTable,  // facts table
    extraction_pipeline: ExtractionPipeline,
}

impl MemorySystem {
    /// 从对话中自动提取事实
    fn extract_facts(&self, conversation: &[Message]) -> Vec<MemoryFact> {
        // 使用 LLM 从对话中提取可记忆的事实
        // 存储到 memory_table
        // 查询时注入到 context (~500 tokens budget)
    }
}

// === 模型管理本地存储 (Ollama 模式) ===

struct LocalModelStore {
    base_path: PathBuf,  // ~/.neotrix/models/
}

impl LocalModelStore {
    /// Content-addressable blob 存储
    fn store_blob(&self, data: &[u8]) -> Sha256 {
        let hash = Sha256::digest(data);
        let path = self.base_path.join("blobs").join(format!("sha256-{}", hex(&hash)));
        fs::write(&path, data).unwrap();
        hash
    }

    /// Manifest 描述模型组成
    fn create_manifest(&self, model: &Model) -> Manifest {
        Manifest {
            name: model.name.clone(),
            layers: vec![
                Layer { media_type: "application/vnd.ollama.image.model", digest: self.store_blob(&model.weights) },
                Layer { media_type: "application/vnd.ollama.image.template", digest: self.store_blob(model.template.as_bytes()) },
                Layer { media_type: "application/vnd.ollama.image.params", digest: self.store_blob(&model.params_json()) },
            ],
        }
    }
}

// === 隐私保护 (Grasberg 模式) ===

struct PrivacyGuard {
    egress_filter: EgressFilter,
    key_store: OsKeychain,  // DPAPI (Windows) / Keychain (macOS) / libsecret (Linux)
}

impl PrivacyGuard {
    fn api_key_encrypt(&self, key: &str) -> EncryptedKey {
        // 不用应用级加密，用 OS 原生密钥库
        self.key_store.encrypt(key)
    }

    fn filter_egress(&self, request: &OutgoingRequest) -> Result<()> {
        // 阻止内部路径/密钥泄露到外部 API
        // Egress Privacy Guard (CONTEXT.md 已定义)
    }
}
```

### NeoTrix 映射
| NeoTrix 组件 | 对应模式 | 当前状态 |
|-------------|---------|---------|
| `nt_memory` (知识守护者) | SQLite KB + FTS5 | ✅ 已有 |
| KB embedding | 向量存储 | ✅ 已有 |
| `experience-tree` | 长期记忆提取 | ✅ 已有 |
| `kv_store` | 配置/模型存储 | ✅ 已有 |
| `nt_shield` | 隐私保护/Egress Guard | ✅ 已有 |
| `nt_core_self` | 自模型/价值函数 | ✅ 已有 |

---

## Cross-Cutting Patterns

### A. Streaming Architecture（流式架构）

所有成功的项目都使用 SSE (Server-Sent Events) 进行 token 流式传输：

```rust
// 统一的流式响应模式
enum StreamEvent {
    Token(String),           // 单个 token
    ToolCall(ToolCall),      // 工具调用
    Thinking(String),        // 推理过程 (Claude/DeepSeek)
    Usage(TokenUsage),       // token 统计
    Done,                    // 流结束
    Error(String),           // 错误
}
```

### B. Process-Per-Model（每模型一个进程）

Ollama 的核心设计：每个加载的模型运行在独立子进程中，通过引用计数管理生命周期。一个模型崩溃不影响其他模型。

### C. Progressive Enhancement（渐进增强）

```rust
// Knovex 的 5 层查询：先免费后付费
// Open WebUI 的 Pipes：先 LLM 后 fallback
// GPT4All 的 GPU 检测：先 CPU 后 GPU
fn progressive_execute(task: &Task) -> Result<Output> {
    // 1. 尝试本地小模型
    // 2. 失败则尝试本地大模型
    // 3. 失败则尝试云端 API
    // 4. 全部失败则返回错误 + 建议
}
```

### D. GGUF as Universal Format（GGUF 作为通用格式）

几乎所有项目都标准化 GGUF 格式：
- Ollama, LM Studio, Jan, GPT4All, KathaGPT, Pern 全部使用 GGUF
- 单文件格式，包含权重+tokenizer+元数据
- Q4_K_M 是最常用的量化级别（50% 体积缩减，质量损失极小）
- 可移植性：模型文件在工具间通用

---

## 与 NeoTrix 桌面架构的综合映射

### 当前 NeoTrix 架构
```
nt_core (E8 + GWT)  →  认知层 (L5)
nt_mind (SEAL)      →  进化层 (L5)
nt_memory (KB)      →  知识层 (L1)
nt_world (感知)     →  感知层 (L2)
nt_act (行动)       →  行动层 (L1)
nt_io (界面)        →  行动层 (L1)
nt_shield (安全)    →  具身层 (L3)
nt_physical (具身)  →  具身层 (L3)
nt_feel (情感)      →  情感层 (L4)
```

### 推荐的桌面架构增强

```
┌─────────────────────────────────────────────────┐
│  WebView (React/Vue — 不可信)                    │
│  ├── Chat UI (流式 token 渲染)                   │
│  ├── Model Browser (HuggingFace 搜索)            │
│  ├── Download Manager (进度推送)                  │
│  └── Settings (provider 配置)                    │
├─────────────────────────────────────────────────┤
│  Tauri v2 Rust Core (可信 — 所有 OS 操作)         │
│  ├── Capability Commands (权限受限)               │
│  ├── Model Router (统一抽象)                     │
│  ├── Local Knowledge Layer (SQLite + FTS5)       │
│  ├── Audit Log (操作审计)                        │
│  ├── Privacy Guard (Egress Filter)              │
│  └── Sidecar Manager (进程生命周期)              │
├─────────────────────────────────────────────────┤
│  Inference Sidecar (独立进程)                     │
│  ├── Ollama (默认本地推理)                       │
│  ├── llama.cpp (可选直接集成)                    │
│  ├── MLX (Apple Silicon 优化)                   │
│  └── Cloud APIs (OpenAI/Anthropic/Gemini...)     │
└─────────────────────────────────────────────────┘
```

### 实施优先级

| 优先级 | Pattern | NeoTrix 收益 |
|--------|---------|-------------|
| P0 | Sidecar Inference Engine | 解锁本地推理能力 |
| P0 | Typed Provider Abstraction | 统一多模型路由 |
| P1 | Secure IPC Boundary | 安全性基础 |
| P1 | Three-Process Architecture | 可靠性基础 |
| P2 | Local-First Knowledge Layer | 与现有 KB 整合 |

---

## 参考项目完整索引

| 项目 | 技术栈 | 关键创新 |
|------|-------|---------|
| **Ollama** | Go + React + WebView | Process-per-model, Content-addressable storage |
| **LM Studio** | Electron + llama.cpp | llmster headless daemon, Stateful REST API, Parallel batching |
| **Jan.ai** | Tauri + React + Rust | Extension system, Dual inference backend (llama.cpp + MLX) |
| **Open WebUI** | SvelteKit + FastAPI | Pipes (Python as model), Mixture of Agents merging |
| **GPT4All** | Qt/QML + C++ | Vulkan cross-vendor GPU, LocalDocs hybrid RAG |
| **Tinybot** | Tauri + React | Rollout persistence, Thread projections |
| **KathaGPT** | Tauri + Rust + React | Just-in-Time model download, Unified stream logic |
| **NOVA** | Tauri + Rust + Python | Named pipe IPC, Plan-hash file actions, DPAPI secrets |
| **xiaodazi** | Tauri + Vue + FastAPI | RVR-B executor, 150+ skills, 3-layer memory |
| **Grasberg** | Electron + React | 120+ providers, Sandboxed browser, OS keychain |
| **Egregor** | Electron + Node | Multi-AI Consilium, Anti-groupthink |
| **Wolffish** | Electron + Node | 15 brain modules, Markdown-as-truth |
| **OpenFlux** | Tauri + TypeScript | Gateway sidecar, Multi-agent |
| **local-ai-workspace** | Tauri + React + FastAPI | ChromaDB + SQLite, Guided onboarding |
| **Knovex** | Electron + React + FastAPI | 5-layer query resolution, 70% zero-cost |
| **Accomplish** | Electron + TypeScript | Provider-agnostic adapter, Action approval |
| **Orkas** | Tauri + React | Commander + specialists, Per-agent self-evolution |
| **Pern** | Tauri + React | Deep integrations (WhatsApp, Discord, Email) |
| **Neural Junkie** | Tauri + React | Bundled Ollama, Multi-agent teams |
| **LLM Space** | Electrobun + React | Runtime abstraction, Local/Remote routing |
| **Chameleon** | Next.js + LM Studio | Model comparison, AI debate mode |
| **Kaiden** | Electron + Svelte | Podman Desktop extension architecture |
| **Horizon AI** | Tauri + React + Python | AirLLM VRAM optimization |
| **Biyan** | Native (iOS/macOS) | Multi-model workspace, Cross-device |

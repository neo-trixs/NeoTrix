//! Llama.cpp 进程管理器 — Rust 原生全自治本地推理
//!
//! 职责: 硬件探测 → 参数自适应 → 自动启动/OOM回退/健康看门狗/多模型管理。
//! 不依赖 Ollama, NeoTrix 完全自主控制本地推理。

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::sync::Arc;
use tokio::sync::Mutex;

// ═══════════════════════════════════════════════════════════
// 硬件探测
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct HardwareProfile {
    pub total_ram_gb: u32,
    pub cpu_cores: u32,
    pub chip: String,
    pub is_apple_silicon: bool,
}

impl HardwareProfile {
    pub fn detect() -> Self {
        let total_ram_gb = Self::detect_ram_gb();
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(4);
        let (chip, is_apple_silicon) = Self::detect_chip();
        Self { total_ram_gb, cpu_cores, chip, is_apple_silicon }
    }

    fn detect_ram_gb() -> u32 {
        if let Ok(output) = Command::new("sysctl").arg("hw.memsize").output() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some(val) = s.split_whitespace().last() {
                if let Ok(bytes) = val.parse::<u64>() {
                    return (bytes / 1024 / 1024 / 1024) as u32;
                }
            }
        }
        16
    }

    fn detect_chip() -> (String, bool) {
        if let Ok(output) = Command::new("sysctl").args(["-n", "machdep.cpu.brand_string"]).output() {
            let chip = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let is_silicon = chip.contains("Apple");
            return (chip, is_silicon);
        }
        ("Unknown".to_string(), false)
    }
}

// ═══════════════════════════════════════════════════════════
// 多模型管理
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct GgufModel {
    pub path: PathBuf,
    pub name: String,
    pub size_gb: f64,
    pub param_count: Option<u32>,
}

impl GgufModel {
    pub fn from_path(path: PathBuf) -> Self {
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let size_gb = std::fs::metadata(&path)
            .map(|m| m.len() as f64 / 1024.0 / 1024.0 / 1024.0)
            .unwrap_or(0.0);
        let param_count = Self::infer_params(&name);
        Self { path, name, size_gb, param_count }
    }

    fn infer_params(name: &str) -> Option<u32> {
        for token in name.split(['-', '_', ' ']) {
            if let Some(num) = token.strip_suffix('B').or_else(|| token.strip_suffix('b')) {
                if let Ok(params) = num.parse::<u32>() {
                    return Some(params);
                }
            }
        }
        None
    }
}

/// 本地 GGUF 搜索目录 —— **唯一真源**
///
/// `catalog::model_pool::LocalGgufSource` 也调这里, 避免"启动进程用的目录"
/// 和"列模型用的目录"各写一份而漂移 (改了一处忘了另一处 = 模型池里看不到
/// 正在跑的权重)。
///
/// 顺序即优先级。`NEOTRIX_MODEL_DIRS` 用 `:` 分隔可覆盖/追加。
/// 第一条兜底用 `CARGO_MANIFEST_DIR` 推出 `<repo>/models`, 不写死
/// `/Users/neo/...` —— 换用户名或换机器就失效。
pub fn model_search_dirs() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut dirs: Vec<PathBuf> = Vec::new();

    if let Ok(custom) = std::env::var("NEOTRIX_MODEL_DIRS") {
        for part in custom.split(':') {
            let p = part.trim();
            if !p.is_empty() {
                dirs.push(PathBuf::from(p));
            }
        }
    }

    // <repo>/models — neotrix-core 的 manifest dir 是 <repo>/neotrix-core
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        if let Some(repo) = Path::new(&manifest).parent() {
            dirs.push(repo.join("models"));
        }
    }
    dirs.push(PathBuf::from("models")); // cwd 相对, 兜底

    if !home.is_empty() {
        dirs.push(PathBuf::from(&home).join(".cache/neotrix/models"));
        dirs.push(PathBuf::from(&home).join(".ollama/models"));
        // Ollama blob storage (flat files)
        dirs.push(PathBuf::from(&home).join(".ollama/models/blobs"));
    }

    dirs
}

/// 扫描所有 GGUF 模型, 按大小降序
pub fn scan_models() -> Vec<GgufModel> {
    let search_dirs = model_search_dirs();
    let mut models = Vec::new();
    for dir in &search_dirs {
        if !dir.exists() { continue; }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_dir() {
                    if let Ok(files) = std::fs::read_dir(&p) {
                        for f in files.filter_map(|e| e.ok()) {
                            if f.path().extension().map(|e| e == "gguf").unwrap_or(false) {
                                models.push(GgufModel::from_path(f.path()));
                            }
                        }
                    }
                } else if p.extension().map(|e| e == "gguf").unwrap_or(false) {
                    models.push(GgufModel::from_path(p));
                }
            }
        }
    }
    models.sort_by(|a, b| b.size_gb.partial_cmp(&a.size_gb).unwrap_or(std::cmp::Ordering::Equal));
    models
}

/// 自动选择最优模型
pub fn select_best_model() -> Option<GgufModel> {
    scan_models().into_iter().next()
}

// ═══════════════════════════════════════════════════════════
// 配置
// ═══════════════════════════════════════════════════════════

/// Thinking / reasoning 模式 → `--reasoning on|off`
///
/// ⚠️ 关闭 thinking **只能**用 `--reasoning off`。两个方向相反的陷阱:
///
/// - `--chat-template-kwargs '{"enable_thinking":false}'` 在近期 build 上
///   **被静默忽略** (llama.cpp#20833)。设了以为关掉了, 日志里仍是
///   `chat template, thinking = 1`。
/// - `--reasoning-budget 0` 更糟: 它会在**每个响应开头造出一个 `</think>`**,
///   见 llama.cpp#20516 / #20548。上游明确说 "`--reasoning off` 能用就把
///   `--reasoning-budget 0` 删掉"。
///
/// 判定是否真的关掉了: 启动日志必须出现
/// `init: chat template, thinking = 0`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningMode {
    /// 不传 `--reasoning`, 用模型/模板的默认行为
    Auto,
    On,
    Off,
}

impl ReasoningMode {
    fn as_flag(self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::On => Some("--reasoning"),
            Self::Off => Some("--reasoning"),
        }
    }

    fn as_value(self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::On => Some("on"),
            Self::Off => Some("off"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LlamaServerConfig {
    pub executable: PathBuf,
    pub model_path: PathBuf,
    pub host: String,
    pub port: u16,
    pub ctx_size: u32,
    pub n_gpu_layers: i32,
    pub parallel: u32,
    pub extra_args: Vec<String>,
    /// FlashAttention (-fa 1) — 实测 +7% throughput
    pub flash_attn: bool,
    /// KV cache quantization type (-ctk / -ctv)
    pub kv_cache_type: String,
    /// mmap/mlock mode (--load-mode mlock in llama.cpp 0.3.0)
    pub load_mode: String,
    /// Batch size for prompt processing (-b)
    pub batch_size: u32,
    /// Threads for CPU inference (-t)
    pub threads: u32,
    /// `--jinja` — 用 GGUF 内嵌的 chat template
    ///
    /// ⚠️ **默认必须是 true。** 缺这个 flag 时 llama-server 退回启发式解析器,
    /// 不认 Qwen3.5/3.6 的 `<tool_call><function=…>` XML 和 `<think>` 分隔符,
    /// 于是工具调用以纯文本漏出、思考块变成乱码 —— 这就是"装完开不了话"的第一条
    /// 根因。上游排查文档原话: 在 Qwen3 / DeepSeek-R1 上缺 `--jinja`,
    /// "that's almost certainly the problem"。
    pub jinja: bool,
    /// thinking 模式 (见 [`ReasoningMode`])
    pub reasoning: ReasoningMode,
    /// `--no-prefill-assistant` — 禁用 assistant prefill
    ///
    /// Qwen3.5 系把 assistant prefill 判为与 `enable_thinking` 冲突, 直接返回
    /// `400 "Assistant response prefill is incompatible with enable_thinking."`
    /// (llama.cpp#20861)。凡是会在多轮里回传不完整 assistant 消息的 agent
    /// 前端 (OpenCode / Copilot 等) 都会踩到, 且间歇复现。NeoTrix 自身是 agent
    /// 框架, 默认开启这个绕过。
    pub no_prefill_assistant: bool,
}

impl Default for LlamaServerConfig {
    fn default() -> Self {
        Self {
            executable: PathBuf::from("llama-server"),
            model_path: PathBuf::new(),
            host: "127.0.0.1".to_string(),
            port: 8080,
            ctx_size: 4096,
            n_gpu_layers: 99,
            parallel: 1,
            extra_args: vec![],
            flash_attn: true,     // 实测 +7%
            kv_cache_type: "q4_0".to_string(),  // 实测最优
            load_mode: "mlock".to_string(),     // llama.cpp 0.3.0
            batch_size: 512,      // 实测最优
            threads: 8,           // M5 实测最优
            jinja: true,          // 缺了 = 工具调用/思考块全漏
            reasoning: ReasoningMode::Auto,
            no_prefill_assistant: true,  // agent 循环必需
        }
    }
}

impl LlamaServerConfig {
    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec![
            "-m".to_string(), self.model_path.to_string_lossy().to_string(),
            "--host".to_string(), self.host.clone(),
            "--port".to_string(), self.port.to_string(),
            // ⚠️ 必须显式给: 省略时 llama.cpp 按 GGUF metadata 尝试分配满
            //    原生窗口 (Qwen3.5 = 262144), 在 16GB 机器上直接 OOM。
            "--ctx-size".to_string(), self.ctx_size.to_string(),
            "--n-gpu-layers".to_string(), self.n_gpu_layers.to_string(),
            "--parallel".to_string(), self.parallel.to_string(),
            "--batch-size".to_string(), self.batch_size.to_string(),
            "--threads".to_string(), self.threads.to_string(),
        ];

        // ── `--jinja`: 用 GGUF 内嵌 chat template ──
        // 缺了会退回启发式解析器, 不认 Qwen3.5/3.6 的 tool_call XML 与
        // <think> 分隔符 → 工具调用/思考块以纯文本漏出 (llama.cpp#20861
        // 上游排查结论)。这是"装完开不了话"的第一根因。
        if self.jinja {
            args.push("--jinja".to_string());
        }

        // ── `--reasoning on|off`: 唯一的正确 thinking 开关 ──
        // 刻意**不**实现 `--reasoning-budget`: 那个 flag 会在响应开头造出
        // </think> (llama.cpp#20516/#20548), 而 --chat-template-kwargs
        // 已被静默忽略 (#20833)。见 ReasoningMode 文档。
        if let (Some(flag), Some(val)) = (self.reasoning.as_flag(), self.reasoning.as_value()) {
            args.push(flag.to_string());
            args.push(val.to_string());
        }

        // ── `--no-prefill-assistant`: 绕开 agent 循环的 400 ──
        // "Assistant response prefill is incompatible with enable_thinking."
        // (llama.cpp#20861)
        if self.no_prefill_assistant {
            args.push("--no-prefill-assistant".to_string());
        }

        // FlashAttention — 实测 9.06 vs 8.37 tok/s
        if self.flash_attn {
            args.push("-fa".to_string());
            args.push("1".to_string());
        }

        // KV cache quantization — 实测 q4_0 最优
        if !self.kv_cache_type.is_empty() && self.kv_cache_type != "f16" {
            args.push("-ctk".to_string());
            args.push(self.kv_cache_type.clone());
            args.push("-ctv".to_string());
            args.push(self.kv_cache_type.clone());
        }

        // load-mode mlock (llama.cpp 0.3.0+)
        if !self.load_mode.is_empty() && self.load_mode != "none" {
            args.push("--load-mode".to_string());
            args.push(self.load_mode.clone());
        }

        args.extend(self.extra_args.clone());
        args
    }

    pub fn base_url(&self) -> String {
        format!("http://{}:{}/v1", self.host, self.port)
    }
}

/// 判定是否为 Qwen3.5/3.6/3.8 系混合架构 (Gated DeltaNet + Gated Attention)
///
/// 依据是文件名 —— `compute_optimal_config` 只拿到路径, 不解析 GGUF metadata。
/// 宁可漏判 (退回保守档) 也不误判。
fn is_qwen_hybrid(model_path: &Path) -> bool {
    let name = model_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    // qwen3.5 / qwen35 / qwen3_5 / qwen3.6 / qwen3.8 …
    let has_qwen = name.contains("qwen");
    let has_hybrid_gen = ["qwen3.5", "qwen35", "qwen3_5", "qwen3.6", "qwen36", "qwen3_6", "qwen3.8", "qwen38", "qwen3_8"]
        .iter()
        .any(|g| name.contains(g));
    has_qwen && has_hybrid_gen
}

/// 基于硬件 + 模型计算最优参数
///
/// M5 16GB 实测基准 (Qwen3.5-9B Q5_K_M, 2026-09-01):
/// `batch=512, flash_attn=1, kv=q4_0, threads=8` → 9.06 tok/s gen, 23.39 tok/s prefill。
/// **ctx 不再取那个测速值**, 见下方说明。
pub fn compute_optimal_config(model_path: &Path, hw: &HardwareProfile) -> LlamaServerConfig {
    let model_size_gb = std::fs::metadata(model_path)
        .map(|m| m.len() as f64 / 1024.0 / 1024.0 / 1024.0)
        .unwrap_or(7.0);
    let n_gpu_layers = if hw.is_apple_silicon { 99 } else { 35 };
    let system_reserved_gb = 4.0;
    let _available_gb = (hw.total_ram_gb as f64 - model_size_gb - system_reserved_gb).max(2.0);
    let threads = hw.cpu_cores.saturating_sub(2).max(2);

    // ═══════════════════════════════════════════════════════════
    // ctx 预算 (2026-09-28 修正)
    // ═══════════════════════════════════════════════════════════
    //
    // 旧逻辑按模型文件大小分档给 2048/4096/8192, 那是为了在 4096 上测 tok/s
    // 而定的值。它有两个致命问题:
    //
    // 1. **Qwen 官方要求 thinking 模式至少 128K 上下文** —— "we advise
    //    maintaining a context length of at least 128K tokens to preserve
    //    thinking capabilities" (Qwen3.6-27B 卡)。ctx=4096 下 thinking 已经
    //    废了, 这是"装完开不了话"的第二条根因。吞吐慢几个点换来能思考是划算的。
    //
    // 2. Qwen3.5/3.6/3.8 是 Gated DeltaNet + Gated Attention 混合架构,
    //    `full_attention_interval = 4` → 32 层里只有 8 层是真注意力, 其余 24 层
    //    是**不随上下文增长**的固定状态。按 Qwen3.5-4B 实测超参
    //    (embedding 2560 / kv_heads 4 / head_dim 160) 算:
    //
    //      KV/token = 8层 × 2(K+V) × 4 kv_heads × 160 × dtype
    //        f16   = 20.0 KB → 262144 tok = 5.37 GB
    //        q4_0  =  5.6 KB → 262144 tok = 1.47 GB
    //
    //    对照同尺寸**纯 Transformer** 4B @f16: 64 KB/token → 262144 = 16.8 GB,
    //    单机装不下。所以混合架构让 262K 在 16GB 机器上真正可行。
    //
    // ⚠️ 前提是**显式**给 `--ctx-size` —— 省略时 llama.cpp 会按 metadata 尝试
    //    分配满原生窗口, 在 16GB 上直接 OOM。这正是 `to_args()` 无条件写
    //    `--ctx-size` 的原因。
    //
    // 非混合架构走原来的保守分档 (纯 Transformer 的 KV 随层数线性增长)。
    let hybrid = is_qwen_hybrid(model_path);
    let ctx_size: u32 = if hybrid {
        if model_size_gb > 12.0 {
            65536    // 27B+: 权重大, 但混合架构 KV 仍便宜
        } else if model_size_gb > 5.0 {
            131072   // 9B 级: 满足 thinking 的 128K 底线
        } else {
            262144   // ≤5GB (2B/4B): 原生窗口满配, KV 仅 ~1.5GB @q4_0
        }
    } else if model_size_gb > 12.0 {
        2048   // 70B+ models: minimize context
    } else if model_size_gb > 5.0 {
        4096   // 7-14B models: 实测最优
    } else {
        8192   // Small models: can afford more context
    };

    let batch_size = if hw.is_apple_silicon { 512 } else { 256 };

    log::info!(
        "[llama] auto-config: {} | {}GB RAM | {} cores | model {:.1}GB | ctx {} ({}) | gpu {} | threads {} | fa=1 | kv=q4_0 | jinja=1",
        hw.chip, hw.total_ram_gb, hw.cpu_cores, model_size_gb, ctx_size,
        if hybrid { "hybrid" } else { "dense" },
        n_gpu_layers, threads
    );

    LlamaServerConfig {
        executable: find_executable().unwrap_or_else(|| PathBuf::from("llama-server")),
        model_path: model_path.to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 8080,
        ctx_size,
        n_gpu_layers,
        parallel: 1,
        extra_args: vec![],
        flash_attn: true,          // 实测 +7%
        kv_cache_type: "q4_0".to_string(),  // 实测最优
        load_mode: "mlock".to_string(),     // llama.cpp 0.3.0
        batch_size,
        threads,
        jinja: true,               // 缺了 = 工具调用/思考块全漏
        // 混合架构默认关 thinking: 4B 在 agent 负载下有 H5H5H5 式重复倾向
        // (llama.cpp PR #23802 讨论实测), 且 --reasoning off 是唯一可靠开关。
        // 需要深度思考时把这里改成 ReasoningMode::On。
        reasoning: ReasoningMode::Off,
        no_prefill_assistant: true,  // agent 循环必需
    }
}

// ═══════════════════════════════════════════════════════════
// 进程管理器
// ═══════════════════════════════════════════════════════════

static GLOBAL_MANAGER: OnceLock<Arc<LlamaProcessManager>> = OnceLock::new();

pub fn global_manager() -> &'static Arc<LlamaProcessManager> {
    GLOBAL_MANAGER.get_or_init(|| {
        Arc::new(LlamaProcessManager::new(LlamaServerConfig::default()))
    })
}

pub struct LlamaProcessManager {
    config: Mutex<LlamaServerConfig>,
    child: Arc<Mutex<Option<Child>>>,
}

impl LlamaProcessManager {
    pub fn new(config: LlamaServerConfig) -> Self {
        Self { config: Mutex::new(config), child: Arc::new(Mutex::new(None)) }
    }

    /// 全流程自适应启动: 硬件探测 → 选模型 → 计算参数 → 启动 → OOM回退 → 看门狗
    pub async fn auto_start(&self) -> Result<(), String> {
        let hw = HardwareProfile::detect();
        let model = select_best_model()
            .ok_or_else(|| "No GGUF model found in ~/.cache/neotrix/models/".to_string())?;

        let base_config = compute_optimal_config(&model.path, &hw);
        *self.config.lock().await = base_config.clone();

        if Self::is_port_in_use(base_config.port).await {
            log::info!("[llama] already running on port {}", base_config.port);
            return Ok(());
        }

        // OOM 回退: 从大到小**完整阶梯**尝试 ctx_size。
        //
        // 2026-09-28: base ctx 上限已从 4096 提到 262144 (混合架构下 KV 只要
        // ~1.5GB @q4_0)。旧写法 `[base, base/2, base/4, 16384, 8192, 4096]`
        // 在 base=262144 时会跳过 32768 档, 降级粒度太粗。
        //
        // 每次尝试 2s 间隔; 真实 OOM 时进程会提前退出被 try_wait 抓到, 快速
        // 返回, 不会吃满 60s 超时。
        let mut ctx_fallbacks: Vec<u32> = Vec::new();
        let mut probe = base_config.ctx_size;
        while probe >= 4096 {
            ctx_fallbacks.push(probe);
            probe /= 2;
        }
        let mut last_err = String::new();
        for &ctx in &ctx_fallbacks {
            let mut cfg = base_config.clone();
            cfg.ctx_size = ctx;
            *self.config.lock().await = cfg.clone();

            log::info!("[llama] attempting ctx_size={}", ctx);
            match self.spawn_and_wait(&cfg).await {
                Ok(()) => {
                    self.spawn_watchdog().await;
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("[llama] failed ctx={}: {}", ctx, e);
                    last_err = e;
                    self.kill_child().await;
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
        Err(format!("All ctx attempts failed. Last: {}", last_err))
    }

    async fn spawn_and_wait(&self, config: &LlamaServerConfig) -> Result<(), String> {
        let args = config.to_args();
        log::info!("[llama] {:?} {}", config.executable, args.join(" "));

        let child = Command::new(&config.executable)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn failed: {}", e))?;

        *self.child.lock().await = Some(child);

        for i in 0..60 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if Self::is_port_in_use(config.port).await {
                log::info!("[llama] ready on port {} ({}s, ctx={})", config.port, i + 1, config.ctx_size);
                return Ok(());
            }
            // 检查进程是否提前退出 (try_wait 需要 &mut, MutexGuard 里是 &Child)
            let exited = {
                let mut guard = self.child.lock().await;
                if let Some(ref mut child) = *guard {
                    child.try_wait().ok().flatten().is_some()
                } else {
                    false
                }
            };
            if exited {
                return Err("llama-server exited prematurely".to_string());
            }
        }
        Err(format!("timeout after 60s on port {}", config.port))
    }

    async fn spawn_watchdog(&self) {
        let child_arc = self.child.clone();
        let config_arc = Arc::new(self.config.lock().await.clone());
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let port = config_arc.port;
                if !Self::is_port_in_use(port).await {
                    log::warn!("[llama watchdog] server down, restarting...");
                    {
                        let mut guard = child_arc.lock().await;
                        if let Some(ref mut c) = *guard { let _ = c.kill(); }
                        *guard = None;
                    }
                    let args = config_arc.to_args();
                    if let Ok(new_child) = Command::new(&config_arc.executable)
                        .args(&args)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                    {
                        *child_arc.lock().await = Some(new_child);
                        for _ in 0..30 {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            if Self::is_port_in_use(port).await {
                                log::info!("[llama watchdog] restarted OK");
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    async fn kill_child(&self) {
        let mut guard = self.child.lock().await;
        if let Some(ref mut child) = *guard {
            let _ = child.kill();
            let _ = child.wait();
        }
        *guard = None;
    }

    pub async fn is_port_in_use(port: u16) -> bool {
        let url = format!("http://127.0.0.1:{}/v1/models", port);
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build() {
                Ok(c) => c,
                Err(_) => return false,
            };
        client.get(&url).send().await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    pub async fn stop(&self) {
        self.kill_child().await;
        log::info!("[llama] stopped");
    }

    pub async fn health_check(&self) -> bool {
        let port = self.config.lock().await.port;
        Self::is_port_in_use(port).await
    }

    pub async fn current_config(&self) -> LlamaServerConfig {
        self.config.lock().await.clone()
    }
}

impl Drop for LlamaProcessManager {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.child.try_lock() {
            if let Some(ref mut child) = *guard { let _ = child.kill(); }
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 查找可执行文件
// ═══════════════════════════════════════════════════════════

pub fn find_executable() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("/opt/homebrew/bin/llama-server"),
        PathBuf::from("/usr/local/bin/llama-server"),
        PathBuf::from("~/.local/bin/llama-server"),
        PathBuf::from("llama-server"),
    ];
    for c in &candidates {
        if c.exists() { return Some(c.clone()); }
    }
    let cellar = Path::new("/opt/homebrew/Cellar/llama.cpp");
    if cellar.exists() {
        if let Ok(rs) = std::fs::read_dir(cellar) {
            let mut versions: Vec<_> = rs
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();
            versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
            if let Some(v) = versions.first() {
                let bin = v.path().join("bin").join("llama-server");
                if bin.exists() { return Some(bin); }
            }
        }
    }
    if let Ok(output) = Command::new("which").arg("llama-server").output() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() { return Some(PathBuf::from(path)); }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detect() {
        // HONESTY: This test asserts hardware properties that are machine-dependent.
        // On CI or non-M-series machines, total_ram_gb and cpu_cores will differ.
        // These assertions verify detection works, not specific hardware values.
        let hw = HardwareProfile::detect();
        assert!(hw.total_ram_gb > 0, "RAM detection should return positive value");
        assert!(hw.cpu_cores > 0, "CPU core detection should return positive value");
    }

    #[test]
    fn test_scan_models() {
        // HONESTY: Requires GGUF models in the expected directory.
        // On machines without models, this test will fail.
        // Mock filesystem or use a test fixture directory.
        let models = scan_models();
        if models.is_empty() {
            // No GGUF models found — expected on machines without local models.
            // Wire remote model catalog and assert non-empty from at least one source.
        } else {
            assert!(models.iter().all(|m| m.size_gb > 0.0),
                "all scanned models must have positive size");
        }
    }

    #[test]
    fn test_select_best() {
        // HONESTY: Depends on scan_models() finding local GGUF models.
        // Use a mock or fixture for deterministic testing.
        let m = select_best_model();
        if let Some(model) = m {
            assert!(model.size_gb > 0.0, "selected model must have positive size");
        }
        // None is acceptable if no local models exist.
    }

    #[test]
    fn test_compute_optimal() {
        // HONESTY: Requires a real model file and hardware profile.
        // Use mock hardware profile and model path for deterministic testing.
        let hw = HardwareProfile::detect();
        if let Some(model) = select_best_model() {
            let cfg = compute_optimal_config(&model.path, &hw);
            assert!(cfg.ctx_size >= 2048, "context size should be at least 2048");
            assert!(cfg.parallel >= 1, "parallel should be at least 1");
            assert!(cfg.n_gpu_layers >= 0, "GPU layers should be non-negative");
        }
        // Skip assertions if no model available — test documents behavior.
    }

    /// 回归锁: `--jinja` 必须在参数里。
    ///
    /// 缺它 llama-server 退回启发式解析器, 不认 Qwen3.5 的 tool_call XML 与
    /// `<think>` 分隔符 → "装完开不了话"。曾因漏加导致整条本地链不可用。
    #[test]
    fn test_jinja_flag_always_present() {
        let cfg = LlamaServerConfig::default();
        let args = cfg.to_args();
        assert!(
            args.iter().any(|a| a == "--jinja"),
            "--jinja 必须无条件出现在 llama-server 参数里, 实际: {:?}",
            args
        );
    }

    /// 回归锁: 关 thinking 只能用 `--reasoning off`。
    ///
    /// 两个方向相反的陷阱 (llama.cpp#20833 / #20516 / #20548):
    /// `--chat-template-kwargs '{"enable_thinking":false}'` 被**静默忽略**;
    /// `--reasoning-budget 0` 反而在响应开头造出 `</think>`。
    /// 两者都不许出现在参数里。
    #[test]
    fn test_no_reasoning_budget_and_no_template_kwargs() {
        for mode in [ReasoningMode::Auto, ReasoningMode::On, ReasoningMode::Off] {
            let cfg = LlamaServerConfig { reasoning: mode, ..LlamaServerConfig::default() };
            let args = cfg.to_args();
            assert!(
                !args.iter().any(|a| a == "--reasoning-budget"),
                "不得下发 --reasoning-budget (mode={:?})", mode
            );
            assert!(
                !args.iter().any(|a| a == "--chat-template-kwargs"),
                "不得下发 --chat-template-kwargs (mode={:?})", mode
            );
            match mode {
                ReasoningMode::Auto => {
                    assert!(!args.iter().any(|a| a == "--reasoning"),
                        "Auto 模式不应下发 --reasoning");
                }
                _ => {
                    let i = args.iter().position(|a| a == "--reasoning")
                        .expect("--reasoning 应存在");
                    let want = if mode == ReasoningMode::On { "on" } else { "off" };
                    assert_eq!(args.get(i + 1).map(String::as_str), Some(want),
                        "--reasoning 后的值应为 {:?}", want);
                }
            }
        }
    }

    /// 回归锁: agent 循环需要 `--no-prefill-assistant`。
    ///
    /// 否则 Qwen3.5 系返回 400 "Assistant response prefill is incompatible with
    /// enable_thinking." (llama.cpp#20861), OpenCode / Copilot 类前端必踩。
    #[test]
    fn test_no_prefill_assistant_default_on() {
        let args = LlamaServerConfig::default().to_args();
        assert!(
            args.iter().any(|a| a == "--no-prefill-assistant"),
            "默认必须带 --no-prefill-assistant, 实际: {:?}", args
        );
    }

    /// 混合架构 (Qwen3.5/3.6/3.8) 的 ctx 必须给到 thinking 底线以上。
    ///
    /// Qwen 官方: "advise maintaining a context length of at least 128K tokens to
    /// preserve thinking capabilities"。旧逻辑一律给 4096 → thinking 已废。
    #[test]
    fn test_hybrid_arch_gets_large_ctx() {
        let hw = HardwareProfile { total_ram_gb: 16, cpu_cores: 10, chip: "Apple M5".into(), is_apple_silicon: true };
        // 用不存在的路径: size 回落 7.0GB, 落在 5~12GB 档 (9B 级)
        let fake9b = PathBuf::from("/nonexistent/Qwen3.5-9B-fable-Q4_K_M.gguf");
        let cfg = compute_optimal_config(&fake9b, &hw);
        assert!(cfg.ctx_size >= 131072, "9B 混合架构 ctx 应 ≥131072, 实际 {}", cfg.ctx_size);

        let fake4b = PathBuf::from("/nonexistent/Qwen3.5-4B-Uncensored-Q4_K_M.gguf");
        let cfg4 = compute_optimal_config(&fake4b, &hw);
        // 4B 权重 <5GB 走不到 (路径不存在 → 回落 7GB), 所以只断言不退回 4096
        assert!(cfg4.ctx_size >= 131072, "4B 混合架构 ctx 应 ≥131072, 实际 {}", cfg4.ctx_size);
    }

    /// 非混合架构 (纯 Transformer) 保持保守分档, 不受混合架构放宽影响。
    #[test]
    fn test_dense_arch_keeps_conservative_ctx() {
        let hw = HardwareProfile { total_ram_gb: 16, cpu_cores: 10, chip: "Apple M5".into(), is_apple_silicon: true };
        let dense = PathBuf::from("/nonexistent/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf");
        let cfg = compute_optimal_config(&dense, &hw);
        assert_eq!(cfg.ctx_size, 4096, "纯 Transformer 8B 应仍是 4096, 实际 {}", cfg.ctx_size);
    }

    /// 混合架构识别: 只认 Qwen3.5+ 世代, 不得把 Qwen3 / Llama 误判进来。
    #[test]
    fn test_is_qwen_hybrid_detection() {
        assert!(is_qwen_hybrid(Path::new("/m/Qwen3.5-4B-uncensored.gguf")));
        assert!(is_qwen_hybrid(Path::new("/m/qwen35-9b.gguf")));
        assert!(is_qwen_hybrid(Path::new("/m/Qwen3_6-27B.gguf")));
        assert!(is_qwen_hybrid(Path::new("/m/Qwen3.8-27B-Uncensored.gguf")));
        // 旧世代 / 其他家族 → false (退回保守档)
        assert!(!is_qwen_hybrid(Path::new("/m/Qwen3-4B-Instruct-2507.gguf")));
        assert!(!is_qwen_hybrid(Path::new("/m/Meta-Llama-3.1-8B.gguf")));
        assert!(!is_qwen_hybrid(Path::new("/m/MiniCPM5-2B.gguf")));
        assert!(!is_qwen_hybrid(Path::new("/m/gemma-4-e4b-it.gguf")));
    }
}

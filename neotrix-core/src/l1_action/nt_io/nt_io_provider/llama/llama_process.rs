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
pub(crate) struct GgufModel {
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

/// 扫描所有 GGUF 模型, 按大小降序
pub(crate) fn scan_models() -> Vec<GgufModel> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/neo".into());
    let search_dirs = [
        // 本地项目 models/ (Neo's current model)
        PathBuf::from("/Users/neo/Downloads/neotrix/models"),
        // 标准缓存目录
        PathBuf::from(&home).join(".cache/neotrix/models"),
        PathBuf::from(&home).join(".ollama/models"),
        // Ollama blob storage (flat files)
        PathBuf::from(&home).join(".ollama/models/blobs"),
    ];
    let mut models = Vec::new();
    for dir in &search_dirs {
        if !dir.exists() { continue; }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if let Ok(files) = std::fs::read_dir(&p) {
                        for f in files.flatten() {
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
pub(crate) fn select_best_model() -> Option<GgufModel> {
    scan_models().into_iter().next()
}

// ═══════════════════════════════════════════════════════════
// 配置
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub(crate) struct LlamaServerConfig {
    pub executable: PathBuf,
    pub model_path: PathBuf,
    pub host: String,
    pub port: u16,
    pub ctx_size: u32,
    pub n_gpu_layers: i32,
    pub parallel: u32,
    pub reasoning_budget: i32,
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
}

impl Default for LlamaServerConfig {
    fn default() -> Self {
        Self {
            executable: PathBuf::from("llama-server"),
            model_path: PathBuf::new(),
            host: "127.0.0.1".to_string(),
            port: 8080,
            ctx_size: 4096,       // 实测最优
            n_gpu_layers: 99,
            reasoning_budget: 2048,
            parallel: 1,
            extra_args: vec![],
            flash_attn: true,     // 实测 +7%
            kv_cache_type: "q4_0".to_string(),  // 实测最优
            load_mode: "mlock".to_string(),     // llama.cpp 0.3.0
            batch_size: 512,      // 实测最优
            threads: 8,           // M5 实测最优
        }
    }
}

impl LlamaServerConfig {
    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec![
            "-m".to_string(), self.model_path.to_string_lossy().to_string(),
            "--host".to_string(), self.host.clone(),
            "--port".to_string(), self.port.to_string(),
            "--ctx-size".to_string(), self.ctx_size.to_string(),
            "--n-gpu-layers".to_string(), self.n_gpu_layers.to_string(),
            "--parallel".to_string(), self.parallel.to_string(),
            "--batch-size".to_string(), self.batch_size.to_string(),
            "--threads".to_string(), self.threads.to_string(),
        ];

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

/// 基于硬件 + 模型计算最优参数 — 使用 M5 16GB 实测基准
pub(crate) fn compute_optimal_config(model_path: &Path, hw: &HardwareProfile) -> LlamaServerConfig {
    let model_size_gb = std::fs::metadata(model_path)
        .map(|m| m.len() as f64 / 1024.0 / 1024.0 / 1024.0)
        .unwrap_or(7.0);
    let n_gpu_layers = if hw.is_apple_silicon { 99 } else { 35 };
    let system_reserved_gb = 4.0;
    let _available_gb = (hw.total_ram_gb as f64 - model_size_gb - system_reserved_gb).max(2.0);
    let threads = hw.cpu_cores.saturating_sub(2).max(2);

    // ═══════════════════════════════════════════════════════════
    // 实测最优参数 (M5 16GB, Qwen3.5-9B Q5_K_M, 2026-09-01):
    //   ctx=4096, batch=512, flash_attn=1, kv=q4_0, threads=8
    //   → 9.06 tok/s gen, 23.39 tok/s prefill
    // ═══════════════════════════════════════════════════════════
    
    // Adaptive: larger models get smaller context to fit in memory
    let ctx_size = if model_size_gb > 12.0 {
        2048   // 70B+ models: minimize context
    } else if model_size_gb > 5.0 {
        4096   // 7-14B models: 实测最优
    } else {
        8192   // Small models: can afford more context
    };

    let batch_size = if hw.is_apple_silicon { 512 } else { 256 };

    log::info!(
        "[llama] auto-config: {} | {}GB RAM | {} cores | model {:.1}GB | ctx {} | gpu {} | threads {} | fa=1 | kv=q4_0",
        hw.chip, hw.total_ram_gb, hw.cpu_cores, model_size_gb, ctx_size, n_gpu_layers, threads
    );

    LlamaServerConfig {
        executable: find_executable().unwrap_or_else(|| PathBuf::from("llama-server")),
        model_path: model_path.to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 8080,
        ctx_size,
        n_gpu_layers,
        parallel: 1,
        reasoning_budget: 2048,
        extra_args: vec![],
        flash_attn: true,          // 实测 +7%
        kv_cache_type: "q4_0".to_string(),  // 实测最优
        load_mode: "mlock".to_string(),     // llama.cpp 0.3.0
        batch_size,
        threads,
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

pub(crate) struct LlamaProcessManager {
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

        // OOM 回退: 从大到小尝试 ctx_size
        let ctx_fallbacks = [
            base_config.ctx_size,
            base_config.ctx_size / 2,
            base_config.ctx_size / 4,
            16384,
            8192,
            4096,
        ];
        let mut last_err = String::new();
        for &ctx in &ctx_fallbacks {
            if ctx < 2048 { continue; }
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

pub(crate) fn find_executable() -> Option<PathBuf> {
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
        let hw = HardwareProfile::detect();
        assert!(hw.total_ram_gb > 0);
        assert!(hw.cpu_cores > 0);
    }

    #[test]
    fn test_scan_models() {
        let models = scan_models();
        assert!(!models.is_empty(), "should find at least one GGUF model");
    }

    #[test]
    fn test_select_best() {
        let m = select_best_model();
        assert!(m.is_some());
        assert!(m.unwrap().size_gb > 0.0);
    }

    #[test]
    fn test_compute_optimal() {
        let hw = HardwareProfile::detect();
        let model = select_best_model().unwrap();
        let cfg = compute_optimal_config(&model.path, &hw);
        assert!(cfg.ctx_size >= 2048);
        assert!(cfg.parallel >= 1);
        assert!(cfg.n_gpu_layers > 0);
    }
}

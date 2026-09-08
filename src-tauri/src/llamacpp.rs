use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::OnceCell;

/// Singleton llama.cpp process manager
static LLAMACPP: OnceCell<Mutex<LlamaCppManager>> = OnceCell::const_new();

/// Model info for the frontend
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub quantization: String,
    pub architecture: String,
}

/// Health status of the local inference
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LlamaHealth {
    pub running: bool,
    pub port: u16,
    pub pid: Option<u32>,
    pub uptime_secs: u64,
    pub model_loaded: Option<String>,
    pub memory_usage_mb: Option<u64>,
}

#[derive(Debug)]
pub struct LlamaCppManager {
    child: Option<Child>,
    port: u16,
    started_at: Option<Instant>,
    current_model: Option<String>,
    models_dir: PathBuf,
}

impl LlamaCppManager {
    fn new() -> Self {
        let models_dir = dirs::home_dir()
            .unwrap_or_default()
            .join("Downloads")
            .join("neotrix")
            .join("models");
        Self {
            child: None,
            port: 8080,
            started_at: None,
            current_model: None,
            models_dir,
        }
    }

    /// Find llama-server binary
    fn find_binary() -> Option<PathBuf> {
        let mut candidates: Vec<PathBuf> = vec![
            PathBuf::from("/opt/homebrew/bin/llama-server"),
            PathBuf::from("/usr/local/bin/llama-server"),
        ];
        if let Some(p) = which_llama_server() {
            candidates.push(p);
        }
        for path in candidates {
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// Find default model (first .gguf in models dir)
    fn find_model(&self) -> Option<PathBuf> {
        if !self.models_dir.exists() {
            return None;
        }
        std::fs::read_dir(&self.models_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "gguf")
                    .unwrap_or(false)
            })
            .min_by_key(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
            .map(|e| e.path())
    }

    /// Start llama-server
    fn start_server(&mut self, model_path: &str) -> Result<(), String> {
        if let Some(ref mut child) = self.child {
            if let Ok(None) = child.try_wait() {
                return Err("Server already running".into());
            }
        }

        let binary = Self::find_binary().ok_or("llama-server not found")?;
        let model = PathBuf::from(model_path);
        if !model.exists() {
            return Err(format!("Model not found: {}", model_path));
        }

        let mut child = Command::new(&binary)
            .args([
                "--host",
                "127.0.0.1",
                "--port",
                &self.port.to_string(),
                "--model",
                model_path,
                "--ctx-size",
                "4096",
                "--parallel",
                "2",
                "--reasoning",
                "off",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start llama-server: {e}"))?;

        // Wait a bit for server to start
        std::thread::sleep(Duration::from_secs(2));

        // Check if it's still running
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!("llama-server exited with status: {status}"));
            }
            Ok(None) => {
                self.child = Some(child);
                self.started_at = Some(Instant::now());
                self.current_model = Some(
                    model
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default(),
                );
                Ok(())
            }
            Err(e) => Err(format!("Failed to check llama-server status: {e}")),
        }
    }

    /// Stop the running server
    fn stop_server(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.started_at = None;
        self.current_model = None;
    }

    /// Get health status
    fn health(&self) -> LlamaHealth {
        let running = self.child.is_some();

        LlamaHealth {
            running,
            port: self.port,
            pid: self.child.as_ref().map(|c| c.id()),
            uptime_secs: self
                .started_at
                .map(|t| t.elapsed().as_secs())
                .unwrap_or(0),
            model_loaded: self.current_model.clone(),
            memory_usage_mb: None,
        }
    }

    /// Scan models directory
    fn scan_models(&self) -> Vec<ModelInfo> {
        if !self.models_dir.exists() {
            return vec![];
        }
        std::fs::read_dir(&self.models_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "gguf")
                            .unwrap_or(false)
                    })
                    .filter_map(|e| {
                        let path = e.path();
                        let meta = e.metadata().ok()?;
                        let name = path
                            .file_stem()?
                            .to_string_lossy()
                            .to_string();
                        let ext = path
                            .extension()?
                            .to_string_lossy()
                            .to_string();
                        if ext != "gguf" {
                            return None;
                        }
                        Some(ModelInfo {
                            name: name.clone(),
                            path: path.to_string_lossy().to_string(),
                            size_bytes: meta.len(),
                            quantization: extract_quant_from_name(&name),
                            architecture: "unknown".into(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Health check via HTTP
    async fn http_health_check(&self) -> bool {
        let url = format!("http://127.0.0.1:{}/health", self.port);
        reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

fn which_llama_server() -> Option<PathBuf> {
    std::process::Command::new("which")
        .arg("llama-server")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !path.is_empty() {
                    Some(PathBuf::from(path))
                } else {
                    None
                }
            } else {
                None
            }
        })
}

fn extract_quant_from_name(name: &str) -> String {
    let lower = name.to_lowercase();
    for quant in &[
        "iq4_nl", "iq4_xs", "iq3_xxs", "iq3_xs", "iq3_s", "iq3_xx", "iq2_xxs", "iq2_xs",
        "iq2_s", "iq1", "q8_0", "q6_k", "q5_k_m", "q5_0", "q5_1", "q4_k_m", "q4_0", "q4_1",
        "q4_k_s", "q5_k_s", "q3_k_m", "q3_k_l", "q3_k_s", "q2_k", "q2_k_s", "f16", "f32",
        "bf16",
    ] {
        if lower.contains(quant) {
            return quant.to_uppercase();
        }
    }
    "unknown".into()
}

/// Public API

pub async fn init(app_handle: AppHandle) {
    let mgr = LlamaCppManager::new();
    LLAMACPP
        .set(std::sync::Mutex::new(mgr))
        .expect("LlamaCppManager already initialized");

    // Auto-start in background
    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        auto_start_loop(handle).await;
    });
}

async fn auto_start_loop(app: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Some(mgr_lock) = LLAMACPP.get() {
            let mgr = mgr_lock.lock().unwrap();
            let health = mgr.health();
            drop(mgr);
            let _ = app.emit("llamacpp:health", health);
        }
    }
}

#[tauri::command]
pub async fn llamacpp_start(model_path: Option<String>) -> Result<String, String> {
    let mgr = LLAMACPP
        .get()
        .ok_or("LlamaCppManager not initialized")?;
    let mut mgr = mgr.lock().map_err(|e| e.to_string())?;

    if let Some(path) = model_path {
        mgr.start_server(&path)?;
    } else if let Some(model) = mgr.find_model() {
        mgr.start_server(&model.to_string_lossy())?;
    } else {
        return Err("No model found in models directory".into());
    }

    Ok(serde_json::to_string(&mgr.health()).unwrap_or_default())
}

#[tauri::command]
pub async fn llamacpp_stop() -> Result<String, String> {
    let mgr = LLAMACPP
        .get()
        .ok_or("LlamaCppManager not initialized")?;
    let mut mgr = mgr.lock().map_err(|e| e.to_string())?;
    mgr.stop_server();
    Ok("Server stopped".into())
}

#[tauri::command]
pub async fn llamacpp_health() -> Result<LlamaHealth, String> {
    let mgr = LLAMACPP
        .get()
        .ok_or("LlamaCppManager not initialized")?;
    let mgr = mgr.lock().map_err(|e| e.to_string())?;
    Ok(mgr.health())
}

#[tauri::command]
pub async fn llamacpp_models() -> Result<Vec<ModelInfo>, String> {
    let mgr = LLAMACPP
        .get()
        .ok_or("LlamaCppManager not initialized")?;
    let mgr = mgr.lock().map_err(|e| e.to_string())?;
    Ok(mgr.scan_models())
}

#[tauri::command]
pub async fn llamacpp_swap(model_path: String) -> Result<String, String> {
    let mgr = LLAMACPP
        .get()
        .ok_or("LlamaCppManager not initialized")?;
    let mut mgr = mgr.lock().map_err(|e| e.to_string())?;
    mgr.stop_server();
    mgr.start_server(&model_path)?;
    Ok(serde_json::to_string(&mgr.health()).unwrap_or_default())
}

#[tauri::command]
pub async fn llamacpp_send(
    messages: Vec<serde_json::Value>,
    _model: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
) -> Result<serde_json::Value, String> {
    let port = {
        let mgr = LLAMACPP
            .get()
            .ok_or("LlamaCppManager not initialized")?;
        let mgr = mgr.lock().map_err(|e| e.to_string())?;
        mgr.port
    };

    let body = serde_json::json!({
        "messages": messages,
        "temperature": temperature.unwrap_or(0.7),
        "max_tokens": max_tokens.unwrap_or(2048),
        "stream": false,
    });

    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .timeout(Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| format!("llama.cpp request failed: {e}"))?;

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Parse llama.cpp response: {e}"))?;

    Ok(json)
}

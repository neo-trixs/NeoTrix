use crate::domain::{DomainPlugin, ActionSpec, DomainError, serde_json};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct LlamacppPlugin {
    state: Mutex<LlamacppState>,
}

struct LlamacppState {
    child: Option<Child>,
    port: u16,
    started_at: Option<Instant>,
    current_model: Option<String>,
    models_dir: PathBuf,
}

impl LlamacppPlugin {
    pub fn new() -> Self {
        let models_dir = dirs::home_dir()
            .unwrap_or_default()
            .join("Downloads")
            .join("neotrix")
            .join("models");
        Self {
            state: Mutex::new(LlamacppState {
                child: None,
                port: 8080,
                started_at: None,
                current_model: None,
                models_dir,
            }),
        }
    }

    fn find_binary() -> Option<PathBuf> {
        let candidates = [
            PathBuf::from("/opt/homebrew/bin/llama-server"),
            PathBuf::from("/usr/local/bin/llama-server"),
        ];
        for c in &candidates {
            if c.exists() {
                return Some(c.clone());
            }
        }
        std::process::Command::new("which")
            .arg("llama-server")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !p.is_empty() { Some(PathBuf::from(p)) } else { None }
                } else {
                    None
                }
            })
    }
}

impl DomainPlugin for LlamacppPlugin {
    fn name(&self) -> &str { "llamacpp" }
    fn description(&self) -> &str { "本地推理：llama.cpp 进程管理、模型扫描、健康检查" }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "health".into(),
                description: "获取 llama.cpp 健康状态".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "models".into(),
                description: "扫描本地模型列表".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "start".into(),
                description: "启动 llama-server".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "stop".into(),
                description: "停止 llama-server".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "swap".into(),
                description: "切换模型".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "send".into(),
                description: "发送推理请求".into(),
                params: vec![],
                returns: "Value".into(),
            },
        ]
    }

    fn call(&self, action: &str, args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        let mut state = self.state.lock().map_err(|e| DomainError::from(e.to_string()))?;

        match action {
            "health" => {
                let running = state.child.is_some();
                Ok(serde_json::json!({
                    "running": running,
                    "port": state.port,
                    "pid": state.child.as_ref().map(|c| c.id()),
                    "uptime_secs": state.started_at.map(|t| t.elapsed().as_secs()).unwrap_or(0),
                    "model_loaded": state.current_model,
                    "binary_found": Self::find_binary().is_some(),
                }))
            }
            "models" => {
                let models = scan_models(&state.models_dir);
                Ok(serde_json::json!({ "models": models }))
            }
            "start" => {
                let model_path = args.get("model_path").and_then(|v| v.as_str());
                if let Some(path) = model_path {
                    start_server(&mut state, path)?;
                } else if let Some(model) = find_model(&state.models_dir) {
                    start_server(&mut state, &model.to_string_lossy())?;
                } else {
                    return Err(DomainError::from("No model found in models directory"));
                }
                Ok(serde_json::json!({ "started": true }))
            }
            "stop" => {
                stop_server(&mut state);
                Ok(serde_json::json!({ "stopped": true }))
            }
            "swap" => {
                let model_path = args.get("model_path").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError::from("model_path required"))?;
                stop_server(&mut state);
                start_server(&mut state, model_path)?;
                Ok(serde_json::json!({ "swapped": true }))
            }
            "send" => {
                let messages = args.get("messages").cloned()
                    .ok_or_else(|| DomainError::from("messages required"))?;
                let temperature = args.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7);
                let max_tokens = args.get("max_tokens").and_then(|v| v.as_u64()).unwrap_or(2048);

                let port = state.port;
                drop(state);

                let body = serde_json::json!({
                    "messages": messages,
                    "temperature": temperature,
                    "max_tokens": max_tokens,
                    "stream": false,
                });

                let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
                let client = reqwest::blocking::Client::new();
                let resp = client.post(&url)
                    .json(&body)
                    .timeout(Duration::from_secs(120))
                    .send()
                    .map_err(|e| DomainError::from(format!("Request failed: {e}")))?;

                let json: serde_json::Value = resp.json()
                    .map_err(|e| DomainError::from(format!("Parse response: {e}")))?;

                Ok(json)
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}

fn start_server(state: &mut LlamacppState, model_path: &str) -> Result<(), DomainError> {
    let binary = LlamacppPlugin::find_binary()
        .ok_or_else(|| DomainError::from("llama-server not found"))?;

    let mut child = Command::new(&binary)
        .args([
            "--host", "127.0.0.1",
            "--port", &state.port.to_string(),
            "--model", model_path,
            "--ctx-size", "4096",
            "--parallel", "2",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| DomainError::from(format!("Failed to start: {e}")))?;

    std::thread::sleep(Duration::from_secs(2));

    match child.try_wait() {
        Ok(Some(status)) => Err(DomainError::from(format!("Exited: {status}"))),
        Ok(None) => {
            state.child = Some(child);
            state.started_at = Some(Instant::now());
            state.current_model = Some(
                PathBuf::from(model_path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default(),
            );
            Ok(())
        }
        Err(e) => Err(DomainError::from(format!("Status check failed: {e}"))),
    }
}

fn stop_server(state: &mut LlamacppState) {
    if let Some(mut child) = state.child.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    state.started_at = None;
    state.current_model = None;
}

fn scan_models(dir: &PathBuf) -> Vec<serde_json::Value> {
    if !dir.exists() {
        return vec![];
    }
    std::fs::read_dir(dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "gguf").unwrap_or(false))
                .filter_map(|e| {
                    let path = e.path();
                    let meta = e.metadata().ok()?;
                    let name = path.file_stem()?.to_string_lossy().to_string();
                    Some(serde_json::json!({
                        "name": name,
                        "path": path.to_string_lossy(),
                        "size_bytes": meta.len(),
                        "quantization": extract_quant(&name),
                    }))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn find_model(dir: &PathBuf) -> Option<PathBuf> {
    if !dir.exists() { return None; }
    std::fs::read_dir(dir).ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "gguf").unwrap_or(false))
        .min_by_key(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .map(|e| e.path())
}

fn extract_quant(name: &str) -> String {
    let lower = name.to_lowercase();
    for q in &["iq4_nl", "q8_0", "q6_k", "q5_k_m", "q4_k_m", "q4_0", "f16", "f32", "bf16"] {
        if lower.contains(q) { return q.to_uppercase(); }
    }
    "unknown".into()
}

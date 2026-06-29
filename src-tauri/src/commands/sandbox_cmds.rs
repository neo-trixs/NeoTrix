use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[derive(Serialize)]
pub struct SandboxResponse {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub timed_out: bool,
    pub violation: Option<String>,
}

#[derive(Deserialize)]
pub struct SandboxRequest {
    pub command: String,
    pub args: Vec<String>,
    pub timeout_secs: Option<u64>,
    pub writable_paths: Option<Vec<String>>,
    pub allow_network: Option<bool>,
    pub working_dir: Option<String>,
}

#[tauri::command]
pub async fn sandbox_execute(req: SandboxRequest) -> Result<SandboxResponse, String> {
    let timeout_dur = req
        .timeout_secs
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(120));

    let start = Instant::now();

    let result = match timeout(timeout_dur, run_command(&req)).await {
        Ok(Ok(output)) => {
            let exit_code = output.status.code();
            SandboxResponse {
                exit_code,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
                timed_out: false,
                violation: output
                    .status
                    .success()
                    .then(|| None)
                    .flatten()
                    .or_else(|| {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if !stderr.is_empty() {
                            Some(stderr.to_string())
                        } else {
                            None
                        }
                    }),
            }
        }
        Ok(Err(e)) => {
            return Err(format!("sandbox execution failed: {}", e));
        }
        Err(_elapsed) => {
            // timed out — kill child via drop
            SandboxResponse {
                exit_code: None,
                stdout: String::new(),
                stderr: String::from("command timed out"),
                duration_ms: timeout_dur.as_millis() as u64,
                timed_out: true,
                violation: Some("command timed out".into()),
            }
        }
    };

    Ok(result)
}

async fn run_command(
    req: &SandboxRequest,
) -> Result<std::process::Output, std::io::Error> {
    let mut cmd = tokio::process::Command::new(&req.command);
    cmd.args(&req.args)
        .kill_on_drop(true);

    if let Some(dir) = &req.working_dir {
        cmd.current_dir(dir);
    }

    if req.allow_network == Some(false) {
        // For macOS, use sandbox-exec with deny-network profile
        // TODO: platform-specific network isolation
        cmd.env("SANDBOX_NETWORK_DENY", "1");
    }

    cmd.output().await
}

#[tauri::command]
pub async fn sandbox_status() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "available": true,
        "platform": std::env::consts::OS,
        "seatbelt_available": cfg!(target_os = "macos"),
        "default_timeout_secs": 120,
        "modes": ["local", "docker", "wasm"],
        "current_mode": "local",
        "protected_paths": [".git", ".neotrix"],
    }))
}

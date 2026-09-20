use crate::ipc::{self, IpcResponse};
use crate::util::cli_finder::find_neotrix_binary;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Serialize, Deserialize)]
pub struct CliOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[tauri::command]
pub async fn run_cli(args: Vec<String>) -> IpcResponse<CliOutput> {
    let bin = match find_neotrix_binary().or_else(|| {
        dirs::home_dir().map(|h| h.join(".cargo/bin/neotrix").to_string_lossy().to_string())
    }) {
        Some(b) => b,
        None => {
            return ipc::err(
                "CLI_NOT_FOUND",
                "neotrix binary not found. Install with: cargo install neotrix",
            )
        }
    };

    let output = match tokio::task::spawn_blocking(move || {
        Command::new(&bin)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to execute neotrix: {e}"))
    })
    .await
    {
        Ok(result) => match result {
            Ok(o) => o,
            Err(e) => return ipc::err("CLI_EXEC_FAILED", &e),
        },
        Err(e) => return ipc::err("TASK_JOIN_FAILED", &format!("Task join error: {e}")),
    };

    ipc::ok(CliOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[tauri::command]
pub async fn get_version() -> IpcResponse<String> {
    let output = run_cli(vec!["--version".into()]).await;
    match output.data {
        Some(d) if d.success => ipc::ok(d.stdout.trim().to_string()),
        Some(d) => ipc::err("CLI_VERSION_FAILED", &d.stderr),
        None => match output.error {
            Some(e) => ipc::err(e.code, e.message),
            None => ipc::err("UNKNOWN", "unknown error"),
        },
    }
}

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Serialize, Deserialize)]
pub struct CliOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn find_neotrix_binary() -> Option<String> {
    if let Ok(output) = Command::new("which").arg("neotrix").output() {
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }
    None
}

#[tauri::command]
pub async fn run_cli(args: Vec<String>) -> Result<CliOutput, String> {
    let bin = find_neotrix_binary()
        .or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".cargo/bin/neotrix").to_string_lossy().to_string())
        })
        .ok_or("neotrix binary not found. Install with: cargo install neotrix")?;

    let output = tokio::task::spawn_blocking(move || {
        Command::new(&bin)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to execute neotrix: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
    .map_err(|e| e)?;

    Ok(CliOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[tauri::command]
pub async fn get_version() -> Result<String, String> {
    let output = run_cli(vec!["--version".into()]).await?;
    if output.success {
        Ok(output.stdout.trim().to_string())
    } else {
        Err(output.stderr)
    }
}

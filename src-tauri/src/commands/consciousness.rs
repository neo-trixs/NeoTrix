use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ConsciousnessStatus {
    pub cycle: u64,
    pub phi: f64,
    pub coherence: f64,
    pub gwt_resonance: f64,
    pub mars_dual_process: bool,
    pub governance_compliant: bool,
    pub fog_level: f64,
    pub domains: Vec<DomainHealth>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct DomainHealth {
    pub name: String,
    pub status: String,
    pub constellation: String,
}

#[derive(Serialize, Deserialize)]
pub struct GrowthReport {
    pub phases: Vec<PhaseSummary>,
    pub total_cycles: u64,
}

#[derive(Serialize, Deserialize)]
pub struct PhaseSummary {
    pub name: String,
    pub duration_ms: u64,
    pub items_produced: usize,
}

#[tauri::command]
pub async fn get_status() -> Result<ConsciousnessStatus, String> {
    let output = super::neotrix_cli::run_cli(vec!["consciousness".into(), "status".into()]).await?;
    if output.success {
        serde_json::from_str(&output.stdout)
            .map_err(|e| format!("Failed to parse consciousness status: {e}"))
    } else {
        Ok(ConsciousnessStatus::default())
    }
}

#[tauri::command]
pub async fn tick_growth(cycles: Option<u64>) -> Result<GrowthReport, String> {
    let n = cycles.unwrap_or(1).to_string();
    let output = super::neotrix_cli::run_cli(vec!["consciousness".into(), "tick".into(), n]).await?;
    if output.success {
        serde_json::from_str(&output.stdout)
            .map_err(|e| format!("Failed to parse growth report: {e}"))
    } else {
        Err(output.stderr)
    }
}

#[tauri::command]
pub async fn run_task(instruction: String) -> Result<String, String> {
    let output = super::neotrix_cli::run_cli(vec![
        "consciousness".into(),
        "task".into(),
        instruction,
    ])
    .await?;
    if output.success {
        Ok(output.stdout)
    } else {
        Err(output.stderr)
    }
}

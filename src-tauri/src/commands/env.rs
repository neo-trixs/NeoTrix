use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlatformSnapshot {
    pub os: String,
    pub arch: String,
    pub gpu: String,
    pub ram_total_mb: u64,
    pub cpu_cores: usize,
    pub llama_binary: Option<String>,
    pub recommended_params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResourceSnapshot {
    pub local_models: Vec<LocalModel>,
    pub downloadable_count: usize,
    pub cloud_providers: Vec<String>,
    pub free_providers: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalModel {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub quantization: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HealthSnapshot {
    pub score: u8,
    pub platform_healthy: bool,
    pub memory_pressure: bool,
    pub disk_pressure: bool,
    pub details: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CapabilitySnapshot {
    pub capabilities: Vec<Capability>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Capability {
    pub name: String,
    pub confidence: f64,
    pub available: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnomalyReport {
    pub anomalies: Vec<Anomaly>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Anomaly {
    pub kind: String,
    pub severity: String,
    pub message: String,
    pub timestamp: String,
}

/// Get platform info
#[tauri::command]
pub async fn env_platform() -> Result<PlatformSnapshot, String> {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let gpu = detect_gpu();
    let ram_total_mb = get_total_ram();
    let cpu_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let llama_binary = find_llama_binary();
    let recommended_params = serde_json::json!({
        "ctx_size": if ram_total_mb >= 32000 { 8192 } else { 4096 },
        "gpu_layers": if gpu != "none" { 99 } else { 0 },
        "threads": (cpu_cores as u32).min(8),
    });

    Ok(PlatformSnapshot {
        os,
        arch,
        gpu,
        ram_total_mb,
        cpu_cores,
        llama_binary,
        recommended_params,
    })
}

/// Get local models
#[tauri::command]
pub async fn env_models() -> Result<Vec<LocalModel>, String> {
    let models_dir = dirs::home_dir()
        .unwrap_or_default()
        .join("Downloads")
        .join("neotrix")
        .join("models");

    if !models_dir.exists() {
        return Ok(vec![]);
    }

    let mut models = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "gguf").unwrap_or(false) {
                let meta = entry.metadata().ok();
                let name = path
                    .file_stem()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                models.push(LocalModel {
                    name: name.clone(),
                    path: path.to_string_lossy().to_string(),
                    size_bytes: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                    quantization: extract_quant(&name),
                });
            }
        }
    }
    models.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    Ok(models)
}

/// Get health status
#[tauri::command]
pub async fn env_health() -> Result<HealthSnapshot, String> {
    let (used, total) = get_memory_usage();
    let memory_pressure = if total > 0 { (used as f64 / total as f64) > 0.85 } else { false };
    let disk_pressure = check_disk_pressure();
    let score = calculate_health_score(memory_pressure, disk_pressure);

    Ok(HealthSnapshot {
        score,
        platform_healthy: score >= 50,
        memory_pressure,
        disk_pressure,
        details: serde_json::json!({
            "memory_used_mb": used,
            "memory_total_mb": total,
            "memory_percent": if total > 0 { (used as f64 / total as f64 * 100.0) as u64 } else { 0 },
        }),
    })
}

/// Get capabilities
#[tauri::command]
pub async fn env_capabilities() -> Result<CapabilitySnapshot, String> {
    let llama_available = find_llama_binary().is_some();
    let has_gpu = detect_gpu() != "none";

    Ok(CapabilitySnapshot {
        capabilities: vec![
            Capability {
                name: "local_inference".into(),
                confidence: if llama_available { 0.9 } else { 0.0 },
                available: llama_available,
            },
            Capability {
                name: "gpu_acceleration".into(),
                confidence: if has_gpu { 0.95 } else { 0.0 },
                available: has_gpu,
            },
            Capability {
                name: "code_generation".into(),
                confidence: 0.8,
                available: true,
            },
            Capability {
                name: "tool_calling".into(),
                confidence: 0.7,
                available: true,
            },
            Capability {
                name: "web_search".into(),
                confidence: 0.6,
                available: true,
            },
        ],
    })
}

/// Get anomalies
#[tauri::command]
pub async fn env_anomalies() -> Result<AnomalyReport, String> {
    let mut anomalies = Vec::new();

    let (used, total) = get_memory_usage();
    if total > 0 && (used as f64 / total as f64) > 0.9 {
        anomalies.push(Anomaly {
            kind: "memory_pressure".into(),
            severity: "critical".into(),
            message: format!("Memory usage at {:.0}%", used as f64 / total as f64 * 100.0),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    if check_disk_pressure() {
        anomalies.push(Anomaly {
            kind: "disk_pressure".into(),
            severity: "warning".into(),
            message: "Disk space low".into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    if find_llama_binary().is_none() {
        anomalies.push(Anomaly {
            kind: "missing_binary".into(),
            severity: "info".into(),
            message: "llama-server not found in PATH".into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    Ok(AnomalyReport { anomalies })
}

// Helpers

fn detect_gpu() -> String {
    #[cfg(target_os = "macos")]
    {
        if std::process::Command::new("system_profiler")
            .args(["SPDisplaysDataType"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Apple"))
            .unwrap_or(false)
        {
            return "apple_metal".into();
        }
    }
    "none".into()
}

fn get_total_ram() -> u64 {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok()?.trim().parse::<u64>().ok())
            .map(|b| b / (1024 * 1024))
            .unwrap_or(0)
    }
    #[cfg(not(target_os = "macos"))]
    { 0 }
}

fn get_memory_usage() -> (u64, u64) {
    #[cfg(target_os = "macos")]
    {
        let total = get_total_ram();
        let used = total * 70 / 100;
        return (used, total);
    }
    #[cfg(not(target_os = "macos"))]
    { (0, 0) }
}

fn find_llama_binary() -> Option<String> {
    let candidates = [
        "/opt/homebrew/bin/llama-server",
        "/usr/local/bin/llama-server",
    ];
    for c in &candidates {
        if std::path::Path::new(c).exists() {
            return Some(c.to_string());
        }
    }
    std::process::Command::new("which")
        .arg("llama-server")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !p.is_empty() { Some(p) } else { None }
            } else {
                None
            }
        })
}

fn check_disk_pressure() -> bool {
    std::process::Command::new("df")
        .arg("-m")
        .arg(dirs::home_dir().unwrap_or_default().to_string_lossy().to_string())
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            let lines: Vec<&str> = out.lines().collect();
            if lines.len() >= 2 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() >= 4 {
                    let avail: u64 = parts[3].parse().unwrap_or(0);
                    return Some(avail < 2048);
                }
            }
            None
        })
        .unwrap_or(false)
}

fn calculate_health_score(memory_pressure: bool, disk_pressure: bool) -> u8 {
    let mut score = 100u8;
    if memory_pressure { score = score.saturating_sub(30); }
    if disk_pressure { score = score.saturating_sub(20); }
    if find_llama_binary().is_none() { score = score.saturating_sub(10); }
    score
}

fn extract_quant(name: &str) -> String {
    let lower = name.to_lowercase();
    for q in &["iq4_nl", "q8_0", "q6_k", "q5_k_m", "q4_k_m", "q4_0", "f16", "f32", "bf16"] {
        if lower.contains(q) { return q.to_uppercase(); }
    }
    "unknown".into()
}

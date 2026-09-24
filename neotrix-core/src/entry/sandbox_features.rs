//! sandbox_features — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::path::PathBuf;

use super::{err, info, success, tokio_runtime};

pub fn run_sandbox_run(code: Option<&str>, runtime: &str, timeout: u64) {
    use neotrix::l3_embodiment::nt_shield::nt_shield_sandbox::cli;
    let runtime = if runtime.is_empty() {
        None
    } else {
        Some(runtime)
    };
    let rt = tokio_runtime();
    rt.block_on(cli::handle_run(code, runtime, Some(timeout)));
}

pub fn run_sandbox_list() {
    neotrix::l3_embodiment::nt_shield::nt_shield_sandbox::cli::handle_list();
}

pub fn run_sandbox_cancel(session_id: &str) {
    neotrix::l3_embodiment::nt_shield::nt_shield_sandbox::cli::handle_cancel(session_id);
}

pub fn run_discover(port: u16, duration_ms: u64, json: bool) {
    // nt_agent_protocol not yet migrated — AgentDiscovery unavailable
    let _ = (port, duration_ms, json);
    eprintln!("Agent discovery requires nt_agent_protocol (not yet migrated)");
}

pub fn run_sandbox_upload(path: &str, session_id: &str) {
    use neotrix::l3_embodiment::nt_shield::nt_shield_sandbox::cli;
    let rt = tokio_runtime();
    rt.block_on(cli::handle_upload(path, session_id));
}

/// Path to stored feature flags
pub(crate) fn features_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let mut path = PathBuf::from(home);
    path.push(".neotrix");
    std::fs::create_dir_all(&path).ok();
    path.push("features.json");
    path
}

pub(crate) fn load_features() -> std::collections::BTreeSet<String> {
    let path = features_path();
    if !path.exists() {
        return std::collections::BTreeSet::new();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

pub(crate) fn save_features(features: &std::collections::BTreeSet<String>) {
    let path = features_path();
    if let Ok(content) = serde_json::to_string_pretty(features) {
        std::fs::write(path, content).ok();
    }
}

pub fn run_features_enable(name: &str) {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        eprintln!("{}", err("Error: feature name cannot be empty"));
        return;
    }
    let mut features = load_features();
    if features.contains(trimmed) {
        println!("  {} feature '{}' is already enabled", info("ℹ"), trimmed);
        return;
    }
    features.insert(trimmed.to_string());
    save_features(&features);
    println!("  {} feature '{}' enabled", success("✓"), trimmed);
}

pub fn run_features_list() {
    let features = load_features();
    if features.is_empty() {
        println!("  {} No feature flags are currently enabled", info("ℹ"));
        println!();
        println!(
            "  Use {} to enable a feature",
            info("neotrix features enable <name>")
        );
        return;
    }
    println!("  {} Enabled feature flags:", success("✓"));
    for f in &features {
        println!("    • {}", f);
    }
}

// ── Config commands ──

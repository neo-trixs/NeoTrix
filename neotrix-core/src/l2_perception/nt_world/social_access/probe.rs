//! Backend health probing utilities
//!
//! Provides CLI-based health checking for social platform backends.
//! Modeled after Agent-Reach's probe_command pattern.

use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

use super::channel::{Backend, ProbeResult};

/// Probe a command
pub fn probe_command(cmd: &str, args: &[String]) -> ProbeResult {
    let start = Instant::now();

    let child = Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let child = match child {
        Ok(c) => c,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return ProbeResult::missing(format!("{}: command not found", cmd));
            }
            return ProbeResult::error(e.to_string(), start.elapsed().as_millis() as u64);
        }
    };

    // Wait with timeout
    let wait_result = std::thread::spawn(move || {
        child.wait_with_output()
    })
    .join();

    let latency = start.elapsed().as_millis() as u64;

    match wait_result {
        Ok(Ok(output)) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                ProbeResult::ok(stdout.trim().to_string(), latency)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                ProbeResult::error(stderr, latency)
            }
        }
        Ok(Err(e)) => ProbeResult::error(e.to_string(), latency),
        Err(_) => ProbeResult::timeout(),
    }
}

/// Probe all backends for a channel, return ordered results
pub fn probe_backends(
    backends: &[Backend],
    _config: &HashMap<String, String>,
) -> Vec<(String, ProbeResult)> {
    let mut results = Vec::new();

    for backend in backends {
        let result = probe_command(
            &backend.probe_cmd,
            &backend.probe_args,
        );
        results.push((backend.name.clone(), result));
    }

    results
}

/// Find the best available backend from a list
pub fn find_best_backend(backends: &[Backend]) -> Option<&Backend> {
    // Sort by weight (descending) and cost_tier (ascending)
    let mut sorted: Vec<&Backend> = backends.iter().collect();
    sorted.sort_by(|a, b| {
        b.weight.cmp(&a.weight)
            .then(a.cost_tier.cmp(&b.cost_tier))
    });

    for backend in sorted {
        let result = probe_command(
            &backend.probe_cmd,
            &backend.probe_args,
        );

        if result.status.is_healthy() {
            return Some(backend);
        }
    }

    None
}

/// Check if a specific command is available
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get version string for a command
pub fn get_version(cmd: &str, version_arg: &str) -> Option<String> {
    let output = Command::new(cmd)
        .arg(version_arg)
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Some(stdout.trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_command_exists() {
        let result = probe_command("echo", &["hello".into()]);
        assert!(result.status.is_healthy());
        assert_eq!(result.output.as_deref(), Some("hello"));
    }

    #[test]
    fn test_probe_command_missing() {
        let result = probe_command(
            "nonexistent_cmd_xyz_12345",
            &[],
        );
        assert_eq!(result.status, super::super::channel::BackendStatus::Missing);
    }

    #[test]
    fn test_command_exists() {
        assert!(command_exists("echo"));
        assert!(!command_exists("nonexistent_cmd_xyz_12345"));
    }

    #[test]
    fn test_get_version() {
        let version = get_version("echo", "test");
        assert!(version.is_some());
    }
}

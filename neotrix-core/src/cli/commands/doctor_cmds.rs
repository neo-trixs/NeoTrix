use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

pub struct DoctorCmd;

impl CliCommand for DoctorCmd {
    fn name(&self) -> &str {
        "/doctor"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/doctor", "/diag", "/diagnose"]
    }

    fn description(&self) -> &str {
        "Run diagnostic checks on the environment"
    }

    fn execute(
        &self,
        _args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let mut report = String::new();
        let mut all_ok = true;

        report.push_str("## NeoTrix Diagnostics\n\n");

        // 1. Version
        report.push_str(&format!("**Version**: {}\n\n", env!("CARGO_PKG_VERSION")));

        // 2. OS info
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let hostname = Command::new("hostname")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        report.push_str(&format!("**OS**: {} ({}) on {}\n\n", os, arch, hostname));

        // 3. Git status
        let git_root = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output();
        if let Ok(output) = git_root {
            if output.status.success() {
                let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let branch = Command::new("git")
                    .args(["rev-parse", "--abbrev-ref", "HEAD"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let dirty = Command::new("git")
                    .args(["status", "--porcelain"])
                    .output()
                    .ok()
                    .map(|o| o.stdout.len() > 0)
                    .unwrap_or(false);
                let status = if dirty { "⚠️ dirty" } else { "✅ clean" };
                report.push_str(&format!("**Git**: {} ({}) - {}\n\n", root, branch, status));
            } else {
                report.push_str("**Git**: ❌ not a git repository\n\n");
                all_ok = false;
            }
        } else {
            report.push_str("**Git**: ⚠️ git not found\n\n");
        }

        // 4. Config file
        let config_path = dirs::config_dir().map(|p| p.join("neotrix").join("config.toml"));
        if let Some(ref path) = config_path {
            if path.exists() {
                report.push_str(&format!(
                    "**Config**: ✅ {} ({} bytes)\n\n",
                    path.display(),
                    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
                ));
            } else {
                report.push_str(&format!(
                    "**Config**: ❌ not found at {}\n\n",
                    path.display()
                ));
                all_ok = false;
            }
        }

        // 5. Brain state
        let brain_path = dirs::home_dir().map(|p| p.join(".neotrix").join("brain.json"));
        if let Some(ref path) = brain_path {
            if path.exists() {
                report.push_str(&format!(
                    "**Brain**: ✅ {} ({} bytes)\n\n",
                    path.display(),
                    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
                ));
            } else {
                report.push_str(&format!(
                    "**Brain**: ⚠️ not found at {}\n\n",
                    path.display()
                ));
            }
        }

        // 6. Knowledge DB
        let kb_path = dirs::home_dir().map(|p| p.join(".neotrix").join("knowledge.db"));
        if let Some(ref path) = kb_path {
            if path.exists() {
                report.push_str(&format!(
                    "**Knowledge DB**: ✅ {} ({} bytes)\n\n",
                    path.display(),
                    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
                ));
            } else {
                report.push_str("**Knowledge DB**: ⚠️ not initialized\n\n");
            }
        }

        // 7. Provider config
        let provider = std::env::var("NEOTRIX_PROVIDER").unwrap_or_else(|_| "not set".to_string());
        let has_key = std::env::var("NEOTRIX_API_KEY").is_ok()
            || std::env::var("OPENAI_API_KEY").is_ok()
            || std::env::var("ANTHROPIC_API_KEY").is_ok();
        report.push_str(&format!(
            "**Provider**: {} (API key: {})\n\n",
            provider,
            if has_key {
                "✅ configured"
            } else {
                "❌ missing"
            }
        ));

        // 8. Daemon status
        let home = crate::core::nt_core_util::home_dir();
        let health_path = home.join(".neotrix/neotrix_daemon.health");
        let health_file = Path::new(&health_path);
        if health_file.exists() {
            let content = std::fs::read_to_string(health_file).unwrap_or_default();
            report.push_str(&format!("**Daemon**: ✅ running ({})\n\n", content.trim()));
        } else {
            report.push_str("**Daemon**: ⚠️ not running\n\n");
        }

        // 9. Disk space
        let home = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let df = Command::new("df")
            .args(["-h", &home])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok());
        if let Some(df_out) = df {
            let lines: Vec<&str> = df_out.lines().collect();
            if lines.len() > 1 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() >= 4 {
                    report.push_str(&format!(
                        "**Disk**: {} free of {} ({} used)\n\n",
                        parts[3], parts[1], parts[4]
                    ));
                }
            }
        }

        // 10. Terminal capabilities
        report.push_str(&format!(
            "**Terminal**: {} colors, {} unicode\n\n",
            if std::env::var("TERM").unwrap_or_default().contains("256") {
                "256"
            } else {
                "basic"
            },
            if std::env::var("LANG").unwrap_or_default().contains("UTF") {
                "✅"
            } else {
                "⚠️"
            }
        ));

        // 11. Memory usage
        if let Ok(usage) = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
        {
            let rss = String::from_utf8_lossy(&usage.stdout).trim().to_string();
            if let Ok(kb) = rss.parse::<u64>() {
                let mb = kb as f64 / 1024.0;
                report.push_str(&format!("**Memory**: {:.1} MB RSS\n\n", mb));
            }
        }

        report.push_str("---\n");
        if all_ok {
            report.push_str("✅ All checks passed\n");
        } else {
            report.push_str("⚠️ Some checks failed — review details above\n");
        }

        CommandOutput::ok(&report)
    }
}

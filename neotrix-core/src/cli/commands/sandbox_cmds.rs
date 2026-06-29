//! Sandbox commands — Docker / network isolation runtime control
//!
//! Provides `/sandbox` to run arbitrary code snippets in a sandboxed runtime
//! via the `nt_shield_sandbox::CloudSandbox` infrastructure (LocalDockerProvider / NoopProvider).

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::cli::sandbox::{global_sandbox, init_sandbox, CliSandboxMode};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_shield_sandbox::{CloudRuntime, CloudSandbox};

/// `/sandbox` — toggle Docker / read-only / disabled + run code snippets.
pub struct SandboxCmd;

impl CliCommand for SandboxCmd {
    fn name(&self) -> &str {
        "/sandbox"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/sb"]
    }
    fn description(&self) -> &str {
        "Sandbox: /sandbox status | set <mode> | run <runtime> <code> [--json]"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");

        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            return self.status(want_json);
        }

        let sub = args[0].as_str();
        match sub {
            "status" | "stat" => self.status(want_json),
            "set" => {
                if args.len() < 2 {
                    return CommandOutput::err("Usage: /sandbox set <disabled|read-only|docker>");
                }
                let mode = CliSandboxMode::from_str(&args[1]);
                init_sandbox(mode);
                let descr = match mode {
                    CliSandboxMode::Disabled => "commands run unsandboxed",
                    CliSandboxMode::ReadOnly => "mutating commands blocked",
                    CliSandboxMode::Docker => "exec via Docker (network=none, 512m, 1 cpu)",
                };
                let msg = format!(
                    "Sandbox mode -> {} ({}). Use /sandbox run <runtime> <code> to execute.",
                    mode.label(),
                    descr
                );
                if want_json {
                    let label = match mode {
                        CliSandboxMode::Disabled => "disabled",
                        CliSandboxMode::ReadOnly => "read-only",
                        CliSandboxMode::Docker => "docker",
                    };
                    return CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "mode": label,
                    }));
                }
                CommandOutput::ok(&msg)
            }
            "run" => {
                if args.len() < 3 {
                    return CommandOutput::err(
                        "Usage: /sandbox run <python3|node18|rust|go|linux> <code>",
                    );
                }
                let runtime = match CloudRuntime::from_str(&args[1]) {
                    Some(r) => r,
                    None => {
                        return CommandOutput::err(&format!(
                            "Unknown runtime '{}'. Available: python3, node18, rust, go, linux",
                            args[1]
                        ))
                    }
                };
                let code = args[2..].join(" ");
                self.run_code(runtime, &code, want_json)
            }
            "runtimes" | "rt" => {
                let names: Vec<&str> = CloudRuntime::variants()
                    .iter()
                    .map(|r| r.as_str())
                    .collect();
                let msg = format!("Cloud runtimes: {}", names.join(", "));
                if want_json {
                    return CommandOutput::ok(&msg)
                        .with_json(serde_json::json!({"runtimes": names}));
                }
                CommandOutput::ok(&msg)
            }
            other => CommandOutput::err(&format!(
                "Unknown subcommand: {}. Available: status, set, run, runtimes",
                other
            )),
        }
    }
}

impl SandboxCmd {
    fn status(&self, want_json: bool) -> CommandOutput {
        let current = global_sandbox()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .mode();
        let label = match current {
            CliSandboxMode::Disabled => "disabled",
            CliSandboxMode::ReadOnly => "read-only",
            CliSandboxMode::Docker => "docker",
        };
        let cloud = CloudSandbox::default_local();
        let provider = cloud.provider_name();
        let msg = format!(
            "Sandbox: {} | Cloud provider: {}\n   Runtimes: python3, node18, rust, go, linux",
            label, provider
        );
        if want_json {
            return CommandOutput::ok(&msg).with_json(serde_json::json!({
                "mode": label,
                "provider": provider,
            }));
        }
        CommandOutput::ok(&msg)
    }

    fn run_code(&self, runtime: CloudRuntime, code: &str, want_json: bool) -> CommandOutput {
        let mut cloud = CloudSandbox::default_local();
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => return CommandOutput::err(&format!("Failed to create tokio runtime: {}", e)),
        };
        match rt.block_on(cloud.run_code(code, runtime)) {
            Ok(result) => {
                let mut msg = String::new();
                msg.push_str(&format!(
                    "{} (exit={}, {:.0}ms)\n",
                    runtime.as_str(),
                    result.exit_code,
                    result.execution_time.as_secs_f64() * 1000.0
                ));
                if !result.stdout.is_empty() {
                    msg.push_str("-- stdout --\n");
                    msg.push_str(&result.stdout);
                    if !result.stdout.ends_with('\n') {
                        msg.push('\n');
                    }
                }
                if !result.stderr.is_empty() {
                    msg.push_str("-- stderr --\n");
                    msg.push_str(&result.stderr);
                    if !result.stderr.ends_with('\n') {
                        msg.push('\n');
                    }
                }
                if want_json {
                    return CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "runtime": runtime.as_str(),
                        "exit_code": result.exit_code,
                        "stdout": result.stdout,
                        "stderr": result.stderr,
                        "execution_time_ms": result.execution_time.as_secs_f64() * 1000.0,
                    }));
                }
                CommandOutput::ok(&msg)
            }
            Err(e) => CommandOutput::err(&format!("Sandbox exec failed: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_status_does_not_panic() {
        let cmd = SandboxCmd;
        let out = cmd.execute(&[], None);
        assert!(out.success);
        assert!(out.message.contains("Sandbox"));
    }

    #[test]
    fn test_sandbox_runtimes_lists_variants() {
        let cmd = SandboxCmd;
        let out = cmd.execute(&["runtimes".into()], None);
        assert!(out.success);
        assert!(out.message.contains("python:3.11"));
        assert!(out.message.contains("node:18"));
        assert!(out.message.contains("rust:latest"));
    }

    #[test]
    fn test_sandbox_set_toggles_mode() {
        let cmd = SandboxCmd;
        let out = cmd.execute(&["set".into(), "read-only".into()], None);
        assert!(out.success);
        let after = cmd.execute(&["status".into()], None);
        assert!(after.message.contains("read-only"));
        let _ = cmd.execute(&["set".into(), "disabled".into()], None);
    }

    #[test]
    fn test_sandbox_run_requires_args() {
        let cmd = SandboxCmd;
        let out = cmd.execute(&["run".into()], None);
        assert!(!out.success);
        assert!(out.message.contains("Usage"));
    }

    #[test]
    fn test_sandbox_run_rejects_unknown_runtime() {
        let cmd = SandboxCmd;
        let out = cmd.execute(&["run".into(), "cobol".into(), "1+1".into()], None);
        assert!(!out.success);
        assert!(out.message.contains("Unknown runtime"));
    }
}

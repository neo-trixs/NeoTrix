use std::collections::HashMap;

#[cfg(feature = "sandbox")]
use std::time::Duration;

use crate::core::nt_core_cap::CapabilityVector;

use super::types::SubAgentResult;

/// Simulate a sub-agent execution on a blocking thread.
///
/// In production this would call an LLM; here we produce a deterministic
/// result based on prompt length and keyword matching.
pub(super) fn simulate_agent_execution(
    prompt: &str,
    _capabilities: Option<&CapabilityVector>,
) -> Result<SubAgentResult, String> {
    use rand::Rng;
    use std::time::Duration;

    let mut rng = rand::thread_rng();

    let seed = prompt.len() as u64;
    let simulated_tokens = 50 + (seed % 200);
    let simulated_duration_ms = 10 + (seed % 90);
    std::thread::sleep(Duration::from_millis(simulated_duration_ms));

    let summary = format!(
        "Analysis of '{}': {} key insights found.",
        if prompt.len() > 40 {
            format!("{}...", &prompt[..40])
        } else {
            prompt.to_string()
        },
        (seed % 5) + 1,
    );

    let evidence_count = 1 + (seed % 5) as usize;
    let evidence: Vec<String> = (0..evidence_count)
        .map(|i| {
            format!(
                "Evidence {}: finding from segment {} (confidence {:.2})",
                i + 1,
                (seed as usize + i) % 10 + 1,
                rng.gen_range(0.6..1.0),
            )
        })
        .collect();

    let mut metrics = HashMap::new();
    metrics.insert("tokens_per_second".into(), 1000.0 / simulated_duration_ms as f64);
    metrics.insert("coherence_score".into(), 0.75 + (seed % 20) as f64 / 100.0);
    metrics.insert("novelty_score".into(), 0.3 + (seed % 30) as f64 / 100.0);

    Ok(SubAgentResult {
        summary,
        evidence,
        execution_metrics: metrics,
        total_tokens: simulated_tokens,
        duration_ms: simulated_duration_ms,
    })
}

/// A sandboxed sub-agent that executes code in an isolated sandbox.
///
/// Supports long-horizon execution with configurable duration limits,
/// periodic progress reporting via a channel, and preemption through
/// tokio timeout. Requires the `sandbox` feature.
#[cfg(feature = "sandbox")]
#[derive(Debug)]
pub struct SandboxAgent {
    /// Unique identifier for this agent.
    pub id: uuid::Uuid,
    /// Code to execute inside the sandbox.
    pub code: String,
    /// The sandbox instance (local/docker/wasm/remote).
    pub sandbox: crate::neotrix::nt_shield_sandbox_entry::Sandbox,
    /// Maximum wall-clock duration before the agent is killed.
    pub max_duration: Duration,
    /// Optional channel for periodic progress reports.
    pub progress_tx: Option<tokio::sync::mpsc::Sender<String>>,
}

#[cfg(feature = "sandbox")]
impl SandboxAgent {
    /// Execute the code in the sandbox with duration limit and progress reporting.
    pub async fn execute(&mut self) -> Result<String, String> {
        use std::time::Instant;
        let code = self.code.clone();
        let id = self.id;
        let sandbox_mode = self.sandbox.mode;
        let work_dir = self.sandbox.work_dir.clone();
        let progress_tx = self.progress_tx.clone();
        let deadline = self.max_duration;
        let start = Instant::now();

        let send_progress = |msg: &str| {
            if let Some(tx) = &progress_tx {
                let _ = tx.try_send(format!("[{}] {}", id, msg));
            }
        };

        send_progress("sandbox: execution started");

        let result: Result<Result<crate::neotrix::nt_shield_sandbox_entry::SandboxResult, tokio::task::JoinError>, tokio::time::error::Elapsed> =
            tokio::time::timeout(deadline, tokio::task::spawn_blocking(move || {
                let script_path =
                    std::path::PathBuf::from(&work_dir).join(format!("sandbox_{}.sh", id));
                let _ = std::fs::create_dir_all(&work_dir);
                let _ = std::fs::write(&script_path, &code);

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755));
                }

                let cmd = script_path.to_string_lossy().to_string();
                let sandbox = crate::neotrix::nt_shield_sandbox_entry::Sandbox::new(sandbox_mode);
                sandbox.execute(&cmd)
            }))
            .await;

        match result {
            Ok(Ok(sr)) if sr.exit_code == 0 => {
                send_progress(&format!("sandbox: completed in {:?}", start.elapsed()));
                Ok(sr.stdout)
            }
            Ok(Ok(sr)) => {
                send_progress(&format!("sandbox: failed (exit={})", sr.exit_code));
                Err(format!("exit {}: {}", sr.exit_code, sr.stderr))
            }
            Ok(Err(join_err)) => {
                send_progress("sandbox: task panicked");
                Err(join_err.to_string())
            }
            Err(_elapsed) => {
                send_progress("sandbox: timed out");
                Err("sandbox execution timed out".to_string())
            }
        }
    }
}

/// Variants of sub-agents supported by the pool.
#[derive(Debug)]
pub enum SubAgentVariant {
    /// Default simulated sub-agent.
    Default,
    /// Sandboxed WASM execution agent.
    #[cfg(feature = "sandbox")]
    Sandbox(SandboxAgent),
}

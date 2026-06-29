use std::collections::HashMap;
use std::time::Duration;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use crate::core::nt_core_cap::CapabilityVector;

use super::types::SubAgentResult;

/// AgentKind — Explorer/Worker/Planner type differentiation.
/// Each kind has distinct context budget, tool access, and execution profile.
/// Inspired by Claude Code's 3 SubAgent types (explorer/plan/worker).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AgentKind {
    /// Explorer: read-only, lightweight context, web+file tools only.
    /// For research, fact-gathering, code reading.
    Explorer,
    /// Worker: execution-oriented, default toolset.
    /// For implementing changes, running tests, writing files.
    Worker,
    /// Planner: heavy reasoning, multi-tool access.
    /// For architecture decisions, complex analysis, dependency resolution.
    Planner,
}

impl AgentKind {
    pub fn label(&self) -> &'static str {
        match self {
            AgentKind::Explorer => "explorer",
            AgentKind::Worker => "worker",
            AgentKind::Planner => "planner",
        }
    }

    pub fn context_budget(&self) -> usize {
        match self {
            AgentKind::Explorer => 2048,
            AgentKind::Worker => 4096,
            AgentKind::Planner => 8192,
        }
    }

    pub fn max_tokens_per_step(&self) -> usize {
        match self {
            AgentKind::Explorer => 512,
            AgentKind::Worker => 1024,
            AgentKind::Planner => 2048,
        }
    }

    pub fn allowed_tools(&self) -> Vec<&'static str> {
        match self {
            AgentKind::Explorer => vec![
                "read",
                "glob",
                "grep",
                "web_search",
                "web_fetch",
                "status",
                "check",
                "list",
                "query",
            ],
            AgentKind::Worker => vec![
                "read",
                "glob",
                "grep",
                "write",
                "edit",
                "delete",
                "bash",
                "compile",
                "test",
                "web_search",
            ],
            AgentKind::Planner => vec![
                "read",
                "glob",
                "grep",
                "web_search",
                "web_fetch",
                "bash",
                "status",
                "query",
                "reason",
                "metrics",
                "profile",
                "diff",
                "log",
            ],
        }
    }

    pub fn timeout_secs(&self) -> u64 {
        match self {
            AgentKind::Explorer => 60,
            AgentKind::Worker => 300,
            AgentKind::Planner => 600,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "explorer" => Some(AgentKind::Explorer),
            "worker" => Some(AgentKind::Worker),
            "planner" => Some(AgentKind::Planner),
            _ => None,
        }
    }
}

/// Simulate a sub-agent execution on a blocking thread.
/// Produces different output based on AgentKind.
pub(super) async fn simulate_agent_execution(
    prompt: &str,
    kind: AgentKind,
    _capabilities: Option<&CapabilityVector>,
) -> Result<SubAgentResult, String> {
    let seed = prompt.len() as u64;
    let simulated_tokens = match kind {
        AgentKind::Explorer => 30 + (seed % 100),
        AgentKind::Worker => 50 + (seed % 200),
        AgentKind::Planner => 100 + (seed % 300),
    };
    let simulated_duration_ms = match kind {
        AgentKind::Explorer => 5 + (seed % 30),
        AgentKind::Worker => 10 + (seed % 90),
        AgentKind::Planner => 30 + (seed % 200),
    };
    tokio::time::sleep(Duration::from_millis(simulated_duration_ms)).await;

    let kind_tag = kind.label();
    let summary = format!(
        "[{}] Analysis of '{}': {} key insights found.",
        kind_tag,
        if prompt.len() > 40 {
            format!("{}...", &prompt[..40])
        } else {
            prompt.to_string()
        },
        (seed % 5) + 1,
    );

    let evidence_count = 1 + (seed % 5) as usize;
    let mut rng = StdRng::seed_from_u64(seed);
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
    metrics.insert(
        "tokens_per_second".into(),
        1000.0 / simulated_duration_ms as f64,
    );
    metrics.insert("coherence_score".into(), 0.75 + (seed % 20) as f64 / 100.0);
    metrics.insert("novelty_score".into(), 0.3 + (seed % 30) as f64 / 100.0);
    metrics.insert(
        "agent_kind".into(),
        match kind {
            AgentKind::Explorer => 0.0,
            AgentKind::Worker => 1.0,
            AgentKind::Planner => 2.0,
        },
    );

    Ok(SubAgentResult {
        summary,
        evidence,
        execution_metrics: metrics,
        total_tokens: simulated_tokens,
        duration_ms: simulated_duration_ms,
    })
}

/// A sandboxed sub-agent that executes code in an isolated sandbox.
#[cfg(feature = "sandbox")]
#[derive(Debug)]
pub struct SandboxAgent {
    pub id: uuid::Uuid,
    pub code: String,
    pub kind: AgentKind,
    pub sandbox: crate::neotrix::nt_shield_sandbox_entry::Sandbox,
    pub max_duration: Duration,
    pub progress_tx: Option<tokio::sync::mpsc::Sender<String>>,
}

#[cfg(feature = "sandbox")]
impl SandboxAgent {
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
                if let Err(e) = tx.try_send(format!("[{}] {}", id, msg)) {
                    log::warn!("execution.rs: try_send failed: {e}");
                }
            }
        };

        send_progress("sandbox: execution started");

        let result: Result<
            Result<
                crate::neotrix::nt_shield_sandbox_entry::SandboxResultEntry,
                tokio::task::JoinError,
            >,
            tokio::time::error::Elapsed,
        > = tokio::time::timeout(
            deadline,
            tokio::task::spawn_blocking(move || {
                let script_path =
                    std::path::PathBuf::from(&work_dir).join(format!("sandbox_{}.sh", id));
                let _ = std::fs::create_dir_all(&work_dir);
                let tmp = script_path.with_extension("tmp.sh");
                let _ = std::fs::write(&tmp, &code);

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755));
                }
                let _ = std::fs::rename(&tmp, &script_path);

                let cmd = script_path.to_string_lossy().to_string();
                let sandbox = crate::neotrix::nt_shield_sandbox_entry::Sandbox::new(sandbox_mode);
                sandbox.execute(&cmd)
            }),
        )
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
    /// Explorer: read-only, lightweight context web+file tools.
    Explorer,
    /// Worker: execution-oriented, default toolset.
    Worker,
    /// Planner: heavy reasoning, multi-tool access.
    Planner,
    /// Sandboxed WASM execution agent.
    #[cfg(feature = "sandbox")]
    Sandbox(SandboxAgent),
}

impl SubAgentVariant {
    pub fn kind(&self) -> AgentKind {
        match self {
            SubAgentVariant::Default | SubAgentVariant::Worker => AgentKind::Worker,
            SubAgentVariant::Explorer => AgentKind::Explorer,
            SubAgentVariant::Planner => AgentKind::Planner,
            #[cfg(feature = "sandbox")]
            SubAgentVariant::Sandbox(sa) => sa.kind,
        }
    }
}

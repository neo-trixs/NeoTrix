// NOT ORPHAN — consumed by self_evolution_loop/core.rs (AutoDeployer field + run_auto_deploy call)
// See self_evolution_loop/core.rs for usage context.
#![forbid(unsafe_code)]

use std::process::Command;

use super::self_evolution_loop::{MutationOp, SelfEvolutionStep};

/// Configuration for the auto-deployment loop (yoyo-inspired).
#[derive(Debug, Clone)]
pub struct AutoDeployConfig {
    /// Master switch.
    pub enabled: bool,
    /// Remote repository URL (e.g. "https://github.com/user/repo.git").
    pub repo: String,
    /// Branch to push to.
    pub branch: String,
    /// How many evolution cycles between deployment attempts.
    pub interval_cycles: u64,
}

impl Default for AutoDeployConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            repo: String::new(),
            branch: "main".to_string(),
            interval_cycles: 100,
        }
    }
}

/// A summary of what was deployed.
#[derive(Debug, Clone)]
pub struct DeployReport {
    /// The git commit hash (first 12 chars).
    pub commit_hash: String,
    /// Number of mutations included in this deploy.
    pub mutations_applied: usize,
    /// Number of files that were changed.
    pub files_changed: usize,
}

/// The yoyo-style auto-deployer: periodically commits and pushes
/// successful mutations when the evolution success rate is high enough.
#[derive(Debug, Clone)]
pub struct AutoDeployer {
    pub config: AutoDeployConfig,
    /// Tracks how many mutations were deployed in the last push.
    last_deploy_count: usize,
    /// The SHA of the last successful deploy.
    last_commit_hash: String,
}

impl Default for AutoDeployer {
    fn default() -> Self {
        Self::new(AutoDeployConfig::default())
    }
}

impl AutoDeployer {
    pub fn new(config: AutoDeployConfig) -> Self {
        Self {
            config,
            last_deploy_count: 0,
            last_commit_hash: String::new(),
        }
    }

    /// Check whether git is available and the working directory is a repo.
    pub fn is_ready(&self) -> bool {
        let git_check = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        matches!(git_check, Ok(s) if s.success())
    }

    /// Main entry: called periodically by the evolution loop.
    ///
    /// `success_rate` should be the fraction of recent mutations that improved
    /// the score. If it exceeds 0.5 and there are new untracked mutations,
    /// a commit + push is attempted.
    pub fn check_and_deploy(
        &mut self,
        success_rate: f64,
        archive: &[SelfEvolutionStep],
    ) -> Option<DeployReport> {
        if !self.config.enabled || success_rate <= 0.5 || archive.is_empty() {
            return None;
        }

        let new_mutations: Vec<&SelfEvolutionStep> = archive
            .iter()
            .filter(|s| s.score_after.unwrap_or(0.0) > s.score_before)
            .collect();

        if new_mutations.is_empty() {
            return None;
        }

        self.do_deploy(&new_mutations)
    }

    fn do_deploy(&mut self, mutations: &[&SelfEvolutionStep]) -> Option<DeployReport> {
        if !self.is_ready() {
            log::warn!("AUTODEPLOY: git not available, skipping deploy");
            return None;
        }

        // Build a summary of what changed
        let mut summary_lines: Vec<String> = Vec::new();
        summary_lines.push("NeoTrix auto-deploy: evolution mutations".to_string());

        for step in mutations {
            match &step.mutation {
                MutationOp::TuneParam { target, delta } => {
                    summary_lines.push(format!(
                        "  TuneParam {} += {:.4} (score {:.3} -> {:.3})",
                        target,
                        delta,
                        step.score_before,
                        step.score_after.unwrap_or(step.score_before)
                    ));
                }
                MutationOp::AddHandler { position, .. } => {
                    summary_lines.push(format!(
                        "  AddHandler at {} (score {:.3} -> {:.3})",
                        position,
                        step.score_before,
                        step.score_after.unwrap_or(step.score_before)
                    ));
                }
                MutationOp::RewriteHandler { name, .. } => {
                    summary_lines.push(format!(
                        "  RewriteHandler {} (score {:.3} -> {:.3})",
                        name,
                        step.score_before,
                        step.score_after.unwrap_or(step.score_before)
                    ));
                }
                MutationOp::SwapPolicy { gates } => {
                    summary_lines.push(format!(
                        "  SwapPolicy [{}] (score {:.3} -> {:.3})",
                        gates.join(", "),
                        step.score_before,
                        step.score_after.unwrap_or(step.score_before)
                    ));
                }
                MutationOp::RewritePrimitive { name, .. } => {
                    summary_lines.push(format!(
                        "  RewritePrimitive {} (score {:.3} -> {:.3})",
                        name,
                        step.score_before,
                        step.score_after.unwrap_or(step.score_before)
                    ));
                }
                MutationOp::RewriteMeta { strategy } => {
                    summary_lines.push(format!(
                        "  RewriteMeta v{} (score {:.3} -> {:.3})",
                        strategy.version,
                        step.score_before,
                        step.score_after.unwrap_or(step.score_before)
                    ));
                }
                MutationOp::SelfModifyProposal {
                    target,
                    target_type,
                    ..
                } => {
                    summary_lines.push(format!(
                        "  SelfModifyProposal {}:{} (score {:.3} -> {:.3})",
                        target_type,
                        target,
                        step.score_before,
                        step.score_after.unwrap_or(step.score_before)
                    ));
                }
            }
        }

        let commit_message = summary_lines.join("\n");

        // Stage everything and commit
        let add_result = Command::new("git")
            .args(["add", "-A"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        if !matches!(add_result, Ok(s) if s.success()) {
            log::error!("AUTODEPLOY: git add failed");
            return None;
        }

        let commit_result = Command::new("git")
            .args(["commit", "-m", &commit_message])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        if !matches!(commit_result, Ok(s) if s.success()) {
            log::warn!("AUTODEPLOY: git commit failed (nothing to commit?)");
            // Count staged changes a different way
            let diff_result = Command::new("git")
                .args(["diff", "--cached", "--stat"])
                .output();
            if let Ok(out) = diff_result {
                let stats = String::from_utf8_lossy(&out.stdout);
                if stats.trim().is_empty() {
                    return None; // nothing to commit
                }
            }
            return None;
        }

        // Get the commit hash
        let hash_output = Command::new("git")
            .args(["rev-parse", "--short=12", "HEAD"])
            .output();
        let commit_hash = match hash_output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => "unknown".to_string(),
        };

        // Count files changed
        let diff_output = Command::new("git")
            .args(["diff", "--cached", "--numstat"])
            .output();
        let files_changed = match diff_output {
            Ok(o) => {
                let out = String::from_utf8_lossy(&o.stdout);
                out.lines().filter(|l| !l.trim().is_empty()).count()
            }
            Err(_) => 0,
        };

        // Push to remote if configured
        if !self.config.repo.is_empty() {
            let push_result = Command::new("git")
                .args([
                    "push",
                    &self.config.repo,
                    &format!("HEAD:{}", self.config.branch),
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            if matches!(push_result, Ok(s) if !s.success()) {
                log::warn!("AUTODEPLOY: git push failed (remote not configured?)");
            }
        }

        self.last_deploy_count = mutations.len();
        self.last_commit_hash = commit_hash.clone();

        Some(DeployReport {
            commit_hash,
            mutations_applied: mutations.len(),
            files_changed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::self_evolution_loop::SelfEvolutionStep;

    fn dummy_step(id: u64, before: f64, after: f64) -> SelfEvolutionStep {
        SelfEvolutionStep {
            id,
            mutation: MutationOp::TuneParam {
                target: "test.param".into(),
                delta: after - before,
            },
            parent_id: 0,
            score_before: before,
            score_after: Some(after),
            compiles: true,
            timestamp: 0,
            generation: 1,
            accepted: false,
            cmp_score: None,
        }
    }

    #[test]
    fn test_deploy_config_defaults() {
        let cfg = AutoDeployConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.branch, "main");
        assert_eq!(cfg.interval_cycles, 100);
    }

    #[test]
    fn test_check_and_deploy_skips_low_success_rate() {
        let mut deployer = AutoDeployer::default();
        let archive = vec![dummy_step(1, 0.5, 0.6)];
        let report = deployer.check_and_deploy(0.3, &archive);
        assert!(report.is_none(), "should skip when success_rate <= 0.5");
    }

    #[test]
    fn test_check_and_deploy_skips_empty_archive() {
        let mut deployer = AutoDeployer::default();
        let report = deployer.check_and_deploy(0.9, &[]);
        assert!(report.is_none(), "should skip when archive is empty");
    }

    #[test]
    fn test_check_and_deploy_disabled() {
        let cfg = AutoDeployConfig {
            enabled: false,
            ..Default::default()
        };
        let mut deployer = AutoDeployer::new(cfg);
        let archive = vec![dummy_step(1, 0.5, 0.6)];
        let report = deployer.check_and_deploy(0.9, &archive);
        assert!(report.is_none(), "should skip when disabled");
    }

    #[test]
    fn test_is_ready_false_when_no_git() {
        let deployer = AutoDeployer::default();
        // Running outside a git repo should return false
        // We'll just check it doesn't panic
        let _ = deployer.is_ready();
    }

    #[test]
    fn test_new_initializes_empty() {
        let deployer = AutoDeployer::default();
        assert!(deployer.last_commit_hash.is_empty());
        assert_eq!(deployer.last_deploy_count, 0);
    }
}

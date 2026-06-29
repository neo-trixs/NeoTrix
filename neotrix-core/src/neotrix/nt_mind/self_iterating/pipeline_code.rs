use super::pipeline_core::{BrainStage, StageDecision};
use super::SelfIteratingBrain;
use crate::make_stage;
pub(crate) use crate::neotrix::nt_core_error::NeoTrixError;

make_stage!(SecurityStage);
impl BrainStage for SecurityStage {
    fn name(&self) -> &str {
        "nt_shield_scan"
    }
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain.task_scratch.current_task.clone();
        let code_context = brain._open_source_insights.clone().unwrap_or_default();
        let scanner = super::secret_scanner::SecretScanner::new();
        let result = scanner.scan_with_context(&task, &code_context);
        if !result.is_safe() {
            let critical_count = result
                .count_by_severity
                .get(&super::secret_scanner::SecretSeverity::Critical)
                .copied()
                .unwrap_or(0);
            log::warn!(
                "[nt_shield-scan] found {} secrets ({} critical) — max_severity={:?}, risk={:.2}",
                result.findings.len(),
                critical_count,
                result.max_severity,
                result.risk_score()
            );
            if let Some(ref mut router) = brain.attention_router {
                for finding in &result.findings {
                    router.wm().broadcast(&format!(
                        "Security alert: {} at line {} — \"{}\"",
                        finding.pattern, finding.line, finding.snippet
                    ));
                }
            }
            if critical_count > 0 {
                brain._set_reward(brain.task_scratch.reward - result.risk_score() * 0.2);
                log::info!(
                    "[nt_shield-scan] penalized reward by {:.4}",
                    result.risk_score() * 0.2
                );
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(SideGitStage);
impl BrainStage for SideGitStage {
    fn name(&self) -> &str {
        "side_git"
    }
    fn frequency(&self) -> usize {
        30
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let workspace = std::env::var("NEOTRIX_WORKSPACE").unwrap_or_else(|_| ".".to_string());
        let ws_path = std::path::Path::new(&workspace);
        if !ws_path.exists() {
            return Ok(StageDecision::Skip("no workspace".into()));
        }
        if let Err(e) = brain.side_git.init() {
            log::warn!("[side_git] init failed: {}", e);
            return Ok(StageDecision::Continue);
        }
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(ws_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                if elapsed.as_secs() < 300 {
                                    files.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
        if !files.is_empty() {
            match brain.side_git.snapshot_files(&files, ws_path) {
                Ok(n) => {
                    if n > 0 {
                        log::info!(
                            "[side_git] snapshotted {} files (total={})",
                            n,
                            brain.side_git.snapshot_count()
                        );
                    }
                }
                Err(e) => log::warn!("[side_git] snapshot error: {}", e),
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(RemoteSyncStage);
impl BrainStage for RemoteSyncStage {
    fn name(&self) -> &str {
        "remote_sync"
    }
    fn frequency(&self) -> usize {
        5
    }

    fn process(&self, _brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        // remote-control integration removed: nt_act_remote_control module was deleted (dead)
        Ok(StageDecision::Continue)
    }
}

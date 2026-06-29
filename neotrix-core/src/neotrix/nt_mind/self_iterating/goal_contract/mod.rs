mod stages;
mod types;
pub use stages::*;
pub use types::*;

use super::SelfIteratingBrain;
use crate::neotrix::nt_world_journal_index::JournalIndex;

/// Write goal contract + evidence to `.neotrix/journal/` as persistent markdown
pub fn write_journal(brain: &SelfIteratingBrain) -> Result<(), std::io::Error> {
    let contract = match brain.goal_state.goal_contract.as_ref() {
        Some(c) => c,
        None => return Ok(()),
    };

    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let journal_dir = home.join(".neotrix").join("journal");
    std::fs::create_dir_all(&journal_dir)?;

    let session_id = chrono::Local::now().format("goal-%Y%m%d-%H%M%S");
    let path = journal_dir.join(format!("{}.md", session_id));

    let mut lines = Vec::new();
    lines.push(format!("# Goal: {}", contract.original_goal));
    lines.push(format!("- Completed: {}", contract.completed));
    lines.push(format!("- Total phases: {}", contract.phases.len()));
    lines.push(String::new());

    for (i, phase) in contract.phases.iter().enumerate() {
        lines.push(format!(
            "## Phase {}: {} ({})",
            i + 1,
            phase.id,
            phase.description
        ));
        lines.push(format!("- Verified: {}", phase.verified));
        for criterion in &phase.done_criteria {
            lines.push(format!(
                "  - [{}] {}",
                if phase.verified { "x" } else { " " },
                criterion
            ));
        }
        if let Some(ref report) = contract.final_verification {
            lines.push(format!(
                "- Criteria met: {}/{}",
                report.met_criteria, report.total_criteria
            ));
        }
        lines.push(String::new());
    }

    let evidence_count = brain.goal_state.phase_evidence.len();
    lines.push(format!("## Evidence ({})", evidence_count));
    for ev in &brain.goal_state.phase_evidence {
        lines.push(format!(
            "- {} (iter {}): {} — {}",
            ev.id,
            ev.iteration,
            ev.description,
            if ev.success { "✅" } else { "❌" }
        ));
    }
    lines.push(String::new());

    if let Some(ref report) = contract.final_verification {
        lines.push(format!(
            "## Verification: {}/{} criteria met",
            report.met_criteria, report.total_criteria
        ));
        if !report.unmet_phases.is_empty() {
            lines.push(format!(
                "- Unmet phases: {}",
                report.unmet_phases.join(", ")
            ));
        }
        for (criterion, reason) in &report.failed_criteria {
            lines.push(format!("  - ❌ {}: {}", criterion, reason));
        }
        lines.push(format!(
            "- Overall: {}",
            if report.overall_success {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        ));
    }

    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, lines.join("\n"))?;
    std::fs::rename(&tmp, &path)?;
    log::info!("[journal] written to {:?}", path);

    // Auto-index into cross-session journal search
    let session_id_str = session_id.to_string();
    let timestamp_str = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    if let Ok(idx) = JournalIndex::open() {
        let evidence_count = brain.goal_state.phase_evidence.len();
        let success = contract
            .final_verification
            .as_ref()
            .map_or(false, |r| r.overall_success);
        if let Err(e) = idx.add_entry(
            &session_id_str,
            &contract.original_goal,
            &timestamp_str,
            evidence_count,
            success,
        ) {
            log::warn!("[journal] index entry failed: {}", e);
        } else {
            log::info!(
                "[journal] indexed as '{}' (total: {})",
                session_id_str,
                idx.count().unwrap_or(0)
            );
        }
    }

    Ok(())
}

/// Public helper: set up the _goal_complete check in the outer loop
pub fn should_stop_seal_loop(brain: &SelfIteratingBrain) -> bool {
    brain.goal_state.goal_complete
        || brain
            .goal_state
            .goal_contract
            .as_ref()
            .map_or(false, |c| c.completed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::MicroEdit;
    use crate::neotrix::nt_expert_routing::TaskType;

    #[test]
    fn test_decompose_code_task() {
        let contract = GoalContract::decompose("实现用户登录功能", TaskType::CodeGeneration);
        assert_eq!(contract.phases.len(), 4);
        assert_eq!(contract.phases[0].id, "understand");
        assert_eq!(contract.phases[2].done_criteria[0], "新增测试覆盖关键路径");
    }

    #[test]
    fn test_decompose_debug_task() {
        let contract = GoalContract::decompose("修复内存泄漏", TaskType::Debugging);
        assert_eq!(contract.phases.len(), 3);
        assert_eq!(contract.phases[0].id, "reproduce");
        assert_eq!(contract.phases[2].id, "fix");
    }

    #[test]
    fn test_decompose_research_task() {
        let contract = GoalContract::decompose("研究Rust异步编程", TaskType::Research);
        assert_eq!(contract.phases.len(), 3);
        assert_eq!(contract.phases[1].id, "synthesize");
    }

    #[test]
    fn test_decompose_code_without_test() {
        let contract = GoalContract::decompose("重构模块", TaskType::CodeGeneration);
        assert_eq!(contract.phases.len(), 3);
    }

    #[test]
    fn test_code_with_test_flag() {
        let contract = GoalContract::decompose("添加单元测试", TaskType::CodeGeneration);
        assert_eq!(contract.phases.len(), 4);
    }

    #[test]
    fn test_advance_phase() {
        let mut contract = GoalContract::decompose("task", TaskType::General);
        assert_eq!(contract.current_phase, 0);
        contract.advance_phase();
        assert_eq!(contract.current_phase, 1);
        contract.advance_phase();
        assert_eq!(contract.current_phase, 2);
        contract.advance_phase();
        assert!(contract.completed);
    }

    #[test]
    fn test_all_phases_done() {
        let mut contract = GoalContract::decompose("task", TaskType::General);
        assert!(!contract.all_phases_done());
        for phase in &mut contract.phases {
            phase.verified = true;
        }
        assert!(contract.all_phases_done());
    }

    #[test]
    fn test_verification_report_all_pass() {
        let mut contract = GoalContract::decompose("task", TaskType::General);
        for phase in &mut contract.phases {
            phase.verified = true;
        }
        let report = contract.verification_report();
        assert!(report.overall_success);
        assert!(report.unmet_phases.is_empty());
    }

    #[test]
    fn test_verification_report_partial() {
        let mut contract = GoalContract::decompose("task", TaskType::General);
        contract.phases[0].verified = true;
        let report = contract.verification_report();
        assert!(!report.overall_success);
        assert_eq!(report.unmet_phases.len(), 2);
    }

    #[test]
    fn test_analyze_failure_no_edits() {
        let recovery = analyze_failure_no_brain("test_stage", -0.2, &[]);
        assert!(matches!(recovery, RecoveryAction::RetryStage));
    }

    #[test]
    fn test_analyze_failure_severe_drop() {
        let edits = vec![MicroEdit::AdjustDimension("ui".into(), -0.5)];
        let recovery = analyze_failure_no_brain("test_stage", -0.5, &edits);
        match recovery {
            RecoveryAction::NarrowEdit { dimension, .. } => assert_eq!(dimension, "ui"),
            _ => panic!("expected NarrowEdit"),
        }
    }

    #[test]
    fn test_evidence_capture() {
        let evidence = PhaseEvidence {
            id: "ev-test-1".into(),
            phase_id: "implement".into(),
            stage_name: "apply_edits".into(),
            evidence_type: EvidenceType::BuildPass,
            description: "构建通过".into(),
            success: true,
            details: "exit code 0".into(),
            iteration: 5,
        };
        assert_eq!(evidence.phase_id, "implement");
        assert!(evidence.success);
    }

    #[test]
    fn test_progress_description() {
        let contract = GoalContract::decompose("task", TaskType::General);
        let desc = contract.progress_description();
        assert!(desc.contains("0/3"));
        assert!(desc.contains("understand"));
    }

    #[test]
    fn test_recovery_action_descriptions() {
        assert_eq!(RecoveryAction::RetryStage.description(), "重试当前阶段");
        let narrow = RecoveryAction::NarrowEdit {
            dimension: "x".into(),
            description: "fix".into(),
        };
        assert!(narrow.description().contains("x"));
        let skip = RecoveryAction::SkipPhase {
            phase_id: "test".into(),
            reason: "not needed".into(),
        };
        assert!(skip.description().contains("test"));
        let abort = RecoveryAction::AbortGoal {
            reason: "invalid".into(),
        };
        assert!(abort.description().contains("invalid"));
    }

    fn analyze_failure_no_brain(stage: &str, reward: f64, edits: &[MicroEdit]) -> RecoveryAction {
        if edits.is_empty() && reward < -0.1 {
            return RecoveryAction::RetryStage;
        }
        if reward < -0.3 {
            let dim = edits
                .last()
                .and_then(|e: &MicroEdit| e.dimension_name().map(String::from))
                .unwrap_or_else(|| "unknown".into());
            return RecoveryAction::NarrowEdit {
                dimension: dim,
                description: format!("阶段 '{}' 导致奖励下降 {:.2}", stage, reward),
            };
        }
        RecoveryAction::RetryStage
    }
}

//! 命令注册表 — default_registry() 注册所有命令

use crate::cli::commands::types::CommandRegistry;
use crate::cli::commands::bench_cmds::BenchmarkCmd;
use crate::cli::commands::agent_cmds::{AgentCmd, DiscoverCmd, McpCmd};
use crate::cli::commands::brain_cmds::E8Cmd;
use crate::cli::commands::second_brain_cmds::BrainCmd;
use crate::cli::commands::consciousness_cmds::ConsciousnessCmd;
use crate::cli::commands::core_cmds::{ClearCmd, CompletionsCmd, ExitCmd, HelpCmd, StatsCmd, VersionCmd, ConfigCmd};
use crate::cli::commands::cost_cmds::{CostCmd, ApprovalCmd};
use crate::cli::commands::budget_cmds::BudgetCmd;
use crate::cli::commands::file_cmds::{FileCreateCmd, FileDiffCmd, FileEditCmd, FilePatchCmd, FileReadCmd, FileWriteCmd};
use crate::cli::commands::git_cmds::{CommitCmd, GitCmd, PrCmd};
use crate::cli::commands::goal_cmds::GoalCmd;
use crate::cli::commands::session_cmds::{CompactCmd, ContextCmd, ForkCmd, HistoryCmd, ResumeCmd, SessionCmd};
use crate::cli::commands::theme_cmd::ThemeCmd;
use crate::cli::commands::connector_cmds::ConnectorCmd;
use crate::cli::commands::review_cmds::ReviewCmd;
use crate::cli::commands::schedule_cmds::ScheduleCmd;
use crate::cli::commands::ui_cmds::{BackgroundCommand, RouterCmd, SideCmd, VimCmd, WorkSpaceCmd};
use crate::cli::commands::wallet_cmd::WalletCmd;
use crate::cli::commands::swap_cmd::{ApproveCmd, SwapCmd, TransferCmd};
use crate::cli::commands::doctor_cmds::DoctorCmd;
use crate::cli::commands::plugin_cmds::PluginCmd;
use crate::cli::commands::search_cmds::SearchCmd;
use crate::cli::commands::model_cmds::ModelCmd;
use crate::cli::commands::profile_cmds::ProfileCmd;
use crate::cli::commands::sandbox_cmds::SandboxCmd;
use crate::cli::commands::hypothesis_cmds::HypothesisCmd;
use crate::cli::commands::evidence_cmds::EvidenceCmd;
use crate::cli::commands::kanban_cmds::BoardCmd;
use crate::cli::commands::plan_cmds::PlanCmd;
use crate::cli::commands::session_recovery_cmds::SessionRecoveryCmd;
use crate::cli::commands::skill_cmds::SkillCmd;
use crate::cli::commands::provider_cmds::ProviderCmd;
use crate::cli::commands::free_cmds::FreeCmd;
use crate::cli::commands::kb_cmds::KbCmd;
use crate::cli::commands::wiki_cmds::WikiCmd;
use crate::cli::commands::self_audit_cmds::SelfAuditCmd;
use crate::cli::commands::osint_cmds::OsintCmd;
use crate::cli::commands::comm_cmds::CommCmd;
use crate::cli::commands::consolidated_cmds::{
    FileCmd, WalletAggCmd, UiAggCmd, GitAggCmd, SessionAggCmd, ConsolidatedAgentCmd,
};

pub fn default_registry() -> CommandRegistry {
    let mut reg = CommandRegistry::new().with_session_logging();

    // System
    reg.register(Box::new(HelpCmd));
    reg.register(Box::new(StatsCmd));
    reg.register(Box::new(ExitCmd));
    reg.register(Box::new(ClearCmd));
    reg.register(Box::new(VersionCmd));
    reg.register(Box::new(CompletionsCmd));
    reg.register(Box::new(ConfigCmd));
    reg.register(Box::new(DoctorCmd));
    reg.register(Box::new(BenchmarkCmd));
    reg.register(Box::new(ConsciousnessCmd));

    // E8 (diagnostic, kept as System)
    reg.register(Box::new(E8Cmd));
    // Second Brain (memory graph command)
    reg.register(Box::new(BrainCmd));

    // File
    reg.register(Box::new(FileCmd));          // aggregator
    reg.register(Box::new(FileReadCmd));
    reg.register(Box::new(FileWriteCmd));
    reg.register(Box::new(FileCreateCmd));
    reg.register(Box::new(FileEditCmd));
    reg.register(Box::new(FilePatchCmd));
    reg.register(Box::new(FileDiffCmd));

    // Git
    reg.register(Box::new(GitCmd));
    reg.register(Box::new(CommitCmd));
    reg.register(Box::new(PrCmd));

    // Session
    reg.register(Box::new(SessionCmd));
    reg.register(Box::new(ResumeCmd));
    reg.register(Box::new(ForkCmd));
    reg.register(Box::new(HistoryCmd));
    reg.register(Box::new(ContextCmd));
    reg.register(Box::new(CompactCmd));

    // Agent
    reg.register(Box::new(AgentCmd));
    reg.register(Box::new(DiscoverCmd));
    reg.register(Box::new(McpCmd));

    // Crypto/Finance
    reg.register(Box::new(WalletAggCmd));
    reg.register(Box::new(WalletCmd));
    reg.register(Box::new(SwapCmd));
    reg.register(Box::new(TransferCmd));
    reg.register(Box::new(ApproveCmd));
    reg.register(Box::new(CostCmd));
    reg.register(Box::new(BudgetCmd));

    // Goal/Plan
    reg.register(Box::new(GoalCmd));
    reg.register(Box::new(PlanCmd));
    reg.register(Box::new(ScheduleCmd));

    // Memory
    reg.register(Box::new(EvidenceCmd));
    reg.register(Box::new(HypothesisCmd));
    reg.register(Box::new(SearchCmd));
    reg.register(Box::new(BoardCmd));
    reg.register(Box::new(KbCmd));
    reg.register(Box::new(WikiCmd));

    // UI/Layout
    reg.register(Box::new(UiAggCmd));
    reg.register(Box::new(BackgroundCommand));
    reg.register(Box::new(SideCmd));
    reg.register(Box::new(RouterCmd));
    reg.register(Box::new(VimCmd));
    reg.register(Box::new(WorkSpaceCmd));
    reg.register(Box::new(ThemeCmd));

    // Provider/Model
    reg.register(Box::new(ProviderCmd));
    reg.register(Box::new(FreeCmd));
    reg.register(Box::new(ModelCmd));

    // Skills
    reg.register(Box::new(SkillCmd));

    // Sandbox
    reg.register(Box::new(SandboxCmd));

    // Other
    reg.register(Box::new(ConnectorCmd));
    reg.register(Box::new(ReviewCmd));
    reg.register(Box::new(PluginCmd));
    reg.register(Box::new(ProfileCmd));
    reg.register(Box::new(SessionRecoveryCmd));
    reg.register(Box::new(ApprovalCmd));

    // Aggregation
    reg.register(Box::new(ConsolidatedAgentCmd));
    reg.register(Box::new(GitAggCmd));
    reg.register(Box::new(SessionAggCmd));

    // L4 capability: Rust-native OSINT and self-audit
    reg.register(Box::new(OsintCmd));
    reg.register(Box::new(SelfAuditCmd));

    // L1 capability: NT-SHIELD 通信伪装层观测
    reg.register(Box::new(CommCmd));

    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry_contains_commands() {
        let reg = default_registry();
        let names = reg.list();
        assert!(names.contains(&"/help"));
        assert!(names.contains(&"/stats"));
        assert!(names.contains(&"/exit"));
        assert!(names.contains(&"/goal"));
        assert!(names.contains(&"/read"));
        assert!(names.contains(&"/git"));
        assert!(names.contains(&"/session"));
        assert!(names.contains(&"/connector"));
        assert!(names.contains(&"/cost"));
        assert!(names.contains(&"/background"));
        assert!(names.contains(&"/discover"));
        assert!(names.contains(&"/profile"));
        assert!(names.contains(&"/plugin"));
        assert!(names.contains(&"/hypothesis"));
        assert!(names.contains(&"/board"));
        assert!(names.contains(&"/e8"));
        assert!(names.contains(&"/session-recovery"));
        assert!(names.len() >= 36, "got {} commands (expected 36+, got {})", names.len(), names.len());
    }

    #[test]
    fn test_default_registry_find_by_name_and_alias() {
        let reg = default_registry();
        assert!(reg.find("/h").is_some());
        assert!(reg.find("/?").is_some());
        assert!(reg.find("/g").is_some());
        // /a was alias for /absorb (removed — auto-backend)
        // /e was alias for /evolve (removed — auto-backend)
    }

    #[test]
    fn test_default_registry_execute_known() {
        let reg = default_registry();
        let result = reg.execute("/help", None);
        if !result.success {
            assert!(result.message.contains("help"), "unexpected shield block: {}", result.message);
        }
    }

    #[test]
    fn test_default_registry_execute_unknown() {
        let reg = default_registry();
        let result = reg.execute("/nonexistent", None);
        assert!(!result.success);
        assert!(result.message.contains("Unknown command"));
    }

    #[test]
    fn test_auto_commands_not_in_registry() {
        let reg = default_registry();
        let names = reg.list();
        // Auto-backend commands should NOT be in CLI
        assert!(!names.contains(&"/absorb"), "absorb should be auto-backend");
        assert!(!names.contains(&"/evolve"), "evolve should be auto-backend");
        assert!(!names.contains(&"/mem"), "mem should be auto-backend");
        assert!(!names.contains(&"/save"), "save should be auto-backend");
        assert!(!names.contains(&"/trace"), "trace should be auto-backend");
        assert!(!names.contains(&"/avatar"), "avatar should be auto-backend");
        assert!(names.contains(&"/skills"), "skills now registered as interactive CLI command");
        assert!(!names.contains(&"/explore"), "explore should be auto-backend");
        assert!(!names.contains(&"/cleanup"), "cleanup should be auto-backend");
        assert!(!names.contains(&"/automation"), "automation should be auto-backend");
    }
}

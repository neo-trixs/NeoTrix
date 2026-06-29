//! Command registry — default_registry() registers all commands

use crate::cli::commands::agent_cmds::{AgentCmd, DiscoverCmd};
use crate::cli::commands::bench_cmds::BenchmarkCmd;
use crate::cli::commands::brain_cmds::{
    AbsorbCmd, AvatarCmd, EvolveCmd, MemCmd, SaveCmd, TraceCmd,
};
use crate::cli::commands::budget_cmds::BudgetCmd;
use crate::cli::commands::connector_cmds::ConnectorCmd;
use crate::cli::commands::core_cmds::{
    ClearCmd, CompletionsCmd, ExitCmd, HelpCmd, StatsCmd, VersionCmd,
};
use crate::cli::commands::cost_cmds::CostCmd;
use crate::cli::commands::doctor_cmds::DoctorCmd;
use crate::cli::commands::file_cmds::{
    FileCreateCmd, FileDiffCmd, FileEditCmd, FilePatchCmd, FileReadCmd, FileWriteCmd,
};
use crate::cli::commands::git_cmds::{CommitCmd, GitCmd, PrCmd};
use crate::cli::commands::goal_cmds::GoalCmd;
use crate::cli::commands::model_cmds::ModelCmd;
use crate::cli::commands::plugin_cmds::PluginCmd;
use crate::cli::commands::profile_cmds::ProfileCmd;
use crate::cli::commands::review_cmds::ReviewCmd;
use crate::cli::commands::sandbox_cmds::SandboxCmd;
use crate::cli::commands::schedule_cmds::ScheduleCmd;
use crate::cli::commands::search_cmds::SearchCmd;
use crate::cli::commands::session_cmds::{
    CompactCmd, ContextCmd, ForkCmd, HistoryCmd, ResumeCmd, SessionCmd,
};
use crate::cli::commands::swap_cmd::{ApproveCmd, SwapCmd, TransferCmd};
use crate::cli::commands::theme_cmd::ThemeCmd;
use crate::cli::commands::types::CommandRegistry;
use crate::cli::commands::ui_cmds::{BackgroundCommand, RouterCmd, SideCmd, VimCmd};

pub fn default_registry() -> CommandRegistry {
    let mut reg = CommandRegistry::new();
    reg.register(Box::new(BenchmarkCmd));
    reg.register(Box::new(HelpCmd));
    reg.register(Box::new(StatsCmd));
    reg.register(Box::new(ExitCmd));
    reg.register(Box::new(AbsorbCmd));
    reg.register(Box::new(EvolveCmd));
    reg.register(Box::new(MemCmd));
    reg.register(Box::new(SaveCmd));
    reg.register(Box::new(AgentCmd));
    reg.register(Box::new(DiscoverCmd));
    reg.register(Box::new(ClearCmd));
    reg.register(Box::new(VersionCmd));
    reg.register(Box::new(CompletionsCmd));
    reg.register(Box::new(ThemeCmd));
    reg.register(Box::new(TraceCmd));
    reg.register(Box::new(GoalCmd));
    reg.register(Box::new(AvatarCmd));
    reg.register(Box::new(FileReadCmd));
    reg.register(Box::new(FileWriteCmd));
    reg.register(Box::new(FileCreateCmd));
    reg.register(Box::new(FileEditCmd));
    reg.register(Box::new(FilePatchCmd));
    reg.register(Box::new(FileDiffCmd));
    reg.register(Box::new(GitCmd));
    reg.register(Box::new(CommitCmd));
    reg.register(Box::new(PrCmd));
    reg.register(Box::new(CostCmd));
    reg.register(Box::new(BudgetCmd));
    reg.register(Box::new(CompactCmd));
    reg.register(Box::new(ContextCmd));
    reg.register(Box::new(SessionCmd));
    reg.register(Box::new(ResumeCmd));
    reg.register(Box::new(ForkCmd));
    reg.register(Box::new(SideCmd));
    reg.register(Box::new(RouterCmd));
    reg.register(Box::new(HistoryCmd));
    reg.register(Box::new(BackgroundCommand));
    reg.register(Box::new(ScheduleCmd));
    reg.register(Box::new(VimCmd));
    reg.register(Box::new(ConnectorCmd));
    reg.register(Box::new(ReviewCmd));
    reg.register(Box::new(SearchCmd));
    reg.register(Box::new(PluginCmd));
    reg.register(Box::new(DoctorCmd));
    reg.register(Box::new(ModelCmd));
    reg.register(Box::new(SwapCmd));
    reg.register(Box::new(TransferCmd));
    reg.register(Box::new(ApproveCmd));
    reg.register(Box::new(ProfileCmd));
    reg.register(Box::new(SandboxCmd));
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
        assert!(names.contains(&"/absorb"));
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
        assert!(names.len() >= 33);
    }

    #[test]
    fn test_default_registry_find_by_name_and_alias() {
        let reg = default_registry();
        assert!(reg.find("/h").is_some());
        assert!(reg.find("/?").is_some());
        assert!(reg.find("/q").is_some());
        assert!(reg.find("/g").is_some());
        assert!(reg.find("/a").is_some());
        assert!(reg.find("/e").is_some());
    }

    #[test]
    fn test_default_registry_execute_known() {
        let reg = default_registry();
        let result = reg.execute("/help", None);
        assert!(result.success);
    }

    #[test]
    fn test_default_registry_execute_unknown() {
        let reg = default_registry();
        let result = reg.execute("/nonexistent", None);
        assert!(!result.success);
        assert!(result.message.contains("Unknown command"));
    }
}

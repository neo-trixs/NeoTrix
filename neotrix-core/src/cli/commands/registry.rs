//! 命令注册表 — default_registry() 注册所有命令

use crate::cli::commands::types::CommandRegistry;
use crate::cli::commands::bench_cmds::BenchmarkCmd;
use crate::cli::commands::agent_cmds::{AgentCmd, DiscoverCmd, McpCmd};
use crate::cli::commands::acp_cmds::AcpCmd;
use crate::cli::commands::brain_cmds::E8Cmd;
use crate::cli::commands::second_brain_cmds::BrainCmd;
use crate::cli::commands::consciousness_cmds::ConsciousnessCmd;
use crate::cli::commands::core_cmds::{ClearCmd, CompletionsCmd, ExitCmd, HelpCmd, StatsCmd, VersionCmd, ConfigCmd, CatalogCmd};
use crate::cli::commands::cost_cmds::{CostCmd, ApprovalCmd};
use crate::cli::commands::budget_cmds::BudgetCmd;
use crate::cli::commands::file_cmds::{FileCreateCmd, FileDiffCmd, FileEditCmd, FilePatchCmd, FileReadCmd, FileWriteCmd};
use crate::cli::commands::git_cmds::{CommitCmd, GitCmd, PrCmd};
use crate::cli::commands::goal_cmds::GoalCmd;
use crate::cli::commands::session_cmds::{CompactCmd, ContextCmd, DistillCmd, ForkCmd, HistoryCmd, ResumeCmd, SessionCmd};
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
use crate::cli::commands::contract_cmds::ContractCmd;
use crate::cli::commands::perm_cmds::PermCmd;
use crate::cli::commands::redact_cmds::RedactCmd;
use crate::cli::commands::profile_cmds::ProfileCmd;
use crate::cli::commands::sandbox_cmds::SandboxCmd;
use crate::cli::commands::judge_cmds::JudgeCmd;
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
use crate::cli::commands::chain_cmds::ChainCmd;
use crate::cli::commands::quantum_cmds::QTestCmd;
use crate::cli::commands::code_graph_cmds::CodeGraphCmd;
use crate::cli::commands::sources_cmds::SourcesCmd;
use crate::cli::commands::explore_cmds::ExploreCmd;
use crate::cli::commands::consolidated_cmds::{
    FileCmd, WalletAggCmd, UiAggCmd, GitAggCmd, SessionAggCmd, ConsolidatedAgentCmd, MemoryAggCmd,
};

pub fn default_registry() -> CommandRegistry {
    let mut reg = CommandRegistry::new().with_session_logging();

    // 动态补全池: /completions 从注册表快照生成候选 (斜杠剥离 + 别名展开 + 去重)
    let completions_pool: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

    // System
    reg.register(Box::new(HelpCmd));
    reg.register(Box::new(StatsCmd));
    reg.register(Box::new(ExitCmd));
    reg.register(Box::new(ClearCmd));
    reg.register(Box::new(VersionCmd));
    reg.register(Box::new(CompletionsCmd::new(completions_pool.clone())));
    reg.register(Box::new(CatalogCmd));
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
    reg.register(Box::new(DistillCmd));

    // Agent
    reg.register(Box::new(AgentCmd));
    reg.register(Box::new(DiscoverCmd));
    reg.register(Box::new(McpCmd));
    reg.register(Box::new(AcpCmd));

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
    reg.register(Box::new(CodeGraphCmd));
    reg.register(Box::new(HypothesisCmd));
    reg.register(Box::new(SearchCmd));
    reg.register(Box::new(BoardCmd));
    reg.register(Box::new(KbCmd));
    reg.register(Box::new(WikiCmd));
    reg.register(Box::new(MemoryAggCmd));

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
    reg.register(Box::new(ContractCmd));
    reg.register(Box::new(PermCmd));
    reg.register(Box::new(RedactCmd));

    // Skills
    reg.register(Box::new(SkillCmd));

    // Sandbox
    reg.register(Box::new(SandboxCmd));
    reg.register(Box::new(JudgeCmd));

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

    // 链路命令 (Chain) — 端到端工作流编排
    reg.register(Box::new(ChainCmd));

    // 量子态测试选择 (Quantum Test) — 变更驱动坍缩, 避免全量测试
    reg.register(Box::new(QTestCmd));

    // 外部知识源 (Sources) — GitHub/书籍/arXiv/Wiki 定向爬取
    reg.register(Box::new(SourcesCmd));
    // 外部探索 (Explore) — URL/GitHub 仓库吸收 + 蒸馏
    reg.register(Box::new(ExploreCmd));

    // 快照: 全部命令名 + 别名 (剥离前导 '/'), 供 /completions 动态生成
    {
        let mut names: Vec<String> = Vec::new();
        for name in reg.list() {
            let trimmed = name.trim_start_matches('/');
            if !names.iter().any(|n| n == trimmed) {
                names.push(trimmed.to_string());
            }
        }
        for name in reg.list() {
            if let Some(cmd) = reg.get(name) {
                for alias in cmd.aliases() {
                    let trimmed = alias.trim_start_matches('/');
                    if !names.iter().any(|n| n == trimmed) {
                        names.push(trimmed.to_string());
                    }
                }
            }
        }
        if let Ok(mut pool) = completions_pool.lock() {
            *pool = names;
        }
    }

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
        assert!(names.contains(&"/catalog"), "catalog should be registered");
        assert!(names.contains(&"/chain"), "chain should be registered");
        assert!(names.len() >= 38, "got {} commands (expected 38+, got {})", names.len(), names.len());
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
    fn test_default_registry_find_and_execute_model() {
        let reg = default_registry();
        // /model must be discoverable via find (by name and alias)
        assert!(reg.find("/model").is_some());
        assert!(reg.find("/provider").is_some());
        assert!(reg.find("/llm").is_some());
        // execute returns a CommandOutput (not a panic / not an unknown-command error)
        let out = reg.execute("/model", None);
        assert!(out.message.contains("model") || out.message.contains("Model")
            || out.message.contains("Usage") || out.message.contains("provider"),
            "unexpected /model output: {}", out.message);
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
        assert!(names.contains(&"/explore"), "explore now registered as interactive CLI command (外部知识探索+alphaXiv)");
        assert!(!names.contains(&"/cleanup"), "cleanup should be auto-backend");
        assert!(!names.contains(&"/automation"), "automation should be auto-backend");
    }

    #[test]
    fn test_command_consolidation_primary_filter() {
        // 命令面精简: 人类只用基础控制命令; 聚合器与领域命令由 agent 后端自我调度
        let reg = default_registry();
        // 聚合器存在 (可被 agent 调度, 但不在人类一级面)
        for agg in ["/file", "/crypto", "/layout", "/vc", "/session-all", "/agent-all", "/memory"] {
            assert!(reg.find(agg).is_some(), "聚合器 {} 应已注册", agg);
        }
        // 独立命令仍可执行 (find 兼容)
        for sub in ["/read", "/write", "/wallet", "/swap", "/kb", "/wiki", "/search", "/evidence",
                    "/board", "/provider", "/model", "/free", "/git", "/commit", "/session",
                    "/resume", "/history", "/context", "/compact", "/distill", "/discover", "/mcp",
                    "/background", "/side", "/route", "/vim", "/workspace", "/theme", "/cost", "/budget"] {
            assert!(reg.find(sub).is_some(), "独立命令 {} 应仍可执行 (兼容)", sub);
        }
        // 人类一级入口 (help/补全) 只含控制命令白名单, 不含聚合器与领域命令
        let primary: Vec<&str> = reg.list_primary();
        for hidden in ["/read", "/write", "/wallet", "/swap", "/kb", "/wiki", "/search", "/evidence",
                       "/board", "/provider", "/model", "/free", "/git", "/commit", "/session",
                       "/resume", "/history", "/context", "/compact", "/distill", "/discover", "/mcp",
                       "/background", "/side", "/route", "/vim", "/workspace", "/theme", "/cost", "/budget",
                       "/file", "/crypto", "/layout", "/vc", "/session-all", "/agent-all", "/memory"] {
            assert!(!primary.contains(&hidden), "{} 是 agent 工具, 不应出现在人类一级入口", hidden);
        }
        // 控制命令白名单应在人类一级入口 (命令面精简 12→7)
        for ctl in ["/help", "/exit", "/clear", "/config", "/stats", "/e8", "/plan"] {
            assert!(primary.contains(&ctl), "控制命令 {} 应在人类一级入口", ctl);
        }
        // 降级命令: 功能合并进 /stats 与 /e8, 仍是 agent 工具但不在人类一级面
        for dg in ["/version", "/doctor", "/completions", "/benchmark", "/consciousness"] {
            assert!(!primary.contains(&dg), "降级命令 {} 不应在人类一级入口", dg);
        }
        // 合并验证: /stats 接收 version|doctor, /e8 接收 consciousness 子命令
        assert_eq!(reg.execute("/stats version", None).success, true, "/stats version 应可执行");
        assert_eq!(reg.execute("/stats doctor", None).success, true, "/stats doctor 应可执行");
        // /e8 consciousness 委派到独立 ConsciousnessCmd: 查询全局意识核心状态,
        // 不依赖 brain 实例 (意识核心是进程级全局), 故无 brain 也应成功
        let cns = reg.execute("/e8 consciousness", None);
        assert_eq!(cns.success, true, "/e8 consciousness 应可查询意识状态 (全局意识核心), got: {}", cns.message);
        // complete 同样收敛: 被隐藏的命令不参与补全, 控制命令参与
        assert!(reg.complete("/read").is_empty(), "/read 是 agent 工具, 不应参与人类补全");
        assert!(reg.complete("/help").iter().any(|c| c == "/help"), "/help 控制命令应参与补全");
    }

    #[test]
    fn test_memory_aggregator_delegates() {
        // /memory 聚合器子命令委派验证
        let reg = default_registry();
        let out = reg.execute("/memory", None);
        assert!(out.message.contains("evidence") && out.message.contains("wiki"),
            "/memory 无参应列出子命令, got: {}", out.message);
        // 未知子命令 → err
        let bad = reg.execute("/memory nonexistent_xyz", None);
        assert!(!bad.success && bad.message.contains("未知子命令"),
            "未知子命令应报错, got: {}", bad.message);
    }

    #[test]
    fn test_session_aggregator_covers_distill() {
        let reg = default_registry();
        let out = reg.execute("/session-all", None);
        assert!(out.message.contains("distill"), "/session-all 应覆盖 distill 子命令");
    }

    #[test]
    fn test_layout_aggregator_covers_route() {
        let reg = default_registry();
        let out = reg.execute("/layout", None);
        assert!(out.message.contains("route"), "/layout 应覆盖 route 子命令");
    }

    #[test]
    fn test_agent_aggregator_covers_acp() {
        let reg = default_registry();
        let out = reg.execute("/agent-all", None);
        assert!(out.message.contains("acp"), "/agent-all 应覆盖 acp 子命令");
    }
}

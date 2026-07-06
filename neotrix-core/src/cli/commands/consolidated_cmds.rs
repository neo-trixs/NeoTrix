//! 智能聚合命令 — /file, /crypto, /layout, /vc, /session-all, /agent-all
//! 统一子命令入口，后端自调用命令已从 CLI 移除（absorb/evolve/mem/save/trace/avatar/skills/explore/cleanup/automation）

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

macro_rules! delegate {
    ($name:expr, $args:expr, $brain:expr) => {{
        let reg = crate::cli::commands::registry::default_registry();
        let input = format!("{} {}", $name, $args.join(" "));
        reg.execute(&input, $brain)
    }};
}

// ====== /file ======

pub struct FileCmd;
impl CliCommand for FileCmd {
    fn name(&self) -> &str { "/file" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "文件操作: /file read|write|create|edit|patch|diff <args>" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("文件操作:\n  /file read <path>       读取文件\n  /file write <path> <c>  写入文件\n  /file create <path>     创建文件\n  /file edit <path> <e>   编辑文件\n  /file patch <path> <p>  应用补丁\n  /file diff <a> <b>      文件差异");
        }
        let sub = args[0].as_str();
        let rest: Vec<String> = args[1..].to_vec();
        match sub {
            "read" => delegate!("/read", &rest, brain),
            "write" => delegate!("/write", &rest, brain),
            "create" => delegate!("/create", &rest, brain),
            "edit" => delegate!("/edit", &rest, brain),
            "patch" => delegate!("/patch", &rest, brain),
            "diff" => delegate!("/diff", &rest, brain),
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: read, write, create, edit, patch, diff", sub)),
        }
    }
}

// ====== /crypto ======

pub struct WalletAggCmd;
impl CliCommand for WalletAggCmd {
    fn name(&self) -> &str { "/crypto" }
    fn aliases(&self) -> Vec<&str> { vec!["/finance"] }
    fn description(&self) -> &str { "加密金融: /crypto wallet|swap|transfer|approve|cost|budget <sub>" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("加密金融:\n  /crypto wallet <sub>      钱包管理\n  /crypto swap <sub>        DEX 交换\n  /crypto transfer <sub>    转账\n  /crypto approve <sub>     Token 授权\n  /crypto cost [detail|budget|reset]  费用追踪\n  /crypto budget <sub>      预算管理");
        }
        let sub = args[0].as_str();
        let rest: Vec<String> = args[1..].to_vec();
        match sub {
            "wallet" => delegate!("/wallet", &rest, brain),
            "swap" => delegate!("/swap", &rest, brain),
            "transfer" => delegate!("/transfer", &rest, brain),
            "approve" => delegate!("/approve", &rest, brain),
            "cost" => delegate!("/cost", &rest, brain),
            "budget" => delegate!("/budget", &rest, brain),
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: wallet, swap, transfer, approve, cost, budget", sub)),
        }
    }
}

// ====== /layout ======

pub struct UiAggCmd;
impl CliCommand for UiAggCmd {
    fn name(&self) -> &str { "/layout" }
    fn aliases(&self) -> Vec<&str> { vec!["/display"] }
    fn description(&self) -> &str { "界面布局: /layout background|side|router|vim|workspace|theme <sub>" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("界面布局:\n  /layout background <sub>  后台任务\n  /layout side <question>   侧边提问\n  /layout router            路由状态\n  /layout vim               Vim 模式\n  /layout workspace <sub>   工作区\n  /layout theme <name>      主题切换");
        }
        let sub = args[0].as_str();
        let rest: Vec<String> = args[1..].to_vec();
        match sub {
            "background" | "bg" => delegate!("/background", &rest, brain),
            "side" => delegate!("/side", &rest, brain),
            "router" => delegate!("/router", &rest, brain),
            "vim" => delegate!("/vim", &rest, brain),
            "workspace" => delegate!("/workspace", &rest, brain),
            "theme" => delegate!("/theme", &rest, brain),
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: background, side, router, vim, workspace, theme", sub)),
        }
    }
}

// ====== /vc (version control) ======

pub struct GitAggCmd;
impl CliCommand for GitAggCmd {
    fn name(&self) -> &str { "/vc" }
    fn aliases(&self) -> Vec<&str> { vec!["/vcs"] }
    fn description(&self) -> &str { "版本控制: /vc git|commit|pr <sub>" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("版本控制:\n  /vc git <sub>   Git 操作\n  /vc commit      提交\n  /vc pr          Pull Request");
        }
        let sub = args[0].as_str();
        let rest: Vec<String> = args[1..].to_vec();
        match sub {
            "git" => delegate!("/git", &rest, brain),
            "commit" => delegate!("/commit", &rest, brain),
            "pr" => delegate!("/pr", &rest, brain),
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: git, commit, pr", sub)),
        }
    }
}

// ====== /session-all ======

pub struct SessionAggCmd;
impl CliCommand for SessionAggCmd {
    fn name(&self) -> &str { "/session-all" }
    fn aliases(&self) -> Vec<&str> { vec!["/sess"] }
    fn description(&self) -> &str { "会话管理: /session-all session|resume|fork|history|context|compact <sub>" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("会话管理:\n  /session-all session <sub>    会话管理\n  /session-all resume <id>      恢复会话\n  /session-all fork             分支会话\n  /session-all history           历史\n  /session-all context <sub>     上下文管理\n  /session-all compact [now]     压缩");
        }
        let sub = args[0].as_str();
        let rest: Vec<String> = args[1..].to_vec();
        match sub {
            "session" => delegate!("/session", &rest, brain),
            "resume" => delegate!("/resume", &rest, brain),
            "fork" => delegate!("/fork", &rest, brain),
            "history" => delegate!("/history", &rest, brain),
            "context" | "ctx" => delegate!("/context", &rest, brain),
            "compact" => delegate!("/compact", &rest, brain),
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: session, resume, fork, history, context, compact", sub)),
        }
    }
}

// ====== /agent-all ======

pub struct ConsolidatedAgentCmd;
impl CliCommand for ConsolidatedAgentCmd {
    fn name(&self) -> &str { "/agent-all" }
    fn aliases(&self) -> Vec<&str> { vec!["/agents-all"] }
    fn description(&self) -> &str { "子代理: /agent-all spawn|list|talk|kill|status|background|tasks|discover|mcp" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("子代理:\n  /agent-all spawn|list|talk|kill|status|background|tasks\n  /agent-all discover [--port] [--duration]\n  /agent-all mcp list|status|discover|search|publish");
        }
        let sub = args[0].as_str();
        let rest: Vec<String> = args[1..].to_vec();
        match sub {
            "spawn" | "list" | "talk" | "kill" | "status" | "background" | "tasks" | "ls" | "bg" | "bglist" =>
                delegate!("/agent", args, brain),
            "discover" | "scan" => delegate!("/discover", &rest, brain),
            "mcp" => delegate!("/mcp", &rest, brain),
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: spawn, list, talk, kill, status, background, tasks, discover, mcp", sub)),
        }
    }
}

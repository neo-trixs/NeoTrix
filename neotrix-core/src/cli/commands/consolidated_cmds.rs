//! 智能聚合命令 — /file, /crypto, /layout, /vc, /session-all, /agent-all
//! 统一子命令入口，后端自调用命令已从 CLI 移除（absorb/evolve/mem/save/trace/avatar/skills/explore/cleanup/automation）

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

macro_rules! delegate {
    ($name:expr, $args:expr, $brain:expr) => {{
        let reg = crate::cli::commands::registry::default_registry();
        match reg.find($name) {
            // Call the target command instance directly, bypassing
            // registry.execute's full dispatch (sandbox/shield/hook re-entry)
            // to avoid the self-referential loop back into this aggregator.
            Some(cmd) => cmd.execute($args, $brain),
            None => CommandOutput::not_found(&format!("Unknown command: {}", $name)),
        }
    }};
}

// ====== /file ======

pub struct FileCmd;
impl CliCommand for FileCmd {
    fn name(&self) -> &str { "/file" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "File Operations: /file read|write|create|edit|patch|diff|consolidate <args>" }
    fn is_primary(&self) -> bool { false }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("文件操作:\n  /file read <path>       读取文件\n  /file write <path> <c>  写入文件\n  /file create <path>     创建文件\n  /file edit <path> <e>   编辑文件\n  /file patch <path> <p>  应用补丁\n  /file diff <a> <b>      文件差异\n  /file consolidate <dir> [out] 合并目录内 xlsx/csv/tsv 表格");
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
            "consolidate" => {
                if rest.is_empty() {
                    return CommandOutput::err("用法: /file consolidate <目录> [输出路径]");
                }
                let src = std::path::PathBuf::from(&rest[0]);
                let out = rest
                    .get(1)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| src.join("native_consolidated.xlsx"));
                match crate::neotrix::consolidate_tables(&src, &out) {
                    Ok(rep) => CommandOutput::ok(&format!(
                        "合并完成: 处理 {} 个文件 / {} 行 / {} 行含 USD 报价\n输出: {}",
                        rep.files_processed,
                        rep.total_rows,
                        rep.usd_rows,
                        out.display()
                    )),
                    Err(e) => CommandOutput::err(&format!("合并失败: {e}")),
                }
            }
            "suggest" => {
                if rest.is_empty() {
                    return CommandOutput::err("用法: /file suggest <目录> — 生成 schema 初稿 (确定性表头匹配 + LLM 增强可选)");
                }
                let src = std::path::PathBuf::from(&rest[0]);
                match crate::neotrix::suggest_schema(&src, None) {
                    Ok(s) => {
                        let mut out = format!(
                            "Schema 初稿 (LLM 增强: {})\n观察表头 {} 个 / 命中 {} 标准列 / 未命中 {} 个:\n",
                            s.llm_enhanced,
                            s.observed_headers.len(),
                            s.matched.len(),
                            s.unmatched.len()
                        );
                        for (std, headers) in &s.matched {
                            out.push_str(&format!("  ✓ {std} ← {}\n", headers.join(", ")));
                        }
                        if !s.unmatched.is_empty() {
                            out.push_str("  ✗ 未命中 (需人工/LLM 归类): ");
                            out.push_str(&s.unmatched.join(", "));
                            out.push('\n');
                        }
                        if !s.suggested_variants.is_empty() {
                            out.push_str("LLM 建议变体 (待确认):\n");
                            for (std, variants) in &s.suggested_variants {
                                out.push_str(&format!("  {std} += {}\n", variants.join(", ")));
                            }
                        }
                        CommandOutput::ok(&out)
                    }
                    Err(e) => CommandOutput::err(&format!("schema 初稿生成失败: {e}")),
                }
            }
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: read, write, create, edit, patch, diff, consolidate, suggest", sub)),
        }
    }
}

// ====== /crypto ======

pub struct WalletAggCmd;
impl CliCommand for WalletAggCmd {
    fn name(&self) -> &str { "/crypto" }
    fn aliases(&self) -> Vec<&str> { vec!["/finance"] }
    fn description(&self) -> &str { "Crypto / Finance: /crypto wallet|swap|transfer|approve|cost|budget <sub>" }
    fn is_primary(&self) -> bool { false }
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
    fn description(&self) -> &str { "Interface & Layout: /layout background|side|router|vim|workspace|theme|route <sub>" }
    fn is_primary(&self) -> bool { false }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("界面布局:\n  /layout background <sub>  后台任务\n  /layout side <question>   侧边提问\n  /layout router            路由状态\n  /layout route <sub>       智能路由\n  /layout vim               Vim 模式\n  /layout workspace <sub>   工作区\n  /layout theme <name>      主题切换");
        }
        let sub = args[0].as_str();
        let rest: Vec<String> = args[1..].to_vec();
        match sub {
            "background" | "bg" => delegate!("/background", &rest, brain),
            "side" => delegate!("/side", &rest, brain),
            "router" | "route" => delegate!("/route", &rest, brain),
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
    fn description(&self) -> &str { "Version Control: /vc git|commit|pr <sub>" }
    fn is_primary(&self) -> bool { false }
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
    fn description(&self) -> &str { "Session Management: /session-all session|resume|fork|history|context|compact|distill <sub>" }
    fn is_primary(&self) -> bool { false }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("会话管理:\n  /session-all session <sub>    会话管理\n  /session-all resume <id>      恢复会话\n  /session-all fork             分支会话\n  /session-all history           历史\n  /session-all context <sub>     上下文管理\n  /session-all compact [now]     压缩\n  /session-all distill           经验蒸馏");
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
            "distill" => delegate!("/distill", &rest, brain),
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: session, resume, fork, history, context, compact, distill", sub)),
        }
    }
}

// ====== /agent-all ======

pub struct ConsolidatedAgentCmd;
impl CliCommand for ConsolidatedAgentCmd {
    fn name(&self) -> &str { "/agent-all" }
    fn aliases(&self) -> Vec<&str> { vec!["/agents-all"] }
    fn description(&self) -> &str { "Subagents: /agent-all spawn|list|talk|kill|status|background|tasks|discover|mcp|acp" }
    fn is_primary(&self) -> bool { false }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("子代理:\n  /agent-all spawn|list|talk|kill|status|background|tasks\n  /agent-all discover [--port] [--duration]\n  /agent-all mcp list|status|discover|search|publish\n  /agent-all acp <sub>   ACP (Agent Client Protocol) 会话");
        }
        let sub = args[0].as_str();
        let rest: Vec<String> = args[1..].to_vec();
        match sub {
            "spawn" | "list" | "talk" | "kill" | "status" | "background" | "tasks" | "ls" | "bg" | "bglist" =>
                delegate!("/agent", args, brain),
            "discover" | "scan" => delegate!("/discover", &rest, brain),
            "mcp" => delegate!("/mcp", &rest, brain),
            "acp" => delegate!("/acp", &rest, brain),
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: spawn, list, talk, kill, status, background, tasks, discover, mcp, acp", sub)),
        }
    }
}

// ====== /memory (聚合: /evidence /hypothesis /search /board /kb /wiki) ======

pub struct MemoryAggCmd;
impl CliCommand for MemoryAggCmd {
    fn name(&self) -> &str { "/memory" }
    fn aliases(&self) -> Vec<&str> { vec!["/mem-aggr", "/knowledge"] }
    fn description(&self) -> &str { "Knowledge & Memory: /memory evidence|hypothesis|search|board|kb|wiki <sub>" }
    fn is_primary(&self) -> bool { false }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("记忆知识库:\n  /memory evidence <sub>     证据管理\n  /memory hypothesis <sub>   假设管理\n  /memory search <q>         KB 检索\n  /memory board [sub]        看板任务\n  /memory kb [sub]           KB 管理\n  /memory wiki [sub]         知识库维基");
        }
        let sub = args[0].as_str();
        let rest: Vec<String> = args[1..].to_vec();
        match sub {
            "evidence" => delegate!("/evidence", &rest, brain),
            "hypothesis" | "hyp" => delegate!("/hypothesis", &rest, brain),
            "search" => delegate!("/search", &rest, brain),
            "board" | "kanban" => delegate!("/board", &rest, brain),
            "kb" => delegate!("/kb", &rest, brain),
            "wiki" => delegate!("/wiki", &rest, brain),
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: evidence, hypothesis, search, board, kb, wiki", sub)),
        }
    }
}

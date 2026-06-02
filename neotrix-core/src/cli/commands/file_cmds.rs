//! 文件操作命令 — Read / Write / Create / Edit / Patch / Diff

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::approval::{ActionType, global_approval};
use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

/// Check if action requires approval. If yes, submit and return blocking message.
fn check_file_approval(args: &[String], action: ActionType) -> Option<CommandOutput> {
    let bypass = args.iter().any(|a| a == "--yes" || a == "-y");
    if bypass {
        return None;
    }
    let engine = global_approval();
    let mut e = engine.lock().expect("global_approval lock");
    if e.require_approval(&action) {
        let pa = e.submit(action);
        Some(CommandOutput::warn(&format!(
            "⏳ 等待审批: {} — 使用 /approval approve {} 批准, 或添加 --yes 跳过审批",
            pa.description, pa.id
        )))
    } else {
        None
    }
}

// ====== /read ======

pub struct FileReadCmd;
impl CliCommand for FileReadCmd {
    fn name(&self) -> &str { "/read" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "读取并显示文件内容: /read <path>" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err("用法: /read <path>");
        }
        let path_str = args.join(" ");
        let path = Path::new(&path_str);
        if !path.exists() {
            return CommandOutput::not_found(&format!("文件不存在: {}", path_str));
        }
        if path.is_dir() {
            return CommandOutput::err(&format!("{} 是目录, 不是文件", path_str));
        }
        match fs::read_to_string(path) {
            Ok(contents) => {
                let line_count = contents.lines().count();
                let output = format!("📄 {} ({} 行, {} 字节):\n{}", path_str, line_count, contents.len(), contents);
                CommandOutput::ok(&output)
            }
            Err(e) => CommandOutput::err(&format!("读取失败: {}", e)),
        }
    }
}

// ====== /write ======

pub struct FileWriteCmd;
impl CliCommand for FileWriteCmd {
    fn name(&self) -> &str { "/write" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "写入文件: /write <path> <content> [--yes]" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err("用法:\n  /write <path> <content>  覆盖写入内容\n  /write --yes <path> <content>  跳过审批\n  /write <path>             在 TUI 外通过 stdin 写入");
        }
        let clean_args: Vec<String> = args.iter().filter(|a| *a != "--yes" && *a != "-y").cloned().collect();
        if clean_args.is_empty() {
            return CommandOutput::err("用法: /write <path> <content> [--yes]");
        }
        let path_str = clean_args[0].clone();
        let path = Path::new(&path_str);
        if path.exists() {
            return CommandOutput::warn(&format!("⚠️ 文件已存在: {}. 如需覆盖请先执行 /read 查看, 或 /create 创建新文件", path_str));
        }
        let content = if clean_args.len() > 1 { clean_args[1..].join(" ") } else { String::new() };
        let preview = if content.len() > 80 { format!("{}… ({} 字节)", &content[..77], content.len()) } else { content.clone() };
        if let Some(block) = check_file_approval(args, ActionType::FileWrite { path: path_str.clone(), content_preview: preview }) {
            return block;
        }
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match fs::write(path, &content) {
            Ok(()) => CommandOutput::ok(&format!("✅ 已写入 {} ({} 字节)", path_str, content.len())),
            Err(e) => CommandOutput::err(&format!("写入失败: {}", e)),
        }
    }
}

// ====== /create ======

pub struct FileCreateCmd;
impl CliCommand for FileCreateCmd {
    fn name(&self) -> &str { "/create" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "创建新文件: /create <path> [content] [--yes]" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err("用法: /create <path> [content] [--yes]");
        }
        let clean_args: Vec<String> = args.iter().filter(|a| *a != "--yes" && *a != "-y").cloned().collect();
        if clean_args.is_empty() {
            return CommandOutput::err("用法: /create <path> [content] [--yes]");
        }
        let path_str = clean_args[0].clone();
        let path = Path::new(&path_str);
        if path.exists() {
            return CommandOutput::err(&format!("文件已存在: {}", path_str));
        }
        if let Some(block) = check_file_approval(args, ActionType::FileCreate { path: path_str.clone() }) {
            return block;
        }
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = if clean_args.len() > 1 { clean_args[1..].join(" ") } else { String::new() };
        match fs::write(path, &content) {
            Ok(()) => CommandOutput::ok(&format!("✅ 已创建 {} ({} 字节)", path_str, content.len())),
            Err(e) => CommandOutput::err(&format!("创建失败: {}", e)),
        }
    }
}

// ====== /edit ======

pub struct FileEditCmd;
impl CliCommand for FileEditCmd {
    fn name(&self) -> &str { "/edit" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "编辑文件: /edit <path> [<行号>:<新内容>] [--yes]" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err("用法: /edit <path> [<行号>:<新内容>] [--yes]");
        }
        let clean_args: Vec<String> = args.iter().filter(|a| *a != "--yes" && *a != "-y").cloned().collect();
        if clean_args.is_empty() {
            return CommandOutput::err("用法: /edit <path> [<行号>:<新内容>] [--yes]");
        }
        let path_str = clean_args[0].clone();
        let path = Path::new(&path_str);
        let orig = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => return CommandOutput::err(&format!("读取失败: {}", e)),
        };
        let lines: Vec<&str> = orig.lines().collect();

        if clean_args.len() < 2 {
            let mut display = format!("📄 {} ({} 行):\n", path_str, lines.len());
            for (i, line) in lines.iter().enumerate() {
                if line.len() > 120 {
                    display.push_str(&format!("{:>4}: {}…\n", i + 1, &line[..117]));
                } else {
                    display.push_str(&format!("{:>4}: {}\n", i + 1, line));
                }
            }
            display.push_str("\n用法: /edit <path> <行号>:<新内容>");
            return CommandOutput::ok(&display);
        }

        let edit = clean_args[1..].join(" ");
        if let Some(colon) = edit.find(':') {
            let line_num: usize = match edit[..colon].trim().parse() {
                Ok(n) if n > 0 && n <= lines.len() => n,
                _ => return CommandOutput::err(&format!("无效行号 (1-{})", lines.len())),
            };
            let new_content = edit[colon + 1..].to_string();
            let old_line = &lines[line_num - 1];
            let diff = format!("L{}: {} → {}", line_num, old_line, new_content);
            if let Some(block) = check_file_approval(args, ActionType::FileEdit { path: path_str.clone(), diff }) {
                return block;
            }
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            new_lines[line_num - 1] = new_content;
            let result = new_lines.join("\n");
            match fs::write(path, &result) {
                Ok(()) => CommandOutput::ok(&format!("✅ 已更新第 {} 行", line_num)),
                Err(e) => CommandOutput::err(&format!("写入失败: {}", e)),
            }
        } else {
            CommandOutput::err("用法: /edit <path> <行号>:<新内容>")
        }
    }
}

// ====== /patch ======

pub struct FilePatchCmd;
impl CliCommand for FilePatchCmd {
    fn name(&self) -> &str { "/patch" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "应用 unified diff 补丁: /patch <path> (需要从 stdin 传入)" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err("用法: /patch <path> (从 stdin 读取 patch, 在 TUI 中不可用)");
        }
        CommandOutput::ok(&format!("📋 /patch 需要从 stdin 读取 patch 数据。请使用 /edit 进行行编辑。"))
    }
}

// ====== /diff ======

pub struct FileDiffCmd;
impl CliCommand for FileDiffCmd {
    fn name(&self) -> &str { "/diff" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "显示文件 git diff: /diff <path>" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err("用法: /diff <path>");
        }
        let path_str = args[0].clone();
        let output = Command::new("git").args(["diff", "--no-color", &path_str]).output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                if !stderr.is_empty() {
                    return CommandOutput::warn(&format!("git 错误: {}", stderr.trim()));
                }
                if stdout.is_empty() {
                    CommandOutput::ok(&format!("📄 {}: 无变更或不受 git 管理", path_str))
                } else {
                    CommandOutput::ok(&format!("📊 {} 的差异:\n{}", path_str, stdout))
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CommandOutput::err("git 未安装")
                } else {
                    CommandOutput::err(&format!("git 执行失败: {}", e))
                }
            }
        }
    }
}

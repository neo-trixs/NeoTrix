//! 智能聚合命令 — /file, /crypto, /layout, /vc, /session-all, /agent-all
//! 统一子命令入口，后端自调用命令已从 CLI 移除（absorb/evolve/mem/save/trace/avatar/skills/explore/cleanup/automation）

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::acp_cmds::AcpCmd;
use crate::cli::commands::file_cmds::{FileCreateCmd, FileEditCmd, FilePatchCmd};
use crate::cli::commands::git_cmds::PrCmd;
use crate::cli::commands::session_cmds::ForkCmd;
use crate::cli::commands::swap_cmd::ApproveCmd;
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
    fn description(&self) -> &str { "File Operations: /file read|write|create|edit|patch|diff|consolidate|schema|suggest|convert|extract|mergepdf|merge|editpdf <args>" }
    fn is_primary(&self) -> bool { false }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("文件操作:\n  /file read <path>       读取文件\n  /file write <path> <c>  写入文件\n  /file create <path>     创建文件\n  /file edit <path> <e>   编辑文件\n  /file patch <path> <p>  应用补丁\n  /file diff <a> <b>      文件差异\n  /file consolidate <dir> [out] 合并目录内 xlsx/csv/tsv 表格 [--schema <name>] [--sheet-mode first|preferred|all]\n  /file schema list|show <name>  列出/查看已注册领域 schema (SchemaStore)\n  /file suggest <dir> [--save <name>]  扫描表头生成 schema 初稿 (可固化)\n  /file convert <in> <out>   图像格式转换 (png/jpeg)\n  /file extract <dir>   目录级统一提取 (混合格式 → 文本/表格清单)\n  /file mergepdf <out> <in1> <in2> ...  结构级合并多个 PDF (页面按序拼接)\n  /file merge <out.docx|pptx> <in1> <in2> ...  结构级合并 Office 文档 (DOCX 段落 / PPTX 幻灯片)\n  /file editpdf <in> <out> <page> <find> [replace] [font.ttf] 编辑 PDF 文本 (span redact + 原位替换)\n  /file tables <in.pdf>   提取 PDF 表格网格 (Markdown 渲染)");
        }
        let sub = args[0].as_str();
        let rest: Vec<String> = args[1..].to_vec();
        match sub {
            "read" => delegate!("/read", &rest, brain),
            "write" => delegate!("/write", &rest, brain),
            "create" => FileCreateCmd.execute(&rest, brain),
            "edit" => FileEditCmd.execute(&rest, brain),
            "patch" => FilePatchCmd.execute(&rest, brain),
            "diff" => delegate!("/diff", &rest, brain),
            "tables" => {
                if rest.len() != 1 {
                    return CommandOutput::err("用法: /file tables <in.pdf>");
                }
                let src = std::path::PathBuf::from(&rest[0]);
                match crate::neotrix::extract_pdf_tables(&src) {
                    Ok(tables) => {
                        let mut out = format!("提取到 {} 张表格\n", tables.len());
                        for (page, cols, md) in tables {
                            out.push_str(&format!("--- 页 {page} ({cols} 列)\n{md}\n"));
                        }
                        CommandOutput::ok(out.trim_end())
                    }
                    Err(e) => CommandOutput::err(&format!("PDF 表格提取失败: {e}")),
                }
            }
            "consolidate" => {
                // 用法: /file consolidate <目录> [输出路径] [--schema <name>] [--sheet-mode first|preferred|all]
                let args: Vec<String> = rest
                    .iter()
                    .filter(|a| !a.starts_with("--"))
                    .cloned()
                    .collect();
                let schema_arg = rest.iter().position(|a| a == "--schema").and_then(|i| rest.get(i + 1).cloned());
                let mode_arg = rest.iter().position(|a| a == "--sheet-mode").and_then(|i| rest.get(i + 1).cloned());
                if args.is_empty() {
                    return CommandOutput::err(
                        "用法: /file consolidate <目录> [输出路径] [--schema <name>] [--sheet-mode first|preferred|all]",
                    );
                }
                let src = std::path::PathBuf::from(&args[0]);
                let out = args
                    .get(1)
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| src.join("native_consolidated.xlsx"));
                // schema: 默认 price_table; 支持外部 JSON schema (SchemaStore)
                let schema_name = schema_arg.as_deref().unwrap_or("price_table");
                let mut store = crate::neotrix::SchemaStore::new();
                let schema = match store.load(schema_name) {
                    Ok(s) => s,
                    Err(e) => return CommandOutput::err(&format!("schema '{schema_name}' 加载失败: {e}")),
                };
                // sheet-mode: 默认按 schema.preferred_sheets (向后兼容)
                let mode = match mode_arg.as_deref() {
                    Some("first") => crate::neotrix::SheetMode::FirstSheet,
                    Some("all") => crate::neotrix::SheetMode::AllSheets,
                    Some("preferred") => {
                        if schema.preferred_sheets.is_empty() {
                            crate::neotrix::SheetMode::AllSheets
                        } else {
                            crate::neotrix::SheetMode::Preferred(schema.preferred_sheets)
                        }
                    }
                    Some(other) => {
                        return CommandOutput::err(&format!(
                            "未知 --sheet-mode '{other}' (可用: first|preferred|all)"
                        ))
                    }
                    None => {
                        if schema.preferred_sheets.is_empty() {
                            crate::neotrix::SheetMode::AllSheets
                        } else {
                            crate::neotrix::SheetMode::Preferred(schema.preferred_sheets)
                        }
                    }
                };
                let mode_desc = match &mode {
                    crate::neotrix::SheetMode::FirstSheet => "首 sheet".to_string(),
                    crate::neotrix::SheetMode::Preferred(_) => "优先 sheet".to_string(),
                    crate::neotrix::SheetMode::AllSheets => "全部 sheet".to_string(),
                };
                let result =
                    crate::neotrix::merge_tables_with_mode(&schema, &src, &out, mode);
                match result {
                    Ok(rep) => CommandOutput::ok(&format!(
                        "合并完成 (schema={schema_name}, {mode_desc}): 处理 {} 个文件 / {} 行 / {} 行含值\n输出: {}",
                        rep.files_processed,
                        rep.total_rows,
                        rep.usd_rows,
                        out.display()
                    )),
                    Err(e) => CommandOutput::err(&format!("合并失败: {e}")),
                }
            }
            "suggest" => {
                // 用法: /file suggest <目录> [--save <name>] [--llm]
                if rest.is_empty() {
                    return CommandOutput::err("用法: /file suggest <目录> [--save <name>] [--llm]");
                }
                let src = std::path::PathBuf::from(&rest[0]);
                let save_name = rest.iter().position(|a| a == "--save").and_then(|i| rest.get(i + 1).cloned());
                let use_llm = rest.iter().any(|a| a == "--llm");
                let llm_fn: Option<&dyn Fn(&str) -> Option<String>> = if use_llm {
                    // LLM 增强可选: 当前无可用回调时降级为确定性
                    None
                } else {
                    None
                };
                match crate::neotrix::suggest_schema(&src, llm_fn) {
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
                        if let Some(name) = &save_name {
                            // --save 显式纳入 LLM 确认变体 (suggest 报告已展示; 固化即确认)
                            let json = s.draft_with_variants();
                            match json.validate_json() {
                                Err(e) => return CommandOutput::err(&format!("schema 草稿校验失败: {e}")),
                                Ok(()) => {}
                            }
                            let store = crate::neotrix::SchemaStore::new();
                            let dir = store.dir().to_path_buf();
                            let path = dir.join(format!("{name}.json"));
                            if let Err(e) = std::fs::create_dir_all(&dir) {
                                return CommandOutput::err(&format!("创建 schema 目录失败: {e}"));
                            }
                            match std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()) {
                                Ok(_) => out.push_str(&format!("\n已固化 schema '{name}' → {}\n用 `/file consolidate <目录> --schema {name}` 复用", path.display())),
                                Err(e) => return CommandOutput::err(&format!("schema 写入失败: {e}")),
                            }
                        }
                        CommandOutput::ok(&out)
                    }
                    Err(e) => CommandOutput::err(&format!("schema 初稿生成失败: {e}")),
                }
            }
            "schema" => {
                // 用法: /file schema list | /file schema show <name>
                if rest.is_empty() {
                    return CommandOutput::err("用法: /file schema list | /file schema show <name>");
                }
                let mut store = crate::neotrix::SchemaStore::new();
                match rest[0].as_str() {
                    "list" => {
                        let names = store.list();
                        let mut out = format!(
                            "已注册 schema ({}):\n  ~/.neotrix/schemas/ + 内置\n",
                            names.len()
                        );
                        for n in &names {
                            out.push_str(&format!("  - {n}\n"));
                        }
                        out.push_str("用 `/file consolidate <dir> --schema <name>` 复用");
                        CommandOutput::ok(&out)
                    }
                    "show" => {
                        let name = match rest.get(1) {
                            Some(n) => n,
                            None => return CommandOutput::err("用法: /file schema show <name>"),
                        };
                        match store.load(name) {
                            Ok(schema) => {
                                let mut out = format!(
                                    "Schema '{}' (标准列 {} 个):\n",
                                    schema.name,
                                    schema.standard_columns.len()
                                );
                                for (i, col) in schema.standard_columns.iter().enumerate() {
                                    out.push_str(&format!("  {}. {col}\n", i + 1));
                                }
                                if !schema.column_variants.is_empty() {
                                    out.push_str("列名变体:\n");
                                    for (std, vs) in schema.column_variants.iter() {
                                        out.push_str(&format!("  {std} ← {}\n", vs.join(", ")));
                                    }
                                }
                                if !schema.preferred_sheets.is_empty() {
                                    out.push_str(&format!(
                                        "优先 sheet: {}\n",
                                        schema.preferred_sheets.join(", ")
                                    ));
                                }
                                CommandOutput::ok(&out)
                            }
                            Err(e) => CommandOutput::err(&format!("schema '{name}' 加载失败: {e}")),
                        }
                    }
                    other => CommandOutput::err(&format!(
                        "未知 schema 子命令 '{other}' (可用: list|show)"
                    )),
                }
            }
            "convert" => {
                // 用法: /file convert <in.png> <out.jpg>  — 图像格式转换 (png/jpeg)
                if rest.len() != 2 {
                    return CommandOutput::err("用法: /file convert <in> <out>  (图像格式转换: png/jpeg)");
                }
                let src = std::path::PathBuf::from(&rest[0]);
                let out = std::path::PathBuf::from(&rest[1]);
                match crate::neotrix::FileAbility::open(&src) {
                    Ok(fa) => match fa.convert_image(&out) {
                        Ok(()) => CommandOutput::ok(&format!(
                            "图像已转换: {} → {}",
                            src.display(),
                            out.display()
                        )),
                        Err(e) => CommandOutput::err(&format!("图像转换失败: {e}")),
                    },
                    Err(e) => CommandOutput::err(&format!("打开源图像失败: {e}")),
                }
            }
            "extract" => {
                // 用法: /file extract <目录>  — 目录级统一提取 (混合格式 → 文本/表格清单)
                if rest.len() != 1 {
                    return CommandOutput::err("用法: /file extract <dir>  (目录级统一提取: office/文本/PDF→文本, xlsx/csv→表格, 图像→元数据)");
                }
                let dir = std::path::PathBuf::from(&rest[0]);
                match crate::neotrix::extract_dir(&dir) {
                    Ok(report) => {
                        let mut out = format!(
                            "目录统一提取: 成功 {} / 失败 {} / 总字符 {}\n",
                            report.succeeded, report.failed, report.total_chars
                        );
                        for e in &report.entries {
                            let status = match &e.error {
                                Some(err) => format!("ERR {err}"),
                                None => format!(
                                    "{} 字符",
                                    if e.text.is_empty() { 0 } else { e.text.chars().count() }
                                ),
                            };
                            out.push_str(&format!(
                                "  {} [{}] 行数{:?} {}\n",
                                e.path, e.kind, e.table_rows, status
                            ));
                        }
                        CommandOutput::ok(&out)
                    }
                    Err(e) => CommandOutput::err(&format!("目录提取失败: {e}")),
                }
            }
            "mergepdf" => {
                // 用法: /file mergepdf <out.pdf> <in1.pdf> <in2.pdf> ...
                if rest.len() < 3 {
                    return CommandOutput::err("用法: /file mergepdf <out.pdf> <in1.pdf> <in2.pdf> ...\n  (结构级合并: 页面按输入顺序拼接, 字体/资源随对象图迁移)");
                }
                let out = std::path::PathBuf::from(&rest[0]);
                let inputs: Vec<std::path::PathBuf> =
                    rest[1..].iter().map(std::path::PathBuf::from).collect();
                match crate::neotrix::merge_pdfs(&inputs) {
                    Ok(bytes) => {
                        let page_count = match lopdf::Document::load_mem(&bytes) {
                            Ok(doc) => doc.get_pages().len(),
                            Err(e) => {
                                return CommandOutput::err(&format!(
                                    "解析合并结果失败: {e}"
                                ))
                            }
                        };
                        if let Err(e) = std::fs::write(&out, &bytes) {
                            return CommandOutput::err(&format!("写出合并结果失败: {e}"));
                        }
                        CommandOutput::ok(&format!(
                            "已合并 {} 个 PDF → {} (共 {} 页)",
                            inputs.len(),
                            out.display(),
                            page_count
                        ))
                    }
                    Err(e) => CommandOutput::err(&format!("PDF 合并失败: {e}")),
                }
            }
            "merge" => {
                // 用法: /file merge <out.docx|pptx> <in1> <in2> ... — Office 结构级合并
                if rest.len() < 3 {
                    return CommandOutput::err("用法: /file merge <out.docx|pptx> <in1> <in2> ...\n  (DOCX 段落拼接 / PPTX 幻灯片追加, 结构级合并)");
                }
                let out = std::path::PathBuf::from(&rest[0]);
                let inputs: Vec<std::path::PathBuf> =
                    rest[1..].iter().map(std::path::PathBuf::from).collect();
                let ext = out
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let result = match ext.as_str() {
                    "docx" => crate::neotrix::merge_docx::merge_docx(&inputs, &out)
                        .map(|r| format!("已合并 {} 个 DOCX → {} (part {})", r.items, out.display(), r.parts)),
                    "pptx" => crate::neotrix::merge_docx::merge_pptx(&inputs, &out)
                        .map(|r| format!("已合并 {} 个 PPTX → {} (slide part {})", r.items, out.display(), r.parts)),
                    _ => Err(crate::neotrix::FileAbilityError::Other(format!(
                        "仅支持 .docx/.pptx 输出, 收到: {ext}"
                    ))),
                };
                match result {
                    Ok(msg) => CommandOutput::ok(&msg),
                    Err(e) => CommandOutput::err(&format!("Office 合并失败: {e}")),
                }
            }
            "editpdf" => {
                // 用法: /file editpdf <in.pdf> <out.pdf> <page> <find> [replace] [font.ttf]
                if rest.len() < 4 {
                    return CommandOutput::err("用法: /file editpdf <in.pdf> <out.pdf> <page> <find> [replace] [font.ttf]\n  替换文本省略 = 仅删除 (redact); 非 Latin-1 文本需提供 font.ttf");
                }
                let src = std::path::PathBuf::from(&rest[0]);
                let out = std::path::PathBuf::from(&rest[1]);
                let page: u32 = match rest[2].parse() {
                    Ok(p) => p,
                    Err(_) => return CommandOutput::err("页号必须为整数 (1-based)"),
                };
                let find = &rest[3];
                let replace = rest.get(4).cloned();
                let has_replace = replace.is_some();
                // 非 Latin-1 替换需真实字体: 显式 font.ttf 优先, 否则自动探测系统字体。
                let auto_needed = replace
                    .as_deref()
                    .is_some_and(|r| r.chars().any(|c| (c as u32) > 0xFF));
                let ttf_bytes = if let Some(p) = rest.get(5) {
                    match std::fs::read(p) {
                        Ok(b) => Some(b),
                        Err(e) => return CommandOutput::err(&format!("读取字体失败: {e}")),
                    }
                } else if auto_needed {
                    neotrix_types::core::file_parser::pdf::find_system_font_for_text(
                        replace.as_deref().unwrap_or_default(),
                    )
                } else {
                    None
                };
                if auto_needed && ttf_bytes.is_none() {
                    return CommandOutput::err(&format!(
                        "替换文本含非 Latin-1 字符但未找到支持的系统字体: {replace:?}。请显式提供 font.ttf"
                    ));
                }
                let edit = crate::neotrix::PdfEdit {
                    page,
                    find: find.clone(),
                    replace,
                };
                match crate::neotrix::edit_pdf(&src, &out, &[edit], ttf_bytes.as_deref()) {
                    Ok(_) => {
                        let mode = if has_replace { "替换" } else { "删除" };
                        CommandOutput::ok(&format!(
                            "PDF {mode}完成: 页 {page} 文本 {find:?}\n输出: {}",
                            out.display()
                        ))
                    }
                    Err(e) => CommandOutput::err(&format!("PDF 编辑失败: {e}")),
                }
            }
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: read, write, create, edit, patch, diff, consolidate, suggest, editpdf", sub)),
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
            "approve" => ApproveCmd.execute(&rest, brain),
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
            "pr" => PrCmd.execute(&rest, brain),
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
            "fork" => ForkCmd.execute(&rest, brain),
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
            "acp" => AcpCmd.execute(&rest, brain),
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

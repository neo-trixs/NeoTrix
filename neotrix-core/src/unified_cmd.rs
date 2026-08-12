//! 统一命令桥 (Unified Command Bridge)
//!
//! 融合 CLI 命令系统与 NoeCodex (Tauri) 命令系统的统一接口层:
//! - `CommandSpec` — 统一命令描述 (name/aliases/category/description/backend)
//! - `CommandBackend` — 命令归属: Cli(neotrix CLI 注册表) / Tauri(NoeCodex 后端)
//! - `unified_catalog()` — 全量统一命令目录 (CLI 侧动态 + Tauri 侧静态)
//!
//! 设计动机 (D26-D30 生产就绪):
//! - CLI 75 个命令与 NoeCodex 400+ 命令此前互不可达
//! - NoeCodex 前端只能 invoke Tauri 命令, 无法执行 /kb /board /goal 等 CLI 命令
//! - 本模块提供双向可寻址的命令描述, 由 Tauri 侧 `unified_cli_execute` 桥接执行

use serde::Serialize;

/// 命令归属后端
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandBackend {
    /// neotrix CLI 注册表 (CommandRegistry)
    Cli,
    /// NoeCodex Tauri 后端 (generate_handler)
    Tauri,
}

/// 统一命令描述
#[derive(Debug, Clone, Serialize)]
pub struct CommandSpec {
    /// 命令名 (含斜杠, 如 /kb)
    pub name: String,
    /// 别名 (含斜杠, 如 /b)
    pub aliases: Vec<String>,
    /// 分类 (中文标签)
    pub category: String,
    /// 命令说明
    pub description: String,
    /// 归属后端
    pub backend: CommandBackend,
    /// 是否支持 --json 结构化输出
    pub json_support: bool,
    /// 是否为内部命令 (不在用户可见 catalog 中, 仅供 /chain 等编排内部调用)
    #[serde(default)]
    pub internal: bool,
}

/// NoeCodex 侧静态命令目录: 后端命令名 → 功能描述
/// 与 src-tauri main.rs 的 generate_handler 列表对齐维护。
/// 此处是唯一静态清单; main.rs 注册仍由 Rust 编译器保证存在性。
pub fn tauri_catalog() -> Vec<CommandSpec> {
    use CommandBackend::Tauri;
    let mut v = Vec::new();
    let mut push = |name: &str, desc: &str| {
        v.push(CommandSpec {
            name: name.to_string(),
            aliases: Vec::new(),
            category: category_of(name).to_string(),
            description: desc.to_string(),
            backend: Tauri,
            json_support: true,
            internal: false,
        });
    };

    // ---- 会话 ----
    push("neocodex_create_session", "创建会话");
    push("neocodex_list_sessions", "列出会话");
    push("neocodex_get_session_messages", "读取会话消息");
    push("neocodex_switch_session", "切换会话");
    push("neocodex_rename_session", "重命名会话");
    push("neocodex_delete_session", "删除会话");
    push("neocodex_archive_session", "归档会话");
    push("neocodex_restore_session", "恢复归档会话");
    push("neocodex_list_archived", "列出归档会话");
    push("neocodex_compact_session", "压缩会话上下文");
    push("neocodex_export_session", "导出会话 (JSONL)");
    push("neocodex_checkpoint_list", "列出检查点");
    push("neocodex_checkpoint_restore", "恢复检查点");

    // ---- 主会话流 ----
    push("neocodex_send_message_stream", "流式发送消息 (SSE)");
    push("neocodex_stop_stream", "停止流式生成");
    push("neocodex_send_side_chat", "侧向对话 (Side Chat)");
    push("neocodex_get_side_chat", "读取侧向对话");
    push("neocodex_regenerate", "重新生成上一条回复");
    push("neocodex_edit_message", "编辑消息");
    push("neocodex_delete_message", "删除消息");
    push("neocodex_set_mode", "设置工作模式");
    push("neocodex_health_report", "健康报告");

    // ---- 项目 / 文件 ----
    push("project_list", "列出项目");
    push("project_create", "创建项目");
    push("detect_project", "探测项目类型");
    push("read_dir_recursive", "递归读取目录树");
    push("read_file", "读取文件");
    push("write_file", "写入文件");
    push("neocodex_search_files", "搜索文件");
    push("neocodex_get_diff", "获取 git diff");
    push("neocodex_apply_diff", "应用 diff");
    push("neocodex_git_status", "git 状态");
    push("neocodex_git_file_status", "文件 git 状态");
    push("neocodex_open_file", "在编辑器中打开文件");

    // ---- 推理 / 知识 ----
    push("agent_reason", "LLM 流式推理");
    push("brain_stats", "Brain 统计");
    push("absorb_source", "吸收知识源");
    push("search_knowledge", "语义搜索知识");
    push("kb_search", "KB FTS5 搜索");
    push("kb_get_node", "KB 节点详情");
    push("kb_get_related", "KB 关联节点");
    push("get_knowledge_graph", "知识图谱");
    push("get_knowledge_stats", "知识统计");

    // ---- 提供者 / MCP ----
    push("provider_status", "LLM 网关状态");
    push("save_provider_config", "保存 Provider 配置");
    push("test_provider", "测试 Provider");
    push("neocodex_mcp_register", "注册 MCP 服务器");
    push("neocodex_mcp_list", "列出 MCP 服务器");
    push("neocodex_mcp_tools", "列出 MCP 工具");

    // ---- 代理 / 自动化 ----
    push("cmd_agent_start", "启动 Agent");
    push("cmd_agent_stop", "停止 Agent");
    push("cmd_agent_status", "Agent 状态");
    // 自动化域高层入口 (对应 /chain 子命令, 内部 CRUD 隐藏)
    push("automation.workflow", "工作流编排: 生成/创建/运行/调度 (内部 19 个 CRUD 命令)");
    push("automation.routine", "例行任务编排: 生成/创建/运行/云同步 (内部 14 个 CRUD 命令)");
    push("automation.loop", "循环调度编排: 创建/启用/执行/统计 (内部 13 个 CRUD 命令)");
    push("automation.background", "后台任务编排: 创建/暂停/恢复/运行/日志 (内部 7 个 CRUD 命令)");
    push("automation.coordinator", "协调器编排: 生成/列表/更新/策略 (内部 6 个 CRUD 命令)");

    // ---- 审查 / 安全 ----
    push("cmd_diff_review", "静态审查 diff");
    push("security_scan_quick_check", "安全快速检查");
    push("gate_run_check", "提交门自检");
    push("undercover_status", "git 隐身状态");
    push("proxy_status", "代理池状态");

    // ---- 桌面 ----
    push("window_close", "关闭窗口");
    push("window_minimize", "最小化窗口");
    push("window_maximize", "最大化窗口");
    push("read_clipboard", "读取剪贴板");
    push("write_clipboard", "写入剪贴板");
    push("buddy_status", "AI 伙伴状态");
    push("memory_list", "记忆列表");
    push("insights_daily", "每日洞察");
    push("unified_session_list", "统一会话总览");

    v
}

/// 为静态 Tauri 目录提供粗粒度分类
fn category_of(name: &str) -> &'static str {
    if name.starts_with("neocodex_") {
        if name.contains("session") || name.contains("checkpoint") || name.contains("archive") {
            return "会话管理";
        }
        if name.contains("diff") || name.contains("file") || name.contains("project") || name.contains("git") {
            return "项目文件";
        }
        if name.contains("mcp") || name.contains("provider") {
            return "提供者/MCP";
        }
        if name.contains("stream") || name.contains("mode") || name.contains("health") {
            return "主会话";
        }
        return "NeoCodex";
    }
    if name.starts_with("kb_") || name.starts_with("get_knowledge") || name.contains("knowledge") {
        return "知识记忆";
    }
    if name.starts_with("proxy") {
        return "网络代理";
    }
    if name.starts_with("security") || name.starts_with("gate") || name.starts_with("undercover") {
        return "安全治理";
    }
    if name.contains("window") || name.contains("clipboard") {
        return "桌面";
    }
    if name.starts_with("automation.") {
        return "自动化";
    }
    if name.contains("agent") || name.contains("coordinator") || name.contains("background")
        || name.contains("routines") || name.contains("loop") || name.contains("workflow") {
        return "自动化";
    }
    if name.contains("memory") || name.contains("insight") {
        return "知识记忆";
    }
    if name.contains("buddy") || name.contains("dream") {
        return "AI 伙伴";
    }
    if name.contains("review") {
        return "审查";
    }
    "系统"
}

/// 从 CLI CommandRegistry 动态构建 CLI 侧统一目录
pub fn cli_catalog() -> Vec<CommandSpec> {
    use crate::cli::commands::types::category_for;
    let reg = crate::cli::commands::registry::default_registry();
    let mut specs = Vec::new();
    for name in reg.list_primary() {
        if let Some(cmd) = reg.get(name) {
            let aliases = cmd.aliases().into_iter().map(String::from).collect::<Vec<_>>();
            specs.push(CommandSpec {
                name: name.to_string(),
                aliases,
                category: category_for(name).label().to_string(),
                description: cmd.description().to_string(),
                backend: CommandBackend::Cli,
                json_support: true,
                internal: false,
            });
        }
    }
    specs
}

/// 全量统一命令目录 (CLI + Tauri)
pub fn unified_catalog() -> Vec<CommandSpec> {
    let mut all = cli_catalog();
    all.extend(tauri_catalog());
    all
}

/// 按后端过滤目录
pub fn catalog_by_backend(backend: CommandBackend) -> Vec<CommandSpec> {
    unified_catalog()
        .into_iter()
        .filter(|s| s.backend == backend)
        .collect()
}

/// 仅用户可见命令 (过滤 internal=true)
pub fn public_catalog() -> Vec<CommandSpec> {
    unified_catalog()
        .into_iter()
        .filter(|s| !s.internal)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_catalog_contains_core_commands() {
        let specs = cli_catalog();
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"/help"), "cli catalog missing /help");
        assert!(names.contains(&"/config"), "cli catalog missing /config");
        assert!(names.contains(&"/plan"), "cli catalog missing /plan");
        // 命令面精简: 聚合器与领域命令是 agent 工具, 不进人类目录
        assert!(!names.contains(&"/memory"), "cli catalog 不应含 agent 工具 /memory");
        assert!(!names.contains(&"/kb"), "cli catalog 不应含 agent 工具 /kb");
        assert!(!names.contains(&"/board"), "cli catalog 不应含 agent 工具 /board");
        assert!(!names.contains(&"/goal"), "cli catalog 不应含 agent 工具 /goal");
        // 控制命令白名单 (命令面精简 12→7: version/doctor→/stats, completions/benchmark 降级, consciousness→/e8)
        for ctl in ["/help", "/exit", "/clear", "/config", "/stats", "/e8", "/plan"] {
            assert!(names.contains(&ctl), "cli catalog missing 控制命令 {}", ctl);
        }
        assert!(names.len() >= 7, "cli catalog too small: {}", names.len());
    }

    #[test]
    fn test_tauri_catalog_nonempty() {
        let specs = tauri_catalog();
        // 5 automation entry points + other commands
        assert!(specs.len() >= 40, "tauri catalog too small: {}", specs.len());
        assert!(specs.iter().any(|s| s.name == "neocodex_create_session"));
        // Verify automation entry points exist
        assert!(specs.iter().any(|s| s.name == "automation.workflow"));
        assert!(specs.iter().any(|s| s.name == "automation.routine"));
        assert!(specs.iter().any(|s| s.name == "automation.loop"));
        assert!(specs.iter().any(|s| s.name == "automation.background"));
        assert!(specs.iter().any(|s| s.name == "automation.coordinator"));
    }

    #[test]
    fn test_unified_catalog_merges_both() {
        let all = unified_catalog();
        assert!(all.iter().any(|s| s.backend == CommandBackend::Cli));
        assert!(all.iter().any(|s| s.backend == CommandBackend::Tauri));
    }

    #[test]
    fn test_catalog_by_backend_filters() {
        let cli_only = catalog_by_backend(CommandBackend::Cli);
        assert!(cli_only.iter().all(|s| s.backend == CommandBackend::Cli));
        let tauri_only = catalog_by_backend(CommandBackend::Tauri);
        assert!(tauri_only.iter().all(|s| s.backend == CommandBackend::Tauri));
    }
}

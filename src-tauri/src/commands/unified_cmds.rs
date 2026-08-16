//! 统一命令桥 — NoeCodex (Tauri) 侧桥接层
//!
//! 让 NoeCodex 前端可以:
//! 1. `unified_command_catalog` — 获取 CLI + NoeCodex 全量统一命令目录
//! 2. `unified_cli_execute` — 真正执行 neotrix CLI 命令系统的命令 (/kb /board /goal ...)
//! 3. `unified_cli_list` — 列出 CLI 侧命令 (方便前端构建命令面板)
//! 4. `unified_full_catalog` — 全量目录 (CLI 动态 + Tauri 自动生成), 单一真源
//!
//! 设计对齐 openchamber "server 侧承载长跑逻辑, UI 只是客户端" 的拓扑:
//! CLI 命令注册表 (CommandRegistry) 作为命令执行的单一真源, NoeCodex UI 通过此桥消费。

use neotrix::unified_cmd::{CommandBackend, CommandSpec, catalog_by_backend, unified_catalog};
use neotrix::neotrix::nt_core_error::NeoTrixError;
use serde::Serialize;
use serde_json::json;

/// Tauri 侧全量命令目录 — 自动生成 (module/action 二维) + 异步命令补充
///
/// 从 `_catalog_table.rs` (gate_gen.py 自动生成) 展开为统一 `CommandSpec` 扁平目录,
/// 覆盖全部同步命令; 异步命令 (SSE/流式) 以模块入口呈现。
/// 这是 Tauri 侧 catalog 的单一真源, 替代 unified_cmd.rs 的静态手写清单 (D74 契约完整性)。
#[tauri::command]
pub fn unified_tauri_full_catalog() -> Result<Vec<CommandSpec>, NeoTrixError> {
    use neotrix::unified_cmd::CommandSpec as S;
    let entries: Vec<serde_json::Value> = include!("_catalog_table.rs");
    let mut specs: Vec<CommandSpec> = entries
        .iter()
        .filter_map(|e| {
            let module = e.get("module")?.as_str()?;
            let action = e.get("action")?.as_str()?;
            // 过滤自引用门面 (unified_invoke 本体) — unified_session 是真实功能保留
            if module == "unified" || module == "desktop" || module == "gate" {
                return None;
            }
            Some(S {
                name: action.to_string(),
                aliases: Vec::new(),
                category: category_of(action).to_string(),
                description: action_description(action),
                backend: CommandBackend::Tauri,
                json_support: true,
                internal: false,
            })
        })
        .collect();
    specs.extend(async_entry_points());
    // 去重 (同名字典序保留第一个, gate 优先) — D74 契约完整性
    let mut seen = std::collections::HashSet::new();
    specs.retain(|s| seen.insert(s.name.clone()));
    Ok(specs)
}

/// 异步/流式命令的模块入口 (SSE 流不经过同步 gate)
fn async_entry_points() -> Vec<CommandSpec> {
    use neotrix::unified_cmd::CommandSpec as S;
    let mut v = Vec::new();
    let mut push = |name: &str, desc: &str| {
        v.push(S {
            name: name.to_string(),
            aliases: Vec::new(),
            category: category_of(name).to_string(),
            description: desc.to_string(),
            backend: CommandBackend::Tauri,
            json_support: true,
            internal: false,
        });
    };
    push("neocodex_create_session", "创建会话");
    push("neocodex_list_sessions", "列出会话");
    push("neocodex_get_session_messages", "读取会话消息");
    push("neocodex_switch_session", "切换会话");
    push("neocodex_rename_session", "重命名会话");
    push("neocodex_delete_session", "删除会话");
    push("neocodex_archive_session", "归档会话");
    push("neocodex_restore_session", "恢复归档会话");
    push("neocodex_list_archived", "列出归档会话");
    push("neocodex_search_sessions", "搜索会话");
    push("neocodex_tag_session", "会话打标签");
    push("neocodex_untag_session", "会话取消标签");
    push("neocodex_set_mode", "设置工作模式");
    push("neocodex_set_provider", "设置 Provider");
    push("neocodex_provider_config", "Provider 配置");
    push("neocodex_delete_message", "删除消息");
    push("computer_keyboard_press", "键盘按键 (修饰键)");
    push("computer_keyboard_type", "键盘输入文本");
    push("computer_mouse_click", "鼠标点击 (左/右)");
    push("computer_mouse_move", "鼠标移动");
    push("cowork_list", "协同会话列表");
    push("cowork_get", "读取协同会话");
    push("cowork_start", "启动协同会话");
    push("cowork_stop", "停止协同会话");
    push("cowork_pause", "暂停协同会话");
    push("cowork_resume", "恢复协同会话");
    push("cowork_delete", "删除协同会话");
    push("cowork_actions", "协同会话动作");
    push("cowork_read_file", "协同读取文件");
    push("cowork_write_file", "协同写入文件");
    push("cowork_scan_files", "协同扫描文件");
    push("cowork_status", "协同会话状态");
    push("cowork_templates", "协同模板");
    push("cowork_list_deliverables", "协同交付物列表");
    push("read_file", "读取文件 (安全路径)");
    push("neocodex_send_message_stream", "流式发送消息 (SSE)");
    push("neocodex_stop_stream", "停止流式生成");
    push("neocodex_send_side_chat", "侧向对话 (SSE)");
    push("neocodex_get_side_chat", "读取侧向对话");
    push("neocodex_regenerate", "重新生成上一条回复");
    push("neocodex_edit_message", "编辑消息");
    push("neocodex_health_report", "健康报告");
    push("agent_reason", "LLM 流式推理");
    push("absorb_source", "吸收知识源");
    push("brain_stats", "Brain 统计");
    push("kb_search", "KB FTS5 搜索");
    push("kb_get_node", "KB 节点详情");
    push("kb_get_related", "KB 关联节点");
    push("get_knowledge_graph", "知识图谱");
    push("get_knowledge_stats", "知识统计");
    push("memory_list", "记忆列表");
    push("memory_search", "语义搜索记忆");
    push("memory_stats", "记忆统计");
    push("provider_status", "LLM 网关状态");
    push("save_provider_config", "保存 Provider 配置");
    push("test_provider", "测试 Provider");
    push("neocodex_mcp_register", "注册 MCP 服务器");
    push("neocodex_mcp_list", "列出 MCP 服务器");
    push("neocodex_mcp_tools", "列出 MCP 工具");
    push("neocodex_search_files", "搜索文件");
    push("neocodex_get_diff", "获取 git diff");
    push("neocodex_apply_diff", "应用 diff");
    push("neocodex_git_status", "git 状态");
    push("neocodex_git_commit", "git 提交");
    push("neocodex_git_push", "git 推送");
    push("neocodex_git_branch", "git 分支");
    push("neocodex_git_checkout", "git 切换分支");
    push("neocodex_git_staged_files", "git 暂存文件");
    push("neocodex_open_file", "在编辑器中打开文件");
    push("neocodex_project_tree", "项目文件树");
    push("neocodex_get_project", "当前项目信息");
    push("neocodex_init_project", "初始化项目");
    push("neocodex_set_project", "设置项目");
    push("neocodex_agent_status", "Agent 状态");
    push("cmd_agent_start", "启动 Agent");
    push("cmd_agent_stop", "停止 Agent");
    push("cmd_agent_status", "Agent 状态");
    push("neocodex_check_update", "检查更新");
    push("neocodex_download_update", "下载更新");
    push("neocodex_restart_app", "重启应用");
    push("neocodex_app_version", "应用版本");
    push("neocodex_checkpoint_list", "列出检查点");
    push("neocodex_checkpoint_restore", "恢复检查点");
    push("neocodex_compact_session", "压缩会话上下文");
    push("neocodex_export_session", "导出会话 (JSONL)");
    push("neocodex_clear_session", "清空会话");
    push("neocodex_feedback", "反馈");
    push("neocodex_open_external", "打开外部链接");
    push("neocodex_file_operation", "文件操作");
    push("plugin_list", "列出插件");
    push("plugin_install", "安装插件");
    push("plugin_uninstall", "卸载插件");
    push("plugin_enable", "启用插件");
    push("plugin_disable", "禁用插件");
    push("plugin_event_log", "插件事件日志");
    push("window_close", "关闭窗口");
    push("window_minimize", "最小化窗口");
    push("window_maximize", "最大化窗口");
    push("buddy_status", "AI 伙伴状态");
    push("computer_get_frontmost_app", "获取前台应用");
    push("computer_get_window_list", "窗口列表");
    push("computer_screen_list", "屏幕列表");
    push("computer_mouse_position", "鼠标位置");
    push("computer_screenshot_and_save", "截图");
    push("voice_get_transcription", "语音转文字");
    push("has_api_key", "是否已配置 API Key");
    push("save_api_key", "保存 API Key");
    push("delete_api_key", "删除 API Key");
    push("kb_geo_points", "地理点云");
    push("kb_geo_stats", "地理统计");
    push("kb_geo_layers", "地理图层");
    push("kb_geo_elevations", "地理高程");
    push("kb_geo_points_pack", "地理点云打包");
    push("kb_geo_offline_pack", "地理离线包");
    push("kb_trajectory_add", "轨迹添加");
    push("kb_trajectory_query", "轨迹查询");
    push("memory_clear", "清空记忆");
    push("memory_export", "导出记忆");
    push("memory_timeline", "记忆时间线");
    v
}

/// 异步命令入口的粗粒度分类
fn category_of(name: &str) -> &'static str {
    if name.contains("session") || name.contains("checkpoint") || name.contains("archive") || name.contains("chat") {
        return "会话管理";
    }
    if name.contains("diff") || name.contains("file") || name.contains("project") || name.contains("git") {
        return "项目文件";
    }
    if name.contains("mcp") || name.contains("provider") || name.contains("api_key") {
        return "提供者/MCP";
    }
    if name.contains("stream") || name.contains("mode") || name.contains("health") || name.contains("feedback") {
        return "主会话";
    }
    if name.contains("kb_") || name.contains("knowledge") || name.contains("memory") || name.contains("insight") {
        return "知识记忆";
    }
    if name.contains("geo_") || name.contains("trajectory") {
        return "地理空间";
    }
    if name.contains("plugin") {
        return "插件";
    }
    if name.contains("computer") || name.contains("mouse") || name.contains("window") || name.contains("clipboard") || name.contains("voice") {
        return "桌面操作";
    }
    if name.contains("agent") || name.contains("workflow") || name.contains("routine") || name.contains("loop")
        || name.contains("background") || name.contains("coordinator") {
        return "自动化";
    }
    if name.contains("buddy") || name.contains("dream") {
        return "AI 伙伴";
    }
    if name.contains("review") || name.contains("security") || name.contains("gate") {
        return "审查安全";
    }
    if name.contains("update") || name.contains("version") || name.contains("app") {
        return "应用";
    }
    "系统"
}

/// 动作名 → 人类可读描述 (自动生成目录无描述, 由动作名推导)
fn action_description(action: &str) -> String {
    let de = action.replace('_', " ");
    de
}

/// 统一命令目录 (CLI + Tauri 全量)
#[tauri::command]
pub fn unified_command_catalog() -> Result<Vec<CommandSpec>, NeoTrixError> {
    Ok(unified_catalog())
}

/// CLI 侧命令目录
#[tauri::command]
pub fn unified_cli_list() -> Result<Vec<CommandSpec>, NeoTrixError> {
    Ok(catalog_by_backend(CommandBackend::Cli))
}

/// Tauri 侧命令目录 (核心子集, 源自 unified_cmd.rs 静态清单)
#[tauri::command]
pub fn unified_tauri_list() -> Result<Vec<CommandSpec>, NeoTrixError> {
    Ok(catalog_by_backend(CommandBackend::Tauri))
}

/// 统一命令执行结果 (JSON 化 CommandOutput)
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedCommandResult {
    pub success: bool,
    pub message: String,
    pub exit_code: i32,
    pub json: Option<serde_json::Value>,
}

/// 执行一条 CLI 命令 (如 "/kb search xxx" 或 "/board list")
///
/// 通过 neotrix CLI CommandRegistry 真正执行, 而非 shell 转发。
/// 命令输出 (CommandOutput) 以 JSON 结构返回, 前端可直接渲染。
#[tauri::command]
pub fn unified_cli_execute(input: String) -> Result<UnifiedCommandResult, NeoTrixError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(NeoTrixError::Config("empty command input".into()));
    }
    // 非斜杠前缀的输入按未知命令处理 (与 CLI 一致)
    let normalized = if input.starts_with('/') { input.to_string() } else { format!("/{}", input) };

    let reg = neotrix::cli::commands::registry::default_registry();
    let out = reg.execute(&normalized, None);

    Ok(UnifiedCommandResult {
        success: out.success,
        message: out.message,
        exit_code: out.exit_code.to_i32(),
        json: out.json,
    })
}

/// 便捷: 查询单条 CLI 命令详情 (供前端命令面板搜索)
#[tauri::command]
pub fn unified_cli_lookup(name: String) -> Result<Option<CommandSpec>, NeoTrixError> {
    let lookup = if name.starts_with('/') { name.clone() } else { format!("/{}", name) };
    let specs = catalog_by_backend(CommandBackend::Cli);
    Ok(specs.into_iter().find(|s| {
        s.name == lookup || s.aliases.contains(&lookup)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_execute_help() {
        let r = unified_cli_execute("/help".into()).unwrap();
        assert!(r.success);
        assert!(r.message.contains("NeoTrix"), "help message: {}", r.message);
    }

    #[test]
    fn test_cli_execute_unknown() {
        let r = unified_cli_execute("/nonexistent-cmd".into()).unwrap();
        assert!(!r.success);
        assert!(r.message.contains("Unknown command"));
    }

    #[test]
    fn test_cli_execute_normalizes_no_slash() {
        let r = unified_cli_execute("help".into()).unwrap();
        assert!(r.success);
    }

    #[test]
    fn test_cli_execute_empty() {
        let r = unified_cli_execute("  ".into());
        assert!(r.is_err());
    }

    #[test]
    fn test_cli_lookup_finds() {
        let found = unified_cli_lookup("help".into()).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "/help");
    }

    #[test]
    fn test_cli_lookup_alias() {
        let found = unified_cli_lookup("/config".into()).unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_cli_lookup_miss() {
        let found = unified_cli_lookup("definitely-not-a-command".into()).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_catalog_has_both_backends() {
        let all = unified_catalog();
        assert!(all.iter().any(|s| s.backend == CommandBackend::Cli));
        assert!(all.iter().any(|s| s.backend == CommandBackend::Tauri));
    }

    #[test]
    fn test_full_catalog_covers_auto_generated_gate() {
        let full = unified_tauri_full_catalog().unwrap();
        assert!(full.len() >= 250, "full catalog too small: {}", full.len());
        // gate 生成的 flat 命令名 (前端可直接 invoke)
        assert!(full.iter().any(|s| s.name == "create_background_task"));
        assert!(full.iter().any(|s| s.name == "term_tabs_list"));
        // 自引用门面被过滤 (unified_cli_*/unified_command_catalog/unified_invoke 本体)
        assert!(!full.iter().any(|s| s.name == "unified_cli_execute"));
        assert!(!full.iter().any(|s| s.name == "unified_command_catalog"));
        assert!(!full.iter().any(|s| s.name == "unified_invoke"));
        // unified_session 是真实功能, 保留
        assert!(full.iter().any(|s| s.name == "unified_session_list"));
        // 异步入口存在
        assert!(full.iter().any(|s| s.name == "neocodex_send_message_stream"));
        assert!(full.iter().any(|s| s.name == "kb_search"));
    }

    #[test]
    fn test_full_catalog_covers_frontend_commands() {
        // 前端实际调用的核心命令必须能在 full catalog 中找到 (契约完整性 D74)
        let full = unified_tauri_full_catalog().unwrap();
        let names: Vec<&str> = full.iter().map(|s| s.name.as_str()).collect();
        for required in [
            "neocodex_create_session",
            "neocodex_list_sessions",
            "neocodex_send_message_stream",
            "neocodex_get_session_messages",
            "neocodex_git_status",
            "neocodex_search_files",
            "kb_search",
            "memory_search",
            "computer_mouse_click",
        ] {
            assert!(names.contains(&required), "full catalog missing frontend cmd {}", required);
        }
    }

    #[test]
    fn test_full_catalog_backends_consistent() {
        let full = unified_tauri_full_catalog().unwrap();
        assert!(full.iter().all(|s| s.backend == CommandBackend::Tauri));
    }
}

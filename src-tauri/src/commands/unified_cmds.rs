//! 统一命令桥 — NoeCodex (Tauri) 侧桥接层
//!
//! 让 NoeCodex 前端可以:
//! 1. `unified_command_catalog` — 获取 CLI + NoeCodex 全量统一命令目录
//! 2. `unified_cli_execute` — 真正执行 neotrix CLI 命令系统的命令 (/kb /board /goal ...)
//! 3. `unified_cli_list` — 列出 CLI 侧命令 (方便前端构建命令面板)
//!
//! 设计对齐 openchamber "server 侧承载长跑逻辑, UI 只是客户端" 的拓扑:
//! CLI 命令注册表 (CommandRegistry) 作为命令执行的单一真源, NoeCodex UI 通过此桥消费。

use neotrix::unified_cmd::{CommandBackend, CommandSpec, catalog_by_backend, unified_catalog};
use neotrix::neotrix::nt_core_error::NeoTrixError;
use serde::Serialize;

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

/// Tauri 侧命令目录
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
        let found = unified_cli_lookup("kb".into()).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "/kb");
    }

    #[test]
    fn test_cli_lookup_alias() {
        let found = unified_cli_lookup("/b".into()).unwrap();
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
}

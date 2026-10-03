#![deny(clippy::unwrap_used)]


use colored::Colorize;



mod clean;
mod desktop;
mod headless;
mod proxy_cmd;
mod standalone;
mod sysops;
mod todo;
mod wiki;
mod provider;
mod brain;
mod daemon_common;
mod exec;
mod status;
mod consciousness;
mod browse;
mod dialog;
mod update;
mod daemon;
mod daemon_evolution;
mod interactive;
mod sandbox_features;
mod config_keys;
mod wallet;
mod agent;
mod social;
pub(crate) use provider::*;
pub(crate) use brain::*;
pub(crate) use daemon_common::*;
pub(crate) use exec::*;
pub(crate) use status::*;
pub(crate) use consciousness::*;
pub(crate) use browse::*;
pub(crate) use dialog::{DialogCmd, run_dialog};
pub(crate) use update::*;
pub(crate) use daemon::*;
pub(crate) use daemon_evolution::*;
pub(crate) use interactive::*;
pub(crate) use sandbox_features::*;
pub(crate) use config_keys::*;
pub(crate) use wallet::*;
pub(crate) use agent::*;

pub use clean::run_clean;
pub use proxy_cmd::run_proxy_cmd;
pub use sysops::run_sysops;
pub use todo::run_todo;
pub use wiki::run_wiki;
pub use social::{
    run_social_auth_x, run_social_doctor, run_social_login, run_social_probe,
    run_social_rank, run_social_sites, run_social_status, run_social_weights,
};
fn success(msg: impl AsRef<str>) -> String {
    msg.as_ref().green().to_string()
}
fn warn(msg: impl AsRef<str>) -> String {
    msg.as_ref().yellow().to_string()
}
fn err(msg: impl AsRef<str>) -> String {
    msg.as_ref().red().to_string()
}
fn dim(msg: impl AsRef<str>) -> String {
    msg.as_ref().dimmed().to_string()
}
fn info(msg: impl AsRef<str>) -> String {
    msg.as_ref().cyan().to_string()
}

/// Create a tokio runtime for the entry layer. Runtime creation failure is
/// unrecoverable at process entry, so we log it and exit rather than unwrap.
fn tokio_runtime() -> tokio::runtime::Runtime {
    match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("{}: failed to create tokio runtime: {}", err("Error"), e);
            std::process::exit(1);
        }
    }
}












fn run_shell_direct(cmd: &str) -> Result<(i32, String, String), String> {
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| format!("shell 执行失败: {}", e))?;
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    Ok((code, stdout, stderr))
}

/// 运行 `git diff --no-color [path]`，返回 stdout（best-effort，失败返回错误信息）。
#[allow(dead_code)] // 保留工具函数：待调用方接线后启用
fn run_git_diff(path: Option<&str>) -> Result<String, String> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["diff", "--no-color"]);
    if let Some(p) = path {
        cmd.arg(p);
    }
    let out = cmd.output().map_err(|e| format!("git 执行失败: {}", e))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

#[cfg(test)]
mod t14b_team_card_tests {
    use super::team_role_to_card;

    #[test]
    fn team_role_maps_to_card() {
        let role = neotrix::agent::team::AgentRole {
            name: "planner".to_string(),
            role: "Task Planner".to_string(),
            goal: "Break down complex tasks".to_string(),
            backstory: "Strategic planner".to_string(),
            tools: vec!["reason".to_string()],
        };
        let card = team_role_to_card("planner", &role);
        assert_eq!(card.id, "planner");
        assert_eq!(card.name, "planner");
        assert_eq!(card.description, "Strategic planner");
        assert_eq!(card.tags, vec!["reason".to_string()]);
        assert_eq!(card.role.role_chain, vec!["Task Planner".to_string()]);
    }
}

#[cfg(test)]
mod nt_entry_tests;

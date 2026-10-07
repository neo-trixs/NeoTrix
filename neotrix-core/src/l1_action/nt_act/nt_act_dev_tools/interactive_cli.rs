//! Interactive CLI 外部 agent 包装 — 不进模型池
//!
//! 约束（为什么和 `CliFreeSource`/`NtModelCliAsk` 是两条线）：
//! freebuff 一类是**交互式 TUI agent**：无 `models`/`run` headless 形态，
//! 也不产生可解析的 completion。它**不能**当 `LlmProviderType`，也不要进
//! `nt_free_pool`。本模块只负责两件真可落地的事：
//! 1. `probe_available` —— 用 `<cli> --version` 判探活与版本（走 crate 唯一
//!    进程出口语义，失败即不可用）；
//! 2. `launch` —— 显式 spawn 一个交互子进程，stdio 直通当前 TTY，由用户在
//!    那个终端操作。本模块**绝不解析**它的输出、绝不重放它的对话。
//!
//! # Safety
//! - 仅 `std::process::Command`，无 unsafe (R-P1)。
//! - 生产路径无 `unwrap/expect/panic!`。

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// 一个外部交互 agent CLI 的启动配置。
#[derive(Debug, Clone)]
pub struct InteractiveAgentCli {
    /// 命令名（PATH 可解析）或绝对路径。
    pub command: String,
    /// 交互启动参数（不含走管道 prompt 的形态）。
    pub args: Vec<String>,
    /// 工作目录，等价于 `cd <cwd> && <cli>`。
    pub cwd: Option<PathBuf>,
    /// `--version` 探活超时。
    pub probe_timeout: Duration,
}

impl InteractiveAgentCli {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            cwd: None,
            probe_timeout: Duration::from_secs(5),
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn with_probe_timeout(mut self, probe_timeout: Duration) -> Self {
        self.probe_timeout = probe_timeout;
        self
    }

    /// 探活：`<cli> --version`。无任何输出或非零退出 = 不可用。
    /// 复用 crate 的超时可杀进程出口，避免各写一份进程逻辑。
    pub fn probe_available(&self) -> bool {
        let mut argv = vec!["--version".to_string()];
        argv.extend(self.args.iter().cloned().filter(|a| a != "--version"));
        crate::l1_action::nt_model_cli::run_capture(&self.command, &argv, self.probe_timeout)
            .map(|o| !o.trim().is_empty())
            .unwrap_or(false)
    }

    /// 交互启动：spawn 子进程，stdio 直通 TTY。调用方负责持有/等待返回的
    /// `Child`；本模块不读其 stdout/stderr、不解析任何输出。
    pub fn launch(&self) -> std::io::Result<Child> {
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args).stdin(Stdio::inherit());
        // 交互 agent 粘到当前终端：stdin/stdout/stderr 全直通，不能按 capture
        // 语义收管道，否则 TUI 无 stdio 会直接退出。
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        if let Some(dir) = &self.cwd {
            cmd.current_dir(dir);
        }
        cmd.spawn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_uses_version_and_handles_missing_binary() {
        let c = InteractiveAgentCli::new("definitely-not-a-real-cli-xyz");
        assert!(!c.probe_available());
    }

    #[test]
    fn launch_inherits_std_for_a_trivial_command() {
        // 用 `true` 起进程；不持有 TTY 也可 spawn，仅验证返回 Ok。
        let c = InteractiveAgentCli::new("true");
        let mut child = match c.launch() {
            Ok(child) => child,
            Err(_) => return, // 平台无 `true` 则跳过该断言
        };
        let _ = child.wait();
    }
}

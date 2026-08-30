#![forbid(unsafe_code)]
//! Local Sandbox — 对标 grok-0.18 Local Docker loopback + harness lite-engine
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SandboxState {
    #[default]
    Stopped,
    Running,
    Error,
}

/// 权限分级 — 吸收 airgorah 最小权限分离: 非特权控制器 + 提权 agent (polkit 提权一次)。
///
/// `Unprivileged` 控制器 (GUI/调度器) 默认运行; 仅特权操作经 `request_elevation` 升到
/// `Elevated` 后, 才允许 `run_privileged_op` 执行 (一次提权, 类比 polkit 单次认证)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeTier {
    #[default]
    Unprivileged,
    Elevated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub use_local_docker: bool,
    pub loopback_only: bool,
    pub bind_port: u16,
    /// 沙箱默认权限分级 — 强化最小权限边界 (R-P42 吸收强化现有节点)。
    #[serde(default)]
    pub privilege: PrivilegeTier,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self { use_local_docker: false, loopback_only: true, bind_port: 8787, privilege: PrivilegeTier::Unprivileged }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalSandbox {
    pub config: SandboxConfig,
    pub state: SandboxState,
    pub last_error: Option<String>,
    /// 运行时权限分级 — 控制器默认非特权, 提权 agent 升至 Elevated。
    #[serde(default)]
    pub privilege: PrivilegeTier,
}

impl LocalSandbox {
    pub fn with_config(config: SandboxConfig) -> Self {
        let privilege = config.privilege;
        Self { config, state: SandboxState::Stopped, last_error: None, privilege }
    }
    pub fn is_local(&self) -> bool {
        self.config.use_local_docker
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.config.loopback_only && self.config.bind_port == 0 {
            return Err("loopback sandbox requires valid port".into());
        }
        Ok(())
    }
    pub fn start(&mut self) -> Result<(), String> {
        self.validate()?;
        self.state = SandboxState::Running;
        Ok(())
    }
    pub fn stop(&mut self) {
        self.state = SandboxState::Stopped;
    }

    /// 提权: 类比 airgorah 的 polkit 单次认证 — 将 agent 升至 Elevated 以执行特权操作。
    /// 非交互环境直接标记 (真实实现应触发一次认证提示)。
    pub fn request_elevation(&mut self) -> Result<(), String> {
        self.privilege = PrivilegeTier::Elevated;
        Ok(())
    }

    /// 最小权限守门: 仅 Elevated agent 可执行特权操作, 否则失败闭环 (fail-closed)。
    /// 控制器 (Unprivileged) 必须先 `request_elevation` 才能委派特权任务给 agent。
    pub fn run_privileged_op<F, T>(&self, op: F) -> Result<T, String>
    where
        F: FnOnce() -> T,
    {
        if self.privilege != PrivilegeTier::Elevated {
            return Err("privileged op blocked: controller is unprivileged, request_elevation first".into());
        }
        Ok(op())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_not_local() {
        let s = LocalSandbox::default();
        assert!(!s.is_local());
        assert!(s.validate().is_ok());
    }
    #[test]
    fn start_stop() {
        let mut s = LocalSandbox::default();
        assert!(s.start().is_ok());
        assert_eq!(s.state, SandboxState::Running);
        s.stop();
        assert_eq!(s.state, SandboxState::Stopped);
    }

    #[test]
    fn default_sandbox_is_unprivileged() {
        let s = LocalSandbox::default();
        assert_eq!(s.privilege, PrivilegeTier::Unprivileged);
        assert_eq!(s.config.privilege, PrivilegeTier::Unprivileged);
    }

    #[test]
    fn privileged_op_requires_elevation() {
        let s = LocalSandbox::default();
        // 控制器未提权 → 特权操作被拒 (最小权限失败闭环)
        let denied = s.run_privileged_op(|| 42);
        assert!(denied.is_err());

        let mut elevated = LocalSandbox::default();
        elevated.request_elevation().unwrap();
        assert_eq!(elevated.privilege, PrivilegeTier::Elevated);
        let ok = elevated.run_privileged_op(|| 42);
        assert_eq!(ok, Ok(42));
    }

    #[test]
    fn with_config_carries_privilege_tier() {
        let cfg = SandboxConfig { privilege: PrivilegeTier::Elevated, ..Default::default() };
        let s = LocalSandbox::with_config(cfg);
        assert_eq!(s.privilege, PrivilegeTier::Elevated);
    }
}

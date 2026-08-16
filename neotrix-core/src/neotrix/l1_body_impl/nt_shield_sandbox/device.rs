// P24: DeviceSandbox (吸收 OpenMinis — 移动端 on-device agent)
// 设备内沙箱: iSH (iOS usermode 模拟) / PRoot (Android user-space chroot) / Chroot。
// 暴露工具注册 + 沙箱会话生命周期; SKILL.md 惰性加载归 nt_mind_skill_engine。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxEngine {
    Ish,
    Proot,
    Chroot,
}

impl SandboxEngine {
    pub fn label(&self) -> &'static str {
        match self {
            SandboxEngine::Ish => "iSH (Linux usermode emulation on iOS)",
            SandboxEngine::Proot => "PRoot (user-space chroot on Android)",
            SandboxEngine::Chroot => "native chroot",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceTool {
    Health,
    Calendar,
    Reminders,
    HomeKit,
    Browser,
    Shell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSpec {
    pub engine: SandboxEngine,
    pub tools: Vec<DeviceTool>,
    pub cert_installed: bool,
}

impl Default for SandboxSpec {
    fn default() -> Self {
        Self {
            engine: SandboxEngine::Proot,
            tools: vec![DeviceTool::Shell, DeviceTool::Browser],
            cert_installed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSession {
    pub session_id: String,
    pub spec: SandboxSpec,
    pub status: SandboxStatus,
    pub started_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxStatus {
    Provisioning,
    Ready,
    Crashed,
    Terminated,
}

pub struct DeviceSandbox {
    pub spec: SandboxSpec,
    sessions: HashMap<String, SandboxSession>,
    next_id: u64,
}

impl DeviceSandbox {
    pub fn new(spec: SandboxSpec) -> Self {
        Self {
            spec,
            sessions: HashMap::new(),
            next_id: 0,
        }
    }

    /// 安装 MITM/代理根证书 (res-downloader 语义: 抓包需先装证书)。
    pub fn install_cert(&mut self) {
        self.spec.cert_installed = true;
    }

    pub fn has_cert(&self) -> bool {
        self.spec.cert_installed
    }

    /// 暴露设备能力为工具面 (OpenMinis 设备集成 → tools)。
    pub fn expose_tool(&mut self, tool: DeviceTool) {
        if !self.spec.tools.contains(&tool) {
            self.spec.tools.push(tool);
        }
    }

    pub fn available_tools(&self) -> &[DeviceTool] {
        &self.spec.tools
    }

    pub fn provision(&mut self) -> Result<SandboxSession, String> {
        if self.spec.engine == SandboxEngine::Chroot && !self.has_cert() {
            // chroot 需要先信任证书才能启动加密隧道
            return Err("sandbox requires certificate trust before provisioning".into());
        }
        let id = format!("devbox-{}", self.next_id);
        self.next_id += 1;
        let session = SandboxSession {
            session_id: id.clone(),
            spec: self.spec.clone(),
            status: SandboxStatus::Ready,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };
        self.sessions.insert(id.clone(), session.clone());
        Ok(session)
    }

    pub fn session(&self, id: &str) -> Option<&SandboxSession> {
        self.sessions.get(id)
    }

    pub fn terminate(&mut self, id: &str) -> Result<(), String> {
        match self.sessions.get_mut(id) {
            Some(s) => {
                s.status = SandboxStatus::Terminated;
                Ok(())
            }
            None => Err(format!("session {id} not found")),
        }
    }

    pub fn active_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| s.status == SandboxStatus::Ready)
            .count()
    }
}

impl crate::core::nt_core_self_test::SelfTest for DeviceSandbox {
    fn name(&self) -> &str {
        "nt_shield_device_sandbox"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut boxed = DeviceSandbox::new(SandboxSpec::default());
        let session = boxed.provision().map_err(|e| vec![e])?;
        if session.status != SandboxStatus::Ready {
            return Err(vec!["session should be Ready".into()]);
        }
        if boxed.active_count() != 1 {
            return Err(vec!["expected 1 active session".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    #[test]
    fn test_provision_ready() {
        let mut b = DeviceSandbox::new(SandboxSpec::default());
        let s = b.provision().expect("provision");
        assert_eq!(s.status, SandboxStatus::Ready);
        assert_eq!(b.active_count(), 1);
    }

    #[test]
    fn test_chroot_requires_cert() {
        let mut b = DeviceSandbox::new(SandboxSpec {
            engine: SandboxEngine::Chroot,
            ..SandboxSpec::default()
        });
        assert!(b.provision().is_err());
        b.install_cert();
        assert!(b.provision().is_ok());
    }

    #[test]
    fn test_expose_tool_dedup() {
        let mut b = DeviceSandbox::new(SandboxSpec::default());
        b.expose_tool(DeviceTool::Health);
        b.expose_tool(DeviceTool::Health);
        assert_eq!(b.available_tools().iter().filter(|t| **t == DeviceTool::Health).count(), 1);
    }

    #[test]
    fn test_terminate() {
        let mut b = DeviceSandbox::new(SandboxSpec::default());
        let s = b.provision().expect("prov");
        b.terminate(&s.session_id).expect("term");
        assert_eq!(b.active_count(), 0);
        assert!(b.terminate("missing").is_err());
    }

    #[test]
    fn test_engine_labels() {
        assert!(SandboxEngine::Ish.label().contains("iSH"));
        assert!(SandboxEngine::Proot.label().contains("PRoot"));
    }

    #[test]
    fn test_selftest() {
        let b = DeviceSandbox::new(SandboxSpec::default());
        assert!(b.self_test().is_ok());
    }
}
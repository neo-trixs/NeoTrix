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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub use_local_docker: bool,
    pub loopback_only: bool,
    pub bind_port: u16,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self { use_local_docker: false, loopback_only: true, bind_port: 8787 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalSandbox {
    pub config: SandboxConfig,
    pub state: SandboxState,
    pub last_error: Option<String>,
}

impl LocalSandbox {
    pub fn with_config(config: SandboxConfig) -> Self {
        Self { config, state: SandboxState::Stopped, last_error: None }
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
}

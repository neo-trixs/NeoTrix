use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub mod cli;
pub mod docker;
pub mod provider;
pub mod remote;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudRuntime {
    Python3,
    Node18,
    RustStable,
    Go1_21,
    GenericLinux,
}

impl CloudRuntime {
    pub fn as_str(&self) -> &'static str {
        match self {
            CloudRuntime::Python3 => "python:3.11",
            CloudRuntime::Node18 => "node:18",
            CloudRuntime::RustStable => "rust:latest",
            CloudRuntime::Go1_21 => "golang:1.21",
            CloudRuntime::GenericLinux => "ubuntu:22.04",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "python3" | "python" | "py" => Some(CloudRuntime::Python3),
            "node18" | "node" | "js" => Some(CloudRuntime::Node18),
            "rust" | "ruststable" | "rs" => Some(CloudRuntime::RustStable),
            "go1.21" | "go" | "golang" => Some(CloudRuntime::Go1_21),
            "linux" | "generic" | "ubuntu" => Some(CloudRuntime::GenericLinux),
            _ => None,
        }
    }

    pub fn variants() -> &'static [CloudRuntime] {
        &[
            CloudRuntime::Python3,
            CloudRuntime::Node18,
            CloudRuntime::RustStable,
            CloudRuntime::Go1_21,
            CloudRuntime::GenericLinux,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudSessionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_time: f64,
    pub memory_mb: f64,
    pub network_kb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time: Duration,
    pub resource_usage: ResourceUsage,
}

pub struct CloudSession {
    pub session_id: String,
    pub status: CloudSessionStatus,
    pub runtime: CloudRuntime,
    provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync>,
}

impl CloudSession {
    pub fn new(
        session_id: String,
        runtime: CloudRuntime,
        provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync>,
    ) -> Self {
        Self {
            session_id,
            status: CloudSessionStatus::Pending,
            runtime,
            provider,
        }
    }

    pub async fn execute(
        &mut self,
        code: &str,
        runtime: CloudRuntime,
    ) -> Result<CloudResult, String> {
        self.status = CloudSessionStatus::Running;
        let result = self.provider.execute(&self.session_id, code, runtime).await?;
        self.status = match result.exit_code {
            0 => CloudSessionStatus::Completed,
            _ if result.execution_time >= Duration::from_secs(300) => CloudSessionStatus::TimedOut,
            _ => CloudSessionStatus::Failed,
        };
        Ok(result)
    }

    pub async fn upload_file(&mut self, path: &str, data: Vec<u8>) -> Result<(), String> {
        self.provider.upload_file(&self.session_id, path, data).await
    }

    pub async fn download_result(&self) -> Result<CloudResult, String> {
        self.provider.download_result(&self.session_id).await
    }

    pub fn stream_logs(&self) -> futures::stream::BoxStream<'static, String> {
        self.provider.stream_logs(&self.session_id)
    }

    pub async fn cancel(&mut self) -> Result<(), String> {
        self.provider.cancel(&self.session_id).await?;
        self.status = CloudSessionStatus::Failed;
        Ok(())
    }
}

pub struct CloudSandbox {
    pub cloud_endpoint: String,
    pub api_key: Option<String>,
    pub max_runtime: Duration,
    pub supported_runtimes: Vec<CloudRuntime>,
    sessions: Vec<CloudSession>,
    provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync>,
}

impl CloudSandbox {
    pub fn new(
        cloud_endpoint: String,
        api_key: Option<String>,
        max_runtime: Duration,
        provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync>,
    ) -> Self {
        Self {
            cloud_endpoint,
            api_key,
            max_runtime,
            supported_runtimes: CloudRuntime::variants().to_vec(),
            sessions: Vec::new(),
            provider,
        }
    }

    pub fn default_local() -> Self {
        let provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync> =
            if Self::docker_available() {
                Arc::new(docker::LocalDockerProvider::new())
            } else {
                Arc::new(provider::NoopProvider)
            };
        Self::new(
            "http://localhost".to_string(),
            None,
            Duration::from_secs(300),
            provider,
        )
    }

    fn docker_available() -> bool {
        std::process::Command::new("docker")
            .args(["info"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    const MAX_SESSIONS: usize = 100;

    pub fn create_session(&mut self, runtime: CloudRuntime) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = CloudSession::new(session_id.clone(), runtime, Arc::clone(&self.provider));
        if self.sessions.len() >= Self::MAX_SESSIONS {
            self.sessions.remove(0);
        }
        self.sessions.push(session);
        session_id
    }

    pub fn get_session(&self, session_id: &str) -> Option<&CloudSession> {
        self.sessions.iter().find(|s| s.session_id == session_id)
    }

    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut CloudSession> {
        self.sessions.iter_mut().find(|s| s.session_id == session_id)
    }

    pub fn list_sessions(&self) -> &[CloudSession] {
        &self.sessions
    }

    pub fn cancel_session(&mut self, session_id: &str) -> Result<(), String> {
        match self.get_session_mut(session_id) {
            Some(session) => {
                let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
                rt.block_on(session.cancel())
            }
            None => Err(format!("Session {} not found", session_id)),
        }
    }

    pub async fn run_code(
        &mut self,
        code: &str,
        runtime: CloudRuntime,
    ) -> Result<CloudResult, String> {
        let session_id = self.create_session(runtime);
        let session = self.get_session_mut(&session_id).ok_or("session creation failed")?;
        session.execute(code, runtime).await
    }

    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn noop_sandbox() -> CloudSandbox {
        let provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync> =
            Arc::new(provider::NoopProvider);
        CloudSandbox::new(
            "http://test".to_string(),
            None,
            Duration::from_secs(60),
            provider,
        )
    }

    #[test]
    fn test_cloud_runtime_from_str() {
        assert_eq!(CloudRuntime::from_str("python3"), Some(CloudRuntime::Python3));
        assert_eq!(CloudRuntime::from_str("python"), Some(CloudRuntime::Python3));
        assert_eq!(CloudRuntime::from_str("py"), Some(CloudRuntime::Python3));
        assert_eq!(CloudRuntime::from_str("node"), Some(CloudRuntime::Node18));
        assert_eq!(CloudRuntime::from_str("rust"), Some(CloudRuntime::RustStable));
        assert_eq!(CloudRuntime::from_str("rs"), Some(CloudRuntime::RustStable));
        assert_eq!(CloudRuntime::from_str("go"), Some(CloudRuntime::Go1_21));
        assert_eq!(CloudRuntime::from_str("linux"), Some(CloudRuntime::GenericLinux));
        assert_eq!(CloudRuntime::from_str("unknown"), None);
        assert_eq!(CloudRuntime::from_str(""), None);
    }

    #[test]
    fn test_cloud_runtime_as_str() {
        assert_eq!(CloudRuntime::Python3.as_str(), "python:3.11");
        assert_eq!(CloudRuntime::Node18.as_str(), "node:18");
        assert_eq!(CloudRuntime::RustStable.as_str(), "rust:latest");
        assert_eq!(CloudRuntime::Go1_21.as_str(), "golang:1.21");
        assert_eq!(CloudRuntime::GenericLinux.as_str(), "ubuntu:22.04");
    }

    #[test]
    fn test_cloud_runtime_variants() {
        let variants = CloudRuntime::variants();
        assert_eq!(variants.len(), 5);
        assert!(variants.contains(&CloudRuntime::Python3));
        assert!(variants.contains(&CloudRuntime::GenericLinux));
    }

    #[test]
    fn test_cloud_sandbox_creation() {
        let sandbox = noop_sandbox();
        assert_eq!(sandbox.cloud_endpoint, "http://test");
        assert!(sandbox.api_key.is_none());
        assert_eq!(sandbox.supported_runtimes.len(), 5);
        assert_eq!(sandbox.list_sessions().len(), 0);
        assert_eq!(sandbox.max_runtime, Duration::from_secs(60));
    }

    #[test]
    fn test_session_creation_and_retrieval() {
        let mut sandbox = noop_sandbox();
        let session_id = sandbox.create_session(CloudRuntime::Python3);
        assert!(!session_id.is_empty());
        assert_eq!(sandbox.list_sessions().len(), 1);
        assert!(sandbox.get_session(&session_id).is_some());
        assert_eq!(
            sandbox.get_session(&session_id).unwrap().runtime,
            CloudRuntime::Python3
        );
        assert_eq!(
            sandbox.get_session(&session_id).unwrap().status,
            CloudSessionStatus::Pending
        );
        assert!(sandbox.get_session("nonexistent").is_none());
    }

    #[test]
    fn test_session_cancel_noop() {
        let mut sandbox = noop_sandbox();
        let session_id = sandbox.create_session(CloudRuntime::Python3);
        let result = sandbox.cancel_session(&session_id);
        assert!(result.is_err(), "NoopProvider cancel should return Err");
        assert!(
            result.unwrap_err().contains("No sandbox provider"),
            "error should mention missing provider"
        );
    }

    #[test]
    fn test_max_sessions_enforcement() {
        let mut sandbox = noop_sandbox();
        for _ in 0..CloudSandbox::MAX_SESSIONS + 10 {
            sandbox.create_session(CloudRuntime::Python3);
        }
        assert!(sandbox.list_sessions().len() <= CloudSandbox::MAX_SESSIONS);
    }
}

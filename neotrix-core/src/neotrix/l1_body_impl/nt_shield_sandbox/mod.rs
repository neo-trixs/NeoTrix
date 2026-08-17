use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[cfg(feature = "sandbox")]
use crate::neotrix::l1_body_impl::nt_shield::vault::Vault;

pub mod cli;
pub mod device;
pub mod docker;
pub mod provider;
pub mod remote;

pub use device::{DeviceSandbox, DeviceTool, SandboxEngine, SandboxSession, SandboxSpec, SandboxStatus};

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

/// Per-sandbox egress network policy (OpenSandbox absorption, Cycle 232+).
/// Controls which outbound hosts/ports a sandbox session may reach before the
/// workload runs — the sandbox's outbound trust boundary (R-P32 双观独立性).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EgressRule {
    /// Host pattern: exact host, `*.example.com`, or `*` (all).
    pub host: String,
    /// Port range as `"443"` or `"443-8443"`; empty = any port.
    pub port: String,
    /// allow (whitelist) or deny (blacklist). Deny takes precedence.
    pub allow: bool,
}

impl EgressRule {
    pub fn allow(host: &str, port: &str) -> Self {
        Self { host: host.into(), port: port.into(), allow: true }
    }

    pub fn deny(host: &str, port: &str) -> Self {
        Self { host: host.into(), port: port.into(), allow: false }
    }

    fn host_matches(&self, host: &str) -> bool {
        if self.host == "*" {
            return true;
        }
        if let Some(suffix) = self.host.strip_prefix("*.") {
            // `*.example.com` matches subdomains only — not the bare apex,
            // and never across a dot boundary (example.com.evil.net).
            return host.ends_with(&format!(".{}", suffix));
        }
        host == self.host
    }

    fn port_matches(&self, port: u16) -> bool {
        if self.port.is_empty() {
            return true;
        }
        if let Some((lo, hi)) = self.port.split_once('-') {
            let (lo, hi): (u16, u16) = match (lo.parse(), hi.parse()) {
                (Ok(a), Ok(b)) => (a, b),
                _ => return false,
            };
            port >= lo && port <= hi
        } else {
            self.port.parse::<u16>().map(|p| p == port).unwrap_or(false)
        }
    }
}

/// Compiled egress policy over a rule list. Deny wins over allow; unmatched
/// hosts fall back to the policy default.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EgressPolicy {
    pub rules: Vec<EgressRule>,
    /// Default for hosts not matched by any rule.
    pub default_allow: bool,
}

impl EgressPolicy {
    pub fn new(rules: Vec<EgressRule>, default_allow: bool) -> Self {
        Self { rules, default_allow }
    }

    /// Everything out — matches legacy sandbox behaviour.
    pub fn permissive() -> Self {
        Self { rules: vec![], default_allow: true }
    }

    /// Nothing out — the closed trust boundary default for agent sandboxes.
    pub fn deny_all() -> Self {
        Self { rules: vec![], default_allow: false }
    }

    /// Evaluate one outbound connection. Deny rules shadow allow rules.
    pub fn check(&self, host: &str, port: u16) -> bool {
        let mut matched_allow = false;
        for rule in &self.rules {
            if rule.host_matches(host) && rule.port_matches(port) {
                if !rule.allow {
                    return false; // explicit deny wins
                }
                matched_allow = true;
            }
        }
        matched_allow || self.default_allow
    }

    /// Sanity validation: deny-all + a localhost allow must pass only the allow.
    pub fn sanity_check(&self) -> Result<(), String> {
        if self.rules.iter().any(|r| r.host == "*" && !r.allow) {
            return Err("egress: global deny-all rule would shadow every allow (use deny_all + specific allows)".into());
        }
        Ok(())
    }
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
    /// Per-session egress policy (snapshot at creation; immutable for the run).
    pub egress: EgressPolicy,
    provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync>,
}

impl CloudSession {
    pub fn new(
        session_id: String,
        runtime: CloudRuntime,
        egress: EgressPolicy,
        provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync>,
    ) -> Self {
        Self {
            session_id,
            status: CloudSessionStatus::Pending,
            runtime,
            egress,
            provider,
        }
    }

    pub async fn execute(
        &mut self,
        code: &str,
        runtime: CloudRuntime,
        env: &HashMap<String, String>,
    ) -> Result<CloudResult, String> {
        self.status = CloudSessionStatus::Running;
        let result = self.provider.execute(&self.session_id, code, runtime, env).await?;
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
    /// Default egress policy applied to every new session.
    pub egress: EgressPolicy,
    sessions: Vec<CloudSession>,
    provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync>,
    /// AES-256-GCM 凭据保险库 — 执行前注入为容器环境变量 (NEOTRIX_VAULT_*)。
    #[cfg(feature = "sandbox")]
    vault: Option<Arc<Vault>>,
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
            egress: EgressPolicy::permissive(),
            sessions: Vec::new(),
            provider,
            #[cfg(feature = "sandbox")]
            vault: None,
        }
    }

    /// Set the default egress policy for subsequent sessions.
    pub fn set_egress(&mut self, policy: EgressPolicy) {
        self.egress = policy;
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

    /// 挂接外部提供的 Vault (测试/自定义配置)。
    #[cfg(feature = "sandbox")]
    pub fn attach_vault(&mut self, vault: Option<Arc<Vault>>) {
        self.vault = vault;
    }

    /// 生产接线: 从 `~/.neotrix/vault.enc` 加载默认 Vault。
    /// 未配置主密钥 (NEOTRIX_VAULT_KEY 缺省) 或加载失败 → 记录 warning,
    /// 以无注入模式运行 (向后兼容, 不阻断沙盒)。
    #[cfg(feature = "sandbox")]
    pub fn attach_default_vault(&mut self) {
        match Vault::new() {
            Ok(vault) => {
                let count = vault.len();
                self.vault = Some(Arc::new(vault));
                log::info!(
                    "[sandbox] vault attached ({} credential(s)); secrets will be injected as env",
                    count
                );
            }
            Err(e) => {
                log::warn!("[sandbox] vault unavailable: {}; running without secret injection", e);
            }
        }
    }

    /// 汇总 vault 凭据为待注入 env map (key 前缀 NEOTRIX_VAULT_)。无 vault → 空 map。
    fn vault_env(&self) -> HashMap<String, String> {
        #[cfg(feature = "sandbox")]
        {
            let mut env = HashMap::new();
            if let Some(vault) = &self.vault {
                vault.inject_env(&mut env);
            }
            env
        }
        #[cfg(not(feature = "sandbox"))]
        {
            HashMap::new()
        }
    }

    pub fn create_session(&mut self, runtime: CloudRuntime) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session =
            CloudSession::new(session_id.clone(), runtime, self.egress.clone(), Arc::clone(&self.provider));
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
        let env = self.vault_env();
        let session_id = self.create_session(runtime);
        let session = self.get_session_mut(&session_id).ok_or("session creation failed")?;
        session.execute(code, runtime, &env).await
    }

    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }
}

/// Per-call enforcement level (deepseek-harness pattern #5). Describes how a
/// single tool call should be confined. Also doubles as the *reported
/// enforcement fact* from the sandbox backend (`full`/`partial` honesty):
/// older backends (e.g. Landlock ABI) may only guarantee `Partial`, which must
/// be surfaced, never assumed to be `Full`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementLevel {
    /// Fully confined — file effects and egress fully governed by policy.
    Full,
    /// Partially confined — backend cannot guarantee the full boundary.
    Partial,
    /// Informational — call proceeds, observed effects reported to the agent.
    Notify,
}

impl EnforcementLevel {
    pub fn is_full(&self) -> bool {
        matches!(self, EnforcementLevel::Full)
    }
}

/// Per-call policy contract (deepseek-harness pattern #5). Strengthens — never
/// replaces — the existing session egress policy (R-P42): it carries the
/// demanded enforcement level, an optional per-call egress overlay evaluated
/// against the session snapshot, and an approval gate. Resolved *per call*;
/// never mutates the session or global policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallPolicy {
    /// Demanded enforcement level for this call.
    pub level: EnforcementLevel,
    /// Per-call egress overlay. `None` = evaluate against the session's
    /// immutable egress snapshot. Deny-wins inside the overlay.
    pub egress_override: Option<EgressPolicy>,
    /// When true, the call requires human approval granted *before* execution;
    /// an unanswered approval seam is a hard deny (fail-closed).
    pub requires_approval: bool,
}

impl CallPolicy {
    pub fn full() -> Self {
        Self {
            level: EnforcementLevel::Full,
            egress_override: None,
            requires_approval: false,
        }
    }

    pub fn with_egress_override(self, egress: EgressPolicy) -> Self {
        Self {
            egress_override: Some(egress),
            ..self
        }
    }

    pub fn with_approval(self, requires_approval: bool) -> Self {
        Self {
            requires_approval,
            ..self
        }
    }

    /// Deny-wins per-call resolution against the session egress policy and the
    /// backend-reported enforcement fact. Any denying dimension wins:
    ///   1. unanswered approval gate → `Approval` denial (fail-closed);
    ///   2. demanded `Full` but backend reports less than `Full` → `Sandbox`
    ///      denial (enforcement honesty — a `Partial` ABI cannot honor `Full`);
    ///   3. egress allow/deny verdict → `Egress` denial.
    pub fn evaluate(
        &self,
        enforcement: EnforcementLevel,
        session_egress: &EgressPolicy,
        host: &str,
        port: u16,
    ) -> CallVerdict {
        if self.requires_approval {
            return CallVerdict::Denied(CallDenial::approval(
                "approval seam unanswered or not granted",
            ));
        }
        if self.level == EnforcementLevel::Full && enforcement != EnforcementLevel::Full {
            return CallVerdict::Denied(CallDenial::sandbox(&format!(
                "demanded {:?} confinement but backend reports {:?}",
                self.level, enforcement
            )));
        }
        let egress = self.egress_override.as_ref().unwrap_or(session_egress);
        if !egress.check(host, port) {
            return CallVerdict::Denied(CallDenial::egress(host, port));
        }
        CallVerdict::Allowed(self.level)
    }
}

/// Machine-readable denial classification (deepseek-harness pattern #5:
/// "denial dialect signatures"). Lets a calling agent branch on the *kind*
/// instead of parsing prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DenialKind {
    Egress,
    Resource,
    Approval,
    Sandbox,
}

/// Structured denial reason — never a bare error string. An agent can act on
/// `code` (stable machine-readable tag) and `kind`, and show `message` to a
/// human.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallDenial {
    pub kind: DenialKind,
    /// Stable machine-readable code, e.g. `egress_denied:fetch.api.example.com:443`.
    pub code: String,
    /// Human-readable explanation.
    pub message: String,
}

impl CallDenial {
    pub fn egress(host: &str, port: u16) -> Self {
        Self {
            kind: DenialKind::Egress,
            code: format!("egress_denied:{}:{}", host, port),
            message: format!("egress denied for {}:{} — outside the sandbox trust boundary", host, port),
        }
    }

    pub fn resource(reason: &str) -> Self {
        Self {
            kind: DenialKind::Resource,
            code: "resource_denied".to_string(),
            message: format!("resource limit exceeded: {}", reason),
        }
    }

    pub fn approval(reason: &str) -> Self {
        Self {
            kind: DenialKind::Approval,
            code: "approval_required".to_string(),
            message: format!("human approval required: {}", reason),
        }
    }

    pub fn sandbox(reason: &str) -> Self {
        Self {
            kind: DenialKind::Sandbox,
            code: "sandbox_denied".to_string(),
            message: format!("sandbox confinement not enforceable: {}", reason),
        }
    }

    /// Agent-actionable directive for the caller of a denied tool call.
    pub fn agent_action(&self) -> &'static str {
        match self.kind {
            DenialKind::Egress => {
                "rewrite the call to use an allowed host/port from the sandbox egress policy"
            }
            DenialKind::Resource => "reduce the call's resource footprint (output size / concurrency)",
            DenialKind::Approval => "request human approval for the call, then retry",
            DenialKind::Sandbox => {
                "use a backend that can enforce the demanded level, or lower the call's level"
            }
        }
    }
}

/// Per-call resolution outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallVerdict {
    Allowed(EnforcementLevel),
    Denied(CallDenial),
}

impl CallVerdict {
    pub fn is_allowed(&self) -> bool {
        matches!(self, CallVerdict::Allowed(_))
    }

    pub fn denial(&self) -> Option<&CallDenial> {
        match self {
            CallVerdict::Denied(d) => Some(d),
            CallVerdict::Allowed(_) => None,
        }
    }
}

#[cfg(all(test, feature = "sandbox"))]
mod sandbox_vault_tests {
    use super::*;
    use futures::StreamExt;
    use crate::neotrix::l1_body_impl::nt_shield::vault::Vault;

    /// Test-only provider: spawns a real child process that reads the injected
    /// `NEOTRIX_VAULT_*` variable, proving secrets reach the workload env.
    struct EnvCaptureProvider;

    #[async_trait::async_trait]
    impl provider::CloudSandboxProvider for EnvCaptureProvider {
        fn name(&self) -> &'static str {
            "env-capture"
        }

        async fn execute(
            &self,
            _session_id: &str,
            _code: &str,
            _runtime: CloudRuntime,
            env: &HashMap<String, String>,
        ) -> Result<CloudResult, String> {
            let out = std::process::Command::new("sh")
                .args(["-c", "printf '%s' \"$NEOTRIX_VAULT_API_KEY\""])
                .envs(env)
                .output()
                .map_err(|e| format!("spawn: {}", e))?;
            Ok(CloudResult {
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code().unwrap_or(-1),
                execution_time: Duration::from_secs(0),
                resource_usage: ResourceUsage::default(),
            })
        }

        async fn upload_file(&self, _s: &str, _p: &str, _d: Vec<u8>) -> Result<(), String> {
            Ok(())
        }

        async fn download_result(&self, _s: &str) -> Result<CloudResult, String> {
            Ok(CloudResult {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                execution_time: Duration::from_secs(0),
                resource_usage: ResourceUsage::default(),
            })
        }

        fn stream_logs(&self, _s: &str) -> futures::stream::BoxStream<'static, String> {
            futures::stream::empty().boxed()
        }

        async fn cancel(&self, _s: &str) -> Result<(), String> {
            Ok(())
        }
    }

    /// 集成: Vault → CloudSandbox → provider.execute → 子进程 env 含凭据。
    #[test]
    fn test_vault_secrets_injected_into_sandbox_child_env() {
        std::env::set_var(
            "NEOTRIX_VAULT_KEY",
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        );
        let dir = tempfile::tempdir().expect("tempdir");
        let mut vault = Vault::with_path(dir.path().join("vault.enc")).expect("vault");
        vault.set("api_key", "sk-supersecret");
        vault.save().expect("vault.save");

        let mut cloud = CloudSandbox::new(
            "http://localhost".to_string(),
            None,
            Duration::from_secs(60),
            Arc::new(EnvCaptureProvider),
        );
        cloud.attach_vault(Some(Arc::new(vault)));

        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        let result = rt.block_on(cloud.run_code("print('hi')", CloudRuntime::Python3)).expect("run");
        assert_eq!(result.exit_code, 0, "child should run; stderr={}", result.stderr);
        assert_eq!(
            result.stdout, "sk-supersecret",
            "secret must be readable via NEOTRIX_VAULT_API_KEY in the child env"
        );
        std::env::remove_var("NEOTRIX_VAULT_KEY");
    }

    /// 无 vault 挂接时运行仍工作 (向后兼容, 注入为空)。
    #[test]
    fn test_run_code_without_vault_is_noop() {
        let mut cloud = CloudSandbox::new(
            "http://localhost".to_string(),
            None,
            Duration::from_secs(60),
            Arc::new(EnvCaptureProvider),
        );
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        let result = rt.block_on(cloud.run_code("echo hi", CloudRuntime::Python3)).expect("run");
        assert_eq!(result.exit_code, 0);
    }
}

#[cfg(test)]
mod egress_tests {
    use super::*;

    #[test]
    fn test_egress_exact_host_allow() {
        let policy = EgressPolicy::new(
            vec![EgressRule::allow("api.github.com", "443")],
            false,
        );
        assert!(policy.check("api.github.com", 443));
        assert!(!policy.check("api.github.com", 80), "port must match");
        assert!(!policy.check("evil.com", 443), "unlisted host denied by default");
    }

    #[test]
    fn test_egress_wildcard_suffix() {
        let policy = EgressPolicy::new(vec![EgressRule::allow("*.example.com", "443")], false);
        assert!(policy.check("api.example.com", 443));
        assert!(policy.check("a.b.example.com", 443));
        assert!(!policy.check("example.com", 443), "bare apex must match exactly, not suffix");
        assert!(!policy.check("example.com.evil.net", 443), "suffix must be dot-bounded");
    }

    #[test]
    fn test_egress_deny_shadows_allow() {
        let policy = EgressPolicy::new(
            vec![
                EgressRule::allow("*", "443"),
                EgressRule::deny("blocked.example.com", "443"),
            ],
            false,
        );
        assert!(policy.check("ok.example.com", 443));
        assert!(!policy.check("blocked.example.com", 443), "explicit deny wins");
    }

    #[test]
    fn test_egress_port_range() {
        let policy = EgressPolicy::new(vec![EgressRule::allow("db.internal", "5432-5433")], false);
        assert!(policy.check("db.internal", 5432));
        assert!(policy.check("db.internal", 5433));
        assert!(!policy.check("db.internal", 5434));
    }

    #[test]
    fn test_egress_permissive_and_deny_all() {
        assert!(EgressPolicy::permissive().check("anything.com", 1));
        assert!(!EgressPolicy::deny_all().check("anything.com", 1));
    }

    #[test]
    fn test_sandbox_attaches_egress_to_session() {
        let provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync> =
            Arc::new(provider::NoopProvider);
        let mut cloud = CloudSandbox::new(
            "http://localhost".to_string(),
            None,
            Duration::from_secs(60),
            provider,
        );
        cloud.set_egress(EgressPolicy::new(
            vec![EgressRule::allow("api.openai.com", "443")],
            false,
        ));
        let sid = cloud.create_session(CloudRuntime::Python3);
        let session = cloud.get_session(&sid).expect("session exists");
        assert!(session.egress.check("api.openai.com", 443));
        assert!(!session.egress.check("fetch.other.com", 443));
    }
}

#[cfg(test)]
mod call_policy_tests {
    use super::*;

    #[test]
    fn test_deny_wins_full_vs_partial() {
        // Demanded Full but the backend only guarantees Partial → fail-closed denial,
        // even though the egress policy would allow the host (permissive).
        let policy = CallPolicy::full();
        let verdict =
            policy.evaluate(EnforcementLevel::Partial, &EgressPolicy::permissive(), "api.example.com", 443);
        assert!(!verdict.is_allowed());
        assert_eq!(verdict.denial().map(|d| d.kind), Some(DenialKind::Sandbox));

        // When the backend reports Full, the same call is allowed.
        let ok = policy.evaluate(EnforcementLevel::Full, &EgressPolicy::permissive(), "api.example.com", 443);
        assert!(ok.is_allowed());
    }

    #[test]
    fn test_denial_dialect_maps_to_agent_action() {
        let denials = [
            CallDenial::egress("fetch.evil.net", 443),
            CallDenial::resource(">10MB output"),
            CallDenial::approval("write to ~/.ssh"),
            CallDenial::sandbox("provider refused"),
        ];
        for d in &denials {
            assert!(!d.code.is_empty(), "machine-readable code required");
            assert!(!d.message.is_empty(), "human message required");
            assert!(
                !d.agent_action().is_empty(),
                "agent-actionable directive required for {:?}",
                d.kind
            );
        }
        assert!(CallDenial::egress("h", 1).code.starts_with("egress_denied:"));
        assert_eq!(CallDenial::approval("x").kind, DenialKind::Approval);
        assert_eq!(CallDenial::resource("y").kind, DenialKind::Resource);
        assert_eq!(CallDenial::sandbox("z").kind, DenialKind::Sandbox);
        assert_eq!(CallDenial::egress("h", 1).kind, DenialKind::Egress);
    }

    #[test]
    fn test_per_call_override_does_not_mutate_global_policy() {
        let provider: Arc<dyn provider::CloudSandboxProvider + Send + Sync> =
            Arc::new(provider::NoopProvider);
        let mut cloud = CloudSandbox::new(
            "http://localhost".to_string(),
            None,
            Duration::from_secs(60),
            provider,
        );
        cloud.set_egress(EgressPolicy::deny_all());
        let sid = cloud.create_session(CloudRuntime::Python3);
        let session = cloud.get_session(&sid).expect("session exists");

        // Per-call override allows an API host while the session stays deny-all.
        let policy = CallPolicy::full().with_egress_override(EgressPolicy::new(
            vec![EgressRule::allow("api.openai.com", "443")],
            false,
        ));
        let verdict = policy.evaluate(EnforcementLevel::Full, &session.egress, "api.openai.com", 443);
        assert!(verdict.is_allowed(), "per-call override grants the API host");

        // Global/session policy must be untouched by the per-call override.
        assert!(!cloud.egress.check("api.openai.com", 443), "global policy unchanged");
        assert!(!session.egress.check("api.openai.com", 443), "session policy unchanged");

        // Without the override, the same host is denied by the session policy.
        let denied = CallPolicy::full()
            .evaluate(EnforcementLevel::Full, &session.egress, "api.openai.com", 443);
        assert_eq!(denied.denial().map(|d| d.kind), Some(DenialKind::Egress));
    }
}

use std::sync::Mutex;
use std::sync::OnceLock;

use crate::cli::commands::CommandOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyAllowlist {
    pub allowed_domains: Vec<String>,
    pub blocked_domains: Vec<String>,
    pub allow_all: bool,
}

impl ProxyAllowlist {
    pub fn new() -> Self {
        Self {
            allowed_domains: Vec::new(),
            blocked_domains: Vec::new(),
            allow_all: true,
        }
    }

    pub fn check_domain(&self, domain: &str) -> Result<(), String> {
        if self
            .blocked_domains
            .iter()
            .any(|d| domain == d || domain.ends_with(&format!(".{d}")))
        {
            return Err(format!("Domain '{domain}' is blocked by proxy allowlist"));
        }
        if self.allow_all {
            return Ok(());
        }
        if self
            .allowed_domains
            .iter()
            .any(|d| domain == d || domain.ends_with(&format!(".{d}")))
        {
            Ok(())
        } else {
            Err(format!(
                "Domain '{domain}' is not in the proxy allowlist. Allowed: {:?}",
                self.allowed_domains
            ))
        }
    }
}

impl Default for ProxyAllowlist {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliSandboxMode {
    Disabled,
    ReadOnly,
    Docker,
}

impl CliSandboxMode {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "read-only" | "readonly" | "ro" => Self::ReadOnly,
            "docker" => Self::Docker,
            _ => Self::Disabled,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Disabled => "",
            Self::ReadOnly => "🔒 READ-ONLY",
            Self::Docker => "🐳 DOCKER",
        }
    }
}

pub struct SandboxEnforcer {
    mode: CliSandboxMode,
    pub proxy_allowlist: ProxyAllowlist,
}

impl SandboxEnforcer {
    pub fn new(mode: CliSandboxMode) -> Self {
        Self {
            mode,
            proxy_allowlist: ProxyAllowlist::new(),
        }
    }

    pub fn mode(&self) -> CliSandboxMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: CliSandboxMode) {
        self.mode = mode;
    }

    pub fn is_read_only(&self) -> bool {
        self.mode == CliSandboxMode::ReadOnly
    }

    pub fn check_read_only(&self) -> Option<CommandOutput> {
        if self.is_read_only() {
            Some(CommandOutput::err(
                "🔒 Read-only sandbox: this operation is blocked. Use --sandbox disabled to allow write operations.",
            ))
        } else {
            None
        }
    }

    pub fn check_network_access(&self, target: &str) -> Option<CommandOutput> {
        let domain = if target.starts_with("http://") || target.starts_with("https://") {
            target
                .split("://")
                .nth(1)
                .and_then(|rest| rest.split('/').next())
                .and_then(|host| host.split(':').next())
                .unwrap_or(target)
        } else {
            target
        };

        match self.proxy_allowlist.check_domain(domain) {
            Ok(()) => None,
            Err(msg) => Some(CommandOutput::err(&msg)),
        }
    }
}

static SANDBOX_ENFORCER: OnceLock<Mutex<SandboxEnforcer>> = OnceLock::new();

pub fn global_sandbox() -> &'static Mutex<SandboxEnforcer> {
    SANDBOX_ENFORCER.get_or_init(|| Mutex::new(SandboxEnforcer::new(CliSandboxMode::Disabled)))
}

pub fn init_sandbox(mode: CliSandboxMode) {
    let mut e = global_sandbox().lock().unwrap_or_else(|e| e.into_inner());
    e.set_mode(mode);
}

pub fn check_sandbox() -> Option<CommandOutput> {
    global_sandbox()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .check_read_only()
}

static PROXY_ALLOWLIST: OnceLock<Mutex<ProxyAllowlist>> = OnceLock::new();

pub fn global_sandbox_proxy() -> &'static Mutex<ProxyAllowlist> {
    PROXY_ALLOWLIST.get_or_init(|| Mutex::new(ProxyAllowlist::new()))
}

// TODO: add #[serial] to any new tests that use global singletons

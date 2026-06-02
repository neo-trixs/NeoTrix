use std::sync::Mutex;
use std::sync::OnceLock;

use crate::cli::commands::CommandOutput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxMode {
    Disabled,
    ReadOnly,
    Docker,
}

impl SandboxMode {
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
    mode: SandboxMode,
}

impl SandboxEnforcer {
    pub fn new(mode: SandboxMode) -> Self {
        Self { mode }
    }

    pub fn mode(&self) -> SandboxMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: SandboxMode) {
        self.mode = mode;
    }

    pub fn is_read_only(&self) -> bool {
        self.mode == SandboxMode::ReadOnly
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
}

static SANDBOX_ENFORCER: OnceLock<Mutex<SandboxEnforcer>> = OnceLock::new();

pub fn global_sandbox() -> &'static Mutex<SandboxEnforcer> {
    SANDBOX_ENFORCER
        .get_or_init(|| Mutex::new(SandboxEnforcer::new(SandboxMode::Disabled)))
}

pub fn init_sandbox(mode: SandboxMode) {
    let mut e = global_sandbox().lock().expect("global_sandbox lock");
    e.set_mode(mode);
}

pub fn check_sandbox() -> Option<CommandOutput> {
    global_sandbox().lock().expect("global_sandbox lock").check_read_only()
}

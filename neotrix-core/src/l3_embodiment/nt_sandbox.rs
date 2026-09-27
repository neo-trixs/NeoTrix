//! L3 Embodiment — Sandbox execution guard (migrated from cli::sandbox).
//!
//! Decoupled from CLI command types: `check_read_only` returns `Option<String>`
//! instead of `CommandOutput` so non-CLI callers (AutoOrchestrator, agent loop)
//! can use it directly.

use std::sync::LazyLock;
use std::sync::Mutex;

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

    pub fn check_read_only(&self) -> Option<String> {
        if self.is_read_only() {
            Some(
                "🔒 Read-only sandbox: this operation is blocked. Use --sandbox disabled to allow write operations."
                    .to_string(),
            )
        } else {
            None
        }
    }
}

/// Global fallback — prefer context-local sandbox config instead.
pub static SANDBOX_ENFORCER: LazyLock<Mutex<SandboxEnforcer>> = LazyLock::new(|| {
    Mutex::new(SandboxEnforcer::new(SandboxMode::Disabled))
});

pub fn global_sandbox() -> &'static Mutex<SandboxEnforcer> {
    &SANDBOX_ENFORCER
}

pub fn init_sandbox(mode: SandboxMode) {
    let mut e = global_sandbox().lock().unwrap_or_else(|e| e.into_inner());
    e.set_mode(mode);
}

pub fn check_sandbox() -> Option<String> {
    global_sandbox()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .check_read_only()
}

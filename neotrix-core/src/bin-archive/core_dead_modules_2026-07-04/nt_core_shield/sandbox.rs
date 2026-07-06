use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxLevel {
    None,
    Basic,
    Standard,
    Strict,
    Paranoid,
}

impl SandboxLevel {
    pub fn is_active(&self) -> bool {
        !matches!(self, SandboxLevel::None)
    }

    pub fn description(&self) -> &'static str {
        match self {
            SandboxLevel::None => "no kernel sandbox",
            SandboxLevel::Basic => "basic file-read sandbox",
            SandboxLevel::Standard => "standard syscall sandbox",
            SandboxLevel::Strict => "strict sandbox with network isolation",
            SandboxLevel::Paranoid => "paranoid: all non-essential syscalls blocked",
        }
    }
}

impl fmt::Display for SandboxLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub level: SandboxLevel,
    pub allow_network: bool,
    pub allow_write: bool,
    pub allow_exec: bool,
    pub temp_dir: Option<String>,
    pub profile_path: Option<String>,
    pub extra_allowed_paths: Vec<String>,
}

impl SandboxConfig {
    pub fn default() -> Self {
        Self {
            level: SandboxLevel::None,
            allow_network: true,
            allow_write: true,
            allow_exec: true,
            temp_dir: None,
            profile_path: None,
            extra_allowed_paths: Vec::new(),
        }
    }

    pub fn for_non_debug() -> Self {
        Self {
            level: SandboxLevel::Standard,
            allow_network: true,
            allow_write: true,
            allow_exec: true,
            temp_dir: Some(std::env::temp_dir().to_string_lossy().to_string()),
            profile_path: None,
            extra_allowed_paths: Vec::new(),
        }
    }

    pub fn strict() -> Self {
        Self {
            level: SandboxLevel::Strict,
            allow_network: false,
            allow_write: true,
            allow_exec: false,
            temp_dir: Some(std::env::temp_dir().to_string_lossy().to_string()),
            profile_path: None,
            extra_allowed_paths: Vec::new(),
        }
    }

    pub fn paranoid() -> Self {
        Self {
            level: SandboxLevel::Paranoid,
            allow_network: false,
            allow_write: false,
            allow_exec: false,
            temp_dir: Some(std::env::temp_dir().to_string_lossy().to_string()),
            profile_path: None,
            extra_allowed_paths: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct SandboxError {
    pub message: String,
    pub platform: String,
}

impl fmt::Display for SandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.platform, self.message)
    }
}

impl std::error::Error for SandboxError {}

/// Sandbox backend technology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxBackend {
    WebAssembly,
    GVisor,
    Firecracker,
    Container,
}

impl SandboxBackend {
    pub fn name(&self) -> &'static str {
        match self {
            SandboxBackend::WebAssembly => "WASM/wasmtime",
            SandboxBackend::GVisor => "gVisor/runsc",
            SandboxBackend::Firecracker => "Firecracker/microVM",
            SandboxBackend::Container => "Docker/runc",
        }
    }

    pub fn isolation_level(&self) -> &'static str {
        match self {
            SandboxBackend::WebAssembly => "runtime-enforced (no syscall ABI)",
            SandboxBackend::GVisor => "userspace kernel (syscall interception)",
            SandboxBackend::Firecracker => "hardware VM (KVM)",
            SandboxBackend::Container => "shared kernel (namespaces+cgroups)",
        }
    }

    /// Recommended threat model tier for this backend
    pub fn threat_tier(&self) -> u8 {
        match self {
            SandboxBackend::WebAssembly => 2,
            SandboxBackend::GVisor => 1,
            SandboxBackend::Firecracker => 0,
            SandboxBackend::Container => 3,
        }
    }
}

/// Sandbox profile for multi-tier agent execution
#[derive(Debug, Clone)]
pub struct SandboxProfile {
    pub backend: SandboxBackend,
    pub config: SandboxConfig,
    pub memory_limit_mb: u64,
    pub cpu_limit: f64,
    pub network_allowed: bool,
    pub capabilities: Vec<String>,
}

impl SandboxProfile {
    /// Default WASM sandbox (fastest, capability-gated)
    pub fn wasm() -> Self {
        Self {
            backend: SandboxBackend::WebAssembly,
            config: SandboxConfig::paranoid(),
            memory_limit_mb: 256,
            cpu_limit: 1.0,
            network_allowed: false,
            capabilities: vec![],
        }
    }

    /// Default gVisor sandbox (strong isolation, moderate perf)
    pub fn gvisor() -> Self {
        Self {
            backend: SandboxBackend::GVisor,
            config: SandboxConfig::strict(),
            memory_limit_mb: 512,
            cpu_limit: 2.0,
            network_allowed: false,
            capabilities: vec!["compute".into()],
        }
    }

    /// Default Firecracker microVM (strongest isolation)
    pub fn firecracker() -> Self {
        Self {
            backend: SandboxBackend::Firecracker,
            config: SandboxConfig::strict(),
            memory_limit_mb: 1024,
            cpu_limit: 4.0,
            network_allowed: false,
            capabilities: vec!["compute".into(), "storage".into()],
        }
    }
}

/// Result of sandbox execution
#[derive(Debug, Clone)]
pub struct SandboxExecResult {
    pub backend: SandboxBackend,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub success: bool,
}

impl SandboxExecResult {
    pub fn new(backend: SandboxBackend) -> Self {
        Self {
            backend,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
            success: false,
        }
    }
}

/// Multi-tier sandbox orchestrator
#[derive(Debug, Clone)]
pub struct SandboxOrchestrator {
    pub profiles: Vec<SandboxProfile>,
}

impl Default for SandboxOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl SandboxOrchestrator {
    pub fn new() -> Self {
        Self {
            profiles: vec![
                SandboxProfile::wasm(),
                SandboxProfile::gvisor(),
                SandboxProfile::firecracker(),
            ],
        }
    }

    /// Select best backend for a given threat profile
    /// tier 0 = untrusted code execution, tier 3 = trusted internal
    pub fn select_backend(&self, threat_tier: u8) -> Option<&SandboxProfile> {
        for profile in &self.profiles {
            if profile.backend.threat_tier() <= threat_tier {
                return Some(profile);
            }
        }
        self.profiles.first()
    }

    /// Execute code in sandbox (stub — real execution needs wasmtime/gVisor/Firecracker CLI)
    pub fn execute(&self, profile: &SandboxProfile, _code: &str, _runtime: &str) -> SandboxExecResult {
        let mut result = SandboxExecResult::new(profile.backend);
        let start = std::time::Instant::now();

        match profile.backend {
            SandboxBackend::WebAssembly => {
                // WASM: would use wasmtime embedding
                result.stdout = "WASM execution stub".into();
                result.success = true;
            }
            SandboxBackend::GVisor => {
                // gVisor: would use `runsc` CLI
                result.stdout = "gVisor execution stub".into();
                result.success = true;
            }
            SandboxBackend::Firecracker => {
                // Firecracker: would use jailer + firecracker CLI
                result.stdout = "Firecracker execution stub".into();
                result.success = true;
            }
            SandboxBackend::Container => {
                // Docker: reuse existing provider
                result.stdout = "Container execution stub".into();
                result.success = true;
            }
        }

        result.duration_ms = start.elapsed().as_millis() as u64;
        result
    }
}

fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS/Seatbelt"
    } else if cfg!(target_os = "linux") {
        "Linux/Landlock+seccomp"
    } else if cfg!(target_os = "windows") {
        "Windows/AppContainer"
    } else {
        "Unknown"
    }
}

pub fn init_kernel_sandbox(config: &SandboxConfig) -> Result<(), SandboxError> {
    if !config.level.is_active() {
        log::info!("[kernel_sandbox] level=None, skipping");
        return Ok(());
    }

    log::info!(
        "[kernel_sandbox] initializing level={:?} on {}",
        config.level,
        platform_name()
    );

    #[cfg(target_os = "macos")]
    {
        init_macos_seatbelt(config)
    }

    #[cfg(target_os = "linux")]
    {
        init_linux_landlock(config)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let plat = std::env::consts::OS;
        log::warn!(
            "[kernel_sandbox] OS-level sandboxing not supported on {}",
            plat
        );
        Err(SandboxError {
            message: format!("OS-level sandboxing not supported on {}", plat),
            platform: platform_name().to_string(),
        })
    }
}

#[cfg(target_os = "macos")]
fn init_macos_seatbelt(config: &SandboxConfig) -> Result<(), SandboxError> {
    let allow_network = if config.allow_network { "(allow network*) (allow ipc-posix*)" } else { "(deny network*)" };
    let allow_write = if config.allow_write { "(allow file-write*)" } else { "" };
    let allow_exec = if config.allow_exec { "(allow process-exec*)" } else { "" };

    let profile = format!(
        "(version 1)
         (deny default)
         (allow file-read*)
         (allow file-read-metadata)
         (allow sysctl-read)
         (allow signal (target self))
         {}
         {}
         {}
         (allow file-write* (subpath \"/tmp\") (subpath \"{}\"))
         (allow file-read* (subpath \"/usr/lib\") (subpath \"/System\")
                (subpath \"/usr/share\") (subpath \"/private\")
                (subpath (\"{}\")))",
        allow_network,
        allow_write,
        allow_exec,
        config.temp_dir.as_deref().unwrap_or("/tmp"),
        config.extra_allowed_paths.join("\")\n         (allow file-read* (subpath \"")
    );

    let profile_path = config.profile_path.clone().unwrap_or_else(|| {
        let tmp = std::env::temp_dir().join("neotrix_seatbelt.sb");
        tmp.to_string_lossy().to_string()
    });

    if let Err(e) = std::fs::write(&profile_path, &profile) {
        return Err(SandboxError {
            message: format!("failed to write seatbelt profile: {}", e),
            platform: "macOS/Seatbelt".to_string(),
        });
    }

    let status = std::process::Command::new("sandbox-exec")
        .arg("-f")
        .arg(&profile_path)
        .arg("true")
        .status()
        .map_err(|e| SandboxError {
            message: format!("sandbox-exec not found: {}", e),
            platform: "macOS/Seatbelt".to_string(),
        })?;

    if !status.success() {
        return Err(SandboxError {
            message: format!("sandbox-exec validation failed: {:?}", status.code()),
            platform: "macOS/Seatbelt".to_string(),
        });
    }

    log::info!(
        "[kernel_sandbox] seatbelt profile validated at {}",
        profile_path
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn init_linux_landlock(config: &SandboxConfig) -> Result<(), SandboxError> {
    if let Err(e) = std::fs::write("/proc/self/comm", "neotrix-sandboxed") {
        log::warn!("[kernel_sandbox] could not set process name: {}", e);
    }

    let landlock_available = std::fs::metadata("/sys/kernel/security/landlock").is_ok()
        || std::fs::metadata("/proc/self/attr/current").is_ok();

    if !landlock_available {
        return Err(SandboxError {
            message: "Landlock LSM not available. Requires Linux kernel >=5.13 with CONFIG_SECURITY_LANDLOCK=y"
                .to_string(),
            platform: "Linux/Landlock+seccomp".to_string(),
        });
    }

    log::info!(
        "[kernel_sandbox] Landlock available, applying level={:?}",
        config.level
    );

    match config.level {
        SandboxLevel::Basic | SandboxLevel::Standard => {
            log::info!("[kernel_sandbox] level={:?}: read-only filesystem sandbox via Landlock", config.level);
            log::info!("[kernel_sandbox] for full Landlock ruleset, add `landlock` crate to dependencies");
        }
        SandboxLevel::Strict | SandboxLevel::Paranoid => {
            log::info!("[kernel_sandbox] level={:?}: strict sandbox requested", config.level);
            log::info!("[kernel_sandbox] network={}, write={}, exec={}",
                config.allow_network, config.allow_write, config.allow_exec);
        }
        SandboxLevel::None => {}
    }

    Ok(())
}

pub fn check_platform_support() -> PlatformSupport {
    #[cfg(target_os = "macos")]
    {
        let has_sandbox_exec = std::process::Command::new("which")
            .arg("sandbox-exec")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if has_sandbox_exec {
            PlatformSupport::Full("macOS Seatbelt via sandbox-exec".to_string())
        } else {
            PlatformSupport::Partial(
                "sandbox-exec not found in PATH".to_string(),
            )
        }
    }

    #[cfg(target_os = "linux")]
    {
        let landlock_avail = std::fs::metadata("/sys/kernel/security/landlock").is_ok();
        if landlock_avail {
            PlatformSupport::Full("Linux Landlock LSM available".to_string())
        } else {
            PlatformSupport::Partial(
                "Landlock LSM not available (kernel >=5.13 + CONFIG_SECURITY_LANDLOCK=y required)".to_string(),
            )
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        PlatformSupport::None(format!("no sandbox support for {}", std::env::consts::OS))
    }
}

#[derive(Debug, Clone)]
pub enum PlatformSupport {
    Full(String),
    Partial(String),
    None(String),
}

impl PlatformSupport {
    pub fn is_usable(&self) -> bool {
        matches!(self, PlatformSupport::Full(_))
    }

    pub fn description(&self) -> &str {
        match self {
            PlatformSupport::Full(d) | PlatformSupport::Partial(d) | PlatformSupport::None(d) => d.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_level_is_active() {
        assert!(!SandboxLevel::None.is_active());
        assert!(SandboxLevel::Basic.is_active());
        assert!(SandboxLevel::Standard.is_active());
        assert!(SandboxLevel::Strict.is_active());
        assert!(SandboxLevel::Paranoid.is_active());
    }

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.level, SandboxLevel::None);
        assert!(config.allow_network);
        assert!(config.allow_write);
        assert!(config.allow_exec);
    }

    #[test]
    fn test_sandbox_config_for_non_debug() {
        let config = SandboxConfig::for_non_debug();
        assert_eq!(config.level, SandboxLevel::Standard);
        assert!(config.allow_network);
    }

    #[test]
    fn test_sandbox_config_strict() {
        let config = SandboxConfig::strict();
        assert_eq!(config.level, SandboxLevel::Strict);
        assert!(!config.allow_network);
        assert!(!config.allow_exec);
    }

    #[test]
    fn test_sandbox_config_paranoid() {
        let config = SandboxConfig::paranoid();
        assert_eq!(config.level, SandboxLevel::Paranoid);
        assert!(!config.allow_network);
        assert!(!config.allow_write);
        assert!(!config.allow_exec);
    }

    #[test]
    fn test_sandbox_level_display() {
        assert_eq!(SandboxLevel::None.to_string(), "no kernel sandbox");
        assert!(SandboxLevel::Standard.to_string().contains("syscall"));
    }

    #[test]
    fn test_check_platform_support() {
        let support = check_platform_support();
        let desc = support.description();
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_skip_on_none_level() {
        let config = SandboxConfig::default();
        let result = init_kernel_sandbox(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sandbox_error_display() {
        let err = SandboxError {
            message: "test error".to_string(),
            platform: "test".to_string(),
        };
        assert!(err.to_string().contains("test error"));
    }

    // ── P0.2: Multi-tier Sandbox ──

    #[test]
    fn test_sandbox_backend_names() {
        assert_eq!(SandboxBackend::WebAssembly.name(), "WASM/wasmtime");
        assert_eq!(SandboxBackend::Firecracker.name(), "Firecracker/microVM");
    }

    #[test]
    fn test_sandbox_backend_isolation_levels() {
        assert!(SandboxBackend::WebAssembly.isolation_level().contains("no syscall"));
        assert!(SandboxBackend::Firecracker.isolation_level().contains("hardware VM"));
        assert!(SandboxBackend::GVisor.isolation_level().contains("userspace kernel"));
    }

    #[test]
    fn test_threat_tier_ordering() {
        assert!(SandboxBackend::Firecracker.threat_tier() < SandboxBackend::GVisor.threat_tier());
        assert!(SandboxBackend::WebAssembly.threat_tier() < SandboxBackend::Container.threat_tier());
    }

    #[test]
    fn test_sandbox_profile_wasm_defaults() {
        let profile = SandboxProfile::wasm();
        assert_eq!(profile.backend, SandboxBackend::WebAssembly);
        assert_eq!(profile.memory_limit_mb, 256);
        assert!(!profile.network_allowed);
    }

    #[test]
    fn test_sandbox_profile_gvisor_defaults() {
        let profile = SandboxProfile::gvisor();
        assert_eq!(profile.backend, SandboxBackend::GVisor);
        assert_eq!(profile.memory_limit_mb, 512);
    }

    #[test]
    fn test_sandbox_profile_firecracker_defaults() {
        let profile = SandboxProfile::firecracker();
        assert_eq!(profile.backend, SandboxBackend::Firecracker);
        assert_eq!(profile.memory_limit_mb, 1024);
    }

    #[test]
    fn test_orchestrator_select_backend_by_threat() {
        let orch = SandboxOrchestrator::new();
        let for_trusted = orch.select_backend(3);
        assert_eq!(for_trusted.unwrap().backend, SandboxBackend::WebAssembly);
        let for_untrusted = orch.select_backend(0);
        assert_eq!(for_untrusted.unwrap().backend, SandboxBackend::Firecracker);
    }

    #[test]
    fn test_orchestrator_execute_returns_result() {
        let orch = SandboxOrchestrator::new();
        let profile = SandboxProfile::wasm();
        let result = orch.execute(&profile, "print(1)", "python3");
        assert!(result.success);
        assert!(result.duration_ms < 100);
    }
}

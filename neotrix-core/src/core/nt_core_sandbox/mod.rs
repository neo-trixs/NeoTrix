//! # Kernel-level sandbox integration
//!
//! Provides OS-level sandbox enforcement using platform-native mechanisms:
//! - **macOS**: Seatbelt (`sandbox_init` via FFI)
//! - **Linux**: Landlock LSM (path/network access control) + seccomp-bpf (syscall filtering)
//! - **Other**: Graceful degradation with log::warn!
//!
//! This is a boot-time initializer, not a per-cycle handler.
//! Sandbox is set once at startup and cannot be escalated at runtime.

mod landlock;
mod seatbelt;
mod seccomp;

/// Kernel sandbox isolation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SandboxLevel {
    /// No kernel sandbox (dev mode, debug builds).
    None,
    /// Basic filesystem isolation only.
    Minimal,
    /// Standard isolation (seccomp + filesystem restrictions).
    Standard,
    /// Full sandbox (all available restrictions).
    Strict,
}

impl SandboxLevel {
    pub fn label(&self) -> &'static str {
        match self {
            SandboxLevel::None => "none",
            SandboxLevel::Minimal => "minimal",
            SandboxLevel::Standard => "standard",
            SandboxLevel::Strict => "strict",
        }
    }
}

/// Configuration for kernel sandbox initialization.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub level: SandboxLevel,
    pub allow_net: bool,
    pub allow_exec: bool,
    pub allow_write: Vec<String>,
    pub allow_read: Vec<String>,
    pub allow_net_hosts: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            level: SandboxLevel::None,
            allow_net: false,
            allow_exec: false,
            allow_write: vec![],
            allow_read: vec![],
            allow_net_hosts: vec![],
        }
    }
}

impl SandboxConfig {
    pub fn for_non_debug() -> Self {
        Self {
            level: SandboxLevel::Standard,
            allow_net: true,
            allow_exec: false,
            allow_write: vec![],
            allow_read: vec!["/tmp".into(), "/var/tmp".into()],
            allow_net_hosts: vec!["localhost".into(), "127.0.0.1".into()],
        }
    }

    pub fn strict() -> Self {
        Self {
            level: SandboxLevel::Strict,
            allow_net: false,
            allow_exec: false,
            allow_write: vec![],
            allow_read: vec![],
            allow_net_hosts: vec![],
        }
    }
}

/// Initialize kernel-level sandbox based on configuration.
///
/// Called once at consciousness startup. In debug mode, always returns Ok
/// and logs that sandbox was skipped. In release mode, dispatches to the
/// platform-specific implementation.
pub fn init_kernel_sandbox(config: &SandboxConfig) -> Result<(), String> {
    if config.level == SandboxLevel::None {
        log::info!("[kernel_sandbox] skipped (level=None)");
        return Ok(());
    }

    // Debug guard: never enable kernel sandbox in debug builds
    let is_debug = cfg!(debug_assertions);
    if is_debug {
        log::warn!(
            "[kernel_sandbox] skipped in debug mode (requested level={})",
            config.level.label()
        );
        return Ok(());
    }

    log::info!(
        "[kernel_sandbox] initializing level={}",
        config.level.label()
    );

    #[cfg(target_os = "macos")]
    {
        init_macos_sandbox(config)
    }

    #[cfg(target_os = "linux")]
    {
        init_linux_sandbox(config)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        log::warn!(
            "[kernel_sandbox] no implementation for target_os={}, skipping",
            std::env::consts::OS
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_level_labels() {
        assert_eq!(SandboxLevel::None.label(), "none");
        assert_eq!(SandboxLevel::Minimal.label(), "minimal");
        assert_eq!(SandboxLevel::Standard.label(), "standard");
        assert_eq!(SandboxLevel::Strict.label(), "strict");
    }

    #[test]
    fn test_sandbox_level_ordering() {
        assert!(SandboxLevel::None < SandboxLevel::Minimal);
        assert!(SandboxLevel::Minimal < SandboxLevel::Standard);
        assert!(SandboxLevel::Standard < SandboxLevel::Strict);
    }

    #[test]
    fn test_config_default() {
        let cfg = SandboxConfig::default();
        assert_eq!(cfg.level, SandboxLevel::None);
        assert!(!cfg.allow_net);
        assert!(!cfg.allow_exec);
        assert!(cfg.allow_write.is_empty());
        assert!(cfg.allow_read.is_empty());
    }

    #[test]
    fn test_config_for_non_debug() {
        let cfg = SandboxConfig::for_non_debug();
        assert_eq!(cfg.level, SandboxLevel::Standard);
        assert!(cfg.allow_net);
        assert!(!cfg.allow_exec);
        assert!(cfg.allow_read.contains(&"/tmp".to_string()));
    }

    #[test]
    fn test_config_strict() {
        let cfg = SandboxConfig::strict();
        assert_eq!(cfg.level, SandboxLevel::Strict);
        assert!(!cfg.allow_net);
        assert!(!cfg.allow_exec);
    }

    #[test]
    fn test_init_kernel_sandbox_none() {
        let cfg = SandboxConfig::default();
        let result = init_kernel_sandbox(&cfg);
        assert!(result.is_ok());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_build_macos_profile_strict() {
        let cfg = SandboxConfig::strict();
        let profile = super::build_macos_profile(&cfg);
        assert!(profile.contains("(version 1)"));
        assert!(profile.contains("(deny default)"));
        assert!(!profile.contains("(allow network*)"));
        assert!(!profile.contains("(allow process-fork)"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_build_macos_profile_with_network() {
        let cfg = SandboxConfig {
            level: SandboxLevel::Standard,
            allow_net: true,
            allow_exec: false,
            allow_write: vec![],
            allow_read: vec!["/custom".into()],
            allow_net_hosts: vec!["api.test.com".into()],
        };
        let profile = super::build_macos_profile(&cfg);
        assert!(profile.contains("(allow network*)"));
        assert!(profile.contains("api.test.com"));
        assert!(profile.contains("/custom"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_build_linux_sandbox_no_crash() {
        let cfg = SandboxConfig::default();
        let result = super::init_linux_sandbox(&cfg);
        // On a system without Landlock/seccomp, this should warn but not error
        assert!(result.is_ok());
    }
}

#[cfg(target_os = "macos")]
fn init_macos_sandbox(config: &SandboxConfig) -> Result<(), String> {
    let profile = build_macos_profile(config);
    seatbelt::enable_seatbelt(&profile)?;
    log::info!(
        "[kernel_sandbox] macOS Seatbelt enabled (level={})",
        config.level.label()
    );
    Ok(())
}

#[cfg(target_os = "macos")]
fn build_macos_profile(config: &SandboxConfig) -> String {
    let mut sbpl = String::from("(version 1)\n");

    sbpl.push_str("(deny default)\n");

    // Allow basic system access
    sbpl.push_str("(allow sysctl-read)\n");
    sbpl.push_str("(allow signal (target self))\n");

    // Allow read access to standard system libraries
    sbpl.push_str("(allow file-read* (subpath \"/usr/lib\"))\n");
    sbpl.push_str("(allow file-read* (subpath \"/System/Library\"))\n");

    // Allow temp directory read/write
    sbpl.push_str("(allow file-read* file-write* (subpath \"/tmp\"))\n");
    sbpl.push_str("(allow file-read* file-write* (subpath \"/var/tmp\"))\n");

    // Application data directory
    if let Ok(home) = std::env::var("HOME") {
        let app_dir = format!("{}/.neotrix", home);
        sbpl.push_str(&format!(
            "(allow file-read* file-write* (subpath \"{}\"))\n",
            app_dir
        ));
    }

    // Configurable read paths
    for p in &config.allow_read {
        sbpl.push_str(&format!("(allow file-read* (subpath \"{}\"))\n", p));
    }

    // Configurable write paths
    for p in &config.allow_write {
        sbpl.push_str(&format!(
            "(allow file-read* file-write* (subpath \"{}\"))\n",
            p
        ));
    }

    // Network
    if config.allow_net {
        sbpl.push_str("(allow network*)\n");
        for host in &config.allow_net_hosts {
            sbpl.push_str(&format!(
                "(allow network-outbound (remote name \"{}\"))\n",
                host
            ));
        }
    } else {
        if !config.allow_net_hosts.is_empty() {
            for host in &config.allow_net_hosts {
                sbpl.push_str(&format!(
                    "(allow network-outbound (remote name \"{}\"))\n",
                    host
                ));
            }
        }
    }

    // Process spawn
    if config.allow_exec {
        sbpl.push_str("(allow process-fork)\n");
        sbpl.push_str("(allow process-exec (subpath \"/bin\"))\n");
        sbpl.push_str("(allow process-exec (subpath \"/usr/bin\"))\n");
    }

    // Deny Mach privilege escalation
    sbpl.push_str("(deny mach*)\n");

    sbpl
}

#[cfg(target_os = "linux")]
fn init_linux_sandbox(config: &SandboxConfig) -> Result<(), String> {
    // 1. Landlock: filesystem restrictions
    let mut landlock_rules = Vec::new();
    for p in &config.allow_read {
        landlock_rules.push(landlock::LandlockRule::PathRead(p.clone()));
    }
    for p in &config.allow_write {
        landlock_rules.push(landlock::LandlockRule::PathWrite(p.clone()));
    }
    if config.level >= SandboxLevel::Minimal {
        landlock_rules.push(landlock::LandlockRule::PathRead("/usr/lib".into()));
        landlock_rules.push(landlock::LandlockRule::PathRead("/usr/share".into()));
        landlock_rules.push(landlock::LandlockRule::PathRead("/tmp".into()));
        landlock_rules.push(landlock::LandlockRule::PathWrite("/tmp".into()));
    }
    if landlock::landlock_available() {
        landlock::enable_landlock(&landlock_rules)?;
        log::info!(
            "[kernel_sandbox] Landlock enabled ({} rules)",
            landlock_rules.len()
        );
    } else {
        log::warn!("[kernel_sandbox] Landlock not available (kernel <5.13 or not enabled)");
    }

    // 2. Seccomp: syscall filtering
    if config.level >= SandboxLevel::Standard {
        let allowed = seccomp::default_allowlist();
        if seccomp::seccomp_available() {
            seccomp::enable_seccomp(&allowed)?;
            log::info!(
                "[kernel_sandbox] seccomp enabled ({} syscalls allowed)",
                allowed.len()
            );
        } else {
            log::warn!("[kernel_sandbox] seccomp not available (kernel or arch not supported)");
        }
    }

    log::info!(
        "[kernel_sandbox] Linux sandbox initialized (level={})",
        config.level.label()
    );
    Ok(())
}

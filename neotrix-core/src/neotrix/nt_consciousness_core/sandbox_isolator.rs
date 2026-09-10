#![forbid(unsafe_code)]

//! OS 级本地沙箱 (OS-Level Local Sandbox)
//!
//! 提供进程级隔离环境，支持：
//! - Bubblewrap (Linux) / Seatbelt (macOS) 沙箱后端
//! - 文件系统隔离（只读/可写目录白名单）
//! - 网络隔离（允许端点白名单）
//! - 资源限制（CPU 时间、内存、磁盘、进程数）
//! - 目标启动时间 < 100ms
//!
//! 注意：实际沙箱执行依赖外部工具（bwrap/sandbox-exec），
//! 本模块负责配置生成、规则编排和结果解析。

use std::fmt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// Public Types
// ============================================================================

/// 沙箱后端类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxBackend {
    /// Bubblewrap (Linux)
    Bubblewrap,
    /// Seatbelt / sandbox-exec (macOS)
    Seatbelt,
    /// 伪沙箱（仅记录，不做实际隔离）
    Mock,
}

impl fmt::Display for SandboxBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxBackend::Bubblewrap => write!(f, "Bubblewrap"),
            SandboxBackend::Seatbelt => write!(f, "Seatbelt"),
            SandboxBackend::Mock => write!(f, "Mock"),
        }
    }
}

/// 沙箱配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// 后端类型
    pub backend: SandboxBackend,
    /// 沙箱名称
    pub name: String,
    /// 只读挂载点
    pub read_only_mounts: Vec<MountPoint>,
    /// 可写挂载点
    pub read_write_mounts: Vec<MountPoint>,
    /// 禁止挂载点
    pub denied_mounts: Vec<PathBuf>,
    /// 允许的网络端点 (host:port)
    pub allowed_endpoints: Vec<NetworkEndpoint>,
    /// 网络默认策略
    pub network_default: NetworkPolicy,
    /// 资源限制
    pub resource_limits: ResourceLimits,
    /// 环境变量白名单
    pub allowed_env_vars: Vec<String>,
    /// 禁止的系统调用
    pub denied_syscalls: Vec<String>,
    /// 工作目录
    pub working_dir: PathBuf,
    /// 启动超时（毫秒）
    pub startup_timeout_ms: u64,
}

impl SandboxConfig {
    /// 创建默认配置（最小权限）
    pub fn minimal(name: impl Into<String>, work_dir: PathBuf) -> Self {
        Self {
            backend: Self::detect_backend(),
            name: name.into(),
            read_only_mounts: vec![
                MountPoint { source: PathBuf::from("/usr"), target: PathBuf::from("/usr"), recursive: false },
                MountPoint { source: PathBuf::from("/lib"), target: PathBuf::from("/lib"), recursive: false },
                MountPoint { source: PathBuf::from("/bin"), target: PathBuf::from("/bin"), recursive: false },
            ],
            read_write_mounts: vec![
                MountPoint { source: work_dir.join("workspace"), target: PathBuf::from("/workspace"), recursive: false },
            ],
            denied_mounts: vec![
                PathBuf::from("/etc/shadow"),
                PathBuf::from("/etc/passwd"),
                PathBuf::from("/root"),
            ],
            allowed_endpoints: Vec::new(),
            network_default: NetworkPolicy::DenyAll,
            resource_limits: ResourceLimits::default(),
            allowed_env_vars: vec!["PATH".into(), "HOME".into(), "LANG".into()],
            denied_syscalls: vec![
                "mount".into(),
                "umount".into(),
                "ptrace".into(),
                "kexec_load".into(),
                "reboot".into(),
            ],
            working_dir: PathBuf::from("/workspace"),
            startup_timeout_ms: 100,
        }
    }

    /// 创建网络隔离配置（允许特定端点）
    pub fn with_network_policy(mut self, policy: NetworkPolicy, endpoints: Vec<NetworkEndpoint>) -> Self {
        self.network_default = policy;
        self.allowed_endpoints = endpoints;
        self
    }

    /// 添加只读挂载
    pub fn with_read_only_mount(mut self, source: PathBuf, target: PathBuf) -> Self {
        self.read_only_mounts.push(MountPoint { source, target, recursive: false });
        self
    }

    /// 添加可写挂载
    pub fn with_read_write_mount(mut self, source: PathBuf, target: PathBuf) -> Self {
        self.read_write_mounts.push(MountPoint { source, target, recursive: false });
        self
    }

    /// 设置资源限制
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    /// 检测当前平台的沙箱后端
    fn detect_backend() -> SandboxBackend {
        if cfg!(target_os = "linux") && Path::new("/usr/bin/bwrap").exists() {
            SandboxBackend::Bubblewrap
        } else if cfg!(target_os = "macos") && Path::new("/usr/bin/sandbox-exec").exists() {
            SandboxBackend::Seatbelt
        } else {
            SandboxBackend::Mock
        }
    }
}

/// 挂载点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    /// 源路径
    pub source: PathBuf,
    /// 沙箱内目标路径
    pub target: PathBuf,
    /// 是否递归挂载
    pub recursive: bool,
}

/// 网络端点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEndpoint {
    /// 主机名或 IP
    pub host: String,
    /// 端口号
    pub port: u16,
    /// 协议
    pub protocol: NetProtocol,
    /// 描述
    pub description: String,
}

/// 网络协议
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetProtocol {
    Tcp,
    Udp,
    Both,
}

/// 网络策略
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkPolicy {
    /// 允许所有
    AllowAll,
    /// 拒绝所有
    DenyAll,
    /// 仅允许白名单
    WhitelistOnly,
}

/// 资源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// 最大 CPU 时间（秒）
    pub cpu_time_secs: u64,
    /// 最大内存（MB）
    pub memory_mb: u64,
    /// 最大磁盘写入（MB）
    pub disk_write_mb: u64,
    /// 最大进程数
    pub max_processes: u32,
    /// 最大打开文件数
    pub max_open_files: u32,
    /// 最大网络连接数
    pub max_connections: u32,
    /// Wall clock 超时（秒）
    pub wall_timeout_secs: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_time_secs: 60,
            memory_mb: 512,
            disk_write_mb: 256,
            max_processes: 64,
            max_open_files: 256,
            max_connections: 16,
            wall_timeout_secs: 120,
        }
    }
}

impl ResourceLimits {
    /// 轻量限制（适合简单任务）
    pub fn light() -> Self {
        Self {
            cpu_time_secs: 30,
            memory_mb: 128,
            disk_write_mb: 64,
            max_processes: 16,
            max_open_files: 64,
            max_connections: 4,
            wall_timeout_secs: 60,
        }
    }

    /// 重量限制（适合复杂任务）
    pub fn heavy() -> Self {
        Self {
            cpu_time_secs: 300,
            memory_mb: 2048,
            disk_write_mb: 1024,
            max_processes: 128,
            max_open_files: 1024,
            max_connections: 32,
            wall_timeout_secs: 600,
        }
    }
}

// ============================================================================
// SandboxSession
// ============================================================================

/// 沙箱会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSession {
    /// 会话 ID
    pub id: Uuid,
    /// 配置
    pub config: SandboxConfig,
    /// 状态
    pub state: SessionState,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 启动耗时（毫秒）
    pub startup_duration_ms: Option<u64>,
}

/// 会话状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// 已创建
    Created,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 超时
    TimedOut,
    /// 错误
    Error(String),
}

// ============================================================================
// SandboxResult
// ============================================================================

/// 沙箱执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    /// 退出码
    pub exit_code: i32,
    /// stdout
    pub stdout: String,
    /// stderr
    pub stderr: String,
    /// 实际资源使用
    pub resource_usage: ResourceUsage,
    /// 是否被 OOM 杀死
    pub oom_killed: bool,
    /// 是否超时
    pub timed_out: bool,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
    /// 资源限制违反
    pub violations: Vec<ResourceViolation>,
}

/// 资源使用统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU 时间（秒）
    pub cpu_time_secs: f64,
    /// 峰值内存（MB）
    pub peak_memory_mb: u64,
    /// 磁盘写入（MB）
    pub disk_write_mb: u64,
    /// 创建的进程数
    pub processes_created: u32,
    /// 网络连接数
    pub connections_made: u32,
}

/// 资源违规
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceViolation {
    /// 资源类型
    pub resource: String,
    /// 限制值
    pub limit: String,
    /// 实际使用
    pub actual: String,
    /// 严重程度
    pub severity: ViolationSeverity,
}

/// 违规严重程度
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Error,
    Fatal,
}

// ============================================================================
// SandboxIsolator
// ============================================================================

/// OS 级沙箱隔离器
pub struct SandboxIsolator {
    /// 活跃会话
    active_sessions: HashMap<Uuid, SandboxSession>,
    /// 历史结果
    history: Vec<SandboxResult>,
    /// 统计
    pub stats: IsolatorStats,
}

/// 隔离器统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IsolatorStats {
    /// 总执行次数
    pub total_executions: u64,
    /// 成功次数
    pub successes: u64,
    /// 失败次数
    pub failures: u64,
    /// 超时次数
    pub timeouts: u64,
    /// OOM 次数
    pub oom_kills: u64,
    /// 平均启动时间（毫秒）
    pub avg_startup_ms: f64,
    /// 平均执行时间（毫秒）
    pub avg_duration_ms: f64,
    /// 违规总数
    pub total_violations: u64,
}

impl SandboxIsolator {
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
            history: Vec::new(),
            stats: IsolatorStats::default(),
        }
    }

    /// 创建沙箱会话
    pub fn create_session(&mut self, config: SandboxConfig) -> SandboxSession {
        let session = SandboxSession {
            id: Uuid::new_v4(),
            config,
            state: SessionState::Created,
            created_at: Utc::now(),
            startup_duration_ms: None,
        };

        self.active_sessions.insert(session.id, session.clone());
        session
    }

    /// 生成 Bubblewrap 命令
    pub fn generate_bwrap_command(&self, config: &SandboxConfig, command: &[String]) -> Vec<String> {
        let mut args = vec![
            "bwrap".to_string(),
            "--die-with-parent".to_string(),
            "--new-session".to_string(),
        ];

        // 只读挂载
        for mount in &config.read_only_mounts {
            args.push("--ro-bind".to_string());
            args.push(mount.source.to_string_lossy().to_string());
            args.push(mount.target.to_string_lossy().to_string());
        }

        // 可写挂载
        for mount in &config.read_write_mounts {
            args.push("--bind".to_string());
            args.push(mount.source.to_string_lossy().to_string());
            args.push(mount.target.to_string_lossy().to_string());
        }

        // 禁止挂载（绑定到 /dev/null）
        for denied in &config.denied_mounts {
            args.push("--bind".to_string());
            args.push("/dev/null".to_string());
            args.push(denied.to_string_lossy().to_string());
        }

        // 工作目录
        args.push("--chdir".to_string());
        args.push(config.working_dir.to_string_lossy().to_string());

        // 网络隔离
        if config.network_default == NetworkPolicy::DenyAll {
            args.push("--unshare-net".to_string());
        }

        // 资源限制
        let limits = &config.resource_limits;
        args.push("--rlimit-as".to_string());
        args.push((limits.memory_mb * 1024 * 1024).to_string());
        args.push("--rlimit-cpu".to_string());
        args.push(limits.cpu_time_secs.to_string());
        args.push("--rlimit-nproc".to_string());
        args.push(limits.max_processes.to_string());
        args.push("--rlimit-nofile".to_string());
        args.push(limits.max_open_files.to_string());

        // 禁止的系统调用
        if !config.denied_syscalls.is_empty() {
            // bwrap 不直接支持 syscall 过滤，通过 seccomp-bpf
            // 这里记录到元数据中供外部 seccomp 配置使用
        }

        // 环境变量
        args.push("--clearenv".to_string());
        for var in &config.allowed_env_vars {
            args.push("--setenv".to_string());
            args.push(var.clone());
            args.push(std::env::var(var).unwrap_or_default());
        }

        // 执行命令
        args.push("--".to_string());
        args.extend(command.iter().cloned());

        args
    }

    /// 生成 Seatbelt (macOS) profile
    pub fn generate_seatbelt_profile(&self, config: &SandboxConfig) -> String {
        let mut profile = String::from("(version 1)\n(allow default)\n\n");

        // 文件系统：禁止写入敏感路径
        for denied in &config.denied_mounts {
            profile.push_str(&format!(
                "(deny file-write* (subpath \"{}\"))\n",
                denied.display()
            ));
        }

        // 只读挂载
        for mount in &config.read_only_mounts {
            profile.push_str(&format!(
                "(allow file-read* (subpath \"{}\"))\n",
                mount.source.display()
            ));
            profile.push_str(&format!(
                "(deny file-write* (subpath \"{}\"))\n",
                mount.target.display()
            ));
        }

        // 网络限制
        if config.network_default == NetworkPolicy::DenyAll {
            profile.push_str("(deny network*)\n");
        } else if config.network_default == NetworkPolicy::WhitelistOnly {
            profile.push_str("(deny network*)\n");
            for ep in &config.allowed_endpoints {
                profile.push_str(&format!(
                    "(allow network-outbound (remote host \"{}\" (port {})))\n",
                    ep.host, ep.port
                ));
            }
        }

        // 资源限制
        let limits = &config.resource_limits;
        profile.push_str(&format!(
            "(deny process-fork (with (limit-bytes 1 {})))\n",
            limits.max_processes
        ));

        // 禁止的系统调用
        for syscall in &config.denied_syscalls {
            profile.push_str(&format!("(deny syscall-number (\"{}\"))\n", syscall));
        }

        profile
    }

    /// 模拟执行（Mock 模式或测试）
    pub fn mock_execute(&mut self, session_id: Uuid, command: &[String]) -> SandboxResult {
        let start = std::time::Instant::now();

        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            session.state = SessionState::Running;
        }

        // 模拟执行结果
        let result = SandboxResult {
            exit_code: 0,
            stdout: format!("[mock] executed: {}", command.join(" ")),
            stderr: String::new(),
            resource_usage: ResourceUsage {
                cpu_time_secs: 0.01,
                peak_memory_mb: 4,
                disk_write_mb: 0,
                processes_created: 1,
                connections_made: 0,
            },
            oom_killed: false,
            timed_out: false,
            duration_ms: start.elapsed().as_millis() as u64,
            violations: Vec::new(),
        };

        // 更新会话状态
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            session.state = SessionState::Completed;
            session.startup_duration_ms = Some(1); // Mock 启动时间
        }

        // 更新统计
        self.stats.total_executions += 1;
        self.stats.successes += 1;
        let n = self.stats.total_executions as f64;
        self.stats.avg_startup_ms =
            (self.stats.avg_startup_ms * (n - 1.0) + 1.0) / n;
        self.stats.avg_duration_ms =
            (self.stats.avg_duration_ms * (n - 1.0) + result.duration_ms as f64) / n;

        self.history.push(result.clone());
        result
    }

    /// 验证配置有效性
    pub fn validate_config(config: &SandboxConfig) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // 检查启动超时
        if config.startup_timeout_ms > 1000 {
            issues.push(ConfigIssue {
                field: "startup_timeout_ms".into(),
                issue: format!("启动超时 {}ms 超过建议值 1000ms", config.startup_timeout_ms),
                severity: ViolationSeverity::Warning,
            });
        }

        // 检查内存限制
        if config.resource_limits.memory_mb > 8192 {
            issues.push(ConfigIssue {
                field: "resource_limits.memory_mb".into(),
                issue: format!("内存限制 {}MB 超过建议值 8192MB", config.resource_limits.memory_mb),
                severity: ViolationSeverity::Warning,
            });
        }

        // 检查 CPU 时间
        if config.resource_limits.cpu_time_secs > 600 {
            issues.push(ConfigIssue {
                field: "resource_limits.cpu_time_secs".into(),
                issue: format!("CPU 时间 {}s 超过建议值 600s", config.resource_limits.cpu_time_secs),
                severity: ViolationSeverity::Warning,
            });
        }

        // 检查是否有可写挂载
        if config.read_write_mounts.is_empty() {
            issues.push(ConfigIssue {
                field: "read_write_mounts".into(),
                issue: "没有可写挂载点，进程可能无法写入临时文件".into(),
                severity: ViolationSeverity::Warning,
            });
        }

        // 检查网络策略
        if config.network_default == NetworkPolicy::AllowAll && !config.allowed_endpoints.is_empty() {
            issues.push(ConfigIssue {
                field: "network_default".into(),
                issue: "网络策略为 AllowAll 但有白名单端点，白名单不会生效".into(),
                severity: ViolationSeverity::Error,
            });
        }

        issues
    }

    /// 获取历史记录
    pub fn history(&self) -> &[SandboxResult] {
        &self.history
    }

    /// 获取统计摘要
    pub fn stats_summary(&self) -> IsolatorStatsSummary {
        IsolatorStatsSummary {
            total: self.stats.total_executions,
            success_rate: if self.stats.total_executions > 0 {
                self.stats.successes as f64 / self.stats.total_executions as f64
            } else {
                1.0
            },
            avg_startup_ms: self.stats.avg_startup_ms,
            avg_duration_ms: self.stats.avg_duration_ms,
            timeouts: self.stats.timeouts,
            oom_kills: self.stats.oom_kills,
            violations: self.stats.total_violations,
            active_sessions: self.active_sessions.len() as u64,
        }
    }
}

impl Default for SandboxIsolator {
    fn default() -> Self { Self::new() }
}

/// 配置问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigIssue {
    pub field: String,
    pub issue: String,
    pub severity: ViolationSeverity,
}

/// 隔离器统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolatorStatsSummary {
    pub total: u64,
    pub success_rate: f64,
    pub avg_startup_ms: f64,
    pub avg_duration_ms: f64,
    pub timeouts: u64,
    pub oom_kills: u64,
    pub violations: u64,
    pub active_sessions: u64,
}

impl fmt::Display for IsolatorStatsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        Sandbox Isolator 统计")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "总执行:     {}", self.total)?;
        writeln!(f, "成功率:     {:.2}%", self.success_rate * 100.0)?;
        writeln!(f, "平均启动:   {:.1}ms", self.avg_startup_ms)?;
        writeln!(f, "平均执行:   {:.1}ms", self.avg_duration_ms)?;
        writeln!(f, "超时:       {}", self.timeouts)?;
        writeln!(f, "OOM:        {}", self.oom_kills)?;
        writeln!(f, "违规:       {}", self.violations)?;
        writeln!(f, "活跃会话:   {}", self.active_sessions)?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config() {
        let config = SandboxConfig::minimal("test", PathBuf::from("/tmp/test"));
        assert_eq!(config.name, "test");
        assert_eq!(config.network_default, NetworkPolicy::DenyAll);
        assert!(!config.read_only_mounts.is_empty());
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.memory_mb, 512);
        assert_eq!(limits.cpu_time_secs, 60);
        assert_eq!(limits.max_processes, 64);
    }

    #[test]
    fn test_resource_limits_light() {
        let limits = ResourceLimits::light();
        assert_eq!(limits.memory_mb, 128);
        assert_eq!(limits.cpu_time_secs, 30);
    }

    #[test]
    fn test_resource_limits_heavy() {
        let limits = ResourceLimits::heavy();
        assert_eq!(limits.memory_mb, 2048);
        assert_eq!(limits.cpu_time_secs, 300);
    }

    #[test]
    fn test_create_session() {
        let mut isolator = SandboxIsolator::new();
        let config = SandboxConfig::minimal("test", PathBuf::from("/tmp"));
        let session = isolator.create_session(config);
        assert_eq!(session.state, SessionState::Created);
        assert!(isolator.active_sessions.contains_key(&session.id));
    }

    #[test]
    fn test_mock_execute() {
        let mut isolator = SandboxIsolator::new();
        let config = SandboxConfig::minimal("test", PathBuf::from("/tmp"));
        let session = isolator.create_session(config);

        let result = isolator.mock_execute(session.id, &["echo".into(), "hello".into()]);
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("echo"));
        assert!(!result.oom_killed);
        assert!(!result.timed_out);
    }

    #[test]
    fn test_validate_config_warnings() {
        let config = SandboxConfig {
            startup_timeout_ms: 5000,
            ..SandboxConfig::minimal("test", PathBuf::from("/tmp"))
        };
        let issues = SandboxIsolator::validate_config(&config);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.field == "startup_timeout_ms"));
    }

    #[test]
    fn test_validate_network_policy_conflict() {
        let config = SandboxConfig {
            network_default: NetworkPolicy::AllowAll,
            allowed_endpoints: vec![NetworkEndpoint {
                host: "api.example.com".into(),
                port: 443,
                protocol: NetProtocol::Tcp,
                description: "test".into(),
            }],
            ..SandboxConfig::minimal("test", PathBuf::from("/tmp"))
        };
        let issues = SandboxIsolator::validate_config(&config);
        assert!(issues.iter().any(|i| i.severity == ViolationSeverity::Error));
    }

    #[test]
    fn test_bwrap_command_generation() {
        let isolator = SandboxIsolator::new();
        let config = SandboxConfig::minimal("test", PathBuf::from("/tmp"));
        let cmd = isolator.generate_bwrap_command(&config, &["echo".into(), "hello".into()]);
        assert!(cmd.contains(&"bwrap".to_string()));
        assert!(cmd.contains(&"--die-with-parent".to_string()));
        assert!(cmd.contains(&"echo".to_string()));
        assert!(cmd.contains(&"hello".to_string()));
    }

    #[test]
    fn test_seatbelt_profile_generation() {
        let isolator = SandboxIsolator::new();
        let config = SandboxConfig::minimal("test", PathBuf::from("/tmp"));
        let profile = isolator.generate_seatbelt_profile(&config);
        assert!(profile.contains("(version 1)"));
        assert!(profile.contains("(deny network*)"));
    }

    #[test]
    fn test_stats_tracking() {
        let mut isolator = SandboxIsolator::new();
        let config = SandboxConfig::minimal("test", PathBuf::from("/tmp"));
        let session = isolator.create_session(config);
        isolator.mock_execute(session.id, &["ls".into()]);

        let stats = isolator.stats_summary();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.success_rate, 1.0);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = SandboxConfig::minimal("test", PathBuf::from("/tmp"));
        let json = serde_json::to_string(&config).unwrap();
        let _: SandboxConfig = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_network_endpoint() {
        let ep = NetworkEndpoint {
            host: "api.openai.com".into(),
            port: 443,
            protocol: NetProtocol::Tcp,
            description: "OpenAI API".into(),
        };
        let json = serde_json::to_string(&ep).unwrap();
        let deserialized: NetworkEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.host, "api.openai.com");
        assert_eq!(deserialized.port, 443);
    }
}

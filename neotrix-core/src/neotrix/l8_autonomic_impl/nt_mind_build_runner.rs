//! BuildRunner — 统一 cargo 工具层 (GAP-4 修复, 设计文档 L1/L2/L3 分层映射)。
//!
//! 设计文档 "Rust 多智能体自审计系统" 的确定性工具层在本项目落地为分层 cargo 调用:
//! - L1 Fast: check / clippy / fmt --check / test --lib / tree / metadata (秒级, 每次可用)
//! - L2 Audit: audit / deny check / outdated (低频, 安全 + 供应链)
//! - L3 Heavy: llvm-cov / miri (重型, 显式触发)
//!
//! 相比既有散落调用 (behavioral_verifier::run_bounded / AutoFixer::cargo_check /
//! self_audit d42 / safe_applier / nt_shield::audit), 本模块统一:
//! 1. 超时 + kill (防 cargo 构建锁/网络挂起卡死持有全局锁的后台 handler)
//! 2. 证据收集 (exit/errors/warnings/stdout/stderr → 结构化 BuildEvidence)
//! 3. Denylist gate (fail-closed 阻断破坏性 cargo 子命令: publish/vendor/install)
//!
//! 设计约束对齐: Deterministic Tools First — 工具层只用确定性 CLI, 不引入 LLM 语义推理。
//! 破坏性/高风险子命令不在此层 (与 NT-SHIELD 安全分层一致)。

use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// 工具层 — 映射设计文档 L1/L2/L3。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildLayer {
    /// L1 快速确定性层 — check/clippy/fmt/test/tree/metadata
    Fast,
    /// L2 审计层 — audit/deny/outdated (安全 + 供应链)
    Audit,
    /// L3 重型层 — llvm-cov/miri (显式触发)
    Heavy,
}

impl BuildLayer {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fast => "L1-fast",
            Self::Audit => "L2-audit",
            Self::Heavy => "L3-heavy",
        }
    }
}

/// L1 快速工具 (确定性, 每次可用)。
pub const L1_TOOLS: &[&str] = &["check", "clippy", "fmt", "test", "tree", "metadata"];
/// L2 审计工具 (低频, 安全 + 供应链)。
pub const L2_TOOLS: &[&str] = &["audit", "deny", "outdated"];
/// L3 重型工具 (显式触发)。
pub const L3_TOOLS: &[&str] = &["llvm-cov", "miri", "expand"];

/// 破坏性 cargo 子命令 — fail-closed 阻断 (R-P1 精神: 零 unsafe 之外, 零破坏性 cargo)。
const DENYLISTED_SUBCOMMANDS: &[&str] = &["publish", "install", "vendor", "clean", "uninstall"];

/// 构建结果证据 — 结构化收集 (exit/计数/输出), 供调用方作为 R-P9/R-P16 双验证证据。
#[derive(Debug, Clone)]
pub struct BuildEvidence {
    pub layer: BuildLayer,
    pub tool: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

impl BuildEvidence {
    pub fn success(&self) -> bool {
        !self.timed_out && self.exit_code == Some(0) && self.error_count == 0
    }

    pub fn summary(&self) -> String {
        let status = if self.timed_out {
            format!("TIMEOUT ({:?})", self.duration_ms)
        } else {
            format!("exit={:?}", self.exit_code)
        };
        format!(
            "[{}] {} {:?}: {} | {} errors, {} warnings",
            self.layer.label(),
            self.tool,
            self.args,
            status,
            self.error_count,
            self.warning_count
        )
    }
}

/// 统一 cargo 构建执行器。
#[derive(Debug, Clone)]
pub struct BuildRunner {
    /// 默认超时秒数 (L1 短, L2/L3 长)。
    pub timeout_secs: u64,
    /// 工作目录 (None = 进程当前目录)。
    pub workdir: Option<std::path::PathBuf>,
}

impl Default for BuildRunner {
    fn default() -> Self {
        Self {
            timeout_secs: 300,
            workdir: None,
        }
    }
}

impl BuildRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_workdir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.workdir = Some(dir.as_ref().to_path_buf());
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Denylist gate — fail-closed: 阻断破坏性/污染性 cargo 子命令。
    pub fn is_blocked(tool: &str) -> bool {
        DENYLISTED_SUBCOMMANDS.contains(&tool)
    }

    /// 运行一个构建工具 (layer 从工具名推断)。
    pub fn run(&self, tool: &str, extra_args: &[&str]) -> Result<BuildEvidence, String> {
        let layer = if L1_TOOLS.contains(&tool) {
            BuildLayer::Fast
        } else if L2_TOOLS.contains(&tool) {
            BuildLayer::Audit
        } else if L3_TOOLS.contains(&tool) {
            BuildLayer::Heavy
        } else {
            // 未知工具拒绝执行 (Deterministic Tools First: 只跑已知确定性工具)
            return Err(format!(
                "BuildRunner: unknown cargo tool '{tool}' (must be in L1/L2/L3)"
            ));
        };
        let timeout = match layer {
            BuildLayer::Fast => self.timeout_secs.min(180),
            BuildLayer::Audit => self.timeout_secs,
            BuildLayer::Heavy => self.timeout_secs.max(600),
        };
        self.run_internal(layer, tool, extra_args, timeout)
    }

    /// 完整执行: 超时 kill + 证据收集。
    fn run_internal(
        &self,
        layer: BuildLayer,
        tool: &str,
        extra_args: &[&str],
        timeout_secs: u64,
    ) -> Result<BuildEvidence, String> {
        let start = Instant::now();

        // Denylist gate (fail-closed)
        if Self::is_blocked(tool) {
            return Err(format!(
                "BuildRunner: tool '{tool}' is on the destructive denylist — blocked (fail-closed)"
            ));
        }

        let mut cmd = Command::new("cargo");
        cmd.arg(tool);
        cmd.args(extra_args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        if let Some(dir) = &self.workdir {
            cmd.current_dir(dir);
        }

        let child = std::sync::Arc::new(std::sync::Mutex::new(
            cmd.spawn()
                .map_err(|e| format!("cargo {tool} spawn failed: {}", e))?,
        ));
        let (tx, rx) = std::sync::mpsc::channel::<Output>();
        let c2 = child.clone();
        std::thread::spawn(move || {
            // 轮询退出 → 收集输出 → 发送
            loop {
                let exited = {
                    let mut g = c2.lock().unwrap_or_else(|e| e.into_inner());
                    g.try_wait().ok().flatten().is_some()
                };
                if exited {
                    break;
                }
                std::thread::sleep(Duration::from_millis(150));
            }
            let out = {
                let mut g = c2.lock().unwrap_or_else(|e| e.into_inner());
                let status = g.wait();
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(s) = g.stdout.as_mut() {
                    let _ = std::io::Read::read_to_end(s, &mut stdout);
                }
                if let Some(e) = g.stderr.as_mut() {
                    let _ = std::io::Read::read_to_end(e, &mut stderr);
                }
                status.map(|st| Output {
                    status: st,
                    stdout,
                    stderr,
                })
            };
            if let Ok(output) = out {
                let _ = tx.send(output);
            }
        });

        match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let all = format!("{stdout}\n{stderr}");
                let error_count = all.matches("error[").count() + all.matches("error:").count();
                let warning_count = all.matches("warning:").count();
                Ok(BuildEvidence {
                    layer,
                    tool: tool.to_string(),
                    args: extra_args.iter().map(|s| s.to_string()).collect(),
                    exit_code: output.status.code(),
                    timed_out: false,
                    error_count,
                    warning_count,
                    stdout,
                    stderr,
                    duration_ms: start.elapsed().as_millis() as u64,
                })
            }
            Err(_) => {
                // 超时: kill 子进程 (reader 线程轮询到退出后自行结束)
                if let Ok(mut g) = child.lock() {
                    let _ = g.kill();
                    let _ = g.wait();
                }
                Ok(BuildEvidence {
                    layer,
                    tool: tool.to_string(),
                    args: extra_args.iter().map(|s| s.to_string()).collect(),
                    exit_code: None,
                    timed_out: true,
                    error_count: 0,
                    warning_count: 0,
                    stdout: String::new(),
                    stderr: "timeout: cargo killed".into(),
                    duration_ms: start.elapsed().as_millis() as u64,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denylist_blocks_destructive() {
        assert!(BuildRunner::is_blocked("publish"));
        assert!(BuildRunner::is_blocked("install"));
        assert!(BuildRunner::is_blocked("vendor"));
        assert!(!BuildRunner::is_blocked("check"));
        assert!(!BuildRunner::is_blocked("test"));
    }

    #[test]
    fn unknown_tool_rejected() {
        let r = BuildRunner::new();
        let res = r.run("totally-bogus-tool", &[]);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("unknown"));
    }

    #[test]
    fn layer_inference() {
        // env-gated: spawn 真实 cargo, 全量并行时与外部 cargo 会话竞争 target 锁
        if std::env::var("NT_E2E_CARGO")
            .map(|v| v == "1")
            .unwrap_or(false)
            != true
        {
            eprintln!("skipped: set NT_E2E_CARGO=1 to run real-cargo layer inference");
            return;
        }
        assert_eq!(
            BuildRunner::new().run("check", &["--lib"]).map(|e| e.layer),
            Ok(BuildLayer::Fast)
        );
        assert_eq!(
            BuildRunner::new().run("audit", &[]).map(|e| e.layer),
            Ok(BuildLayer::Audit)
        );
        assert_eq!(
            BuildRunner::new().run("llvm-cov", &[]).map(|e| e.layer),
            Ok(BuildLayer::Heavy)
        );
    }

    #[test]
    fn run_check_collects_evidence() {
        // env-gated: spawn 真实 cargo check (可能慢; 小 target 下应秒级完成)
        if std::env::var("NT_E2E_CARGO")
            .map(|v| v == "1")
            .unwrap_or(false)
            != true
        {
            eprintln!("skipped: set NT_E2E_CARGO=1 to run real-cargo evidence collection");
            return;
        }
        let runner = BuildRunner::new().with_timeout(300);
        match runner.run("check", &["--lib"]) {
            Ok(ev) => {
                assert!(!ev.timed_out);
                assert_eq!(ev.tool, "check");
                assert!(!ev.stdout.is_empty() || !ev.stderr.is_empty());
            }
            Err(e) => panic!("check should run: {e}"),
        }
    }

    #[test]
    fn summary_format() {
        let ev = BuildEvidence {
            layer: BuildLayer::Fast,
            tool: "check".into(),
            args: vec!["--lib".into()],
            exit_code: Some(0),
            timed_out: false,
            error_count: 0,
            warning_count: 2,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 42,
        };
        assert!(ev.summary().contains("[L1-fast]"));
        assert!(ev.summary().contains("check"));
        assert!(ev.summary().contains("0 errors, 2 warnings"));
        assert!(ev.success());
    }
}

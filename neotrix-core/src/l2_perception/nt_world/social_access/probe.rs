//! Backend health probing utilities
//!
//! Provides CLI-based health checking for social platform backends.
//! Modeled after Agent-Reach's probe_command pattern.
//!
//! # ⛔ 历史缺陷：假 timeout（2026-10-03 修复）
//!
//! 原实现把 `child.wait_with_output()` 丢进 `thread::spawn` 然后 `.join()`，
//! 并把它当作「等待超时」处理：
//!
//! ```text
//! let wait_result = std::thread::spawn(move || { child.wait_with_output() }).join();
//! ```
//!
//! **`.join()` 会阻塞到子进程结束为止，`.Err(_)` 分支（唯一产生 `Timeout`
//! 的地方）只在子进程 `panic` 时才可达。** 于是一个 `sleep 3600` 的后端
//! 会让探测永久挂死，`Backend::probe_timeout` 形同虚设。
//!
//! 本仓实测复现：编译等价程序跑 `sleep 3600`，`join` 在 300ms 后仍阻塞，
//! 进程只能被外部 `kill`。⇒ 这不是理论风险，是可挂死整条 doctor 链的 P0。
//!
//! **修法**：`Child::try_wait()` 轮询 + 到期 `kill()`，真超时。
//! 不引入 `wait-timeout` crate（零新依赖，符合本仓 minimal-change 惯例）。

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::channel::{Backend, ProbeResult};

/// `try_wait` 轮询间隔。取值权衡见 [`run_with_timeout`]。
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// 带**真实超时**地运行一个命令，返回其输出。
///
/// 与历史实现的关键差异：超时由本函数自己判定并 `kill()` 子进程，
/// 不依赖任何「等待线程是否结束」的间接信号。
///
/// ## 为什么轮询而不是 `wait_timeout` crate
///
/// 零新依赖。轮询代价是至多 `POLL_INTERVAL` 的响应延迟，对健康探测
/// （秒级命令）完全可接受，而引入依赖要过 feature 门控与供应链审查。
///
/// ## 管道死锁防护
///
/// 子进程的 stdout/stderr 若填满管道缓冲区就会永久阻塞写入方。
/// 因此超时 `kill()` 之后**必须继续读取两条管道**，否则可能挂死。
/// 管道大小在 macOS/Linux 上通常 64 KiB，CLI 探测输出远小于此，
/// 但 `kill()` 与读取之间仍存在竞态窗口，故此处不依赖「输出一定很小」。
pub fn run_with_timeout(
    cmd: &str,
    args: &[String],
    timeout: Duration,
) -> Result<RunOutcome, std::io::Error> {
    let start = Instant::now();
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // 先取走管道（所有权移出 child），再进入等待循环。
    //
    // **必须并发读取，不能等到子进程退出后再读。**
    // ⛔ 本实现第一版把读取放在等待循环**之后**，被自测抓到：管道缓冲区
    //    （macOS/Linux 通常 64 KiB）填满后子进程阻塞在 write 上，
    //    `try_wait()` 于是永远返回 `Ok(None)`，最后被判成超时 ——
    //    即「大输出 = 假超时」。回归测试 `test_large_output_does_not_deadlock`
    //    守的就是这条。
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let stdout_rx = spawn_reader(stdout_pipe);
    let stderr_rx = spawn_reader(stderr_pipe);

    let deadline = start + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    // kill 后必须 reap，否则留下僵尸进程。
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(POLL_INTERVAL);
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                // 读者线程仍在跑；它们会在管道关闭后自行结束。
                let _ = stdout_rx;
                let _ = stderr_rx;
                return Err(e);
            }
        }
    };

    // 超时分支下 `status` 为 None，此时子进程已被 kill，
    // 读者线程会在管道关闭后立刻返回（残余输出通常为空）。
    let stdout = join_reader(stdout_rx);
    let stderr = join_reader(stderr_rx);

    Ok(RunOutcome {
        success: status.is_some_and(|s| s.success()),
        stdout,
        stderr,
        latency: start.elapsed(),
        timed_out: status.is_none(),
    })
}

/// 一次带超时运行的结局。
#[derive(Debug, Clone)]
pub struct RunOutcome {
    /// 子进程是否以 0 退出。
    pub success: bool,
    /// stdout（UTF-8 有损转换）。
    pub stdout: String,
    /// stderr（UTF-8 有损转换）。
    pub stderr: String,
    /// 实测耗时。
    pub latency: Duration,
    /// 是否因超时被 kill。
    pub timed_out: bool,
}

/// 启动一个后台线程把管道读干净（读到 EOF 为止）。
///
/// 返回 `None` 表示调用方未提供管道。
fn spawn_reader<R>(pipe: Option<R>) -> Option<std::thread::JoinHandle<String>>
where
    R: std::io::Read + Send + 'static,
{
    pipe.map(|mut p| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = p.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        })
    })
}

/// 回收读者线程。线程 panic 时返回空串而非传播 panic。
fn join_reader(handle: Option<std::thread::JoinHandle<String>>) -> String {
    handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default()
}

/// Probe a command with an explicit timeout.
pub fn probe_command_with_timeout(
    cmd: &str,
    args: &[String],
    timeout: Duration,
) -> ProbeResult {
    let start = Instant::now();

    match run_with_timeout(cmd, args, timeout) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            ProbeResult::missing(format!("{}: command not found", cmd))
        }
        Err(e) => ProbeResult::error(e.to_string(), start.elapsed().as_millis() as u64),
        Ok(out) if out.timed_out => ProbeResult::timeout(),
        Ok(out) => {
            let latency_ms = start.elapsed().as_millis() as u64;

            // **登录墙判别必须排在 `success` 之前**（自测抓到）。
            // ⛔ 第一版把它放在 `Ok(out) if out.success` 分支**之后**，
            //    于是退出码为 0 的 HTML 输出先被 `success` 臂匹配并返回 Ok，
            //    这个判别对**它唯一想覆盖的情形**完全不可达。
            //
            //    这正是 bird/opencli 未登录时的形态：`exit 0` + 登录页 HTML。
            let combined = if out.stdout.trim().is_empty() {
                out.stderr.clone()
            } else {
                format!("{}\n{}", out.stdout, out.stderr)
            };
            if super::nt_payload_guard::is_login_walled("text/plain", &combined) {
                return ProbeResult::warn(
                    format!(
                        "installed but not authenticated (HTML login page from {}); \
                         run the login flow first",
                        cmd
                    ),
                    latency_ms,
                );
            }

            if out.success {
                ProbeResult::ok(out.stdout.trim().to_string(), latency_ms)
            } else {
                ProbeResult::error(out.stderr.trim().to_string(), latency_ms)
            }
        }
    }
}

/// Probe a command using the default probe timeout.
pub fn probe_command(cmd: &str, args: &[String]) -> ProbeResult {
    probe_command_with_timeout(cmd, args, DEFAULT_PROBE_TIMEOUT)
}

/// 默认探测超时。与 `Backend::new` 的 `probe_timeout` 默认值保持一致。
pub const DEFAULT_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Probe all backends for a channel, return ordered results
pub fn probe_backends(
    backends: &[Backend],
    _config: &HashMap<String, String>,
) -> Vec<(String, ProbeResult)> {
    let mut results = Vec::new();

    for backend in backends {
        // 每个后端用自己的 `probe_timeout`（此前被忽略 ⇒ 真超时形同虚设）
        let result = probe_command_with_timeout(
            &backend.probe_cmd,
            &backend.probe_args,
            backend.probe_timeout,
        );
        results.push((backend.name.clone(), result));
    }

    results
}

/// Find the best available backend from a list
pub fn find_best_backend(backends: &[Backend]) -> Option<&Backend> {
    // Sort by weight (descending) and cost_tier (ascending)
    let mut sorted: Vec<&Backend> = backends.iter().collect();
    sorted.sort_by(|a, b| {
        b.weight.cmp(&a.weight)
            .then(a.cost_tier.cmp(&b.cost_tier))
    });

    for backend in sorted {
        let result = probe_command_with_timeout(
            &backend.probe_cmd,
            &backend.probe_args,
            backend.probe_timeout,
        );

        if result.status.is_healthy() {
            return Some(backend);
        }
    }

    None
}

/// Check if a specific command is available
pub fn command_exists(cmd: &str) -> bool {
    // ⛔ 不再用 `which`：`which` 是外部二进制，在精简镜像 / chroot / 沙箱里常常不存在，
    //    且依赖 `$PATH` 查找语义。直接 spawn 目标命令更可靠 ——
    //    这也是 Agent-Reach「真实探测非命令存在性」的同一原则。
    //
    // ⛔ 用 `--version` 而非裸执行：裸执行 `bird` 这类交互式 CLI 会挂住等输入。
    //    这里只关心「能否启动」，不关心它打印什么。
    matches!(
        run_with_timeout(cmd, &["--version".to_string()], DEFAULT_PROBE_TIMEOUT),
        Ok(o) if !o.timed_out
    )
}

/// Get version string for a command
pub fn get_version(cmd: &str, version_arg: &str) -> Option<String> {
    let out = run_with_timeout(cmd, &[version_arg.to_string()], DEFAULT_PROBE_TIMEOUT).ok()?;
    if out.timed_out || !out.success {
        return None;
    }
    let trimmed = out.stdout.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_command_exists() {
        let result = probe_command("echo", &["hello".into()]);
        assert!(result.status.is_healthy());
        assert_eq!(result.output.as_deref(), Some("hello"));
    }

    #[test]
    fn test_probe_command_missing() {
        let result = probe_command(
            "nonexistent_cmd_xyz_12345",
            &[],
        );
        assert_eq!(result.status, super::super::channel::BackendStatus::Missing);
    }

    #[test]
    fn test_command_exists() {
        assert!(command_exists("echo"));
        assert!(!command_exists("nonexistent_cmd_xyz_12345"));
    }

    #[test]
    fn test_get_version() {
        let version = get_version("echo", "test");
        assert!(version.is_some());
    }

    // ── D1 回归测试：真超时 ────────────────────────────────────────
    //
    // ⛔ 修复前本测试会**永久挂死**：旧实现 `.join()` 阻塞到子进程结束。
    //    挂死测试无法报失败，只会拖住整个套件 —— 缺陷本身的表现形式。
    //    故超时窗口刻意设小（200ms），并断言总耗时 < 5s：
    //    即使未来回归，也是「失败」而非「挂死」。

    #[test]
    fn test_hanging_command_times_out_instead_of_blocking_forever() {
        let started = Instant::now();
        let result =
            probe_command_with_timeout("sleep", &["30".into()], Duration::from_millis(200));
        let elapsed = started.elapsed();

        assert_eq!(result.status, super::super::channel::BackendStatus::Timeout);
        assert!(
            elapsed < Duration::from_secs(5),
            "probe took {:?}, expected <5s (timeout was 200ms)",
            elapsed
        );
    }

    #[test]
    fn test_run_with_timeout_reports_timed_out_flag() {
        let out = run_with_timeout("sleep", &["30".into()], Duration::from_millis(150))
            .expect("spawn of `sleep` should succeed on unix");
        assert!(out.timed_out);
        assert!(!out.success);
    }

    #[test]
    fn test_run_with_timeout_fast_command_not_flagged() {
        let out = run_with_timeout("echo", &["ok".into()], Duration::from_secs(5))
            .expect("spawn of `echo` should succeed on unix");
        assert!(!out.timed_out);
        assert!(out.success);
        assert_eq!(out.stdout.trim(), "ok");
    }

    #[test]
    fn test_run_with_timeout_missing_binary_is_io_error() {
        let err = run_with_timeout(
            "nonexistent_cmd_xyz_12345",
            &[],
            Duration::from_millis(100),
        )
        .expect_err("missing binary must be an Err");
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_large_output_does_not_deadlock() {
        // 管道缓冲死锁防护：输出远超典型 64 KiB 缓冲区。
        let out = run_with_timeout(
            "sh",
            &[
                "-c".into(),
                "i=0; while [ $i -lt 4000 ]; do echo 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'; i=$((i+1)); done"
                    .into(),
            ],
            Duration::from_secs(10),
        )
        .expect("spawn should succeed");
        assert!(!out.timed_out, "large output must not be misread as timeout");
        assert!(out.success);
        assert!(
            out.stdout.len() > 100_000,
            "got only {} bytes",
            out.stdout.len()
        );
    }

    // ── 吸收 OpenCLI：装了但没登录 ≠ 健康 ──────────────────────────

    #[test]
    fn test_html_output_marks_backend_as_not_authenticated() {
        // 模拟 bird/opencli 未登录时把登录页 HTML 打到 stdout 且退出码为 0
        let script = "echo '<!DOCTYPE html><html><body>Log in to X</body></html>'; exit 0";
        let result =
            probe_command_with_timeout("sh", &["-c".into(), script.into()], Duration::from_secs(5));

        match result.status {
            super::super::channel::BackendStatus::Warn(msg) => {
                assert!(msg.contains("not authenticated"), "unexpected hint: {}", msg);
            }
            other => panic!("expected Warn(login wall), got {:?}", other),
        }
    }
}

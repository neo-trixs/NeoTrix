//! Daemon Monitor — 守护进程监控工具
//!
//! 移植自 scripts/daemon-monitor.sh
//! 支持 status/start/stop/restart/log 命令

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// 守护进程监控器
pub struct DaemonMonitor {
    daemon_bin: PathBuf,
    pidfile: PathBuf,
    logfile: PathBuf,
    healthfile: PathBuf,
}

impl DaemonMonitor {
    pub fn new(root: &Path) -> Self {
        let daemon_bin = root.join("target/debug/daemon");
        let pidfile = PathBuf::from("/tmp/neotrix_daemon.pid");
        let logfile = PathBuf::from("/tmp/neotrix/daemon.log");
        let healthfile = PathBuf::from("/tmp/neotrix_daemon.health");

        Self {
            daemon_bin,
            pidfile,
            logfile,
            healthfile,
        }
    }

    /// 获取当前状态
    pub fn status(&self) -> Result<String, String> {
        if self.pidfile.exists() {
            let pid = fs::read_to_string(&self.pidfile)
                .map_err(|e| format!("Failed to read pidfile: {}", e))?
                .trim()
                .to_string();

            // 检查进程是否存在
            let output = Command::new("kill")
                .args(&["-0", &pid])
                .output()
                .map_err(|e| format!("Failed to check process: {}", e))?;

            if output.status.success() {
                let mut result = format!("✅ Daemon running (PID: {})", pid);

                if self.healthfile.exists() {
                    let health = fs::read_to_string(&self.healthfile)
                        .unwrap_or_else(|_| "No health data".to_string());
                    result.push_str(&format!("\n{}", health));
                }

                // 获取内存和运行时间
                let ps_output = Command::new("ps")
                    .args(&["-o", "rss=", "-p", &pid])
                    .output()
                    .ok();
                if let Some(output) = ps_output {
                    if output.status.success() {
                        let rss = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        result.push_str(&format!("\nMemory: {} KB", rss));
                    }
                }

                let ps_output = Command::new("ps")
                    .args(&["-o", "etime=", "-p", &pid])
                    .output()
                    .ok();
                if let Some(output) = ps_output {
                    if output.status.success() {
                        let etime = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        result.push_str(&format!("\nUptime: {}", etime));
                    }
                }

                Ok(result)
            } else {
                Ok("❌ Daemon not running".to_string())
            }
        } else {
            Ok("❌ Daemon not running".to_string())
        }
    }

    /// 启动守护进程
    pub fn start(&self) -> Result<String, String> {
        // 检查是否已运行
        if self.pidfile.exists() {
            let pid = fs::read_to_string(&self.pidfile)
                .map_err(|e| format!("Failed to read pidfile: {}", e))?
                .trim()
                .to_string();

            let output = Command::new("kill")
                .args(&["-0", &pid])
                .output()
                .map_err(|e| format!("Failed to check process: {}", e))?;

            if output.status.success() {
                return Ok(format!("Already running (PID: {})", pid));
            }
        }

        // 创建日志目录
        if let Some(parent) = self.logfile.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create log directory: {}", e))?;
        }

        // 启动守护进程
        let output = Command::new(&self.daemon_bin)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start daemon: {}", e))?;

        let pid = output.id().to_string();
        fs::write(&self.pidfile, &pid)
            .map_err(|e| format!("Failed to write pidfile: {}", e))?;

        Ok(format!("✅ Started (PID: {})", pid))
    }

    /// 停止守护进程
    pub fn stop(&self) -> Result<String, String> {
        if self.pidfile.exists() {
            let pid = fs::read_to_string(&self.pidfile)
                .map_err(|e| format!("Failed to read pidfile: {}", e))?
                .trim()
                .to_string();

            let output = Command::new("kill")
                .arg(&pid)
                .output()
                .map_err(|e| format!("Failed to stop daemon: {}", e))?;

            if output.status.success() {
                fs::remove_file(&self.pidfile)
                    .map_err(|e| format!("Failed to remove pidfile: {}", e))?;
                Ok(format!("✅ Stopped (PID: {})", pid))
            } else {
                Ok("❌ Not running".to_string())
            }
        } else {
            // 尝试 pkill
            let output = Command::new("pkill")
                .args(&["-f", "target/debug/daemon"])
                .output()
                .map_err(|e| format!("Failed to pkill daemon: {}", e))?;

            if output.status.success() {
                Ok("✅ Stopped".to_string())
            } else {
                Ok("❌ Not running".to_string())
            }
        }
    }

    /// 重启守护进程
    pub fn restart(&self) -> Result<String, String> {
        self.stop()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.start()
    }

    /// 查看日志
    pub fn log(&self) -> Result<String, String> {
        if self.logfile.exists() {
            let content = fs::read_to_string(&self.logfile)
                .map_err(|e| format!("Failed to read log file: {}", e))?;
            Ok(content)
        } else {
            Ok("No log file found".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_daemon_monitor_new() {
        let monitor = DaemonMonitor::new(&PathBuf::from("."));
        assert_eq!(monitor.pidfile, PathBuf::from("/tmp/neotrix_daemon.pid"));
    }
}

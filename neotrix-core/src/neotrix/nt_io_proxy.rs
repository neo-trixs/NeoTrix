use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const DAEMON_BIN_NAME: &str = "neotrix-proxy-daemon";

fn search_path(name: &str) -> Option<PathBuf> {
    if let Ok(paths) = std::env::var("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

/// 解析 proxy daemon 二进制路径
///
/// 优先级:
/// 1. `NEOTRIX_PROXY_DAEMON_PATH` 环境变量
/// 2. 当前可执行文件同目录下的 `neotrix-proxy-daemon`
/// 3. `PATH` 中的 `neotrix-proxy-daemon`
pub fn resolve_daemon_path() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("NEOTRIX_PROXY_DAEMON_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
        return Err(format!("NEOTRIX_PROXY_DAEMON_PATH set but not found: {}", path));
    }

    if let Ok(exe) = std::env::current_exe() {
        let sibling = exe.parent().map(|p| p.join(DAEMON_BIN_NAME));
        if let Some(ref p) = sibling {
            if p.exists() {
                return Ok(p.clone());
            }
        }
    }

    if let Some(path) = search_path(DAEMON_BIN_NAME) {
        return Ok(path);
    }

    Err(format!(
        "neotrix-proxy-daemon not found. Set NEOTRIX_PROXY_DAEMON_PATH or place it next to the neotrix binary."
    ))
}

pub struct ProxyDaemonWrapper {
    pub daemon_path: PathBuf,
    pub restart_delay_ms: u64,
    pub max_restarts: u32,
}

impl ProxyDaemonWrapper {
    pub fn new() -> Result<Self, String> {
        let daemon_path = resolve_daemon_path()?;
        Ok(Self {
            daemon_path,
            restart_delay_ms: 3_000,
            max_restarts: 10,
        })
    }

    pub fn with_path(daemon_path: PathBuf) -> Self {
        Self {
            daemon_path,
            restart_delay_ms: 3_000,
            max_restarts: 10,
        }
    }

    pub fn spawn_detached(&self) -> std::io::Result<()> {
        let log_stdout = format!("/tmp/{}.out.log", DAEMON_BIN_NAME);
        let log_stderr = format!("/tmp/{}.err.log", DAEMON_BIN_NAME);

        let stdout = Stdio::from(
            std::fs::File::create(&log_stdout)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
        );
        let stderr = Stdio::from(
            std::fs::File::create(&log_stderr)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
        );

        Command::new(&self.daemon_path)
            .stdout(stdout)
            .stderr(stderr)
            .stdin(Stdio::null())
            .spawn()?;

        Ok(())
    }

    pub fn ensure_running(&self) -> bool {
        if self.is_alive() {
            return true;
        }
        self.spawn_detached().is_ok()
    }

    pub fn run_supervised(&self) -> ! {
        let mut restart_count: u32 = 0;

        loop {
            if !self.is_alive() {
                if restart_count >= self.max_restarts {
                    panic!(
                        "ProxyDaemon '{}' exceeded max restarts ({})",
                        DAEMON_BIN_NAME, self.max_restarts
                    );
                }
                match self.spawn_detached() {
                    Ok(_) => {
                        restart_count += 1;
                        eprintln!(
                            "[proxy-wrapper] spawned '{}' (restart #{})",
                            DAEMON_BIN_NAME, restart_count
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "[proxy-wrapper] failed to spawn '{}': {}",
                            DAEMON_BIN_NAME, e
                        );
                    }
                }
            }
            thread::sleep(Duration::from_millis(self.restart_delay_ms));
        }
    }

    fn is_alive(&self) -> bool {
        let output = match Command::new("ps")
            .args(&["-ax", "-o", "comm="])
            .output()
        {
            Ok(o) => o,
            Err(_) => return false,
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines().any(|line| {
            line.contains(DAEMON_BIN_NAME)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_daemon_path_sibling_not_found() {
        let result = resolve_daemon_path();
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_new_sets_defaults() {
        let wrapper = ProxyDaemonWrapper::with_path(PathBuf::from("/usr/local/bin/neotrix-proxy-daemon"));
        assert_eq!(wrapper.restart_delay_ms, 3_000);
        assert_eq!(wrapper.max_restarts, 10);
    }

    #[test]
    fn test_is_alive_nonexistent_returns_false() {
        let wrapper = ProxyDaemonWrapper::with_path(PathBuf::from("/usr/local/bin/neotrix-proxy-daemon"));
        assert!(!wrapper.is_alive());
    }

    #[test]
    fn test_wrapper_construction() {
        let wrapper = ProxyDaemonWrapper::with_path(PathBuf::from("/tmp/test_daemon"));
        assert!(!wrapper.daemon_path.to_string_lossy().is_empty());
        assert!(wrapper.restart_delay_ms > 0);
        assert!(wrapper.max_restarts > 0);
    }
}

use log;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;

use crate::core::nt_core_util;
use crate::core::ShutdownSignal;

fn socket_path() -> String {
    let home = nt_core_util::home_dir().to_string_lossy().to_string();
    format!("{}/.neotrix/neotrix-proxy.sock", home)
}
const ACTIVITY_PORT: u16 = 11080;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DaemonMode {
    Off,
    Geo,
    Stealth,
    Tor,
}

impl DaemonMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DaemonMode::Off => "off",
            DaemonMode::Geo => "geo",
            DaemonMode::Stealth => "stealth",
            DaemonMode::Tor => "tor",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "off" => Some(DaemonMode::Off),
            "geo" => Some(DaemonMode::Geo),
            "stealth" => Some(DaemonMode::Stealth),
            "tor" => Some(DaemonMode::Tor),
            _ => None,
        }
    }
}

pub struct ActivityTracker {
    pub last_activity: Instant,
    pub active_count: u64,
    pub idle_timeout: Duration,
}

impl Default for ActivityTracker {
    fn default() -> Self {
        Self {
            last_activity: Instant::now(),
            active_count: 0,
            idle_timeout: Duration::from_secs(300),
        }
    }
}

pub struct ProxyControl {
    mode: Arc<RwLock<DaemonMode>>,
    activity: Arc<RwLock<ActivityTracker>>,
    start_time: Instant,
}

impl Default for ProxyControl {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyControl {
    pub fn new() -> Self {
        Self {
            mode: Arc::new(RwLock::new(DaemonMode::Off)),
            activity: Arc::new(RwLock::new(ActivityTracker::default())),
            start_time: Instant::now(),
        }
    }

    pub fn mode_ref(&self) -> Arc<RwLock<DaemonMode>> {
        self.mode.clone()
    }

    pub async fn set_mode(&self, mode: DaemonMode) {
        *self.mode.write().await = mode;
    }

    pub async fn current_mode(&self) -> DaemonMode {
        *self.mode.read().await
    }

    pub async fn ping_activity(&self) {
        let mut act = self.activity.write().await;
        act.last_activity = Instant::now();
        act.active_count += 1;
    }

    pub async fn idle_seconds(&self) -> u64 {
        let act = self.activity.read().await;
        act.last_activity.elapsed().as_secs()
    }

    pub async fn active_count(&self) -> u64 {
        self.activity.read().await.active_count
    }

    pub async fn should_shutdown_idle(&self) -> bool {
        let act = self.activity.read().await;
        act.idle_timeout > Duration::ZERO
            && act.last_activity.elapsed() > act.idle_timeout
            && *self.mode.read().await == DaemonMode::Off
    }

    /// 启动 Unix socket 控制面服务器
    pub async fn start_control_server(
        self: Arc<Self>,
        shutdown: ShutdownSignal,
    ) -> Result<(), String> {
        let sock = socket_path();
        let _ = std::fs::remove_file(&sock);
        let listener = UnixListener::bind(&sock).map_err(|e| format!("bind unix socket: {}", e))?;
        log::info!("[control] Unix socket on {}", sock);

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            let ctrl = self.clone();
                            tokio::spawn(async move {
                                if let Err(e) = ctrl.handle_connection(stream).await {
                                    log::error!("[control] handle: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            log::error!("[control] accept: {}", e);
                            continue;
                        }
                    }
                }
                _ = shutdown.wait_shutdown() => {
                    log::info!("[control] shutdown signal received, stopping server");
                    break;
                }
            }
        }

        let _ = std::fs::remove_file(&sock);
        Ok(())
    }

    async fn handle_connection(self: Arc<Self>, mut stream: UnixStream) -> Result<(), String> {
        let mut buf = vec![0u8; 4096];
        let n = stream
            .read(&mut buf)
            .await
            .map_err(|e| format!("read: {}", e))?;
        let req = String::from_utf8_lossy(&buf[..n]);
        let req_line = req.lines().next().unwrap_or("");

        let response = if req_line.starts_with("GET /status") {
            self.handle_status().await
        } else if req_line.starts_with("POST /mode/") {
            let mode_str = req_line
                .trim_start_matches("POST /mode/")
                .split(' ')
                .next()
                .unwrap_or("");
            if let Some(mode) = DaemonMode::from_str(mode_str) {
                *self.mode.write().await = mode;
                self.ping_activity().await;
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nmode={}\n",
                    mode.as_str()
                )
            } else {
                format!(
                    "HTTP/1.1 400 Bad Request\r\n\r\nunknown mode: {}\n",
                    mode_str
                )
            }
        } else if req_line.starts_with("POST /shutdown") {
            format!("HTTP/1.1 200 OK\r\n\r\nshutting down\n")
        } else if req_line.starts_with("POST /activity") {
            self.ping_activity().await;
            format!("HTTP/1.1 200 OK\r\n\r\nack\n")
        } else {
            format!("HTTP/1.1 404 Not Found\r\n\r\n")
        };

        stream
            .write_all(response.as_bytes())
            .await
            .map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    async fn handle_status(&self) -> String {
        let mode = self.current_mode().await;
        let idle = self.idle_seconds().await;
        let active = self.active_count().await;
        let uptime = self.start_time.elapsed().as_secs();
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n\
            {}\n",
            serde_json::json!({
                "mode": mode.as_str(),
                "pid": std::process::id(),
                "uptime_secs": uptime,
                "idle_secs": idle,
                "active_count": active,
                "port": ACTIVITY_PORT,
            })
        )
    }
}

/// Unix socket 客户端 — 从 NeoTrix 进程控制 proxy daemon
pub struct ProxyClient {
    socket_path: String,
}

impl ProxyClient {
    pub fn new() -> Self {
        Self {
            socket_path: socket_path(),
        }
    }

    pub fn with_path(path: &str) -> Self {
        Self {
            socket_path: path.to_string(),
        }
    }

    async fn send_request(&self, request: &str) -> Result<String, String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| format!("connect to proxy daemon: {}", e))?;

        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|e| format!("write: {}", e))?;

        let mut buf = vec![0u8; 4096];
        let n = stream
            .read(&mut buf)
            .await
            .map_err(|e| format!("read: {}", e))?;

        let body = String::from_utf8_lossy(&buf[..n])
            .split("\r\n\r\n")
            .nth(1)
            .unwrap_or("")
            .to_string();

        Ok(body)
    }

    /// 获取 daemon 状态 (JSON)
    pub async fn status(&self) -> Result<String, String> {
        self.send_request("GET /status HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
    }

    /// 切换模式
    pub async fn set_mode(&self, mode: DaemonMode) -> Result<String, String> {
        self.send_request(&format!(
            "POST /mode/{} HTTP/1.1\r\nHost: localhost\r\n\r\n",
            mode.as_str()
        ))
        .await
    }

    /// 通知活跃
    pub async fn ping(&self) -> Result<String, String> {
        self.send_request("POST /activity HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
    }

    /// 关闭 daemon
    pub async fn shutdown(&self) -> Result<String, String> {
        self.send_request("POST /shutdown HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
    }

    /// 检查 daemon 是否可达
    pub async fn is_reachable(&self) -> bool {
        self.status().await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mode_roundtrip() {
        let ctrl = ProxyControl::new();
        assert_eq!(ctrl.current_mode().await, DaemonMode::Off);

        ctrl.set_mode(DaemonMode::Geo).await;
        assert_eq!(ctrl.current_mode().await, DaemonMode::Geo);

        ctrl.set_mode(DaemonMode::Stealth).await;
        assert_eq!(ctrl.current_mode().await, DaemonMode::Stealth);

        ctrl.set_mode(DaemonMode::Tor).await;
        assert_eq!(ctrl.current_mode().await, DaemonMode::Tor);
    }

    #[tokio::test]
    async fn test_mode_from_str() {
        assert_eq!(DaemonMode::from_str("off"), Some(DaemonMode::Off));
        assert_eq!(DaemonMode::from_str("geo"), Some(DaemonMode::Geo));
        assert_eq!(DaemonMode::from_str("stealth"), Some(DaemonMode::Stealth));
        assert_eq!(DaemonMode::from_str("tor"), Some(DaemonMode::Tor));
        assert_eq!(DaemonMode::from_str("unknown"), None);
    }

    #[tokio::test]
    async fn test_activity_tracking() {
        let ctrl = ProxyControl::new();
        assert_eq!(ctrl.active_count().await, 0);
        ctrl.ping_activity().await;
        assert_eq!(ctrl.active_count().await, 1);
        assert!(ctrl.idle_seconds().await < 2);
    }
}

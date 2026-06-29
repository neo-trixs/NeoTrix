use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub listen_addr: String,
    pub daemon_token: String,
    pub allowed_runtimes: Vec<String>,
    pub max_concurrent: u32,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:0".into(),
            daemon_token: String::new(),
            allowed_runtimes: vec!["sandbox".into(), "native".into()],
            max_concurrent: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonMetrics {
    pub total_tasks: u64,
    pub completed: u64,
    pub failed: u64,
    pub avg_duration_ms: f64,
}

impl Default for DaemonMetrics {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            completed: 0,
            failed: 0,
            avg_duration_ms: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHandle {
    pub request_id: Uuid,
    pub runtime_type: String,
    pub start_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub request_id: Uuid,
    pub status: String,
    pub output: String,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub agent_id: String,
    pub task: String,
    pub context: Option<serde_json::Value>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_secs: u64,
    pub active_tasks: usize,
    pub metrics: DaemonMetrics,
    pub config_summary: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    pub request_id: Uuid,
}

pub struct DaemonState {
    pub config: DaemonConfig,
    pub active_tasks: Arc<Mutex<HashMap<Uuid, TaskHandle>>>,
    pub metrics: Arc<Mutex<DaemonMetrics>>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl DaemonState {
    pub fn new(config: DaemonConfig) -> Self {
        Self {
            config,
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(DaemonMetrics::default())),
            started_at: chrono::Utc::now(),
        }
    }

    pub fn uptime_secs(&self) -> u64 {
        (chrono::Utc::now() - self.started_at).num_seconds().max(0) as u64
    }
}

pub struct DaemonServer {
    pub state: Arc<DaemonState>,
}

impl DaemonServer {
    pub fn new(config: DaemonConfig) -> Self {
        Self {
            state: Arc::new(DaemonState::new(config)),
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.state.config.listen_addr).await?;
        let addr = listener.local_addr()?;
        eprintln!("[nt-daemon] listening on http://{}", addr);

        loop {
            let (stream, peer) = listener.accept().await?;
            let state = self.state.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, state).await {
                    eprintln!("[nt-daemon] connection error from {}: {}", peer, e);
                }
            });
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    state: Arc<DaemonState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut request_line = String::new();
    buf_reader.read_line(&mut request_line).await?;
    let request_line = request_line.trim().to_string();
    if request_line.is_empty() {
        return Ok(());
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        send_status(&mut writer, 400, "Bad Request").await?;
        return Ok(());
    }
    let method = parts[0];
    let path = parts[1];

    let mut headers = HashMap::new();
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            break;
        }
        if let Some(idx) = trimmed.find(':') {
            let key = trimmed[..idx].trim().to_lowercase();
            let value = trimmed[idx + 1..].trim().to_string();
            if key == "content-length" {
                content_length = value.parse().unwrap_or(0);
            }
            headers.insert(key, value);
        }
    }

    let token = state.config.daemon_token.clone();
    if !token.is_empty() {
        let auth = headers.get("authorization").cloned().unwrap_or_default();
        let expected = format!("Bearer {}", token);
        if auth != expected {
            send_json(
                &mut writer,
                401,
                &serde_json::json!({"error": "unauthorized"}),
            )
            .await?;
            return Ok(());
        }
    }

    let mut body = String::new();
    if content_length > 0 {
        let mut buf = vec![0u8; content_length];
        buf_reader.read_exact(&mut buf).await?;
        body = String::from_utf8_lossy(&buf).to_string();
    }

    match (method, path) {
        ("GET", "/health") => handle_health(&mut writer, &state).await?,
        ("POST", "/execute") => handle_execute(&mut writer, &state, &body).await?,
        ("POST", "/cancel") => handle_cancel(&mut writer, &state, &body).await?,
        _ => {
            send_json(
                &mut writer,
                404,
                &serde_json::json!({"error": "not found"}),
            )
            .await?;
        }
    }

    Ok(())
}

async fn handle_health(
    writer: &mut tokio::net::tcp::WriteHalf<'_>,
    state: &DaemonState,
) -> Result<(), Box<dyn std::error::Error>> {
    let active = state.active_tasks.lock().unwrap().len();
    let metrics = state.metrics.lock().unwrap().clone();
    let config_summary = serde_json::json!({
        "listen_addr": state.config.listen_addr,
        "max_concurrent": state.config.max_concurrent,
        "allowed_runtimes": state.config.allowed_runtimes,
    });
    let resp = HealthResponse {
        status: "ok".into(),
        uptime_secs: state.uptime_secs(),
        active_tasks: active,
        metrics,
        config_summary,
    };
    send_json(writer, 200, &resp).await
}

async fn handle_execute(
    writer: &mut tokio::net::tcp::WriteHalf<'_>,
    state: &Arc<DaemonState>,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let req: ExecuteRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => {
            send_json(
                writer,
                400,
                &serde_json::json!({"error": format!("invalid request: {}", e)}),
            )
            .await?;
            return Ok(());
        }
    };

    let request_id = Uuid::new_v4();
    let now = Utc::now();
    let started_at = now.to_rfc3339();

    let too_many = {
        let mut tasks = state.active_tasks.lock().unwrap();
        let max = state.config.max_concurrent as usize;
        if tasks.len() >= max {
            true
        } else {
            tasks.insert(
                request_id,
                TaskHandle {
                    request_id,
                    runtime_type: "sandbox".into(),
                    start_time: started_at.clone(),
                },
            );
            false
        }
    };
    if too_many {
        send_json(
            writer,
            429,
            &serde_json::json!({"error": "too many concurrent tasks"}),
        )
        .await?;
        return Ok(());
    }

    {
        let mut m = state.metrics.lock().unwrap();
        m.total_tasks += 1;
    }

    let state_clone = state.clone();
    let rid = request_id;
    tokio::spawn(async move {
        let timeout = req.timeout_secs.unwrap_or(30);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout),
            simulate_execution(&req.task),
        )
        .await;

        let mut m = state_clone.metrics.lock().unwrap();
        match result {
            Ok(Ok(_output)) => {
                m.completed += 1;
                let delta = (Utc::now() - now).num_milliseconds() as f64;
                m.avg_duration_ms =
                    (m.avg_duration_ms * (m.completed - 1) as f64 + delta) / m.completed as f64;
            }
            _ => {
                m.failed += 1;
            }
        }
        state_clone.active_tasks.lock().unwrap().remove(&rid);
    });

    let resp = ExecutionResponse {
        request_id,
        status: "pending".into(),
        output: String::new(),
        error: None,
        started_at,
        completed_at: None,
    };
    send_json(writer, 202, &resp).await
}

async fn handle_cancel(
    writer: &mut tokio::net::tcp::WriteHalf<'_>,
    state: &DaemonState,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let req: CancelRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => {
            send_json(
                writer,
                400,
                &serde_json::json!({"error": format!("invalid cancel request: {}", e)}),
            )
            .await?;
            return Ok(());
        }
    };

    let removed = state.active_tasks.lock().unwrap().remove(&req.request_id).is_some();
    if removed {
        send_json(
            writer,
            200,
            &serde_json::json!({"status": "cancelled", "request_id": req.request_id}),
        )
        .await
    } else {
        send_json(
            writer,
            404,
            &serde_json::json!({"error": "task not found", "request_id": req.request_id}),
        )
        .await
    }
}

async fn simulate_execution(task: &str) -> Result<String, String> {
    let duration = std::time::Duration::from_millis(
        (task.len() as u64 * 10).min(2000),
    );
    tokio::time::sleep(duration).await;
    Ok(format!("executed: {}", task))
}

async fn send_status(
    writer: &mut tokio::net::tcp::WriteHalf<'_>,
    status: u16,
    msg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::json!({"error": msg}).to_string();
    send_raw(writer, status, &body).await
}

async fn send_json<T: Serialize>(
    writer: &mut tokio::net::tcp::WriteHalf<'_>,
    status: u16,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::to_string(value)?;
    send_raw(writer, status, &body).await
}

async fn send_raw(
    writer: &mut tokio::net::tcp::WriteHalf<'_>,
    status: u16,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        429 => "Too Many Requests",
        _ => "Unknown",
    };
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        reason,
        body.len()
    );
    writer.write_all(header.as_bytes()).await?;
    writer.write_all(body.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

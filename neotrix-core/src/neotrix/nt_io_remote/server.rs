use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

use crate::neotrix::error::{NeoTrixResult, NeoTrixError};
use crate::neotrix::nt_io_remote::{
    auth::{self, Authenticator},
    connection::{ConnectionManager, ConnectionState},
    CommandHandler, RemoteCommand, RemoteResponse, ResponseStatus,
};

/// TCP remote control server
pub struct RemoteServer {
    port: u16,
    auth: Arc<dyn Authenticator>,
    connection_manager: Arc<ConnectionManager>,
    handler: Option<Arc<dyn CommandHandler>>,
    #[allow(dead_code)]
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl RemoteServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            auth: auth::authenticator_from_method(auth::AuthMethod::NoAuth),
            connection_manager: Arc::new(ConnectionManager::new()),
            handler: None,
            shutdown_tx: None,
        }
    }

    pub fn with_auth(mut self, auth: Arc<dyn Authenticator>) -> Self {
        self.auth = auth;
        self
    }

    pub fn with_connection_manager(mut self, mgr: Arc<ConnectionManager>) -> Self {
        self.connection_manager = mgr;
        self
    }

    pub fn with_handler(mut self, handler: Arc<dyn CommandHandler>) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn connection_manager(&self) -> &Arc<ConnectionManager> {
        &self.connection_manager
    }

    /// Start the remote control server
    pub async fn start(self: Arc<Self>) -> NeoTrixResult<u16> {
        let mut port = self.port;
        let listener = loop {
            match TcpListener::bind(("0.0.0.0", port)).await {
                Ok(l) => break l,
                Err(_) if port < self.port + 5 => { port += 1; }
                Err(e) => return Err(NeoTrixError::Network(format!("RemoteControl bind failed: {}", e))),
            }
        };
        let actual_port = port;
        log::info!("[remote-control] listening on port {}", actual_port);

        let (_shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        let cm = self.connection_manager.clone();
        ConnectionManager::start_heartbeat_monitor(cm, 15, 90);

        let server = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept = listener.accept() => {
                        match accept {
                            Ok((stream, addr)) => {
                                let server = server.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = server.handle_tcp_connection(stream, addr.to_string()).await {
                                        log::warn!("[remote-control] connection error: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                log::error!("[remote-control] accept error: {}", e);
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        log::info!("[remote-control] shutting down");
                        break;
                    }
                }
            }
        });

        Ok(actual_port)
    }

    async fn handle_tcp_connection(self: Arc<Self>, mut stream: TcpStream, peer_addr: String) -> NeoTrixResult<()> {
        let conn_id = format!(
            "rc-{}-{}",
            peer_addr,
            &uuid::Uuid::new_v4().to_string()[..8]
        );
        let _id = self.connection_manager.register(conn_id.clone(), peer_addr.clone());
        self.connection_manager.update_state(&conn_id, ConnectionState::Pending);

        stream.set_nodelay(true).ok();
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        self.connection_manager.set_sender(&conn_id, tx);

        let mut buf = vec![0u8; 8192];

        // Authentication phase
        let n = timeout(Duration::from_secs(10), stream.read(&mut buf)).await
            .map_err(|_| NeoTrixError::Network("auth timeout".into()))?
            .map_err(|e| NeoTrixError::Network(format!("read error: {}", e)))?;

        if n == 0 {
            self.connection_manager.remove(&conn_id);
            return Ok(());
        }

        let auth_line = String::from_utf8_lossy(&buf[..n]).trim().to_string();
        if !self.auth.authenticate(&auth_line) {
            stream.write_all(b"{\"status\":\"Unauthorized\",\"data\":null,\"error\":\"Unauthorized\"}\n").await
                .map_err(|e| NeoTrixError::Network(format!("write error: {}", e)))?;
            self.connection_manager.remove(&conn_id);
            return Ok(());
        }

        self.connection_manager.update_state(&conn_id, ConnectionState::Authenticated);
        stream.write_all(b"AUTH_OK\n").await
            .map_err(|e| NeoTrixError::Network(format!("write error: {}", e)))?;

        // Command loop
        loop {
            tokio::select! {
                read = stream.read(&mut buf) => {
                    let n = match read {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(_) => break,
                    };

                    self.connection_manager.validate_payload(&buf[..n])?;
                    self.connection_manager.record_command(&conn_id);

                    let raw = String::from_utf8_lossy(&buf[..n]);
                    let trimmed = raw.trim();

                    if trimmed == "PING" {
                        stream.write_all(b"PONG\n").await.ok();
                        continue;
                    }

                    match serde_json::from_str::<RemoteCommand>(trimmed) {
                        Ok(cmd) => {
                            let cmd_name = cmd.to_string().split('(').next().unwrap_or("").to_string();
                            if !self.connection_manager.validate_command(&cmd_name) {
                                let resp = RemoteResponse::err("Command not allowed");
                                let json = serde_json::to_string(&resp).unwrap_or_default();
                                stream.write_all(json.as_bytes()).await.ok();
                                stream.write_all(b"\n").await.ok();
                                continue;
                            }

                            let resp = self.execute_command(cmd).await;
                            let json = serde_json::to_string(&resp).unwrap_or_default();
                            stream.write_all(json.as_bytes()).await.ok();
                            stream.write_all(b"\n").await.ok();
                        }
                        Err(e) => {
                            let resp = RemoteResponse::err(format!("Invalid command: {}", e));
                            let json = serde_json::to_string(&resp).unwrap_or_default();
                            stream.write_all(json.as_bytes()).await.ok();
                            stream.write_all(b"\n").await.ok();
                        }
                    }
                }
                Some(msg) = rx.recv() => {
                    stream.write_all(msg.as_bytes()).await.ok();
                    stream.write_all(b"\n").await.ok();
                }
            }
        }

        self.connection_manager.update_state(&conn_id, ConnectionState::Disconnected);
        log::info!("[remote-control] connection {} disconnected", conn_id);
        Ok(())
    }

    async fn execute_command(&self, cmd: RemoteCommand) -> RemoteResponse {
        let handler = match &self.handler {
            Some(h) => h,
            None => return RemoteResponse::err("No CommandHandler configured"),
        };

        match cmd {
            RemoteCommand::HealthCheck => {
                match handler.health().await {
                    Ok(data) => RemoteResponse::ok(data),
                    Err(e) => RemoteResponse::err(e),
                }
            }
            RemoteCommand::ListSessions => {
                match handler.list_sessions().await {
                    Ok(sessions) => RemoteResponse::ok(serde_json::json!({ "sessions": sessions })),
                    Err(e) => RemoteResponse::err(e),
                }
            }
            RemoteCommand::GetSession { id } => {
                match handler.get_session(&id).await {
                    Ok(data) => RemoteResponse::ok(data),
                    Err(e) => RemoteResponse::err(e),
                }
            }
            RemoteCommand::QueryState { ref scope } => {
                match handler.query_state(scope).await {
                    Ok(data) => RemoteResponse::ok(data),
                    Err(e) => RemoteResponse::err(e),
                }
            }
            RemoteCommand::ExecuteTask { prompt, session_id } => {
                match handler.execute_task(&prompt, session_id.as_deref()).await {
                    Ok(output) => RemoteResponse::ok(serde_json::json!({
                        "session_id": session_id,
                        "result": output,
                    })),
                    Err(e) => RemoteResponse::err(format!("Task execution failed: {}", e)),
                }
            }
            RemoteCommand::RunCommand { command, args } => {
                match handler.run_command(&command, &args).await {
                    Ok(data) => RemoteResponse::ok(data),
                    Err(e) => RemoteResponse::err(e),
                }
            }
            RemoteCommand::Shutdown => {
                let _ = handler.shutdown().await;
                RemoteResponse {
                    status: ResponseStatus::ShuttingDown,
                    data: serde_json::json!({ "message": "shutting down" }),
                    error: None,
                }
            }
        }
    }
}

impl Clone for RemoteServer {
    fn clone(&self) -> Self {
        Self {
            port: self.port,
            auth: self.auth.clone(),
            connection_manager: self.connection_manager.clone(),
            handler: self.handler.clone(),
            shutdown_tx: None,
        }
    }
}

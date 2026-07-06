use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::neotrix::l1_body_impl::nt_l1_error::{L1Result, L1Error};
use crate::neotrix::nt_io_remote::{auth::ClientAuth, RemoteCommand, RemoteResponse};

/// Client for connecting to a NeoTrix RemoteControl server
pub struct RemoteClient {
    host: String,
    port: u16,
    auth: ClientAuth,
    stream: Option<TcpStream>,
    connect_timeout: Duration,
    response_timeout: Duration,
}

impl RemoteClient {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            auth: ClientAuth::no_auth(),
            stream: None,
            connect_timeout: Duration::from_secs(10),
            response_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_auth(mut self, auth: ClientAuth) -> Self {
        self.auth = auth;
        self
    }

    pub fn with_timeouts(mut self, connect: Duration, response: Duration) -> Self {
        self.connect_timeout = connect;
        self.response_timeout = response;
        self
    }

    /// Connect to the remote server
    pub async fn connect(&mut self) -> L1Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let stream = timeout(self.connect_timeout, TcpStream::connect(&addr)).await
            .map_err(|_| L1Error::Network(format!("connect timeout to {}", addr)))?
            .map_err(|e| L1Error::Network(format!("connect failed: {}", e)))?;

        stream.set_nodelay(true).ok();
        self.stream = Some(stream);
        self.authenticate().await
    }

    async fn authenticate(&mut self) -> L1Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| L1Error::Network("not connected".into()))?;
        let header = self.auth.header_value();

        stream.write_all(header.as_bytes()).await
            .map_err(|e| L1Error::Network(format!("auth write: {}", e)))?;
        stream.write_all(b"\n").await
            .map_err(|e| L1Error::Network(format!("auth write: {}", e)))?;

        let mut buf = [0u8; 128];
        let n = timeout(Duration::from_secs(5), stream.read(&mut buf)).await
            .map_err(|_| L1Error::Network("auth response timeout".into()))?
            .map_err(|e| L1Error::Network(format!("auth read: {}", e)))?;

        let response = String::from_utf8_lossy(&buf[..n]).trim().to_string();
        if response == "AUTH_OK" {
            Ok(())
        } else if response.contains("Unauthorized") {
            Err(L1Error::Network("Authentication failed: unauthorized".into()))
        } else {
            Err(L1Error::Network(format!("Unexpected auth response: {}", response)))
        }
    }

    /// Send a command and receive response
    pub async fn send_command(&mut self, cmd: &RemoteCommand) -> L1Result<RemoteResponse> {
        let stream = self.stream.as_mut().ok_or_else(|| L1Error::Network("not connected; call connect() first".into()))?;

        let json = serde_json::to_string(cmd)
            .map_err(|e| L1Error::Network(format!("serialize: {}", e)))?;

        stream.write_all(json.as_bytes()).await
            .map_err(|e| L1Error::Network(format!("write: {}", e)))?;
        stream.write_all(b"\n").await
            .map_err(|e| L1Error::Network(format!("write: {}", e)))?;

        let mut buf = vec![0u8; 65536];
        let n = timeout(self.response_timeout, stream.read(&mut buf)).await
            .map_err(|_| L1Error::Network("response timeout".into()))?
            .map_err(|e| L1Error::Network(format!("read: {}", e)))?;

        if n == 0 {
            return Err(L1Error::Network("connection closed".into()));
        }

        let raw = String::from_utf8_lossy(&buf[..n]).trim().to_string();
        serde_json::from_str(&raw)
            .map_err(|e| L1Error::Network(format!("deserialize response: {} (raw: {})", e, raw)))
    }

    /// Convenience: send health check
    pub async fn health_check(&mut self) -> L1Result<RemoteResponse> {
        self.send_command(&RemoteCommand::HealthCheck).await
    }

    /// Disconnect from the server
    pub async fn disconnect(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
}

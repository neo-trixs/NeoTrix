pub mod ws_server;
pub mod sse_server;
pub mod live_monitor;

pub struct WsServer;
impl WsServer {
    pub fn new() -> Self {
        Self
    }
}

pub struct SseServer;
impl SseServer {
    pub fn new() -> Self {
        Self
    }
}

pub struct LiveStreamMonitor;
impl LiveStreamMonitor {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WsMessage {
    pub event: String,
    pub data: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SseEvent {
    pub event: String,
    pub data: String,
}

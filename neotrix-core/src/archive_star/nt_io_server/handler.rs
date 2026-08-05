use super::protocol::{ACPMessage, ACPResponse};

pub struct ACPHandler;

impl Default for ACPHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ACPHandler {
    pub fn new() -> Self { Self }
    pub fn handle(&self, msg: ACPMessage) -> ACPResponse {
        match msg {
            ACPMessage::Ping => ACPResponse::Pong,
            ACPMessage::Shutdown => ACPResponse::Pong,
        }
    }
}

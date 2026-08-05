pub mod protocol;
pub mod handler;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

pub struct NeoTrixACPServer;

impl Default for NeoTrixACPServer {
    fn default() -> Self {
        Self::new()
    }
}

impl NeoTrixACPServer {
    pub fn new() -> Self { Self }
    pub fn server_info() -> ServerInfo {
        ServerInfo { name: "neotrix".into(), version: env!("CARGO_PKG_VERSION").into() }
    }
}

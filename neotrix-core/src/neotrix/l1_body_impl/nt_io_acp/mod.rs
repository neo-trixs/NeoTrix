pub mod handler;
pub mod protocol;
pub mod router;
pub mod session;
pub mod transport;

use std::io;
use std::sync::Arc;

use session::SessionManager;
use transport::StdioTransport;

/// ACP Agent — owns transport, sessions, and handler.
pub struct AcpAgent {
    transport: Arc<StdioTransport>,
    sessions: Arc<SessionManager>,
}

impl Default for AcpAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl AcpAgent {
    pub fn new() -> Self {
        Self {
            transport: Arc::new(StdioTransport::new()),
            sessions: Arc::new(SessionManager::new()),
        }
    }

    /// Start the ACP listen loop (blocking, stdio-based).
    /// Reads JSON-RPC messages from stdin, dispatches, writes responses to stdout.
    pub fn run(&self) -> io::Result<()> {
        let router = router::AcpRouter::new(self.sessions.clone(), self.transport.clone());
        let transport = self.transport.clone();

        transport.listen(|msg| {
            if let Some(response) = router.route(msg) {
                transport.write_message(&response)?;
            }
            Ok(())
        })
    }

    /// Access the running flag for graceful shutdown.
    pub fn shutdown_flag(&self) -> Arc<std::sync::atomic::AtomicBool> {
        self.transport.running_flag()
    }

    /// Request graceful shutdown.
    pub fn shutdown(&self) {
        self.transport.shutdown();
    }

    /// ACP server info (matches the old NeoTrixACPServer interface)
    pub fn server_info() -> super::nt_io_server::ServerInfo {
        super::nt_io_server::ServerInfo {
            name: "neotrix".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

/// Check if we're running in ACP mode based on the `NEOTRIX_ACP` env var.
pub fn is_acp_mode() -> bool {
    std::env::var("NEOTRIX_ACP").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acp_agent_creation() {
        let agent = AcpAgent::new();
        assert!(agent.shutdown_flag().load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_shutdown() {
        let agent = AcpAgent::new();
        agent.shutdown();
        assert!(!agent.shutdown_flag().load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_server_info() {
        let info = AcpAgent::server_info();
        assert_eq!(info.name, "neotrix");
        assert!(!info.version.is_empty());
    }

    #[test]
    fn test_is_acp_mode() {
        // Default should be false
        assert!(!is_acp_mode());
    }
}

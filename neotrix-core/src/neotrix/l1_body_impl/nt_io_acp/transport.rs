use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::protocol::JsonRpcMessage;

/// Stdio-based JSON-RPC transport for ACP.
/// Reads newline-delimited JSON from stdin, writes to stdout.
pub struct StdioTransport {
    running: Arc<AtomicBool>,
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }

    /// Read one JSON-RPC message from stdin.
    /// Returns `None` on EOF or when shutdown flag is set.
    pub fn read_message(&self) -> io::Result<Option<JsonRpcMessage>> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let mut line = String::new();
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        match serde_json::from_str::<JsonRpcMessage>(trimmed) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => {
                log::warn!("[acp] failed to parse message: {}", e);
                Ok(None)
            }
        }
    }

    /// Write a JSON-RPC message to stdout.
    pub fn write_message(&self, msg: &JsonRpcMessage) -> io::Result<()> {
        let json = serde_json::to_string(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{}", json)?;
        stdout.flush()?;
        Ok(())
    }

    /// Request graceful shutdown.
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Listen loop: calls `handler` for every received message.
    /// Returns on EOF or shutdown.
    pub fn listen<F>(&self, mut handler: F) -> io::Result<()>
    where
        F: FnMut(JsonRpcMessage) -> io::Result<()>,
    {
        loop {
            if !self.running.load(Ordering::Relaxed) {
                break;
            }
            match self.read_message()? {
                Some(msg) => handler(msg)?,
                None => break,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transport_roundtrip() {
        let _transport = StdioTransport::new();
        let msg = JsonRpcMessage::success(1, json!({"ok": true}));
        // We can't easily test stdio in unit tests, but we can verify the
        // serialization roundtrip that transport uses internally.
        let json = serde_json::to_string(&msg).unwrap();
        let back: JsonRpcMessage = serde_json::from_str(&json).unwrap();
        match back {
            JsonRpcMessage::Response { id, result, .. } => {
                assert_eq!(id, 1);
                assert!(result.is_some());
            }
            _ => panic!("expected response"),
        }
    }

    #[test]
    fn test_shutdown_flag() {
        let transport = StdioTransport::new();
        assert!(transport.running.load(Ordering::Relaxed));
        transport.shutdown();
        assert!(!transport.running.load(Ordering::Relaxed));
        let result = transport.read_message();
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}

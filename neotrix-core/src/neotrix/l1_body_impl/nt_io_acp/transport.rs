use std::io::{self, Read, Write};
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
    /// A malformed or blank line is logged and skipped (does not terminate
    /// the server), so a stray newline or spec-legal request that fails
    /// strict `id: u64` decoding cannot kill the whole ACP process.
    pub fn read_message(&self) -> io::Result<Option<JsonRpcMessage>> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let mut line = String::new();
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        // Bound the frame size: a client streaming without a newline must
        // not grow the buffer without limit. Lines longer than 1 MiB are
        // drained (skipped) rather than accumulated.
        let mut chunk = [0u8; 1024];
        let mut total = 0usize;
        const MAX_LINE: usize = 1024 * 1024;
        loop {
            let read = reader.read(&mut chunk)?;
            if read == 0 {
                if total == 0 { return Ok(None); }
                break;
            }
            total += read;
            if total > MAX_LINE {
                // Drain the rest of the oversized line, then skip it.
                let mut drain = [0u8; 4096];
                loop {
                    let n = reader.read(&mut drain)?;
                    if n == 0 { return self.read_message(); }
                    if drain[..n].contains(&b'\n') { return self.read_message(); }
                }
            }
            let text = String::from_utf8_lossy(&chunk[..read]);
            line.push_str(&text);
            if text.contains('\n') {
                break;
            }
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return self.read_message();
        }
        match serde_json::from_str::<JsonRpcMessage>(trimmed) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => {
                log::warn!("[acp] failed to parse message (skipping): {}", e);
                self.read_message()
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

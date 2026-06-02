use super::types::{FileEntry, FileIndex, SyncMessage};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

/// TCP server that handles incoming sync requests from peers.
pub struct SyncServer {
    listener: TcpListener,
    port: u16,
    local_index_fn: Box<dyn Fn(&str) -> Option<FileIndex> + Send>,
}

/// TCP client that connects to a remote peer for sync.
pub struct SyncClient;

impl SyncServer {
    /// Bind to a port for incoming sync connections.
    pub fn bind<F>(port: u16, index_fn: F) -> Result<Self, String>
    where
        F: Fn(&str) -> Option<FileIndex> + Send + 'static,
    {
        let listener = TcpListener::bind(("0.0.0.0", port))
            .map_err(|e| format!("Bind sync TCP {}: {}", port, e))?;
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("Set nonblock: {}", e))?;
        Ok(Self {
            listener,
            port,
            local_index_fn: Box::new(index_fn),
        })
    }

    pub fn port(&self) -> u16 { self.port }

    /// Accept one pending connection (non-blocking). Returns (Some(peer_addr), received message) or None.
    pub fn accept_one(&self) -> Option<(String, SyncMessage)> {
        match self.listener.accept() {
            Ok((mut stream, addr)) => {
                let peer = addr.to_string();
                let msg = read_message(&mut stream)?;
                Some((peer, msg))
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
            Err(e) => {
                log::warn!("[sync] accept error: {}", e);
                None
            }
        }
    }

    /// Handle an incoming request: process message, send response, optionally write file.
    pub fn handle_request(&self, stream: &mut TcpStream, msg: SyncMessage) -> bool {
        match msg {
            SyncMessage::IndexRequest { path } => {
                let index = (self.local_index_fn)(&path).unwrap_or_else(|| FileIndex::empty(path));
                write_message(stream, &SyncMessage::IndexResponse { index })
            }
            SyncMessage::GetFile { relative_path } => {
                let root = Path::new(&relative_path);
                if root.exists() && root.is_file() {
                    let metadata = match root.metadata() {
                        Ok(m) => m,
                        Err(_) => return write_message(stream, &SyncMessage::Error { message: "metadata error".into() }),
                    };
                    let size = metadata.len();
                    let modified = metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    let checksum = match file_checksum(root) {
                        Ok(c) => c,
                        Err(_) => return write_message(stream, &SyncMessage::Error { message: "checksum error".into() }),
                    };
                    let ok = write_message(stream, &SyncMessage::FileContent {
                        relative_path: relative_path.clone(),
                        size,
                        modified,
                        checksum,
                    });
                    if ok {
                        let _ = stream.write_all(&std::fs::read(root).unwrap_or_default());
                    }
                    ok
                } else {
                    write_message(stream, &SyncMessage::Error { message: format!("not found: {}", relative_path) })
                }
            }
            SyncMessage::PutFile { relative_path, size, modified, checksum } => {
                let target = Path::new(&relative_path);
                if let Some(parent) = target.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let mut data = vec![0u8; size as usize];
                let read_ok = stream.read_exact(&mut data).is_ok();
                if !read_ok {
                    return write_message(stream, &SyncMessage::Error { message: "read body failed".into() });
                }
                let actual_hex = format!("{:x}", Sha256::digest(&data));
                if actual_hex != checksum {
                    return write_message(stream, &SyncMessage::Error { message: "checksum mismatch".into() });
                }
                match std::fs::write(target, &data) {
                    Ok(_) => {
                        let ftime = UNIX_EPOCH + Duration::from_secs(modified as u64);
                        let _ = filetime::set_file_mtime(target, filetime::FileTime::from_system_time(ftime));
                        write_message(stream, &SyncMessage::Ack { message: format!("received: {}", relative_path) })
                    }
                    Err(e) => write_message(stream, &SyncMessage::Error { message: format!("write: {}", e) }),
                }
            }
            _ => write_message(stream, &SyncMessage::Error { message: "unknown command".into() }),
        }
    }
}

impl SyncClient {
    /// Request a remote file index from a peer.
    pub fn request_index(host: &str, port: u16, path: &str) -> Result<FileIndex, String> {
        let mut stream = connect(host, port)?;
        write_message(&mut stream, &SyncMessage::IndexRequest { path: path.into() });
        match read_message(&mut stream) {
            Some(SyncMessage::IndexResponse { index }) => Ok(index),
            Some(SyncMessage::Error { message }) => Err(message),
            _ => Err("unexpected response".into()),
        }
    }

    /// Pull a file from a remote peer.
    pub fn pull_file(host: &str, port: u16, dest_dir: &Path, entry: &FileEntry) -> Result<(), String> {
        let mut stream = connect(host, port)?;
        write_message(&mut stream, &SyncMessage::GetFile { relative_path: entry.relative_path.clone() });

        match read_message(&mut stream) {
            Some(SyncMessage::FileContent { relative_path, size, modified, checksum }) => {
                let target = dest_dir.join(&relative_path);
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| format!("create dir: {}", e))?;
                }
                let mut data = vec![0u8; size as usize];
                stream.read_exact(&mut data).map_err(|e| format!("read body: {}", e))?;
                let actual = Sha256::digest(&data);
                let actual_hex = format!("{:x}", actual);
                if actual_hex != checksum {
                    return Err("checksum mismatch".into());
                }
                std::fs::write(&target, &data).map_err(|e| format!("write: {}", e))?;
                let ftime = UNIX_EPOCH + Duration::from_secs(modified as u64);
                let _ = filetime::set_file_mtime(&target, filetime::FileTime::from_system_time(ftime));
                Ok(())
            }
            Some(SyncMessage::Error { message }) => Err(message),
            _ => Err("unexpected response".into()),
        }
    }

    /// Push a file to a remote peer.
    pub fn push_file(host: &str, port: u16, local_path: &Path, entry: &FileEntry) -> Result<(), String> {
        let mut stream = connect(host, port)?;
        let data = std::fs::read(local_path).map_err(|e| format!("read: {}", e))?;

        write_message(&mut stream, &SyncMessage::PutFile {
            relative_path: entry.relative_path.clone(),
            size: entry.size,
            modified: entry.modified,
            checksum: entry.checksum.clone(),
        });
        stream.write_all(&data).map_err(|e| format!("write body: {}", e))?;

        match read_message(&mut stream) {
            Some(SyncMessage::Ack { .. }) => Ok(()),
            Some(SyncMessage::Error { message }) => Err(message),
            _ => Err("unexpected response".into()),
        }
    }
}

// ── Helpers ──

fn connect(host: &str, port: u16) -> Result<TcpStream, String> {
    let addr = format!("{}:{}", host, port);
    TcpStream::connect_timeout(&addr.parse().map_err(|e| format!("addr: {}", e))?, Duration::from_secs(10))
        .map_err(|e| format!("connect {}: {}", addr, e))
}

fn write_message(stream: &mut TcpStream, msg: &SyncMessage) -> bool {
    let json = serde_json::to_string(msg).unwrap_or_default();
    let mut data = json.into_bytes();
    data.push(b'\n');
    stream.write_all(&data).is_ok()
}

fn read_message(stream: &mut TcpStream) -> Option<SyncMessage> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    if line.is_empty() {
        return None;
    }
    serde_json::from_str(&line).ok()
}

fn file_checksum(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("open: {}", e))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| format!("hash: {}", e))?;
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_message_serde() {
        let msg = SyncMessage::IndexRequest { path: "/tmp".into() };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("IndexRequest"));
        let back: SyncMessage = serde_json::from_str(&json).unwrap();
        match back {
            SyncMessage::IndexRequest { path } => assert_eq!(path, "/tmp"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_put_file_message() {
        let msg = SyncMessage::PutFile {
            relative_path: "docs/readme.md".into(),
            size: 1024,
            modified: 1700000000,
            checksum: "abc123".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("PutFile"));
        assert!(json.contains("docs/readme.md"));
    }
}

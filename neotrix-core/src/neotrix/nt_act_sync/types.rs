use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A discovered peer that can participate in file sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPeer {
    pub id: String,
    pub name: String,
    pub host: String,
    pub tcp_port: u16,
    pub last_seen: i64,
    pub is_connected: bool,
}

/// A directory pair configured for sync between two peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDir {
    pub local_path: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub bidirectional: bool,
}

/// A sync relationship between this instance and a remote peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPair {
    pub peer_id: String,
    pub peer_name: String,
    pub peer_host: String,
    pub peer_port: u16,
    pub directories: Vec<SyncDir>,
    pub last_sync: Option<i64>,
    pub status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Idle,
    Scanning,
    Syncing { current_file: String, progress: f64 },
    Error(String),
}

/// A single file entry in a local or remote index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub relative_path: String,
    pub size: u64,
    pub modified: i64,
    pub checksum: String,
}

/// Full index of a directory at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndex {
    pub root: String,
    pub files: Vec<FileEntry>,
    pub total_size: u64,
    pub file_count: u32,
}

/// Diff between two indexes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDiff {
    pub to_send: Vec<FileEntry>,
    pub to_receive: Vec<FileEntry>,
    pub to_delete_local: Vec<String>,
    pub to_delete_remote: Vec<String>,
    pub total_bytes: u64,
}

/// Wire protocol messages (transmitted as JSON-lines over TCP).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum SyncMessage {
    /// Request remote index for a path
    IndexRequest { path: String },
    /// Response with file index
    IndexResponse { index: FileIndex },
    /// Request a file's content
    GetFile { relative_path: String },
    /// File content response (followed by raw bytes)
    FileContent {
        relative_path: String,
        size: u64,
        modified: i64,
        checksum: String,
    },
    /// Push file content (sender sends raw bytes after this)
    PutFile {
        relative_path: String,
        size: u64,
        modified: i64,
        checksum: String,
    },
    /// Acknowledge receipt
    Ack { message: String },
    /// Error
    Error { message: String },
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::Idle
    }
}

impl SyncPeer {
    pub fn new(id: String, name: String, host: String, tcp_port: u16) -> Self {
        Self {
            id,
            name,
            host,
            tcp_port,
            last_seen: 0,
            is_connected: false,
        }
    }
}

impl SyncPair {
    pub fn new(peer: &SyncPeer) -> Self {
        Self {
            peer_id: peer.id.clone(),
            peer_name: peer.name.clone(),
            peer_host: peer.host.clone(),
            peer_port: peer.tcp_port,
            directories: Vec::new(),
            last_sync: None,
            status: SyncStatus::Idle,
        }
    }
}

impl FileEntry {
    pub fn new(relative_path: PathBuf, size: u64, modified: i64, checksum: String) -> Self {
        Self {
            relative_path: relative_path.to_string_lossy().to_string(),
            size,
            modified,
            checksum,
        }
    }
}

impl FileIndex {
    pub fn empty(root: String) -> Self {
        Self {
            root,
            files: Vec::new(),
            total_size: 0,
            file_count: 0,
        }
    }
}

impl SyncDiff {
    /// Compute diff: local vs remote. Returns files to send, receive, and delete.
    pub fn compute(local: &FileIndex, remote: &FileIndex) -> Self {
        let local_map: HashMap<&str, &FileEntry> = local
            .files
            .iter()
            .map(|f| (f.relative_path.as_str(), f))
            .collect();
        let remote_map: HashMap<&str, &FileEntry> = remote
            .files
            .iter()
            .map(|f| (f.relative_path.as_str(), f))
            .collect();

        let mut to_send = Vec::new();
        let mut to_receive = Vec::new();
        let mut to_delete_local = Vec::new();
        let mut to_delete_remote = Vec::new();

        // Files on remote but not local, or newer on remote → receive
        for (path, remote_file) in &remote_map {
            match local_map.get(path) {
                Some(local_file) => {
                    if remote_file.modified > local_file.modified
                        && remote_file.checksum != local_file.checksum
                    {
                        to_receive.push((*remote_file).clone());
                    }
                }
                None => {
                    to_receive.push((*remote_file).clone());
                }
            }
        }

        // Files on local but not remote, or newer on local → send
        for (path, local_file) in &local_map {
            match remote_map.get(path) {
                Some(remote_file) => {
                    if local_file.modified > remote_file.modified
                        && local_file.checksum != remote_file.checksum
                    {
                        to_send.push((*local_file).clone());
                    }
                }
                None => {
                    to_send.push((*local_file).clone());
                }
            }
        }

        // Files deleted on remote should be deleted locally
        for (path, _) in &local_map {
            if !remote_map.contains_key(path) {
                to_delete_local.push(path.to_string());
            }
        }

        // Files deleted on local should be deleted on remote
        for (path, _) in &remote_map {
            if !local_map.contains_key(path) {
                to_delete_remote.push(path.to_string());
            }
        }

        let total_bytes: u64 = to_send
            .iter()
            .chain(to_receive.iter())
            .map(|f| f.size)
            .sum();

        Self {
            to_send,
            to_receive,
            to_delete_local,
            to_delete_remote,
            total_bytes,
        }
    }
}

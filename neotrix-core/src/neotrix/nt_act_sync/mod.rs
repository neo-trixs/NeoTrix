mod index;
mod transfer;
mod types;

pub use index::*;
pub use transfer::*;
pub use types::*;

use crate::neotrix::nt_agent_protocol::discovery::{AgentDiscovery, AgentInfo};
use std::collections::HashMap;
use std::path::Path;

/// FileSync orchestrator — manages discovery, indexing, and transfer lifecycle.
pub struct FileSync {
    discovery: AgentDiscovery,
    server: Option<SyncServer>,
    pairs: Vec<SyncPair>,
    peers: HashMap<String, SyncPeer>,
    local_root: String,
    known_peers: Vec<AgentInfo>,
}

impl FileSync {
    pub fn new(discovery_port: u16, sync_port: u16, local_root: String) -> Result<Self, String> {
        let discovery = AgentDiscovery::new(discovery_port).map_err(|e| e.to_string())?;

        let server = match SyncServer::bind(sync_port, {
            let root = local_root.clone();
            move |_path| {
                let p = Path::new(&root);
                if p.exists() {
                    scan_directory(
                        p,
                        &[],
                        &[".git".into(), "target".into(), "node_modules".into()],
                    )
                    .ok()
                } else {
                    None
                }
            }
        }) {
            Ok(s) => Some(s),
            Err(e) => {
                log::warn!("[nt_act_sync] server bind failed: {}", e);
                None
            }
        };

        Ok(Self {
            discovery,
            server,
            pairs: Vec::new(),
            peers: HashMap::new(),
            local_root,
            known_peers: Vec::new(),
        })
    }

    /// Scan network for peers running NeoTrix with file sync enabled.
    pub fn discover_peers(&mut self, duration_ms: u64) -> Result<Vec<SyncPeer>, String> {
        let agents = self
            .discovery
            .discover(duration_ms)
            .map_err(|e| e.to_string())?;
        self.known_peers = agents.clone();
        let mut peers = Vec::new();
        for agent in &agents {
            let peer = SyncPeer::new(
                agent.id.clone(),
                agent.name.clone(),
                agent.host.clone(),
                agent.port + 1,
            );
            self.peers.insert(agent.id.clone(), peer.clone());
            peers.push(peer);
        }
        Ok(peers)
    }

    /// Get known peers (from last scan).
    pub fn known_peers(&self) -> &[AgentInfo] {
        &self.known_peers
    }

    /// Add a sync pair (relationship with a peer).
    pub fn add_pair(&mut self, peer_id: &str) -> Result<(), String> {
        if self.pairs.iter().any(|p| p.peer_id == peer_id) {
            return Ok(()); // already exists
        }
        let peer = self
            .peers
            .get(peer_id)
            .ok_or_else(|| format!("peer not found: {}", peer_id))?;
        self.pairs.push(SyncPair::new(peer));
        Ok(())
    }

    /// Remove a sync pair.
    pub fn remove_pair(&mut self, peer_id: &str) {
        self.pairs.retain(|p| p.peer_id != peer_id);
    }

    /// Get all sync pairs.
    pub fn pairs(&self) -> &[SyncPair] {
        &self.pairs
    }

    /// Get mutable reference to pairs.
    pub fn pairs_mut(&mut self) -> &mut Vec<SyncPair> {
        &mut self.pairs
    }

    /// Scan local directory and compare with remote peer. Returns the diff.
    pub fn compute_diff(&self, peer_id: &str) -> Result<(FileIndex, FileIndex, SyncDiff), String> {
        let pair = self
            .pairs
            .iter()
            .find(|p| p.peer_id == peer_id)
            .ok_or_else(|| format!("pair not found: {}", peer_id))?;

        let local = scan_directory(
            Path::new(&self.local_root),
            &pair
                .directories
                .first()
                .map(|d| d.include_patterns.clone())
                .unwrap_or_default(),
            &pair
                .directories
                .first()
                .map(|d| d.exclude_patterns.clone())
                .unwrap_or_default(),
        )?;

        let remote = SyncClient::request_index(&pair.peer_host, pair.peer_port, &self.local_root)?;

        let diff = SyncDiff::compute(&local, &remote);
        Ok((local, remote, diff))
    }

    /// Execute a sync: send and receive files based on diff.
    pub fn execute_sync(&mut self, peer_id: &str) -> Result<u64, String> {
        let pair = self
            .pairs
            .iter()
            .find(|p| p.peer_id == peer_id)
            .ok_or_else(|| format!("pair not found: {}", peer_id))?;

        let local = scan_directory(
            Path::new(&self.local_root),
            &[],
            &[".git".into(), "target".into(), "node_modules".into()],
        )?;

        let remote = SyncClient::request_index(&pair.peer_host, pair.peer_port, &self.local_root)?;
        let diff = SyncDiff::compute(&local, &remote);
        let mut total = 0u64;

        // Send files to remote
        for entry in &diff.to_send {
            let local_path = Path::new(&self.local_root).join(&entry.relative_path);
            SyncClient::push_file(&pair.peer_host, pair.peer_port, &local_path, entry)?;
            total += entry.size;
        }

        // Receive files from remote
        for entry in &diff.to_receive {
            SyncClient::pull_file(
                &pair.peer_host,
                pair.peer_port,
                Path::new(&self.local_root),
                entry,
            )?;
            total += entry.size;
        }

        // Update last_sync
        if let Some(p) = self.pairs.iter_mut().find(|p| p.peer_id == peer_id) {
            p.last_sync = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            );
        }

        Ok(total)
    }

    /// The sync port this server is listening on.
    pub fn server_port(&self) -> u16 {
        self.server.as_ref().map(|s| s.port()).unwrap_or(0)
    }

    /// Check and process one incoming sync request.
    pub fn poll_incoming(&self) {
        if let Some(server) = &self.server {
            if let Some((_peer, msg)) = server.accept_one() {
                // For now, log incoming requests
                log::info!("[nt_act_sync] incoming: {:?}", msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_scan_single_file() {
        let dir = std::env::temp_dir().join("neotrix_fsync_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut f = fs::File::create(dir.join("test.txt")).unwrap();
        f.write_all(b"hello world").unwrap();

        let index = scan_directory(&dir, &[], &[]).unwrap();
        assert_eq!(index.file_count, 1);
        assert_eq!(index.files[0].size, 11);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_diff_identical_indexes() {
        let dir = std::env::temp_dir().join("neotrix_fsync_diff");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut f = fs::File::create(dir.join("a.txt")).unwrap();
        f.write_all(b"same").unwrap();

        let local = scan_directory(&dir, &[], &[]).unwrap();
        let remote = local.clone();
        let diff = SyncDiff::compute(&local, &remote);

        assert!(diff.to_send.is_empty());
        assert!(diff.to_receive.is_empty());
        assert!(diff.to_delete_local.is_empty());
        assert!(diff.to_delete_remote.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_diff_new_file_on_remote() {
        let local_dir = std::env::temp_dir().join("neotrix_fsync_local");
        let remote_dir = std::env::temp_dir().join("neotrix_fsync_remote");
        let _ = fs::remove_dir_all(&local_dir);
        let _ = fs::remove_dir_all(&remote_dir);
        fs::create_dir_all(&local_dir).unwrap();
        fs::create_dir_all(&remote_dir).unwrap();

        // Local has a.txt
        let mut f = fs::File::create(local_dir.join("a.txt")).unwrap();
        f.write_all(b"local").unwrap();

        // Remote has a.txt and b.txt
        let mut f = fs::File::create(remote_dir.join("a.txt")).unwrap();
        f.write_all(b"local").unwrap();
        let mut f = fs::File::create(remote_dir.join("b.txt")).unwrap();
        f.write_all(b"remote only").unwrap();

        let local = scan_directory(&local_dir, &[], &[]).unwrap();
        let remote = scan_directory(&remote_dir, &[], &[]).unwrap();
        let diff = SyncDiff::compute(&local, &remote);

        assert_eq!(diff.to_receive.len(), 1);
        assert_eq!(diff.to_receive[0].relative_path, "b.txt");
        assert!(diff.to_send.is_empty());
        let _ = fs::remove_dir_all(&local_dir);
        let _ = fs::remove_dir_all(&remote_dir);
    }
}

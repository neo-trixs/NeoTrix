use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

pub type SyncState = Arc<Mutex<Option<neotrix::neotrix::file_sync::FileSync>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPairInfo {
    pub peer_id: String,
    pub peer_name: String,
    pub peer_host: String,
    pub status: String,
    pub last_sync: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDiffInfo {
    pub to_send: Vec<String>,
    pub to_receive: Vec<String>,
    pub total_bytes: u64,
}

/// Initialize the file sync subsystem with discovery + sync server.
#[tauri::command]
pub fn sync_init(state: State<'_, SyncState>, discovery_port: u16, sync_port: u16, local_root: String) -> Result<(), String> {
    let sync = neotrix::neotrix::file_sync::FileSync::new(discovery_port, sync_port, local_root)?;
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    *guard = Some(sync);
    Ok(())
}

/// Scan network for peers running NeoTrix.
#[tauri::command]
pub fn sync_discover(state: State<'_, SyncState>, duration_ms: u64) -> Result<Vec<PeerInfo>, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let sync = guard.as_mut().ok_or("sync not initialized")?;
    let peers = sync.discover_peers(duration_ms)?;
    Ok(peers.into_iter().map(|p| PeerInfo {
        id: p.id,
        name: p.name,
        host: p.host,
        port: p.tcp_port,
        last_seen: p.last_seen,
    }).collect())
}

/// Add a sync pair with a discovered peer.
#[tauri::command]
pub fn sync_add_pair(state: State<'_, SyncState>, peer_id: String, local_path: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let sync = guard.as_mut().ok_or("sync not initialized")?;
    sync.add_pair(&peer_id)?;
    if let Some(pair) = sync.pairs_mut().iter_mut().find(|p| p.peer_id == peer_id) {
        pair.directories.push(neotrix::neotrix::file_sync::SyncDir {
            local_path,
            include_patterns: Vec::new(),
            exclude_patterns: vec![".git".into(), "target".into(), "node_modules".into()],
            bidirectional: true,
        });
    }
    Ok(())
}

/// Remove a sync pair.
#[tauri::command]
pub fn sync_remove_pair(state: State<'_, SyncState>, peer_id: String) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let sync = guard.as_mut().ok_or("sync not initialized")?;
    sync.remove_pair(&peer_id);
    Ok(())
}

/// List all configured sync pairs.
#[tauri::command]
pub fn sync_list_pairs(state: State<'_, SyncState>) -> Result<Vec<SyncPairInfo>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let sync = guard.as_ref().ok_or("sync not initialized")?;
    Ok(sync.pairs().iter().map(|p| SyncPairInfo {
        peer_id: p.peer_id.clone(),
        peer_name: p.peer_name.clone(),
        peer_host: p.peer_host.clone(),
        status: format!("{:?}", p.status),
        last_sync: p.last_sync,
    }).collect())
}

/// Compute diff with a peer (scan local + request remote index).
#[tauri::command]
pub fn sync_preview(state: State<'_, SyncState>, peer_id: String) -> Result<SyncDiffInfo, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let sync = guard.as_mut().ok_or("sync not initialized")?;
    let (_local, _remote, diff) = sync.compute_diff(&peer_id)?;
    Ok(SyncDiffInfo {
        to_send: diff.to_send.into_iter().map(|f| f.relative_path).collect(),
        to_receive: diff.to_receive.into_iter().map(|f| f.relative_path).collect(),
        total_bytes: diff.total_bytes,
    })
}

/// Execute sync with a peer.
#[tauri::command]
pub fn sync_start(state: State<'_, SyncState>, peer_id: String) -> Result<u64, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let sync = guard.as_mut().ok_or("sync not initialized")?;
    sync.execute_sync(&peer_id)
}

/// Get the current sync status.
#[tauri::command]
pub fn sync_status(state: State<'_, SyncState>) -> Result<String, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let sync = guard.as_ref().ok_or("sync not initialized")?;
    Ok(format!(
        "server_port: {}, peers: {}",
        sync.server_port(),
        sync.known_peers().len(),
    ))
}

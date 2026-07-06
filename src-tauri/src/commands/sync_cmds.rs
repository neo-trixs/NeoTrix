use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;
use neotrix::neotrix::nt_core_error::NeoTrixError;

pub type SyncState = Arc<Mutex<Option<neotrix::neotrix::nt_act_sync::FileSync>>>;

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
pub fn sync_init(state: State<'_, SyncState>, discovery_port: u16, sync_port: u16, local_root: String) -> Result<(), NeoTrixError> {
    let sync = neotrix::neotrix::nt_act_sync::FileSync::new(discovery_port, sync_port, local_root)?;
    let mut guard = state.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    *guard = Some(sync);
    Ok(())
}

/// Scan network for peers running NeoTrix.
#[tauri::command]
pub fn sync_discover(state: State<'_, SyncState>, duration_ms: u64) -> Result<Vec<PeerInfo>, NeoTrixError> {
    let mut guard = state.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let sync = guard.as_mut().ok_or_else(|| NeoTrixError::Memory("sync not initialized".into()))?;
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
pub fn sync_add_pair(state: State<'_, SyncState>, peer_id: String, local_path: String) -> Result<(), NeoTrixError> {
    let mut guard = state.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let sync = guard.as_mut().ok_or_else(|| NeoTrixError::Memory("sync not initialized".into()))?;
    sync.add_pair(&peer_id)?;
    if let Some(pair) = sync.pairs_mut().iter_mut().find(|p| p.peer_id == peer_id) {
        pair.directories.push(neotrix::neotrix::nt_act_sync::SyncDir {
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
pub fn sync_remove_pair(state: State<'_, SyncState>, peer_id: String) -> Result<(), NeoTrixError> {
    let mut guard = state.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let sync = guard.as_mut().ok_or_else(|| NeoTrixError::Memory("sync not initialized".into()))?;
    sync.remove_pair(&peer_id);
    Ok(())
}

/// List all configured sync pairs.
#[tauri::command]
pub fn sync_list_pairs(state: State<'_, SyncState>) -> Result<Vec<SyncPairInfo>, NeoTrixError> {
    let guard = state.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let sync = guard.as_ref().ok_or_else(|| NeoTrixError::Memory("sync not initialized".into()))?;
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
pub fn sync_preview(state: State<'_, SyncState>, peer_id: String) -> Result<SyncDiffInfo, NeoTrixError> {
    let mut guard = state.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let sync = guard.as_mut().ok_or_else(|| NeoTrixError::Memory("sync not initialized".into()))?;
    let (_local, _remote, diff) = sync.compute_diff(&peer_id)?;
    Ok(SyncDiffInfo {
        to_send: diff.to_send.into_iter().map(|f| f.relative_path).collect(),
        to_receive: diff.to_receive.into_iter().map(|f| f.relative_path).collect(),
        total_bytes: diff.total_bytes,
    })
}

/// Execute sync with a peer.
#[tauri::command]
pub fn sync_start(state: State<'_, SyncState>, peer_id: String) -> Result<u64, NeoTrixError> {
    let mut guard = state.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let sync = guard.as_mut().ok_or_else(|| NeoTrixError::Memory("sync not initialized".into()))?;
    sync.execute_sync(&peer_id).map_err(NeoTrixError::Memory)
}

/// Get the current sync status.
#[tauri::command]
pub fn sync_status(state: State<'_, SyncState>) -> Result<String, NeoTrixError> {
    let guard = state.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let sync = guard.as_ref().ok_or_else(|| NeoTrixError::Memory("sync not initialized".into()))?;
    Ok(format!(
        "server_port: {}, peers: {}",
        sync.server_port(),
        sync.known_peers().len(),
    ))
}

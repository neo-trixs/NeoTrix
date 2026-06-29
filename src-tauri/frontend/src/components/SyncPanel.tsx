import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useStore } from "../store";

interface Peer {
  id: string;
  name: string;
  host: string;
  port: number;
  last_seen: number;
}

interface SyncPair {
  peer_id: string;
  peer_name: string;
  peer_host: string;
  status: string;
  last_sync: number | null;
}

interface SyncDiff {
  to_send: string[];
  to_receive: string[];
  total_bytes: number;
}

const SyncPanel: React.FC = () => {
  const setSyncVisible = useStore((s) => (s as any).setSyncVisible);
  const [peers, setPeers] = useState<Peer[]>([]);
  const [pairs, setPairs] = useState<SyncPair[]>([]);
  const [diff, setDiff] = useState<SyncDiff | null>(null);
  const [status, setStatus] = useState("");
  const [scanning, setScanning] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [localRoot, setLocalRoot] = useState("");

  const [syncStatus, setSyncStatus] = useState("idle");
  const [lastSyncTime, setLastSyncTime] = useState<string | null>(null);
  const [lastFilesSynced, setLastFilesSynced] = useState(0);

  useEffect(() => {
    invoke("sync_status").then((s) => setStatus(s as string)).catch(() => {});
  }, []);

  useEffect(() => {
    const unlisten = listen<{ status: string; files_synced: number; duration_ms: number; timestamp: string }>("sync-complete", (event) => {
      setSyncStatus(event.payload.status);
      setLastSyncTime(event.payload.timestamp);
      setLastFilesSynced(event.payload.files_synced);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const initSync = useCallback(async () => {
    try {
      await invoke("sync_init", { discoveryPort: 42069, syncPort: 42070, localRoot: localRoot || "~/.neotrix" });
      setStatus("ready");
    } catch (e) {
      setStatus(`error: ${e}`);
    }
  }, [localRoot]);

  const discover = useCallback(async () => {
    setScanning(true);
    try {
      const result = await invoke<Peer[]>("sync_discover", { durationMs: 3000 });
      setPeers(result);
    } catch (e) {
      setStatus(`discover error: ${e}`);
    }
    setScanning(false);
  }, []);

  const addPair = useCallback(async (peerId: string) => {
    try {
      await invoke("sync_add_pair", { peerId, localPath: localRoot || "~/.neotrix" });
      await listPairs();
    } catch (e) {
      setStatus(`add pair error: ${e}`);
    }
  }, [localRoot]);

  const removePair = useCallback(async (peerId: string) => {
    try {
      await invoke("sync_remove_pair", { peerId });
      await listPairs();
    } catch (e) {
      setStatus(`remove error: ${e}`);
    }
  }, []);

  const listPairs = useCallback(async () => {
    try {
      const result = await invoke<SyncPair[]>("sync_list_pairs");
      setPairs(result);
    } catch (e) {
      setStatus(`list error: ${e}`);
    }
  }, []);

  const preview = useCallback(async (peerId: string) => {
    try {
      const result = await invoke<SyncDiff>("sync_preview", { peerId });
      setDiff(result);
    } catch (e) {
      setStatus(`preview error: ${e}`);
    }
  }, []);

  const startSync = useCallback(async (peerId: string) => {
    setSyncing(true);
    try {
      const bytes = await invoke<number>("sync_start", { peerId });
      setStatus(`synced ${bytes} bytes`);
      setDiff(null);
      await listPairs();
    } catch (e) {
      setStatus(`sync error: ${e}`);
    }
    setSyncing(false);
  }, [listPairs]);

  const fmtBytes = (b: number) => {
    if (b < 1024) return `${b} B`;
    if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
    return `${(b / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="sync-panel">
      <div className="sync-panel-toolbar">
        <h2>File Sync</h2>
        <button className="btn-icon" onClick={() => setSyncVisible?.(false)} title="Close">✕</button>
      </div>

      <div className="sync-panel-body">
        <div className="sync-section">
          <div className="sync-section-row">
            <input
              className="sync-input"
              placeholder="Local root path (e.g. ~/.neotrix)"
              value={localRoot}
              onChange={(e) => setLocalRoot(e.target.value)}
            />
            <button className="btn btn-primary" onClick={initSync}>Init</button>
          </div>
          <div className="sync-status">{status}</div>
        </div>

        <div className="sync-section">
          <div className="sync-section-header">
            <span>Auto-Sync Status</span>
          </div>
          <div className="sync-status-row">
            <span className={`sync-indicator ${syncStatus}`} />
            <span>Status: {syncStatus}</span>
            {lastSyncTime && (
              <span>Last sync: {new Date(lastSyncTime).toLocaleTimeString()}</span>
            )}
            {lastFilesSynced > 0 && <span>Synced: {lastFilesSynced} files</span>}
          </div>
        </div>

        <div className="sync-section">
          <div className="sync-section-header">
            <span>Discovered Peers</span>
            <button className="btn btn-small" onClick={discover} disabled={scanning}>
              {scanning ? "Scanning..." : "Scan"}
            </button>
          </div>
          {peers.length === 0 && <div className="sync-empty">No peers found. Click Scan to discover.</div>}
          {peers.map((p) => (
            <div key={p.id} className="sync-peer-row">
              <div className="sync-peer-info">
                <strong>{p.name}</strong>
                <span className="sync-peer-addr">{p.host}:{p.port}</span>
              </div>
              <button className="btn btn-small btn-primary" onClick={() => addPair(p.id)}>Add</button>
            </div>
          ))}
        </div>

        <div className="sync-section">
          <div className="sync-section-header">
            <span>Sync Pairs</span>
            <button className="btn btn-small" onClick={listPairs}>Refresh</button>
          </div>
          {pairs.length === 0 && <div className="sync-empty">No sync pairs configured.</div>}
          {pairs.map((p) => (
            <div key={p.peer_id} className="sync-pair-card">
              <div className="sync-pair-header">
                <strong>{p.peer_name}</strong>
                <span className={`sync-pair-status ${p.status === "Idle" ? "idle" : "active"}`}>{p.status}</span>
              </div>
              <div className="sync-pair-actions">
                <button className="btn btn-small" onClick={() => preview(p.peer_id)}>Preview</button>
                <button className="btn btn-small btn-primary" onClick={() => startSync(p.peer_id)} disabled={syncing}>
                  {syncing ? "Syncing..." : "Sync Now"}
                </button>
                <button className="btn btn-small btn-danger" onClick={() => removePair(p.peer_id)}>Remove</button>
              </div>
              {p.last_sync && <div className="sync-pair-meta">Last sync: {new Date(p.last_sync * 1000).toLocaleString()}</div>}
            </div>
          ))}
        </div>

        {diff && (
          <div className="sync-section">
            <div className="sync-section-header">
              <span>Sync Preview ({fmtBytes(diff.total_bytes)} total)</span>
              <button className="btn btn-small" onClick={() => setDiff(null)}>Clear</button>
            </div>
            {diff.to_send.length > 0 && (
              <div className="sync-diff-group">
                <div className="sync-diff-label">To Send ({diff.to_send.length})</div>
                {diff.to_send.map((f, i) => <div key={i} className="sync-file-row send">{f}</div>)}
              </div>
            )}
            {diff.to_receive.length > 0 && (
              <div className="sync-diff-group">
                <div className="sync-diff-label">To Receive ({diff.to_receive.length})</div>
                {diff.to_receive.map((f, i) => <div key={i} className="sync-file-row receive">{f}</div>)}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default SyncPanel;

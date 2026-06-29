import { useState, useEffect } from 'react';

interface McpServerConfig {
  name: string;
  command: string;
  args: string[];
  enabled: boolean;
  env: Record<string, string>;
}

export function McpSettings() {
  const [servers, setServers] = useState<McpServerConfig[]>([]);
  const [editing, setEditing] = useState<McpServerConfig | null>(null);

  useEffect(() => {
    loadServers();
  }, []);

  async function loadServers() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<McpServerConfig[]>('mcp_list_servers');
      setServers(result);
    } catch { /* Tauri not available */ }
  }

  async function toggleServer(name: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('mcp_toggle_server', { name });
      loadServers();
    } catch { /* ignore */ }
  }

  async function deleteServer(name: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('mcp_delete_server', { name });
      loadServers();
    } catch { /* ignore */ }
  }

  async function saveServer(config: McpServerConfig) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('mcp_save_server', { config });
      setEditing(null);
      loadServers();
    } catch { /* ignore */ }
  }

  return (
    <div style={{ padding: '16px' }}>
      <h3 style={{ marginBottom: 12, fontSize: 14, fontWeight: 600 }}>MCP Servers</h3>
      
      {servers.map((s) => (
        <div key={s.name} style={{
          display: 'flex', alignItems: 'center', gap: 8,
          padding: '8px 12px', marginBottom: 6,
          background: 'rgba(255,255,255,0.05)', borderRadius: 8,
          fontSize: 13,
        }}>
          <div style={{ flex: 1 }}>
            <div style={{ fontWeight: 500 }}>{s.name}</div>
            <div style={{ opacity: 0.6, fontSize: 12 }}>{s.command} {s.args.join(' ')}</div>
          </div>
          <button onClick={() => toggleServer(s.name)}
            style={{
              padding: '4px 10px', borderRadius: 6, border: '1px solid rgba(255,255,255,0.1)',
              background: s.enabled ? 'rgba(52,199,89,0.2)' : 'rgba(255,255,255,0.05)',
              color: s.enabled ? '#34C759' : '#888', cursor: 'pointer', fontSize: 12,
            }}>
            {s.enabled ? 'ON' : 'OFF'}
          </button>
          <button onClick={() => setEditing({...s})}
            style={{ padding: '4px 8px', borderRadius: 6, border: 'none', background: 'rgba(255,255,255,0.1)', cursor: 'pointer', fontSize: 12 }}>
            Edit
          </button>
          <button onClick={() => deleteServer(s.name)}
            style={{ padding: '4px 8px', borderRadius: 6, border: 'none', background: 'rgba(255,59,48,0.2)', color: '#FF3B30', cursor: 'pointer', fontSize: 12 }}>
            ×
          </button>
        </div>
      ))}

      <button onClick={() => setEditing({
        name: '', command: '', args: [], enabled: false, env: {},
      })}
        style={{ marginTop: 8, padding: '6px 14px', borderRadius: 8, border: '1px dashed rgba(255,255,255,0.2)',
          background: 'transparent', color: '#888', cursor: 'pointer', width: '100%', fontSize: 13 }}>
        + Add MCP Server
      </button>

      {editing && (
        <div style={{
          position: 'fixed', inset: 0, display: 'flex', alignItems: 'center', justifyContent: 'center',
          background: 'rgba(0,0,0,0.5)', zIndex: 1000,
        }}>
          <div style={{
            background: '#1C1C1E', borderRadius: 12, padding: 24, width: 400,
            border: '1px solid rgba(255,255,255,0.1)',
          }}>
            <h4 style={{ marginTop: 0, marginBottom: 16 }}>{editing.name ? 'Edit' : 'Add'} MCP Server</h4>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
              <label style={{ fontSize: 12, opacity: 0.6 }}>Name</label>
              <input value={editing.name} onChange={e => setEditing({...editing, name: e.target.value})}
                style={{ padding: '8px 12px', borderRadius: 8, border: '1px solid rgba(255,255,255,0.1)', background: 'rgba(0,0,0,0.3)', color: '#fff' }} />
              
              <label style={{ fontSize: 12, opacity: 0.6 }}>Command</label>
              <input value={editing.command} onChange={e => setEditing({...editing, command: e.target.value})}
                style={{ padding: '8px 12px', borderRadius: 8, border: '1px solid rgba(255,255,255,0.1)', background: 'rgba(0,0,0,0.3)', color: '#fff' }} />
              
              <label style={{ fontSize: 12, opacity: 0.6 }}>Arguments (space-separated)</label>
              <input value={editing.args.join(' ')} onChange={e => setEditing({...editing, args: e.target.value.split(' ').filter(Boolean)})}
                style={{ padding: '8px 12px', borderRadius: 8, border: '1px solid rgba(255,255,255,0.1)', background: 'rgba(0,0,0,0.3)', color: '#fff' }} />
            </div>
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 16 }}>
              <button onClick={() => setEditing(null)}
                style={{ padding: '8px 16px', borderRadius: 8, border: 'none', background: 'rgba(255,255,255,0.1)', cursor: 'pointer' }}>
                Cancel
              </button>
              <button onClick={() => saveServer(editing)}
                style={{ padding: '8px 16px', borderRadius: 8, border: 'none', background: '#0A84FF', color: '#fff', cursor: 'pointer' }}>
                Save
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

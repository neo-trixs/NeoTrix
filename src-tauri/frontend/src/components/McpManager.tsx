import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store";

interface McpServerConfig {
  name: string;
  command: string;
  args: string[];
  enabled: boolean;
  env: Record<string, string>;
}

const McpManager: React.FC = () => {
  const setMcpManagerVisible = useStore((s) => s.setMcpManagerVisible);
  const [servers, setServers] = useState<McpServerConfig[]>([]);
  const [editing, setEditing] = useState<McpServerConfig | null>(null);
  const [envInput, setEnvInput] = useState("");
  const [toolView, setToolView] = useState<string | null>(null);

  const loadServers = useCallback(async () => {
    try {
      const result = await invoke<McpServerConfig[]>("mcp_list_servers");
      setServers(result);
    } catch { /* Tauri not available */ }
  }, []);

  useEffect(() => { loadServers(); }, [loadServers]);

  const toggleServer = async (name: string) => {
    try {
      await invoke("mcp_toggle_server", { name });
      loadServers();
    } catch { /* ignore */ }
  };

  const deleteServer = async (name: string) => {
    try {
      await invoke("mcp_delete_server", { name });
      loadServers();
    } catch { /* ignore */ }
  };

  const saveServer = async (config: McpServerConfig) => {
    try {
      await invoke("mcp_save_server", { config });
      setEditing(null);
      loadServers();
    } catch { /* ignore */ }
  };

  const openEdit = (server?: McpServerConfig) => {
    if (server) {
      setEditing({ ...server });
      setEnvInput(Object.entries(server.env).map(([k, v]) => `${k}=${v}`).join("\n"));
    } else {
      setEditing({ name: "", command: "", args: [], enabled: false, env: {} });
      setEnvInput("");
    }
  };

  const parseEnvInput = (input: string): Record<string, string> => {
    const env: Record<string, string> = {};
    input.split("\n").filter(Boolean).forEach((line) => {
      const eq = line.indexOf("=");
      if (eq > 0) env[line.slice(0, eq).trim()] = line.slice(eq + 1).trim();
    });
    return env;
  };

  return (
    <div className="evolution-panel">
      <div className="evolution-panel-toolbar">
        <div className="evolution-panel-toolbar-left">
          <h2>MCP Servers</h2>
          <span className="evo-iteration-badge">{servers.length}</span>
        </div>
        <div className="evolution-panel-toolbar-right">
          <button className="btn-icon" onClick={loadServers} title="Refresh">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
              <path d="M1 7a6 6 0 0 1 11.2-3M13 7a6 6 0 0 1-11.2 3" />
              <path d="M12.2 1v3h-3M1.8 13v-3h3" />
            </svg>
          </button>
          <button className="btn-icon" onClick={() => setMcpManagerVisible(false)} title="Close">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <path d="M3 3l8 8M11 3l-8 8" />
            </svg>
          </button>
        </div>
      </div>

      <div className="evolution-panel-body">
        <div className="evo-card" style={{ borderTopColor: "#0a84ff" }}>
          <div className="evo-card-title">Servers</div>
          <div className="evo-card-body">
            {servers.length === 0 && (
              <div className="evo-flag-clear" style={{ padding: "8px 0", textAlign: "center", opacity: 0.6 }}>
                No MCP servers configured
              </div>
            )}
            {servers.map((s) => (
              <div key={s.name} style={{
                display: "flex", alignItems: "center", gap: 10,
                padding: "8px 10px", marginBottom: 6,
                background: "rgba(255,255,255,0.04)", borderRadius: 8,
              }}>
                <span style={{
                  width: 8, height: 8, borderRadius: "50%", flexShrink: 0,
                  background: s.enabled ? "#34C759" : "rgba(255,255,255,0.2)",
                }} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontWeight: 500, fontSize: 13 }}>{s.name}</div>
                  <div style={{ opacity: 0.5, fontSize: 11, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {s.command} {s.args.join(" ")}
                  </div>
                  {Object.keys(s.env).length > 0 && (
                    <div style={{ opacity: 0.4, fontSize: 11, marginTop: 2 }}>
                      {Object.keys(s.env).length} env var(s)
                    </div>
                  )}
                </div>
                <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
                  <button onClick={() => toggleServer(s.name)}
                    style={{
                      padding: "3px 10px", borderRadius: 6, border: "1px solid rgba(255,255,255,0.12)",
                      background: s.enabled ? "rgba(52,199,89,0.2)" : "rgba(255,255,255,0.05)",
                      color: s.enabled ? "#34C759" : "#888", cursor: "pointer", fontSize: 11, fontWeight: 500,
                    }}>
                    {s.enabled ? "ON" : "OFF"}
                  </button>
                  <button onClick={() => openEdit(s)}
                    style={{ padding: "3px 8px", borderRadius: 6, border: "none", background: "rgba(255,255,255,0.1)", color: "#ccc", cursor: "pointer", fontSize: 11 }}>
                    Edit
                  </button>
                  <button onClick={() => deleteServer(s.name)}
                    style={{ padding: "3px 8px", borderRadius: 6, border: "none", background: "rgba(255,59,48,0.2)", color: "#FF3B30", cursor: "pointer", fontSize: 11 }}>
                    ×
                  </button>
                </div>
              </div>
            ))}
            <button onClick={() => openEdit()}
              style={{
                marginTop: 4, padding: "6px 14px", borderRadius: 8, border: "1px dashed rgba(255,255,255,0.15)",
                background: "transparent", color: "#888", cursor: "pointer", width: "100%", fontSize: 12,
              }}>
              + Add MCP Server
            </button>
          </div>
        </div>

        <div className="evo-card" style={{ borderTopColor: "#34c759", marginTop: 12 }}>
          <div className="evo-card-title">
            <span>Tools</span>
            {toolView && (
              <button className="btn-ghost btn-sm" onClick={() => setToolView(null)} style={{ marginLeft: 8 }}>
                ← Back
              </button>
            )}
          </div>
          <div className="evo-card-body">
            {!toolView ? (
              servers.filter((s) => s.enabled).map((s) => (
                <div key={s.name}
                  onClick={() => setToolView(s.name)}
                  style={{
                    display: "flex", alignItems: "center", gap: 8,
                    padding: "8px 10px", marginBottom: 4,
                    background: "rgba(255,255,255,0.04)", borderRadius: 8,
                    cursor: "pointer", fontSize: 13,
                  }}>
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3" style={{ opacity: 0.5, flexShrink: 0 }}>
                    <rect x="1" y="1" width="10" height="10" rx="2" />
                    <path d="M1 4h10" />
                  </svg>
                  <span style={{ flex: 1 }}>{s.name}</span>
                  <span style={{ opacity: 0.4, fontSize: 11 }}>{s.command}</span>
                </div>
              ))
            ) : (
              <div style={{ padding: 4, fontSize: 13 }}>
                <div style={{ fontWeight: 500, marginBottom: 8 }}>{toolView}</div>
                {(servers.find((s) => s.name === toolView)?.args.length ?? 0) > 0 && (
                  <div style={{ opacity: 0.6, marginBottom: 4 }}>
                    Args: {servers.find((s) => s.name === toolView)?.args.join(" ")}
                  </div>
                )}
                {Object.keys(servers.find((s) => s.name === toolView)?.env ?? {}).length > 0 && (
                  <div style={{ opacity: 0.6 }}>
                    Env: {Object.entries(servers.find((s) => s.name === toolView)?.env ?? {}).map(([k, v]) => `${k}=${v}`).join(", ")}
                  </div>
                )}
                {!servers.find((s) => s.name === toolView) && (
                  <div style={{ opacity: 0.5 }}>Server not found</div>
                )}
              </div>
            )}
            {servers.filter((s) => s.enabled).length === 0 && !toolView && (
              <div className="evo-flag-clear" style={{ padding: "8px 0", textAlign: "center", opacity: 0.5, fontSize: 12 }}>
                No enabled servers
              </div>
            )}
          </div>
        </div>
      </div>

      {editing && (
        <div style={{
          position: "fixed", inset: 0, display: "flex", alignItems: "center", justifyContent: "center",
          background: "rgba(0,0,0,0.5)", zIndex: 1000,
        }} onClick={() => setEditing(null)}>
          <div style={{
            background: "#1C1C1E", borderRadius: 12, padding: 24, width: 420,
            border: "1px solid rgba(255,255,255,0.1)",
            maxHeight: "80vh", overflow: "auto",
          }} onClick={(e) => e.stopPropagation()}>
            <h4 style={{ marginTop: 0, marginBottom: 16, fontSize: 15, fontWeight: 600 }}>
              {editing.name ? `Edit: ${editing.name}` : "Add MCP Server"}
            </h4>
            <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
              <label style={{ fontSize: 12, opacity: 0.6 }}>Name</label>
              <input value={editing.name} onChange={(e) => setEditing({ ...editing, name: e.target.value })}
                placeholder="my-server"
                style={{ padding: "8px 12px", borderRadius: 8, border: "1px solid rgba(255,255,255,0.1)", background: "rgba(0,0,0,0.3)", color: "#fff", fontSize: 13 }} />

              <label style={{ fontSize: 12, opacity: 0.6 }}>Command</label>
              <input value={editing.command} onChange={(e) => setEditing({ ...editing, command: e.target.value })}
                placeholder="npx"
                style={{ padding: "8px 12px", borderRadius: 8, border: "1px solid rgba(255,255,255,0.1)", background: "rgba(0,0,0,0.3)", color: "#fff", fontSize: 13 }} />

              <label style={{ fontSize: 12, opacity: 0.6 }}>Arguments (space-separated)</label>
              <input value={editing.args.join(" ")} onChange={(e) => setEditing({ ...editing, args: e.target.value.split(" ").filter(Boolean) })}
                placeholder="-y @modelcontextprotocol/server-filesystem /path"
                style={{ padding: "8px 12px", borderRadius: 8, border: "1px solid rgba(255,255,255,0.1)", background: "rgba(0,0,0,0.3)", color: "#fff", fontSize: 13 }} />

              <label style={{ fontSize: 12, opacity: 0.6 }}>Env vars (KEY=VALUE, one per line)</label>
              <textarea value={envInput} onChange={(e) => {
                setEnvInput(e.target.value);
                setEditing({ ...editing, env: parseEnvInput(e.target.value) });
              }}
                placeholder="API_KEY=sk-..."
                rows={4}
                style={{ padding: "8px 12px", borderRadius: 8, border: "1px solid rgba(255,255,255,0.1)", background: "rgba(0,0,0,0.3)", color: "#fff", fontSize: 13, fontFamily: "monospace", resize: "vertical" }} />
            </div>
            <div style={{ display: "flex", gap: 8, justifyContent: "flex-end", marginTop: 16 }}>
              <button onClick={() => setEditing(null)}
                style={{ padding: "8px 16px", borderRadius: 8, border: "none", background: "rgba(255,255,255,0.1)", color: "#ccc", cursor: "pointer", fontSize: 13 }}>
                Cancel
              </button>
              <button onClick={() => saveServer(editing)}
                disabled={!editing.name}
                style={{
                  padding: "8px 16px", borderRadius: 8, border: "none",
                  background: editing.name ? "#0A84FF" : "rgba(10,132,255,0.3)",
                  color: editing.name ? "#fff" : "rgba(255,255,255,0.3)",
                  cursor: editing.name ? "pointer" : "default", fontSize: 13,
                }}>
                Save
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default McpManager;

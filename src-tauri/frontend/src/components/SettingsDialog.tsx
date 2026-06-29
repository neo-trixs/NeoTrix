import React, { useState, useEffect } from "react";
import { useStore } from "../store";
import type { AppSettings } from "../types";
import * as api from "../lib/api";
import ProviderConfig from "./ProviderConfig";
import KnowledgeBase from "./KnowledgeBase";

interface McpServer {
  name: string;
  command: string;
  args: string[];
  enabled: boolean;
}

type SettingsTab = "general" | "provider" | "knowledge" | "shortcuts" | "mcp";

const TABS: { id: SettingsTab; label: string; icon: string }[] = [
  { id: "general", label: "General", icon: "G" },
  { id: "provider", label: "Provider", icon: "P" },
  { id: "knowledge", label: "Knowledge", icon: "K" },
  { id: "shortcuts", label: "Shortcuts", icon: "S" },
  { id: "mcp", label: "MCP Servers", icon: "M" },
];

const SettingsDialog: React.FC = () => {
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");

  const settings = useStore((s) => s.settings);
  const setSettings = useStore((s) => s.setSettings);
  const providerConfig = useStore((s) => s.providerConfig);
  const setProviderConfig = useStore((s) => s.setProviderConfig);
  const knowledgeBase = useStore((s) => s.knowledgeBase);
  const setKnowledgeBase = useStore((s) => s.setKnowledgeBase);
  const setShowSettings = useStore((s) => s.setShowSettings);

  const [localSettings, setLocalSettings] = useState(settings);
  const [dirty, setDirty] = useState(false);

  const [mcpServers, setMcpServers] = useState<McpServer[]>([]);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newName, setNewName] = useState("");
  const [newCommand, setNewCommand] = useState("");
  const [newArgs, setNewArgs] = useState("");

  const [knowledgeResults, setKnowledgeResults] = useState<{ id: string; title: string; content: string; relevance: number }[]>([]);

  useEffect(() => {
    if (activeTab === "mcp") {
      (async () => {
        try {
          const { invoke } = await import("@tauri-apps/api/core");
          const servers = await invoke<McpServer[]>("mcp_list_servers");
          setMcpServers(servers);
        } catch {
          setMcpServers([]);
        }
      })();
    }
  }, [activeTab]);

  const handleSaveSettings = async () => {
    setSettings(localSettings);
    setDirty(false);
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("save_provider_config", { payload: providerConfig });
    } catch {
      /* persistence best-effort */
    }
  };

  const handleTestProvider = async (_config: typeof providerConfig): Promise<boolean> => {
    return api.testProviderConnection(_config);
  };

  const handleRefreshProvider = async () => {
    try {
      const config = await api.getCurrentProvider();
      setProviderConfig(config);
    } catch {
      /* refresh best-effort */
    }
  };

  const handleSearchKnowledge = async (query: string) => {
    if (!query.trim()) {
      setKnowledgeResults([]);
      return;
    }
    const results = await api.searchKnowledge(query);
    setKnowledgeResults(results);
  };

  const handleMcpToggle = async (name: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("mcp_toggle_server", { name });
      setMcpServers((prev) =>
        prev.map((s) => (s.name === name ? { ...s, enabled: !s.enabled } : s))
      );
    } catch {
      /* toggle best-effort */
    }
  };

  const handleMcpSave = async (server: McpServer) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("mcp_save_server", { config: server });
      setMcpServers((prev) =>
        prev.map((s) => (s.name === server.name ? server : s))
      );
    } catch {
      /* save best-effort */
    }
  };

  const handleMcpDelete = async (name: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("mcp_delete_server", { name });
      setMcpServers((prev) => prev.filter((s) => s.name !== name));
    } catch {
      /* delete best-effort */
    }
  };

  const handleMcpAdd = async () => {
    if (!newName.trim() || !newCommand.trim()) return;
    const server: McpServer = {
      name: newName.trim(),
      command: newCommand.trim(),
      args: newArgs
        .split(" ")
        .map((a) => a.trim())
        .filter(Boolean),
      enabled: true,
    };
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("mcp_save_server", { config: server });
      setMcpServers((prev) => [...prev, server]);
    } catch {
      /* add best-effort */
    }
    setNewName("");
    setNewCommand("");
    setNewArgs("");
    setShowAddForm(false);
  };

  return (
    <div className="settings-overlay" onClick={() => setShowSettings(false)}>
      <div className="settings-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="settings-close-btn" onClick={() => setShowSettings(false)}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>

        <div className="settings-body">
          <div className="settings-sidebar">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                className={`settings-tab-btn ${activeTab === tab.id ? "active" : ""}`}
                onClick={() => setActiveTab(tab.id)}
              >
                <span style={{ width: 20, textAlign: "center", flexShrink: 0 }}>{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </div>

          <div className="settings-content">
            {activeTab === "general" && (
              <div className="settings-section">
                <div className="settings-section-title">General Preferences</div>
                <div className="settings-group">
                  <label>Theme</label>
                  <select
                    value={localSettings.theme}
                    onChange={(e) => {
                      setLocalSettings({ ...localSettings, theme: e.target.value as AppSettings["theme"] });
                      setDirty(true);
                    }}
                  >
                    <option value="light">Light</option>
                    <option value="dark">Dark</option>
                    <option value="system">System</option>
                  </select>
                </div>

                <div className="settings-group">
                  <label>Font Size ({localSettings.fontSize}px)</label>
                  <input
                    type="range"
                    min="11"
                    max="20"
                    step="1"
                    value={localSettings.fontSize}
                    onChange={(e) => {
                      setLocalSettings({ ...localSettings, fontSize: parseInt(e.target.value) });
                      setDirty(true);
                    }}
                  />
                </div>

                <div className="settings-group">
                  <label>Language</label>
                  <select
                    value={localSettings.language}
                    onChange={(e) => {
                      setLocalSettings({ ...localSettings, language: e.target.value as AppSettings["language"] });
                      setDirty(true);
                    }}
                  >
                    <option value="zh-CN">中文</option>
                    <option value="en-US">English</option>
                  </select>
                </div>

                <div className="settings-group" style={{ flexDirection: "row", alignItems: "center", gap: 10 }}>
                  <button
                    className={`toggle ${localSettings.autoSave ? "active" : ""}`}
                    onClick={() => {
                      setLocalSettings({ ...localSettings, autoSave: !localSettings.autoSave });
                      setDirty(true);
                    }}
                  >
                    <span className="toggle-knob" />
                  </button>
                  <label style={{ margin: 0 }}>Auto-save sessions</label>
                </div>

                <div className="settings-group">
                  <label>Terminal Path</label>
                  <input
                    type="text"
                    value={localSettings.terminalPath}
                    onChange={(e) => {
                      setLocalSettings({ ...localSettings, terminalPath: e.target.value });
                      setDirty(true);
                    }}
                    placeholder="/bin/zsh"
                  />
                </div>

                <div className="settings-group">
                  <label>Max Sessions</label>
                  <input
                    type="number"
                    min="1"
                    max="100"
                    value={localSettings.maxSessions}
                    onChange={(e) => {
                      setLocalSettings({ ...localSettings, maxSessions: parseInt(e.target.value) || 20 });
                      setDirty(true);
                    }}
                  />
                </div>
              </div>
            )}

            {activeTab === "provider" && (
              <div className="settings-section">
                <div style={{ display: "flex", justifyContent: "flex-end", marginBottom: 8 }}>
                  <button className="btn-secondary" onClick={handleRefreshProvider}>
                    Refresh
                  </button>
                </div>
                <ProviderConfig
                  config={providerConfig}
                  onSave={setProviderConfig}
                  onTest={handleTestProvider}
                />
              </div>
            )}

            {activeTab === "knowledge" && (
              <div className="settings-section">
                <KnowledgeBase
                  entries={knowledgeBase}
                  onAdd={(entry) => {
                    const newEntry = { ...entry, id: `k-${Date.now()}`, created: Date.now(), updated: Date.now() };
                    setKnowledgeBase([...knowledgeBase, newEntry as any]);
                  }}
                  onDelete={(id) => setKnowledgeBase(knowledgeBase.filter((e) => e.id !== id))}
                  onSearch={handleSearchKnowledge}
                />
                {knowledgeResults.length > 0 && (
                  <div className="knowledge-search-results">
                    <div className="settings-section-title" style={{ marginTop: 16 }}>Search Results</div>
                    {knowledgeResults.map((r, i) => (
                      <div key={r.id || i} className="knowledge-entry" style={{ marginTop: 8 }}>
                        <div className="knowledge-entry-title">{r.title}</div>
                        <div className="knowledge-entry-source" style={{ fontSize: 12, opacity: 0.7 }}>
                          Relevance: {(r.relevance * 100).toFixed(0)}%
                        </div>
                        <div style={{ fontSize: 12, marginTop: 4, opacity: 0.8 }}>{r.content.slice(0, 200)}</div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {activeTab === "shortcuts" && (
              <div className="settings-section">
                <div className="settings-section-title">Keyboard Shortcuts</div>
                {[
                  { keys: "Cmd+K", desc: "Command palette" },
                  { keys: "Cmd+L", desc: "Focus input" },
                  { keys: "Cmd+,", desc: "Open settings" },
                  { keys: "Cmd+.", desc: "Cycle mode (Chat/Plan/Agent)" },
                  { keys: "Cmd+B", desc: "Toggle sidebar" },
                  { keys: "Cmd+/", desc: "Show shortcuts" },
                  { keys: "Cmd+T", desc: "New session" },
                  { keys: "Cmd+W", desc: "Close session" },
                  { keys: "⇧⌘E", desc: "Evolution panel" },
                  { keys: "⇧⌘P", desc: "Proxy panel" },
                  { keys: "⇧⌘S", desc: "Sandbox panel" },
                  { keys: "⇧⌘D", desc: "Consciousness dashboard" },
                  { keys: "⇧⌘M", desc: "Moments feed" },
                ].map((s, i) => (
                  <div key={i} className="shortcuts-row">
                    <span className="shortcuts-label">{s.desc}</span>
                    <span className="shortcuts-keys">
                      {s.keys.split("+").map((k, j) => (
                        <React.Fragment key={j}>
                          {j > 0 && <span className="shortcuts-plus">+</span>}
                          <kbd className="shortcuts-key">{k}</kbd>
                        </React.Fragment>
                      ))}
                    </span>
                  </div>
                ))}
              </div>
            )}

            {activeTab === "mcp" && (
              <div className="settings-section">
                <div className="settings-section-title">MCP Servers</div>
                <div style={{ display: "flex", justifyContent: "flex-end", marginBottom: 8 }}>
                  <button className="btn-primary" onClick={() => setShowAddForm(!showAddForm)}>
                    {showAddForm ? "Cancel" : "Add Server"}
                  </button>
                </div>

                {showAddForm && (
                  <div className="mcp-form" style={{ marginBottom: 16 }}>
                    <div className="settings-group">
                      <label>Name</label>
                      <input
                        type="text"
                        value={newName}
                        onChange={(e) => setNewName(e.target.value)}
                        placeholder="my-server"
                      />
                    </div>
                    <div className="settings-group">
                      <label>Command</label>
                      <input
                        type="text"
                        value={newCommand}
                        onChange={(e) => setNewCommand(e.target.value)}
                        placeholder="npx"
                      />
                    </div>
                    <div className="settings-group">
                      <label>Args (space-separated)</label>
                      <input
                        type="text"
                        value={newArgs}
                        onChange={(e) => setNewArgs(e.target.value)}
                        placeholder="-y @modelcontextprotocol/server-filesystem /path"
                      />
                    </div>
                    <button className="btn-primary" onClick={handleMcpAdd} style={{ marginTop: 8 }}>
                      Save
                    </button>
                  </div>
                )}

                {mcpServers.length === 0 ? (
                  <p style={{ opacity: 0.6, fontStyle: "italic" }}>No MCP servers configured.</p>
                ) : (
                  mcpServers.map((server) => (
                    <div key={server.name} className="mcp-server-row" style={{ marginBottom: 12 }}>
                      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                        <div style={{ fontWeight: 600 }}>{server.name}</div>
                        <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                          <button
                            className={`toggle ${server.enabled ? "active" : ""}`}
                            onClick={() => handleMcpToggle(server.name)}
                            style={{ margin: 0 }}
                          >
                            <span className="toggle-knob" />
                          </button>
                          <button
                            className="btn-icon"
                            onClick={() => handleMcpDelete(server.name)}
                            title="Delete server"
                          >
                            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                              <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
                            </svg>
                          </button>
                        </div>
                      </div>
                      <div style={{ fontSize: 12, opacity: 0.7, fontFamily: "monospace", marginTop: 4 }}>
                        {server.command} {server.args.join(" ")}
                      </div>
                    </div>
                  ))
                )}
              </div>
            )}
          </div>
        </div>

        <div className="settings-footer">
          <button className="btn-secondary" onClick={() => setShowSettings(false)}>Close</button>
          {activeTab === "general" && dirty && (
            <button className="btn-primary" onClick={handleSaveSettings}>Save</button>
          )}
        </div>
      </div>
    </div>
  );
};

export default SettingsDialog;

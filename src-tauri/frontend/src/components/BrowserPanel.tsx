import React, { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import * as api from "../lib/api";

type PanelTab = "browser" | "search" | "tools" | "credentials" | "agents" | "xfeed";

export const BrowserPanel: React.FC = () => {
  const [tab, setTab] = React.useState<PanelTab>("browser");

  return (
    <div className="browser-panel">
      <div className="browser-tabs">
        <button className={`browser-tab ${tab === "browser" ? "active" : ""}`} onClick={() => setTab("browser")}>🌐 Browser</button>
        <button className={`browser-tab ${tab === "xfeed" ? "active" : ""}`} onClick={() => setTab("xfeed")}>𝕏 Feed</button>
        <button className={`browser-tab ${tab === "search" ? "active" : ""}`} onClick={() => setTab("search")}>🔍 Search</button>
        <button className={`browser-tab ${tab === "credentials" ? "active" : ""}`} onClick={() => setTab("credentials")}>🔑 Login</button>
        <button className={`browser-tab ${tab === "agents" ? "active" : ""}`} onClick={() => setTab("agents")}>🤖 Agents</button>
        <button className={`browser-tab ${tab === "tools" ? "active" : ""}`} onClick={() => setTab("tools")}>⚡ Tools</button>
      </div>

      <div className="browser-content">
        {tab === "browser" && <BrowserView />}
        {tab === "xfeed" && <XFeedView />}
        {tab === "search" && <SearchView />}
        {tab === "credentials" && <CredentialsView />}
        {tab === "agents" && <AgentsView />}
        {tab === "tools" && <ToolsView />}
      </div>
    </div>
  );
};

// ─── X Feed Tab ──────────────────────────────────────────────────────────────

const XFeedView: React.FC = () => {
  const [status, setStatus] = useState("");
  const [xStatus, setXStatus] = useState<api.XAutoScrollStatus | null>(null);
  const [profile, setProfile] = useState<api.XHumanProfile | null>(null);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const pollRef = useRef<number | null>(null);

  useEffect(() => {
    if (xStatus?.session_active) {
      pollRef.current = window.setInterval(async () => {
        try {
          const s = await api.browserXStatus();
          setXStatus(s);
        } catch { /* ignore */ }
      }, 3000);
    }
    return () => { if (pollRef.current) clearInterval(pollRef.current); };
  }, [xStatus?.session_active]);

  useEffect(() => {
    const unlisten = listen<api.XAbsorptionEvent>("x:knowledge-absorbed", (event) => {
      const e = event.payload;
      setStatus(`Absorbed ${e.count} tweets (avg N=${e.avg_negentropy.toFixed(3)})`);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const handleStart = useCallback(async () => {
    setStatus("Starting X session...");
    try {
      const msg = await api.browserXStartSession();
      setStatus(msg);
      setXStatus(await api.browserXStatus());
      const p = await api.browserXHumanProfile();
      setProfile(p);
    } catch (e: any) {
      setStatus(`Error: ${e}`);
    }
  }, []);

  const handleLogin = useCallback(async () => {
    if (!username || !password) { setStatus("Username and password required"); return; }
    setStatus("Logging in to X.com...");
    try {
      const msg = await api.browserXLogin(username, password);
      setStatus(msg);
      setXStatus(await api.browserXStatus());

      // 登录成功后自动打开浏览器窗口到 X.com
      try {
        await api.browserOpen("https://x.com/home");
        setStatus("Login complete — X.com opened in browser");
      } catch {
        setStatus("Login complete (browser window unavailable)");
      }
    } catch (e: any) {
      setStatus(`Login error: ${e}`);
    }
  }, [username, password]);

  const handleHumanScroll = useCallback(async () => {
    setStatus("Auto-scrolling with human behavior...");
    try {
      const msg = await api.browserXHumanScroll();
      setStatus(msg);
      setXStatus(await api.browserXStatus());
    } catch (e: any) {
      setStatus(`Scroll error: ${e}`);
    }
  }, []);

  const handleStop = useCallback(async () => {
    setStatus("Stopping session...");
    try {
      const msg = await api.browserXStopSession();
      setStatus(msg);
      setXStatus(null);
      setProfile(null);
    } catch (e: any) {
      setStatus(`Error: ${e}`);
    }
  }, []);

  const handleRefreshStatus = useCallback(async () => {
    try {
      setXStatus(await api.browserXStatus());
      setProfile(await api.browserXHumanProfile());
      setStatus("Status refreshed");
    } catch {
      setStatus("No active session");
    }
  }, []);

  const sessionActive = xStatus?.session_active ?? false;

  return (
    <div className="xfeed-view">
      <div className="xfeed-header">
        <h3>𝕏 自动浏览</h3>
        <span className={`agent-status-badge ${sessionActive ? "active" : "inactive"}`}>
          {sessionActive ? "Session Active" : "Disconnected"}
        </span>
      </div>

      <div className="xfeed-controls">
        <div className="cred-form" style={{ marginBottom: 8 }}>
          <input className="cred-input" value={username} onChange={e => setUsername(e.target.value)} placeholder="X.com username/email" disabled={sessionActive} />
          <input className="cred-input" type="password" value={password} onChange={e => setPassword(e.target.value)} placeholder="X.com password" disabled={sessionActive} />
        </div>
        <div className="xfeed-buttons">
          {!sessionActive ? (
            <>
              <button className="btn-primary btn-sm" onClick={handleStart}>▶ Start Session</button>
              <button className="btn-sm" onClick={handleLogin} disabled={!username || !password}>🔑 Log In</button>
            </>
          ) : (
            <>
              <button className="btn-sm" onClick={handleHumanScroll}>🔄 Auto-Scroll 30s</button>
              <button className="btn-sm" onClick={handleRefreshStatus}>↻ Refresh</button>
              <button className="btn-sm" onClick={handleStop}>⏹ Stop Session</button>
            </>
          )}
        </div>
      </div>

      {status && <div className="browser-status">{status}</div>}

      {xStatus && (
        <div className="browser-info" style={{ marginTop: 8 }}>
          <div className="browser-info-row"><span className="info-label">Status:</span> {xStatus.running ? "Scrolling..." : "Idle"}</div>
          <div className="browser-info-row"><span className="info-label">Tweets Seen:</span> {xStatus.tweet_count}</div>
          <div className="browser-info-row"><span className="info-label">Absorbed:</span> {xStatus.absorbed}</div>
          <div className="browser-info-row"><span className="info-label">Avg Negentropy:</span> {xStatus.negentropy_avg.toFixed(3)}</div>
          <div className="browser-info-row"><span className="info-label">URL:</span> {xStatus.current_url || "—"}</div>
        </div>
      )}

      {profile && (
        <details style={{ marginTop: 8, fontSize: 12, opacity: 0.7 }}>
          <summary>Human Behavior Profile (anti-detection)</summary>
          <div className="browser-info" style={{ marginTop: 4 }}>
            <div className="browser-info-row"><span className="info-label">Scroll Speed:</span> {(profile.scroll_speed * 100).toFixed(0)}%</div>
            <div className="browser-info-row"><span className="info-label">Pause Range:</span> {profile.pause_range[0]}–{profile.pause_range[1]}ms</div>
            <div className="browser-info-row"><span className="info-label">Variance:</span> {(profile.scroll_variance * 100).toFixed(0)}%</div>
            <div className="browser-info-row"><span className="info-label">Mouse Trail:</span> {profile.mouse_trail ? "Yes" : "No"}</div>
            <div className="browser-info-row"><span className="info-label">Interaction Rate:</span> {(profile.interaction_rate * 100).toFixed(0)}%</div>
            <div className="browser-info-row"><span className="info-label">UA:</span> <span style={{ fontSize: 10, wordBreak: "break-all" }}>{profile.user_agent}</span></div>
          </div>
        </details>
      )}

      {!sessionActive && (
        <div className="browser-placeholder" style={{ marginTop: 16 }}>
          <div className="placeholder-icon">𝕏</div>
          <div className="placeholder-text">Start a session and log in to auto-browse X.com</div>
          <div className="placeholder-hint">Human-like scrolling • Anti-detection • Negentropy absorption</div>
        </div>
      )}
    </div>
  );
};

// ─── Browser Tab ────────────────────────────────────────────────────────────

const BrowserView: React.FC = () => {
  const [url, setUrl] = React.useState("https://web.whatsapp.com");
  const [browserState, setBrowserState] = React.useState<api.BrowserState | null>(null);
  const [status, setStatus] = React.useState("");

  const handleOpen = useCallback(async () => {
    setStatus("Opening browser...");
    try {
      const state = await api.browserOpen(url);
      setBrowserState(state);
      setStatus(`Opened: ${state.url}`);

      // 自动检测 WebApp + 无痕采集
      const agent = await api.browserAgentDetect(url, state.title);
      if (agent) {
        setStatus(`Detected: ${agent.name} — ${agent.actions.length} actions available`);
      }

      // 延迟后自动提取内容入队意识管道 (无痕采集)
      setTimeout(async () => {
        try {
          setStatus("Collecting page content...");
          const extractResult = await api.browserExtractContent(url);
          setStatus(`Content collected: ${extractResult.title}`);
        } catch (_) {
          // 静默
        }
      }, 3000);
    } catch (e: any) {
      setStatus(`Error: ${e}`);
    }
  }, [url]);

  const handleClose = useCallback(async () => {
    try {
      await api.browserClose();
      setBrowserState(null);
      setStatus("Browser closed");
    } catch (e: any) {
      setStatus(`Error: ${e}`);
    }
  }, []);

  const handleExtract = useCallback(async () => {
    setStatus("Extracting content...");
    try {
      const result = await api.browserExtractContent(url);
      setStatus(`Extracted: ${result.title} — ${result.summary}`);
    } catch (e: any) {
      setStatus(`Extract error: ${e}`);
    }
  }, [url]);

  return (
    <div className="browser-view">
      <div className="browser-toolbar">
        <input
          className="browser-url-input"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleOpen()}
          placeholder="Enter URL..."
        />
        <button className="btn-primary btn-sm" onClick={handleOpen} title="Open in browser window">Open</button>
        {browserState?.is_open && (
          <>
            <button className="btn-sm" onClick={async () => { try { await api.browserBack(); setStatus("Back"); } catch(e: any) { setStatus(`Error: ${e}`); } }} title="Go back">◀</button>
            <button className="btn-sm" onClick={async () => { try { await api.browserForward(); setStatus("Forward"); } catch(e: any) { setStatus(`Error: ${e}`); } }} title="Go forward">▶</button>
            <button className="btn-sm" onClick={handleExtract} title="Extract page content">Extract</button>
            <button className="btn-sm" onClick={async () => { try { await api.browserReload(); setStatus("Reloaded"); } catch(e: any) { setStatus(`Error: ${e}`); } }} title="Reload">↻</button>
            <button className="btn-sm" onClick={handleClose} title="Close browser">✕</button>
          </>
        )}
      </div>

      {browserState?.is_open && (
        <div className="browser-info">
          <div className="browser-info-row"><span className="info-label">Title:</span> {browserState.title}</div>
          <div className="browser-info-row"><span className="info-label">URL:</span> {browserState.url}</div>
        </div>
      )}

      {status && <div className="browser-status">{status}</div>}

      {!browserState?.is_open && (
        <div className="browser-placeholder">
          <div className="placeholder-icon">🌐</div>
          <div className="placeholder-text">Open a web page to start browsing</div>
          <div className="placeholder-hint">Supports WhatsApp Web, Gmail, GitHub, and more</div>
        </div>
      )}
    </div>
  );
};

// ─── Search Tab ─────────────────────────────────────────────────────────────

const SearchView: React.FC = () => {
  const [query, setQuery] = React.useState("");
  const [results, setResults] = React.useState<api.SearchResultItem[]>([]);
  const [searching, setSearching] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  const handleSearch = useCallback(async () => {
    if (!query.trim()) return;
    setSearching(true);
    setError(null);
    try {
      const items = await api.toolSearch(query, 8);
      setResults(items);
    } catch (e: any) {
      setError(e.toString());
      setResults([]);
    } finally {
      setSearching(false);
    }
  }, [query]);

  return (
    <div className="search-view">
      <div className="search-bar">
        <input
          className="search-input"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSearch()}
          placeholder="Search the web..."
        />
        <button className="btn-primary btn-sm" onClick={handleSearch} disabled={searching}>
          {searching ? "..." : "Search"}
        </button>
      </div>

      {error && <div className="search-error">{error}</div>}

      <div className="search-results">
        {results.map((r, i) => (
          <div key={i} className="search-result-card" onClick={() => window.open(r.url, "_blank")}>
            <div className="result-title">{r.title || r.url}</div>
            <div className="result-url">{r.url}</div>
            <div className="result-snippet">{r.snippet || "No description"}</div>
          </div>
        ))}
        {results.length === 0 && !searching && !error && (
          <div className="search-placeholder">
            <div className="placeholder-icon">🔍</div>
            <div className="placeholder-text">Enter a query to search</div>
          </div>
        )}
      </div>
    </div>
  );
};

// ─── Credentials Tab ────────────────────────────────────────────────────────

const CredentialsView: React.FC = () => {
  const [credentials, setCredentials] = React.useState<api.CredentialInfo[]>([]);
  const [domain, setDomain] = React.useState("");
  const [username, setUsername] = React.useState("");
  const [password, setPassword] = React.useState("");
  const [notes, setNotes] = React.useState("");
  const [status, setStatus] = React.useState("");

  const loadCredentials = useCallback(async () => {
    try {
      const list = await api.browserCredentialList();
      setCredentials(list);
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => { loadCredentials(); }, [loadCredentials]);

  const handleStore = useCallback(async () => {
    if (!domain || !username || !password) {
      setStatus("Domain, username, and password required");
      return;
    }
    try {
      await api.browserCredentialStore(domain, username, password, notes || undefined);
      setStatus("Credential saved");
      setDomain(""); setUsername(""); setPassword(""); setNotes("");
      loadCredentials();
    } catch (e: any) {
      setStatus(`Error: ${e}`);
    }
  }, [domain, username, password, notes, loadCredentials]);

  const handleAutofill = useCallback(async (d: string) => {
    try {
      const result = await api.browserCredentialAutofill(d);
      setStatus(result);
    } catch (e: any) {
      setStatus(`Error: ${e}`);
    }
  }, []);

  const handleDelete = useCallback(async (id: string) => {
    try {
      await api.browserCredentialRemove(id);
      loadCredentials();
    } catch (e: any) {
      setStatus(`Error: ${e}`);
    }
  }, [loadCredentials]);

  return (
    <div className="credentials-view">
      {status && <div className="cred-status">{status}</div>}

      <div className="cred-form">
        <input className="cred-input" value={domain} onChange={e => setDomain(e.target.value)} placeholder="Domain (e.g. whatsapp.com)" />
        <input className="cred-input" value={username} onChange={e => setUsername(e.target.value)} placeholder="Username / Email" />
        <input className="cred-input" type="password" value={password} onChange={e => setPassword(e.target.value)} placeholder="Password" />
        <input className="cred-input" value={notes} onChange={e => setNotes(e.target.value)} placeholder="Notes (optional)" />
        <button className="btn-primary btn-sm" onClick={handleStore}>Save Credential</button>
      </div>

      <div className="cred-list">
        {credentials.map(c => (
          <div key={c.id} className="cred-item">
            <div className="cred-item-domain">{c.domain}</div>
            <div className="cred-item-user">{c.username}</div>
            <div className="cred-item-actions">
              <button className="btn-sm" onClick={() => handleAutofill(c.domain)} title="Auto-fill in browser">🔑 Fill</button>
              <button className="btn-sm" onClick={() => handleDelete(c.id)} title="Delete">✕</button>
            </div>
          </div>
        ))}
        {credentials.length === 0 && (
          <div className="cred-empty">No saved credentials</div>
        )}
      </div>
    </div>
  );
};

// ─── Agents Tab ─────────────────────────────────────────────────────────────

const AgentsView: React.FC = () => {
  const [agents, setAgents] = React.useState<api.WebAppAgentInfo[]>([]);
  const [status, setStatus] = React.useState("");
  const [pendingAction, setPendingAction] = React.useState<string | null>(null);

  const loadAgents = useCallback(async () => {
    try {
      const list = await api.browserAgentList();
      setAgents(list);
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => { loadAgents(); }, [loadAgents]);

  // 监听 agent-updated 事件, 自动刷新列表
  useEffect(() => {
    const unlisten = listen("browser:agent-updated", () => {
      loadAgents();
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [loadAgents]);

  const handleAction = useCallback(async (agentId: string, actionId: string, actionLabel: string) => {
    setPendingAction(actionId);
    setStatus(`Executing: ${actionLabel}...`);
    try {
      const result = await api.browserAgentExecute(agentId, actionId);
      setStatus(result);
    } catch (e: any) {
      setStatus(`Error: ${e}`);
    } finally {
      setPendingAction(null);
    }
  }, []);

  return (
    <div className="agents-view">
      {status && <div className="agent-status">{status}</div>}

      {agents.map(agent => (
        <div key={agent.id} className="agent-card">
          <div className="agent-header">
            <span className="agent-name">{agent.name}</span>
            <span className={`agent-status-badge ${agent.is_active ? "active" : "inactive"}`}>
              {agent.is_active ? "Active" : "Inactive"}
            </span>
          </div>
          <div className="agent-url">{agent.url_pattern}</div>
          <div className="agent-actions">
            {agent.actions.map(action => (
              <button
                key={action.id}
                className="btn-sm agent-action-btn"
                onClick={() => handleAction(agent.id, action.id, action.label)}
                disabled={pendingAction === action.id}
              >
                {pendingAction === action.id ? "..." : action.label}
              </button>
            ))}
          </div>
        </div>
      ))}

      {agents.length === 0 && (
        <div className="agents-placeholder">
          <div className="placeholder-icon">🤖</div>
          <div className="placeholder-text">No web app agents detected</div>
          <div className="placeholder-hint">Open a web app (WhatsApp, Gmail, GitHub) in the Browser tab to create an agent</div>
        </div>
      )}

      <button className="btn-sm" onClick={loadAgents} style={{ marginTop: 8 }}>
        ↻ Refresh Agents
      </button>
    </div>
  );
};

// ─── Tools Tab ──────────────────────────────────────────────────────────────

const ToolsView: React.FC = () => {
  const [input, setInput] = React.useState("");
  const [logs, setLogs] = React.useState<{ tool: string; success: boolean; output: string; duration: number }[]>([]);

  const handleExecute = useCallback(async () => {
    const trimmed = input.trim();
    if (!trimmed) return;

    const parts = trimmed.split(/\s+/);
    const tool = parts[0];
    const args: Record<string, unknown> = {};

    if (parts.length > 1) {
      if (tool === "bash") {
        args.command = parts.slice(1).join(" ");
      } else if (tool === "read" || tool === "write" || tool === "edit") {
        args.path = parts[1];
        if (tool === "write") args.content = parts.slice(2).join(" ");
        if (tool === "edit") {
          args.old = parts[2] || "";
          args.new = parts.slice(3).join(" ");
        }
      } else if (tool === "webfetch") {
        args.url = parts[1];
      } else {
        args.query = parts.slice(1).join(" ");
      }
    }

    try {
      const result = await api.toolExecute(tool, args);
      setLogs(prev => [{ tool, success: result.success, output: result.output.slice(0, 200), duration: result.duration_ms }, ...prev].slice(0, 50));
    } catch (e: any) {
      setLogs(prev => [{ tool, success: false, output: e.toString(), duration: 0 }, ...prev].slice(0, 50));
    }
    setInput("");
  }, [input]);

  return (
    <div className="tools-view">
      <div className="tools-input-row">
        <input
          className="tools-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleExecute()}
          placeholder="read /write /bash 'cmd' /webfetch url ..."
        />
        <button className="btn-primary btn-sm" onClick={handleExecute}>Run</button>
      </div>

      <div className="tools-log">
        {logs.map((log, i) => (
          <div key={i} className={`tool-log-entry ${log.success ? "success" : "error"}`}>
            <div className="tool-log-header">
              <span className="tool-name">{log.tool}</span>
              <span className="tool-status">{log.success ? "✓" : "✗"}</span>
              <span className="tool-duration">{log.duration}ms</span>
            </div>
            <div className="tool-log-output">{log.output}</div>
          </div>
        ))}
        {logs.length === 0 && (
          <div className="tools-placeholder">
            <div className="placeholder-icon">⚡</div>
            <div className="placeholder-text">Run a tool command</div>
          </div>
        )}
      </div>
    </div>
  );
};

export default BrowserPanel;

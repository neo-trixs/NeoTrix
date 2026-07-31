import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useStore } from "../stores";
import { ChatView, CommandPalette, ModelSelector, SessionSidebar, SettingsView, ShortcutHelp } from "../components/neocodex";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import styles from "./NeoCodexPage.module.css";

const THEME_ORDER = ["light", "dark", "system"] as const;

const THEME_LABELS: Record<string, string> = {
  light: "主题: 浅色",
  dark: "主题: 深色",
  system: "主题: 跟随系统",
};

export default function NeoCodexPage() {
  const {
    settings,
    setSettings,
    neocodexMode,
    neocodexMessages,
    neocodexStreaming,
    neocodexSessions,
    neocodexActiveSessionId,
    setNeoCodexMode,
    setNeoCodexMessages,
    addNeoCodexMessage,
    setNeoCodexStreaming,
    setNeoCodexSessions,
    setNeoCodexActiveSession,
  } = useStore();

  const [agentBusy, setAgentBusy] = React.useState(false);
  const [showSidebar, setShowSidebar] = React.useState(true);
  const [showSettings, setShowSettings] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [shortcutHelpOpen, setShortcutHelpOpen] = useState(false);
  const [focusMode, setFocusMode] = useState(false);
  const [health, setHealth] = useState<any>(null);
  const [viewMode, setViewMode] = useState<"verbose" | "normal" | "summary">("normal");
  const stopRef = useRef(false);
  const [sideChatOpen, setSideChatOpen] = useState(false);
  const [sideChatMessages, setSideChatMessages] = useState<Array<{ role: string; content: string }>>([]);
  const [sideChatInput, setSideChatInput] = useState("");

  // Load sessions on mount
  useEffect(() => {
    const loadSessions = async () => {
      try {
        const sessions = await invoke("neocodex_list_sessions") as any[];
        setNeoCodexSessions(sessions);
      } catch (e) {
        console.error("Failed to load sessions:", e);
      }
    };
    loadSessions();
  }, []);

  const loadHealth = useCallback(async () => {
    try {
      setHealth(await invoke("neocodex_health_report"));
    } catch {}
  }, []);

  useEffect(() => {
    loadHealth();
    const timer = setInterval(loadHealth, 15000);
    return () => clearInterval(timer);
  }, [loadHealth]);

  const handleStop = () => {
    stopRef.current = true;
  };

  const handleSend = async (content: string) => {
    stopRef.current = false;
    setAgentBusy(true);
    addNeoCodexMessage({ role: "user", content, timestamp: Date.now() });
    setNeoCodexStreaming({ content: "", role: "assistant" });

    let accumulated = "";
    const unlisten = await listen<string>("neocodex_stream_token", (event) => {
      accumulated += event.payload;
      setNeoCodexStreaming({ content: accumulated, role: "assistant" });
      if (stopRef.current) return;
    });

    try {
      const response = await invoke("neocodex_send_message_stream", { content }) as string;
      setNeoCodexStreaming(null);
      addNeoCodexMessage({ role: "assistant", content: response, timestamp: Date.now() });
      refreshSessions();
      loadHealth();
    } catch (e) {
      console.error("Send failed:", e);
      setNeoCodexStreaming(null);
      addNeoCodexMessage({ role: "error", content: `Error: ${e}`, timestamp: Date.now() });
    } finally {
      stopRef.current = false;
      unlisten();
      setAgentBusy(false);
    }
  };

  const refreshSessions = async () => {
    try {
      const sessions = await invoke("neocodex_list_sessions") as any[];
      setNeoCodexSessions(sessions);
    } catch (e) {
      console.error("Failed to refresh sessions:", e);
    }
  };

  const handleModeChange = async (mode: string) => {
    try {
      await invoke("neocodex_set_mode", { mode });
      setNeoCodexMode(mode as "Agent" | "Shell" | "Plan");
    } catch (e) {
      console.error("Mode set failed:", e);
    }
  };

  const handleSessionSelect = async (session: any) => {
    try {
      await invoke("neocodex_switch_session", { sessionId: session.id });
      setNeoCodexActiveSession(session.id);
      const items = await invoke("neocodex_get_session_messages", { sessionId: session.id }) as any[];
      setNeoCodexMessages(items.map((m) => ({
        role: m.role,
        content: m.content,
        contentType: "markdown",
        timestamp: m.timestamp,
      })));
    } catch (e) {
      console.error("Switch session failed:", e);
    }
  };

  const handleSessionDelete = (sessionId: string) => {
    if (neocodexActiveSessionId === sessionId) {
      setNeoCodexActiveSession(null);
      setNeoCodexMessages([]);
    }
    refreshSessions();
  };

  const usage = health?.context_usage || 0;
  const usagePct = Math.round(usage * 100);
  const usageColor = usage < 0.7 ? "var(--success)" : usage <= 0.9 ? "var(--warning)" : "var(--danger)";
  const usageCirc = 2 * Math.PI * 7;

  const viewModeLabel = viewMode === "verbose" ? "详细" : viewMode === "normal" ? "正常" : "摘要";

  const cycleViewMode = () => {
    setViewMode((v) => (v === "verbose" ? "normal" : v === "normal" ? "summary" : "verbose"));
  };

  const handleSideChatSend = () => {
    const content = sideChatInput.trim();
    if (!content) return;
    setSideChatMessages((prev) => [...prev, { role: "user", content }]);
    setSideChatInput("");
    setSideChatMessages((prev) => [...prev, { role: "assistant", content: "侧聊仅本地记录，不影响主会话。" }]);
  };

  // Command palette items
  const paletteItems = useMemo(() => {
    const items: Array<{ id: string; label: string; hint?: string; onSelect: () => void }> = [
      { id: "new", label: "新建会话", hint: "⌘N", onSelect: () => window.dispatchEvent(new CustomEvent("neotrix:new-session")) },
      { id: "settings", label: "设置", hint: "⌘,", onSelect: () => setShowSettings(true) },
      { id: "sidebar", label: showSidebar ? "收起侧栏" : "展开侧栏", hint: "⌘B", onSelect: () => setShowSidebar((v) => !v) },
      { id: "focus", label: focusMode ? "退出专注模式" : "专注模式", hint: "⌘Shift+F", onSelect: () => setFocusMode((v) => !v) },
      { id: "viewmode", label: "切换视图模式", hint: "Ctrl+O", onSelect: () => cycleViewMode() },
      { id: "sidechat", label: "侧聊", hint: "⌘+;", onSelect: () => setSideChatOpen((v) => !v) },
    ];
    (["Agent", "Shell", "Plan"] as const).forEach((m) => {
      items.push({ id: `mode-${m}`, label: `切换到 ${m} 模式`, hint: "Mode", onSelect: () => handleModeChange(m) });
    });
    neocodexSessions.forEach((s: any) => {
      items.push({ id: `session-${s.id}`, label: s.name || "未命名会话", hint: s.mode, onSelect: () => handleSessionSelect(s) });
    });
    return items;
  }, [neocodexSessions, showSidebar, focusMode]);

  // Keyboard shortcuts: Cmd+K palette, Cmd+N new session, Cmd+B toggle sidebar, Ctrl+Tab cycle, Cmd+Shift+F focus
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setPaletteOpen((v) => !v);
      } else if (e.key === "/" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setShortcutHelpOpen((v) => !v);
      } else if (e.key === "n" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        window.dispatchEvent(new CustomEvent("neotrix:new-session"));
      } else if (e.key === "b" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setShowSidebar((v) => !v);
      } else if (e.key === "f" && e.metaKey && e.shiftKey) {
        e.preventDefault();
        setFocusMode((v) => !v);
      } else if (e.key === "o" && e.ctrlKey) {
        e.preventDefault();
        setViewMode(v => v === "verbose" ? "normal" : v === "normal" ? "summary" : "verbose");
      } else if (e.key === "Escape" && agentBusy) {
        e.preventDefault();
        handleStop();
      } else if (e.key === "w" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        if (neocodexActiveSessionId) handleSessionDelete(neocodexActiveSessionId);
      } else if (e.key === ";" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setSideChatOpen((v) => !v);
      } else if (e.key === "Tab" && e.ctrlKey) {
        e.preventDefault();
        if (neocodexSessions.length === 0) return;
        const idx = neocodexSessions.findIndex((s: any) => s.id === neocodexActiveSessionId);
        const next = neocodexSessions[(idx + 1) % neocodexSessions.length];
        handleSessionSelect(next);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [neocodexSessions, neocodexActiveSessionId, focusMode, agentBusy]);

  return (
    <div className={styles.container}>
      {showSidebar && (
        <aside className={styles.sidebar}>
          <div className={styles.sidebarHeader}>
            <h2>NeoCodex</h2>
            <button className={styles.sidebarToggle} onClick={() => setShowSidebar(false)}>
              <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M10 4l-4 3 4 3" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </button>
          </div>
          <SessionSidebar
            activeSessionId={neocodexActiveSessionId}
            onSessionSelect={handleSessionSelect}
            onSessionDelete={handleSessionDelete}
          />
        </aside>
      )}

      <main className={`${styles.main} ${focusMode ? styles.focusMode : ""}`}>
        <header className={`${styles.topBar} ${focusMode ? styles.focusMode : ""}`}>
          <button className={styles.mobileMenuBtn} onClick={() => setShowSidebar(true)}>
            <svg width="20" height="20" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M4 4l4 3-4 3" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
          <div className={styles.topBarCenter}>
            <ModelSelector />
            <select
              value={neocodexMode}
              onChange={(e) => handleModeChange(e.target.value)}
              className={styles.modeSelect}
              disabled={agentBusy}
              title="Mode"
            >
              <option value="Agent">Agent</option>
              <option value="Shell">Shell</option>
              <option value="Plan">Plan</option>
            </select>
          </div>
          <div className={styles.topBarRight}>
            <button
              className={`${styles.settingsBtn} ${styles.viewModeBtn}`}
              onClick={cycleViewMode}
              title={`视图模式: ${viewModeLabel}（Ctrl+O 切换）`}
            >
              {viewModeLabel}
            </button>
            <button
              className={styles.settingsBtn}
              onClick={() => {
                const idx = THEME_ORDER.indexOf(settings.theme as (typeof THEME_ORDER)[number]);
                setSettings({ ...settings, theme: THEME_ORDER[(idx + 1) % THEME_ORDER.length] });
              }}
              title={THEME_LABELS[settings.theme]}
            >
              <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <circle cx="7" cy="7" r="2.6" />
                <path d="M7 1v2M7 11v2M1 7h2M11 7h2M2.9 2.9l1.4 1.4M9.7 9.7l1.4 1.4M2.9 11.1l1.4-1.4M9.7 4.3l1.4-1.4" strokeLinecap="round" />
              </svg>
            </button>
            <button
              className={`${styles.settingsBtn} ${showSettings ? styles.settingsActive : ""}`}
              onClick={() => setShowSettings(!showSettings)}
              title={showSettings ? "返回对话" : "设置"}
            >
              <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <circle cx="7" cy="7" r="2.2" />
                <path d="M7 1.5v1.8M7 10.7v1.8M1.5 7h1.8M10.7 7h1.8M3.1 3.1l1.3 1.3M9.6 9.6l1.3 1.3M3.1 10.9l1.3-1.3M9.6 4.4l1.3-1.3" strokeLinecap="round" />
              </svg>
            </button>
            <button
              className={styles.settingsBtn}
              onClick={() => setFocusMode((v) => !v)}
              title={focusMode ? "退出专注" : "专注模式"}
            >
              <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <circle cx="7" cy="7" r="3" />
                <path d="M7 1v2M7 11v2M1 7h2M11 7h2" strokeLinecap="round" />
              </svg>
            </button>
            <button className={styles.settingsBtn} title={`上下文用量 ${usagePct}%`}>
              <span className={styles.usageRing}>
                <svg width="30" height="30" viewBox="0 0 20 20">
                  <circle cx="10" cy="10" r="7" fill="none" stroke="var(--border-primary)" strokeWidth="2" />
                  <circle
                    cx="10"
                    cy="10"
                    r="7"
                    fill="none"
                    stroke={usageColor}
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeDasharray={`${(usageCirc * usage).toFixed(2)} ${usageCirc.toFixed(2)}`}
                    transform="rotate(-90 10 10)"
                  />
                </svg>
                <span className={styles.usageRingText}>{usagePct}%</span>
              </span>
            </button>
          </div>
        </header>

        <div className={styles.chatArea}>
          {showSettings ? (
            <SettingsView />
          ) : (
            <ChatView
              messages={neocodexMessages}
              streamingContent={neocodexStreaming?.content}
              streamingRole={neocodexStreaming?.role}
              agentBusy={agentBusy}
              viewMode={viewMode}
              onSend={handleSend}
              onStop={handleStop}
              onDelete={(idx) => setNeoCodexMessages(neocodexMessages.filter((_, i) => i !== idx))}
            />
          )}
          {sideChatOpen && (
            <div className={styles.sideChat}>
              <div className={styles.sideChatHeader}>
                <span>侧聊</span>
                <button className={styles.sideChatClose} onClick={() => setSideChatOpen(false)}>✕</button>
              </div>
              <div className={styles.sideChatMessages}>
                {sideChatMessages.map((m, i) => (
                  <div
                    key={i}
                    className={`${styles.sideChatMsg} ${m.role === "user" ? styles.sideChatMsgUser : styles.sideChatMsgAssistant}`}
                  >
                    {m.content}
                  </div>
                ))}
              </div>
              <div className={styles.sideChatInputRow}>
                <input
                  className={styles.sideChatInput}
                  value={sideChatInput}
                  onChange={(e) => setSideChatInput(e.target.value)}
                  onKeyDown={(e) => { if (e.key === "Enter") handleSideChatSend(); }}
                  placeholder="输入内容..."
                />
                <button className={styles.sideChatSend} onClick={handleSideChatSend}>发送</button>
              </div>
              <div className={styles.sideChatHint}>侧聊不写入主会话 ⌘+; 关闭</div>
            </div>
          )}
        </div>

        <footer className={styles.statusBar}>
          <span className={styles.statusItem}>
            <span className={styles.statusDot} style={{ background: agentBusy ? "var(--success)" : "var(--fg-tertiary)" }} />
            {agentBusy ? "运行中…" : "就绪"}
          </span>
          <span className={styles.statusItem}>{neocodexMode}</span>
          <span className={styles.statusItem} title="模型">{health?.provider_model || "—"}</span>
          <span className={styles.statusItem} title="上下文用量">
            Context {health ? `${Math.round((health.context_usage || 0) * 100)}%` : "—"}
          </span>
          <span className={styles.statusItem} title="会话数">{neocodexSessions.length} 会话</span>
        </footer>
      </main>

      <CommandPalette open={paletteOpen} items={paletteItems} onClose={() => setPaletteOpen(false)} />
      <ShortcutHelp open={shortcutHelpOpen} onClose={() => setShortcutHelpOpen(false)} />
    </div>
  );
}

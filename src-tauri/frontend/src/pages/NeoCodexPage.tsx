import React, { useEffect, useState } from "react";
import { useStore } from "../stores";
import { ChatView, ModelSelector, SessionSidebar, SettingsView } from "../components/neocodex";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import styles from "./NeoCodexPage.module.css";

export default function NeoCodexPage() {
  const {
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

  const handleSend = async (content: string) => {
    setAgentBusy(true);
    addNeoCodexMessage({ role: "user", content, timestamp: Date.now() });
    setNeoCodexStreaming({ content: "", role: "assistant" });

    let accumulated = "";
    const unlisten = await listen<string>("neocodex_stream_token", (event) => {
      accumulated += event.payload;
      setNeoCodexStreaming({ content: accumulated, role: "assistant" });
    });

    try {
      const response = await invoke("neocodex_send_message_stream", { content }) as string;
      setNeoCodexStreaming(null);
      addNeoCodexMessage({ role: "assistant", content: response, timestamp: Date.now() });
      refreshSessions();
    } catch (e) {
      console.error("Send failed:", e);
      setNeoCodexStreaming(null);
      addNeoCodexMessage({ role: "error", content: `Error: ${e}`, timestamp: Date.now() });
    } finally {
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

  // Keyboard shortcuts: Cmd+N new session, Cmd+B toggle sidebar, Ctrl+Tab cycle
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "n" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        window.dispatchEvent(new CustomEvent("neotrix:new-session"));
      } else if (e.key === "b" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setShowSidebar((v) => !v);
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
  }, [neocodexSessions, neocodexActiveSessionId]);

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

      <main className={styles.main}>
        <header className={styles.topBar}>
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
              className={`${styles.settingsBtn} ${showSettings ? styles.settingsActive : ""}`}
              onClick={() => setShowSettings(!showSettings)}
              title={showSettings ? "返回对话" : "设置"}
            >
              <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <circle cx="7" cy="7" r="2.2" />
                <path d="M7 1.5v1.8M7 10.7v1.8M1.5 7h1.8M10.7 7h1.8M3.1 3.1l1.3 1.3M9.6 9.6l1.3 1.3M3.1 10.9l1.3-1.3M9.6 4.4l1.3-1.3" strokeLinecap="round" />
              </svg>
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
              onSend={handleSend}
            />
          )}
        </div>
      </main>
    </div>
  );
}

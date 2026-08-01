import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useStore } from "../stores";
import { ChatView, CommandPalette, DiffPane, FileTreePanel, ModelSelector, PreviewPane, SessionSidebar, SettingsView, ShortcutHelp, TerminalPane } from "../components/neocodex";
import type { Attachment } from "../types";
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
  const [fileTreeOpen, setFileTreeOpen] = React.useState(false);
  const [showUsage, setShowUsage] = React.useState(false);
  const [terminalOpen, setTerminalOpen] = React.useState(false);
  const [diffOpen, setDiffOpen] = React.useState(false);
  const [previewOpen, setPreviewOpen] = React.useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [viewsMenuOpen, setViewsMenuOpen] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [shortcutHelpOpen, setShortcutHelpOpen] = useState(false);
  const [focusMode, setFocusMode] = useState(false);
  const [health, setHealth] = useState<any>(null);
  const [gitStatus, setGitStatus] = useState<{ branch: string; dirty: boolean } | null>(null);
  const [updating, setUpdating] = useState(false);
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

  // Native menu → frontend events: Cmd+Shift+U check updates, Cmd+K palette,
  // Cmd+, settings. Auto-check for updates once on launch (engineering gap:
  // Claude/Codex both surface a version banner without manual action).
  const addNotification = useStore((s) => s.addNotification);
  const updatingRef = useRef(false);
  useEffect(() => {
    const runUpdate = async () => {
      if (updatingRef.current) return;
      updatingRef.current = true;
      setUpdating(true);
      try {
        await invoke("neocodex_download_update");
      } catch (e) {
        addNotification({ type: "error", message: `更新失败: ${e}`, duration: 6000 });
      } finally {
        updatingRef.current = false;
        setUpdating(false);
      }
    };
    const notifyUpdate = (r: any) => {
      if (r?.available) {
        addNotification({
          type: "info",
          message: `发现新版本 v${r.latest}（当前 v${r.current}）`,
          duration: 30000,
          action: updatingRef.current ? { label: "下载中…", onClick: () => {} } : { label: "立即更新", onClick: runUpdate },
        });
      }
    };
    const unlistenPalette = listen("neocodex-open-palette", () => setPaletteOpen((v) => !v));
    const unlistenSettings = listen("open-settings", () => setShowSettings(true));
    const unlistenUpdates = listen("neocodex-check-updates", () => {
      invoke("neocodex_check_update").then(notifyUpdate).catch((e) => console.error("Check update failed:", e));
    });
    // Auto-check once shortly after launch (silent unless an update is found).
    const timer = setTimeout(() => {
      invoke("neocodex_check_update").then(notifyUpdate).catch(() => {});
    }, 3000);
    return () => { unlistenPalette.then((f) => f()); unlistenSettings.then((f) => f()); unlistenUpdates.then((f) => f()); clearTimeout(timer); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [addNotification]);

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

  const loadGitStatus = useCallback(async () => {
    try {
      setGitStatus(await invoke("neocodex_git_status"));
    } catch {
      setGitStatus(null);
    }
  }, []);

  useEffect(() => {
    loadGitStatus();
    const timer = setInterval(loadGitStatus, 20000);
    return () => clearInterval(timer);
  }, [loadGitStatus]);

  const handleStop = () => {
    stopRef.current = true;
    invoke("neocodex_stop_stream").catch((e) => console.error("Stop failed:", e));
  };

  const handleSend = async (content: string, attachments?: Attachment[], regenerate?: boolean) => {
    if (agentBusy && !regenerate) return;
    stopRef.current = false;
    setAgentBusy(true);
    if (!regenerate) {
      addNeoCodexMessage({
        role: "user",
        content,
        timestamp: Date.now(),
        attachments: attachments && attachments.length > 0 ? attachments : undefined,
      });
    }
    setNeoCodexStreaming({ content: "", role: "assistant" });

    let accumulated = "";
    let unlisten: (() => void) | undefined;
    let unlistenDone: (() => void) | undefined;
    try {
      unlisten = await listen<string>("neocodex_stream_token", (event) => {
        accumulated += event.payload;
        setNeoCodexStreaming({ content: accumulated, role: "assistant" });
        if (stopRef.current) return;
      });
      unlistenDone = await listen<any>("neocodex_stream_done", (event) => {
        if (event.payload?.cancelled) {
          stopRef.current = true;
        }
      });
    } catch (e) {
      console.error("Stream listen failed:", e);
      setNeoCodexStreaming(null);
      addNeoCodexMessage({ role: "error", content: `Error: ${e}`, timestamp: Date.now() });
      setAgentBusy(false);
      return;
    }

    try {
      const payload = (attachments && attachments.length > 0
        ? { content, attachments: attachments.map((a) => ({ name: a.name, size: a.size, mime_type: a.mimeType, data: a.data })) }
        : { content }) as any;
      if (regenerate) {
        payload.regenerate = true;
      }
      const response = await invoke("neocodex_send_message_stream", payload) as string;
      setNeoCodexStreaming(null);
      const wasCancelled = stopRef.current;
      addNeoCodexMessage({
        role: "assistant",
        content: wasCancelled ? `${response}\n\n> ⏹ 已停止生成` : response,
        timestamp: Date.now(),
      });
      refreshSessions();
      loadHealth();
    } catch (e) {
      console.error("Send failed:", e);
      setNeoCodexStreaming(null);
      addNeoCodexMessage({ role: "error", content: `Error: ${e}`, timestamp: Date.now() });
    } finally {
      stopRef.current = false;
      unlisten?.();
      unlistenDone?.();
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

  const handleNewSession = async () => {
    try {
      const info = await invoke("neocodex_create_session", { name: null }) as any;
      const session = {
        id: info.id,
        name: info.name || "新会话",
        mode: info.mode || "Agent",
        message_count: info.message_count || 0,
        messages: [],
        wire_path: info.wire_path || "",
        created_at: Date.now(),
        updated_at: Date.now(),
      };
      setNeoCodexSessions([
        session,
        ...(useStore.getState().neocodexSessions?.filter((s: any) => s.id !== session.id) || []),
      ]);
      setNeoCodexActiveSession(session.id);
      setNeoCodexMessages([]);
      setSideChatMessages([]);
      // Ensure the sidebar surfaces the new session even if it was hidden.
      setShowSidebar(true);
      setFileTreeOpen(false);
      window.dispatchEvent(new CustomEvent("neotrix:sessions-changed"));
    } catch (e) {
      console.error("Create session failed:", e);
    }
  };

  // Listen for the new-session event at page level so Cmd+N and the palette
  // work regardless of sidebar visibility / active tab (SessionSidebar is
  // unmounted when the sidebar is collapsed or the files tab is shown).
  useEffect(() => {
    const onNew = () => handleNewSession();
    window.addEventListener("neotrix:new-session", onNew);
    return () => window.removeEventListener("neotrix:new-session", onNew);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleModeChange = async (mode: string) => {
    try {
      await invoke("neocodex_set_mode", { mode });
      setNeoCodexMode(mode as "Agent" | "Shell" | "Plan");
    } catch (e) {
      console.error("Mode set failed:", e);
    }
  };

  const sessionSwitchRef = useRef(0);

  const handleSessionSelect = async (session: any) => {
    const token = ++sessionSwitchRef.current;
    try {
      await invoke("neocodex_switch_session", { sessionId: session.id });
      if (sessionSwitchRef.current !== token) return;
      setNeoCodexActiveSession(session.id);
      // Make sure a freshly created session (via sidebar dialog) is reflected
      // in the store list used by the palette + status bar.
      setNeoCodexSessions(
        useStore.getState().neocodexSessions?.some((s: any) => s.id === session.id)
          ? useStore.getState().neocodexSessions
          : [session, ...(useStore.getState().neocodexSessions || [])]
      );
      const items = await invoke("neocodex_get_session_messages", { sessionId: session.id }) as any[];
      if (sessionSwitchRef.current !== token) return;
      setNeoCodexMessages(items.map((m) => ({
        id: m.id,
        role: m.role,
        content: m.content,
        contentType: "markdown",
        timestamp: m.timestamp,
        attachments: m.attachments?.map((a: any) => ({
          id: `${a.name}-${a.size}`,
          name: a.name,
          size: a.size,
          mimeType: a.mime_type,
          data: a.data || "",
        })),
      })));
      const sideChat = await invoke("neocodex_get_side_chat", { sessionId: session.id }) as any[];
      if (sessionSwitchRef.current !== token) return;
      setSideChatMessages(sideChat.map((m) => ({ role: m.role, content: m.content })));
    } catch (e) {
      console.error("Switch session failed:", e);
    }
  };

  const handleSessionDelete = (sessionId: string) => {
    if (neocodexActiveSessionId === sessionId) {
      setNeoCodexActiveSession(null);
      setNeoCodexMessages([]);
      setSideChatMessages([]);
    }
    refreshSessions();
  };

  const reloadMessages = async () => {
    if (!neocodexActiveSessionId) return;
    try {
      const items = await invoke("neocodex_get_session_messages", { sessionId: neocodexActiveSessionId }) as any[];
      setNeoCodexMessages(items.map((m) => ({
        id: m.id,
        role: m.role,
        content: m.content,
        contentType: "markdown",
        timestamp: m.timestamp,
        attachments: m.attachments?.map((a: any) => ({
          id: `${a.name}-${a.size}`,
          name: a.name,
          size: a.size,
          mimeType: a.mime_type,
          data: a.data || "",
        })),
      })));
    } catch (e) {
      console.error("Reload messages failed:", e);
    }
  };

  const handleEditMessage = async (id: number, content: string) => {
    if (!neocodexActiveSessionId) return;
    try {
      await invoke("neocodex_edit_message", { sessionId: neocodexActiveSessionId, index: id, content });
      await reloadMessages();
    } catch (e) {
      console.error("Edit message failed:", e);
    }
  };

  const handleDeleteMessage = async (id: number) => {
    if (!neocodexActiveSessionId) return;
    try {
      await invoke("neocodex_delete_message", { sessionId: neocodexActiveSessionId, index: id });
      await reloadMessages();
    } catch (e) {
      console.error("Delete message failed:", e);
    }
  };

  const handleRegenerateMessage = async (id: number) => {
    if (!neocodexActiveSessionId) return;
    try {
      const items = await invoke("neocodex_regenerate", { sessionId: neocodexActiveSessionId, index: id }) as any[];
      const lastUser = [...items].reverse().find((m) => m.role === "user");
      setNeoCodexMessages(items.map((m) => ({
        id: m.id,
        role: m.role,
        content: m.content,
        contentType: "markdown",
        timestamp: m.timestamp,
      })));
      if (lastUser) {
        handleSend(lastUser.content, undefined, true);
      }
    } catch (e) {
      console.error("Regenerate failed:", e);
    }
  };

  const usage = health?.context_usage || 0;
  const usagePct = Math.round(usage * 100);
  const usageColor = usage < 0.7 ? "var(--success)" : usage <= 0.9 ? "var(--warning)" : "var(--danger)";
  const usageCirc = 2 * Math.PI * 7;

  const viewModeLabel = viewMode === "verbose" ? "详细" : viewMode === "normal" ? "正常" : "摘要";

  const cycleViewMode = () => {
    setViewMode((v) => (v === "verbose" ? "normal" : v === "normal" ? "summary" : "verbose"));
  };

  const handleSideChatSend = async () => {
    const content = sideChatInput.trim();
    if (!content || !neocodexActiveSessionId) return;
    try {
      const items = await invoke("neocodex_send_side_chat", { sessionId: neocodexActiveSessionId, content }) as any[];
      setSideChatMessages(items.map((m) => ({ role: m.role, content: m.content })));
      setSideChatInput("");
    } catch (e) {
      console.error("Side chat send failed:", e);
      addNotification({ type: "error", message: "侧聊发送失败，请重试", duration: 3000 });
    }
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
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName;
      const inEditable = tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target?.isContentEditable;
      // ⌘K/⌘/ palette + help are global (harmless in inputs); everything else
      // must not steal keystrokes the user is typing into a field.
      const isPalette = e.key === "k" && (e.metaKey || e.ctrlKey);
      const isHelp = e.key === "/" && (e.metaKey || e.ctrlKey);
      if (inEditable && !isPalette && !isHelp) return;
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
      } else if (e.key === "Escape" && showUsage) {
        e.preventDefault();
        setShowUsage(false);
      } else if (e.key === "Escape" && viewsMenuOpen) {
        e.preventDefault();
        setViewsMenuOpen(false);
      } else if (e.key === "Escape" && agentBusy) {
        e.preventDefault();
        handleStop();
      } else if (e.key === "w" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        if (neocodexActiveSessionId) handleSessionDelete(neocodexActiveSessionId);
      } else if (e.key === ";" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setSideChatOpen((v) => !v);
      } else if (e.key === "`" && e.ctrlKey) {
        e.preventDefault();
        setTerminalOpen((v) => !v);
      } else if (e.key === "d" && e.metaKey && e.shiftKey) {
        e.preventDefault();
        setDiffOpen((v) => !v);
      } else if (e.key === "p" && e.metaKey && e.shiftKey) {
        e.preventDefault();
        setPreviewOpen((v) => !v);
      } else if (e.key === "Tab" && e.ctrlKey) {
        e.preventDefault();
        if (neocodexSessions.length === 0) return;
        const idx = neocodexSessions.findIndex((s: any) => s.id === neocodexActiveSessionId);
        const delta = e.shiftKey ? -1 : 1;
        const next = neocodexSessions[(idx + delta + neocodexSessions.length) % neocodexSessions.length];
        handleSessionSelect(next);
      } else if (/^[1-9]$/.test(e.key) && (e.metaKey || e.ctrlKey)) {
        // Numbered session switch (Cmd+1..9) — a differentiator neither Claude
        // Code Desktop nor Codex Desktop offers.
        e.preventDefault();
        const n = Number(e.key);
        const target = neocodexSessions[n - 1];
        if (target && target.id !== neocodexActiveSessionId) {
          handleSessionSelect(target);
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [neocodexSessions, neocodexActiveSessionId, focusMode, agentBusy, viewsMenuOpen, showUsage]);

  return (
    <div className={styles.container}>
      {showSidebar && (
        <aside className={styles.sidebar}>
          <div className={styles.sidebarHeader}>
            <h2>NeoCodex</h2>
            <button className={styles.sidebarToggle} onClick={() => setShowSidebar(false)} title="收起侧栏">
              <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M10 4l-4 3 4 3" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </button>
          </div>
          <div className={styles.sidebarTabs}>
            <button
              type="button"
              data-testid="sidebar-tab-sessions"
              className={`${styles.sidebarTab} ${!fileTreeOpen ? styles.sidebarTabActive : ""}`}
              onClick={() => setFileTreeOpen(false)}
            >
              会话
            </button>
            <button
              type="button"
              data-testid="sidebar-tab-files"
              className={`${styles.sidebarTab} ${fileTreeOpen ? styles.sidebarTabActive : ""}`}
              onClick={() => setFileTreeOpen(true)}
              title="文件树"
            >
              文件
            </button>
          </div>
          {fileTreeOpen ? (
            <FileTreePanel onPick={(p) => {
              setFileTreeOpen(false);
              window.dispatchEvent(new CustomEvent("neotrix:mention-file", { detail: p }));
            }} />
          ) : (
            <SessionSidebar
              activeSessionId={neocodexActiveSessionId}
              onSessionSelect={handleSessionSelect}
              onSessionDelete={handleSessionDelete}
              onSessionArchive={() => refreshSessions()}
            />
          )}
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
              data-testid="mode-select"
              title="Mode"
            >
              <option value="Agent">Agent</option>
              <option value="Shell">Shell</option>
              <option value="Plan">Plan</option>
            </select>
            <div className={styles.viewsMenuWrap}>
              <button
                type="button"
                data-testid="views-menu-btn"
                className={`${styles.viewsMenuBtn} ${viewsMenuOpen ? styles.viewsMenuActive : ""}`}
                onClick={() => setViewsMenuOpen((v) => !v)}
                title="视图面板"
              >
                视图 ▾
              </button>
              {viewsMenuOpen && (
                <div className={styles.viewsMenu}>
                  <button
                    type="button"
                    data-testid="views-menu-terminal"
                    className={`${styles.viewsMenuItem} ${terminalOpen ? styles.viewsMenuItemActive : ""}`}
                    onClick={() => { setTerminalOpen((v) => !v); setViewsMenuOpen(false); }}
                  >
                    终端 {terminalOpen ? "✓" : ""}
                  </button>
                  <button
                    type="button"
                    data-testid="views-menu-diff"
                    className={`${styles.viewsMenuItem} ${diffOpen ? styles.viewsMenuItemActive : ""}`}
                    onClick={() => { setDiffOpen((v) => !v); setViewsMenuOpen(false); }}
                  >
                    Diff 查看器 {diffOpen ? "✓" : ""}
                  </button>
                  <button
                    type="button"
                    data-testid="views-menu-preview"
                    className={`${styles.viewsMenuItem} ${previewOpen ? styles.viewsMenuItemActive : ""}`}
                    onClick={() => { setPreviewOpen((v) => !v); setViewsMenuOpen(false); }}
                  >
                    App 预览 {previewOpen ? "✓" : ""}
                  </button>
                </div>
              )}
            </div>
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
            <button
              className={`${styles.settingsBtn} ${showUsage ? styles.settingsActive : ""}`}
              title={`上下文用量 ${usagePct}%（点击查看成本明细）`}
              onClick={() => setShowUsage((v) => !v)}
            >
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

        {showUsage && (
          <div
            className={styles.usagePopover}
            onClick={(e) => {
              if (e.target === e.currentTarget) setShowUsage(false);
            }}
          >
            <div className={styles.usagePopoverTitle}>
              用量 · 成本
              <button className={styles.usageClose} onClick={() => setShowUsage(false)} title="关闭" aria-label="关闭用量弹窗">✕</button>
            </div>
            <div className={styles.usageRow}>
              <span>上下文占用</span>
              <strong>{usagePct}%</strong>
            </div>
            <div className={styles.usageRow}>
              <span>Tokens 已用</span>
              <strong>{health?.tokens_used ?? 0}</strong>
            </div>
            <div className={styles.usageRow}>
              <span>本次会话成本</span>
              <strong>{health?.cost_spent != null ? `$${Number(health.cost_spent).toFixed(4)}` : "—"}</strong>
            </div>
            <div className={styles.usageRow}>
              <span>预算</span>
              <strong>{health?.cost_budget != null ? `$${Number(health.cost_budget).toFixed(2)}` : "—"}</strong>
            </div>
            {health?.provider_model && (
              <div className={styles.usageRow}>
                <span>当前模型</span>
                <strong>{health.provider_model}</strong>
              </div>
            )}
          </div>
        )}

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
              contextUsage={health?.context_usage || 0}
              onSend={handleSend}
              onStop={handleStop}
              onEdit={handleEditMessage}
              onDelete={handleDeleteMessage}
              onRegenerate={handleRegenerateMessage}
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

        {terminalOpen && (
          <div className={styles.pane}>
            <TerminalPane />
          </div>
        )}
        {diffOpen && (
          <div className={styles.pane}>
            <DiffPane />
          </div>
        )}
        {previewOpen && (
          <div className={styles.pane}>
            <PreviewPane />
          </div>
        )}

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
          {gitStatus && (
            <span className={styles.statusItem} title={gitStatus.dirty ? "有未提交改动" : "工作区干净"}>
              <span className={`${styles.statusDot} ${gitStatus.dirty ? styles.gitDirty : ""}`} />
              {gitStatus.branch}
            </span>
          )}
          <span className={styles.statusItem} title="会话数">{neocodexSessions.length} 会话</span>
        </footer>
      </main>

      <CommandPalette open={paletteOpen} items={paletteItems} onClose={() => setPaletteOpen(false)} />
      <ShortcutHelp open={shortcutHelpOpen} onClose={() => setShortcutHelpOpen(false)} />
    </div>
  );
}

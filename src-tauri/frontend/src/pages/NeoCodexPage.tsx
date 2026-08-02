import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useStore } from "../stores";
import { CapabilityHealthPane, ChatView, CommandPalette, DiffPane, FileTreePanel, ModelSelector, PreviewPane, SessionSidebar, ShortcutHelp, TaskPane, TerminalPane } from "../components/neocodex";
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
  const navigate = useNavigate();
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
  const [capabilityOpen, setCapabilityOpen] = React.useState(false);
  const [viewsMenuOpen, setViewsMenuOpen] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [paletteMode, setPaletteMode] = useState<"command" | "file">("command");
  const [fileItems, setFileItems] = useState<Array<{ id: string; label: string; hint?: string; onSelect: () => void }>>([]);
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
  const [renamingTitle, setRenamingTitle] = useState(false);
  const [titleDraft, setTitleDraft] = useState("");
  const [timelineOpen, setTimelineOpen] = useState(false);
  const [checkpoints, setCheckpoints] = useState<Array<{ id: string; created_at: string; message_count: number }>>([]);
  const [checkpointsLoading, setCheckpointsLoading] = useState(false);
  const [pendingCheckpointRestore, setPendingCheckpointRestore] = useState<string | null>(null);
  const [compacting, setCompacting] = useState(false);
  const [taskPaneOpen, setTaskPaneOpen] = useState(false);
  const [taskSteps, setTaskSteps] = useState<Array<{ id: string; name: string; args: string; startedAt: number; status: "running" | "done"; success?: boolean }>>([]);
  const [taskStartedAt, setTaskStartedAt] = useState<number | null>(null);
  const [, setTaskClock] = useState(0);

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
    const unlistenUpdates = listen("neocodex-check-updates", () => {
      invoke("neocodex_check_update").then(notifyUpdate).catch((e) => console.error("Check update failed:", e));
    });
    // Auto-check once shortly after launch (silent unless an update is found).
    const timer = setTimeout(() => {
      invoke("neocodex_check_update").then(notifyUpdate).catch(() => {});
    }, 3000);
    return () => { unlistenPalette.then((f) => f()); unlistenUpdates.then((f) => f()); clearTimeout(timer); };
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
    // Pin the session this send started in. If the user switches sessions while
    // the reply is streaming, stale tokens/messages must not leak into the new
    // session (P1: race fix). `sessionSwitchRef` is bumped by handleSessionSelect.
    const sendSessionId = neocodexActiveSessionId;
    const sendToken = sessionSwitchRef.current;
    setAgentBusy(true);
    // Reset the live task tracker for this turn.
    setTaskSteps([]);
    setTaskStartedAt(Date.now());
    setTaskClock((c) => c + 1);
    if (!regenerate) {
      addNeoCodexMessage({
        role: "user",
        content,
        timestamp: Date.now(),
        attachments: attachments && attachments.length > 0 ? attachments : undefined,
      });
    }
    // If a session switch already raced past our snapshot, abort the send.
    // The token listener for this send would target another session, so bail.
    const stillThisSession = () =>
      sendSessionId === useStore.getState().neocodexActiveSessionId && sendToken === sessionSwitchRef.current;

    const pushMessage = (message: {
      role: "assistant" | "error";
      content: string;
      timestamp: number;
    }) => {
      if (!stillThisSession()) return;
      addNeoCodexMessage(message);
    };

    setNeoCodexStreaming({ content: "", role: "assistant" });

    let accumulated = "";
    let unlisten: (() => void) | undefined;
    let unlistenDone: (() => void) | undefined;
    try {
      unlisten = await listen<string>("neocodex_stream_token", (event) => {
        if (!stillThisSession()) return;
        accumulated += event.payload;
        setNeoCodexStreaming({ content: accumulated, role: "assistant" });
        // Parse tool-call markers (`<tool name="...">args</tool>`) out of the
        // streamed content to feed the live task pane, mirroring how the core
        // extracts structured tool calls from the raw model output.
        const re = /<tool\s+name="([^"]+)">([\s\S]*?)<\/tool>/g;
        let m: RegExpExecArray | null;
        const seen = new Set(taskSteps.map((s) => s.id));
        let dirty = false;
        while ((m = re.exec(accumulated)) !== null) {
          const id = `tool-${m.index}`;
          if (!seen.has(id)) {
            seen.add(id);
            setTaskSteps((prev) => [...prev, { id, name: m![1], args: m![2].trim(), startedAt: Date.now(), status: "running" }]);
            dirty = true;
          }
        }
        if (dirty) setTaskClock((c) => c + 1);
        if (stopRef.current) return;
      });
      unlistenDone = await listen<any>("neocodex_stream_done", (event) => {
        if (event.payload?.cancelled) {
          stopRef.current = true;
        }
      });
    } catch (e) {
      console.error("Stream listen failed:", e);
      if (!stillThisSession()) {
        setAgentBusy(false);
        return;
      }
      setNeoCodexStreaming(null);
      pushMessage({ role: "error", content: `Error: ${e}`, timestamp: Date.now() });
      setAgentBusy(false);
      return;
    }

    try {
      const payload =
        attachments && attachments.length > 0
          ? { content, attachments: attachments.map((a) => ({ name: a.name, size: a.size, mime_type: a.mimeType, data: a.data })) }
          : { content };
      if (regenerate) {
        (payload as any).regenerate = true;
      }
      (payload as any).permission_mode = useStore.getState().settings?.permissionMode || "auto";
      const generated = await invoke("neocodex_send_message_stream", payload) as string;
      setNeoCodexStreaming(null);
      const wasCancelled = stopRef.current;
      pushMessage({
        role: "assistant",
        content: wasCancelled ? `${generated}\n\n> ⏹ 已停止生成` : generated,
        timestamp: Date.now(),
      });
      if (!stillThisSession()) return;
      // All parsed tool steps from this turn are complete once the reply lands.
      setTaskSteps((prev) => prev.map((s) => (s.status === "running" ? { ...s, status: "done" as const, success: true } : s)));
      setTaskStartedAt(null);
      refreshSessions();
      loadHealth();
      // Permission-mode review gate (Claude Code Manual / AcceptEdits parity):
      // in review modes, surface the working-tree diff for per-file accept /
      // reject after every turn instead of silently applying agent edits.
      const permMode = useStore.getState().settings?.permissionMode || "auto";
      if (!wasCancelled && (permMode === "manual" || permMode === "accept")) {
        setDiffOpen(true);
        addNotification({ type: "info", message: permMode === "manual" ? "本轮改动待审阅，请在 Diff 面板逐文件接受/拒绝" : "改动已应用，可在 Diff 面板复核", duration: 3000 });
      }
      // Codex/Claude parity: surface an opt-out notification when a turn
      // completes while the user may be looking elsewhere.
      if (!wasCancelled && useStore.getState().settings?.notifyOnComplete) {
        const active = useStore.getState().neocodexSessions?.find((s: any) => s.id === useStore.getState().neocodexActiveSessionId);
        invoke("send_notification", {
          title: active?.name || "NeoCodex",
          body: "任务已完成，回复已生成",
        }).catch(() => {});
      }
    } catch (e) {
      if (!stillThisSession()) return;
      console.error("Send failed:", e);
      setNeoCodexStreaming(null);
      pushMessage({ role: "error", content: `Error: ${e}`, timestamp: Date.now() });
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
  // The native menu accelerator and the webview keydown handler can both fire
  // this event; dedupe within a short window so a single Cmd+N never creates
  // two sessions.
  useEffect(() => {
    let lastNew = 0;
    const onNew = () => {
      const now = Date.now();
      if (now - lastNew < 300) return;
      lastNew = now;
      handleNewSession();
    };
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

  const activeSession = neocodexSessions.find((s: any) => s.id === neocodexActiveSessionId) ?? null;

  const commitTitleRename = async () => {
    setRenamingTitle(false);
    const name = titleDraft.trim();
    if (!name || !neocodexActiveSessionId) return;
    try {
      await invoke("neocodex_rename_session", { sessionId: neocodexActiveSessionId, name });
      refreshSessions();
    } catch (e) {
      console.error("Rename failed:", e);
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

  const [pendingDeleteSession, setPendingDeleteSession] = useState<string | null>(null);
  const pendingDeleteName = pendingDeleteSession
    ? neocodexSessions.find((s: any) => s.id === pendingDeleteSession)?.name || "此会话"
    : "";

  // Destructive delete is gated behind an explicit confirm. Accidental ⌘W or a
  // stray sidebar click must never irreversibly drop a conversation (P1).
  const requestSessionDelete = (sessionId: string) => {
    setPendingDeleteSession(sessionId);
  };

  const handleSessionDelete = async (sessionId: string) => {
    try {
      await invoke("neocodex_delete_session", { sessionId });
    } catch (e) {
      console.error("Failed to delete session:", e);
    }
    if (neocodexActiveSessionId === sessionId) {
      setNeoCodexActiveSession(null);
      setNeoCodexMessages([]);
      setSideChatMessages([]);
    }
    refreshSessions();
    window.dispatchEvent(new CustomEvent("neotrix:sessions-changed"));
    addNotification({ type: "info", message: "会话已删除", duration: 2000 });
  };

  // Map a backend message `id` (a counter over the full wire thread, including
  // tool/system events) to the zero-based "visible" thread index the backend's
  // edit/delete/regenerate commands expect (only user/assistant messages).
  // Prevents deleting/editing the wrong message when tool events sit between
  // turns (P2: id-vs-index mismatch).
  const visibleIndexFor = (id: number): number => {
    let vis = -1;
    for (const m of neocodexMessages) {
      if (m.id == null) continue;
      if (m.id > id) break;
      if (m.role === "user" || m.role === "assistant") vis += 1;
    }
    return vis;
  };

  const loadCheckpoints = async () => {
    if (!neocodexActiveSessionId) return;
    setCheckpointsLoading(true);
    try {
      const list = await invoke("neocodex_checkpoint_list", { sessionId: neocodexActiveSessionId }) as Array<{ id: string; created_at: string; message_count: number }>;
      setCheckpoints(list);
    } catch (e) {
      console.error("Failed to list checkpoints:", e);
      setCheckpoints([]);
    }
    setCheckpointsLoading(false);
  };

  const openTimeline = () => {
    setTimelineOpen(true);
    loadCheckpoints();
  };

  const handleCheckpointRestore = async (checkpointId: string) => {
    if (!neocodexActiveSessionId) return;
    try {
      await invoke("neocodex_checkpoint_restore", { sessionId: neocodexActiveSessionId, checkpointId });
      await reloadMessages();
      setPendingCheckpointRestore(null);
      addNotification({ type: "success", message: "已回退到检查点", duration: 3000 });
    } catch (e) {
      console.error("Failed to restore checkpoint:", e);
      addNotification({ type: "error", message: `回退失败: ${e}`, duration: 4000 });
      setPendingCheckpointRestore(null);
    }
  };

  const reloadMessages = async () => {
    if (!neocodexActiveSessionId) return;
    try {
      const items = await invoke("neocodex_get_session_messages", { sessionId: neocodexActiveSessionId }) as any[];      setNeoCodexMessages(items.map((m) => ({
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

  const handleCompact = async () => {
    if (!neocodexActiveSessionId) return;
    setCompacting(true);
    try {
      await invoke("neocodex_compact_session", { sessionId: neocodexActiveSessionId });
      await reloadMessages();
      addNotification({ type: "success", message: "上下文已压缩，早期消息已截断", duration: 3000 });
    } catch (e) {
      console.error("Compact failed:", e);
      addNotification({ type: "error", message: `压缩失败: ${e}`, duration: 4000 });
    }
    setCompacting(false);
  };

  const handleEditMessage = async (id: number, content: string) => {
    if (!neocodexActiveSessionId) return;
    try {
      const idx = visibleIndexFor(id);
      if (idx < 0) return;
      await invoke("neocodex_edit_message", { sessionId: neocodexActiveSessionId, index: idx, content });
      await reloadMessages();
    } catch (e) {
      console.error("Edit message failed:", e);
    }
  };

  const handleDeleteMessage = async (id: number) => {
    if (!neocodexActiveSessionId) return;
    const idx = visibleIndexFor(id);
    if (idx < 0) return;
    try {
      await invoke("neocodex_delete_message", { sessionId: neocodexActiveSessionId, index: idx });
      await reloadMessages();
    } catch (e) {
      console.error("Delete message failed:", e);
    }
  };

  const handleRegenerateMessage = async (id: number) => {
    if (!neocodexActiveSessionId) return;
    const idx = visibleIndexFor(id);
    if (idx < 0) return;
    try {
      const items = await invoke("neocodex_regenerate", { sessionId: neocodexActiveSessionId, index: idx }) as any[];
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
      { id: "settings", label: "设置", hint: "⌘,", onSelect: () => navigate("/settings") },
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

  const openFilePalette = useCallback(async () => {
    setPaletteMode("file");
    setFileItems([]);
    setPaletteOpen(true);
    try {
      const files = await invoke<string[]>("neocodex_search_files", { query: "" });
      setFileItems(
        (files || []).slice(0, 100).map((f) => ({
          id: `file-${f}`,
          label: f,
          hint: "文件",
          onSelect: () => window.dispatchEvent(new CustomEvent("neotrix:mention-file", { detail: f })),
        }))
      );
    } catch {
      setFileItems([]);
    }
  }, []);

  // Keyboard shortcuts: Cmd+K palette, Cmd+P file palette, Cmd+N new session, Cmd+B toggle sidebar, Ctrl+Tab cycle, Cmd+Shift+F focus
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName;
      const inEditable = tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target?.isContentEditable;
      // ⌘K/⌘P/⌘/ palette + help are global (harmless in inputs); everything else
      // must not steal keystrokes the user is typing into a field.
      const isPalette = (e.key === "k" || e.key === "p") && (e.metaKey || e.ctrlKey);
      const isHelp = e.key === "/" && (e.metaKey || e.ctrlKey);
      if (inEditable && !isPalette && !isHelp) return;
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setPaletteMode("command");
        setPaletteOpen((v) => !v);
      } else if (e.key === "p" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        openFilePalette();
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
        if (neocodexActiveSessionId) requestSessionDelete(neocodexActiveSessionId);
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
  }, [neocodexSessions, neocodexActiveSessionId, focusMode, agentBusy, viewsMenuOpen, showUsage, openFilePalette]);

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
              onSessionDelete={requestSessionDelete}
              onSessionArchive={() => refreshSessions()}
            />
          )}
        </aside>
      )}

      <main className={`${styles.main} ${focusMode ? styles.focusMode : ""}`}>
        <header className={`${styles.topBar} ${focusMode ? styles.focusMode : ""}`}>
          <button className={styles.mobileMenuBtn} onClick={() => setShowSidebar(true)} title="打开侧栏" aria-label="打开侧栏">
            <svg width="20" height="20" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M4 4l4 3-4 3" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
          <div className={styles.topBarCenter}>
            {activeSession && (
              <div className={styles.sessionToolbar} data-testid="session-toolbar">
                {renamingTitle ? (
                  <input
                    className={styles.sessionTitleInput}
                    value={titleDraft}
                    onChange={(e) => setTitleDraft(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") commitTitleRename();
                      if (e.key === "Escape") { setRenamingTitle(false); setTitleDraft(activeSession.name); }
                    }}
                    onBlur={commitTitleRename}
                    autoFocus
                    data-testid="session-title-input"
                  />
                ) : (
                  <button
                    className={styles.sessionTitle}
                    onClick={() => { setTitleDraft(activeSession.name); setRenamingTitle(true); }}
                    title="点击重命名会话"
                    data-testid="session-title"
                  >
                    {activeSession.name || "未命名会话"}
                  </button>
                )}
                <span className={styles.sessionProject} title={activeSession.wire_path || "未绑定项目"}>
                  {activeSession.wire_path ? activeSession.wire_path.split(/[\\/]/).filter(Boolean).pop() : "本地"}
                </span>
                {gitStatus && (
                  <span className={`${styles.branchChip} ${gitStatus.dirty ? styles.branchChipDirty : ""}`} title={gitStatus.dirty ? "有未提交改动" : "工作区干净"}>
                    {gitStatus.branch}
                  </span>
                )}
              </div>
            )}
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
            <select
              value={settings?.permissionMode || "auto"}
              onChange={(e) => setSettings({ ...settings, permissionMode: e.target.value as any })}
              className={styles.modeSelect}
              disabled={agentBusy}
              data-testid="permission-mode-select"
              title="权限模式: 控制每轮编辑的审批方式"
            >
              <option value="auto">Auto</option>
              <option value="accept">AcceptEdits</option>
              <option value="manual">Manual</option>
              <option value="plan">Plan</option>
            </select>
            <button
              className={styles.topBarBtn}
              onClick={openTimeline}
              disabled={!activeSession}
              data-testid="timeline-open"
              title="检查点时间线 (回退到历史状态)"
            >
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                <circle cx="8" cy="8" r="5.5"/>
                <path d="M8 5v3l2 1.5" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
              时间线
            </button>
            <button
              className={`${styles.topBarBtn} ${taskPaneOpen ? styles.topBarBtnActive : ""}`}
              onClick={() => setTaskPaneOpen((v) => !v)}
              data-testid="task-pane-toggle"
              title="任务面板：实时查看工具调用步骤与耗时"
            >
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                <rect x="2" y="3" width="12" height="10" rx="1.5"/>
                <path d="M5 8h6M5 11h3" strokeLinecap="round"/>
              </svg>
              任务
            </button>
            <div className={styles.viewsMenuWrap}>
              <button
                type="button"
                data-testid="views-menu-btn"
                className={`${styles.viewsMenuBtn} ${viewsMenuOpen ? styles.viewsMenuActive : ""}`}
                onClick={() => setViewsMenuOpen((v) => !v)}
                title="视图面板"
                aria-expanded={viewsMenuOpen}
                aria-haspopup="menu"
              >
                视图 ▾
              </button>
              {viewsMenuOpen && (
                <div className={styles.viewsMenu} role="menu">
                  <button
                    type="button"
                    data-testid="views-menu-terminal"
                    role="menuitem"
                    className={`${styles.viewsMenuItem} ${terminalOpen ? styles.viewsMenuItemActive : ""}`}
                    onClick={() => { setTerminalOpen((v) => !v); setViewsMenuOpen(false); }}
                  >
                    终端 {terminalOpen ? "✓" : ""}
                  </button>
                  <button
                    type="button"
                    data-testid="views-menu-diff"
                    role="menuitem"
                    className={`${styles.viewsMenuItem} ${diffOpen ? styles.viewsMenuItemActive : ""}`}
                    onClick={() => { setDiffOpen((v) => !v); setViewsMenuOpen(false); }}
                  >
                    Diff 查看器 {diffOpen ? "✓" : ""}
                  </button>
                  <button
                    type="button"
                    data-testid="views-menu-preview"
                    role="menuitem"
                    className={`${styles.viewsMenuItem} ${previewOpen ? styles.viewsMenuItemActive : ""}`}
                    onClick={() => { setPreviewOpen((v) => !v); setViewsMenuOpen(false); }}
                  >
                    App 预览 {previewOpen ? "✓" : ""}
                  </button>
                  <button
                    type="button"
                    data-testid="views-menu-capability"
                    role="menuitem"
                    className={`${styles.viewsMenuItem} ${capabilityOpen ? styles.viewsMenuItemActive : ""}`}
                    onClick={() => { setCapabilityOpen((v) => !v); setViewsMenuOpen(false); }}
                  >
                    能力网健康 {capabilityOpen ? "✓" : ""}
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
              aria-label="切换主题"
            >
              <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <circle cx="7" cy="7" r="2.6" />
                <path d="M7 1v2M7 11v2M1 7h2M11 7h2M2.9 2.9l1.4 1.4M9.7 9.7l1.4 1.4M2.9 11.1l1.4-1.4M9.7 4.3l1.4-1.4" strokeLinecap="round" />
              </svg>
            </button>
            <button
              className={styles.settingsBtn}
              onClick={() => navigate("/settings")}
              title="设置"
              aria-label="打开设置"
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
              aria-label={focusMode ? "退出专注" : "专注模式"}
            >
              <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <circle cx="7" cy="7" r="3" />
                <path d="M7 1v2M7 11v2M1 7h2M11 7h2" strokeLinecap="round" />
              </svg>
            </button>
            <button
              className={`${styles.settingsBtn} ${showUsage ? styles.settingsActive : ""}`}
              title={`上下文用量 ${usagePct}%（点击查看成本明细）`}
              aria-label={`上下文用量 ${usagePct}%`}
              onClick={() => setShowUsage((v) => !v)}
              data-testid="usage-toggle"
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
            {usage >= 0.85 && (
              <div className={styles.compactBanner}>
                <span>上下文接近上限{usage >= 0.95 ? "，继续对话可能被截断" : ""}。建议压缩后继续。</span>
                <button
                  className={styles.compactBtn}
                  data-testid="usage-compact"
                  disabled={agentBusy || compacting}
                  onClick={handleCompact}
                >
                  {compacting ? "压缩中…" : "压缩上下文"}
                </button>
              </div>
            )}
          </div>
        )}

        <div className={styles.chatArea}>
          <ChatView
            messages={neocodexMessages}
            streamingContent={neocodexStreaming?.content}
            streamingRole={neocodexStreaming?.role}
            agentBusy={agentBusy}
            viewMode={viewMode}
            contextUsage={health?.context_usage || 0}
            recentSessions={neocodexSessions as any[]}
            onRecentSessionSelect={(id) => {
              const s = neocodexSessions.find((x: any) => x.id === id);
              if (s) handleSessionSelect(s);
            }}
            onSend={handleSend}
            onStop={handleStop}
            onEdit={handleEditMessage}
            onDelete={handleDeleteMessage}
            onRegenerate={handleRegenerateMessage}
          />
          {sideChatOpen && (
            <div className={styles.sideChat}>
              <div className={styles.sideChatHeader}>
                <span>侧聊</span>
                <button className={styles.sideChatClose} onClick={() => setSideChatOpen(false)} title="关闭侧聊" aria-label="关闭侧聊">✕</button>
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
        {taskPaneOpen && (
          <div className={styles.pane}>
            <TaskPane steps={taskSteps} startedAt={taskStartedAt} />
          </div>
        )}
        {previewOpen && (
          <div className={styles.pane}>
            <PreviewPane />
          </div>
        )}
        {capabilityOpen && (
          <div className={styles.pane}>
            <CapabilityHealthPane data={health} />
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

      <CommandPalette
        open={paletteOpen}
        items={paletteMode === "file" ? fileItems : paletteItems}
        onClose={() => setPaletteOpen(false)}
        placeholder={paletteMode === "file" ? "搜索文件… (⌘P)" : "搜索会话或执行命令…"}
      />
      <ShortcutHelp open={shortcutHelpOpen} onClose={() => setShortcutHelpOpen(false)} />
      {pendingDeleteSession && (
        <div className={styles.confirmOverlay} role="dialog" aria-modal="true" aria-label="删除会话确认">
          <div className={styles.confirmDialog}>
            <h3>删除会话</h3>
            <p>
              确定要永久删除「{pendingDeleteName}」吗？此操作不可撤销，会话及全部消息将被清除。
            </p>
            <div className={styles.confirmActions}>
              <button
                className={styles.confirmCancel}
                data-testid="confirm-delete-cancel"
                onClick={() => setPendingDeleteSession(null)}
              >
                取消
              </button>
              <button
                className={styles.confirmDeleteBtn}
                data-testid="confirm-delete-confirm"
                onClick={() => {
                  const id = pendingDeleteSession;
                  setPendingDeleteSession(null);
                  handleSessionDelete(id);
                }}
              >
                永久删除
              </button>
            </div>
          </div>
        </div>
      )}
      {timelineOpen && (
        <div className={styles.confirmOverlay} role="dialog" aria-modal="true" aria-label="检查点时间线" data-testid="timeline-panel" onClick={() => setTimelineOpen(false)}>
          <div className={styles.timelineDialog} onClick={(e) => e.stopPropagation()}>
            <h3>检查点时间线</h3>
            <p className={styles.timelineSub}>
              {activeSession?.name || "此会话"} 的历史状态，可回退到任意检查点。回退会用该点消息重建会话。
            </p>
            {checkpointsLoading ? (
              <div className={styles.timelineEmpty}>加载中…</div>
            ) : checkpoints.length === 0 ? (
              <div className={styles.timelineEmpty}>暂无检查点。每次发送消息会自动创建。</div>
            ) : (
              <ul className={styles.timelineList}>
                {checkpoints.map((cp, i) => (
                  <li key={cp.id} className={styles.timelineItem}>
                    <div className={styles.timelineInfo}>
                      <span className={styles.timelineTime}>
                        {i === 0 ? "最新" : ""} {new Date(cp.created_at).toLocaleString()}
                      </span>
                      <span className={styles.timelineCount}>{cp.message_count} 条消息</span>
                    </div>
                    <button
                      className={styles.confirmDeleteBtn}
                      data-testid={`timeline-restore-${cp.id}`}
                      onClick={() => setPendingCheckpointRestore(cp.id)}
                    >
                      回退到此
                    </button>
                  </li>
                ))}
              </ul>
            )}
            <div className={styles.confirmActions}>
              <button
                className={styles.confirmCancel}
                data-testid="timeline-close"
                onClick={() => setTimelineOpen(false)}
              >
                关闭
              </button>
            </div>
          </div>
        </div>
      )}
      {pendingCheckpointRestore && (
        <div className={styles.confirmOverlay} role="dialog" aria-modal="true" aria-label="回退检查点确认" data-testid="timeline-restore-confirm">
          <div className={styles.confirmDialog}>
            <h3>回退到检查点</h3>
            <p>确定要回退吗？当前会话内容将被该检查点的消息替换。建议先确认已暂存需要保留的改动。</p>
            <div className={styles.confirmActions}>
              <button
                className={styles.confirmCancel}
                data-testid="timeline-restore-cancel"
                onClick={() => setPendingCheckpointRestore(null)}
              >
                取消
              </button>
              <button
                className={styles.confirmDeleteBtn}
                data-testid="timeline-restore-confirm-btn"
                onClick={() => {
                  const id = pendingCheckpointRestore;
                  setPendingCheckpointRestore(null);
                  handleCheckpointRestore(id);
                }}
              >
                确认回退
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

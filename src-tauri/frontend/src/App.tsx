import React, { Suspense, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Outlet, useNavigate, useLocation } from "react-router-dom";
import { useStore } from "./stores";
import NeoTrixSidebar from "./components/NeoTrixSidebar";
import UserPopover from "./components/UserPopover";
import StatusBar from "./components/StatusBar";
import PetBar from "./components/PetBar";
import RightPanel from "./components/RightPanel";
import CommandPalette from "./components/CommandPalette";
import ErrorBoundary from "./components/ErrorBoundary";
import NotificationToast from "./components/NotificationToast";
import "./components/CommandPalette.css";
import TopBar from "./components/TopBar";
import type { Attachment, ContextMenuItem, Session } from "./types";
import * as api from "./lib/api";
import { getCurrent } from "@tauri-apps/plugin-deep-link";
import { check } from "@tauri-apps/plugin-updater";

const Onboarding = React.lazy(() => import("./components/Onboarding"));
const SearchOverlay = React.lazy(() => import("./components/SearchOverlay"));
const ShortcutsPanel = React.lazy(() => import("./components/ShortcutsPanel"));
// ProxyConfigPanel no longer rendered as overlay; proxy is now a route /proxy
const PermissionDialog = React.lazy(() => import("./components/PermissionDialog"));
const ContextMenu = React.lazy(() => import("./components/ContextMenu"));
const DiffViewer = React.lazy(() => import("./components/DiffViewer"));

function Lazy({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<div className="panel-loading" />}>{children}</Suspense>;
}

const App: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const locationRef = useRef(location);
  locationRef.current = location;

  const sessions = useStore((s) => s.sessions);
  const activeSessionIndex = useStore((s) => s.activeSessionIndex);
  const statusText = useStore((s) => s.statusText);
  const agentBusy = useStore((s) => s.agentBusy);
  const pendingPermission = useStore((s) => s.pendingPermission);
  const settings = useStore((s) => s.settings);
  const streamingContent = useStore((s) => s.streamingContent);
  const showOnboarding = useStore((s) => s.showOnboarding);
  const showShortcuts = useStore((s) => s.showShortcuts);
  const showSearch = useStore((s) => s.showSearch);
  const showCommandPalette = useStore((s) => s.showCommandPalette);
  const setShowSearch = useStore((s) => s.setShowSearch);
  const setShowCommandPalette = useStore((s) => s.setShowCommandPalette);
  const updateAvailable = useStore((s) => s.updateAvailable);
  const updateStatus = useStore((s) => s.updateStatus);
  const setUpdateStatus = useStore((s) => s.setUpdateStatus);
  const setUpdateProgress = useStore((s) => s.setUpdateProgress);
  const updateProgress = useStore((s) => s.updateProgress);
  const addNotification = useStore((s) => s.addNotification);

  const theme = settings.theme;

  const pushMessage = useStore((s) => s.pushMessage);
  const addSession = useStore((s) => s.addSession);
  const setActiveSessionIndex = useStore((s) => s.setActiveSessionIndex);
  const setStatusText = useStore((s) => s.setStatusText);
  const setAgentBusy = useStore((s) => s.setAgentBusy);
  const setPendingPermission = useStore((s) => s.setPendingPermission);
  const setShowShortcuts = useStore((s) => s.setShowShortcuts);
  const setSettings = useStore((s) => s.setSettings);
  const setStreamingContent = useStore((s) => s.setStreamingContent);
  const appendStreamingContent = useStore((s) => s.appendStreamingContent);
  const commitStreamingContent = useStore((s) => s.commitStreamingContent);
  const clearStreamingContent = useStore((s) => s.clearStreamingContent);
  const openEditor = useStore((s) => s.openEditor);
  const closeEditor = useStore((s) => s.closeEditor);
  const proxyStatus = useStore((s) => s.proxyStatus);
  const setProxyStatus = useStore((s) => s.setProxyStatus);
  const sidebarCollapsed = useStore((s) => s.sidebarCollapsed);
  const setSidebarCollapsed = useStore((s) => s.setSidebarCollapsed);
  const pendingDiff = useStore((s) => s.pendingDiff);
  const setPendingDiff = useStore((s) => s.setPendingDiff);

  const [input, setInput] = React.useState("");
  const [multiLine, setMultiLine] = React.useState(false);
  const abortRef = useRef<AbortController | null>(null);
  const terminalSessionId = useRef(`term-${Date.now()}`);
  const [terminalStatus, setTerminalStatus] = React.useState("");
  const [contextMenu, setContextMenu] = React.useState<{ x: number; y: number; items: ContextMenuItem[] } | null>(null);

  // ── Poll permissions ──
  useEffect(() => {
    const timer = setInterval(async () => {
      try {
        const perms = await api.getPendingPermissions();
        if (perms.length > 0 && !useStore.getState().pendingPermission) {
          setPendingPermission(perms[0]);
        }
      } catch { }
    }, 3000);
    return () => clearInterval(timer);
  }, [setPendingPermission]);

  // ── Apply theme ──
  useEffect(() => {
    applyTheme(settings.theme);
  }, [settings.theme]);

  // ── Tauri event listeners ──
  useEffect(() => {
    const unlistenToken = listen<{ token: string; full: string; error?: string }>("streaming-token", (event) => {
      const store = useStore.getState();
      if (event.payload.error) {
        store.pushMessage("error", event.payload.error);
        store.clearStreamingContent();
        store.setAgentBusy(false);
        store.setStatusText("Ready");
        return;
      }
      store.appendStreamingContent(event.payload.token);
    });
    const unlistenDone = listen<{ full: string }>("streaming-done", (event) => {
      const store = useStore.getState();
      store.commitStreamingContent("assistant", "markdown");
      store.setAgentBusy(false);
      store.setStatusText("Ready");
      api.distillMessage(event.payload.full).catch(() => {});
    });
    const unlistenTask = listen<{ title: string; body: string }>("task-complete", (event) => {
      useStore.getState().addNotification({ type: "info", message: `Task complete: ${event.payload.title}`, duration: 5000 });
    });
    return () => {
      unlistenToken.then((fn) => fn());
      unlistenDone.then((fn) => fn());
      unlistenTask.then((fn) => fn());
    };
  }, []);

  // ── Deep link ──
  useEffect(() => {
    getCurrent().then((urls) => {
      if (urls && urls.length > 0) handleDeepLink(urls.join(","));
    });
  }, []);

  // ── Settings/proxy/sync listeners ──
  useEffect(() => {
    const unlistenSettings = listen("open-settings", () => navigate("/settings"));
    const unlistenProxy = listen<string>("proxy-mode-change", (event) => {
      useStore.getState().setStatusText(`Proxy: ${event.payload}`);
    });
    const unlistenSync = listen<{ status: string; files_synced: number; duration_ms: number; timestamp: string }>("sync-complete", (event) => {
      if (event.payload.status === "ok") {
        addNotification({ type: "success", message: `Sync: ${event.payload.files_synced} files in ${event.payload.duration_ms}ms`, duration: 5000 });
      } else {
        addNotification({ type: "error", message: `Sync failed`, duration: 5000 });
      }
    });
    const unlistenProxyOpen = listen("open-proxy-status", () => navigate("/proxy"));
    return () => {
      unlistenSettings.then((fn) => fn());
      unlistenProxy.then((fn) => fn());
      unlistenSync.then((fn) => fn());
      unlistenProxyOpen.then((fn) => fn());
    };
  }, [navigate]);

  // ── Update check ──
  useEffect(() => {
    const checkUpdate = async () => {
      try {
        const update = await check();
        if (update?.available) {
          setUpdateStatus(true, `v${update.version}`);
          pushMessage("system", `Update available: v${update.version}`);
          addNotification({ type: "info", message: `Update v${update.version} available`, duration: 10000 });
        }
      } catch {}
    };
    checkUpdate();
  }, []);

  // ── Window state persistence ──
  useEffect(() => {
    let unload: Array<() => void> = [];
    const initWindowState = async () => {
      try {
        const mod = await import("@tauri-apps/api/window");
        const win = (mod as any).getCurrentWindow();
        const saved = (() => { try { return JSON.parse(localStorage.getItem("neotrix_window_state") || "null"); } catch { return null; } })();
        if (saved) {
          try {
            const Pos = (mod as any).PhysicalPosition;
            const Size = (mod as any).PhysicalSize;
            await win.setPosition(new Pos(saved.x, saved.y));
            await win.setSize(new Size(saved.w, saved.h));
          } catch {}
        }
        const saveState = async () => {
          try {
            const pos = await win.getPosition();
            const size = await win.getSize();
            localStorage.setItem("neotrix_window_state", JSON.stringify({ x: pos.x, y: pos.y, w: size.width, h: size.height }));
          } catch {}
        };
        const un1 = await win.onResized(saveState);
        const un2 = await win.onMoved(saveState);
        unload = [un1, un2];
      } catch {}
    };
    initWindowState();
    return () => unload.forEach((fn) => fn());
  }, []);

  function handleDeepLink(url: string) {
    const decoded = decodeURIComponent(url);
    pushMessage("system", `Deep link: ${decoded}`);
  }

  // ── Keyboard shortcuts ──
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      const st = useStore.getState();

      if (mod && e.key === ",") {
        e.preventDefault(); navigate("/settings");
      } else if (mod && e.key === "b") {
        e.preventDefault(); st.setShowFileTree(!st.showFileTree);
      } else if (mod && e.key === "e") {
        e.preventDefault(); st.setEvolutionVisible(!st.evolutionVisible);
      } else if (mod && e.key === "k") {
        e.preventDefault(); setShowCommandPalette(true);
      } else if (mod && e.key === "f") {
        e.preventDefault(); st.setShowSearch(true);
      } else if (mod && e.key === "n") {
        e.preventDefault(); st.addSession();
      } else if (mod && e.shiftKey && e.key === "B") {
        e.preventDefault(); setSidebarCollapsed(!st.sidebarCollapsed);
      } else if (e.key === "Escape") {
        if (st.showCommandPalette) st.setShowCommandPalette(false);
        else if (st.showSearch) st.setShowSearch(false);
        else if (st.showShortcuts) st.setShowShortcuts(false);
        else if (st.editorState.open) closeEditor();
        else if (locationRef.current.pathname !== "/") navigate("/");
      } else if (mod && e.shiftKey && e.key === "[") {
        e.preventDefault();
        if (st.activeSessionIndex > 0) st.setActiveSessionIndex(st.activeSessionIndex - 1);
      } else if (mod && e.shiftKey && e.key === "]") {
        e.preventDefault();
        if (st.activeSessionIndex < st.sessions.length - 1) st.setActiveSessionIndex(st.activeSessionIndex + 1);
      } else if (mod && e.key === "Tab") {
        e.preventDefault();
        const next = e.shiftKey
          ? (st.activeSessionIndex > 0 ? st.activeSessionIndex - 1 : st.sessions.length - 1)
          : (st.activeSessionIndex < st.sessions.length - 1 ? st.activeSessionIndex + 1 : 0);
        st.setActiveSessionIndex(next);
      } else if (mod && e.key >= "1" && e.key <= "8") {
        e.preventDefault();
        const idx = parseInt(e.key) - 1;
        if (idx < st.sessions.length) st.setActiveSessionIndex(idx);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate, closeEditor]);

  // ── Context menu ──
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      setContextMenu(null);
      const target = e.target as HTMLElement;
      const fileNode = target.closest("[data-file-path]") as HTMLElement | null;
      const sessionTab = target.closest("[data-session-index]") as HTMLElement | null;

      if (fileNode) {
        e.preventDefault();
        const path = fileNode.dataset.filePath || "";
        setContextMenu({
          x: e.clientX, y: e.clientY,
          items: [
            { label: "Open in Editor", icon: "✎", action: () => openEditor(path) },
            { label: "Copy Path", icon: "📋", action: () => navigator.clipboard.writeText(path).catch(() => {}) },
            { label: "Reveal in Finder", icon: "📁", action: () => invoke("cmd_project_open", { path: path.substring(0, path.lastIndexOf("/")) || "." }).catch(() => {}) },
          ],
        });
      } else if (sessionTab) {
        e.preventDefault();
        const idx = parseInt(sessionTab.dataset.sessionIndex || "0");
        setContextMenu({
          x: e.clientX, y: e.clientY,
          items: [
            { label: "Close", icon: "✕", action: () => { if (sessions.length > 1) useStore.getState().removeSession(idx); } },
            { label: "Fork", icon: "⑂", action: () => {
              const original = sessions[idx];
              const dup: Session = { id: `s-${Date.now()}`, name: `${original.name} (copy)`, messages: original.messages.map((m) => ({ ...m })) };
              const next = [...sessions]; next.splice(idx + 1, 0, dup);
              useStore.getState().setSessions(next);
              useStore.getState().setActiveSessionIndex(idx + 1);
            }},
          ],
        });
      }
    };
    document.addEventListener("contextmenu", handler);
    return () => document.removeEventListener("contextmenu", handler);
  }, [sessions, openEditor]);

  // ── Submit ──
  const handleSubmit = useCallback(async (text: string, attachments?: Attachment[]) => {
    if (!text.trim()) return;
    if (text.startsWith("/")) { await handleCommand(text); return; }

    setAgentBusy(true);
    setStatusText("思考中...");
    pushMessage("user", text, "markdown", attachments);
    clearStreamingContent();

    const abort = new AbortController();
    abortRef.current = abort;

    try {
      const result = await api.agentReason(text);
      if (abort.signal.aborted) return;
      if (result.success && result.output) pushMessage("assistant", result.output);
      else pushMessage("error", result.output || "Agent returned empty response");
    } catch (e) {
      if (abort.signal.aborted) return;
      pushMessage("error", `Request failed: ${e}`);
    }

    setAgentBusy(false);
    setStatusText("Ready");
    abortRef.current = null;
    api.distillMessage(text).catch(() => {});
  }, [pushMessage, setAgentBusy, setStatusText, clearStreamingContent]);

  const handleCommand = async (text: string) => {
    const cmd = text.slice(1).trim().toLowerCase();
    setAgentBusy(true);
    setStatusText(`Executing: ${cmd}`);
    try {
      if (cmd === "stats" || cmd === "brain") {
        const stats = await api.getBrainStatsV2();
        pushMessage("assistant", `## Brain Stats\n\n- Iterations: ${stats.iteration}\n- Absorbed: ${stats.absorb_count}\n- Capability Sum: ${stats.capability_sum.toFixed(3)}\n- Memories: ${stats.memory_count}\n- Engine Active: ${stats.engine_active}\n- Capability Vector: [${stats.capability_vector.map(v => v.toFixed(3)).slice(0, 6).join(", ")}…]`);
      } else if (cmd.startsWith("diff")) {
        const diffBlocks = await api.getDiffUnstaged();
        useStore.getState().setPendingDiff({ blocks: diffBlocks, filename: "Working changes" });
        pushMessage("system", `Diff: ${diffBlocks.length} blocks`);
      } else if (cmd === "help") {
        pushMessage("system", "Commands: /stats, /brain, /diff, /help");
      } else {
        pushMessage("system", `Unknown command: ${cmd}. Type /help`);
      }
    } catch (e) {
      pushMessage("error", `Command failed: ${e}`);
    }
    setAgentBusy(false);
    setStatusText("Ready");
  };

  const handlePermissionApprove = useCallback(async (id: string) => {
    try { await api.respondPermission(id, true); pushMessage("system", `Permission: ${id} approved`); } catch (e) { pushMessage("error", `Approve failed: ${e}`); }
    setPendingPermission(null);
  }, [pushMessage, setPendingPermission]);

  const handlePermissionDeny = useCallback(async (id: string) => {
    try { await api.respondPermission(id, false); pushMessage("system", `Permission: ${id} denied`); } catch (e) { pushMessage("error", `Deny failed: ${e}`); }
    setPendingPermission(null);
  }, [pushMessage, setPendingPermission]);

  const handleToggleTheme = useCallback(() => {
    const order: Array<"light" | "dark" | "system"> = ["light", "dark", "system"];
    const idx = order.indexOf(useStore.getState().settings.theme);
    setSettings({ ...useStore.getState().settings, theme: order[(idx + 1) % order.length] });
  }, [setSettings]);

  function applyTheme(theme: "light" | "dark" | "system") {
    const isDark = theme === "dark" || (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.setAttribute("data-theme", isDark ? "dark" : "light");
  }

  const showTerminal = location.pathname === "/terminal";

  return (
    <div className="app-container">
      {showOnboarding && <Lazy><Onboarding /></Lazy>}

      {showSearch && (
        <ErrorBoundary><Lazy><SearchOverlay /></Lazy></ErrorBoundary>
      )}

      <CommandPalette />

      {showShortcuts && (
        <ErrorBoundary><Lazy><ShortcutsPanel onClose={() => setShowShortcuts(false)} /></Lazy></ErrorBoundary>
      )}

      {pendingPermission && (
        <ErrorBoundary><Lazy><PermissionDialog request={pendingPermission} onApprove={handlePermissionApprove} onDeny={handlePermissionDeny} /></Lazy></ErrorBoundary>
      )}

      {/* ── Middle: 2-zone layout (sidebar + main) ── */}
      <div className="app-body">
        {/* Left: NeoTrix Sidebar (nav + recents + user) */}
        <NeoTrixSidebar />

        {/* Center: Main panel */}
        <ErrorBoundary fallback={
          <div className="main-panel-error">
            <h3>Panel render error</h3>
            <button className="nt-btn-primary" onClick={() => window.location.reload()}>Restore</button>
          </div>
        }>
          <div className="main-panel">
            {pendingDiff && (
              <Lazy><DiffViewer
                diffBlocks={pendingDiff.blocks}
                filename={pendingDiff.filename}
                onApply={() => { setPendingDiff(null); pushMessage("system", "Diff applied"); }}
                onReject={() => { setPendingDiff(null); pushMessage("system", "Diff rejected"); }}
              /></Lazy>
            )}

            {/* Top Bar — only on main page */}
            {!showTerminal && (
              <TopBar
                sessionName={sessions[activeSessionIndex]?.name || "Conversation"}
                providerConfig={useStore.getState().providerConfig}
                onRename={(name) => useStore.getState().renameSession(activeSessionIndex, name)}
                onExport={() => useStore.getState().exportSession(sessions[activeSessionIndex]?.id || "")}
                onImport={() => useStore.getState().importSession()}
                onClear={() => pushMessage("system", "Clear conversation")}
                onDelete={() => { if (sessions.length > 1) useStore.getState().removeSession(activeSessionIndex); }}
              />
            )}

            <Outlet context={{
              input, setInput, multiLine, setMultiLine,
              handleSubmit,
              terminalSessionId, terminalStatus, setTerminalStatus,
            } as import("./router").AppOutletContext} />

            <PetBar />

            <StatusBar
              text={statusText}
              agentBusy={agentBusy}
              sessionIndex={activeSessionIndex + 1}
              sessionCount={sessions.length}
              showTerminal={showTerminal}
              terminalStatus={terminalStatus}
              onOpenSettings={() => navigate("/settings")}
              onSelectProject={async () => {
                try {
                  await invoke("read_dir_recursive", { path: ".", max_depth: 1 });
                  useStore.getState().setProjectPath(".");
                  useStore.getState().setShowFileTree(true);
                  setStatusText("Project loaded");
                } catch (e) { setStatusText(`Failed: ${e}`); }
              }}
              onToggleTerminal={() => navigate(showTerminal ? "/" : "/terminal")}
              onToggleTheme={handleToggleTheme}
              onOpenProxy={() => navigate("/proxy")}
              proxyStatus={proxyStatus}
              theme={theme}
            />
          </div>
        </ErrorBoundary>

        {/* Right: Optional panel */}
        <RightPanel />
      </div>

      <UserPopover />

      {contextMenu && (
        <Lazy><ContextMenu x={contextMenu.x} y={contextMenu.y} items={contextMenu.items} onClose={() => setContextMenu(null)} /></Lazy>
      )}

      {updateAvailable && updateProgress > 0 && updateProgress < 100 && (
        <div className="update-progress-bar">
          <div className="update-progress-fill" style={{ width: `${updateProgress}%` }} />
          <span className="update-progress-text">{updateProgress}%</span>
        </div>
      )}

      <NotificationToast />
    </div>
  );
};

export default App;

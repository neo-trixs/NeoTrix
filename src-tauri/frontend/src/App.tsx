import React, { Suspense, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useStore } from "./store";
import SessionList from "./components/SessionList";
import TabBar from "./components/TabBar";
import ChatPanel from "./components/ChatPanel";
import InputPanel from "./components/InputPanel";
import StatusBar from "./components/StatusBar";
import ErrorBoundary from "./components/ErrorBoundary";
import NotificationToast from "./components/NotificationToast";
import type { DiffBlock, PermissionRequest, KnowledgeEntry, ProviderConfig, AppSettings, Message, Session, ContextMenuItem } from "./types";
import * as api from "./lib/api";
import { getCurrent } from "@tauri-apps/plugin-deep-link";
import { check } from "@tauri-apps/plugin-updater";

const Onboarding = React.lazy(() => import("./components/Onboarding"));
const SearchOverlay = React.lazy(() => import("./components/SearchOverlay"));
const ShortcutsPanel = React.lazy(() => import("./components/ShortcutsPanel"));
const Settings = React.lazy(() => import("./components/Settings"));
const ProxyPanel = React.lazy(() => import("./components/ProxyPanel"));
const PermissionDialog = React.lazy(() => import("./components/PermissionDialog"));
const FileTree = React.lazy(() => import("./components/FileTree"));
const DiffViewer = React.lazy(() => import("./components/DiffViewer"));
const Terminal = React.lazy(() => import("./components/Terminal"));
const EvolutionPanel = React.lazy(() => import("./components/EvolutionPanel"));
const SyncPanel = React.lazy(() => import("./components/SyncPanel"));
const AgentFlow = React.lazy(() => import("./components/AgentFlow"));
const AgentMaker = React.lazy(() => import("./components/AgentMaker"));
const VirtualOS = React.lazy(() => import("./components/VirtualOS"));
const SplitView = React.lazy(() => import("./components/SplitView"));
const CodeEditor = React.lazy(() => import("./components/CodeEditor"));
const ContextMenu = React.lazy(() => import("./components/ContextMenu"));

function Lazy({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<div className="panel-loading" />}>{children}</Suspense>;
}

const App: React.FC = () => {
  const sessions = useStore((s) => s.sessions);
  const activeSessionIndex = useStore((s) => s.activeSessionIndex);
  const statusText = useStore((s) => s.statusText);
  const agentBusy = useStore((s) => s.agentBusy);
  const showSettings = useStore((s) => s.showSettings);
  const projectPath = useStore((s) => s.projectPath);
  const showFileTree = useStore((s) => s.showFileTree);
  const pendingPermission = useStore((s) => s.pendingPermission);
  const providerConfig = useStore((s) => s.providerConfig);
  const knowledgeBase = useStore((s) => s.knowledgeBase);
  const settings = useStore((s) => s.settings);
  const streamingContent = useStore((s) => s.streamingContent);
  const streamingContentType = useStore((s) => s.streamingContentType);
  const showTerminal = useStore((s) => s.showTerminal);
  const showOnboarding = useStore((s) => s.showOnboarding);
  const showShortcuts = useStore((s) => s.showShortcuts);
  const showSearch = useStore((s) => s.showSearch);
  const setShowSearch = useStore((s) => s.setShowSearch);
  const splitViewActive = useStore((s) => s.splitViewActive);
  const virtualOSActive = useStore((s) => s.virtualOSActive);
  const agentFlowActive = useStore((s) => s.agentFlowActive);
  const evolutionVisible = useStore((s) => s.evolutionVisible);
  const syncVisible = useStore((s) => (s as any).syncVisible);
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
  const reorderSessions = useStore((s) => s.reorderSessions);
  const renameSession = useStore((s) => s.renameSession);
  const setStatusText = useStore((s) => s.setStatusText);
  const setAgentBusy = useStore((s) => s.setAgentBusy);
  const setShowSettings = useStore((s) => s.setShowSettings);
  const setProjectPath = useStore((s) => s.setProjectPath);
  const setShowFileTree = useStore((s) => s.setShowFileTree);
  const setPendingPermission = useStore((s) => s.setPendingPermission);
  const setKnowledgeBase = useStore((s) => s.setKnowledgeBase);
  const setProviderConfig = useStore((s) => s.setProviderConfig);
  const setSettings = useStore((s) => s.setSettings);
  const setStreamingContent = useStore((s) => s.setStreamingContent);
  const appendStreamingContent = useStore((s) => s.appendStreamingContent);
  const commitStreamingContent = useStore((s) => s.commitStreamingContent);
  const clearStreamingContent = useStore((s) => s.clearStreamingContent);
  const setShowTerminal = useStore((s) => s.setShowTerminal);
  const setShowShortcuts = useStore((s) => s.setShowShortcuts);
  const setSplitViewActive = useStore((s) => s.setSplitViewActive);
  const setVirtualOSActive = useStore((s) => s.setVirtualOSActive);
  const agentMakerActive = useStore((s) => s.agentMakerActive);
  const setAgentMakerActive = useStore((s) => s.setAgentMakerActive);
  const setAgentFlowActive = useStore((s) => s.setAgentFlowActive);
  const setEvolutionVisible = useStore((s) => s.setEvolutionVisible);
  const setSyncVisible = useStore((s) => (s as any).setSyncVisible);
  const editorState = useStore((s) => s.editorState);
  const openEditor = useStore((s) => s.openEditor);
  const closeEditor = useStore((s) => s.closeEditor);
  const proxyVisible = useStore((s) => s.proxyVisible);
  const proxyStatus = useStore((s) => s.proxyStatus);
  const setProxyVisible = useStore((s) => s.setProxyVisible);
  const setProxyStatus = useStore((s) => s.setProxyStatus);

  const activeMessages = sessions[activeSessionIndex]?.messages || [];

  const [input, setInput] = React.useState("");
  const [multiLine, setMultiLine] = React.useState(false);
  const [diffData, setDiffData] = React.useState<{ blocks: DiffBlock[]; filename?: string } | null>(null);
  const abortRef = useRef<AbortController | null>(null);
  const terminalSessionId = useRef(`term-${Date.now()}`);
  const [terminalStatus, setTerminalStatus] = React.useState("");
  const [contextMenu, setContextMenu] = React.useState<{ x: number; y: number; items: ContextMenuItem[] } | null>(null);

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

  useEffect(() => {
    applyTheme(settings.theme);
  }, [settings.theme]);

  useEffect(() => {
    const unlistenToken = listen<{ token: string; full: string; error?: string }>("streaming-token", (event) => {
      const store = useStore.getState();
      if (event.payload.error) {
        store.pushMessage("error", event.payload.error);
        store.clearStreamingContent();
        store.setAgentBusy(false);
        store.setStatusText("就绪");
        return;
      }
      store.appendStreamingContent(event.payload.token);
    });
    const unlistenDone = listen<{ full: string }>("streaming-done", (event) => {
      const store = useStore.getState();
      store.commitStreamingContent("assistant", "markdown");
      store.setAgentBusy(false);
      store.setStatusText("就绪");
      api.distillMessage(event.payload.full).catch(() => {});
    });
    const unlistenTask = listen<{ title: string; body: string }>("task-complete", (event) => {
      console.log(`Task complete: ${event.payload.title}`);
    });
    return () => {
      unlistenToken.then((fn) => fn());
      unlistenDone.then((fn) => fn());
      unlistenTask.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    getCurrent().then((urls) => {
      if (urls && urls.length > 0) handleDeepLink(urls.join(","));
    });
  }, []);

  useEffect(() => {
    const unlistenSettings = listen("open-settings", () => {
      useStore.getState().setShowSettings(true);
    });
    const unlistenProxy = listen<string>("proxy-mode-change", (event) => {
      console.log("Proxy mode changed:", event.payload);
      useStore.getState().setStatusText(`代理模式: ${event.payload}`);
    });
    const unlistenSync = listen<{ status: string; files_synced: number; duration_ms: number; timestamp: string }>("sync-complete", (event) => {
      const store = useStore.getState();
      if (event.payload.status === "ok") {
        store.addNotification({ type: "success", message: `Sync: ${event.payload.files_synced} files in ${event.payload.duration_ms}ms`, duration: 5000 });
      } else {
        store.addNotification({ type: "error", message: `Sync failed`, duration: 5000 });
      }
    });
    const unlistenProxyOpen = listen("open-proxy-status", () => {
      useStore.getState().setProxyVisible(true);
    });
    return () => {
      unlistenSettings.then((fn) => fn());
      unlistenProxy.then((fn) => fn());
      unlistenSync.then((fn) => fn());
      unlistenProxyOpen.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    const checkUpdate = async () => {
      try {
        const update = await check();
        if (update?.available) {
          console.log(`Update available: ${update.version}`);
          const store = useStore.getState();
          store.setUpdateStatus(true, `v${update.version}`);
          store.pushMessage("system", `Update available: v${update.version}`);
          store.addNotification({ type: "info", message: `Update available: v${update.version}`, duration: 10000 });
        }
      } catch (e) {
        console.log("Update check failed:", e);
      }
    };
    checkUpdate();
  }, []);

  const handleDownloadUpdate = useCallback(async () => {
    try {
      const update = await check();
      if (!update?.available) return;
      setUpdateStatus(true, "Downloading...");
      setUpdateProgress(0);
      await update.download((event) => {
        if (event.event === "progress") {
          const pct = Math.round((event.data.progress / event.data.total) * 100);
          setUpdateProgress(pct);
        }
      });
      setUpdateProgress(100);
      setUpdateStatus(true, "Downloaded — restart to install");
      addNotification({ type: "success", message: `Update v${update.version} downloaded. Restart to install.`, duration: 0 });
    } catch (e) {
      setUpdateStatus(true, "Download failed");
      addNotification({ type: "error", message: `Update download failed: ${e}`, duration: 5000 });
    }
  }, [setUpdateStatus, setUpdateProgress, addNotification]);

  const handleInstallUpdate = useCallback(async () => {
    try {
      const update = await check();
      await update?.install();
    } catch (e) {
      addNotification({ type: "error", message: `Install failed: ${e}`, duration: 5000 });
    }
  }, [addNotification]);

  useEffect(() => {
    let unload: Array<() => void> = [];
    const initWindowState = async () => {
      try {
        const mod = await import("@tauri-apps/api/window");
        const win = (mod as any).getCurrentWindow();
        const saved = (() => { try { const r = localStorage.getItem("neotrix_window_state"); return r ? JSON.parse(r) : null; } catch { return null; } })();
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
    return () => { unload.forEach((fn) => fn()); };
  }, []);

  function handleDeepLink(url: string) {
    const decoded = decodeURIComponent(url);
    pushMessage("system", `Deep link received: ${decoded}`);
    const store = useStore.getState();
    if (decoded.startsWith("neotrix://")) {
      const payload = decoded.slice("neotrix://".length);
      if (payload.startsWith("session/")) {
        store.setStatusText(`Deep link: session ${payload.slice(8)}`);
      }
    }
  }

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      const st = useStore.getState();

      if (mod && e.key === "n") {
        e.preventDefault();
        st.addSession();
      } else if (mod && e.key === "w") {
        if (st.sessions.length > 1) {
          e.preventDefault();
          st.removeSession(st.activeSessionIndex);
        }
      } else if (mod && e.key === ",") {
        e.preventDefault();
        st.setShowSettings(!st.showSettings);
      } else if (mod && e.key === "b") {
        e.preventDefault();
        st.setShowFileTree(!st.showFileTree);
      } else if (mod && !e.shiftKey && e.key === "t") {
        e.preventDefault();
        st.addSession();
      } else if (mod && e.key === "e") {
        e.preventDefault();
        st.setEvolutionVisible(!st.evolutionVisible);
      } else if (mod && e.key === "f") {
        e.preventDefault();
        st.setAgentFlowActive(!st.agentFlowActive);
      } else if (e.key === "Escape") {
        if (st.showSearch) st.setShowSearch(false);
        else if (st.showShortcuts) st.setShowShortcuts(false);
        else if (st.showSettings) st.setShowSettings(false);
        else if (st.showFileTree) st.setShowFileTree(false);
        else if (st.showTerminal) st.setShowTerminal(false);
        else if (st.editorState.open) st.closeEditor();
      } else if (mod && e.shiftKey && e.key === "I") {
        e.preventDefault();
        if (!st.editorState.open && st.projectPath) {
          st.openEditor(st.projectPath + "/src/main.rs").catch(() => {});
        }
      } else if (mod && e.shiftKey && e.key === "[") {
        e.preventDefault();
        if (st.activeSessionIndex > 0) st.setActiveSessionIndex(st.activeSessionIndex - 1);
      } else if (mod && e.shiftKey && e.key === "]") {
        e.preventDefault();
        if (st.activeSessionIndex < st.sessions.length - 1) st.setActiveSessionIndex(st.activeSessionIndex + 1);
      } else if (mod && e.key === "Tab") {
        e.preventDefault();
        if (e.shiftKey) {
          const prev = st.activeSessionIndex > 0 ? st.activeSessionIndex - 1 : st.sessions.length - 1;
          st.setActiveSessionIndex(prev);
        } else {
          const next = st.activeSessionIndex < st.sessions.length - 1 ? st.activeSessionIndex + 1 : 0;
          st.setActiveSessionIndex(next);
        }
      } else if (mod && e.key >= "1" && e.key <= "8") {
        e.preventDefault();
        const idx = parseInt(e.key) - 1;
        if (idx < st.sessions.length) st.setActiveSessionIndex(idx);
      } else if (mod && e.key === "r") {
        e.preventDefault();
        st.setShowSearch(!st.showSearch);
      } else if (mod && e.key === "/") {
        e.preventDefault();
        st.setShowShortcuts(!st.showShortcuts);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      setContextMenu(null);
      const target = e.target as HTMLElement;
      const fileNode = target.closest("[data-file-path]") as HTMLElement | null;
      const chatMessage = target.closest("[data-message-text]") as HTMLElement | null;
      const sessionTab = target.closest("[data-session-index]") as HTMLElement | null;

      if (fileNode) {
        e.preventDefault();
        const path = fileNode.dataset.filePath || "";
        const name = path.split("/").pop() || path;
        setContextMenu({
          x: e.clientX, y: e.clientY,
          items: [
            { label: "Open in Editor", icon: "✎", action: () => openEditor(path) },
            { label: "Copy Path", icon: "📋", action: () => navigator.clipboard.writeText(path).catch(() => {}) },
            { label: "Reveal in Finder", icon: "📁", action: () => { invoke("cmd_project_open", { path: path.substring(0, path.lastIndexOf("/")) || "." }).catch(() => {}); } },
          ],
        });
      } else if (sessionTab) {
        e.preventDefault();
        const idx = parseInt(sessionTab.dataset.sessionIndex || "0");
        const sessions = useStore.getState().sessions;
        setContextMenu({
          x: e.clientX, y: e.clientY,
          items: [
            { label: "Close", icon: "✕", action: () => { if (sessions.length > 1) useStore.getState().removeSession(idx); } },
            { label: "Close Others", icon: "▬", action: () => {
              const kept = sessions[idx];
              useStore.getState().setSessions([kept]);
              useStore.getState().setActiveSessionIndex(0);
            }},
            { label: "Fork", icon: "⑂", action: () => {
              const original = sessions[idx];
              const dup: Session = { id: `s-${Date.now()}`, name: `${original.name} (copy)`, messages: original.messages.map((m) => ({ ...m })) };
              const next = [...sessions];
              next.splice(idx + 1, 0, dup);
              useStore.getState().setSessions(next);
              useStore.getState().setActiveSessionIndex(idx + 1);
            }},
            { label: "Export", icon: "⬇", action: () => {
              const json = JSON.stringify(sessions[idx], null, 2);
              const blob = new Blob([json], { type: "application/json" });
              const url = URL.createObjectURL(blob);
              const a = document.createElement("a");
              a.href = url; a.download = `${sessions[idx].name}.json`;
              a.click(); URL.revokeObjectURL(url);
            }},
          ],
        });
      } else if (chatMessage) {
        const text = chatMessage.dataset.messageText || "";
        if (window.getSelection()?.toString()) {
          e.preventDefault();
          const selected = window.getSelection()?.toString() || "";
          setContextMenu({
            x: e.clientX, y: e.clientY,
            items: [
              { label: "Copy", icon: "📋", action: () => navigator.clipboard.writeText(selected).catch(() => {}) },
              { label: "Ask AI about this", icon: "🤖", action: () => {
                useStore.getState().setStatusText(`询问 AI 关于: ${selected.slice(0, 40)}...`);
              }},
              { label: "Explain code", icon: "💡", action: () => {
                useStore.getState().setStatusText(`解释代码: ${selected.slice(0, 40)}...`);
              }},
            ],
          });
        } else {
          e.preventDefault();
          setContextMenu({
            x: e.clientX, y: e.clientY,
            items: [
              { label: "Copy Message", icon: "📋", action: () => navigator.clipboard.writeText(text).catch(() => {}) },
            ],
          });
        }
      }
    };
    document.addEventListener("contextmenu", handler);
    return () => document.removeEventListener("contextmenu", handler);
  }, [openEditor]);

  const handleSubmit = useCallback(async (text: string) => {
    if (!text.trim()) return;

    if (text.startsWith("/")) {
      await handleCommand(text);
      return;
    }

    setAgentBusy(true);
    setStatusText("思考中...");
    pushMessage("user", text);
    clearStreamingContent();

    const abort = new AbortController();
    abortRef.current = abort;

    try {
      const result = await api.agentReason(text);
      if (abort.signal.aborted) return;
      if (result.success && result.output) {
        pushMessage("assistant", result.output);
      } else {
        pushMessage("error", result.output || "Agent returned empty response");
      }
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
        setDiffData({ blocks: diffBlocks, filename: "Working changes" });
        pushMessage("system", `Diff: ${diffBlocks.length} blocks`);
      } else if (cmd === "help") {
        pushMessage("system", `Commands: /stats, /brain, /diff, /help`);
      } else {
        pushMessage("system", `Unknown command: ${cmd}. Type /help for available commands`);
      }
    } catch (e) {
      pushMessage("error", `Command failed: ${e}`);
    }

    setAgentBusy(false);
    setStatusText("Ready");
  };

  const handleSelectProject = useCallback(async () => {
    try {
      await invoke("read_dir_recursive", { path: ".", maxDepth: 1 });
      setProjectPath(".");
      setShowFileTree(true);
      setStatusText("Project loaded");
    } catch (e) {
      setStatusText(`Project load failed: ${e}`);
    }
  }, [setProjectPath, setShowFileTree, setStatusText]);

  const handlePermissionApprove = useCallback(async (id: string) => {
    try {
      await api.respondPermission(id, true);
      pushMessage("system", `Permission approved: ${id}`);
    } catch (e) {
      pushMessage("error", `Approve failed: ${e}`);
    }
    setPendingPermission(null);
  }, [pushMessage, setPendingPermission]);

  const handlePermissionDeny = useCallback(async (id: string) => {
    try {
      await api.respondPermission(id, false);
      pushMessage("system", `Permission denied: ${id}`);
    } catch (e) {
      pushMessage("error", `Deny failed: ${e}`);
    }
    setPendingPermission(null);
  }, [pushMessage, setPendingPermission]);

  const handleSaveProvider = useCallback((config: ProviderConfig) => {
    setProviderConfig(config);
    setStatusText(`Provider saved: ${config.name} / ${config.model}`);
  }, [setProviderConfig, setStatusText]);

  const handleTestProvider = useCallback(async (config: ProviderConfig): Promise<boolean> => {
    setStatusText("Testing connection...");
    const ok = await api.testProviderConnection(config);
    setStatusText(ok ? "Connection OK" : "Connection failed");
    return ok;
  }, [setStatusText]);

  const handleAddKnowledge = useCallback((entry: Omit<KnowledgeEntry, "id" | "created" | "updated">) => {
    const newEntry: KnowledgeEntry = {
      ...entry,
      id: `k-${Date.now()}`,
      created: Date.now(),
      updated: Date.now(),
    };
    setKnowledgeBase([...knowledgeBase, newEntry]);
    setStatusText(`Knowledge added: ${entry.title}`);
  }, [knowledgeBase, setKnowledgeBase, setStatusText]);

  const handleDeleteKnowledge = useCallback((id: string) => {
    setKnowledgeBase(knowledgeBase.filter((e) => e.id !== id));
  }, [knowledgeBase, setKnowledgeBase]);

  const handleSearchKnowledge = useCallback(async (query: string) => {
    if (!query.trim()) return;
    setStatusText(`Searching: ${query}`);
    const results = await api.searchKnowledge(query);
    if (results.length > 0) {
      setStatusText(`Found ${results.length} results`);
    } else {
      setStatusText("No results");
    }
  }, [setStatusText]);

  const handleSaveSettings = useCallback((newSettings: AppSettings) => {
    setSettings(newSettings);
    setStatusText("Settings saved");
  }, [setSettings, setStatusText]);

  const handleDuplicateSession = useCallback((index: number) => {
    const sessions = useStore.getState().sessions;
    const original = sessions[index];
    const id = `s-${Date.now()}`;
    const dup: Session = {
      id,
      name: `${original.name} (copy)`,
      messages: original.messages.map((m) => ({ ...m })),
    };
    const next = [...sessions];
    next.splice(index + 1, 0, dup);
    useStore.getState().setSessions(next);
    useStore.getState().setActiveSessionIndex(index + 1);
  }, []);

  const handleCloseOtherTabs = useCallback((keepIndex: number) => {
    const state = useStore.getState();
    const kept = state.sessions[keepIndex];
    const newActive = 0;
    useStore.getState().setSessions([kept]);
    useStore.getState().setActiveSessionIndex(0);
  }, []);

  const handleToggleTheme = useCallback(() => {
    const current = useStore.getState().settings.theme;
    const order: Array<"light" | "dark" | "system"> = ["light", "dark", "system"];
    const idx = order.indexOf(current);
    const next = order[(idx + 1) % order.length];
    setSettings({ ...useStore.getState().settings, theme: next });
  }, [setSettings]);

  function applyTheme(theme: "light" | "dark" | "system") {
    const isDark = theme === "dark" || (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.setAttribute("data-theme", isDark ? "dark" : "light");
  }

  return (
    <div className="app-container">
      {showOnboarding && <Lazy><Onboarding /></Lazy>}

      {showSearch && (
        <ErrorBoundary>
          <Lazy><SearchOverlay /></Lazy>
        </ErrorBoundary>
      )}

      {showShortcuts && (
        <ErrorBoundary>
          <Lazy><ShortcutsPanel onClose={() => setShowShortcuts(false)} /></Lazy>
        </ErrorBoundary>
      )}

      {showSettings && (
        <ErrorBoundary>
          <Lazy><Settings
            settings={settings}
            providerConfig={providerConfig}
            knowledgeBase={knowledgeBase}
            onSaveSettings={handleSaveSettings}
            onSaveProvider={handleSaveProvider}
            onTestProvider={handleTestProvider}
            onAddKnowledge={handleAddKnowledge}
            onDeleteKnowledge={handleDeleteKnowledge}
            onSearchKnowledge={handleSearchKnowledge}
            onClose={() => setShowSettings(false)}
          /></Lazy>
        </ErrorBoundary>
      )}

      {proxyVisible && (
        <ErrorBoundary>
          <Lazy><ProxyPanel
            status={proxyStatus}
            onStatusChange={setProxyStatus}
            onClose={() => setProxyVisible(false)}
          /></Lazy>
        </ErrorBoundary>
      )}

      {pendingPermission && (
        <ErrorBoundary>
          <Lazy><PermissionDialog
            request={pendingPermission}
            onApprove={handlePermissionApprove}
            onDeny={handlePermissionDeny}
          /></Lazy>
        </ErrorBoundary>
      )}

      <ErrorBoundary>
        <SessionList
          sessions={sessions}
          activeSession={activeSessionIndex}
          onSelect={setActiveSessionIndex}
          onNew={addSession}
        />
      </ErrorBoundary>

      <ErrorBoundary fallback={
        <div className="glass-panel" style={{ padding: 20, margin: 8 }}>
          <h3>Panel render error</h3>
          <button className="btn-primary" onClick={() => window.location.reload()}>Restore default view</button>
        </div>
      }>
        <div className="main-panel">
          <TabBar
            sessions={sessions}
            activeSession={activeSessionIndex}
            onSelect={setActiveSessionIndex}
            onNew={addSession}
            onClose={(i) => { if (sessions.length > 1) useStore.getState().removeSession(i); }}
            onRename={renameSession}
            onReorder={reorderSessions}
            onDuplicate={handleDuplicateSession}
            onCloseOthers={handleCloseOtherTabs}
          />
          {diffData && (
            <Lazy><DiffViewer
              diffBlocks={diffData.blocks}
              filename={diffData.filename}
              onApply={() => { setDiffData(null); pushMessage("system", "Diff applied"); }}
              onReject={() => { setDiffData(null); pushMessage("system", "Diff rejected"); }}
            /></Lazy>
          )}

          {showTerminal && (
            <ErrorBoundary>
              <Lazy><Terminal
                sessionId={terminalSessionId.current}
                onClose={() => { setShowTerminal(false); setTerminalStatus(""); }}
                onStatusChange={setTerminalStatus}
              /></Lazy>
            </ErrorBoundary>
          )}

          <div className="app-toolbar">
            <button
              className={`btn-icon${splitViewActive ? " active" : ""}`}
              onClick={() => setSplitViewActive(!splitViewActive)}
              title="Split View"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                <rect x="1" y="2" width="14" height="12" rx="2" />
                <line x1="8" y1="2" x2="8" y2="14" />
              </svg>
            </button>
            <button
              className={`btn-icon${virtualOSActive ? " active" : ""}`}
              onClick={() => setVirtualOSActive(!virtualOSActive)}
              title="Virtual OS Mode"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
                <rect x="1" y="1" width="14" height="10" rx="1.5" />
                <line x1="1" y1="6" x2="15" y2="6" />
                <line x1="5" y1="11" x2="5" y2="15" />
                <line x1="11" y1="11" x2="11" y2="15" />
                <line x1="3" y1="15" x2="13" y2="15" />
                <circle cx="8" cy="3.5" r="0.8" fill="currentColor" stroke="none" />
              </svg>
            </button>
            <button
              className={`btn-icon${agentMakerActive ? " active" : ""}`}
              onClick={() => setAgentMakerActive(!agentMakerActive)}
              title="Agent Maker"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
                <rect x="2" y="3" width="12" height="10" rx="2" />
                <circle cx="8" cy="8" r="2" />
                <path d="M8 4V3M8 13v-1M4 8H3M13 8h-1M5.05 5.05l-.7-.7M11.65 11.65l.7.7M5.05 10.95l-.7.7M11.65 4.35l.7-.7" />
              </svg>
            </button>
            <button
              className={`btn-icon${agentFlowActive ? " active" : ""}`}
              onClick={() => setAgentFlowActive(!agentFlowActive)}
              title="Agent Flow (Cmd+F)"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="4" cy="4" r="2" />
                <circle cx="4" cy="12" r="2" />
                <circle cx="12" cy="8" r="2" />
                <path d="M6 4h4l2 4M6 12h4l2-4M4 6v4" />
              </svg>
            </button>
            <button
              className={`btn-icon${syncVisible ? " active" : ""}`}
              onClick={() => setSyncVisible(!syncVisible)}
              title="File Sync"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
                <path d="M3 8a5 5 0 0 1 8-4M13 8a5 5 0 0 1-8 4" />
                <path d="M11 4h3v3M5 12H2v-3" />
              </svg>
            </button>
            <button
              className={`btn-icon${evolutionVisible ? " active" : ""}`}
              onClick={() => setEvolutionVisible(!evolutionVisible)}
              title="Evolution Dashboard (Cmd+E)"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
                <path d="M2 14L5 9l3 3 6-8" />
                <circle cx="13" cy="5" r="1.5" fill="currentColor" stroke="none" />
              </svg>
            </button>
            <button
              className="btn-icon"
              onClick={() => setShowShortcuts(true)}
              title="Keyboard Shortcuts (Cmd+/)"
            >
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                <rect x="1.5" y="3.5" width="11" height="7" rx="1.5" stroke="currentColor" strokeWidth="1.2" />
                <path d="M4 5.5v3M7 5.5v3M10 5.5v3M5.5 7h3" stroke="currentColor" strokeWidth="1" strokeLinecap="round" />
              </svg>
            </button>
          </div>

          {evolutionVisible ? (
            <Lazy><EvolutionPanel /></Lazy>
          ) : syncVisible ? (
            <Lazy><SyncPanel /></Lazy>
          ) : agentFlowActive ? (
            <Lazy><AgentFlow /></Lazy>
          ) : agentMakerActive ? (
            <Lazy><AgentMaker /></Lazy>
          ) : virtualOSActive ? (
            <Lazy><VirtualOS /></Lazy>
          ) : splitViewActive ? (
            <Lazy><SplitView /></Lazy>
          ) : (
            <>
              <ChatPanel
                messages={activeMessages}
                agentBusy={agentBusy}
                streamingContent={streamingContent}
                streamingContentType={streamingContentType}
              />
              <InputPanel
                value={input}
                onChange={setInput}
                onSubmit={handleSubmit}
                multiLine={multiLine}
                onMultiLineToggle={() => setMultiLine(!multiLine)}
                disabled={agentBusy}
              />
            </>
          )}

          <StatusBar
            text={statusText}
            agentBusy={agentBusy}
            sessionIndex={activeSessionIndex + 1}
            sessionCount={sessions.length}
            showTerminal={showTerminal}
            terminalStatus={terminalStatus}
            onOpenSettings={() => setShowSettings(true)}
            onSelectProject={handleSelectProject}
            onToggleTerminal={() => setShowTerminal(!showTerminal)}
            onToggleTheme={handleToggleTheme}
            onOpenProxy={() => setProxyVisible(true)}
            proxyStatus={proxyStatus}
            theme={theme}
          />
        </div>
      </ErrorBoundary>

      {showFileTree && projectPath && (
        <ErrorBoundary>
          <Lazy><FileTree
            rootPath={projectPath}
            onClose={() => setShowFileTree(false)}
            onStatusChange={setStatusText}
          /></Lazy>
        </ErrorBoundary>
      )}

      {editorState.open && (
        <ErrorBoundary>
          <div className="editor-panel">
            <Lazy><CodeEditor
              filePath={editorState.filePath}
              initialContent={editorState.initialContent}
              language={editorState.language}
              onClose={closeEditor}
              onSave={(path) => {
                setStatusText(`Saved: ${path}`);
              }}
            /></Lazy>
          </div>
        </ErrorBoundary>
      )}

      {contextMenu && (
        <Lazy><ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={contextMenu.items}
          onClose={() => setContextMenu(null)}
        /></Lazy>
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

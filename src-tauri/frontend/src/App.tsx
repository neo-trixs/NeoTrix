import React, { useEffect, useRef, useCallback, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useStore } from "./store";
import ChatPanel from "./components/ChatPanel";
import InputPanel from "./components/InputPanel";
import StatusBar from "./components/StatusBar";
import TabBar from "./components/TabBar";
import SessionSidebar from "./components/SessionSidebar";
import RightPanel from "./components/RightPanel";
import SettingsDialog from "./components/SettingsDialog";
import ToastContainer from "./components/ToastContainer";
import { ErrorBoundary } from "./components/ErrorBoundary";
import SmartCanvas from "./components/SmartCanvas";
import ShortcutsModal from "./components/ShortcutsModal";
import TaskProgressPanel from "./components/TaskProgressPanel";
import PermissionDialog from "./components/PermissionDialog";
import CommandPalette from "./components/CommandPalette";
import ContextMenu from "./components/ContextMenu";
import Terminal from "./components/Terminal";
import * as api from "./lib/api";
import EvolutionPanel from "./components/EvolutionPanel";
import ProxyPanel from "./components/ProxyPanel";
import McpManager from "./components/McpManager";
import SandboxControl from "./components/SandboxControl";
import MomentFeed from "./components/MomentFeed";
import ConsciousnessDashboard from "./components/ConsciousnessDashboard";
import ProjectRules from "./components/ProjectRules";
import Onboarding from "./components/Onboarding";
import AgentDashboard from "./components/AgentDashboard";

const App: React.FC = () => {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [rightPanelOpen, setRightPanelOpen] = useState(true);
  const [terminalHeight, setTerminalHeight] = useState(280);
  const terminalResizeRef = useRef<{ startY: number; startH: number } | null>(null);

  const sessions = useStore((s) => s.sessions);
  const activeSessionIndex = useStore((s) => s.activeSessionIndex);
  const setActiveSessionIndex = useStore((s) => s.setActiveSessionIndex);
  const addSession = useStore((s) => s.addSession);
  const removeSession = useStore((s) => s.removeSession);
  const renameSession = useStore((s) => s.renameSession);
  const reorderSessions = useStore((s) => s.reorderSessions);

  const statusText = useStore((s) => s.statusText);
  const agentBusy = useStore((s) => s.agentBusy);
  const streamingContent = useStore((s) => s.streamingContent);
  const streamingContentType = useStore((s) => s.streamingContentType);
  const settings = useStore((s) => s.settings);
  const pendingPermission = useStore((s) => s.pendingPermission);
  const canvasActive = useStore((s) => s.canvasActive);
  const contextChips = useStore((s) => s.contextChips);

  const pushMessage = useStore((s) => s.pushMessage);
  const setPendingPermission = useStore((s) => s.setPendingPermission);
  const appendStreamingContent = useStore((s) => s.appendStreamingContent);
  const commitStreamingContent = useStore((s) => s.commitStreamingContent);
  const clearStreamingContent = useStore((s) => s.clearStreamingContent);
  const setCanvasNodes = useStore((s) => s.setCanvasNodes);
  const setContextChips = useStore((s) => s.setContextChips);
  const addContextChip = useStore((s) => s.addContextChip);
  const removeContextChip = useStore((s) => s.removeContextChip);
  const setShowSettings = useStore((s) => s.setShowSettings);
  const showSettings = useStore((s) => s.showSettings);
  const setAgentMode = useStore((s) => s.setAgentMode);
  const agentMode = useStore((s) => s.agentMode);
  const showShortcuts = useStore((s) => s.showShortcuts);
  const setShowShortcuts = useStore((s) => s.setShowShortcuts);
  const showCommandPalette = useStore((s) => s.showCommandPalette);
  const setShowCommandPalette = useStore((s) => s.setShowCommandPalette);
  const showOnboarding = useStore((s) => s.showOnboarding);
  const projectRulesVisible = useStore((s) => s.projectRulesVisible);
  const setProjectRulesVisible = useStore((s) => s.setProjectRulesVisible);
  const showTerminal = useStore((s) => s.showTerminal);
  const setShowTerminal = useStore((s) => s.setShowTerminal);
  const evolutionVisible = useStore((s) => s.evolutionVisible);
  const proxyVisible = useStore((s) => s.proxyVisible);
  const proxyStatus = useStore((s) => s.proxyStatus);
  const sandboxVisible = useStore((s) => s.sandboxVisible);
  const sandboxStatus = useStore((s) => s.sandboxStatus);
  const momentFeedVisible = useStore((s) => s.momentFeedVisible);
  const consciousnessDashboardVisible = useStore((s) => s.consciousnessDashboardVisible);
  const syncVisible = useStore((s) => s.syncVisible);
  const setEvolutionVisible = useStore((s) => s.setEvolutionVisible);
  const setProxyVisible = useStore((s) => s.setProxyVisible);
  const setProxyStatus = useStore((s) => s.setProxyStatus);
  const setSandboxVisible = useStore((s) => s.setSandboxVisible);
  const setSandboxStatus = useStore((s) => s.setSandboxStatus);
  const setMomentFeedVisible = useStore((s) => s.setMomentFeedVisible);
  const setConsciousnessDashboardVisible = useStore((s) => s.setConsciousnessDashboardVisible);
  const agentDashboardVisible = useStore((s) => s.agentDashboardVisible);
  const setAgentDashboardVisible = useStore((s) => s.setAgentDashboardVisible);
  const addNotification = useStore((s) => s.addNotification);

  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number } | null>(null);

  const abortRef = useRef<AbortController | null>(null);
  const activeMessages = sessions[activeSessionIndex]?.messages || [];

  useEffect(() => {
    const unsubs: (() => void)[] = [];
    listen<{ token: string; full: string; error?: string }>("streaming-token", (event) => {
      const store = useStore.getState();
      if (event.payload.error) {
        store.pushMessage("error", event.payload.error);
        store.clearStreamingContent();
        store.setAgentBusy(false);
        store.setStatusText("Ready");
        return;
      }
      store.appendStreamingContent(event.payload.token);
    }).then((fn) => unsubs.push(fn));
    listen<{ full: string }>("streaming-done", (event) => {
      const store = useStore.getState();
      store.commitStreamingContent("assistant", "markdown");
      store.setAgentBusy(false);
      store.setStatusText("Ready");
      api.distillMessage(event.payload.full).catch(() => {});
    }).then((fn) => unsubs.push(fn));
    listen<{ nodes: any[]; edges: any[] }>("canvas-update", (event) => {
      const store = useStore.getState();
      if (event.payload.nodes?.length) {
        store.setCanvasNodes(event.payload.nodes, event.payload.edges || []);
      }
    }).then((fn) => unsubs.push(fn));
    listen<any>("consciousness-tick", (event) => {
      const store = useStore.getState();
      const p = event.payload;
      store.setConsciousnessData(p);

      let raw = (1 - (p.c_score ?? 0.5)) * 100;
      if (p.load_mode === "active") raw = Math.min(100, raw + 15);
      else if (p.load_mode === "idle") raw = Math.max(0, raw - 15);
      store.setContextUsage(Math.round(Math.min(100, Math.max(0, raw))));

      if (store.agentBusy) {
        store.setConsciousnessStep((store.consciousnessStep + 1) % 13);
      }

      store.setCurrentModel({
        id: "claude-sonnet-4",
        name: "Claude Sonnet 4",
        provider: "anthropic",
        capabilities: ["code", "reason"],
        context_length: 1000000,
      });
      if (p.proxy_running !== undefined) {
        store.setProxyStatus({
          running: p.proxy_running,
          mode: p.proxy_mode || "off",
          pid: p.proxy_pid || 0,
          port: p.proxy_port || 11080,
          uptime_secs: p.proxy_uptime || 0,
          active_count: p.proxy_active_count || 0,
          idle_secs: p.proxy_idle_secs || 0,
        });
      }
      if (p.evolution_active) store.setEvolutionVisible(true);
      if (p.consciousness_anomaly) store.setConsciousnessDashboardVisible(true);
      if (p.moment_feed_update) store.setMomentFeedVisible(true);
    }).then((fn) => unsubs.push(fn));
    listen<any>("proxy-status", (event) => {
      const store = useStore.getState();
      store.setProxyStatus(event.payload);
    }).then((fn) => unsubs.push(fn));
    listen<any>("sandbox-status", (event) => {
      const store = useStore.getState();
      store.setSandboxStatus(event.payload);
    }).then((fn) => unsubs.push(fn));
    return () => unsubs.forEach((fn) => fn());
  }, []);

  useEffect(() => {
    const isDark = settings.theme === "dark" || (settings.theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.setAttribute("data-theme", isDark ? "dark" : "light");
  }, [settings.theme]);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    const st = useStore.getState();
    if (st.pendingPermission) {
      if (e.key === "Enter") {
        e.preventDefault();
        api.respondPermission(st.pendingPermission.id, true).catch(() => {});
        st.setPendingPermission(null);
        return;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        api.respondPermission(st.pendingPermission.id, false).catch(() => {});
        st.setPendingPermission(null);
        return;
      }
    }
    if (e.key === "Escape" && (e.target as HTMLElement)?.tagName !== "TEXTAREA" && (e.target as HTMLElement)?.tagName !== "INPUT") {
      useStore.getState().setShowSettings(false);
    }
    if (e.key === "l" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      const input = document.querySelector<HTMLTextAreaElement>(".input-field");
      input?.focus();
    }
    if (e.key === "," && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      useStore.getState().setShowSettings(true);
    }
    if (e.key === "." && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      const modeCycle: Record<string, string> = { chat: "plan", plan: "agent", agent: "chat" };
      const cur = useStore.getState().agentMode;
      useStore.getState().setAgentMode(modeCycle[cur] as any);
    }
    if (e.key === "/" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      useStore.getState().setShowShortcuts(!useStore.getState().showShortcuts);
    }
    if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      useStore.getState().setShowCommandPalette(!useStore.getState().showCommandPalette);
    }
    if (e.key === "b" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      setSidebarOpen((v) => !v);
    }
    if ((e.key === "Escape" || e.key === "Escape") && (e.metaKey || e.ctrlKey)) {
      abortRef.current?.abort();
    }
    if (e.key === "e" && (e.metaKey || e.ctrlKey) && e.shiftKey) {
      e.preventDefault();
      useStore.getState().setEvolutionVisible(!useStore.getState().evolutionVisible);
    }
    if (e.key === "p" && (e.metaKey || e.ctrlKey) && e.shiftKey) {
      e.preventDefault();
      useStore.getState().setProxyVisible(!useStore.getState().proxyVisible);
    }
    if (e.key === "s" && (e.metaKey || e.ctrlKey) && e.shiftKey) {
      e.preventDefault();
      useStore.getState().setSandboxVisible(!useStore.getState().sandboxVisible);
    }
    if (e.key === "d" && (e.metaKey || e.ctrlKey) && e.shiftKey) {
      e.preventDefault();
      useStore.getState().setConsciousnessDashboardVisible(!useStore.getState().consciousnessDashboardVisible);
    }
    if (e.key === "a" && (e.metaKey || e.ctrlKey) && e.shiftKey) {
      e.preventDefault();
      useStore.getState().setAgentDashboardVisible(!useStore.getState().agentDashboardVisible);
    }
    if (e.key === "m" && (e.metaKey || e.ctrlKey) && e.shiftKey) {
      e.preventDefault();
      useStore.getState().setMcpManagerVisible(!useStore.getState().mcpManagerVisible);
    }
    if (e.key === "t" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      useStore.getState().addSession();
    }
    if (e.key === "w" && (e.metaKey || e.ctrlKey) && (e.target as HTMLElement)?.tagName !== "TEXTAREA") {
      e.preventDefault();
      const store = useStore.getState();
      if (store.sessions.length > 1) {
        store.removeSession(store.activeSessionIndex);
      }
    }
    if (e.key === "`" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      const st = useStore.getState();
      st.setShowTerminal(!st.showTerminal);
    }
    if (e.key === "r" && (e.metaKey || e.ctrlKey) && e.shiftKey) {
      e.preventDefault();
      const st = useStore.getState();
      st.setProjectRulesVisible(!st.projectRulesVisible);
    }
  }, []);

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const handleSubmit = async (text: string) => {
    if (!text.trim()) return;
    const store = useStore.getState();
    store.setAgentBusy(true);
    store.setStatusText("Thinking...");
    store.pushMessage("user", text);
    store.clearStreamingContent();
    const abort = new AbortController();
    abortRef.current = abort;
    try {
      const result = await api.agentReason(text);
      if (abort.signal.aborted) return;
      if (result.success) {
        store.pushMessage("assistant", result.output || "");
      } else {
        store.pushMessage("error", result.output || "Request failed");
      }
    } catch (e) {
      if (!abort.signal.aborted) store.pushMessage("error", `Request failed: ${e}`);
    }
    store.setAgentBusy(false);
    store.setStatusText("Ready");
    abortRef.current = null;
    api.distillMessage(text).catch(() => {});
  };

  return (
    <div className="app-layout">
      <SessionSidebar isOpen={sidebarOpen} onToggle={() => setSidebarOpen((v) => !v)} />

      <div className="app-center">
        <ErrorBoundary>
          <TabBar
            sessions={sessions}
            activeSession={activeSessionIndex}
            onSelect={setActiveSessionIndex}
            onNew={addSession}
            onClose={removeSession}
            onRename={renameSession}
            onReorder={reorderSessions}
            onDuplicate={(i) => {
              const s = sessions[i];
              if (!s) return;
              const newSession = { id: `s-${Date.now()}`, name: `${s.name} (copy)`, messages: [...s.messages] };
              useStore.setState((state) => {
                const next = [...state.sessions, newSession];
                return { sessions: next };
              });
            }}
            onCloseOthers={(i) => {
              const s = sessions[i];
              if (!s) return;
              useStore.setState({ sessions: [s], activeSessionIndex: 0 });
            }}
          />
          <TaskProgressPanel />
          <div className="chat-area glass">
            <ChatPanel messages={activeMessages} agentBusy={agentBusy}
              streamingContent={streamingContent} streamingContentType={streamingContentType} />
            <InputPanel onSubmit={handleSubmit} disabled={agentBusy} />
          </div>
          <StatusBar />
          {/* Terminal dock with slide animation */}
          <div className={`terminal-dock-wrapper ${showTerminal ? 'open' : ''}`} style={showTerminal ? { maxHeight: terminalHeight } : undefined}>
            <div
              className="terminal-resize-handle"
              onMouseDown={(e) => {
                e.preventDefault();
                const startY = e.clientY;
                const startH = terminalHeight;
                terminalResizeRef.current = { startY, startH };
                const handleMove = (ev: MouseEvent) => {
                  if (!terminalResizeRef.current) return;
                  const delta = terminalResizeRef.current.startY - ev.clientY;
                  setTerminalHeight(Math.max(100, Math.min(600, terminalResizeRef.current.startH + delta)));
                };
                const handleUp = () => {
                  terminalResizeRef.current = null;
                  window.removeEventListener('mousemove', handleMove);
                  window.removeEventListener('mouseup', handleUp);
                };
                window.addEventListener('mousemove', handleMove);
                window.addEventListener('mouseup', handleUp);
              }}
            />
            <div className="terminal-dock glass-strong">
              <div className="terminal-header">
                <span className="terminal-title">Terminal</span>
                <div className="terminal-actions">
                  <button className="btn-ghost btn-sm" onClick={() => setShowTerminal(false)}>
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round">
                      <path d="M3 3l6 6M9 3l-6 6" />
                    </svg>
                  </button>
                </div>
              </div>
              <div className="terminal-body">
                <Terminal sessionId={sessions[activeSessionIndex]?.id || "default"} />
              </div>
            </div>
          </div>
        </ErrorBoundary>
      </div>

      {rightPanelOpen && <RightPanel />}

      {canvasActive ? <SmartCanvas /> : null}

      {showShortcuts && <ShortcutsModal onClose={() => setShowShortcuts(false)} />}

      {showSettings && <SettingsDialog />}

      {pendingPermission && (
        <PermissionDialog
          request={pendingPermission}
          onApprove={(id) => { api.respondPermission(id, true).catch(() => {}); setPendingPermission(null); }}
          onDeny={(id) => { api.respondPermission(id, false).catch(() => {}); setPendingPermission(null); }}
        />
      )}

      {showCommandPalette && <CommandPalette />}

      {projectRulesVisible && <ProjectRules />}

      {showOnboarding && <Onboarding />}

      <ToastContainer />

      {ctxMenu && (
        <ContextMenu
          x={ctxMenu.x}
          y={ctxMenu.y}
          onClose={() => setCtxMenu(null)}
          items={[
            { label: "New session", shortcut: "Cmd+N", action: () => useStore.getState().addSession() },
            { label: "Command palette", shortcut: "Cmd+K", action: () => useStore.getState().setShowCommandPalette(true) },
            { label: "Toggle sidebar", shortcut: "Cmd+B", action: () => setSidebarOpen((v) => !v) },
            { label: "Toggle right panel", shortcut: "", action: () => setRightPanelOpen((v) => !v) },
            { divider: true, label: "", action: () => {} },
            { label: "Settings", shortcut: "Cmd+,", action: () => useStore.getState().setShowSettings(true) },
            { label: "Shortcuts", shortcut: "Cmd+/", action: () => useStore.getState().setShowShortcuts(true) },
            { divider: true, label: "", action: () => {} },
            { label: "Evolution (\u21E7\u2325E)", shortcut: "\u21E7\u2325E", action: () => useStore.getState().setEvolutionVisible(true) },
            { label: "Proxy (\u21E7\u2325P)", shortcut: "\u21E7\u2325P", action: () => useStore.getState().setProxyVisible(true) },
            { label: "Sandbox (\u21E7\u2325S)", shortcut: "\u21E7\u2325S", action: () => useStore.getState().setSandboxVisible(true) },
            { label: "Dashboard (\u21E7\u2325D)", shortcut: "\u21E7\u2325D", action: () => useStore.getState().setConsciousnessDashboardVisible(true) },
            { label: "Agent Dashboard (\u21E7\u2325A)", shortcut: "\u21E7\u2325A", action: () => useStore.getState().setAgentDashboardVisible(true) },
            { label: "Moments (\u21E7\u2325N)", shortcut: "\u21E7\u2325N", action: () => useStore.getState().setMomentFeedVisible(true) },
            { label: "MCP Manager (\u21E7\u2325M)", shortcut: "\u21E7\u2325M", action: () => useStore.getState().setMcpManagerVisible(true) },
            { divider: true, label: "", action: () => {} },
            { label: "Project Rules (\u21E7\u2325R)", shortcut: "\u21E7\u2325R", action: () => useStore.getState().setProjectRulesVisible(true) },
          ]}
        />
      )}

      {evolutionVisible && <EvolutionPanel />}
      {mcpManagerVisible && <McpManager />}
      {proxyVisible && (
        <ProxyPanel
          status={proxyStatus}
          onStatusChange={setProxyStatus}
          onClose={() => setProxyVisible(false)}
        />
      )}
      {sandboxVisible && <SandboxControl onClose={() => setSandboxVisible(false)} />}
      {momentFeedVisible && <MomentFeed />}
      {consciousnessDashboardVisible && <ConsciousnessDashboard />}
      {agentDashboardVisible && <AgentDashboard />}
    </div>
  );
};

export default App;

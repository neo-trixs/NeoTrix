import { create } from "zustand";
import type { Session, Message, ProviderConfig, PermissionRequest, KnowledgeEntry, AppSettings, AgentPreset, DesktopWindow, EvolutionState, EditorState, ProxyStatus, FeedState, BrainHealth, BrainEvent, CanvasNode, CanvasEdge, ContextChip, AgentMode, ModelInfo, SandboxStatus } from "./types";
import type { ProviderId } from "./types";
import { persistence } from "./lib/persistence";

const PERSIST_DEBOUNCE = 500;

interface AppState {
  sessions: Session[];
  activeSessionIndex: number;
  statusText: string;
  agentBusy: boolean;
  showSettings: boolean;
  projectPath: string;
  showFileTree: boolean;
  pendingPermission: PermissionRequest | null;
  providerConfig: ProviderConfig;
  knowledgeBase: KnowledgeEntry[];
  settings: AppSettings;
  streamingContent: string;
  streamingContentType: "markdown" | "html" | "text";
  splitViewActive: boolean;
  showTerminal: boolean;
  showOnboarding: boolean;
  showShortcuts: boolean;
  showSearch: boolean;
  searchQuery: string;
  setShowSearch: (show: boolean) => void;
  setSearchQuery: (query: string) => void;
  agentMakerActive: boolean;
  agentFlowActive: boolean;
  evolutionVisible: boolean;
  sandboxVisible: boolean;
  sandboxStatus: SandboxStatus;
  evolutionState: EvolutionState;
  customPresets: AgentPreset[];
  syncVisible: boolean;
  proxyVisible: boolean;
  proxyStatus: ProxyStatus;

  setSessions: (sessions: Session[]) => void;
  setActiveSessionIndex: (index: number) => void;
  setStatusText: (text: string) => void;
  setAgentBusy: (busy: boolean) => void;
  setShowSettings: (show: boolean) => void;
  setProjectPath: (path: string) => void;
  setShowFileTree: (show: boolean) => void;
  setPendingPermission: (req: PermissionRequest | null) => void;
  setProviderConfig: (config: ProviderConfig) => void;
  setKnowledgeBase: (entries: KnowledgeEntry[]) => void;
  setSettings: (settings: AppSettings) => void;

  pushMessage: (role: Message["role"], content: string, contentType?: "markdown" | "html" | "text") => void;
  addSession: () => void;
  activeMessages: () => Message[];
  setStreamingContent: (content: string, type?: "markdown" | "html" | "text") => void;
  appendStreamingContent: (chunk: string) => void;
  commitStreamingContent: (role?: "assistant" | "user", type?: "markdown" | "html" | "text") => void;
  clearStreamingContent: () => void;
  setShowTerminal: (show: boolean) => void;
  setSplitViewActive: (active: boolean) => void;
  setShowOnboarding: (show: boolean) => void;
  setShowShortcuts: (show: boolean) => void;
  removeSession: (index: number) => void;
  reorderSessions: (fromIndex: number, toIndex: number) => void;
  renameSession: (index: number, name: string) => void;
  setAgentMakerActive: (active: boolean) => void;
  setAgentFlowActive: (active: boolean) => void;
  setEvolutionVisible: (show: boolean) => void;
  setSandboxVisible: (show: boolean) => void;
  setSandboxStatus: (status: SandboxStatus) => void;
  setEvolutionState: (state: EvolutionState) => void;
  setSyncVisible: (show: boolean) => void;
  setProxyVisible: (show: boolean) => void;
  setProxyStatus: (status: ProxyStatus) => void;
  addCustomPreset: (preset: AgentPreset) => void;
  removeCustomPreset: (id: string) => void;
  virtualOSActive: boolean;
  setVirtualOSActive: (active: boolean) => void;
  desktopWindows: DesktopWindow[];
  openWindow: (win: DesktopWindow) => void;
  closeWindow: (id: string) => void;
  focusWindow: (id: string) => void;
  moveWindow: (id: string, x: number, y: number) => void;
  toggleMinimizeWindow: (id: string) => void;
  editorState: EditorState;
  openEditor: (filePath: string) => Promise<void>;
  closeEditor: () => void;
  forkSession: (id: string) => Promise<void>;
  exportSession: (id: string) => Promise<void>;
  importSession: () => Promise<void>;
  updateAvailable: boolean;
  updateStatus: string;
  updateProgress: number;
  setUpdateStatus: (available: boolean, status?: string) => void;
  setUpdateProgress: (progress: number) => void;
  notifications: Notification[];
  addNotification: (n: Omit<Notification, "id">) => void;
  removeNotification: (id: string) => void;
  showCommandPalette: boolean;
  setShowCommandPalette: (show: boolean) => void;
  projectRulesVisible: boolean;
  setProjectRulesVisible: (show: boolean) => void;
  momentFeedVisible: boolean;
  setMomentFeedVisible: (show: boolean) => void;
  feedState: FeedState | null;
  feedLoading: boolean;
  setFeedState: (s: FeedState | null) => void;
  setFeedLoading: (v: boolean) => void;
  setConsciousnessDashboardVisible: (show: boolean) => void;
  consciousnessDashboardVisible: boolean;
  consciousnessData: Record<string, any> | null;
  consciousnessStep: number;
  setConsciousnessData: (data: Record<string, any> | null) => void;
  setConsciousnessStep: (step: number) => void;
  agentDashboardVisible: boolean;
  setAgentDashboardVisible: (show: boolean) => void;
  brainHealth: BrainHealth | null;
  brainEvents: BrainEvent[];
  canvasActive: boolean;
  canvasNodes: CanvasNode[];
  canvasEdges: CanvasEdge[];
  setCanvasNodes: (nodes: CanvasNode[], edges: CanvasEdge[]) => void;
  setCanvasActive: (active: boolean) => void;
  // Rich UI state (Claude Code / Codex inspired)
  mcpManagerVisible: boolean;
  setMcpManagerVisible: (show: boolean) => void;
  contextChips: ContextChip[];
  setContextChips: (chips: ContextChip[]) => void;
  addContextChip: (chip: ContextChip) => void;
  removeContextChip: (id: string) => void;
  agentMode: AgentMode;
  setAgentMode: (mode: AgentMode) => void;
  contextUsage: number;
  setContextUsage: (pct: number) => void;
  currentModel: ModelInfo | null;
  setCurrentModel: (m: ModelInfo | null) => void;
  currentGitBranch: string;
  setCurrentGitBranch: (b: string) => void;
}

export interface Notification {
  id: string;
  type: "success" | "error" | "warning" | "info";
  message: string;
  duration?: number;
}

const DEFAULT_PROVIDER: ProviderConfig = {
  id: "anthropic",
  name: "Anthropic Claude",
  model: "claude-sonnet-4-20250514",
  apiKey: "",
  learningRate: 0.05,
};

const DEFAULT_SETTINGS: AppSettings = {
  theme: "light",
  fontSize: 13,
  autoSave: true,
  language: "zh-CN",
  terminalPath: "",
  maxSessions: 20,
};

let persistTimer: ReturnType<typeof setTimeout> | null = null;

function schedulePersist(sessions: Session[]) {
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    persistence.saveSessions(sessions);
  }, PERSIST_DEBOUNCE);
}

const savedProvider = persistence.loadProviderConfig();
const savedSettings = persistence.loadSettings();
const savedKnowledge = persistence.loadKnowledgeBase();
const savedSessions = persistence.loadSessions();
let onboardingDone = false;
try { onboardingDone = localStorage.getItem("neotrix_onboarding_done") === "true"; } catch {};
const hasApiKey = !!(savedProvider?.apiKey);

function loadCustomPresets(): AgentPreset[] {
  try {
    const raw = localStorage.getItem("neotrix_presets");
    return raw ? JSON.parse(raw) : [];
  } catch { return []; }
}

function saveCustomPresets(presets: AgentPreset[]) {
  try { localStorage.setItem("neotrix_presets", JSON.stringify(presets)); } catch {}
}

export const useStore = create<AppState>((set, get) => ({
  sessions: savedSessions.length > 0 ? savedSessions : [{ id: "default", name: "默认会话", messages: [] }],
  activeSessionIndex: 0,
  statusText: savedProvider ? "就绪" : "就绪 | Provider: 未配置",
  agentBusy: false,
  showSettings: false,
  projectPath: "",
  showFileTree: false,
  pendingPermission: null,
  providerConfig: savedProvider ?? DEFAULT_PROVIDER,
  knowledgeBase: savedKnowledge,
  settings: savedSettings ?? DEFAULT_SETTINGS,
  streamingContent: "",
  streamingContentType: "markdown",
  showTerminal: false,
  splitViewActive: false,
  showOnboarding: !onboardingDone && !hasApiKey,
  showShortcuts: false,
  showSearch: false,
  searchQuery: "",
  agentMakerActive: false,
  agentFlowActive: false,
  evolutionVisible: false,
  sandboxVisible: false,
  sandboxStatus: { running: false, active_count: 0, mode: "off", uptime_secs: 0, default_timeout_ms: 30000, allow_network: false, recent_executions: 0, failed_count: 0 },
  syncVisible: false,
  proxyVisible: false,
  proxyStatus: { running: false, mode: "off", pid: 0, port: 11080, uptime_secs: 0, active_count: 0, idle_secs: 0 },
  evolutionState: {
    iteration: 0, strategy: 'Direct', contextUsage: 0,
    intrinsicReward: 0.5, confidence: 0.5, errorRate: 0, noveltyScore: 0,
    shouldExplore: true, stabilityScore: 0.5, flagsCount: 0, repairsCount: 0,
    archiveSnapshots: 0, selfRepairs: 0,
  },
  customPresets: loadCustomPresets(),
  virtualOSActive: false,
  desktopWindows: [],
  editorState: { open: false, filePath: "", initialContent: "", language: "" },
  updateAvailable: false,
  updateStatus: "",
  updateProgress: 0,
  notifications: [],
  momentFeedVisible: false,
  feedState: null,
  feedLoading: false,
  consciousnessDashboardVisible: false,
  consciousnessData: null,
  consciousnessStep: 0,
  agentDashboardVisible: false,
  brainHealth: null,
  brainEvents: [],
  canvasActive: false,
  canvasNodes: [],
  canvasEdges: [],
  setCanvasNodes: (nodes, edges) => set({ canvasNodes: nodes, canvasEdges: edges, canvasActive: true }),
  setCanvasActive: (active) => set({ canvasActive: active }),
  contextChips: [],
  setContextChips: (chips) => set({ contextChips: chips }),
  addContextChip: (chip) => set((s) => ({ contextChips: [...s.contextChips.filter((c) => c.id !== chip.id), chip] })),
  removeContextChip: (id) => set((s) => ({ contextChips: s.contextChips.filter((c) => c.id !== id) })),
  agentMode: "chat",
  setAgentMode: (mode) => set({ agentMode: mode }),
  contextUsage: 0,
  mcpManagerVisible: false,
  setMcpManagerVisible: (show) => set({ mcpManagerVisible: show }),
  setContextUsage: (pct) => set({ contextUsage: Math.min(100, Math.max(0, pct)) }),
  currentModel: { id: "claude-sonnet-4", name: "Claude Sonnet 4", provider: "anthropic", capabilities: ["code","reason"], context_length: 1000000 },
  setCurrentModel: (m) => set({ currentModel: m }),
  currentGitBranch: "main",
  setCurrentGitBranch: (b) => set({ currentGitBranch: b }),

  setSessions: (sessions) => {
    set({ sessions });
    schedulePersist(sessions);
  },
  setActiveSessionIndex: (index) => set({ activeSessionIndex: index }),
  setStatusText: (text) => set({ statusText: text }),
  setAgentBusy: (busy) => set({ agentBusy: busy }),
  setShowSettings: (show) => set({ showSettings: show }),
  setProjectPath: (path) => set({ projectPath: path }),
  setShowFileTree: (show) => set({ showFileTree: show }),
  setPendingPermission: (req) => set({ pendingPermission: req }),

  setProviderConfig: (config) => {
    set({ providerConfig: config });
    persistence.saveProviderConfig(config);
  },
  setKnowledgeBase: (entries) => {
    set({ knowledgeBase: entries });
    persistence.saveKnowledgeBase(entries);
  },
  setSettings: (settings) => {
    set({ settings });
    persistence.saveSettings(settings);
  },

  pushMessage: (role, content, contentType) => set((state) => {
    const next = [...state.sessions];
    const session = { ...next[state.activeSessionIndex] };
    session.messages = [...session.messages, { role, content, contentType, timestamp: Date.now() }];
    next[state.activeSessionIndex] = session;
    schedulePersist(next);
    return { sessions: next };
  }),

  addSession: () => set((state) => {
    const id = `s-${Date.now()}`;
    const name = `会话 ${state.sessions.length + 1}`;
    const next = [...state.sessions, { id, name, messages: [] }];
    schedulePersist(next);
    return { sessions: next, activeSessionIndex: state.sessions.length };
  }),

  activeMessages: () => {
    const state = get();
    return state.sessions[state.activeSessionIndex]?.messages || [];
  },

  setStreamingContent: (content, type) => set({
    streamingContent: content,
    ...(type ? { streamingContentType: type } : {}),
  }),

  appendStreamingContent: (chunk) => set((state) => ({
    streamingContent: state.streamingContent + chunk,
  })),

  commitStreamingContent: (role, type) => set((state) => {
    if (!state.streamingContent) return {};
    const next = [...state.sessions];
    const session = { ...next[state.activeSessionIndex] };
    session.messages = [
      ...session.messages,
      {
        role: role || "assistant",
        content: state.streamingContent,
        contentType: type || state.streamingContentType,
        timestamp: Date.now(),
      },
    ];
    next[state.activeSessionIndex] = session;
    schedulePersist(next);
    return { sessions: next, streamingContent: "", streamingContentType: "markdown" };
  }),

  clearStreamingContent: () => set({
    streamingContent: "",
    streamingContentType: "markdown",
  }),
  setShowTerminal: (show) => set({ showTerminal: show }),
  setSplitViewActive: (active) => set({ splitViewActive: active }),
  setShowOnboarding: (show) => set({ showOnboarding: show }),
  removeSession: (index) => set((state) => {
    if (state.sessions.length <= 1) return {};
    const next = [...state.sessions];
    next.splice(index, 1);
    const newIndex = index >= next.length ? next.length - 1 : index;
    schedulePersist(next);
    return { sessions: next, activeSessionIndex: newIndex };
  }),
  reorderSessions: (fromIndex, toIndex) => set((state) => {
    const next = [...state.sessions];
    const [moved] = next.splice(fromIndex, 1);
    next.splice(toIndex, 0, moved);
    let newActive = state.activeSessionIndex;
    if (fromIndex === state.activeSessionIndex) {
      newActive = toIndex;
    } else {
      if (fromIndex < state.activeSessionIndex && toIndex >= state.activeSessionIndex) {
        newActive = state.activeSessionIndex - 1;
      } else if (fromIndex > state.activeSessionIndex && toIndex <= state.activeSessionIndex) {
        newActive = state.activeSessionIndex + 1;
      }
    }
    if (newActive < 0) newActive = 0;
    if (newActive >= next.length) newActive = next.length - 1;
    schedulePersist(next);
    return { sessions: next, activeSessionIndex: newActive };
  }),
  renameSession: (index, name) => set((state) => {
    const next = [...state.sessions];
    next[index] = { ...next[index], name };
    schedulePersist(next);
    return { sessions: next };
  }),
  setShowShortcuts: (show) => set({ showShortcuts: show }),
  setShowSearch: (show) => set({ showSearch: show }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setAgentMakerActive: (active) => set({ agentMakerActive: active }),
  setAgentFlowActive: (active) => set({ agentFlowActive: active }),
  setEvolutionVisible: (show) => set({ evolutionVisible: show }),
  setSandboxVisible: (show) => set({ sandboxVisible: show }),
  setSandboxStatus: (status) => set({ sandboxStatus: status }),
  setEvolutionState: (state) => set({ evolutionState: state }),
  setSyncVisible: (show) => set({ syncVisible: show }),
  setProxyVisible: (show) => set({ proxyVisible: show }),
  setProxyStatus: (status) => set({ proxyStatus: status }),
  addCustomPreset: (preset) => set((state) => {
    const next = [...state.customPresets, preset];
    saveCustomPresets(next);
    return { customPresets: next };
  }),
  removeCustomPreset: (id) => set((state) => {
    const next = state.customPresets.filter((p) => p.id !== id);
    saveCustomPresets(next);
    return { customPresets: next };
  }),
  setUpdateStatus: (available, status) => set({ updateAvailable: available, updateStatus: status || "" }),
  setUpdateProgress: (progress) => set({ updateProgress: progress }),
  setVirtualOSActive: (active) => set({ virtualOSActive: active }),
  openWindow: (win) => set((state) => ({ desktopWindows: [...state.desktopWindows, win] })),
  closeWindow: (id) => set((state) => ({ desktopWindows: state.desktopWindows.filter((w) => w.id !== id) })),
  focusWindow: (id) => set((state) => {
    const maxZ = Math.max(...state.desktopWindows.map((w) => w.zIndex), 0);
    return { desktopWindows: state.desktopWindows.map((w) => w.id === id ? { ...w, zIndex: maxZ + 1 } : w) };
  }),
  moveWindow: (id, x, y) => set((state) => ({
    desktopWindows: state.desktopWindows.map((w) => w.id === id ? { ...w, x, y } : w),
  })),
  toggleMinimizeWindow: (id) => set((state) => ({
    desktopWindows: state.desktopWindows.map((w) => w.id === id ? { ...w, minimized: !w.minimized } : w),
  })),
  openEditor: async (filePath: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const content = await invoke<string>("read_file", { path: filePath });
      const ext = filePath.split(".").pop()?.toLowerCase() || "";
      const langMap: Record<string, string> = {
        rs: "rust", ts: "typescript", tsx: "typescript", js: "javascript", jsx: "javascript",
        py: "python", html: "html", htm: "html", css: "css", json: "json", md: "markdown",
        mdx: "markdown", yaml: "yaml", yml: "yaml", toml: "toml", sh: "bash", bash: "bash",
        zsh: "bash", sql: "sql", go: "go", java: "java", kt: "kotlin", swift: "swift",
      };
      const language = langMap[ext] || "";
      set({ editorState: { open: true, filePath, initialContent: content, language } });
    } catch (e) {
      console.error("Failed to open file:", e);
      set({ statusText: `打开文件失败: ${e}` });
    }
  },
  closeEditor: () => set({ editorState: { open: false, filePath: "", initialContent: "", language: "" } }),
  forkSession: async (id: string) => {
    try {
      const newId = await invoke<string>("cmd_session_fork", { id });
      set((state) => {
        const src = state.sessions.find((s) => s.id === id);
        if (!src) return {};
        const newSession: Session = { id: newId, name: `${src.name} (副本)`, messages: [...src.messages] };
        const next = [...state.sessions, newSession];
        schedulePersist(next);
        return { sessions: next };
      });
    } catch (e) {
      console.error("Fork session failed:", e);
      set({ statusText: `复制会话失败: ${e}` });
    }
  },
  exportSession: async (id: string) => {
    try {
      const json = await invoke<string>("cmd_session_export_json", { id });
      const { save } = await import("@tauri-apps/plugin-dialog");
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");
      const path = await save({ defaultPath: `session-${id.slice(0, 8)}.json` });
      if (path) {
        await writeTextFile(path, json);
        set({ statusText: `会话已导出到 ${path}` });
      }
    } catch (e) {
      console.error("Export session failed:", e);
    }
  },
  importSession: async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { readTextFile } = await import("@tauri-apps/plugin-fs");
      const path = await open({ multiple: false, directories: false, filters: [{ name: "会话", extensions: ["json"] }] });
      if (!path) return;
      const content = await readTextFile(path as string);
      const importedIds = await invoke<string>("cmd_session_import_json", { json: content });
      const ids = importedIds.split(",").filter(Boolean);
      if (ids.length === 0) return;
      const newSessions = ids.map((newId: string) => ({ id: newId, name: "已导入会话", messages: [] as Message[] }));
      set((state) => {
        const next = [...state.sessions, ...newSessions];
        schedulePersist(next);
        return { sessions: next, statusText: `已导入 ${ids.length} 个会话` };
      });
    } catch (e) {
      console.error("Import session failed:", e);
      set({ statusText: `导入会话失败: ${e}` });
    }
  },
  addNotification: (n) => {
    const id = `notif-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    const notification: Notification = { ...n, id };
    set((state) => ({ notifications: [...state.notifications, notification] }));
    const duration = n.duration ?? 5000;
    if (duration > 0) {
      setTimeout(() => {
        set((state) => ({ notifications: state.notifications.filter((x) => x.id !== id) }));
      }, duration);
    }
  },
  showCommandPalette: false,
  setShowCommandPalette: (show) => set({ showCommandPalette: show }),
  projectRulesVisible: false,
  setProjectRulesVisible: (show) => set({ projectRulesVisible: show }),
  setMomentFeedVisible: (show) => set({ momentFeedVisible: show }),
  setFeedState: (s) => set({ feedState: s }),
  setFeedLoading: (v) => set({ feedLoading: v }),
  setConsciousnessDashboardVisible: (show) => set({ consciousnessDashboardVisible: show }),
  setConsciousnessData: (data) => set({ consciousnessData: data }),
  setConsciousnessStep: (step) => set({ consciousnessStep: step }),
  setAgentDashboardVisible: (show) => set({ agentDashboardVisible: show }),
  removeNotification: (id) => set((state) => ({ notifications: state.notifications.filter((x) => x.id !== id) })),
}));

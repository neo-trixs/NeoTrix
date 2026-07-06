import { invoke } from "@tauri-apps/api/core";
import type { Session, Message, PermissionRequest, KnowledgeEntry, AppSettings, Attachment } from "../types";
import { persistence } from "../lib/persistence";
import {
  schedulePersist,
  savedProvider,
  savedSessions,
  savedSettings,
  savedKnowledge,
  DEFAULT_SETTINGS,
} from "./store-utils";

export interface SessionSlice {
  sessions: Session[];
  activeSessionIndex: number;
  statusText: string;
  projectPath: string;
  showFileTree: boolean;
  pendingPermission: PermissionRequest | null;
  knowledgeBase: KnowledgeEntry[];
  settings: AppSettings;
  sessionSearchQuery: string;

  setSessions: (sessions: Session[]) => void;
  setActiveSessionIndex: (index: number) => void;
  setStatusText: (text: string) => void;
  setProjectPath: (path: string) => void;
  setShowFileTree: (show: boolean) => void;
  setPendingPermission: (req: PermissionRequest | null) => void;
  setKnowledgeBase: (entries: KnowledgeEntry[]) => void;
  setSettings: (settings: AppSettings) => void;
  pushMessage: (role: Message["role"], content: string, contentType?: "markdown" | "html" | "text", attachments?: Attachment[]) => void;
  addSession: () => void;
  activeMessages: () => Message[];
  removeSession: (index: number) => void;
  reorderSessions: (fromIndex: number, toIndex: number) => void;
  renameSession: (index: number, name: string) => void;
  forkSession: (id: string) => Promise<void>;
  exportSession: (id: string) => Promise<void>;
  importSession: () => Promise<void>;
  pinSession: (index: number) => void;
  updateMessage: (sessionIndex: number, messageIndex: number, content: string) => void;
  setSessionSearchQuery: (query: string) => void;
  getFilteredSessions: () => Session[];
}

function getTimeGroup(ts: number): string {
  const now = Date.now();
  const diff = now - ts;
  if (diff < 86400000) return "Today";
  if (diff < 172800000) return "Yesterday";
  if (diff < 604800000) return "This week";
  return "Earlier";
}

export const createSessionSlice = (set: any, get: any) => ({
  sessions: savedSessions.length > 0 ? savedSessions.map((s: Session) => ({ ...s, pinned: s.pinned || false, lastActive: s.lastActive || Date.now() })) : [{ id: "default", name: "默认会话", messages: [], pinned: false, lastActive: Date.now() }],
  activeSessionIndex: 0,
  statusText: savedProvider ? "就绪" : "就绪 | Provider: 未配置",
  projectPath: "",
  showFileTree: false,
  pendingPermission: null,
  knowledgeBase: savedKnowledge,
  settings: savedSettings ?? DEFAULT_SETTINGS,
  sessionSearchQuery: "",

  setSessions: (sessions: Session[]) => {
    set({ sessions });
    schedulePersist(sessions);
  },
  setActiveSessionIndex: (index: number) => set({ activeSessionIndex: index }),
  setStatusText: (text: string) => set({ statusText: text }),
  setProjectPath: (path: string) => set({ projectPath: path }),
  setShowFileTree: (show: boolean) => set({ showFileTree: show }),
  setPendingPermission: (req: PermissionRequest | null) => set({ pendingPermission: req }),
  setKnowledgeBase: (entries: KnowledgeEntry[]) => {
    set({ knowledgeBase: entries });
    persistence.saveKnowledgeBase(entries);
  },
  setSettings: (settings: AppSettings) => {
    set({ settings });
    persistence.saveSettings(settings);
  },

  pushMessage: (role: Message["role"], content: string, contentType?: "markdown" | "html" | "text", attachments?: Attachment[]) => set((state: any) => {
    const next = [...state.sessions];
    const session = { ...next[state.activeSessionIndex] };
    const msg: Message = { role, content, contentType, timestamp: Date.now() };
    if (attachments && attachments.length > 0) msg.attachments = attachments;
    session.messages = [...session.messages, msg];
    session.lastActive = Date.now();
    next[state.activeSessionIndex] = session;
    schedulePersist(next);
    return { sessions: next };
  }),

  addSession: () => set((state: any) => {
    const id = `s-${Date.now()}`;
    const name = `会话 ${state.sessions.length + 1}`;
    const now = Date.now();
    const next = [...state.sessions, { id, name, messages: [], pinned: false, lastActive: now }];
    schedulePersist(next);
    return { sessions: next, activeSessionIndex: state.sessions.length };
  }),

  activeMessages: () => {
    const state = get();
    return state.sessions[state.activeSessionIndex]?.messages || [];
  },

  removeSession: (index: number) => set((state: any) => {
    if (state.sessions.length <= 1) return {};
    const next = [...state.sessions];
    next.splice(index, 1);
    const newIndex = index >= next.length ? next.length - 1 : index;
    schedulePersist(next);
    return { sessions: next, activeSessionIndex: newIndex };
  }),

  reorderSessions: (fromIndex: number, toIndex: number) => set((state: any) => {
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

  renameSession: (index: number, name: string) => set((state: any) => {
    const next = [...state.sessions];
    next[index] = { ...next[index], name };
    schedulePersist(next);
    return { sessions: next };
  }),

  pinSession: (index: number) => set((state: any) => {
    const next = [...state.sessions];
    next[index] = { ...next[index], pinned: !next[index].pinned };
    schedulePersist(next);
    return { sessions: next };
  }),

  updateMessage: (sessionIndex: number, messageIndex: number, content: string) => set((state: any) => {
    const next = [...state.sessions];
    const session = { ...next[sessionIndex] };
    const msgs = [...session.messages];
    msgs[messageIndex] = { ...msgs[messageIndex], content };
    session.messages = msgs;
    session.lastActive = Date.now();
    next[sessionIndex] = session;
    schedulePersist(next);
    return { sessions: next };
  }),

  setSessionSearchQuery: (query: string) => set({ sessionSearchQuery: query }),

  getFilteredSessions: () => {
    const state = get();
    const q = state.sessionSearchQuery?.toLowerCase().trim();
    if (!q) return state.sessions;
    return state.sessions.filter((s: Session) =>
      s.name.toLowerCase().includes(q) ||
      s.messages.some((m: Message) => m.content.toLowerCase().includes(q))
    );
  },

  forkSession: async (id: string) => {
    try {
      const newId = await invoke<string>("cmd_session_fork", { id });
      set((state: any) => {
        const src = state.sessions.find((s: Session) => s.id === id);
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
      set((state: any) => {
        const next = [...state.sessions, ...newSessions];
        schedulePersist(next);
        return { sessions: next, statusText: `已导入 ${ids.length} 个会话` };
      });
    } catch (e) {
      console.error("Import session failed:", e);
      set({ statusText: `导入会话失败: ${e}` });
    }
  },
});

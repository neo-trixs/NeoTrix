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
  pendingPermission: PermissionRequest | null;
  knowledgeBase: KnowledgeEntry[];
  settings: AppSettings;

  setSessions: (sessions: Session[]) => void;
  setActiveSessionIndex: (index: number) => void;
  setStatusText: (text: string) => void;
  setPendingPermission: (req: PermissionRequest | null) => void;
  setKnowledgeBase: (entries: KnowledgeEntry[]) => void;
  setSettings: (settings: AppSettings) => void;
  pushMessage: (role: Message["role"], content: string, contentType?: "markdown" | "html" | "text", attachments?: Attachment[]) => void;
  addSession: () => void;
  activeMessages: () => Message[];
  removeSession: (index: number) => void;
  renameSession: (index: number, name: string) => void;
  updateMessage: (sessionIndex: number, messageIndex: number, content: string) => void;
}

export const createSessionSlice = (set: any, get: any) => ({
  sessions: savedSessions.length > 0 ? savedSessions.map((s: Session) => ({ ...s, pinned: s.pinned || false, lastActive: s.lastActive || Date.now() })) : [{ id: "default", name: "默认会话", messages: [], pinned: false, lastActive: Date.now() }],
  activeSessionIndex: 0,
  statusText: savedProvider ? "就绪" : "就绪 | Provider: 未配置",
  pendingPermission: null,
  knowledgeBase: savedKnowledge,
  settings: savedSettings ?? DEFAULT_SETTINGS,

  setSessions: (sessions: Session[]) => {
    set({ sessions });
    schedulePersist(sessions);
  },
  setActiveSessionIndex: (index: number) => set({ activeSessionIndex: index }),
  setStatusText: (text: string) => set({ statusText: text }),
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

  renameSession: (index: number, name: string) => set((state: any) => {
    const next = [...state.sessions];
    next[index] = { ...next[index], name };
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
});

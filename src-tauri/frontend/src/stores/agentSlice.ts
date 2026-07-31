import type { Attachment, NeoCodexMode } from "../types";

type NeoCodexChatMessage = { role: string; content: string; contentType?: "markdown" | "html" | "text"; timestamp?: number; attachments?: Attachment[] };

export interface AgentSlice {
  // NeoCodex Desktop state
  neocodexMode: NeoCodexMode;
  neocodexMessages: NeoCodexChatMessage[];
  neocodexStreaming: { content: string; role: "user" | "assistant" } | null;
  neocodexSessions: Array<{ id: string; name: string; mode: string; messages: any[]; wire_path: string; created_at: number; updated_at: number }>;
  neocodexActiveSessionId: string | null;

  // NeoCodex actions
  setNeoCodexMode: (mode: NeoCodexMode) => void;
  setNeoCodexMessages: (messages: NeoCodexChatMessage[]) => void;
  addNeoCodexMessage: (message: NeoCodexChatMessage) => void;
  setNeoCodexStreaming: (streaming: { content: string; role: "user" | "assistant" } | null) => void;
  setNeoCodexSessions: (sessions: Array<{ id: string; name: string; mode: string; messages: any[]; wire_path: string; created_at: number; updated_at: number }>) => void;
  setNeoCodexActiveSession: (sessionId: string | null) => void;
}

export const createAgentSlice = (set: any) => ({
  // NeoCodex initial state
  neocodexMode: "Agent",
  neocodexMessages: [],
  neocodexStreaming: null,
  neocodexSessions: [],
  neocodexActiveSessionId: null,

  // NeoCodex actions
  setNeoCodexMode: (mode: NeoCodexMode) => set({ neocodexMode: mode }),
  setNeoCodexMessages: (messages: NeoCodexChatMessage[]) => set({ neocodexMessages: messages }),
  addNeoCodexMessage: (message: NeoCodexChatMessage) => set((state: any) => ({ neocodexMessages: [...state.neocodexMessages, message] })),
  setNeoCodexStreaming: (streaming: { content: string; role: "user" | "assistant" } | null) => set({ neocodexStreaming: streaming }),
  setNeoCodexSessions: (sessions: Array<{ id: string; name: string; mode: string; messages: any[]; wire_path: string; created_at: number; updated_at: number }>) => set({ neocodexSessions: sessions }),
  setNeoCodexActiveSession: (sessionId: string | null) => set({ neocodexActiveSessionId: sessionId }),
});

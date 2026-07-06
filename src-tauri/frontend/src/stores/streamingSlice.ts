import type { Message } from "../types";
import { schedulePersist } from "./store-utils";

export interface StreamingSlice {
  streamingContent: string;
  streamingContentType: "markdown" | "html" | "text";

  setStreamingContent: (content: string, type?: "markdown" | "html" | "text") => void;
  appendStreamingContent: (chunk: string) => void;
  commitStreamingContent: (role?: "assistant" | "user", type?: "markdown" | "html" | "text") => void;
  clearStreamingContent: () => void;
}

export const createStreamingSlice = (set: any, get: any) => ({
  streamingContent: "",
  streamingContentType: "markdown" as const,

  setStreamingContent: (content: string, type?: "markdown" | "html" | "text") => set({
    streamingContent: content,
    ...(type ? { streamingContentType: type } : {}),
  }),

  appendStreamingContent: (chunk: string) => set((state: any) => ({
    streamingContent: state.streamingContent + chunk,
  })),

  commitStreamingContent: (role?: "assistant" | "user", type?: "markdown" | "html" | "text") => set((state: any) => {
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
});

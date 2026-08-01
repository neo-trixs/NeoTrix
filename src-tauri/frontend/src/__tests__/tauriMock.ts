export type InvokeHandler = (args: any) => any;

interface TauriTestGlobals {
  __invokeHandlers?: Record<string, InvokeHandler>;
}

const g = globalThis as TauriTestGlobals;
g.__invokeHandlers = g.__invokeHandlers ?? {};

/** Register a per-command invoke handler for the current test. */
export function mockInvoke(cmd: string, handler: InvokeHandler) {
  g.__invokeHandlers![cmd] = handler;
}

/** Clear all custom invoke handlers (restores defaults). */
export function resetInvokeMocks() {
  g.__invokeHandlers = {};
}

/** Convenience factory: session list + per-session messages. */
export function sessionFixture() {
  const sessions = [
    { id: "s-1", name: "重构缓存层", mode: "Agent", message_count: 3, wire_path: "/sessions/s-1.jsonl", created_at: 0, updated_at: 1700000000 },
    { id: "s-2", name: "调研 RAG", mode: "Plan", message_count: 1, wire_path: "/sessions/s-2.jsonl", created_at: 0, updated_at: 1600000000 },
  ];
  mockInvoke("neocodex_list_sessions", () => sessions);
  mockInvoke("neocodex_list_archived", () => [
    { id: "s-3", name: "旧会话", mode: "Agent", message_count: 5, wire_path: "/sessions/archived/s-3.jsonl", created_at: 0, updated_at: 1500000000 },
  ]);
  mockInvoke("neocodex_get_session_messages", () => [
    { id: 0, role: "user", content: "帮我重构缓存层", timestamp: 1700000000 },
    { id: 1, role: "assistant", content: "好的，我来分析缓存策略。", timestamp: 1700000001 },
  ]);
  mockInvoke("neocodex_get_side_chat", () => []);
  return sessions;
}

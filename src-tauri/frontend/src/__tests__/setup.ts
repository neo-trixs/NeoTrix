import "@testing-library/jest-dom";
import { vi } from "vitest";

Element.prototype.scrollIntoView = () => {};
Element.prototype.scrollBy = () => {};

class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
window.ResizeObserver = MockResizeObserver as any;

// jsdom in this vitest setup does not provide a working localStorage; install
// a small in-memory shim so components that persist UI prefs (e.g. pinned
// sessions) behave like the real webview.
const store = new Map<string, string>();
Object.defineProperty(window, "localStorage", {
  configurable: true,
  value: {
    getItem: (k: string) => store.get(k) ?? null,
    setItem: (k: string, v: string) => void store.set(k, String(v)),
    removeItem: (k: string) => void store.delete(k),
    clear: () => store.clear(),
  },
});

// Global Tauri mock. Components call `invoke` during render/mount; route to a
// per-command handler table set by tests via `mockInvoke` (from tauriMock).
// Unknown commands return safe defaults so interaction tests stay focused on
// UI behavior instead of backend wiring.
interface TauriTestGlobals {
  __invokeHandlers?: Record<string, (args: any) => any>;
}
const g = globalThis as TauriTestGlobals;
g.__invokeHandlers = g.__invokeHandlers ?? {};

const defaultResponses: Record<string, (args: any) => any> = {
  neocodex_list_sessions: () => [],
  neocodex_list_archived: () => [],
  neocodex_get_session_messages: () => [],
  neocodex_get_side_chat: () => [],
  neocodex_health_report: () => ({ context_usage: 0, turn_count: 0, tool_call_count: 0, tokens_used: 0, context_turns: 0, provider_count: 0, provider_resolvable: false }),
  neocodex_check_update: () => ({ current: "0.0.0", available: false, latest: "0.0.0", error: null }),
  neocodex_app_version: () => "0.18.0",
  neocodex_provider_config: () => ({ provider_count: 1, resolvable: true, active_model: "test", providers: [{ name: "test", model: "test", resolvable: true }] }),
  neocodex_search_files: () => [],
  cmd_diff_unstaged: () => [{ type: "added", content: "fn new()", line_start: 0 }],
  cmd_diff_staged: () => [],
  cmd_diff_file: () => [],
  cmd_diff_stage: () => ["a.rs"],
  cmd_diff_unstage: () => [],
  cmd_diff_commit: () => null,
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: any) => {
    const handlers = (globalThis as TauriTestGlobals).__invokeHandlers;
    const handler = handlers?.[cmd];
    const resolved = handler ? handler(args ?? {}) : (defaultResponses[cmd]?.(args ?? {}) ?? undefined);
    return Promise.resolve(resolved);
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("@tauri-apps/plugin-deep-link", () => ({
  getCurrent: vi.fn().mockResolvedValue([]),
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  readDir: vi.fn(() => Promise.resolve([])),
  readTextFile: vi.fn(() => Promise.resolve("")),
  writeTextFile: vi.fn(() => Promise.resolve()),
}));

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: vi.fn(() => Promise.resolve()),
}));

vi.mock("@tauri-apps/plugin-notification", () => ({
  sendNotification: vi.fn(() => Promise.resolve()),
}));

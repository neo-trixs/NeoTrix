import { describe, it, expect, beforeAll, vi } from "vitest";
import { getInvoke, invoke, isTauri, listen } from "../ipc";

describe("ipc.ts", () => {
  it("detects non-Tauri environment", () => {
    expect(isTauri()).toBe(false);
    expect(getInvoke()).toBeNull();
  });

  it("invoke throws outside Tauri", async () => {
    await expect(invoke("some_cmd")).rejects.toThrow(/IPC unavailable/);
  });

  it("listen throws outside Tauri", async () => {
    await expect(listen("ev", () => {})).rejects.toThrow(/listen unavailable/);
  });

  it("uses injected __TAURI_INTERNALS__ when present", async () => {
    const g = globalThis as Record<string, unknown>;
    const calls: string[] = [];
    g.__TAURI_INTERNALS__ = {
      invoke: async (cmd: string) => {
        calls.push(cmd);
        return { ok: true };
      },
    };
    expect(isTauri()).toBe(true);
    const res = await invoke("neocodex_app_version");
    expect(res).toEqual({ ok: true });
    expect(calls).toEqual(["neocodex_app_version"]);
    delete g.__TAURI_INTERNALS__;
  });

  it("listen wraps handler through transformCallback and normalizes payload", async () => {
    const g = globalThis as Record<string, unknown>;
    const captured: Array<{ event: string; handler: unknown; target: unknown }> = [];
    let cb: ((raw: unknown) => void) | null = null;
    g.__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, args: Record<string, unknown>) => {
        if (cmd === "plugin:event|listen") {
          captured.push(args as { event: string; handler: unknown; target: unknown });
          return () => {};
        }
        return null;
      },
      transformCallback: (fn: (raw: unknown) => void) => {
        cb = fn;
        return 42;
      },
    };
    const seen: unknown[] = [];
    await listen("test_event", (p: unknown) => seen.push(p));
    // handler arg must be a numeric callback id, NOT a JS function
    expect(captured[0].handler).toBe(42);
    expect(captured[0].event).toBe("test_event");
    // real Tauri shape: { event, id, payload }
    cb!({ event: "test_event", id: 1, payload: "hello" });
    // e2e fixtures mock shape: raw payload
    cb!("raw-payload");
    expect(seen).toEqual(["hello", "raw-payload"]);
    delete g.__TAURI_INTERNALS__;
  });
});

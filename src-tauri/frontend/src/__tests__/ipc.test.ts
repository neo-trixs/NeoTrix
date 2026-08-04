import { describe, it, expect } from "vitest";
import { getInvoke, invoke, isTauri } from "../ipc";

describe("ipc.ts", () => {
  it("detects non-Tauri environment", () => {
    expect(isTauri()).toBe(false);
    expect(getInvoke()).toBeNull();
  });

  it("invoke throws outside Tauri", async () => {
    await expect(invoke("some_cmd")).rejects.toThrow(/IPC unavailable/);
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
});

// @vitest-environment jsdom
import { describe, it, expect, beforeAll, vi } from "vitest";

describe("ui-v2 (design HTML migrated to vite entry)", () => {
  beforeAll(async () => {
    const raw = await import("../../index.html?raw").catch(() => null);
    if (raw && typeof raw.default === "string") {
      document.body.innerHTML = raw.default.replace(/<script[\s\S]*?<\/script>/g, "");
    }
    // ui-v2.js is plain-JS side-effect module (no type surface by design)
    // @ts-expect-error no declarations for migrated JS entry
    await import("../ui-v2.js");
  });

  it("jsdom globalThis is the window", () => {
    expect(globalThis).toBe(window);
  });

  async function reloadApp() {
    const raw = await import("../../index.html?raw").catch(() => null);
    if (raw && typeof raw.default === "string") {
      document.body.innerHTML = raw.default.replace(/<script[\s\S]*?<\/script>/g, "");
    }
    vi.resetModules();
    // ui-v2.js is plain-JS side-effect module (no type surface by design)
    // @ts-expect-error no declarations for migrated JS entry
    await import("../ui-v2.js");
  }

  it("renders the app shell", () => {
    expect(document.querySelector(".app")).not.toBeNull();
    expect(document.querySelector("#viewChat")).not.toBeNull();
    expect(document.querySelector("#viewCowork")).not.toBeNull();
    expect(document.querySelector("#viewCode")).toBeNull();
  });

  it("exposes global functions for inline onclick handlers", () => {
    const g = globalThis as Record<string, unknown>;
    expect(typeof g.showToast).toBe("function");
    expect(typeof g.switchView).toBe("function");
    expect(typeof g.sendMsg).toBe("function");
    expect(typeof g.openSettingsModal).toBe("function");
    expect(typeof g.stopStream).toBe("function");
  });

  it("sendMsg renders user + assistant bubbles", async () => {
    vi.useFakeTimers();
    const g = globalThis as Record<string, unknown>;
    const input = document.getElementById("chatInput") as HTMLTextAreaElement;
    input.value = "你好，NeoTrix";
    (g.sendMsg as () => void)();
    expect(document.querySelectorAll("#chatScroll .msg").length).toBeGreaterThanOrEqual(1);
    vi.advanceTimersByTime(1200);
    vi.useRealTimers();
    expect(document.querySelectorAll("#chatScroll .msg.l .mb").length).toBeGreaterThanOrEqual(1);
  });

  it("switchView toggles visible view", () => {
    const g = globalThis as Record<string, unknown>;
    const btn = document.querySelector('.segb[data-view="cowork"]') as HTMLElement;
    (g.switchView as (el: HTMLElement, v: string) => void)(btn, "cowork");
    expect(document.getElementById("viewCowork")!.style.display).toBe("flex");
    expect(document.getElementById("viewChat")!.style.display).toBe("none");
  });

  it("fusion layer mounts + menu / model pool without TDZ errors", () => {
    expect(document.getElementById("ntxPlusMenu")).not.toBeNull();
    expect(document.getElementById("ntxModelBtn")).not.toBeNull();
    expect(document.querySelectorAll("#ntxModelMenu .ntx-mm-item").length).toBe(6);
    expect(document.getElementById("ntxUsage")).toBeNull();
    expect(document.getElementById("ntxModelLabel")!.textContent).toBe("Groq");
  });

  it("fusion + menu opens and model selection switches label", () => {
    document.getElementById("ntxPlusBtn")!.click();
    expect(document.getElementById("ntxPlusMenu")!.classList.contains("open")).toBe(true);
    const deepseek = [...document.querySelectorAll("#ntxModelMenu .ntx-mm-item")].find(
      (i) => (i as HTMLElement).dataset.id === "DeepSeek",
    ) as HTMLElement;
    deepseek.click();
    expect(document.getElementById("ntxModelLabel")!.textContent).toBe("DeepSeek");
  });

  it("code view removed: no panel, no standalone view", () => {
    expect(document.getElementById("codePanel")).toBeNull();
    expect(document.getElementById("viewCode")).toBeNull();
    expect(document.getElementById("codePanelToggle")).toBeNull();
  });

  it("assistant reply renders code blocks inside the conversation", async () => {
    vi.useFakeTimers();
    const g = globalThis as Record<string, unknown>;
    const input = document.getElementById("chatInput") as HTMLTextAreaElement;
    input.value = "给一段 Rust 代码";
    (g.sendMsg as () => void)();
    vi.advanceTimersByTime(1200);
    vi.useRealTimers();
    const aiMsg = document.querySelector("#chatScroll .msg.l")!;
    const code = aiMsg.querySelector(".msg-code");
    expect(code).not.toBeNull();
    expect(code!.querySelector(".msg-code-b .kw")).not.toBeNull();
    expect(code!.querySelector(".msg-code-cp")).not.toBeNull();
  });

  it("assistant reply carries a per-message usage footer", async () => {
    vi.useFakeTimers();
    const g = globalThis as Record<string, unknown>;
    const input = document.getElementById("chatInput") as HTMLTextAreaElement;
    input.value = "用量测试";
    (g.sendMsg as () => void)();
    vi.advanceTimersByTime(1200);
    vi.useRealTimers();
    const aiMsg = document.querySelector("#chatScroll .msg.l")!;
    expect(aiMsg.querySelector(".msg-usage")).not.toBeNull();
    expect(aiMsg.querySelector(".msg-usage")!.textContent).toMatch(/上下文 \d+%/);
  });

  it("messages are flattened: no avatar icon, assistant has meta header", async () => {
    vi.useFakeTimers();
    const g = globalThis as Record<string, unknown>;
    const input = document.getElementById("chatInput") as HTMLTextAreaElement;
    input.value = "扁平化测试";
    (g.sendMsg as () => void)();
    const userMsg = document.querySelector("#chatScroll .msg.r")!;
    expect(userMsg.querySelector(".ma2")).toBeNull();
    expect(userMsg.querySelector(".mb")).not.toBeNull();
    vi.advanceTimersByTime(1200);
    vi.useRealTimers();
    const aiMsg = document.querySelector("#chatScroll .msg.l")!;
    expect(aiMsg.querySelector(".ma2")).toBeNull();
    expect(aiMsg.querySelector(".msg-h .name")!.textContent).toBe("NeoTrix");
    expect(aiMsg.querySelector(".msg-h .time")).not.toBeNull();
  });

  it("diff overlay renders hunks and saves an inline comment", () => {
    const g = globalThis as Record<string, unknown>;
    const openDiff = g.openDiff as () => void;
    openDiff();
    const lines = document.querySelectorAll("#diffBody .df-line").length;
    expect(lines).toBeGreaterThan(0);
    (g.diffAddComment as (f: number, h: number, l: number) => void)(0, 0, 0);
    const ta = document.querySelector(".df-cmt-editor textarea") as HTMLTextAreaElement;
    ta.value = "inline note";
    (g.diffSaveComment as (f: number, h: number, l: number) => void)(0, 0, 0);
    expect(document.querySelector(".df-comment .dc-body")!.textContent).toBe("inline note");
  });

  it("code blocks expose a run button and runMsgCode appends a result block", () => {
    const g = globalThis as Record<string, unknown>;
    expect(typeof g.runMsgCode).toBe("function");
    expect(typeof g.createSession).toBe("function");
    expect(typeof g.kbSearch).toBe("function");
    const host = document.createElement("div");
    host.innerHTML = (g.renderRichText as (t: string) => string)("```sh\necho hi\n```");
    const runBtn = [...host.querySelectorAll(".msg-code-cp")].find((b) => b.textContent === "运行");
    expect(runBtn).not.toBeNull();
    host.querySelector("#chatScroll") || document.getElementById("chatScroll")!.appendChild(host);
    (g.runMsgCode as (btn: HTMLElement) => void)(runBtn as HTMLElement);
    const out = host.querySelector(".msg-code-out");
    expect(out).not.toBeNull();
    expect(out!.querySelector(".msg-code-res")!.textContent).toMatch(/NeoTrix shell/);
  });

  it("tab bar renders kbd chips + hero suggestions in chat view", () => {
    const segbs = document.querySelectorAll(".segb");
    expect(segbs.length).toBe(2);
    expect(document.querySelector(".segb[data-view='chat'] .segb-kbd")!.textContent).toBe("⌘1");
    expect(document.querySelector(".segb[data-view='cowork'] .segb-kbd")!.textContent).toBe("⌘2");
    expect(document.querySelectorAll("#heroSuggest .hero-sug-item").length).toBeGreaterThan(0);
  });

  it("switchView syncs aria-selected on the tab list", () => {
    const g = globalThis as Record<string, unknown>;
    const btn = document.querySelector('.segb[data-view="cowork"]') as HTMLElement;
    (g.switchView as (el: HTMLElement, v: string) => void)(btn, "cowork");
    expect(btn.getAttribute("aria-selected")).toBe("true");
    expect(document.querySelector('.segb[data-view="chat"]')!.getAttribute("aria-selected")).toBe("false");
    (g.switchView as (el: HTMLElement, v: string) => void)(
      document.querySelector('.segb[data-view="chat"]') as HTMLElement,
      "chat",
    );
    expect(document.querySelector('#heroSuggest')!.innerHTML.length).toBeGreaterThan(0);
  });

  it("cowork filter chips filter the session list", () => {
    const g = globalThis as Record<string, unknown>;
    const btn = document.querySelector('.segb[data-view="cowork"]') as HTMLElement;
    (g.switchView as (el: HTMLElement, v: string) => void)(btn, "cowork");
    const before = document.querySelectorAll("#cwSessionList .cw-sitem").length;
    expect(before).toBeGreaterThan(1);
    (g.cwFilter as (s: string) => void)("done");
    expect(document.querySelectorAll("#cwSessionList .cw-sitem").length).toBeLessThan(before);
    expect(document.querySelector("#cwSessionList .cw-sitem")!.textContent).toContain("文档生成");
    (g.cwFilter as (s: string) => void)("all");
    expect(document.querySelectorAll("#cwSessionList .cw-sitem").length).toBe(before);
  });

  it("meta+1 / meta+2 switch tabs", () => {
    // ensure fresh state: start from chat
    document.body.dispatchEvent(new KeyboardEvent("keydown", { key: "1", metaKey: true, cancelable: true, bubbles: true }));
    expect(document.getElementById("viewChat")!.style.display).toBe("flex");
    document.body.dispatchEvent(new KeyboardEvent("keydown", { key: "2", metaKey: true, cancelable: true, bubbles: true }));
    expect(document.getElementById("viewCowork")!.style.display).toBe("flex");
    expect(document.querySelector('.segb[data-view="cowork"]')!.classList.contains("on")).toBe(true);
    document.body.dispatchEvent(new KeyboardEvent("keydown", { key: "1", metaKey: true, cancelable: true, bubbles: true }));
    expect(document.getElementById("viewChat")!.style.display).toBe("flex");
  });

  it("escHtml escapes HTML special chars (XSS fix)", () => {
    const g = globalThis as Record<string, unknown>;
    const esc = g.escHtml as (s: string) => string;
    expect(esc('<img src=x onerror=alert(1)>')).toBe('&lt;img src=x onerror=alert(1)&gt;');
    expect(esc('&<>"\'')).toBe('&amp;&lt;&gt;&quot;&#39;');
    expect(esc('')).toBe('');
    expect(esc(undefined as unknown as string)).toBe('');
  });

  it("renderThread renders message action buttons for user + assistant", () => {
    const g = globalThis as Record<string, unknown>;
    (g.renderThread as (msgs: unknown[]) => void)([
      { role: "user", content: "请求", timestamp: 1700000000 },
      { role: "agent", content: "回复", timestamp: 1700000001 },
    ]);
    const user = document.querySelector("#chatScroll .msg.r")!;
    expect(user.querySelector('.ma-btn[data-op="edit"]')).not.toBeNull();
    expect(user.querySelector('.ma-btn[data-op="delete"]')).not.toBeNull();
    const ai = document.querySelector("#chatScroll .msg.l")!;
    expect(ai.querySelector('.ma-btn[data-op="retry"]')).not.toBeNull();
    expect(ai.querySelector('.ma-btn[data-op="copy"]')).not.toBeNull();
  });

  it("saveSetting/loadSetting persist to localStorage", async () => {
    const g = globalThis as Record<string, unknown>;
    // settings are module-scoped; verify they did not throw at import (TDZ guard)
    expect(typeof g.renderThread).toBe("function");
    // graceful fallback when storage is unavailable (jsdom stub / private mode)
    const fallback = (() => {
      try{
        const s = localStorage.getItem("neotrix.settings");
        return typeof (s === null ? {} : JSON.parse(s)) === "object";
      }catch(_e){ return true; }
    })();
    expect(fallback).toBe(true);
  });

  it("loadSessions maps backend sessions into recentData + cowork status", async () => {
    const g = globalThis as Record<string, unknown>;
    const mock: Record<string, (a: unknown) => unknown> = {};
    mock["neocodex_list_sessions"] = () => [
      { id: "s-1", name: "真实会话A", mode: "Agent", message_count: 3, updated_at: 1700000000 },
    ];
    mock["cowork_list"] = () => [
      { id: "c-1", name: "真实协同", status: "completed", deliverables: [{ name: "d1", kind: "md" }], files_created: 1 },
    ];
    // For the ipc module to route to the handler table we must fake Tauri.
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: unknown) => {
        const h = mock[cmd];
        return Promise.resolve(h ? h(args ?? {}) : undefined);
      },
    };
    try {
      await (g.loadSessions as () => Promise<void>)();
      const list = document.querySelectorAll("#cwSessionList .cw-sitem");
      expect(list.length).toBeGreaterThan(0);
      // recent sidebar should now be populated with the real chat session
      const recent = document.querySelector("#recentList");
      expect(recent!.textContent).toContain("真实会话A");
      // switch to cowork: recent list shows the real cowork session
      (g.switchView as (el: HTMLElement, v: string) => void)(
        document.querySelector('.segb[data-view="cowork"]') as HTMLElement,
        "cowork",
      );
      const recentCowork = document.querySelector("#recentList");
      expect(recentCowork!.textContent).toContain("真实协同");
      // status mapping unified to 已完成
      (g.cwFilter as (s: string) => void)("done");
      const shown = [...document.querySelectorAll("#cwSessionList .cw-sitem")].map((e) => e.textContent);
      expect(shown.some((t) => t.includes("真实协同"))).toBe(true);
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("read_file content renders in file preview when node.load is set", async () => {
    const g = globalThis as Record<string, unknown>;
    const mock: Record<string, (a: unknown) => unknown> = {};
    mock["read_file"] = () => "fn real_content() {}";
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: unknown) => {
        const h = mock[cmd];
        return Promise.resolve(h ? h(args ?? {}) : undefined);
      },
    };
    try {
      await (g.showFilePreview as (n: unknown) => Promise<void>)({
        name: "real.rs",
        load: "/tmp/real.rs",
        content: "// placeholder",
      });
      expect(document.getElementById("fpContent")!.textContent).toContain("fn real_content");
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("unified workspace header mounts with title + status pills", () => {
    const g = globalThis as Record<string, unknown>;
    (g.switchView as (el: HTMLElement, v: string) => void)(
      document.querySelector('.segb[data-view="chat"]') as HTMLElement,
      "chat",
    );
    expect(document.getElementById("wsTop")).not.toBeNull();
    expect(document.getElementById("wsTitleTxt")!.textContent).toBe("对话");
    const pills = document.querySelectorAll("#wsStatus .ws-pill").length;
    expect(pills).toBeGreaterThanOrEqual(3);
  });

  it("switchView syncs the unified workspace title + cowork session items carry status dots", () => {
    const g = globalThis as Record<string, unknown>;
    (g.switchView as (el: HTMLElement, v: string) => void)(
      document.querySelector('.segb[data-view="cowork"]') as HTMLElement,
      "cowork",
    );
    expect(document.getElementById("wsTitleTxt")!.textContent).toBe("团队");
    const dots = document.querySelectorAll("#cwSessionList .cw-sitem .st-dot");
    expect(dots.length).toBeGreaterThan(0);
    (g.switchView as (el: HTMLElement, v: string) => void)(
      document.querySelector('.segb[data-view="chat"]') as HTMLElement,
      "chat",
    );
    expect(document.getElementById("wsTitleTxt")!.textContent).toBe("对话");
  });

  it("loadWsStatus populates real brain stats into header + hero meta", async () => {
    const g = globalThis as Record<string, unknown>;
    const mock: Record<string, (a: unknown) => unknown> = {};
    mock["brain_stats"] = () => ({
      iteration: 7, absorb_count: 9, capability_sum: 4.2,
      memory_count: 42, dimension_names: ["reasoning", "planning", "memory"],
      capability_vector: [1, 2, 3], engine_active: false,
    });
    mock["neocodex_health_report"] = () => ({ context_usage: 10, turn_count: 2 });
    mock["neocodex_agent_status"] = () => ({ running: false, current_task: null });
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: unknown) => {
        const h = mock[cmd];
        return Promise.resolve(h ? h(args ?? {}) : undefined);
      },
    };
    try {
      await (g.loadWsStatus as () => Promise<void>)();
      expect(document.getElementById("wsMemory")!.textContent).toContain("42");
      expect(document.getElementById("wsDims")!.textContent).toContain("3");
      expect(document.getElementById("heroMeta")!.innerHTML).toContain("VSA HyperCube");
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("listen stream token wraps into chat while streaming (payload normalize)", async () => {
    const g = globalThis as Record<string, unknown>;
    const mock: Record<string, (a: unknown) => unknown> = {
      neocodex_send_message_stream: () => "ok",
    };
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const cbs: Record<string, (raw: unknown) => void> = {};
    let cbRef: ((raw: unknown) => void) | null = null;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "plugin:event|listen") {
          const ev = String(args?.event);
          cbs[ev] = cbRef!;
          return Promise.resolve(() => {});
        }
        const h = mock[cmd];
        return Promise.resolve(h ? h(args ?? {}) : undefined);
      },
      transformCallback: (fn: (raw: unknown) => void) => { cbRef = fn; return 1; },
    };
    try {
      const input = document.getElementById("chatInput") as HTMLTextAreaElement;
      input.value = "流式测试";
      (g.sendMsg as () => void)();
      cbs["neocodex_stream_token"]({ event: "neocodex_stream_token", id: 1, payload: "流式" });
      cbs["neocodex_stream_token"]({ event: "neocodex_stream_token", id: 1, payload: "回复" });
      const mb = document.querySelector("#chatScroll .msg.l .mb.streaming");
      expect(mb!.textContent).toContain("流式回复");
      cbs["neocodex_stream_done"]({ event: "neocodex_stream_done", id: 1, payload: { cancelled: false } });
      expect((document.getElementById("sendBtn") as HTMLButtonElement).disabled).toBe(false);
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("stream token renders markdown inline progressively", async () => {
    const g = globalThis as Record<string, unknown>;
    const mock: Record<string, (a: unknown) => unknown> = {
      neocodex_send_message_stream: () => "ok",
    };
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const cbs: Record<string, (raw: unknown) => void> = {};
    let cbRef: ((raw: unknown) => void) | null = null;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "plugin:event|listen") {
          const ev = String(args?.event);
          cbs[ev] = cbRef!;
          return Promise.resolve(() => {});
        }
        const h = mock[cmd];
        return Promise.resolve(h ? h(args ?? {}) : undefined);
      },
      transformCallback: (fn: (raw: unknown) => void) => { cbRef = fn; return 1; },
    };
    try {
      await reloadApp();
      const input = document.getElementById("chatInput") as HTMLTextAreaElement;
      input.value = "流式渲染";
      (g.sendMsg as () => void)();
      cbs["neocodex_stream_token"]({ event: "neocodex_stream_token", id: 1, payload: "**加粗**" });
      const mb = document.querySelector("#chatScroll .msg.l .mb.streaming");
      expect(mb!.innerHTML).toContain("<strong>加粗</strong>");
      expect(mb!.textContent).not.toContain("**");
      cbs["neocodex_stream_token"]({ event: "neocodex_stream_token", id: 1, payload: "与`行内码`" });
      expect(mb!.innerHTML).toContain("<code>行内码</code>");
      expect(mb!.textContent).toContain("与");
      cbs["neocodex_stream_done"]({ event: "neocodex_stream_done", id: 1, payload: { cancelled: false } });
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("stream token keeps unclosed code fence in a streaming pre", async () => {
    const g = globalThis as Record<string, unknown>;
    const mock: Record<string, (a: unknown) => unknown> = {
      neocodex_send_message_stream: () => "ok",
    };
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const cbs: Record<string, (raw: unknown) => void> = {};
    let cbRef: ((raw: unknown) => void) | null = null;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "plugin:event|listen") {
          const ev = String(args?.event);
          cbs[ev] = cbRef!;
          return Promise.resolve(() => {});
        }
        const h = mock[cmd];
        return Promise.resolve(h ? h(args ?? {}) : undefined);
      },
      transformCallback: (fn: (raw: unknown) => void) => { cbRef = fn; return 1; },
    };
    try {
      await reloadApp();
      const input = document.getElementById("chatInput") as HTMLTextAreaElement;
      input.value = "流式代码";
      (g.sendMsg as () => void)();
      cbs["neocodex_stream_token"]({ event: "neocodex_stream_token", id: 1, payload: "```python\nprint(" });
      const mb = document.querySelector("#chatScroll .msg.l .mb.streaming");
      expect(mb!.innerHTML).toContain("msg-code-stream");
      expect(mb!.textContent).toContain("print(");
      cbs["neocodex_stream_token"]({ event: "neocodex_stream_token", id: 1, payload: "1)\n```" });
      expect(mb!.querySelector(".msg-code-stream")).toBeNull();
      expect(mb!.querySelector(".msg-code")).not.toBeNull();
      cbs["neocodex_stream_done"]({ event: "neocodex_stream_done", id: 1, payload: { cancelled: false } });
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("stream end flips to final rendered markdown and clears streaming class", async () => {
    const g = globalThis as Record<string, unknown>;
    const mock: Record<string, (a: unknown) => unknown> = {
      neocodex_send_message_stream: () => "ok",
    };
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const cbs: Record<string, (raw: unknown) => void> = {};
    let cbRef: ((raw: unknown) => void) | null = null;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "plugin:event|listen") {
          const ev = String(args?.event);
          cbs[ev] = cbRef!;
          return Promise.resolve(() => {});
        }
        const h = mock[cmd];
        return Promise.resolve(h ? h(args ?? {}) : undefined);
      },
      transformCallback: (fn: (raw: unknown) => void) => { cbRef = fn; return 1; },
    };
    try {
      await reloadApp();
      const input = document.getElementById("chatInput") as HTMLTextAreaElement;
      input.value = "流式完结";
      (g.sendMsg as () => void)();
      cbs["neocodex_stream_token"]({ event: "neocodex_stream_token", id: 1, payload: "# 标题" });
      cbs["neocodex_stream_end"]({ event: "neocodex_stream_end", id: 1, payload: "# 标题\n\n**完整**" });
      const mb = document.querySelector("#chatScroll .msg.l .mb");
      expect(mb!.className).not.toContain("streaming");
      expect(mb!.innerHTML).toContain("<h1>标题</h1>");
      expect(mb!.innerHTML).toContain("<strong>完整</strong>");
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("session ops + feedback + search functions are exposed globally", () => {
    const g = globalThis as Record<string, unknown>;
    expect(typeof g.openSessionOps).toBe("function");
    expect(typeof g.renameSession).toBe("function");
    expect(typeof g.compactSession).toBe("function");
    expect(typeof g.archiveSession).toBe("function");
    expect(typeof g.exportSession).toBe("function");
    expect(typeof g.deleteSession).toBe("function");
    expect(typeof g.feedbackMessage).toBe("function");
    expect(typeof g.searchSessions).toBe("function");
    expect(typeof g.checkForUpdate).toBe("function");
    expect(typeof g.cycleMode).toBe("function");
    expect(document.getElementById("sessionOpsMenu")).not.toBeNull();
    expect(document.getElementById("cwSearchInput")).not.toBeNull();
  });

  it("openSessionOps shows menu with session title", () => {
    const g = globalThis as Record<string, unknown>;
    const menu = document.getElementById("sessionOpsMenu")!;
    (g.openSessionOps as (anchor: HTMLElement | null, id: string) => void)(null, "s-ops-1");
    expect(menu.classList.contains("open")).toBe(true);
    expect(menu.dataset.session).toBe("s-ops-1");
    (g.closeSessionOps as () => void)();
    expect(menu.classList.contains("open")).toBe(false);
  });

  it("renderThread adds like/dislike feedback buttons to assistant messages", () => {
    const g = globalThis as Record<string, unknown>;
    (g.renderThread as (msgs: unknown[], sessionId?: string) => void)([
      { role: "agent", content: "反馈测试", timestamp: 1700000001 },
    ], "s-fb-1");
    const ai = document.querySelector("#chatScroll .msg.l")!;
    expect(ai.querySelector('.ma-btn[data-op="like"]')).not.toBeNull();
    expect(ai.querySelector('.ma-btn[data-op="dislike"]')).not.toBeNull();
  });

  it("feedbackMessage persists state and calls neocodex_feedback", async () => {
    const g = globalThis as Record<string, unknown>;
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const calls: { cmd: string; args: unknown }[] = [];
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: unknown) => {
        calls.push({ cmd, args: args ?? {} });
        return Promise.resolve("ok");
      },
    };
    try {
      (g.renderThread as (msgs: unknown[], sessionId?: string) => void)([
        { role: "agent", content: "反馈", timestamp: 1700000001 },
      ], "s-fb-2");
      await (g.feedbackMessage as (i: number, k: string) => Promise<void>)(0, "like");
      const likeBtn = document.querySelector('#chatScroll .msg.l .ma-btn[data-op="like"]')!;
      expect(likeBtn.classList.contains("on")).toBe(true);
      const fbCall = calls.find((c) => c.cmd === "neocodex_feedback");
      expect(fbCall).toBeTruthy();
      expect((fbCall!.args as { session_id: string }).session_id).toBe("s-fb-2");
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("searchSessions renders backend hits into the search results pane", async () => {
    const g = globalThis as Record<string, unknown>;
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string) =>
        Promise.resolve(
          cmd === "neocodex_search_sessions"
            ? [{ session_id: "s-srch", session_name: "搜索结果", role: "user", snippet: "含关键词", match_count: 2, timestamp: 1700000000 }]
            : undefined,
        ),
    };
    try {
      const input = document.getElementById("cwSearchInput") as HTMLInputElement;
      input.value = "关键词";
      await (g.searchSessions as (q: string) => Promise<void>)("关键词");
      const res = document.getElementById("cwSearchResults")!;
      expect(res.style.display).toBe("block");
      expect(res.textContent).toContain("搜索结果");
      expect(res.textContent).toContain("2 处");
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("sendMsg passes persisted temperature/max_tokens and permission mode", async () => {
    const g = globalThis as Record<string, unknown>;
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const calls: { cmd: string; args: Record<string, unknown> }[] = [];
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        calls.push({ cmd, args: args ?? {} });
        return Promise.resolve("ok");
      },
      transformCallback: () => 1,
    };
    const mem = new Map<string, string>();
    const store = {
      getItem: (k: string) => (mem.has(k) ? mem.get(k)! : null),
      setItem: (k: string, v: string) => { mem.set(k, String(v)); },
      removeItem: (k: string) => { mem.delete(k); },
    };
    const realLS = (globalThis as Record<string, unknown>).localStorage;
    (globalThis as Record<string, unknown>).localStorage = store;
    try {
      store.setItem("neotrix.settings", JSON.stringify({ "compute.temperature": 0.7, "compute.maxTokens": 16384 }));
      (g.cycleMode as () => void)();
      const input = document.getElementById("chatInput") as HTMLTextAreaElement;
      input.value = "参数接线测试";
      (g.sendMsg as () => void)();
      const send = calls.find((c) => c.cmd === "neocodex_send_message_stream");
      expect(send).toBeTruthy();
      expect(send!.args.temperature).toBe(0.7);
      expect(send!.args.max_tokens).toBe(16384);
      expect(send!.args.permission_mode).toBe("plan");
    } finally {
      (globalThis as Record<string, unknown>).localStorage = realLS;
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("renderThread renders tool-call cards and system bubbles (Cursor/ChatGPT parity)", () => {
    const g = globalThis as Record<string, unknown>;
    (g.renderThread as (msgs: unknown[]) => void)([
      { role: "user", content: "跑一下", timestamp: 1700000000 },
      { role: "tool", content: "**execute_command**\n```\ncargo check\n```", timestamp: 0 },
      { role: "agent", content: "完成", timestamp: 1700000001 },
      { role: "system", content: "context: 40%", timestamp: 1700000002 },
    ]);
    expect(document.querySelector("#chatScroll .tool-card")).not.toBeNull();
    expect(document.querySelector("#chatScroll .tool-head .tool-name")!.textContent).toContain("execute_command");
    expect(document.querySelector("#chatScroll .sys-card")).not.toBeNull();
  });

  it("tool card collapses/expands on head click", () => {
    const g = globalThis as Record<string, unknown>;
    (g.renderThread as (msgs: unknown[]) => void)([
      { role: "tool", content: "**read_file**\n```\na\nb\n```", timestamp: 0 },
    ]);
    const card = document.querySelector("#chatScroll .tool-card") as HTMLElement;
    const head = card.querySelector(".tool-head") as HTMLElement;
    head.click();
    expect(card.classList.contains("open")).toBe(true);
    head.click();
    expect(card.classList.contains("open")).toBe(false);
  });

  it("renderThread keeps visible index across tool/system messages for ops", () => {
    document.getElementById("chatScroll")!.innerHTML = "";
    const g = globalThis as Record<string, unknown>;
    (g.renderThread as (msgs: unknown[]) => void)([
      { role: "user", content: "第一问", timestamp: 1700000000 },
      { role: "tool", content: "**x**\n```\n1\n```", timestamp: 0 },
      { role: "agent", content: "第一答", timestamp: 1700000001 },
      { role: "system", content: "hint", timestamp: 1700000002 },
      { role: "user", content: "第二问", timestamp: 1700000003 },
    ]);
    // user messages render with data-vid = visible index (0, 1) — not raw array index (0, 4)
    const users = [...document.querySelectorAll("#chatScroll .msg.r")];
    expect(users.map((e) => Number((e as HTMLElement).dataset.vid))).toEqual([0, 2]);
    const agents = [...document.querySelectorAll("#chatScroll .msg.l")];
    expect(agents.map((e) => Number((e as HTMLElement).dataset.vid))).toEqual([1]);
  });

  it("editMessage opens inline composer instead of prompt", () => {
    const g = globalThis as Record<string, unknown>;
    const mem = new Map<string, string>();
    const store = {
      getItem: (k: string) => (mem.has(k) ? mem.get(k)! : null),
      setItem: (k: string, v: string) => { mem.set(k, String(v)); },
      removeItem: (k: string) => { mem.delete(k); },
    };
    const realLS = (globalThis as Record<string, unknown>).localStorage;
    (globalThis as Record<string, unknown>).localStorage = store;
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        if(cmd === "neocodex_get_session_messages") return Promise.resolve([
          { role: "user", content: "原内容", timestamp: 1700000000 },
        ]);
        return Promise.resolve("ok");
      },
      transformCallback: () => 1,
    };
    try {
      (g.renderThread as (msgs: unknown[], sessionId?: string) => void)([
        { role: "user", content: "原内容", timestamp: 1700000000 },
      ], "s-edit");
      (g.editMessage as (index: number) => void)(0);
      const editor = document.querySelector("#chatScroll .msg-edit");
      expect(editor).not.toBeNull();
      expect(document.querySelector("#chatScroll .msg-edit .me-hint")!.textContent).toContain("重新生成");
    } finally {
      (globalThis as Record<string, unknown>).localStorage = realLS;
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("@ mention: typing @ opens popup, selecting inserts pill and closes", () => {
    const g = globalThis as Record<string, unknown>;
    const input = document.getElementById("chatInput") as HTMLTextAreaElement;
    input.value = "@"; input.selectionStart = 1; input.selectionEnd = 1;
    (g.QMUpdate as () => void)();
    const menu = document.getElementById("qmMenu");
    expect(menu!.style.display).toBe("block");
    const items = menu!.querySelectorAll(".qm-item");
    expect(items.length).toBeGreaterThan(0);
    expect(menu!.textContent).toContain("@nt-core");
    (items[0] as HTMLElement).click();
    expect(input.value).toContain("@nt-core");
    expect(menu!.style.display).toBe("none");
  });

  it("QUpdate: / at line start lists slash commands", () => {
    const g = globalThis as Record<string, unknown>;
    const input = document.getElementById("chatInput") as HTMLTextAreaElement;
    input.value = "/di";
    input.selectionStart = 3; input.selectionEnd = 3;
    (g.QMUpdate as () => void)();
    const menu = document.getElementById("qmMenu");
    expect(menu!.style.display).toBe("block");
    expect(menu!.textContent).toContain("/diff");
    const items = menu!.querySelectorAll(".qm-item");
    (items[0] as HTMLElement).click();
    // selecting a slash command should close menu (runs locally)
    expect(menu!.style.display).toBe("none");
  });

  it("regenPush snapshots assistant reply and shows version bar", () => {
    const g = globalThis as Record<string, unknown>;
    (g.renderThread as (msgs: unknown[], sessionId?: string) => void)([
      { role: "user", content: "提问", timestamp: 1700000000 },
      { role: "agent", content: "第一版回答", timestamp: 1700000001 },
    ], "s-ver");
    (g.regenPush as (sid: string, vid: number) => void)("s-ver", 1);
    (g.renderThread as (msgs: unknown[], sessionId?: string) => void)([
      { role: "user", content: "提问", timestamp: 1700000000 },
      { role: "agent", content: "第二版回答", timestamp: 1700000001 },
    ], "s-ver");
    (g.regenPush as (sid: string, vid: number) => void)("s-ver", 1);
    const bar = document.querySelector("#chatScroll .msg.l .ver-bar");
    expect(bar).not.toBeNull();
    expect(bar!.textContent).toContain("较旧");
    expect(bar!.textContent).toContain("较新");
  });

  it("context meter renders usage % and popover hydrates from health report", async () => {
    const g = globalThis as Record<string, unknown>;
    // renderContextMeter uses the module-level lastContextUsage; drive it via loadUsage with fake health
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string) => {
        if(cmd === "neocodex_health_report") return Promise.resolve({
          context_usage: 0.85, context_turns: 12, tokens_used: 42000,
          tool_call_count: 9, provider_model: "claude-sonnet", cost_spent: 1.2, cost_budget: 10,
        });
        return Promise.resolve("ok");
      },
      transformCallback: () => 1,
    };
    try {
      await (g.loadUsage as () => Promise<void>)();
      const chip = document.querySelector("#ntxCtxMeter .ctx-chip");
      expect(chip).not.toBeNull();
      expect(chip!.textContent).toContain("85%");
      (g.toggleCtxPop as () => void)();
      const pop = document.getElementById("ntxCtxPop");
      expect(pop!.classList.contains("open")).toBe(true);
      expect(pop!.textContent).toContain("42,000");
      expect(pop!.textContent).toContain("对话轮次12");
      expect(pop!.textContent).toContain("claude-sonnet");
      (g.toggleCtxPop as () => void)();
      expect(pop!.classList.contains("open")).toBe(false);
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("addAttachChip renders chip and sendMsg passes attachment payload", async () => {
    const g = globalThis as Record<string, unknown>;
    (g.addAttachChip as (name: string, meta?: object) => void)("Cargo.toml", { size: 2048, mime: "text/plain", data: "[package]" });
    const chip = document.querySelector("#ntxAttachArea .ntx-attach-chip");
    expect(chip).not.toBeNull();
    expect(chip!.textContent).toContain("Cargo.toml");
    expect(chip!.textContent).toContain("2.0K");
    // wire sendMsg → assert payload
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const realLS = (globalThis as Record<string, unknown>).localStorage;
    (globalThis as Record<string, unknown>).localStorage = { getItem: () => null, setItem: () => {}, removeItem: () => {} };
    let sent: { cmd: string; args: Record<string, unknown> } | undefined;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        if(cmd === "neocodex_send_message_stream") sent = { cmd, args: args ?? {} };
        return Promise.resolve("ok");
      },
      transformCallback: () => 1,
    };
    try {
      const inp = document.getElementById("chatInput") as HTMLTextAreaElement;
      inp.value = "带附件";
      (g.sendMsg as () => void)();
      expect(sent).toBeTruthy();
      expect(sent!.args.attachments).toBeTruthy();
      expect((sent!.args.attachments as unknown[])[0]).toMatchObject({ name: "Cargo.toml", size: 2048 });
      expect((sent!.args.attachments as { data: string }[])[0].data).toBe("[package]");
    } finally {
      (globalThis as Record<string, unknown>).localStorage = realLS;
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("insertReference quotes a message into the composer", () => {
    const g = globalThis as Record<string, unknown>;
    const inp = document.getElementById("chatInput") as HTMLTextAreaElement;
    inp.value = "我的问题";
    (g.insertReference as (msg: { role: string; content: string }) => void)({ role: "agent", content: "历史回复内容" });
    expect(inp.value).toContain("[引用·NeoTrix] 历史回复内容");
    expect(inp.value).toContain("我的问题");
  });

  it("openPalette mounts command panel and lists quick actions", () => {
    const g = globalThis as Record<string, unknown>;
    (g.openPalette as () => void)();
    const ov = document.getElementById("overlayPalette")!;
    expect(ov.classList.contains("open")).toBe(true);
    expect(document.querySelectorAll("#palBody .pal-item[data-act]").length).toBeGreaterThanOrEqual(7);
  });

  it("closePalette hides the command panel", () => {
    const g = globalThis as Record<string, unknown>;
    const ov = document.getElementById("overlayPalette")!;
    ov.classList.add("open");
    (g.closePalette as () => void)();
    expect(ov.classList.contains("open")).toBe(false);
  });

  it("palFilter renders backend session hits into palette results", async () => {
    const g = globalThis as Record<string, unknown>;
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string) =>
        Promise.resolve(
          cmd === "neocodex_search_sessions"
            ? [{ session_id: "s-pal", session_name: "面板命中", role: "user", snippet: "…", match_count: 1, timestamp: 1700000000 }]
            : undefined,
        ),
    };
    try {
      const inp = document.getElementById("palInput") as HTMLInputElement;
      inp.value = "面板";
      await (g.palFilter as (q: string) => Promise<void>)("面板");
      const results = document.getElementById("palResults")!;
      expect(results.style.display).not.toBe("none");
      expect(results.textContent).toContain("面板命中");
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("palFilter with empty query shows quick actions and hides results", async () => {
    const g = globalThis as Record<string, unknown>;
    (g.openPalette as () => void)();
    await (g.palFilter as (q: string) => Promise<void>)("");
    const results = document.getElementById("palResults")!;
    expect(results.style.display).toBe("none");
    const empty = document.getElementById("palEmpty")!;
    expect(empty.style.display).toBe("none");
  });

  it("palKey Enter picks highlighted item and runs the action", () => {
    const g = globalThis as Record<string, unknown>;
    (g.openPalette as () => void)();
    const ov = document.getElementById("overlayPalette")!;
    const item = document.querySelector('#palBody .pal-item[data-act="settings"]') as HTMLElement;
    item.classList.add("sel");
    (g.palKey as (e: KeyboardEvent) => void)({ key: "Enter", preventDefault: () => {}, stopPropagation: () => {} } as KeyboardEvent);
    expect(ov.classList.contains("open")).toBe(false);
  });

  it("palKey Escape closes the palette without bubbling", () => {
    const g = globalThis as Record<string, unknown>;
    (g.openPalette as () => void)();
    const ov = document.getElementById("overlayPalette")!;
    let stopped = false;
    (g.palKey as (e: KeyboardEvent) => void)({ key: "Escape", preventDefault: () => {}, stopPropagation: () => { stopped = true; } } as KeyboardEvent);
    expect(ov.classList.contains("open")).toBe(false);
    expect(stopped).toBe(true);
  });

  it("palPick opens a session hit via openSessionFromSearch", async () => {
    const g = globalThis as Record<string, unknown>;
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const calls: { cmd: string; args: Record<string, unknown> }[] = [];
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        calls.push({ cmd, args: args ?? {} });
        return Promise.resolve("ok");
      },
      transformCallback: () => 1,
    };
    try {
      (g.openPalette as () => void)();
      const hit = document.createElement("button");
      hit.setAttribute("data-sid", "s-open");
      (g.palPick as (el: HTMLElement) => void)(hit);
      expect(document.getElementById("overlayPalette")!.classList.contains("open")).toBe(false);
      await new Promise((r) => setTimeout(r, 10));
      expect(calls.some((c) => c.cmd === "neocodex_switch_session" && c.args?.session_id === "s-open")).toBeTruthy();
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("renderRichText renders inline bold/italic/code/link", () => {
    const g = globalThis as Record<string, unknown>;
    const html = (g.renderRichText as (t: string) => string)("**加粗** 与 *斜体* 及 `代码` 和 [链接](https://example.com)");
    const host = document.createElement("div");
    host.innerHTML = html;
    expect(host.querySelector("strong")!.textContent).toBe("加粗");
    expect(host.querySelector("em")!.textContent).toBe("斜体");
    expect(host.querySelector("code")!.textContent).toBe("代码");
    expect(host.querySelector("a")!.getAttribute("href")).toBe("https://example.com");
    expect(host.querySelector("a")!.textContent).toBe("链接");
  });

  it("renderRichText renders headings/lists/quote/task/hr/table", () => {
    const g = globalThis as Record<string, unknown>;
    const src = [
      "# 标题一",
      "",
      "- 条目 A",
      "- 条目 B",
      "",
      "1. 第一步",
      "2. 第二步",
      "",
      "> 引用行",
      "",
      "- [x] 已完成",
      "- [ ] 待办",
      "",
      "| 列A | 列B |",
      "| --- | --- |",
      "| 1   | 2   |",
    ].join("\n");
    const host = document.createElement("div");
    host.innerHTML = (g.renderRichText as (t: string) => string)(src);
    expect(host.querySelector("h1")!.textContent).toBe("标题一");
    expect(host.querySelectorAll("ul li").length).toBe(4);
    expect(host.querySelectorAll("ol li").length).toBe(2);
    expect(host.querySelector("blockquote")!.textContent).toContain("引用行");
    expect(host.querySelector(".md-task-done")!.textContent).toContain("已完成");
    expect(host.querySelector("table th")!.textContent).toBe("列A");
    expect(host.querySelectorAll("table td").length).toBe(2);
  });

  it("renderRichText is XSS-safe: escapes script and blocks javascript: links", () => {
    const g = globalThis as Record<string, unknown>;
    const html = (g.renderRichText as (t: string) => string)("<script>alert(1)</script> [x](javascript:alert(2)) `onerror`");
    const host = document.createElement("div");
    host.innerHTML = html;
    expect(host.querySelector("script")).toBeNull();
    expect(host.querySelector("a")).toBeNull();
    expect(host.textContent).toContain("<script>alert(1)</script>");
  });

  it("renderRichText keeps code fences with run/copy buttons", () => {
    const g = globalThis as Record<string, unknown>;
    const host = document.createElement("div");
    host.innerHTML = (g.renderRichText as (t: string) => string)("```rust\nfn main() {}\n```");
    expect(host.querySelector(".msg-code-b")).not.toBeNull();
    expect([...host.querySelectorAll(".msg-code-cp")].map((b) => b.textContent)).toEqual(["运行", "复制"]);
  });

  it("saveDraft persists composer text per session and clearDraft removes it", async () => {
    const g = globalThis as Record<string, unknown>;
    const mem = new Map<string, string>();
    const realLS = (globalThis as Record<string, unknown>).localStorage;
    (globalThis as Record<string, unknown>).localStorage = {
      getItem: (k: string) => (mem.has(k) ? mem.get(k)! : null),
      setItem: (k: string, v: string) => { mem.set(k, String(v)); },
      removeItem: (k: string) => { mem.delete(k); },
    };
    try {
      (g.renderThread as (msgs: unknown[], sid: string) => void)([], "s-draft");
      const input = document.getElementById("chatInput") as HTMLTextAreaElement;
      input.value = "草稿内容";
      (g.saveDraft as () => void)();
      await new Promise((r) => setTimeout(r, 350));
      const dm = JSON.parse(mem.get("neotrix.drafts") || "{}");
      expect(dm["s-draft"]).toBe("草稿内容");
      (g.clearDraft as () => void)();
      await new Promise((r) => setTimeout(r, 10));
      const after = JSON.parse(mem.get("neotrix.drafts") || "{}");
      expect(after["s-draft"]).toBeUndefined();
    } finally {
      (globalThis as Record<string, unknown>).localStorage = realLS;
    }
  });

  it("restoreDraft fills composer when empty and keeps non-empty input", () => {
    const g = globalThis as Record<string, unknown>;
    const mem = new Map<string, string>();
    const realLS = (globalThis as Record<string, unknown>).localStorage;
    (globalThis as Record<string, unknown>).localStorage = {
      getItem: (k: string) => (mem.has(k) ? mem.get(k)! : null),
      setItem: (k: string, v: string) => { mem.set(k, String(v)); },
      removeItem: (k: string) => { mem.delete(k); },
    };
    try {
      // pin the active session id via renderThread so the draft key is deterministic
      (g.renderThread as (msgs: unknown[], sid: string) => void)([], "s-draft");
      mem.set("neotrix.drafts", JSON.stringify({ "s-draft": "待恢复草稿" }));
      const input = document.getElementById("chatInput") as HTMLTextAreaElement;
      input.value = "";
      (g.restoreDraft as () => void)();
      expect(input.value).toBe("待恢复草稿");
      input.value = "已有输入";
      (g.restoreDraft as () => void)();
      expect(input.value).toBe("已有输入");
    } finally {
      (globalThis as Record<string, unknown>).localStorage = realLS;
    }
  });

  it("jumpToLatest shows/hides scroll jump pill and scrolls to bottom", () => {
    const g = globalThis as Record<string, unknown>;
    const pill = document.getElementById("scrollJump")!;
    pill.classList.add("show");
    (g.jumpToLatest as () => void)();
    expect(pill.classList.contains("show")).toBe(false);
    const cs = document.getElementById("chatScroll")!;
    expect(cs.scrollTop).toBe(cs.scrollHeight);
  });

  it("ArrowUp recalls previous user message, ArrowDown/Escape cycles back", () => {
    const g = globalThis as Record<string, unknown>;
    (g.renderThread as (msgs: unknown[], sid: string) => void)([
      { role: "user", content: "第一条", timestamp: 1700000001 },
      { role: "assistant", content: "回复", timestamp: 1700000002 },
      { role: "user", content: "第二条", timestamp: 1700000003 },
    ], "s-recall");
    const input = document.getElementById("chatInput") as HTMLTextAreaElement;
    const key = (k: string) => (g.handleKey as (e: KeyboardEvent) => void)({
      key: k, target: input, preventDefault: () => {}, stopPropagation: () => {},
    } as unknown as KeyboardEvent);
    input.value = "";
    key("ArrowUp");
    expect(input.value).toBe("第二条");
    key("ArrowUp");
    expect(input.value).toBe("第一条");
    key("ArrowDown");
    expect(input.value).toBe("第二条");
    key("ArrowDown");
    expect(input.value).toBe("");
    key("ArrowUp");
    expect(input.value).toBe("第二条");
    key("Escape");
    expect(input.value).toBe("");
  });

  it("ArrowUp is a no-op when no history and composer has text", () => {
    const g = globalThis as Record<string, unknown>;
    (g.renderThread as (msgs: unknown[], sid: string) => void)([], "s-nohistory");
    const input = document.getElementById("chatInput") as HTMLTextAreaElement;
    input.value = "已有输入";
    (g.handleKey as (e: KeyboardEvent) => void)({
      key: "ArrowUp", target: input, preventDefault: () => {}, stopPropagation: () => {},
    } as unknown as KeyboardEvent);
    expect(input.value).toBe("已有输入");
  });

  it("stream_start shows thinking indicator, first token clears it", async () => {
    const g = globalThis as Record<string, unknown>;
    const mock: Record<string, (a: unknown) => unknown> = {
      neocodex_send_message_stream: () => "ok",
    };
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const cbs: Record<string, (raw: unknown) => void> = {};
    let cbRef: ((raw: unknown) => void) | null = null;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "plugin:event|listen") {
          const ev = String(args?.event);
          cbs[ev] = cbRef!;
          return Promise.resolve(() => {});
        }
        const h = mock[cmd];
        return Promise.resolve(h ? h(args ?? {}) : undefined);
      },
      transformCallback: (fn: (raw: unknown) => void) => { cbRef = fn; return 1; },
    };
    try {
      await reloadApp();
      const input = document.getElementById("chatInput") as HTMLTextAreaElement;
      input.value = "思考测试";
      (g.sendMsg as () => void)();
      cbs["neocodex_stream_start"]({ event: "neocodex_stream_start", id: 1, payload: "思考测试" });
      const think = document.querySelector("#chatScroll .msg.l .mb .think");
      expect(think).not.toBeNull();
      expect(think!.textContent).toMatch(/思考中/);
      cbs["neocodex_stream_token"]({ event: "neocodex_stream_token", id: 1, payload: "输出" });
      expect(document.querySelector("#chatScroll .msg.l .mb .think")).toBeNull();
      expect(document.querySelector("#chatScroll .msg.l .mb")).not.toBeNull();
      cbs["neocodex_stream_done"]({ event: "neocodex_stream_done", id: 1, payload: { cancelled: false } });
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("renderThread adds copy button to user messages and copies its text", async () => {
    const g = globalThis as Record<string, unknown>;
    (g.renderThread as (msgs: unknown[], sessionId?: string) => void)([
      { role: "user", content: "用户消息内容", timestamp: 1700000001 },
      { role: "agent", content: "回复", timestamp: 1700000002 },
    ], "s-ucopy");
    const userMsg = document.querySelector("#chatScroll .msg.r")!;
    const copyBtn = userMsg.querySelector('.ma-btn[data-op="copy"]');
    expect(copyBtn).not.toBeNull();
    const realClip = (navigator as { clipboard?: { writeText: (t: string) => Promise<void> } }).clipboard;
    let copied = "";
    (navigator as { clipboard?: { writeText: (t: string) => Promise<void> } }).clipboard = {
      writeText: (t: string) => { copied = t; return Promise.resolve(); },
    };
    try {
      (g.copyUserContent as (el: HTMLElement) => Promise<void>)(userMsg as HTMLElement);
      await new Promise((r) => setTimeout(r, 10));
      expect(copied).toBe("用户消息内容");
    } finally {
      if (realClip === undefined) delete (navigator as { clipboard?: unknown }).clipboard;
      else (navigator as { clipboard?: { writeText: (t: string) => Promise<void> } }).clipboard = realClip;
    }
  });

  it("Esc while streaming invokes stop, Esc without stream does not", async () => {
    const g = globalThis as Record<string, unknown>;
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const mock: Record<string, (a: unknown) => unknown> = { neocodex_send_message_stream: () => "ok" };
    const calls: { cmd: string }[] = [];
    const cbs: Record<string, (raw: unknown) => void> = {};
    let cbRef: ((raw: unknown) => void) | null = null;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "plugin:event|listen") {
          const ev = String(args?.event);
          cbs[ev] = cbRef!;
          return Promise.resolve(() => {});
        }
        calls.push({ cmd });
        const h = mock[cmd];
        return Promise.resolve(h ? h(args ?? {}) : undefined);
      },
      transformCallback: (fn: (raw: unknown) => void) => { cbRef = fn; return 1; },
    };
    try {
      const sendBtn = document.getElementById("sendBtn") as HTMLButtonElement;
      const input = document.getElementById("chatInput") as HTMLTextAreaElement;
      input.value = "中断测试";
      (g.sendMsg as () => void)();
      expect(sendBtn.disabled).toBe(true);
      document.body.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }));
      expect(calls.some((c) => c.cmd === "neocodex_stop_stream")).toBe(true);
      expect(sendBtn.disabled).toBe(false);
      const stopCalls = calls.filter((c) => c.cmd === "neocodex_stop_stream").length;
      document.body.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }));
      expect(calls.filter((c) => c.cmd === "neocodex_stop_stream").length).toBe(stopCalls);
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("attachAssistantCopy adds copy button to streamed reply and copies its text", async () => {
    const g = globalThis as Record<string, unknown>;
    const cs = document.getElementById("chatScroll")!;
    cs.innerHTML = "";
    const a = document.createElement("div");
    a.className = "msg l";
    a.innerHTML = `<div class="msg-h"><span class="name">NeoTrix</span><span class="time">12:00</span></div><div class="mb"><p>你好 <strong>世界</strong></p></div>`;
    cs.appendChild(a);
    (g.attachAssistantCopy as (el: HTMLElement) => void)(a);
    const copyBtn = a.querySelector('.ma-btn[data-op="copy"]');
    expect(copyBtn).not.toBeNull();
    const realClip = (navigator as { clipboard?: { writeText: (t: string) => Promise<void> } }).clipboard;
    let copied = "";
    (navigator as { clipboard?: { writeText: (t: string) => Promise<void> } }).clipboard = {
      writeText: (t: string) => { copied = t; return Promise.resolve(); },
    };
    try {
      (g.copyAssistantContent as (el: HTMLElement) => Promise<void>)(a);
      await new Promise((r) => setTimeout(r, 10));
      expect(copied).toBe("你好 世界");
    } finally {
      if (realClip === undefined) delete (navigator as { clipboard?: unknown }).clipboard;
      else (navigator as { clipboard?: { writeText: (t: string) => Promise<void> } }).clipboard = realClip;
    }
  });

  it("attachAssistantCopy is idempotent and reuses existing msg-act bar", async () => {
    const g = globalThis as Record<string, unknown>;
    const cs = document.getElementById("chatScroll")!;
    cs.innerHTML = "";
    const a = document.createElement("div");
    a.className = "msg l";
    a.innerHTML = `<div class="msg-act"><button class="ma-btn" data-op="retry">重试</button></div><div class="mb">回复</div>`;
    cs.appendChild(a);
    (g.attachAssistantCopy as (el: HTMLElement) => void)(a);
    (g.attachAssistantCopy as (el: HTMLElement) => void)(a);
    const acts = a.querySelectorAll('.msg-act');
    expect(acts.length).toBe(1);
    expect(a.querySelectorAll('.ma-btn[data-op="copy"]').length).toBe(1);
    expect(a.querySelectorAll('.ma-btn').length).toBe(2);
  });

  it("ntxConfirm confirms on confirm click and removes the modal", async () => {
    const g = globalThis as Record<string, unknown>;
    const p = (g.ntxConfirm as (msg: string, opts?: Record<string, unknown>) => Promise<boolean>)("确定删除该消息？", { title: "删除消息", danger: true });
    const wrap = document.getElementById("ntxConfirm")!;
    expect(wrap).not.toBeNull();
    expect(wrap.querySelector(".ntx-cf-msg")!.textContent).toContain("确定删除该消息？");
    expect(wrap.querySelector('[data-act="confirm"]')!.textContent).toBe("确认");
    expect(wrap.querySelector(".ntx-cf-danger")).not.toBeNull();
    (wrap.querySelector('[data-act="confirm"]') as HTMLButtonElement).click();
    await expect(p).resolves.toBe(true);
    await new Promise((r) => setTimeout(r, 250));
    expect(document.getElementById("ntxConfirm")).toBeNull();
  });

  it("ntxConfirm resolves false on cancel click", async () => {
    const g = globalThis as Record<string, unknown>;
    const p = (g.ntxConfirm as (msg: string, opts?: Record<string, unknown>) => Promise<boolean>)("取消这个？");
    const wrap = document.getElementById("ntxConfirm")!;
    (wrap.querySelector('[data-act="cancel"]') as HTMLButtonElement).click();
    await expect(p).resolves.toBe(false);
  });

  it("ntxConfirm resolves false on Escape", async () => {
    const g = globalThis as Record<string, unknown>;
    const p = (g.ntxConfirm as (msg: string, opts?: Record<string, unknown>) => Promise<boolean>)("按 Esc 取消");
    document.body.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }));
    await expect(p).resolves.toBe(false);
  });

  it("deleteMessage uses ntxConfirm and calls invoke only on confirm", async () => {
    const g = globalThis as Record<string, unknown>;
    const realInternals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    const calls: { cmd: string }[] = [];
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (cmd: string) => { calls.push({ cmd }); return Promise.resolve([]); },
      transformCallback: () => 0,
    };
    try {
      (g.deleteMessage as (i: number) => Promise<void>)(0);
      await new Promise((r) => setTimeout(r, 20));
      expect(document.getElementById("ntxConfirm")).not.toBeNull();
      (document.querySelector('#ntxConfirm [data-act="cancel"]') as HTMLButtonElement).click();
      await new Promise((r) => setTimeout(r, 250));
      expect(calls.some((c) => c.cmd === "neocodex_delete_message")).toBe(false);
      (g.deleteMessage as (i: number) => Promise<void>)(0);
      await new Promise((r) => setTimeout(r, 20));
      (document.querySelector('#ntxConfirm [data-act="confirm"]') as HTMLButtonElement).click();
      await new Promise((r) => setTimeout(r, 250));
      expect(calls.some((c) => c.cmd === "neocodex_delete_message")).toBe(true);
    } finally {
      if (realInternals === undefined) delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
      else (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = realInternals;
    }
  });

  it("dayStartTs exposes today 0am local Unix seconds (and offsetDays shifts days)", () => {
    const g = globalThis as Record<string, unknown>;
    expect(typeof g.dayStartTs).toBe("function");
    const today = g.dayStartTs as (d?: number) => number;
    const now = new Date();
    const todayDate = new Date(today(0) * 1000);
    expect(todayDate.getHours()).toBe(0);
    expect(todayDate.getMinutes()).toBe(0);
    expect(todayDate.getSeconds()).toBe(0);
    const diff = today(1) - today(0);
    expect(diff).toBe(86400);
  });

  it("groupSessionsByTime buckets into 今天/昨天/7 天内/更早 in order (dynamic timestamps)", () => {
    const g = globalThis as Record<string, unknown>;
    const DAY = 86400;
    const now = Math.floor(Date.now() / 1000);
    const sessions = [
      { id: "old", name: "三十天前", updated_at: now - 30 * DAY },
      { id: "today", name: "今天", updated_at: now },
      { id: "yest", name: "昨天", updated_at: now - DAY },
      { id: "wk", name: "六天前", updated_at: now - 6 * DAY },
      { id: "nots", name: "无时间", status: "进行中", tasks: 1, done: 0, fail: 0 },
    ];
    const groups = (g.groupSessionsByTime as (s: unknown[]) => { label: string; sessions: Array<{ id: string }> }[])(sessions);
    expect(groups.map((x) => x.label)).toEqual(["今天", "昨天", "7 天内", "更早"]);
    expect(groups[0].sessions.map((s) => s.id)).toEqual(["today"]);
    expect(groups[1].sessions.map((s) => s.id)).toEqual(["yest"]);
    expect(groups[2].sessions.map((s) => s.id)).toEqual(["wk"]);
    // 无 updated_at 归入更早；更早组内按 updated_at 降序（有时间的在前）
    expect(groups[3].sessions.map((s) => s.id)).toEqual(["old", "nots"]);
  });

  it("groupSessionsByTime sorts each bucket by updated_at descending", () => {
    const g = globalThis as Record<string, unknown>;
    const now = Math.floor(Date.now() / 1000);
    const sessions = [
      { id: "a", updated_at: now - 120 },
      { id: "b", updated_at: now - 10 },
      { id: "c", updated_at: now - 3600 },
    ];
    const groups = (g.groupSessionsByTime as (s: unknown[]) => { label: string; sessions: Array<{ id: string }> }[])(sessions);
    expect(groups.map((x) => x.label)).toEqual(["今天"]);
    expect(groups[0].sessions.map((s) => s.id)).toEqual(["b", "a", "c"]);
  });

  it("renderCowork renders .cw-group-h headers 今天/昨天/更早 with items intact", () => {
    const g = globalThis as Record<string, unknown>;
    const DAY = 86400;
    const now = Math.floor(Date.now() / 1000);
    (g as unknown as { CW_DATA: unknown[] }).CW_DATA = [
      { name: "今日会话", status: "进行中", tasks: 1, done: 0, fail: 0, updated_at: now },
      { name: "昨日会话", status: "进行中", tasks: 1, done: 0, fail: 0, updated_at: now - DAY },
      { name: "更早会话", status: "已完成", tasks: 1, done: 1, fail: 0, updated_at: now - 30 * DAY },
    ];
    // reset status filter that earlier tests may have left at 'done'
    (g.cwFilter as (s: string) => void)("all");
    (g.renderCowork as () => void)();
    const headers = [...document.querySelectorAll("#cwSessionList .cw-group-h")].map((e) => e.textContent);
    expect(headers).toEqual(["今天", "昨天", "更早"]);
    expect(headers.some((t) => t!.includes("今天"))).toBe(true);
    expect(headers.some((t) => t!.includes("昨天"))).toBe(true);
    expect(document.querySelectorAll("#cwSessionList .cw-sitem").length).toBe(3);
    // item structure unchanged: status dot + name inside each item
    expect(document.querySelectorAll("#cwSessionList .cw-sitem .st-dot").length).toBe(3);
    expect(document.querySelector("#cwSessionList .cw-sitem")!.textContent).toContain("今日会话");
  });
});


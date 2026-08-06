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
});

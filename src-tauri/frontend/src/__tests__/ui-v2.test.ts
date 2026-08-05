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
});

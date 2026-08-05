import { test, expect, mockCommand, invokeCalls, emitEvent } from "./fixtures";

test.describe("NeoTrix IPC interaction flows (mocked Tauri)", () => {
  test("session list is requested from backend", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => [
      { id: "s-1", name: "重构缓存层", mode: "Agent", message_count: 3 },
      { id: "s-2", name: "调研 RAG", mode: "Plan", message_count: 1 },
    ]);
    await page.goto("/");
    await page.locator('.segb[data-view="cowork"]').click({ force: true });
    await expect(page.locator("#cwSessionList .cw-sitem").first()).toBeVisible({ timeout: 10_000 });

    const calls = await invokeCalls(page);
    expect(calls.some((c) => c.cmd === "neocodex_list_sessions")).toBeTruthy();
  });

  test("send message drives the stream-token event into the chat", async ({ page }) => {
    await mockCommand(page, "neocodex_send_message_stream", () => "ok");
    await page.goto("/");
    const textarea = page.locator("#chatInput");
    await textarea.fill("帮我分析 RAG 检索");
    await textarea.press("Enter");

    await expect(page.locator("#chatScroll .msg.r")).toContainText("帮我分析 RAG 检索", { timeout: 10_000 });

    await emitEvent(page, "neocodex_stream_token", "这是");
    await emitEvent(page, "neocodex_stream_token", "流式回复");
    await emitEvent(page, "neocodex_stream_done", { cancelled: false });

    const last = page.locator("#chatScroll .msg.l .mb").last();
    await expect(last).toContainText("流式回复", { timeout: 10_000 });

    const calls = await invokeCalls(page);
    expect(calls.some((c) => c.cmd === "neocodex_send_message_stream")).toBeTruthy();
  });

  test("MCP register invokes backend with parsed args", async ({ page }) => {
    await mockCommand(page, "neocodex_mcp_register", () => "ok");
    await page.goto("/");
    await page.keyboard.press("Meta+,");
    await page.locator(".st-item", { hasText: "代理 · 网关" }).click({ force: true });
    await page.locator("#mcpName").fill("search-tool");
    await page.locator("#mcpCmd").fill("python -m mcp_search");
    await page.locator("#mcpArgs").fill("--port,8080");
    await page.locator("button", { hasText: "注册" }).click({ force: true });

    const calls = await invokeCalls(page);
    const reg = calls.find((c) => c.cmd === "neocodex_mcp_register");
    expect(reg).toBeTruthy();
    expect(reg.args).toMatchObject({ name: "search-tool", command: "python -m mcp_search" });
  });

  test("KB search delegates to backend and renders results", async ({ page }) => {
    await mockCommand(page, "kb_search", () => [
      { id: "n-1", title: "VSA HyperCube", summary: "向量符号架构知识库" },
    ]);
    await page.goto("/");
    await page.keyboard.press("Meta+,");
    await page.locator(".st-item", { hasText: "数据" }).click({ force: true });
    await page.locator("#kbSearchInput").fill("VSA");
    await page.locator("#kbSearchInput").press("Enter");
    const res = page.locator("#kbResults");
    await expect(res).toContainText("VSA HyperCube", { timeout: 10_000 });

    const calls = await invokeCalls(page);
    expect(calls.some((c) => c.cmd === "kb_search")).toBeTruthy();
  });

  test("diff overlay opens with backend diff fallback to sample", async ({ page }) => {
    await page.goto("/");
    await page.locator("#ntxPlusBtn").click({ force: true });
    await page.locator('.ntx-pm-item[data-act="diff"]').click({ force: true });
    await expect(page.locator("#overlayDiff")).toHaveClass(/open/);
    await expect(page.locator("#diffTitle")).toContainText("代码变更");
    await expect(page.locator("#diffBody")).not.toBeEmpty();
  });

  test("no uncaught errors during a chat send", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto("/");
    const textarea = page.locator("#chatInput");
    await textarea.fill("你好");
    await textarea.press("Enter");
    await page.waitForTimeout(800);
    expect(errors).toEqual([]);
  });

  test("stop button appears while streaming and calls neocodex_stop_stream", async ({ page }) => {
    await mockCommand(page, "neocodex_send_message_stream", () => "ok");
    await mockCommand(page, "neocodex_stop_stream", () => "ok");
    await page.goto("/");
    const textarea = page.locator("#chatInput");
    await textarea.fill("长任务");
    await textarea.press("Enter");
    await expect(page.locator("#stopBtn")).toBeVisible({ timeout: 10_000 });
    await page.locator("#stopBtn").click({ force: true });
    const calls = await invokeCalls(page);
    expect(calls.some((c) => c.cmd === "neocodex_stop_stream")).toBeTruthy();
    await expect(page.locator("#stopBtn")).toBeHidden({ timeout: 10_000 });
  });

  test("loaded thread exposes retry/delete actions wired to backend ops", async ({ page }) => {
    await mockCommand(page, "neocodex_get_session_messages", () => [
      { role: "user", content: "给我优化缓存", timestamp: 1700000000 },
      { role: "agent", content: "已优化", timestamp: 1700000001 },
    ]);
    await mockCommand(page, "neocodex_regenerate", () => [
      { role: "user", content: "给我优化缓存", timestamp: 1700000000 },
    ]);
    await page.goto("/");
    const textarea = page.locator("#chatInput");
    await textarea.fill("给我优化缓存");
    await textarea.press("Enter");
    await page.waitForTimeout(300);
    // switch to a session id and reload the thread to attach action buttons
    await page.evaluate(() => {
      const w = window as unknown as Record<string, unknown>;
      (w.renderThread as (msgs: unknown[], sessionId?: string) => void)([
        { role: "user", content: "给我优化缓存", timestamp: 1700000000 },
        { role: "agent", content: "已优化", timestamp: 1700000001 },
      ], "s-1");
    });
    await expect(page.locator("#chatScroll .msg.l .ma-btn[data-op='retry']")).toBeVisible({ timeout: 10_000 });
    await page.locator("#chatScroll .msg.l .ma-btn[data-op='retry']").click({ force: true });
    const calls = await invokeCalls(page);
    expect(calls.some((c) => c.cmd === "neocodex_regenerate")).toBeTruthy();
  });
});
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

  test("stream token renders markdown inline live, code fence completes at close", async ({ page }) => {
    await mockCommand(page, "neocodex_send_message_stream", () => "ok");
    await page.goto("/");
    const textarea = page.locator("#chatInput");
    await textarea.fill("流式 markdown");
    await textarea.press("Enter");

    await expect(page.locator("#chatScroll .msg.r")).toContainText("流式 markdown", { timeout: 10_000 });

    await emitEvent(page, "neocodex_stream_token", "**加粗**\n");
    const mb = page.locator("#chatScroll .msg.l .mb.streaming").last();
    await expect(mb.locator("strong")).toHaveText("加粗", { timeout: 10_000 });
    expect(await mb.evaluate((el) => el.textContent)).not.toContain("**");

    await emitEvent(page, "neocodex_stream_token", "```python\nprint(");
    const pre = mb.locator("pre.msg-code-stream");
    await expect(pre).toHaveCount(1, { timeout: 10_000 });
    await expect(pre).toContainText("print(");

    await emitEvent(page, "neocodex_stream_token", "1)\n```");
    await expect(mb.locator("pre.msg-code-stream")).toHaveCount(0, { timeout: 10_000 });
    await expect(mb.locator(".msg-code-b")).toHaveCount(1, { timeout: 10_000 });

    const done = page.locator("#chatScroll .msg.l .mb").last();
    await expect(done.locator(".msg-code-b")).toContainText("print(1)", { timeout: 10_000 });

    await emitEvent(page, "neocodex_stream_done", { cancelled: false });
    await expect(done).not.toHaveClass(/streaming/, { timeout: 10_000 });
  });

  test("MCP register invokes backend with parsed args", async ({ page }) => {
    await mockCommand(page, "neocodex_mcp_register", () => "ok");
    await page.goto("/");
    await page.keyboard.press("Meta+,");
    await page.locator(".st-item", { hasText: "代理 · 网关" }).click({ force: true });
    await page.locator("#mcpName").fill("search-tool");
    await page.locator("#mcpCmd").fill("python -m mcp_search");
    await page.locator("#mcpArgs").fill("--port,8080");
    await page.locator("#overlaySettings button", { hasText: "注册" }).click({ force: true });

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

  test("assistant message exposes like/dislike feedback wired to neocodex_feedback", async ({ page }) => {
    await mockCommand(page, "neocodex_feedback", () => "ok");
    await page.goto("/");
    await page.evaluate(() => {
      const w = window as unknown as Record<string, unknown>;
      (w.renderThread as (msgs: unknown[], sessionId?: string) => void)([
        { role: "user", content: "反馈", timestamp: 1700000000 },
        { role: "agent", content: "回答", timestamp: 1700000001 },
      ], "s-fb");
    });
    const like = page.locator('#chatScroll .msg.l .ma-btn[data-op="like"]');
    await expect(like).toBeVisible({ timeout: 10_000 });
    await like.click({ force: true });
    await expect(like).toHaveClass(/on/);
    const calls = await invokeCalls(page);
    const fb = calls.find((c) => c.cmd === "neocodex_feedback");
    expect(fb).toBeTruthy();
    expect(fb.args).toMatchObject({ session_id: "s-fb" });
  });

  test("session ops menu opens from toolbar and shows rename/archive/export", async ({ page }) => {
    await page.goto("/");
    await page.evaluate(() => {
      const w = window as unknown as Record<string, unknown>;
      (w.openSessionOps as (anchor: HTMLElement | null, id: string) => void)(null, "s-ops");
    });
    const menu = page.locator("#sessionOpsMenu");
    await expect(menu).toHaveClass(/open/);
    await expect(menu.locator(".ses-item", { hasText: "重命名" })).toBeVisible();
    await expect(menu.locator(".ses-item", { hasText: /^归档$/ })).toBeVisible();
    await expect(menu.locator(".ses-item", { hasText: "查看归档" })).toBeVisible();
    await expect(menu.locator(".ses-item", { hasText: "时间线" })).toBeVisible();
    await expect(menu.locator(".ses-item", { hasText: "导出" })).toBeVisible();
    await expect(menu.locator(".ses-item.danger", { hasText: "删除会话" })).toBeVisible();
  });

  test("session search queries backend and renders hits", async ({ page }) => {
    await mockCommand(page, "neocodex_search_sessions", () => [
      { session_id: "s-hit", session_name: "全文命中", role: "agent", snippet: "缓存重建完成", match_count: 3, timestamp: 1700000000 },
    ]);
    await page.goto("/");
    await page.locator('.segb[data-view="cowork"]').click({ force: true });
    await page.locator('#cwSearchInput').fill("缓存");
    const res = page.locator("#cwSearchResults");
    await expect(res).toContainText("全文命中", { timeout: 10_000 });
    await expect(res).toContainText("3 处");
    const calls = await invokeCalls(page);
    expect(calls.some((c) => c.cmd === "neocodex_search_sessions")).toBeTruthy();
  });

  test("archived sessions overlay lists backend items and restores", async ({ page }) => {
    await mockCommand(page, "neocodex_list_archived", () => [
      { id: "a-1", name: "旧项目归档…", mode: "Agent", message_count: 12, updated_at: 1700000000 },
    ]);
    await mockCommand(page, "neocodex_restore_session", () => "Restored session a-1");
    await page.goto("/");
    await page.evaluate(() => {
      const w = window as unknown as Record<string, unknown>;
      (w.openArchivedSessions as () => void)();
    });
    const overlay = page.locator("#overlayArchived");
    await expect(overlay).toHaveClass(/open/);
    await expect(page.locator("#archivedBody")).toContainText("旧项目归档…", { timeout: 10_000 });
    await page.locator("#archivedBody .arch-btn.restore").first().click({ force: true });
    const calls = await invokeCalls(page);
    expect(calls.some((c) => c.cmd === "neocodex_restore_session" && c.args?.session_id === "a-1")).toBeTruthy();
  });

  test("checkpoint timeline lists snapshots and rewinds", async ({ page }) => {
    await mockCommand(page, "neocodex_checkpoint_list", () => [
      { id: "s-ck-1000.jsonl", created_at: 1700000010, message_count: 5 },
      { id: "s-ck-900.jsonl", created_at: 1700000000, message_count: 3 },
    ]);
    await mockCommand(page, "neocodex_checkpoint_restore", () => [
      { role: "user", content: "回滚后内容", timestamp: 1700000000 },
    ]);
    await page.goto("/");
    page.on("dialog", (d) => d.accept());
    await page.evaluate(() => {
      const w = window as unknown as Record<string, unknown>;
      (w.openSessionOps as (anchor: HTMLElement | null, id: string) => void)(null, "s-ck");
      (w.openCheckpointTimeline as () => void)();
    });
    const overlay = page.locator("#overlayCheckpoints");
    await expect(overlay).toHaveClass(/open/);
    await expect(page.locator("#checkpointBody")).toContainText("快照 #2", { timeout: 10_000 });
    await page.locator("#checkpointBody .arch-btn.restore").first().click({ force: true });
    const calls = await invokeCalls(page);
    expect(calls.some((c) => c.cmd === "neocodex_checkpoint_restore" && c.args?.checkpoint_id === "s-ck-1000.jsonl")).toBeTruthy();
  });

  test("real backend diff renders files/hunks and apply_diff wires accept", async ({ page }) => {
    await mockCommand(page, "neocodex_get_diff", () => ({
      files: [
        { path: "src/app.rs", hunks: [
          { lines: [
            { t: "ctx", o: 1, n: 1, s: "fn main() {" },
            { t: "del", o: 2, n: null, s: "    todo!()" },
            { t: "add", o: null, n: 2, s: "    run();" },
          ]},
        ]},
      ],
    }));
    await mockCommand(page, "neocodex_apply_diff", () => "ok");
    await page.goto("/");
    await page.locator("#ntxPlusBtn").click({ force: true });
    await page.locator('.ntx-pm-item[data-act="diff"]').click({ force: true });
    await expect(page.locator("#overlayDiff")).toHaveClass(/open/);
    await expect(page.locator("#diffTitle")).toContainText("1 文件");
    await expect(page.locator("#diffBody .df-path")).toContainText("src/app.rs");
    await expect(page.locator("#diffBody .df-line.add")).toContainText("run();");
    await expect(page.locator("#diffTitle")).not.toContainText("示例数据");
    await page.locator("#diffBody .df-act.accept").first().click({ force: true });
    const calls = await invokeCalls(page);
    const ap = calls.find((c) => c.cmd === "neocodex_apply_diff");
    expect(ap).toBeTruthy();
    expect(ap.args).toMatchObject({ path: "src/app.rs", action: "accept" });
    await expect(page.locator("#diffBody .df-done-tag")).toContainText("已接受");
  });

  test("@ mention popup opens on @ and inserts pill on select", async ({ page }) => {
    await page.goto("/");
    const textarea = page.locator("#chatInput");
    await textarea.click();
    await textarea.pressSequentially("@");
    const menu = page.locator("#qmMenu");
    await expect(menu).toBeVisible({ timeout: 10_000 });
    await expect(menu.locator(".qm-item").first()).toContainText("@nt-core");
    await menu.locator(".qm-item").first().click();
    await expect(textarea).toHaveValue(/@nt-core/);
  });

  test("/ slash command menu lists commands", async ({ page }) => {
    await page.goto("/");
    const textarea = page.locator("#chatInput");
    await textarea.click();
    await textarea.pressSequentially("/di");
    const menu = page.locator("#qmMenu");
    await expect(menu).toBeVisible({ timeout: 10_000 });
    await expect(menu.locator(".qm-item").first()).toContainText("/diff");
  });

  test("context meter renders health usage and opens breakdown popover", async ({ page }) => {
    await mockCommand(page, "neocodex_health_report", () => ({
      context_usage: 0.85, context_turns: 12, tokens_used: 42000,
      tool_call_count: 9, provider_model: "claude-sonnet", cost_spent: 1.2, cost_budget: 10,
    }));
    await page.goto("/");
    const chip = page.locator("#ntxCtxMeter .ctx-chip");
    await expect(chip).toContainText("85%", { timeout: 10_000 });
    await chip.click();
    const pop = page.locator("#ntxCtxPop");
    await expect(pop).toHaveClass(/open/);
    await expect(pop).toContainText("42,000");
    await expect(pop).toContainText("claude-sonnet");
  });

  test("add attachment chip via plus menu and sendMsg passes attachments", async ({ page }) => {
    await mockCommand(page, "neocodex_send_message_stream", () => "ok");
    await page.goto("/");
    // drive the chip directly through the exposed global (matches Tauri file flow)
    await page.evaluate(() => {
      (window as any).addAttachChip("src/app.rs", { size: 512, mime: "text/plain", data: "fn main(){}" });
    });
    await expect(page.locator("#ntxAttachArea .ntx-attach-chip")).toContainText("src/app.rs");
    await page.locator("#chatInput").fill("分析这个文件");
    await page.locator("#chatInput").press("Enter");
    await expect(page.locator("#chatScroll .msg.r")).toContainText("分析这个文件", { timeout: 10_000 });
    const calls = await invokeCalls(page);
    const send = calls.find((c) => c.cmd === "neocodex_send_message_stream");
    expect(send).toBeTruthy();
    expect(send.args.attachments).toBeTruthy();
    expect(send.args.attachments[0]).toMatchObject({ name: "src/app.rs", size: 512 });
    // chips cleared after send
    await expect(page.locator("#ntxAttachArea .ntx-attach-chip")).toHaveCount(0);
  });

  test("ref picker lists recent messages and insertReference quotes into composer", async ({ page }) => {
    await mockCommand(page, "neocodex_get_session_messages", () => [
      { role: "user", content: "早前的提问", timestamp: 1700000000 },
      { role: "agent", content: "早前的回答", timestamp: 1700000001 },
    ]);
    await page.goto("/");
    // simulate an active session id then open the ref picker
    await page.evaluate(() => {
      (window as any).renderThread([
        { role: "user", content: "早前的提问", timestamp: 1700000000 },
        { role: "agent", content: "早前的回答", timestamp: 1700000001 },
      ], "s-ref");
      (window as any).openRefPicker();
    });
    const picker = page.locator("#ntxRefPicker");
    await expect(picker).toHaveClass(/open/);
    await expect(picker.locator(".rf-item").first()).toContainText("早前的提问");
    await picker.locator(".rf-item").first().click();
    await expect(page.locator("#chatInput")).toHaveValue(/\[引用·我\] 早前的提问/);
    await expect(picker).not.toHaveClass(/open/);
  });

  test("command palette opens via ⌘K and searches sessions", async ({ page }) => {
    await mockCommand(page, "neocodex_search_sessions", () => [
      { session_id: "s-pal", session_name: "面板命中会话", role: "agent", snippet: "含关键词", match_count: 2, timestamp: 1700000000 },
    ]);
    await page.goto("/");
    await page.keyboard.press("Meta+k");
    const ov = page.locator("#overlayPalette");
    await expect(ov).toHaveClass(/open/);
    await expect(ov.locator(".pal-item[data-act]")).toHaveCount(7);
    await page.locator("#palInput").fill("面板");
    const res = page.locator("#palResults");
    await expect(res).toContainText("面板命中会话", { timeout: 10_000 });
    await expect(res).toContainText("2 处");
    const calls = await invokeCalls(page);
    expect(calls.some((c) => c.cmd === "neocodex_search_sessions" && c.args?.query === "面板")).toBeTruthy();
  });

  test("palette quick action opens settings and sidebar search button opens palette", async ({ page }) => {
    await page.goto("/");
    await page.evaluate(() => {
      (window as any).openPalette();
    });
    const ov = page.locator("#overlayPalette");
    await expect(ov).toHaveClass(/open/);
    await ov.locator('.pal-item[data-act="settings"]').click({ force: true });
    await expect(ov).not.toHaveClass(/open/);
    await expect(page.locator("#overlaySettings")).toHaveClass(/open/);
    // sidebar search button now opens the palette instead of a dead toast
    await page.locator('.sbn[onclick*="openPalette"]').click({ force: true });
    await expect(page.locator("#overlayPalette")).toHaveClass(/open/);
    await page.locator("#palInput").press("Escape");
    await expect(page.locator("#overlayPalette")).not.toHaveClass(/open/);
  });

  test("assistant markdown renders tables, tasks, headings and inline styles", async ({ page }) => {
    await page.goto("/");
    await page.evaluate(() => {
      (window as any).renderThread([
        {
          role: "agent",
          content: "# 方案\n\n**重点** 与 `code`\n\n| 步骤 | 状态 |\n| --- | --- |\n| 初始化 | 完成 |\n| 部署 | 待办 |\n\n- [x] 核对\n- [ ] 上线\n",
          timestamp: 1700000000,
        },
      ], "s-md");
    });
    const mb = page.locator("#chatScroll .msg.l .mb");
    await expect(mb.locator("h1")).toContainText("方案");
    await expect(mb.locator("strong")).toContainText("重点");
    await expect(mb.locator("code")).toContainText("code");
    await expect(mb.locator("table th").first()).toContainText("步骤");
    await expect(mb.locator("table td").nth(1)).toContainText("完成");
    await expect(mb.locator(".md-task-done")).toContainText("核对");
    await expect(mb.locator(".md-task:not(.md-task-done)")).toContainText("上线");
  });

  test("markdown XSS is neutralized in assistant output", async ({ page }) => {
    await page.goto("/");
    await page.evaluate(() => {
      (window as any).renderThread([
        { role: "agent", content: "<img src=x onerror=alert(1)> [click](javascript:alert(2))", timestamp: 1700000000 },
      ], "s-xss");
    });
    const mb = page.locator("#chatScroll .msg.l .mb");
    await expect(mb.locator("img")).toHaveCount(0);
    await expect(mb.locator("a")).toHaveCount(0);
    await expect(mb).toContainText("[click](javascript:alert(2))");
  });

  test("composer draft persists across session reload and restores", async ({ page }) => {
    await page.goto("/");
    await page.evaluate(() => {
      (window as any).renderThread([], "s-draft-e2e");
    });
    await page.locator("#chatInput").fill("稍后发送的草稿");
    // saveDraft is debounced; wait for persistence
    await page.waitForTimeout(500);
    let drafts = await page.evaluate(() => localStorage.getItem("neotrix.drafts") || "{}");
    expect(JSON.parse(drafts)["s-draft-e2e"]).toBe("稍后发送的草稿");
    // simulate switching away and back: empty composer + renderThread re-entry
    await page.evaluate(() => {
      const input = document.getElementById("chatInput") as HTMLTextAreaElement;
      input.value = "";
      (window as any).renderThread([], "s-draft-e2e");
    });
    await expect(page.locator("#chatInput")).toHaveValue("稍后发送的草稿");
  });

  test("streaming scroll pill appears when scrolled up and jumpToLatest restores", async ({ page }) => {
    await page.goto("/");
    const cs = page.locator("#chatScroll");
    await page.evaluate(() => {
      (window as any).renderThread([], "s-scroll");
      // build tall content so the container overflows and scrollTop can sit away from bottom
      const s = document.getElementById("chatScroll");
      s.style.display = "flex";
      s.innerHTML = "";
      for (let k = 0; k < 40; k++) {
        const m = document.createElement("div");
        m.className = "msg l";
        m.style.height = "48px";
        m.innerHTML = '<div class="mb">垫片内容 ' + k + "</div>";
        s.appendChild(m);
      }
      const a = document.createElement("div");
      a.className = "msg l";
      a.innerHTML = '<div class="mb streaming">部分输出…</div>';
      s.appendChild(a);
      // user scrolls away from the bottom while a stream is active
      s.scrollTop = 0;
      s.dispatchEvent(new Event("scroll"));
    });
    await expect(page.locator("#scrollJump")).toHaveClass(/show/);
    await page.locator("#scrollJump").click({ force: true });
    await expect(page.locator("#scrollJump")).not.toHaveClass(/show/);
    const maxScroll = await cs.evaluate((el) => el.scrollHeight - el.clientHeight);
    await expect(cs).toHaveJSProperty("scrollTop", maxScroll);
  });

  test("ArrowUp in empty composer recalls last user message, Escape clears", async ({ page }) => {
    await mockCommand(page, "neocodex_send_message_stream", () => "ok");
    await page.goto("/");
    await page.evaluate(() => {
      (window as any).renderThread([
        { role: "user", content: "第一条问题", timestamp: 1700000001 },
        { role: "assistant", content: "回答一", timestamp: 1700000002 },
        { role: "user", content: "第二条问题", timestamp: 1700000003 },
      ], "s-recall");
    });
    const textarea = page.locator("#chatInput");
    await textarea.click();
    await textarea.press("ArrowUp");
    await expect(textarea).toHaveValue("第二条问题", { timeout: 10_000 });
    await textarea.press("ArrowUp");
    await expect(textarea).toHaveValue("第一条问题");
    await textarea.press("Escape");
    await expect(textarea).toHaveValue("");
  });

  test("stream_start shows live thinking indicator, token replaces it", async ({ page }) => {
    await mockCommand(page, "neocodex_send_message_stream", () => "ok");
    await page.goto("/");
    const textarea = page.locator("#chatInput");
    await textarea.fill("思考演示");
    await textarea.press("Enter");

    await expect(page.locator("#chatScroll .msg.r")).toContainText("思考演示", { timeout: 10_000 });

    await emitEvent(page, "neocodex_stream_start", "思考演示");
    const think = page.locator("#chatScroll .msg.l .mb .think");
    await expect(think).toHaveCount(1, { timeout: 10_000 });
    await expect(think).toContainText(/思考中.*\d+s?/, { timeout: 10_000 });

    await emitEvent(page, "neocodex_stream_token", "首段输出");
    await expect(page.locator("#chatScroll .msg.l .mb .think")).toHaveCount(0, { timeout: 10_000 });
    await expect(page.locator("#chatScroll .msg.l .mb")).toContainText("首段输出", { timeout: 10_000 });

    await emitEvent(page, "neocodex_stream_done", { cancelled: false });
  });

  test("Escape while streaming stops generation (ChatGPT parity)", async ({ page }) => {
    await mockCommand(page, "neocodex_send_message_stream", () => "ok");
    await mockCommand(page, "neocodex_stop_stream", () => {});
    await page.goto("/");
    const textarea = page.locator("#chatInput");
    await textarea.fill("中断回复");
    await textarea.press("Enter");
    await expect(page.locator("#chatScroll .msg.r")).toContainText("中断回复", { timeout: 10_000 });
    await emitEvent(page, "neocodex_stream_start", "中断回复");
    await expect(page.locator("#chatScroll .msg.l .mb .think")).toHaveCount(1, { timeout: 10_000 });
    const sendBtn = page.locator("#sendBtn");
    await expect(sendBtn).toBeDisabled();
    await page.keyboard.press("Escape");
    await expect(sendBtn).toBeEnabled();
    const stopCalls = (await invokeCalls(page)).filter((c) => c.cmd === "neocodex_stop_stream");
    expect(stopCalls.length).toBe(1);
    await page.keyboard.press("Escape");
    const stopCalls2 = (await invokeCalls(page)).filter((c) => c.cmd === "neocodex_stop_stream");
    expect(stopCalls2.length).toBe(1);
  });

  test("user messages expose a copy button that copies their text", async ({ page }) => {
    await page.goto("/");
    let copied = "";
    await page.evaluate(() => {
      Object.defineProperty(navigator, "clipboard", {
        value: { writeText: (t: string) => { (window as any).__copied = t; return Promise.resolve(); } },
        configurable: true,
      });
    });
    await page.evaluate(() => {
      (window as any).renderThread([
        { role: "user", content: "可复制的用户问题", timestamp: 1700000001 },
        { role: "assistant", content: "回答", timestamp: 1700000002 },
      ], "s-copy");
    });
    const msg = page.locator("#chatScroll .msg.r").first();
    const copyBtn = msg.locator('.ma-btn[data-op="copy"]');
    await expect(copyBtn).toHaveCount(1, { timeout: 10_000 });
    await copyBtn.hover({ force: true });
    await copyBtn.click({ force: true });
    await page.waitForFunction(() => (window as any).__copied !== undefined);
    copied = await page.evaluate(() => (window as any).__copied);
    expect(copied).toBe("可复制的用户问题");
  });

  test("streamed assistant reply gains a copy button after stream_end and copies rendered text", async ({ page }) => {
    await mockCommand(page, "neocodex_send_message_stream", () => "ok");
    await page.goto("/");
    await page.evaluate(() => {
      Object.defineProperty(navigator, "clipboard", {
        value: { writeText: (t: string) => { (window as any).__copied = t; return Promise.resolve(); } },
        configurable: true,
      });
    });
    const textarea = page.locator("#chatInput");
    await textarea.fill("给我一段回复");
    await textarea.press("Enter");
    await expect(page.locator("#chatScroll .msg.r")).toContainText("给我一段回复", { timeout: 10_000 });
    await emitEvent(page, "neocodex_stream_start", "给我一段回复");
    await emitEvent(page, "neocodex_stream_token", "第一行内容");
    await emitEvent(page, "neocodex_stream_end", "第一行内容\n\n第二行内容");
    const reply = page.locator("#chatScroll .msg.l").first();
    const copyBtn = reply.locator('.ma-btn[data-op="copy"]');
    await expect(copyBtn).toHaveCount(1, { timeout: 10_000 });
    await copyBtn.hover({ force: true });
    await copyBtn.click({ force: true });
    await page.waitForFunction(() => (window as any).__copied !== undefined);
    const copied = await page.evaluate(() => (window as any).__copied);
    expect(copied).toContain("第一行内容");
    expect(copied).toContain("第二行内容");
  });
});
import { test, expect, mockCommand, invokeCalls, emitEvent } from "./fixtures";

test.describe("NeoCodex interaction flows (mocked IPC)", () => {
  test("session list loads from backend and delete delegates the backend call", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => [
      { id: "s-1", name: "重构缓存层", mode: "Agent", message_count: 3, wire_path: "/sessions/s-1.jsonl", created_at: 0, updated_at: 1700000000 },
      { id: "s-2", name: "调研 RAG", mode: "Plan", message_count: 1, wire_path: "/sessions/s-2.jsonl", created_at: 0, updated_at: 1600000000 },
    ]);
    await mockCommand(page, "neocodex_list_archived", () => []);
    await mockCommand(page, "neocodex_delete_session", () => "ok");
    await page.goto("/");
    await page.getByTestId("sidebar-tab-sessions").click();
    await expect(page.getByText("重构缓存层").first()).toBeVisible({ timeout: 10_000 });
    await expect(page.getByText("调研 RAG").first()).toBeVisible();

    await page.getByTitle("删除会话").first().click();
    await expect(page.getByRole("dialog", { name: "删除会话确认" })).toBeVisible();
    await page.getByTestId("confirm-delete-confirm").click();

    const calls = await invokeCalls(page);
    expect(calls).toContainEqual(expect.objectContaining({ cmd: "neocodex_delete_session", args: { sessionId: "s-1" } }));
  });

  test("Cmd+N creates a new session directly", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => []);
    await mockCommand(page, "neocodex_list_archived", () => []);
    await mockCommand(page, "neocodex_create_session", () => ({
      id: "s-new",
      name: "新会话",
      mode: "Agent",
      message_count: 0,
      wire_path: "/sessions/s-new.jsonl",
      created_at: 0,
      updated_at: 1700000100,
    }));
    await page.goto("/");
    await page.getByTestId("sidebar-tab-sessions").click();
    await expect(page.getByTestId("sidebar-tab-sessions")).toBeVisible({ timeout: 10_000 });

    await page.keyboard.press("Meta+n");
    await expect(page.getByText("新会话").first()).toBeVisible({ timeout: 10_000 });

    const calls = await invokeCalls(page);
    expect(calls).toContainEqual(expect.objectContaining({ cmd: "neocodex_create_session" }));
  });

  test("send message drives the stream-token event into the chat", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => []);
    await mockCommand(page, "neocodex_list_archived", () => []);
    await mockCommand(page, "neocodex_send_message_stream", () => "流式回复");
    await page.goto("/");
    await page.getByTestId("sidebar-tab-sessions").click();
    await expect(page.getByTestId("sidebar-tab-sessions")).toBeVisible({ timeout: 10_000 });

    const textarea = page.locator("textarea").first();
    await textarea.fill("帮我分析 RAG 检索");
    await textarea.press("Enter");

    await expect(page.locator("main").last()).toContainText("帮我分析 RAG 检索", { timeout: 10_000 });

    await emitEvent(page, "neocodex_stream_token", "这是");
    await emitEvent(page, "neocodex_stream_token", "流式回复");
    await emitEvent(page, "neocodex_stream_done", { cancelled: false });

    await expect(page.locator("main").last()).toContainText("流式回复", { timeout: 10_000 });

    const calls = await invokeCalls(page);
    expect(calls).toContainEqual(
      expect.objectContaining({ cmd: "neocodex_send_message_stream", args: { content: "帮我分析 RAG 检索", permission_mode: "auto" } })
    );
  });

  test("settings advanced fields persist to localStorage", async ({ page }) => {
    await page.goto("/settings");
    await page.getByTestId("settings-tab-advanced").click();
    await expect(page.getByTestId("settings-maxTokens")).toBeVisible({ timeout: 10_000 });

    await page.getByTestId("settings-maxTokens").fill("16384");
    await page.getByTestId("settings-maxTokens").blur();

    const saved = await page.evaluate(() => {
      try {
        const raw = localStorage.getItem("neotrix_settings");
        return raw ? JSON.parse(raw) : null;
      } catch {
        return null;
      }
    });
    expect(saved).toEqual(expect.objectContaining({ maxTokens: 16384 }));
  });

  test("Cmd+P opens the file palette and picking a file mentions it", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => []);
    await mockCommand(page, "neocodex_search_files", () => ["src/lib.rs", "src/main.rs", "README.md"]);
    await page.goto("/");
    await page.getByTestId("sidebar-tab-sessions").click();
    await expect(page.getByTestId("sidebar-tab-sessions")).toBeVisible({ timeout: 10_000 });

    await page.keyboard.press("Meta+p");
    const palette = page.getByTestId("command-palette");
    await expect(palette).toBeVisible({ timeout: 10_000 });
    await expect(palette.getByTestId("palette-input")).toBeFocused();

    await page.getByText("src/lib.rs", { exact: true }).first().click();
    await expect(palette).not.toBeVisible();

    const calls = await invokeCalls(page);
    expect(calls).toContainEqual(expect.objectContaining({ cmd: "neocodex_search_files", args: { query: "" } }));
  });

  test("diff review button triggers cmd_diff_review and shows the score", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => []);
    await mockCommand(page, "cmd_diff_changed_files", () => ({
      staged: [],
      unstaged: [{ status: "M", path: "src/lib.rs" }],
      untracked: [],
    }));
    await mockCommand(page, "cmd_diff_review", () => ({
      score: 92,
      summary: "整体质量良好，存在少量可改进项。",
      critical: 0,
      warning: 1,
      info: 2,
      files: [
        {
          path: "src/lib.rs",
          issues: [{ line: 42, category: "complexity", severity: "warning", message: "函数过长", suggestion: "拆分为小函数" }],
        },
      ],
    }));
    await page.goto("/");
    await page.getByTestId("views-menu-btn").click();
    await page.getByTestId("views-menu-diff").click();
    await expect(page.getByTestId("diff-review")).toBeVisible({ timeout: 10_000 });

    await page.getByTestId("diff-review").click();
    const panel = page.getByTestId("diff-review-panel");
    await expect(panel).toBeVisible({ timeout: 10_000 });
    await expect(panel).toContainText("92");

    const calls = await invokeCalls(page);
    expect(calls).toContainEqual(expect.objectContaining({ cmd: "cmd_diff_review" }));
  });

  test("welcome empty state shows quick action chips that prefill the composer", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => []);
    await mockCommand(page, "neocodex_list_archived", () => []);
    await page.goto("/");
    await page.getByTestId("sidebar-tab-sessions").click();
    await expect(page.getByTestId("quick-actions")).toBeVisible({ timeout: 10_000 });
    await expect(page.getByText("分析项目结构")).toBeVisible();

    await page.getByTestId("quick-action-排查 Bug").click();
    const textarea = page.locator("textarea").first();
    await expect(textarea).toBeFocused();
    await expect(textarea).toHaveValue(/排查/);
  });

  test("streamed code block renders a per-code-block copy button", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => []);
    await mockCommand(page, "neocodex_list_archived", () => []);
    await mockCommand(page, "neocodex_send_message_stream", () => "```ts\nconst x = 1;\n```");
    await page.goto("/");
    await page.getByTestId("sidebar-tab-sessions").click();
    await expect(page.getByTestId("sidebar-tab-sessions")).toBeVisible({ timeout: 10_000 });

    const textarea = page.locator("textarea").first();
    await textarea.fill("给个代码示例");
    await textarea.press("Enter");
    await expect(page.locator("main").last()).toContainText("const x = 1;", { timeout: 10_000 });

    const copyBtn = page.locator(".codeblock-copy").first();
    await expect(copyBtn).toBeVisible();
    await expect(copyBtn).toHaveAttribute("title", "复制代码");
  });

  test("slash menu exposes parity commands and inserts into the composer", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => []);
    await mockCommand(page, "neocodex_list_archived", () => []);
    await page.goto("/");
    await page.getByTestId("sidebar-tab-sessions").click();
    await expect(page.getByTestId("sidebar-tab-sessions")).toBeVisible({ timeout: 10_000 });

    const textarea = page.locator("textarea").first();
    await textarea.fill("/mod");
    await expect(page.getByText("切换模型")).toBeVisible({ timeout: 10_000 });
    await textarea.press("Enter");
    await expect(textarea).toHaveValue("/model ");

    await textarea.fill("/init");
    await expect(page.getByText("初始化项目")).toBeVisible({ timeout: 10_000 });
    await textarea.press("Enter");
    await expect(textarea).toHaveValue("/init ");
  });

  test("capability health pane opens from the views menu and shows the 7-domain network", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => []);
    await mockCommand(page, "neocodex_list_archived", () => []);
    await mockCommand(page, "neocodex_health_report", () => ({
      mode: "Agent",
      turn_count: 8,
      tool_call_count: 12,
      tokens_used: 1000,
      context_usage: 0.42,
      context_turns: 5,
      provider_count: 1,
      provider_resolvable: true,
      provider_model: "gateway",
      session_writable: true,
      goals_active: true,
      cost_spent: 0.01,
      cost_budget: 1.0,
      subagent_results: 5,
      consciousness_attached: true,
      brain_attached: true,
      event_bus_attached: true,
      evolution_iterations: 3,
      tool_grounding_degraded: false,
    }));
    await page.goto("/");
    await page.getByTestId("sidebar-tab-sessions").click();
    await expect(page.getByTestId("sidebar-tab-sessions")).toBeVisible({ timeout: 10_000 });

    await page.getByTestId("views-menu-btn").click();
    await page.getByTestId("views-menu-capability").click();

    const pane = page.getByTestId("capability-health");
    await expect(pane).toBeVisible({ timeout: 10_000 });
    await expect(pane.getByText("能力网健康")).toBeVisible();
    await expect(pane.getByText("核心链路通畅")).toBeVisible();
    await expect(pane.locator("[data-domain='NT-CORE']")).toBeVisible();
    await expect(pane.locator("[data-domain='NT-IO']")).toBeVisible();
    await expect(pane.getByText(/自我进化 3 次/)).toBeVisible();
  });

  test("task pane toggles from the topbar and renders the empty state", async ({ page }) => {
    await mockCommand(page, "neocodex_list_sessions", () => []);
    await mockCommand(page, "neocodex_list_archived", () => []);
    await page.goto("/");
    await page.getByTestId("sidebar-tab-sessions").click();
    await expect(page.getByTestId("sidebar-tab-sessions")).toBeVisible({ timeout: 10_000 });

    await page.getByTestId("task-pane-toggle").click();
    const pane = page.getByTestId("task-pane");
    await expect(pane).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId("task-pane-empty")).toBeVisible();

    await page.getByTestId("task-pane-toggle").click();
    await expect(pane).not.toBeVisible();
  });

  test("accent color picker persists the accent setting", async ({ page }) => {
    await page.goto("/settings");
    await page.getByTestId("settings-tab-theme").click();
    await expect(page.getByTestId("settings-accent-emerald")).toBeVisible({ timeout: 10_000 });

    await page.getByTestId("settings-accent-emerald").click();

    const saved = await page.evaluate(() => {
      try {
        const raw = localStorage.getItem("neotrix_settings");
        return raw ? JSON.parse(raw) : null;
      } catch {
        return null;
      }
    });
    expect(saved).toEqual(expect.objectContaining({ accent: "emerald" }));
  });
});
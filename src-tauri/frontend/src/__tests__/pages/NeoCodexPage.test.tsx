import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import NeoCodexPage from "../../pages/NeoCodexPage";
import { useStore } from "../../stores";
import { mockInvoke, resetInvokeMocks } from "../../__tests__/tauriMock";

const renderPage = () =>
  render(
    <MemoryRouter>
      <NeoCodexPage />
    </MemoryRouter>
  );

describe("NeoCodexPage new-session handling", () => {
  beforeEach(() => {
    resetInvokeMocks();
    useStore.setState({ neocodexSessions: [], neocodexActiveSessionId: null });
    mockInvoke("neocodex_create_session", () => ({
      id: "s-cmdn",
      name: "新会话",
      mode: "Agent",
      message_count: 0,
      wire_path: "",
      updated_at: 0,
    }));
  });

  it("creates a session via Cmd+N even when the sidebar is collapsed", async () => {
    renderPage();

    const collapseBtn = await screen.findByTitle("收起侧栏");
    fireEvent.click(collapseBtn);
    expect(screen.queryByTitle("新建会话")).toBeNull();

    const createSpy = vi.fn(() => ({
      id: "s-cmdn",
      name: "新会话",
      mode: "Agent",
      message_count: 0,
      wire_path: "",
      updated_at: 0,
    }));
    mockInvoke("neocodex_create_session", createSpy);
    fireEvent.keyDown(window, { key: "n", metaKey: true });

    await waitFor(() => {
      expect(createSpy).toHaveBeenCalled();
      expect(useStore.getState().neocodexActiveSessionId).toBe("s-cmdn");
    });
  });

  it("creates a session when the neotrix:new-session event fires with the sidebar collapsed", async () => {
    renderPage();

    fireEvent.click(await screen.findByTitle("收起侧栏"));
    expect(screen.queryByTitle("新建会话")).toBeNull();

    const createSpy = vi.fn(() => ({
      id: "s-event",
      name: "新会话",
      mode: "Agent",
      message_count: 0,
      wire_path: "",
      updated_at: 0,
    }));
    mockInvoke("neocodex_create_session", createSpy);
    window.dispatchEvent(new CustomEvent("neotrix:new-session"));

    await waitFor(() => {
      expect(createSpy).toHaveBeenCalled();
      expect(useStore.getState().neocodexActiveSessionId).toBe("s-event");
    });
  });

  it("Cmd+1..9 switches to the session at that position (numbered switch)", async () => {
    mockInvoke("neocodex_list_sessions", () => [
      { id: "s-a", name: "会话A", mode: "Agent", message_count: 1, wire_path: "", created_at: 0, updated_at: 0 },
      { id: "s-b", name: "会话B", mode: "Agent", message_count: 1, wire_path: "", created_at: 0, updated_at: 0 },
      { id: "s-c", name: "会话C", mode: "Agent", message_count: 1, wire_path: "", created_at: 0, updated_at: 0 },
    ]);
    mockInvoke("neocodex_switch_session", () => null);
    mockInvoke("neocodex_get_session_messages", () => []);
    mockInvoke("neocodex_get_side_chat", () => []);
    renderPage();

    await waitFor(() => expect(useStore.getState().neocodexSessions.length).toBe(3));

    fireEvent.keyDown(window, { key: "2", metaKey: true });
    await waitFor(() => expect(useStore.getState().neocodexActiveSessionId).toBe("s-b"));

    fireEvent.keyDown(window, { key: "3", metaKey: true });
    await waitFor(() => expect(useStore.getState().neocodexActiveSessionId).toBe("s-c"));
  });
});

describe("NeoCodexPage session toolbar", () => {
  beforeEach(() => {
    resetInvokeMocks();
    useStore.setState({ neocodexSessions: [], neocodexActiveSessionId: "s-t" });
    mockInvoke("neocodex_list_sessions", () => [
      { id: "s-t", name: "初始标题", mode: "Agent", message_count: 0, wire_path: "/repo", created_at: 0, updated_at: 0 },
    ]);
    mockInvoke("neocodex_switch_session", () => null);
    mockInvoke("neocodex_get_session_messages", () => []);
    mockInvoke("neocodex_get_side_chat", () => []);
  });

  it("shows the active session title, project, and branch chip", async () => {
    mockInvoke("neocodex_git_status", () => ({ branch: "feat/g1-g5", dirty: false }));
    renderPage();
    const title = await screen.findByTestId("session-title");
    expect(title).toHaveTextContent("初始标题");
    expect((await screen.findAllByTitle(/工作区干净/)).length).toBeGreaterThan(0);
    expect((await screen.findAllByText("feat/g1-g5")).length).toBeGreaterThan(0);
  });

  it("renames the session via the inline title input (Enter commits)", async () => {
    const renameSpy = vi.fn(() => null);
    mockInvoke("neocodex_rename_session", renameSpy);
    renderPage();
    await screen.findByTestId("session-title");
    fireEvent.click(screen.getByTestId("session-title"));
    const input = screen.getByTestId("session-title-input");
    fireEvent.change(input, { target: { value: "新标题" } });
    fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(renameSpy).toHaveBeenCalledWith({ sessionId: "s-t", name: "新标题" }));
  });

  it("Escape cancels the inline rename without committing", async () => {
    const renameSpy = vi.fn(() => null);
    mockInvoke("neocodex_rename_session", renameSpy);
    renderPage();
    await screen.findByTestId("session-title");
    fireEvent.click(screen.getByTestId("session-title"));
    const input = screen.getByTestId("session-title-input");
    fireEvent.change(input, { target: { value: "不该提交" } });
    fireEvent.keyDown(input, { key: "Escape" });
    await waitFor(() => expect(screen.getByTestId("session-title")).toBeInTheDocument());
    expect(renameSpy).not.toHaveBeenCalled();
  });
});

describe("NeoCodexPage file palette (Cmd+P)", () => {
  beforeEach(() => {
    resetInvokeMocks();
    useStore.setState({ neocodexSessions: [], neocodexActiveSessionId: null });
    mockInvoke("neocodex_search_files", () => ["src/main.rs", "src/lib.rs", "README.md"]);
    mockInvoke("neocodex_switch_session", () => null);
    mockInvoke("neocodex_get_session_messages", () => []);
    mockInvoke("neocodex_get_side_chat", () => []);
  });

  it("Cmd+P opens the palette in file-search mode with file items", async () => {
    renderPage();
    await screen.findByTestId("session-toolbar").catch(() => {});
    fireEvent.keyDown(window, { key: "p", metaKey: true });
    const input = await screen.findByTestId("palette-input");
    expect(input).toHaveAttribute("placeholder", expect.stringContaining("搜索文件"));
    await waitFor(() => expect(screen.getByText("src/main.rs")).toBeInTheDocument());
    expect(screen.getByText("README.md")).toBeInTheDocument();
  });

  it("selecting a file dispatches neotrix:mention-file", async () => {
    const dispatched: string[] = [];
    window.addEventListener("neotrix:mention-file", (e) => dispatched.push((e as CustomEvent).detail));
    renderPage();
    await screen.findByTestId("session-toolbar").catch(() => {});
    fireEvent.keyDown(window, { key: "p", metaKey: true });
    fireEvent.click(await screen.findByText("src/lib.rs"));
    await waitFor(() => expect(dispatched).toContain("src/lib.rs"));
    window.removeEventListener("neotrix:mention-file", () => {});
  });

  it("Cmd+K still opens the command palette (default mode)", async () => {
    renderPage();
    await screen.findByTestId("session-toolbar").catch(() => {});
    fireEvent.keyDown(window, { key: "k", metaKey: true });
    const input = await screen.findByTestId("palette-input");
    expect(input).toHaveAttribute("placeholder", expect.stringContaining("搜索会话"));
  });
});

describe("NeoCodexPage destructive delete confirmation", () => {
  beforeEach(() => {
    resetInvokeMocks();
    useStore.setState({ neocodexSessions: [], neocodexActiveSessionId: "s-del" });
    mockInvoke("neocodex_list_sessions", () => [
      { id: "s-del", name: "重要会话", mode: "Agent", message_count: 3, wire_path: "/repo", created_at: 0, updated_at: 0 },
    ]);
    mockInvoke("neocodex_switch_session", () => null);
    mockInvoke("neocodex_get_session_messages", () => []);
    mockInvoke("neocodex_get_side_chat", () => []);
    mockInvoke("neocodex_delete_session", () => null);
  });

  it("⌘W opens a confirmation dialog instead of deleting immediately", async () => {
    const deleteSpy = vi.fn(() => null);
    mockInvoke("neocodex_delete_session", deleteSpy);
    renderPage();
    await screen.findByTestId("session-title");

    fireEvent.keyDown(window, { key: "w", metaKey: true });
    const dialog = await screen.findByRole("dialog", { name: "删除会话确认" });
    expect(dialog).toHaveTextContent("重要会话");
    expect(deleteSpy).not.toHaveBeenCalled();
  });

  it("cancel closes the dialog without deleting", async () => {
    const deleteSpy = vi.fn(() => null);
    mockInvoke("neocodex_delete_session", deleteSpy);
    renderPage();
    await screen.findByTestId("session-title");

    fireEvent.keyDown(window, { key: "w", metaKey: true });
    fireEvent.click(await screen.findByTestId("confirm-delete-cancel"));

    await waitFor(() => expect(screen.queryByRole("dialog", { name: "删除会话确认" })).toBeNull());
    expect(deleteSpy).not.toHaveBeenCalled();
  });

  it("confirm executes the delete for the targeted session", async () => {
    const deleteSpy = vi.fn(() => null);
    mockInvoke("neocodex_delete_session", deleteSpy);
    renderPage();
    await screen.findByTestId("session-title");

    fireEvent.keyDown(window, { key: "w", metaKey: true });
    fireEvent.click(await screen.findByTestId("confirm-delete-confirm"));

    await waitFor(() => expect(deleteSpy).toHaveBeenCalledWith({ sessionId: "s-del" }));
  });
});

describe("NeoCodexPage checkpoint timeline", () => {
  beforeEach(() => {
    resetInvokeMocks();
    useStore.setState({ neocodexSessions: [], neocodexActiveSessionId: "s-tl" });
    mockInvoke("neocodex_list_sessions", () => [
      { id: "s-tl", name: "时间线会话", mode: "Agent", message_count: 4, wire_path: "/repo", created_at: 0, updated_at: 0 },
    ]);
    mockInvoke("neocodex_switch_session", () => null);
    mockInvoke("neocodex_get_session_messages", () => []);
    mockInvoke("neocodex_get_side_chat", () => []);
    mockInvoke("neocodex_checkpoint_list", () => [
      { id: "s-tl-1000.jsonl", created_at: 1000, message_count: 2 },
      { id: "s-tl-2000.jsonl", created_at: 2000, message_count: 4 },
    ]);
  });

  it("timeline button lists checkpoints for the active session", async () => {
    renderPage();
    await screen.findByTestId("session-title");

    fireEvent.click(screen.getByTestId("timeline-open"));
    expect(await screen.findByTestId("timeline-panel")).toBeInTheDocument();
    expect(screen.getByText("2 条消息")).toBeInTheDocument();
    expect(screen.getByText("4 条消息")).toBeInTheDocument();
  });

  it("restore requires a confirm step before calling the backend", async () => {
    const restoreSpy = vi.fn(() => []);
    mockInvoke("neocodex_checkpoint_restore", restoreSpy);
    renderPage();
    await screen.findByTestId("session-title");

    fireEvent.click(screen.getByTestId("timeline-open"));
    fireEvent.click(await screen.findByTestId("timeline-restore-s-tl-1000.jsonl"));
    expect(await screen.findByTestId("timeline-restore-confirm")).toBeInTheDocument();
    expect(restoreSpy).not.toHaveBeenCalled();

    fireEvent.click(screen.getByTestId("timeline-restore-confirm-btn"));
    await waitFor(() => expect(restoreSpy).toHaveBeenCalledWith({ sessionId: "s-tl", checkpointId: "s-tl-1000.jsonl" }));
  });

  it("cancel in restore confirm aborts without calling backend", async () => {
    const restoreSpy = vi.fn(() => []);
    mockInvoke("neocodex_checkpoint_restore", restoreSpy);
    renderPage();
    await screen.findByTestId("session-title");

    fireEvent.click(screen.getByTestId("timeline-open"));
    fireEvent.click(await screen.findByTestId("timeline-restore-s-tl-1000.jsonl"));
    fireEvent.click(await screen.findByTestId("timeline-restore-cancel"));

    await waitFor(() => expect(screen.queryByTestId("timeline-restore-confirm")).toBeNull());
    expect(restoreSpy).not.toHaveBeenCalled();
  });
});

describe("NeoCodexPage context compaction", () => {
  beforeEach(() => {
    resetInvokeMocks();
    useStore.setState({ neocodexSessions: [], neocodexActiveSessionId: "s-cmp" });
    mockInvoke("neocodex_list_sessions", () => [
      { id: "s-cmp", name: "压缩会话", mode: "Agent", message_count: 10, wire_path: "/repo", created_at: 0, updated_at: 0 },
    ]);
    mockInvoke("neocodex_switch_session", () => null);
    mockInvoke("neocodex_get_session_messages", () => []);
    mockInvoke("neocodex_get_side_chat", () => []);
    mockInvoke("neocodex_health_report", () => ({ context_usage: 0.93, turn_count: 9, tool_call_count: 12, tokens_used: 48000, context_turns: 9, provider_count: 0, provider_resolvable: false, provider_model: "mock", session_writable: true, goals_active: false, cost_spent: 0, cost_budget: 0 }));
  });

  it("shows the compact action when context usage is near the limit", async () => {
    renderPage();
    fireEvent.click(await screen.findByTestId("usage-toggle"));
    expect(await screen.findByTestId("usage-compact")).toBeInTheDocument();
  });

  it("compact button calls neocodex_compact_session for the active session", async () => {
    const compactSpy = vi.fn(() => []);
    mockInvoke("neocodex_compact_session", compactSpy);
    renderPage();
    await screen.findByTestId("session-title");

    fireEvent.click(screen.getByTestId("usage-toggle"));
    fireEvent.click(await screen.findByTestId("usage-compact"));
    await waitFor(() => expect(compactSpy).toHaveBeenCalledWith({ sessionId: "s-cmp" }));
  });
});

describe("NeoCodexPage task pane", () => {
  beforeEach(() => {
    resetInvokeMocks();
    useStore.setState({ neocodexSessions: [], neocodexActiveSessionId: "s-task" });
    mockInvoke("neocodex_list_sessions", () => [
      { id: "s-task", name: "任务会话", mode: "Agent", message_count: 0, wire_path: "/repo", created_at: 0, updated_at: 0 },
    ]);
    mockInvoke("neocodex_switch_session", () => null);
    mockInvoke("neocodex_get_session_messages", () => []);
    mockInvoke("neocodex_get_side_chat", () => []);
  });

  it("task pane toggle opens the live task panel", async () => {
    renderPage();
    await screen.findByTestId("session-title");

    fireEvent.click(screen.getByTestId("rail-btn-task"));
    expect(await screen.findByTestId("task-pane")).toBeInTheDocument();
    expect(screen.getByTestId("task-pane-empty")).toBeInTheDocument();
  });
});

describe("NeoCodexPage — status bar parity", () => {
  beforeEach(() => {
    resetInvokeMocks();
    useStore.setState({ neocodexSessions: [], neocodexActiveSessionId: null });
    mockInvoke("neocodex_health_report", () => ({ context_usage: 0.1, provider_model: "mock-lgm", tokens_used: 0, cost_spent: 0, cost_budget: 0 }));
    mockInvoke("neocodex_create_session", () => ({ id: "s-sb", name: "会话", mode: "Agent", message_count: 0, wire_path: "", updated_at: 0 }));
    mockInvoke("neocodex_switch_session", () => {});
    mockInvoke("neocodex_get_session_messages", () => []);
    mockInvoke("neocodex_get_side_chat", () => []);
  });

  it("renders harness permission chip reflecting settings.permissionMode", async () => {
    useStore.setState({ settings: { ...(useStore.getState().settings || {}), permissionMode: "accept" } });
    renderPage();
    await screen.findByTestId("status-bar");
    expect(screen.getByTestId("status-bar")).toHaveTextContent("接受");
  });

  it("does not show sync indicator at idle; no crash", async () => {
    renderPage();
    await screen.findByTestId("status-bar");
    expect(screen.queryByTestId("status-sync")).not.toBeInTheDocument();
  });
});

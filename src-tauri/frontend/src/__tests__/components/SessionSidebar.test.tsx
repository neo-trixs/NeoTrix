import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SessionSidebar } from "../../components/neocodex/SessionSidebar";
import { useStore } from "../../stores";
import { mockInvoke, resetInvokeMocks, sessionFixture } from "../tauriMock";

function renderSidebar() {
  const onSelect = vi.fn();
  const onDelete = vi.fn();
  const onArchive = vi.fn();
  render(<SessionSidebar activeSessionId={null} onSessionSelect={onSelect} onSessionDelete={onDelete} onSessionArchive={onArchive} />);
  return { onSelect, onDelete, onArchive };
}

beforeEach(() => {
  resetInvokeMocks();
  useStore.setState({ notifications: [] });
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("SessionSidebar — session interactions", () => {
  it("lists sessions from backend", async () => {
    sessionFixture();
    renderSidebar();
    await waitFor(() => expect(screen.getByText("重构缓存层")).toBeInTheDocument());
    expect(screen.getByText("调研 RAG")).toBeInTheDocument();
  });

  it("new session dialog creates via invoke and calls onSessionSelect", async () => {
    sessionFixture();
    const { onSelect } = renderSidebar();
    let created: any = null;
    mockInvoke("neocodex_create_session", (args) => {
      created = args;
      return { id: "s-new", name: args.name, mode: "Agent", message_count: 0, wire_path: "/sessions/s-new.jsonl", created_at: 0, updated_at: 0 };
    });
    await waitFor(() => expect(screen.getByTitle("新建会话")).toBeInTheDocument());
    await userEvent.click(screen.getByTitle("新建会话"));
    await userEvent.type(screen.getByPlaceholderText("会话名称"), "新会话测试");
    await userEvent.keyboard("{Enter}");
    await waitFor(() => expect(created).toEqual({ name: "新会话测试" }));
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ id: "s-new" }));
  });

  it("delete session delegates to onDelete and removes item", async () => {
    sessionFixture();
    const { onDelete } = renderSidebar();
    await waitFor(() => expect(screen.getByText("重构缓存层")).toBeInTheDocument());
    const rows = screen.getAllByTitle("删除会话");
    await userEvent.click(rows[0]);
    expect(onDelete).toHaveBeenCalledWith("s-1");
  });

  it("archive session invokes backend and moves item to archived", async () => {
    sessionFixture();
    const { onArchive } = renderSidebar();
    const archived: string[] = [];
    mockInvoke("neocodex_archive_session", (args) => { archived.push(args.sessionId); return "ok"; });
    await waitFor(() => expect(screen.getByText("重构缓存层")).toBeInTheDocument());
    const archiveBtns = screen.getAllByTitle("归档会话");
    await userEvent.click(archiveBtns[0]);
    await waitFor(() => expect(archived).toContain("s-1"));
    expect(onArchive).toHaveBeenCalledWith("s-1");
  });

  it("archived section shows archived sessions and restores them", async () => {
    sessionFixture();
    const { onSelect } = renderSidebar();
    const restored: string[] = [];
    mockInvoke("neocodex_restore_session", (args) => { restored.push(args.sessionId); return "ok"; });
    await waitFor(() => expect(screen.getByText(/已归档 \(1\)/)).toBeInTheDocument());
    await userEvent.click(screen.getByText(/已归档 \(1\)/));
    await waitFor(() => expect(screen.getByText("旧会话")).toBeInTheDocument());
    await userEvent.click(screen.getByTitle("恢复会话"));
    await waitFor(() => expect(restored).toContain("s-3"));
  });

  it("deleting from archived uses delete handler", async () => {
    sessionFixture();
    const { onDelete } = renderSidebar();
    await waitFor(() => expect(screen.getByText(/已归档 \(1\)/)).toBeInTheDocument());
    await userEvent.click(screen.getByText(/已归档 \(1\)/));
    const delBtn = screen.getAllByTitle("永久删除");
    await userEvent.click(delBtn[0]);
    expect(onDelete).toHaveBeenCalledWith("s-3");
  });

  it("rename updates the session name via backend", async () => {
    sessionFixture();
    const renamed: Array<{ sessionId: string; name: string }> = [];
    mockInvoke("neocodex_rename_session", (args) => { renamed.push(args); return { ...args, id: args.sessionId }; });
    renderSidebar();
    await waitFor(() => expect(screen.getByText("重构缓存层")).toBeInTheDocument());
    const renameBtns = screen.getAllByTitle("重命名");
    await userEvent.click(renameBtns[0]);
    const input = screen.getByDisplayValue("重构缓存层");
    await userEvent.clear(input);
    await userEvent.type(input, "新名字");
    await userEvent.keyboard("{Enter}");
    await waitFor(() => expect(renamed).toEqual([{ sessionId: "s-1", name: "新名字" }]));
    await waitFor(() => expect(screen.getByText("新名字")).toBeInTheDocument());
  });

  it("pin toggles pinned state", async () => {
    sessionFixture();
    renderSidebar();
    await waitFor(() => expect(screen.getByText("重构缓存层")).toBeInTheDocument());
    const pinBtn = screen.getAllByTitle("置顶");
    await userEvent.click(pinBtn[0]);
    await waitFor(() => expect(screen.getByText("📌 置顶")).toBeInTheDocument());
  });

  it("search filters sessions", async () => {
    sessionFixture();
    renderSidebar();
    await waitFor(() => expect(screen.getByText("重构缓存层")).toBeInTheDocument());
    await userEvent.type(screen.getByPlaceholderText("搜索会话…"), "RAG");
    await waitFor(() => expect(screen.getByText("调研 RAG")).toBeInTheDocument());
    expect(screen.queryByText("重构缓存层")).not.toBeInTheDocument();
  });

  it("refreshes its list when neotrix:sessions-changed fires (Cmd+N sync)", async () => {
    let sessions = [
      { id: "s-1", name: "重构缓存层", mode: "Agent", message_count: 3, wire_path: "/sessions/s-1.jsonl", created_at: 0, updated_at: 1700000000 },
    ];
    mockInvoke("neocodex_list_sessions", () => sessions);
    renderSidebar();
    await waitFor(() => expect(screen.getByText("重构缓存层")).toBeInTheDocument());
    expect(screen.queryByText("新会话CmdN")).not.toBeInTheDocument();

    // Page creates a session via Cmd+N, updates its store, then dispatches the event.
    sessions = [
      { id: "s-2", name: "新会话CmdN", mode: "Agent", message_count: 0, wire_path: "/sessions/s-2.jsonl", created_at: 0, updated_at: 1700000001 },
      ...sessions,
    ];
    fireEvent(window, new Event("neotrix:sessions-changed"));
    await waitFor(() => expect(screen.getByText("新会话CmdN")).toBeInTheDocument());
  });

  it("imports a session JSON via the hidden file input and shows a toast", async () => {
    const importSpy = vi.fn((args: { json: string }) => "imp-1");
    mockInvoke("cmd_session_import_json", importSpy);
    const notify = vi.fn();
    useStore.setState({ addNotification: notify });
    renderSidebar();
    await waitFor(() => expect(screen.getByTitle("导入会话 JSON")).toBeInTheDocument());
    const input = screen.getByTestId("session-import-input") as HTMLInputElement;
    const file = new File([JSON.stringify({ format_version: 1, sessions: [{ name: "导入的会话", message_count: 1 }] })], "export.json", { type: "application/json" });
    fireEvent.change(input, { target: { files: [file] } });
    await waitFor(() => expect(importSpy).toHaveBeenCalled());
    const arg = importSpy.mock.calls[0][0];
    expect(JSON.parse(arg.json).format_version).toBe(1);
    expect(notify).toHaveBeenCalledWith(expect.objectContaining({ type: "success" }));
  });

  it("import failure shows an error toast", async () => {
    mockInvoke("cmd_session_import_json", () => {
      throw new Error("bad json");
    });
    const notify = vi.fn();
    useStore.setState({ addNotification: notify });
    renderSidebar();
    await waitFor(() => expect(screen.getByTitle("导入会话 JSON")).toBeInTheDocument());
    const input = screen.getByTestId("session-import-input") as HTMLInputElement;
    const file = new File(["not-json"], "bad.json", { type: "application/json" });
    fireEvent.change(input, { target: { files: [file] } });
    await waitFor(() => expect(notify).toHaveBeenCalledWith(expect.objectContaining({ type: "error" })));
  });
});

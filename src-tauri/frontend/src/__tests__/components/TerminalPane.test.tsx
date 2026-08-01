import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { TerminalPane } from "../../components/neocodex/TerminalPane";
import { mockInvoke, resetInvokeMocks } from "../../__tests__/tauriMock";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (event: string, cb: any) => {
    const handlers = (globalThis as any).__eventHandlers ??= {};
    handlers[event] ??= [];
    handlers[event].push(cb);
    return () => {
      const list = handlers[event];
      const i = list.indexOf(cb);
      if (i >= 0) list.splice(i, 1);
    };
  }),
}));

function emit(event: string, payload: any) {
  const list = (globalThis as any).__eventHandlers?.[event];
  list?.slice().forEach((h: any) => h({ payload }));
}

describe("TerminalPane", () => {
  beforeEach(() => {
    resetInvokeMocks();
    (globalThis as any).__eventHandlers = {};
    mockInvoke("pty_spawn", () => "s-pty-1");
    mockInvoke("pty_write", () => null);
    mockInvoke("pty_close", () => null);
  });

  it("spawns a pty and renders streamed output", async () => {
    render(<TerminalPane />);
    await waitFor(() => expect(screen.getByTestId("terminal-input")).toBeInTheDocument());
    emit("pty-output-s-pty-1", "hello world\n");
    await waitFor(() => expect(screen.getByText("hello world")).toBeInTheDocument());
  });

  it("sends the entered command on Enter", async () => {
    const writeSpy = vi.fn(() => null);
    mockInvoke("pty_write", writeSpy);
    render(<TerminalPane />);
    await waitFor(() => expect(screen.getByTestId("terminal-input")).toBeInTheDocument());
    const input = screen.getByTestId("terminal-input");
    fireEvent.change(input, { target: { value: "npm test" } });
    fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(writeSpy).toHaveBeenCalledWith({ sessionId: "s-pty-1", data: "npm test\n" }));
    expect(screen.getByText("$ npm test")).toBeInTheDocument();
  });

  it("clears output on Ctrl+L", async () => {
    const writeSpy = vi.fn(() => null);
    mockInvoke("pty_write", writeSpy);
    render(<TerminalPane />);
    await waitFor(() => expect(screen.getByTestId("terminal-input")).toBeInTheDocument());
    const input = screen.getByTestId("terminal-input");
    fireEvent.change(input, { target: { value: "echo hi" } });
    fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(screen.getByText("$ echo hi")).toBeInTheDocument());
    fireEvent.keyDown(input, { key: "l", ctrlKey: true });
    expect(screen.queryByText("$ echo hi")).not.toBeInTheDocument();
  });

  it("shows an error state when pty spawn fails", async () => {
    mockInvoke("pty_spawn", () => {
      throw new Error("spawn failed");
    });
    render(<TerminalPane />);
    await waitFor(() => expect(screen.getByText(/spawn failed/)).toBeInTheDocument());
  });

  it("closes the pty on unmount", async () => {
    const closeSpy = vi.fn(() => null);
    mockInvoke("pty_close", closeSpy);
    const { unmount } = render(<TerminalPane />);
    await waitFor(() => expect(screen.getByTestId("terminal-input")).toBeInTheDocument());
    act(() => unmount());
    expect(closeSpy).toHaveBeenCalledWith({ sessionId: "s-pty-1" });
  });

  it("adds a terminal tab that spawns its own pty session", async () => {
    const spawnSpy = vi.fn(() => "s-pty-2");
    mockInvoke("pty_spawn", spawnSpy);
    render(<TerminalPane />);
    await waitFor(() => expect(screen.getByTestId("terminal-input")).toBeInTheDocument());
    fireEvent.click(screen.getByTestId("terminal-add"));
    // New tab active → second pty spawned.
    await waitFor(() => expect(spawnSpy).toHaveBeenCalledTimes(2));
    // Tab bar shows 2 tabs.
    expect(screen.getAllByRole("tab")).toHaveLength(2);
  });

  it("switching tabs keeps prior tab output intact (tabs stay mounted)", async () => {
    mockInvoke("pty_spawn", () => "s-pty-1");
    render(<TerminalPane />);
    await waitFor(() => expect(screen.getByTestId("terminal-input")).toBeInTheDocument());
    emit("pty-output-s-pty-1", "first tab\n");
    await waitFor(() => expect(screen.getByText("first tab")).toBeInTheDocument());
    // Add a second tab and switch back to the first.
    fireEvent.click(screen.getByTestId("terminal-add"));
    const tabs = screen.getAllByRole("tab");
    fireEvent.click(tabs[0]);
    expect(screen.getByText("first tab")).toBeInTheDocument();
  });

  it("closing the last tab replaces it with a fresh one (never empty)", async () => {
    const closeSpy = vi.fn(() => null);
    mockInvoke("pty_close", closeSpy);
    render(<TerminalPane />);
    await waitFor(() => expect(screen.getByTestId("terminal-input")).toBeInTheDocument());
    const closeBtn = screen.getByTitle("关闭标签");
    fireEvent.click(closeBtn);
    // A replacement tab is mounted with a new pty.
    await waitFor(() => expect(screen.getByTestId("terminal-input")).toBeInTheDocument());
    expect(screen.getAllByRole("tab")).toHaveLength(1);
  });
});

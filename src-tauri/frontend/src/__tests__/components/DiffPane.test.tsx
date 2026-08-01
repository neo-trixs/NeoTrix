import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { DiffPane } from "../../components/neocodex/DiffPane";
import { mockInvoke, resetInvokeMocks } from "../tauriMock";

beforeEach(() => {
  resetInvokeMocks();
  vi.restoreAllMocks();
});

describe("DiffPane", () => {
  it("renders scope tabs and loads unstaged diff by default", async () => {
    mockInvoke("cmd_diff_unstaged", () => [{ type: "added", content: "fn new()", line_start: 0 }]);
    render(<DiffPane />);
    expect(screen.getByText("未暂存")).toBeInTheDocument();
    await screen.findByText("fn new()");
  });

  it("switches scope to staged and shows empty state", async () => {
    mockInvoke("cmd_diff_staged", () => []);
    render(<DiffPane />);
    fireEvent.click(screen.getByText("已暂存"));
    await screen.findByText("无改动");
  });

  it("stages all changes via the 暂存 action", async () => {
    const stageSpy = vi.fn(() => ["a.rs"]);
    mockInvoke("cmd_diff_stage", stageSpy);
    render(<DiffPane />);
    await waitFor(() => expect(screen.getByTestId("diff-stage-all")).not.toBeDisabled());
    fireEvent.click(screen.getByTestId("diff-stage-all"));
    await waitFor(() => expect(stageSpy).toHaveBeenCalled());
  });

  it("unstage action calls cmd_diff_unstage", async () => {
    const unstageSpy = vi.fn(() => []);
    mockInvoke("cmd_diff_unstage", unstageSpy);
    render(<DiffPane />);
    await waitFor(() => expect(screen.getByTestId("diff-unstage-all")).not.toBeDisabled());
    fireEvent.click(screen.getByTestId("diff-unstage-all"));
    await waitFor(() => expect(unstageSpy).toHaveBeenCalled());
  });

  it("commit button disabled until a message is typed, then commits", async () => {
    const commitSpy = vi.fn(() => null);
    mockInvoke("cmd_diff_commit", commitSpy);
    render(<DiffPane />);
    const commitBtn = screen.getByTestId("diff-commit");
    expect(commitBtn).toBeDisabled();
    fireEvent.change(screen.getByTestId("diff-commit-msg"), { target: { value: "fix: cache" } });
    expect(commitBtn).not.toBeDisabled();
    fireEvent.click(commitBtn);
    await waitFor(() => expect(commitSpy).toHaveBeenCalledWith({ message: "fix: cache" }));
  });

  it("file scope shows path input and 查看 button", () => {
    render(<DiffPane />);
    fireEvent.click(screen.getByText("文件"));
    expect(screen.getByPlaceholderText("文件路径（相对仓库根）")).toBeInTheDocument();
    expect(screen.getByText("查看")).toBeInTheDocument();
  });

  it("does not fire cmd_diff_file per keystroke in the path input (debounce/storm fix)", async () => {
    const fileSpy = vi.fn(() => [{ type: "added", content: "x", line_start: 0 }]);
    mockInvoke("cmd_diff_file", fileSpy);
    render(<DiffPane />);
    fireEvent.click(screen.getByText("文件"));
    const input = screen.getByPlaceholderText("文件路径（相对仓库根）");
    // Typing several chars must NOT trigger an IPC each time.
    fireEvent.change(input, { target: { value: "s" } });
    fireEvent.change(input, { target: { value: "sr" } });
    fireEvent.change(input, { target: { value: "src" } });
    expect(fileSpy).not.toHaveBeenCalled();
    // Only the explicit 查看 button triggers the load.
    fireEvent.click(screen.getByText("查看"));
    await waitFor(() => expect(fileSpy).toHaveBeenCalledTimes(1));
    expect(fileSpy).toHaveBeenCalledWith({ path: "src" });
  });
});

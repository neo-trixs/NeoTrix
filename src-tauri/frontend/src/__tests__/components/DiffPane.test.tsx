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

  it("renders the changed-file list grouped by staged/unstaged/untracked", async () => {
    mockInvoke("cmd_diff_changed_files", () => ({
      staged: [{ status: "M", path: "src/main.rs" }],
      unstaged: [{ status: "M", path: "README.md" }],
      untracked: [{ status: "??", path: "notes.txt" }],
    }));
    render(<DiffPane />);
    expect(await screen.findByText("已暂存 (1)")).toBeInTheDocument();
    expect(screen.getByText("未暂存 (1)")).toBeInTheDocument();
    expect(screen.getByText("未跟踪 (1)")).toBeInTheDocument();
    expect(screen.getByText("src/main.rs")).toBeInTheDocument();
    expect(screen.getByText("README.md")).toBeInTheDocument();
    expect(screen.getByText("notes.txt")).toBeInTheDocument();
  });

  it("clicking a file in the list loads its per-file diff (cmd_diff_file)", async () => {
    const fileSpy = vi.fn(() => [{ type: "added", content: "let x = 1;", line_start: 0 }]);
    mockInvoke("cmd_diff_changed_files", () => ({
      staged: [],
      unstaged: [{ status: "M", path: "src/lib.rs" }],
      untracked: [],
    }));
    mockInvoke("cmd_diff_file", fileSpy);
    render(<DiffPane />);
    fireEvent.click(await screen.findByText("src/lib.rs"));
    await waitFor(() => expect(fileSpy).toHaveBeenCalledWith({ path: "src/lib.rs" }));
    expect(await screen.findByText("let x = 1;")).toBeInTheDocument();
  });

  it("shows empty-file state when the repo has no changes", async () => {
    mockInvoke("cmd_diff_changed_files", () => ({ staged: [], unstaged: [], untracked: [] }));
    render(<DiffPane />);
    expect(await screen.findByText("无改动文件")).toBeInTheDocument();
  });

  it("refreshes the file list after staging (stage → list reload)", async () => {
    const stageSpy = vi.fn(() => ["src/lib.rs"]);
    const filesSpy = vi.fn(() => ({ staged: [{ status: "M", path: "src/lib.rs" }], unstaged: [], untracked: [] }));
    mockInvoke("cmd_diff_stage", stageSpy);
    mockInvoke("cmd_diff_changed_files", filesSpy);
    render(<DiffPane />);
    await waitFor(() => expect(screen.getByTestId("diff-stage-all")).not.toBeDisabled());
    fireEvent.click(screen.getByTestId("diff-stage-all"));
    await waitFor(() => expect(filesSpy).toHaveBeenCalledTimes(2));
  });

  it("base scope shows branch input and loads cmd_diff_base on 对比", async () => {
    const baseSpy = vi.fn(() => [{ type: "added", content: "base-line", line_start: 0 }]);
    mockInvoke("cmd_diff_base", baseSpy);
    render(<DiffPane />);
    fireEvent.click(screen.getByText("基线分支"));
    const input = screen.getByTestId("diff-base-branch");
    expect(input).toBeInTheDocument();
    expect(input).toHaveValue("main");
    fireEvent.change(input, { target: { value: "origin/main" } });
    fireEvent.click(screen.getByTestId("diff-base-load"));
    await waitFor(() => expect(baseSpy).toHaveBeenCalledWith({ base: "origin/main" }));
    expect(await screen.findByText("base-line")).toBeInTheDocument();
  });

  it("base scope does not auto-fire per keystroke (only explicit 对比)", async () => {
    const baseSpy = vi.fn(() => []);
    mockInvoke("cmd_diff_base", baseSpy);
    render(<DiffPane />);
    fireEvent.click(screen.getByText("基线分支"));
    const input = screen.getByTestId("diff-base-branch");
    // Typing without pressing Enter/对比 must not fire extra IPC beyond the
    // scope-change auto-load (which used the default "main").
    const callsAfterScope = baseSpy.mock.calls.length;
    fireEvent.change(input, { target: { value: "m" } });
    fireEvent.change(input, { target: { value: "ma" } });
    fireEvent.change(input, { target: { value: "main" } });
    expect(baseSpy.mock.calls.length).toBe(callsAfterScope);
  });
});

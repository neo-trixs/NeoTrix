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
    const input = screen.getByPlaceholderText("文件路径（相对仓库根）");    // Typing several chars must NOT trigger an IPC each time.
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

  it("AI 审查 button calls cmd_diff_review and renders the review panel", async () => {
    const reviewSpy = vi.fn(() => ({
      pr_title: "工作区变更",
      total_files: 1,
      total_issues: 2,
      critical: 1,
      warning: 1,
      info: 0,
      score: 80,
      summary: "Found 2 issues (1 critical, 1 warning, 0 info) across 1 files. Score: 80/100",
      files: [
        {
          path: "src/main.rs",
          additions: 3,
          deletions: 0,
          issues: [
            { line: 4, severity: "critical", category: "security", message: "Possible hardcoded credential", suggestion: "Use env vars" },
            { line: 8, severity: "warning", category: "debug", message: "Debug print statement left in code", suggestion: null },
          ],
        },
      ],
    }));
    mockInvoke("cmd_diff_review", reviewSpy);
    render(<DiffPane />);
    const reviewBtn = screen.getByTestId("diff-review");
    await waitFor(() => expect(reviewBtn).not.toBeDisabled());
    fireEvent.click(reviewBtn);
    await waitFor(() => expect(reviewSpy).toHaveBeenCalled());
    expect(await screen.findByTestId("diff-review-panel")).toBeInTheDocument();
    expect(screen.getByText("得分 80/100")).toBeInTheDocument();
    expect(screen.getByText("src/main.rs")).toBeInTheDocument();
    expect(screen.getByText("Possible hardcoded credential")).toBeInTheDocument();
    expect(screen.getByText("Debug print statement left in code")).toBeInTheDocument();
  });

  it("AI 审查 shows empty-review ok state", async () => {
    mockInvoke("cmd_diff_review", () => ({
      pr_title: "工作区变更",
      total_files: 0,
      total_issues: 0,
      critical: 0,
      warning: 0,
      info: 0,
      score: 100,
      summary: "Found 0 issues across 0 files. Score: 100/100",
      files: [],
    }));
    render(<DiffPane />);
    fireEvent.click(screen.getByTestId("diff-review"));
    expect(await screen.findByTestId("diff-review-panel")).toBeInTheDocument();
    expect(screen.getByText(/未发现问题/)).toBeInTheDocument();
  });

  it("per-file accept stages only that file (cmd_diff_stage with path)", async () => {
    const stageSpy = vi.fn(() => []);
    mockInvoke("cmd_diff_changed_files", () => ({
      staged: [],
      unstaged: [{ status: "M", path: "src/a.rs" }],
      untracked: [],
    }));
    mockInvoke("cmd_diff_stage", stageSpy);
    render(<DiffPane />);
    fireEvent.click(await screen.findByTestId("diff-accept-src/a.rs"));
    await waitFor(() => expect(stageSpy).toHaveBeenCalledWith({ paths: ["src/a.rs"] }));
  });

  it("per-file reject restores that file (cmd_diff_restore with path)", async () => {
    const restoreSpy = vi.fn(() => []);
    mockInvoke("cmd_diff_changed_files", () => ({
      staged: [],
      unstaged: [{ status: "M", path: "src/a.rs" }],
      untracked: [],
    }));
    mockInvoke("cmd_diff_restore", restoreSpy);
    render(<DiffPane />);
    fireEvent.click(await screen.findByTestId("diff-reject-src/a.rs"));
    await waitFor(() => expect(restoreSpy).toHaveBeenCalledWith({ paths: ["src/a.rs"] }));
  });

  it("untracked files show accept (stage add) and reject", async () => {
    mockInvoke("cmd_diff_changed_files", () => ({
      staged: [],
      unstaged: [],
      untracked: [{ status: "??", path: "notes.txt" }],
    }));
    render(<DiffPane />);
    expect(await screen.findByTestId("diff-reject-notes.txt")).toBeInTheDocument();
    // P2-4: untracked new files must be stageable (git add) — parity with
    // Claude/Codex review, which can accept new files into the review set.
    expect(screen.getByTestId("diff-accept-notes.txt")).toBeInTheDocument();
  });
});

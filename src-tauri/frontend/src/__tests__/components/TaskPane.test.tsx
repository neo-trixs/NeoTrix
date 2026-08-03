import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TaskPane } from "../../components/neocodex/TaskPane";

describe("TaskPane", () => {
  it("shows empty state with guidance when no steps exist", () => {
    render(<TaskPane steps={[]} startedAt={null} />);
    expect(screen.getByTestId("task-pane")).toBeInTheDocument();
    expect(screen.getByTestId("task-pane-empty")).toBeInTheDocument();
    expect(screen.getByText(/总计: 0/)).toBeInTheDocument();
    expect(screen.getByText(/运行中: 0/)).toBeInTheDocument();
    expect(screen.getByText(/完成: 0/)).toBeInTheDocument();
    expect(screen.getByText(/失败: 0/)).toBeInTheDocument();
  });

  it("lists steps with running status and rendered args", () => {
    render(
      <TaskPane
        startedAt={Date.now()}
        steps={[{ id: "tool-0", name: "read", args: "src/main.rs", startedAt: Date.now(), status: "running" }]}
      />
    );
    expect(screen.getByTestId("task-step-tool-0")).toBeInTheDocument();
    expect(screen.getByText("read")).toBeInTheDocument();
    expect(screen.getByText(/总计: 1/)).toBeInTheDocument();
    expect(screen.getByText(/运行中: 1/)).toBeInTheDocument();
    expect(screen.getByText(/完成: 0/)).toBeInTheDocument();
    expect(screen.getByText(/失败: 0/)).toBeInTheDocument();
  });

  it("shows done/success status after completion", () => {
    render(
      <TaskPane
        startedAt={null}
        steps={[
          { id: "tool-0", name: "read", args: "a.rs", startedAt: 0, status: "done", success: true },
          { id: "tool-1", name: "edit", args: "b.rs", startedAt: 0, status: "done", success: false },
        ]}
      />
    );
    expect(screen.getByText(/总计: 2/)).toBeInTheDocument();
    expect(screen.getByText(/运行中: 0/)).toBeInTheDocument();
    expect(screen.getByText(/完成: 1/)).toBeInTheDocument();
    expect(screen.getByText(/失败: 1/)).toBeInTheDocument();
    expect(screen.queryByTestId("task-pane-empty")).toBeNull();
  });

  it("shows duration for completed steps with doneAt", () => {
    const now = Date.now();
    render(
      <TaskPane
        startedAt={now - 5000}
        steps={[
          { id: "tool-0", name: "read", args: "a.rs", startedAt: now - 3000, doneAt: now - 1000, status: "done", success: true },
        ]}
      />
    );
    expect(screen.getByText(/2\.0s|2s/)).toBeInTheDocument();
  });

  it("expands args on click and shows pretty JSON", () => {
    const args = '{"path": "src/main.rs", "line": 10, "extra": "very long string that exceeds 80 chars to trigger expansion"}';
    render(
      <TaskPane
        startedAt={Date.now()}
        steps={[{ id: "tool-0", name: "read", args, startedAt: Date.now(), status: "running" }]}
      />
    );
    const argsEl = screen.getByText(/path/);
    fireEvent.click(argsEl);
    const pre = screen.getByTestId("task-step-tool-0").querySelector("pre");
    expect(pre).toBeInTheDocument();
    expect(pre).toHaveTextContent(/src\/main\.rs/);
    expect(pre).toHaveTextContent(/line.*10/);
  });

it("copies args to clipboard on copy button click", async () => {
    const args = '{"path": "test.rs", "extra": "very long string that exceeds 80 chars to show copy button"}';
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText } });

    render(
      <TaskPane
        startedAt={Date.now()}
        steps={[{ id: "tool-0", name: "read", args, startedAt: Date.now(), status: "running" }]}
      />
    );

    const copyBtn = screen.getByTitle("复制参数");
    fireEvent.click(copyBtn);

    expect(writeText).toHaveBeenCalledWith(args);
    vi.unstubAllGlobals();
  });

  it("supports keyboard: Enter to expand/collapse", () => {
    const args = '{"path": "src/main.rs", "extra": "very long string that exceeds 80 chars to trigger expansion"}';
    render(
      <TaskPane
        startedAt={Date.now()}
        steps={[{ id: "tool-0", name: "read", args, startedAt: Date.now(), status: "running" }]}
      />
    );

    const item = screen.getByTestId("task-step-tool-0");
    item.focus();
    fireEvent.keyDown(item, { key: "Enter" });

    expect(screen.getByText(/src\/main\.rs/)).toBeInTheDocument();
  });

  it("supports keyboard: C to copy args", () => {
    const args = '{"path": "test.rs", "extra": "very long string that exceeds 80 chars to show copy button"}';
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText } });

    render(
      <TaskPane
        startedAt={Date.now()}
        steps={[{ id: "tool-0", name: "read", args, startedAt: Date.now(), status: "running" }]}
      />
    );

    const item = screen.getByTestId("task-step-tool-0");
    item.focus();
    fireEvent.keyDown(item, { key: "c" });

    expect(writeText).toHaveBeenCalledWith(args);
    vi.unstubAllGlobals();
  });

  it("shows nested steps with indentation", () => {
    render(
      <TaskPane
        startedAt={Date.now()}
        steps={[
          { id: "tool-0", name: "parent", args: "{}", startedAt: Date.now(), status: "running", depth: 0 },
          { id: "tool-1", name: "child", args: "{}", startedAt: Date.now(), status: "running", depth: 1, parentId: "tool-0" },
        ]}
      />
    );
    const child = screen.getByTestId("task-step-tool-1");
    expect(child).toHaveStyle({ marginLeft: "16px" });
  });
});
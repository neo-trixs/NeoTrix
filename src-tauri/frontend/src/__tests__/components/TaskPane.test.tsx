import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { TaskPane } from "../../components/neocodex/TaskPane";

describe("TaskPane", () => {
  it("shows empty state with guidance when no steps exist", () => {
    render(<TaskPane steps={[]} startedAt={null} />);
    expect(screen.getByTestId("task-pane")).toBeInTheDocument();
    expect(screen.getByTestId("task-pane-empty")).toBeInTheDocument();
    expect(screen.getByText(/0 完成 · 0 运行中 · 尚无任务/)).toBeInTheDocument();
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
    expect(screen.getByText("0 完成 · 1 运行中")).toBeInTheDocument();
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
    expect(screen.getByText("2 完成 · 0 运行中")).toBeInTheDocument();
    expect(screen.queryByTestId("task-pane-empty")).toBeNull();
  });
});

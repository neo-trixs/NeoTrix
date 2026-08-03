import React from "react";
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ContextPanel } from "../../components/neocodex/ContextPanel";

vi.mock("../../components/neocodex/index", () => {
  const mk = (name: string) => (props: object) =>
    React.createElement("div", { "data-testid": `mock-${name}`, ...props }, "fake");
  return {
    TaskPane: mk("task"),
    DiffPane: mk("diff"),
    PreviewPane: mk("preview"),
    TerminalPane: mk("terminal"),
    CapabilityHealthPane: mk("health"),
    FileTreePanel: mk("file"),
  };
});

const base = {
  taskSteps: [] as Array<{
    id: string;
    name: string;
    args: string;
    startedAt: number;
    status: "running" | "done";
    success?: boolean;
  }>,
  taskStartedAt: null,
  health: null,
};

describe("ContextPanel route-by-rail-tab", () => {
  it("renders TaskPane for tasks tab", () => {
    render(<ContextPanel activeTab="tasks" {...base} />);
    expect(screen.getByTestId("mock-task")).toBeInTheDocument();
  });

  it("renders DiffPane for review tab", () => {
    render(<ContextPanel activeTab="review" {...base} />);
    expect(screen.getByTestId("mock-diff")).toBeInTheDocument();
  });

  it("renders PreviewPane for browser tab", () => {
    render(<ContextPanel activeTab="browser" {...base} />);
    expect(screen.getByTestId("mock-preview")).toBeInTheDocument();
  });

  it("renders TerminalPane for terminal tab", () => {
    render(<ContextPanel activeTab="terminal" {...base} />);
    expect(screen.getByTestId("mock-terminal")).toBeInTheDocument();
  });

  it("passes onFilePick through to FileTreePanel for file tab", () => {
    const onFilePick = vi.fn();
    const { container } = render(<ContextPanel activeTab="file" {...base} onFilePick={onFilePick} />);
    expect(screen.getByTestId("mock-file")).toBeInTheDocument();
    expect(container.innerHTML).toContain("fake");
    expect(onFilePick).toBeDefined();
  });
});
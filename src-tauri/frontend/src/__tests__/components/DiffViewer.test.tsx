import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import DiffViewer from "../../components/DiffViewer";
import type { DiffBlock } from "../../types";

const diffBlocks: DiffBlock[] = [
  { type: "added", content: 'println!("hello");', lineStart: 10 },
  { type: "removed", content: 'print("hello");', lineStart: 10 },
  { type: "unchanged", content: "// end of file", lineStart: 12 },
];

describe("DiffViewer", () => {
  it("renders diff blocks with content", () => {
    render(<DiffViewer diffBlocks={diffBlocks} />);
    expect(screen.getByText('println!("hello");')).toBeInTheDocument();
    expect(screen.getByText('print("hello");')).toBeInTheDocument();
    expect(screen.getByText("// end of file")).toBeInTheDocument();
  });

  it("shows diff stats", () => {
    render(<DiffViewer diffBlocks={diffBlocks} />);
    expect(screen.getByText("+1")).toBeInTheDocument();
    expect(screen.getByText("-1")).toBeInTheDocument();
    expect(screen.getByText("=1")).toBeInTheDocument();
  });

  it("renders added lines with diff-added class", () => {
    const { container } = render(<DiffViewer diffBlocks={diffBlocks} />);
    const lines = container.querySelectorAll(".diff-line");
    expect(lines[0].className).toContain("diff-added");
    expect(lines[1].className).toContain("diff-removed");
    expect(lines[2].className).toContain("diff-unchanged");
  });

  it("shows filename when provided", () => {
    render(<DiffViewer diffBlocks={diffBlocks} filename="src/main.rs" />);
    expect(screen.getByText("src/main.rs")).toBeInTheDocument();
  });

  it("shows Apply button when onApply is provided", () => {
    render(<DiffViewer diffBlocks={diffBlocks} onApply={vi.fn()} />);
    expect(screen.getByRole("button", { name: "接受" })).toBeInTheDocument();
  });

  it("shows Reject button when onReject is provided", () => {
    render(<DiffViewer diffBlocks={diffBlocks} onReject={vi.fn()} />);
    expect(screen.getByRole("button", { name: "拒绝" })).toBeInTheDocument();
  });

  it("does not show Apply/Reject buttons when handlers are not provided", () => {
    render(<DiffViewer diffBlocks={diffBlocks} />);
    expect(screen.queryByRole("button", { name: "接受" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "拒绝" })).not.toBeInTheDocument();
  });

  it("calls onApply when Apply button is clicked", () => {
    const onApply = vi.fn();
    render(<DiffViewer diffBlocks={diffBlocks} onApply={onApply} />);
    fireEvent.click(screen.getByRole("button", { name: "接受" }));
    expect(onApply).toHaveBeenCalledOnce();
  });

  it("calls onReject when Reject button is clicked", () => {
    const onReject = vi.fn();
    render(<DiffViewer diffBlocks={diffBlocks} onReject={onReject} />);
    fireEvent.click(screen.getByRole("button", { name: "拒绝" }));
    expect(onReject).toHaveBeenCalledOnce();
  });

  it("shows collapsed state summary when collapsed", () => {
    const { container } = render(<DiffViewer diffBlocks={diffBlocks} filename="test.ts" />);
    // Click collapse button
    const collapseBtn = container.querySelector(".btn-icon");
    fireEvent.click(collapseBtn!);
    expect(screen.getByText(/test.ts/)).toBeInTheDocument();
    expect(screen.getByText(/1 处添加 \/ 1 处删除/)).toBeInTheDocument();
    expect(screen.getByText("展开")).toBeInTheDocument();
  });

  it("expands when collapsed view is clicked", () => {
    const { container } = render(<DiffViewer diffBlocks={diffBlocks} />);
    // Collapse first
    const collapseBtn = container.querySelector(".btn-icon");
    fireEvent.click(collapseBtn!);
    // Now click the collapsed view to expand
    fireEvent.click(screen.getByText("展开"));
    // Should show diff content again
    expect(screen.getByText('println!("hello");')).toBeInTheDocument();
  });
});

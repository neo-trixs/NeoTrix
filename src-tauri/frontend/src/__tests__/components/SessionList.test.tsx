import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import SessionList from "../../components/SessionList";
import type { Session } from "../../types";

const sessions: Session[] = [
  { id: "s1", name: "会话一", messages: [{ role: "user", content: "a", contentType: "text" }] },
  { id: "s2", name: "会话二", messages: [] },
  { id: "s3", name: "会话三", messages: [{ role: "user", content: "b", contentType: "text" }, { role: "assistant", content: "c", contentType: "markdown" }] },
];

const defaultProps = {
  sessions,
  activeSession: 0,
  onSelect: vi.fn(),
  onNew: vi.fn(),
};

describe("SessionList", () => {
  it("lists all sessions", () => {
    render(<SessionList {...defaultProps} />);
    expect(screen.getByText("会话一")).toBeInTheDocument();
    expect(screen.getByText("会话二")).toBeInTheDocument();
    expect(screen.getByText("会话三")).toBeInTheDocument();
  });

  it("shows message counts", () => {
    render(<SessionList {...defaultProps} />);
    expect(screen.getByText("1 条消息")).toBeInTheDocument();
    expect(screen.getByText("0 条消息")).toBeInTheDocument();
    expect(screen.getByText("2 条消息")).toBeInTheDocument();
  });

  it("highlights active session", () => {
    const { container } = render(<SessionList {...defaultProps} activeSession={1} />);
    const items = container.querySelectorAll(".session-item");
    expect(items[0].className).not.toContain("active");
    expect(items[1].className).toContain("active");
    expect(items[2].className).not.toContain("active");
  });

  it("calls onSelect when a session is clicked", () => {
    const onSelect = vi.fn();
    render(<SessionList {...defaultProps} onSelect={onSelect} />);
    fireEvent.click(screen.getByText("会话二"));
    expect(onSelect).toHaveBeenCalledWith(1);
  });

  it("calls onNew when new session button is clicked", () => {
    const onNew = vi.fn();
    render(<SessionList {...defaultProps} onNew={onNew} />);
    fireEvent.click(screen.getByTitle("新建会话 (Ctrl+N)"));
    expect(onNew).toHaveBeenCalledOnce();
  });

  it("renders import session button", () => {
    render(<SessionList {...defaultProps} />);
    expect(screen.getByTitle("导入会话")).toBeInTheDocument();
  });

  it("renders fork and export buttons for each session", () => {
    render(<SessionList {...defaultProps} />);
    const forkButtons = screen.getAllByTitle("复制会话");
    const exportButtons = screen.getAllByTitle("导出会话");
    expect(forkButtons).toHaveLength(3);
    expect(exportButtons).toHaveLength(3);
  });

  it("session click on fork button does not trigger onSelect", () => {
    const onSelect = vi.fn();
    render(<SessionList {...defaultProps} onSelect={onSelect} />);
    const forkButtons = screen.getAllByTitle("复制会话");
    fireEvent.click(forkButtons[0]);
    expect(onSelect).not.toHaveBeenCalled();
  });
});

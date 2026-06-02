import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import StatusBar from "../../components/StatusBar";

const defaultProps = {
  text: "就绪",
  agentBusy: false,
  sessionIndex: 1,
  sessionCount: 3,
  onOpenSettings: vi.fn(),
  onSelectProject: vi.fn(),
  onToggleTheme: vi.fn(),
  theme: "light",
};

describe("StatusBar", () => {
  it("renders status text", () => {
    render(<StatusBar {...defaultProps} />);
    expect(screen.getByText("就绪")).toBeInTheDocument();
  });

  it("shows session index and count", () => {
    render(<StatusBar {...defaultProps} sessionIndex={2} sessionCount={5} />);
    expect(screen.getByText("会话 2/5")).toBeInTheDocument();
  });

  it("shows busy indicator when agentBusy is true", () => {
    render(<StatusBar {...defaultProps} agentBusy={true} text="思考中..." />);
    expect(screen.getByText("思考中...")).toBeInTheDocument();
    expect(screen.getByText("⏳")).toBeInTheDocument();
    expect(screen.getByText("会话 1/3")).toBeInTheDocument();
  });

  it("does not show spinner when idle", () => {
    render(<StatusBar {...defaultProps} />);
    expect(screen.queryByText("⏳")).not.toBeInTheDocument();
  });

  it("has a busy class on the container when agent is busy", () => {
    const { container } = render(<StatusBar {...defaultProps} agentBusy={true} />);
    expect(container.querySelector(".status-bar")?.className).toContain("busy");
  });

  it("calls onOpenSettings when settings button clicked", () => {
    const onOpenSettings = vi.fn();
    render(<StatusBar {...defaultProps} onOpenSettings={onOpenSettings} />);
    const buttons = screen.getAllByRole("button");
    const settingsBtn = buttons[buttons.length - 1];
    fireEvent.click(settingsBtn);
    expect(onOpenSettings).toHaveBeenCalledOnce();
  });

  it("calls onSelectProject when project button clicked", () => {
    const onSelectProject = vi.fn();
    render(<StatusBar {...defaultProps} onSelectProject={onSelectProject} />);
    const buttons = screen.getAllByRole("button");
    const projectBtn = buttons[buttons.length - 2];
    fireEvent.click(projectBtn);
    expect(onSelectProject).toHaveBeenCalledOnce();
  });

  it("calls onToggleTerminal when terminal button clicked", () => {
    const onToggleTerminal = vi.fn();
    render(<StatusBar {...defaultProps} onToggleTerminal={onToggleTerminal} />);
    const buttons = screen.getAllByRole("button");
    const terminalBtn = buttons[1];
    fireEvent.click(terminalBtn);
    expect(onToggleTerminal).toHaveBeenCalledOnce();
  });

  it("calls onToggleTheme when theme button clicked", () => {
    const onToggleTheme = vi.fn();
    render(<StatusBar {...defaultProps} onToggleTheme={onToggleTheme} />);
    const buttons = screen.getAllByRole("button");
    const themeBtn = buttons[0];
    fireEvent.click(themeBtn);
    expect(onToggleTheme).toHaveBeenCalledOnce();
  });

  it("shows terminal status when provided", () => {
    render(<StatusBar {...defaultProps} terminalStatus="运行中" />);
    expect(screen.getByText("运行中")).toBeInTheDocument();
  });

  it("shows active class on terminal button when terminal is visible", () => {
    const { container } = render(<StatusBar {...defaultProps} showTerminal={true} />);
    const buttons = container.querySelectorAll(".status-btn");
    expect(buttons[1].className).toContain("active");
  });
});

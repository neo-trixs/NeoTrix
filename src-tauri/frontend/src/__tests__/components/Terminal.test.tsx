import { describe, it, expect, vi, beforeEach } from "vitest";

const mockInvoke = vi.hoisted(() => vi.fn());
const mockListen = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mockListen }));

const mockTerminal = vi.hoisted(() => ({
  loadAddon: vi.fn(), open: vi.fn(), write: vi.fn(), onData: vi.fn(), onResize: vi.fn(), dispose: vi.fn(),
}));

const mockFitAddon = vi.hoisted(() => ({
  fit: vi.fn(), proposeDimensions: vi.fn().mockReturnValue({ cols: 80, rows: 24 }),
}));

vi.mock("../../lib/xterm-loader", () => ({
  getXterm: () => Promise.resolve({ Terminal: vi.fn().mockImplementation(() => mockTerminal) }),
  getXtermAddonFit: () => Promise.resolve({ FitAddon: vi.fn().mockImplementation(() => mockFitAddon) }),
}));

import { render, screen, fireEvent } from "@testing-library/react";
import Terminal from "../../components/Terminal";

const defaultProps = { sessionId: "term-1", onClose: vi.fn(), onStatusChange: vi.fn() };

describe("Terminal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue("pty-abc");
    mockListen.mockResolvedValue(vi.fn());
  });

  it("renders terminal panel with header", () => {
    render(<Terminal {...defaultProps} />);
    expect(screen.getByText("终端")).toBeInTheDocument();
  });

  it("renders close button", () => {
    render(<Terminal {...defaultProps} />);
    expect(screen.getByTitle("关闭终端")).toBeInTheDocument();
  });

  it("shows loading state initially", () => {
    render(<Terminal {...defaultProps} />);
    expect(screen.getByText("加载终端...")).toBeInTheDocument();
  });

  it("calls onClose when close button clicked", () => {
    const onClose = vi.fn();
    render(<Terminal {...defaultProps} onClose={onClose} />);
    screen.getByTitle("关闭终端").click();
    expect(onClose).toHaveBeenCalledOnce();
  });
});

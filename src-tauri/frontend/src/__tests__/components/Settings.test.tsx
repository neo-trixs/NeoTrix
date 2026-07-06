import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import Settings from "../../components/Settings";
import type { AppSettings, ProviderConfig, KnowledgeEntry } from "../../types";
import { DEFAULT_SETTINGS } from "../../stores/store-utils";

const defaultSettings: AppSettings = { ...DEFAULT_SETTINGS };

const defaultProvider: ProviderConfig = {
  id: "anthropic",
  name: "Anthropic Claude",
  model: "claude-sonnet-4-20250514",
  apiKey: "",
  learningRate: 0.05,
};

const defaultProps = {
  settings: defaultSettings,
  providerConfig: defaultProvider,
  knowledgeBase: [] as KnowledgeEntry[],
  onSaveSettings: vi.fn(),
  onSaveProvider: vi.fn(),
  onAddKnowledge: vi.fn(),
  onDeleteKnowledge: vi.fn(),
  onSearchKnowledge: vi.fn(),
  onClose: vi.fn(),
};

describe("Settings", () => {
  it("renders dialog with title", () => {
    render(<Settings {...defaultProps} />);
    expect(screen.getByText("设置")).toBeInTheDocument();
  });

  it("renders general tab by default", () => {
    render(<Settings {...defaultProps} />);
    expect(screen.getByRole("tab", { name: "通用" })).toHaveAttribute("aria-selected", "true");
  });

  it("renders all seven tabs", () => {
    render(<Settings {...defaultProps} />);
    expect(screen.getByRole("tab", { name: "Provider" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "通用" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "API" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "知识库" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "隐私" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "快捷键" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "关于" })).toBeInTheDocument();
  });

  it("switches to provider settings tab when clicked", () => {
    render(<Settings {...defaultProps} />);
    expect(screen.getByText("主题")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("tab", { name: "Provider" }));
    expect(screen.getByRole("tab", { name: "Provider" })).toHaveAttribute("aria-selected", "true");
  });

  it("switches to knowledge tab when clicked", () => {
    render(<Settings {...defaultProps} />);
    fireEvent.click(screen.getByRole("tab", { name: "知识库" }));
    expect(screen.getByRole("tab", { name: "知识库" })).toHaveAttribute("aria-selected", "true");
  });

  it("calls onClose when close button is clicked", () => {
    const onClose = vi.fn();
    render(<Settings {...defaultProps} onClose={onClose} />);
    fireEvent.click(screen.getByLabelText("Close settings"));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("calls onClose when overlay is clicked", () => {
    const onClose = vi.fn();
    const { container } = render(<Settings {...defaultProps} onClose={onClose} />);
    const overlay = container.firstChild as HTMLElement;
    fireEvent.click(overlay);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("does not call onClose when panel is clicked", () => {
    const onClose = vi.fn();
    render(<Settings {...defaultProps} onClose={onClose} />);
    const dialog = screen.getByRole("dialog");
    fireEvent.click(dialog);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("shows save button in general tab", () => {
    render(<Settings {...defaultProps} />);
    fireEvent.click(screen.getByRole("tab", { name: "通用" }));
    expect(screen.getByText("保存")).toBeInTheDocument();
  });

  it("shows close button", () => {
    render(<Settings {...defaultProps} />);
    expect(screen.getByText("关闭")).toBeInTheDocument();
  });

  it("calls onSaveSettings when save is clicked in general tab", () => {
    const onSaveSettings = vi.fn();
    const onClose = vi.fn();
    render(<Settings {...defaultProps} onSaveSettings={onSaveSettings} onClose={onClose} />);
    fireEvent.click(screen.getByRole("tab", { name: "通用" }));
    fireEvent.click(screen.getByText("保存"));
    expect(onSaveSettings).toHaveBeenCalledWith(defaultSettings);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("updates theme selection in general tab", () => {
    render(<Settings {...defaultProps} />);
    fireEvent.click(screen.getByRole("tab", { name: "通用" }));
    const themeSelect = screen.getByDisplayValue("浅色");
    fireEvent.change(themeSelect, { target: { value: "dark" } });
    expect(screen.getByDisplayValue("深色")).toBeInTheDocument();
  });

  it("updates fontSize in general tab", () => {
    render(<Settings {...defaultProps} />);
    fireEvent.click(screen.getByRole("tab", { name: "通用" }));
    const slider = screen.getByDisplayValue("13");
    fireEvent.change(slider, { target: { value: "16" } });
    expect(screen.getByText("字体大小 (16px)")).toBeInTheDocument();
  });
});
